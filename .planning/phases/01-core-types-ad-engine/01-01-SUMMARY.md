---
phase: 01-core-types-ad-engine
plan: 01
subsystem: ad-engine
tags: [ctaylor, automatic-differentiation, recursive-multiplication, compose, num-trait, const-generics]

# Dependency graph
requires: []
provides:
  - "CTaylor<f64, N> struct with stack-allocated coefficient arrays and all arithmetic operators"
  - "Recursive multiplication algorithms (mul_recursive, mul_set_recursive, multo_skipconst)"
  - "compose() Horner evaluation for Taylor coefficient composition"
  - "Num trait with full method set including value_f64() and set_constant()"
  - "Num implementation for f64 (complete including erf, sqrtx_asinh_sqrtx)"
  - "Num implementation for CTaylor (arithmetic + stubs for transcendentals)"
  - "Cargo workspace structure with crates/* member pattern"
affects: [01-02, 01-03, xcfun-core, xcfun-functionals]

# Tech tracking
tech-stack:
  added: [approx (dev-dependency)]
  patterns: [const-generic-size-parameter, recursive-slice-splitting, borrow-checker-temporary-copy]

key-files:
  created:
    - crates/xcfun-ad/Cargo.toml
    - crates/xcfun-ad/src/lib.rs
    - crates/xcfun-ad/src/ctaylor.rs
    - crates/xcfun-ad/src/compose.rs
    - crates/xcfun-ad/src/num.rs
  modified:
    - Cargo.toml

key-decisions:
  - "N parameter represents array SIZE (power of 2) not variable count, due to stable Rust const generic limitation"
  - "Used Vec<f64> temporary in multo_skipconst to satisfy borrow checker for simultaneous read/write of slice halves"
  - "CTaylor transcendental methods are panic stubs pending Plan 03 implementation"

patterns-established:
  - "Const generic SIZE pattern: CTaylor<f64, N> where N = 1<<nvar (e.g., N=4 for 2 variables)"
  - "Recursive slice splitting: all compose algorithms operate on &[f64] slices halved at each recursion level"
  - "TDD with comprehensive tests for each algorithm: constructors, arithmetic, compose, Num trait"

requirements-completed: [AD-01, AD-02, AD-06, AD-07]

# Metrics
duration: 7min
completed: 2026-04-17
---

# Phase 1 Plan 1: AD Engine Core Summary

**CTaylor<f64, N> with recursive O(3^N) multiplication, Horner compose, and Num trait for f64/CTaylor dual evaluation**

## Performance

- **Duration:** 7 min
- **Started:** 2026-04-17T15:10:16Z
- **Completed:** 2026-04-17T15:17:52Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- CTaylor struct with const-generic N for stack-allocated coefficient arrays (N=1..256)
- Complete port of C++ ctaylor_rec recursive multiplication algorithms (mul, mul_set, multo_skipconst, compose)
- Division via compose with inverse Taylor coefficients (recurrence: t[k] = -t[k-1]/q0)
- Full Num trait with 20+ methods including value_f64() and set_constant() for DensityVars regularization
- f64 Num implementation complete including erf (Abramowitz & Stegun) and sqrtx_asinh_sqrtx
- 54 tests passing covering all algorithms and trait implementations

## Task Commits

Each task was committed atomically:

1. **Task 1: Workspace setup + CTaylor struct + recursive multiplication + compose** - `210e442` (feat)
2. **Task 2: Num trait definition and CTaylor Num implementation** - `1708e83` (feat)

## Files Created/Modified
- `Cargo.toml` - Workspace root with crates/* members
- `crates/xcfun-ad/Cargo.toml` - AD crate with zero runtime dependencies
- `crates/xcfun-ad/src/lib.rs` - Crate re-exports (CTaylor, Num, VAR constants)
- `crates/xcfun-ad/src/ctaylor.rs` - CTaylor struct, constructors, all arithmetic and scalar ops
- `crates/xcfun-ad/src/compose.rs` - Recursive multiply, multo_skipconst, compose algorithms
- `crates/xcfun-ad/src/num.rs` - Num trait definition, f64 impl (complete), CTaylor impl (arithmetic + stubs)
- `.gitignore` - Ignore target/ directory

## Decisions Made
- **N as SIZE not NVAR:** Stable Rust does not support `[T; 1 << N]` in const generic context (`generic_const_exprs` is nightly-only). Restructured N to represent the array size directly (powers of 2). Users write `CTaylor::<f64, 4>` for 2 variables instead of `CTaylor::<f64, 2>`. A `nvar_from_size()` const fn computes variable count from size.
- **Vec temporary in multo_skipconst:** The C++ code reads `dst[..half]` while writing `dst[half..]` simultaneously. Rust's borrow checker prevents this. Used `dst[..half].to_vec()` as a temporary copy (max 64 f64s = 512 bytes at N=7).
- **Transcendental stubs:** CTaylor Num methods for exp/log/sqrt/etc. are panic stubs. Plan 03 will implement them via tmath.rs compose-based Taylor expansion.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Changed CTaylor generic parameter semantics for stable Rust**
- **Found during:** Task 1 (CTaylor struct implementation)
- **Issue:** Plan specified `pub c: [T; 1 << N]` which requires `generic_const_exprs` (nightly-only feature). Stable Rust cannot use const generic parameters in const expressions for array sizes.
- **Fix:** Changed N to represent the array SIZE directly (must be power of 2) instead of variable count. Added `nvar_from_size()` const fn and `NVAR` associated const. All constructors and algorithms work identically; only the type parameter semantic changed.
- **Files modified:** crates/xcfun-ad/src/ctaylor.rs
- **Verification:** All 54 tests pass. cargo check clean.
- **Committed in:** 210e442 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary adaptation for stable Rust. API is functionally identical; downstream crates use `CTaylor::<f64, {1 << N}>` or concrete sizes. No scope reduction.

## Known Stubs

| File | Line | Stub | Resolution |
|------|------|------|------------|
| crates/xcfun-ad/src/num.rs | ~195-250 | CTaylor transcendental panic stubs (exp, log, sqrt, etc.) | Plan 03 (01-03-PLAN.md) implements via tmath.rs |

These stubs are intentional and documented in the plan. They do not prevent the plan's goal (AD engine core with arithmetic and compose).

## Issues Encountered
None beyond the const generic limitation documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- xcfun-ad crate compiles with zero runtime dependencies
- CTaylor arithmetic and compose ready for Plan 02 (xcfun-core types)
- Num trait defined for Plan 02 to use in DensityVars<T>
- Compose infrastructure ready for Plan 03 (transcendental functions)

## Self-Check: PASSED

All created files verified present. Both task commits verified in git log.

---
*Phase: 01-core-types-ad-engine*
*Completed: 2026-04-17*
