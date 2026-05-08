//! Short-range LDA correlation (range-separated). **LDA-07.**
//!
//! # Source
//! - `xcfun-master/src/functionals/ldaerfc.cpp:1-144` (full file — Qrpa + dpol + g0f + ecorrlr + ldaerfc)
//!
//! # D-24 Tier-2 Tolerance Override (USER-APPROVED 2026-04-20)
//!
//! Upstream `xcfun-master/src/functionals/ldaerfc.cpp:124` uses `test_threshold = 1e-7`.
//! cubecl 0.10-pre.3 `Float::erf` is a polyfill (~1.3e-8 ULP) — not directly used here,
//! but pw92eps + many pow/exp compositions combine to produce ~2e-8 final-output
//! rel-error vs C++ libm in the LDAERF chain (RESEARCH §"D-19 LDAERF Tolerance Analysis").
//! Per CONTEXT D-24, Phase 2 tier-2 uses 1e-7 for this functional, MATCHING upstream's
//! own self-test threshold. NOT silent widening — report.html annotates LDAERF rows
//! with `1e-7 (D-24 override)` for full transparency.
//!
//! Phase 6 revisits with libm-call hybrid when CUDA/Wgpu drift also enters scope.
//!
//! # Implementation
//! Direct operation-order port of the C++ source. Range-separation parameter
//! `mu = XC_RANGESEP_MU = 0.4` hard-coded (Phase 5 RS-01..10 will wire runtime API).
//!
//! # 2026-04-21 constants-correctness fix (Plan 02-04 Wave-1B-14c)
//!
//! Initial Wave-1B-12 port hard-coded several f64 constants (most notably
//! `QRPA_B2`, `DPOL_LEAD_SCALE`, `ECORRLR_ALPHA`, `ECORRLR_CF{,_SQ}`, and the
//! `coe5` prefactor `-9/(40*sqrt(2π))`) whose symbolic expansions drifted from
//! what libm `pow`/`log`/`sqrt` actually produce at IEEE-754 f64 precision.
//! `QRPA_B2` alone was wrong by ~3e-4 *relative*, and `DPOL_LEAD_SCALE` by
//! ~2.6e-4 — far outside the 1e-7 D-24 tier-1 budget. The tier-1 self-test
//! (Wave-1B-14b) flagged a 6.3e-6 rel-error on element [0] of the XC_A_B
//! order-2 output grid, which diagnosed the discrepancy.
//!
//! Every literal has been regenerated using Python+libm (verified to match
//! glibc `pow`/`cbrt`/`log`/`sqrt` bit-for-bit) and each constant carries an
//! inline derivation. With corrected literals, Python reference evaluation
//! matches the upstream target `-1.4579390272267870e-01` to **0 relative
//! error** — confirming the root cause is strictly the constant literals,
//! not any kernel/AD algorithmic drift (CONTEXT D-19 algorithmic-identity
//! port invariant holds).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use super::pw92eps::pw92_eps;
use crate::density_vars::DensVarsDev;

const RANGESEP_MU_F64: f64 = 0.4_f64;

// Qrpa constants (ldaerfc.cpp:24-28) — all f64-precise per 1e-7 D-24 threshold.
//
// The constants are computed *exactly* as the C++ reference does (operation
// order matches libm/glibc at IEEE-754 f64 precision). Values were regenerated
// 2026-04-21 (see Plan 02-04 Wave-1B-14c fix note) — the previous precomputed
// f64 literals were derived from an incorrect symbolic expansion and induced
// ~6e-6 relative error in the LDAERFC tier-1 test, well above the 1e-7 D-24
// threshold.
//
// Acoul = 2 * (log(2) - 1) / (π²)
//   = 2 * (0.6931471805599453 - 1) / 9.869604401089358
//   = -0.0621813817393098        (libm-computed f64)
const QRPA_ACOUL: f64 = -0.0621813817393098_f64;
const QRPA_A2: f64 = 5.84605_f64;
const QRPA_C2: f64 = 3.91744_f64;
const QRPA_D2: f64 = 3.44851_f64;
// b2 = d2 - 3 / (2π * Acoul) * pow(4 / (9π), 1/3)
//    = 3.44851 - 3 / (2π * -0.0621813817393098) * 0.521061761197848
//    = 3.44851 - (-7.67905...) * 0.521061761197848
//    = 3.44851 + 4.001015382634055
//    = 7.4495253826340555         (libm-computed f64)
const QRPA_B2: f64 = 7.4495253826340555_f64;

// dpol constants (ldaerfc.cpp:34-40).
// cf = pow(9π/4, 1/3) = 1.9191582926775128   (libm pow / cbrt)
const DPOL_CF: f64 = 1.9191582926775128_f64;
// cf² = cf * cf = 3.683168552352866
const DPOL_CF_SQ: f64 = 3.683168552352866_f64;
const DPOL_P2P: f64 = 0.04_f64;
const DPOL_P3P: f64 = 0.4319_f64;
// pow(2, 5/3) / 5 = 0.6349604207872799
#[allow(dead_code)]
const DPOL_TWO_53_OVER_5: f64 = 0.6349604207872799_f64;
// P3P - 0.454555 = 0.4319 - 0.454555 = -0.022655 (exact in f64)
const DPOL_P3P_MINUS: f64 = -0.022655_f64;

// g0f constants (ldaerfc.cpp:47-52).
const G0F_C0F: f64 = 0.0819306_f64;
const G0F_D0F: f64 = 0.752411_f64;
const G0F_E0F: f64 = -0.0127713_f64;
const G0F_F0F: f64 = 0.00185898_f64;
// D0F - 0.7317 = 0.020711000000000045 (IEEE-754 f64 exact result).
const G0F_D0F_MINUS: f64 = 0.020711000000000045_f64;

// ecorrlr constants (ldaerfc.cpp:57-67).
// alpha = pow(4/9/π, 1/3) = 0.521061761197848
const ECORRLR_ALPHA: f64 = 0.521061761197848_f64;
// cf = 1/alpha = 1.9191582926775133 (note: differs from DPOL_CF in the last
// ULP due to the different formula in C++; xcfun's ecorrlr uses `1/alpha`,
// dpol uses `pow(9π/4, 1/3)` directly).
const ECORRLR_CF: f64 = 1.9191582926775133_f64;
// ECORRLR cf² = cf*cf = 3.6831685523528677 (matches DPOL_CF_SQ to 1 ULP).
const ECORRLR_CF_SQ: f64 = 3.6831685523528677_f64;
const ECORRLR_ADIB: f64 = 0.784949_f64;
const ECORRLR_Q1A: f64 = -0.388_f64;
const ECORRLR_Q2A: f64 = 0.676_f64;
const ECORRLR_Q3A: f64 = 0.547_f64;
const ECORRLR_T1A: f64 = -4.95_f64;
const ECORRLR_T2A: f64 = 1.0_f64;
const ECORRLR_T3A: f64 = 0.31_f64;

// sqrt(2π) = 2.5066282746310002 (libm sqrt on 2π f64).
const SQRT_TWO_PI: f64 = 2.5066282746310002_f64;

// ---------------------------------------------------------------------------
//  Qrpa(x) = Acoul * log((1 + x*(a2 + x*(b2 + c2*x))) / (1 + x*(a2 + d2*x)))
//  Port of ldaerfc.cpp:23-31.
//
//  Operation order (innermost outwards):
//    num_inner_b = c2 * x + b2     (addition: c2*x + b2_const)
//    num_mid     = x * num_inner_b
//    num_sum     = a2 + num_mid
//    num_outer   = x * num_sum
//    num         = 1 + num_outer
//
//    den_mid     = d2 * x
//    den_sum     = a2 + den_mid
//    den_outer   = x * den_sum
//    den         = 1 + den_outer
//
//    ratio       = num / den
//    log_ratio   = log(ratio)
//    out         = Acoul * log_ratio
// ---------------------------------------------------------------------------

#[cube]
fn qrpa<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Pre-construct constant-coefficient CTaylors for scalar-add sites.
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);

    let mut a2_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut a2_const, n);
    a2_const[0] = F::cast_from(QRPA_A2);

    let mut b2_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut b2_const, n);
    b2_const[0] = F::cast_from(QRPA_B2);

    // Numerator Horner: num_inner_b = c2*x + b2
    let mut c2_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, F::cast_from(QRPA_C2), &mut c2_x, n);
    let mut num_inner_b = Array::<F>::new(size);
    ctaylor_add::<F>(&c2_x, &b2_const, &mut num_inner_b, n);

    // num_mid = x * num_inner_b
    let mut num_mid = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &num_inner_b, &mut num_mid, n);

    // num_sum = a2 + num_mid
    let mut num_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&a2_const, &num_mid, &mut num_sum, n);

    // num_outer = x * num_sum
    let mut num_outer = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &num_sum, &mut num_outer, n);

    // num = 1 + num_outer
    let mut num_arr = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &num_outer, &mut num_arr, n);

    // Denominator Horner: den_mid = d2 * x
    let mut den_mid = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, F::cast_from(QRPA_D2), &mut den_mid, n);
    // den_sum = a2 + den_mid
    let mut den_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&a2_const, &den_mid, &mut den_sum, n);
    // den_outer = x * den_sum
    let mut den_outer = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &den_sum, &mut den_outer, n);
    // den = 1 + den_outer
    let mut den_arr = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &den_outer, &mut den_arr, n);

    // ratio = num / den
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den_arr, &mut inv_den, n);
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&num_arr, &inv_den, &mut ratio, n);

    // log_ratio = log(ratio)
    let mut log_ratio = Array::<F>::new(size);
    ctaylor_log::<F>(&ratio, &mut log_ratio, n);

    // out = Acoul * log_ratio
    ctaylor_scalar_mul::<F>(&log_ratio, F::cast_from(QRPA_ACOUL), out, n);
}

// ---------------------------------------------------------------------------
//  dpol(rs) = (2^(5/3)/5) * cf² / rs² * (1 + (P3P - 0.454555)*rs) / (1 + P3P*rs + P2P*rs²)
//  Port of ldaerfc.cpp:33-40.
//
//  Operation order:
//    rs2 = rs * rs
//    inv_rs2 = 1/rs2
//    scale = (2^(5/3)/5) * cf²       (host-precomputed = 2.33926...)
//    lead  = scale * inv_rs2
//    num_tail = (P3P - 0.454555) * rs
//    num_bracket = 1 + num_tail
//    den_linear = P3P * rs
//    den_quad   = P2P * rs2
//    den_sum    = den_linear + den_quad
//    den        = 1 + den_sum
//    inv_den    = 1/den
//    ratio      = num_bracket * inv_den
//    out        = lead * ratio
// ---------------------------------------------------------------------------

// Precompute: (2^(5/3)/5) * cf² = 0.6349604207872799 * 3.683168552352866
//           = 2.3386662538324523 (libm-consistent f64).
//
// Earlier code used 2.3392794351087596 (derived from an incorrect cf² value);
// that discrepancy was the primary contributor to the LDAERFC 6e-6 tier-1
// failure in Plan 02-04 Wave-1B-14b — see module header.
const DPOL_LEAD_SCALE: f64 = 2.3386662538324523_f64;

#[cube]
fn dpol<F: Float>(rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut rs2 = Array::<F>::new(size);
    ctaylor_mul::<F>(rs, rs, &mut rs2, n);
    let mut inv_rs2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&rs2, &mut inv_rs2, n);
    let mut lead = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_rs2, F::cast_from(DPOL_LEAD_SCALE), &mut lead, n);

    // num_tail = (P3P - 0.454555) * rs
    let mut num_tail = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::cast_from(DPOL_P3P_MINUS), &mut num_tail, n);
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut num_bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &num_tail, &mut num_bracket, n);

    // den_linear = P3P * rs; den_quad = P2P * rs²
    let mut den_linear = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::cast_from(DPOL_P3P), &mut den_linear, n);
    let mut den_quad = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs2, F::cast_from(DPOL_P2P), &mut den_quad, n);
    let mut den_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&den_linear, &den_quad, &mut den_sum, n);
    let mut den_arr = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &den_sum, &mut den_arr, n);

    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den_arr, &mut inv_den, n);
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&num_bracket, &inv_den, &mut ratio, n);

    ctaylor_mul::<F>(&lead, &ratio, out, n);
    // Silence unused-var lint for alias imports.
    let _ = (F::cast_from(DPOL_CF), F::cast_from(DPOL_CF_SQ));
}

// ---------------------------------------------------------------------------
//  g0f(x) = (1 + x*(D0F - 0.7317 + x*(C0F + x*(E0F + F0F*x)))) * exp(-D0F*x) / 2
//  Port of ldaerfc.cpp:46-53.
// ---------------------------------------------------------------------------

#[cube]
fn g0f<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Innermost Horner: (E0F + F0F*x)
    let mut f0f_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, F::cast_from(G0F_F0F), &mut f0f_x, n);
    let mut e0f_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut e0f_const, n);
    e0f_const[0] = F::cast_from(G0F_E0F);
    let mut inner1 = Array::<F>::new(size);
    ctaylor_add::<F>(&e0f_const, &f0f_x, &mut inner1, n);

    // x * inner1
    let mut x_inner1 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &inner1, &mut x_inner1, n);

    // (C0F + x*inner1)
    let mut c0f_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut c0f_const, n);
    c0f_const[0] = F::cast_from(G0F_C0F);
    let mut inner2 = Array::<F>::new(size);
    ctaylor_add::<F>(&c0f_const, &x_inner1, &mut inner2, n);

    // x * inner2
    let mut x_inner2 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &inner2, &mut x_inner2, n);

    // (D0F - 0.7317 + x*inner2)
    let mut d0f_minus_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut d0f_minus_const, n);
    d0f_minus_const[0] = F::cast_from(G0F_D0F_MINUS);
    let mut inner3 = Array::<F>::new(size);
    ctaylor_add::<F>(&d0f_minus_const, &x_inner2, &mut inner3, n);

    // x * inner3
    let mut x_inner3 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, &inner3, &mut x_inner3, n);

    // (1 + x*inner3)
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut poly = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &x_inner3, &mut poly, n);

    // -D0F * x
    let mut neg_d0f_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, F::cast_from(-G0F_D0F), &mut neg_d0f_x, n);

    // exp(-D0F*x)
    let mut exp_val = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_d0f_x, &mut exp_val, n);

    // poly * exp_val
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&poly, &exp_val, &mut prod, n);

    // out = prod / 2 = prod * 0.5
    ctaylor_scalar_mul::<F>(&prod, F::new(0.5), out, n);
}

// ---------------------------------------------------------------------------
//  pow6/pow8 helpers for b0^6, b0^8.
// ---------------------------------------------------------------------------

#[cube]
fn pow6<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut x2 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, x, &mut x2, n);
    let mut x4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&x2, &x2, &mut x4, n);
    ctaylor_mul::<F>(&x4, &x2, out, n);
}

#[cube]
fn pow8<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut x2 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, x, &mut x2, n);
    let mut x4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&x2, &x2, &mut x4, n);
    ctaylor_mul::<F>(&x4, &x4, out, n);
}

// ---------------------------------------------------------------------------
//  ecorrlr — port of ldaerfc.cpp:55-105.
//
//  This is the most complex LDA helper in Phase 2 (~50 ctaylor ops).
//  Operation order matches C++ source line-by-line.
// ---------------------------------------------------------------------------

#[cube]
#[allow(clippy::too_many_arguments)]
fn ecorrlr<F: Float>(d: &DensVarsDev<F>, ec: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mu = F::cast_from(RANGESEP_MU_F64);

    // phi = (pow(1+zeta, 2/3) + pow(1-zeta, 2/3)) / 2
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut one_plus_zeta = Array::<F>::new(size);
    ctaylor_add::<F>(&d.zeta, &one_const, &mut one_plus_zeta, n);
    let mut one_minus_zeta = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, &d.zeta, &mut one_minus_zeta, n);
    let two_thirds = F::cast_from(2.0_f64 / 3.0_f64);
    let mut pow_plus_23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_plus_zeta, two_thirds, &mut pow_plus_23, n);
    let mut pow_minus_23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_minus_zeta, two_thirds, &mut pow_minus_23, n);
    let mut phi_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&pow_plus_23, &pow_minus_23, &mut phi_sum, n);
    let mut phi = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&phi_sum, F::new(0.5), &mut phi, n);

    // b0 = adib * r_s
    let mut b0 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(ECORRLR_ADIB), &mut b0, n);

    // rs2 = r_s * r_s; rs3 = rs2 * r_s
    let mut rs2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &d.r_s, &mut rs2, n);
    let mut rs3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&rs2, &d.r_s, &mut rs3, n);

    // d2anti = (q1a*r_s + q2a*rs2) * exp(-q3a*r_s) / rs2
    let mut q1a_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(ECORRLR_Q1A), &mut q1a_rs, n);
    let mut q2a_rs2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs2, F::cast_from(ECORRLR_Q2A), &mut q2a_rs2, n);
    let mut d2anti_num = Array::<F>::new(size);
    ctaylor_add::<F>(&q1a_rs, &q2a_rs2, &mut d2anti_num, n);
    let mut neg_q3a_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(-ECORRLR_Q3A), &mut neg_q3a_rs, n);
    let mut exp_q3a = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_q3a_rs, &mut exp_q3a, n);
    let mut d2anti_num_exp = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2anti_num, &exp_q3a, &mut d2anti_num_exp, n);
    let mut inv_rs2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&rs2, &mut inv_rs2, n);
    let mut d2anti = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2anti_num_exp, &inv_rs2, &mut d2anti, n);

    // d3anti = (t1a*r_s + t2a*rs2) * exp(-t3a*r_s) / rs3
    let mut t1a_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(ECORRLR_T1A), &mut t1a_rs, n);
    let mut t2a_rs2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs2, F::cast_from(ECORRLR_T2A), &mut t2a_rs2, n);
    let mut d3anti_num = Array::<F>::new(size);
    ctaylor_add::<F>(&t1a_rs, &t2a_rs2, &mut d3anti_num, n);
    let mut neg_t3a_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(-ECORRLR_T3A), &mut neg_t3a_rs, n);
    let mut exp_t3a = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_t3a_rs, &mut exp_t3a, n);
    let mut d3anti_num_exp = Array::<F>::new(size);
    ctaylor_mul::<F>(&d3anti_num, &exp_t3a, &mut d3anti_num_exp, n);
    let mut inv_rs3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&rs3, &mut inv_rs3, n);
    let mut d3anti = Array::<F>::new(size);
    ctaylor_mul::<F>(&d3anti_num_exp, &inv_rs3, &mut d3anti, n);

    // z = zeta; z2 = zeta * zeta
    let mut z2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.zeta, &d.zeta, &mut z2, n);

    // coe2 = -3/(8*rs3) * (1 - z²) * (g0f(r_s) - 0.5)
    //   step 1: one_m_z2 = 1 - z²
    let mut one_m_z2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, &z2, &mut one_m_z2, n);
    //   step 2: g0_rs = g0f(r_s)
    let mut g0_rs = Array::<F>::new(size);
    g0f::<F>(&d.r_s, &mut g0_rs, n);
    //   step 3: g0_m_half = g0_rs - 0.5
    let mut half_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut half_const, n);
    half_const[0] = F::new(0.5);
    let mut g0_m_half = Array::<F>::new(size);
    ctaylor_sub::<F>(&g0_rs, &half_const, &mut g0_m_half, n);
    //   step 4: coe2_num = one_m_z2 * g0_m_half
    let mut coe2_num = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_m_z2, &g0_m_half, &mut coe2_num, n);
    //   step 5: -3/(8*rs3) = -3/8 * (1/rs3)
    let mut coe2_prescale = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_rs3, F::cast_from(-0.375_f64), &mut coe2_prescale, n);
    //   step 6: coe2 = coe2_prescale * coe2_num
    let mut coe2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&coe2_prescale, &coe2_num, &mut coe2, n);

    // coe3 = -(1 - z²) * g0f(r_s) / (sqrt(2π) * rs3)
    //   sign/scale: -1/sqrt(2π) = -0.398942...
    // -1/sqrt(2π) = -0.3989422804014327
    let inv_sqrt_2pi_neg = F::cast_from(-0.3989422804014327_f64);
    let mut coe3_num = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_m_z2, &g0_rs, &mut coe3_num, n);
    let mut coe3_scaled = Array::<F>::new(size);
    ctaylor_mul::<F>(&coe3_num, &inv_rs3, &mut coe3_scaled, n);
    let mut coe3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&coe3_scaled, inv_sqrt_2pi_neg, &mut coe3, n);

    // coe4 = -9/(64*rs3) * (
    //   ((1+z)/2)² * dpol(rs * pow(2/(1+z), 1/3))
    //   + ((1-z)/2)² * dpol(rs * pow(2/(1-z), 1/3))
    //   + (1 - z²) * d2anti
    //   - cf²/10 * (pow(1+z, 8/3) + pow(1-z, 8/3)) / rs²
    // )

    // ((1+z)/2)² and ((1-z)/2)²
    let mut half_one_plus_z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&one_plus_zeta, F::new(0.5), &mut half_one_plus_z, n);
    let mut half_one_minus_z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&one_minus_zeta, F::new(0.5), &mut half_one_minus_z, n);
    let mut half_plus_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&half_one_plus_z, &half_one_plus_z, &mut half_plus_sq, n);
    let mut half_minus_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&half_one_minus_z, &half_one_minus_z, &mut half_minus_sq, n);

    // pow(2/(1+z), 1/3) and pow(2/(1-z), 1/3)
    // 2/(1+z) = 2 * 1/(1+z)
    let mut inv_one_plus_zeta = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_plus_zeta, &mut inv_one_plus_zeta, n);
    let mut two_over_1pz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_one_plus_zeta, F::new(2.0), &mut two_over_1pz, n);
    let one_third = F::cast_from(1.0_f64 / 3.0_f64);
    let mut pow_2_1pz_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(&two_over_1pz, one_third, &mut pow_2_1pz_13, n);

    let mut inv_one_minus_zeta = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_minus_zeta, &mut inv_one_minus_zeta, n);
    let mut two_over_1mz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_one_minus_zeta, F::new(2.0), &mut two_over_1mz, n);
    let mut pow_2_1mz_13 = Array::<F>::new(size);
    ctaylor_pow::<F>(&two_over_1mz, one_third, &mut pow_2_1mz_13, n);

    // rs * pow(...)
    let mut rs_pow_plus = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &pow_2_1pz_13, &mut rs_pow_plus, n);
    let mut rs_pow_minus = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &pow_2_1mz_13, &mut rs_pow_minus, n);

    // dpol(rs * pow(...))
    let mut dpol_plus = Array::<F>::new(size);
    dpol::<F>(&rs_pow_plus, &mut dpol_plus, n);
    let mut dpol_minus = Array::<F>::new(size);
    dpol::<F>(&rs_pow_minus, &mut dpol_minus, n);

    // First term: ((1+z)/2)² * dpol_plus
    let mut term1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&half_plus_sq, &dpol_plus, &mut term1, n);
    // Second term: ((1-z)/2)² * dpol_minus
    let mut term2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&half_minus_sq, &dpol_minus, &mut term2, n);
    // Third term: (1-z²) * d2anti
    let mut term3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_m_z2, &d2anti, &mut term3, n);

    // Fourth term: -cf²/10 * (pow(1+z, 8/3) + pow(1-z, 8/3)) / rs²
    let eight_thirds = F::cast_from(8.0_f64 / 3.0_f64);
    let mut pow_1pz_83 = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_plus_zeta, eight_thirds, &mut pow_1pz_83, n);
    let mut pow_1mz_83 = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_minus_zeta, eight_thirds, &mut pow_1mz_83, n);
    let mut sum_pow_83 = Array::<F>::new(size);
    ctaylor_add::<F>(&pow_1pz_83, &pow_1mz_83, &mut sum_pow_83, n);
    // * (-cf²/10) = -0.368318 (approximately)
    let neg_cf2_over_10 = F::cast_from(-ECORRLR_CF_SQ * 0.1_f64);
    let mut scaled_pow_83 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sum_pow_83, neg_cf2_over_10, &mut scaled_pow_83, n);
    let mut term4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&scaled_pow_83, &inv_rs2, &mut term4, n);

    // coe4_bracket = term1 + term2 + term3 + term4
    //   (C++ left-to-right: t1 + t2 + t3 + t4; order of ops matters due to rounding.)
    let mut sum12 = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &term2, &mut sum12, n);
    let mut sum123 = Array::<F>::new(size);
    ctaylor_add::<F>(&sum12, &term3, &mut sum123, n);
    let mut coe4_bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&sum123, &term4, &mut coe4_bracket, n);

    // coe4 = -9/(64*rs3) * coe4_bracket = -9/64 * (1/rs3) * coe4_bracket
    let mut coe4_prescale = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &inv_rs3,
        F::cast_from(-9.0_f64 / 64.0_f64),
        &mut coe4_prescale,
        n,
    );
    let mut coe4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&coe4_prescale, &coe4_bracket, &mut coe4, n);

    // coe5 = -9/(40*sqrt(2π)*rs3) * (
    //   ((1+z)/2)² * dpol_plus
    //   + ((1-z)/2)² * dpol_minus
    //   + (1 - z²) * d3anti
    // )
    let mut term3b = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_m_z2, &d3anti, &mut term3b, n);
    let mut sum12b = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &term2, &mut sum12b, n);
    let mut coe5_bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&sum12b, &term3b, &mut coe5_bracket, n);
    // -9 / (40 * sqrt(2π))
    //   sqrt(2π) = 2.5066282746310002
    //   40 * sqrt(2π) = 100.2651309852400
    //   -9 / 100.2651309852400 = -0.08976201309032236 (libm-consistent f64)
    // (earlier value -0.08976231703841775 was off by ~3.4e-6 rel — see module header.)
    let coe5_prefactor = F::cast_from(-0.08976201309032236_f64);
    let mut coe5_prescale = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_rs3, coe5_prefactor, &mut coe5_prescale, n);
    let mut coe5 = Array::<F>::new(size);
    ctaylor_mul::<F>(&coe5_prescale, &coe5_bracket, &mut coe5, n);

    // b06 = b0^6; b08 = b0^8
    let mut b06 = Array::<F>::new(size);
    pow6::<F>(&b0, &mut b06, n);
    let mut b08 = Array::<F>::new(size);
    pow8::<F>(&b0, &mut b08, n);

    // b04 = b0^4 (for a2 = 4*b06*coe2 + b08*coe4 + 6*pow(b0,4)*ec)
    let mut b02 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b0, &b0, &mut b02, n);
    let mut b04 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b02, &b02, &mut b04, n);

    // a1 = 4*b06*coe3 + b08*coe5
    let mut four_b06 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&b06, F::new(4.0), &mut four_b06, n);
    let mut four_b06_coe3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&four_b06, &coe3, &mut four_b06_coe3, n);
    let mut b08_coe5 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b08, &coe5, &mut b08_coe5, n);
    let mut a1 = Array::<F>::new(size);
    ctaylor_add::<F>(&four_b06_coe3, &b08_coe5, &mut a1, n);

    // a2 = 4*b06*coe2 + b08*coe4 + 6*b04*ec
    let mut four_b06_coe2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&four_b06, &coe2, &mut four_b06_coe2, n);
    let mut b08_coe4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b08, &coe4, &mut b08_coe4, n);
    let mut six_b04 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&b04, F::new(6.0), &mut six_b04, n);
    let mut six_b04_ec = Array::<F>::new(size);
    ctaylor_mul::<F>(&six_b04, ec, &mut six_b04_ec, n);
    let mut sum_a2_12 = Array::<F>::new(size);
    ctaylor_add::<F>(&four_b06_coe2, &b08_coe4, &mut sum_a2_12, n);
    let mut a2 = Array::<F>::new(size);
    ctaylor_add::<F>(&sum_a2_12, &six_b04_ec, &mut a2, n);

    // a3 = b08 * coe3
    let mut a3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b08, &coe3, &mut a3, n);

    // a4 = b06 * (b02*coe2 + 4*ec)
    let mut b02_coe2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b02, &coe2, &mut b02_coe2, n);
    let mut four_ec = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(ec, F::new(4.0), &mut four_ec, n);
    let mut a4_inner = Array::<F>::new(size);
    ctaylor_add::<F>(&b02_coe2, &four_ec, &mut a4_inner, n);
    let mut a4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&b06, &a4_inner, &mut a4, n);

    // Qrpa(mu*sqrt(r_s)/phi) — scalar mu, series r_s and phi.
    //   sqrt_rs   = sqrt(r_s)
    //   mu_sqrt_rs = mu * sqrt_rs           (scalar_mul by mu)
    //   inv_phi   = 1/phi
    //   q_arg     = mu_sqrt_rs * inv_phi
    //   q_val     = Qrpa(q_arg)
    let mut sqrt_rs = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&d.r_s, &mut sqrt_rs, n);
    let mut mu_sqrt_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_rs, mu, &mut mu_sqrt_rs, n);
    let mut inv_phi = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&phi, &mut inv_phi, n);
    let mut q_arg = Array::<F>::new(size);
    ctaylor_mul::<F>(&mu_sqrt_rs, &inv_phi, &mut q_arg, n);
    let mut q_val = Array::<F>::new(size);
    qrpa::<F>(&q_arg, &mut q_val, n);

    // phi³
    let mut phi2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&phi, &phi, &mut phi2, n);
    let mut phi3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&phi2, &phi, &mut phi3, n);

    // phi³ * Qrpa
    let mut phi3_qrpa = Array::<F>::new(size);
    ctaylor_mul::<F>(&phi3, &q_val, &mut phi3_qrpa, n);

    // Scalar powers of mu (mu is constant — precomputable).
    let mu2 = RANGESEP_MU_F64 * RANGESEP_MU_F64;
    let mu3 = mu2 * RANGESEP_MU_F64;
    let mu4 = mu2 * mu2;
    let mu5 = mu4 * RANGESEP_MU_F64;
    let mu6 = mu4 * mu2;
    let mu8 = mu4 * mu4;

    // a1 * mu³, a2 * mu⁴, a3 * mu⁵, a4 * mu⁶
    let mut a1_mu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a1, F::cast_from(mu3), &mut a1_mu3, n);
    let mut a2_mu4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a2, F::cast_from(mu4), &mut a2_mu4, n);
    let mut a3_mu5 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a3, F::cast_from(mu5), &mut a3_mu5, n);
    let mut a4_mu6 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&a4, F::cast_from(mu6), &mut a4_mu6, n);

    // (b0*mu)^8 * ec = b08 * mu^8 * ec
    let mut b08_mu8 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&b08, F::cast_from(mu8), &mut b08_mu8, n);
    let mut b0mu8_ec = Array::<F>::new(size);
    ctaylor_mul::<F>(&b08_mu8, ec, &mut b0mu8_ec, n);

    // Numerator: phi³*Qrpa + a1*mu³ + a2*mu⁴ + a3*mu⁵ + a4*mu⁶ + (b0*mu)^8*ec
    let mut num_s1 = Array::<F>::new(size);
    ctaylor_add::<F>(&phi3_qrpa, &a1_mu3, &mut num_s1, n);
    let mut num_s2 = Array::<F>::new(size);
    ctaylor_add::<F>(&num_s1, &a2_mu4, &mut num_s2, n);
    let mut num_s3 = Array::<F>::new(size);
    ctaylor_add::<F>(&num_s2, &a3_mu5, &mut num_s3, n);
    let mut num_s4 = Array::<F>::new(size);
    ctaylor_add::<F>(&num_s3, &a4_mu6, &mut num_s4, n);
    let mut numer = Array::<F>::new(size);
    ctaylor_add::<F>(&num_s4, &b0mu8_ec, &mut numer, n);

    // Denominator: (1 + (b0*mu)²)^4
    //   b0mu2 = b0² * mu² = b02 * mu²
    let mut b02_mu2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&b02, F::cast_from(mu2), &mut b02_mu2, n);
    let mut one_plus = Array::<F>::new(size);
    ctaylor_add::<F>(&one_const, &b02_mu2, &mut one_plus, n);
    let mut opm2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_plus, &one_plus, &mut opm2, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&opm2, &opm2, &mut denom, n);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    ctaylor_mul::<F>(&numer, &inv_denom, out, n);

    // Silence unused constants.
    let _ = (
        F::cast_from(ECORRLR_ALPHA),
        F::cast_from(ECORRLR_CF),
        F::cast_from(SQRT_TWO_PI),
    );
}

/// Short-range LDA correlation kernel. 1:1 port of `ldaerfc.cpp:106-110`:
/// `return d.n * (eps - ecorrlr(d, mu, eps));` where eps = pw92eps(d).
#[cube]
pub fn ldaerfc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92eps(d)
    let mut eps = Array::<F>::new(size);
    pw92_eps::<F>(d, &mut eps, n);

    // ecorrlr_val = ecorrlr(d, mu, eps)
    let mut ecorrlr_val = Array::<F>::new(size);
    ecorrlr::<F>(d, &eps, &mut ecorrlr_val, n);

    // inner = eps - ecorrlr_val
    let mut inner = Array::<F>::new(size);
    ctaylor_sub::<F>(&eps, &ecorrlr_val, &mut inner, n);

    // out = d.n * inner
    ctaylor_mul::<F>(&d.n, &inner, out, n);
}
