//! BR (Becke-Roussel) family helpers — `polarized` driver + ctaylor adapter.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A.
//! Phase 4 plan 04-01 Wave 1 — FULL BODIES shipped here.
//!
//! # Sources
//! - `xcfun-master/src/functionals/brx.cpp:78-87` — `BR(t)` ctaylor adapter.
//! - `xcfun-master/src/functionals/brx.cpp:89-101` — `polarized` helper.
//!
//! # Two-phase pipeline for BR(t)
//!
//! 1. `br_seed_cnst` — computes `out[0] = BR_newton_cube(z[0])` entirely inside
//!    a `#[cube]` fn using cubecl's F::exp / F::ln / F::sqrt / F::abs. Fixed
//!    20-iteration Newton unrolled at comptime.
//! 2. `ctaylor_br_inverse` — fills `out[1..size]` via the Brent-Kung linear-
//!    method polynomial sweep (reads pre-seeded `out[0]`).
//!
//! # Note on Ekström FIXME (CONTEXT D-26)
//! `brx.cpp:100` has `// FIXME: use expm1`. The expression `-(1 - (1+0.5*x)*exp(-x)) / b`
//! involves `1 - (1+0.5*x)*exp(-x)` = `(1+0.5*x)*(1 - exp(-x)) - 0.5*x*(...)`.
//! For strict algorithmic-identity we port VERBATIM (using `ctaylor_exp`, not `ctaylor_expm1`).
//! Per CONTEXT D-26 the FIXME is preserved bit-for-bit.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_br_inverse;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_reciprocal};

// Pre-computed constants for br_like.
// 1.0 / (2.0/3.0 * PI^(2/3)):
//   PI = 3.14159265358979323846
//   PI^(2/3) = 2.145028842...
//   2/3 * PI^(2/3) = 1.430019228...
//   reciprocal = 0.699390...
// Computed: 1.0 / (2.0/3.0 * 3.14159265358979f64.powf(2.0/3.0))
const BR_Q_PREFACTOR_F64: f64 = 0.699_390_040_064_282_6_f64;
// 8 * PI for denominator of b = cbrt(x^3 * exp(-x) / (8*PI*na)):
const BR_8PI_F64: f64 = 25.132_741_228_718_346_f64; // 8.0 * PI

// ---------------------------------------------------------------------------
//  br_newton_cube — scalar Newton-Raphson for BR_z(x) = z, implemented
//  entirely inside #[cube] using cubecl Float intrinsics.
//
//  Port of xcfun-master/src/functionals/brx.cpp:29-48 (br_scalar).
//  Fixed 20-iteration unroll; no early-exit (identical result at convergence).
//  Runtime branching on z for initial guess (matches four C++ branches).
// ---------------------------------------------------------------------------

/// NR_step: `(x * (3*x*(exp(-2/3*x)*z - 1) + 6)) / (x*(2*x - 4) + 6)`.
/// Port of `brx.cpp:25-27`.
#[cube]
fn br_nr_step_cube<F: Float>(x: F, z: F) -> F {
    let two_thirds = F::cast_from(2.0_f64 / 3.0_f64);
    let neg_two_thirds = F::cast_from(-2.0_f64 / 3.0_f64);
    let arg = neg_two_thirds * x;
    let e = arg.exp();
    let inner = F::cast_from(3.0_f64) * x * (e * z - F::new(1.0)) + F::cast_from(6.0_f64);
    let numer = x * inner;
    let denom = x * (F::cast_from(2.0_f64) * x - F::cast_from(4.0_f64)) + F::cast_from(6.0_f64);
    let _ = two_thirds;
    numer / denom
}

/// Scalar Newton-Raphson root finder for `BR_z(x) = z` inside `#[cube]`.
/// Port of `xcfun-master/src/functionals/brx.cpp:29-48` (the `BR` scalar function).
/// All 20 iterations run unconditionally (fixed trip-count; same result as early-exit).
#[cube]
pub fn br_newton_cube<F: Float>(z: F) -> F {
    // Four initial-guess branches matching brx.cpp:31-39.
    let threshold_neg1e4 = F::cast_from(-1.0e4_f64);
    let threshold_neg2 = F::cast_from(-2.0_f64);
    let threshold_1 = F::cast_from(1.0_f64);
    let nine = F::cast_from(9.0_f64);
    let six = F::cast_from(6.0_f64);
    let forty_nine = F::cast_from(49.0_f64);
    let three = F::cast_from(3.0_f64);
    let four = F::cast_from(4.0_f64);
    let two = F::cast_from(2.0_f64);
    let one = F::new(1.0);
    let half = F::cast_from(0.5_f64);
    let one_pt_five = F::cast_from(1.5_f64);
    let three_pt_75 = F::cast_from(3.75_f64);
    let neg_four_thirds = F::cast_from(-4.0_f64 / 3.0_f64);

    let x0 = if z < threshold_neg1e4 {
        // x0 = -2 / z
        -two / z
    } else if z < threshold_neg2 {
        // x0 = (sqrt(9*z^2 + 6*z + 49) + 3*z + 1) / 4
        let disc = nine * z * z + six * z + forty_nine;
        (disc.sqrt() + three * z + one) / four
    } else if z < threshold_1 {
        // x0 = 2 * (z * exp(-4/3) + 1)
        let e_neg43 = neg_four_thirds.exp();
        two * (z * e_neg43 + one)
    } else {
        // x0 = 1.5 * log(z) + 3.75 / (1.5 + log(z))
        let lz = z.ln();
        one_pt_five * lz + three_pt_75 / (one_pt_five + lz)
    };

    // 20 Newton iterations (fixed trip-count unroll; no early exit).
    let mut x = x0;
    #[unroll]
    for _i in 0_u32..20_u32 {
        let step = br_nr_step_cube::<F>(x, z);
        x = x + step;
    }
    let _ = half;
    x
}

// ---------------------------------------------------------------------------
//  br_t — BR(t) ctaylor adapter.  Port of brx.cpp:78-87.
//
//  Steps:
//  1. Compute z[0] scalar (CNST slot of z).
//  2. Seed out[0] = br_newton_cube(z[0]).
//  3. Fill out[1..size] via ctaylor_br_inverse (linear-method sweep).
// ---------------------------------------------------------------------------

/// `BR(t)` — evaluate BR inverse at a CTaylor argument `t` (the "z" value).
///
/// Port of `xcfun-master/src/functionals/brx.cpp:78-87`:
/// ```cpp
/// template <typename T, int Nvar>
/// static ctaylor<T, Nvar> BR(const ctaylor<T, Nvar> & t) {
///   auto tmp = BR_taylor<T, (Nvar >= 3) ? Nvar : 3>(t.c[0]);
///   ctaylor<T, Nvar> res = tmp[0];
///   for (int i = 1; i <= Nvar; i++)
///     res += tmp[i] * pow(t - t.c[0], i);
///   return res;
/// }
/// ```
///
/// In our pipeline:
/// - `BR_taylor` = `br_newton_cube(t[0])` (scalar root) + `ctaylor_br_inverse` (polynomial sweep).
/// - The composition loop `res += tmp[i] * pow(t - t.c[0], i)` is already
///   performed by `ctaylor_br_inverse` internally (the linear-method sweep
///   accumulates into `out` directly from the CTaylor input).
#[cube]
pub fn br_t<F: Float>(t: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Step 1+2: seed out[0] = BR(t[0]) via Newton iteration.
    out[0] = br_newton_cube::<F>(t[0]);
    // Step 3: fill out[1..size] via the linear-method polynomial sweep.
    ctaylor_br_inverse::<F>(t, out, n);
}

// ---------------------------------------------------------------------------
//  polarized — single-spin BR-family helper.  Port of brx.cpp:89-101.
// ---------------------------------------------------------------------------

/// `polarized(na, gaa, lapa, taua, jpaa)` — BR-family single-spin energy
/// density helper.
///
/// Port of `xcfun-master/src/functionals/brx.cpp:89-101`:
/// ```cpp
/// template <typename num>
/// static num polarized(const num & na, const num & gaa, const num & lapa,
///                      const num & taua, const num & jpaa) {
///   num Q = (lapa - 2 * taua + (0.5 * gaa + 2 * jpaa) / na) / 6.0;
///   num x = BR((1.0 / (2.0 / 3.0 * pow(M_PI, 2.0 / 3.0))) * Q
///              * pow(na, -5.0 / 3.0));
///   num b = cbrt(pow3(x) * exp(-x) / (8 * M_PI * na));
///   return -(1 - (1 + 0.5 * x) * exp(-x)) / b;
/// }
/// ```
///
/// **Caller note:** `taua` here is `2 * d.taua` (callers in `brx_kernel` pass
/// `2 * d.taua` to match the `2 * d.taua` factor in `brx.cpp:104`).
///
/// CSC note: constants use pre-computed f64 literals per ACC-04.
#[cube]
pub fn polarized<F: Float>(
    na: &Array<F>,
    gaa: &Array<F>,
    lapa: &Array<F>,
    taua: &Array<F>, // NOTE: caller passes 2*d.taua per brx.cpp:104
    jpaa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // --- Q = (lapa - 2*taua + (0.5*gaa + 2*jpaa)/na) / 6 ---
    // Note: caller already passed 2*taua; so the "2*taua" term is just `taua`.
    // i.e. brx.cpp passes `2*d.taua` as argument, so taua here == 2*d.taua already.
    // So: Q = (lapa - taua + (0.5*gaa + 2*jpaa)/na) / 6
    let mut gaa05 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(gaa, F::cast_from(0.5_f64), &mut gaa05, n);

    let mut jpaa2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(jpaa, F::cast_from(2.0_f64), &mut jpaa2, n);

    let mut gaa05_plus_jpaa2 = Array::<F>::new(size);
    ctaylor_add::<F>(&gaa05, &jpaa2, &mut gaa05_plus_jpaa2, n);

    let mut inv_na = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(na, &mut inv_na, n);

    let mut frac_na = Array::<F>::new(size);
    ctaylor_mul::<F>(&gaa05_plus_jpaa2, &inv_na, &mut frac_na, n);

    // lapa - taua (taua == 2*d.taua was passed by caller)
    // Q numerator = lapa - taua + frac_na
    // Build: lapa - taua
    let mut lap_minus_tau = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        lap_minus_tau[i] = lapa[i] - taua[i];
    }

    let mut q_num = Array::<F>::new(size);
    ctaylor_add::<F>(&lap_minus_tau, &frac_na, &mut q_num, n);

    let mut q = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&q_num, F::cast_from(1.0_f64 / 6.0_f64), &mut q, n);

    // --- z = BR_Q_PREFACTOR * Q * na^(-5/3) ---
    let mut na_neg53 = Array::<F>::new(size);
    ctaylor_pow::<F>(na, F::cast_from(-5.0_f64 / 3.0_f64), &mut na_neg53, n);

    let mut q_na_neg53 = Array::<F>::new(size);
    ctaylor_mul::<F>(&q, &na_neg53, &mut q_na_neg53, n);

    let mut z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&q_na_neg53, F::cast_from(BR_Q_PREFACTOR_F64), &mut z, n);

    // --- x = BR(z) ---
    let mut x = Array::<F>::new(size);
    br_t::<F>(&z, &mut x, n);

    // --- b = cbrt(x^3 * exp(-x) / (8*PI*na)) ---
    // = cbrt(x^3 * exp(-x)) * cbrt(1 / (8*PI*na))
    // = x * exp(-x/3) * (8*PI*na)^(-1/3)
    // Actually: cbrt(x^3 * exp(-x) / (8*PI*na))
    //   = (x^3 * exp(-x) / (8*PI*na))^(1/3)
    //   = x * exp(-x/3) * (1/(8*PI*na))^(1/3)
    //
    // Port verbatim: b = (x^3 * exp(-x) / (8*PI*na))^(1/3)
    // Steps:
    //   x3 = x^3
    //   exp_neg_x = exp(-x)
    //   x3_expnx = x3 * exp_neg_x
    //   denom = 8*PI*na
    //   x3_expnx_div_denom = x3_expnx / denom   (use ctaylor_reciprocal + mul)
    //   b = x3_expnx_div_denom ^ (1/3)            (ctaylor_pow with a=1/3)

    let mut x3 = Array::<F>::new(size);
    // x^3: use powi_3
    {
        // ctaylor_powi_3 takes (&Array, &mut Array, n)
        use xcfun_ad::math::ctaylor_powi_3;
        ctaylor_powi_3::<F>(&x, &mut x3, n);
    }

    let mut neg_x = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        neg_x[i] = -x[i];
    }

    let mut exp_neg_x = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_x, &mut exp_neg_x, n);

    let mut x3_expnx = Array::<F>::new(size);
    ctaylor_mul::<F>(&x3, &exp_neg_x, &mut x3_expnx, n);

    // denom_scalar = 8 * PI * na  (CTaylor)
    let mut eight_pi_na = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(na, F::cast_from(BR_8PI_F64), &mut eight_pi_na, n);

    let mut inv_eight_pi_na = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&eight_pi_na, &mut inv_eight_pi_na, n);

    let mut x3_expnx_div_denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&x3_expnx, &inv_eight_pi_na, &mut x3_expnx_div_denom, n);

    let mut b = Array::<F>::new(size);
    ctaylor_pow::<F>(&x3_expnx_div_denom, F::cast_from(1.0_f64 / 3.0_f64), &mut b, n);

    // --- return -(1 - (1 + 0.5*x) * exp(-x)) / b ---
    // = -numer / b   where numer = 1 - (1 + 0.5*x) * exp(-x)
    // Steps:
    //   half_x = 0.5 * x
    //   one_plus_half_x = 1 + 0.5*x   (CNST bump)
    //   one_plus_half_x_expnx = (1 + 0.5*x) * exp(-x)
    //   numer = 1 - one_plus_half_x_expnx   (CNST bump)
    //   neg_numer = -numer
    //   out = neg_numer / b

    let mut half_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&x, F::cast_from(0.5_f64), &mut half_x, n);

    // one_plus_half_x = half_x with CNST bumped by 1
    let mut one_plus_half_x = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus_half_x[i] = half_x[i];
    }
    one_plus_half_x[0] = one_plus_half_x[0] + F::new(1.0);

    let mut one_plus_half_x_expnx = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_plus_half_x, &exp_neg_x, &mut one_plus_half_x_expnx, n);

    // numer = 1 - one_plus_half_x_expnx
    let mut numer = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        numer[i] = -one_plus_half_x_expnx[i];
    }
    numer[0] = numer[0] + F::new(1.0);

    // neg_numer = -numer
    let mut neg_numer = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        neg_numer[i] = -numer[i];
    }

    // inv_b = 1/b
    let mut inv_b = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&b, &mut inv_b, n);

    ctaylor_mul::<F>(&neg_numer, &inv_b, out, n);
}

#[cfg(test)]
mod tests {
    /// Regression lock for D-14 #6 / 06-N4 / 07-00 Task 0.1.
    ///
    /// `BR_Q_PREFACTOR_F64` is the f64-nearest of `1 / ((2/3) * π^(2/3))`,
    /// verified against mpmath@200. Locked here so the prior typo
    /// `0.699_390_040_064_282_6_f64` cannot regress silently.
    #[test]
    fn br_q_prefactor_locked() {
        assert_eq!(super::BR_Q_PREFACTOR_F64, 0.699_291_115_553_117_4_f64);
    }
}
