use std::collections::{HashMap, HashSet};

use crate::ir::{DataSeg, Expr, FuncIR, ModuleIR, Stmt};
use crate::types::{field_c_ty, recover};

/// String display mode (`--strings=`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StringsMode {
    Off,
    #[default]
    Comment,
    /// Emit a `static const` table for referenced strings + `s_<addr>[i]` refs.
    Defs,
}

/// Emitter view switches (`--no-struct`, `--strings=`).
#[derive(Debug, Clone)]
pub struct EmitConfig {
    pub struct_on: bool,
    pub strings: StringsMode,
}

impl Default for EmitConfig {
    fn default() -> Self {
        EmitConfig {
            struct_on: true,
            strings: StringsMode::Comment,
        }
    }
}

/// Printable run inside a data segment.
#[derive(Debug, Clone)]
pub struct StrEntry {
    pub addr: u64,
    pub seg: u32,
    pub off: u64,
    pub text: String,
    pub len: usize,
    pub nul: bool,
}

pub fn collect_strings(data: &[DataSeg]) -> Vec<StrEntry> {
    let mut out = Vec::new();
    for d in data {
        let Some(base) = d.offset else { continue };
        let start = base as i64 as u64;
        let b = &d.bytes;
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if !(0x20..=0x7e).contains(&c) && c != b'\t' && c != b'\n' && c != b'\r' {
                i += 1;
                continue;
            }
            let mut j = i;
            while j < b.len()
                && ((0x20..=0x7e).contains(&b[j]) || b[j] == b'\t' || b[j] == b'\n' || b[j] == b'\r')
            {
                j += 1;
            }
            if j - i >= 4 {
                let nul = j < b.len() && b[j] == 0;
                out.push(StrEntry {
                    addr: start.wrapping_add(i as u64),
                    seg: d.idx,
                    off: i as u64,
                    text: escape_c_str(&b[i..j]),
                    len: j - i,
                    nul,
                });
            }
            i = j.max(i + 1);
        }
    }
    out.sort_by_key(|e| e.addr);
    out
}

struct EmitCtx<'a> {
    layouts: &'a HashMap<String, (String, u64)>,
    data: &'a [DataSeg],
    strings: &'a [StrEntry],
    mode: StringsMode,
    struct_on: bool,
    /// labels targeted by goto (for collapsing unreferenced blocks)
    ref_labels: &'a HashSet<String>,
}

fn indent(n: usize) -> String {
    // Cap visual nesting: real modules nest ~1000 deep via dispatch
    // scaffolding. Whitespace carries no semantics in C and labels
    // disambiguate scopes, so clamp to keep lines on screen.
    "  ".repeat(n.min(32))
}

fn c_ty_of_load(ty: &str) -> &'static str {
    if ty == "v128" {
        "__v128"
    } else if ty.contains("w1") {
        "int8_t"
    } else if ty.contains("w2") {
        "int16_t"
    } else if ty.contains("f32") {
        "float"
    } else if ty.contains("f64") {
        "double"
    } else if ty.contains("i64") && !ty.contains("w4") {
        // i64/w8 loads; i64/w4 is a 4-byte zero-extended load -> int32_t
        "int64_t"
    } else if ty == "atomic/w8" {
        "int64_t"
    } else {
        "int32_t"
    }
}

fn emit_expr(e: &Expr, ctx: &EmitCtx) -> String {
    match e {
        Expr::ConstI32(v) => format!("{v}"),
        Expr::ConstI64(v) => format!("{v}ll"),
        Expr::ConstF32(b) => {
            let f = f32::from_bits(*b);
            if f.is_finite() {
                format!("{f:?}")
            } else {
                format!("((float)(0x{b:08x}))")
            }
        }
        Expr::ConstF64(b) => {
            let f = f64::from_bits(*b);
            if f.is_finite() {
                format!("{f:?}")
            } else {
                format!("((double)(0x{b:016x}))")
            }
        }
        Expr::Local(i) => format!("l{i}"),
        Expr::Tmp(i) => format!("t{i}"),
        Expr::Global(i) => format!("g{i}"),
        Expr::MemorySize(m) => format!("memory_size({m})"),
        Expr::Binop { op, lhs, rhs } => {
            format!("({} {} {})", emit_expr(lhs, ctx), op, emit_expr(rhs, ctx))
        }
        Expr::Unop { op, v } => {
            if op == "memgrow(" {
                format!("memgrow({}))", emit_expr(v, ctx))
            } else {
                format!("({}{})", op, emit_expr(v, ctx))
            }
        }
        Expr::Load { ty, base, offset } => {
            // defs-mode byte reads from known strings render as s_<addr>[i]
            if ctx.mode == StringsMode::Defs {
                if let Some((sym, idx)) = string_index(base, *offset, ty, ctx) {
                    return format!("{sym}[{idx}]");
                }
            }
            // constant base folds into a single absolute address
            if let Expr::ConstI32(a) = **base {
                let total = (a as i64 as u64).wrapping_add(*offset);
                if ty.starts_with("atomic") {
                    return format!("atomic.load(mem + {total})");
                }
                let ct = c_ty_of_load(ty);
                return format!("*({ct}*)(mem + {total})");
            }
            if let Expr::ConstI64(a) = **base {
                let total = (a as u64).wrapping_add(*offset);
                if ty.starts_with("atomic") {
                    return format!("atomic.load(mem + {total})");
                }
                let ct = c_ty_of_load(ty);
                return format!("*({ct}*)(mem + {total})");
            }
            if ty.starts_with("atomic") {
                let b = emit_expr(base, ctx);
                if *offset == 0 {
                    return format!("atomic.load(mem + ({b}))");
                } else {
                    return format!("atomic.load(mem + ({b}) + {offset})");
                }
            }
            let ct = c_ty_of_load(ty);
            let b = emit_expr(base, ctx);
            if *offset == 0 {
                format!("*({ct}*)(mem + ({b}))")
            } else {
                format!("*({ct}*)(mem + ({b}) + {offset})")
            }
        }
        Expr::Call { func: _, name, args } => {
            let a: Vec<String> = args.iter().map(|x| emit_expr(x, ctx)).collect();
            format!("{name}({})", a.join(", "))
        }
        Expr::CallIndirect { index, args, .. } => {
            let a: Vec<String> = args.iter().map(|x| emit_expr(x, ctx)).collect();
            format!("table_call({})({})", emit_expr(index, ctx), a.join(", "))
        }
        Expr::Select { c, a, b } => {
            format!(
                "({} ? {} : {})",
                emit_expr(c, ctx),
                emit_expr(a, ctx),
                emit_expr(b, ctx)
            )
        }
        Expr::Simd { op, args } => {
            let a: Vec<String> = args.iter().map(|x| emit_expr(x, ctx)).collect();
            format!("{op}({})", a.join(", "))
        }
        Expr::Raw(s) => s.clone(),
        Expr::Unknown(s) => format!("/*{s}*/0"),
    }
}

/// If base+offset matches a recovered struct field, render as struct access.
/// The layouts map is pre-expanded through copyprop aliases by the caller.
fn struct_access(base: &Expr, offset: u64, ctx: &EmitCtx) -> Option<String> {
    if !ctx.struct_on {
        return None;
    }
    let key = match base {
        Expr::Local(i) => format!("l{i}"),
        Expr::Tmp(i) => format!("t{i}"),
        Expr::Global(i) => format!("g{i}"),
        _ => return None,
    };
    ctx.layouts
        .get(&format!("{key}+{offset}"))
        .cloned()
        .map(|(s, _)| s)
}

/// Byte read from a known string in defs mode: `s_<addr>[i]`.
/// Only width-1 loads; anything else stays numeric (comment mode covers it).
fn string_index(base: &Expr, mem_offset: u64, ty: &str, ctx: &EmitCtx) -> Option<(String, String)> {
    if ctx.mode != StringsMode::Defs || ctx.strings.is_empty() {
        return None;
    }
    // Only byte loads read characters; wider loads stay numeric.
    if !ty.contains("w1") {
        return None;
    }
    // split base into (dynamic, const) parts
    let (dyn_part, c): (Option<&Expr>, u64) = match base {
        Expr::ConstI32(v) => (None, *v as i64 as u64),
        Expr::ConstI64(v) => (None, *v as u64),
        Expr::Binop { op, lhs, rhs } if op == "+" => match (&**lhs, &**rhs) {
            (Expr::ConstI32(v), d) | (d, Expr::ConstI32(v)) => (Some(d), *v as i64 as u64),
            (Expr::ConstI64(v), d) | (d, Expr::ConstI64(v)) => (Some(d), *v as u64),
            _ => return None,
        },
        _ => return None,
    };
    let total = c.wrapping_add(mem_offset);
    let s = ctx
        .strings
        .iter()
        .find(|e| total >= e.addr && total < e.addr + e.len as u64)?;
    let k = total - s.addr;
    let sym = format!("s_{}", s.addr);
    let idx = match dyn_part {
        None => format!("{k}"),
        Some(d) => {
            if k == 0 {
                emit_expr(d, ctx)
            } else {
                format!("({} + {k})", emit_expr(d, ctx))
            }
        }
    };
    Some((sym, idx))
}

/// If a memory access resolves to a constant address inside a data
/// segment, name it. With strings on, prefer the string literal.
/// Handles both direct constants and `dyn + const` address math.
/// `is_byte` gates the `sym[idx]` indexing form to byte reads; wider
/// reads of string bytes stay numeric (a 4/8-byte load isn't characters).
fn data_note(base: &Expr, mem_offset: u64, is_byte: bool, ctx: &EmitCtx) -> Option<String> {
    if ctx.mode == StringsMode::Off {
        return None;
    }
    let a: Option<u64> = match base {
        Expr::ConstI32(v) => Some((*v as i64 as u64).wrapping_add(mem_offset)),
        Expr::ConstI64(v) => Some((*v as u64).wrapping_add(mem_offset)),
        _ => None,
    };
    if let Some(a) = a {
        if let Some(s) = ctx
            .strings
            .iter()
            .find(|e| a >= e.addr && a < e.addr + e.len as u64)
        {
            let shown: String = s.text.chars().take(32).collect();
            let more = if s.text.chars().count() > 32 {
                format!("…+{}b", s.len)
            } else {
                String::new()
            };
            return Some(format!("\"{shown}\"{more}"));
        }
        for d in ctx.data {
            if let Some(off) = d.offset {
                let start = off as i64 as u64;
                let end = start.wrapping_add(d.bytes.len() as u64);
                if a >= start && a < end {
                    return Some(format!("data{}+{}", d.idx, a - start));
                }
            }
        }
        return None;
    }
    // `dyn + const` address math: only name it when the constant part is a
    // string base and the access is a byte read — i.e. `sym[idx]` indexing,
    // the same rule as defs mode. Anything else stays numeric (honest).
    if !is_byte {
        return None;
    }
    if let Expr::Binop { op, lhs, rhs } = base {
        if op != "+" {
            return None;
        }
        let (dyn_part, c) = match (&**lhs, &**rhs) {
            (Expr::ConstI32(v), d) | (d, Expr::ConstI32(v)) => (d, *v as i64 as u64),
            (Expr::ConstI64(v), d) | (d, Expr::ConstI64(v)) => (d, *v as u64),
            _ => return None,
        };
        let total = c.wrapping_add(mem_offset);
        let s = ctx
            .strings
            .iter()
            .find(|e| total >= e.addr && total < e.addr + e.len as u64)?;
        let k = total - s.addr;
        let idx = if k == 0 {
            emit_expr(dyn_part, ctx)
        } else {
            format!("({} + {k})", emit_expr(dyn_part, ctx))
        };
        return Some(format!("s_{}[{}]", s.addr, idx));
    }
    None
}

fn emit_stmts(stmts: &[Stmt], out: &mut String, lvl: usize, ctx: &EmitCtx) {
    for s in stmts {
        match s {
            Stmt::Assign { dst, expr } => {
                // peephole: skip self-assignments like `l3 = l3` (tee lowering residue)
                if emit_expr(expr, ctx) == *dst {
                    continue;
                }
                // annotate loads from known data segments (statics/globals)
                let note = match expr {
                    Expr::Load { base, offset, ty } => {
                        data_note(base, *offset, ty.contains("w1"), ctx)
                            .map(|n| format!(" /* {n} */"))
                            .unwrap_or_default()
                    }
                    _ => String::new(),
                };
                out.push_str(&format!("{}{} = {};{note}\n", indent(lvl), dst, emit_expr(expr, ctx)));
            }
            Stmt::Store { ty, base, offset, value } => {
                let ct = c_ty_of_load(ty);
                if ty.starts_with("atomic") {
                    // atomicity is semantic; keep it visible
                    let b = emit_expr(base, ctx);
                    let addr = if *offset == 0 {
                        format!("mem + ({b})")
                    } else {
                        format!("mem + ({b}) + {offset}")
                    };
                    out.push_str(&format!(
                        "{}atomic.store({addr}, {});\n",
                        indent(lvl),
                        emit_expr(value, ctx)
                    ));
                } else if let Some(acc) = struct_access(base, *offset, ctx) {
                    out.push_str(&format!("{}{} = {};\n", indent(lvl), acc, emit_expr(value, ctx)));
                } else if let Expr::ConstI32(a) = base {
                    let total = (*a as i64 as u64).wrapping_add(*offset);
                    let note = data_note(base, *offset, ty.contains("w1"), ctx)
                        .map(|n| format!(" /* {n} */"))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "{}*({ct}*)(mem + {total}) = {};{note}\n",
                        indent(lvl),
                        emit_expr(value, ctx)
                    ));
                } else {
                    let b = emit_expr(base, ctx);
                    let note = data_note(base, *offset, ty.contains("w1"), ctx)
                        .map(|n| format!(" /* {n} */"))
                        .unwrap_or_default();
                    if *offset == 0 {
                        out.push_str(&format!(
                            "{}*({ct}*)(mem + ({b})) = {};{note}\n",
                            indent(lvl),
                            emit_expr(value, ctx)
                        ));
                    } else {
                        out.push_str(&format!(
                            "{}*({ct}*)(mem + ({b}) + {offset}) = {};{note}\n",
                            indent(lvl),
                            emit_expr(value, ctx)
                        ));
                    }
                }
            }
            Stmt::ExprStmt(e) => {
                out.push_str(&format!("{}{};\n", indent(lvl), emit_expr(e, ctx)));
            }
            Stmt::If { cond, then_b, else_b } => {
                out.push_str(&format!("{}if ({}) {{\n", indent(lvl), emit_expr(cond, ctx)));
                emit_stmts(then_b, out, lvl + 1, ctx);
                if !else_b.is_empty() {
                    out.push_str(&format!("{}}} else {{\n", indent(lvl)));
                    emit_stmts(else_b, out, lvl + 1, ctx);
                }
                out.push_str(&format!("{}}}\n", indent(lvl)));
            }
            Stmt::Block { label, body } => {
                // unreferenced labels with straight-line bodies collapse to braces
                let referenced = ctx.ref_labels.contains(label);
                let ends_in_goto = matches!(
                    body.last(),
                    Some(Stmt::Br { .. } | Stmt::BrIf { .. } | Stmt::BrTable { .. })
                );
                if !referenced && !ends_in_goto {
                    out.push_str(&format!("{}/* block */ {{\n", indent(lvl)));
                    emit_stmts(body, out, lvl + 1, ctx);
                    out.push_str(&format!("{}}}\n", indent(lvl)));
                } else {
                    out.push_str(&format!("{}do {{ /* block {label} */\n", indent(lvl)));
                    emit_stmts(body, out, lvl + 1, ctx);
                    out.push_str(&format!(
                        "{}}} while (0);\n{}__end_{label}: ;\n",
                        indent(lvl),
                        indent(lvl)
                    ));
                }
            }
            Stmt::Loop { label, body } => {
                out.push_str(&format!("{}__head_{label}: while (1) {{ /* loop {label} */\n", indent(lvl)));
                emit_stmts(body, out, lvl + 1, ctx);
                out.push_str(&format!("{}}}\n{}__end_{label}: ;\n", indent(lvl), indent(lvl)));
            }
            Stmt::Br { label, is_loop, .. } => {
                if *is_loop {
                    out.push_str(&format!("{}goto __head_{label}; /* continue */\n", indent(lvl)));
                } else {
                    out.push_str(&format!("{}goto __end_{label}; /* break */\n", indent(lvl)));
                }
            }
            Stmt::BrIf { label, is_loop, cond, .. } => {
                if *is_loop {
                    out.push_str(&format!(
                        "{}if ({}) goto __head_{label};\n",
                        indent(lvl),
                        emit_expr(cond, ctx)
                    ));
                } else {
                    out.push_str(&format!(
                        "{}if ({}) goto __end_{label};\n",
                        indent(lvl),
                        emit_expr(cond, ctx)
                    ));
                }
            }
            Stmt::BrTable { index, targets, default } => {
                out.push_str(&format!("{}switch ({}) {{\n", indent(lvl), emit_expr(index, ctx)));
                for (i, (_, label, is_loop)) in targets.iter().enumerate() {
                    let dst = if *is_loop {
                        format!("__head_{label}")
                    } else {
                        format!("__end_{label}")
                    };
                    out.push_str(&format!("{}case {i}: goto {dst};\n", indent(lvl + 1)));
                }
                let (_, dlabel, dis_loop) = default;
                let dst = if *dis_loop {
                    format!("__head_{dlabel}")
                } else {
                    format!("__end_{dlabel}")
                };
                out.push_str(&format!("{}default: goto {dst};\n", indent(lvl + 1)));
                out.push_str(&format!("{}}}\n", indent(lvl)));
            }
            Stmt::Return { values } => {
                if values.is_empty() {
                    out.push_str(&format!("{}return;\n", indent(lvl)));
                } else {
                    // Patch loads inside return through struct-access map is handled
                    // at Expr level only for stores; keep loads raw but correct.
                    let v: Vec<String> = values.iter().map(|x| emit_expr(x, ctx)).collect();
                    out.push_str(&format!("{}return {};\n", indent(lvl), v.join(", ")));
                }
            }
            Stmt::Comment(c) => {
                out.push_str(&format!("{}/* {c} */\n", indent(lvl)));
            }
        }
    }
}

fn escape_c_str(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        match b {
            0x20..=0x7e if *b != b'"' && *b != b'\\' => s.push(*b as char),
            b'\n' => s.push_str("\\n"),
            _ => s.push_str(&format!("\\x{b:02x}")),
        }
    }
    s
}

pub fn emit_c(m: &ModuleIR) -> String {
    emit_c_with(m, &EmitConfig::default())
}

/// Resolve a struct-layout base through copyprop aliases (cycle-guarded).
fn resolve_alias<'a>(base: &str, aliases: &'a HashMap<String, String>) -> String {
    let mut cur = base.to_string();
    for _ in 0..16 {
        match aliases.get(&cur) {
            Some(n) if *n != cur => cur = n.clone(),
            _ => break,
        }
    }
    cur
}

fn collect_ref_labels(stmts: &[Stmt], set: &mut HashSet<String>) {
    for s in stmts {
        match s {
            Stmt::Br { label, .. } | Stmt::BrIf { label, .. } => {
                set.insert(label.clone());
            }
            Stmt::BrTable { targets, default, .. } => {
                for (_, l, _) in targets {
                    set.insert(l.clone());
                }
                set.insert(default.1.clone());
            }
            Stmt::If { then_b, else_b, .. } => {
                collect_ref_labels(then_b, set);
                collect_ref_labels(else_b, set);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => collect_ref_labels(body, set),
            _ => {}
        }
    }
}

/// Absolute addresses of string-table entries referenced by byte loads.
fn collect_str_refs_expr(e: &Expr, strings: &[StrEntry], acc: &mut std::collections::BTreeSet<u64>) {
    match e {
        Expr::Load { base, offset, ty } => {
            if ty.contains("w1") {
                let total: Option<u64> = match &**base {
                    Expr::ConstI32(v) => Some((*v as i64 as u64).wrapping_add(*offset)),
                    Expr::ConstI64(v) => Some((*v as u64).wrapping_add(*offset)),
                    Expr::Binop { op, lhs, rhs } if op == "+" => {
                        let c = match (&**lhs, &**rhs) {
                            (Expr::ConstI32(v), _) => Some(*v as i64 as u64),
                            (_, Expr::ConstI32(v)) => Some(*v as i64 as u64),
                            (Expr::ConstI64(v), _) => Some(*v as u64),
                            (_, Expr::ConstI64(v)) => Some(*v as u64),
                            _ => None,
                        };
                        c.map(|c| c.wrapping_add(*offset))
                    }
                    _ => {
                        collect_str_refs_expr(base, strings, acc);
                        None
                    }
                };
                if let Some(t) = total {
                    if let Some(s) = strings
                        .iter()
                        .find(|x| t >= x.addr && t < x.addr + x.len as u64)
                    {
                        acc.insert(s.addr);
                    }
                }
            } else {
                collect_str_refs_expr(base, strings, acc);
            }
        }
        Expr::Binop { lhs, rhs, .. } => {
            collect_str_refs_expr(lhs, strings, acc);
            collect_str_refs_expr(rhs, strings, acc);
        }
        Expr::Unop { v, .. } => collect_str_refs_expr(v, strings, acc),
        Expr::Call { args, .. } => {
            for a in args {
                collect_str_refs_expr(a, strings, acc);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            collect_str_refs_expr(index, strings, acc);
            for a in args {
                collect_str_refs_expr(a, strings, acc);
            }
        }
        Expr::Select { c, a, b } => {
            collect_str_refs_expr(c, strings, acc);
            collect_str_refs_expr(a, strings, acc);
            collect_str_refs_expr(b, strings, acc);
        }
        Expr::Simd { args, .. } => {
            for a in args {
                collect_str_refs_expr(a, strings, acc);
            }
        }
        _ => {}
    }
}

fn collect_str_refs(stmts: &[Stmt], strings: &[StrEntry], acc: &mut std::collections::BTreeSet<u64>) {
    for s in stmts {
        match s {
            Stmt::Assign { expr, .. } => collect_str_refs_expr(expr, strings, acc),
            Stmt::Store { base, value, .. } => {
                collect_str_refs_expr(base, strings, acc);
                collect_str_refs_expr(value, strings, acc);
            }
            Stmt::ExprStmt(e) => collect_str_refs_expr(e, strings, acc),
            Stmt::If { cond, then_b, else_b } => {
                collect_str_refs_expr(cond, strings, acc);
                collect_str_refs(then_b, strings, acc);
                collect_str_refs(else_b, strings, acc);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                collect_str_refs(body, strings, acc)
            }
            Stmt::BrIf { cond, .. } => collect_str_refs_expr(cond, strings, acc),
            Stmt::BrTable { index, .. } => collect_str_refs_expr(index, strings, acc),
            Stmt::Return { values } => {
                for v in values {
                    collect_str_refs_expr(v, strings, acc);
                }
            }
            _ => {}
        }
    }
}

pub fn emit_c_with(m: &ModuleIR, cfg: &EmitConfig) -> String {
    let mut out = String::new();
    out.push_str("#include <stdint.h>\n#include <stddef.h>\n#include <math.h>\n");
    out.push_str("typedef unsigned __int128 __v128_u __attribute__((vector_size(16)));\n");
    out.push_str("typedef __v128_u __v128;\n");
    out.push_str("static uint8_t *mem = 0; /* wasm linear memory base (host provides) */\n");
    out.push_str("static uint32_t memory_size(uint32_t m) { (void)m; return 0; }\n");
    out.push_str("static uint32_t memgrow(uint32_t d) { (void)d; return (uint32_t)-1; }\n\n");

    let strings = collect_strings(&m.data);

    if !m.imports.is_empty() {
        out.push_str("/* imports (host-provided) */\n");
        for im in &m.imports {
            if im.kind == "func" {
                let ret = if im.results.is_empty() {
                    "void".to_string()
                } else {
                    im.results[0].to_string()
                };
                let ps: Vec<String> = im
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| format!("{p} a{i}"))
                    .collect();
                out.push_str(&format!(
                    "{} {}({}); /* {}.{} */\n",
                    ret,
                    im.name,
                    ps.join(", "),
                    im.module,
                    im.name
                ));
            } else {
                out.push_str(&format!("/* import {}.{} : {} */\n", im.module, im.name, im.kind));
            }
        }
        out.push('\n');
    }
    if !m.globals.is_empty() {
        out.push_str("/* globals */\n");
        for g in &m.globals {
            let mut_ = if g.mutable { "/*mut*/ " } else { "" };
            out.push_str(&format!("static {} {}g{} = 0;\n", g.ty, mut_, g.idx));
        }
        out.push('\n');
    }

    // forward decls (defined funcs use display names; imported funcs are f<num> stubs)
    for f in &m.funcs {
        // mirror the per-function signature rewrite so decls match definitions
        let mut decl_struct: HashMap<String, String> = HashMap::new();
        if cfg.struct_on {
            for s in recover(f) {
                decl_struct.entry(s.base.clone()).or_insert(s.name.clone());
            }
        }
        out.push_str(&format!("{} {}(", ret_ty(f), f.name));
        let mut ps: Vec<String> = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            if let Some(s) = decl_struct.get(&format!("l{i}")) {
                ps.push(format!("{s}* p{i}"));
            } else {
                ps.push(format!("{p} p{i}"));
            }
        }
        out.push_str(&format!("{});\n", ps.join(", ")));
    }
    out.push('\n');

    // merged struct view: group identical offset-signatures across funcs
    let mut sig_first: HashMap<Vec<u64>, String> = HashMap::new();
    let mut struct_defs: Vec<String> = Vec::new();
    if cfg.struct_on {
        for f in &m.funcs {
            for s in recover(f) {
            let mut sig: Vec<u64> = s.fields.iter().map(|x| x.offset).collect();
            sig.sort();
            if let Some(existing) = sig_first.get(&sig) {
                struct_defs.push(format!(
                    "/* {} (f{}:{}) aliases {} */\ntypedef {} {};",
                    s.name,
                    f.idx,
                    s.base,
                    existing,
                    existing,
                    s.name
                ));
            } else {
                sig_first.insert(sig, s.name.clone());
                let mut d = format!(
                    "/* recovered from {} via base {} */\ntypedef struct {} {{\n",
                    f.name, s.base, s.name
                );
                for fl in &s.fields {
                    d.push_str(&format!(
                        "  {} f_off_{}; /* +{} w{} x{} */\n",
                        field_c_ty(fl),
                        fl.offset,
                        fl.offset,
                        fl.width,
                        fl.count
                    ));
                }
                d.push_str(&format!("}} {};", s.name));
                struct_defs.push(d);
                }
            }
        }
    }
    for d in struct_defs {
        out.push_str(&d);
        out.push('\n');
    }
    out.push('\n');

    // defs-mode string table: only strings actually referenced by loads
    if cfg.strings == StringsMode::Defs && !strings.is_empty() {
        let mut refs = std::collections::BTreeSet::new();
        for f in &m.funcs {
            collect_str_refs(&f.body, &strings, &mut refs);
        }
        if !refs.is_empty() {
            out.push_str("/* referenced strings */\n");
            for addr in refs {
                if let Some(s) = strings.iter().find(|e| e.addr == addr) {
                    out.push_str(&format!(
                        "static const char s_{addr}[] = \"{}\"; /* data{}+{} ({}b{}) */\n",
                        s.text,
                        s.seg,
                        s.off,
                        s.len,
                        if s.nul { ", NUL" } else { "" }
                    ));
                }
            }
            out.push('\n');
        }
    }

    for f in &m.funcs {
        // build struct-access map: "lN+off" -> "((S*)lN)->f_off_O"
        let mut layouts: HashMap<String, (String, u64)> = HashMap::new();
        // base -> struct name (for signature rewrite of pointer params)
        let mut base_struct: HashMap<String, String> = HashMap::new();
        if cfg.struct_on {
            for s in recover(f) {
                base_struct.entry(s.base.clone()).or_insert(s.name.clone());
                for fl in &s.fields {
                    layouts.insert(
                        format!("{}+{}", s.base, fl.offset),
                        (format!("(({}*){})->f_off_{}", s.name, s.base, fl.offset), fl.offset),
                    );
                }
            }
            // expand through copyprop aliases: uses may name the new base
            // while layouts were recorded under the old one.
            let mut extra = Vec::new();
            for (old, new) in &f.aliases {
                let final_base = resolve_alias(new, &f.aliases);
                for (k, (text, off)) in &layouts {
                    if let Some(rest) = k.strip_prefix(&format!("{old}+")) {
                        let new_text = text.replace(
                            &format!("){old})->"),
                            &format!("){final_base})->"),
                        );
                        extra.push((format!("{final_base}+{rest}"), (new_text, *off)));
                    }
                }
                // alias the struct name itself for signature rewrite
                if let Some(sname) = base_struct.get(old).cloned() {
                    base_struct.entry(final_base).or_insert(sname);
                }
            }
            for (k, v) in extra {
                layouts.entry(k).or_insert(v);
            }
        }
        // params that feed call_indirect indices are table indices, not plain ints
        let mut table_idx_params: HashMap<usize, bool> = HashMap::new();
        collect_indirect_params(&f.body, f.params.len(), &mut table_idx_params);
        let param_ty = |i: usize, p: &crate::ir::WasmTy| -> String {
            if let Some(s) = base_struct.get(&format!("l{i}")) {
                return format!("{s}*");
            }
            p.to_string()
        };
        out.push_str(&format!("{} {}(", ret_ty(f), f.name));
        let mut ps: Vec<String> = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            let mut decl = format!("{} p{i}", param_ty(i, p));
            if table_idx_params.get(&i).copied().unwrap_or(false) {
                decl.push_str(" /* table index */");
            }
            ps.push(decl);
        }
        out.push_str(&format!("{}) {{\n", ps.join(", ")));
        for (i, p) in f.params.iter().enumerate() {
            out.push_str(&format!("  {} l{i} = p{i};\n", param_ty(i, p)));
        }
        for (j, l) in f.locals.iter().enumerate() {
            let idx = f.params.len() + j;
            out.push_str(&format!("  {l} l{idx} = 0;\n"));
        }
        let mut tmps: Vec<u32> = Vec::new();
        collect_tmps(&f.body, &mut tmps);
        tmps.sort_unstable();
        tmps.dedup();
        for t in tmps {
            out.push_str(&format!("  int32_t t{t} = 0; /* synthetic (type TODO) */\n"));
        }
        if cfg.struct_on {
            for s in recover(f) {
                out.push_str(&format!(
                    "  /* hint: {} is {}* ({} fields) */\n",
                    s.base,
                    s.name,
                    s.fields.len()
                ));
            }
        }
        // stack-pointer heuristic note (Emscripten-style global 0)
        if f.body.iter().any(|s| stmt_uses_global0(s)) {
            out.push_str("  /* note: uses g0 (possible C stack pointer) — SROA not yet applied */\n");
        }
        let mut ref_labels = HashSet::new();
        collect_ref_labels(&f.body, &mut ref_labels);
        let ctx = EmitCtx {
            layouts: &layouts,
            data: &m.data,
            strings: &strings,
            mode: cfg.strings,
            struct_on: cfg.struct_on,
            ref_labels: &ref_labels,
        };
        emit_stmts(&f.body, &mut out, 1, &ctx);
        let ends_with_return = f.body.last().is_some_and(|s| matches!(s, Stmt::Return { .. }));
        if !ends_with_return {
            if f.results.is_empty() {
                out.push_str("}\n\n");
            } else {
                out.push_str(&format!(
                    "  return 0; /* TODO: missing result {} */\n}}\n\n",
                    f.results[0]
                ));
            }
        } else {
            out.push_str("}\n\n");
        }
    }

    if !m.data.is_empty() {
        out.push_str("/* data segments */\n");
        for d in &m.data {
            let preview: Vec<u8> = d.bytes.iter().cloned().take(64).collect();
            let printable = preview.iter().filter(|b| b.is_ascii_graphic() || **b == b' ').count();
            if d.bytes.len() <= 128 && printable * 2 >= preview.len() {
                out.push_str(&format!(
                    "/* data{} @{:?} ({} bytes): \"{}\" */\n",
                    d.idx,
                    d.offset,
                    d.bytes.len(),
                    escape_c_str(&d.bytes)
                ));
            } else {
                // short binary segments: also dump LE i32 words (jump tables, int arrays)
                let mut extra = String::new();
                if d.bytes.len() <= 64 && d.bytes.len() % 4 == 0 && !d.bytes.is_empty() {
                    let words: Vec<String> = d
                        .bytes
                        .chunks_exact(4)
                        .map(|w| {
                            let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]);
                            format!("{v}")
                        })
                        .collect();
                    extra = format!(" words:[{}]", words.join(","));
                }
                // list embedded strings so large mixed rodata stays searchable:
                // one line per run (truncated), not just a count.
                let seg_strings: Vec<&StrEntry> =
                    strings.iter().filter(|e| e.seg == d.idx).collect();
                if !seg_strings.is_empty() {
                    extra.push_str(&format!(" strings({})", seg_strings.len()));
                }
                out.push_str(&format!(
                    "/* data{} @{:?} ({} bytes, binary{}) */\n",
                    d.idx,
                    d.offset,
                    d.bytes.len(),
                    extra
                ));
                for e in seg_strings {
                    let shown: String = e.text.chars().take(64).collect();
                    let more = if e.text.chars().count() > 64 {
                        format!("…+{}b", e.len)
                    } else {
                        String::new()
                    };
                    out.push_str(&format!(
                        "/*   [+{}] \"{}\"{}{} */\n",
                        e.off,
                        shown,
                        more,
                        if e.nul { ", NUL" } else { "" }
                    ));
                }
            }
        }
    }
    out.push_str("/* tables */\n");
    for (ti, tab) in m.tables.iter().enumerate() {
        let names: Vec<String> = tab
            .iter()
            .map(|o| {
                o.and_then(|idx| {
                    m.funcs
                        .iter()
                        .find(|f| f.idx == idx)
                        .map(|f| f.name.clone())
                })
                .unwrap_or("NULL".into())
            })
            .collect();
        out.push_str(&format!("/* table{ti}: [{}] */\n", names.join(", ")));
    }
    out
}

fn stmt_uses_global0(s: &Stmt) -> bool {
    match s {
        Stmt::Assign { expr, .. } => expr_uses_global0(expr),
        Stmt::Store { base, value, .. } => expr_uses_global0(base) || expr_uses_global0(value),
        Stmt::ExprStmt(e) => expr_uses_global0(e),
        Stmt::If { cond, then_b, else_b } => {
            expr_uses_global0(cond)
                || then_b.iter().any(stmt_uses_global0)
                || else_b.iter().any(stmt_uses_global0)
        }
        Stmt::Block { body, .. } | Stmt::Loop { body, .. } => body.iter().any(stmt_uses_global0),
        Stmt::BrIf { cond, .. } => expr_uses_global0(cond),
        Stmt::BrTable { index, .. } => expr_uses_global0(index),
        Stmt::Return { values } => values.iter().any(expr_uses_global0),
        _ => false,
    }
}

fn expr_uses_global0(e: &Expr) -> bool {
    match e {
        Expr::Global(0) => true,
        Expr::Binop { lhs, rhs, .. } => expr_uses_global0(lhs) || expr_uses_global0(rhs),
        Expr::Unop { v, .. } => expr_uses_global0(v),
        Expr::Load { base, .. } => expr_uses_global0(base),
        Expr::Call { args, .. } => args.iter().any(expr_uses_global0),
        Expr::CallIndirect { index, args, .. } => {
            expr_uses_global0(index) || args.iter().any(expr_uses_global0)
        }
        Expr::Select { c, a, b } => {
            expr_uses_global0(c) || expr_uses_global0(a) || expr_uses_global0(b)
        }
        Expr::Simd { args, .. } => args.iter().any(expr_uses_global0),
        _ => false,
    }
}

fn ret_ty(f: &FuncIR) -> String {
    if f.results.is_empty() {
        "void".into()
    } else {
        f.results[0].to_string()
    }
}

fn collect_indirect_params(stmts: &[Stmt], nparams: usize, acc: &mut HashMap<usize, bool>) {
    for s in stmts {
        match s {
            Stmt::ExprStmt(Expr::CallIndirect { index, .. }) => {
                if let Expr::Local(i) = **index {
                    if (i as usize) < nparams {
                        acc.insert(i as usize, true);
                    }
                }
            }
            Stmt::Assign { expr: Expr::CallIndirect { index, .. }, .. } => {
                if let Expr::Local(i) = **index {
                    if (i as usize) < nparams {
                        acc.insert(i as usize, true);
                    }
                }
            }
            Stmt::If { then_b, else_b, .. } => {
                collect_indirect_params(then_b, nparams, acc);
                collect_indirect_params(else_b, nparams, acc);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                collect_indirect_params(body, nparams, acc)
            }
            _ => {}
        }
    }
}

/// Absolute const addresses read/written by a body (for data filtering).
fn collect_data_addrs_expr(e: &Expr, acc: &mut Vec<u64>) {
    match e {
        Expr::Load { base, offset, .. } => {
            const_total(base, *offset).map(|a| acc.push(a));
            collect_data_addrs_expr(base, acc);
        }
        Expr::Binop { lhs, rhs, .. } => {
            collect_data_addrs_expr(lhs, acc);
            collect_data_addrs_expr(rhs, acc);
        }
        Expr::Unop { v, .. } => collect_data_addrs_expr(v, acc),
        Expr::Call { args, .. } => {
            for a in args {
                collect_data_addrs_expr(a, acc);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            collect_data_addrs_expr(index, acc);
            for a in args {
                collect_data_addrs_expr(a, acc);
            }
        }
        Expr::Select { c, a, b } => {
            collect_data_addrs_expr(c, acc);
            collect_data_addrs_expr(a, acc);
            collect_data_addrs_expr(b, acc);
        }
        Expr::Simd { args, .. } => {
            for a in args {
                collect_data_addrs_expr(a, acc);
            }
        }
        _ => {}
    }
}

fn const_total(base: &Expr, mem_offset: u64) -> Option<u64> {
    match base {
        Expr::ConstI32(v) => Some((*v as i64 as u64).wrapping_add(mem_offset)),
        Expr::ConstI64(v) => Some((*v as u64).wrapping_add(mem_offset)),
        Expr::Binop { op, lhs, rhs } if op == "+" => {
            let c = match (&**lhs, &**rhs) {
                (Expr::ConstI32(v), _) | (_, Expr::ConstI32(v)) => Some(*v as i64 as u64),
                (Expr::ConstI64(v), _) | (_, Expr::ConstI64(v)) => Some(*v as u64),
                _ => None,
            };
            c.map(|c| c.wrapping_add(mem_offset))
        }
        _ => None,
    }
}

/// Absolute const addresses read/written by a body (for data filtering).
pub fn collect_data_addrs(stmts: &[Stmt], acc: &mut Vec<u64>) {
    for s in stmts {
        match s {
            Stmt::Assign { expr, .. } => collect_data_addrs_expr(expr, acc),
            Stmt::Store { base, offset, value, .. } => {
                // the stored VALUE's address (for `mem + CONST` stores the base is it)
                if let Some(a) = const_total(base, *offset) {
                    acc.push(a);
                }
                collect_data_addrs_expr(base, acc);
                collect_data_addrs_expr(value, acc);
            }
            Stmt::ExprStmt(e) => collect_data_addrs_expr(e, acc),
            Stmt::If { cond, then_b, else_b } => {
                collect_data_addrs_expr(cond, acc);
                collect_data_addrs(then_b, acc);
                collect_data_addrs(else_b, acc);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                collect_data_addrs(body, acc)
            }
            Stmt::BrIf { cond, .. } => collect_data_addrs_expr(cond, acc),
            Stmt::BrTable { index, .. } => collect_data_addrs_expr(index, acc),
            Stmt::Return { values } => {
                for v in values {
                    collect_data_addrs_expr(v, acc);
                }
            }
            _ => {}
        }
    }
}

fn collect_tmps(stmts: &[Stmt], acc: &mut Vec<u32>) {    for s in stmts {
        match s {
            Stmt::Assign { dst, .. } => {
                if let Some(n) = dst.strip_prefix('t').and_then(|x| x.parse::<u32>().ok()) {
                    acc.push(n);
                }
            }
            Stmt::If { then_b, else_b, .. } => {
                collect_tmps(then_b, acc);
                collect_tmps(else_b, acc);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => collect_tmps(body, acc),
            _ => {}
        }
    }
}
