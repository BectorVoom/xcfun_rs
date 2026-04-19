//! `atan_expand` — Taylor series of `atan(a + y)` in `y`, around `y = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:180-198`.
//!
//! # C++ source (tmath.hpp:180-198)
//!
//! ```cpp
//! // Use that d/dx atan(x) = 1/(1 + x^2),
//! // Taylor expand in x^2 and integrate.
//! template <class T, int Ndeg> static void atan_expand(T * t, T a) {
//!   // Calculate taylor expansion of 1/(1+a^2+x)
//!   T x[Ndeg + 1];
//!   inv_expand<T, Ndeg>(t, 1 + a * a);
//!   // insert x = 2*a*x + x^2
//!   x[0] = 0;
//!   if (Ndeg > 0)
//!     x[1] = 2 * a;
//!   if (Ndeg > 1)
//!     x[2] = 1;
//!   for (int i = 3; i <= Ndeg; i++)
//!     x[i] = 0;
//!   tfuns<T, Ndeg>::compose(t, x);
//!   // Integrate each term and set the constant
//!   tfuns<T, Ndeg>::integrate(t);
//!   t[0] = atan(a);
//! }
//! ```
//!
//! # Identity
//!
//! `atan(a + y)` is the antiderivative in `y` of `1 / (1 + (a+y)²)`. Expand
//! `(a + y)² = a² + 2ay + y²`, so the denominator is `1 + a² + (2ay + y²)`.
//! Let `u(y) = 2ay + y²` (so `u(0) = 0`). Then
//!
//!   1 / (1 + a² + u(y)) = (1/(1+a²)) * sum_{i≥0} (-u)^i / (1+a²)^i
//!
//! The C++ algorithm builds this in three steps:
//! 1. `inv_expand(t, 1 + a²)` — fills `t` with the Taylor series of
//!    `1/(1 + a² + y)` in `y` (coefficients are `(-1)^i / (1+a²)^(i+1)`).
//! 2. `tfuns::compose(t, x)` where `x = [0, 2a, 1, 0, ...]` — substitutes
//!    `y ↦ u(y) = 2ay + y²`, turning `t` into the Taylor series of
//!    `1/(1 + (a+y)²)`.
//! 3. `tfuns::integrate(t)` — anti-derivative; final `t[0]` set to `atan(a)`.
//!
//! # Precondition
//!
//! None. `atan` is analytic on all reals.
//!
//! # Cubecl 0.10-pre.3 notes
//!
//! - `F::atan(x)` is callable as `x.atan()` (method form — see Plan 01-03
//!   Observed Quirks).
//! - Scratch `x` buffer is allocated via `Array::<F>::new(comptime!((n+1) as usize))`
//!   — this lowers to stack-local storage under cubecl-cpu (tested below).

use cubecl::prelude::*;

use crate::expand::inv::inv_expand;
use crate::tfuns::{tfuns_compose, tfuns_integrate};

/// Fill `t[0..=n]` with the Taylor coefficients of `atan(a + y)` at `y = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn atan_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:184 — T x[Ndeg + 1] scratch.
    let x_len = comptime!((n + 1) as usize);
    let mut x = Array::<F>::new(x_len);

    // tmath.hpp:185 — `inv_expand<T, Ndeg>(t, 1 + a*a)` — fills t with the
    // Taylor coefficients of 1/(1 + a² + y) in `y`.
    let one = F::new(1.0);
    let a_sq = a * a;
    let one_plus_a_sq = one + a_sq;
    inv_expand::<F>(t, one_plus_a_sq, n);

    // tmath.hpp:186-193 — build x = [0, 2a, 1, 0, 0, ...] (compose invariant
    // x[0] = 0; see tmath.hpp:79).
    let zero = F::new(0.0);
    let two = F::new(2.0);
    x[0] = zero;
    // Emit the `if (Ndeg > 0) x[1] = 2*a` branch at comptime.
    if comptime!(n >= 1) {
        x[1] = two * a;
    }
    if comptime!(n >= 2) {
        x[2] = one;
    }
    // tmath.hpp:192-193 — `for (int i = 3; i <= Ndeg; i++) x[i] = 0`.
    if comptime!(n >= 3) {
        #[unroll]
        for i in 3_u32..=n {
            let ki = i as usize;
            x[ki] = zero;
        }
    }

    // tmath.hpp:194 — tfuns::compose(t, x) — substitute y ↦ 2ay + y² into t.
    tfuns_compose::<F>(t, &x, n);
    // tmath.hpp:196 — tfuns::integrate(t) — anti-derivative (leaves t[0]
    // undefined per the tfuns contract).
    tfuns_integrate::<F>(t, n);
    // tmath.hpp:197 — t[0] = atan(a).
    t[0] = a.atan();
}
