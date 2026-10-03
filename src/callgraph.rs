//! Inter-procedural facts: call graph, SCCs, per-function summaries.
//!
//! Passes run per function today; values threaded through calls are
//! invisible. This module provides the shared facts (see
//! docs/CFG_FULL_PLAN.md Phase C): graph from `calls` + resolved
//! `indirects`, Tarjan SCCs for callee-first scheduling, and small
//! summaries consumed by module passes.

use std::collections::{HashMap, HashSet};

use crate::ir::{Expr, ModuleIR};

/// Tiny per-function summary. Deliberately small: every field must be
/// cheap to compute and obviously sound to consume.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    /// Single constant result with no other effects (`return 5;` and
    /// nothing else). Sound to substitute at pure-arg call sites.
    pub ret_const: Option<Expr>,
    /// Function may write linear memory (any `Store` in body).
    pub writes_mem: bool,
}

/// Compute summaries for all functions in the module.
pub fn summarize(m: &ModuleIR) -> HashMap<u32, Summary> {
    m.funcs
        .iter()
        .map(|f| {
            let ret_const = match f.body.as_slice() {
                [crate::ir::Stmt::Return { values }] if values.len() == 1 => {
                    let v = &values[0];
                    match v {
                        Expr::ConstI32(_) | Expr::ConstI64(_) => Some(v.clone()),
                        _ => None,
                    }
                }
                _ => None,
            };
            let writes_mem = f.accesses.iter().any(|a| a.is_write);
            (f.idx, Summary { ret_const, writes_mem })
        })
        .collect()
}

/// Call-graph adjacency: caller idx → callee idxs (direct + resolved
/// indirect; unresolved stay absent — honest, not guessed).
pub fn graph(m: &ModuleIR) -> HashMap<u32, Vec<u32>> {
    m.funcs
        .iter()
        .map(|f| {
            let mut cs = f.calls.clone();
            cs.extend(f.indirects.iter().flatten());
            cs.sort_unstable();
            cs.dedup();
            (f.idx, cs)
        })
        .collect()
}

/// Tarjan SCCs in reverse-topological order (callees first). Single-node
/// SCCs (no self-loop) come out dependency-ordered; recursive cycles
/// stay grouped so consumers can fall back to current behavior on them.
pub fn sccs(m: &ModuleIR) -> Vec<Vec<u32>> {
    let g = graph(m);
    let mut index = 0u32;
    let mut stack: Vec<u32> = Vec::new();
    let mut on_stack: HashSet<u32> = HashSet::new();
    let mut idx_of: HashMap<u32, u32> = HashMap::new();
    let mut low: HashMap<u32, u32> = HashMap::new();
    let mut out: Vec<Vec<u32>> = Vec::new();

    fn strongconnect(
        v: u32,
        g: &HashMap<u32, Vec<u32>>,
        index: &mut u32,
        stack: &mut Vec<u32>,
        on_stack: &mut HashSet<u32>,
        idx_of: &mut HashMap<u32, u32>,
        low: &mut HashMap<u32, u32>,
        out: &mut Vec<Vec<u32>>,
    ) {
        idx_of.insert(v, *index);
        low.insert(v, *index);
        *index += 1;
        stack.push(v);
        on_stack.insert(v);
        if let Some(succs) = g.get(&v) {
            for &w in succs {
                if !idx_of.contains_key(&w) {
                    strongconnect(w, g, index, stack, on_stack, idx_of, low, out);
                    let lv = low[&v].min(low[&w]);
                    low.insert(v, lv);
                } else if on_stack.contains(&w) {
                    let lv = low[&v].min(idx_of[&w]);
                    low.insert(v, lv);
                }
            }
        }
        if low[&v] == idx_of[&v] {
            let mut scc = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack.remove(&w);
                scc.push(w);
                if w == v {
                    break;
                }
            }
            out.push(scc);
        }
    }

    // Deterministic roots: sorted function indices.
    let mut roots: Vec<u32> = m.funcs.iter().map(|f| f.idx).collect();
    roots.sort_unstable();
    for v in roots {
        if !idx_of.contains_key(&v) {
            strongconnect(v, &g, &mut index, &mut stack, &mut on_stack, &mut idx_of, &mut low, &mut out);
        }
    }
    out
}
