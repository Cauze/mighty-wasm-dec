# CFG + Inter-proc Plan (#3 + #4)

Status: DESIGN ONLY — do not implement without review. Builds on the
`Var` keys and `FuncPass` trait (docs/VAR_REFACTOR.md, docs/LIB_REFACTOR.md).

## #3 — CFG with dominators (per function)

**Problem.** Passes reason about nested `Vec<Stmt>` with pre-order
sequence numbers. That breaks across branches: a def in `then_b` and a
use after the `If` get adjacent seqs, but the def doesn't dominate the
use. The copyprop order guard is therefore conservative in some places
and unsound in others (accepted + documented risk).

**Shape.** New `src/cfg.rs` (library-visible):

- `struct Cfg { blocks: Vec<Block>, entry: BlockId }`, `Block { stmts:
  Vec<FlatStmt>, succ: Vec<BlockId>, pred: Vec<BlockId> }` where
  `FlatStmt` is straight-line IR: `Assign`, `Store`, `ExprStmt`, `Comment`
  + terminators `Jump`, `Branch { cond, .. }`, `Switch`, `Return`.
- Build by flattening `FuncIR.body`: `Block/Loop/If` labels become CFG
  nodes, `Br*` become edges (resolve via the existing label map).
- Dominators via iterative dataflow (Cooper–Harvey–Kennedy, simple +
  fast enough) or Lengauer–Tarjan if functions get huge; DOMINANCE is
  the API, algorithm is an detail.
- Keep a two-way map (CFG node ↔ original `Stmt` path) so passes can
  keep working on `FuncIR` during migration, and so the emitter (which
  still consumes structured `Stmt`) needs no changes in phase 1.

**Migration order (each step byte-identical, tested):**

1. CFG construction + verification pass: build CFG for every corpus
   function, assert edge/statement coverage (every `Stmt` reachable,
   every `Br` lands). No behavior change (new `StructDomPass`? no —
   a `#[cfg(test)]` consistency harness first).
2. Reimplement the copyprop order guard on dominance
   (`def dominates use`, no intervening def on any path) behind the
   same `FuncPass` interface. Differential-test vs seq-guard outputs.
3. Real DCE (dead across branches), loop-invariant code motion for
   `Loop` bodies, transitive inline composition (fixes the chain cliff
   documented in docs/VAR_REFACTOR.md — def/use chains become explicit).
4. Only then consider emitting FROM the CFG (restructuring). NOT in
   this plan — the structured emitter stays.

**Risks.** `BrTable`-to-loop-head edges, `func_end` relooseness,
exceptional edges (`throw` unwinds — model as edge to a synthetic exit
or ignore with a documented soundness hole, as now). Multi-value stack
polymorphism after `unreachable`: keep the `Unknown` pads as explicit
CFG facts, never silent.

## #4 — Inter-proc pass scheduling

**Problem.** Passes run per function in index order. Values threaded
through calls (constant args, thread-local state) are invisible, and
the `tN`-heuristic + `null`-indirect limits can't lift without
cross-function facts.

**Shape.** New `src/callgraph.rs` (or extend `types::merge_notes`):

- Call graph from `FuncIR.calls` + resolved `indirects` (already
  collected). SCCs via Tarjan; process bottom-up (callees first).
- Per-function summaries, computed once and shared: `Summary {
  const_argsostructed?, writes_mem: bool, calls: Vec<u32>, ret_const:
  Option<Const> }`. Start small: `writes_mem` + `ret_const` only.
- A second `FuncPass`-adjacent trait for whole-module passes:
  `trait ModulePass { fn run(&self, m: &mut ModuleIR, ctx) -> bool }`,
  scheduled over SCCs. `optimize_func_with` stays the intra-proc driver;
  a new `optimize_module_with` runs module passes then per-func passes
  in callee-first order.
- First real clients: inter-proc constant propagation (const actuals
  → specialize callee view, read-only), indirect-target narrowing via
  table snapshots (already emitted — consume them).

**Risks.** Fixed-point cost on 4000-function modules (bound rounds,
cache summaries); function-pointer args (stay `null`-honest);
recursion (SCC single-node fallback = current behavior).

## Verification (for the implementation phase)

1. `cargo test` green throughout; new CFG tests (all corpus functions
   build a covering CFG; dominator spot-checks on fac/switch shapes).
2. Corpus sweep byte-identical after steps 1–2 (new machinery, old
   decisions), then *reviewed* diffs only for steps 3+ (better output).
3. Stress: chain case must improve (target: 2000-chain under 2s debug);
   sqlite3 full-module time must not regress.
4. No public API breakage beyond additive (`cfg`, `callgraph` modules,
   `ModulePass` trait).
