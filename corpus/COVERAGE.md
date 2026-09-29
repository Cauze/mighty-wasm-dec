# Opcode coverage (wasmparser 0.239)

Every opcode group has an explicit lifting arm except stack-switching
(no text toolchain to produce it; honest fallback with exact... nearest
fallback). Verified 2026-09-28: all 15 C-corpus + 7 wat-corpus files
decompile with zero `unhandled` / `stack-underflow` / `br_depth` markers.

## By proposal group

| Group | Ops (~) | Status | Exercised by |
|---|---|---|---|
| mvp control/vars/memory/numeric | ~200 | explicit | all C files |
| sign_extension (5) | 5 | explicit | `intops` |
| saturating float→int (8) | 8 | explicit (casts) | `conv` |
| bulk_memory (7) | 7 | explicit pseudo-calls | `bulk.wat`; `memory` (lowered inline by LLVM) |
| reference_types (11) | 11 | explicit pseudo-calls | `refs.wat` |
| tail_call (2) | 2 | explicit (call + return) | `tail.wat` |
| memory_control discard (1) | 1 | explicit pseudo-call | arm present (no producer; bulk-adjacent) |
| threads/atomics (~70) | ~70 | explicit; loads/stores recorded as accesses; rmw/cmpxchg named | `atomic.wat` |
| simd core (~230) | ~230 | explicit: loads/stores/const/lanes exact; arith/cmp by arity with wat names; lane indices kept | `simd.wat`, `simd.c` (`-msimd128`) |
| relaxed_simd (~20) | ~20 | same arity groups | arms present (no easy producer) |
| gc struct/array/i31/cast (~30) | ~30 | explicit pseudo-calls; custom-arity `struct.new` via recorded field counts | `gc.wat` |
| exceptions try_table/throw (3) | 3 | explicit (`Try` frames, throw=trap) | `except.wat` |
| legacy try/catch/rethrow/delegate (5) | 5 | arms present; `wat` crate rejects the text form (proposal removed) | — (documented gap) |
| function_references (5) | 5 | explicit (`call.ref`, br-on as br_if) | `tail.wat` |
| shared-everything atomics (~35) | ~35 | explicit pseudo-calls per arity | arms present (no producer) |
| wide_arithmetic (4) | 4 | explicit (two-result push) | arms present (no easy producer) |
| stack_switching (6) | 6 | fallback only (no text/binary producer available) | documented gap |
| fallback (future ops) | — | arity-guess + wat-style mnemonic comment, never silent | — |

## Notes

* `br_on_cast` target approximated to innermost scope (payload depth
  needs scope-shape info the lifter doesn't retain); commented.
* `struct.new` without recorded field counts approximates the stack;
  recorded for all module-defined struct types.
* Legacy-exception and stack-switching arms exist so old/future binaries
  can't panic the lifter; they are not corpus-exercised (see above).
