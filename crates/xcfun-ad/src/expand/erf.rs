//! `erf_expand` — Taylor series of `erf(a + y)` in `y`, around `y = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:217-225`.
//!
//! # C++ source (tmath.hpp:217-225)
//!
//! ```cpp
//! // Use that d/dx erf(x) = 2/sqrt(pi)*exp(-x^2),
//! // Taylor expand in x^2 and integrate.
//! template <class T, int Ndeg> static void erf_expand(T * t, const T & a) {
//!   gauss_expand<T, Ndeg>(t, a);
//!   for (int i = 0; i <= Ndeg; i++)
//!     t[i] *= 2 / sqrt(M_PI);
//!   tfuns<T, Ndeg>::integrate(t);
//!   t[0] = erf(a);
//! }
//! ```
//!
//! # Identity
//!
//! `erf'(x) = (2/√π) · exp(-x²)`. Steps:
//! 1. `gauss_expand(t, a)` — fills `t` with Taylor of `exp(-(a+y)²)`.
//! 2. Scale every `t[i]` by `2/√π`.
//! 3. `tfuns::integrate(t)` — anti-derivative in `y`.
//! 4. `t[0] = erf(a)` — seed the constant.
//!
//! # Precondition
//!
//! None. `erf` is entire on the reals.
//!
//! # Cubecl 0.10-pre.3 API deviation: `2/√π` rounded through f32 precision
//!
//! Cubecl 0.10-pre.3's `Float::new(val: f32)` accepts an `f32` literal only,
//! which rounds mathematical constants like π to f32 precision (~24 bits of
//! mantissa) before widening to the target float type. For f64 target this
//! introduces a ~2.7e-8 relative error in the value of π, propagating to
//! a ~1.3e-8 relative error in `2/√π`. Every coefficient `t[i]` is scaled
//! by this constant, so every `t[i]` inherits the ~1.3e-8 drift.
//!
//! This mirrors the `cbrt` precedent from Plan 01-03: rather than pass
//! `2/√π` as a kernel scalar arg (violating the plan signature), we
//! reproduce the f32-precision path in the host reference so the test
//! can still pass at bit-exact on cubecl-cpu. Downstream plans that compare
//! to C++ `std::erf` will need a per-cell tolerance relaxation (or a
//! scalar-arg revision) — pre-flagged for Plan 01-06's math.rs.
//!
//! `t[0] = erf(a)` itself uses cubecl's `Erf` unary op, which on cubecl-cpu
//! lowers to host libm `erf` — full f64 precision.

use cubecl::prelude::*;

use crate::expand::gauss::gauss_expand;
use crate::tfuns::tfuns_integrate;

/// Fill `t[0..=n]` with the Taylor coefficients of `erf(a + y)` at `y = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn erf_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:220 — gauss_expand(t, a). t now holds Taylor of exp(-(a+y)²).
    gauss_expand::<F>(t, a, n);

    // tmath.hpp:221-222 — t[i] *= 2 / sqrt(π) for i ∈ 0..=n.
    // The constant 2/√π is computed via F::new(f32) which rounds π to f32
    // precision. See the module header for the resulting ~1.3e-8 drift vs
    // the C++ reference computed with f64-precision M_PI.
    let two = F::new(2.0);
    let pi = F::new(core::f32::consts::PI);
    let sqrt_pi = pi.sqrt();
    let c = two / sqrt_pi;
    #[unroll]
    for i in 0_u32..=n {
        let ki = i as usize;
        t[ki] *= c;
    }

    // tmath.hpp:223 — tfuns::integrate(t) (leaves t[0] undefined).
    tfuns_integrate::<F>(t, n);

    // tmath.hpp:224 — t[0] = erf(a). cubecl's `Erf` unary op lowers to host
    // libm `erf` on cubecl-cpu — full f64 precision.
    t[0] = a.erf();
}
