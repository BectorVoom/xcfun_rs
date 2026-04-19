//! `inv_expand` — Taylor series of `1 / (a + x)` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:124-129`.
//!
//! # C++ source (tmath.hpp:124-129)
//!
//! ```cpp
//! template <class T, int N> static void inv_expand(T * t, const T & a) {
//!   assert(a != 0 && "1/(a+x) not analytic at a = 0");
//!   t[0] = 1 / a;
//!   for (int i = 1; i <= N; i++)
//!     t[i] = -t[i - 1] * t[0];
//! }
//! ```
//!
//! # Identity
//!
//! `1 / (a + x) = sum_{i>=0} (-1)^i / a^{i+1} * x^i`.
//! Recurrence: `t[0] = 1/a`, `t[i] = -t[i-1] * t[0]` (which equals
//! `(-1)^i / a^{i+1}`).
//!
//! # Precondition
//!
//! `a != 0`. The C++ reference enforces this via `assert!` (tmath.hpp:125).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! CONTEXT.md D-11 mandates the `assert!` be active in release builds,
//! but cubecl 0.10-pre.3's `#[cube]` macro rejects both `assert!` and
//! the debug-only form inside kernel bodies ("Unsupported macro"). This
//! falls under CONTEXT.md D-05's explicit fallback clause (host-side
//! guard at kernel entry). Callers must verify `a != 0` before launching
//! this kernel. Silent-NaN on `1/0` remains the correctness risk
//! (Pitfall P10).

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `1 / (a + x)`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells. See the
/// module header for the precondition and operation-order rationale.
#[cube]
pub fn inv_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:125 — `assert(a != 0 ...)`. Precondition guard moved to
    // host-side callers (cubecl 0.10-pre.3 rejects host-style
    // assertions inside `#[cube]` bodies). See module header for the
    // D-05 / D-11 fallback justification.

    // tmath.hpp:126 — `t[0] = 1 / a`
    let t0 = F::new(1.0) / a;
    t[0] = t0;

    // tmath.hpp:127-128 — `t[i] = -t[i-1] * t[0]`.
    // Explicit `let` per SP-2 defeats compiler re-association. The `-x`
    // operator is on `Float` via `core::ops::Neg` (confirmed in
    // `ctaylor_neg` in `ctaylor.rs`).
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let prev = t[k - 1];
        let neg_prev = -prev;
        t[k] = neg_prev * t0;
    }
}
