use std::collections::HashMap;

use anyhow::Result;
use wasmparser::{FunctionBody, Operator};

use crate::ir::{Expr, FuncIR, MemAccess, ModuleIR, Stmt};
use crate::parse::{func_display_name, ModuleMeta};

struct Frame {
    kind: FrameKind,
    label: String,
    then_stmts: Vec<Stmt>,
    else_stmts: Vec<Stmt>,
    in_else: bool,
    entry_depth: usize,
    results: usize, // block result arity (0/1 supported)
    /// Merge tmp for `br`-carried block results (Block with results==1).
    /// `br`/`br_if`/`br_table` targeting this frame assign the carried
    /// value here; `End` merges the fallthrough value the same way.
    /// (Loops need none: `br`-to-loop carries params, i.e. 0 for MVP.)
    phi: Option<u32>,
    /// Set when any branch assigned `phi` (so a diverged fallthrough can
    /// still yield the branch value instead of an Unknown pad).
    phi_assigned: bool,
}

#[derive(PartialEq, Eq)]
enum FrameKind {
    Block,
    Loop,
    If,
    /// try/try_table: End closes like Block; Catch switches to handler arm.
    Try,
}

fn cur_stmts<'a>(frames: &'a mut Vec<Frame>, top: &'a mut Vec<Stmt>) -> &'a mut Vec<Stmt> {
    if let Some(f) = frames.last_mut() {
        if (f.kind == FrameKind::If || f.kind == FrameKind::Try) && f.in_else {
            &mut f.else_stmts
        } else {
            &mut f.then_stmts
        }
    } else {
        top
    }
}

/// Best-effort wat-style mnemonic for honest fallback rendering:
/// `I8x16Add` -> `i8x16.add`, `MemoryAtomicWait32` -> `memory.atomic.wait32`.
/// Only used on generic-fallback (SIMD/atomic/GC) paths, not hot MVP ops.
fn op_mnemonic(op: &Operator) -> String {
    let dbg = format!("{op:?}");
    let name = dbg.split([' ', '{', '(']).next().unwrap_or("unknown");
    let chars: Vec<char> = name.chars().collect();
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    for (i, c) in chars.iter().enumerate() {
        let next_is_lower = chars.get(i + 1).map(|n| n.is_lowercase()).unwrap_or(false);
        let prev_is_lower_or_digit = i > 0
            && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
        let is_last = i + 1 == chars.len();
        // split before Uppercase that starts a new word: either followed by
        // lowercase (Add, Lane) or following lower/digit (Convert|I32, Gt|S).
        if c.is_uppercase() && !cur.is_empty() && cur.len() > 1 && (next_is_lower || prev_is_lower_or_digit || is_last) {
            // avoid splitting a lone leading capital run like "V128" internals:
            // only split when the boundary is real (next-lower) or at a
            // lower/digit->Upper transition that isn't mid-acronym.
            let prev_is_upper = i > 0 && chars[i - 1].is_uppercase();
            if next_is_lower || !prev_is_upper {
                parts.push(std::mem::take(&mut cur));
            }
        }
        cur.push(c.to_lowercase().next().unwrap());
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    parts.join(".")
}

/// Pop N values, push `m` generic `op(args)` results (m>1 suffixes outputs).
fn simd_pop_push(op: &Operator, stack: &mut Vec<Expr>, n: usize, m: usize) {
    let args = pop_n_or_unknown(stack, n);
    let name = op_mnemonic(op);
    if m <= 1 {
        stack.push(Expr::Simd { op: name, args });
    } else {
        for (i, suffix) in (["lo", "hi", "c", "d"]).iter().enumerate().take(m) {
            stack.push(Expr::Simd {
                op: format!("{name}#{suffix}"),
                args: args.clone(),
            });
            let _ = i;
        }
    }
}

fn simd_stmt(op: &Operator, stack: &mut Vec<Expr>, frames: &mut Vec<Frame>, top: &mut Vec<Stmt>, n: usize) {
    let args = pop_n_or_unknown(stack, n);
    let name = op_mnemonic(op);
    cur_stmts(frames, top).push(Stmt::ExprStmt(Expr::Simd { op: name, args }));
}

fn block_results(blockty: &wasmparser::BlockType) -> usize {
    match blockty {
        wasmparser::BlockType::Empty => 0,
        wasmparser::BlockType::Type(_) => 1,
        wasmparser::BlockType::FuncType(_) => 1, // MVP: assume <=1; multi-value lowered to 1 + comment
    }
}

fn pop_or_unknown(stack: &mut Vec<Expr>) -> Expr {
    stack.pop().unwrap_or(Expr::Unknown("stack-underflow".into()))
}

fn pop_n_or_unknown(stack: &mut Vec<Expr>, n: usize) -> Vec<Expr> {
    let mut v = Vec::new();
    for _ in 0..n {
        v.push(pop_or_unknown(stack));
    }
    v.reverse();
    v
}

fn binop(op: &str, stack: &mut Vec<Expr>) {
    // NB: move (don't clone) — cloning subtrees here is quadratic on
    // long expression chains and hangs real-world modules.
    let rhs = pop_or_unknown(stack);
    let lhs = pop_or_unknown(stack);
    stack.push(Expr::Binop {
        op: op.to_string(),
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    });
}

fn unop(op: &str, stack: &mut Vec<Expr>) {
    let a = pop_or_unknown(stack);
    stack.push(Expr::Unop {
        op: op.to_string(),
        v: Box::new(a),
    });
}

fn base_desc(e: &Expr) -> String {
    // Iterative + depth-capped: expression trees can be O(function size)
    // deep on machine-generated modules; a recursive full walk per
    // load/store is quadratic and overflows the stack. The (+ const)
    // unwrap this exists for is shallow in practice.
    let mut cur = e;
    for _ in 0..64 {
        match cur {
            Expr::Local(i) => return format!("l{i}"),
            Expr::Tmp(i) => return format!("t{i}"),
            Expr::Global(i) => return format!("g{i}"),
            Expr::Binop { lhs, rhs, op } if op == "+" => {
                if matches!(**rhs, Expr::ConstI32(_) | Expr::ConstI64(_)) {
                    cur = lhs;
                    continue;
                }
                if matches!(**lhs, Expr::ConstI32(_) | Expr::ConstI64(_)) {
                    cur = rhs;
                    continue;
                }
                return "expr".to_string();
            }
            _ => return "expr".to_string(),
        }
    }
    "expr".to_string()
}

pub fn lift_module(bytes: &[u8], meta: &mut ModuleMeta) -> Result<ModuleIR> {
    let bodies = collect_bodies(bytes)?;
    let import_funcs = meta.import_func_count;
    let wanted: Vec<u32> = (0..bodies.len())
        .map(|i| import_funcs + i as u32)
        .collect();
    let mut funcs = Vec::new();
    for (i, body) in bodies.into_iter().enumerate() {
        let idx = import_funcs + i as u32;
        debug_assert_eq!(idx, wanted[i]);
        let f = lift_function(idx, &body, meta)?;
        funcs.push(f);
    }
    finish_module(meta, funcs)
}

/// Lift only the requested function indices (fast single-function queries
/// on huge modules; bodies are ordered `body[i] == func import_count + i`).
/// Call/table metadata still comes from the full module meta.
pub fn lift_selected(
    bodies: &[FunctionBody],
    meta: &mut ModuleMeta,
    wanted: &[u32],
) -> Result<ModuleIR> {
    let import_funcs = meta.import_func_count;
    let mut funcs = Vec::new();
    for &idx in wanted {
        let bi = (idx - import_funcs) as usize;
        if let Some(body) = bodies.get(bi) {
            funcs.push(lift_function(idx, body, meta)?);
        }
    }
    finish_module(meta, funcs)
}

/// Collect code bodies without lifting (for selective queries).
pub fn collect_bodies(bytes: &[u8]) -> Result<Vec<FunctionBody<'_>>> {
    let mut bodies = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        let payload = payload?;
        if let wasmparser::Payload::CodeSectionEntry(body) = payload {
            bodies.push(body);
        }
    }
    Ok(bodies)
}

fn finish_module(meta: &mut ModuleMeta, funcs: Vec<FuncIR>) -> Result<ModuleIR> {
    Ok(ModuleIR {
        funcs,
        tables: meta.tables.clone(),
        memories: meta.memories,
        // Move (don't clone) data bytes out of meta (perf R6): the old
        // `b.clone()` kept full rodata resident 2× (meta + ModuleIR).
        data: std::mem::take(&mut meta.data)
            .into_iter()
            .enumerate()
            .map(|(i, (off, b))| crate::ir::DataSeg { idx: i as u32, offset: off, bytes: b })
            .collect(),
        globals: meta
            .globals
            .iter()
            .enumerate()
            .map(|(i, (t, m))| crate::ir::GlobalDesc {
                idx: i as u32,
                ty: t.clone(),
                mutable: *m,
            })
            .collect(),
        imports: {
            let mut out = Vec::new();
            for (fi, nm) in meta.import_func_names.iter().enumerate() {
                let (p, r) = meta.func_sig(fi as u32);
                let (m, n) = meta
                    .imports
                    .iter()
                    .filter(|(_, _, k)| k == "func")
                    .nth(fi)
                    .map(|(m, n, _)| (m.clone(), n.clone()))
                    .unwrap_or((String::new(), nm.clone()));
                out.push(crate::ir::ImportDesc {
                    module: m,
                    name: nm.clone(),
                    kind: "func".to_string(),
                    params: p,
                    results: r,
                });
                let _ = n;
            }
            // non-func imports for reference
            for (m, n, k) in &meta.imports {
                if k != "func" {
                    out.push(crate::ir::ImportDesc {
                        module: m.clone(),
                        name: n.clone(),
                        kind: k.clone(),
                        params: Vec::new(),
                        results: Vec::new(),
                    });
                }
            }
            out
        },
    })
}

fn lift_function(idx: u32, body: &FunctionBody, meta: &ModuleMeta) -> Result<FuncIR> {
    // locals
    let mut locals: Vec<crate::ir::WasmTy> = Vec::new();
    {
        let mut lr = body.get_locals_reader()?;
        for _ in 0..lr.get_count() {
            let (count, ty) = lr.read()?;
            let t = match ty {
                wasmparser::ValType::I32 => crate::ir::WasmTy::I32,
                wasmparser::ValType::I64 => crate::ir::WasmTy::I64,
                wasmparser::ValType::F32 => crate::ir::WasmTy::F32,
                wasmparser::ValType::F64 => crate::ir::WasmTy::F64,
                wasmparser::ValType::V128 => crate::ir::WasmTy::V128,
                wasmparser::ValType::Ref(r) => {
                    if r.is_func_ref() {
                        crate::ir::WasmTy::FuncRef
                    } else {
                        crate::ir::WasmTy::ExternRef
                    }
                }
            };
            for _ in 0..count {
                locals.push(t.clone());
            }
        }
    }
    let (params, results) = meta.func_sig(idx);

    let name = func_display_name(idx, meta);
    let mut stack: Vec<Expr> = Vec::new();
    let mut top: Vec<Stmt> = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut tmp: u32 = 0;
    let mut label_seq: u32 = 0;
    let mut accesses: Vec<MemAccess> = Vec::new();
    let mut calls: Vec<u32> = Vec::new();
    let mut indirects: Vec<Option<u32>> = Vec::new();

    // local const tracking for call_indirect resolution (single const assign)
    let mut local_const: std::collections::HashMap<u32, Option<i32>> = std::collections::HashMap::new();
    let mut cond_stack: Vec<Expr> = Vec::new();
    let mut then_results: Vec<Expr> = Vec::new();
    let mut try_handler: Option<(String, Vec<Stmt>)> = None;

    let mut ops = body.get_operators_reader()?;
    while !ops.eof() {
        let op = ops.read()?;
        match op {
            Operator::Unreachable => {
                // Trap: terminate flow. Clear the value stack to the current
                // frame entry so dead followers don't leak into tmps/returns.
                let entry = frames.last().map(|f| f.entry_depth).unwrap_or(0);
                stack.truncate(entry);
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment("unreachable (trap)".into()));
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::ExprStmt(Expr::Raw("__builtin_trap()".into())));
            }
            Operator::Nop => {}
            Operator::Block { blockty } => {
                let r = block_results(&blockty);
                label_seq += 1;
                // Result-carrying blocks merge `br` values through a phi tmp.
                let phi = if r == 1 {
                    let t = tmp;
                    tmp += 1;
                    Some(t)
                } else {
                    None
                };
                frames.push(Frame {
                    kind: FrameKind::Block,
                    label: format!("B{label_seq}"),
                    then_stmts: Vec::new(),
                    else_stmts: Vec::new(),
                    in_else: false,
                    entry_depth: stack.len(),
                    results: r,
                    phi,
                    phi_assigned: false,
                });
                if r > 1 {
                    cur_stmts(&mut frames, &mut top).push(Stmt::Comment(
                        "multi-value block: only first result modeled".into(),
                    ));
                }
            }
            Operator::Loop { blockty } => {
                let r = block_results(&blockty);
                label_seq += 1;
                frames.push(Frame {
                    kind: FrameKind::Loop,
                    label: format!("L{label_seq}"),
                    then_stmts: Vec::new(),
                    else_stmts: Vec::new(),
                    in_else: false,
                    entry_depth: stack.len(),
                    results: r,
                    // No phi: br-to-loop carries params (0 for MVP);
                    // the loop result comes from fallthrough at End.
                    phi: None,
                    phi_assigned: false,
                });
            }
            Operator::If { blockty } => {
                let r = block_results(&blockty);
                let cond = pop_or_unknown(&mut stack);
                label_seq += 1;
                frames.push(Frame {
                    kind: FrameKind::If,
                    label: format!("I{label_seq}"),
                    then_stmts: Vec::new(),
                    else_stmts: Vec::new(),
                    in_else: false,
                    entry_depth: stack.len(),
                    results: r,
                    // If-results use the then_results merge at End, not phi.
                    phi: None,
                    phi_assigned: false,
                });
                // stash cond for End handling
                cond_stack.push(cond);
            }
            Operator::Else => {
                if let Some(f) = frames.last_mut() {
                    assert!(
                        f.kind == FrameKind::If || f.kind == FrameKind::Try,
                        "else without if/try"
                    );
                    // capture then-result if any
                    if f.results == 1 {
                        // then branch should have pushed 1 value; pop it aside
                        let v = pop_or_unknown(&mut stack);
                        then_results.push(v);
                        // reset stack to entry depth for else branch
                        while stack.len() > f.entry_depth {
                            stack.pop();
                        }
                    }
                    f.in_else = true;
                }
            }
            Operator::End => {
                let Some(f) = frames.pop() else {
                    // function-body terminator End (no open block) — ignore
                    continue;
                };
                let stmt = match f.kind {
                    FrameKind::Block => {
                        // Merge `br`-carried + fallthrough values through phi:
                        // every incoming edge assigned t<phi>; the fallthrough
                        // assigns it here; End pushes the merged tmp.
                        if f.results == 1 {
                            if let Some(p) = f.phi {
                                if stack.len() > f.entry_depth {
                                    let fall = pop_or_unknown(&mut stack);
                                    while stack.len() > f.entry_depth {
                                        stack.pop();
                                    }
                                    let mut body = f.then_stmts;
                                    body.push(Stmt::Assign {
                                        dst: format!("t{p}"),
                                        expr: fall,
                                    });
                                    stack.push(Expr::Tmp(p));
                                    Stmt::Block { label: f.label, body }
                                } else if f.phi_assigned {
                                    stack.push(Expr::Tmp(p));
                                    Stmt::Block { label: f.label, body: f.then_stmts }
                                } else {
                                    // Diverged bodies (unreachable on all paths)
                                    // leave no result: pad honestly.
                                    while stack.len() < f.entry_depth + 1 {
                                        stack.push(Expr::Unknown("diverged-block-result".into()));
                                    }
                                    Stmt::Block { label: f.label, body: f.then_stmts }
                                }
                            } else {
                                // results>1 (multi-value): only first modeled.
                                while stack.len() < f.entry_depth + 1 {
                                    stack.push(Expr::Unknown("diverged-block-result".into()));
                                }
                                Stmt::Block { label: f.label, body: f.then_stmts }
                            }
                        } else {
                            // No result: leave the stack as-is (balanced code
                            // is exactly at entry depth here).
                            Stmt::Block {
                                label: f.label,
                                body: f.then_stmts,
                            }
                        }
                    }
                    FrameKind::Try => {
                        // try body as one block; handler arm (catch) as a second block
                        let handler = if f.in_else { Some(f.else_stmts) } else { None };
                        // stash handler for emission after the main push below
                        try_handler = handler.map(|h| (f.label.clone(), h));
                        while stack.len() > f.entry_depth + f.results.min(1) {
                            stack.pop();
                        }
                        while stack.len() < f.entry_depth + f.results.min(1) {
                            stack.push(Expr::Unknown("diverged-block-result".into()));
                        }
                        Stmt::Block {
                            label: f.label,
                            body: f.then_stmts,
                        }
                    }
                    FrameKind::Loop => {
                        while stack.len() < f.entry_depth + f.results.min(1) {
                            stack.push(Expr::Unknown("diverged-block-result".into()));
                        }
                        Stmt::Loop {
                            label: f.label,
                            body: f.then_stmts,
                        }
                    }
                    FrameKind::If => {
                        let cond = cond_stack.pop().unwrap_or(Expr::Unknown("if-cond".into()));
                        let mut then_b = f.then_stmts;
                        let else_b = if f.in_else { f.else_stmts } else { Vec::new() };
                        if f.results == 1 {
                            if f.in_else {
                                let else_v = pop_or_unknown(&mut stack);
                                while stack.len() > f.entry_depth {
                                    stack.pop();
                                }
                                let then_v = then_results.pop().unwrap_or(Expr::Unknown("then".into()));
                                // assign phi tmp in both branches
                                let t = tmp;
                                tmp += 1;
                                then_b.push(Stmt::Assign {
                                    dst: format!("t{t}"),
                                    expr: then_v,
                                });
                                let mut else_b2 = else_b;
                                else_b2.push(Stmt::Assign {
                                    dst: format!("t{t}"),
                                    expr: else_v,
                                });
                                stack.push(Expr::Tmp(t));
                                Stmt::If {
                                    cond,
                                    then_b,
                                    else_b: else_b2,
                                }
                            } else {
                                // no else: then-result + implicit else (keep entry value?)
                                let then_v = pop_or_unknown(&mut stack);
                                while stack.len() > f.entry_depth {
                                    stack.pop();
                                }
                                let t = tmp;
                                tmp += 1;
                                then_b.push(Stmt::Assign {
                                    dst: format!("t{t}"),
                                    expr: then_v,
                                });
                                stack.push(Expr::Tmp(t));
                                Stmt::If {
                                    cond,
                                    then_b,
                                    else_b: Vec::new(),
                                }
                            }
                        } else {
                            // ensure stack restored to entry depth (drop stray branch values)
                            while stack.len() > f.entry_depth {
                                stack.pop();
                            }
                            Stmt::If {
                                cond,
                                then_b,
                                else_b,
                            }
                        }
                    }
                };
                cur_stmts(&mut frames, &mut top).push(stmt);
                // second block for try/catch handler arm, if any
                if let Some((label, body)) = try_handler.take() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::Comment(
                        "catch handler (exceptional edge; normal flow falls through)".into(),
                    ));
                    cur_stmts(&mut frames, &mut top).push(Stmt::Block {
                        label: format!("{label}-catch"),
                        body,
                    });
                }
            }
            Operator::Br { relative_depth } => {
                let (label, is_loop, _) = resolve_target(&frames, relative_depth);
                if label == "func_end" {
                    // Top-level `br` exits the function: lower to return.
                    let mut vals = Vec::new();
                    for _ in 0..results.len() {
                        vals.push(pop_or_unknown(&mut stack));
                    }
                    vals.reverse();
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return { values: vals });
                    stack.clear();
                    continue;
                }
                // `br` to a result-block carries values: merge through phi.
                let phi = frames
                    .iter()
                    .rev()
                    .nth(relative_depth as usize)
                    .and_then(|f| f.phi);
                if let Some(p) = phi {
                    let v = pop_or_unknown(&mut stack);
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{p}"),
                        expr: v,
                    });
                    if let Some(f) = frames.iter_mut().rev().nth(relative_depth as usize) {
                        f.phi_assigned = true;
                    }
                }
                // Unconditional transfer: fallthrough code up to End is dead.
                // Reset to the CURRENT frame entry (not the target's): the
                // target path is gone, and dead fallthrough must not inherit
                // outer values nor starve inner ones (both would mislead).
                if let Some(f) = frames.last() {
                    let entry = f.entry_depth;
                    stack.truncate(entry.min(stack.len()));
                }
                cur_stmts(&mut frames, &mut top).push(Stmt::Br {
                    depth: relative_depth,
                    label,
                    is_loop,
                });
            }
            Operator::BrIf { relative_depth } => {
                let c = pop_or_unknown(&mut stack);
                let (label, is_loop, _entry) = resolve_target(&frames, relative_depth);
                if label == "func_end" {
                    // Conditional function exit: `if (c) return vals`.
                    let mut vals = Vec::new();
                    for _ in 0..results.len() {
                        vals.push(pop_or_unknown(&mut stack));
                    }
                    vals.reverse();
                    cur_stmts(&mut frames, &mut top).push(Stmt::If {
                        cond: c,
                        then_b: vec![Stmt::Return { values: vals }],
                        else_b: Vec::new(),
                    });
                    // fallthrough keeps stack as-is (minus cond+results)
                    continue;
                }
                let phi = frames
                    .iter()
                    .rev()
                    .nth(relative_depth as usize)
                    .and_then(|f| f.phi);
                if let Some(p) = phi {
                    // Conditional merge: assign phi only on the taken path.
                    let v = pop_or_unknown(&mut stack);
                    if let Some(f) = frames.iter_mut().rev().nth(relative_depth as usize) {
                        f.phi_assigned = true;
                    }
                    cur_stmts(&mut frames, &mut top).push(Stmt::If {
                        cond: c,
                        then_b: vec![
                            Stmt::Assign {
                                dst: format!("t{p}"),
                                expr: v,
                            },
                            Stmt::Br {
                                depth: relative_depth,
                                label,
                                is_loop,
                            },
                        ],
                        else_b: Vec::new(),
                    });
                    continue;
                }
                // br_if does not unwind on fallthrough; keep stack as-is
                cur_stmts(&mut frames, &mut top).push(Stmt::BrIf {
                    depth: relative_depth,
                    label,
                    is_loop,
                    cond: c,
                });
            }
            Operator::BrTable { targets } => {
                let index = pop_or_unknown(&mut stack);
                let mut tvec: Vec<(u32, String, bool)> = Vec::new();
                for t in targets.targets() {
                    let d = t.unwrap_or(0);
                    let (label, is_loop, _) = resolve_target(&frames, d);
                    tvec.push((d, label, is_loop));
                }
                let def = targets.default();
                let (dlabel, dis_loop, _) = resolve_target(&frames, def);
                // Result-carrying br_table: every target with a phi merges
                // the same carried value. The transfer is unconditional, so
                // pre-assigning all phis before the switch is sound (exactly
                // one target is taken; dead fallthrough never reads them).
                let any_phi = tvec
                    .iter()
                    .map(|(d, _, _)| *d)
                    .chain(std::iter::once(def))
                    .any(|d| {
                        frames.iter().rev().nth(d as usize).and_then(|f| f.phi).is_some()
                    });
                let any_func_end = tvec.iter().any(|(_, l, _)| l == "func_end") || dlabel == "func_end";
                if any_phi || (any_func_end && !results.is_empty()) {
                    let v = pop_or_unknown(&mut stack);
                    if any_func_end {
                        cur_stmts(&mut frames, &mut top).push(Stmt::Comment(
                            "br_table targets function end with values: per-target returns not merged; fallthrough value used"
                                .into(),
                        ));
                    }
                    // Collect target depths carrying phis first (borrow), then assign.
                    let mut phi_depths = Vec::new();
                    for (d, _, _) in &tvec {
                        if frames.iter().rev().nth(*d as usize).and_then(|f| f.phi).is_some() {
                            phi_depths.push(*d);
                        }
                    }
                    if frames.iter().rev().nth(def as usize).and_then(|f| f.phi).is_some() {
                        phi_depths.push(def);
                    }
                    for d in phi_depths {
                        let p = frames.iter().rev().nth(d as usize).and_then(|f| f.phi).unwrap();
                        cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                            dst: format!("t{p}"),
                            expr: v.clone(),
                        });
                        if let Some(f) = frames.iter_mut().rev().nth(d as usize) {
                            f.phi_assigned = true;
                        }
                    }
                    if !any_phi {
                        // Values target only func_end (unmergable): keep the
                        // stack honest by dropping the carried value openly.
                        cur_stmts(&mut frames, &mut top).push(Stmt::Comment(
                            "br_table value to function end dropped (unmerged)".into(),
                        ));
                    }
                    let _ = v;
                }
                // Same dead-fallthrough rule as Br: reset to current entry.
                if let Some(f) = frames.last() {
                    let entry = f.entry_depth;
                    stack.truncate(entry.min(stack.len()));
                }
                cur_stmts(&mut frames, &mut top).push(Stmt::BrTable {
                    index,
                    targets: tvec,
                    default: (def, dlabel, dis_loop),
                });
            }
            Operator::Return => {
                // pop results
                let mut vals = Vec::new();
                for _ in 0..results.len() {
                    vals.push(pop_or_unknown(&mut stack));
                }
                vals.reverse();
                cur_stmts(&mut frames, &mut top).push(Stmt::Return { values: vals });
                // code after return (up to End) is dead: drop leftovers so
                // they don't leak into tmps or later underflows.
                stack.clear();
            }
            Operator::Call { function_index } => {
                let (p, _r) = meta.func_sig(function_index);
                let args = pop_n_or_unknown(&mut stack, p.len());
                calls.push(function_index);
                let nr = meta.func_sig(function_index).1.len();
                let cname = meta.call_name(function_index);
                let dst = if nr == 0 {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Call {
                        func: function_index,
                        name: cname,
                        args,
                    }));
                    None
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: Expr::Call {
                            func: function_index,
                            name: cname,
                            args,
                        },
                    });
                    // push extra unknowns for multi-result beyond first
                    for _ in 1..nr {
                        let t2 = tmp;
                        tmp += 1;
                        cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                            "multi-result call: spill t{t2}"
                        )));
                        stack.push(Expr::Tmp(t2));
                    }
                    Some(t)
                };
                if let Some(t) = dst {
                    stack.push(Expr::Tmp(t));
                }
            }
            Operator::CallIndirect { type_index, table_index, .. } => {
                let (p, r) = meta
                    .types
                    .get(type_index as usize)
                    .cloned()
                    .unwrap_or((Vec::new(), Vec::new()));
                // Stack order: [args..., index] with index on top —
                // pop index FIRST, then args (popping args first swaps them).
                let index = pop_or_unknown(&mut stack);
                let args = pop_n_or_unknown(&mut stack, p.len());
                // resolve if const
                let resolved = index.const_i32().and_then(|v| {
                    if v < 0 {
                        return None;
                    }
                    meta.tables
                        .get(table_index as usize)
                        .and_then(|t| t.get(v as usize).cloned().flatten())
                });
                // also try local const: if index is Local(i) with known const
                let resolved2 = match &index {
                    Expr::Local(li) => local_const.get(li).cloned().flatten().and_then(|v| {
                        if v < 0 {
                            return None;
                        }
                        meta.tables
                            .get(table_index as usize)
                            .and_then(|t| t.get(v as usize).cloned().flatten())
                    }),
                    _ => None,
                };
                indirects.push(resolved.or(resolved2));
                let e = Expr::CallIndirect {
                    type_idx: type_index,
                    table: table_index,
                    index: Box::new(index),
                    args,
                };
                if r.is_empty() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(e));
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: e,
                    });
                    stack.push(Expr::Tmp(t));
                }
            }
            Operator::Drop => {
                let _ = pop_or_unknown(&mut stack);
            }
            Operator::Select | Operator::TypedSelect { .. } => {
                let c = pop_or_unknown(&mut stack);
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                stack.push(Expr::Select {
                    c: Box::new(c),
                    a: Box::new(a),
                    b: Box::new(b),
                });
            }
            Operator::LocalGet { local_index } => stack.push(Expr::Local(local_index)),
            Operator::LocalSet { local_index } => {
                let v = pop_or_unknown(&mut stack);
                if let Expr::ConstI32(c) = &v {
                    local_const.insert(local_index, Some(*c));
                } else {
                    local_const.insert(local_index, None);
                }
                cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                    dst: format!("l{local_index}"),
                    expr: v,
                });
            }
            Operator::LocalTee { local_index } => {
                let v = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                    dst: format!("l{local_index}"),
                    expr: v,
                });
                stack.push(Expr::Local(local_index));
            }
            Operator::GlobalGet { global_index } => stack.push(Expr::Global(global_index)),
            Operator::GlobalSet { global_index } => {
                let v = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                    dst: format!("g{global_index}"),
                    expr: v,
                });
            }
            // consts
            Operator::I32Const { value } => stack.push(Expr::ConstI32(value)),
            Operator::I64Const { value } => stack.push(Expr::ConstI64(value)),
            Operator::F32Const { value } => stack.push(Expr::ConstF32(value.bits())),
            Operator::F64Const { value } => stack.push(Expr::ConstF64(value.bits())),
            // integer arithmetic (representative set; rest via generic fallback below)
            Operator::I32Eqz => unop("!", &mut stack),
            Operator::I32Eq => binop("==", &mut stack),
            Operator::I32Ne => binop("!=", &mut stack),
            Operator::I32LtS => binop("<", &mut stack),
            Operator::I32LtU => binop("<u32", &mut stack),
            Operator::I32GtS => binop(">", &mut stack),
            Operator::I32GtU => binop(">u32", &mut stack),
            Operator::I32LeS => binop("<=", &mut stack),
            Operator::I32LeU => binop("<=u32", &mut stack),
            Operator::I32GeS => binop(">=", &mut stack),
            Operator::I32GeU => binop(">=u32", &mut stack),
            Operator::I64Eqz => unop("!", &mut stack),
            Operator::I64Eq => binop("==", &mut stack),
            Operator::I64Ne => binop("!=", &mut stack),
            Operator::I64LtS => binop("<", &mut stack),
            Operator::I64LtU => binop("<u64", &mut stack),
            Operator::I64GtS => binop(">", &mut stack),
            Operator::I64GtU => binop(">u64", &mut stack),
            Operator::I64LeS => binop("<=", &mut stack),
            Operator::I64LeU => binop("<=u64", &mut stack),
            Operator::I64GeS => binop(">=", &mut stack),
            Operator::I64GeU => binop(">=u64", &mut stack),
            Operator::F32Eq => binop("==", &mut stack),
            Operator::F32Ne => binop("!=", &mut stack),
            Operator::F32Lt => binop("<", &mut stack),
            Operator::F32Gt => binop(">", &mut stack),
            Operator::F32Le => binop("<=", &mut stack),
            Operator::F32Ge => binop(">=", &mut stack),
            Operator::F64Eq => binop("==", &mut stack),
            Operator::F64Ne => binop("!=", &mut stack),
            Operator::F64Lt => binop("<", &mut stack),
            Operator::F64Gt => binop(">", &mut stack),
            Operator::F64Le => binop("<=", &mut stack),
            Operator::F64Ge => binop(">=", &mut stack),
            Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt
            | Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![v],
                });
            }
            Operator::I32Add => {
                // const-fold i32.add of two consts (feeds indirect resolution + readability)
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                match (a, b) {
                    (Expr::ConstI32(x), Expr::ConstI32(y)) => {
                        stack.push(Expr::ConstI32(x.wrapping_add(y)))
                    }
                    (x, y) => stack.push(Expr::Binop {
                        op: "+".into(),
                        lhs: Box::new(x),
                        rhs: Box::new(y),
                    }),
                }
            }
            Operator::I32Sub => binop("-", &mut stack),
            Operator::I32Mul => binop("*", &mut stack),
            Operator::I32DivS => binop("/", &mut stack),
            Operator::I32DivU => binop("/u32", &mut stack),
            Operator::I32RemS => binop("%", &mut stack),
            Operator::I32RemU => binop("%u32", &mut stack),
            Operator::I32And => binop("&", &mut stack),
            Operator::I32Or => binop("|", &mut stack),
            Operator::I32Xor => binop("^", &mut stack),
            Operator::I32Shl => binop("<<", &mut stack),
            Operator::I32ShrS => binop(">>", &mut stack),
            Operator::I32ShrU => binop(">>u32", &mut stack),
            Operator::I32Rotl | Operator::I32Rotr => {
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![a, b],
                });
            }
            Operator::I64Add => binop("+", &mut stack),
            Operator::I64Sub => binop("-", &mut stack),
            Operator::I64Mul => binop("*", &mut stack),
            Operator::I64DivS => binop("/", &mut stack),
            Operator::I64DivU => binop("/u64", &mut stack),
            Operator::I64RemS => binop("%", &mut stack),
            Operator::I64RemU => binop("%u64", &mut stack),
            Operator::I64And => binop("&", &mut stack),
            Operator::I64Or => binop("|", &mut stack),
            Operator::I64Xor => binop("^", &mut stack),
            Operator::I64Shl => binop("<<", &mut stack),
            Operator::I64ShrS => binop(">>", &mut stack),
            Operator::I64ShrU => binop(">>u64", &mut stack),
            Operator::I64Rotl | Operator::I64Rotr => {
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![a, b],
                });
            }
            Operator::F32Add | Operator::F64Add => binop("+", &mut stack),
            Operator::F32Sub | Operator::F64Sub => binop("-", &mut stack),
            Operator::F32Mul | Operator::F64Mul => binop("*", &mut stack),
            Operator::F32Div | Operator::F64Div => binop("/", &mut stack),
            Operator::F32Min | Operator::F64Min => binop("min", &mut stack),
            Operator::F32Max | Operator::F64Max => binop("max", &mut stack),
            Operator::I32WrapI64 => unop("(int32_t)", &mut stack),
            Operator::I64ExtendI32S => {
                unop("(int32_t)", &mut stack);
                unop("(int64_t)", &mut stack);
            }
            Operator::I64ExtendI32U => {
                unop("(uint32_t)", &mut stack);
                unop("(int64_t)", &mut stack);
            }
            // loads (signedness preserved: S vs U matters for <8/4-byte
            // loads — zero- vs sign-extension changes the value)
            Operator::I32Load { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 4,
                    is_write: false,
                    dom_ty: "i32".into(),
                });
                stack.push(Expr::Load {
                    ty: "i32".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I32Load8S { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 1,
                    is_write: false,
                    dom_ty: "i32.s8".into(),
                });
                stack.push(Expr::Load {
                    ty: "i32/s1".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I32Load8U { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 1,
                    is_write: false,
                    dom_ty: "i32.u8".into(),
                });
                stack.push(Expr::Load {
                    ty: "i32/u1".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I32Load16S { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 2,
                    is_write: false,
                    dom_ty: "i32.s16".into(),
                });
                stack.push(Expr::Load {
                    ty: "i32/s2".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I32Load16U { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 2,
                    is_write: false,
                    dom_ty: "i32.u16".into(),
                });
                stack.push(Expr::Load {
                    ty: "i32/u2".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 8,
                    is_write: false,
                    dom_ty: "i64".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load8S { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 1,
                    is_write: false,
                    dom_ty: "i64.s8".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/s1".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load8U { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 1,
                    is_write: false,
                    dom_ty: "i64.u8".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/u1".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load16S { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 2,
                    is_write: false,
                    dom_ty: "i64.s16".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/s2".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load16U { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 2,
                    is_write: false,
                    dom_ty: "i64.u16".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/u2".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load32S { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 4,
                    is_write: false,
                    dom_ty: "i64.s32".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/s4".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::I64Load32U { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 4,
                    is_write: false,
                    dom_ty: "i64.u32".into(),
                });
                stack.push(Expr::Load {
                    ty: "i64/u4".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::F32Load { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 4,
                    is_write: false,
                    dom_ty: "f32".into(),
                });
                stack.push(Expr::Load {
                    ty: "f32".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            Operator::F64Load { memarg } => {
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 8,
                    is_write: false,
                    dom_ty: "f64".into(),
                });
                stack.push(Expr::Load {
                    ty: "f64".into(),
                    base: Box::new(base),
                    offset: memarg.offset,
                });
            }
            // stores
            Operator::I32Store { memarg }
            | Operator::I32Store8 { memarg }
            | Operator::I32Store16 { memarg } => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let w = store_width(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: w,
                    is_write: true,
                    dom_ty: "i32".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: format!("i32/w{w}"),
                    base,
                    offset: memarg.offset,
                    value: val,
                });
            }
            Operator::I64Store { memarg }
            | Operator::I64Store8 { memarg }
            | Operator::I64Store16 { memarg }
            | Operator::I64Store32 { memarg } => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let w = store_width(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: w,
                    is_write: true,
                    dom_ty: "i64".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: format!("i64/w{w}"),
                    base,
                    offset: memarg.offset,
                    value: val,
                });
            }
            Operator::F32Store { memarg } => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 4,
                    is_write: true,
                    dom_ty: "f32".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: "f32".into(),
                    base,
                    offset: memarg.offset,
                    value: val,
                });
            }
            Operator::F64Store { memarg } => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 8,
                    is_write: true,
                    dom_ty: "f64".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: "f64".into(),
                    base,
                    offset: memarg.offset,
                    value: val,
                });
            }
            Operator::MemorySize { .. } => stack.push(Expr::MemorySize(0)),
            Operator::MemoryGrow { .. } => {
                let d = pop_or_unknown(&mut stack);
                stack.push(Expr::Unop {
                    op: "memgrow(".into(),
                    v: Box::new(d),
                });
            }
            // numeric conversions with signedness preserved (trunc/convert
            // S vs U differ on large values; reinterpret is bitwise, not numeric)
            Operator::I32TruncF32S | Operator::I32TruncF64S => unop("(int32_t)", &mut stack),
            Operator::I32TruncF32U | Operator::I32TruncF64U => unop("(uint32_t)", &mut stack),
            Operator::I32ReinterpretF32 => {
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd { op: "f32_to_u32".into(), args: vec![v] });
            }
            Operator::I32Extend8S => {
                unop("(int8_t)", &mut stack);
                unop("(int32_t)", &mut stack);
            }
            Operator::I32Extend16S => {
                unop("(int16_t)", &mut stack);
                unop("(int32_t)", &mut stack);
            }
            Operator::I32TruncSatF32S | Operator::I32TruncSatF64S => {
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd { op: name, args: vec![v] });
            }
            Operator::I32TruncSatF32U | Operator::I32TruncSatF64U => {
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd { op: name, args: vec![v] });
            }
            Operator::I64TruncF32S | Operator::I64TruncF64S => unop("(int64_t)", &mut stack),
            Operator::I64TruncF32U | Operator::I64TruncF64U => unop("(uint64_t)", &mut stack),
            Operator::I64ReinterpretF64 => {
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd { op: "f64_to_u64".into(), args: vec![v] });
            }
            Operator::I64Extend8S => {
                unop("(int8_t)", &mut stack);
                unop("(int64_t)", &mut stack);
            }
            Operator::I64Extend16S => {
                unop("(int16_t)", &mut stack);
                unop("(int64_t)", &mut stack);
            }
            Operator::I64Extend32S => {
                unop("(int32_t)", &mut stack);
                unop("(int64_t)", &mut stack);
            }
            Operator::I64TruncSatF32S | Operator::I64TruncSatF64S
            | Operator::I64TruncSatF32U | Operator::I64TruncSatF64U => {
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd { op: name, args: vec![v] });
            }
            Operator::F32ConvertI32S => {
                unop("(int32_t)", &mut stack);
                unop("(float)", &mut stack);
            }
            Operator::F32ConvertI32U => {
                unop("(uint32_t)", &mut stack);
                unop("(float)", &mut stack);
            }
            Operator::F32ConvertI64S => {
                unop("(int64_t)", &mut stack);
                unop("(float)", &mut stack);
            }
            Operator::F32ConvertI64U => {
                unop("(uint64_t)", &mut stack);
                unop("(float)", &mut stack);
            }
            Operator::F32DemoteF64 => unop("(float)", &mut stack),
            Operator::F32ReinterpretI32 => {
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd { op: "u32_to_float".into(), args: vec![v] });
            }
            Operator::F64ConvertI32S => {
                unop("(int32_t)", &mut stack);
                unop("(double)", &mut stack);
            }
            Operator::F64ConvertI32U => {
                unop("(uint32_t)", &mut stack);
                unop("(double)", &mut stack);
            }
            Operator::F64ConvertI64S => {
                unop("(int64_t)", &mut stack);
                unop("(double)", &mut stack);
            }
            Operator::F64ConvertI64U => {
                unop("(uint64_t)", &mut stack);
                unop("(double)", &mut stack);
            }
            Operator::F64PromoteF32 => unop("(double)", &mut stack),
            Operator::F64ReinterpretI64 => {
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd { op: "u64_to_double".into(), args: vec![v] });
            }
            Operator::F32Neg
            | Operator::F32Abs
            | Operator::F32Ceil
            | Operator::F32Floor
            | Operator::F32Trunc
            | Operator::F32Nearest
            | Operator::F32Sqrt => unop("f32op", &mut stack),
            Operator::F64Neg
            | Operator::F64Abs
            | Operator::F64Ceil
            | Operator::F64Floor
            | Operator::F64Trunc
            | Operator::F64Nearest
            | Operator::F64Sqrt => unop("f64op", &mut stack),
            Operator::F32Copysign | Operator::F64Copysign => binop("copysign", &mut stack),
            // ---- bulk memory (exact arities, readable pseudo-calls) ----
            Operator::MemoryInit { data_index, .. } => {
                let len = pop_or_unknown(&mut stack);
                let src = pop_or_unknown(&mut stack);
                let dst = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "memory.init".into(),
                    args: vec![dst, src, len, Expr::ConstI32(data_index as i32)],
                }));
            }
            Operator::MemoryCopy { .. } => {
                let len = pop_or_unknown(&mut stack);
                let src = pop_or_unknown(&mut stack);
                let dst = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "memory.copy".into(),
                    args: vec![dst, src, len],
                }));
            }
            Operator::MemoryFill { .. } => {
                let len = pop_or_unknown(&mut stack);
                let val = pop_or_unknown(&mut stack);
                let dst = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "memory.fill".into(),
                    args: vec![dst, val, len],
                }));
            }
            Operator::DataDrop { data_index } => {
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                    "data.drop {data_index}"
                )));
            }
            Operator::TableInit { elem_index, table } => {
                let len = pop_or_unknown(&mut stack);
                let src = pop_or_unknown(&mut stack);
                let dst = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "table.init".into(),
                    args: vec![
                        dst,
                        src,
                        len,
                        Expr::ConstI32(elem_index as i32),
                        Expr::ConstI32(table as i32),
                    ],
                }));
            }
            Operator::ElemDrop { elem_index } => {
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                    "elem.drop {elem_index}"
                )));
            }
            Operator::TableCopy { .. } => {
                let len = pop_or_unknown(&mut stack);
                let src = pop_or_unknown(&mut stack);
                let dst = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "table.copy".into(),
                    args: vec![dst, src, len],
                }));
            }
            Operator::MemoryDiscard { .. } => {
                let a = pop_or_unknown(&mut stack);
                let b = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "memory.discard".into(),
                    args: vec![b, a],
                }));
            }
            // ---- reference types ----
            Operator::RefNull { .. } => stack.push(Expr::Simd {
                op: "ref.null".into(),
                args: Vec::new(),
            }),
            Operator::RefIsNull => {
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "ref.is_null".into(),
                    args: vec![v],
                });
            }
            Operator::RefFunc { function_index } => stack.push(Expr::Simd {
                op: "ref.func".into(),
                args: vec![Expr::ConstI32(function_index as i32)],
            }),
            Operator::TableGet { table } => {
                let i = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "table.get".into(),
                    args: vec![i, Expr::ConstI32(table as i32)],
                });
            }
            Operator::TableSet { table } => {
                let v = pop_or_unknown(&mut stack);
                let i = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "table.set".into(),
                    args: vec![i, v, Expr::ConstI32(table as i32)],
                }));
            }
            Operator::TableGrow { table } => {
                let n = pop_or_unknown(&mut stack);
                let v = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "table.grow".into(),
                    args: vec![v, n, Expr::ConstI32(table as i32)],
                });
            }
            Operator::TableSize { table } => stack.push(Expr::Simd {
                op: "table.size".into(),
                args: vec![Expr::ConstI32(table as i32)],
            }),
            Operator::TableFill { table } => {
                let n = pop_or_unknown(&mut stack);
                let v = pop_or_unknown(&mut stack);
                let i = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "table.fill".into(),
                    args: vec![i, v, n, Expr::ConstI32(table as i32)],
                }));
            }
            Operator::TypedSelectMulti { .. } => {
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::Comment("multi-result select: first result modeled".into()));
                let c = pop_or_unknown(&mut stack);
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                stack.push(Expr::Select {
                    c: Box::new(c),
                    a: Box::new(a),
                    b: Box::new(b),
                });
            }
            // ---- tail calls: call, then return its results ----
            Operator::ReturnCall { function_index } => {
                let (p, r) = meta.func_sig(function_index);
                let args = pop_n_or_unknown(&mut stack, p.len());
                calls.push(function_index);
                let cname = meta.call_name(function_index);
                if r.is_empty() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Call {
                        func: function_index,
                        name: cname,
                        args,
                    }));
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return { values: Vec::new() });
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: Expr::Call {
                            func: function_index,
                            name: cname,
                            args,
                        },
                    });
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return {
                        values: vec![Expr::Tmp(t)],
                    });
                    stack.push(Expr::Unknown("tail-call-unreachable".into()));
                }
            }
            Operator::ReturnCallIndirect { type_index, table_index } => {
                let (p, r) = meta
                    .types
                    .get(type_index as usize)
                    .cloned()
                    .unwrap_or((Vec::new(), Vec::new()));
                // Same stack order as call_indirect: index on top.
                let index = pop_or_unknown(&mut stack);
                let args = pop_n_or_unknown(&mut stack, p.len());
                indirects.push(index.const_i32().and_then(|v| {
                    (v >= 0).then(|| {
                        meta.tables
                            .get(table_index as usize)
                            .and_then(|t| t.get(v as usize).cloned().flatten())
                    })
                    .flatten()
                }));
                let e = Expr::CallIndirect {
                    type_idx: type_index,
                    table: table_index,
                    index: Box::new(index),
                    args,
                };
                if r.is_empty() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(e));
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return { values: Vec::new() });
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: e,
                    });
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return {
                        values: vec![Expr::Tmp(t)],
                    });
                    stack.push(Expr::Unknown("tail-call-unreachable".into()));
                }
            }
            // ---- function references ----
            Operator::CallRef { type_index } => {
                let (p, r) = meta
                    .types
                    .get(type_index as usize)
                    .cloned()
                    .unwrap_or((Vec::new(), Vec::new()));
                // Stack order: [args..., funcref] with ref on top.
                let fref = pop_or_unknown(&mut stack);
                let args = pop_n_or_unknown(&mut stack, p.len());
                let mut all = vec![fref];
                all.extend(args);
                if r.is_empty() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                        op: "call.ref".into(),
                        args: all,
                    }));
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: Expr::Simd {
                            op: "call.ref".into(),
                            args: all,
                        },
                    });
                    stack.push(Expr::Tmp(t));
                }
            }
            Operator::ReturnCallRef { type_index } => {
                let (p, r) = meta
                    .types
                    .get(type_index as usize)
                    .cloned()
                    .unwrap_or((Vec::new(), Vec::new()));
                // Same stack order as call_ref: ref on top.
                let fref = pop_or_unknown(&mut stack);
                let args = pop_n_or_unknown(&mut stack, p.len());
                let mut all = vec![fref];
                all.extend(args);
                if r.is_empty() {
                    cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                        op: "call.ref".into(),
                        args: all,
                    }));
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return { values: Vec::new() });
                } else {
                    let t = tmp;
                    tmp += 1;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Assign {
                        dst: format!("t{t}"),
                        expr: Expr::Simd {
                            op: "call.ref".into(),
                            args: all,
                        },
                    });
                    cur_stmts(&mut frames, &mut top).push(Stmt::Return {
                        values: vec![Expr::Tmp(t)],
                    });
                    stack.push(Expr::Unknown("tail-call-unreachable".into()));
                }
            }
            Operator::RefAsNonNull => {
                // assertion only; identity on the value stack
            }
            Operator::BrOnNull { relative_depth } => {
                let r = pop_or_unknown(&mut stack);
                let (label, is_loop, _) = resolve_target(&frames, relative_depth);
                stack.push(r.clone());
                cur_stmts(&mut frames, &mut top).push(Stmt::BrIf {
                    depth: relative_depth,
                    label,
                    is_loop,
                    cond: Expr::Simd {
                        op: "ref.is_null".into(),
                        args: vec![r],
                    },
                });
            }
            Operator::BrOnNonNull { relative_depth } => {
                let r = pop_or_unknown(&mut stack);
                let (label, is_loop, _) = resolve_target(&frames, relative_depth);
                stack.push(r.clone());
                cur_stmts(&mut frames, &mut top).push(Stmt::BrIf {
                    depth: relative_depth,
                    label,
                    is_loop,
                    cond: Expr::Simd {
                        op: "ref.is_non_null".into(),
                        args: vec![r],
                    },
                });
            }
            // ---- GC (pseudo-calls with exact arities) ----
            Operator::RefEq => simd_pop_push(&op, &mut stack, 2, 1),
            Operator::StructNew { struct_type_index } => {
                match meta.struct_arity.get(&struct_type_index).cloned() {
                    Some(n) => {
                        let mut args = pop_n_or_unknown(&mut stack, n);
                        args.push(Expr::ConstI32(struct_type_index as i32));
                        stack.push(Expr::Simd {
                            op: "struct.new".into(),
                            args,
                        });
                    }
                    None => {
                        cur_stmts(&mut frames, &mut top).push(Stmt::Comment(
                            "struct.new with unknown field count: stack approximated".into(),
                        ));
                        stack.push(Expr::Simd {
                            op: "struct.new".into(),
                            args: vec![Expr::ConstI32(struct_type_index as i32)],
                        });
                    }
                }
            }
            Operator::StructNewDefault { struct_type_index } => stack.push(Expr::Simd {
                op: "struct.new_default".into(),
                args: vec![Expr::ConstI32(struct_type_index as i32)],
            }),
            Operator::StructGet { struct_type_index, field_index }
            | Operator::StructGetS { struct_type_index, field_index }
            | Operator::StructGetU { struct_type_index, field_index } => {
                let sref = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![
                        sref,
                        Expr::ConstI32(struct_type_index as i32),
                        Expr::ConstI32(field_index as i32),
                    ],
                });
            }
            Operator::StructSet { struct_type_index, field_index } => {
                let v = pop_or_unknown(&mut stack);
                let s = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "struct.set".into(),
                    args: vec![
                        s,
                        v,
                        Expr::ConstI32(struct_type_index as i32),
                        Expr::ConstI32(field_index as i32),
                    ],
                }));
            }
            Operator::ArrayNew { array_type_index } => {
                // Stack order: [init, len] with len on top (wasmtime-verified:
                // `(array.new $a (3)(99))` yields len 99 of init 3).
                let len = pop_or_unknown(&mut stack);
                let init = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "array.new".into(),
                    args: vec![len, init, Expr::ConstI32(array_type_index as i32)],
                });
            }
            Operator::ArrayNewDefault { array_type_index } => {
                let len = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "array.new_default".into(),
                    args: vec![len, Expr::ConstI32(array_type_index as i32)],
                });
            }
            Operator::ArrayNewFixed { array_type_index, array_size } => {
                let args = pop_n_or_unknown(&mut stack, array_size as usize);
                let mut all = args;
                all.push(Expr::ConstI32(array_type_index as i32));
                stack.push(Expr::Simd {
                    op: "array.new_fixed".into(),
                    args: all,
                });
            }
            Operator::ArrayNewData { array_type_index, array_data_index } => {
                let len = pop_or_unknown(&mut stack);
                let off = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "array.new_data".into(),
                    args: vec![
                        off,
                        len,
                        Expr::ConstI32(array_type_index as i32),
                        Expr::ConstI32(array_data_index as i32),
                    ],
                });
            }
            Operator::ArrayNewElem { array_type_index, array_elem_index } => {
                let len = pop_or_unknown(&mut stack);
                let off = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "array.new_elem".into(),
                    args: vec![
                        off,
                        len,
                        Expr::ConstI32(array_type_index as i32),
                        Expr::ConstI32(array_elem_index as i32),
                    ],
                });
            }
            Operator::ArrayGet { array_type_index }
            | Operator::ArrayGetS { array_type_index }
            | Operator::ArrayGetU { array_type_index } => {
                let i = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![a, i, Expr::ConstI32(array_type_index as i32)],
                });
            }
            Operator::RefTestNonNull { hty } | Operator::RefTestNullable { hty } => {
                let r = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![r, Expr::Raw(format!("{hty:?}"))],
                });
            }
            Operator::RefCastNonNull { hty } | Operator::RefCastNullable { hty } => {
                let r = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![r, Expr::Raw(format!("{hty:?}"))],
                });
            }
            op @ (Operator::ArrayLen
            | Operator::AnyConvertExtern
            | Operator::ExternConvertAny
            | Operator::RefI31
            | Operator::I31GetS
            | Operator::I31GetU) => {
                simd_pop_push(&op, &mut stack, 1, 1);
            }
            Operator::ArraySet { array_type_index } => {
                let v = pop_or_unknown(&mut stack);
                let i = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "array.set".into(),
                    args: vec![a, i, v, Expr::ConstI32(array_type_index as i32)],
                }));
            }
            Operator::ArrayFill { array_type_index } => {
                let n = pop_or_unknown(&mut stack);
                let v = pop_or_unknown(&mut stack);
                let i = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "array.fill".into(),
                    args: vec![a, i, v, n, Expr::ConstI32(array_type_index as i32)],
                }));
            }
            Operator::ArrayCopy { array_type_index_dst, array_type_index_src } => {
                let n = pop_or_unknown(&mut stack);
                let si = pop_or_unknown(&mut stack);
                let sa = pop_or_unknown(&mut stack);
                let di = pop_or_unknown(&mut stack);
                let da = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "array.copy".into(),
                    args: vec![
                        da,
                        di,
                        sa,
                        si,
                        n,
                        Expr::ConstI32(array_type_index_dst as i32),
                        Expr::ConstI32(array_type_index_src as i32),
                    ],
                }));
            }
            Operator::ArrayInitData { array_type_index, array_data_index } => {
                let n = pop_or_unknown(&mut stack);
                let si = pop_or_unknown(&mut stack);
                let di = pop_or_unknown(&mut stack);
                let da = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "array.init_data".into(),
                    args: vec![
                        da,
                        di,
                        si,
                        n,
                        Expr::ConstI32(array_type_index as i32),
                        Expr::ConstI32(array_data_index as i32),
                    ],
                }));
            }
            Operator::ArrayInitElem { array_type_index, array_elem_index } => {
                let n = pop_or_unknown(&mut stack);
                let si = pop_or_unknown(&mut stack);
                let di = pop_or_unknown(&mut stack);
                let da = pop_or_unknown(&mut stack);
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "array.init_elem".into(),
                    args: vec![
                        da,
                        di,
                        si,
                        n,
                        Expr::ConstI32(array_type_index as i32),
                        Expr::ConstI32(array_elem_index as i32),
                    ],
                }));
            }
            op @ (Operator::BrOnCast { .. } | Operator::BrOnCastFail { .. }) => {
                let r = pop_or_unknown(&mut stack);
                // target depth is inside the payload; resolve best-effort to innermost
                let (label, is_loop, _) = resolve_target(&frames, 0);
                let name = op_mnemonic(&op);
                stack.push(r.clone());
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                    "{name}: target approximated to innermost scope"
                )));
                cur_stmts(&mut frames, &mut top).push(Stmt::BrIf {
                    depth: 0,
                    label,
                    is_loop,
                    cond: Expr::Simd {
                        op: name,
                        args: vec![r],
                    },
                });
            }
            // ---- exceptions ----
            Operator::TryTable { try_table } => {
                let r = block_results(&try_table.ty);
                label_seq += 1;
                let catches = try_table
                    .catches
                    .iter()
                    .map(|c| format!("{c:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                frames.push(Frame {
                    kind: FrameKind::Try,
                    label: format!("T{label_seq}"),
                    then_stmts: Vec::new(),
                    else_stmts: Vec::new(),
                    in_else: false,
                    entry_depth: stack.len(),
                    results: r.min(1),
                    phi: None,
                    phi_assigned: false,
                });
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::Comment(format!("try_table [{catches}]")));
            }
            Operator::Throw { tag_index } => {
                let entry = frames.last().map(|f| f.entry_depth).unwrap_or(0);
                stack.truncate(entry);
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::Comment(format!("throw tag {tag_index} (unwinds)")));
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::ExprStmt(Expr::Raw("__builtin_trap()".into())));
            }
            Operator::ThrowRef => {
                let _ = pop_or_unknown(&mut stack);
                let entry = frames.last().map(|f| f.entry_depth).unwrap_or(0);
                stack.truncate(entry);
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::Comment("throw_ref (unwinds)".into()));
                cur_stmts(&mut frames, &mut top)
                    .push(Stmt::ExprStmt(Expr::Raw("__builtin_trap()".into())));
            }
            Operator::Try { blockty } => {
                let r = block_results(&blockty);
                label_seq += 1;
                frames.push(Frame {
                    kind: FrameKind::Try,
                    label: format!("T{label_seq}"),
                    then_stmts: Vec::new(),
                    else_stmts: Vec::new(),
                    in_else: false,
                    entry_depth: stack.len(),
                    results: r,
                    phi: None,
                    phi_assigned: false,
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment("try {".into()));
            }
            Operator::Catch { tag_index } => {
                if let Some(f) = frames.last_mut() {
                    assert!(f.kind == FrameKind::Try, "catch without try");
                    f.in_else = true;
                    cur_stmts(&mut frames, &mut top)
                        .push(Stmt::Comment(format!("catch tag {tag_index}:")));
                }
            }
            Operator::CatchAll => {
                if let Some(f) = frames.last_mut() {
                    assert!(f.kind == FrameKind::Try, "catch_all without try");
                    f.in_else = true;
                    cur_stmts(&mut frames, &mut top).push(Stmt::Comment("catch_all:".into()));
                }
            }
            Operator::Rethrow { relative_depth } => {
                let (label, is_loop, _) = resolve_target(&frames, relative_depth);
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                    "rethrow -> {label} (unwinds past it)"
                )));
                cur_stmts(&mut frames, &mut top).push(Stmt::Br {
                    depth: relative_depth,
                    label,
                    is_loop,
                });
            }
            Operator::Delegate { relative_depth } => {
                // closes the try, delegates to an outer handler
                if let Some(f) = frames.pop() {
                    assert!(f.kind == FrameKind::Try, "delegate without try");
                    let label = f.label.clone();
                    cur_stmts(&mut frames, &mut top).push(Stmt::Block {
                        label,
                        body: f.then_stmts,
                    });
                }
                let (label, is_loop, _) = resolve_target(&frames, relative_depth);
                cur_stmts(&mut frames, &mut top).push(Stmt::Comment(format!(
                    "delegate -> {label}"
                )));
                cur_stmts(&mut frames, &mut top).push(Stmt::Br {
                    depth: relative_depth,
                    label,
                    is_loop,
                });
            }
            // ---- threads: atomic loads/stores (recorded as accesses) ----
            op @ (Operator::I32AtomicLoad { .. }
            | Operator::I32AtomicLoad8U { .. }
            | Operator::I32AtomicLoad16U { .. }
            | Operator::I64AtomicLoad { .. }
            | Operator::I64AtomicLoad8U { .. }
            | Operator::I64AtomicLoad16U { .. }
            | Operator::I64AtomicLoad32U { .. }) => {
                let base = pop_or_unknown(&mut stack);
                // MemArg is the only payload; offset via debug is unreliable,
                // so re-match for the offset below.
                let (offset, w) = atomic_mem(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset,
                    width: w,
                    is_write: false,
                    dom_ty: "atomic".into(),
                });
                stack.push(Expr::Load {
                    ty: format!("atomic/w{w}"),
                    base: Box::new(base),
                    offset,
                });
            }
            op @ (Operator::I32AtomicStore { .. }
            | Operator::I32AtomicStore8 { .. }
            | Operator::I32AtomicStore16 { .. }
            | Operator::I64AtomicStore { .. }
            | Operator::I64AtomicStore8 { .. }
            | Operator::I64AtomicStore16 { .. }
            | Operator::I64AtomicStore32 { .. }) => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let (offset, w) = atomic_mem(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset,
                    width: w,
                    is_write: true,
                    dom_ty: "atomic".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: format!("atomic/w{w}"),
                    base,
                    offset,
                    value: val,
                });
            }
            // read-modify-write: 2 -> 1 (address, value)
            op @ (Operator::I32AtomicRmwAdd { .. }
            | Operator::I64AtomicRmwAdd { .. }
            | Operator::I32AtomicRmw8AddU { .. }
            | Operator::I32AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw8AddU { .. }
            | Operator::I64AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw32AddU { .. }
            | Operator::I32AtomicRmwSub { .. }
            | Operator::I64AtomicRmwSub { .. }
            | Operator::I32AtomicRmw8SubU { .. }
            | Operator::I32AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw8SubU { .. }
            | Operator::I64AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw32SubU { .. }
            | Operator::I32AtomicRmwAnd { .. }
            | Operator::I64AtomicRmwAnd { .. }
            | Operator::I32AtomicRmw8AndU { .. }
            | Operator::I32AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw8AndU { .. }
            | Operator::I64AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw32AndU { .. }
            | Operator::I32AtomicRmwOr { .. }
            | Operator::I64AtomicRmwOr { .. }
            | Operator::I32AtomicRmw8OrU { .. }
            | Operator::I32AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw8OrU { .. }
            | Operator::I64AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw32OrU { .. }
            | Operator::I32AtomicRmwXor { .. }
            | Operator::I64AtomicRmwXor { .. }
            | Operator::I32AtomicRmw8XorU { .. }
            | Operator::I32AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw8XorU { .. }
            | Operator::I64AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw32XorU { .. }
            | Operator::I32AtomicRmwXchg { .. }
            | Operator::I64AtomicRmwXchg { .. }
            | Operator::I32AtomicRmw8XchgU { .. }
            | Operator::I32AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw8XchgU { .. }
            | Operator::I64AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw32XchgU { .. }) => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let (offset, w) = atomic_mem(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset,
                    width: w,
                    is_write: true,
                    dom_ty: "atomic-rmw".into(),
                });
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![base, val],
                });
            }
            // compare-exchange: 3 -> 1 (address, expected, replacement)
            op @ (Operator::I32AtomicRmwCmpxchg { .. }
            | Operator::I64AtomicRmwCmpxchg { .. }
            | Operator::I32AtomicRmw8CmpxchgU { .. }
            | Operator::I32AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw8CmpxchgU { .. }
            | Operator::I64AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw32CmpxchgU { .. }) => {
                let rep = pop_or_unknown(&mut stack);
                let exp = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let (offset, w) = atomic_mem(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset,
                    width: w,
                    is_write: true,
                    dom_ty: "atomic-cmpxchg".into(),
                });
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![base, exp, rep],
                });
            }
            Operator::MemoryAtomicNotify { .. } => {
                let n = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                stack.push(Expr::Simd {
                    op: "memory.atomic.notify".into(),
                    args: vec![base, n],
                });
            }
            op @ (Operator::MemoryAtomicWait32 { .. } | Operator::MemoryAtomicWait64 { .. }) => {
                let timeout = pop_or_unknown(&mut stack);
                let exp = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![base, exp, timeout],
                });
            }
            Operator::AtomicFence => {
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: "atomic.fence".into(),
                    args: Vec::new(),
                }));
            }
            // ---- v128 memory ops ----
            op @ (Operator::V128Load { .. }
            | Operator::V128Load8x8S { .. }
            | Operator::V128Load8x8U { .. }
            | Operator::V128Load16x4S { .. }
            | Operator::V128Load16x4U { .. }
            | Operator::V128Load32x2S { .. }
            | Operator::V128Load32x2U { .. }
            | Operator::V128Load8Splat { .. }
            | Operator::V128Load16Splat { .. }
            | Operator::V128Load32Splat { .. }
            | Operator::V128Load64Splat { .. }
            | Operator::V128Load32Zero { .. }
            | Operator::V128Load64Zero { .. }) => {
                let base = pop_or_unknown(&mut stack);
                let offset = v128_mem_offset(&op);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset,
                    width: 16,
                    is_write: false,
                    dom_ty: "v128".into(),
                });
                stack.push(Expr::Load {
                    ty: "v128".into(),
                    base: Box::new(base),
                    offset,
                });
            }
            Operator::V128Store { memarg } => {
                let val = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                accesses.push(MemAccess {
                    base_desc: base_desc(&base),
                    offset: memarg.offset,
                    width: 16,
                    is_write: true,
                    dom_ty: "v128".into(),
                });
                cur_stmts(&mut frames, &mut top).push(Stmt::Store {
                    ty: "v128".into(),
                    base,
                    offset: memarg.offset,
                    value: val,
                });
            }
            Operator::V128Const { value } => {
                let b = value.bytes();
                let lo = i64::from_le_bytes(b[0..8].try_into().unwrap());
                let hi = i64::from_le_bytes(b[8..16].try_into().unwrap());
                stack.push(Expr::Simd {
                    op: "v128.const".into(),
                    args: vec![Expr::ConstI64(lo), Expr::ConstI64(hi)],
                });
            }
            // lane loads: 2 -> 1 (address, vector); lane kept for accuracy
            op @ (Operator::V128Load8Lane { lane, .. }
            | Operator::V128Load16Lane { lane, .. }
            | Operator::V128Load32Lane { lane, .. }
            | Operator::V128Load64Lane { lane, .. }) => {
                let v = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                let lane = lane;
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![base, v, Expr::ConstI32(lane as i32)],
                });
            }
            // lane stores: 2 -> 0 (address, vector)
            op @ (Operator::V128Store8Lane { lane, .. }
            | Operator::V128Store16Lane { lane, .. }
            | Operator::V128Store32Lane { lane, .. }
            | Operator::V128Store64Lane { lane, .. }) => {
                let v = pop_or_unknown(&mut stack);
                let base = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                let lane = lane;
                cur_stmts(&mut frames, &mut top).push(Stmt::ExprStmt(Expr::Simd {
                    op: name,
                    args: vec![base, v, Expr::ConstI32(lane as i32)],
                }));
            }
            // extract-lane keeps its lane index (lane 0 vs 3 differ!)
            op @ (Operator::I8x16ExtractLaneS { lane, .. }
            | Operator::I8x16ExtractLaneU { lane, .. }
            | Operator::I16x8ExtractLaneS { lane, .. }
            | Operator::I16x8ExtractLaneU { lane, .. }
            | Operator::I32x4ExtractLane { lane, .. }
            | Operator::I64x2ExtractLane { lane, .. }
            | Operator::F32x4ExtractLane { lane, .. }
            | Operator::F64x2ExtractLane { lane, .. }) => {
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                let lane = lane;
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![v, Expr::ConstI32(lane as i32)],
                });
            }
            // replace-lane keeps its lane index
            op @ (Operator::I8x16ReplaceLane { lane, .. }
            | Operator::I16x8ReplaceLane { lane, .. }
            | Operator::I32x4ReplaceLane { lane, .. }
            | Operator::I64x2ReplaceLane { lane, .. }
            | Operator::F32x4ReplaceLane { lane, .. }
            | Operator::F64x2ReplaceLane { lane, .. }) => {
                let val = pop_or_unknown(&mut stack);
                let v = pop_or_unknown(&mut stack);
                let name = op_mnemonic(&op);
                let lane = lane;
                stack.push(Expr::Simd {
                    op: name,
                    args: vec![v, val, Expr::ConstI32(lane as i32)],
                });
            }
            Operator::I8x16Shuffle { lanes } => {
                let b = pop_or_unknown(&mut stack);
                let a = pop_or_unknown(&mut stack);
                let mut args = vec![a, b];
                for l in lanes {
                    args.push(Expr::ConstI32(l as i32));
                }
                stack.push(Expr::Simd {
                    op: "i8x16.shuffle".into(),
                    args,
                });
            }
            // ---- generic SIMD by arity (wat-style names via mnemonic) ----
            op @ (Operator::I8x16Splat
            | Operator::I16x8Splat
            | Operator::I32x4Splat
            | Operator::I64x2Splat
            | Operator::F32x4Splat
            | Operator::F64x2Splat
            | Operator::I8x16Abs
            | Operator::I8x16Neg
            | Operator::I8x16Popcnt
            | Operator::I8x16AllTrue
            | Operator::I8x16Bitmask
            | Operator::I16x8Abs
            | Operator::I16x8Neg
            | Operator::I16x8AllTrue
            | Operator::I16x8Bitmask
            | Operator::I32x4Abs
            | Operator::I32x4Neg
            | Operator::I32x4AllTrue
            | Operator::I32x4Bitmask
            | Operator::I64x2Abs
            | Operator::I64x2Neg
            | Operator::I64x2AllTrue
            | Operator::I64x2Bitmask
            | Operator::F32x4Abs
            | Operator::F32x4Neg
            | Operator::F32x4Ceil
            | Operator::F32x4Floor
            | Operator::F32x4Trunc
            | Operator::F32x4Nearest
            | Operator::F32x4Sqrt
            | Operator::F64x2Abs
            | Operator::F64x2Neg
            | Operator::F64x2Ceil
            | Operator::F64x2Floor
            | Operator::F64x2Trunc
            | Operator::F64x2Nearest
            | Operator::F64x2Sqrt
            | Operator::V128Not
            | Operator::V128AnyTrue
            | Operator::I16x8ExtAddPairwiseI8x16S
            | Operator::I16x8ExtAddPairwiseI8x16U
            | Operator::I16x8ExtendLowI8x16S
            | Operator::I16x8ExtendHighI8x16S
            | Operator::I16x8ExtendLowI8x16U
            | Operator::I16x8ExtendHighI8x16U
            | Operator::I32x4ExtendLowI16x8S
            | Operator::I32x4ExtendHighI16x8S
            | Operator::I32x4ExtendLowI16x8U
            | Operator::I32x4ExtendHighI16x8U
            | Operator::I64x2ExtendLowI32x4S
            | Operator::I64x2ExtendHighI32x4S
            | Operator::I64x2ExtendLowI32x4U
            | Operator::I64x2ExtendHighI32x4U
            | Operator::I32x4ExtAddPairwiseI16x8S
            | Operator::I32x4ExtAddPairwiseI16x8U
            | Operator::F32x4ConvertI32x4S
            | Operator::F32x4ConvertI32x4U
            | Operator::I32x4TruncSatF32x4S
            | Operator::I32x4TruncSatF32x4U
            | Operator::I32x4TruncSatF64x2SZero
            | Operator::I32x4TruncSatF64x2UZero
            | Operator::F64x2ConvertLowI32x4S
            | Operator::F64x2ConvertLowI32x4U
            | Operator::F32x4DemoteF64x2Zero
            | Operator::F64x2PromoteLowF32x4
            | Operator::I32x4RelaxedTruncF32x4S
            | Operator::I32x4RelaxedTruncF32x4U
            | Operator::I32x4RelaxedTruncF64x2SZero
            | Operator::I32x4RelaxedTruncF64x2UZero) => {
                simd_pop_push(&op, &mut stack, 1, 1);
            }
            op @ (Operator::I8x16Swizzle
            | Operator::I8x16Eq
            | Operator::I8x16Ne
            | Operator::I8x16LtS
            | Operator::I8x16LtU
            | Operator::I8x16GtS
            | Operator::I8x16GtU
            | Operator::I8x16LeS
            | Operator::I8x16LeU
            | Operator::I8x16GeS
            | Operator::I8x16GeU
            | Operator::I16x8Eq
            | Operator::I16x8Ne
            | Operator::I16x8LtS
            | Operator::I16x8LtU
            | Operator::I16x8GtS
            | Operator::I16x8GtU
            | Operator::I16x8LeS
            | Operator::I16x8LeU
            | Operator::I16x8GeS
            | Operator::I16x8GeU
            | Operator::I32x4Eq
            | Operator::I32x4Ne
            | Operator::I32x4LtS
            | Operator::I32x4LtU
            | Operator::I32x4GtS
            | Operator::I32x4GtU
            | Operator::I32x4LeS
            | Operator::I32x4LeU
            | Operator::I32x4GeS
            | Operator::I32x4GeU
            | Operator::I64x2Eq
            | Operator::I64x2Ne
            | Operator::I64x2LtS
            | Operator::I64x2GtS
            | Operator::I64x2LeS
            | Operator::I64x2GeS
            | Operator::F32x4Eq
            | Operator::F32x4Ne
            | Operator::F32x4Lt
            | Operator::F32x4Gt
            | Operator::F32x4Le
            | Operator::F32x4Ge
            | Operator::F64x2Eq
            | Operator::F64x2Ne
            | Operator::F64x2Lt
            | Operator::F64x2Gt
            | Operator::F64x2Le
            | Operator::F64x2Ge
            | Operator::V128And
            | Operator::V128AndNot
            | Operator::V128Or
            | Operator::V128Xor
            | Operator::I8x16Shl
            | Operator::I8x16ShrS
            | Operator::I8x16ShrU
            | Operator::I8x16Add
            | Operator::I8x16AddSatS
            | Operator::I8x16AddSatU
            | Operator::I8x16Sub
            | Operator::I8x16SubSatS
            | Operator::I8x16SubSatU
            | Operator::I8x16MinS
            | Operator::I8x16MinU
            | Operator::I8x16MaxS
            | Operator::I8x16MaxU
            | Operator::I8x16AvgrU
            | Operator::I8x16NarrowI16x8S
            | Operator::I8x16NarrowI16x8U
            | Operator::I16x8Shl
            | Operator::I16x8ShrS
            | Operator::I16x8ShrU
            | Operator::I16x8Add
            | Operator::I16x8AddSatS
            | Operator::I16x8AddSatU
            | Operator::I16x8Sub
            | Operator::I16x8SubSatS
            | Operator::I16x8SubSatU
            | Operator::I16x8Mul
            | Operator::I16x8MinS
            | Operator::I16x8MinU
            | Operator::I16x8MaxS
            | Operator::I16x8MaxU
            | Operator::I16x8AvgrU
            | Operator::I16x8NarrowI32x4S
            | Operator::I16x8NarrowI32x4U
            | Operator::I16x8Q15MulrSatS
            | Operator::I16x8ExtMulLowI8x16S
            | Operator::I16x8ExtMulHighI8x16S
            | Operator::I16x8ExtMulLowI8x16U
            | Operator::I16x8ExtMulHighI8x16U
            | Operator::I32x4Shl
            | Operator::I32x4ShrS
            | Operator::I32x4ShrU
            | Operator::I32x4Add
            | Operator::I32x4Sub
            | Operator::I32x4Mul
            | Operator::I32x4MinS
            | Operator::I32x4MinU
            | Operator::I32x4MaxS
            | Operator::I32x4MaxU
            | Operator::I32x4DotI16x8S
            | Operator::I32x4ExtMulLowI16x8S
            | Operator::I32x4ExtMulHighI16x8S
            | Operator::I32x4ExtMulLowI16x8U
            | Operator::I32x4ExtMulHighI16x8U
            | Operator::I64x2Shl
            | Operator::I64x2ShrS
            | Operator::I64x2ShrU
            | Operator::I64x2Add
            | Operator::I64x2Sub
            | Operator::I64x2Mul
            | Operator::I64x2ExtMulLowI32x4S
            | Operator::I64x2ExtMulHighI32x4S
            | Operator::I64x2ExtMulLowI32x4U
            | Operator::I64x2ExtMulHighI32x4U
            | Operator::F32x4Add
            | Operator::F32x4Sub
            | Operator::F32x4Mul
            | Operator::F32x4Div
            | Operator::F32x4Min
            | Operator::F32x4Max
            | Operator::F32x4PMin
            | Operator::F32x4PMax
            | Operator::F64x2Add
            | Operator::F64x2Sub
            | Operator::F64x2Mul
            | Operator::F64x2Div
            | Operator::F64x2Min
            | Operator::F64x2Max
            | Operator::F64x2PMin
            | Operator::F64x2PMax
            | Operator::I8x16RelaxedSwizzle
            | Operator::F32x4RelaxedMin
            | Operator::F32x4RelaxedMax
            | Operator::F64x2RelaxedMin
            | Operator::F64x2RelaxedMax
            | Operator::I16x8RelaxedQ15mulrS
            | Operator::I16x8RelaxedDotI8x16I7x16S) => {
                simd_pop_push(&op, &mut stack, 2, 1);
            }
            op @ (Operator::V128Bitselect
            | Operator::F32x4RelaxedMadd
            | Operator::F32x4RelaxedNmadd
            | Operator::F64x2RelaxedMadd
            | Operator::F64x2RelaxedNmadd
            | Operator::I8x16RelaxedLaneselect
            | Operator::I16x8RelaxedLaneselect
            | Operator::I32x4RelaxedLaneselect
            | Operator::I64x2RelaxedLaneselect
            | Operator::I32x4RelaxedDotI8x16I7x16AddS) => {
                simd_pop_push(&op, &mut stack, 3, 1);
            }
            // ---- wide arithmetic: two results ----
            op @ (Operator::I64Add128
            | Operator::I64Sub128
            | Operator::I64MulWideS
            | Operator::I64MulWideU) => {
                let n = if matches!(
                    op,
                    Operator::I64Add128 | Operator::I64Sub128
                ) {
                    4
                } else {
                    2
                };
                simd_pop_push(&op, &mut stack, n, 2);
            }
            // ---- shared-everything-threads atomics (pseudo-calls) ----
            op @ (Operator::GlobalAtomicGet { .. }
            | Operator::TableAtomicGet { .. }
            | Operator::StructAtomicGet { .. }
            | Operator::StructAtomicGetS { .. }
            | Operator::StructAtomicGetU { .. }
            | Operator::GlobalAtomicRmwAdd { .. }
            | Operator::GlobalAtomicRmwSub { .. }
            | Operator::GlobalAtomicRmwAnd { .. }
            | Operator::GlobalAtomicRmwOr { .. }
            | Operator::GlobalAtomicRmwXor { .. }
            | Operator::GlobalAtomicRmwXchg { .. }) => {
                simd_pop_push(&op, &mut stack, 1, 1);
            }
            op @ (Operator::ArrayAtomicGet { .. }
            | Operator::ArrayAtomicGetS { .. }
            | Operator::ArrayAtomicGetU { .. }) => {
                simd_pop_push(&op, &mut stack, 2, 1);
            }
            Operator::GlobalAtomicSet { .. } => {
                simd_stmt(&op, &mut stack, &mut frames, &mut top, 1);
            }
            op @ (Operator::TableAtomicSet { .. } | Operator::StructAtomicSet { .. }) => {
                simd_stmt(&op, &mut stack, &mut frames, &mut top, 2);
            }
            op @ (Operator::TableAtomicRmwXchg { .. }
            | Operator::StructAtomicRmwAdd { .. }
            | Operator::StructAtomicRmwSub { .. }
            | Operator::StructAtomicRmwAnd { .. }
            | Operator::StructAtomicRmwOr { .. }
            | Operator::StructAtomicRmwXor { .. }
            | Operator::StructAtomicRmwXchg { .. }
            | Operator::ArrayAtomicRmwAdd { .. }
            | Operator::ArrayAtomicRmwSub { .. }
            | Operator::ArrayAtomicRmwAnd { .. }
            | Operator::ArrayAtomicRmwOr { .. }
            | Operator::ArrayAtomicRmwXor { .. }
            | Operator::ArrayAtomicRmwXchg { .. }) => {
                // note: global/table variants of these are 1->1 or 2->1;
                // table.rmw.xchg takes (index, value): 2 inputs.
                simd_pop_push(&op, &mut stack, 2, 1);
            }
            op @ (Operator::ArrayAtomicSet { .. }
            | Operator::TableAtomicRmwCmpxchg { .. }
            | Operator::StructAtomicRmwCmpxchg { .. }) => {
                // array.set: 3 inputs; cmpxchg on table/struct: 3 inputs
                let is_set = matches!(op, Operator::ArrayAtomicSet { .. });
                if is_set {
                    simd_stmt(&op, &mut stack, &mut frames, &mut top, 3);
                } else {
                    simd_pop_push(&op, &mut stack, 3, 1);
                }
            }
            op @ Operator::GlobalAtomicRmwCmpxchg { .. } => {
                simd_pop_push(&op, &mut stack, 2, 1);
            }
            op @ Operator::ArrayAtomicRmwCmpxchg { .. } => {
                simd_pop_push(&op, &mut stack, 4, 1);
            }
            op @ Operator::RefI31Shared => {
                simd_pop_push(&op, &mut stack, 1, 1);
            }
            other => {
                // Safety net for future/unknown ops (stack-switching etc.).
                // Everything with known arity has an explicit arm above.
                // Guess: pop up to 2, push Unknown, with wat-style mnemonic
                // so output never silently lies.
                let short = op_mnemonic(&other);
                // memory/table bulk ops consume more; handle common ones explicitly:
                if short.contains("store") || short.contains("copy") || short.contains("fill") || short.contains("init") {
                    let _ = pop_or_unknown(&mut stack);
                    let _ = pop_or_unknown(&mut stack);
                    let _ = pop_or_unknown(&mut stack);
                    cur_stmts(&mut frames, &mut top)
                        .push(Stmt::Comment(format!("unhandled {short}: dropped 3 stack values")));
                } else if short.contains("load") || short.contains("get") || short.contains("size") {
                    let _ = pop_or_unknown(&mut stack);
                    stack.push(Expr::Unknown(short.clone()));
                    cur_stmts(&mut frames, &mut top)
                        .push(Stmt::Comment(format!("unhandled {short}: pushed unknown")));
                } else {
                    let a = pop_or_unknown(&mut stack);
                    let b = if stack.is_empty() { None } else { stack.pop() };
                    let mut inputs = vec![a];
                    if let Some(x) = b {
                        inputs.push(x);
                    }
                    stack.push(Expr::Unknown(short.clone()));
                    cur_stmts(&mut frames, &mut top)
                        .push(Stmt::Comment(format!("unhandled {short} ({} inputs)", inputs.len())));
                }
            }
        }
    }

    // drain remaining stack: leftovers -> tmps, results -> implicit return.
    // tail-call markers are control-flow, not values: drop them (an explicit
    // Return was already emitted for the tail call).
    stack.retain(|e| !matches!(e, Expr::Unknown(s) if s == "tail-call-unreachable"));
    if results.is_empty() {
        while !stack.is_empty() {
            let v = stack.remove(0);
            let t = tmp;
            tmp += 1;
            top.push(Stmt::Assign {
                dst: format!("t{t}"),
                expr: v,
            });
        }
    } else {
        // keep last N as return values
        let mut rets: Vec<Expr> = Vec::new();
        while rets.len() < results.len() && !stack.is_empty() {
            rets.push(stack.pop().unwrap());
        }
        rets.reverse();
        while !stack.is_empty() {
            let v = stack.remove(0);
            let t = tmp;
            tmp += 1;
            top.push(Stmt::Assign {
                dst: format!("t{t}"),
                expr: v,
            });
        }
        if !rets.is_empty() {
            top.push(Stmt::Return { values: rets });
        }
    }

    Ok(FuncIR {
        idx,
        name,
        params,
        results,
        locals,
        body: top,
        accesses,
        calls,
        indirects,
        aliases: HashMap::new(),
    })
}

fn resolve_target(frames: &[Frame], depth: u32) -> (String, bool, usize) {
    if let Some(f) = frames.iter().rev().nth(depth as usize) {
        let is_loop = f.kind == FrameKind::Loop;
        (f.label.clone(), is_loop, f.entry_depth)
    } else {
        ("func_end".to_string(), false, 0)
    }
}

fn atomic_mem(op: &Operator) -> (u64, u8) {
    let dbg = format!("{op:?}");
    // offset is not carried in a uniform field across all atomic variants;
    // extract from the MemArg via a re-match on the few shapes we use.
    // All atomic ops carry exactly one MemArg; parse offset from debug is
    // fragile, so match the concrete variants instead.
    macro_rules! m {
        ($($v:pat => $w:expr),*) => {
            match op {
                $($v => $w,)*
                _ => 4,
            }
        };
    }
    let w: u8 = m! {
        Operator::I32AtomicLoad { .. }
        | Operator::I32AtomicStore { .. }
        | Operator::I32AtomicRmwAdd { .. }
        | Operator::I32AtomicRmwSub { .. }
        | Operator::I32AtomicRmwAnd { .. }
        | Operator::I32AtomicRmwOr { .. }
        | Operator::I32AtomicRmwXor { .. }
        | Operator::I32AtomicRmwXchg { .. }
        | Operator::I32AtomicRmwCmpxchg { .. } => 4,
        Operator::I64AtomicLoad { .. }
        | Operator::I64AtomicStore { .. }
        | Operator::I64AtomicRmwAdd { .. }
        | Operator::I64AtomicRmwSub { .. }
        | Operator::I64AtomicRmwAnd { .. }
        | Operator::I64AtomicRmwOr { .. }
        | Operator::I64AtomicRmwXor { .. }
        | Operator::I64AtomicRmwXchg { .. }
        | Operator::I64AtomicRmwCmpxchg { .. } => 8,
        Operator::I32AtomicLoad8U { .. }
        | Operator::I32AtomicStore8 { .. }
        | Operator::I32AtomicRmw8AddU { .. }
        | Operator::I32AtomicRmw8SubU { .. }
        | Operator::I32AtomicRmw8AndU { .. }
        | Operator::I32AtomicRmw8OrU { .. }
        | Operator::I32AtomicRmw8XorU { .. }
        | Operator::I32AtomicRmw8XchgU { .. }
        | Operator::I32AtomicRmw8CmpxchgU { .. }
        | Operator::I64AtomicLoad8U { .. }
        | Operator::I64AtomicStore8 { .. }
        | Operator::I64AtomicRmw8AddU { .. }
        | Operator::I64AtomicRmw8SubU { .. }
        | Operator::I64AtomicRmw8AndU { .. }
        | Operator::I64AtomicRmw8OrU { .. }
        | Operator::I64AtomicRmw8XchgU { .. }
        | Operator::I64AtomicRmw8CmpxchgU { .. } => 1,
        Operator::I32AtomicLoad16U { .. }
        | Operator::I32AtomicStore16 { .. }
        | Operator::I32AtomicRmw16AddU { .. }
        | Operator::I32AtomicRmw16SubU { .. }
        | Operator::I32AtomicRmw16AndU { .. }
        | Operator::I32AtomicRmw16OrU { .. }
        | Operator::I32AtomicRmw16XorU { .. }
        | Operator::I32AtomicRmw16XchgU { .. }
        | Operator::I32AtomicRmw16CmpxchgU { .. }
        | Operator::I64AtomicLoad16U { .. }
        | Operator::I64AtomicStore16 { .. }
        | Operator::I64AtomicRmw16AddU { .. }
        | Operator::I64AtomicRmw16SubU { .. }
        | Operator::I64AtomicRmw16AndU { .. }
        | Operator::I64AtomicRmw16OrU { .. }
        | Operator::I64AtomicRmw16XchgU { .. }
        | Operator::I64AtomicRmw16CmpxchgU { .. } => 2,
        Operator::I64AtomicLoad32U { .. }
        | Operator::I64AtomicStore32 { .. }
        | Operator::I64AtomicRmw32AddU { .. }
        | Operator::I64AtomicRmw32SubU { .. }
        | Operator::I64AtomicRmw32AndU { .. }
        | Operator::I64AtomicRmw32OrU { .. }
        | Operator::I64AtomicRmw32XorU { .. }
        | Operator::I64AtomicRmw32XchgU { .. }
        | Operator::I64AtomicRmw32CmpxchgU { .. } => 4
    };
    // offset: all these variants carry `memarg`; extract uniformly
    let offset = match op {
        Operator::I32AtomicLoad { memarg }
        | Operator::I32AtomicLoad8U { memarg }
        | Operator::I32AtomicLoad16U { memarg }
        | Operator::I64AtomicLoad { memarg }
        | Operator::I64AtomicLoad8U { memarg }
        | Operator::I64AtomicLoad16U { memarg }
        | Operator::I64AtomicLoad32U { memarg }
        | Operator::I32AtomicStore { memarg }
        | Operator::I32AtomicStore8 { memarg }
        | Operator::I32AtomicStore16 { memarg }
        | Operator::I64AtomicStore { memarg }
        | Operator::I64AtomicStore8 { memarg }
        | Operator::I64AtomicStore16 { memarg }
        | Operator::I64AtomicStore32 { memarg }
        | Operator::I32AtomicRmwAdd { memarg }
        | Operator::I64AtomicRmwAdd { memarg }
        | Operator::I32AtomicRmw8AddU { memarg }
        | Operator::I32AtomicRmw16AddU { memarg }
        | Operator::I64AtomicRmw8AddU { memarg }
        | Operator::I64AtomicRmw16AddU { memarg }
        | Operator::I64AtomicRmw32AddU { memarg }
        | Operator::I32AtomicRmwSub { memarg }
        | Operator::I64AtomicRmwSub { memarg }
        | Operator::I32AtomicRmw8SubU { memarg }
        | Operator::I32AtomicRmw16SubU { memarg }
        | Operator::I64AtomicRmw8SubU { memarg }
        | Operator::I64AtomicRmw16SubU { memarg }
        | Operator::I64AtomicRmw32SubU { memarg }
        | Operator::I32AtomicRmwAnd { memarg }
        | Operator::I64AtomicRmwAnd { memarg }
        | Operator::I32AtomicRmw8AndU { memarg }
        | Operator::I32AtomicRmw16AndU { memarg }
        | Operator::I64AtomicRmw8AndU { memarg }
        | Operator::I64AtomicRmw16AndU { memarg }
        | Operator::I64AtomicRmw32AndU { memarg }
        | Operator::I32AtomicRmwOr { memarg }
        | Operator::I64AtomicRmwOr { memarg }
        | Operator::I32AtomicRmw8OrU { memarg }
        | Operator::I32AtomicRmw16OrU { memarg }
        | Operator::I64AtomicRmw8OrU { memarg }
        | Operator::I64AtomicRmw16OrU { memarg }
        | Operator::I64AtomicRmw32OrU { memarg }
        | Operator::I32AtomicRmwXor { memarg }
        | Operator::I64AtomicRmwXor { memarg }
        | Operator::I32AtomicRmw8XorU { memarg }
        | Operator::I32AtomicRmw16XorU { memarg }
        | Operator::I64AtomicRmw8XorU { memarg }
        | Operator::I64AtomicRmw16XorU { memarg }
        | Operator::I64AtomicRmw32XorU { memarg }
        | Operator::I32AtomicRmwXchg { memarg }
        | Operator::I64AtomicRmwXchg { memarg }
        | Operator::I32AtomicRmw8XchgU { memarg }
        | Operator::I32AtomicRmw16XchgU { memarg }
        | Operator::I64AtomicRmw8XchgU { memarg }
        | Operator::I64AtomicRmw16XchgU { memarg }
        | Operator::I64AtomicRmw32XchgU { memarg }
        | Operator::I32AtomicRmwCmpxchg { memarg }
        | Operator::I64AtomicRmwCmpxchg { memarg }
        | Operator::I32AtomicRmw8CmpxchgU { memarg }
        | Operator::I32AtomicRmw16CmpxchgU { memarg }
        | Operator::I64AtomicRmw8CmpxchgU { memarg }
        | Operator::I64AtomicRmw16CmpxchgU { memarg }
        | Operator::I64AtomicRmw32CmpxchgU { memarg } => memarg.offset,
        _ => {
            let _ = dbg;
            0
        }
    };
    (offset, w)
}

fn v128_mem_offset(op: &Operator) -> u64 {
    match op {
        Operator::V128Load { memarg }
        | Operator::V128Load8x8S { memarg }
        | Operator::V128Load8x8U { memarg }
        | Operator::V128Load16x4S { memarg }
        | Operator::V128Load16x4U { memarg }
        | Operator::V128Load32x2S { memarg }
        | Operator::V128Load32x2U { memarg }
        | Operator::V128Load8Splat { memarg }
        | Operator::V128Load16Splat { memarg }
        | Operator::V128Load32Splat { memarg }
        | Operator::V128Load64Splat { memarg }
        | Operator::V128Load32Zero { memarg }
        | Operator::V128Load64Zero { memarg } => memarg.offset,
        _ => 0,
    }
}

fn load_width(op: &Operator) -> u8 {
    use wasmparser::Operator as O;
    match op {
        O::I32Load { .. } => 4,
        O::I32Load8S { .. } | O::I32Load8U { .. } => 1,
        O::I32Load16S { .. } | O::I32Load16U { .. } => 2,
        O::I64Load { .. } => 8,
        O::I64Load8S { .. } | O::I64Load8U { .. } => 1,
        O::I64Load16S { .. } | O::I64Load16U { .. } => 2,
        O::I64Load32S { .. } | O::I64Load32U { .. } => 4,
        O::I32Store { .. } => 4,
        O::I32Store8 { .. } => 1,
        O::I32Store16 { .. } => 2,
        O::I64Store { .. } => 8,
        O::I64Store8 { .. } => 1,
        O::I64Store16 { .. } => 2,
        O::I64Store32 { .. } => 4,
        _ => 4,
    }
}

fn store_width(op: &Operator) -> u8 {
    load_width(op)
}

