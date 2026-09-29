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

fn subst_in_expr(e: &mut Expr, name: &str, rep: &Expr) -> bool {
    let mut hit = false;
    match e {
        Expr::Local(i) if format!("l{i}") == name => {
            *e = rep.clone();
            hit = true;
        }
        Expr::Tmp(i) if format!("t{i}") == name => {
            *e = rep.clone();
            hit = true;
        }
        Expr::Binop { lhs, rhs, .. } => {
            hit |= subst_in_expr(lhs, name, rep);
            hit |= subst_in_expr(rhs, name, rep);
        }
        Expr::Unop { v, .. } => hit |= subst_in_expr(v, name, rep),
        Expr::Load { base, .. } => hit |= subst_in_expr(base, name, rep),
        Expr::Call { args, .. } => {
            for a in args.iter_mut() {
                hit |= subst_in_expr(a, name, rep);
            }
        }
        Expr::CallIndirect { index, args, .. } => {
            hit |= subst_in_expr(index, name, rep);
            for a in args.iter_mut() {
                hit |= subst_in_expr(a, name, rep);
            }
        }
        Expr::Select { c, a, b } => {
            hit |= subst_in_expr(c, name, rep);
            hit |= subst_in_expr(a, name, rep);
            hit |= subst_in_expr(b, name, rep);
        }
        Expr::Simd { args, .. } => {
            for a in args.iter_mut() {
                hit |= subst_in_expr(a, name, rep);
            }
        }
        _ => {}
    }
    hit
}

fn subst_in_stmts(stmts: &mut [Stmt], name: &str, rep: &Expr) -> bool {
    let mut hit = false;
    for s in stmts.iter_mut() {
        match s {
            Stmt::Assign { expr, .. } => hit |= subst_in_expr(expr, name, rep),
            Stmt::Store { base, value, .. } => {
                hit |= subst_in_expr(base, name, rep);
                hit |= subst_in_expr(value, name, rep);
            }
            Stmt::ExprStmt(e) => hit |= subst_in_expr(e, name, rep),
            Stmt::If { cond, then_b, else_b } => {
                hit |= subst_in_expr(cond, name, rep);
                hit |= subst_in_stmts(then_b, name, rep);
                hit |= subst_in_stmts(else_b, name, rep);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                hit |= subst_in_stmts(body, name, rep)
            }
            Stmt::BrIf { cond, .. } => hit |= subst_in_expr(cond, name, rep),
            Stmt::BrTable { index, .. } => hit |= subst_in_expr(index, name, rep),
            Stmt::Return { values } => {
                for v in values.iter_mut() {
                    hit |= subst_in_expr(v, name, rep);
                }
            }
            _ => {}
        }
    }
    hit
}

/// Remove the (single) `name = ...` assign. Returns true if removed.
fn remove_assign(stmts: &mut Vec<Stmt>, name: &str) -> bool {
    let mut i = 0;
    while i < stmts.len() {
        let is_target = matches!(&stmts[i], Stmt::Assign { dst, .. } if dst == name);
        if is_target {
            stmts.remove(i);
            return true;
        }
        // recurse
        match &mut stmts[i] {
            Stmt::If { then_b, else_b, .. } => {
                if remove_assign(then_b, name) || remove_assign(else_b, name) {
                    return true;
                }
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
                if remove_assign(body, name) {
                    return true;
                }
            }
            _ => {}
        }
        i += 1;
    }
    false
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

fn inline_and_dce(stmts: &mut Vec<Stmt>, opt: &OptConfig, aliases: &mut AliasMap) -> bool {
    let mut changed = false;
    // Apply one inline at a time, recomputing maps after each: candidate
    // replacement exprs go stale otherwise (e.g. t1 = f(t0) inlined after
    // t0 was already substituted leaves a dangling t0).
    if opt.inline {
        loop {
            let mut uses = HashMap::new();
            count_uses_stmts(stmts, &mut uses);
            let mut assigns = HashMap::new();
            collect_assigns_loop(stmts, false, &mut assigns);
            let mut cand: Option<(String, Expr)> = None;
            let mut names: Vec<&String> = assigns.keys().collect();
            names.sort();
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
                let inlineable = is_pure(expr)
                    || matches!(
                        expr,
                        Expr::Call { .. } | Expr::CallIndirect { .. } | Expr::Load { .. }
                    )
                    || matches!(expr, Expr::Simd { op, args }
                        if args.iter().all(is_pure) && !simd_has_effect(op));
                if inlineable {
                    cand = Some((dst.clone(), expr.clone()));
                    break;
                }
            }
            match cand {
                Some((name, rep)) => {
                    // record simple local->local aliases for struct-layout keys
                    if let Expr::Local(m) = &rep {
                        aliases.insert(name.clone(), format!("l{m}"));
                    }
                    if subst_in_stmts(stmts, &name, &rep) {
                        remove_assign(stmts, &name);
                        changed = true;
                    } else {
                        break;
                    }
                }
                None => break,
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
    // fixpoint: flattening pass-through chains exposes more empties
    for _ in 0..10000 {
        let n_before = count_stmts(stmts);
        simplify_list(stmts, &labels);
        if count_stmts(stmts) == n_before {
            break;
        }
    }
}

fn count_stmts(stmts: &[Stmt]) -> usize {
    stmts
        .iter()
        .map(|s| match s {
            Stmt::If { then_b, else_b, .. } => 1 + count_stmts(then_b) + count_stmts(else_b),
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => 1 + count_stmts(body),
            _ => 1,
        })
        .sum()
}

fn simplify_list(stmts: &mut Vec<Stmt>, labels: &HashSet<String>) {
    // post-order first
    for s in stmts.iter_mut() {
        match s {
            Stmt::If { then_b, else_b, .. } => {
                simplify_list(then_b, labels);
                simplify_list(else_b, labels);
            }
            Stmt::Block { body, .. } | Stmt::Loop { body, .. } => simplify_list(body, labels),
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
                    }
                }
                body.is_empty() && !labels.contains(label)
            }            Stmt::Loop { label, body } => {
                // trailing continue-to-self is redundant
                if let Some(Stmt::Br { label: l, is_loop: true, .. }) = body.last() {
                    if l == label {
                        body.pop();
                    }
                }
                body.is_empty() && !labels.contains(label)
            }
            Stmt::If { then_b, else_b, .. } => then_b.is_empty() && else_b.is_empty(),
            _ => false,
        };
        if drop_me {
            stmts.remove(i);
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
            // re-examine the spliced statement (don't advance)
        } else {
            i += 1;
        }
    }
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
