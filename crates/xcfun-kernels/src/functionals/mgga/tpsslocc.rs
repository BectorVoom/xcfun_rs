//! XC_TPSSLOCC — TPSSloc correlation functional. MGGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/tpsslocc.cpp`
//!
//! # Description
//! TPSSloc correlation uses PBEloc (local PBE with position-dependent beta0)
//! instead of standard PBE correlation. Port is verbatim line-for-line.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

#![allow(non_snake_case)]

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_2, ctaylor_powi_3,
    ctaylor_reciprocal, ctaylor_sqrt,
};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::PBEC_D2_PREFACTOR_F64;
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;
use crate::functionals::mgga::shared::tpss_like::ctaylor_max;

// param_gamma = (1-log(2))/π² = 0.031090690869654895
const PBEC_GAMMA: f64 = 0.031_090_690_869_654_9_f64;

/// `phi(d) = 2^(-1/3) * n_m13^2 * (sqrt(a_43) + sqrt(b_43))`.
/// Port of `tpsslocc.cpp:20-22` (same as pbec_eps phi_reorganised).
#[cube]
fn locc_phi<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, out, n);
}

/// `pbeloc_eps(d)` — PBEloc correlation energy per particle.
/// Port of `tpsslocc.cpp:24-41`.
/// Uses beta = beta0 + aa*d2*ff where ff = 1 - exp(-r_s^2).
#[cube]
fn pbeloc_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    const BETA0: f64 = 0.0375_f64;
    const AA: f64 = 0.08_f64;

    let mut u = Array::<F>::new(size);
    locc_phi::<F>(d, &mut u, n);

    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);

    // d2 = PREFACTOR * gnn / (u^2 * n^(7/3))
    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // ff = 1 - exp(-r_s^2)
    let mut rs_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.r_s, &mut rs_sq, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_rs_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs_sq, neg_one, &mut neg_rs_sq, n);
    let mut exp_neg = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_rs_sq, &mut exp_neg, n);
    // ff = 1 - exp(...)
    let mut ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&exp_neg, neg_one, &mut ff, n);
    ff[0] = ff[0] + F::new(1.0);

    // beta = beta0 + aa*d2*ff
    let mut d2_ff = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &ff, &mut d2_ff, n);
    let mut aa_d2_ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2_ff, F::cast_from(AA), &mut aa_d2_ff, n);
    // beta as CTaylor starting from scalar beta0
    let mut beta = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        beta[i] = aa_d2_ff[i];
    }
    beta[0] = beta[0] + F::cast_from(BETA0);

    // bg = beta / gamma
    let mut bg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&beta, F::cast_from(1.0_f64 / PBEC_GAMMA), &mut bg, n);

    // eps = pw92_eps(d)
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // A = bg / expm1(-eps / (gamma*u3))
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&u3, F::cast_from(PBEC_GAMMA), &mut gu3, n);
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps, &inv_gu3, &mut prod, n);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);
    let mut A = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &inv_em1, &mut A, n);

    // d2A = d2 * A
    let mut d2A = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &A, &mut d2A, n);

    // 1 + d2A
    let mut one_d2A = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2A[i] = d2A[i];
    }
    one_d2A[0] = one_d2A[0] + F::new(1.0);

    // d2A * (1 + d2A)
    let mut d2A_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2A, &one_d2A, &mut d2A_sq, n);

    // 1 + d2A*(1+d2A)
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = d2A_sq[i];
    }
    den[0] = den[0] + F::new(1.0);

    // bg * d2 * (1+d2A)
    let mut bg_d2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &d2, &mut bg_d2, n);
    let mut num_h = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg_d2, &one_d2A, &mut num_h, n);

    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num_h, &inv_den, &mut frac, n);
    frac[0] = frac[0] + F::new(1.0);

    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&frac, &mut lg, n);

    let mut H = Array::<F>::new(size);
    ctaylor_mul::<F>(&gu3, &lg, &mut H, n);

    ctaylor_add::<F>(&eps, &H, out, n);
}

/// `pbeloc_eps_pola(a, gaa)` — fully polarized PBEloc eps.
/// Port of `tpsslocc.cpp:43-61`.
#[cube]
fn pbeloc_eps_pola<F: Float>(a: &Array<F>, gaa: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    const BETA0: f64 = 0.0375_f64;
    const AA: f64 = 0.08_f64;

    // u = 2^(-1/3) (fully polarized phi)
    const PHI_POLAR: f64 = 0.793_700_525_984_099_8_f64;
    let mut u = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        u[i] = F::new(0.0);
    }
    u[0] = F::cast_from(PHI_POLAR);

    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);
    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);

    // d2 = PREFACTOR * gaa / (u^2 * a^(7/3))
    let mut a_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(a, F::cast_from(7.0_f64 / 3.0_f64), &mut a_73, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &a_73, &mut denom, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_denom, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // rs = (3/(4π))^(1/3) * a^(-1/3)
    const RS_PREF: f64 = 0.620_350_490_899_400_1_f64;
    let mut rs_raw = Array::<F>::new(size);
    ctaylor_pow::<F>(a, F::cast_from(-1.0_f64 / 3.0_f64), &mut rs_raw, n);
    let mut rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs_raw, F::cast_from(RS_PREF), &mut rs, n);

    // ff = 1 - exp(-rs^2)
    let mut rs_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&rs, &mut rs_sq, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_rs_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs_sq, neg_one, &mut neg_rs_sq, n);
    let mut exp_neg = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_rs_sq, &mut exp_neg, n);
    let mut ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&exp_neg, neg_one, &mut ff, n);
    ff[0] = ff[0] + F::new(1.0);

    // beta = beta0 + aa*d2*ff
    let mut d2_ff = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &ff, &mut d2_ff, n);
    let mut aa_d2_ff = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2_ff, F::cast_from(AA), &mut aa_d2_ff, n);
    let mut beta = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        beta[i] = aa_d2_ff[i];
    }
    beta[0] = beta[0] + F::cast_from(BETA0);

    let mut bg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&beta, F::cast_from(1.0_f64 / PBEC_GAMMA), &mut bg, n);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(a, &mut eps, n);

    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&u3, F::cast_from(PBEC_GAMMA), &mut gu3, n);
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps, &inv_gu3, &mut prod, n);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);
    let mut A = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &inv_em1, &mut A, n);

    let mut d2A = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &A, &mut d2A, n);
    let mut one_d2A = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2A[i] = d2A[i];
    }
    one_d2A[0] = one_d2A[0] + F::new(1.0);
    let mut d2A_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2A, &one_d2A, &mut d2A_sq, n);
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = d2A_sq[i];
    }
    den[0] = den[0] + F::new(1.0);
    let mut bg_d2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg, &d2, &mut bg_d2, n);
    let mut num_h = Array::<F>::new(size);
    ctaylor_mul::<F>(&bg_d2, &one_d2A, &mut num_h, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num_h, &inv_den, &mut frac, n);
    frac[0] = frac[0] + F::new(1.0);
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&frac, &mut lg, n);
    let mut H = Array::<F>::new(size);
    ctaylor_mul::<F>(&gu3, &lg, &mut H, n);

    ctaylor_add::<F>(&eps, &H, out, n);
}

/// `C(d)` factor for tpsslocc. Port of `tpsslocc.cpp:63-69`.
/// C0 = 0.35 + 0.87*ζ² + 0.50*ζ⁴ + 2.26*ζ⁶  (Note: 0.35, not 0.53!)
#[cube]
fn locc_C<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // gzeta2 = (n²*gss - 2*n*s*gns + s²*gnn) / n^4
    let mut n2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.n, &mut n2, n);
    let mut t1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&n2, &d.gss, &mut t1, n);
    let mut ns = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &d.s, &mut ns, n);
    let mut t2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ns, &d.gns, &mut t2_raw, n);
    let neg_two = F::new(0.0) - F::new(2.0);
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&t2_raw, neg_two, &mut t2, n);
    let mut s2_arr = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.s, &mut s2_arr, n);
    let mut t3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&s2_arr, &d.gnn, &mut t3, n);
    let mut gzeta2_num1 = Array::<F>::new(size);
    ctaylor_add::<F>(&t1, &t2, &mut gzeta2_num1, n);
    let mut gzeta2_num = Array::<F>::new(size);
    ctaylor_add::<F>(&gzeta2_num1, &t3, &mut gzeta2_num, n);
    let mut n4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&n2, &mut n4, n);
    let mut inv_n4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n4, &mut inv_n4, n);
    let mut gzeta2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2_num, &inv_n4, &mut gzeta2, n);

    // xi2
    const FOUR_3PI2_23: f64 = 38.283_120_002_509_214_f64;
    let mut n_m23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-2.0_f64 / 3.0_f64), &mut n_m23, n);
    let mut xi2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&gzeta2, &n_m23, &mut xi2_raw, n);
    let mut xi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_raw, F::cast_from(1.0_f64 / FOUR_3PI2_23), &mut xi2, n);

    // C0 = 0.35 + 0.87*z2 + 0.50*z4 + 2.26*z6
    let mut z2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.zeta, &mut z2, n);
    let mut z4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&z2, &mut z4, n);
    let mut z6 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z4, &z2, &mut z6, n);
    let mut t87z2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z2, F::cast_from(0.87_f64), &mut t87z2, n);
    let mut t50z4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z4, F::cast_from(0.50_f64), &mut t50z4, n);
    let mut t226z6 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z6, F::cast_from(2.26_f64), &mut t226z6, n);
    let mut C0_raw = Array::<F>::new(size);
    ctaylor_add::<F>(&t87z2, &t50z4, &mut C0_raw, n);
    let mut C0 = Array::<F>::new(size);
    ctaylor_add::<F>(&C0_raw, &t226z6, &mut C0, n);
    C0[0] = C0[0] + F::cast_from(0.35_f64);

    // ufunc(zeta, -4/3) via explicit pow
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut pz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        pz[i] = d.zeta[i];
    }
    pz[0] = pz[0] + F::new(1.0);
    let mut mz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.zeta, neg_one, &mut mz, n);
    mz[0] = mz[0] + F::new(1.0);
    let mut pz_pow = Array::<F>::new(size);
    ctaylor_pow::<F>(&pz, F::cast_from(-4.0_f64 / 3.0_f64), &mut pz_pow, n);
    let mut mz_pow = Array::<F>::new(size);
    ctaylor_pow::<F>(&mz, F::cast_from(-4.0_f64 / 3.0_f64), &mut mz_pow, n);
    let mut uf = Array::<F>::new(size);
    ctaylor_add::<F>(&pz_pow, &mz_pow, &mut uf, n);

    // 0.5 * xi2 * ufunc
    let mut xi2_uf_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&xi2, &uf, &mut xi2_uf_raw, n);
    let mut xi2_uf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xi2_uf_raw, F::cast_from(0.5_f64), &mut xi2_uf, n);
    xi2_uf[0] = xi2_uf[0] + F::new(1.0);

    // (...)^(-4)
    let mut pow4 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&xi2_uf, &mut pow4, n);
    let mut pow4b = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&pow4, &mut pow4b, n);
    let mut inv_pow4 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&pow4b, &mut inv_pow4, n);

    ctaylor_mul::<F>(&C0, &inv_pow4, out, n);
}

/// `epsc_summax(d)`. Port of `tpsslocc.cpp:71-82`.
#[cube]
fn locc_epsc_summax<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut epsc_pbeloc = Array::<F>::new(size);
    pbeloc_eps::<F>(d, &mut epsc_pbeloc, n);

    let mut epsc_pbeloc_a = Array::<F>::new(size);
    pbeloc_eps_pola::<F>(&d.a, &d.gaa, &mut epsc_pbeloc_a, n);

    let mut epsc_pbeloc_b = Array::<F>::new(size);
    pbeloc_eps_pola::<F>(&d.b, &d.gbb, &mut epsc_pbeloc_b, n);

    let mut max_a = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbeloc, &epsc_pbeloc_a, &mut max_a, n);

    let mut max_b = Array::<F>::new(size);
    ctaylor_max::<F>(&epsc_pbeloc, &epsc_pbeloc_b, &mut max_b, n);

    let mut a_max = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &max_a, &mut a_max, n);

    let mut b_max = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &max_b, &mut b_max, n);

    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&a_max, &b_max, &mut sum, n);

    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    ctaylor_mul::<F>(&sum, &inv_n, out, n);
}

/// `epsc_revpkzb(d)`. Port of `tpsslocc.cpp:84-90`.
///
/// Phase 6 D-10 — takes an explicit `tau` parameter so the kernel-body
/// clamp `ctaylor_max(d.tau, tau_w)` flows through. Body otherwise
/// line-for-line identical to the original (Plan 04-10 Path-B-confirmed
/// algorithmically faithful port).
#[cube]
fn locc_epsc_revpkzb_with_tau<F: Float>(
    d: &DensVarsDev<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // tauwtau2 = (gnn / (8*n*tau))^2 — uses explicit `tau` (clamped).
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, tau, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);
    let mut inv_8nt = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&eight_n_tau, &mut inv_8nt, n);
    let mut gnn_8nt = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_8nt, &mut gnn_8nt, n);
    let mut tauwtau2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&gnn_8nt, &mut tauwtau2, n);

    let mut epsc_sum = Array::<F>::new(size);
    locc_epsc_summax::<F>(d, &mut epsc_sum, n);

    let mut epsc_pbeloc = Array::<F>::new(size);
    pbeloc_eps::<F>(d, &mut epsc_pbeloc, n);

    let mut CC = Array::<F>::new(size);
    locc_C::<F>(d, &mut CC, n);

    // epsc_pbeloc * (1 + CC*t2)
    let mut CC_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&CC, &tauwtau2, &mut CC_t2, n);
    let mut pbe_CC_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&epsc_pbeloc, &CC_t2, &mut pbe_CC_t2, n);
    let mut lhs = Array::<F>::new(size);
    ctaylor_add::<F>(&epsc_pbeloc, &pbe_CC_t2, &mut lhs, n);

    // (1 + CC)*t2*epsc_sum
    let mut one_CC = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_CC[i] = CC[i];
    }
    one_CC[0] = one_CC[0] + F::new(1.0);
    let mut one_CC_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_CC, &tauwtau2, &mut one_CC_t2, n);
    let mut rhs = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_CC_t2, &epsc_sum, &mut rhs, n);

    ctaylor_sub::<F>(&lhs, &rhs, out, n);
}

/// `energy(d)`. Port of `tpsslocc.cpp:92-97`.
#[cube]
pub fn tpsslocc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Phase 6 D-10 — hard-clamp tau to tau_w. See `tpssc.rs` for rationale.
    // The clamped tau flows into BOTH the inner `locc_epsc_revpkzb_with_tau`
    // helper AND the outer `tauwtau3 = (gnn/(8 n tau))^3` factor here.
    let mut tau_w = Array::<F>::new(size);
    crate::functionals::mgga::shared::tpss_like::build_tau_w::<F>(d, &mut tau_w, n);
    let mut tau_clamped = Array::<F>::new(size);
    crate::functionals::mgga::shared::tpss_like::ctaylor_max::<F>(
        &d.tau,
        &tau_w,
        &mut tau_clamped,
        n,
    );

    let mut eps_pkzb = Array::<F>::new(size);
    locc_epsc_revpkzb_with_tau::<F>(d, &tau_clamped, &mut eps_pkzb, n);

    // tauwtau3 = (gnn / (8*n*tau))^3 — uses clamped tau.
    let mut eight_n_tau_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &tau_clamped, &mut eight_n_tau_raw, n);
    let mut eight_n_tau = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eight_n_tau_raw, F::cast_from(8.0_f64), &mut eight_n_tau, n);
    let mut inv_8nt = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&eight_n_tau, &mut inv_8nt, n);
    let mut gnn_8nt = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_8nt, &mut gnn_8nt, n);
    let mut tauwtau3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&gnn_8nt, &mut tauwtau3, n);

    // dd = 4.5
    const DD: f64 = 4.5_f64;

    // n * eps_pkzb * (1 + dd * eps_pkzb * tauwtau3)
    let mut dd_eps_t3_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_pkzb, &tauwtau3, &mut dd_eps_t3_raw, n);
    let mut dd_eps_t3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&dd_eps_t3_raw, F::cast_from(DD), &mut dd_eps_t3, n);
    dd_eps_t3[0] = dd_eps_t3[0] + F::new(1.0);

    let mut n_eps = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &eps_pkzb, &mut n_eps, n);
    ctaylor_mul::<F>(&n_eps, &dd_eps_t3, out, n);
}
