//! Phase-4/6 cleanup passes: const-fold, copy/DCE + tmp inline,
//! label-aware block simplify, struct-init split. All accuracy-preserving.

use std::collections::{HashMap, HashSet};

use crate::cfg;
use crate::ir::{Expr, FuncIR, ModuleIR, Stmt, Var};

/// Per-pass switches (CLI `--no-*`). All default on; `--no-opt` disables all.
#[derive(Debug, Clone)]
pub struct OptConfig {
    pub fold: bool,
    pub inline: bool,
    pub dce: bool,
    pub simplify: bool,
}

impl Default for OptConfig {
    fn default() -> Self {
        OptConfig {
            fold: true,
            inline: true,
            dce: true,
            simplify: true,
        }
    }
}

impl OptConfig {
    pub fn all_off() -> Self {
        OptConfig {
            fold: false,
            inline: false,
            dce: false,
            simplify: false,
        }
    }
}

pub fn optimize_func(f: &mut FuncIR) {
    optimize_func_with(f, &OptConfig::default());
}

/// Copy-propagation aliases: old base name -> new base name, so the
/// emitter can resolve struct layouts recorded under pre-prop names.
/// Only populated for simple (Local) replacements.
pub type AliasMap = HashMap<String, String>;

/// Shared per-function state for passes. Passes that discover renames
/// (copyprop) record them here so later passes and the emitter agree.
#[derive(Debug, Default)]
pub struct PassCtx {
    pub aliases: AliasMap,
}

/// A single function pass. `run` returns true if it changed the function.
/// Implement this to write your own deobfuscation/optimization pass —
/// see `examples/const_on_right.rs`.
pub trait FuncPass {
    fn name(&self) -> &'static str;
    fn run(&self, f: &mut FuncIR, ctx: &mut PassCtx) -> bool;
}

/// Constant folding + const-branch pruning.
pub struct FoldPass;
/// Single-use tmp/local inlining + dead-code elimination.
pub struct InlineDcePass {
    pub inline: bool,
    pub dce: bool,
}
/// Empty-block removal + pass-through flattening.
pub struct SimplifyPass;
/// Packed-i64-init splitting (needs i32 evidence; gated with folding).
pub struct SplitInitPass;

impl FuncPass for FoldPass {
    fn name(&self) -> &'static str {
        "fold"
    }
    fn run(&self, f: &mut FuncIR, _ctx: &mut PassCtx) -> bool {
        fold_stmts_inplace(&mut f.body)
    }
}

impl FuncPass for InlineDcePass {
    fn name(&self) -> &'static str {
        "inline-dce"
    }
    fn run(&self, f: &mut FuncIR, ctx: &mut PassCtx) -> bool {
        let opt = OptConfig {
            fold: false,
            inline: self.inline,
            dce: self.dce,
            simplify: false,
        };
        inline_and_dce(&mut f.body, &opt, &mut ctx.aliases)
    }
}

impl FuncPass for SimplifyPass {
    fn name(&self) -> &'static str {
        "simplify"
    }
    fn run(&self, f: &mut FuncIR, _ctx: &mut PassCtx) -> bool {
        simplify_blocks(&mut f.body)
    }
}

impl FuncPass for SplitInitPass {
    fn name(&self) -> &'static str {
        "split-init"
    }
    fn run(&self, f: &mut FuncIR, _ctx: &mut PassCtx) -> bool {
        split_init_stores(f)
    }
}

/// A whole-module pass (Phase C). Runs after per-function optimization;
/// sees the call graph, summaries, and all bodies.
pub trait ModulePass {
    fn name(&self) -> &'static str;
    fn run(&self, m: &mut ModuleIR) -> bool;
}

/// Fold direct calls to `return <const>` callees into the constant.
/// Sound: the callee body is a lone `return const` (no effects), and
/// only pure-arg call sites rewrite (dropping effectful args would
/// change semantics).
pub struct ConstRetPass;

impl ModulePass for ConstRetPass {
    fn name(&self) -> &'static str {
        "const-ret"
    }
    fn run(&self, m: &mut ModuleIR) -> bool {
        let sums = crate::callgraph::summarize(m);
        let mut changed = false;
        for f in m.funcs.iter_mut() {
            changed |= fold_const_calls(&mut f.body, &sums);
        }
        changed
    }
}

fn fold_const_calls(
    stmts: &mut Vec<Stmt>,
    sums: &HashMap<u32, crate::callgraph::Summary>,
) -> bool {
    fn expr(e: &mut Expr, sums: &HashMap<u32, crate::callgraph::Summary>) -> bool {
        let mut changed = false;
        match e {
            Expr::Call { func, args, .. } => {
                for a in args.iter_mut() {
                    changed |= expr(a, sums);
                }
                if let Some(c) = sums.get(func).and_then(|s| s.ret_const.clone()) {
                    if args.iter().all(|a| a.is_pure()) {
                        *e = c;
                        changed = true;
                    }
                }
            }
            Expr::CallIndirect { index, args, .. } => {
                changed |= expr(index, sums);
                for a in args.iter_mut() {
                    changed |= expr(a, sums);
                }
            }
            Expr::Binop { lhs, rhs, .. } => {
                changed |= expr(lhs, sums);
                changed |= expr(rhs, sums);
            }
            Expr::Unop { v, .. } => changed |= expr(v, sums),
            Expr::Load { base, .. } => changed |= expr(base, sums),
            Expr::Select { c, a, b } => {
                changed |= expr(c, sums);
                changed |= expr(a, sums);
                changed |= expr(b, sums);
            }
            Expr::Simd { args, .. } => {
                for a in args.iter_mut() {
                    changed |= expr(a, sums);
                }
            }
            _ => {}
        }
        changed
    }
    let mut changed = false;
    for s in stmts.iter_mut() {
        match s {
            Stmt::Assign { expr: e, .. } => changed |= expr(e, sums),
            Stmt::Store { base, value, .. } => {
                changed |= expr(base, sums);
                changed |= expr(value, sums);
            }
            Stmt::ExprStmt(e) => {
                changed |= expr(e, sums);
                // A call reduced to a bare const statement is dead.
                if matches!(e, Expr::ConstI32(_) | Expr::ConstI64(_)) {
                    *s = Stmt::Comment("const-ret: pure call folded away".into());
                    changed = true;
                }
            }
            Stmt::If { cond, then_b, else_b } => {
                changed |= expr(cond, sums);
                changed |= fold_const_calls(then_b, sums);
                changed |= fold_const_calls(else_b, sums);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                changed |= fold_const_calls(body, sums)
            }
            Stmt::BrIf { cond, .. } => changed |= expr(cond, sums),
            Stmt::BrTable { index, .. } => changed |= expr(index, sums),
            Stmt::Return { values } => {
                for v in values.iter_mut() {
                    changed |= expr(v, sums);
                }
            }
            _ => {}
        }
    }
    changed
}

pub fn optimize_func_with(f: &mut FuncIR, opt: &OptConfig) -> AliasMap {
    let mut ctx = PassCtx::default();
    if !opt.fold && !opt.inline && !opt.dce && !opt.simplify {
        return ctx.aliases;
    }
    let fold = FoldPass;
    let inline = InlineDcePass { inline: opt.inline, dce: opt.dce };
    let simp = SimplifyPass;
    let split = SplitInitPass;
    for _ in 0..2 {
        let mut changed = false;
        if opt.fold {
            changed |= fold.run(f, &mut ctx);
        }
        if opt.inline || opt.dce {
            changed |= inline.run(f, &mut ctx);
        }
        if !changed {
            break;
        }
    }
    if opt.simplify {
        simp.run(f, &mut ctx);
    }
    if opt.fold {
        // i64-init split is a value rewrite; gated with folding.
        split.run(f, &mut ctx);
    }
    f.aliases = ctx.aliases.clone();
    ctx.aliases
}


// ---------- purity + depth ----------

/// Simd ops with observable effects (atomics, table/memory mutation).
/// Pure lane arithmetic may inline like any pure expr.
fn simd_has_effect(op: &str) -> bool {
    op.starts_with("atomic.")
        || op.starts_with("memory.")
        || op.starts_with("table.set")
        || op.starts_with("table.grow")
        || op.starts_with("table.fill")
        || op.starts_with("table.init")
        || op.starts_with("struct.set")
        || op.starts_with("array.set")
        || op.starts_with("array.fill")
        || op.starts_with("array.copy")
        || op.starts_with("array.init")
}

/// True if evaluating `e` has no observable effect (safe to duplicate/eliminate).
/// Thin wrapper over [`Expr::is_pure`] kept for compatibility.
pub fn is_pure(e: &Expr) -> bool {
    e.is_pure()
}

/// Expression-tree depth (inlining budget heuristic).
/// Thin wrapper over [`Expr::depth`] kept for compatibility.
pub fn expr_depth(e: &Expr) -> usize {
    e.depth()
}

// ---------- const fold ----------

fn fold_i32(op: &str, a: i32, b: i32) -> Option<i32> {
    match op {
        "+" => Some(a.wrapping_add(b)),
        "-" => Some(a.wrapping_sub(b)),
        "*" => Some(a.wrapping_mul(b)),
        "/" => {
            if b == 0 {
                None
            } else {
                Some(a.wrapping_div(b))
            }
        }
        "%" => {
            if b == 0 {
                None
            } else {
                Some(a.wrapping_rem(b))
            }
        }
        "&" => Some(a & b),
        "|" => Some(a | b),
        "^" => Some(a ^ b),
        "<<" => Some(a.wrapping_shl(b as u32)),
        ">>" => Some(a.wrapping_shr(b as u32)),
        "==" => Some((a == b) as i32),
        "!=" => Some((a != b) as i32),
        "<" => Some((a < b) as i32),
        ">" => Some((a > b) as i32),
        "<=" => Some((a <= b) as i32),
        ">=" => Some((a >= b) as i32),
        _ => None,
    }
}

fn fold_i64(op: &str, a: i64, b: i64) -> Option<i64> {
    match op {
        "+" => Some(a.wrapping_add(b)),
        "-" => Some(a.wrapping_sub(b)),
        "*" => Some(a.wrapping_mul(b)),
        "/" => {
            if b == 0 {
                None
            } else {
                Some(a.wrapping_div(b))
            }
        }
        "%" => {
            if b == 0 {
                None
            } else {
                Some(a.wrapping_rem(b))
            }
        }
        "&" => Some(a & b),
        "|" => Some(a | b),
        "^" => Some(a ^ b),
        "<<" => Some(a.wrapping_shl(b as u32)),
        ">>" => Some(a.wrapping_shr(b as u32)),
        "==" => Some((a == b) as i64),
        "!=" => Some((a != b) as i64),
        "<" => Some((a < b) as i64),
        ">" => Some((a > b) as i64),
        "<=" => Some((a <= b) as i64),
        ">=" => Some((a >= b) as i64),
        _ => None,
    }
}

fn fold_range_idiom(op: &str, l: &Expr, r: &Expr) -> Option<Expr> {
    // Two spellings of the same compare: explicit `(uintN)` casts around a
    // plain `<` (hand-written/lenient producers), or our own `<uN` op which
    // carries the casts implicitly (from `*_lt_u` wasm ops).
    let (w, inner, c2) = if op == "<" {
        match (l, r) {
            (
                Expr::Unop { op: lc, v: li },
                Expr::Unop { op: rc, v: rv },
            ) if lc == "(uint32_t)" && rc == "(uint32_t)" => match **rv {
                Expr::ConstI32(c) => (4u32, &**li, c as u32 as u64),
                _ => return None,
            },
            (
                Expr::Unop { op: lc, v: li },
                Expr::Unop { op: rc, v: rv },
            ) if lc == "(uint64_t)" && rc == "(uint64_t)" => match **rv {
                Expr::ConstI64(c) => (8u32, &**li, c as u64),
                _ => return None,
            },
            _ => return None,
        }
    } else if op == "<u32" || op == "<u64" {
        let w = if op == "<u32" { 4 } else { 8 };
        let c = match (w, r) {
            (4, Expr::ConstI32(c)) => *c as u32 as u64,
            (8, Expr::ConstI64(c)) => *c as u64,
            _ => return None,
        };
        (w as u32, l, c)
    } else {
        return None;
    };
    let mask: u64 = if w == 4 { 0xffff_ffff } else { 0xffff_ffff_ffff_ffff };
    // Inner must be `x - C1` or `x + (-C1)`; const width must match the
    // compare width (mixed widths can't arise from lifting; refusing them
    // keeps the rewrite provably equivalent).
    let stripped = inner.strip_casts();
    let (x, c1) = match stripped {
        Expr::Binop { op: aop, lhs, rhs } if aop == "-" => {
            let k = match (w, &**rhs) {
                (4, Expr::ConstI32(k)) => *k as u32 as u64,
                (8, Expr::ConstI64(k)) => *k as u64,
                _ => return None,
            };
            (&**lhs, k)
        }
        Expr::Binop { op: aop, lhs, rhs } if aop == "+" => {
            let k = match (w, &**rhs) {
                (4, Expr::ConstI32(k)) => (*k as u32).wrapping_neg() as u64,
                (8, Expr::ConstI64(k)) => (*k as u64).wrapping_neg(),
                _ => return None,
            };
            (&**lhs, k)
        }
        _ => return None,
    };
    match x {
        // `x` is used in the rewrite AS-IS (casts included): duplicating
        // pure casts over a var is sound, while stripping them would change
        // extension semantics (zero- vs sign-extend on negative values).
        _ if matches!(
            x.strip_casts(),
            Expr::Local(_)
                | Expr::Tmp(_)
                | Expr::Global(_)
                | Expr::ConstI32(_)
                | Expr::ConstI64(_)
        ) => {}
        _ => return None,
    }
    // C1 + C2 == 1 (mod 2^w) ⟺ condition is `x == 0 || x > C1-1`.
    if (c1.wrapping_add(c2) & mask) != 1 {
        return None;
    }
    let k = c1.wrapping_sub(1) & mask;
    let (zero, kk) = if w == 4 {
        (Expr::ConstI32(0), Expr::ConstI32(k as u32 as i32))
    } else {
        (Expr::ConstI64(0), Expr::ConstI64(k as i64))
    };
    let gt_op = if w == 4 { ">u32" } else { ">u64" };
    Some(Expr::Binop {
        op: "||".into(),
        lhs: Box::new(Expr::Binop {
            op: "==".into(),
            lhs: Box::new(x.clone()),
            rhs: Box::new(zero),
        }),
        rhs: Box::new(Expr::Binop {
            op: gt_op.into(),
            lhs: Box::new(x.clone()),
            rhs: Box::new(kk),
        }),
    })
}

fn fold_expr(e: Expr) -> Expr {
    match e {
        Expr::Binop { op, lhs, rhs } => {
            let l = Box::new(fold_expr(*lhs));
            let r = Box::new(fold_expr(*rhs));
            // identity rules first (clean address math like `(0) + 1024`)
            match (&op[..], &*l, &*r) {
                ("+", Expr::ConstI32(0), _) => return *r,
                ("+", _, Expr::ConstI32(0)) => return *l,
                ("+", Expr::ConstI64(0), _) => return *r,
                ("+", _, Expr::ConstI64(0)) => return *l,
                ("-", _, Expr::ConstI32(0)) => return *l,
                ("-", _, Expr::ConstI64(0)) => return *l,
                _ => {}
            }
            match (&op[..], &*l, &*r) {
                (_, Expr::ConstI32(a), Expr::ConstI32(b)) => {
                    if let Some(v) = fold_i32(&op, *a, *b) {
                        return Expr::ConstI32(v);
                    }
                }
                (_, Expr::ConstI64(a), Expr::ConstI64(b)) => {
                    if let Some(v) = fold_i64(&op, *a, *b) {
                        return Expr::ConstI64(v);
                    }
                }
                _ => {}
            }
            // Unsigned-wraparound range idiom: clang merges `x == 0 || x > MAX`
            // into `(x-K) <u (2^w-K+1)` (single compare). Recognize
            // `((uintN)(x -/+ C1) < (uintN)(C2))` with C1+C2 == 1 (mod 2^w)
            // and restore the readable disjunction (sqlite3Malloc's
            // `n==0 || n>2147483391`). The `>uN` half keeps UNSIGNED
            // semantics — a signed `>` would diverge for negative x.
            // `x` must be a bare var (no duplicated side effects).
            if let Some(rw) = fold_range_idiom(&op, &l, &r) {
                return rw;
            }
            Expr::Binop { op, lhs: l, rhs: r }
        }
        Expr::Unop { op, v } => {
            let inner = fold_expr(*v);
            if op == "!" {
                if let Expr::ConstI32(c) = inner {
                    return Expr::ConstI32((c == 0) as i32);
                }
                // double negation cancels out
                if let Expr::Unop { op: inner_op, v: inner_v } = inner {
                    if inner_op == "!" {
                        return *inner_v;
                    }
                    return Expr::Unop {
                        op,
                        v: Box::new(Expr::Unop { op: inner_op, v: inner_v }),
                    };
                }
                return Expr::Unop {
                    op,
                    v: Box::new(inner),
                };
            }
            Expr::Unop {
                op,
                v: Box::new(inner),
            }
        }
        Expr::Select { c, a, b } => {
            let fc = Box::new(fold_expr(*c));
            let fa = Box::new(fold_expr(*a));
            let fb = Box::new(fold_expr(*b));
            if let Expr::ConstI32(k) = *fc {
                return if k != 0 { *fa } else { *fb };
            }
            Expr::Select { c: fc, a: fa, b: fb }
        }
        Expr::Simd { op, args } => Expr::Simd {
            op,
            args: args.into_iter().map(fold_expr).collect(),
        },
        Expr::Load { ty, base, offset } => Expr::Load {
            ty,
            base: Box::new(fold_expr(*base)),
            offset,
        },
        Expr::Call { func, name, args } => Expr::Call {
            func,
            name,
            args: args.into_iter().map(fold_expr).collect(),
        },
        Expr::CallIndirect {
            type_idx,
            table,
            index,
            args,
        } => Expr::CallIndirect {
            type_idx,
            table,
            index: Box::new(fold_expr(*index)),
            args: args.into_iter().map(fold_expr).collect(),
        },
        other => other,
    }
}

fn fold_stmts_inplace(stmts: &mut Vec<Stmt>) -> bool {
    let original_len = stmts.len();
    let taken = std::mem::take(stmts);
    let (mut out, changed, _) = fold_block(taken);
    // recurse into kept structured bodies (fold_block already recursed, but
    // keep this shape simple: fold_block handles nesting itself)
    stmts.append(&mut out);
    changed || stmts.len() != original_len
}

fn fold_block(stmts: Vec<Stmt>) -> (Vec<Stmt>, bool, bool) {
    let mut out = Vec::with_capacity(stmts.len());
    let mut changed = false;
    for s in stmts {
        match s {
            Stmt::Assign { dst, expr } => out.push(Stmt::Assign {
                dst,
                expr: fold_expr(expr),
            }),
            Stmt::Store { ty, base, offset, value } => out.push(Stmt::Store {
                ty,
                base: fold_expr(base),
                offset,
                value: fold_expr(value),
            }),
            Stmt::ExprStmt(e) => out.push(Stmt::ExprStmt(fold_expr(e))),
            Stmt::If { cond, then_b, else_b } => {
                let c = fold_expr(cond);
                let (t, ct, _) = fold_block(then_b);
                let (e_, ce, _) = fold_block(else_b);
                changed |= ct || ce;
                if let Expr::ConstI32(k) = c {
                    changed = true;
                    out.extend(if k != 0 { t } else { e_ });
                } else {
                    out.push(Stmt::If {
                        cond: c,
                        then_b: t,
                        else_b: e_,
                    });
                }
            }
            Stmt::Block { label, body } => {
                let (b, c, _) = fold_block(body);
                changed |= c;
                out.push(Stmt::Block { label, body: b });
            }
            Stmt::Loop { label, body } => {
                let (b, c, _) = fold_block(body);
                changed |= c;
                out.push(Stmt::Loop { label, body: b });
            }
            Stmt::BrIf { depth, label, is_loop, cond } => {
                let c = fold_expr(cond);
                if let Expr::ConstI32(k) = c {
                    changed = true;
                    if k != 0 {
                        out.push(Stmt::Br { depth, label, is_loop });
                    }
                    // cond 0 -> drop
                } else {
                    out.push(Stmt::BrIf { depth, label, is_loop, cond: c });
                }
            }
            Stmt::BrTable { index, targets, default } => out.push(Stmt::BrTable {
                index: fold_expr(index),
                targets,
                default,
            }),
            Stmt::Return { values } => out.push(Stmt::Return {
                values: values.into_iter().map(fold_expr).collect(),
            }),
            Stmt::Comment(_) | Stmt::Br { .. } => out.push(s),
        }
    }
    (out, changed, false)
}

// ---------- use counts / inline / DCE ----------

fn count_uses_expr(e: &Expr, map: &mut HashMap<Var, usize>) {
    match e {
        Expr::Local(i) => *map.entry(Var::Local(*i)).or_default() += 1,
        Expr::Tmp(i) => *map.entry(Var::Tmp(*i)).or_default() += 1,
        Expr::Global(i) => *map.entry(Var::Global(*i)).or_default() += 1,
        Expr::Binop { lhs, rhs, .. } => {
            count_uses_expr(lhs, map);
            count_uses_expr(rhs, map);
        }
        Expr::Unop { v, .. } => count_uses_expr(v, map),
        Expr::Load { base, .. } => count_uses_expr(base, map),
        Expr::Call { args, .. } => {
            for a in args {
                count_uses_expr(a, map);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            count_uses_expr(index, map);
            for a in args {
                count_uses_expr(a, map);
            }
        }
        Expr::Select { c, a, b } => {
            count_uses_expr(c, map);
            count_uses_expr(a, map);
            count_uses_expr(b, map);
        }
        Expr::Simd { args, .. } => {
            for a in args {
                count_uses_expr(a, map);
            }
        }
        _ => {}
    }
}

/// Count `Var` mentions per statement list (pass-author toolkit).
pub fn count_uses_stmts(stmts: &[Stmt], map: &mut HashMap<Var, usize>) {
    for s in stmts {
        match s {
            Stmt::Assign { expr, .. } => count_uses_expr(expr, map),
            Stmt::Store { base, value, .. } => {
                count_uses_expr(base, map);
                count_uses_expr(value, map);
            }
            Stmt::ExprStmt(e) => count_uses_expr(e, map),
            Stmt::If { cond, then_b, else_b } => {
                count_uses_expr(cond, map);
                count_uses_stmts(then_b, map);
                count_uses_stmts(else_b, map);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => count_uses_stmts(body, map),
            Stmt::BrIf { cond, .. } => count_uses_expr(cond, map),
            Stmt::BrTable { index, .. } => count_uses_expr(index, map),
            Stmt::Return { values } => {
                for v in values {
                    count_uses_expr(v, map);
                }
            }
            _ => {}
        }
    }
}

/// Like collect_assigns but tracks whether each definition sits inside a
/// `Loop` body (loop-carried redefinition risk for copyprop).
fn collect_assigns_loop(
    stmts: &[Stmt],
    in_loop: bool,
    map: &mut HashMap<Var, (Expr, usize, bool)>,
) {
    for s in stmts {
        match s {
            Stmt::Assign { dst, expr } => {
                let Some(v) = Var::parse(dst) else { continue };
                map.entry(v)
                    .and_modify(|(_, n, il)| {
                        *n += 1;
                        *il = *il || in_loop;
                    })
                    .or_insert((expr.clone(), 1, in_loop));
            }
            Stmt::If { then_b, else_b, .. } => {
                collect_assigns_loop(then_b, in_loop, map);
                collect_assigns_loop(else_b, in_loop, map);
            }
            Stmt::Block { body, .. } => collect_assigns_loop(body, in_loop, map),
            Stmt::Loop { body, .. } => collect_assigns_loop(body, true, map),
            _ => {}
        }
    }
}

fn dce_list(stmts: &mut Vec<Stmt>, uses: &HashMap<Var, usize>) -> bool {
    let mut changed = false;
    let mut i = 0;
    while i < stmts.len() {
        // recurse first
        match &mut stmts[i] {
            Stmt::If { then_b, else_b, .. } => {
                changed |= dce_list(then_b, uses);
                changed |= dce_list(else_b, uses);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                changed |= dce_list(body, uses);
            }
            _ => {}
        }
        if let Stmt::Assign { dst, expr } = &stmts[i] {
            // Only locals/tmps participate: globals (gN) are visible
            // cross-function (e.g. the g0 stack pointer), so their
            // assignments are always kept.
            let v = Var::parse(dst);
            let is_local = matches!(v, Some(Var::Local(_)) | Some(Var::Tmp(_)));
            let used = v.and_then(|v| uses.get(&v)).copied().unwrap_or(0);
            if is_local && used == 0 {
                let expr = expr.clone();
                let dst_c = dst.clone();
                if is_pure(&expr) {
                    stmts.remove(i);
                } else {
                    // keep effect, drop dead destination
                    stmts[i] = Stmt::ExprStmt(expr);
                }
                changed = true;
                let _ = dst_c;
                continue;
            }
        }
        i += 1;
    }
    changed
}

/// Variable slots mentioned in `e` (pass-author toolkit).
pub fn rep_vars(e: &Expr, acc: &mut Vec<Var>) {
    e.vars(acc);
}

/// Pre-order statement sequence numbers for def/use order checks:
/// `defs[name]` = seqs of `name = ...` assigns, `uses[name]` = seqs of
/// statements whose expressions mention `name` (including a self-copy RHS).
fn collect_seq_expr(e: &Expr, seq: usize, uses: &mut HashMap<Var, Vec<usize>>) {
    match e {
        Expr::Local(i) => uses.entry(Var::Local(*i)).or_default().push(seq),
        Expr::Tmp(i) => uses.entry(Var::Tmp(*i)).or_default().push(seq),
        Expr::Global(i) => uses.entry(Var::Global(*i)).or_default().push(seq),
        Expr::Binop { lhs, rhs, .. } => {
            collect_seq_expr(lhs, seq, uses);
            collect_seq_expr(rhs, seq, uses);
        }
        Expr::Unop { v, .. } => collect_seq_expr(v, seq, uses),
        Expr::Load { base, .. } => collect_seq_expr(base, seq, uses),
        Expr::Call { args, .. } => {
            for a in args {
                collect_seq_expr(a, seq, uses);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            collect_seq_expr(index, seq, uses);
            for a in args {
                collect_seq_expr(a, seq, uses);
            }
        }
        Expr::Select { c, a, b } => {
            collect_seq_expr(c, seq, uses);
            collect_seq_expr(a, seq, uses);
            collect_seq_expr(b, seq, uses);
        }
        Expr::Simd { args, .. } => {
            for a in args {
                collect_seq_expr(a, seq, uses);
            }
        }
        _ => {}
    }
}

/// Pre-order def/use statement sequence numbers (staleness checks).
pub fn collect_def_use_seq(
    stmts: &[Stmt],
    seq: &mut usize,
    defs: &mut HashMap<Var, Vec<usize>>,
    uses: &mut HashMap<Var, Vec<usize>>,
) {
    for s in stmts {
        let cur = *seq;
        *seq += 1;
        match s {
            Stmt::Assign { dst, expr } => {
                if let Some(v) = Var::parse(dst) {
                    defs.entry(v).or_default().push(cur);
                }
                collect_seq_expr(expr, cur, uses);
            }
            Stmt::Store { base, value, .. } => {
                collect_seq_expr(base, cur, uses);
                collect_seq_expr(value, cur, uses);
            }
            Stmt::ExprStmt(e) => collect_seq_expr(e, cur, uses),
            Stmt::If { cond, then_b, else_b } => {
                collect_seq_expr(cond, cur, uses);
                collect_def_use_seq(then_b, seq, defs, uses);
                collect_def_use_seq(else_b, seq, defs, uses);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                collect_def_use_seq(body, seq, defs, uses)
            }
            Stmt::BrIf { cond, .. } => collect_seq_expr(cond, cur, uses),
            Stmt::BrTable { index, .. } => collect_seq_expr(index, cur, uses),
            Stmt::Return { values } => {
                for v in values {
                    collect_seq_expr(v, cur, uses);
                }
            }
            _ => {}
        }
    }
}
/// Substitute ALL names in `map` in a single walk (perf: one O(n) pass
/// instead of one full rescan per inlined variable). Records which names
/// actually hit so callers only remove dead defs that were consumed.
fn subst_all_in_expr(e: &mut Expr, map: &HashMap<Var, Expr>, hit: &mut HashSet<Var>) -> bool {
    match e {
        Expr::Local(i) => {
            let k = Var::Local(*i);
            if let Some(rep) = map.get(&k) {
                *e = rep.clone();
                hit.insert(k);
                return true;
            }
            false
        }
        Expr::Tmp(i) => {
            let k = Var::Tmp(*i);
            if let Some(rep) = map.get(&k) {
                *e = rep.clone();
                hit.insert(k);
                return true;
            }
            false
        }
        Expr::Binop { lhs, rhs, .. } => {
            subst_all_in_expr(lhs, map, hit) | subst_all_in_expr(rhs, map, hit)
        }
        Expr::Unop { v, .. } => subst_all_in_expr(v, map, hit),
        Expr::Load { base, .. } => subst_all_in_expr(base, map, hit),
        Expr::Call { args, .. } => {
            let mut h = false;
            for a in args.iter_mut() {
                h |= subst_all_in_expr(a, map, hit);
            }
            h
        }
        Expr::CallIndirect { index, args, .. } => {
            let mut h = subst_all_in_expr(index, map, hit);
            for a in args.iter_mut() {
                h |= subst_all_in_expr(a, map, hit);
            }
            h
        }
        Expr::Select { c, a, b } => {
            subst_all_in_expr(c, map, hit)
                | subst_all_in_expr(a, map, hit)
                | subst_all_in_expr(b, map, hit)
        }
        Expr::Simd { args, .. } => {
            let mut h = false;
            for a in args.iter_mut() {
                h |= subst_all_in_expr(a, map, hit);
            }
            h
        }
        _ => false,
    }
}

/// Substitute every name in `map` in one walk; records hits (pass-author toolkit).
pub fn subst_all_in_stmts(
    stmts: &mut [Stmt],
    map: &HashMap<Var, Expr>,
    hit: &mut HashSet<Var>,
) -> bool {
    let mut h = false;
    for s in stmts.iter_mut() {
        match s {
            Stmt::Assign { expr, .. } => h |= subst_all_in_expr(expr, map, hit),
            Stmt::Store { base, value, .. } => {
                h |= subst_all_in_expr(base, map, hit);
                h |= subst_all_in_expr(value, map, hit);
            }
            Stmt::ExprStmt(e) => h |= subst_all_in_expr(e, map, hit),
            Stmt::If { cond, then_b, else_b } => {
                h |= subst_all_in_expr(cond, map, hit);
                h |= subst_all_in_stmts(then_b, map, hit);
                h |= subst_all_in_stmts(else_b, map, hit);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                h |= subst_all_in_stmts(body, map, hit)
            }
            Stmt::BrIf { cond, .. } => h |= subst_all_in_expr(cond, map, hit),
            Stmt::BrTable { index, .. } => h |= subst_all_in_expr(index, map, hit),
            Stmt::Return { values } => {
                for v in values.iter_mut() {
                    h |= subst_all_in_expr(v, map, hit);
                }
            }
            _ => {}
        }
    }
    h
}

/// Remove all dead assigns in one sweep (perf: single O(n) pass instead
/// of one O(n) `Vec::remove` memmove per variable).
pub fn sweep_assigns(stmts: &mut Vec<Stmt>, dead: &HashSet<Var>) -> bool {
    let mut changed = false;
    for s in stmts.iter_mut() {
        match s {
            Stmt::If { then_b, else_b, .. } => {
                changed |= sweep_assigns(then_b, dead);
                changed |= sweep_assigns(else_b, dead);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                changed |= sweep_assigns(body, dead);
            }
            _ => {}
        }
    }
    let before = stmts.len();
    stmts.retain(|s| match s {
        Stmt::Assign { dst, .. } => Var::parse(dst).map_or(true, |v| !dead.contains(&v)),
        _ => true,
    });
    changed || stmts.len() != before
}

/// Positional order check for straight-line bodies (exact there):
/// single def (D) precedes single use (U), no rep-var def strictly between.
fn order_ok_seq(
    def_seqs: &HashMap<Var, Vec<usize>>,
    use_seqs: &HashMap<Var, Vec<usize>>,
    dst: &Var,
    reps: &[Var],
) -> bool {
    match (
        def_seqs.get(dst).and_then(|v| v.first()),
        use_seqs.get(dst).and_then(|v| v.first()),
    ) {
        (Some(&d), Some(&u)) if d <= u => !reps.iter().any(|v| {
            def_seqs.get(v).map_or(false, |ss| {
                ss.iter().any(|&s| s > d && s < u)
            })
        }),
        _ => false,
    }
}

/// Dominance order check for bodies with control flow: the single def
/// block dominates the single use block, and no rep-var def sits on any
/// acyclic def→use path (endpoints excluded, as in the seq check).
/// Skips walks entirely when the expr mentions no vars.
fn cfg_order_ok(g: &cfg::Cfg, dom: &[Vec<u64>], dst: Var, expr: &Expr) -> bool {
    let (Some(d), Some(u)) = (
        g.def_blocks.get(&dst).and_then(|v| match v[..] {
            [only] => Some(only),
            _ => None,
        }),
        g.use_blocks.get(&dst).and_then(|v| match v[..] {
            [only] => Some(only),
            _ => None,
        }),
    ) else {
        return false;
    };
    if !cfg::dominates_bits(dom, d, u) {
        return false;
    }
    let mut reps = Vec::new();
    expr.vars(&mut reps);
    if reps.is_empty() {
        return true;
    }
    // Loop-closing back-edges are ignored (see `is_back_edge`): a def
    // reachable only by looping around executes after the use in
    // iteration order. This restores same-iteration inlining (VM dispatch
    // loops) that plain `blocks_between` rejects via cycle paths, while
    // branch-skipped defs (the `bon` miscompile class) still refuse.
    let between = cfg::blocks_between_acyclic(g, dom, d, u);
    !reps.iter().any(|v| {
        g.def_blocks.get(v).map_or(false, |bs| {
            bs.iter().any(|&b| b != d && b != u && between.contains(&b))
        })
    })
}

/// True if the body contains any control-flow statement (ordering then
/// needs dominance, not positions).
fn has_control_flow(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::If { .. } | Stmt::Block { .. } | Stmt::Loop { .. } => true,
        Stmt::Br { .. } | Stmt::BrIf { .. } | Stmt::BrTable { .. } => true,
        _ => false,
    })
}

fn inline_and_dce(stmts: &mut Vec<Stmt>, opt: &OptConfig, aliases: &mut AliasMap) -> bool {
    let mut changed = false;
    // Batched inlining (perf): one map computation per round, one
    // substitution walk + one removal sweep for the whole batch — the old
    // one-candidate-at-a-time loop was O(V × n) and dominated real cost
    // (2000 single-use tmps: ~4.9s → ~50ms). Candidates whose replacement
    // mentions another candidate are deferred to the next round so staged
    // substitutions never go stale (t1 = f(t0) waits for t0).
    if opt.inline {
        // CFG order-guard setup, hoisted out of the round loop:
        // `has_control_flow` only sees top-level Block/Loop/If/Br, which
        // inline rounds never add/remove (subst touches exprs, sweep drops
        // only Assigns), so the decision is stable across rounds.
        let want_cfg = has_control_flow(stmts);
        // Dominator cache: CFG topology is likewise invariant across inline
        // rounds, so the expensive dom survives; the cheap fingerprint
        // (O(n+e) per round) detects the impossible change and recomputes,
        // preserving exact decisions.
        let mut cached_fp: Option<u64> = None;
        let mut cached_dom: Vec<Vec<u64>> = Vec::new();
        loop {
            let mut uses = HashMap::new();
            count_uses_stmts(stmts, &mut uses);
            let mut assigns = HashMap::new();
            collect_assigns_loop(stmts, false, &mut assigns);
            // Sort by rendered name: same candidate order as the old
            // string-keyed version, so output is byte-identical. Names are
            // rendered once per round, not per comparison (perf: sort_by
            // with inline rendering allocates 2 Strings per comparison).
            let mut names: Vec<(&Var, String)> =
                assigns.keys().map(|v| (v, v.name())).collect();
            names.sort_by(|a, b| a.1.cmp(&b.1));
            let mut batch: Vec<(Var, Expr)> = Vec::new();
            // Stage 1: cheap guards (single def/use, loop, inlineable).
            let mut maybe: Vec<(Var, Expr)> = Vec::new();
            for (dst, _) in names {
                // Only locals/tmps inline: globals (gN) are cross-function
                // visible, so their assignments always stand (as before).
                if !matches!(dst, Var::Local(_) | Var::Tmp(_)) {
                    continue;
                }
                let (expr, n, in_loop) = &assigns[dst];
                if *n != 1 || uses.get(dst).copied().unwrap_or(0) != 1 || expr.depth() > 3 {
                    continue;
                }
                // Locals propagate only when the single definition sits
                // outside any loop (no loop-carried redefinition).
                if !dst.is_tmp() && *in_loop {
                    continue;
                }
                let inlineable = expr.is_pure()
                    || matches!(
                        expr,
                        Expr::Call { .. } | Expr::CallIndirect { .. } | Expr::Load { .. }
                    )
                    || matches!(expr, Expr::Simd { op, args }
                        if args.iter().all(|a| a.is_pure()) && !simd_has_effect(op));
                if inlineable {
                    maybe.push((*dst, expr.clone()));
                }
            }
            if maybe.is_empty() {
                break;
            }
            // Stage 2: order guard. Straight-line bodies use the positional
            // check (exact there, and free); control flow gets real
            // dominance from the CFG (seq numbers lie across branches).
            // Bitset + RPO dataflow with RPO convergence and per-round dom
            // reuse (topology never changes here) keeps even ~1000-deep
            // dispatch scaffolding cheap. See docs/CFG_FULL_PLAN.md Phase B.
            if want_cfg {
                let g = cfg::build(stmts);
                let fp = cfg::topo_fingerprint(&g);
                if cached_fp != Some(fp) {
                    cached_dom = cfg::dominators_bitset(&g);
                    cached_fp = Some(fp);
                }
                for (dst, expr) in maybe {
                    if cfg_order_ok(&g, &cached_dom, dst, &expr) {
                        batch.push((dst, expr));
                    }
                }
            } else {
                // Seq maps computed lazily only for the seq path (saves a
                // full walk + HashMaps per round when the CFG path is taken).
                let mut def_seqs: HashMap<Var, Vec<usize>> = HashMap::new();
                let mut use_seqs: HashMap<Var, Vec<usize>> = HashMap::new();
                {
                    let mut seq = 0usize;
                    collect_def_use_seq(stmts, &mut seq, &mut def_seqs, &mut use_seqs);
                }
                for (dst, expr) in maybe {
                    let mut reps = Vec::new();
                    expr.vars(&mut reps);
                    if order_ok_seq(&def_seqs, &use_seqs, &dst, &reps) {
                        batch.push((dst, expr));
                    }
                }
            }
            if batch.is_empty() {
                break;
            }
            // Defer candidates whose rep mentions another candidate this
            // round (staged substitution would go stale).
            let batch_names: HashSet<&Var> = batch.iter().map(|(n, _)| n).collect();
            let mut map: HashMap<Var, Expr> = HashMap::new();
            for (name, rep) in &batch {
                let mut reps = Vec::new();
                rep.vars(&mut reps);
                if reps.iter().any(|v| batch_names.contains(v)) {
                    continue;
                }
                map.insert(*name, rep.clone());
            }
            if map.is_empty() {
                // All candidates overlap (dependency chain): fall back to a
                // single first-alphabetical inline to guarantee progress.
                let (name, rep) = batch.into_iter().next().unwrap();
                map.insert(name, rep);
            }
            // record simple local->local aliases for struct-layout keys
            for (name, rep) in &map {
                if let Expr::Local(m) = rep {
                    aliases.insert(name.name(), format!("l{m}"));
                }
            }
            let mut hit = HashSet::new();
            if subst_all_in_stmts(stmts, &map, &mut hit) {
                // Only sweep defs that were actually consumed; a candidate
                // whose single use vanished (DCE interplay) keeps its def.
                sweep_assigns(stmts, &hit);
                changed = true;
            } else {
                break;
            }
        }
    }

    // recompute uses after inlining, then DCE
    if opt.dce {
        let mut uses2 = HashMap::new();
        count_uses_stmts(stmts, &mut uses2);
        changed |= dce_list(stmts, &uses2);
    }
    changed
}

// ---------- block simplify ----------

fn collect_labels(stmts: &[Stmt], set: &mut HashSet<String>) {
    for s in stmts {
        match s {
            Stmt::Br { label, .. } => {
                set.insert(label.clone());
            }
            Stmt::BrIf { label, .. } => {
                set.insert(label.clone());
            }
            Stmt::BrTable { targets, default, .. } => {
                for (_, l, _) in targets {
                    set.insert(l.clone());
                }
                set.insert(default.1.clone());
            }
            Stmt::If { then_b, else_b, .. } => {
                collect_labels(then_b, set);
                collect_labels(else_b, set);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => collect_labels(body, set),
            _ => {}
        }
    }
}

fn simplify_blocks(stmts: &mut Vec<Stmt>) -> bool {
    let mut labels = HashSet::new();
    collect_labels(stmts, &mut labels);
    // fixpoint: flattening pass-through chains exposes more empties.
    // `simplify_list` reports whether it changed anything, so this is one
    // walk per round instead of two full count walks per round (perf R8);
    // the bound is on real progress, not a flat 10000 iterations.
    let mut changed = false;
    for _ in 0..1000 {
        if !simplify_list(stmts, &labels) {
            break;
        }
        changed = true;
    }
    changed
}

fn simplify_list(stmts: &mut Vec<Stmt>, labels: &HashSet<String>) -> bool {
    let mut changed = false;
    // post-order first
    for s in stmts.iter_mut() {
        match s {
            Stmt::If { then_b, else_b, .. } => {
                changed |= simplify_list(then_b, labels);
                changed |= simplify_list(else_b, labels);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                changed |= simplify_list(body, labels)
            }
            _ => {}
        }
    }
    let mut i = 0;
    while i < stmts.len() {
        let drop_me = match &mut stmts[i] {
            Stmt::Block { label, body } => {
                // trailing self-break is redundant (falls through to the label anyway)
                if let Some(Stmt::Br { label: l, is_loop: false, .. }) = body.last() {
                    if l == label {
                        body.pop();
                        changed = true;
                    }
                }
                body.is_empty() && !labels.contains(label)
            }            Stmt::Loop { label, body } => {
                // NOTE: a trailing continue-to-self (`br` to the loop head)
                // must be KEPT. An earlier revision popped it as "redundant",
                // but the emitter lowers fallthrough to loop EXIT (appended
                // `break`), so dropping the back-edge turned loop-back into
                // loop-exit — silent wrong-code on sqlite3_step/VdbeExec.
                // (The Block self-break pop above is genuinely safe: a
                // do/while(0) block falls through to exit anyway.)
                body.is_empty() && !labels.contains(label)
            }
            Stmt::If { then_b, else_b, .. } => then_b.is_empty() && else_b.is_empty(),
            _ => false,
        };
        if drop_me {
            stmts.remove(i);
            changed = true;
            continue;
        }
        // flatten pass-through chains: Block whose only child is a
        // Block/Loop/If and whose own label is unreferenced adds nothing.
        // (Inner labels stay valid; fixpoint in simplify_blocks peels chains.)
        let flatten = match &stmts[i] {
            Stmt::Block { label, body }
                if body.len() == 1 && !labels.contains(label) =>
            {
                matches!(
                    body[0],
                    Stmt::Block { .. } | Stmt::Loop { .. } | Stmt::If { .. }
                )
            }
            _ => false,
        };
        if flatten {
            let inner = match stmts.remove(i) {
                Stmt::Block { body, .. } => body.into_iter().next().unwrap(),
                _ => unreachable!(),
            };
            stmts.insert(i, inner);
            changed = true;
            // re-examine the spliced statement (don't advance)
        } else {
            i += 1;
        }
    }
    changed
}

// ---------- struct-init split ----------

fn split_init_stores(f: &mut FuncIR) -> bool {
    // offsets with ANY i32 (4-byte) access evidence (raw accesses, not merged
    // widths: the i64 store itself would otherwise widen the field to 8).
    let mut i32_offs: HashMap<Var, HashSet<u64>> = HashMap::new();
    for a in &f.accesses {
        // `Var::parse` rejects the "expr" bucket and anything non-var.
        let Some(k) = Var::parse(&a.base_desc) else {
            continue;
        };
        if a.width == 4 {
            i32_offs.entry(k).or_default().insert(a.offset);
        }
    }
    if i32_offs.is_empty() {
        return false;
    }
    split_in_list(&mut f.body, &i32_offs)
}

fn base_key(e: &Expr) -> Option<Var> {
    match e {
        Expr::Local(i) => Some(Var::Local(*i)),
        Expr::Tmp(i) => Some(Var::Tmp(*i)),
        Expr::Global(i) => Some(Var::Global(*i)),
        _ => None,
    }
}

fn split_in_list(stmts: &mut Vec<Stmt>, fields: &HashMap<Var, HashSet<u64>>) -> bool {
    let mut changed = false;
    let mut i = 0;
    while i < stmts.len() {
        match &mut stmts[i] {
            Stmt::If { then_b, else_b, .. } => {
                changed |= split_in_list(then_b, fields);
                changed |= split_in_list(else_b, fields);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                changed |= split_in_list(body, fields)
            }
            _ => {}
        }
        let replacement = match &stmts[i] {
            Stmt::Store { ty, base, offset, value } => {
                let is_i64 = ty.contains("i64") && !ty.contains("w1") && !ty.contains("w2") && !ty.contains("w4") || ty == "i64/w8";
                if !is_i64 {
                    None
                } else if let (Some(k), Expr::ConstI64(v)) = (base_key(base), value) {
                    let set = fields.get(&k);
                    if set.map(|s| s.contains(offset) && s.contains(&(offset + 4))).unwrap_or(false) {
                        let v = *v;
                        let lo = (v as u32) as i32;
                        let hi = ((v >> 32) as u32) as i32;
                        let b = base.clone();
                        Some(vec![
                            Stmt::Comment(format!("split i64 init {v:#x} into 2xi32 LE")),
                            Stmt::Store {
                                ty: "i32/w4".into(),
                                base: b.clone(),
                                offset: *offset,
                                value: Expr::ConstI32(lo),
                            },
                            Stmt::Store {
                                ty: "i32/w4".into(),
                                base: b,
                                offset: offset + 4,
                                value: Expr::ConstI32(hi),
                            },
                        ])
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(stmts2) = replacement {
            let n = stmts2.len();
            stmts.splice(i..i + 1, stmts2);
            i += n;
            changed = true;
        } else {
            i += 1;
        }
    }
    changed
}
