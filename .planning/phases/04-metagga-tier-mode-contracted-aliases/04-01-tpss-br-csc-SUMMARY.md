---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "01"
subsystem: xcfun-eval / mgga
tags: [mgga, tpss, br-family, csc, dispatch, validation]
dependency_graph:
  requires: [04-00]
  provides: [tpssx_kernel, tpssc_kernel, revtpssx_kernel, revtpssc_kernel, tpsslocc_kernel, brx_kernel, brc_kernel, brxc_kernel, csc_kernel]
  affects: [dispatch_kernel, supports, validation/build.rs, validation/c_stubs.cpp]
tech_stack:
  added: []
  patterns: [ctaylor-in-place-fix, br-newton-cube, polarized-helper, csc-energy-cube]
key_files:
  created:
    - crates/xcfun-eval/src/functionals/mgga/tpssx.rs
    - crates/xcfun-eval/src/functionals/mgga/tpssc.rs
    - crates/xcfun-eval/src/functionals/mgga/revtpssx.rs
    - crates/xcfun-eval/src/functionals/mgga/revtpssc.rs
    - crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs
    - crates/xcfun-eval/src/functionals/mgga/brx.rs
    - crates/xcfun-eval/src/functionals/mgga/brc.rs
    - crates/xcfun-eval/src/functionals/mgga/brxc.rs
    - crates/xcfun-eval/src/functionals/mgga/csc.rs
  modified:
    - crates/xcfun-eval/src/functionals/mgga/mod.rs
    - crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs
    - crates/xcfun-eval/src/functionals/mgga/shared/constants.rs
    - crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs
    - crates/xcfun-eval/src/functionals/mgga/shared/cs.rs
    - crates/xcfun-eval/src/dispatch.rs
    - validation/build.rs
    - validation/c_stubs.cpp
decisions:
  - "BR Newton-inverse implemented entirely in #[cube] using br_newton_cube (20-iter fixed unroll) — no host-side pre-seeding needed; cubecl Float intrinsics (exp/ln/sqrt/abs) cover all branches"
  - "polarized() helper takes 2*taua from caller matching brx.cpp:104 call convention"
  - "Ekström FIXME (brx.cpp:100) preserved verbatim using ctaylor_exp not ctaylor_expm1 per CONTEXT D-26 algorithmic-identity rule"
  - "BRX/BRC/BRXC co-located in brx.rs mirroring single-file brx.cpp structure; brc.rs/brxc.rs are thin pub use re-exports"
  - "ctaylor_abs helper added for runtime-conditional sign flip on CNST slot per T-04-01-01 threat model"
  - "cs.rs refactored from individual-field skeleton to DensVarsDev signature to match kernel call convention"
metrics:
  duration: "~3 hours (continued across sessions)"
  completed: "2026-04-26"
  tasks_completed: 2
  files_changed: 17
  insertions: 2914
  deletions: 190
---

# Phase 04 Plan 01: TPSS + BR + CSC Kernel Ports Summary

9 functional kernel bodies (TPSS×5, BR×3, CSC×1) as `#[cube]` functions wired into dispatch; dispatch extended to 55 supported functional IDs; validation C++ build updated (+7 .cpp files, -9 stubs); tier-1 self-tests GREEN.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | TPSS family (5 kernels) + validation | 2bfe9b7 | tpssx.rs, tpssc.rs, revtpssx.rs, revtpssc.rs, tpsslocc.rs |
| 2 | BR family (3) + CSC (1) + dispatch | 6287312 | brx.rs, brc.rs, brxc.rs, csc.rs, br_like.rs, cs.rs |

## Verification Results

- `cargo build -p xcfun-eval --release` — EXIT 0, warnings only
- `cargo test -p xcfun-eval --test self_tests --features testing` — 1/1 passed (19s)
- `cargo test -p validation` — 10/10 passed
- `grep -n "XC_TPSSX\|XC_BRX\|XC_CSC" dispatch.rs` — all 3 match arms present
- `grep -c "tpss\|brx\|\"cs\"" validation/build.rs` — 8 matches
- No `mul_add` in any new mgga kernel file

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] In-place borrow conflicts in tpss_like.rs (~40 instances)**
- **Found during:** Task 1
- **Issue:** `ctaylor_foo(&x, ..., &mut x, n)` borrows `x` both immutably and mutably — rejected by Rust borrow checker
- **Fix:** Every in-place operation uses a distinct `_raw` or sequentially-numbered output variable
- **Files modified:** `shared/tpss_like.rs`, `tpsslocc.rs`
- **Commit:** 2bfe9b7

**2. [Rule 1 - Bug] `TPSS_E_F64.sqrt()` type mismatch inside `#[cube]`**
- **Found during:** Task 1
- **Issue:** `f64::sqrt` inside `#[cube]` returns `NativeExpand<f64>`, not `f64`; `f64::sqrt` is not `const fn` so cannot be used in `const` context
- **Fix:** Pre-computed `TPSS_SQRT_E_F64` and `REVTPSS_SQRT_E_F64` as literal `f64` constants in `constants.rs`; replaced sqrt calls with inline `F::cast_from(CONSTANT * ...)`
- **Files modified:** `shared/constants.rs`, `shared/tpss_like.rs`
- **Commit:** 2bfe9b7

**3. [Rule 2 - Missing Critical Functionality] Wave-0 br_like.rs skeleton not implementable as-is**
- **Found during:** Task 2
- **Issue:** Wave-0 `br_like.rs` skeleton noted "Host-side: seed out[0] = br_scalar(t[0])" as step 1 of `br_t`. Host-side functions cannot be called from inside `#[cube]`. The skeleton's placeholder bodies produced zero output.
- **Fix:** Implemented `br_newton_cube<F: Float>(z: F) -> F` as a fully `#[cube]`-compatible Newton solver using cubecl Float intrinsics (`F::exp`, `F::ln`, `F::sqrt`, `F::abs`). 20-iteration fixed unroll replaces the early-exit loop (identical result at convergence). Added runtime branching for the four initial-guess branches. Updated `br_t` to call `br_newton_cube(t[0])` then `ctaylor_br_inverse`.
- **Files modified:** `shared/br_like.rs`
- **Commit:** 6287312

**4. [Rule 2 - Missing Critical Functionality] cs.rs signature mismatch**
- **Found during:** Task 2
- **Issue:** Wave-0 `cs.rs` skeleton had `csc_energy(a, b, n_density, ...)` taking individual `&Array<F>` parameters. All kernel bodies use `d: &DensVarsDev<F>`. The skeleton couldn't be called as `csc_energy(d, out, n)`.
- **Fix:** Refactored `csc_energy` to take `(d: &DensVarsDev<F>, out: &mut Array<F>, n)`, implemented full body.
- **Files modified:** `shared/cs.rs`
- **Commit:** 6287312

## Known Stubs

None. All 9 kernel bodies are fully implemented (no placeholder returns, no hardcoded zeros).

## Threat Flags

No new attack surface introduced. All changes are pure numerical kernel ports with no FFI entry points. Per plan threat model T-04-01-01/T-04-01-02 — both accepted.

## Self-Check: PASSED

Files exist:
- `crates/xcfun-eval/src/functionals/mgga/tpssx.rs` — FOUND
- `crates/xcfun-eval/src/functionals/mgga/brx.rs` — FOUND
- `crates/xcfun-eval/src/functionals/mgga/csc.rs` — FOUND
- `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-01-tpss-br-csc-SUMMARY.md` — FOUND

Commits exist:
- `2bfe9b7` — FOUND (feat(04-01): port TPSS family)
- `6287312` — FOUND (feat(04-01): port BR family + CSC)
