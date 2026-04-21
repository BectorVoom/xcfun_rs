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
//! # `2/√π` constant — f64 precision via `F::cast_from`
//!
//! Cubecl 0.10-pre.3's `Float::new(val: f32)` accepts an `f32` literal only,
//! so `F::new(core::f32::consts::PI)` rounds π to f32 precision (~24 bits of
//! mantissa) before widening. At f64 target that costs ~2.7e-8 relative error
//! in π, cascading to ~1.3e-8 in `2/√π` — every `t[i]` inherits the drift.
//!
//! Plan 02-06 (Phase 2 tier-2 parity) surfaced this as the dominant
//! contribution to XC_LDAERFX order-2 error (~0.1 peak) and the smaller
//! LDAERFC/LDAERFC_JT order-2 drifts. Fix: pre-compute the f64 value on
//! the host and inject it via `F::cast_from::<f64>`, which preserves full
//! f64 precision through the `Cast` trait (same pattern used by
//! `density_vars::build` for the `1/3`, `4/3`, `-1/3` LDA exponents).
//!
//! Constant used: `2/√π = 1.1283791670955126_f64` (libm double-precision
//! `2.0 / libm::sqrt(std::f64::consts::PI)`).
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
    // Constant `2/√π` computed at host f64 precision and injected via
    // `F::cast_from::<f64>`. See the module header for why `F::new(f32)` is
    // unusable here (π rounds to f32 before widening).
    // Value: `2.0 / libm::sqrt(std::f64::consts::PI)` = 1.1283791670955126_f64,
    // exact to all 17 decimals matching C++ `2.0 / std::sqrt(M_PI)`.
    let c = F::cast_from(1.1283791670955126_f64);
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
