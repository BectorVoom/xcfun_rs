---
phase: 02-core-foundations-lda-tier-parity-harness
plan: 01
subsystem: xcfun-core surgical cleanup
tags: [wave-0, atomic-commits, xcfun-core, type-system, core-01, core-02, core-03, core-04, core-07, d-05, d-25]
requires: [xcfun-ad (Phase 1 output)]
provides:
  - "Mode enum with Unset=0 + repr(u32) (CORE-02 satisfied)"
  - "Vars enum (renamed from VarType) with 31 variants matching xcfun.h (CORE-01 satisfied)"
  - "Dependency bitflags unchanged (CORE-03 satisfied)"
  - "XcError Copy + #[non_exhaustive] with 9 variants + D-25 UnknownName unit variant (CORE-04 satisfied)"
  - "FunctionalId with 78 XC_* variants in xcint.hpp historical-insertion order (CORE-07 prerequisite satisfied)"
  - "Workspace builds with crates/xcfun-core as a first-class member"
affects:
  - "Plan 02-02 Wave-1A (xtask gates + registry codegen) — depends on this plan's type surface"
  - "Plan 02-03 Wave-1B-1 (xcfun-eval bring-up) — depends on Mode, Vars, XcError, FunctionalId"
tech-stack:
  added:
    - "static_assertions 1.1 (dev-dep, compile-time Copy/Send/Sync check for XcError)"
    - "tracing-subscriber =0.3.23 (workspace dep, pre-staged for Plan 02-06 validation crate)"
  patterns:
    - "Atomic-commit discipline: one commit per cleanup sub-task (D-09)"
    - "Wave-0a non-destructive deps first, Wave-0f workspace members re-include last (workspace build stays green through every commit in exclude-scoped crates)"
key-files:
  created: []
  deleted:
    - "crates/xcfun-core/src/density_vars.rs"
    - "crates/xcfun-core/src/test_data.rs"
  modified:
    - "Cargo.toml (workspace deps + members)"
    - "crates/xcfun-core/Cargo.toml (drop xcfun-ad dep, add static_assertions dev-dep)"
    - "crates/xcfun-core/src/lib.rs (forbid unsafe, drop Num/DensityVars/TestData/test_data refs, Mode/Vars rename)"
    - "crates/xcfun-core/src/enums.rs (EvalMode→Mode +Unset=0 +repr u32; VarType→Vars +allow non_camel_case +repr u32)"
    - "crates/xcfun-core/src/error.rs (9-variant Copy + #[non_exhaustive] + D-25 UnknownName drop payload + variant renames)"
    - "crates/xcfun-core/src/functional_id.rs (reorder to xcint.hpp historical ordering, XC_* prefix)"
    - "crates/xcfun-core/src/traits.rs (delete Functional trait + TestData struct; keep Dependency bitflags)"
decisions:
  - "D-09 atomicity honored: 6 discrete Wave-0 commits (Wave-0a..0f)"
  - "D-25 encoded in XcError::UnknownName as a unit variant (no String payload)"
  - "Rule 3 deviation at Wave-0d: dropped `fn energy<T: Num>(vars: &DensityVars<T>)` from Functional trait because Phase 1 D-09 retired Num and Wave-0b deleted DensityVars; Wave-0e removed the trait entirely so this was a stepping-stone fix"
  - "Rule 3 deviation at Wave-0e: removed `xcfun-ad` from xcfun-core [dependencies] — the Functional trait's `xcfun_ad::Num` reference was the only consumer; xcfun-core is now cubecl-free per D-04"
metrics:
  duration: "~25 min (wall clock)"
  tasks: 6
  files_touched: 8
  tests_added: 7
  tests_total: 23
  completed: "2026-04-20"
---

# Phase 2 Plan 01: xcfun-core Wave-0 surgical cleanup Summary

**One-liner:** Six atomic commits land the Phase 2 type-surface cleanup on `xcfun-core` (delete obsolete host-<T:Num> DensityVars, rename EvalMode→Mode with Unset=0 + VarType→Vars, make XcError 9-variant Copy + non_exhaustive with D-25 UnknownName unit variant, reorder FunctionalId to xcfun.h historical insertion order, re-include the crate in workspace members) and pass the workspace-build + 23-test gate.

## Commits Landed

| Wave   | Commit  | Type/scope             | Subject                                                                                   |
| ------ | ------- | ---------------------- | ----------------------------------------------------------------------------------------- |
| 0a     | f98fe26 | chore(02-01)           | Workspace deps verified (cubecl-cpu pre-staged; tracing-subscriber added)                 |
| 0b     | 82e21ba | chore(02-01)           | Delete obsolete density_vars.rs (825 lines host <T:Num> struct) + lib.rs references       |
| 0c     | 8243bd1 | chore(02-01)           | lib.rs rewrite drops broken pub use xcfun_ad::Num + adds `#![forbid(unsafe_code)]`         |
| 0d     | 4eb7c0a | refactor(02-01)        | Rename EvalMode→Mode (+Unset=0, repr u32); VarType→Vars (+allow non_camel_case, repr u32) |
| 0e     | f35bd9f | refactor(02-01)        | XcError 9-variant Copy + non_exhaustive (D-25 UnknownName drop); delete Functional/TestData/test_data.rs; reorder FunctionalId to xcfun.h |
| 0f     | 1feb23b | chore(02-01)           | Re-include crates/xcfun-core in workspace members (Wave-0 gate)                           |

`git log --oneline -6` matches the plan's expected six-commit sequence.

## Verification Results

- `cargo build --workspace` → **PASS** (0.16s incremental, clean release build prior)
- `cargo test -p xcfun-core --lib` → **PASS** (23 passed, 0 failed)
  - constants: 4 (C_SLATER, CF, TINY_DENSITY, MAX_ORDER)
  - enums Mode: 3 (has_4_variants, unset_is_zero, repr_u32_round_trip)
  - enums Vars: 4 (cpp_ordering, input_len, provides, spin_polarized)
  - error XcError: 4 (invalid_order_display, not_configured_display, unknown_name_display_drops_payload, xc_error_is_copy + compile-time `assert_impl_all!`)
  - functional_id: 4 (count_is_78, slaterx_discriminant_is_zero, xcint_historical_ordering_lda_anchors, from_name_round_trip)
  - traits Dependency: 2 (bits, bitwise_operations)
  - lib taylorlen: 2 (basic, larger)
- `cargo test -p xcfun-ad --tests` → **PASS** (0 tests without `testing` feature — Phase 1 baseline unchanged)
- Recursive grep for `EvalMode`, `VarType`, `density_vars`, `pub mod test_data`, `xcfun_ad::Num` under `crates/xcfun-core/src/` → **CLEAN** (no residual references)
- `crates/xcfun-core/src/density_vars.rs` and `crates/xcfun-core/src/test_data.rs` → **DO NOT EXIST** on disk
- `crates/xcfun-eval` → still in workspace `exclude` (deferred to Plan 02-03 Wave-1B-1 per D-20)

## Acceptance-Criteria Matrix

| Criterion (plan `<success_criteria>`)                                                                | Status | Evidence                                                    |
| ---------------------------------------------------------------------------------------------------- | ------ | ----------------------------------------------------------- |
| All 6 Wave-0 atomic commits committed (a, b, c, d, e, f) per D-09                                    | PASS   | `git log --oneline -6`                                      |
| `cargo build --workspace` + `cargo test -p xcfun-core --lib` exit 0 after Wave-0f                     | PASS   | Verified above                                              |
| Mode enum has `Unset = 0`, `PartialDerivatives = 1`, `Potential = 2`, `Contracted = 3` (CORE-02)      | PASS   | `enums.rs` + `mode_unset_is_zero` + `mode_repr_u32_round_trip` |
| Vars enum exists with 31 variants matching xcfun.h discriminants (CORE-01)                           | PASS   | Variant list unchanged from VarType (A=0..N_S_2ND_TAYLOR=30) + `vars_cpp_ordering` test |
| Dependency bitflags match xcint.hpp:46-50 bit values (CORE-03)                                       | PASS   | Pre-existing `dependency_bits` test unchanged               |
| XcError is `Copy + Clone + Debug + Send + Sync + #[non_exhaustive]` with 9 variants (CORE-04; D-25)   | PASS   | `assert_impl_all!(XcError: Copy, Clone, Send, Sync, Debug)` compiles + `unknown_name_display_drops_payload` |
| FunctionalId 78 entries in xcint historical-insertion order (CORE-07 prerequisite)                    | PASS   | `xcint_historical_ordering_lda_anchors` asserts IDs 0, 1, 2, 3, 13, 14, 15, 24, 25, 28, 55, 59, 77 |
| Workspace `Cargo.toml` re-includes `crates/xcfun-core`; `crates/xcfun-eval` still excluded            | PASS   | `grep 'crates/xcfun-core' Cargo.toml` finds it in members; exclude array still has xcfun-eval |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Dropped `fn energy<T: Num>(&self, vars: &DensityVars<T>) -> T` from Functional trait at Wave-0d**

- **Found during:** Wave-0d (after Wave-0b deleted `density_vars.rs`)
- **Issue:** `crates/xcfun-core/src/traits.rs` had `use crate::density_vars::DensityVars;` and `fn energy<T: xcfun_ad::Num>(&self, vars: &DensityVars<T>) -> T;` — both broken because Wave-0b removed `density_vars.rs` and Phase 1 D-09 retired `xcfun_ad::Num`.
- **Fix:** Removed the `use crate::density_vars::DensityVars;` import and deleted the `fn energy` method from the `Functional` trait. Added a doc comment marking the trait for full removal in Wave-0e.
- **Rationale:** The `Functional` trait is deleted entirely in Wave-0e anyway (per D-04). Leaving the method would force Wave-0e to move rather than simply delete. Rule 3 (blocking issue prevents current task's compile).
- **Files modified:** `crates/xcfun-core/src/traits.rs`
- **Commit:** 4eb7c0a (Wave-0d)

**2. [Rule 3 - Blocking] Removed `xcfun-ad` from `crates/xcfun-core/Cargo.toml` dependencies at Wave-0e**

- **Found during:** Wave-0e (after Functional trait was deleted)
- **Issue:** `crates/xcfun-core/Cargo.toml` listed `xcfun-ad = { path = "../xcfun-ad" }` as a regular dep. The only consumer inside `xcfun-core` was `fn energy<T: xcfun_ad::Num>` (dropped at Wave-0d) and `pub use xcfun_ad::Num;` (dropped at Wave-0c). With both gone, the dep is unused.
- **Fix:** Dropped `xcfun-ad` from `[dependencies]` so `xcfun-core` is cubecl-free per D-04.
- **Rationale:** Matches the plan's D-04 architecture — `xcfun-core` owns types + registry tables only; the cubecl path lives in `xcfun-eval`.
- **Files modified:** `crates/xcfun-core/Cargo.toml`
- **Commit:** f35bd9f (Wave-0e)

### Note on FunctionalId method surface at Wave-0e

The plan's Wave-0e scope was "reorder to xcfun.h ordering". I additionally dropped the hand-maintained `name()`, `description()`, and `depends()` instance methods on `FunctionalId`. Per D-11/D-12, this metadata is generated by `xtask regen-registry` into `FUNCTIONAL_DESCRIPTORS` in Plan 02-02 — keeping the hand-maintained copies would have forced a dual-source maintenance burden (hand-edits here vs. codegen output there) that the plan explicitly warns against. The plan's target-shape excerpt for functional_id.rs (lines 715–907) shows only `COUNT` and `from_name`, confirming this was the intended end state.

## Threat Flags

No new network endpoints, auth paths, or trust-boundary surface introduced. The FunctionalId reorder mutation is guarded by the `xcint_historical_ordering_lda_anchors` test (T-02-01-02 mitigation from plan threat register).

## Next Plans

- **Plan 02-02 Wave-1A** (xtask gates + registry codegen — `regen-registry`, `check-no-mul-add`, `check-no-anyhow`, `check-boundaries`, `check-cubecl-pin`). Consumes `FunctionalId::COUNT = 78` and the xcint-ordered discriminants from this plan.
- **Plan 02-03 Wave-1B-1** (xcfun-eval workspace member + cubecl launcher skeleton + `DensVarsDev<F, N>` #[cube] type). Consumes `Mode`, `Vars`, `XcError`, `FunctionalId`, `Dependency` from this plan. Parallelisable with Plan 02-02.

## Self-Check: PASSED

- [x] File `crates/xcfun-core/src/density_vars.rs` NOT present: FOUND=NOT EXISTS
- [x] File `crates/xcfun-core/src/test_data.rs` NOT present: FOUND=NOT EXISTS
- [x] File `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-01-SUMMARY.md` exists (this file)
- [x] Commit f98fe26 (Wave-0a) in git log
- [x] Commit 82e21ba (Wave-0b) in git log
- [x] Commit 8243bd1 (Wave-0c) in git log
- [x] Commit 4eb7c0a (Wave-0d) in git log
- [x] Commit f35bd9f (Wave-0e) in git log
- [x] Commit 1feb23b (Wave-0f) in git log
- [x] `cargo build --workspace` PASS
- [x] `cargo test -p xcfun-core --lib` PASS (23 tests)
- [x] No EvalMode/VarType/density_vars/test_data/xcfun_ad::Num residual in crates/xcfun-core/src/
