//! `log_expand` — Taylor series of `log(x0 + x)` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:142-151`.
//!
//! # C++ source (tmath.hpp:142-151)
//!
//! ```cpp
//! template <class T, int N> static void log_expand(T * t, const T & x0) {
//!   assert(x0 > 0 && "log(x) not real analytic at x <= 0");
//!   t[0] = log(x0);
//!   T x0inv = 1 / x0;
//!   T xn = x0inv;
//!   for (int i = 1; i <= N; i++) {
//!     t[i] = (xn / double(i)) * (2 * (i & 1) - 1);
//!     xn *= x0inv;
//!   }
//! }
//! ```
//!
//! # Identity
//!
//! `log(x0 + x) = log(x0) + sum_{i>=1} (-1)^(i+1) / (i * x0^i) * x^i`.
//! Equivalently: coefficient of `x^i` is `x0^{-i} / i * (2*(i&1) - 1)`,
//! where `(2*(i&1) - 1)` toggles `+1, -1, +1, -1, …` as `i` runs
//! `1, 2, 3, 4, …`.
//!
//! # Sign-factor check
//!
//! - `i = 1`: `2*(1 & 1) - 1 = 2*1 - 1 = +1` → `t[1] = +x0inv/1`
//! - `i = 2`: `2*(2 & 1) - 1 = 2*0 - 1 = -1` → `t[2] = -x0inv^2/2`
//! - `i = 3`: `2*(3 & 1) - 1 = 2*1 - 1 = +1` → `t[3] = +x0inv^3/3`
//!
//! Cross-check for `x0 = 1`: `log(1 + x) = x - x^2/2 + x^3/3 - …`, so
//! `t = [0, 1, -1/2, 1/3, …]`. Matches.
//!
//! # Operation order (verbatim)
//!
//! The C++ update is `t[i] = (xn / i) * sign; xn *= x0inv` — `xn` is
//! **consumed before** its multiply-update, i.e. at iteration `i` the
//! factor used is `x0inv^i` (because `xn` starts at `x0inv` at `i = 1`).
//! This port preserves that ordering exactly.
//!
//! # Precondition
//!
//! `x0 > 0`. The C++ reference enforces this via `assert!` (tmath.hpp:143).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! CONTEXT.md D-11 mandates the `assert!` be active in release builds,
//! but cubecl 0.10-pre.3's `#[cube]` macro rejects both `assert!` and
//! the debug-only form inside kernel bodies ("Unsupported macro"). This
//! falls under CONTEXT.md D-05's explicit fallback clause (host-side
//! guard at kernel entry). Callers must verify `x0 > 0` before launching
//! this kernel.

use cubecl::prelude::*;

/// Fill `t[0..=n]` with the Taylor coefficients of `log(x0 + x)` at `x = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn log_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // tmath.hpp:143 — `assert(x0 > 0 ...)`. Precondition guard moved to
    // host-side callers (cubecl 0.10-pre.3 rejects host-style
    // assertions inside `#[cube]` bodies). See module header for the
    // D-05 / D-11 fallback justification.

    // tmath.hpp:144-146
    t[0] = x0.ln();
    let x0inv = F::new(1.0) / x0;
    let mut xn = x0inv;

    // tmath.hpp:147-150 — sign = 2 * (i & 1) - 1. Computed host-side at
    // kernel-build time: `i` is the unrolled loop counter (u32), so the
    // sign is a `#[comptime]` value and lowered to a pair of `+F::new(1.0)`
    // / `-F::new(1.0)` constants. Explicit `let` per SP-2.
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        // 2 * (i & 1) - 1 ∈ {+1, -1}; keep the cast explicit per SP-2.
        let sign_int = 2_i32 * ((i as i32) & 1_i32) - 1_i32;
        let sign_f = F::cast_from(sign_int);
        // Division first, then sign multiplication — matches C++
        // `(xn / double(i)) * (2 * (i & 1) - 1)`.
        let div = xn / i_f;
        t[k] = div * sign_f;
        // tmath.hpp:149 — `xn *= x0inv` AFTER the use.
        xn *= x0inv;
    }
}
