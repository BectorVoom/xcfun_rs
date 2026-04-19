//! Taylor series of `log(a+x) = log(1+x/a) + log(a)`. Port of
//! `xcfun-master/external/upstream/taylor/tmath.hpp:142-151`.
//!
//! # C++ recurrence (tmath.hpp:142-151)
//!
//! ```cpp
//! // Log series log(a+x) = log(1+x/a) + log(a)
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
//! # Mathematical identity
//!
//! `log(a+x) = log(a) + sum_{i>=1} (-1)^(i+1) * (x/a)^i / i`. The `sign`
//! factor `2 * (i & 1) - 1` is `+1` for odd `i` and `-1` for even `i`,
//! matching `(-1)^(i+1)`.
//!
//! # Precondition
//!
//! `x0 > 0`. `log(x)` is not real-analytic for non-positive real `x`.
//! Enforced with `assert!` (active in release per D-11) to catch the
//! silent-NaN failure mode P10.

/// Fill `t[0..=N]` with Taylor coefficients of `log(x0+x)` where `N = t.len() - 1`.
///
/// Writes into caller-provided slice; no heap allocation.
///
/// # Panics
///
/// Panics if `x0 <= 0.0`. `log(x)` is not real-analytic at `x <= 0`.
pub fn log_expand(t: &mut [f64], x0: f64) {
    // tmath.hpp:143: assert(x0 > 0 && "log(x) not real analytic at x <= 0");
    assert!(x0 > 0.0, "log(x) not real analytic at x <= 0");

    // tmath.hpp:144: t[0] = log(x0);
    t[0] = log_f64(x0);

    // tmath.hpp:145: T x0inv = 1 / x0;
    let x0inv = 1.0 / x0;

    // tmath.hpp:146: T xn = x0inv;
    let mut xn = x0inv;

    // tmath.hpp:147-150: for (int i = 1; i <= N; i++) {
    //     t[i] = (xn / double(i)) * (2 * (i & 1) - 1);
    //     xn *= x0inv;
    // }
    // D-08: bind each sub-expression so operator order mirrors C++.
    for i in 1..t.len() {
        let i_f = i as f64;
        let sign = (2 * ((i & 1) as i32) - 1) as f64;
        let term = xn / i_f;
        t[i] = term * sign;
        xn = xn * x0inv;
    }
}

#[inline]
fn log_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.ln()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::log(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// N=0: `t == [log(x0)]`.
    #[test]
    fn log_expand_n0_at_one() {
        let mut t = [0.0_f64; 1];
        log_expand(&mut t, 1.0);
        // log(1) = 0 exactly.
        assert_eq!(t[0].to_bits(), 0.0_f64.to_bits());
    }

    /// N=3 at `x0 = 1.0`:
    ///  - t[0] = log(1) = 0
    ///  - t[1] = (1/1) * (+1) = 1
    ///  - t[2] = (1/2) * (-1) = -0.5
    ///  - t[3] = (1/3) * (+1) = 1/3 — asserted ULP-close (1/3 not exact in f64).
    #[test]
    fn log_expand_n3_at_one() {
        let mut t = [0.0_f64; 4];
        log_expand(&mut t, 1.0);
        assert_eq!(t[0].to_bits(), 0.0_f64.to_bits());
        assert_eq!(t[1].to_bits(), 1.0_f64.to_bits());
        assert_eq!(t[2].to_bits(), (-0.5_f64).to_bits());
        let expected_3 = 1.0_f64 / 3.0;
        let diff_3 = (t[3].to_bits() as i64 - expected_3.to_bits() as i64).unsigned_abs();
        assert!(diff_3 <= 2, "t[3] ulp={diff_3}, got {} expected {}", t[3], expected_3);
    }

    /// Precondition `x0 > 0` must panic for `x0 = 0.0`.
    #[test]
    #[should_panic(expected = "log(x) not real analytic at x <= 0")]
    fn log_expand_panics_on_zero() {
        let mut t = [0.0_f64; 4];
        log_expand(&mut t, 0.0);
    }

    /// Precondition `x0 > 0` must panic for negative input.
    #[test]
    #[should_panic(expected = "log(x) not real analytic at x <= 0")]
    fn log_expand_panics_on_neg() {
        let mut t = [0.0_f64; 4];
        log_expand(&mut t, -1.0);
    }

    /// At `x0 = 2.0`: t[0] = ln(2); t[1] = 1/2 = 0.5 (exact); t[2] = -(1/4)/2 = -0.125 (exact).
    #[test]
    fn log_expand_n2_at_two() {
        let mut t = [0.0_f64; 3];
        log_expand(&mut t, 2.0);
        assert_eq!(t[0].to_bits(), (2.0_f64).ln().to_bits());
        assert_eq!(t[1].to_bits(), 0.5_f64.to_bits());
        assert_eq!(t[2].to_bits(), (-0.125_f64).to_bits());
    }
}
