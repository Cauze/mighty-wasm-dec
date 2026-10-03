# Full Plan: CFG + Inter-proc + Emitter Restructuring (#3 + #4)

STATUS: Phase A, B (guard), C, D IMPLEMENTED. Phase B remainder
(transitive composition, cross-branch DCE) deferred — see notes.
Supersedes the design-only docs/CFG_PLAN.md for execution (that file
remains as rationale). Goal: real control-flow reasoning, cross-function
facts, and goto-light output — with tests and corpus proof at every phase.

## Phase A — CFG construction + dominance (no behavior change)

New `src/cfg.rs` (public API):

- `BlockId`, `FlatStmt` (straight-line only: `Assign`/`Store`/`ExprStmt`/
  `Comment`; `dst` stays `String` — no IR churn),
  `Terminator::{Jump, Branch{cond,..}, Switch{..}, Return, Unreachable}`,
  `CfgBlock { stmts, term, preds }`, `Cfg { blocks, entry }`.
- `build(func: &FuncIR) -> Cfg`: flatten `Block/Loop/If` to nodes,
  `Br*` to edges via the label map. Dead code after unconditional
  transfers routes to `Unreachable` (never silent).
- `dominators(cfg) -> Vec<HashSet<BlockId>>` (iterative dataflow;
  adequate for our function sizes), `dominates()`,
  `blocks_between(cfg, from, to)` (forward-reachable ∩ can-reach),
  `def_blocks` / `use_blocks` maps keyed by `Var`.
- Tests: every corpus function builds a covering CFG (all `Br` resolve,
  all stmts placed); dominator spot-checks on fac/switch shapes;
  `blocks_between` on diamond/loop shapes.

## Phase B — CFG-driven passes (same output, stronger proof) [DONE]

- Copyprop order guard reimplemented on dominance: def-block dominates
  use-block + no rep-var def on any acyclic def→use path (endpoints
  excluded, matching seq semantics). Straight-line bodies keep the
  positional check (exact, free).
- Iteration order matters: the first cut used cyclic `blocks_between`
  and diverged ~57k lines on f2422 (loop-around paths made every
  loop-carried rep look stale). Excluding loop-closing back-edges
  (`is_back_edge`: head dominates tail) restored same-iteration
  inlining — f2422 is byte-identical to the seq guard either way —
  while branch-skipped defs (the `bon` miscompile class) still refuse
  via dominance failure. No depth gate remains.
- Scale story (perf subagent, then refined): bitset+RPO dominators
  (f2422's dom: >120s → 93ms), dom reuse across inline rounds via
  topology fingerprint, per-candidate walks replaced by
  `blocks_between_acyclic` with fast-path skips, lazy seq maps.
  f2422 scoped: 0.4s, 13,971 dispatch cases intact. Full sqlite3
  module: 32.5s → 1.17s. `cfgdump` got the 512MB-stack treatment
  after overflowing on f2422.
- Differential result: `bon` (genuine miscompile fixed:
  branch-skipped def no longer inlined into the join),
  `recursion/fib` (converged identical after teaching the CFG that
  `tee`-residue self-copies aren't defs), full `sqlite3` kept-tmp
  decls 1702 → 310 with struct-hint count unchanged.
- DEFERRED: transitive inline composition (chain cliff stays),
  cross-branch DCE. Both need the fixpoint-behavior review this
  phase didn't have room for.

## Phase C — Module passes (#4) [DONE, driver deferred]

- `src/callgraph.rs`: graph from `calls` + resolved `indirects`,
  Tarjan SCCs (tested, deterministic roots), callee-first order.
- `trait ModulePass { run(&self, m: &mut ModuleIR) -> bool }` in
  `passes.rs`; first client `ConstRetPass` folds direct calls to
  single-`return const` callees (pure-args guard), wired in `main.rs`
  under `opt.fold`. Fires on real code (5 sites in full-module sqlite3).
- NOT YET: `optimize_module_with` driver (module passes run, then
  per-func passes in SCC order) — per-func order is still index order
  (`--order=calls` only affects emission). Next step when a second
  module pass needs scheduling.

## Phase D — Emitter: break/continue lowering (goto-light C) [DONE]

- Single-level `Br`-to-`Block` → `break` (inside its own `do{}while(0)`),
  `Br`-to-`Loop` → `continue`; deeper targets and `BrTable` keep
  `goto` (C has no labeled break). Same-target rule provably binds
  correctly (no intervening construct possible by construction).
- `do{}while(0)` form still follows IR references (a `break` needs a
  loop even when no `goto` remains — braces would be invalid C);
  unused `__end_` labels may remain (valid, dead).
- Corpus: 18 files changed, all reviewed (single-level gotos become
  break/continue, multi-level + dispatch gotos preserved); zero
  dangling labels across all outputs (checked mechanically);
  `br_table` keeps `switch/case/goto`.

## Verification (every phase)

1. `cargo test --bins` green; new tests per phase (cfg coverage,
   dominance, guard differential, break/continue, const-ret).
2. Comprehensive corpus sweep byte-identical where promised
   (A: all; B: all, then reviewed diffs; D: reviewed diffs only),
   including full `sqlite3.wasm` module + scoped set + marker scan.
3. Stress: chain case target <2s debug; sqlite3 full-module no regression.
4. Commit per phase, push at the end if all green.
