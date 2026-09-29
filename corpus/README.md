# Corpus (real Emscripten builds)

Small, focused C→wasm corpus for testing readability/accuracy.
Recompilability of decompiler output is explicitly **not** a goal;
readability and accuracy are.

## Layout

* `src/*.c` — 15 focused sources (`fac`, `struct`, `call`, `indirect`,
  `strings`, `switch`, `globals`, `array`, `float`, `intops`, `conv`,
  `memory`, `control`, `recursion`, `simd` — the last built with `-msimd128`)
* `wat/*.wat` — 15 hand-written files, no toolchain (read directly):
  `mvp-01..08` core cases (loop, struct, calls, indirect, memory, stack,
  br_table, strings) + 7 proposal files (`simd`, `atomic`, `bulk`,
  `refs`, `tail`, `gc`, `except`)
* `wasm/*.wasm` — build artifacts (gitignored; rebuild via woodpecker, see below)
* `wasm/BUILD.txt` — toolchain + flags record
* `decompiled/*.c` — expected decompiler outputs for the C files (gitignored; regenerate locally, see below)
* `COVERAGE.md` — opcode group × status table

## Rebuilding

Requires Docker. From the project root:

```
woodpecker-cli lint .woodpecker/corpus.yml
woodpecker-cli exec --backend-engine docker .woodpecker/corpus.yml
```

`--backend-engine docker` is required: the default local backend would run
`emcc` on Windows instead of inside `emscripten/emsdk:4.0.10`.

Flags: `-O1 -g -s STANDALONE_WASM=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0`.
`-g` keeps the name section so `fac`/`move_x`/`apply` survive;
`-O1` + `noinline` keeps test functions separate without startup bloat.

## Refreshing expected outputs

```
cargo build --release
foreach ($f in "fac","struct","call","indirect","strings","switch","globals","array","float","intops","conv","memory","control","recursion","simd") { ./target/release/mighty-wasm-dec.exe corpus/wasm/$f.wasm --out corpus/decompiled/$f.c }
# --user-only gives the focused view (user funcs + __original_main)
./target/release/mighty-wasm-dec.exe corpus/wasm/fac.wasm --user-only
```

Clean criteria: no `stack-underflow`, no `Unknown`, no `br_depth`,
no hex-float leftovers (`(float)(0x…)`), no `unhandled`
(author's check: `Select-String -Path corpus/decompiled/*.c -Pattern "stack-underflow|Unknown|br_depth|\(\(float\)\(0x|\(\(double\)\(0x|unhandled"`).
Dynamic `call_indirect` through a parameter correctly stays `null` in `--json`
(`apply` case) — honest, not a failure.
