---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "04"
subsystem: api
tags: [alias-engine, parameters, registry, xtask, tdd, ffi-prep]

# Dependency graph
requires:
  - phase: 02-substrate
    provides: FunctionalId enum, ALIASES stub, FUNCTIONAL_DESCRIPTORS, registry pipeline
  - phase: 04-00-substrate
    provides: ParameterId/PARAMETERS scaffold expectations, settings[82] D-05 decision
  - phase: 04-03-m0x-blocx
    provides: 78 functional dispatch (live registry size for settings array)
provides:
  - ParameterId enum (4 variants, #[repr(u32)] discriminants 78..=81)
  - PARAMETERS [ParameterEntry; 4] static registry with defaults
  - 46-entry ALIASES static slice populated from aliases.cpp
  - Functional::new() / set() / get() with 3-case recursion (functional/parameter/alias)
  - settings: [f64; 82] field replacing parameters: [f64; 4]
  - DEFAULT_SETTINGS [f64; 82] const block initializer
  - xtask regen-registry extension parsing aliases.cpp + common_parameters.cpp
  - Drift gate coverage for parameters.rs (sha256 stamp)
affects: [05-c-abi, 05-python-bindings, 04-05-mode-contracted, 04-06-validation-signoff]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "alias engine — case-insensitive name lookup with priority order: functional → parameter → alias"
    - "settings[82] = 78 functional weights ⊕ 4 parameter values (mirrors XCFunctional.hpp:35)"
    - "additive accumulation for functionals (settings[id] += value)"
    - "overwrite semantics for parameters (settings[id] = value)"
    - "FIXME-preserved: alias parameter terms multiplied by value (algorithmic identity for 1e-12)"
    - "regen-registry extension via Rust-side C source parsing (no extractor changes)"

key-files:
  created:
    - "crates/xcfun-core/src/registry/generated/parameters.rs — PARAMETERS static slice"
    - "crates/xcfun-core/src/registry/generated/parameters.rs.sha256 — drift stamp"
    - "crates/xcfun-core/tests/parameter_and_alias_registry.rs — 13 RED→GREEN tests"
    - "crates/xcfun-eval/tests/alias_canary.rs — 16 alias canary tests"
  modified:
    - "crates/xcfun-core/src/enums.rs — added ParameterId enum + 3 unit tests"
    - "crates/xcfun-core/src/lib.rs — re-export ParameterId/PARAMETERS/ParameterEntry"
    - "crates/xcfun-core/src/registry/mod.rs — include parameters.rs, import ParameterId"
    - "crates/xcfun-core/src/registry/generated/ALIASES.rs — populated 46 entries"
    - "crates/xcfun-core/src/registry/generated/ALIASES.rs.sha256 — refreshed stamp"
    - "crates/xcfun-core/tests/registry_tables.rs — aliases test now expects len == 46"
    - "crates/xcfun-eval/src/functional.rs — Functional::{new,set,get}, settings[82], DEFAULT_SETTINGS"
    - "crates/xcfun-eval/tests/{potential_lda,potential_gga,potential_parity,self_tests}.rs — DEFAULT_PARAMETERS → DEFAULT_SETTINGS"
    - "validation/src/driver.rs — DEFAULT_PARAMETERS → DEFAULT_SETTINGS (2 sites)"
    - "xtask/src/bin/regen_registry.rs — Rust-side parsers + emitters for aliases + parameters"

key-decisions:
  - "Hand-rolled C-string + f64 token parser in regen-registry instead of extending the C++ extractor — keeps the new pipeline in pure Rust, zero extra deps, ~250 LOC."
  - "Preserved C++ FIXME at XCFunctional.cpp:393 (alias parameter terms ARE multiplied by value) — required by the algorithmic-identity rule for 1e-12 parity."
  - "Functional::new() returns Mode::Unset / vars=A_B / order=0 / empty weights with parameter defaults — matches XCFunctional() constructor; downstream code calling eval() must still set mode + weights explicitly."
  - "Lookup priority order matches C++ verbatim (functional → parameter → alias). Names like OPTX, PBEX (which exist as both alias and functional) route to the functional case first."
  - "Settings array kept orthogonal to weights — set/get manipulate settings[]; weights remains the static slice consumed by eval(). C ABI / Phase 5 facade will bridge the two when xcfun_set is wired through to the launch loop."
  - "validation/c_stubs.cpp left untouched — the regen-registry would re-emit 67 stubs but the file was deliberately stripped to 0 stubs by Phase 4 prior plans (native Rust ports now provide the FUNCTIONAL specializations); reverted to the committed empty form."

patterns-established:
  - "Pattern: case-insensitive symbol lookup mirrors C strcasecmp via .eq_ignore_ascii_case() on bare names with optional XC_ prefix stripping."
  - "Pattern: const-block initializers for static arrays with non-trivial defaults (DEFAULT_SETTINGS uses ParameterId::XC_RANGESEP_MU as usize indexing in const context)."
  - "Pattern: regen-registry includes both extractor-driven (functional descriptors, vars table) and Rust-parser-driven (aliases, parameters) emitters under the same drift gate."

requirements-completed: [ALIAS-01, ALIAS-02, ALIAS-03, ALIAS-04, ALIAS-05, ALIAS-06]

# Metrics
duration: 25min
completed: 2026-04-26
---

# Phase 04 Plan 04: Alias Engine + Parameter Table Summary

**ParameterId enum (4 variants) + 46-alias static registry from aliases.cpp + Functional::set/get implementing the 3-case recursion (functional additive / parameter overwrite / alias recursive) with settings[82] mirroring XCFunctional.hpp:35.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-04-26T08:43:25Z
- **Completed:** 2026-04-26T09:08:36Z
- **Tasks:** 2 (each TDD: RED → GREEN)
- **Files modified:** 11 (+ 4 created)

## Accomplishments

- **Parameter registry shipped end-to-end.** `ParameterId` enum at discriminants 78..=81 mirrors `list_of_functionals.hpp:99-105`. `PARAMETERS [ParameterEntry; 4]` carries name (XC_-stripped), description, and default — sourced from `common_parameters.cpp:17-29`.
- **All 46 aliases populated.** `ALIASES` static slice now contains every entry from `aliases.cpp:17-138`, including the negative-weight canary `camcompx → beckecamx -1.0` and the high-precision constants in `kt2`, `OPTX`, `KT3X`.
- **Alias engine bit-identical to C++.** `Functional::set` ports `xcfun_set` (XCFunctional.cpp:369-405) with the FIXME at L393 preserved (parameter terms ARE multiplied by `value` through aliases) — required by the 1e-12 algorithmic-identity contract.
- **Settings array expanded.** `parameters: [f64; 4]` → `settings: [f64; 82]` updated across 17 call sites (in-file unit tests, 4 integration test files, validation driver).
- **xtask regen-registry extended.** Rust-side parsers for `aliases.cpp` and `common_parameters.cpp` (no C++ extractor changes); drift gate now covers `parameters.rs` alongside the existing 3 generated files.
- **45 new test assertions** (13 in `parameter_and_alias_registry.rs`, 16 in `alias_canary.rs`, 3 in enums.rs, 13 unit tests across functional.rs touched paths).

## Task Commits

Each task was developed strict-TDD (RED commit then GREEN commit):

1. **Task 1: ParameterId + parameter/alias registry + xtask extension**
   - RED: `2afc81b` — `test(04-04): add RED tests for ParameterId + parameter/alias registry` (13 tests)
   - GREEN: `50dd83a` — `feat(04-04): ParameterId enum + 46-alias / 4-parameter registry + xtask extension`
2. **Task 2: Functional::set/get alias engine + settings[82] expansion**
   - RED: `a4e7d44` — `test(04-04): add RED tests for Functional::set/get alias engine` (16 tests)
   - GREEN: `45ccb7f` — `feat(04-04): Functional::set/get alias engine + settings[82] expansion`

**Plan metadata:** (this SUMMARY commit)

## Files Created/Modified

### Created
- `crates/xcfun-core/src/registry/generated/parameters.rs` — `[ParameterEntry; 4]` static slice with defaults from `common_parameters.cpp:17-29`. Generated by extended xtask regen-registry.
- `crates/xcfun-core/src/registry/generated/parameters.rs.sha256` — drift stamp.
- `crates/xcfun-core/tests/parameter_and_alias_registry.rs` — 13 RED→GREEN tests for ParameterId discriminants, case-insensitive lookup, PARAMETERS layout, ALIASES count and content (b3lyp, kt2, camcompx, camb3lyp documented weights; cross-table integrity check).
- `crates/xcfun-eval/tests/alias_canary.rs` — 16 alias canary tests covering all PLAN §behavior expectations.

### Modified
- `crates/xcfun-core/src/enums.rs` — added `ParameterId` enum (4 variants, `#[repr(u32)]` 78..=81), `from_name` (case-insensitive with optional `XC_` prefix), `default_value` (const fn), `COUNT = 4`. 3 new unit tests.
- `crates/xcfun-core/src/lib.rs` — re-export `ParameterId`, `PARAMETERS`, `ParameterEntry`.
- `crates/xcfun-core/src/registry/mod.rs` — `include!("generated/parameters.rs")` + `use crate::enums::ParameterId` in shared scope.
- `crates/xcfun-core/src/registry/generated/ALIASES.rs` — populated 46 entries (was empty stub `&[]`).
- `crates/xcfun-core/src/registry/generated/ALIASES.rs.sha256` — refreshed by regen.
- `crates/xcfun-core/tests/registry_tables.rs` — `aliases_empty_for_phase_2` → `aliases_populated_in_phase_4` (now expects `ALIASES.len() == 46`).
- `crates/xcfun-eval/src/functional.rs` — `Functional::new()` constructor, `Functional::set` (3-case recursion), `Functional::get` (functional + parameter only), `settings: [f64; 82]` field, `DEFAULT_SETTINGS` const, `Default` impl. 12 in-file unit-test sites updated `parameters: DEFAULT_PARAMETERS` → `settings: DEFAULT_SETTINGS`.
- `crates/xcfun-eval/tests/potential_lda.rs`, `potential_gga.rs`, `potential_parity.rs`, `self_tests.rs` — same field rename.
- `validation/src/driver.rs` — same field rename (2 sites: partial-derivatives driver + potential driver).
- `xtask/src/bin/regen_registry.rs` — added `parse_aliases_cpp` + `parse_common_parameters_cpp` Rust parsers (with `strip_cpp_comments` helper), `emit_aliases_rs(records)` and `emit_parameters_rs(records)` emitters, integrated into the main pipeline + drift-gate.

## Decisions Made

1. **Rust-side C parsing for aliases + parameters** instead of extending the C++ extractor. The data is small (50 entries total), the parsers are ~250 LOC, and keeping them in Rust avoids touching `xtask/assets/regen_registry/extractor.cpp` (which would re-trigger the extractor recompile chain).
2. **C++ FIXME at XCFunctional.cpp:393 preserved verbatim** — alias parameter terms are multiplied by `value`. The plan calls this out explicitly: "The algorithmic-identity rule forbids 'fixing' the FIXME." Test `test_camcompx_parameter_overwrite_with_value_weight` pins this behaviour.
3. **`settings` and `weights` kept orthogonal** — `set/get` mutate `settings[]`; `eval()` reads `weights`. The C ABI in Phase 5 will bridge them when `xcfun_eval` flows through `xcfun_set`. This avoids breaking the 30+ existing call sites that build `Functional` via struct literal with a static `weights` slice.
4. **`Functional::new()` defaults to `Mode::Unset` and `vars: Vars::A_B`** — the latter is a placeholder; callers must call `eval_setup` (or set fields explicitly) before `eval()`. Matches the C++ constructor which leaves `mode = XC_MODE_UNSET`.
5. **`validation/c_stubs.cpp` left at the committed empty form** — running `regen-registry` would emit 67 C++ stubs (the historical baseline before Phase 3+4 stripped them as native Rust ports landed). Re-emitting the stubs would create duplicate-symbol link errors. The drift-gate `--check` still passes because it compares regen output to the `.sha256` stamp, not to the on-disk file.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Existing `aliases_empty_for_phase_2` test would assert FALSE after populating ALIASES**
- **Found during:** Task 1 (registry build)
- **Issue:** `crates/xcfun-core/tests/registry_tables.rs:106-110` asserted `ALIASES.len() == 0`. Populating ALIASES to 46 entries breaks this immediately.
- **Fix:** Renamed to `aliases_populated_in_phase_4` and changed the assertion to `== 46`.
- **Files modified:** `crates/xcfun-core/tests/registry_tables.rs`
- **Verification:** `cargo test -p xcfun-core` — 9 registry tests pass.
- **Committed in:** `50dd83a` (Task 1 GREEN).

**2. [Rule 3 - Blocking] xcfun-master not present inside the parallel-executor worktree**
- **Found during:** Task 1 (running `cargo run -p xtask --bin regen-registry`)
- **Issue:** `.gitignore` excludes `xcfun-master`; the worktree under `.claude/worktrees/agent-…` had no copy, so the extractor binary failed with "not a directory: xcfun-master/src/functionals".
- **Fix:** Created a symlink `worktrees/.../xcfun-master → /home/chemtech/workspace/xcfun_rs/xcfun-master`. The symlink is not committed (matches `.gitignore`).
- **Files modified:** None tracked (symlink is gitignored).
- **Verification:** `cargo run -p xtask --bin regen-registry --check` exits 0.
- **Committed in:** N/A — environmental fix only.

**3. [Rule 1 - Discovery] `validation/c_stubs.cpp` would be reverted by regen-registry**
- **Found during:** Task 1 (first `cargo run -p xtask --bin regen-registry` invocation)
- **Issue:** Regen-registry blindly emits 67 FUNCTIONAL stub macros. Phase 3 + Phase 4 prior plans manually stripped this file to 0 stubs as native Rust ports landed; bringing the stubs back would cause duplicate-symbol link errors in the validation crate.
- **Fix:** Reverted `validation/c_stubs.cpp` via `git checkout HEAD --` after the regen run; left the regen tool itself unchanged (the gap between regen output and on-disk file is a pre-existing inconsistency the prior plans introduced — out of scope for Plan 04-04). The drift-gate `--check` still passes because it compares regen-output hashes against the committed `.sha256` stamps, not against the on-disk file.
- **Files modified:** `validation/c_stubs.cpp` (reverted to HEAD).
- **Verification:** `cargo build -p validation` succeeds; `cargo run -p xtask --bin regen-registry --check` exits 0.
- **Committed in:** N/A (no net diff to validation/c_stubs.cpp).
- **Logged for follow-up:** When a future plan touches the regen-registry c_stubs emitter, it should be made aware of the native-port list per Phase 3+4 to avoid emitting stubs for already-ported functionals.

---

**Total deviations:** 3 auto-fixed (1 bug, 1 blocking, 1 discovery / no-op).
**Impact on plan:** All within the deviation rules' scope. No scope creep; no architectural changes.

## Issues Encountered

- **Pre-existing drift-gate semantics:** The regen-registry's `--check` mode compares regen-output hashes against the committed `.sha256` stamp, but does NOT verify the on-disk generated file matches the stamp. This means the on-disk `validation/c_stubs.cpp` (manually stripped to 0 stubs in Phase 4 prior plans) and the stamp value (still pointing at the 67-stub regen output) are inconsistent. This was discovered but is out of scope for Plan 04-04. Documented above under deviation 3.

## Verification Evidence

```text
$ cargo test -p xcfun-core
test result: ok. 26 passed (lib unit tests including 3 ParameterId tests)
test result: ok. 13 passed (parameter_and_alias_registry.rs)
test result: ok. 9 passed (registry_tables.rs)

$ cargo test -p xcfun-eval --features testing
test result: ok. 17 passed (lib unit tests)
test result: ok. 16 passed (alias_canary.rs)
test result: ok. 2 passed (potential_lda.rs)
test result: ok. 1 passed (potential_parity.rs)
test result: ok. 1 passed (self_tests.rs)
[+ pack_ctaylor_inputs, regularize_*, cubecl_densvars_spike — all pass]

$ cargo run -p xtask --bin regen-registry -- --check
regen-registry --check: OK (no drift)

$ cargo check --workspace
Finished `dev` profile in 22.56s (warnings only, pre-existing)

$ grep -rn 'use anyhow' crates/xcfun-eval/src/ crates/xcfun-core/src/
(no matches)

$ grep -n 'mul_add' crates/xcfun-eval/src/functional.rs
(no matches)
```

## User Setup Required

None - no external service configuration required. The existing symlink `xcfun-master → workspace/xcfun_rs/xcfun-master` is environmental and gitignored; reproduction in a fresh worktree requires the same symlink for `xtask regen-registry` to find the C++ source tree.

## Next Phase Readiness

- **Plan 04-05 (mode-contracted)** can build on the alias engine: `xcfun_set("camb3lyp", 1.0)` now produces a fully-populated `settings[]` ready for the contracted-mode launch path.
- **Plan 04-06 (validation-signoff)** can reference any of the 46 aliases by name through `Functional::set`.
- **Phase 5 (C ABI)** has the surface ready: `xcfun_set/xcfun_get` map directly to `Functional::set/Functional::get`. Bridging `settings[]` to the launch loop's `weights` slice is the last gap.
- **No blockers** — all xcfun-core, xcfun-eval, and validation crates compile and test clean.

## Self-Check: PASSED

Verification:

- `crates/xcfun-core/src/registry/generated/parameters.rs` exists ✓
- `crates/xcfun-core/src/registry/generated/parameters.rs.sha256` exists ✓
- `crates/xcfun-core/tests/parameter_and_alias_registry.rs` exists ✓
- `crates/xcfun-eval/tests/alias_canary.rs` exists ✓
- Commits: `2afc81b`, `50dd83a`, `a4e7d44`, `45ccb7f` all present in `git log` ✓
- All target tests pass under `cargo test -p xcfun-core` and `cargo test -p xcfun-eval --features testing` ✓
- Drift gate `cargo run -p xtask --bin regen-registry --check` exits 0 ✓

---
*Phase: 04-metagga-tier-mode-contracted-aliases*
*Completed: 2026-04-26*
