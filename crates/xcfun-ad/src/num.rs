//! Num trait -- numeric abstraction for both f64 and CTaylor.
//!
//! This trait provides the common interface that functionals use to be generic
//! over both scalar evaluation (f64) and automatic differentiation (CTaylor).

use crate::ctaylor::CTaylor;
use crate::math;

/// Numeric trait for types usable in xcfun functionals.
///
/// Provides arithmetic, transcendental functions, and comparison operations.
/// Implemented for `f64` (scalar evaluation) and `CTaylor<f64, N>` (AD evaluation).
pub trait Num:
    Clone
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
    + std::ops::AddAssign
    + std::ops::SubAssign
    + std::ops::MulAssign
    + Sized
{
    fn from_f64(val: f64) -> Self;
    fn zero() -> Self;
    fn one() -> Self;

    /// Get the scalar (constant-term) value as f64.
    /// For f64: returns self.
    /// For CTaylor: returns c[0].
    /// Required by DensityVars regularization (D-12) to inspect the constant term.
    fn value_f64(&self) -> f64;

    /// Set only the constant term, preserving all derivative coefficients.
    /// For f64: sets self = val.
    /// For CTaylor: sets self.c[0] = val, leaving c[1..] unchanged.
    /// Required by DensityVars regularization (D-12) to clamp density without
    /// destroying derivative information.
    fn set_constant(&mut self, val: f64);

    fn exp(self) -> Self;
    fn log(self) -> Self;
    fn sqrt(self) -> Self;
    fn cbrt(self) -> Self;
    fn pow(self, exponent: f64) -> Self;
    fn powi(self, n: i32) -> Self;
    fn abs(self) -> Self;

    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn atan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;

    fn asinh(self) -> Self;
    fn erf(self) -> Self;
    fn sqrtx_asinh_sqrtx(self) -> Self;

    fn lt(&self, other: &Self) -> bool;
    fn gt(&self, other: &Self) -> bool;
}

// =============================================================================
// f64 implementation
// =============================================================================

impl Num for f64 {
    fn from_f64(val: f64) -> Self {
        val
    }

    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn value_f64(&self) -> f64 {
        *self
    }

    fn set_constant(&mut self, val: f64) {
        *self = val;
    }

    fn exp(self) -> Self {
        f64::exp(self)
    }

    fn log(self) -> Self {
        f64::ln(self)
    }

    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }

    fn cbrt(self) -> Self {
        f64::cbrt(self)
    }

    fn pow(self, exponent: f64) -> Self {
        f64::powf(self, exponent)
    }

    fn powi(self, n: i32) -> Self {
        f64::powi(self, n)
    }

    fn abs(self) -> Self {
        f64::abs(self)
    }

    fn sin(self) -> Self {
        f64::sin(self)
    }

    fn cos(self) -> Self {
        f64::cos(self)
    }

    fn atan(self) -> Self {
        f64::atan(self)
    }

    fn asin(self) -> Self {
        f64::asin(self)
    }

    fn acos(self) -> Self {
        f64::acos(self)
    }

    fn asinh(self) -> Self {
        f64::asinh(self)
    }

    /// Error function using Abramowitz & Stegun approximation (same as C++ xcfun).
    /// Maximum error: 1.5e-7.
    fn erf(self) -> Self {
        // erf(-x) = -erf(x)
        let sign = if self < 0.0 { -1.0 } else { 1.0 };
        let x = self.abs();

        let t = 1.0 / (1.0 + 0.3275911 * x);
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;

        let poly = a1 * t + a2 * t * t + a3 * t * t * t + a4 * t * t * t * t
            + a5 * t * t * t * t * t;
        let result = 1.0 - poly * f64::exp(-x * x);

        sign * result
    }

    /// Compute sqrt(x) * asinh(sqrt(x)), handling x < 0.
    fn sqrtx_asinh_sqrtx(self) -> Self {
        if self < 0.0 {
            // For negative x, use the identity:
            // sqrt(x)*asinh(sqrt(x)) for x<0 is sqrt(-x)*asin(sqrt(-x))
            // because asinh(ix) = i*asin(x)
            let mx = -self;
            let s = mx.sqrt();
            s * s.asin()
        } else {
            let s = self.sqrt();
            s * s.asinh()
        }
    }

    fn lt(&self, other: &Self) -> bool {
        *self < *other
    }

    fn gt(&self, other: &Self) -> bool {
        *self > *other
    }
}

// =============================================================================
// CTaylor<f64, N> implementation
// =============================================================================

impl<const N: usize> Num for CTaylor<f64, N> {
    fn from_f64(val: f64) -> Self {
        CTaylor::constant(val)
    }

    fn zero() -> Self {
        CTaylor {
            c: [0.0; N],
        }
    }

    fn one() -> Self {
        CTaylor::constant(1.0)
    }

    fn value_f64(&self) -> f64 {
        self.c[0]
    }

    fn set_constant(&mut self, val: f64) {
        self.c[0] = val;
    }

    fn exp(self) -> Self {
        math::ctaylor_exp(self)
    }

    fn log(self) -> Self {
        math::ctaylor_log(self)
    }

    fn sqrt(self) -> Self {
        math::ctaylor_sqrt(self)
    }

    fn cbrt(self) -> Self {
        math::ctaylor_cbrt(self)
    }

    fn pow(self, exponent: f64) -> Self {
        math::ctaylor_pow(self, exponent)
    }

    fn powi(self, n: i32) -> Self {
        math::ctaylor_powi(self, n)
    }

    fn abs(self) -> Self {
        math::ctaylor_abs(self)
    }

    fn sin(self) -> Self {
        math::ctaylor_sin(self)
    }

    fn cos(self) -> Self {
        math::ctaylor_cos(self)
    }

    fn atan(self) -> Self {
        math::ctaylor_atan(self)
    }

    fn asin(self) -> Self {
        math::ctaylor_asin(self)
    }

    fn acos(self) -> Self {
        math::ctaylor_acos(self)
    }

    fn asinh(self) -> Self {
        math::ctaylor_asinh(self)
    }

    fn erf(self) -> Self {
        math::ctaylor_erf(self)
    }

    fn sqrtx_asinh_sqrtx(self) -> Self {
        math::ctaylor_sqrtx_asinh_sqrtx(self)
    }

    fn lt(&self, other: &Self) -> bool {
        self.c[0] < other.c[0]
    }

    fn gt(&self, other: &Self) -> bool {
        self.c[0] > other.c[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // =========================================================================
    // f64 Num tests
    // =========================================================================

    #[test]
    fn test_f64_from_f64() {
        assert_eq!(f64::from_f64(3.14), 3.14);
    }

    #[test]
    fn test_f64_zero() {
        assert_eq!(f64::zero(), 0.0);
    }

    #[test]
    fn test_f64_one() {
        assert_eq!(f64::one(), 1.0);
    }

    #[test]
    fn test_f64_exp() {
        assert_relative_eq!(Num::exp(1.0_f64), 1.0_f64.exp(), epsilon = 1e-15);
    }

    #[test]
    fn test_f64_log() {
        assert_relative_eq!(Num::log(2.0_f64), 2.0_f64.ln(), epsilon = 1e-15);
    }

    #[test]
    fn test_f64_sqrt() {
        assert_relative_eq!(Num::sqrt(4.0_f64), 2.0, epsilon = 1e-15);
    }

    #[test]
    fn test_f64_pow() {
        assert_relative_eq!(Num::pow(2.0_f64, 3.0), 8.0, epsilon = 1e-15);
    }

    #[test]
    fn test_f64_lt() {
        assert!(Num::lt(&1.0_f64, &2.0));
        assert!(!Num::lt(&2.0_f64, &1.0));
    }

    #[test]
    fn test_f64_gt() {
        assert!(Num::gt(&2.0_f64, &1.0));
        assert!(!Num::gt(&1.0_f64, &2.0));
    }

    #[test]
    fn test_f64_value_f64() {
        assert_eq!(Num::value_f64(&3.14_f64), 3.14);
    }

    #[test]
    fn test_f64_set_constant() {
        let mut x = 1.0_f64;
        Num::set_constant(&mut x, 5.0);
        assert_eq!(x, 5.0);
    }

    #[test]
    fn test_f64_erf() {
        // erf(0) = 0
        assert_relative_eq!(Num::erf(0.0_f64), 0.0, epsilon = 1e-7);
        // erf(1) ~ 0.8427007929
        assert_relative_eq!(Num::erf(1.0_f64), 0.8427007929, epsilon = 1e-6);
        // erf(-1) = -erf(1)
        assert_relative_eq!(Num::erf(-1.0_f64), -0.8427007929, epsilon = 1e-6);
    }

    #[test]
    fn test_f64_sqrtx_asinh_sqrtx() {
        // sqrt(4) * asinh(sqrt(4)) = 2 * asinh(2) = 2 * 1.4436... = 2.8872...
        let result = Num::sqrtx_asinh_sqrtx(4.0_f64);
        let expected = 2.0 * 2.0_f64.asinh();
        assert_relative_eq!(result, expected, epsilon = 1e-12);
    }

    #[test]
    fn test_f64_trig() {
        assert_relative_eq!(Num::sin(0.5_f64), 0.5_f64.sin(), epsilon = 1e-15);
        assert_relative_eq!(Num::cos(0.5_f64), 0.5_f64.cos(), epsilon = 1e-15);
        assert_relative_eq!(Num::atan(0.5_f64), 0.5_f64.atan(), epsilon = 1e-15);
    }

    #[test]
    fn test_f64_cbrt() {
        assert_relative_eq!(Num::cbrt(27.0_f64), 3.0, epsilon = 1e-15);
    }

    #[test]
    fn test_f64_powi() {
        assert_relative_eq!(Num::powi(3.0_f64, 4), 81.0, epsilon = 1e-12);
    }

    #[test]
    fn test_f64_abs() {
        assert_eq!(Num::abs(-5.0_f64), 5.0);
        assert_eq!(Num::abs(5.0_f64), 5.0);
    }

    #[test]
    fn test_f64_asinh() {
        assert_relative_eq!(Num::asinh(1.0_f64), 1.0_f64.asinh(), epsilon = 1e-15);
    }

    // =========================================================================
    // CTaylor Num tests
    // =========================================================================

    #[test]
    fn test_ctaylor_from_f64() {
        let t = CTaylor::<f64, 4>::from_f64(3.14);
        assert_eq!(t.c[0], 3.14);
        assert_eq!(t.c[1], 0.0);
        assert_eq!(t.c[2], 0.0);
        assert_eq!(t.c[3], 0.0);
    }

    #[test]
    fn test_ctaylor_zero() {
        let t = CTaylor::<f64, 4>::zero();
        for i in 0..4 {
            assert_eq!(t.c[i], 0.0, "c[{i}] should be 0.0");
        }
    }

    #[test]
    fn test_ctaylor_one() {
        let t = CTaylor::<f64, 4>::one();
        assert_eq!(t.c[0], 1.0);
        assert_eq!(t.c[1], 0.0);
        assert_eq!(t.c[2], 0.0);
        assert_eq!(t.c[3], 0.0);
    }

    #[test]
    fn test_ctaylor_lt() {
        let a = CTaylor::<f64, 4>::constant(1.0);
        let b = CTaylor::<f64, 4>::constant(2.0);
        assert!(Num::lt(&a, &b));
        assert!(!Num::lt(&b, &a));
    }

    #[test]
    fn test_ctaylor_gt() {
        let a = CTaylor::<f64, 4>::constant(2.0);
        let b = CTaylor::<f64, 4>::constant(1.0);
        assert!(Num::gt(&a, &b));
        assert!(!Num::gt(&b, &a));
    }

    #[test]
    fn test_ctaylor_value_f64() {
        // variable(2.0, 0) should have value_f64() == 2.0
        let t = CTaylor::<f64, 4>::variable(2.0, 0);
        assert_eq!(Num::value_f64(&t), 2.0);
    }

    #[test]
    fn test_ctaylor_set_constant_preserves_derivatives() {
        // variable(2.0, 0) has c = [2.0, 1.0, 0.0, 0.0]
        // set_constant(5.0) should give c = [5.0, 1.0, 0.0, 0.0]
        let mut t = CTaylor::<f64, 4>::variable(2.0, 0);
        Num::set_constant(&mut t, 5.0);
        assert_eq!(t.c[0], 5.0);
        assert_eq!(t.c[1], 1.0); // derivative preserved!
        assert_eq!(t.c[2], 0.0);
        assert_eq!(t.c[3], 0.0);
    }

    #[test]
    fn test_ctaylor_lt_compares_c0_only() {
        // Create two CTaylor with same c[0] but different derivatives
        let mut a = CTaylor::<f64, 4>::variable(1.0, 0); // [1, 1, 0, 0]
        let b = CTaylor::<f64, 4>::constant(1.0); // [1, 0, 0, 0]
        // Same c[0], so neither lt nor gt
        assert!(!Num::lt(&a, &b));
        assert!(!Num::gt(&a, &b));
        // Change a's c[0] to be less
        a.c[0] = 0.5;
        assert!(Num::lt(&a, &b));
    }
}
