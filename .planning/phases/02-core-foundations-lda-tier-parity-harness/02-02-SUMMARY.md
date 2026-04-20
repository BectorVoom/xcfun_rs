---
phase: 02-core-foundations-lda-tier-parity-harness
plan: 02
subsystem: xtask gates + auto-generated registry tables
tags: [wave-1a, xtask, registry, extractor, ACC-05, ACC-06, QG-01, QG-02, QG-06, QG-07, CORE-07, CORE-08, CORE-09, CORE-10]
requires: [02-01 (xcfun-core type surface, FunctionalId xcfun.h historical ordering)]
provides:
  - "xtask regen-registry binary + C++ extractor → auto-generated FUNCTIONAL_DESCRIPTORS (78 entries, 35 fully populated), VARS_TABLE (31 rows), ALIASES (empty)"
  - "xtask check-no-mul-add (ACC-06), check-no-anyhow (QG-01), check-boundaries (QG-02), check-cubecl-pin (QG-06) gates — all exit 0 against current workspace"
  - "xtask check-no-fma extended (ACC-05 inheritance): per-target SCAN_TARGETS table, forward-compat skip when xcfun-eval is not yet in the workspace"
  - "xtask validate CLI wrapper delegating to `cargo run -p validation --release -- <args>` (stub until Plan 02-06 adds validation crate)"
  - "crates/xcfun-core/src/registry/{mod.rs, generated/{FUNCTIONAL_DESCRIPTORS, VARS_TABLE, ALIASES}.rs + .sha256} — auto-generated, committed with drift-detecting stamps"
  - "crates/xcfun-core/tests/registry_tables.rs — 9 integration assertions (row counts, LDA test-data presence, D-24 threshold, Phase-2 empty aliases)"
affects:
  - "Plan 02-03 Wave-1B (xcfun-eval bring-up) — can consume FUNCTIONAL_DESCRIPTORS + VARS_TABLE for dependency + row-length validation at test time"
  - "Plan 02-04 (LDA tier-1 self-tests) — consumes test_in/test_out/test_threshold from FUNCTIONAL_DESCRIPTORS for 7 LDAs with upstream data"
  - "Plan 02-06 (validation harness) — tier-2 driver will iterate FUNCTIONAL_DESCRIPTORS to build the {functional,vars,mode,order} evaluation matrix"
  - "CI configuration (future) — should run all 5 xtask QG gates on every PR"
tech-stack:
  added:
    - "walkdir 2.x (xtask dep; crates/xcfun-eval source-tree walk for mul_add detection)"
    - "toml 0.8 (xtask dep; Cargo.toml [dependencies] parsing for anyhow-boundary gate)"
    - "tempfile 3.x (xtask dep; unused in the --check path after simplification — kept for future expansion)"
  patterns:
    - "Auto-generated source + SHA-256 stamp committed alongside (D-09 atomicity + QG-07 drift gate)"
    - "C++ extractor compiled fresh at xtask invocation via $CXX (mirrors Phase 1 regen-ad-fixtures)"
    - "nested `mod generated { ... }` with single import + triple `include!` — avoids duplicate-import collisions across sibling generated files"
    - "forward-compat xtask gates: check if target crate is in workspace metadata before invoking cargo rustc; skip gracefully with a stdout note"
key-files:
  created:
    - "xtask/assets/regen_registry/extractor.cpp"
    - "xtask/src/bin/regen_registry.rs"
    - "xtask/src/bin/check_no_mul_add.rs"
    - "xtask/src/bin/check_no_anyhow.rs"
    - "xtask/src/bin/check_boundaries.rs"
    - "xtask/src/bin/check_cubecl_pin.rs"
    - "xtask/src/bin/validate.rs"
    - "crates/xcfun-core/src/registry/mod.rs"
    - "crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs"
    - "crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs.sha256"
    - "crates/xcfun-core/src/registry/generated/VARS_TABLE.rs"
    - "crates/xcfun-core/src/registry/generated/VARS_TABLE.rs.sha256"
    - "crates/xcfun-core/src/registry/generated/ALIASES.rs"
    - "crates/xcfun-core/src/registry/generated/ALIASES.rs.sha256"
    - "crates/xcfun-core/tests/registry_tables.rs"
  modified:
    - "xtask/Cargo.toml (6 new [[bin]] + walkdir/toml/tempfile deps)"
    - "xtask/src/bin/check_no_fma.rs (SCAN_TARGETS table + forward-compat xcfun-eval skip)"
    - "crates/xcfun-core/src/lib.rs (pub mod registry + re-exports)"
    - "Cargo.lock (walkdir, toml, tempfile, hashbrown, indexmap, ... pulled in by xtask)"
decisions:
  - "D-09 atomicity: 3 discrete Wave-1A commits (Wave-1A-1 Cargo manifest + stubs, Wave-1A-2 extractor + regen-registry + generated files, Wave-1A-3 QG gates + registry tests)"
  - "Extractor design: regex-based parser targeting the regular FUNCTIONAL(XC_*) = { ... }; shape; handles nested `#ifdef HIGH_DENSITY`/`INEXACT_PI` in p86c.cpp via iterated regex fixed-point (keeps #else branches)"
  - "Rust codegen: `include!` into a single `mod generated { ... }` with shared imports — removes duplicate-import errors across sibling generated files"
  - "Rule 3 (Blocking) deviation at Wave-1A-1: plan said `cargo build -p xtask MUST pass` after manifest-only change, but `cargo build -p xtask` compiles ALL declared bins; created stub source files for the 6 new bins so Task 1 commits a compilable state. Real implementations land in Wave-1A-2 and Wave-1A-3."
  - "Rule 1 (Bug) deviation at Wave-1A-3: plan asserted 8 LDAs with test data including VWN3C; upstream VWN3C macro carries no test_in/test_out — adjusted integration test to the 7-LDA set that matches extractor reality (SLATERX, VWN5C, PW92C, PZ81C, TFK, LDAERFX, LDAERFC). Added explicit `lda_descriptors_without_upstream_data` test covering the 4 LDAs without upstream data (VWN3C, LDAERFC_JT, TW, VWK)."
  - "Bonus scope (non-deviation): extractor successfully parses 35 fully-populated FunctionalDescriptor entries (every upstream macro with a test-data tail), not just the 8 LDAs scoped by the plan. Remaining 43 are stubs. Total 78."
metrics:
  duration: "~45 min (wall clock, includes debugging extractor regex for nested ifdef in p86c.cpp)"
  tasks: 3
  files_created: 15
  files_modified: 4
  tests_added: 9
  commits: 3
  completed: "2026-04-20"
---

# Phase 2 Plan 02: xtask gates + registry codegen Summary

**One-liner:** Three atomic commits land the Wave 1A infrastructure tier — a cc-compiled C++ extractor that walks every FUNCTIONAL macro in xcfun-master, a Rust code generator that emits 78-entry FUNCTIONAL_DESCRIPTORS + 31-row VARS_TABLE + empty ALIASES with SHA-256 drift stamps, five per-commit QG xtask gates (check-no-mul-add, check-no-anyhow, check-boundaries, check-cubecl-pin, validate wrapper), and an extended check-no-fma that opts in to xcfun-eval kernel symbols once Plan 02-03 lands the crate.

## Commits Landed

| Wave | Commit | Type/scope | Subject |
| ---- | ------ | ---------- | ------- |
| 1A-1 | d4eb40a | chore(02-02) | xtask Cargo.toml — 6 new bin entries + walkdir/toml/tempfile + stub source files (so `cargo build -p xtask` stays green) |
| 1A-2 | 4f6a238 | feat(02-02) | regen-registry xtask binary + `xtask/assets/regen_registry/extractor.cpp` + `crates/xcfun-core/src/registry/{mod.rs, generated/*.rs + *.sha256}` |
| 1A-3 | c4b34e4 | feat(02-02) | xtask QG gates (check-no-mul-add, check-no-anyhow, check-boundaries, check-cubecl-pin, validate wrapper) + check-no-fma ACC-05 extension + registry_tables integration test (9 assertions) |

## Verification Results

```
cargo build --workspace                                  PASS
cargo run -p xtask --bin regen-registry                  PASS (writes 3 .rs + 3 .sha256)
cargo run -p xtask --bin regen-registry -- --check       PASS (no drift)
cargo run -p xtask --bin check-no-mul-add                PASS (xcfun-eval dir missing — vacuous)
cargo run -p xtask --bin check-no-anyhow                 PASS (7 library crate(s) checked)
cargo run -p xtask --bin check-boundaries                PASS (2 gated: xcfun-ad, xcfun-core)
cargo run -p xtask --bin check-cubecl-pin                PASS (2 crates @ 0.10.0-pre.3)
cargo run -p xtask --bin check-no-fma                    PASS (xcfun-eval skipped — Plan 02-03 adds it)
cargo test -p xcfun-core --lib                           23 passed
cargo test -p xcfun-core --test registry_tables          9 passed
```

Row counts on the committed generated sources:

- `FUNCTIONAL_DESCRIPTORS.rs`: 78 entries (35 fully-populated struct literals + 43 `FunctionalDescriptor::stub(...)`)
- `VARS_TABLE.rs`: 31 entries matching `xcfun-master/src/xcint.cpp:93-135`
- `ALIASES.rs`: empty slice (`pub static ALIASES: &[Alias] = &[]`)

## Acceptance-Criteria Matrix

| Criterion (plan `<success_criteria>`) | Status | Evidence |
| ------------------------------------- | ------ | -------- |
| xtask Cargo.toml extended with 6 new `[[bin]]` entries + deps (plan truth #1) | PASS | `git show d4eb40a` |
| `xtask/assets/regen_registry/extractor.cpp` present and compilable | PASS | `c++ -std=c++17 -O2 xtask/assets/regen_registry/extractor.cpp` exits 0 |
| `xtask/src/bin/regen_registry.rs` regenerates generated/*.rs + .sha256 (plan truths #2-#5) | PASS | Committed files; `regen-registry --check` exits 0 |
| 4 QG gates check-no-mul-add / check-no-anyhow / check-boundaries / check-cubecl-pin (plan truths #6-#9) | PASS | Each `cargo run` exits 0 |
| `validate` bin skeleton | PASS | Compiles; delegates to `cargo run -p validation --release -- <args>` (will fail until Plan 02-06) |
| `crates/xcfun-core/tests/registry_tables.rs` green (plan truth #10) | PASS | 9/9 tests pass |
| `cargo run -p xtask --bin regen-registry -- --check` exits 0 | PASS | See above |
| All 4 QG binaries exit 0 against current tree | PASS | See above |
| Each task committed atomically with `chore(02-02):` / `feat(02-02):` prefixes | PASS | 3 commits in `git log --oneline -3` |
| No modifications to STATE.md or ROADMAP.md | PASS | Orchestrator owns those writes; plan executor touched neither |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Stub source files created in Wave-1A-1 so `cargo build -p xtask` stays green after the Cargo.toml change.**

- **Found during:** Task 1 (Wave-1A-1) initial `cargo build -p xtask` invocation.
- **Issue:** The plan text says "`cargo build -p xtask` MUST pass — no other changes in this commit." It also claims "cargo only builds bins when invoked by name; cargo build -p xtask only runs when bin source exists." That is incorrect: `cargo build -p xtask` compiles EVERY bin declared in the manifest. Adding 6 `[[bin]]` entries without source files broke the build with six `couldn't read ...rs` errors.
- **Fix:** Wrote trivial 6-line stub source files for all 6 new bins. Each stub exits 1 with a message pointing to the wave that lands the real implementation. Wave-1A-2 replaces `regen_registry.rs`; Wave-1A-3 replaces the remaining five.
- **Files modified:** Added `xtask/src/bin/{regen_registry, check_no_mul_add, check_no_anyhow, check_boundaries, check_cubecl_pin, validate}.rs` as stubs.
- **Commit:** d4eb40a (Wave-1A-1).

**2. [Rule 1 — Bug] Plan asserted 8 LDAs with `test_in`/`test_out`; upstream VWN3C macro has no test data.**

- **Found during:** Writing `registry_tables.rs` integration test.
- **Issue:** Plan text `lda_descriptors_have_test_data` lists `[SLATERX, VWN3C, VWN5C, PW92C, PZ81C, TFK, LDAERFX, LDAERFC]` = 8 IDs. However `xcfun-master/src/functionals/vwn3.cpp` defines `FUNCTIONAL(XC_VWN3C) = { "VWN3 LDA Correlation functional", "...", XC_DENSITY, ENERGY_FUNCTION(vwn3c)};` — the macro ends right after `ENERGY_FUNCTION`, so the extractor correctly emits `test_in: null`. Asserting VWN3C has `test_in.is_some()` would make the test fail.
- **Fix:** Adjusted the assertion list to the 7 LDAs that actually carry upstream test data (SLATERX, VWN5C, PW92C, PZ81C, TFK, LDAERFX, LDAERFC). Added a complementary test `lda_descriptors_without_upstream_data` covering VWN3C + LDAERFC_JT + TW + VWK (4 LDAs without upstream test data; tier-2 vs C++ covers them per Plan 02-06).
- **Files modified:** `crates/xcfun-core/tests/registry_tables.rs`.
- **Commit:** c4b34e4 (Wave-1A-3).

**3. [Rule 3 — Blocking] Nested `#ifdef` in `p86c.cpp` initially broke the extractor.**

- **Found during:** Wave-1A-2 first run of regen-registry against xcfun-master.
- **Issue:** `xcfun-master/src/functionals/p86c.cpp` wraps its test data in a 2-level conditional: `#ifdef HIGH_DENSITY ... #ifdef INEXACT_PI ... #else ... #endif #else ... #ifdef INEXACT_PI ... #else ... #endif #endif`. My first `resolve_high_density` regex had a `.*?` lazy payload that matched across the nested `#ifdef` boundaries, collapsing the wrong pair.
- **Fix:** Tightened the regex to `#ifdef <IDENT> <non-ifdef-payload> #else <non-ifdef-payload> #endif` (explicit negative lookahead for `#ifdef|#else|#endif` inside each payload), and run it iteratively until fixed-point so the innermost pair always collapses first. Now handles arbitrary nesting depth (capped at 32 iterations).
- **Files modified:** `xtask/assets/regen_registry/extractor.cpp` `resolve_high_density` function.
- **Commit:** 4f6a238 (Wave-1A-2).

### Bonus Scope (non-deviation)

The C++ extractor successfully parses **every** functional's FUNCTIONAL macro, not just the 11 LDAs. Result: 35 fully-populated entries in `FUNCTIONAL_DESCRIPTORS` (every xcfun-master functional that ships a test-data tail), plus 43 stubs (M06*, SCAN*, PBE*, etc. whose test data lives in separate `_corr` / `_eps` helper .cpp files the extractor does not yet follow). The plan only required 8 LDAs; this is extra coverage for future phases.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns at trust boundaries. The extractor writes to `crates/xcfun-core/src/registry/generated/` and nowhere else. T-02-02-01 (tampering) mitigated by the SHA-256 stamps + `--check` mode. T-02-02-02 (spoofing via malicious FUNCTIONAL macro) is out of scope because xcfun-master is vendored and content-hash-pinned.

## Next Plans

- **Plan 02-03 Wave-1B-1** (xcfun-eval workspace member + cubecl launcher skeleton + `DensVarsDev<F, N>` `#[cube]` type). Will consume `FUNCTIONAL_DESCRIPTORS` + `VARS_TABLE` for row-length and dependency validation at test time. Once xcfun-eval is in the workspace, `check-no-mul-add` starts scanning `crates/xcfun-eval/src/functionals/` and `check-no-fma` extends its xcfun-eval asm scan.
- **Plan 02-04** (LDA tier-1 self-tests). Will iterate `FUNCTIONAL_DESCRIPTORS[id]` for the 7 LDAs with `test_in.is_some()` and assert 1e-12 parity; LDAERFX/LDAERFC relax to 1e-7 per D-24 (already encoded in the generated descriptors).
- **Plan 02-06** (validation harness). Will be the first consumer of the `validate` xtask wrapper.

## Self-Check: PASSED

- [x] File `xtask/assets/regen_registry/extractor.cpp` present
- [x] File `xtask/src/bin/regen_registry.rs` present (non-stub)
- [x] File `xtask/src/bin/check_no_mul_add.rs` present (non-stub)
- [x] File `xtask/src/bin/check_no_anyhow.rs` present (non-stub)
- [x] File `xtask/src/bin/check_boundaries.rs` present (non-stub)
- [x] File `xtask/src/bin/check_cubecl_pin.rs` present (non-stub)
- [x] File `xtask/src/bin/validate.rs` present (non-stub)
- [x] File `crates/xcfun-core/src/registry/mod.rs` present
- [x] File `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs` present (78 entries)
- [x] File `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` present (31 entries)
- [x] File `crates/xcfun-core/src/registry/generated/ALIASES.rs` present (empty slice)
- [x] All three `.sha256` stamp files present
- [x] File `crates/xcfun-core/tests/registry_tables.rs` present
- [x] Commit d4eb40a (Wave-1A-1) in git log
- [x] Commit 4f6a238 (Wave-1A-2) in git log
- [x] Commit c4b34e4 (Wave-1A-3) in git log
- [x] `cargo build --workspace` PASS
- [x] `cargo test -p xcfun-core --test registry_tables` PASS (9/9)
- [x] `cargo run -p xtask --bin regen-registry -- --check` exits 0
- [x] All 4 QG binaries (check-no-mul-add, check-no-anyhow, check-boundaries, check-cubecl-pin) exit 0
- [x] check-no-fma exits 0 with xcfun-eval skipped (ACC-05 forward-compat)
- [x] No modifications to `.planning/STATE.md` or `.planning/ROADMAP.md` (orchestrator owns those)
