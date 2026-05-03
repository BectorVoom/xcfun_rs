//! CSC (Colle-Salvetti correlation) helper.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A.
//! Phase 4 plan 04-01 Wave 1 — FULL BODY shipped here.
//!
//! # Source
//! - `xcfun-master/src/functionals/cs.cpp:17-27` — `csc(d)` energy body.
//!
//! # Formula (port of `cs.cpp:17-27`)
//!
//! ```cpp
//! template <typename num> static num csc(const densvars<num> & d) {
//!   parameter a = 1.0;
//!   parameter b = 1.0;
//!   parameter c = 1.0;
//!   parameter dpar = 1.0;
//!   num gamma = 2 * (1 - (d.a*d.a + d.b*d.b) / (d.n*d.n));
//!   num curv = d.a*d.taua + d.b*d.taub - (1.0/8.0)*d.gnn - (d.jpaa + d.jpbb);
//!   return -a * gamma *
//!          (d.n + 2*b*pow(d.n, -5.0/3.0) * curv * exp(-c*d.n_m13)) /
//!          (1 + dpar*d.n_m13);
//! }
//! ```
//!
//! All four parameters (a, b, c, dpar) are 1.0 per the C++ source.
//! Constants folded to literals per ACC-04.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;

/// CSC correlation energy density.
///
/// Port of `xcfun-master/src/functionals/cs.cpp:17-27`.
/// Parameters a = b = c = dpar = 1.0.
#[cube]
pub fn csc_energy<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // --- gamma = 2 * (1 - (a^2 + b^2) / n^2) ---
    let mut a2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &d.a, &mut a2, n);

    let mut b2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &d.b, &mut b2, n);

    let mut a2_plus_b2 = Array::<F>::new(size);
    ctaylor_add::<F>(&a2, &b2, &mut a2_plus_b2, n);

    let mut n2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.n, &mut n2, n);

    let mut inv_n2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n2, &mut inv_n2, n);

    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&a2_plus_b2, &inv_n2, &mut frac, n);

    // gamma_half = 1 - frac
    let mut gamma_half = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        gamma_half[i] = -frac[i];
    }
    gamma_half[0] = gamma_half[0] + F::new(1.0);

    // gamma = 2 * gamma_half
    let mut gamma = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&gamma_half, F::cast_from(2.0_f64), &mut gamma, n);

    // --- curv = a*taua + b*taub - (1/8)*gnn - (jpaa + jpbb) ---
    let mut a_taua = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &d.taua, &mut a_taua, n);

    let mut b_taub = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &d.taub, &mut b_taub, n);

    let mut sum_ab_tau = Array::<F>::new(size);
    ctaylor_add::<F>(&a_taua, &b_taub, &mut sum_ab_tau, n);

    let mut eighth_gnn = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.gnn, F::cast_from(0.125_f64), &mut eighth_gnn, n);

    let mut jp_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&d.jpaa, &d.jpbb, &mut jp_sum, n);

    // curv = sum_ab_tau - eighth_gnn - jp_sum
    let mut curv_raw = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        curv_raw[i] = sum_ab_tau[i] - eighth_gnn[i];
    }
    let mut curv = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        curv[i] = curv_raw[i] - jp_sum[i];
    }

    // --- n^(-5/3) ---
    let mut n_neg53 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-5.0_f64 / 3.0_f64), &mut n_neg53, n);

    // --- exp(-c * n_m13) = exp(-n_m13) (c=1.0) ---
    let mut neg_nm13 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        neg_nm13[i] = -d.n_m13[i];
    }
    let mut exp_neg_nm13 = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_nm13, &mut exp_neg_nm13, n);

    // --- inner bracket: n + 2*b*n^(-5/3)*curv*exp(-n_m13) ---
    // (b = 1.0)
    // term2 = 2 * n^(-5/3) * curv * exp(-n_m13)
    let mut n53_curv_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&n_neg53, &curv, &mut n53_curv_raw, n);

    let mut n53_curv_exp_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&n53_curv_raw, &exp_neg_nm13, &mut n53_curv_exp_raw, n);

    let mut term2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n53_curv_exp_raw, F::cast_from(2.0_f64), &mut term2, n);

    // bracket = n + term2
    let mut bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&d.n, &term2, &mut bracket, n);

    // --- denominator: 1 + dpar*n_m13 = 1 + n_m13 (dpar=1.0) ---
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = d.n_m13[i];
    }
    denom[0] = denom[0] + F::new(1.0);

    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // --- result: -a * gamma * bracket / denom = -gamma * bracket / denom (a=1.0) ---
    let mut gamma_bracket_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&gamma, &bracket, &mut gamma_bracket_raw, n);

    let mut gamma_bracket_div = Array::<F>::new(size);
    ctaylor_mul::<F>(&gamma_bracket_raw, &inv_denom, &mut gamma_bracket_div, n);

    // negate: out = -gamma_bracket_div
    #[unroll]
    for i in 0..size {
        out[i] = -gamma_bracket_div[i];
    }
}
