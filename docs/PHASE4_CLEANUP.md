# Phase 4 — Cleanup passes + corpus expansion

Goal: cleaner readable output. Recompilability is NOT a goal.

## Passes (new `src/passes.rs`, pure Rust on `Stmt`/`Expr`)

All passes are accuracy-preserving rewrites. Run order per function:
fold → inline/dce → block-simplify → init-split. Repeat fold+inline once
(second round catches chains like `t0=fac(6);t1=t0;return t1`).

### P1 const-fold + branch-fold
- Expr: i32/i64 `+ - * / % & | ^ << >>`, comparisons `== != < > <= >=`
  on consts; `!` on const; `select` with const cond.
- Stmt: `if (0)/if (1)` → keep one arm; `br_if (0)` → drop, `br_if (1)` → `br`.
- Expected: `if ((!1))` / `if ((0 <= 0))` in `_start`/`libc_exit_fini` disappear.

### P2 copy/DCE + single-use tmp inline
- Use-count `tN`/`lN` over body. Drop dead `Assign` with pure expr
  (const/local/tmp/binop/unop/select); impure expr (call, load, trap, raw)
  becomes `ExprStmt` so effects stay.
- Inline: `tN` assigned once with pure expr of depth ≤ 3 and used exactly
  once → substitute, delete assign. (`t0 = fac(6); return t0` → `return fac(6)`.)
- Keep existing self-assign peephole.

### P3 block-simplify (label-aware)
- Collect goto target labels first. Only touch blocks/loops/ifs no one jumps to.
- Empty `Block`/`Loop` → remove. `If` with both arms empty → remove.
- `do{}while(0)` stays (it IS the block); empty ones removed by above.

### P4 struct-init split
- `*(i64*)(base+off) = ConstI64(v)` where layout has i32 fields at
  `off` and `off+4` → two `i32` stores of lo/hi halves.
- Fixes `*(int64_t*)(mem+(l0)+8) = 17179869187ll` → two field stores.

### P5 signature rewrite (emit-side)
- If param `l{i}` matches a recovered layout, emit param as `S*` instead
  of `int32_t`, and type the `l{i} = p{i}` copy accordingly.
- `apply` table-index param gets `/* table index */` comment.

### P6 user-only view (CLI `--user-only`)
- Hide runtime: `__wasm_call_ctors dummy libc_exit_fini exit _Exit
  _emscripten_stack_restore emscripten_stack_get_current`. Keep `_start`
  (shows wiring) + all user funcs. Forward decls filtered too.
- `--no-opt` skips P1–P4 for debugging.

## Corpus expansion (4 new files)

* `switch.c` — `switch` over int → `br_table`/`switch` round-trip.
* `globals.c` — mutable global + getter/setter (global-type recovery probe).
* `array.c` — array loop sum (index-scale `base[i]` readability probe).
* `float.c` — `double` arithmetic (f64 load/store + convert probe).

Woodpecker loop becomes `fac struct call indirect strings switch globals array float`.
Same flags. Expected outputs regenerated to `corpus/decompiled/`.

## Acceptance

* `cargo test` green (add: fold removes `if((0<=0))`, inline removes `t0=fac(6);return t0` shape, init-split test).
* Corpus grep clean: no `stack-underflow|Unknown|br_depth` (existing) plus
  no `if ((!1))`, no `l3 = l3`-style self-assign, no `17179869187ll`-style packed init.
* `corpus/decompiled/*.c` regenerated with default flags; `--user-only` spot-checked.
