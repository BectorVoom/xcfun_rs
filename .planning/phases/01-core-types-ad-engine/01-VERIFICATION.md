---
phase: 01-core-types-ad-engine
verified: 2026-04-17T16:00:00Z
status: passed
score: 5/5
overrides_applied: 0
---

# Phase 1: Core Types + AD Engine Verification Report

**Phase Goal:** Developers can define density variables, enumerate functionals, and compute arbitrary-order derivatives of composed mathematical functions
**Verified:** 2026-04-17T16:00:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | DensityVars can be constructed from raw input arrays for all 30 VarType variants | VERIFIED | `density_vars.rs` (825 lines) has explicit match arms for all 31 VarType variants (30 usable + NrVars). from_input() returns `Result<Self, XcError>` with input length validation. Regularization uses `value_f64()` and `set_constant()` from Num trait. 33 tests pass in xcfun-core. |
| 2 | CTaylor produces correct derivatives for exp, log, pow, sqrt, sin, cos, erf at orders 0-6 | VERIFIED | `tmath.rs` (928 lines) contains all Taylor expansion generators. `math.rs` (688 lines) contains all 15 CTaylor transcendental methods using tmath+compose pattern. No panic stubs in `num.rs`. Tests verify derivatives against analytical values at 1e-12 tolerance. 118 tests pass in xcfun-ad. |
| 3 | Composed functions (chain rule) yield correct mixed partial derivatives verified against known analytical values | VERIFIED | `compose.rs` (314 lines) implements mul_recursive, mul_set_recursive, multo_skipconst, compose. All math functions pipe through compose. Tests in math.rs verify chain rule compositions (e.g., exp(sin(x)), exp(x^2)). |
| 4 | AD engine handles edge cases (near-zero density, extreme coefficients) without NaN or panic | VERIFIED | Five stability tests in math.rs: `stability_exp_large` (x=700), `stability_exp_700`, `stability_log_small` (x=1e-14), `stability_sqrt_small` (x=1e-300), `stability_pow_fractional_near_zero` (x=1e-10). All pass. |
| 5 | FunctionalId enum covers all 78 variants with name lookup and dependency metadata | VERIFIED | `functional_id.rs` (548 lines) has `pub const COUNT: usize = 78`, `from_name()` with case-insensitive lookup, `name()`, `description()`, `depends()` returning Dependency bitflags. Tests verify SlaterX=0, from_name case insensitivity, depends() output. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/xcfun-ad/src/ctaylor.rs` | CTaylor struct with constructors and arithmetic | VERIFIED | 501 lines, `pub struct CTaylor`, const-generic N, all arithmetic ops, scalar ops |
| `crates/xcfun-ad/src/num.rs` | Num trait and f64/CTaylor implementations | VERIFIED | 473 lines, `pub trait Num`, impl for f64 (complete), impl for CTaylor (complete, no stubs) |
| `crates/xcfun-ad/src/compose.rs` | compose(), mul_recursive, mul_set_recursive, multo_skipconst | VERIFIED | 314 lines, all four functions present |
| `crates/xcfun-ad/src/tmath.rs` | Taylor expansion generators | VERIFIED | 928 lines, 14 expansion generators including exp, log, pow, sqrt, sin, cos, erf, asinh, sqrtx_asinh_sqrtx |
| `crates/xcfun-ad/src/math.rs` | CTaylor transcendental implementations | VERIFIED | 688 lines, 15 ctaylor_* functions using tmath+compose pattern |
| `crates/xcfun-ad/src/lib.rs` | Crate re-exports | VERIFIED | Exports CTaylor, Num, VAR constants, modules |
| `crates/xcfun-core/src/density_vars.rs` | DensityVars struct with from_input | VERIFIED | 825 lines, handles all 31 VarType variants with regularization |
| `crates/xcfun-core/src/enums.rs` | EvalMode and VarType enums | VERIFIED | 322 lines, VarType with C++ ordering (A_AX_AY_AZ=19), input_len(), provides(), is_spin_polarized() |
| `crates/xcfun-core/src/functional_id.rs` | FunctionalId with 78 variants | VERIFIED | 548 lines, COUNT=78, from_name, depends, name, description |
| `crates/xcfun-core/src/traits.rs` | Functional trait, Dependency bitflags | VERIFIED | 75 lines, `pub trait Functional: Send + Sync`, Dependency bitflags |
| `crates/xcfun-core/src/error.rs` | XcError enum | VERIFIED | 88 lines, `pub enum XcError` with thiserror |
| `crates/xcfun-core/src/constants.rs` | Physical constants | VERIFIED | 66 lines, C_SLATER, CF, TINY_DENSITY, MAX_ORDER |
| `crates/xcfun-core/src/lib.rs` | Re-exports and taylorlen | VERIFIED | 59 lines, taylorlen const fn, all module re-exports |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| math.rs | tmath.rs | math calls tmath expansion generators | WIRED | `exp_expand`, `log_expand`, etc. called from all ctaylor_* functions |
| math.rs | compose.rs | All transcendental functions use compose() | WIRED | 13 `compose::compose()` calls found in math.rs |
| num.rs | math.rs | CTaylor Num::exp delegates to ctaylor_exp | WIRED | `math::ctaylor_exp`, `math::ctaylor_log`, `math::ctaylor_sqrt` etc. confirmed in num.rs |
| density_vars.rs | num.rs | T: Num bound, uses value_f64() and set_constant() | WIRED | Regularization at line 17-18 uses `x.value_f64()` and `x.set_constant()` |
| traits.rs | density_vars.rs | Functional::energy takes DensityVars<T> | WIRED | `fn energy<T: Num>(&self, vars: &DensityVars<T>) -> T` in trait definition |
| ctaylor.rs | compose.rs | CTaylor::div uses compose for reciprocal | WIRED | Division delegates through compose pattern |
| functional_id.rs | traits.rs | depends() returns Dependency | WIRED | `pub fn depends(&self) -> Dependency` at line 387 |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All workspace tests pass | `cargo test --workspace` | 151 passed, 0 failed | PASS |
| Workspace compiles cleanly | `cargo check --workspace` | Finished dev (warnings only) | PASS |
| No panic stubs in num.rs | `grep panic! num.rs` | No matches | PASS |
| Stability tests exist and pass | `cargo test -p xcfun-ad stability` | 5 stability tests pass | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CORE-01 | 01-02 | DensityVars<T> struct with 25 fields and from_input() for all 30 VarType variants | SATISFIED | density_vars.rs: 825 lines, all variants handled |
| CORE-02 | 01-02 | EvalMode enum with mode validation | SATISFIED | enums.rs: 3 variants (PartialDerivatives, Potential, Contracted) |
| CORE-03 | 01-02 | VarType enum (30 variants) with metadata | SATISFIED | enums.rs: 31 variants with input_len(), provides(), is_spin_polarized() |
| CORE-04 | 01-02 | FunctionalId enum (78 variants) with metadata | SATISFIED | functional_id.rs: COUNT=78, from_name, depends, name, description |
| CORE-05 | 01-02 | Dependency bitflags | SATISFIED | traits.rs: DENSITY, GRADIENT, LAPLACIAN, KINETIC, JP |
| CORE-06 | 01-02 | XcError enum with thiserror | SATISFIED | error.rs: 7 variants with thiserror derive |
| CORE-07 | 01-02 | Physical constants module | SATISFIED | constants.rs: C_SLATER, CF, TINY_DENSITY, MAX_ORDER |
| CORE-08 | 01-02 | Functional trait definition | SATISFIED | traits.rs: `pub trait Functional: Send + Sync` with energy, depends, id, description, test_data |
| AD-01 | 01-01 | CTaylor<T, N> struct with const generic N | SATISFIED | ctaylor.rs: `pub struct CTaylor<T, const N: usize>` (N=SIZE, power of 2) |
| AD-02 | 01-01 | All arithmetic operators with recursive multiplication | SATISFIED | ctaylor.rs: Add, Sub, Mul, Div, Neg, scalar ops. compose.rs: mul_recursive, mul_set_recursive |
| AD-03 | 01-03 | Transcendental functions (exp, log, pow, sqrt, cbrt, abs) | SATISFIED | math.rs: ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_sqrt, ctaylor_cbrt, ctaylor_abs |
| AD-04 | 01-03 | Trigonometric functions (sin, cos, atan, asin, acos) | SATISFIED | math.rs: ctaylor_sin, ctaylor_cos, ctaylor_atan, ctaylor_asin, ctaylor_acos |
| AD-05 | 01-03 | Special functions (asinh, erf, sqrtx_asinh_sqrtx) | SATISFIED | math.rs: ctaylor_asinh, ctaylor_erf, ctaylor_sqrtx_asinh_sqrtx |
| AD-06 | 01-01 | Num trait with f64 and CTaylor implementations | SATISFIED | num.rs: `pub trait Num`, impl for f64 (complete), impl for CTaylor (complete) |
| AD-07 | 01-01 | Taylor composition (chain rule) | SATISFIED | compose.rs: `pub fn compose` with Horner evaluation via multo_skipconst |
| AD-08 | 01-02 | taylorlen() function | SATISFIED | lib.rs: `pub const fn taylorlen` with iterative binomial computation |
| AD-09 | 01-03 | Numerical stability near zero, infinity, extreme coefficients | SATISFIED | 5 stability tests in math.rs covering exp(700), log(1e-14), sqrt(1e-300), pow(1e-10, 4/3) |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| ctaylor.rs | 41 | panic! for invalid CTaylor size | Info | Valid compile-time guard, not a stub |
| math.rs | 24 | panic! for invalid CTaylor size | Info | Valid compile-time guard, not a stub |
| tmath.rs | 529 | Dead code warning: taylor1d_mul | Info | Unused helper function, no functional impact |

### Human Verification Required

None. All phase deliverables are programmatically verifiable through tests and code inspection.

### Gaps Summary

No gaps found. All 5 roadmap success criteria are verified. All 17 requirement IDs (CORE-01 through CORE-08, AD-01 through AD-09) are satisfied with implementation evidence. The workspace compiles cleanly and all 151 tests pass.

---

_Verified: 2026-04-17T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
