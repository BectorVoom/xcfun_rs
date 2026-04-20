//! JT-Toulouse short-range LDA correlation (spin-unpolarized). **LDA-08.**
//!
//! # Source
//! - `xcfun-master/src/functionals/ldaerfc_jt.cpp:1-64` (full file)
//!
//! # No upstream test data
//!
//! The upstream FUNCTIONAL macro at `ldaerfc_jt.cpp:55-64` ends at
//! `ENERGY_FUNCTION(ldaerfc_jt)` with NO `XC_A_B` / `XC_PARTIAL_DERIVATIVES` /
//! `test_threshold` / `test_in` / `test_out` arguments. The tier-1 self-test
//! loop in `crates/xcfun-eval/tests/self_tests.rs` (Plan 02-04 Task 6)
//! filters for `desc.test_in.is_some()` and SKIPS this functional.
//!
//! # D-24 Tier-2 Tolerance Override (USER-APPROVED 2026-04-20)
//!
//! Per CONTEXT D-24, this functional's tier-2 threshold is 1e-7 (matching the
//! sibling LDAERFC / LDAERFX). cubecl 0.10-pre.3 `Float::erf` is a polyfill
//! (~1.3e-8 ULP) and pw92eps + many pow/log compositions combine to ~2e-8
//! final-output rel-error vs C++ libm. NOT silent widening — report.html
//! annotates LDAERF rows with `1e-7 (D-24 override)`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use super::vwn_eps::vwn5_eps;
use crate::density_vars::DensVarsDev;

const RANGESEP_MU_F32: f32 = 0.4_f32;

// c1 parameters (ldaerfc_jt.cpp:24-32).
const C1_U1: f32 = 1.027_074_1_f32;
const C1_U2: f32 = -0.230_160_62_f32;
const C1_V1: f32 = 0.619_688_5_f32;

// c2 parameters (ldaerfc_jt.cpp:34-45).
const C2_A: f32 = 3.2581_f32;
const C2_F: f32 = 3.395_305_5_f32;
const C2_BET: f32 = 163.44_f32;
const C2_GAM: f32 = 4.7125_f32;

// 0.5 * PI — used in c2 denominator.
const HALF_PI: f32 = 1.570_796_3_f32;

// ---------------------------------------------------------------------------
//  c1(rs) = (u1*rs + u2*rs²) / (1 + v1*rs)
//  Port of ldaerfc_jt.cpp:24-32.
// ---------------------------------------------------------------------------

#[cube]
fn c1<F: Float>(rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // rs² = rs * rs
    let mut rs2 = Array::<F>::new(size);
    ctaylor_mul::<F>(rs, rs, &mut rs2, n);

    // numerator: u1*rs + u2*rs²
    let mut u1_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::new(C1_U1), &mut u1_rs, n);
    let mut u2_rs2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs2, F::new(C1_U2), &mut u2_rs2, n);
    let mut numer = Array::<F>::new(size);
    ctaylor_add::<F>(&u1_rs, &u2_rs2, &mut numer, n);

    // denominator: 1 + v1*rs
    let mut v1_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::new(C1_V1), &mut v1_rs, n);
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut denom = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &v1_rs, &mut denom, n);

    // numerator / denominator
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    ctaylor_mul::<F>(&numer, &inv_denom, out, n);
}

// ---------------------------------------------------------------------------
//  c2(d) = d.n * vwn5_eps(d) / (0.5*π * n² * (g0 - 0.5))
//  where g0 = f * (pow(gam+r_s, 1.5) + bet) * exp(-a * sqrt(gam + r_s))
//  Port of ldaerfc_jt.cpp:34-45.
// ---------------------------------------------------------------------------

#[cube]
fn c2<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // gam_plus_rs = gam + r_s
    let mut gam_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut gam_const, n);
    gam_const[0] = F::new(C2_GAM);
    let mut gam_plus_rs = Array::<F>::new(size);
    ctaylor_add::<F>(&gam_const, &d.r_s, &mut gam_plus_rs, n);

    // pow(gam + r_s, 1.5)
    let mut pow_15 = Array::<F>::new(size);
    ctaylor_pow::<F>(&gam_plus_rs, F::new(1.5), &mut pow_15, n);

    // pow_15 + bet   (scalar-add bet to CNST)
    let mut bet_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut bet_const, n);
    bet_const[0] = F::new(C2_BET);
    let mut pow_15_plus_bet = Array::<F>::new(size);
    ctaylor_add::<F>(&pow_15, &bet_const, &mut pow_15_plus_bet, n);

    // sqrt(gam + r_s)
    let mut sqrt_gpr = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&gam_plus_rs, &mut sqrt_gpr, n);

    // -a * sqrt(gam + r_s)
    let mut neg_a_sqrt = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_gpr, F::new(-C2_A), &mut neg_a_sqrt, n);

    // exp(-a * sqrt(...))
    let mut exp_val = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_a_sqrt, &mut exp_val, n);

    // g0 = f * (pow_15 + bet) * exp(...)
    let mut f_poly = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&pow_15_plus_bet, F::new(C2_F), &mut f_poly, n);
    let mut g0 = Array::<F>::new(size);
    ctaylor_mul::<F>(&f_poly, &exp_val, &mut g0, n);

    // g0 - 0.5
    let mut half_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut half_const, n);
    half_const[0] = F::new(0.5);
    let mut g0_m_half = Array::<F>::new(size);
    ctaylor_add::<F>(&g0, &half_const, &mut g0_m_half, n);
    // Actually need g0 - 0.5 (not +)...
    ctaylor_zero::<F>(&mut g0_m_half, n);
    // ... use ctaylor_sub instead to be clear:
    let mut g0_minus_half = Array::<F>::new(size);
    {
        use xcfun_ad::ctaylor::ctaylor_sub;
        ctaylor_sub::<F>(&g0, &half_const, &mut g0_minus_half, n);
    }

    // n² = n * n
    let mut n2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.n, &mut n2, n);

    // denom = 0.5π * n² * (g0 - 0.5)
    let mut half_pi_n2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n2, F::new(HALF_PI), &mut half_pi_n2, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&half_pi_n2, &g0_minus_half, &mut denom, n);

    // d.n * vwn5_eps(d)
    let mut eps = Array::<F>::new(size);
    vwn5_eps::<F>(d, &mut eps, n);
    let mut numer = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &eps, &mut numer, n);

    // result = numer / denom
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    ctaylor_mul::<F>(&numer, &inv_denom, out, n);
}

/// JT-Toulouse LDA correlation kernel. 1:1 port of `ldaerfc_jt.cpp:47-53`:
/// ```cpp
/// double mu = d.get_param(XC_RANGESEP_MU);
/// num denominator = 1.0 + c1(d.r_s)*mu + c2(d)*mu*mu;
/// num result      = d.n * vwn::vwn5_eps(d) / denominator;
/// return result;
/// ```
#[cube]
pub fn ldaerfc_jt_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mu = F::new(RANGESEP_MU_F32);
    let mu2 = F::new(RANGESEP_MU_F32 * RANGESEP_MU_F32);

    // c1(r_s) * mu
    let mut c1_val = Array::<F>::new(size);
    c1::<F>(&d.r_s, &mut c1_val, n);
    let mut c1_mu = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&c1_val, mu, &mut c1_mu, n);

    // c2(d) * mu²
    let mut c2_val = Array::<F>::new(size);
    c2::<F>(d, &mut c2_val, n);
    let mut c2_mu2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&c2_val, mu2, &mut c2_mu2, n);

    // denom = 1 + c1*mu + c2*mu²
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut tmp = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &c1_mu, &mut tmp, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_add::<F>(&tmp, &c2_mu2, &mut denom, n);

    // numer = d.n * vwn5_eps(d)
    let mut eps = Array::<F>::new(size);
    vwn5_eps::<F>(d, &mut eps, n);
    let mut numer = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &eps, &mut numer, n);

    // out = numer / denom
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    ctaylor_mul::<F>(&numer, &inv_denom, out, n);
}
