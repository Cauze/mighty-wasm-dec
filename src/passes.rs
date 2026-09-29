//! Phase-4/6 cleanup passes: const-fold, copy/DCE + tmp inline,
//! label-aware block simplify, struct-init split. All accuracy-preserving.

use std::collections::{HashMap, HashSet};

use crate::ir::{Expr, FuncIR, Stmt};

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

pub fn optimize_func_with(f: &mut FuncIR, opt: &OptConfig) -> AliasMap {
    let mut aliases = HashMap::new();
    if !opt.fold && !opt.inline && !opt.dce && !opt.simplify {
        return aliases;
    }
    for _ in 0..2 {
        let mut changed = false;
        if opt.fold {
            changed |= fold_stmts_inplace(&mut f.body);
        }
        if opt.inline || opt.dce {
            changed |= inline_and_dce(&mut f.body, opt, &mut aliases);
        }
        if !changed {
            break;
        }
    }
    if opt.simplify {
        simplify_blocks(&mut f.body);
    }
    if opt.fold {
        // i64-init split is a value rewrite; gated with folding.
        split_init_stores(f);
    }
    f.aliases = aliases.clone();
    aliases
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

fn is_pure(e: &Expr) -> bool {    match e {
        Expr::ConstI32(_)
        | Expr::ConstI64(_)
        | Expr::ConstF32(_)
        | Expr::ConstF64(_)
        | Expr::Local(_)
        | Expr::Tmp(_)
        | Expr::Global(_) => true,
        Expr::Binop { lhs, rhs, .. } => is_pure(lhs) && is_pure(rhs),
        Expr::Unop { v, .. } => is_pure(v),
        Expr::Select { c, a, b } => is_pure(c) && is_pure(a) && is_pure(b),
        _ => false,
    }
}

fn expr_depth(e: &Expr) -> usize {
    match e {
        Expr::Binop { lhs, rhs, .. } => 1 + expr_depth(lhs).max(expr_depth(rhs)),
        Expr::Unop { v, .. } => 1 + expr_depth(v),
        Expr::Select { c, a, b } => 1 + expr_depth(c).max(expr_depth(a).max(expr_depth(b))),
        Expr::Load { base, .. } => 1 + expr_depth(base),
        Expr::Call { args, .. } => 1 + args.iter().map(expr_depth).max().unwrap_or(0),
        Expr::CallIndirect { index, args, .. } => {
            1 + expr_depth(index).max(args.iter().map(expr_depth).max().unwrap_or(0))
        }
        _ => 0,
    }
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

fn count_uses_expr(e: &Expr, map: &mut HashMap<String, usize>) {
    match e {
        Expr::Local(i) => *map.entry(format!("l{i}")).or_default() += 1,
        Expr::Tmp(i) => *map.entry(format!("t{i}")).or_default() += 1,
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

fn count_uses_stmts(stmts: &[Stmt], map: &mut HashMap<String, usize>) {
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
    map: &mut HashMap<String, (Expr, usize, bool)>,
) {
    for s in stmts {
        match s {
            Stmt::Assign { dst, expr } => {
                map.entry(dst.clone())
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

fn dce_list(stmts: &mut Vec<Stmt>, uses: &HashMap<String, usize>) -> bool {
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
            let is_local = dst.starts_with('l') || dst.starts_with('t');
            let used = uses.get(dst).copied().unwrap_or(0);
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

fn rep_var_names(e: &Expr, acc: &mut Vec<String>) {
    match e {
        Expr::Local(i) => acc.push(format!("l{i}")),
        Expr::Tmp(i) => acc.push(format!("t{i}")),
        Expr::Binop { lhs, rhs, .. } => {
            rep_var_names(lhs, acc);
            rep_var_names(rhs, acc);
        }
        Expr::Unop { v, .. } => rep_var_names(v, acc),
        Expr::Load { base, .. } => rep_var_names(base, acc),
        Expr::Call { args, .. } => {
            for a in args {
                rep_var_names(a, acc);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            rep_var_names(index, acc);
            for a in args {
                rep_var_names(a, acc);
            }
        }
        Expr::Select { c, a, b } => {
            rep_var_names(c, acc);
            rep_var_names(a, acc);
            rep_var_names(b, acc);
        }
        Expr::Simd { args, .. } => {
            for a in args {
                rep_var_names(a, acc);
            }
        }
        _ => {}
    }
}

/// Pre-order statement sequence numbers for def/use order checks:
/// `defs[name]` = seqs of `name = ...` assigns, `uses[name]` = seqs of
/// statements whose expressions mention `name` (including a self-copy RHS).
fn collect_seq_expr(e: &Expr, seq: usize, uses: &mut HashMap<String, Vec<usize>>) {
    match e {
        Expr::Local(i) => uses.entry(format!("l{i}")).or_default().push(seq),
        Expr::Tmp(i) => uses.entry(format!("t{i}")).or_default().push(seq),
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

fn collect_def_use_seq(
    stmts: &[Stmt],
    seq: &mut usize,
    defs: &mut HashMap<String, Vec<usize>>,
    uses: &mut HashMap<String, Vec<usize>>,
) {
    for s in stmts {
        let cur = *seq;
        *seq += 1;
        match s {
            Stmt::Assign { dst, expr } => {
                defs.entry(dst.clone()).or_default().push(cur);
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
fn subst_all_in_expr(e: &mut Expr, map: &HashMap<String, Expr>, hit: &mut HashSet<String>) -> bool {
    match e {
        Expr::Local(i) => {
            let k = format!("l{i}");
            if let Some(rep) = map.get(&k) {
                *e = rep.clone();
                hit.insert(k);
                return true;
            }
            false
        }
        Expr::Tmp(i) => {
            let k = format!("t{i}");
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

fn subst_all_in_stmts(
    stmts: &mut [Stmt],
    map: &HashMap<String, Expr>,
    hit: &mut HashSet<String>,
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

/// Remove all dead `name = ...` assigns in one sweep (perf: single O(n)
/// pass instead of one O(n) `Vec::remove` memmove per variable).
fn sweep_assigns(stmts: &mut Vec<Stmt>, dead: &HashSet<String>) -> bool {
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
        Stmt::Assign { dst, .. } => !dead.contains(dst),
        _ => true,
    });
    changed || stmts.len() != before
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
        loop {
            let mut uses = HashMap::new();
            count_uses_stmts(stmts, &mut uses);
            let mut assigns = HashMap::new();
            collect_assigns_loop(stmts, false, &mut assigns);
            // def/use sequence numbers for the order guard below
            let mut def_seqs: HashMap<String, Vec<usize>> = HashMap::new();
            let mut use_seqs: HashMap<String, Vec<usize>> = HashMap::new();
            {
                let mut seq = 0usize;
                collect_def_use_seq(stmts, &mut seq, &mut def_seqs, &mut use_seqs);
            }
            let mut names: Vec<&String> = assigns.keys().collect();
            names.sort();
            let mut batch: Vec<(String, Expr)> = Vec::new();
            for dst in names {
                let (expr, n, in_loop) = &assigns[dst];
                if *n != 1 || uses.get(dst).copied().unwrap_or(0) != 1 || expr_depth(expr) > 3 {
                    continue;
                }
                let is_tmp = dst.starts_with('t');
                // Locals propagate only when the single definition sits
                // outside any loop (no loop-carried redefinition).
                if !is_tmp && *in_loop {
                    continue;
                }
                // Order guard (accuracy): the single def (D) must precede
                // the single use (U), and no redefinition of any variable
                // mentioned in the replacement may sit strictly between
                // them — else the inlined value is stale (intops: l3=l2
                // with l2 clobbered turned q+r into r+r; same for param
                // sources clobbered after the copy).
                let order_ok = match (
                    def_seqs.get(dst.as_str()).and_then(|v| v.first()),
                    use_seqs.get(dst.as_str()).and_then(|v| v.first()),
                ) {
                    (Some(&d), Some(&u)) if d <= u => {
                        let mut reps = Vec::new();
                        rep_var_names(expr, &mut reps);
                        !reps.iter().any(|v| {
                            def_seqs.get(v.as_str()).map_or(false, |ss| {
                                ss.iter().any(|&s| s > d && s < u)
                            })
                        })
                    }
                    _ => false,
                };
                if !order_ok {
                    continue;
                }
                let inlineable = is_pure(expr)
                    || matches!(
                        expr,
                        Expr::Call { .. } | Expr::CallIndirect { .. } | Expr::Load { .. }
                    )
                    || matches!(expr, Expr::Simd { op, args }
                        if args.iter().all(is_pure) && !simd_has_effect(op));
                if inlineable {
                    batch.push(((*dst).clone(), expr.clone()));
                }
            }
            if batch.is_empty() {
                break;
            }
            // Defer candidates whose rep mentions another candidate this
            // round (staged substitution would go stale).
            let batch_names: HashSet<&str> = batch.iter().map(|(n, _)| n.as_str()).collect();
            let mut map: HashMap<String, Expr> = HashMap::new();
            for (name, rep) in &batch {
                let mut reps = Vec::new();
                rep_var_names(rep, &mut reps);
                if reps.iter().any(|v| batch_names.contains(v.as_str())) {
                    continue;
                }
                map.insert(name.clone(), rep.clone());
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
                    aliases.insert(name.clone(), format!("l{m}"));
                }
            }
            let mut hit = HashSet::new();
            if subst_all_in_stmts(stmts, &map, &mut hit) {
                // Only sweep defs that were actually consumed; a candidate
                // whose single use vanished (DCE interplay) keeps its def.
                let dead: HashSet<String> = hit.into_iter().collect();
                sweep_assigns(stmts, &dead);
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

fn simplify_blocks(stmts: &mut Vec<Stmt>) {
    let mut labels = HashSet::new();
    collect_labels(stmts, &mut labels);
    // fixpoint: flattening pass-through chains exposes more empties.
    // `simplify_list` reports whether it changed anything, so this is one
    // walk per round instead of two full count walks per round (perf R8);
    // the bound is on real progress, not a flat 10000 iterations.
    for _ in 0..1000 {
        if !simplify_list(stmts, &labels) {
            break;
        }
    }
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
                // trailing continue-to-self is redundant
                if let Some(Stmt::Br { label: l, is_loop: true, .. }) = body.last() {
                    if l == label {
                        body.pop();
                        changed = true;
                    }
                }
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

fn split_init_stores(f: &mut FuncIR) {
    // offsets with ANY i32 (4-byte) access evidence (raw accesses, not merged
    // widths: the i64 store itself would otherwise widen the field to 8).
    let mut i32_offs: HashMap<String, HashSet<u64>> = HashMap::new();
    for a in &f.accesses {
        if a.base_desc == "expr" {
            continue;
        }
        if a.width == 4 {
            i32_offs
                .entry(a.base_desc.clone())
                .or_default()
                .insert(a.offset);
        }
    }
    if i32_offs.is_empty() {
        return;
    }
    split_in_list(&mut f.body, &i32_offs);
}

fn base_key(e: &Expr) -> Option<String> {
    match e {
        Expr::Local(i) => Some(format!("l{i}")),
        Expr::Tmp(i) => Some(format!("t{i}")),
        Expr::Global(i) => Some(format!("g{i}")),
        _ => None,
    }
}

fn split_in_list(stmts: &mut Vec<Stmt>, fields: &HashMap<String, HashSet<u64>>) {
    let mut i = 0;
    while i < stmts.len() {
        match &mut stmts[i] {
            Stmt::If { then_b, else_b, .. } => {
                split_in_list(then_b, fields);
                split_in_list(else_b, fields);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => split_in_list(body, fields),
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
        } else {
            i += 1;
        }
    }
}
