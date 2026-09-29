# Library + Pass-Trait Refactor Plan

Goal: make the IR and pipeline usable as a library so third parties can
write deobfuscation/optimization passes, without changing decompiler
behavior. Refactor is mechanical: same semantics, new shape.

## 1. Crate split (binary + library)

- Add `src/lib.rs` declaring the existing modules as public:
  `pub mod emit; pub mod ir; pub mod lift; pub mod parse; pub mod passes; pub mod types;`
- `src/main.rs` becomes a thin CLI: delete its `mod ...;` lines, import via
  `use mighty_wasm_dec::{emit, ir, lift, parse, passes, types};`.
  `Args`, `RUNTIME_FUNCS`, `load_bytes`, `run`, and the `#[cfg(test)]` suite
  stay in the binary (adjust imports only).
- Cargo auto-discovers both targets (no `Cargo.toml` change needed).
- Check: every item the library needs is already `pub`
  (`lift_module`, `lift_selected`, `parse_meta`, `emit_c_with`, `EmitConfig`,
  `OptConfig`, `optimize_func_with`, `recover`, IR types). Fill any gaps.

## 2. `Pass` trait (`src/passes.rs`)

```rust
/// Per-function analysis shared by passes (use/def counts, def/use order).
pub struct PassCtx { ... }

/// A single decompiler pass. Returns true if it changed the function.
pub trait FuncPass {
    fn name(&self) -> &'static str;
    fn run(&self, f: &mut FuncIR, ctx: &mut PassCtx) -> bool;
}
```

- Port existing passes as structs: `FoldPass`, `InlineDcePass`
  (owns the `AliasMap` contribution via `PassCtx`), `SimplifyPass`,
  `SplitInitPass` (whole-function shape: takes `&mut FuncIR`, not stmts).
- Keep `OptConfig` + `optimize_func_with` as the driver (runs the trait
  objects in the current order/rounds) so CLI flags and all 27 tests keep
  working unchanged.
- Publish the pass-author toolkit: `PassCtx` builders (use counts, def/use
  seq maps), `subst_all_in_stmts`, statement walkers. These already exist
  as private helpers; make them `pub` with docs.

## 3. Example external pass (`examples/`)

- `examples/const_on_right.rs`: standalone program using only the public
  API — parse → lift → run a custom `CommutativeCanonPass`
  (`(5 + l0)` → `(l0 + 5)`, const-on-right canonical form to help
  downstream pattern matching) → emit. Proves the library boundary:
  it must not need any `pub(crate)` items.
- If the example needs something private, widen visibility instead of
  cheating (that friction is the point of the exercise).

## 4. Docs

- This file (the plan).
- `README.md`: short "Using as a library" section (dependency snippet,
  parse→lift→pass→emit skeleton, link to the example).

## 5. Explicit non-goals (follow-ups, not this refactor)

- Typed `Var` ids instead of `"l{i}"` strings; `width()`/`is_pure()`
  method surface on `Expr`. Behavior-preserving but wide churn — separate commit.
- CFG/dominator analysis, inter-proc passes, pass manager with fixpoint
  scheduling. The trait shape allows them later.
- No output changes: verify with `cargo test` (27 tests) + byte-identical
  corpus sweep (`corpus/wat/*`, `corpus/wasm/*` vs pre-refactor outputs).

## 6. Verification

1. `cargo test` — all green.
2. `cargo build --examples` — example compiles.
3. Corpus sweep byte-identical to pre-refactor baseline.
4. Run the example on one `corpus/wat` file, eyeball output.
5. Commit + push.
