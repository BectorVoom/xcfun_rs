//! Taylor series of `exp(x0+x) = exp(x0) * exp(x)`. Port of
//! `xcfun-master/external/upstream/taylor/tmath.hpp:132-139`.
//!
//! # C++ recurrence (tmath.hpp:132-139)
//!
//! ```cpp
//! // Evaluate the taylor series of exp(x0+x)=exp(x0)*exp(x)
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
//! # Mathematical identity
//!
//! `exp(x0+x) = exp(x0) * sum_{i>=0} x^i / i!`, so the coefficient of `x^i`
//! is `exp(x0) / i!`. The recurrence accumulates `ifac = i!` and divides.

/// Fill `t[0..=N]` with Taylor coefficients of `exp(x0+x)` where `N = t.len() - 1`.
///
/// Writes into caller-provided slice; no heap allocation. `t[0] = exp(x0)`
/// uses `std::f64::exp` (default `std` feature) or `libm::exp` (when `std`
/// is off and `libm` is on) — see D-14.
pub fn exp_expand(t: &mut [f64], x0: f64) {
    // tmath.hpp:133: T ifac = 1;
    let mut ifac: f64 = 1.0;

    // tmath.hpp:134: t[0] = exp(x0);
    t[0] = exp_f64(x0);

    // tmath.hpp:135-138: for (int i = 1; i <= Ndeg; i++) { ifac *= i; t[i] = t[0] / ifac; }
    // D-08: explicit rebind of `ifac` mirrors C++ `ifac *= i`.
    for i in 1..t.len() {
        let i_f = i as f64;
        ifac = ifac * i_f;
        t[i] = t[0] / ifac;
    }
}

#[inline]
fn exp_f64(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.exp()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::exp(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// N=0: `t == [exp(x0)]`. For `x0 = 1.0`, bit-equal to the build host's
    /// `(1.0_f64).exp()`.
    #[test]
    fn exp_expand_n0() {
        let mut t = [0.0_f64; 1];
        exp_expand(&mut t, 1.0);
        assert_eq!(t[0].to_bits(), (1.0_f64).exp().to_bits());
    }

    /// N=4 at `x0 = 0.0`: `exp(0) = 1`, so `t[i] = 1/i!` exactly for
    /// `i = 0..=2` (each reciprocal factorial is representable in f64), and
    /// within 2 ULP for `i = 3, 4`.
    #[test]
    fn exp_expand_n4_at_zero() {
        let mut t = [0.0_f64; 5];
        exp_expand(&mut t, 0.0);
        // exp(0) = 1 exactly.
        assert_eq!(t[0].to_bits(), 1.0_f64.to_bits());
        assert_eq!(t[1].to_bits(), 1.0_f64.to_bits()); // 1/1
        assert_eq!(t[2].to_bits(), 0.5_f64.to_bits()); // 1/2

        // 1/6 and 1/24 are not exactly representable but the recurrence
        // (1/(2*3)) and (1/(6*4)) match the direct values bit-for-bit on x86-64.
        let expected_3 = 1.0_f64 / 6.0;
        let expected_4 = expected_3 / 4.0;
        let diff_3 = (t[3].to_bits() as i64 - expected_3.to_bits() as i64).unsigned_abs();
        let diff_4 = (t[4].to_bits() as i64 - expected_4.to_bits() as i64).unsigned_abs();
        assert!(diff_3 <= 2, "t[3] ulp={diff_3}, got {} expected {}", t[3], expected_3);
        assert!(diff_4 <= 2, "t[4] ulp={diff_4}, got {} expected {}", t[4], expected_4);
    }

    /// At `x0 = 1.0`, `t[0] = e` and each subsequent `t[i] = e / i!`.
    #[test]
    fn exp_expand_n2_at_one() {
        let mut t = [0.0_f64; 3];
        exp_expand(&mut t, 1.0);
        let e = (1.0_f64).exp();
        assert_eq!(t[0].to_bits(), e.to_bits());
        assert_eq!(t[1].to_bits(), e.to_bits()); // e / 1! = e
        // t[2] = e / 2! = e / 2 — bit-equal to `e / 2.0` on IEEE-754 f64.
        let expected_2 = e / 2.0;
        assert_eq!(t[2].to_bits(), expected_2.to_bits());
    }
}
