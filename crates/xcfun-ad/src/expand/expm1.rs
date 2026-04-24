//! `expm1_expand` — Taylor series of `exp(x0 + x) - 1` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:85-102`.
//! (The upstream function is `expm1(const ctaylor &)`; this expand form
//!  builds the scalar series used by `ctaylor_rec::compose`.)
//!
//! # C++ source (ctaylor_math.hpp:85-102)
//!
//! ```cpp
//! template <class T, int Nvar>
//! static ctaylor<T, Nvar> expm1(const ctaylor<T, Nvar> & t) {
//!   T tmp[Nvar + 1];
//!   exp_expand<T, Nvar>(tmp, t.c[0]);
//!   // Only constant value is affected by the cancellation
//!   if (fabs(t.c[0]) > 1e-3)
//!     tmp[0] -= 1;
//!   else
//!     tmp[0] = 2 * exp(t.c[0] / 2) * sinh(t.c[0] / 2);
//!   ctaylor<T, Nvar> res;
//!   ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
//!   return res;
//! }
//! ```
//!
//! # Identity
//!
//! The `i`-th derivative of `expm1(x)` equals the `i`-th derivative of
//! `exp(x)` for every `i >= 1`. Only the constant term differs:
//!
//! - `t[0] = exp(x0) - 1`  for `|x0| > 1e-3` (no cancellation),
//! - `t[0] = 2·exp(x0/2)·sinh(x0/2)`  for `|x0| <= 1e-3` (algebraically
//!   identical to `exp(x0) - 1` but evaluated via the stable identity to
//!   avoid catastrophic cancellation as `x0 → 0`).
//! - `t[i] = exp(x0) / i!`  for all `i >= 1`.
//!
//! # Precondition
//!
//! None. `expm1` is analytic everywhere on the reals.

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `expm1(x0 + x) = exp(x0+x) - 1`
/// at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
///
/// Port of `ctaylor_math.hpp:85-102`. The stable-bracket correction applies
/// only to `t[0]` (the constant term); higher-order coefficients `t[i>=1]`
/// equal `exp(x0) / i!` unchanged (derivatives of `expm1` match derivatives
/// of `exp`).
#[cube]
pub fn expm1_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // Step 1 — mirror exp_expand: fills t[0] = exp(x0), t[i] = exp(x0)/i! for i>=1.
    // ctaylor_math.hpp:91 — exp_expand<T, Nvar>(tmp, t.c[0]);
    let mut ifac = F::new(1.0);
    t[0] = x0.exp();
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        ifac *= i_f;
        t[k] = t[0] / ifac;
    }

    // Step 2 — stable-bracket correction for t[0] only
    // (ctaylor_math.hpp:93-97):
    //   if (fabs(t.c[0]) > 1e-3)  tmp[0] -= 1;
    //   else                      tmp[0] = 2 * exp(t.c[0] / 2) * sinh(t.c[0] / 2);
    //
    // The 1e-3 threshold is the upstream-blessed pivot: above it, `exp(x0) - 1`
    // is fine in f64; below it, the `2·exp(x0/2)·sinh(x0/2)` identity preserves
    // precision because `sinh(y)` for small `y` is computed via its own libm
    // path (no subtractive cancellation inside `sinh`). cubecl 0.10-pre.3
    // `Float` includes a `Sinh` bound — see cubecl-core/.../float.rs:29.
    let threshold = F::cast_from(1e-3_f64);
    let abs_x0 = x0.abs();
    if abs_x0 > threshold {
        // Safe path: direct expm1 = exp - 1.
        t[0] = t[0] - F::new(1.0);
    } else {
        // Stable-bracket path: 2 * exp(x0/2) * sinh(x0/2).
        // Operation order preserved left-to-right (SP-2, CLAUDE.md).
        let half = x0 / F::new(2.0);
        let e_half = half.exp();
        let s_half = half.sinh();
        let two = F::new(2.0);
        let m1 = two * e_half;
        t[0] = m1 * s_half;
    }
}
