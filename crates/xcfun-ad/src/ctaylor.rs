//! CTaylor<T, N> -- tensored first-order polynomial with N coefficients.
//!
//! This is the core automatic differentiation type. N must be a power of 2
//! (N = 2^nvar where nvar is the number of variables). Each coefficient
//! corresponds to a subset of variables (encoded as a bitmask index). The
//! recursive multiplication algorithm ensures multilinear behavior: terms
//! like x_i^2 are automatically dropped.
//!
//! Usage: `CTaylor::<f64, {1 << 3}>` for 3 variables (8 coefficients).
//!
//! Note: On stable Rust, const generic expressions like `[T; 1 << N]` are
//! not supported. Therefore N represents the array SIZE directly (must be a
//! power of 2), not the number of variables.

use crate::compose;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Bitmask constants for coefficient indexing.
pub const CNST: usize = 0;
pub const VAR0: usize = 1;
pub const VAR1: usize = 2;
pub const VAR2: usize = 4;
pub const VAR3: usize = 8;
pub const VAR4: usize = 16;
pub const VAR5: usize = 32;
pub const VAR6: usize = 64;
pub const VAR7: usize = 128;

/// Compute the number of variables from the size (log2).
const fn nvar_from_size(size: usize) -> usize {
    match size {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        64 => 6,
        128 => 7,
        256 => 8,
        _ => panic!("CTaylor size must be a power of 2 between 1 and 256"),
    }
}

/// Tensored first-order polynomial with `N` coefficients.
///
/// `N` must be a power of 2: 1, 2, 4, 8, 16, 32, 64, 128, or 256.
/// The number of variables is `log2(N)`.
///
/// Coefficients are stored with the first half not depending on the last
/// variable, while the second half does. This is repeated recursively.
/// For example with 3 vars (N=8): `1 x y xy z zx zy zxy`
#[derive(Clone, Debug)]
pub struct CTaylor<T, const N: usize> {
    pub c: [T; N],
}

impl<const N: usize> CTaylor<f64, N> {
    /// The number of coefficients.
    pub const SIZE: usize = N;

    /// The number of variables (log2(N)).
    pub const NVAR: usize = nvar_from_size(N);

    /// Create a constant CTaylor (only c[0] is set, rest are zero).
    pub fn constant(value: f64) -> Self {
        let c = std::array::from_fn(|i| if i == 0 { value } else { 0.0 });
        CTaylor { c }
    }

    /// Create a variable CTaylor with unit first derivative.
    ///
    /// `value` is the point at which to evaluate, `var` is the variable index.
    /// Sets c[0] = value, c[1 << var] = 1.0, all others = 0.0.
    pub fn variable(value: f64, var: usize) -> Self {
        debug_assert!(
            var < Self::NVAR,
            "variable index {var} must be < NVAR={}",
            Self::NVAR
        );
        let mut c = [0.0; N];
        c[0] = value;
        c[1 << var] = 1.0;
        CTaylor { c }
    }

    /// Create a variable CTaylor with a custom derivative value.
    pub fn variable_with_deriv(value: f64, var: usize, deriv: f64) -> Self {
        debug_assert!(
            var < Self::NVAR,
            "variable index {var} must be < NVAR={}",
            Self::NVAR
        );
        let mut c = [0.0; N];
        c[0] = value;
        c[1 << var] = deriv;
        CTaylor { c }
    }

    /// Get the constant term (function value).
    pub fn value(&self) -> &f64 {
        &self.c[0]
    }

    /// Get a coefficient by index.
    pub fn get(&self, index: usize) -> &f64 {
        &self.c[index]
    }
}

// --- Element-wise Add ---

impl<const N: usize> Add for CTaylor<f64, N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        CTaylor {
            c: std::array::from_fn(|i| self.c[i] + rhs.c[i]),
        }
    }
}

impl<const N: usize> AddAssign for CTaylor<f64, N> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.c[i] += rhs.c[i];
        }
    }
}

// --- Element-wise Sub ---

impl<const N: usize> Sub for CTaylor<f64, N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        CTaylor {
            c: std::array::from_fn(|i| self.c[i] - rhs.c[i]),
        }
    }
}

impl<const N: usize> SubAssign for CTaylor<f64, N> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.c[i] -= rhs.c[i];
        }
    }
}

// --- Neg ---

impl<const N: usize> Neg for CTaylor<f64, N> {
    type Output = Self;
    fn neg(self) -> Self {
        CTaylor {
            c: std::array::from_fn(|i| -self.c[i]),
        }
    }
}

// --- Mul (using recursive algorithm) ---

impl<const N: usize> Mul for CTaylor<f64, N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut result = [0.0; N];
        compose::mul_set_recursive(&mut result, &self.c, &rhs.c);
        CTaylor { c: result }
    }
}

impl<const N: usize> MulAssign for CTaylor<f64, N> {
    fn mul_assign(&mut self, rhs: Self) {
        let mut result = [0.0; N];
        compose::mul_set_recursive(&mut result, &self.c, &rhs.c);
        self.c = result;
    }
}

// --- Div (via compose with inverse Taylor coefficients) ---

impl<const N: usize> Div for CTaylor<f64, N> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        // Compute 1/rhs via compose, then multiply by self.
        // Inverse Taylor coefficients: t[0] = 1/q0, t[k] = -t[k-1] / q0
        let q0 = rhs.c[0];
        debug_assert!(q0.abs() > 0.0, "division by zero: CTaylor divisor has c[0] == 0");

        let nvar = Self::NVAR;
        let ncoeffs = nvar + 1;
        let mut inv_coeffs = vec![0.0; ncoeffs];
        inv_coeffs[0] = 1.0 / q0;
        for k in 1..ncoeffs {
            inv_coeffs[k] = -inv_coeffs[k - 1] / q0;
        }

        // compose: result = sum_k inv_coeffs[k] * (rhs - rhs[0])^k
        let mut inv_result = [0.0; N];
        compose::compose(&mut inv_result, &rhs.c, &inv_coeffs);

        // Multiply self * (1/rhs)
        let mut result = [0.0; N];
        compose::mul_set_recursive(&mut result, &self.c, &inv_result);
        CTaylor { c: result }
    }
}

impl<const N: usize> DivAssign for CTaylor<f64, N> {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone() / rhs;
    }
}

// --- Scalar operations ---

// f64 * CTaylor
impl<const N: usize> Mul<CTaylor<f64, N>> for f64 {
    type Output = CTaylor<f64, N>;
    fn mul(self, rhs: CTaylor<f64, N>) -> CTaylor<f64, N> {
        CTaylor {
            c: std::array::from_fn(|i| self * rhs.c[i]),
        }
    }
}

// CTaylor * f64
impl<const N: usize> Mul<f64> for CTaylor<f64, N> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        CTaylor {
            c: std::array::from_fn(|i| self.c[i] * rhs),
        }
    }
}

// CTaylor + f64
impl<const N: usize> Add<f64> for CTaylor<f64, N> {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        let mut result = self;
        result.c[0] += rhs;
        result
    }
}

// f64 + CTaylor
impl<const N: usize> Add<CTaylor<f64, N>> for f64 {
    type Output = CTaylor<f64, N>;
    fn add(self, rhs: CTaylor<f64, N>) -> CTaylor<f64, N> {
        let mut result = rhs;
        result.c[0] += self;
        result
    }
}

// CTaylor - f64
impl<const N: usize> Sub<f64> for CTaylor<f64, N> {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        let mut result = self;
        result.c[0] -= rhs;
        result
    }
}

// f64 - CTaylor
impl<const N: usize> Sub<CTaylor<f64, N>> for f64 {
    type Output = CTaylor<f64, N>;
    fn sub(self, rhs: CTaylor<f64, N>) -> CTaylor<f64, N> {
        let mut result = -rhs;
        result.c[0] += self;
        result
    }
}

// CTaylor / f64
impl<const N: usize> Div<f64> for CTaylor<f64, N> {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        CTaylor {
            c: std::array::from_fn(|i| self.c[i] / rhs),
        }
    }
}

// f64 / CTaylor
impl<const N: usize> Div<CTaylor<f64, N>> for f64 {
    type Output = CTaylor<f64, N>;
    fn div(self, rhs: CTaylor<f64, N>) -> CTaylor<f64, N> {
        CTaylor::<f64, N>::constant(self) / rhs
    }
}

// MulAssign<f64>
impl<const N: usize> MulAssign<f64> for CTaylor<f64, N> {
    fn mul_assign(&mut self, rhs: f64) {
        for i in 0..N {
            self.c[i] *= rhs;
        }
    }
}

// AddAssign<f64>
impl<const N: usize> AddAssign<f64> for CTaylor<f64, N> {
    fn add_assign(&mut self, rhs: f64) {
        self.c[0] += rhs;
    }
}

// SubAssign<f64>
impl<const N: usize> SubAssign<f64> for CTaylor<f64, N> {
    fn sub_assign(&mut self, rhs: f64) {
        self.c[0] -= rhs;
    }
}

// DivAssign<f64>
impl<const N: usize> DivAssign<f64> for CTaylor<f64, N> {
    fn div_assign(&mut self, rhs: f64) {
        for i in 0..N {
            self.c[i] /= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // N=1 means 1 coefficient (0 variables)
    // N=4 means 4 coefficients (2 variables)
    // N=8 means 8 coefficients (3 variables)

    #[test]
    fn test_constant_n0() {
        let t = CTaylor::<f64, 1>::constant(3.0);
        assert_eq!(*t.value(), 3.0);
    }

    #[test]
    fn test_variable_n3() {
        // 3 variables -> N = 8
        let t = CTaylor::<f64, 8>::variable(2.0, 0);
        assert_eq!(t.c[0], 2.0);
        assert_eq!(t.c[1], 1.0); // VAR0 = 1 << 0 = 1
        assert_eq!(t.c[2], 0.0); // other vars zero
        assert_eq!(t.c[3], 0.0);
        assert_eq!(t.c[4], 0.0);
    }

    #[test]
    fn test_variable_other_coefficients_zero() {
        let t = CTaylor::<f64, 8>::variable(2.0, 0);
        // c[1] = 1.0 (VAR0), all others from c[2..] should be 0
        for i in 2..8 {
            assert_eq!(t.c[i], 0.0, "c[{i}] should be 0.0");
        }
    }

    #[test]
    fn test_addition_elementwise() {
        // 2 variables -> N = 4
        let a = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let b = CTaylor::<f64, 4> {
            c: [10.0, 20.0, 30.0, 40.0],
        };
        let c = a + b;
        assert_eq!(c.c, [11.0, 22.0, 33.0, 44.0]);
    }

    #[test]
    fn test_subtraction_elementwise() {
        let a = CTaylor::<f64, 4> {
            c: [10.0, 20.0, 30.0, 40.0],
        };
        let b = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let c = a - b;
        assert_eq!(c.c, [9.0, 18.0, 27.0, 36.0]);
    }

    #[test]
    fn test_multiplication_cross_terms() {
        // (1 + x) * (1 + y) for 2 vars (N=4)
        let a = CTaylor::<f64, 4>::variable(1.0, 0); // [1, 1, 0, 0]
        let b = CTaylor::<f64, 4>::variable(1.0, 1); // [1, 0, 1, 0]
        let c = a * b;
        assert_eq!(c.c[CNST], 1.0);
        assert_eq!(c.c[VAR0], 1.0);
        assert_eq!(c.c[VAR1], 1.0);
        assert_eq!(c.c[VAR0 | VAR1], 1.0);
    }

    #[test]
    fn test_multilinear_property() {
        // x * x: the multilinear property means no quadratic terms
        // For 2 vars (N=4), x = variable(1.0, 0) = [1, 1, 0, 0]
        // x * x = [1, 2, 0, 0]
        let x = CTaylor::<f64, 4>::variable(1.0, 0);
        let xx = x.clone() * x;
        assert_eq!(xx.c[CNST], 1.0);
        assert_eq!(xx.c[VAR0], 2.0);
        assert_eq!(xx.c[VAR1], 0.0);
        assert_eq!(xx.c[VAR0 | VAR1], 0.0);
    }

    #[test]
    fn test_division_by_constant() {
        // (2 + x) / (1) should give [2, 1, 0, 0]
        let a = CTaylor::<f64, 4>::variable(2.0, 0);
        let b = CTaylor::<f64, 4>::constant(1.0);
        let c = a / b;
        assert_relative_eq!(c.c[0], 2.0, epsilon = 1e-12);
        assert_relative_eq!(c.c[VAR0], 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_division_by_variable() {
        // 1 / (2 + x): f(x) = 1/(2+x), f(0) = 0.5, f'(0) = -1/4
        let a = CTaylor::<f64, 4>::constant(1.0);
        let b = CTaylor::<f64, 4>::variable(2.0, 0);
        let c = a / b;
        assert_relative_eq!(c.c[0], 0.5, epsilon = 1e-12);
        assert_relative_eq!(c.c[VAR0], -0.25, epsilon = 1e-12);
    }

    #[test]
    fn test_scalar_mul_ctaylor() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let r = 3.0 * t;
        assert_eq!(r.c, [3.0, 6.0, 9.0, 12.0]);
    }

    #[test]
    fn test_ctaylor_mul_scalar() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let r = t * 3.0;
        assert_eq!(r.c, [3.0, 6.0, 9.0, 12.0]);
    }

    #[test]
    fn test_ctaylor_add_scalar() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let r = t + 10.0;
        assert_eq!(r.c, [11.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_scalar_add_ctaylor() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let r = 10.0 + t;
        assert_eq!(r.c, [11.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_scalar_sub_ctaylor() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, 2.0, 3.0, 4.0],
        };
        let r = 10.0 - t;
        assert_eq!(r.c, [9.0, -2.0, -3.0, -4.0]);
    }

    #[test]
    fn test_ctaylor_sub_scalar() {
        let t = CTaylor::<f64, 4> {
            c: [10.0, 2.0, 3.0, 4.0],
        };
        let r = t - 1.0;
        assert_eq!(r.c, [9.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_neg() {
        let t = CTaylor::<f64, 4> {
            c: [1.0, -2.0, 3.0, -4.0],
        };
        let r = -t;
        assert_eq!(r.c, [-1.0, 2.0, -3.0, 4.0]);
    }

    #[test]
    fn test_variable_with_deriv() {
        let t = CTaylor::<f64, 4>::variable_with_deriv(3.0, 1, 2.5);
        assert_eq!(t.c[0], 3.0);
        assert_eq!(t.c[VAR1], 2.5);
        assert_eq!(t.c[VAR0], 0.0);
    }
}
