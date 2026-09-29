# Phase 6 — Readability: strings, copyprop, collapse, ordering, toggles

Goal: cleaner output on big stripped binaries (cf. `lua_worker.wasm`).
Accuracy first; every rewrite documented. Recompilability NOT a goal.

## S1 string table + three display modes

Build (per module, in `parse`/`types`): scan data segments for printable
runs (≥4 chars, allow `\n\t`, stop at first other byte; record NUL-terminated
runs with their length). Map: `addr -> (seg, off, bytes, is_nul_term)`.

Resolution: reuse const-folding; an address counts if it folds to
`ConstI32/ConstI64` (after the existing `+0` identities). Covers direct
`*(mem + 1024)` and `base+off` with const base. Dynamic addresses stay numeric.

Modes (`--strings=`):
* `off` — today's behavior minus even the segment preview? No: previews stay;
  per-access notes off.
* `comment` (default) — append `/* "auth-ok" */` (truncated to 32 chars +
  `…` + length) to the load/store/assign. Zero semantic claim.
* `defs` — emit `static const char s_<addr>[] = "...";` for referenced
  strings + reference `s_<addr>` in code. Reads like real C; the symbol
  also gives `--func-name`-style greppability. Risk (why not default):
  implies NUL-termination and segment ownership the lifter didn't prove;
  non-NUL runs are emitted with explicit length comment.

Recommendation: default `comment`; `defs` opt-in. Never inline raw literals
at the access site (lies about type: it's a byte in linear memory, and
width>1 accesses aren't characters at all).

## S2 local copy propagation (lN)

Today only `tN` inline. Extend to locals with a soundness rule that fits the
existing Stmt tree:
* Always safe: `lN = pM` (params immutable) and `lN = const` with exactly one
  static assignment in the function.
* General case: exactly one static assignment AND the definition is not
  inside a `Loop` body (loop-carried redefinition risk). Uses anywhere.
* Ordering vs type recovery: run `recover()` BEFORE copyprop; carry an
  alias map (`lN -> pM`/expr) so layout keys (`S_2_l0` via base `l0`) keep
  resolving after substitution. Struct-field rendering must consult the
  alias map, not raw base names.
* Keep one declaration per local (emitter still declares `lN`); dead ones
  fall to existing DCE.

Expected effect on lua output: most `l0 = p0; ... l0 ...` chains dissolve,
`l3 = l2;` single-use temps vanish.

## S3 collapse

* Empty functions (`f4(){}`) → single `void f4() {}` line (already) + new:
  `--user-only` drops them only if unreferenced? No — keep, one line is fine.
* `do{}while(0)` blocks whose label is unreferenced AND whose body doesn't
  end in `goto` → plain `{ }` without label decl. (Referenced ones keep labels.)
* `if (cond) { goto L; }` immediately followed by `__end` fallthrough…
  leave (goto lowering is honest for irreducible flow).

## S4 emission ordering (`--order=`)

* `index` (today): function index order.
* `calls` (new): BFS from exports/`_start`; callees after callers where
  possible (stable by index within a level). Runtime (`RUNTIME_FUNCS`) sinks
  to the end regardless. Helps reading `run_some_code`-style dispatchers.

## S5 option toggles (OptConfig)

Today: only `--no-opt` (master switch). Nothing else is toggleable —
struct recovery, const-fold, inline, DCE, simplify, init-split, signature
rewrite are always on.

Proposed (all default-on except noted):
```
--no-opt            master switch (exists)
--no-fold           const/branch folding
--no-inline         tmp + copyprop inlining
--no-dce            dead-code elimination
--no-simplify       block/label simplification
--no-struct         struct recovery + field rendering + signature rewrite
--no-strings        = --strings=off
--strings=off|comment|defs   (default comment)
--order=index|calls (default index)
--user-only         (exists)
```
Implementation: `passes::OptConfig` + `emit::EmitConfig` structs, threaded
from CLI. Each toggle is one `if` at the pass boundary — no refactoring of
pass bodies. `--json` always reports unfiltered facts (toggles are views).

## Acceptance

* `cargo test` green (+ tests: string comment on const addr; copyprop of
  `lN=pM`; toggle off leaves output unchanged vs `--no-opt` subset).
* `lua_worker` user-only output shrinks ~20%+ lines with zero new markers;
  corpus stays marker-free in default mode AND with every toggle flipped.
* Docs: `--help` + README flag table + COVERAGE notes for strings mode.
