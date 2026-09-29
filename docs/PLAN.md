# mighty-wasm-dec — Plan

Goal: full-Rust `wasm -> C` decompiler with type analysis, no LLVM, no JVM.

## 1. Research summary

* `nneonneo/ghidra-wasm-plugin` (Java, P-code): best interactive RE, Wasm 1.0+SIMD only, slow, GPL-3. Proves P-code + stack-as-registers works.
* `NotDec` (C++/LLVM-14, Retypd+PNDiff+MemorySSA): best readable/recompilable C (100% recompile, 85% struct fields vs 9% Ghidra), but incomplete, fragile emitter, `O(n^3)` inter-proc blows up (>24h on large `-O0`), heavy build. Valuable for *concepts*, not code reuse.
* `kuna` (Rust Ghidra-port): fast, agent-first, Apache-2.0, but **no wasm frontend** (`specs/Ghidra/Processors/` has no wasm).
* `jlucaso1/ghidrs/wre` (Rust): expression-tree IR + callgraph + struct-merge + taint, JSON for LLMs. No SSA, pseudo-C not compilable, `SIMD->Unknown`, 1 commit. Good UX reference, not a decompiler core.
* Binja MLIL/HLIL: good IR, closed-source, no Rust ownership — rejected.

Conclusion: build from scratch in Rust reusing parser crates, own the SSA + types + emitter.

## 2. Chosen full-Rust stack

```
.wasm/.wat
 -> wasmparser (+ wat for .wat inputs)   // parse, validate, names
 -> own structured IR (expr-tree + block/loop/if)  // preserve wasm structure, cf. Binaryen/wre-ir
 -> SSA-lite + CFG (petgraph)            // phase 1 own; phase 2 swap in cfglib/analyssa
 -> custom type analysis (Retypd-lite)   // ptr-vs-int, SROA/stack-split, struct layout + merge
 -> Stackifier/Relooper structuring      // block-> {}, loop-> while(1), br-> break/continue/goto
 -> C emitter (typedefs + funcs)         // recompilable subset first
```

Crate pins (MVP): `wasmparser=0.239`, `wat=1`, `clap=4`, `anyhow=1`, `serde_json=1`, `petgraph=0.8`.
Phase-2 candidates: `cfglib` (git, dom/SSA/MemorySSA/GVN), `analyssa=0.6` (target-agnostic SSA), `egg` (simplify).

Why not LLVM (`inkwell/llvm-sys`): only C-API subset, no MemorySSA/custom passes, needs system LLVM, kills single-binary distribution.
Why not CLIF as primary IR: loses block structure; use as reference only.

## 3. Architecture (src/)

```
main.rs      CLI: decompile <file.wasm|.wat> [--func N] [--json] [--const-prop]
parse.rs     wasmparser: types, funcs, memories, tables, elems, data, names/exports
ir.rs        Expr, Stmt, FuncIR, structured Block/Loop/If; Wasm value types
lift.rs      stack-machine -> Expr trees + synthetic tmps; block-param handling; const tracking
callgraph.rs direct + const-resolved call_indirect via elems; invoke_* shim handling (TODO)
types.rs     ptr-vs-scalar, (offset,width,r/w,domtype) per base -> StructLayout + union-find merge
constprop.rs intra-block const fold + local single-const prop (feeds indirect resolution)
emit.rs      C typedefs + signatures (i32->int32_t etc) + structured stmts + gotos for irreducible
```

Key algorithms:
* Lifter simulates operand stack per function, `local.tee` as assign-expr, block results via synthetic locals.
* Type analysis is per-local advisory in MVP, per-SSA-value in full (future work, same limitation as `wre` today).
* Indirect resolution only when index is const (direct or after constprop) — documented limit.
* SIMD/atomic/ref-types lower to `/* unknown op */` comment + `asm` placeholder, never guessed.

## 4. C output contract

* This repo: readable pseudo-C with recovered `struct S0 { ... }`, typed pointer locals as comments + `int32_t*` where confident, `goto` with resolved labels for irreducible `br`. Recompilability is explicitly **not** a goal — readability and accuracy are.
* Phase 2 shipped: `mem + base` accesses, struct-field stores, merged aliases, imports/globals/data emission.

Low (no types) vs high (with types):
```c
// low: *(int32_t*)(mem+l0+12)
// high:
typedef struct { int32_t x; int32_t y; } S0;
void f(S0 *p) { p->y++; }
```

## 5. Hand-written inputs (now `corpus/wat/mvp-*`; was `samples/`)

Hand-written WAT -> .wasm via `wat` crate (no toolchain needed):
* `mvp-01-fac.wat` — loop + locals (fact iterative)
* `mvp-02-struct.wat` — `base+0/4` load/store struct pattern
* `mvp-03-ifelse-call.wat` — if/else + direct call
* `mvp-04-indirect.wat` — table + call_indirect with const index
* `mvp-05-memory.wat` — data segment + string xref pattern
* `mvp-06-stack.wat` — Emscripten-style `g0` stack frame + SROA hint [phase 2]
* `mvp-07-brtable.wat` — `br_table` -> `switch` with resolved labels [phase 2]
* `mvp-08-strings.wat` — printable vs binary data rendering [phase 2]

Merged into `corpus/wat/` 2026-09-29 (separate `samples/` dir removed);
proposal files (`simd`, `atomic`, …) live alongside under the same scheme.

CLI accepts `.wat` directly (compiles in-memory) and `.wasm`.

## 6. Build / test — status 2026-09-28 (phase 2 done)

```
cargo test   # 4/4 green: loop, struct+field, label resolution, mem base
```

Done in phase 2: br depth -> `__end_/__head_` labels + stack truncation,
`mem + base` loads/stores, struct-field stores, merged struct aliases,
imports/globals/data emission, `--list/--func/--func-name/--out`, 3 new samples.
Next: SSA/MemorySSA middle end, Retypd-lite, SROA notes for `g0`.

## 8. Corpus (phase 3, 2026-09-28; expanded phase 4)

* Location: `corpus/` — `src/*.c` (fac/struct/call/indirect/strings +
  switch/globals/array/float), `wasm/*.wasm` + `BUILD.txt`,
  `decompiled/*.c` expected outputs.
* Build: `woodpecker-cli exec --backend-engine docker .woodpecker/corpus.yml`
  (docker backend required; local backend would run emcc on Windows).
  Image `emscripten/emsdk:4.0.10`, flags `-O1 -g -s STANDALONE_WASM=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0`.
* Corpus-driven fixes: import func type/name offset (was shifting every signature),
  named import calls + decls, `unreachable` as trap with stack clear, exact
  load/store widths (was misreading `i64` as `int8`), table names, self-assign
  peephole. All files decompile with no `stack-underflow`/`Unknown`/`br_depth`.

## 9. Cleanup passes (phase 4, 2026-09-28 — see PHASE4_CLEANUP.md)

* `src/passes.rs`: const-fold + `+0` identities + branch-fold, single-use
  tmp inline (incl. calls/loads, recomputed per inline — stale-subst bug fixed),
  DCE (locals/tmps only — globals kept), label-aware block simplify,
  i64-init split w/ raw-access evidence.
* Emit: float literals decoded (`2.5` not `0x4004…`), const-base addresses
  folded (`mem + 1024`), data-segment cross-refs (`/* data0+0 */`), LE word
  dumps for tables (`words:[67,66,65,65]`, `words:[10,20,30,40]`),
  struct-typed params (`move_x(S_2_l0* p0)`), table-index comments,
  atomic loads/stores rendered (`atomic.load/store`), v128 c-type `__v128`.
* CLI: `--user-only` (user funcs + `__original_main`), `--no-opt`.
  Sat-truncation ops (`I32TruncSatF64S` family), copysign, clz/ctz/popcnt
  (`i32.clz`), rotl/rotr (`i32.rotl`) lifted.
* 11/11 tests green.

## 10. Comprehensive opcodes (phase 5, 2026-09-28 — see PHASE5_COVERAGE.md)

* `Expr::Simd{op,args}` + `op_mnemonic()` (wat-style dotted names);
  atomic accesses recorded for struct recovery; `FrameKind::Try`.
* Explicit arms: bulk memory, ref-types, tail calls (+return), funcrefs,
  GC (field counts recorded), exceptions, ~70 thread atomics, ~230 SIMD
  (+relaxed), shared-everything atomics, wide arithmetic. Fallback kept as
  safety net with mnemonic comments.
* Corpus: 15 C files (new: intops/conv/memory/control/recursion/simd) +
  7 hand wat (`corpus/wat/`). See `corpus/COVERAGE.md`. All 22 clean.

## 12. Scale robustness (2026-09-29, from 5.8MB/3255-func Flare module)

* 512MB worker-thread stack (deep nesting overflows 8MB main stack).
* Move-don't-clone in hot path (`binop`, `local.tee` cloned whole subtrees:
  quadratic hang + overflow).
* Depth-capped iterative `base_desc` (per-access full-tree walks were
  quadratic on machine-generated expression chains).

## 11. Readability pass (phase 6, 2026-09-28 — see PHASE6_READABILITY.md)

* Strings: table of printable runs; `--strings=off|comment|defs`
  (default comment `"text…"`; defs emits `static const char s_<addr>[]`
  + `s_1024[i]` byte refs). Const-folded addresses link loads to segments.
* Copyprop for locals (single static assign outside loops) with alias map
  so struct layouts survive substitution; DCE still globals-safe.
* Collapse: unreferenced straight-line blocks render as plain braces.
* `--order=calls` (BFS from exports, runtime sinks last).
* `OptConfig`/`EmitConfig` toggles (`--no-fold/--no-inline/--no-dce/`
  `--no-simplify/--no-struct`); `--json` always unfiltered.
* 16/16 tests green; corpus marker-free in default mode.

## 6. Build / test

```
cargo build --release
cargo run -- corpus/wat/mvp-01-fac.wat
cargo run -- corpus/wat/mvp-02-struct.wat --func 0
cargo run -- corpus/wat/mvp-04-indirect.wat --json   # dossier for LLMs
cargo test   # lifter + struct-recovery unit tests
```

Success criteria MVP: all 5 samples decompile without panic, struct sample yields `typedef struct`, indirect sample resolves edge in `--json`, no `Unknown` on MVP ops.

## 7. Risks / non-goals

* No GC/tail-call/exceptions/threads in MVP (comment fallback).
* No inter-proc Retypd (O(n^3)) — intra-proc only + union-find merge with cap.
* Locals reused for ptr+scalar may mis-hint (documented, fix with full SSA later).
* Large Emscripten binaries (MBs): streaming parse OK, but constprop bounded to keep time linear.
