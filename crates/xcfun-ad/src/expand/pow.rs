//! `pow_expand` — Taylor series of `(x0 + x)^a` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:154-161`.
//!
//! # C++ source (tmath.hpp:154-161)
//!
//! ```cpp
//! /* Use that (x0+x)^a=x0^a*(1+x/x0)^a */
//! template <class T, int N> static void pow_expand(T * t, T x0, T a) {
//!   if (x0 <= 0)
//!     assert(x0 > 0 && "pow(x,a) not real analytic at x <= 0");
//!   t[0] = pow(x0, a);
//!   T x0inv = 1 / x0;
//!   for (int i = 1; i <= N; i++)
//!     t[i] = t[i - 1] * x0inv * (a - i + 1) / i;
//! }
//! ```
//!
//! # Identity
//!
//! `(x0 + x)^a = x0^a * (1 + x/x0)^a = x0^a * sum_{i>=0} C(a, i) * (x/x0)^i`.
//! Recurrence: `t[0] = x0^a`, `t[i] = t[i-1] * (1/x0) * (a - i + 1) / i`.
//!
//! The precondition is on `x0`, **not** on `a` — pre-pivot PATTERNS.md note.
//! `a` may be any real.
//!
//! # Precondition
//!
//! `x0 > 0`. The C++ reference enforces this via `assert!` (tmath.hpp:156).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! CONTEXT.md D-11 mandates the `assert!` be active in release builds,
//! but cubecl 0.10-pre.3's `#[cube]` macro rejects host-style assertion
//! macros inside kernel bodies. This falls under CONTEXT.md D-05's
//! explicit fallback clause (host-side guard at kernel entry). Callers
//! must verify `x0 > 0` before launching.

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `(x0 + x)^a` at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn pow_expand<F: Float>(t: &mut Array<F>, x0: F, a: F, #[comptime] n: u32) {
    // tmath.hpp:155-156 — precondition moved to host-side guard.

    // tmath.hpp:157 — `t[0] = pow(x0, a)` via cubecl's `Powf` trait.
    t[0] = x0.powf(a);

    // tmath.hpp:158 — `T x0inv = 1 / x0`
    let x0inv = F::new(1.0) / x0;

    // tmath.hpp:159-160 — recurrence `t[i] = t[i-1] * x0inv * (a - i + 1) / i`.
    // Explicit `let` per SP-2 preserves C++ left-to-right associativity.
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        // (a - i + 1) — C++ left-assoc: ((a - i) + 1)
        let a_minus_i = a - i_f;
        let a_minus_i_plus_1 = a_minus_i + F::new(1.0);
        // t[i-1] * x0inv * (a - i + 1) / i  — C++ left-to-right
        let s1 = t[k - 1] * x0inv;
        let s2 = s1 * a_minus_i_plus_1;
        t[k] = s2 / i_f;
    }
}
