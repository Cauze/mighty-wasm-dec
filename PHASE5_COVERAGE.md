# Phase 5 — Comprehensive opcode coverage

Goal: exercise every wasm opcode class; every op either has an explicit
lifting arm or an honest arity-correct fallback. Recompilability NOT a goal.

## Opcode inventory (wasmparser 0.239, from `for_each_operator` groups)

| Group | Count (~) | Lifting plan |
|---|---|---|
| mvp control/vars/memory/numeric | ~200 | already explicit |
| sign_extension (5) | 5 | already explicit |
| saturating_float_to_int (8) | 8 | already explicit (casts) |
| bulk_memory (7): init/drop/copy/fill/table-init/elem-drop/table-copy | 7 | NEW: pseudo-calls `memory.copy(dst,src,len)` etc. |
| reference_types (11): typed-select/ref-null/is-null/func/table-fill/get/set/grow/size | 11 | NEW: `ref.null()`, `ref.is_null(x)`, `table.get(i)`… |
| tail_call (2) | 2 | NEW: call + `return` |
| memory_control (1): discard | 1 | NEW: pseudo-call |
| threads/atomics (~70): load/store/rmw/cmpxchg/notify/wait/fence | ~70 | NEW: loads/stores recorded as accesses; rmw → `atomic.rmw.add(b,v)`; cmpxchg 3→1; wait/notify/fence pseudo-calls |
| simd (~180: loads/stores/const/lanes/arith/cmp/bitwise/conv) | ~180 | NEW: `V128Load/Store/Const` explicit; lane ops explicit; rest via `simd.unop/binop/ternop` groups with real wat names |
| relaxed_simd (~20) | ~20 | same generic simd groups (3→1 laneselect/madd explicit) |
| gc (~30: struct/array/ref-test/cast/i31) | ~30 | NEW: pseudo-calls with exact arities; custom-arity `struct.new/array.new.fixed` approximated + commented |
| exceptions (3+5 legacy): try_table/throw(+ref)/try/catch/rethrow/delegate | 8 | NEW: `FrameKind::Try`, throw = trap-like |
| function_references (5): call_ref/br_on_null/ref_as_non_null | 5 | NEW: `call.ref`, br-on as br_if, as_non_null identity |
| shared_everything_threads (~35) | ~35 | NEW: pseudo-calls with exact arities |
| wide_arithmetic (4: add128/sub128/mulwide) | 4 | NEW: two-result push |
| stack_switching (6) | 6 | fallback (toolchain can't easily produce; honest comment) |

Fallback (anything missed/future): arity-correct push of `Unknown` + wat-style
mnemonic comment (via `op_mnemeneic()` camel→`i8x16.add` conversion).

## IR/emit changes

* `Expr::Simd { op, args }` rendered `op(a, b)`; counted in uses/subst/fold
  (fold recurses, never folds through); `base_desc` → `expr`.
* Atomic loads/stores record `MemAccess` (dom `atomic-i32` etc.) so struct
  recovery keeps working on threaded code.
* `FrameKind::Try` (End closes like Block; Else/Catch switches arm).

## Corpus additions

C (emscripten, woodpecker): `intops` (div/rem/clz/rot/sign-ext),
`conv` (trunc/extend/float converts), `memory` (memcpy/memset → bulk),
`control` (nested loops/continues), `recursion` (fib),
`simd` (`wasm_simd128.h`, built with `-msimd128`).
Hand wat (`corpus/wat/`, no toolchain): `simd.wat`, `atomic.wat`,
`bulk.wat`, `refs.wat`, `tail.wat`, `gc.wat`, `except.wat`
(whatever the `wat` crate validates; rejects documented in COVERAGE.md).

## Acceptance

* `cargo test` green (+ new tests: bulk pseudo-call, atomic access, simd render).
* `corpus/COVERAGE.md`: group table with status + per-file `unhandled` grep empty
  except documented toolchain-limited proposals.
* `corpus/decompiled/` regenerated (C only; wat outputs spot-checked, not stored).
