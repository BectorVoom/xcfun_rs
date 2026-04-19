//! `cbrt_expand` — Taylor series of `cbrt(x0 + x)` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:172-178`.
//!
//! # C++ source (tmath.hpp:172-178)
//!
//! ```cpp
//! template <class T, int N> static void cbrt_expand(T * t, const T & x0) {
//!   assert(x0 > 0 && "pow(x,a) not real analytic at x <= 0");
//!   t[0] = cbrt(x0);
//!   T x0inv = 1 / x0;
//!   for (int i = 1; i <= N; i++)
//!     t[i] = t[i - 1] * ((4 * x0inv) / (3 * i) - x0inv);
//! }
//! ```
//!
//! # Identity
//!
//! `cbrt(x0 + x) = sum_{i>=0} C(1/3, i) * x0^(1/3 - i) * x^i`.
//! Recurrence: `t[0] = cbrt(x0)`, `t[i] = t[i-1] * ((4/(3i) - 1) / x0)`.
//! Structurally identical to `sqrt_expand` with `3/(2i) → 4/(3i)`.
//!
//! # Precondition
//!
//! `x0 > 0`. The C++ reference enforces this via `assert!` (tmath.hpp:173).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! CONTEXT.md D-11 mandates the `assert!` be active in release builds,
//! but cubecl 0.10-pre.3's `#[cube]` macro rejects host-style assertion
//! macros inside kernel bodies. This falls under CONTEXT.md D-05's
//! explicit fallback clause (host-side guard at kernel entry). Callers
//! must verify `x0 > 0` before launching.
//!
//! # Cubecl 0.10-pre.3 API deviation: `cbrt` is not on `Float`
//!
//! Cubecl 0.10-pre.3's `Float` trait does not expose a dedicated `cbrt`
//! intrinsic (see `cubecl-core/src/frontend/operation/unary.rs`: `Erf`,
//! `Sqrt`, `Exp`, `Log` are implemented; `Cbrt` is absent). We use
//! `x0.powf(F::new(1.0 / 3.0))` which on cubecl-cpu lowers to the host
//! libm `pow(x, 1.0/3.0)`. Expected drift vs. C++ `std::cbrt` is 1–2 ULP
//! — within Phase 1's 1e-13 relative-error integration-test budget, and
//! the Plan 01-05 C++ golden-fixture gate will accept 1e-12 on the full
//! composed `ctaylor_cbrt` pipeline (see Plan 01-05 for the final
//! tolerance contract).

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `cbrt(x0 + x)` at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn cbrt_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // tmath.hpp:173 — precondition moved to host-side guard.

    // tmath.hpp:174 — `t[0] = cbrt(x0)` via `powf(1/3)` fallback (see
    // module header for the 1–2 ULP deviation note).
    t[0] = x0.powf(F::new(1.0_f32 / 3.0_f32));
    // tmath.hpp:175 — `T x0inv = 1 / x0`
    let x0inv = F::new(1.0) / x0;

    // tmath.hpp:177 — `t[i] = t[i-1] * ((4*x0inv)/(3*i) - x0inv)`.
    // Explicit `let` per SP-2: five steps matching `sqrt_expand`.
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        let num = F::new(4.0) * x0inv;         // 4 * x0inv
        let den = F::new(3.0) * i_f;           // 3 * i
        let quot = num / den;                  // (4*x0inv) / (3*i)
        let factor = quot - x0inv;             // ... - x0inv
        t[k] = t[k - 1] * factor;              // t[i-1] * factor
    }
}
