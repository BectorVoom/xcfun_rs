//! CTaylor transcendental function implementations.
//!
//! Each function follows the pattern: generate Taylor coefficients via tmath,
//! then apply compose() to evaluate the function on the CTaylor polynomial.
//!
//! Direct port of C++ `ctaylor_math.hpp` from xcfun.

use crate::compose;
use crate::ctaylor::CTaylor;
use crate::tmath;

/// Helper: compute number of variables from array size (log2).
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
        _ => panic!("Invalid CTaylor size"),
    }
}

/// exp(x) for CTaylor
pub fn ctaylor_exp<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::exp_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// ln(x) for CTaylor
pub fn ctaylor_log<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::log_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// x^a for CTaylor (fractional/real exponent)
pub fn ctaylor_pow<const N: usize>(x: CTaylor<f64, N>, a: f64) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::pow_expand(&mut coeffs, x.c[0], a);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// sqrt(x) for CTaylor
pub fn ctaylor_sqrt<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::sqrt_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// cbrt(x) for CTaylor
pub fn ctaylor_cbrt<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::cbrt_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// abs(x) for CTaylor: if c[0] >= 0, return x; else negate all coefficients
pub fn ctaylor_abs<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    if x.c[0] < 0.0 {
        -x
    } else {
        x
    }
}

/// sin(x) for CTaylor
pub fn ctaylor_sin<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::sin_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// cos(x) for CTaylor
pub fn ctaylor_cos<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::cos_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// atan(x) for CTaylor
pub fn ctaylor_atan<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::atan_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// asin(x) for CTaylor
pub fn ctaylor_asin<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::asin_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// acos(x) for CTaylor
pub fn ctaylor_acos<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::acos_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// asinh(x) for CTaylor
pub fn ctaylor_asinh<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::asinh_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// erf(x) for CTaylor
pub fn ctaylor_erf<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);
    let mut coeffs = vec![0.0; nvar + 1];
    tmath::erf_expand(&mut coeffs, x.c[0]);
    let mut result = [0.0; N];
    compose::compose(&mut result, &x.c, &coeffs);
    CTaylor { c: result }
}

/// sqrt(x)*asinh(sqrt(x)) for CTaylor.
///
/// Uses Pade approximant near x=0 per C++ implementation, with the [8,8] Pade
/// coefficients for |x0| < 0.5, and direct sqrt*asinh for larger values.
pub fn ctaylor_sqrtx_asinh_sqrtx<const N: usize>(x: CTaylor<f64, N>) -> CTaylor<f64, N> {
    let nvar = nvar_from_size(N);

    if x.c[0].abs() < 0.5 {
        // Pade [8,8] approximation from C++ ctaylor_math.hpp
        #[allow(clippy::excessive_precision)]
        const P: [f64; 9] = [
            0.0,
            3.510921856028398e3,
            1.23624388373212e4,
            1.734847003883674e4,
            1.235072285222234e4,
            4.691117148130619e3,
            9.119186273274577e2,
            7.815848629220836e1,
            1.96088643023654e0,
        ];
        #[allow(clippy::excessive_precision)]
        const Q: [f64; 9] = [
            3.510921856028398e3,
            1.29475924799926e4,
            1.924308297963337e4,
            1.474357149568687e4,
            6.176496729255528e3,
            1.379806958043824e3,
            1.471833349002349e2,
            5.666278232986776e0,
            2.865104054302032e-2,
        ];

        // Evaluate P(x0)/Q(x0) as a 1D Taylor polynomial, then compose with CTaylor
        // 1. Shift P and Q polynomials to be centered at x0
        // 2. Compute inv(Q_shifted) * P_shifted as 1D Taylor coefficients
        // 3. Compose result with CTaylor

        let x0 = x.c[0];
        let ntaylor = nvar + 1;

        // Evaluate shifted Q polynomial at x0 using Horner
        let mut q_shifted = vec![0.0; ntaylor.min(9)];
        let mut p_shifted = vec![0.0; ntaylor.min(9)];

        // Compute Taylor coefficients of P(x0+h) and Q(x0+h)
        // For polynomial P(z) = sum_k P[k]*z^k, the Taylor expansion at x0 is:
        // P(x0+h) = sum_{j=0}^{deg} t_j * h^j where t_j = P^(j)(x0)/j!
        // t_j = sum_{k=j}^{deg} P[k] * C(k,j) * x0^(k-j)
        let pdeg = 8;
        for j in 0..ntaylor.min(9) {
            let mut sum_p = 0.0;
            let mut sum_q = 0.0;
            for k in j..=pdeg {
                let binom = tmath_binom(k, j);
                let x0_pow = x0.powi((k - j) as i32);
                sum_p += P[k] * binom * x0_pow;
                sum_q += Q[k] * binom * x0_pow;
            }
            p_shifted[j] = sum_p;
            q_shifted[j] = sum_q;
        }

        // Compute P/Q as 1D Taylor: inv(Q) * P
        // First compute inv expansion of Q[0] (the constant term of shifted Q)
        let mut inv_q = vec![0.0; ntaylor];
        tmath::inv_expand(&mut inv_q, q_shifted[0]);

        // Compose inv_q with shifted Q polynomial (deviation from constant)
        let mut q_comp = vec![0.0; ntaylor];
        q_comp[0] = 0.0; // deviation from constant
        for i in 1..ntaylor.min(q_shifted.len()) {
            q_comp[i] = q_shifted[i];
        }
        tmath::taylor1d_compose(&mut inv_q, &q_comp);

        // Multiply by P_shifted
        let mut pq = vec![0.0; ntaylor];
        for i in 0..ntaylor {
            for j in 0..=i {
                if j < inv_q.len() && (i - j) < p_shifted.len() {
                    pq[i] += inv_q[j] * p_shifted[i - j];
                }
            }
        }

        // Now compose pq (1D Taylor coefficients) with CTaylor
        let mut result = [0.0; N];
        compose::compose(&mut result, &x.c, &pq);
        CTaylor { c: result }
    } else {
        // Direct: sqrt(x) * asinh(sqrt(x))
        let s = ctaylor_sqrt(x);
        let a = ctaylor_asinh(s.clone());
        s * a
    }
}

/// x^n for CTaylor (integer exponent).
///
/// For n >= 0: repeated multiplication (works at x=0).
/// For n < 0: use pow_expand with a = n as f64.
pub fn ctaylor_powi<const N: usize>(x: CTaylor<f64, N>, n: i32) -> CTaylor<f64, N> {
    if n > 0 {
        let mut result = x.clone();
        for _ in 1..n {
            result = result * x.clone();
        }
        result
    } else if n == 0 {
        CTaylor::constant(1.0)
    } else {
        // Negative integer: use pow_expand
        ctaylor_pow(x, n as f64)
    }
}

/// Binomial coefficient C(n, k) for Pade computation
fn tmath_binom(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let mut result = 1.0;
    for i in 0..k {
        result *= (n - i) as f64;
        result /= (i + 1) as f64;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ctaylor::{CNST, VAR0, VAR1};
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    // =========================================================================
    // Single-variable derivative tests
    // =========================================================================

    #[test]
    fn test_ctaylor_exp_at_1() {
        // exp(x) at x=1, 1 variable (N=2)
        let x = CTaylor::<f64, 2>::variable(1.0, 0);
        let r = ctaylor_exp(x);
        let e = 1.0_f64.exp();
        assert_relative_eq!(r.c[CNST], e, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], e, epsilon = 1e-12); // d/dx exp(x) = exp(x)
    }

    #[test]
    fn test_ctaylor_exp_all_derivatives_equal_e() {
        // For exp(x) at x=1, all mixed partials through a single variable
        // are just e (since d^k/dx^k exp(x) = exp(x))
        // With N=4 (2 vars), seed var0 only:
        let x = CTaylor::<f64, 4>::variable(1.0, 0);
        let r = ctaylor_exp(x);
        let e = 1.0_f64.exp();
        assert_relative_eq!(r.c[CNST], e, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], e, epsilon = 1e-12);
        // VAR1 and VAR0|VAR1 should be 0 since var1 wasn't seeded
        assert_relative_eq!(r.c[VAR1], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0 | VAR1], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_log_at_2() {
        let x = CTaylor::<f64, 2>::variable(2.0, 0);
        let r = ctaylor_log(x);
        assert_relative_eq!(r.c[CNST], 2.0_f64.ln(), epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 0.5, epsilon = 1e-12); // 1/x at x=2
    }

    #[test]
    fn test_ctaylor_sqrt_at_4() {
        let x = CTaylor::<f64, 2>::variable(4.0, 0);
        let r = ctaylor_sqrt(x);
        assert_relative_eq!(r.c[CNST], 2.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 0.25, epsilon = 1e-12); // 1/(2*sqrt(4))
    }

    #[test]
    fn test_ctaylor_pow_square() {
        // pow(x, 2.0) at x=3: f(3)=9, f'(3)=6
        let x = CTaylor::<f64, 2>::variable(3.0, 0);
        let r = ctaylor_pow(x, 2.0);
        assert_relative_eq!(r.c[CNST], 9.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 6.0, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_sin_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_sin(x);
        assert_relative_eq!(r.c[CNST], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0, epsilon = 1e-12); // cos(0)
    }

    #[test]
    fn test_ctaylor_cos_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_cos(x);
        assert_relative_eq!(r.c[CNST], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 0.0, epsilon = 1e-12); // -sin(0)
    }

    #[test]
    fn test_ctaylor_atan_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_atan(x);
        assert_relative_eq!(r.c[CNST], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0, epsilon = 1e-12); // 1/(1+0^2)
    }

    #[test]
    fn test_ctaylor_erf_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_erf(x);
        assert_relative_eq!(r.c[CNST], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 2.0 / PI.sqrt(), epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_asinh_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_asinh(x);
        assert_relative_eq!(r.c[CNST], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0, epsilon = 1e-12); // 1/sqrt(1+0)
    }

    #[test]
    fn test_ctaylor_asin_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_asin(x);
        assert_relative_eq!(r.c[CNST], 0.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_acos_at_0() {
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let r = ctaylor_acos(x);
        assert_relative_eq!(r.c[CNST], std::f64::consts::FRAC_PI_2, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], -1.0, epsilon = 1e-12);
    }

    // =========================================================================
    // Second derivative tests (N=4, 2 variables, seed var0 only)
    // =========================================================================

    #[test]
    fn test_ctaylor_log_second_deriv() {
        // d^2/dx^2 ln(x) = -1/x^2. At x=2: -0.25
        // With 2 vars, seed var0 only, the VAR0|VAR1 slot captures d^2/dxdx
        // when we seed both vars as the SAME variable.
        // Actually with N=4 (2 vars), if we seed only var0,
        // c[VAR0|VAR1] = 0 since var1 isn't active.
        // To get second derivative, we need to seed both var0 and var1 at x=2.
        let mut x = CTaylor::<f64, 4> { c: [0.0; 4] };
        x.c[0] = 2.0;   // value
        x.c[VAR0] = 1.0; // dx
        x.c[VAR1] = 1.0; // dx (same variable seeded twice)
        let r = ctaylor_log(x);
        assert_relative_eq!(r.c[CNST], 2.0_f64.ln(), epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 0.5, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR1], 0.5, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0 | VAR1], -0.25, epsilon = 1e-12); // d2/dx2
    }

    #[test]
    fn test_ctaylor_sqrt_second_deriv() {
        // d/dx sqrt(x) = 1/(2*sqrt(x)), d2/dx2 = -1/(4*x^(3/2))
        // At x=4: d2/dx2 = -1/(4*8) = -0.03125
        let mut x = CTaylor::<f64, 4> { c: [0.0; 4] };
        x.c[0] = 4.0;
        x.c[VAR0] = 1.0;
        x.c[VAR1] = 1.0;
        let r = ctaylor_sqrt(x);
        assert_relative_eq!(r.c[VAR0 | VAR1], -0.03125, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_cos_second_deriv() {
        // d2/dx2 cos(x) = -cos(x). At x=0: -1
        let mut x = CTaylor::<f64, 4> { c: [0.0; 4] };
        x.c[0] = 0.0;
        x.c[VAR0] = 1.0;
        x.c[VAR1] = 1.0;
        let r = ctaylor_cos(x);
        assert_relative_eq!(r.c[VAR0 | VAR1], -1.0, epsilon = 1e-12);
    }

    // =========================================================================
    // Composition / chain rule tests
    // =========================================================================

    #[test]
    fn test_chain_rule_exp_2x() {
        // d/dx exp(2x) = 2*exp(2x). At x=0: 2
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let two_x = x * 2.0;
        let r = ctaylor_exp(two_x);
        assert_relative_eq!(r.c[CNST], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 2.0, epsilon = 1e-12);
    }

    #[test]
    fn test_chain_rule_sin_exp() {
        // d/dx sin(exp(x)) = cos(exp(x)) * exp(x)
        // At x=0: cos(1) * 1 = cos(1)
        let x = CTaylor::<f64, 2>::variable(0.0, 0);
        let ex = ctaylor_exp(x);
        let r = ctaylor_sin(ex);
        assert_relative_eq!(r.c[CNST], 1.0_f64.sin(), epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0_f64.cos(), epsilon = 1e-10);
    }

    #[test]
    fn test_chain_rule_exp_x_squared() {
        // d/dx exp(x^2) = 2x * exp(x^2). At x=1: 2*e
        let x = CTaylor::<f64, 4>::variable(1.0, 0);
        let y = CTaylor::<f64, 4>::variable(1.0, 1); // same value, different var
        // x^2 simulated as x*y where both seeded at same point
        let xy = x * y;
        let r = ctaylor_exp(xy);
        let e = 1.0_f64.exp();
        assert_relative_eq!(r.c[CNST], e, epsilon = 1e-12);
        // d/dx = e * y_val = e, d/dy = e * x_val = e
        assert_relative_eq!(r.c[VAR0], e, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR1], e, epsilon = 1e-12);
    }

    // =========================================================================
    // Mixed partial derivative tests
    // =========================================================================

    #[test]
    fn test_mixed_partial_xy() {
        // f(x,y) = x*y: d2/(dx dy) = 1
        let x = CTaylor::<f64, 4>::variable(2.0, 0);
        let y = CTaylor::<f64, 4>::variable(3.0, 1);
        let r = x * y;
        assert_relative_eq!(r.c[CNST], 6.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 3.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR1], 2.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0 | VAR1], 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_mixed_partial_exp_x_plus_y() {
        // f(x,y) = exp(x+y), d2/(dx dy) = exp(x+y)
        // At x=0, y=0: d2/(dx dy) = 1
        let x = CTaylor::<f64, 4>::variable(0.0, 0);
        let y = CTaylor::<f64, 4>::variable(0.0, 1);
        let r = ctaylor_exp(x + y);
        assert_relative_eq!(r.c[CNST], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0 | VAR1], 1.0, epsilon = 1e-12);
    }

    // =========================================================================
    // abs tests
    // =========================================================================

    #[test]
    fn test_ctaylor_abs_positive() {
        let x = CTaylor::<f64, 2>::variable(3.0, 0);
        let r = ctaylor_abs(x.clone());
        assert_relative_eq!(r.c[0], x.c[0], epsilon = 1e-12);
        assert_relative_eq!(r.c[1], x.c[1], epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_abs_negative() {
        let x = CTaylor::<f64, 2>::variable(-3.0, 0);
        let r = ctaylor_abs(x);
        assert_relative_eq!(r.c[0], 3.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[1], -1.0, epsilon = 1e-12);
    }

    // =========================================================================
    // powi tests
    // =========================================================================

    #[test]
    fn test_ctaylor_powi_3() {
        // x^3 at x=2: f(2)=8, f'(2)=12
        let x = CTaylor::<f64, 2>::variable(2.0, 0);
        let r = ctaylor_powi(x, 3);
        assert_relative_eq!(r.c[CNST], 8.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 12.0, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_powi_0() {
        let x = CTaylor::<f64, 2>::variable(5.0, 0);
        let r = ctaylor_powi(x, 0);
        assert_relative_eq!(r.c[CNST], 1.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_ctaylor_powi_matches_mul() {
        // x^3 via powi should match x*x*x
        let x = CTaylor::<f64, 4>::variable(2.0, 0);
        let r_powi = ctaylor_powi(x.clone(), 3);
        let r_mul = x.clone() * x.clone() * x;
        for i in 0..4 {
            assert_relative_eq!(r_powi.c[i], r_mul.c[i], epsilon = 1e-12);
        }
    }

    // =========================================================================
    // sqrtx_asinh_sqrtx tests
    // =========================================================================

    #[test]
    fn test_ctaylor_sqrtx_asinh_sqrtx_at_1() {
        let x = CTaylor::<f64, 2>::variable(1.0, 0);
        let r = ctaylor_sqrtx_asinh_sqrtx(x);
        let expected = 1.0_f64.sqrt() * 1.0_f64.sqrt().asinh();
        assert_relative_eq!(r.c[CNST], expected, epsilon = 1e-10);
    }

    #[test]
    fn test_ctaylor_sqrtx_asinh_sqrtx_near_zero() {
        // Should use Pade and not NaN
        let x = CTaylor::<f64, 2>::variable(0.1, 0);
        let r = ctaylor_sqrtx_asinh_sqrtx(x);
        assert!(r.c[CNST].is_finite());
        assert!(r.c[VAR0].is_finite());
        let expected = 0.1_f64.sqrt() * 0.1_f64.sqrt().asinh();
        assert_relative_eq!(r.c[CNST], expected, epsilon = 1e-10);
    }

    // =========================================================================
    // N=0 (energy only) tests
    // =========================================================================

    #[test]
    fn test_n0_exp() {
        let x = CTaylor::<f64, 1>::constant(1.0);
        let r = ctaylor_exp(x);
        assert_relative_eq!(r.c[0], 1.0_f64.exp(), epsilon = 1e-12);
    }

    #[test]
    fn test_n0_log() {
        let x = CTaylor::<f64, 1>::constant(2.0);
        let r = ctaylor_log(x);
        assert_relative_eq!(r.c[0], 2.0_f64.ln(), epsilon = 1e-12);
    }

    #[test]
    fn test_n0_sin() {
        let x = CTaylor::<f64, 1>::constant(0.5);
        let r = ctaylor_sin(x);
        assert_relative_eq!(r.c[0], 0.5_f64.sin(), epsilon = 1e-12);
    }

    // =========================================================================
    // N=7 (maximum, 128 coefficients) tests
    // =========================================================================

    #[test]
    fn test_n7_exp_compiles_and_runs() {
        let x = CTaylor::<f64, 128>::variable(1.0, 0);
        let r = ctaylor_exp(x);
        assert_relative_eq!(r.c[0], 1.0_f64.exp(), epsilon = 1e-12);
    }

    // =========================================================================
    // Stability tests
    // =========================================================================

    #[test]
    fn stability_exp_large() {
        // exp(500) is large but finite
        let x = CTaylor::<f64, 2>::variable(500.0, 0);
        let r = ctaylor_exp(x);
        assert!(r.c[0].is_finite());
        assert!(r.c[1].is_finite());
    }

    #[test]
    fn stability_log_small() {
        let x = CTaylor::<f64, 2>::variable(1e-14, 0);
        let r = ctaylor_log(x);
        assert!(r.c[0].is_finite());
        assert!(r.c[1].is_finite());
    }

    #[test]
    fn stability_sqrt_small() {
        let x = CTaylor::<f64, 2>::variable(1e-300, 0);
        let r = ctaylor_sqrt(x);
        assert!(r.c[0].is_finite());
        // derivative 1/(2*sqrt(1e-300)) is huge but finite
        assert!(r.c[1].is_finite());
    }

    #[test]
    fn stability_pow_fractional_near_zero() {
        let x = CTaylor::<f64, 2>::variable(1e-10, 0);
        let r = ctaylor_pow(x, 4.0 / 3.0);
        assert!(r.c[0].is_finite());
        assert!(r.c[1].is_finite());
    }

    #[test]
    fn stability_exp_700() {
        // exp(700) ~ 1e304, still finite
        let x = CTaylor::<f64, 2>::variable(700.0, 0);
        let r = ctaylor_exp(x);
        assert!(r.c[0].is_finite());
    }

    // =========================================================================
    // cbrt test
    // =========================================================================

    #[test]
    fn test_ctaylor_cbrt_at_8() {
        let x = CTaylor::<f64, 2>::variable(8.0, 0);
        let r = ctaylor_cbrt(x);
        assert_relative_eq!(r.c[CNST], 2.0, epsilon = 1e-12);
        assert_relative_eq!(r.c[VAR0], 1.0 / 12.0, epsilon = 1e-12);
    }
}
