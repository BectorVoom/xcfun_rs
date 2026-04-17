//! Shared LDA helper functions.
//!
//! Generic over `T: Num` for use with both f64 (scalar) and CTaylor (AD).
//! Ported character-for-character from C++ xcfun sources.

use xcfun_ad::Num;

// =============================================================================
// VWN parameters (from vwn.hpp)
// =============================================================================

// VWN5 parameters (non-XCFUN_VWN5_REF, default)
// p = [x0, A, b, c]
pub const VWN5_PARA: [f64; 4] = [-0.10498, 0.0621814, 3.72744, 12.9352];
pub const VWN5_FERRO: [f64; 4] = [-0.325, 0.0310907, 7.06042, 18.0578];
// Note: inter[1] = -1/(3*pi^2), computed at compile time
pub const VWN5_INTER: [f64; 4] = [
    -0.0047584,
    -1.0 / (3.0 * std::f64::consts::PI * std::f64::consts::PI),
    1.13107,
    13.0045,
];

// VWN3 parameters (from vwn.hpp vwn3_eps)
pub const VWN3_PARA: [f64; 4] = [-0.4092860, 0.0621814, 13.0720, 42.7198];
pub const VWN3_FERRO: [f64; 4] = [-0.7432940, 0.0310907, 20.1231, 101.578];

// =============================================================================
// PW92 parameters (from pw92eps.hpp PW92C_PARAMS)
// =============================================================================

// TUVWXYP[3][7] -- each row is [t0, t1, t2, t3, t4, t5, t6]
// The 7th element is always 1 in C++ but is included for indexing consistency.
pub const PW92_TUVWXYP: [[f64; 7]; 3] = [
    [0.03109070, 0.21370, 7.59570, 3.5876, 1.63820, 0.49294, 1.0],
    [0.01554535, 0.20548, 14.1189, 6.1977, 3.36620, 0.62517, 1.0],
    [0.01688690, 0.11125, 10.3570, 3.6231, 0.88026, 0.49671, 1.0],
];

// =============================================================================
// PZ81 parameters (from pz81c.hpp)
// =============================================================================

// c[4][4] in C++, but the low-density arrays are [3] and high-density are [4].
// We store them as separate arrays to match C++ indexing exactly.
// Low density: CB1B2 arrays (3 elements each) -- used by Eld
pub const PZ81_C0: [f64; 3] = [-0.1423, 1.0529, 0.3334];
pub const PZ81_C1: [f64; 3] = [-0.0843, 1.3981, 0.2611];
// High density: c arrays (4 elements each) -- used by Ehd
pub const PZ81_C2: [f64; 4] = [0.0311, -0.048, 0.0020, -0.0116];
pub const PZ81_C3: [f64; 4] = [0.01555000000, -0.0269, 0.0007, -0.0048];

// =============================================================================
// Spin interpolation (from specmath.hpp)
// =============================================================================

/// Spin interpolation function: (1+x)^a + (1-x)^a
///
/// Port of C++ `ufunc(x, a)` from specmath.hpp.
pub fn ufunc<T: Num>(x: &T, a: f64) -> T {
    let one = T::one();
    let plus = one.clone() + x.clone();
    let minus = one - x.clone();
    plus.pow(a) + minus.pow(a)
}

// =============================================================================
// VWN parameterized function (from vwn.hpp)
// =============================================================================

/// VWN parameterized correlation function.
///
/// Port of C++ `vwn::vwn_f(s, p)` from vwn.hpp.
/// Parameters p = [x0, A, b, c].
///
/// Pre-computes all f64 constants from parameters outside the generic path,
/// then injects via T::from_f64().
pub fn vwn_f<T: Num>(s: &T, p: &[f64; 4]) -> T {
    let x0 = p[0];
    let a_param = p[1];
    let b = p[2];
    let c = p[3];

    // Pre-compute f64 constants (matching C++ vwn_a, vwn_b, vwn_c)
    let x0_sq = x0 * x0;
    let denom = x0_sq + x0 * b + c;
    let ratio = x0 * b / denom;

    let vwn_a_val = ratio - 1.0;
    let vwn_b_val = 2.0 * (ratio - 1.0) + 2.0;

    let disc = (4.0 * c - b * b).sqrt();
    let vwn_c_val = 2.0 * b
        * (1.0 / disc - x0 / (denom * disc / (b + 2.0 * x0)));

    // Generic path: compute X(s), Y(s), Z(s)
    // X(s) = s^2 + b*s + c
    let s_sq = s.clone() * s.clone();
    let vwn_x = s_sq + T::from_f64(b) * s.clone() + T::from_f64(c);

    // Y(s) = s - x0
    let vwn_y = s.clone() - T::from_f64(x0);

    // Z(s) = disc / (2*s + b)
    let two_s_plus_b = T::from_f64(2.0) * s.clone() + T::from_f64(b);
    let vwn_z = T::from_f64(disc) / two_s_plus_b;

    // result = 0.5 * A * (2*ln(s) + a*ln(X(s)) - b*ln(Y(s)) + c*atan(Z(s)))
    let result = T::from_f64(0.5 * a_param)
        * (T::from_f64(2.0) * s.clone().log()
            + T::from_f64(vwn_a_val) * vwn_x.log()
            - T::from_f64(vwn_b_val) * vwn_y.log()
            + T::from_f64(vwn_c_val) * vwn_z.atan());

    result
}

// =============================================================================
// PW92 parameterized function (from pw92eps.hpp)
// =============================================================================

/// PW92 parameterized correlation function.
///
/// Port of C++ `pw92eps::eopt(sqrtr, t)` from pw92eps.hpp.
/// t = [t0, t1, t2, t3, t4, t5, t6].
///
/// Formula: -2*t[0] * (1 + t[1]*sqrtr^2) * ln(1 + 0.5/(t[0] * sqrtr * (t[2] + sqrtr*(t[3] + sqrtr*(t[4] + t[5]*sqrtr)))))
pub fn pw92_eopt<T: Num>(sqrt_r: &T, t: &[f64; 7]) -> T {
    let sqrtr_sq = sqrt_r.clone() * sqrt_r.clone();

    // (1 + t[1]*sqrtr^2)
    let bracket = T::one() + T::from_f64(t[1]) * sqrtr_sq;

    // Nested polynomial in sqrt_r for denominator:
    // sqrtr * (t[2] + sqrtr*(t[3] + sqrtr*(t[4] + t[5]*sqrtr)))
    let inner = T::from_f64(t[4]) + T::from_f64(t[5]) * sqrt_r.clone();
    let inner = T::from_f64(t[3]) + sqrt_r.clone() * inner;
    let inner = T::from_f64(t[2]) + sqrt_r.clone() * inner;
    let denom_product = sqrt_r.clone() * inner;

    // 1 + 0.5 / (t[0] * denom_product)
    let log_arg =
        T::one() + T::from_f64(0.5) / (T::from_f64(t[0]) * denom_product);

    // -2*t[0] * bracket * ln(log_arg)
    T::from_f64(-2.0 * t[0]) * bracket * log_arg.log()
}

/// PW92 spin interpolation omega(zeta).
///
/// Port of C++ `pw92eps::omega(z)` from pw92eps.hpp (non-XCFUN_REF_PW92C path).
/// Formula: (ufunc(zeta, 4/3) - 2) / (2 * 2^(1/3) - 2)
pub fn pw92_omega<T: Num>(zeta: &T) -> T {
    // Compute denominator as f64 constant (NOT the hardcoded 0.5198421)
    let denom = 2.0 * 2.0_f64.powf(1.0 / 3.0) - 2.0;

    let u = ufunc(zeta, 4.0 / 3.0);
    (u - T::from_f64(2.0)) / T::from_f64(denom)
}

// =============================================================================
// PZ81 helper functions (from pz81c.hpp)
// =============================================================================

/// PZ81 low-density correlation: CB[0] / (1 + CB[1]*sqrt(x) + CB[2]*x)
///
/// Port of C++ `pz81eps::Eld(x, CB1B2)` from pz81c.hpp.
pub fn pz81_eld<T: Num>(x: &T, cb: &[f64; 3]) -> T {
    let sqrt_x = x.clone().sqrt();
    T::from_f64(cb[0])
        / (T::one() + T::from_f64(cb[1]) * sqrt_x + T::from_f64(cb[2]) * x.clone())
}

/// PZ81 high-density correlation: c[1] + log(x) * (c[0] + x * c[2]) + c[3] * x
///
/// Port of C++ `pz81eps::Ehd(x, c)` from pz81c.hpp.
/// Note: C++ is `c[1] + log(x) * (c[0] + x * c[2]) + c[3] * x`.
pub fn pz81_ehd<T: Num>(x: &T, c: &[f64; 4]) -> T {
    T::from_f64(c[1])
        + x.clone().log() * (T::from_f64(c[0]) + x.clone() * T::from_f64(c[2]))
        + T::from_f64(c[3]) * x.clone()
}
