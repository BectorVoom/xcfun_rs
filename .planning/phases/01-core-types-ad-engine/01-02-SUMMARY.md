---
phase: 01-core-types-ad-engine
plan: 02
subsystem: core
tags: [types, enums, bitflags, density-vars, functional-id, error-handling, thiserror]

requires:
  - phase: 01-core-types-ad-engine/plan-01
    provides: "xcfun-ad crate with Num trait, CTaylor, value_f64(), set_constant()"
provides:
  - "DensityVars<T> with from_input() for all 31 VarType variants"
  - "VarType enum (31 variants) with input_len(), provides(), is_spin_polarized()"
  - "FunctionalId enum (78 variants) with from_name(), name(), description(), depends()"
  - "EvalMode enum (3 variants)"
  - "Dependency bitflags (DENSITY, GRADIENT, LAPLACIAN, KINETIC, JP)"
  - "XcError with 7 variants and FFI error codes"
  - "Physical constants matching C++ xcfun (C_SLATER, CF, TINY_DENSITY)"
  - "taylorlen() const fn"
  - "Functional trait definition"
  - "Stub crates: xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python"
affects: [functionals, eval, gpu, ffi, python]

tech-stack:
  added: [thiserror, bitflags]
  patterns: [flat-densvars-struct, regularization-via-num-trait, c++-matching-enum-ordering]

key-files:
  created:
    - crates/xcfun-core/Cargo.toml
    - crates/xcfun-core/src/lib.rs
    - crates/xcfun-core/src/constants.rs
    - crates/xcfun-core/src/error.rs
    - crates/xcfun-core/src/enums.rs
    - crates/xcfun-core/src/traits.rs
    - crates/xcfun-core/src/functional_id.rs
    - crates/xcfun-core/src/density_vars.rs
    - crates/xcfun-core/src/test_data.rs
    - crates/xcfun-functionals/Cargo.toml
    - crates/xcfun-functionals/src/lib.rs
    - crates/xcfun-eval/Cargo.toml
    - crates/xcfun-eval/src/lib.rs
    - crates/xcfun-gpu/Cargo.toml
    - crates/xcfun-gpu/src/lib.rs
    - crates/xcfun-ffi/Cargo.toml
    - crates/xcfun-ffi/src/lib.rs
    - crates/xcfun-python/Cargo.toml
    - crates/xcfun-python/src/lib.rs
  modified: []

key-decisions:
  - "Used C++ xcfun.h VarType ordering (A_AX_AY_AZ=19), not design doc ordering"
  - "C_SLATER = 0.9305 from C++ pow(81/(32*pi),1/3), not design doc formula 0.7386"
  - "DensityVars::from_input returns Result for input length validation (T-01-03 threat mitigation)"

patterns-established:
  - "Regularization pattern: value_f64() to check, set_constant() to clamp, preserving derivatives"
  - "VarType match-based construction: each variant explicitly handles all field assignments"

requirements-completed: [CORE-01, CORE-02, CORE-03, CORE-04, CORE-05, CORE-06, CORE-07, CORE-08, AD-08]

duration: 8min
completed: 2026-04-17
---

# Phase 01 Plan 02: Core Types + AD Engine Summary

**xcfun-core crate with DensityVars<T> from_input for 31 VarType variants, 78 FunctionalId variants with metadata, Dependency bitflags, XcError, taylorlen, and 5 stub workspace crates**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-17T15:20:55Z
- **Completed:** 2026-04-17T15:29:05Z
- **Tasks:** 2
- **Files modified:** 19

## Accomplishments
- Complete xcfun-core type system matching C++ xcfun definitions exactly
- DensityVars construction from all 31 input variable types with regularization
- Full 7-crate workspace compiles (xcfun-ad, xcfun-core, xcfun-functionals, xcfun-eval, xcfun-gpu, xcfun-ffi, xcfun-python)
- 33 passing tests covering constants, enums, bitflags, error types, taylorlen, FunctionalId, DensityVars

## Task Commits

Each task was committed atomically:

1. **Task 1: Constants, error types, Dependency bitflags, EvalMode, VarType, taylorlen** - `eaed9b3` (feat)
2. **Task 2: FunctionalId 78 variants, DensityVars from_input, stub crates** - `4e91367` (feat)

## Files Created/Modified
- `crates/xcfun-core/Cargo.toml` - Crate manifest with xcfun-ad, thiserror, bitflags deps
- `crates/xcfun-core/src/lib.rs` - Module declarations, re-exports, taylorlen()
- `crates/xcfun-core/src/constants.rs` - C_SLATER, CF, TINY_DENSITY, MAX_ORDER, RS_PREFACTOR
- `crates/xcfun-core/src/error.rs` - XcError enum with 7 variants, ffi_code()
- `crates/xcfun-core/src/enums.rs` - EvalMode (3 variants), VarType (31 variants, C++ ordering)
- `crates/xcfun-core/src/traits.rs` - Dependency bitflags, TestData, Functional trait
- `crates/xcfun-core/src/functional_id.rs` - FunctionalId (78 variants) with from_name, name, description, depends
- `crates/xcfun-core/src/density_vars.rs` - DensityVars<T> with from_input for all VarType variants
- `crates/xcfun-core/src/test_data.rs` - Placeholder for Phase 2 test data
- `crates/xcfun-functionals/{Cargo.toml,src/lib.rs}` - Stub crate
- `crates/xcfun-eval/{Cargo.toml,src/lib.rs}` - Stub crate
- `crates/xcfun-gpu/{Cargo.toml,src/lib.rs}` - Stub crate
- `crates/xcfun-ffi/{Cargo.toml,src/lib.rs}` - Stub crate
- `crates/xcfun-python/{Cargo.toml,src/lib.rs}` - Stub crate

## Decisions Made
- Used C++ xcfun.h VarType ordering (A_AX_AY_AZ=19, not design doc value of 8) to match C++ binary compatibility
- C_SLATER = 0.9305257363491002 from C++ formula pow(81/(32*pi),1/3), not design doc formula (3/2)*(3/(4*pi))^(1/3) = 0.7386
- DensityVars::from_input returns Result<Self, XcError> for input length validation, mitigating T-01-03 (DoS via array length)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- xcfun-core type system complete, ready for functional implementations (Phase 2)
- All downstream crate stubs in place for workspace structure
- Num trait dependency from xcfun-ad fully integrated

## Self-Check: PASSED

All 13 key files found. Both task commits (eaed9b3, 4e91367) verified in git log. SUMMARY.md created.

---
*Phase: 01-core-types-ad-engine*
*Completed: 2026-04-17*
