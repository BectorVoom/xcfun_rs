//! `sqrt_expand` — Taylor series of `sqrt(x0 + x)` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:164-170`.
//!
//! # C++ source (tmath.hpp:164-170)
//!
//! ```cpp
//! /* Use that (x0+x)^a=x0^a*(1+x/x0)^a */
//! template <class T, int N> static void sqrt_expand(T * t, const T & x0) {
//!   assert(x0 > 0 && "sqrt(x) not real analytic at x <= 0");
//!   t[0] = sqrt(x0);
//!   T x0inv = 1 / x0;
//!   for (int i = 1; i <= N; i++)
//!     t[i] = t[i - 1] * ((3 * x0inv) / (2 * i) - x0inv);
//! }
//! ```
//!
//! # Identity
//!
//! `sqrt(x0 + x) = sum_{i>=0} C(1/2, i) * x0^(1/2 - i) * x^i`.
//! Recurrence: `t[0] = sqrt(x0)`, `t[i] = t[i-1] * ((3/(2i) - 1) / x0)`.
//! (Equivalent to `pow_expand` specialised at `a = 1/2`.)
//!
//! # Precondition
//!
//! `x0 > 0`. The C++ reference enforces this via `assert!` (tmath.hpp:165).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! CONTEXT.md D-11 mandates the `assert!` be active in release builds,
//! but cubecl 0.10-pre.3's `#[cube]` macro rejects host-style assertion
//! macros inside kernel bodies. This falls under CONTEXT.md D-05's
//! explicit fallback clause (host-side guard at kernel entry). Callers
//! must verify `x0 > 0` before launching.

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `sqrt(x0 + x)` at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn sqrt_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // tmath.hpp:165 — precondition moved to host-side guard.

    // tmath.hpp:166 — `t[0] = sqrt(x0)` via cubecl's `Sqrt` trait.
    t[0] = x0.sqrt();
    // tmath.hpp:167 — `T x0inv = 1 / x0`
    let x0inv = F::new(1.0) / x0;

    // tmath.hpp:169 — `t[i] = t[i-1] * ((3*x0inv)/(2*i) - x0inv)`.
    // Explicit `let` per SP-2: five steps matching PATTERNS.md example.
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        let num = F::new(3.0) * x0inv; // 3 * x0inv
        let den = F::new(2.0) * i_f; // 2 * i
        let quot = num / den; // (3*x0inv) / (2*i)
        let factor = quot - x0inv; // ... - x0inv
        t[k] = t[k - 1] * factor; // t[i-1] * factor
    }
}
