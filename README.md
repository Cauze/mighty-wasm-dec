# mighty-wasm-dec (Phase 6: readability)

Full-Rust `wasm -> C` decompiler. Goal is **readability and accuracy**;
recompilability of the output is explicitly not a goal.

Stack: `wasmparser` + `wat` frontend, own structured IR, intra-proc
type analysis (ptr-vs-int, struct `(offset,width)` grouping + cross-func
merge aliases), `block`/`loop` structuring with resolved labels, cleanup
passes (`src/passes.rs`), C emitter with `mem`-relative accesses,
struct-field rendering, and string modes.

## Usage

```
cargo build --release
./target/release/mighty-wasm-dec.exe samples/01-fac.wat
./target/release/mighty-wasm-dec.exe --list corpus/wasm/indirect.wasm
./target/release/mighty-wasm-dec.exe --func-name fac corpus/wasm/fac.wasm
./target/release/mighty-wasm-dec.exe --user-only --order=calls corpus/wasm/indirect.wasm
./target/release/mighty-wasm-dec.exe --strings=defs corpus/wasm/strings.wasm
cargo test
```

Accepts `.wasm` and `.wat` (compiled in-memory via `wat` crate).

## Flags

```
--func <N>        only this func index          --func-name <S>  only this name
--list            list functions, exit          --json           machine dossier (unfiltered)
--out <F>         write to file                 --user-only      hide runtime + empty funcs
--no-opt          skip all passes               --no-fold/--no-inline/--no-dce/--no-simplify/--no-struct
--no-strings      no per-access notes           --strings=off|comment|defs (default comment)
--order=index|calls  emission order (default index)
```

`--user-only` is name-based: on stripped binaries (all `fN`) it only drops
empty stubs — use `--order=calls` + `--func-name` to navigate those.

## Samples (8)

* `01-fac` — loop/branch lowering, implicit return recovery
* `02-struct` — `typedef struct S_0_l0` + `((S*)l0)->f_off_N` stores
* `03-ifelse-call` — if/else with result phi, direct calls
* `04-indirect` — table snapshot `[f0,f1]`, `call_indirect`
* `05-memory` — byte load via `mem`
* `06-stack` — Emscripten-style `g0` stack frame + `S_0_l1` + SP note
* `07-brtable` — `switch` with resolved labels
* `08-strings` — data segments rendered (`"auth-ok"`, binary preview)

Outputs in `out/*.c`.

## Limits (honest)

* Intra-proc types only; reused locals may mis-hint (per-SSA inference = future).
* Indirect resolves only on const index; dynamic vtables stay unresolved (honest `null`).
* SIMD/atomic/GC → named pseudo-calls (`i8x16.add`, `atomic.load`,
  `struct.new`), never guessed; see `corpus/COVERAGE.md`.
* Synthetic `tN` are `int32_t` heuristic; widths/signedness not C-correct yet.
* Output is readable pseudo-C; recompilability is not a goal.

## Corpus

Real Emscripten builds live in `corpus/` (see `corpus/README.md`):
`src/*.c`, `wasm/*.wasm` (+`BUILD.txt`), `decompiled/*.c` expected outputs,
built via `woodpecker-cli exec --backend-engine docker .woodpecker/corpus.yml`.

Next: `cfglib`/`analyssa` SSA + MemorySSA, Retypd-lite constraints,
recompilable emitter, large Emscripten binary test.
