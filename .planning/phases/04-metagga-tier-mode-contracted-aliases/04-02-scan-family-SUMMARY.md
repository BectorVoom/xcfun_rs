---
phase: 04-metagga-tier-mode-contracted-aliases
plan: "02"
subsystem: xcfun-eval / mgga / SCAN family
tags: [mgga, scan, r2scan, r4scan, cubecl, comptime-dispatch]
dependency_graph:
  requires: [04-00-substrate]
  provides: [SCAN family kernels — ids 45-54]
  affects: [dispatch.rs, validation/build.rs, validation/c_stubs.cpp]
tech_stack:
  added: []
  patterns:
    - comptime ialpha/iinterp/idelec dispatch in get_SCAN_Fx and r2SCAN_C
    - module-level Rust const + F::cast_from() for all derived constants inside #[cube]
    - _u32 literal suffix on all comptime integer arguments
key_files:
  created:
    - crates/xcfun-eval/src/functionals/mgga/scanx.rs
    - crates/xcfun-eval/src/functionals/mgga/scanc.rs
    - crates/xcfun-eval/src/functionals/mgga/rscanx.rs
    - crates/xcfun-eval/src/functionals/mgga/rscanc.rs
    - crates/xcfun-eval/src/functionals/mgga/rppscanx.rs
    - crates/xcfun-eval/src/functionals/mgga/rppscanc.rs
    - crates/xcfun-eval/src/functionals/mgga/r2scanx.rs
    - crates/xcfun-eval/src/functionals/mgga/r2scanc.rs
    - crates/xcfun-eval/src/functionals/mgga/r4scanx.rs
    - crates/xcfun-eval/src/functionals/mgga/r4scanc.rs
  modified:
    - crates/xcfun-eval/src/functionals/mgga/mod.rs
    - crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs
    - crates/xcfun-eval/src/dispatch.rs
    - validation/build.rs
    - validation/c_stubs.cpp
decisions:
  - "scan_like.rs: all derived constants hoisted to module-level Rust const; inside #[cube] use F::cast_from(CONST) — this is the only pattern that avoids NativeExpand<f64> type errors"
  - "Exchange kernels: comptime ialpha/iinterp/idelfx tuple (not a single idelec) matches the actual scan_like.rs API which was fully built in 04-00"
  - "All integer literals passed as #[comptime] u32 use _u32 suffix (0_u32, 1_u32, 2_u32) — bare 0/1/2 infer as i32 and cause From<i32> compile errors"
  - "validation build failure is pre-existing in the worktree (xcfun-master not accessible at ../xcfun-master relative path); confirmed identical failure on unmodified prior commit"
metrics:
  duration: "multi-session (context limit hit once)"
  completed: "2026-04-26"
  tasks_completed: 2
  files_created: 10
  files_modified: 5
---

# Phase 04 Plan 02: SCAN Family Kernels Summary

**One-liner:** SCAN/rSCAN/r++SCAN/r2SCAN/r4SCAN exchange+correlation via comptime (ialpha, iinterp, idelfx/idelec) dispatch into shared scan_like helpers, with 42-constant module-level hoisting fix in scan_like.rs.

## What Was Built

10 SCAN-family metaGGA kernels (ids 45–54) for the xcfun-eval crate:

| ID | Symbol     | Kernel function     | Parameters (ialpha, iinterp, idelfx/idelec) |
|----|------------|---------------------|---------------------------------------------|
| 45 | XC_SCANC   | scanc_kernel        | (0, 0, 0)                                   |
| 46 | XC_SCANX   | scanx_kernel        | (0, 0, 0)                                   |
| 47 | XC_RSCANC  | rscanc_kernel       | (1, 1, 0)                                   |
| 48 | XC_RSCANX  | rscanx_kernel       | (1, 1, 0)                                   |
| 49 | XC_RPPSCANC| rppscanc_kernel     | (2, 1, 0)                                   |
| 50 | XC_RPPSCANX| rppscanx_kernel     | (2, 1, 0)                                   |
| 51 | XC_R2SCANC | r2scanc_kernel      | (2, 1, 1)                                   |
| 52 | XC_R2SCANX | r2scanx_kernel      | (2, 1, 1)                                   |
| 53 | XC_R4SCANC | r4scanc_kernel      | (2, 1, 2)                                   |
| 54 | XC_R4SCANX | r4scanx_kernel      | (2, 1, 2)                                   |

All exchange kernels spin-decompose: `0.5 * (fx(2*rho_a, 4*gaa, 2*tau_a) + fx(2*rho_b, 4*gbb, 2*tau_b))`.
All correlation kernels delegate directly to `r2SCAN_C(d, out, ialpha, iinterp, idelec, n)`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] NativeExpand<f64> type errors in scan_like.rs (42 errors)**
- **Found during:** Task 1 (first build attempt)
- **Issue:** Inside `#[cube]` functions, `let x: f64 = expr` wraps to `NativeExpand<f64>`. Using derived constants as `let k: f64 = A * B` inside `#[cube]` is not valid CubeCL — the macro treats them as runtime values.
- **Fix:** Hoisted all 42+ derived constants to module-level Rust `const` values. Inside `#[cube]`, reference via `F::cast_from(CONST_NAME)`. Added constants: `H0X_VAL`, `H1X_CNST`, `C2_X`, `DAMP_DENOM_X`, `DAMP_DENOM_C`, `ETA_VAL`, `ETA_TERM`, `C_AA`, `C_PA`, `C_PP`, `DAMP4_A_SQ`, `DAMP4_P4`, `AGE_C2_MU`, and 3 full sets of gcor2 coefficients (EU/EP/ALFM — 9 params each = 27 constants).
- **Files modified:** `crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs`
- **Commit:** 2541c92

**2. [Rule 1 - Bug] &&Array<F> double-reference in r2SCAN_C**
- **Found during:** Task 1 (same build)
- **Issue:** `let n_dens = &d.n` gives `&Array<F>`; passing `&n_dens` to a `#[cube]` function expecting `&Array<F>` produced `&&Array<F>` — a type error.
- **Fix:** Removed the extra `&` at all 4 call sites in `r2SCAN_C`.
- **Files modified:** `crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs`
- **Commit:** 2541c92

**3. [Rule 1 - Bug] u32/i32 literal mismatch in comptime arguments**
- **Found during:** Tasks 1 and 2 (multiple files)
- **Issue:** Bare integer literals `0, 1, 2` passed as `#[comptime] u32` parameters infer as `i32`, triggering `From<i32>: not satisfied for u32` errors.
- **Fix:** Applied `_u32` suffix to all integer literals passed as `#[comptime] u32` args: in `get_lsda1` (0_u32, 1_u32, 2_u32), in all exchange kernel `get_SCAN_Fx` calls (e.g. `1_u32, 1_u32, 0_u32`), and in all correlation `scanx_kernel` calls (0_u32, 0_u32, 0_u32).
- **Files modified:** `scan_like.rs`, `rscanx.rs`, `rppscanx.rs`, `r2scanx.rs`, `r4scanx.rs`, `scanx.rs`
- **Commit:** 2541c92

### Pre-existing Issues (out of scope, not fixed)

**validation crate build failure** — `validation/build.rs` uses `let xcfun_root = "../xcfun-master"` which resolves relative to the build script's manifest directory. In the git worktree (`worktrees/agent-ab8d7f83f335f19e7/`), the `xcfun-master` directory does not exist at that relative path. Confirmed identical failure on the prior commit (before any changes in this plan). This is a worktree infrastructure limitation, not introduced here.

## Structural Verification

```
grep -n "XC_SCANX|XC_R4SCANX|XC_R4SCANC" dispatch.rs  -> 6 matches
grep -c "SCAN" validation/build.rs                      -> 12
grep -rn "mul_add" mgga/ | grep scan                    -> 0 (only doc comment)
cargo build -p xcfun-eval                               -> Finished (0 errors, 11 warnings)
```

## Known Stubs

None. All 10 SCAN kernels are fully wired. Remaining stubs in `c_stubs.cpp` are M05/M06 family (13 entries) awaiting Plan 04-03.

## Self-Check: PASSED

All 10 kernel files found on disk. Commit 2541c92 present in git log. `cargo build -p xcfun-eval` exits 0 (11 warnings, 0 errors).
