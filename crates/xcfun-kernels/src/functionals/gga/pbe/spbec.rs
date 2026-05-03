//! XC_SPBEC — Simplified PBE correlation. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/spbec.cpp:21-45`
//!
//! # Formula
//! ```cpp
//! eps = vwn::vwn5_eps(d);
//! p   = phi(d);
//! t2  = (cbrt(M_PI/3) / 16) · gnn · n_m13 / pow2(p · n);
//! return n · (eps + H_spbe(t2, eps, p³));
//! ```
//! W5 audit: `cbrt(M_PI/3)/16` = `PBEC_D2_PREFACTOR_F64` numerically; we keep
//! the `cbrt`-derived constant per ACC-06 algorithmic-identity rules.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_expm1, ctaylor_log, ctaylor_powi_2, ctaylor_powi_3, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::PBEC_D2_PREFACTOR_F64;
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::vwn_eps;

// SPBEC paper constants (spbec.cpp:22-24). Note: β = 0.031091 is
// referenced only via the precomputed β/γ ratio; we keep the literal
// 0.466006... to avoid ULP drift from a runtime division.
const SPBEC_GAMM_F64: f64 = 0.066725_f64;
const SPBEC_BETA_GAMMA_F64: f64 = 0.466_006_366_055_452_4_f64; // 0.031091 / 0.066725

/// W5: We use the same numerical value as PBEC_D2_PREFACTOR_F64
/// (= cbrt(π/3) / 16 = 0.06346820609770369). Algebraically identical
/// to (1/12 · 3^(5/6) / π^(-1/6))^2.
const SPBEC_T2_PREFACTOR_F64: f64 = PBEC_D2_PREFACTOR_F64;

/// Local `H_spbe(t2, eps, phi3)` per spbec.cpp:30-33:
/// ```cpp
/// G = beta_gamma / expm1(-eps / (beta_gamma · phi3))
/// return gamm · phi3 · log(1 + beta_gamma · t2 / (1 + t2 · G))
/// ```
#[cube]
fn h_spbe<F: Float>(
    t2: &Array<F>,
    eps: &Array<F>,
    phi3: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step: bg_phi3 = beta_gamma · phi3.
    let mut bg_phi3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(phi3, F::cast_from(SPBEC_BETA_GAMMA_F64), &mut bg_phi3, n);

    // Step: inv_bg_phi3 = 1 / bg_phi3.
    let mut inv_bg_phi3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&bg_phi3, &mut inv_bg_phi3, n);

    // Step: arg_neg = eps · inv_bg_phi3.
    let mut arg_pos = Array::<F>::new(size);
    ctaylor_mul::<F>(eps, &inv_bg_phi3, &mut arg_pos, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&arg_pos, neg_one, &mut arg, n);

    // Step: em1 = expm1(arg).
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);

    // Step: inv_em1 = 1 / em1.
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);

    // Step: g = beta_gamma · inv_em1.
    let mut g = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_em1, F::cast_from(SPBEC_BETA_GAMMA_F64), &mut g, n);

    // Step: t2g = t2 · g.
    let mut t2g = Array::<F>::new(size);
    ctaylor_mul::<F>(t2, &g, &mut t2g, n);

    // Step: one_t2g = 1 + t2g.
    let mut one_t2g = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_t2g[i] = t2g[i];
    }
    one_t2g[0] = one_t2g[0] + F::new(1.0);

    // Step: bg_t2 = beta_gamma · t2.
    let mut bg_t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(t2, F::cast_from(SPBEC_BETA_GAMMA_F64), &mut bg_t2, n);

    // Step: inv_one_t2g = 1 / one_t2g.
    let mut inv_one_t2g = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_t2g, &mut inv_one_t2g, n);

    // Step: frac = bg_t2 · inv_one_t2g.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg_t2, &inv_one_t2g, &mut frac, n);

    // Step: lg_arg = 1 + frac.
    let mut lg_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        lg_arg[i] = frac[i];
    }
    lg_arg[0] = lg_arg[0] + F::new(1.0);

    // Step: lg = log(lg_arg).
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&lg_arg, &mut lg, n);

    // Step: gam_phi3 = gamm · phi3.
    let mut gam_phi3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(phi3, F::cast_from(SPBEC_GAMM_F64), &mut gam_phi3, n);

    // Step: out = gam_phi3 · lg.
    ctaylor_mul::<F>(&gam_phi3, &lg, out, n);
}

#[cube]
pub fn spbec_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = vwn5_eps(d).
    let mut eps = Array::<F>::new(size);
    vwn_eps::vwn5_eps::<F>(d, &mut eps, n);

    // p = phi_reorganised.
    let mut p = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut p, n);

    // p3 = p^3.
    let mut p3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&p, &mut p3, n);

    // pn = p · n.
    let mut pn = Array::<F>::new(size);
    ctaylor_mul::<F>(&p, &d.n, &mut pn, n);

    // pn2 = (p · n)^2.
    let mut pn2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&pn, &mut pn2, n);

    // inv_pn2 = 1 / pn2.
    let mut inv_pn2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&pn2, &mut inv_pn2, n);

    // gnn_nm13 = gnn · n_m13.
    let mut gnn_nm13 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &d.n_m13, &mut gnn_nm13, n);

    // ratio = gnn_nm13 · inv_pn2.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(&gnn_nm13, &inv_pn2, &mut ratio, n);

    // t2 = SPBEC_T2_PREFACTOR · ratio.
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ratio, F::cast_from(SPBEC_T2_PREFACTOR_F64), &mut t2, n);

    // h = H_spbe(t2, eps, p^3).
    let mut h = Array::<F>::new(size);
    h_spbe::<F>(&t2, &eps, &p3, &mut h, n);

    // sum = eps + h.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &h, &mut sum, n);

    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum, out, n);
}
