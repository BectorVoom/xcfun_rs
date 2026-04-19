//! `exp_expand` — Taylor series of `exp(x0 + x)` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:132-139`.
//!
//! # C++ source (tmath.hpp:132-139)
//!
//! ```cpp
//! template <class T, int Ndeg> static void exp_expand(T * t, const T & x0) {
//!   T ifac = 1;
//!   t[0] = exp(x0);
//!   for (int i = 1; i <= Ndeg; i++) {
//!     ifac *= i;
//!     t[i] = t[0] / ifac;
//!   }
//! }
//! ```
//!
//! # Identity
//!
//! `exp(x0 + x) = exp(x0) * sum_{i>=0} x^i / i!`.
//! Recurrence: `t[0] = exp(x0)`, `t[i] = t[0] / i!` (where `i!` is built
//! cumulatively by `ifac *= i`).
//!
//! # Precondition
//!
//! None. `exp` is analytic everywhere on the reals.

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `exp(x0 + x)` at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn exp_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // tmath.hpp:133 — `T ifac = 1`
    let mut ifac = F::new(1.0);
    // tmath.hpp:134 — `t[0] = exp(x0)` via cubecl's `Exp` trait method.
    t[0] = x0.exp();
    // tmath.hpp:135-138 — cumulative factorial + division. Operation
    // order preserved left-to-right via explicit `let` (SP-2).
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        ifac *= i_f;
        t[k] = t[0] / ifac;
    }
}
