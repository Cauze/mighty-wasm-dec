#![allow(dead_code)]
use mighty_wasm_dec::{emit, lift, parse, passes, types};

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mighty-wasm-dec", version, about = "full-Rust wasm->C decompiler (MVP)")]
struct Args {
    input: PathBuf,
    /// only emit this func index (filters output, not just a comment)
    #[arg(long)]
    func: Option<u32>,
    /// only emit func with this display name
    #[arg(long)]
    func_name: Option<String>,
    /// list functions and exit
    #[arg(long)]
    list: bool,
    #[arg(long)]
    json: bool,
    /// write C output to file instead of stdout
    #[arg(long)]
    out: Option<PathBuf>,
    /// hide runtime boilerplate (keep user funcs + __original_main)
    #[arg(long)]
    user_only: bool,
    /// skip cleanup passes (debug raw lifter output)
    #[arg(long)]
    no_opt: bool,
    /// disable const/branch folding
    #[arg(long)]
    no_fold: bool,
    /// disable tmp/copyprop inlining
    #[arg(long)]
    no_inline: bool,
    /// disable dead-code elimination
    #[arg(long)]
    no_dce: bool,
    /// disable block/label simplification
    #[arg(long)]
    no_simplify: bool,
    /// disable struct recovery + field/signature rendering
    #[arg(long)]
    no_struct: bool,
    /// disable per-access string/data notes
    #[arg(long)]
    no_strings: bool,
    /// string display: off | comment | defs (static const table + s_<addr> refs)
    #[arg(long, default_value = "comment")]
    strings: String,
    /// cap visual nesting depth (levels of 2 spaces); unlimited by default
    #[arg(long)]
    max_indent: Option<usize>,
    /// emission order: index | calls (BFS from exports, runtime sinks last)
    #[arg(long, default_value = "index")]
    order: String,
}

/// Emscripten/WASI runtime functions hidden by --user-only.
/// _start is wiring (ctors + main + exit); see it with --func-name _start.
const RUNTIME_FUNCS: &[&str] = &[
    "__wasm_call_ctors",
    "_start",
    "dummy",
    "libc_exit_fini",
    "exit",
    "_Exit",
    "_emscripten_stack_restore",
    "emscripten_stack_get_current",
];

fn load_bytes(path: &std::path::Path) -> Result<Vec<u8>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "wat" {
        let text = std::fs::read_to_string(path).context("read wat")?;
        let bytes = wat::parse_str(&text).context("wat->wasm")?;
        Ok(bytes)
    } else {
        Ok(std::fs::read(path).context("read wasm")?)
    }
}

fn main() -> Result<()> {
    // Real-world modules nest blocks/expressions deep enough to overflow the
    // default 8MB main-thread stack (recursive lifter/emitter). Run everything
    // on a big-stack thread instead.
    let args = Args::parse();
    std::thread::Builder::new()
        .name("decompile".into())
        .stack_size(512 * 1024 * 1024)
        .spawn(move || run(args))
        .context("spawn worker")?
        .join()
        .unwrap_or_else(|_| Err(anyhow::anyhow!("worker panicked")))
}

fn run(args: Args) -> Result<()> {
    let bytes = load_bytes(&args.input)?;
    let mut meta = parse::parse_meta(&bytes)?;

    if args.list {
        // No lifting needed: signatures + names come straight from meta,
        // so this stays instant even on multi-MB modules.
        let ndef = meta.func_types.len() as u32;
        for idx in 0..meta.import_func_count + ndef {
            let (p, r) = meta.func_sig(idx);
            let name = meta.call_name(idx);
            let tag = if idx < meta.import_func_count {
                " [import]"
            } else {
                ""
            };
            println!(
                "{idx} {name}{tag}({}) -> [{}]",
                p.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","),
                r.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","),
            );
        }
        return Ok(());
    }

    // Selective lifting: --func/--func-name resolve to indices first so
    // huge modules don't pay for functions nobody asked about.
    let selective = args.func.is_some() || args.func_name.is_some();
    let mut mir = if selective {
        let bodies = lift::collect_bodies(&bytes)?;
        let mut wanted: Vec<u32> = Vec::new();
        if let Some(n) = args.func {
            wanted.push(n);
        }
        if let Some(nm) = &args.func_name {
            let ndef = meta.func_types.len() as u32;
            for idx in 0..meta.import_func_count + ndef {
                if meta.call_name(idx) == *nm {
                    wanted.push(idx);
                }
            }
        }
        wanted.sort_unstable();
        wanted.dedup();
        let mut mir = lift::lift_selected(&bodies, &mut meta, &wanted)?;
        // single-function view: trim the world to what the query touches,
        // otherwise 62k-segment dumps and 241-global preambles drown a
        // ~70-line function (sqlite3DecOrHexToI64 was 94KB, 82 of it data).
        emit::trim_for_scoped_view(&mut mir);
        mir
    } else {
        lift::lift_module(&bytes, &mut meta)?
    };

    let opt = if args.no_opt {
        passes::OptConfig::all_off()
    } else {
        passes::OptConfig {
            fold: !args.no_fold,
            inline: !args.no_inline,
            dce: !args.no_dce,
            simplify: !args.no_simplify,
        }
    };
    for f in mir.funcs.iter_mut() {
        passes::optimize_func_with(f, &opt);
    }
    // Inter-proc const folding (Phase C): fold direct calls to lone
    // `return const` bodies. Gated with folding like its intra-proc kin.
    if opt.fold {
        use mighty_wasm_dec::passes::ModulePass;
        passes::ConstRetPass.run(&mut mir);
    }

    let strings_mode = if args.no_strings {
        emit::StringsMode::Off
    } else {
        match args.strings.as_str() {
            "off" => emit::StringsMode::Off,
            "defs" => emit::StringsMode::Defs,
            _ => emit::StringsMode::Comment,
        }
    };
    let ecfg = emit::EmitConfig {
        struct_on: !args.no_struct,
        strings: strings_mode,
        max_indent: args.max_indent,
    };

    if args.list {
        // unreachable: handled before lifting above
        return Ok(());
    }

    if args.user_only {
        mir.funcs.retain(|f| {
            !RUNTIME_FUNCS.contains(&f.name.as_str()) && !f.body.is_empty()
        });
    }
    if args.order == "calls" {
        // BFS from exports + _start over direct calls; runtime sinks last.
        let mut rank: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
        let mut roots: Vec<u32> = meta.exports.keys().cloned().collect();
        for f in &mir.funcs {
            if f.name == "_start" || f.name == "__original_main" {
                roots.push(f.idx);
            }
        }
        roots.sort_unstable();
        roots.dedup();
        for r in roots {
            if !rank.contains_key(&r) {
                rank.insert(r, rank.len());
                queue.push_back(r);
            }
        }
        // adjacency by func idx (neighbor lists pre-sorted once; the old
        // code cloned + re-sorted per BFS visit)
        let adj: std::collections::HashMap<u32, Vec<u32>> = mir
            .funcs
            .iter()
            .map(|f| {
                let mut cs = f.calls.clone();
                cs.extend(f.indirects.iter().flatten());
                cs.sort_unstable();
                cs.dedup();
                (f.idx, cs)
            })
            .collect();
        while let Some(n) = queue.pop_front() {
            if let Some(cs) = adj.get(&n) {
                for c in cs {
                    if !rank.contains_key(c) {
                        rank.insert(*c, rank.len());
                        queue.push_back(*c);
                    }
                }
            }
        }
        mir.funcs.sort_by_key(|f| {
            let is_rt = RUNTIME_FUNCS.contains(&f.name.as_str());
            (is_rt, rank.get(&f.idx).cloned().unwrap_or(usize::MAX), f.idx)
        });
    }

    if args.json {
        let funcs: Vec<_> = mir
            .funcs
            .iter()
            .map(|f| {
                serde_json::json!({
                    "idx": f.idx,
                    "name": f.name,
                    "params": f.params.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                    "results": f.results.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
                    "calls": f.calls,
                    "indirects": f.indirects,
                    "accesses": f.accesses.iter().map(|a| serde_json::json!({
                        "base": a.base_desc, "off": a.offset, "w": a.width,
                        "write": a.is_write, "dom": a.dom_ty
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let doc = serde_json::to_string_pretty(&serde_json::json!({
            "file": args.input.to_string_lossy(),
            "funcs": funcs,
            "tables": mir.tables,
            "imports": mir.imports.iter().map(|i| serde_json::json!({
                "module": i.module, "name": i.name, "kind": i.kind
            })).collect::<Vec<_>>(),
            "globals": mir.globals.iter().map(|g| serde_json::json!({
                "idx": g.idx, "ty": g.ty.to_string(), "mut": g.mutable
            })).collect::<Vec<_>>(),
            "data": mir.data.iter().map(|d| serde_json::json!({
                "idx": d.idx, "off": d.offset, "len": d.bytes.len()
            })).collect::<Vec<_>>(),
            "merge_notes": types::merge_notes(&mir.funcs),
        }))?;
        if let Some(p) = &args.out {
            std::fs::write(p, doc).context("write json")?;
        } else {
            println!("{doc}");
        }
        return Ok(());
    }

    let c = emit::emit_c_with(&mir, &ecfg);
    if let Some(p) = &args.out {
        std::fs::write(p, c).context("write c")?;
    } else {
        println!("{c}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mighty_wasm_dec::cfg;

    fn lift_wat(wat: &str) -> mighty_wasm_dec::ir::FuncIR {
        let bytes = wat::parse_str(wat).unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mir = lift::lift_module(&bytes, &mut meta).unwrap();
        mir.funcs.into_iter().last().unwrap()
    }

    fn decompile_wat(wat: &str) -> String {
        let bytes = wat::parse_str(wat).unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mir = lift::lift_module(&bytes, &mut meta).unwrap();
        emit::emit_c(&mir)
    }

    fn decompile_opt_wat(wat: &str) -> String {
        let bytes = wat::parse_str(wat).unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mut mir = lift::lift_module(&bytes, &mut meta).unwrap();
        for f in mir.funcs.iter_mut() {
            passes::optimize_func(f);
        }
        emit::emit_c(&mir)
    }

    fn decompile_cfg_wat(
        wat: &str,
        opt: &passes::OptConfig,
        ecfg: &emit::EmitConfig,
    ) -> String {
        let bytes = wat::parse_str(wat).unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mut mir = lift::lift_module(&bytes, &mut meta).unwrap();
        for f in mir.funcs.iter_mut() {
            passes::optimize_func_with(f, opt);
        }
        emit::emit_c_with(&mir, ecfg)
    }

    #[test]
    fn fac_has_loop() {
        let c = decompile_wat(r#"(module (func $fac (param i32) (result i32)
          (local i32) i32.const 1 local.set 1
          (block (loop (br_if 1 (i32.eqz (local.get 0)))
            (local.set 1 (i32.mul (local.get 1) (local.get 0)))
            (local.set 0 (i32.sub (local.get 0) (i32.const 1))) (br 0)))
          local.get 1))"#);
        assert!(c.contains("while (1)"), "expected loop lowering, got:\n{c}");
        assert!(c.contains("return l1"), "expected return, got:\n{c}");
    }

    #[test]
    fn struct_recovery() {
        let c = decompile_wat(r#"(module (memory 1)
          (func (param i32) (result i32)
            (i32.store offset=4 (local.get 0) (i32.const 41))
            (i32.store (local.get 0) (i32.const 1))
            (i32.load offset=4 (local.get 0))))"#);
        assert!(c.contains("typedef struct"), "expected struct, got:\n{c}");
        assert!(c.contains("f_off_4"), "expected field, got:\n{c}");
    }

    #[test]
    fn br_labels_resolve() {
        let c = decompile_wat(r#"(module (func (param i32) (result i32)
          (block (br 0)) (i32.const 7)))"#);
        assert!(!c.contains("br_depth"), "depth gotos must be resolved, got:\n{c}");
        assert!(c.contains("__end_"), "expected end label, got:\n{c}");
    }

    #[test]
    fn mem_access_uses_mem_base() {
        let c = decompile_wat(r#"(module (memory 1) (func (param i32) (result i32)
          (i32.load (local.get 0))))"#);
        assert!(c.contains("mem +"), "expected mem base, got:\n{c}");
    }

    #[test]
    fn fold_removes_const_branch() {
        let c = decompile_opt_wat(r#"(module (func (result i32)
          (if (result i32) (i32.const 0) (then (i32.const 1)) (else (i32.const 2)))))"#);
        assert!(!c.contains("if ("), "expected folded branch, got:\n{c}");
        assert!(c.contains("return 2"), "expected else value, got:\n{c}");
    }

    #[test]
    fn inline_removes_single_use_tmp() {
        // t0 = f(6); return t0  ->  return f(6)
        let c = decompile_opt_wat(r#"(module
          (func $id (param i32) (result i32) (local.get 0))
          (func (result i32) (call $id (i32.const 6))))"#);
        assert!(c.contains("return id(6)"), "expected inlined return, got:\n{c}");
        assert!(!c.contains("t0 ="), "expected no tmp, got:\n{c}");
    }

    #[test]
    fn bulk_memory_renders_pseudo_calls() {
        let c = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32 i32 i32)
            (memory.fill (local.get 0) (local.get 1) (local.get 2))
            (memory.copy (local.get 0) (local.get 1) (local.get 2))))"#);
        assert!(c.contains("memory.fill"), "expected fill, got:\n{c}");
        assert!(c.contains("memory.copy"), "expected copy, got:\n{c}");
        assert!(!c.contains("unhandled"), "expected no fallback, got:\n{c}");
    }

    #[test]
    fn ref_and_table_ops_named() {
        let c = decompile_opt_wat(r#"(module
          (table 1 funcref)
          (func (param i32) (result i32)
            (table.grow (ref.null func) (local.get 0))))"#);
        assert!(c.contains("table.grow"), "expected grow, got:\n{c}");
        assert!(c.contains("ref.null"), "expected null, got:\n{c}");
    }

    #[test]
    fn simd_arith_named_not_unknown() {
        let c = decompile_opt_wat(r#"(module
          (func (param v128 v128) (result v128)
            (i8x16.add (local.get 0) (local.get 1))))"#);
        assert!(c.contains("i8x16.add"), "expected simd name, got:\n{c}");
        assert!(!c.contains("unhandled"), "expected no fallback, got:\n{c}");
    }

    #[test]
    fn tail_call_returns() {
        let c = decompile_opt_wat(r#"(module
          (func $t (param i32) (result i32) (local.get 0))
          (func (param i32) (result i32) (return_call $t (local.get 0))))"#);
        assert!(c.contains("return"), "expected return, got:\n{c}");
        assert!(c.contains("t("), "expected call, got:\n{c}");
    }

    #[test]
    fn string_comment_on_const_addr() {
        let c = decompile_opt_wat(r#"(module (memory 1)
          (data (i32.const 16) "hi-ok")
          (func (result i32) (i32.load8_u (i32.const 16))))"#);
        assert!(c.contains("hi-ok"), "expected string note, got:\n{c}");
    }

    #[test]
    fn string_defs_emit_table() {
        let cfg = emit::EmitConfig {
            struct_on: true,
            strings: emit::StringsMode::Defs,
            max_indent: None,
        };
        let c = decompile_cfg_wat(
            r#"(module (memory 1)
              (data (i32.const 32) "auth-ok")
              (func (param i32) (result i32)
                (i32.load8_u (i32.add (local.get 0) (i32.const 32))))) "#,
            &passes::OptConfig::default(),
            &cfg,
        );
        assert!(c.contains("static const char s_32"), "expected table, got:\n{c}");
        assert!(c.contains("s_32[l0]"), "expected indexed ref, got:\n{c}");
    }

    #[test]
    fn copyprop_param_copy() {
        // l1 = l0 (single, outside loop), one use -> substituted
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32)
            (local i32)
            (local.set 1 (local.get 0))
            (i32.add (local.get 1) (i32.const 1))))"#);
        assert!(!c.contains("l1 = l0"), "expected copy gone, got:\n{c}");
        assert!(c.contains("return (l0 + 1)"), "expected prop, got:\n{c}");
    }

    #[test]
    fn toggle_no_struct_hides_typedefs() {
        let wat = r#"(module (memory 1)
          (func (param i32) (result i32)
            (i32.store offset=4 (local.get 0) (i32.const 41))
            (i32.store (local.get 0) (i32.const 1))
            (i32.load offset=4 (local.get 0))))"#;
        let cfg = emit::EmitConfig {
            struct_on: false,
            strings: emit::StringsMode::Comment,
            max_indent: None,
        };
        let c = decompile_cfg_wat(wat, &passes::OptConfig::default(), &cfg);
        assert!(!c.contains("typedef struct"), "expected no structs, got:\n{c}");
        assert!(c.contains("f_off") == false || c.contains("mem +"), "sanity:\n{c}");
    }

    #[test]
    fn toggle_no_fold_keeps_const_branch() {
        let wat = r#"(module (func (result i32)
          (if (result i32) (i32.const 0) (then (i32.const 1)) (else (i32.const 2)))))"#;
        let off = passes::OptConfig::all_off();
        let cfg = emit::EmitConfig::default();
        let c = decompile_cfg_wat(wat, &off, &cfg);
        assert!(c.contains("if ("), "expected raw branch, got:\n{c}");
    }

    #[test]
    fn narrow_i64_load_renders_int32() {
        // i64.load32_u is a 4-byte zero-extending load, not an 8-byte one
        // (found by differential check against wasm-objdump on f2422).
        let c = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32) (result i64)
            (i64.load32_u (local.get 0))))"#);
        assert!(
            c.contains("*(uint32_t*)"),
            "expected 4-byte unsigned load, got:\n{c}"
        );
        assert!(
            !c.contains("*(int64_t*)"),
            "must not widen to 8 bytes, got:\n{c}"
        );
        // signed variant sign-extends instead
        let cs = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32) (result i64)
            (i64.load32_s (local.get 0))))"#);
        assert!(
            cs.contains("*(int32_t*)"),
            "expected 4-byte signed load, got:\n{cs}"
        );
    }

    #[test]
    fn init_split_produces_two_stores() {
        // i32 evidence at 8 and 12 + packed i64 init at 8 -> split into halves
        let c = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32)
            (i32.store offset=8 (local.get 0) (i32.const 9))
            (i32.store offset=12 (local.get 0) (i32.const 10))
            (i64.store offset=8 (local.get 0) (i64.const 3))))"#);
        assert!(c.contains("split i64 init"), "expected split comment, got:\n{c}");
        assert!(c.contains("f_off_8") || c.contains("+ 8) = 3"), "expected split stores, got:\n{c}");
    }

    #[test]
    fn flatten_passthrough_chain() {
        // block{block{block{x}}} with unreferenced labels -> single block
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32)
            (block (block (block (i32.add (local.get 0) (i32.const 1)))))))"#);
        assert!(!c.contains("B2") && !c.contains("B3"), "expected flattened, got:\n{c}");
        assert!(c.contains("return (l0 + 1)"), "expected value, got:\n{c}");
    }

    #[test]
    fn loop_continue_preserved() {
        // Loop ending in unconditional continue must keep looping — now
        // as `continue` (single-level), not `goto`.
        // (An earlier revision dropped the back-edge and the emitter's
        // fallthrough `break` turned it into loop-exit: wrong-code on
        // sqlite3_step/VdbeExec. The `br_if` above is the real exit edge.)
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32) (local i32)
            (local.set 1 (i32.const 0))
            (block (loop
              (br_if 1 (i32.ge_s (local.get 1) (local.get 0)))
              (local.set 1 (i32.add (local.get 1) (i32.const 1)))
              (br 0)))
            (local.get 1)))"#);
        assert!(c.contains("continue;"), "expected loop-back continue, got:\n{c}");
    }

    #[test]
    fn single_level_break_not_goto() {
        // `br` to the immediately enclosing block renders `break`.
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32)
            (block $b (br_if $b (i32.eqz (local.get 0))) (i32.const 7))))"#);
        assert!(c.contains("break;"), "expected break, got:\n{c}");
        assert!(!c.contains("goto __end_"), "expected no goto, got:\n{c}");
    }

    #[test]
    fn multi_level_branch_stays_goto() {
        // `br` two levels out cannot be `break` (C has no labeled break).
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32)
            (block $outer
              (block $inner
                (br $outer))
              (i32.const 1))))"#);
        assert!(c.contains("goto __end_"), "expected goto, got:\n{c}");
    }

    #[test]
    fn loop_fallthrough_exits() {
        // Genuine fallthrough (no trailing branch): the C `while (1)`
        // needs the appended break or fac/control-style loops hang.
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32) (local i32)
            (local.set 1 (i32.const 0))
            (block $exit (loop $l
              (br_if $exit (i32.ge_s (local.get 1) (local.get 0)))
              (local.set 1 (i32.add (local.get 1) (i32.const 1)))))
            (local.get 1)))"#);
        assert!(c.contains("break;"), "expected loop-exit break, got:\n{c}");
    }

    #[test]
    fn indirect_operand_order() {
        // (call_indirect (type $t) (local.get 1) (local.get 0)):
        // stack is [arg=l1, index=l0]; index must stay the table key.
        let c = decompile_opt_wat(r#"(module
          (type $t (func (param i32) (result i32)))
          (table 2 funcref)
          (elem (i32.const 0) $a $b)
          (func $a (param i32) (result i32) (i32.add (local.get 0) (i32.const 1)))
          (func $b (param i32) (result i32) (i32.mul (local.get 0) (i32.const 2)))
          (func (export "run") (param i32 i32) (result i32)
            (call_indirect (type $t) (local.get 1) (local.get 0))))"#);
        assert!(c.contains("table_call(l0)(l1)"), "expected index l0, got:\n{c}");
    }

    #[test]
    fn unsigned_cmp_and_shift() {
        // *_u ops need unsigned C semantics (plain > and >> are signed).
        let c = decompile_opt_wat(r#"(module
          (func (param i32 i32) (result i32)
            (i32.gt_u (local.get 0) (local.get 1))))"#);
        assert!(c.contains("(uint32_t)(l0) > (uint32_t)(l1)"), "expected unsigned compare, got:\n{c}");
        let c2 = decompile_opt_wat(r#"(module
          (func (param i32 i32) (result i32)
            (i32.shr_u (local.get 0) (local.get 1))))"#);
        assert!(c2.contains("(uint32_t)(l0) >>"), "expected logical shift, got:\n{c2}");
        let c3 = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32) (result i32) (i32.load8_u (local.get 0))))"#);
        assert!(c3.contains("*(uint8_t*)"), "expected zero-extending load, got:\n{c3}");
        let c4 = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32) (result i32) (i32.load8_s (local.get 0))))"#);
        assert!(c4.contains("*(int8_t*)"), "expected sign-extending load, got:\n{c4}");
    }

    #[test]
    fn br_carries_block_value() {
        // (block (result i32) (br 0 (5)) (6)) must yield 5, not 6.
        let c = decompile_opt_wat(r#"(module
          (func (result i32)
            (block (result i32) (br 0 (i32.const 5)) (i32.const 6))))"#);
        assert!(c.contains("t0 = 5") && c.contains("return t0"), "expected phi merge, got:\n{c}");
        assert!(!c.contains("return 6"), "must not take fallthrough, got:\n{c}");
    }

    #[test]
    fn array_new_len_init_order() {
        // Stack is [init, len] with len on top; rendering is (len, init).
        let c = decompile_opt_wat(r#"(module
          (type $a (array i32))
          (func (param i32 i32) (result anyref)
            (array.new $a (local.get 0) (local.get 1))))"#);
        assert!(c.contains("array.new(l1, l0,"), "expected (len, init) order, got:\n{c}");
    }

    #[test]
    fn nonfinite_float_consts() {
        // inf/NaN must not render as integer-to-float casts (miscompile).
        let c = decompile_opt_wat(r#"(module
          (func (result f32) (f32.const inf)))"#);
        assert!(c.contains("INFINITY") && !c.contains("0x7f800000"), "expected INFINITY, got:\n{c}");
        let c2 = decompile_opt_wat(r#"(module
          (func (result f64) (f64.const -nan:0x12345)))"#);
        assert!(c2.contains("u64_to_double"), "expected bit-preserving NaN, got:\n{c2}");
    }

    #[test]
    fn stale_copy_not_propagated() {
        // l1=l0, then l0 clobbered, then use l1: must keep the copy.
        let c = decompile_opt_wat(r#"(module
          (func (param i32) (result i32) (local i32)
            (local.set 1 (local.get 0))
            (local.set 0 (i32.const 99))
            (local.get 1)))"#);
        assert!(c.contains("l1 = l0") || c.contains("return l1"), "expected copy kept, got:\n{c}");
        assert!(!c.contains("return 99"), "must not use clobbered value, got:\n{c}");
    }

    #[test]
    fn max_indent_caps_nesting() {
        // Referenced labels survive all passes, so the `goto` sits 4 deep
        // (8 spaces) unless capped.
        let wat = r#"(module
          (func (param i32) (result i32)
            (block $exit
              (loop $l
                (if (i32.eqz (local.get 0)) (then (br $exit)))
                (br $l)))
            (i32.const 7)))"#;
        let off = passes::OptConfig::all_off();
        let plain = emit::EmitConfig {
            struct_on: true,
            strings: emit::StringsMode::Comment,
            max_indent: None,
        };
        let uncapped = decompile_cfg_wat(wat, &off, &plain);
        assert!(uncapped.contains("\n        goto"), "expected 8-space indent, got:\n{uncapped}");
        let cfg = emit::EmitConfig {
            struct_on: true,
            strings: emit::StringsMode::Comment,
            max_indent: Some(1),
        };
        let capped = decompile_cfg_wat(wat, &off, &cfg);
        assert!(!capped.contains("\n        goto"), "expected capped indent, got:\n{capped}");
        assert!(capped.contains("goto __end_"), "expected goto kept, got:\n{capped}");
    }

    #[test]
    fn range_idiom_folds() {
        // clang merges `x == 0 || x > MAX` into `(x-K) <u (2^w-K+1)`;
        // restore the readable disjunction (unsigned `>` stays unsigned).
        let c = decompile_opt_wat(r#"(module
          (func (param i64) (result i32)
            (i64.lt_u
              (i64.sub (local.get 0) (i64.const 2147483392))
              (i64.const -2147483391))))"#);
        assert!(c.contains("||"), "expected disjunction, got:\n{c}");
        assert!(c.contains("== 0"), "expected null check, got:\n{c}");
        assert!(c.contains("uint64_t"), "expected unsigned compare, got:\n{c}");
        // non-idiom shapes must not rewrite (C1+C2 != 1 here)
        let c2 = decompile_opt_wat(r#"(module
          (func (param i64) (result i32)
            (i64.lt_u
              (i64.sub (local.get 0) (i64.const 100))
              (i64.const 200))))"#);
        assert!(!c2.contains("||"), "must not rewrite, got:\n{c2}");
    }

    #[test]
    fn scoped_trim_drops_unreferenced() {
        // Dynamic-only access: no const addrs, no globals -> everything trimmed.
        let bytes = wat::parse_str(r#"(module (memory 1)
          (global $g (mut i32) (i32.const 0))
          (data (i32.const 100) "hello-world")
          (func (param i32) (result i32) (i32.load8_u (local.get 0))))"#)
        .unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mut mir = lift::lift_module(&bytes, &mut meta).unwrap();
        emit::trim_for_scoped_view(&mut mir);
        assert!(mir.data.is_empty(), "expected no segments, got {:?}", mir.data.len());
        assert!(mir.globals.is_empty(), "expected no globals");
    }

    #[test]
    fn scoped_trim_keeps_referenced_range() {
        // Const load at 1100 in a 2000-byte segment at 1000 + global use:
        // segment survives trimmed (1100-256 .. 1100+256), global survives.
        let mut data = "(module (memory 1) (global $g (mut i32) (i32.const 0)) (data (i32.const 1000) \"".to_string();
        data.push_str(&"ab".repeat(1000));
        data.push_str("\") (func (result i32) (i32.add (i32.load8_u (i32.const 1500)) (global.get 0))))");
        let bytes = wat::parse_str(&data).unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mut mir = lift::lift_module(&bytes, &mut meta).unwrap();
        emit::trim_for_scoped_view(&mut mir);
        assert_eq!(mir.data.len(), 1, "expected one segment");
        assert_eq!(mir.data[0].offset, Some(1244));
        assert_eq!(mir.data[0].bytes.len(), 513);
        assert_eq!(mir.globals.len(), 1, "expected used global kept");
        // notes still resolve inside the trimmed window
        let c = emit::emit_c(&mir);
        assert!(c.contains("mem + (1500)") || c.contains("1500"), "expected addr note, got:\n{c}");
    }

    #[test]
    fn var_parse_and_name() {        use mighty_wasm_dec::ir::{Expr, Var};
        assert_eq!(Var::parse("l3"), Some(Var::Local(3)));
        assert_eq!(Var::parse("t0"), Some(Var::Tmp(0)));
        assert_eq!(Var::parse("g12"), Some(Var::Global(12)));
        assert_eq!(Var::parse(""), None);
        assert_eq!(Var::parse("l"), None);
        assert_eq!(Var::parse("expr"), None);
        assert_eq!(Var::parse("S_0_l0"), None);
        assert_eq!(Var::Local(10).name(), "l10");
        assert_eq!(Var::Tmp(1).to_string(), "t1");
        // Numeric ordering (NOT lexicographic: Local(2) < Local(10), while
        // "l10" < "l2" as strings — candidate sorting uses name() to keep
        // the historical order byte-identical).
        assert!(Var::Local(2) < Var::Local(10));
        // Expr surface used by passes.
        let e = Expr::Binop {
            op: "+".into(),
            lhs: Box::new(Expr::Local(1)),
            rhs: Box::new(Expr::ConstI32(5)),
        };
        let mut vs = Vec::new();
        e.vars(&mut vs);
        assert_eq!(vs, vec![Var::Local(1)]);
        assert!(e.is_pure());
        assert_eq!(e.depth(), 1);
        let casted = Expr::Unop {
            op: "(uint32_t)".into(),
            v: Box::new(Expr::Local(2)),
        };
        assert!(matches!(casted.strip_casts(), Expr::Local(2)));
    }

    #[test]
    fn cfg_covers_branches_and_loops() {
        // if/else with returns in both arms + loop + br_table: every edge
        // must resolve (no placeholder targets left behind).
        let f = lift_wat(r#"(module
          (func (param i32) (result i32) (local i32)
            (if (i32.eqz (local.get 0)) (then (return (i32.const 1))) (else (nop)))
            (block $b (loop $l
              (br_if $b (i32.ge_s (local.get 1) (local.get 0)))
              (local.set 1 (i32.add (local.get 1) (i32.const 1)))
              (br $l)))
            (block (br_table 0 0 (local.get 1)))
            (i32.const 7)))"#);
        let g = cfg::build(&f.body);
        // No unresolved branch-taken edges.
        for b in &g.blocks {
            if let cfg::Terminator::Branch { then_b, .. } = b.term {
                assert_ne!(then_b, usize::MAX, "unresolved BrIf in block {}", b.id);
            }
        }
        // def/use maps see the counter local.
        use mighty_wasm_dec::ir::Var;
        assert!(!g.def_blocks.get(&Var::Local(1)).unwrap_or(&vec![]).is_empty());
        assert!(!g.use_blocks.get(&Var::Local(0)).unwrap_or(&vec![]).is_empty());
        // Entry reaches the return-bearing blocks (no lost code). The
        // function-exit block may itself be unreachable (code after an
        // unconditional `br_table` is dead) — only reachable returns
        // must be dominated by entry.
        let dom = cfg::dominators(&g);
        let reach = cfg::reachable(&g);
        let ret_blocks: Vec<_> = g
            .blocks
            .iter()
            .filter(|b| matches!(b.term, cfg::Terminator::Return(_)) && reach.contains(&b.id))
            .map(|b| b.id)
            .collect();
        assert!(!ret_blocks.is_empty());
        for r in ret_blocks {
            assert!(cfg::dominates(&dom, g.entry, r), "entry must dominate {r}");
        }
    }

    #[test]
    fn cfg_diamond_dominance() {
        // if/else join: entry dominates the join, arms don't dominate it.
        let f = lift_wat(r#"(module
          (func (param i32) (result i32)
            (if (result i32) (local.get 0)
              (then (i32.const 1)) (else (i32.const 2)))))"#);
        let g = cfg::build(&f.body);
        let dom = cfg::dominators(&g);
        // Join = the block with 2 preds that isn't entry.
        let join = g.blocks.iter().find(|b| b.preds.len() == 2).map(|b| b.id);
        let Some(j) = join else { panic!("expected diamond join") };
        assert!(cfg::dominates(&dom, g.entry, j));
        for b in &g.blocks {
            if b.id != g.entry && b.id != j && !b.stmts.is_empty() {
                assert!(!cfg::dominates(&dom, b.id, j), "arm {} must not dominate join", b.id);
            }
        }
        // blocks_between(entry, join) covers the diamond arms.
        let between = cfg::blocks_between(&g, g.entry, j);
        assert!(between.contains(&g.entry) && between.contains(&j));
        assert!(between.len() >= 4, "diamond has entry+2 arms+join, got {}", between.len());
    }

    #[test]
    fn const_ret_folds_across_functions() {
        use mighty_wasm_dec::callgraph;
        use mighty_wasm_dec::passes::ModulePass;
        let bytes = wat::parse_str(r#"(module
          (func $k (result i32) (i32.const 41))
          (func (result i32) (i32.add (call $k) (i32.const 1))))"#)
        .unwrap();
        let mut meta = parse::parse_meta(&bytes).unwrap();
        let mut mir = lift::lift_module(&bytes, &mut meta).unwrap();
        for f in mir.funcs.iter_mut() {
            passes::optimize_func(f);
        }
        // call graph sees the edge; callee summarizes to a const
        let g = callgraph::graph(&mir);
        assert!(g.values().any(|cs| cs.contains(&0)));
        let sums = callgraph::summarize(&mir);
        assert!(sums.values().any(|s| s.ret_const.is_some()));
        assert!(callgraph::sccs(&mir).iter().all(|scc| !scc.is_empty()));
        passes::ConstRetPass.run(&mut mir);
        let c = emit::emit_c(&mir);
        assert!(c.contains("return (41 + 1)") || c.contains("return 42"), "expected folded const, got:\n{c}");
    }

    #[test]
    fn narrow_store_not_widened() {
        // i32 store at +0 with a dominant i64 field there must not render
        // through the 8-byte field (fill_blob: tag store via f_off_0).
        let c = decompile_opt_wat(r#"(module (memory 1)
          (func (param i32 i32)
            (i64.store (local.get 0) (i64.const 1))
            (i64.store (local.get 0) (i64.const 2))
            (i64.store offset=8 (local.get 0) (i64.const 3))
            (i32.store (local.get 0) (local.get 1))))"#);
        assert!(
            c.contains("*(int32_t*)(mem + (l0)) = l1"),
            "expected raw narrow store, got:\n{c}"
        );
    }
}
