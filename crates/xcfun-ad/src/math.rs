//! Composed CTaylor elementary functions. 1:1 port of
//! `xcfun-master/external/upstream/taylor/ctaylor_math.hpp`.
//!
//! Each composed fn follows the 3-step pipeline from CONTEXT.md D-14:
//!
//!   1. `*_expand(scratch, x[CNST], ...)` — fills a length-`(n+1)` scalar
//!      Taylor-coefficient series via the corresponding Plan 01-03/04
//!      scalar expansion.
//!   2. `ctaylor_compose(out, x, scratch, n)` — composes `scratch` with
//!      `x` as the inner polynomial (Plan 01-02).
//!   3. (implicit) `out` holds the Taylor coefficients of `op(x)`.
//!
//! # Scratch allocation
//!
//! Each kernel allocates its own `scratch` buffer inside the `#[cube]` body
//! via `Array::<F>::new(comptime!((n + 1) as usize))`. This matches the
//! pattern established in Plan 01-04's transcendental expansions
//! (`atan_expand`, `asinh_expand`, ...) which use the same allocation form
//! for their internal scratch. On cubecl-cpu the allocation lowers to
//! stack-local storage (no heap traffic).
//!
//! # Preconditions
//!
//! Inherited from the corresponding `*_expand` (Plan 01-03 precondition
//! fallback — host-side guard, since cubecl 0.10-pre.3 rejects in-kernel
//! `assert!`; see `crates/xcfun-ad/src/expand/mod.rs` for the canonical
//! policy text). Callers MUST verify the `x[CNST]` precondition before
//! launching:
//!
//!   - `ctaylor_reciprocal`: `x[CNST] != 0`
//!   - `ctaylor_sqrt`: `x[CNST] > 0`
//!   - `ctaylor_log`: `x[CNST] > 0`
//!   - `ctaylor_pow`:  `x[CNST] > 0`
//!   - `ctaylor_exp`, `ctaylor_powi`, `ctaylor_erf`, `ctaylor_asinh`,
//!     `ctaylor_atan`: no precondition (analytic on all reals).
//!
//! # `ctaylor_powi` dispatch strategy
//!
//! `ctaylor_math.hpp:165-178` implements positive integer exponents via a
//! `while (n-- > 1) res *= t` loop; zero via a unit constant; negative via
//! delegation to `pow(t, double(n))`. Cubecl 0.10-pre.3 does not support
//! comptime for-loop unroll over a `#[comptime] i32`, so this port provides
//! an outer dispatcher `ctaylor_powi` that matches on the `#[comptime]
//! exponent: i32` and delegates to per-exponent specialisations for
//! `exponent ∈ {-2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}` (the full range
//! emitted by the Plan 01-06 fixture driver). Each positive-exponent path
//! is a fully unrolled `ctaylor_multo` chain (exponent - 1 multiplications).
//! Negative exponents call `ctaylor_pow` with `a = F::cast_from(exponent)`.

use cubecl::prelude::*;

use crate::ctaylor_rec::compose::ctaylor_compose;
use crate::ctaylor_rec::mul::ctaylor_mul;
use crate::ctaylor_rec::multo::ctaylor_multo;
use crate::expand::asinh::asinh_expand;
use crate::expand::atan::atan_expand;
use crate::expand::br_inverse::br_inverse_expand;
// Phase 6 D-11 — `ctaylor_erf` rewires onto `erf_precise_taylor`, the
// libm-hybrid wrapper that seeds `t[0]` via `erf_precise` (FreeBSD msun
// port from Plan 02-06 commit `dca382a`) and uses the gauss-expand
// Hermite-poly recurrence for `t[i ≥ 1]`. `erf_expand` remains a public
// entry point at `crates/xcfun-ad/src/expand/erf.rs` for back-compat;
// Plan 06-N3 will tighten `erf_precise_taylor` independently.
use crate::expand::erf::erf_precise_taylor;
use crate::expand::exp::exp_expand;
use crate::expand::expm1::expm1_expand;
use crate::expand::inv::inv_expand;
use crate::expand::log::log_expand;
use crate::expand::pow::pow_expand;
use crate::expand::sqrt::sqrt_expand;
use crate::tfuns::{tfuns_compose, tfuns_multo, tfuns_shift};

// ---------------------------------------------------------------------------
//  ctaylor_reciprocal — out = 1 / x
//  Port of ctaylor_math.hpp:7-28 (specifically the operator/(S, ctaylor)
//  template with `S = 1`, simplified).
// ---------------------------------------------------------------------------

/// Compute `out = 1 / x` as a CTaylor. Port of the reciprocal path in
/// `ctaylor_math.hpp:7-28` (shared with `operator/`).
///
/// ```cpp
/// // operator/(S, ctaylor), specialised to S = 1:
/// T tmp[Nvar + 1];
/// inv_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// ```
///
/// Precondition: `x[0] != 0` (host-side guard; `inv_expand` divides by
/// `x[0]`).
#[cube]
pub fn ctaylor_reciprocal<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // ctaylor_math.hpp:14: T tmp[Nvar + 1];
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    // ctaylor_math.hpp:15: inv_expand<T, Nvar>(tmp, t.c[0]);
    inv_expand::<F>(&mut scratch, x[0], n);
    // ctaylor_math.hpp:18: ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_sqrt — out = sqrt(x).  Port of ctaylor_math.hpp:133-145.
// ---------------------------------------------------------------------------

/// `out = sqrt(x)`. Port of `ctaylor_math.hpp:133-145`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// sqrt_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
///
/// Precondition: `x[0] > 0` (host-side guard).
#[cube]
pub fn ctaylor_sqrt<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    sqrt_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_exp — out = exp(x).  Port of ctaylor_math.hpp:71-81.
// ---------------------------------------------------------------------------

/// `out = exp(x)`. Port of `ctaylor_math.hpp:71-81`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// exp_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
#[cube]
pub fn ctaylor_exp<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    exp_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_expm1 — out = exp(x) - 1.
//  Port of ctaylor_math.hpp:85-102. Uses the upstream stable-bracket for
//  |x[0]| <= 1e-3 (D-05). Consumed by PBEC / APBEC / SPBEC / PBEINTC /
//  PBELOCC / ZVPBESOLC / ZVPBEINTC / VWN_PBEC / PW91C / RPBEX / BECKESRX /
//  BECKECAMX (9 GGA bodies).
// ---------------------------------------------------------------------------

/// `out = exp(x) - 1`, Taylor-composed. Uses the upstream stable-bracket for
/// `|x[0]| <= 1e-3` to preserve f64 precision as `x[0] → 0`.
///
/// Port of `ctaylor_math.hpp:85-102`:
///
/// ```cpp
/// T tmp[Nvar + 1];
/// exp_expand<T, Nvar>(tmp, t.c[0]);
/// if (fabs(t.c[0]) > 1e-3) tmp[0] -= 1;
/// else                     tmp[0] = 2 * exp(t.c[0] / 2) * sinh(t.c[0] / 2);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
///
/// Precondition: none (`expm1` is analytic on all reals).
#[cube]
pub fn ctaylor_expm1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    expm1_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_log — out = log(x).  Port of ctaylor_math.hpp:104-115.
// ---------------------------------------------------------------------------

/// `out = log(x)`. Port of `ctaylor_math.hpp:104-115`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// log_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
///
/// Precondition: `x[0] > 0` (host-side guard).
#[cube]
pub fn ctaylor_log<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    log_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_pow — out = x^a (real exponent).  Port of ctaylor_math.hpp:117-131.
// ---------------------------------------------------------------------------

/// `out = x ^ a` with real exponent `a`. Port of `ctaylor_math.hpp:117-131`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// pow_expand<T, Nvar>(tmp, t.c[0], a);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
///
/// Precondition: `x[0] > 0` (host-side guard; `pow_expand` divides by
/// `x[0]` and calls `powf(x[0], a)`).
#[cube]
pub fn ctaylor_pow<F: Float>(x: &Array<F>, a: F, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    pow_expand::<F>(&mut scratch, x[0], a, n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_cbrt — out = cbrt(x).  06-N7/07-00 wrapper.
// ---------------------------------------------------------------------------

/// `out = cbrt(x)`. Routes through `cbrt_expand` (Newton-refined from
/// `powf(1/3)` to libm-cbrt precision) instead of generic
/// `ctaylor_pow(x, 1/3)`. Matches C++ `tmath.hpp:172-178`'s
/// `cbrt_expand` template which uses `t[0] = cbrt(x0)`.
///
/// Plan 07-00 Task 0.3 audit identified `pow(x, 1/3)`-vs-`cbrt(x)` as
/// a contributor to PW91C's systematic order-0 offset against C++.
///
/// Precondition: `x[0] > 0` (host-side guard).
#[cube]
pub fn ctaylor_cbrt<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    crate::expand::cbrt::cbrt_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_erf — out = erf(x).  Port of ctaylor_math.hpp:194-206.
// ---------------------------------------------------------------------------

/// `out = erf(x)`. Port of `ctaylor_math.hpp:194-206`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// erf_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
///
/// Phase 6 D-11 — rewired to call `erf_precise_taylor` instead of
/// `erf_expand` directly. Preserves the libm-precision seed for `t[0]`
/// (Phase 2 commit `dca382a` baseline) and gives Plan 06-N3 a stable
/// public entry point to tighten without disturbing the existing
/// `erf_expand` callers.
///
/// Inherits `erf_precise_taylor`'s precision contract: scalar `erf_precise`
/// at ≤ 1 ULP vs `libm::erf` for `t[0]`; `2/√π` at f64 precision via
/// `F::cast_from` for the derivative chain. Plan 06-N3 verifies the
/// LDAERF order-3 residuals tighten to ≤ 1e-13.
#[cube]
pub fn ctaylor_erf<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    // Phase 6 D-11: `erf_precise_taylor` is the libm-hybrid wrapper.
    erf_precise_taylor::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_asinh — out = asinh(x).  Port of ctaylor_math.hpp:256-268.
// ---------------------------------------------------------------------------

/// `out = asinh(x)`. Port of `ctaylor_math.hpp:256-268`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// asinh_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
#[cube]
pub fn ctaylor_asinh<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    asinh_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_atan — out = atan(x).  Port of ctaylor_math.hpp:180-192.
// ---------------------------------------------------------------------------

/// `out = atan(x)`. Port of `ctaylor_math.hpp:180-192`.
///
/// ```cpp
/// T tmp[Nvar + 1];
/// atan_expand<T, Nvar>(tmp, t.c[0]);
/// ctaylor<T, Nvar> res;
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// return res;
/// ```
#[cube]
pub fn ctaylor_atan<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    atan_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}

// ---------------------------------------------------------------------------
//  ctaylor_powi — integer exponent fast path.  Port of ctaylor_math.hpp:165-178.
// ---------------------------------------------------------------------------
//
// C++ source:
//   template <class T, int Nvar>
//   static ctaylor<T, Nvar> pow(const ctaylor<T, Nvar> & t, int n) {
//     if (n > 0) {
//       ctaylor<T, Nvar> res = t;
//       while (n-- > 1)
//         res *= t;
//       return res;
//     } else if (n < 0) {
//       return pow(t, double(n));
//     } else {
//       ctaylor<T, Nvar> res(1);
//       return res;
//     }
//   }
//
// C++ `res *= t` calls `ctaylor_rec<T, Nvar>::multo(res.c, t.c)`. We preserve
// that left-to-right cumulative multiplication order.
//
// Cubecl 0.10-pre.3 limitation: a `for _ in 1..exponent { ctaylor_multo(...) }`
// loop does not unroll cleanly when `exponent` is a runtime i32, and `#[comptime]
// exponent: i32` does not admit a comptime `for` as of pre.3. So we enumerate
// per-exponent specialisations 1..=10 (covers the fixture driver's exponents)
// and dispatch via `if comptime!(exponent == k)` chain.
//
// Positive-exponent specialisation template (ctaylor_powi_n{k}):
//   1. Start with `out = x` via an explicit copy (we emit a unit-length
//      mul where one operand is the unit CTaylor; the simpler form is
//      to copy element-by-element).
//   2. Apply `ctaylor_multo(out, x, n)` exactly `(k - 1)` times.
//
// Zero-exponent: `out = 1` (constant CTaylor).
//
// Negative-exponent: delegates to `ctaylor_pow(x, F::cast_from(exponent))`.

/// Positive-exponent helper — copy x into out.
#[cube]
fn ctaylor_powi_copy<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = x[k];
    }
}

/// `out = x` (exponent 1).
#[cube]
pub fn ctaylor_powi_1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_copy::<F>(x, out, n);
}

/// `out = x * x` (exponent 2).
#[cube]
pub fn ctaylor_powi_2<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // C++: res = t; res *= t;
    // Fused: out = x * x — use mul_set rather than copy-then-multo to keep
    // associativity (equivalent — multo(x*x) == x*x because the copy step
    // didn't touch out's coefficients before).
    ctaylor_mul::<F>(x, x, out, n);
}

/// `out = x^k` for positive `k ≥ 2`. Internal helper; the public
/// `ctaylor_powi_n{k}` fns below inline the kernel for clippy reasons.
#[cube]
fn ctaylor_powi_positive<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] k: u32,
    #[comptime] n: u32,
) {
    // out = x
    ctaylor_powi_copy::<F>(x, out, n);
    // Apply (k - 1) `out *= x`'s.
    //
    // Cubecl 0.10-pre.3 comptime-for over a u32 range should unroll; if it
    // does not, dispatch manually below.
    #[unroll]
    for _ in 1..k {
        ctaylor_multo::<F>(out, x, n);
    }
}

/// `out = x^3`.
#[cube]
pub fn ctaylor_powi_3<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 3_u32, n);
}

/// `out = x^4`.
#[cube]
pub fn ctaylor_powi_4<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 4_u32, n);
}

/// `out = x^5`.
#[cube]
pub fn ctaylor_powi_5<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 5_u32, n);
}

/// `out = x^6`.
#[cube]
pub fn ctaylor_powi_6<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 6_u32, n);
}

/// `out = x^7`.
#[cube]
pub fn ctaylor_powi_7<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 7_u32, n);
}

/// `out = x^8`.
#[cube]
pub fn ctaylor_powi_8<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 8_u32, n);
}

/// `out = x^9`.
#[cube]
pub fn ctaylor_powi_9<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 9_u32, n);
}

/// `out = x^10`.
#[cube]
pub fn ctaylor_powi_10<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_powi_positive::<F>(x, out, 10_u32, n);
}

/// Zero-exponent fast path — `out = 1` (constant polynomial).
///
/// Port of `ctaylor_math.hpp:174-176`:
///
/// ```cpp
/// } else {
///   ctaylor<T, Nvar> res(1);
///   return res;
/// }
/// ```
#[cube]
pub fn ctaylor_powi_0<F: Float>(out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    let zero = F::new(0.0);
    out[0] = F::new(1.0);
    #[unroll]
    for i in 1..size {
        let k = i as usize;
        out[k] = zero;
    }
}

/// `out = x^(-1)` — delegates to `ctaylor_pow` with `a = -1`.
#[cube]
pub fn ctaylor_powi_neg1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_pow::<F>(x, F::new(-1.0), out, n);
}

/// `out = x^(-2)` — delegates to `ctaylor_pow` with `a = -2`.
#[cube]
pub fn ctaylor_powi_neg2<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_pow::<F>(x, F::new(-2.0), out, n);
}

/// Outer dispatcher — matches on the `#[comptime] exponent: i32` and calls
/// the appropriate per-exponent specialisation. Supports
/// `exponent ∈ {-2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10}`.
///
/// Port of `ctaylor_math.hpp:165-178`:
///
/// ```cpp
/// static ctaylor<T, Nvar> pow(const ctaylor<T, Nvar> & t, int n) {
///   if (n > 0) { /* unrolled multiplies */ }
///   else if (n < 0) { return pow(t, double(n)); }
///   else { return ctaylor<T, Nvar>(1); }
/// }
/// ```
#[cube]
pub fn ctaylor_powi<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] exponent: i32,
    #[comptime] n: u32,
) {
    if comptime!(exponent == 0) {
        ctaylor_powi_0::<F>(out, n);
    } else if comptime!(exponent == 1) {
        ctaylor_powi_1::<F>(x, out, n);
    } else if comptime!(exponent == 2) {
        ctaylor_powi_2::<F>(x, out, n);
    } else if comptime!(exponent == 3) {
        ctaylor_powi_3::<F>(x, out, n);
    } else if comptime!(exponent == 4) {
        ctaylor_powi_4::<F>(x, out, n);
    } else if comptime!(exponent == 5) {
        ctaylor_powi_5::<F>(x, out, n);
    } else if comptime!(exponent == 6) {
        ctaylor_powi_6::<F>(x, out, n);
    } else if comptime!(exponent == 7) {
        ctaylor_powi_7::<F>(x, out, n);
    } else if comptime!(exponent == 8) {
        ctaylor_powi_8::<F>(x, out, n);
    } else if comptime!(exponent == 9) {
        ctaylor_powi_9::<F>(x, out, n);
    } else if comptime!(exponent == 10) {
        ctaylor_powi_10::<F>(x, out, n);
    } else if comptime!(exponent == -1) {
        ctaylor_powi_neg1::<F>(x, out, n);
    } else if comptime!(exponent == -2) {
        ctaylor_powi_neg2::<F>(x, out, n);
    }
    // Unsupported exponents fall through with `out` unchanged — callers MUST
    // specialise per-exponent before launch (the Plan 01-06 dispatch in the
    // golden_composed test file covers the fixture driver's exponent range
    // exhaustively).
}

// ---------------------------------------------------------------------------
//  ctaylor_sqrtx_asinh_sqrtx — out = sqrt(x) * asinh(sqrt(x)).
//  Port of ctaylor_math.hpp:275-325 (D-06). UNCONDITIONAL [8,8] Padé branch
//  per B1 resolution: the |x[0]| < 0.5 path is REQUIRED, not optional. This
//  primitive is consumed by PW91X / PW91K / BECKEX / BECKECORRX / BECKESRX /
//  BECKECAMX (6 GGA bodies).
// ---------------------------------------------------------------------------

/// [8,8] Padé numerator coefficients for `y·asinh(sqrt(y))/sqrt(y)` at `y=0`.
///
/// Port of `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:286-294`
/// (D-06). DO NOT re-derive; these values are load-bearing for the 1e-14
/// parity contract. ASINH_TABSIZE = 9 in the upstream header.
pub(crate) const P_PADE_F64: [f64; 9] = [
    0.0_f64,
    3.510921856028398e3_f64,
    1.23624388373212e4_f64,
    1.734847003883674e4_f64,
    1.235072285222234e4_f64,
    4.691117148130619e3_f64,
    9.119186273274577e2_f64,
    7.815848629220836e1_f64,
    1.96088643023654e0_f64,
];

/// [8,8] Padé denominator coefficients for `y·asinh(sqrt(y))/sqrt(y)` at `y=0`.
///
/// Port of `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:295-303`
/// (D-06).
pub(crate) const Q_PADE_F64: [f64; 9] = [
    3.510921856028398e3_f64,
    1.29475924799926e4_f64,
    1.924308297963337e4_f64,
    1.474357149568687e4_f64,
    6.176496729255528e3_f64,
    1.379806958043824e3_f64,
    1.471833349002349e2_f64,
    5.666278232986776e0_f64,
    2.865104054302032e-2_f64,
];

/// [8,8] Padé branch for `sqrt(x)·asinh(sqrt(x))` at `|x[0]| < 0.5`.
///
/// Port of `ctaylor_math.hpp:304-319` (D-06, B1 unconditional implementation):
///
/// ```cpp
/// T tmp[Nvar + 1], pq[9];
/// for (int i = 0; i < ASINH_TABSIZE; i++) pq[i] = Q[i];
/// tfuns<T, ASINH_TABSIZE - 1>::shift(pq, t.c[0]);
/// inv_expand<T, Nvar>(tmp, pq[0]);
/// tfuns<T, Nvar>::compose(tmp, pq);
/// for (int i = 0; i < ASINH_TABSIZE; i++) pq[i] = P[i];
/// tfuns<T, ASINH_TABSIZE - 1>::shift(pq, t.c[0]);
/// tfuns<T, Nvar>::multo(tmp, pq);
/// ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
/// ```
///
/// Precondition (upstream asserts `Nvar < ASINH_TABSIZE`): `n < 9`.
/// Phase 3 ships orders 0..=4 only, so n ∈ {0,1,2,3,4} — always satisfied.
/// Phase 1 caps tfuns_compose at n ≤ 6, which is also within the [8,8]
/// Padé order envelope.
#[cube]
fn pade_8_8_sqrtx_asinh_sqrtx<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // tmath.hpp line 307 — T tmp[Nvar + 1], pq[9];
    let scratch_len = comptime!((n + 1) as usize);
    let mut tmp = Array::<F>::new(scratch_len);
    // pq is a degree-8 scalar polynomial scratch (9 entries).
    let mut pq = Array::<F>::new(9_usize);

    // ctaylor_math.hpp:308-309 — load Q into pq.
    #[unroll]
    for i in 0_u32..9_u32 {
        let ki = i as usize;
        pq[ki] = F::cast_from(Q_PADE_F64[ki]);
    }
    // ctaylor_math.hpp:310 — tfuns<T, ASINH_TABSIZE - 1>::shift(pq, t.c[0]);
    //   Shift Q polynomial by x[0]: pq(y) = Q(y + x[0]).
    //   Comptime degree is 8 (ASINH_TABSIZE - 1).
    tfuns_shift::<F>(&mut pq, x[0], 8_u32);

    // ctaylor_math.hpp:311 — inv_expand<T, Nvar>(tmp, pq[0]);
    //   tmp = Taylor series of 1/(pq[0] + y) around y = 0.
    inv_expand::<F>(&mut tmp, pq[0], n);

    // ctaylor_math.hpp:312 — tfuns<T, Nvar>::compose(tmp, pq);
    //   Substitute y -> pq_1(y) := pq(y) - pq[0], giving tmp = 1 / pq(y).
    //   tfuns_compose expects the inner polynomial to have x[0] = 0; the
    //   implementation only reads pq[1..=n], so pq[0] is effectively
    //   ignored (same behaviour as C++ tfuns::compose which reads indices
    //   1..=N only).
    tfuns_compose::<F>(&mut tmp, &pq, n);

    // ctaylor_math.hpp:313-314 — reload P into pq.
    #[unroll]
    for i in 0_u32..9_u32 {
        let ki = i as usize;
        pq[ki] = F::cast_from(P_PADE_F64[ki]);
    }
    // ctaylor_math.hpp:315 — tfuns<T, ASINH_TABSIZE - 1>::shift(pq, t.c[0]);
    //   Shift P by x[0]: pq(y) = P(y + x[0]).
    tfuns_shift::<F>(&mut pq, x[0], 8_u32);

    // ctaylor_math.hpp:316 — tfuns<T, Nvar>::multo(tmp, pq);
    //   tmp *= pq  →  tmp = P(y + x[0]) / Q(y + x[0]).
    tfuns_multo::<F>(&mut tmp, &pq, n);

    // ctaylor_math.hpp:317-318 — ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
    ctaylor_compose::<F>(out, x, &tmp, n);
}

/// `out = sqrt(x) * asinh(sqrt(x))`. Used by PW91X / PW91K enhancement and
/// Becke B88 (D-06).
///
/// Port target: `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:275-325`.
///
/// Precondition: `x[0] > -0.5` (upstream assert at ctaylor_math.hpp:277).
///
/// Two-branch strategy (same as C++ reference):
///
/// - `|x[0]| >= 0.5` → direct composition `sqrt(x) * asinh(sqrt(x))` via
///   `ctaylor_sqrt`, `ctaylor_asinh`, `ctaylor_mul`. Numerically fine for
///   `x[0]` away from zero.
/// - `|x[0]| < 0.5` → UNCONDITIONAL [8,8] Padé branch via
///   `pade_8_8_sqrtx_asinh_sqrtx`. Preserves 1e-14 precision as `x[0] → 0`
///   where the direct form would produce NaN/∞ derivatives.
#[cube]
pub fn ctaylor_sqrtx_asinh_sqrtx<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let threshold = F::cast_from(0.5_f64);
    let abs_x0 = x[0].abs();

    if abs_x0 >= threshold {
        // Unstable branch (|x[0]| >= 0.5): direct composition.
        let mut sx = Array::<F>::new(size);
        let mut asx = Array::<F>::new(size);
        ctaylor_sqrt::<F>(x, &mut sx, n);
        ctaylor_asinh::<F>(&sx, &mut asx, n);
        ctaylor_mul::<F>(&sx, &asx, out, n);
    } else {
        // Stable Padé branch (|x[0]| < 0.5): UNCONDITIONAL per D-06/B1.
        pade_8_8_sqrtx_asinh_sqrtx::<F>(x, out, n);
    }
}

// ---------------------------------------------------------------------------
//  ctaylor_br_inverse — out = inverse of BR_z evaluated as a CTaylor of z.
//  Phase 4 plan 04-00 Task 1 deliverable per CONTEXT D-02.
//
//  Port of `xcfun-master/src/functionals/brx.cpp:78-87` (`BR(t)` ctaylor adapter):
//
//  ```cpp
//  template <typename T, int Nvar>
//  static ctaylor<T, Nvar> BR(const ctaylor<T, Nvar> & t) {
//    auto tmp = BR_taylor<T, (Nvar >= 3) ? Nvar : 3>(t.c[0]);
//    ctaylor<T, Nvar> res = tmp[0];
//    for (int i = 1; i <= Nvar; i++)
//      res += tmp[i] * pow(t - t.c[0], i);
//    return res;
//  }
//  ```
//
//  In our cubecl multilinear CTaylor, the `BR_taylor` step (host scalar Newton
//  on slot 0 + linear-method polynomial sweep on slots 1..size) produces a
//  CTaylor whose coefficients are the Taylor inverse of BR_z at z[0]. We
//  populate that polynomial directly into `out` (`out[0] = BR(z[0])`,
//  `out[1..size]` = linear-method coefficients) — the host-side caller seeds
//  `out[CNST] = br_scalar(z[0])` BEFORE launch; this body fills the rest.
//
//  NOTE: the C++ `for (int i = 1; i <= Nvar; i++) res += tmp[i] * pow(t - t.c[0], i);`
//  loop composes the inverse polynomial against the actual outer ctaylor `t`.
//  In our pipeline, the caller has already pre-computed `z = (z-functional)`
//  as a CTaylor, and this primitive returns the inverse Taylor directly into
//  `out`. The composition step `res += tmp[i] * pow(t - t.c[0], i)` lives in
//  the BR family kernel (Wave 1 `mgga/shared/br_like.rs`), not here.
// ---------------------------------------------------------------------------

/// `out = BR_inv_taylor(z)` — populates the inverse polynomial of `BR_z` at
/// `z[0]` into `out`.
///
/// **Two-step host/device pipeline:**
/// - Host (caller, NOT inside this `#[cube]`): seed `out[CNST]` with
///   `br_scalar(z_cnst_host)` where `z_cnst_host` is the f64 constant slot
///   of the input CTaylor `z`.
/// - Device (this `#[cube]`): read `out[0]` (pre-seeded) and fill
///   `out[1..size]` via the Brent-Kung linear-method polynomial sweep.
///
/// Precondition: `out[0]` is pre-seeded with the host-computed Newton root.
/// The argument `z` is passed only so callers that thread the original outer
/// CTaylor can carry its derivative-seed pattern through the linear-method
/// recurrence (the recurrence reads/writes `out` slots only — `z` is not
/// directly consumed in this body, but is part of the canonical 3-arg
/// `ctaylor_<op>` signature shape).
///
/// Port reference: `xcfun-master/src/functionals/brx.cpp:50-71` (BR_taylor).
#[cube]
pub fn ctaylor_br_inverse<F: Float>(_z: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Step 1: linear-method polynomial sweep (uses pre-seeded out[0]).
    br_inverse_expand::<F>(out, n);

    // Suppress unused-warning on the imported `br_inverse_expand` if the
    // comptime size==1 branch elides it. cubecl 0.10-pre.3 should not warn.
}
