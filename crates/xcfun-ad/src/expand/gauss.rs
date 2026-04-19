//! `gauss_expand` — Taylor series of `exp(-(a + y)²)` in `y`, around `y = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:200-215`.
//!
//! # C++ source (tmath.hpp:200-215)
//!
//! ```cpp
//! /*
//!    Taylor expansion of exp(-(a+x)^2) =
//!    exp(-a^2-2a*x)*exp(-x^2)
//!    Just doing a composition is unstable near 0.
//!  */
//! template <class T, int Ndeg> static void gauss_expand(T * t, const T & a) {
//!   exp_expand<T, Ndeg>(t, -a * a);
//!   tfuns<T, Ndeg>::stretch(t, -2 * a);
//!   T g[Ndeg + 1];
//!   g[0] = 1;
//!   for (int i = 1; i <= Ndeg; i += 2)
//!     g[i] = 0;
//!   for (int i = 1; i <= Ndeg / 2; i++)
//!     g[2 * i] = -g[2 * (i - 1)] / i;
//!   tfuns<T, Ndeg>::multo(t, g);
//! }
//! ```
//!
//! # Identity
//!
//! `exp(-(a + y)²) = exp(-a² - 2ay - y²) = exp(-a² + y') * exp(-y²)` where
//! `y' = -2ay`. Implementation decomposes:
//! 1. `exp_expand(t, -a²)` — fills `t` with Taylor of `exp(-a² + y)` in `y`
//!    (coefficient of `y^i` is `exp(-a²) / i!`).
//! 2. `tfuns::stretch(t, -2a)` — replaces `y ↦ -2ay`, so `t` now holds
//!    the Taylor of `exp(-a² - 2ay)` truncated at order n.
//! 3. Build `g` = Taylor of `exp(-y²)` = `[1, 0, -1, 0, 1/2, 0, -1/6, ...]`
//!    (even-only, alternating sign, factorial denom: `g[2k] = (-1)^k / k!`).
//! 4. `tfuns::multo(t, g)` — in-place `t *= g`. Final: `t` is the Taylor
//!    series of `exp(-(a+y)²)`.
//!
//! # Precondition
//!
//! None. `gauss(y) = exp(-y²)` is entire on the reals.
//!
//! # Cubecl 0.10-pre.3 notes
//!
//! - Scratch `g` buffer is allocated via `Array::<F>::new(comptime!((n+1) as usize))`.
//! - The inner `for (i = 1..=Ndeg/2)` loop iterates `n/2` times; we unroll
//!   it with a comptime guard `if comptime!(2*i <= n)`.

use cubecl::prelude::*;

use crate::expand::exp::exp_expand;
use crate::tfuns::{tfuns_multo, tfuns_stretch};

/// Fill `t[0..=n]` with the Taylor coefficients of `exp(-(a + y)²)` at `y = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn gauss_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:206 — exp_expand(t, -a*a).
    let neg_a_sq = -(a * a);
    exp_expand::<F>(t, neg_a_sq, n);

    // tmath.hpp:207 — tfuns::stretch(t, -2 * a).
    let neg_two_a = F::new(-2.0) * a;
    tfuns_stretch::<F>(t, neg_two_a, n);

    // tmath.hpp:208 — T g[Ndeg + 1] scratch.
    let g_len = comptime!((n + 1) as usize);
    let mut g = Array::<F>::new(g_len);

    // tmath.hpp:209 — g[0] = 1.
    let one = F::new(1.0);
    let zero = F::new(0.0);
    g[0] = one;
    // tmath.hpp:210-211 — zero out all odd slots.
    // Rather than a stride-2 loop (which needs special cubecl handling),
    // zero the full range 1..=n then overwrite even slots in the next loop.
    // This is strictly equivalent: for odd i, g[i] stays 0; for even i, it
    // gets overwritten by the recurrence.
    if comptime!(n >= 1) {
        #[unroll]
        for i in 1_u32..=n {
            let ki = i as usize;
            g[ki] = zero;
        }
    }
    // tmath.hpp:212-213 — g[2i] = -g[2(i-1)] / i for i ∈ 1..=n/2.
    // Unroll over 1..=n/2 with a comptime guard.
    if comptime!(n >= 2) {
        #[unroll]
        for i in 1_u32..=n {
            if comptime!(2_u32 * i <= n) {
                let k2i = (2_u32 * i) as usize;
                let k2im = (2_u32 * (i - 1_u32)) as usize;
                let i_f = F::cast_from(i);
                let prev = g[k2im];
                let neg_prev = -prev;
                g[k2i] = neg_prev / i_f;
            }
        }
    }

    // tmath.hpp:214 — tfuns::multo(t, g). t *= g, descending write order.
    tfuns_multo::<F>(t, &g, n);
}
