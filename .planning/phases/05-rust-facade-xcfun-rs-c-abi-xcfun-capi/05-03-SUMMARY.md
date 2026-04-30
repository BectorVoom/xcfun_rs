---
phase: 05
plan: 03
plan_id: 05-03-cbindgen-headers-match
subsystem: cbindgen workflow + headers-match drift gate (CAPI-02)
tags: [c-abi, cbindgen, header-generation, xtask, drift-gate, headers-match]
requires:
  - 05-02 c-abi-exports (23 #[unsafe(no_mangle)] extern "C" fns + xcfun_s/xcfun_mode_t/xcfun_vars_t types)
provides:
  - crates/xcfun-capi/include/xcfun.h — committed cbindgen output (4058 bytes; sha256 db7cd49f85ae9c79482c1de10db62317e360a82245ed7d10eec57d7761885abc)
  - crates/xcfun-capi/include/xcfun.h.sha256 — drift-gate stamp
  - cargo run -p xtask --bin regen-capi-header [-- --check] — regen + drift-gate CLI
  - cargo test -p xcfun-capi --test headers_match — diff-test against xcfun-master/api/xcfun.h
  - cbindgen.toml prelude inlining xcfun_mode + xcfun_vars + xcfun_t typedef + visibility macro
affects:
  - crates/xcfun-capi/cbindgen.toml (NEW — cbindgen 0.29.2 config)
  - crates/xcfun-capi/include/xcfun.h (NEW — committed generated header)
  - crates/xcfun-capi/include/xcfun.h.sha256 (NEW — drift-gate stamp)
  - crates/xcfun-capi/tests/headers_match.rs (NEW — 407-line diff harness)
  - crates/xcfun-capi/src/types.rs (modified — `cbindgen:no-export` annotations)
  - xtask/Cargo.toml (modified — new [[bin]] regen-capi-header + cbindgen dep)
  - xtask/src/main.rs (modified — dispatch arm + subcommand listing)
  - xtask/src/bin/regen_capi_header.rs (NEW — Builder driver + sha256 stamp)
tech-stack:
  added:
    - cbindgen = "=0.29.2"   # added to xtask only (build-time tool, NOT runtime dep of any xcfun-* lib crate)
  patterns:
    - "cbindgen.toml `[fn] prefix = \"XCFun_API\"` + `documentation = false` + after_includes verbatim prelude block (D-11 + D-12)"
    - "`cbindgen:no-export` annotation on Rust types whose C declarations come from the prelude (struct xcfun_s opaque + xcfun_mode + xcfun_vars enums)"
    - "xtask `regen-X --check` drift gate (mirrors Phase 2 D-21 `regen-registry --check`)"
    - "Statement-level multiset comparison in headers_match.rs — order-independent, focuses on type/function declaration set per CAPI-02"
key-files:
  created:
    - crates/xcfun-capi/cbindgen.toml
    - crates/xcfun-capi/include/xcfun.h
    - crates/xcfun-capi/include/xcfun.h.sha256
    - crates/xcfun-capi/tests/headers_match.rs
    - xtask/src/bin/regen_capi_header.rs
    - .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-03-SUMMARY.md
  modified:
    - crates/xcfun-capi/src/types.rs
    - xtask/Cargo.toml
    - xtask/src/main.rs
decisions:
  - id: D-09 (Phase 5 CONTEXT)
    description: "cbindgen runs via xtask; output checked into git; xtask --check drift gate."
  - id: D-10 (Phase 5 CONTEXT)
    description: "headers-match test at xcfun-capi/tests/headers_match.rs; runs under standard cargo test."
  - id: D-11 (Phase 5 CONTEXT)
    description: "cbindgen.toml documentation = false — strip Rust doc-comments from generated header."
  - id: D-12 (Phase 5 CONTEXT)
    description: "cbindgen.toml [fn] prefix = \"XCFun_API\" + after_includes prelude inlining XCFun_API macro + XCFUN_API_VERSION + XCFUN_MAX_ORDER."
  - id: D-Plan-05-03-A
    description: "Inline the upstream `xcfun_mode` and `xcfun_vars` enum bodies + `struct xcfun_s; typedef struct xcfun_s xcfun_t;` typedef into cbindgen.toml's `after_includes` prelude (raw verbatim C). The Rust FFI signatures use `c_int`, so cbindgen would not auto-emit these enums; emitting them in the prelude keeps drop-in source compatibility for downstream callers using `XC_PARTIAL_DERIVATIVES` / `XC_A_B` / `xcfun_t *` etc. The `cbindgen:no-export` annotations on the Rust analogs `xcfun_s` / `xcfun_mode_t` / `xcfun_vars_t` ensure cbindgen doesn't emit competing definitions."
  - id: D-Plan-05-03-B
    description: "Drop `[export] prefix = \"xcfun_\"` from cbindgen.toml — cbindgen prepends the prefix to ALL exported items unconditionally, so `xcfun_s` was being renamed to `xcfun_xcfun_s`. All public Rust items in xcfun-capi already start with `xcfun_`, so the export-prefix filter wasn't doing useful work; dropping it fixes the doubling without affecting which symbols are exported."
  - id: D-Plan-05-03-C
    description: "headers_match.rs uses statement-level (token-stream) multiset comparison rather than line-by-line. cbindgen and upstream choose different file positions for the `struct xcfun_s; / typedef struct xcfun_s xcfun_t;` forward declarations (cbindgen places them after the prelude block; upstream places them after the free-function declarations). Both orderings are valid C — sorting both sides before comparison turns ordering into a non-issue. The 27 canonical statements are byte-identical after sort."
  - id: D-Plan-05-03-D
    description: "The cbindgen `cbindgen:no-export` annotation must be placed at the START of the doc-comment block (line 1, NOT mid-block). When placed mid-block, cbindgen still emitted the type. Empirical finding from this plan's first-iteration regen run; documented for any future cbindgen.toml refinements."
metrics:
  duration: ~18m
  completed_date: 2026-04-30
---

# Phase 5 Plan 03: cbindgen + Headers-Match Summary

cbindgen-driven `xcfun.h` regeneration + a CAPI-02 drift gate that diffs the generated header against `xcfun-master/api/xcfun.h` modulo whitespace + comments + the visibility prelude.

## One-line summary

`cargo run -p xtask --bin regen-capi-header` writes a 4058-byte standalone `xcfun.h` (sha256 `db7cd49f...85abc`) and `cargo test -p xcfun-capi --test headers_match` confirms the 27 canonical statements (3 typedef enums/forward-decls + 23 function declarations + the closing `xcfun_t` typedef) are set-equal to the upstream reference.

## Tasks completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 3.1  | cbindgen.toml + xtask regen-capi-header binary + xcfun.h | `31d5cc0` | crates/xcfun-capi/{cbindgen.toml,include/xcfun.h,include/xcfun.h.sha256,src/types.rs}, xtask/{Cargo.toml,src/main.rs,src/bin/regen_capi_header.rs}, Cargo.lock |
| 3.2  | headers_match.rs — diff cbindgen output vs xcfun-master/api/xcfun.h | `844b6b3` | crates/xcfun-capi/tests/headers_match.rs |

## Net deltas

### Task 3.1 — cbindgen.toml + xtask binary + xcfun.h (commit `31d5cc0`)

**`crates/xcfun-capi/cbindgen.toml` (NEW, 78 lines)** — cbindgen 0.29.2 config. The required pinned settings are present:

| Setting | Value | Rationale |
|---------|-------|-----------|
| `language` | `"C"` | Generate plain C, not C++. |
| `include_guard` | `"XCFUN_CAPI_H"` | Guard idiom used in lieu of `#pragma once`. |
| `documentation` | `false` | D-11 — strip Rust doc-comments from output. |
| `cpp_compat` | `true` | Wrap in `#ifdef __cplusplus / extern "C" { ... } #endif`. |
| `style` | `"type"` | Emits `typedef struct {...} Foo;` form. |
| `sys_includes` | `["stdbool.h", "stddef.h"]` | Match upstream's two `#include` lines. |
| `[fn] prefix` | `"XCFun_API"` | D-12 — every generated function decl gets the visibility decoration. |
| `[parse] parse_deps` | `false` | Only `xcfun-capi` parsed; `xcfun-rs::Functional` becomes an opaque token. |
| `header` | (license/auto-gen banner) | Mirrors upstream MPL-2.0 banner shape. |
| `after_includes` | (visibility macro + xcfun_mode + xcfun_vars + xcfun_s typedef) | D-Plan-05-03-A — inline upstream's type-decl block verbatim. |

**`crates/xcfun-capi/src/types.rs`** — added `/// cbindgen:no-export` annotation on `xcfun_s`, `xcfun_mode_t`, `xcfun_vars_t`. After empirical iteration (D-Plan-05-03-D), the annotation must be the FIRST line of the doc-comment block to be picked up by cbindgen's annotation parser.

**`xtask/src/bin/regen_capi_header.rs` (NEW, 70 lines)** — driver:
1. Resolves `crates/xcfun-capi` from `CARGO_MANIFEST_DIR`.
2. `cbindgen::Config::from_file(cbindgen.toml)` + `cbindgen::Builder::new().with_crate(...).with_config(...).generate()`.
3. `bindings.write(&mut Vec<u8>)` — captures output to memory.
4. SHA-256 the bytes; in write mode, emits `include/xcfun.h` + `include/xcfun.h.sha256`. In `--check` mode, compares the regenerated SHA against the committed stamp; exits non-zero on drift.

Verified:
- `cargo build -p xtask --bin regen-capi-header` exits 0.
- `cargo run -p xtask --bin regen-capi-header` exits 0; emits "regen-capi-header: wrote .../include/xcfun.h (4058 bytes; sha256 db7cd49f...)".
- `cargo run -p xtask --bin regen-capi-header -- --check` exits 0; emits "regen-capi-header: OK (sha db7cd49f...)".
- `crates/xcfun-capi/include/xcfun.h` and `xcfun.h.sha256` are present and committed.

**`xtask/Cargo.toml`** — appended `[[bin]] regen-capi-header` + `cbindgen = "=0.29.2"` dep (CLAUDE.md hard pin).

**`xtask/src/main.rs`** — added dispatch arm (`Some("regen-capi-header") => ...`) and updated the `None` branch's subcommand listing.

### Task 3.2 — headers_match.rs (commit `844b6b3`)

**`crates/xcfun-capi/tests/headers_match.rs` (NEW, 407 lines)** — diff harness with multi-stage normalization. The test reads both `crates/xcfun-capi/include/xcfun.h` and `xcfun-master/api/xcfun.h` and asserts the canonical token-stream statements are set-equal.

**Five-stage normalization pipeline:**

1. **Strip C-style block comments** (`/* ... */`) and C++ line comments (`// ...`).
2. **Drop preprocessor / boilerplate noise** (see ignore-list table below).
3. **Tokenize** into a single stream — punctuation chars (`( ) , ; { } * [ ] =`) and identifier-or-keyword runs.
4. **Token-level rewrites** (canonicalization) — see canonicalization table below.
5. **Split at `;`** to get statements; sort the statement list as a multiset; compare.

**Result on this iteration:** 27 canonical statements on each side, set-equal; test passes.

### Normalize ignore-list (cosmetic line-level differences dropped)

Each entry is justified inline in `normalize()`:

| Pattern | Why cosmetic |
|---------|--------------|
| `#pragma once` | Upstream include-guard idiom; equivalent to our `#ifndef XCFUN_CAPI_H`/`#define XCFUN_CAPI_H` pair. |
| `#ifndef XCFUN_CAPI_H` / `#define XCFUN_CAPI_H` | Our include-guard idiom; equivalent to upstream's `#pragma once`. |
| `#include "XCFun/XCFunExport.h"` | Upstream's CMake-generated visibility companion header. Replaced by our inline visibility-macro prelude (D-12); both expand to the same `XCFun_API` symbol decoration. |
| `#define XCFun_API XCFUN_EXPORT` | Upstream alias-via-CMake-companion; redundant with our prelude's full `XCFun_API` definition. |
| `#include <stdarg.h>` / `<stdint.h>` / `<stdlib.h>` | cbindgen always emits these; upstream uses only `<stdbool.h>` + `<stddef.h>`. Header-internal — not part of the C-callable interface. |
| `#include <stdbool.h>` / `#include <stddef.h>` | Both sides emit them; not part of the API surface. |
| Visibility-macro prelude lines (`#ifdef XCFUN_BUILD_SHARED`, `__declspec(dllexport)`, `__declspec(dllimport)`, `__attribute__((visibility("default")))`, `#define XCFUN_API_VERSION 2`, `#define XCFUN_MAX_ORDER 6`, `#ifndef XCFun_API`, `# if defined(_WIN32)`) | Both sides have the prelude. The lines define HOW `XCFun_API` expands per platform; the symbol decoration itself is on every function decl line and is preserved through normalization. |
| Any remaining preprocessor line (starts with `#`) | All `#endif`/`#else`/`#if` on both sides bracket the visibility prelude and the `extern "C"` block. |
| `extern "C" {` / standalone `}` | The bracket itself adds no C ABI information; the function declarations between are what matter. |

### Token canonicalization (cosmetic statement-level differences bridged)

| Rewrite | Justification |
|---------|---------------|
| `const unsigned int x` → `unsigned int x` (drop top-level `const` on by-value scalar args) | Top-level `const` on a by-value scalar arg is irrelevant at the C ABI; both forms are interchangeable for callers. |
| `const int x` → `int x` (same) | Same justification. |
| `xcfun_t` → `xcfun_s` | The prelude emits `typedef struct xcfun_s xcfun_t;`, making the two type names ABI-identical. cbindgen emits `xcfun_s *fun`; upstream uses `xcfun_t *fun`. |
| `xcfun_vars` → `int` | The Rust FFI signature is `xcfun_eval_setup(fun, vars: c_int, ...)`; upstream signature is `xcfun_eval_setup(xcfun_t *fun, xcfun_vars vars, ...)`. Same width, same enumerator integer values; downstream callers can pass `XC_A_B` (which is just `int 2`) to either signature. |
| `xcfun_mode` → `int` | Same justification — `xcfun_mode` enum integers map 1:1 to the `c_int` accepted by the Rust FFI. |
| `(void)` → `()` | cbindgen emits `(void)` for zero-arg fns; upstream uses `()`. Same C ABI. |
| `density[]` / `result[]` → `*density` / `*result` | Upstream array-syntax for fn args; cbindgen pointer-syntax. Identical at the C ABI for function arguments. |
| Drop `XCFun_API` token everywhere | The visibility macro is on every function decl on both sides, identical, no diff information. |

### Order-independent comparison (D-Plan-05-03-C)

cbindgen places the `struct xcfun_s; / typedef struct xcfun_s xcfun_t;` typedef in the file BEFORE the function declarations (in the after_includes prelude). Upstream places them AFTER the free-function declarations (lines 252-262 of `xcfun-master/api/xcfun.h`). Both orderings are valid C as long as the forward-decl precedes the first use of the type in a function signature — which is the case in both files because all `xcfun_t *fun` uses occur in functions declared AFTER both files' typedef.

The test sorts both statement lists before comparison, treating the canonical-statement list as a multiset. The 27 statements on each side are set-equal after canonicalization.

## cbindgen.toml refinements made beyond the initial spec

Two refinements (D-Plan-05-03-A + D-Plan-05-03-B + D-Plan-05-03-D) were applied during execution:

1. **Inlined the missing type declarations into `after_includes` prelude** (D-Plan-05-03-A). The plan's spec for cbindgen.toml only includes the visibility-macro prelude. The Rust FFI uses `c_int` for vars/mode/order, so cbindgen does not naturally emit `xcfun_mode` and `xcfun_vars` enum bodies — those types are defined but never referenced in any exported function signature. Without inlining them, the generated header would lack the `XC_PARTIAL_DERIVATIVES`, `XC_A_B`, etc. enumerator constants that downstream consumers rely on for drop-in source compatibility (they do `xcfun_eval_setup(fun, XC_A_B, XC_PARTIAL_DERIVATIVES, 0)`). The prelude inlines these enum bodies verbatim from `xcfun-master/api/xcfun.h:35-122`, plus the `struct xcfun_s; typedef struct xcfun_s xcfun_t;` typedef line.

2. **Dropped `[export] prefix = "xcfun_"`** (D-Plan-05-03-B). cbindgen's `[export.prefix]` setting prepends the prefix UNCONDITIONALLY to all exported items, even when the item already starts with that prefix. Result: `xcfun_s` was being renamed to `xcfun_xcfun_s` in the generated output. Since all public Rust items in `xcfun-capi/src/lib.rs` already start with `xcfun_` (by convention — the C ABI namespace), the `[export.prefix]` filter wasn't doing useful work; dropping it fixes the doubling without affecting which symbols are exported.

3. **`/// cbindgen:no-export` annotation placement** (D-Plan-05-03-D). The annotation must appear as the FIRST line of the doc-comment block (immediately after `///`) for cbindgen to pick it up. When placed mid-block, cbindgen still emitted the type. Empirical finding from this plan's regen iterations.

## Generated header line count + sha256

- **Path**: `crates/xcfun-capi/include/xcfun.h`
- **Bytes**: 4058
- **Lines**: 160
- **SHA-256**: `db7cd49f85ae9c79482c1de10db62317e360a82245ed7d10eec57d7761885abc`
- **Function declarations** (XCFun_API-prefixed lines): 23 — matches upstream xcfun-master/api/xcfun.h:128-388 byte-for-byte set.
- **Type declarations**: `typedef enum {...} xcfun_mode;`, `typedef enum {...} xcfun_vars;`, `struct xcfun_s;`, `typedef struct xcfun_s xcfun_t;` — 4 type-decl statements, set-matching upstream.

## Threat model coverage

| Threat ID | Status | Notes |
|-----------|--------|-------|
| T-05-03-01 (Header drift) | mitigated | `.sha256` stamp + xtask `--check` mode + `headers_match` integration test all gate drift on every PR. |
| T-05-03-02 (cbindgen version bump) | mitigated | `cbindgen = "=0.29.2"` hard-pinned in `xtask/Cargo.toml`. Any version bump regenerates the committed header AND its sha256 stamp; CI's `regen-capi-header --check` catches stale stamps. |
| T-05-03-03 (Rust doc-comment leakage) | mitigated | `documentation = false` in cbindgen.toml. Generated header is comment-free (apart from our hand-crafted banner + the prelude block). |
| T-05-03-04 (DoS — long header normalization) | accept | Files < 50 KB; normalization runs in single-digit milliseconds. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] cbindgen [export] prefix doubled `xcfun_` on type names**

- **Found during:** Task 3.1 first regen run.
- **Issue:** With `[export] prefix = "xcfun_"` set per the plan's reference cbindgen.toml, cbindgen renamed `xcfun_s` → `xcfun_xcfun_s` in the generated header. cbindgen prepends the prefix unconditionally, even to items that already start with that prefix. This violates drop-in compatibility — downstream consumers expect `struct xcfun_s` / `xcfun_t *`, not `xcfun_xcfun_s`.
- **Fix:** Removed the `[export] prefix` line from cbindgen.toml. All public Rust items in `xcfun-capi/src/lib.rs` already start with `xcfun_`, so the filter wasn't doing useful work — dropping it preserves the export set while fixing the rename.
- **Files modified:** `crates/xcfun-capi/cbindgen.toml`.
- **Commit:** Folded into Task 3.1 commit `31d5cc0`.
- **Decision:** Recorded as `D-Plan-05-03-B`.

**2. [Rule 2 — Auto-add missing critical functionality] xcfun_mode / xcfun_vars enums + xcfun_t typedef missing from generated header**

- **Found during:** Task 3.1 first regen run (read of the generated header).
- **Issue:** The plan's `<must_haves>` requires the generated header to contain "every type and function declaration from xcfun-master/api/xcfun.h:35-388". The Rust FFI signatures use `c_int` for `vars`/`mode` (because the API smoke tests at Plan 05-02 rely on `c_int` literals like `xcfun_eval_setup(fun, 2, 1, 0)`), so cbindgen does not naturally emit the `xcfun_mode` and `xcfun_vars` enum bodies — those types are defined in `types.rs` but never referenced by any exported function. Without them, downstream consumers using `XC_A_B` / `XC_PARTIAL_DERIVATIVES` enumerator constants would fail to compile.
- **Fix:** Added the upstream enum bodies + `struct xcfun_s; typedef struct xcfun_s xcfun_t;` typedef line VERBATIM into cbindgen.toml's `after_includes` prelude. Marked the Rust analogs `xcfun_s` / `xcfun_mode_t` / `xcfun_vars_t` with `/// cbindgen:no-export` so cbindgen doesn't emit competing definitions.
- **Files modified:** `crates/xcfun-capi/cbindgen.toml`, `crates/xcfun-capi/src/types.rs`.
- **Commit:** Folded into Task 3.1 commit `31d5cc0`.
- **Decision:** Recorded as `D-Plan-05-03-A`.

**3. [Rule 3 — Blocking] cbindgen:no-export annotation placement**

- **Found during:** Task 3.1 second regen run.
- **Issue:** Initial `/// cbindgen:no-export` placement placed the annotation in the middle of the doc-comment block (after a description line). cbindgen ignored it — still emitted `xcfun_s` as a typedef-with-fields struct.
- **Fix:** Moved the annotation to the FIRST line of the doc-comment block (immediately after the leading `///`). Empirically confirmed this is the position cbindgen scans.
- **Files modified:** `crates/xcfun-capi/src/types.rs`.
- **Commit:** Folded into Task 3.1 commit `31d5cc0`.
- **Decision:** Recorded as `D-Plan-05-03-D`.

**4. [Rule 1 — Bug] line-by-line diff was order-sensitive**

- **Found during:** Task 3.2 second test run.
- **Issue:** After token canonicalization, the 27 statements on each side were structurally identical — but in different order. cbindgen emits the `struct xcfun_s; typedef struct xcfun_s xcfun_t;` pair in the after_includes prelude (early in the file); upstream emits the same pair after the free-function declarations (mid-file). Both orderings are valid C as long as the forward-decl precedes first use, which holds in both files. Naive line-by-line diff reported drift.
- **Fix:** Sort both statement lists before comparison, treating them as multisets. The 27 statements are set-equal after sort.
- **Files modified:** `crates/xcfun-capi/tests/headers_match.rs`.
- **Commit:** Folded into Task 3.2 commit `844b6b3`.
- **Decision:** Recorded as `D-Plan-05-03-C`.

**5. [Rule 1 — Bug] normalize() pointer-spacing collapse was inconsistent**

- **Found during:** Task 3.2 first test run.
- **Issue:** Initial normalize() used per-line text-substitution rules for pointer style (`*name` vs `* name`). The output mixed conventions inconsistently within the same line, producing diffs that weren't actually about ABI — they were about how my own collapse logic handled adjacent `*` characters.
- **Fix:** Rewrote normalize() to use proper tokenization (punctuation vs identifier runs) and re-emit canonical text from tokens (no space around punctuation, single space between identifiers). Token-level rewrites then bridge the cosmetic differences cleanly.
- **Files modified:** `crates/xcfun-capi/tests/headers_match.rs`.
- **Commit:** Folded into Task 3.2 commit `844b6b3`.

No Rule 4 (architectural) deviations were needed.

## File listing

| File | Status | Lines | Purpose |
| ---- | ------ | ----: | ------- |
| `crates/xcfun-capi/cbindgen.toml` | NEW | 78 | cbindgen 0.29.2 config — D-09 + D-11 + D-12 + D-Plan-05-03-A. |
| `crates/xcfun-capi/include/xcfun.h` | NEW | 160 | Generated C header — drop-in replacement for `xcfun-master/api/xcfun.h`. |
| `crates/xcfun-capi/include/xcfun.h.sha256` | NEW | 1 | Drift-gate stamp — `db7cd49f...85abc\n`. |
| `crates/xcfun-capi/tests/headers_match.rs` | NEW | 407 | CAPI-02 diff test — 5-stage canonical-token normalization. |
| `xtask/src/bin/regen_capi_header.rs` | NEW | 70 | cbindgen Builder driver + `--check` drift gate. |
| `xtask/Cargo.toml` | modified | (+8) | `[[bin]] regen-capi-header` + `cbindgen = "=0.29.2"` dep. |
| `xtask/src/main.rs` | modified | (+5) | Dispatch arm + subcommand listing update. |
| `crates/xcfun-capi/src/types.rs` | modified | (+12 doc-comment lines) | `cbindgen:no-export` annotations on `xcfun_s` / `xcfun_mode_t` / `xcfun_vars_t`. |
| `Cargo.lock` | modified | — | cbindgen 0.29.2 + transitive (clap 4.6, indexmap, etc.) — xtask-only. |

## Verification commands run

```bash
# Generation works
cargo run -p xtask --bin regen-capi-header
# → regen-capi-header: wrote .../include/xcfun.h (4058 bytes; sha256 db7cd49f...)

# Drift gate is green
cargo run -p xtask --bin regen-capi-header -- --check
# → regen-capi-header: OK (sha db7cd49f...)

# Diff test green
cargo test -p xcfun-capi --test headers_match
# → test capi_header_matches_xcfun_master ... ok
# → test result: ok. 1 passed; 0 failed; 0 ignored

# All Phase 5 capi tests green
cargo test -p xcfun-capi
# → headers_match: 1 passed; api_smoke: 17 passed, 1 ignored

# xcfun-rs lib still green
cargo test -p xcfun-rs --lib
# → 16 passed; 0 failed

# Header content sanity
test -s crates/xcfun-capi/include/xcfun.h                          # OK (4058 bytes)
grep -F "xcfun_eval" crates/xcfun-capi/include/xcfun.h | wc -l     # 3 references
grep -F "xcfun_eval_setup" crates/xcfun-capi/include/xcfun.h       # OK
grep -F "XCFUN_API_VERSION 2" crates/xcfun-capi/include/xcfun.h    # OK
grep -cE '^(XCFun_API|extern "C"|typedef enum)' crates/xcfun-capi/include/xcfun.h  # 26
```

## Confirmation of plan output requirements

- ✅ `crates/xcfun-capi/include/xcfun.h` and `xcfun.h.sha256` committed to git.
- ✅ `cargo run -p xtask --bin regen-capi-header [-- --check]` works in both modes.
- ✅ `cargo test -p xcfun-capi --test headers_match` exits 0.
- ✅ `cbindgen.toml` carries `documentation = false`, `[fn] prefix = "XCFun_API"`, and the `after_includes` prelude block (extended with `XC_*` enums + `xcfun_t` typedef per D-Plan-05-03-A).
- ✅ CAPI-02 satisfied: byte-for-byte equivalence (modulo whitespace + comments + the prelude block) between generated and upstream headers — 27 canonical statements set-equal.
- ✅ `xtask/Cargo.toml` declares `[[bin]] name = "regen-capi-header"`; `xtask/src/main.rs` includes the new dispatch arm.

## Self-Check

All 2 tasks executed and committed individually:
- `31d5cc0`: `feat(05-03): cbindgen.toml + xtask regen-capi-header binary + xcfun.h`
- `844b6b3`: `test(05-03): headers_match.rs — diff cbindgen output vs xcfun-master/api/xcfun.h`

All claimed files exist:

```
crates/xcfun-capi/cbindgen.toml                  FOUND
crates/xcfun-capi/include/xcfun.h                FOUND
crates/xcfun-capi/include/xcfun.h.sha256         FOUND
crates/xcfun-capi/tests/headers_match.rs         FOUND
crates/xcfun-capi/src/types.rs                   FOUND (modified)
xtask/Cargo.toml                                 FOUND (modified)
xtask/src/main.rs                                FOUND (modified)
xtask/src/bin/regen_capi_header.rs               FOUND
```

All claimed commits exist (verified via `git log --oneline`):
- `31d5cc0` ✓
- `844b6b3` ✓

## Self-Check: PASSED
