//! `asinh_expand` — Taylor series of `asinh(a + y)` in `y`, around `y = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:259-274`.
//!
//! # C++ source (tmath.hpp:259-274)
//!
//! ```cpp
//! // hyperbolic arcsin function. d/dx asinh(x) = 1/sqrt(1+x^2)
//! // 1 + (a+x)^2 = 1+a^2 + 2ax + x^2
//! template <class T, int Ndeg> static void asinh_expand(T * t, const T & a) {
//!   T tmp[Ndeg + 1];
//!   tmp[0] = 1 + a * a;
//!   if (Ndeg > 0)
//!     tmp[1] = 2 * a;
//!   if (Ndeg > 1)
//!     tmp[2] = 1;
//!   for (int i = 3; i <= Ndeg; i++)
//!     tmp[i] = 0;
//!   pow_expand<T, Ndeg>(t, tmp[0], -0.5);
//!   tfuns<T, Ndeg>::compose(t, tmp);
//!   tfuns<T, Ndeg>::integrate(t);
//!   t[0] = asinh(a);
//! }
//! ```
//!
//! # Identity
//!
//! `asinh'(x) = 1 / sqrt(1 + x²)`. Expand the derivative in `y` around the
//! point `a`, then integrate:
//!
//!   1 / sqrt(1 + (a+y)²) = 1 / sqrt(1 + a² + u(y))
//!
//! where `u(y) = 2ay + y²`. Build the series of `(1 + a² + s)^(-1/2)` in
//! `s` via `pow_expand`, substitute `s ↦ u(y)` via `tfuns::compose`, then
//! `tfuns::integrate` to recover `asinh(a+y)`. Constant term is `asinh(a)`.
//!
//! # Note re. upstream asin/acos typos
//!
//! `tmath.hpp:290` (`asin_expand`) and `tmath.hpp:313` (`acos_expand`)
//! appear to contain transcription typos where `t[0] = asinh(a)` was
//! written where the intended values are `asin(a)` and `acos(a)`
//! respectively. Those are not ported in Phase 1 (see CONTEXT.md and the
//! future Phase 2+ plan tracks). This file (`asinh.rs`) implements the
//! correct asinh series — "asinh IS the function" here, not a typo port.
//!
//! # Precondition
//!
//! None. `asinh` is analytic on all reals (domain and range are all real
//! numbers; the underlying `pow_expand` is called with `x0 = 1 + a²` which
//! is always ≥ 1 > 0, so pow_expand's `x0 > 0` precondition is
//! unconditionally satisfied).
//!
//! # Cubecl 0.10-pre.3 notes
//!
//! - `F::asinh(x)` is callable as `x.asinh()` (method form). The `ArcSinh`
//!   unary op is emitted as `Arithmetic::ArcSinh` per
//!   `cubecl-core/src/frontend/operation/unary.rs`.
//! - Scratch `tmp` buffer is allocated via `Array::<F>::new(comptime!((n+1) as usize))`.

use cubecl::prelude::*;

use crate::expand::pow::pow_expand;
use crate::tfuns::{tfuns_compose, tfuns_integrate};

/// Fill `t[0..=n]` with the Taylor coefficients of `asinh(a + y)` at `y = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn asinh_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:262 — T tmp[Ndeg + 1] scratch.
    let tmp_len = comptime!((n + 1) as usize);
    let mut tmp = Array::<F>::new(tmp_len);

    // tmath.hpp:263-269 — build tmp = [1 + a², 2a, 1, 0, 0, ...].
    let one = F::new(1.0);
    let two = F::new(2.0);
    let zero = F::new(0.0);
    let a_sq = a * a;
    let one_plus_a_sq = one + a_sq;
    tmp[0] = one_plus_a_sq;
    if comptime!(n >= 1) {
        tmp[1] = two * a;
    }
    if comptime!(n >= 2) {
        tmp[2] = one;
    }
    if comptime!(n >= 3) {
        #[unroll]
        for i in 3_u32..=n {
            let ki = i as usize;
            tmp[ki] = zero;
        }
    }

    // tmath.hpp:270 — pow_expand(t, tmp[0], -0.5). Exponent is a real, so
    // pass as F::new(-0.5). `tmp[0]` is always > 0 (it's 1 + a²), so
    // pow_expand's host-side precondition is trivially satisfied here.
    let neg_half = F::new(-0.5);
    pow_expand::<F>(t, tmp[0], neg_half, n);

    // tmath.hpp:271 — tfuns::compose(t, tmp). Note: compose ignores tmp[0]
    // (tmath.hpp:79 says "assuming x[0] = 0"; compose's per-case bodies at
    // :82-113 never reference x[0]), so the non-zero tmp[0] here is benign.
    tfuns_compose::<F>(t, &tmp, n);

    // tmath.hpp:272 — tfuns::integrate(t) (leaves t[0] undefined).
    tfuns_integrate::<F>(t, n);

    // tmath.hpp:273 — t[0] = asinh(a).
    t[0] = a.asinh();
}
