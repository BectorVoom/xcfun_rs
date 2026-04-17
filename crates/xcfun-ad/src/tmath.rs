//! Taylor expansion generators for transcendental functions.
//!
//! Each function fills a coefficient array `t[0..=degree]` with the Taylor
//! coefficients of the function expanded around point `x0`. These are 1D
//! expansions used by the compose-based CTaylor transcendental functions.
//!
//! Direct port of C++ `tmath.hpp` from xcfun.

use std::f64::consts::PI;

/// Taylor expansion of exp(x0 + h): t[k] = exp(x0) / k!
pub fn exp_expand(t: &mut [f64], x0: f64) {
    let mut ifac = 1.0_f64;
    t[0] = x0.exp();
    for i in 1..t.len() {
        ifac *= i as f64;
        t[i] = t[0] / ifac;
    }
}

/// Taylor expansion of ln(x0 + h)
/// t[0] = ln(x0), t[k] = (-1)^(k+1) / (k * x0^k)
pub fn log_expand(t: &mut [f64], x0: f64) {
    debug_assert!(x0 > 0.0, "log(x) not real analytic at x <= 0");
    t[0] = x0.ln();
    let x0inv = 1.0 / x0;
    let mut xn = x0inv;
    for i in 1..t.len() {
        // sign pattern: +, -, +, - ... => (-1)^(i+1) = 2*(i&1) - 1 as i64
        let sign = (2 * (i & 1) as i64 - 1) as f64;
        t[i] = xn / (i as f64) * sign;
        xn *= x0inv;
    }
}

/// Taylor expansion of (x0 + h)^a using falling factorial recurrence
pub fn pow_expand(t: &mut [f64], x0: f64, a: f64) {
    debug_assert!(x0 > 0.0, "pow(x,a) not real analytic at x <= 0");
    t[0] = x0.powf(a);
    let x0inv = 1.0 / x0;
    for i in 1..t.len() {
        t[i] = t[i - 1] * x0inv * (a - (i as f64 - 1.0)) / (i as f64);
    }
}

/// Taylor expansion of sqrt(x0 + h) -- optimized special case of pow(x0, 0.5)
pub fn sqrt_expand(t: &mut [f64], x0: f64) {
    debug_assert!(x0 > 0.0, "sqrt(x) not real analytic at x <= 0");
    t[0] = x0.sqrt();
    let x0inv = 1.0 / x0;
    for i in 1..t.len() {
        t[i] = t[i - 1] * ((3.0 * x0inv) / (2.0 * i as f64) - x0inv);
    }
}

/// Taylor expansion of cbrt(x0 + h) -- cube root
pub fn cbrt_expand(t: &mut [f64], x0: f64) {
    debug_assert!(x0 > 0.0, "cbrt(x) not real analytic at x <= 0");
    t[0] = x0.cbrt();
    let x0inv = 1.0 / x0;
    for i in 1..t.len() {
        t[i] = t[i - 1] * ((4.0 * x0inv) / (3.0 * i as f64) - x0inv);
    }
}

/// Taylor expansion of 1/(x0 + h)
pub fn inv_expand(t: &mut [f64], x0: f64) {
    debug_assert!(x0 != 0.0, "1/(a+x) not analytic at a = 0");
    t[0] = 1.0 / x0;
    for i in 1..t.len() {
        t[i] = -t[i - 1] * t[0];
    }
}

/// Taylor expansion of sin(x0 + h)
///
/// Follows C++ tmath.hpp pattern exactly.
pub fn sin_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;
    if ndeg > 0 {
        let s = x0.sin();
        let c = x0.cos();
        let mut fac = 1.0_f64;
        let mut i = 0;
        while 2 * i < ndeg {
            t[2 * i] = fac * s;
            fac /= (2 * i + 1) as f64;
            t[2 * i + 1] = fac * c;
            fac /= -((2 * i + 2) as f64);
            i += 1;
        }
        if ndeg % 2 == 0 {
            t[ndeg] = s * fac;
        }
    } else {
        t[0] = x0.sin();
    }
}

/// Taylor expansion of cos(x0 + h)
///
/// Follows C++ tmath.hpp pattern exactly.
pub fn cos_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;
    if ndeg > 0 {
        let s = x0.sin();
        let c = x0.cos();
        let mut fac = 1.0_f64;
        let mut i = 0;
        while 2 * i < ndeg {
            t[2 * i] = fac * c;
            fac /= -((2 * i + 1) as f64);
            t[2 * i + 1] = fac * s;
            fac /= (2 * i + 2) as f64;
            i += 1;
        }
        if ndeg % 2 == 0 {
            t[ndeg] = c * fac;
        }
    } else {
        t[0] = x0.cos();
    }
}

/// Taylor expansion of atan(x0 + h) via derivative expansion + integration.
///
/// d/dx atan(x) = 1/(1+x^2). Expand 1/(1+a^2+x), compose with 2*a*x + x^2,
/// integrate, set constant = atan(a).
pub fn atan_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    // Calculate taylor expansion of 1/(1+a^2+x)
    inv_expand(t, 1.0 + x0 * x0);

    // Build composition polynomial: x[0]=0, x[1]=2*a, x[2]=1, rest 0
    let mut x = vec![0.0; ndeg + 1];
    if ndeg > 0 {
        x[1] = 2.0 * x0;
    }
    if ndeg > 1 {
        x[2] = 1.0;
    }

    // Compose t with x
    taylor1d_compose(t, &x);

    // Integrate
    integrate(t);
    t[0] = x0.atan();
}

/// Gaussian expansion: Taylor expansion of exp(-(x0+h)^2).
///
/// Uses the factored approach from C++ tmath.hpp:
/// exp(-(a+x)^2) = exp(-a^2 - 2ax) * exp(-x^2)
fn gauss_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    // exp(-a^2) expanded and stretched by -2a
    exp_expand(t, -x0 * x0);
    taylor1d_stretch(t, -2.0 * x0);

    // exp(-x^2) as 1D Taylor: g[0]=1, g[odd]=0, g[2k]=(-1)^k / k!
    let mut g = vec![0.0; ndeg + 1];
    g[0] = 1.0;
    for i in (1..=ndeg).step_by(2) {
        g[i] = 0.0;
    }
    let mut i = 1;
    while 2 * i <= ndeg {
        g[2 * i] = -g[2 * (i - 1)] / (i as f64);
        i += 1;
    }

    // Multiply t by g
    taylor1d_multo(t, &g);
}

/// Taylor expansion of erf(x0 + h) via Gaussian expansion + integration.
///
/// d/dx erf(x) = 2/sqrt(pi) * exp(-x^2).
pub fn erf_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;
    gauss_expand(t, x0);
    let scale = 2.0 / PI.sqrt();
    for i in 0..=ndeg {
        t[i] *= scale;
    }
    integrate(t);
    t[0] = libm::erf(x0);
}

/// Taylor expansion of asinh(x0 + h) via derivative expansion + integration.
///
/// d/dx asinh(x) = 1/sqrt(1+x^2) = (1+x^2)^(-1/2).
pub fn asinh_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    // Build 1 + (a+x)^2 polynomial: tmp[0] = 1+a^2, tmp[1] = 2a, tmp[2] = 1
    let mut tmp = vec![0.0; ndeg + 1];
    tmp[0] = 1.0 + x0 * x0;
    if ndeg > 0 {
        tmp[1] = 2.0 * x0;
    }
    if ndeg > 1 {
        tmp[2] = 1.0;
    }

    // pow_expand of tmp[0] with a=-0.5
    pow_expand(t, tmp[0], -0.5);

    // Compose with the polynomial
    taylor1d_compose(t, &tmp);

    // Integrate and set constant
    integrate(t);
    t[0] = x0.asinh();
}

/// Taylor expansion of asin(x0 + h) via derivative expansion + integration.
///
/// d/dx asin(x) = 1/sqrt(1-x^2) = (1-x^2)^(-1/2).
pub fn asin_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    // Build 1 - (a+x)^2 polynomial: tmp[0] = 1-a^2, tmp[1] = -2a, tmp[2] = -1
    let mut tmp = vec![0.0; ndeg + 1];
    tmp[0] = 1.0 - x0 * x0;
    if ndeg > 0 {
        tmp[1] = -2.0 * x0;
    }
    if ndeg > 1 {
        tmp[2] = -1.0;
    }

    pow_expand(t, tmp[0], -0.5);
    taylor1d_compose(t, &tmp);
    integrate(t);
    t[0] = x0.asin();
}

/// Taylor expansion of acos(x0 + h).
///
/// d/dx acos(x) = -1/sqrt(1-x^2), so negate asin derivative coefficients.
pub fn acos_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    // Same derivative computation as asin
    let mut tmp = vec![0.0; ndeg + 1];
    tmp[0] = 1.0 - x0 * x0;
    if ndeg > 0 {
        tmp[1] = -2.0 * x0;
    }
    if ndeg > 1 {
        tmp[2] = -1.0;
    }

    pow_expand(t, tmp[0], -0.5);
    taylor1d_compose(t, &tmp);

    // Negate (acos' = -asin')
    for i in 0..=ndeg {
        t[i] = -t[i];
    }

    integrate(t);
    t[0] = x0.acos();
}

/// Taylor expansion of sqrt(x)*asinh(sqrt(x)) around x0.
/// Uses Pade approximant near x=0 for numerical stability.
///
/// For large x: compute directly from sqrt and asinh expansions.
/// For small x (< 1e-4): use Pade approximant to avoid 0/0 form.
pub fn sqrtx_asinh_sqrtx_expand(t: &mut [f64], x0: f64) {
    let ndeg = t.len() - 1;

    if x0 < 1e-4 {
        // Pade approximant for sqrt(x)*asinh(sqrt(x)) near x=0
        // Taylor series: x - x^2/6 + 3x^3/40 - 15x^4/336 + ...
        // Use f(x) = x * (1 - x/6 + 3x^2/40 - 5x^3/112 + ...)
        // which means f(x0+h) needs to be expanded.
        //
        // f(x) = sum_{n=0}^inf c_n * x^(n+1) where
        // c_0 = 1, c_1 = -1/6, c_2 = 3/40, c_3 = -5/112, c_4 = 35/1152, ...
        //
        // Actually, the function is g(x) = sqrt(x)*asinh(sqrt(x))
        // g(x) = x^(1/2) * asinh(x^(1/2))
        // For small x, asinh(u) = u - u^3/6 + 3u^5/40 - ...
        // g(x) = x^(1/2)*(x^(1/2) - x^(3/2)/6 + 3x^(5/2)/40 - ...)
        // g(x) = x - x^2/6 + 3x^3/40 - 5x^4/112 + 35x^5/1152 - ...
        //
        // We use the derivative expansion approach:
        // g'(x) = asinh(sqrt(x)) / (2*sqrt(x)) + 1/(2*(1+x))
        // For x near 0:
        // asinh(u)/u = 1 - u^2/6 + 3u^4/40 - ... so asinh(sqrt(x))/(2*sqrt(x)) = (1 - x/6 + 3x^2/40 - ...)/2
        // 1/(2*(1+x)) = (1 - x + x^2 - ...)/2
        // g'(x) = 1 - x/3 + ... (finite at 0)
        //
        // Use polynomial evaluation for the Pade coefficients.
        // Coefficients of g(x) = sum a_k * x^k:
        // a_0 = 0, a_1 = 1, a_2 = -1/6, a_3 = 3/40, a_4 = -5/112, a_5 = 35/1152, a_6 = -63/2816, a_7 = 231/13312
        let pade_coeffs: [f64; 8] = [
            0.0,
            1.0,
            -1.0 / 6.0,
            3.0 / 40.0,
            -5.0 / 112.0,
            35.0 / 1152.0,
            -63.0 / 2816.0,
            231.0 / 13312.0,
        ];

        // Compute Taylor expansion of g(x0 + h) using these polynomial coefficients
        // evaluated via Horner's method at x0 with derivatives
        // t[k] = (1/k!) * d^k/dx^k g(x) evaluated at x0 via polynomial chain
        //
        // Since g is a polynomial (truncated), its Taylor expansion around x0 is exact.
        // g(x0+h) = sum_{k=0}^{deg} t[k] * h^k where t[k] = g^(k)(x0)/k!
        //
        // For a polynomial p(x) = sum a_j x^j, the Taylor expansion around x0 is:
        // Use the standard polynomial Taylor shift.
        let max_terms = (ndeg + 1).min(pade_coeffs.len());
        // Start with the polynomial coefficients
        let mut poly = vec![0.0; max_terms];
        for i in 0..max_terms {
            poly[i] = pade_coeffs[i];
        }

        // Taylor shift: compute coefficients of p(x0 + h) from p(x)
        // Using repeated synthetic division (Horner shifts)
        for i in 0..max_terms {
            for j in (i + 1..max_terms).rev() {
                poly[j - 1] += x0 * poly[j];
            }
        }

        // poly now contains the Taylor coefficients of g(x0+h)
        // But we need to account for factorial: the shift gives p(x0+h) = sum poly[k] * h^k / k! * k!
        // Actually, the above shift gives: poly[k] = (1/k!) d^k p / dx^k at x0 ... not quite.
        // Let me redo this properly.

        // For polynomial p(x) = sum_{j=0}^{m} a_j x^j,
        // p(x0+h) = sum_{k=0}^m t_k h^k where t_k = sum_{j=k}^{m} a_j * C(j,k) * x0^(j-k)
        // i.e., t_k = (1/k!) p^(k)(x0)

        // Reset and compute properly
        for v in t.iter_mut() {
            *v = 0.0;
        }
        let m = pade_coeffs.len() - 1;
        for k in 0..=ndeg.min(m) {
            let mut tk = 0.0;
            for j in k..=m {
                // binomial(j, k) * a_j * x0^(j-k)
                tk += pade_coeffs[j] * binom(j, k) * x0.powi((j - k) as i32);
            }
            t[k] = tk;
        }
    } else {
        // For larger x: compute directly
        // f(x) = sqrt(x) * asinh(sqrt(x))
        // f'(x) = asinh(sqrt(x)) / (2*sqrt(x)) + 1/(2*(1+x))
        //
        // We can build this using the available expansion functions.
        let s = x0.sqrt();

        // Method: expand asinh(sqrt(x0+h)) and sqrt(x0+h) then multiply
        // But we need single-variable Taylor math for this.
        //
        // Alternative: use the derivative approach.
        // g'(x) = asinh(sqrt(x))/(2*sqrt(x)) + 1/(2*(1+x))
        //
        // Expand 1/(2*(1+x)) around x0: this is 0.5 * inv_expand at (1+x0)
        let mut term1 = vec![0.0; ndeg + 1];
        inv_expand(&mut term1, 1.0 + x0);
        for v in term1.iter_mut() {
            *v *= 0.5;
        }

        // Expand asinh(sqrt(x))/(2*sqrt(x)):
        // Let u = sqrt(x), then asinh(u)/(2u) = asinh(sqrt(x))/(2*sqrt(x))
        // We need the Taylor expansion of this around x0.
        //
        // asinh(u)/u = 1 - u^2/6 + 3u^4/40 - 5u^6/112 + ...
        // = 1 - x/6 + 3x^2/40 - 5x^3/112 + ... (substituting u^2 = x)
        // So asinh(sqrt(x))/(2*sqrt(x)) = (1 - x/6 + 3x^2/40 - ...)/2
        //
        // This is a power series in x, expandable around x0.
        // Let h(x) = asinh(sqrt(x))/(2*sqrt(x))
        //
        // Actually, for the general approach, compute it numerically:
        // h(x) can be gotten from d/dx [sqrt(x)*asinh(sqrt(x))] - 1/(2(1+x))
        // = asinh(sqrt(x))/(2*sqrt(x))
        // Its Taylor expansion is what we need for term2.
        //
        // Let's use a simpler approach: compute f(x) and its derivatives numerically.
        // For x > 1e-4 this is fine.
        //
        // Use the pow/asinh expansion tools:
        // sqrt(x) around x0: sqrt_expand
        // asinh(sqrt(x0)) around sqrt(x0): asinh_expand
        // Compose them. But we need 1D Taylor composition.
        //
        // sqrt(x0+h) = sum s_k h^k (from sqrt_expand)
        // asinh(y0+g) = sum a_k g^k where y0 = sqrt(x0), g = sqrt(x0+h) - sqrt(x0)
        //
        // The g polynomial is [0, s_1, s_2, ...] (the sqrt expansion minus constant)
        // Then compose a_k with this g polynomial to get asinh(sqrt(x0+h))
        // Then multiply by the sqrt expansion to get f(x0+h)

        let mut s_exp = vec![0.0; ndeg + 2]; // one extra for multiplication
        sqrt_expand(&mut s_exp, x0);

        let mut a_exp = vec![0.0; ndeg + 2];
        asinh_expand(&mut a_exp, s);

        // Build the g polynomial (sqrt expansion shifted: g[0]=0, g[k] = s_exp[k] for k>=1)
        // Already in the right form since s_exp[0] = sqrt(x0) and we want deviation from that
        let mut g_poly = vec![0.0; ndeg + 2];
        for i in 1..g_poly.len() {
            g_poly[i] = s_exp[i];
        }

        // Compose asinh coefficients with g polynomial
        taylor1d_compose(&mut a_exp, &g_poly);
        // Now a_exp contains Taylor expansion of asinh(sqrt(x0+h))

        // Multiply a_exp by s_exp to get f(x0+h) = sqrt(x0+h) * asinh(sqrt(x0+h))
        let mut result = vec![0.0; ndeg + 1];
        for k in 0..=ndeg {
            for j in 0..=k {
                result[k] += s_exp[j] * a_exp[k - j];
            }
        }

        for k in 0..=ndeg {
            t[k] = result[k];
        }
    }
}

/// 1D Taylor composition following C++ tmath.hpp `tfuns::compose`.
///
/// Computes f(x(h)) in place. `f` contains Taylor coefficients of the outer function,
/// `x` is the inner polynomial where `x[0]` must be 0 (or is treated as a shift).
/// The composition replaces f with the composed result.
///
/// This matches the C++ switch-case compose that handles orders 0-6.
pub fn taylor1d_compose(f: &mut [f64], x: &[f64]) {
    let n = f.len() - 1; // degree
    debug_assert_eq!(x.len(), f.len());
    // x[0] should be 0 for composition (deviation from constant already subtracted)

    // Use a general algorithm that works for any degree up to 7.
    // From C++ tmath.hpp, the composition is done using Faa di Bruno's formula
    // applied as explicit expressions for each degree.
    //
    // We'll use a Horner-like approach for the 1D case:
    // result = f[n]
    // for i in (n-1)..=0: result = result * x + f[i]
    // But this is NOT correct because x[0] != 0 in general for these polynomials.
    //
    // The C++ approach assumes x[0] = 0 and uses explicit Faa di Bruno formulas.
    // Let's implement it using the explicit formulas for each order.

    // For the C++ compose, f[k] gets replaced with the k-th Taylor coefficient
    // of f(x(h)) where x(h) = x[1]*h + x[2]*h^2 + x[3]*h^3 + ...

    // Actually, looking at C++ more carefully: the compose function uses fall-through
    // switch cases that build from high order down to low order.
    // The key insight is that f[] initially contains the Taylor coefficients of the
    // outer function, and x[] is the inner polynomial with x[0]=0.

    // Generic approach: compute powers of x(h) and accumulate
    // p[k] = x(h)^k as a 1D Taylor polynomial

    // Actually, let's just do it the simple way: compute new coefficients
    let deg = n;
    let mut result = vec![0.0; deg + 1];

    // x_power[k] will be the k-th power of (x[1]*h + x[2]*h^2 + ...)
    // x_power_0 = [1, 0, 0, ...]
    // x_power_k = x_power_{k-1} * x_poly (truncated multiply)
    let mut x_power = vec![vec![0.0; deg + 1]; deg + 1];
    x_power[0][0] = 1.0;

    // x_poly is x with x[0]=0
    let mut x_poly = vec![0.0; deg + 1];
    for i in 1..=deg {
        x_poly[i] = x[i];
    }

    for k in 1..=deg {
        // x_power[k] = x_power[k-1] * x_poly (truncated 1D multiply)
        for i in 0..=deg {
            let mut sum = 0.0;
            for j in 0..=i {
                sum += x_power[k - 1][j] * x_poly[i - j];
            }
            x_power[k][i] = sum;
        }
    }

    // result = sum_{k=0}^{deg} f[k] * x_power[k]
    for k in 0..=deg {
        for i in 0..=deg {
            result[i] += f[k] * x_power[k][i];
        }
    }

    // Copy back
    for i in 0..=deg {
        f[i] = result[i];
    }
}

/// Integration helper: shift coefficients right by one position and divide by index.
/// Sets t[0] = 0.0 (caller must set the integration constant).
fn integrate(t: &mut [f64]) {
    let n = t.len() - 1;
    for i in (1..=n).rev() {
        t[i] = t[i - 1] / (i as f64);
    }
    t[0] = 0.0;
}

/// 1D Taylor multiply: z = x * y (truncated to same degree)
fn taylor1d_mul(z: &mut [f64], x: &[f64], y: &[f64]) {
    let n = z.len() - 1;
    for i in 0..=n {
        z[i] = 0.0;
        for j in 0..=i {
            z[i] += x[j] * y[i - j];
        }
    }
}

/// 1D Taylor in-place multiply: z *= x (truncated to degree)
fn taylor1d_multo(z: &mut [f64], x: &[f64]) {
    let n = z.len() - 1;
    // Must work from high order down to avoid overwriting
    for i in (0..=n).rev() {
        let mut sum = 0.0;
        for j in 0..=i {
            sum += z[j] * x[i - j];
        }
        z[i] = sum;
    }
}

/// 1D Taylor stretch: t[i] *= a^i
fn taylor1d_stretch(t: &mut [f64], a: f64) {
    let mut an = a;
    for i in 1..t.len() {
        t[i] *= an;
        an *= a;
    }
}

/// Binomial coefficient C(n, k)
fn binom(n: usize, k: usize) -> f64 {
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
    use approx::assert_relative_eq;

    // =========================================================================
    // exp_expand tests
    // =========================================================================

    #[test]
    fn test_exp_expand_at_1() {
        let mut t = [0.0; 4]; // degree 3
        exp_expand(&mut t, 1.0);
        let e = 1.0_f64.exp();
        assert_relative_eq!(t[0], e, epsilon = 1e-12);
        assert_relative_eq!(t[1], e, epsilon = 1e-12);
        assert_relative_eq!(t[2], e / 2.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], e / 6.0, epsilon = 1e-12);
    }

    #[test]
    fn test_exp_expand_at_0() {
        let mut t = [0.0; 4];
        exp_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.5, epsilon = 1e-12);
        assert_relative_eq!(t[3], 1.0 / 6.0, epsilon = 1e-12);
    }

    // =========================================================================
    // log_expand tests
    // =========================================================================

    #[test]
    fn test_log_expand_at_1() {
        let mut t = [0.0; 4];
        log_expand(&mut t, 1.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.5, epsilon = 1e-12);
        assert_relative_eq!(t[3], 1.0 / 3.0, epsilon = 1e-12);
    }

    #[test]
    fn test_log_expand_at_2() {
        let mut t = [0.0; 3];
        log_expand(&mut t, 2.0);
        assert_relative_eq!(t[0], 2.0_f64.ln(), epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.5, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.125, epsilon = 1e-12);
    }

    // =========================================================================
    // pow_expand tests
    // =========================================================================

    #[test]
    fn test_pow_expand_sqrt() {
        let mut t = [0.0; 3];
        pow_expand(&mut t, 4.0, 0.5);
        assert_relative_eq!(t[0], 2.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.25, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.015625, epsilon = 1e-12);
    }

    // =========================================================================
    // sqrt_expand tests
    // =========================================================================

    #[test]
    fn test_sqrt_expand_at_4() {
        let mut t = [0.0; 3];
        sqrt_expand(&mut t, 4.0);
        assert_relative_eq!(t[0], 2.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.25, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.015625, epsilon = 1e-12);
    }

    #[test]
    fn test_sqrt_matches_pow_half() {
        let mut t_sqrt = [0.0; 5];
        let mut t_pow = [0.0; 5];
        sqrt_expand(&mut t_sqrt, 3.0);
        pow_expand(&mut t_pow, 3.0, 0.5);
        for i in 0..5 {
            assert_relative_eq!(t_sqrt[i], t_pow[i], epsilon = 1e-12);
        }
    }

    // =========================================================================
    // cbrt_expand tests
    // =========================================================================

    #[test]
    fn test_cbrt_expand_at_8() {
        let mut t = [0.0; 2];
        cbrt_expand(&mut t, 8.0);
        assert_relative_eq!(t[0], 2.0, epsilon = 1e-12);
        // d/dx cbrt(x) at x=8 = (1/3) * 8^(-2/3) = 1/(3*4) = 1/12
        assert_relative_eq!(t[1], 1.0 / 12.0, epsilon = 1e-12);
    }

    // =========================================================================
    // inv_expand tests
    // =========================================================================

    #[test]
    fn test_inv_expand_at_2() {
        let mut t = [0.0; 4];
        inv_expand(&mut t, 2.0);
        assert_relative_eq!(t[0], 0.5, epsilon = 1e-12);
        assert_relative_eq!(t[1], -0.25, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.125, epsilon = 1e-12);
        assert_relative_eq!(t[3], -0.0625, epsilon = 1e-12);
    }

    // =========================================================================
    // sin_expand tests
    // =========================================================================

    #[test]
    fn test_sin_expand_at_0() {
        let mut t = [0.0; 5];
        sin_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], -1.0 / 6.0, epsilon = 1e-12);
        assert_relative_eq!(t[4], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_sin_expand_at_pi_over_2() {
        // sin(pi/2 + h) = cos(h) = 1 - h^2/2 + h^4/24 - ...
        let mut t = [0.0; 5];
        sin_expand(&mut t, std::f64::consts::FRAC_PI_2);
        assert_relative_eq!(t[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.5, epsilon = 1e-12);
        assert_relative_eq!(t[3], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[4], 1.0 / 24.0, epsilon = 1e-12);
    }

    // =========================================================================
    // cos_expand tests
    // =========================================================================

    #[test]
    fn test_cos_expand_at_0() {
        let mut t = [0.0; 5];
        cos_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], -0.5, epsilon = 1e-12);
        assert_relative_eq!(t[3], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[4], 1.0 / 24.0, epsilon = 1e-12);
    }

    // =========================================================================
    // atan_expand tests
    // =========================================================================

    #[test]
    fn test_atan_expand_at_0() {
        let mut t = [0.0; 4];
        atan_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], -1.0 / 3.0, epsilon = 1e-12);
    }

    #[test]
    fn test_atan_expand_at_1() {
        // atan(1+h): atan(1) = pi/4, atan'(1) = 1/(1+1) = 0.5
        let mut t = [0.0; 3];
        atan_expand(&mut t, 1.0);
        assert_relative_eq!(t[0], std::f64::consts::FRAC_PI_4, epsilon = 1e-12);
        assert_relative_eq!(t[1], 0.5, epsilon = 1e-12);
    }

    // =========================================================================
    // erf_expand tests
    // =========================================================================

    #[test]
    fn test_erf_expand_at_0() {
        let mut t = [0.0; 3];
        erf_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 2.0 / PI.sqrt(), epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
    }

    // =========================================================================
    // asinh_expand tests
    // =========================================================================

    #[test]
    fn test_asinh_expand_at_0() {
        let mut t = [0.0; 4];
        asinh_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], -1.0 / 6.0, epsilon = 1e-12);
    }

    // =========================================================================
    // asin_expand tests
    // =========================================================================

    #[test]
    fn test_asin_expand_at_0() {
        // asin(0+h): asin(0) = 0, asin'(0) = 1/sqrt(1) = 1
        // asin''(0) = 0, asin'''(0)/3! = 1/6
        let mut t = [0.0; 4];
        asin_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], 1.0 / 6.0, epsilon = 1e-12);
    }

    // =========================================================================
    // acos_expand tests
    // =========================================================================

    #[test]
    fn test_acos_expand_at_0() {
        // acos(0) = pi/2, acos'(0) = -1, acos''(0)/2! = 0, acos'''(0)/3! = -1/6
        let mut t = [0.0; 4];
        acos_expand(&mut t, 0.0);
        assert_relative_eq!(t[0], std::f64::consts::FRAC_PI_2, epsilon = 1e-12);
        assert_relative_eq!(t[1], -1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], -1.0 / 6.0, epsilon = 1e-12);
    }

    // =========================================================================
    // taylor1d_compose tests
    // =========================================================================

    #[test]
    fn test_taylor1d_compose_identity() {
        // f(x) = [1, 2, 3] composed with identity x[0]=0, x[1]=1, x[2]=0
        let mut f = [1.0, 2.0, 3.0];
        let x = [0.0, 1.0, 0.0];
        taylor1d_compose(&mut f, &x);
        assert_relative_eq!(f[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(f[1], 2.0, epsilon = 1e-12);
        assert_relative_eq!(f[2], 3.0, epsilon = 1e-12);
    }

    #[test]
    fn test_taylor1d_compose_scaling() {
        // f(x) = [0, 1, 0] (= x), composed with x -> 2x: x[0]=0, x[1]=2
        // f(2x) = 2x, so result should be [0, 2, 0]
        let mut f = [0.0, 1.0, 0.0];
        let x = [0.0, 2.0, 0.0];
        taylor1d_compose(&mut f, &x);
        assert_relative_eq!(f[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(f[1], 2.0, epsilon = 1e-12);
        assert_relative_eq!(f[2], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_taylor1d_compose_quadratic() {
        // f(x) = [0, 0, 1] (= x^2), composed with x -> 2h: x[0]=0, x[1]=2, x[2]=0
        // f(2h) = 4h^2, so result should be [0, 0, 4]
        let mut f = [0.0, 0.0, 1.0];
        let x = [0.0, 2.0, 0.0];
        taylor1d_compose(&mut f, &x);
        assert_relative_eq!(f[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(f[1], 0.0, epsilon = 1e-12);
        assert_relative_eq!(f[2], 4.0, epsilon = 1e-12);
    }

    // =========================================================================
    // sqrtx_asinh_sqrtx_expand tests
    // =========================================================================

    #[test]
    fn test_sqrtx_asinh_sqrtx_at_1() {
        let mut t = [0.0; 2];
        sqrtx_asinh_sqrtx_expand(&mut t, 1.0);
        let expected = 1.0_f64.sqrt() * 1.0_f64.sqrt().asinh();
        assert_relative_eq!(t[0], expected, epsilon = 1e-12);
    }

    #[test]
    fn test_sqrtx_asinh_sqrtx_near_zero() {
        // Near x=0, the function should use Pade approximant and not NaN
        let mut t = [0.0; 3];
        sqrtx_asinh_sqrtx_expand(&mut t, 1e-10);
        assert!(t[0].is_finite());
        assert!(t[1].is_finite());
        assert!(t[2].is_finite());
        // At x~0: f(x) ~ x, so f(1e-10) ~ 1e-10
        assert_relative_eq!(t[0], 1e-10, epsilon = 1e-15);
    }

    #[test]
    fn test_sqrtx_asinh_sqrtx_at_zero() {
        let mut t = [0.0; 4];
        sqrtx_asinh_sqrtx_expand(&mut t, 0.0);
        // f(0) = 0, f'(0) = 1, f''(0)/2! = -1/6
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], -1.0 / 6.0, epsilon = 1e-12);
        assert_relative_eq!(t[3], 3.0 / 40.0, epsilon = 1e-12);
    }

    // =========================================================================
    // integrate tests
    // =========================================================================

    #[test]
    fn test_integrate_basic() {
        // Integrate [f0, f1, f2] -> [0, f0/1, f1/2]
        let mut t = [3.0, 6.0, 10.0];
        integrate(&mut t);
        assert_relative_eq!(t[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(t[1], 3.0, epsilon = 1e-12);
        assert_relative_eq!(t[2], 3.0, epsilon = 1e-12); // 6/2
    }

    // =========================================================================
    // Higher-order tests
    // =========================================================================

    #[test]
    fn test_exp_expand_degree_7() {
        let mut t = [0.0; 8]; // degree 7
        exp_expand(&mut t, 0.0);
        // 1/k! for k=0..7
        let expected = [1.0, 1.0, 0.5, 1.0/6.0, 1.0/24.0, 1.0/120.0, 1.0/720.0, 1.0/5040.0];
        for i in 0..8 {
            assert_relative_eq!(t[i], expected[i], epsilon = 1e-12);
        }
    }

    #[test]
    fn test_sin_cos_derivative_relation() {
        // d/dx sin(x) = cos(x), so sin_expand[k+1] * (k+1) = cos_expand[k]
        let mut s = [0.0; 7];
        let mut c = [0.0; 7];
        sin_expand(&mut s, 1.0);
        cos_expand(&mut c, 1.0);
        for k in 0..6 {
            assert_relative_eq!(s[k + 1] * (k + 1) as f64, c[k], epsilon = 1e-12);
        }
    }
}
