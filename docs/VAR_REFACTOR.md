# Var-IDs + Expr-Surface Refactor Plan (#1 + #2)

## #1 — Typed `Var` ids

New `ir::Var` (`Local(u32) | Tmp(u32) | Global(u32)`, `Copy+Hash+Ord`)
with `Var::parse("l3")` / `Var::name()` bridging the `String` IR boundary.

- `passes.rs`: all use/def/subst maps keyed by `Var`, not `String`.
  `count_uses_expr` matches `Local/Tmp` directly (no `format!` per node);
  `subst_all` compares integers; `dst` strings parsed ONCE per `Assign`.
- Candidate sorting stays by rendered `name()` so output order is
  byte-identical to the string-keyed version.
- `Stmt::Assign.dst`, `AliasMap`, emitter layout keys stay `String`
  (IR/emitter boundary — cold paths, not worth the churn now).
- Public toolkit renames: `rep_var_names` → `rep_vars` (`Vec<Var>`);
  `count_uses_stmts` / `collect_def_use_seq` / `subst_all_in_stmts` /
  `sweep_assigns` take `Var`-keyed maps. (`0.x` lib: allowed breakage;
  `docs/LIB_REFACTOR.md` toolkit list updated.)

## #2 — `Expr` method surface

New methods on `Expr` (replacing scattered free helpers):

- `is_pure()`, `depth()` (replace `passes::is_pure`, `passes::expr_depth`;
  old names kept as thin wrappers — no breakage),
- `vars(&self, acc: &mut Vec<Var>)` (replaces `rep_var_names`),
- `strip_casts(&self)` (replaces free `strip_casts` in fold).

`load_byte_width` / `base_key` stay (emitter/lift-local, typed enough).

## Verification

1. `cargo test --bins` — 31 green (no test changes expected).
2. `cargo build --examples` clean, zero warnings.
3. Corpus sweep (15 wat + 16 wasm incl. sqlite3) byte-identical to
   pre-refactor baseline — proves the key change is semantics-preserving.
4. Spot stress: independent-tmp batch must stay fast (~130ms debug for
   2000 single-use call tmps); dependency *chains* (`l_{i+1}=f(l_i)`)
   remain quadratic-by-rounds (~85s debug pre-existing, unchanged) —
   batching defers chained candidates one round each. Transitive
   composition would fix it but changes inline fixpoint behavior; left
   for the CFG era (def/use chains become explicit there).

## Non-goals (see docs/CFG_PLAN.md for #3/#4)

- No versioning of locals (still one `Var` per index — SSA versions come
  with #3); no `dst: Var` migration; no behavior change whatsoever.
