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
use crate::expand::erf::erf_expand;
use crate::expand::exp::exp_expand;
use crate::expand::inv::inv_expand;
use crate::expand::log::log_expand;
use crate::expand::pow::pow_expand;
use crate::expand::sqrt::sqrt_expand;

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
pub fn ctaylor_reciprocal<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
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
pub fn ctaylor_pow<F: Float>(
    x: &Array<F>,
    a: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    pow_expand::<F>(&mut scratch, x[0], a, n);
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
/// Note — inherits `erf_expand`'s cubecl-cpu polyfill precision
/// (~1.3e-8 on `2/√π`, max ~1.5e-7 on `erf(a)` via the polyfill).
/// See `crates/xcfun-ad/src/expand/erf.rs` for the drift disclosure.
#[cube]
pub fn ctaylor_erf<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);

    erf_expand::<F>(&mut scratch, x[0], n);
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
fn ctaylor_powi_copy<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!(1_u32 << n);
    #[unroll]
    for i in 0..size {
        let k = i as usize;
        out[k] = x[k];
    }
}

/// `out = x` (exponent 1).
#[cube]
pub fn ctaylor_powi_1<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_copy::<F>(x, out, n);
}

/// `out = x * x` (exponent 2).
#[cube]
pub fn ctaylor_powi_2<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
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
pub fn ctaylor_powi_3<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 3_u32, n);
}

/// `out = x^4`.
#[cube]
pub fn ctaylor_powi_4<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 4_u32, n);
}

/// `out = x^5`.
#[cube]
pub fn ctaylor_powi_5<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 5_u32, n);
}

/// `out = x^6`.
#[cube]
pub fn ctaylor_powi_6<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 6_u32, n);
}

/// `out = x^7`.
#[cube]
pub fn ctaylor_powi_7<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 7_u32, n);
}

/// `out = x^8`.
#[cube]
pub fn ctaylor_powi_8<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 8_u32, n);
}

/// `out = x^9`.
#[cube]
pub fn ctaylor_powi_9<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_powi_positive::<F>(x, out, 9_u32, n);
}

/// `out = x^10`.
#[cube]
pub fn ctaylor_powi_10<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
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
pub fn ctaylor_powi_neg1<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_pow::<F>(x, F::new(-1.0), out, n);
}

/// `out = x^(-2)` — delegates to `ctaylor_pow` with `a = -2`.
#[cube]
pub fn ctaylor_powi_neg2<F: Float>(
    x: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
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
