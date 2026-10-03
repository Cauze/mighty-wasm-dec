//! Control-flow graph for a lifted function: flat blocks + terminators.
//!
//! Passes reason about nested `Vec<Stmt>` today (pre-order sequence
//! numbers), which breaks across branches. The CFG gives real dominance
//! queries for copyprop/DCE without touching the structured IR the
//! emitter consumes: build with [`build`], then ask [`dominators`],
//! [`dominates`], [`blocks_between`].

use std::collections::{HashMap, HashSet};

use crate::ir::{Expr, Stmt, Var};

/// CFG node index.
pub type BlockId = usize;

/// Straight-line statements only; control flow lives in [`Terminator`].
/// `dst` stays `String` (IR boundary — parse to [`Var`] when keying maps).
#[derive(Debug, Clone)]
pub enum FlatStmt {
    Assign { dst: String, expr: Expr },
    Store { ty: String, base: Expr, offset: u64, value: Expr },
    ExprStmt(Expr),
    Comment(String),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    /// Fallthrough to the next block (also used for structured joins).
    Next(BlockId),
    Jump(BlockId),
    Branch {
        cond: Expr,
        then_b: BlockId,
        else_b: BlockId,
    },
    Switch {
        index: Expr,
        arms: Vec<BlockId>,
        default: BlockId,
    },
    Return(Vec<Expr>),
    /// Dead code after an unconditional transfer, or `unreachable`.
    /// Placeholder while edges resolve; `resolve` replaces what it can.
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct CfgBlock {
    pub id: BlockId,
    pub stmts: Vec<FlatStmt>,
    /// Wasm label of the structured construct this block belongs to
    /// (empty for synthetic join blocks).
    pub label: String,
    pub term: Terminator,
    pub preds: Vec<BlockId>,
}

#[derive(Debug, Clone, Default)]
pub struct Cfg {
    pub blocks: Vec<CfgBlock>,
    pub entry: BlockId,
    /// Wasm label → block that *follows* the construct end (`__end_`).
    pub end_of: HashMap<String, BlockId>,
    /// Wasm label → loop-head block (`__head_`).
    pub head_of: HashMap<String, BlockId>,
    /// Var → blocks containing a def.
    pub def_blocks: HashMap<Var, Vec<BlockId>>,
    /// Var → blocks containing a use (expression mention).
    pub use_blocks: HashMap<Var, Vec<BlockId>>,
}

struct Builder {
    blocks: Vec<CfgBlock>,
    end_of: HashMap<String, BlockId>,
    head_of: HashMap<String, BlockId>,
    def_blocks: HashMap<Var, Vec<BlockId>>,
    use_blocks: HashMap<Var, Vec<BlockId>>,
    /// Pending plain jumps: (from, label, is_loop).
    jumps: Vec<(BlockId, String, bool)>,
    /// Pending conditional-taken edges: (from, label, is_loop).
    branches: Vec<(BlockId, String, bool)>,
    /// Pending switch arms: (case_block, label, is_loop).
    cases: Vec<(BlockId, String, bool)>,
    dead: BlockId,
}

/// `dst = dst` copies (tee-lowering residue) carry no new value.
fn is_self_assign(dst: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Local(i) => Var::parse(dst) == Some(Var::Local(*i)),
        Expr::Tmp(i) => Var::parse(dst) == Some(Var::Tmp(*i)),
        Expr::Global(i) => Var::parse(dst) == Some(Var::Global(*i)),
        _ => false,
    }
}

impl Builder {
    fn new() -> Self {        let mut b = Builder {
            blocks: Vec::new(),
            end_of: HashMap::new(),
            head_of: HashMap::new(),
            def_blocks: HashMap::new(),
            use_blocks: HashMap::new(),
            jumps: Vec::new(),
            branches: Vec::new(),
            cases: Vec::new(),
            dead: 0,
        };
        b.dead = b.fresh_block("dead".into());
        b.blocks[b.dead].term = Terminator::Unreachable;
        b
    }

    fn fresh_block(&mut self, label: String) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(CfgBlock {
            id,
            stmts: Vec::new(),
            label,
            term: Terminator::Unreachable,
            preds: Vec::new(),
        });
        id
    }

    fn note_def(&mut self, dst: &str, bid: BlockId) {
        if let Some(v) = Var::parse(dst) {
            self.def_blocks.entry(v).or_default().push(bid);
        }
    }

    fn note_uses_expr(&mut self, e: &Expr, bid: BlockId) {
        let mut vs = Vec::new();
        e.vars(&mut vs);
        for v in vs {
            self.use_blocks.entry(v).or_default().push(bid);
        }
    }

    fn push_flat(&mut self, bid: BlockId, s: FlatStmt) {
        match &s {
            FlatStmt::Assign { dst, expr } => {
                // Self-copies (`l0 = l0`, tee-lowering residue) are not
                // redefinitions: recording them as defs makes staleness
                // checks refuse valid inlines (fib's `t0` across a loop
                // head full of self-copies). The emitter already skips them.
                if !is_self_assign(dst, expr) {
                    self.note_def(dst, bid);
                }
                self.note_uses_expr(expr, bid);
            }
            FlatStmt::Store { base, value, .. } => {
                self.note_uses_expr(base, bid);
                self.note_uses_expr(value, bid);
            }
            FlatStmt::ExprStmt(e) => self.note_uses_expr(e, bid),
            FlatStmt::Comment(_) => {}
        }
        self.blocks[bid].stmts.push(s);
    }

    fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if !self.blocks[to].preds.contains(&from) {
            self.blocks[to].preds.push(from);
        }
    }

    fn target_of(&self, label: &str, is_loop: bool) -> Option<BlockId> {
        if is_loop {
            self.head_of.get(label).copied()
        } else {
            self.end_of.get(label).copied()
        }
    }

    /// Second pass: turn pending label jumps into block edges.
    fn resolve(&mut self) {
        let jumps = std::mem::take(&mut self.jumps);
        for (from, label, is_loop) in jumps {
            if let Some(t) = self.target_of(&label, is_loop) {
                self.blocks[from].term = Terminator::Jump(t);
                self.add_edge(from, t);
            }
        }
        let branches = std::mem::take(&mut self.branches);
        for (from, label, is_loop) in branches {
            if let Some(t) = self.target_of(&label, is_loop) {
                if let Terminator::Branch { then_b, .. } = &mut self.blocks[from].term {
                    *then_b = t;
                }
                self.add_edge(from, t);
            }
        }
        let cases = std::mem::take(&mut self.cases);
        for (case_block, label, is_loop) in cases {
            if let Some(t) = self.target_of(&label, is_loop) {
                self.blocks[case_block].term = Terminator::Jump(t);
                self.add_edge(case_block, t);
            }
        }
    }
}

/// Build a CFG for a statement list (usually a function body). Total:
/// every statement lands in exactly one block; every resolvable branch
/// becomes an edge.
pub fn build(body: &[Stmt]) -> Cfg {
    let mut b = Builder::new();
    let entry = b.fresh_block(String::new());
    let exit = b.fresh_block("exit".into());
    let mut cur = entry;
    let mut live = true;
    build_list(body, &mut b, &mut cur, &mut live);
    // Fallthrough off the end reaches the exit block.
    if live && matches!(b.blocks[cur].term, Terminator::Unreachable) {
        b.blocks[cur].term = Terminator::Next(exit);
        b.add_edge(cur, exit);
    }
    b.blocks[exit].term = Terminator::Return(Vec::new());
    b.resolve();
    Cfg {
        blocks: b.blocks,
        entry,
        end_of: b.end_of,
        head_of: b.head_of,
        def_blocks: b.def_blocks,
        use_blocks: b.use_blocks,
    }
}

fn link_next(b: &mut Builder, from: BlockId, to: BlockId) {
    b.blocks[from].term = Terminator::Next(to);
    b.add_edge(from, to);
}

/// Linearize `stmts` starting at `*cur`. On return, `*cur` is the block
/// fallthrough continues in and `*live` says whether it is reachable.
fn build_list(stmts: &[Stmt], b: &mut Builder, cur: &mut BlockId, live: &mut bool) {
    for s in stmts {
        if !*live {
            // Dead code after an unconditional transfer: park it in the
            // dead block so coverage stays total but it poisons nothing.
            push_dead(s, b);
            continue;
        }
        match s {
            Stmt::Assign { dst, expr } => b.push_flat(
                *cur,
                FlatStmt::Assign { dst: dst.clone(), expr: expr.clone() },
            ),
            Stmt::Store { ty, base, offset, value } => b.push_flat(
                *cur,
                FlatStmt::Store {
                    ty: ty.clone(),
                    base: base.clone(),
                    offset: *offset,
                    value: value.clone(),
                },
            ),
            Stmt::ExprStmt(e) => b.push_flat(*cur, FlatStmt::ExprStmt(e.clone())),
            Stmt::Comment(c) => b.push_flat(*cur, FlatStmt::Comment(c.clone())),
            Stmt::If { cond, then_b, else_b } => {
                let then_id = b.fresh_block(String::new());
                let else_id = b.fresh_block(String::new());
                let join_id = b.fresh_block(String::new());
                b.note_uses_expr(cond, *cur);
                b.blocks[*cur].term = Terminator::Branch {
                    cond: cond.clone(),
                    then_b: then_id,
                    else_b: else_id,
                };
                b.add_edge(*cur, then_id);
                b.add_edge(*cur, else_id);
                let mut t_live = true;
                let mut t_cur = then_id;
                build_list(then_b, b, &mut t_cur, &mut t_live);
                if t_live && matches!(b.blocks[t_cur].term, Terminator::Unreachable) {
                    link_next(b, t_cur, join_id);
                }
                let mut e_live = true;
                let mut e_cur = else_id;
                build_list(else_b, b, &mut e_cur, &mut e_live);
                if e_live && matches!(b.blocks[e_cur].term, Terminator::Unreachable) {
                    link_next(b, e_cur, join_id);
                }
                *cur = join_id;
            }
            Stmt::Block { label, body } => {
                let body_id = b.fresh_block(label.clone());
                let end_id = b.fresh_block(label.clone());
                b.end_of.insert(label.clone(), end_id);
                link_next(b, *cur, body_id);
                let mut b_live = true;
                let mut b_cur = body_id;
                build_list(body, b, &mut b_cur, &mut b_live);
                if b_live && matches!(b.blocks[b_cur].term, Terminator::Unreachable) {
                    link_next(b, b_cur, end_id);
                }
                *cur = end_id;
            }
            Stmt::Loop { label, body } => {
                let head_id = b.fresh_block(label.clone());
                let end_id = b.fresh_block(label.clone());
                b.head_of.insert(label.clone(), head_id);
                b.end_of.insert(label.clone(), end_id);
                link_next(b, *cur, head_id);
                let mut b_live = true;
                let mut b_cur = head_id;
                build_list(body, b, &mut b_cur, &mut b_live);
                if b_live && matches!(b.blocks[b_cur].term, Terminator::Unreachable) {
                    // Wasm `loop` falls through to exit; the C emitter
                    // appends `break` for the same reason.
                    link_next(b, b_cur, end_id);
                }
                *cur = end_id;
            }
            Stmt::Br { label, is_loop, .. } => {
                b.jumps.push((*cur, label.clone(), *is_loop));
                *live = false;
            }
            Stmt::BrIf { label, is_loop, cond, .. } => {
                let cont_id = b.fresh_block(String::new());
                b.note_uses_expr(cond, *cur);
                b.blocks[*cur].term = Terminator::Branch {
                    cond: cond.clone(),
                    then_b: usize::MAX,
                    else_b: cont_id,
                };
                b.add_edge(*cur, cont_id);
                b.branches.push((*cur, label.clone(), *is_loop));
                *cur = cont_id;
            }
            Stmt::BrTable { index, targets, default } => {
                b.note_uses_expr(index, *cur);
                let mut arm_ids = Vec::new();
                for (_, label, is_loop) in targets {
                    let aid = b.fresh_block(format!("{label}-case"));
                    b.cases.push((aid, label.clone(), *is_loop));
                    b.add_edge(*cur, aid);
                    arm_ids.push(aid);
                }
                let (_, dlabel, dis_loop) = default;
                let did = b.fresh_block(format!("{dlabel}-default"));
                b.cases.push((did, dlabel.clone(), *dis_loop));
                b.add_edge(*cur, did);
                b.blocks[*cur].term = Terminator::Switch {
                    index: index.clone(),
                    arms: arm_ids,
                    default: did,
                };
                *live = false;
            }
            Stmt::Return { values } => {
                for v in values {
                    b.note_uses_expr(v, *cur);
                }
                b.blocks[*cur].term = Terminator::Return(values.clone());
                *live = false;
            }
        }
    }
}

/// Statements after an unconditional transfer: keep them covered by
/// routing copies into the dead block.
fn push_dead(s: &Stmt, b: &mut Builder) {
    let dead = b.dead;
    match s {
        Stmt::Assign { dst, expr } => b.push_flat(
            dead,
            FlatStmt::Assign { dst: dst.clone(), expr: expr.clone() },
        ),
        Stmt::ExprStmt(e) => b.push_flat(dead, FlatStmt::ExprStmt(e.clone())),
        Stmt::Comment(c) => b.push_flat(dead, FlatStmt::Comment(c.clone())),
        Stmt::Store { ty, base, offset, value } => b.push_flat(
            dead,
            FlatStmt::Store {
                ty: ty.clone(),
                base: base.clone(),
                offset: *offset,
                value: value.clone(),
            },
        ),
        Stmt::If { cond, then_b, else_b } => {
            b.note_uses_expr(cond, dead);
            let (mut c1, mut l1, mut c2, mut l2) = (dead, true, dead, true);
            build_list(then_b, b, &mut c1, &mut l1);
            build_list(else_b, b, &mut c2, &mut l2);
        }
        Stmt::Block { body, .. } | Stmt::Loop { body, .. } => {
            let (mut c, mut l) = (dead, true);
            build_list(body, b, &mut c, &mut l);
        }
        _ => {}
    }
}

/// Dominator sets (iterative dataflow): `dom[b]` contains `b` and every
/// block on all paths from entry to `b`. Unreachable blocks dominate
/// only themselves.
pub fn dominators(cfg: &Cfg) -> Vec<HashSet<BlockId>> {
    let n = cfg.blocks.len();
    let all: HashSet<BlockId> = (0..n).collect();
    let mut dom: Vec<HashSet<BlockId>> = vec![HashSet::new(); n];
    dom[cfg.entry] = HashSet::from([cfg.entry]);
    let mut reach = HashSet::from([cfg.entry]);
    let mut stack = vec![cfg.entry];
    while let Some(x) = stack.pop() {
        for s in succ_of(&cfg.blocks[x]) {
            if reach.insert(s) {
                stack.push(s);
            }
        }
    }
    for blk in 0..n {
        if blk == cfg.entry {
            continue;
        }
        dom[blk] = if reach.contains(&blk) {
            all.clone()
        } else {
            HashSet::from([blk])
        };
    }
    let mut changed = true;
    while changed {
        changed = false;
        for blk in 0..n {
            if blk == cfg.entry || !reach.contains(&blk) {
                continue;
            }
            let preds = &cfg.blocks[blk].preds;
            if preds.is_empty() {
                continue;
            }
            let mut new: Option<HashSet<BlockId>> = None;
            for p in preds {
                if !reach.contains(p) {
                    continue;
                }
                new = Some(match new {
                    None => dom[*p].clone(),
                    Some(acc) => acc.intersection(&dom[*p]).copied().collect(),
                });
            }
            if let Some(mut set) = new {
                set.insert(blk);
                if set != dom[blk] {
                    dom[blk] = set;
                    changed = true;
                }
            }
        }
    }
    dom
}

/// Does `a` dominate `b` (every entry→`b` path passes through `a`)?
pub fn dominates(dom: &[HashSet<BlockId>], a: BlockId, b: BlockId) -> bool {
    dom.get(b).map_or(false, |s| s.contains(&a))
}

fn succ_of(blk: &CfgBlock) -> Vec<BlockId> {
    match &blk.term {
        Terminator::Next(t) | Terminator::Jump(t) => vec![*t],
        Terminator::Branch { then_b, else_b, .. } => {
            let mut v = vec![*else_b];
            if *then_b != usize::MAX {
                v.push(*then_b);
            }
            v
        }
        Terminator::Switch { arms, default, .. } => {
            let mut v = arms.clone();
            v.push(*default);
            v
        }
        Terminator::Return(_) | Terminator::Unreachable => Vec::new(),
    }
}

/// Blocks reachable from entry (following all terminator edges).
pub fn reachable(cfg: &Cfg) -> HashSet<BlockId> {
    let mut set = HashSet::new();
    let mut stack = vec![cfg.entry];
    while let Some(x) = stack.pop() {
        if !set.insert(x) {
            continue;
        }
        for s in succ_of(&cfg.blocks[x]) {
            stack.push(s);
        }
    }
    set
}

/// Blocks on some path from `from` to `to` (inclusive): forward-reachable
/// from `from` ∩ can-reach `to`. Used for staleness checks.
pub fn blocks_between(cfg: &Cfg, from: BlockId, to: BlockId) -> HashSet<BlockId> {
    let mut fwd = HashSet::new();
    let mut stack = vec![from];
    while let Some(x) = stack.pop() {
        if !fwd.insert(x) {
            continue;
        }
        for s in succ_of(&cfg.blocks[x]) {
            stack.push(s);
        }
    }
    let mut back = HashSet::new();
    let mut stack = vec![to];
    while let Some(x) = stack.pop() {
        if !back.insert(x) {
            continue;
        }
        for p in &cfg.blocks[x].preds {
            stack.push(*p);
        }
    }
    fwd.intersection(&back).copied().collect()
}
