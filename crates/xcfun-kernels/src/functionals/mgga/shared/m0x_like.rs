//! Minnesota M05/M06 family helpers (M0x kinetic-energy enhancement substrate).
//!
//! FULL BODY port of `xcfun-master/src/functionals/m0xy_fun.hpp` (Wave 3,
//! plan 04-03). Replaces Wave-0 SKELETONs.
//!
//! # Sources
//! - `xcfun-master/src/functionals/m0xy_fun.hpp:1-262` — full module port.
//! - Reuses `pw92eps::pw92_eps` and `pw92eps::pw92eps_polarized` from
//!   `crates/xcfun-eval/src/functionals/lda/pw92eps.rs`.
//! - Reuses `pw91_like::chi2` and `pw91_like::pw91k_prefactor` from
//!   `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs`.
//!
//! # Pitfall P11 (12-coef fw Horner)
//! `m0x_fw` evaluates `poly(w, 12, a)` in descending Horner order per
//! `specmath.hpp:24-33`:
//!   `res = a[11]; res = res*w + a[10]; res = res*w + a[9]; ... res = res*w + a[0]`
//! Each step is an explicit-let-binding (Rust `Array<F>` per intermediate)
//! to suppress compiler reordering / FMA emission per CLAUDE.md ACC-06.
//!
//! # API design
//! M05/M06 family kernels need to pass per-functional 5-, 6-, or 12-coefficient
//! parameter arrays to these helpers. We pass the coefficients as **separate
//! scalar `F` arguments** (not as `Array<F>`) — the cubecl 0.10-pre.3 surface
//! supports F-typed scalars freely and avoids host-side seeding overhead.

// Match upstream C++ naming (`Dsigma`, etc) — algorithmic-identity rule.
#![allow(non_snake_case)]

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_reciprocal};

use super::constants::{
    M0X_ALPHA_C_ANTIPARALLEL_F64, M0X_ALPHA_C_PARALLEL_F64, M0X_ALPHA_X_F64, M0X_SCALEFACTOR_TF_F64,
};
use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pw91_like;
use crate::functionals::lda::pw92eps;

// `CF = 0.3 * (3*PI²)^(2/3)` per `xcfun-master/src/functionals/constants.hpp:31`.
// Matches Phase 2 `LYP_CF_F64` value (verified at strict 1e-12).
const M0X_CF_F64: f64 = 2.871_234_000_188_191_0_f64;

/// `M0X_CF_F64 * M0X_SCALEFACTOR_TF_F64` — scale-corrected Thomas-Fermi
/// constant subtracted in `zet`. Pre-computed at module load.
const M0X_CF_TIMES_SCALEFACTOR_TF: f64 = 9.115_599_744_691_195_f64;
// = 2.871_234_000_188_191_0 * 3.174_802_103_936_40

// ---------------------------------------------------------------------------
//  zet — kinetic-energy density working variable.
//  Port of m0xy_fun.hpp:64-68:
//    return 2 * tau / pow(rho, 5.0/3.0) - CF * scalefactorTFconst;
// ---------------------------------------------------------------------------

/// `zet(rho, tau) = 2·tau / ρ^(5/3) - CF · scalefactor_TF`.
#[cube]
pub fn m0x_zet<F: Float>(rho: &Array<F>, tau: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: rho_m53 = rho^(-5/3).
    let mut rho_m53 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(-5.0_f64 / 3.0_f64), &mut rho_m53, n);

    // Step 2: tau_over = tau · rho^(-5/3).
    let mut tau_over = Array::<F>::new(size);
    ctaylor_mul::<F>(tau, &rho_m53, &mut tau_over, n);

    // Step 3: out = 2 · tau_over.
    ctaylor_scalar_mul::<F>(&tau_over, F::cast_from(2.0_f64), out, n);

    // Step 4: subtract CF·scalefactor from CNST slot.
    out[0] = out[0] - F::cast_from(M0X_CF_TIMES_SCALEFACTOR_TF);
}

// ---------------------------------------------------------------------------
//  gamma — denominator function.  Port of m0xy_fun.hpp:73-76:
//    return 1 + alpha * (chi2 + zet);
// ---------------------------------------------------------------------------

/// `gamma(α, χ², zet) = 1 + α · (χ² + zet)`.
#[cube]
pub fn m0x_gamma<F: Float>(
    alpha: F,
    chi2: &Array<F>,
    zet: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: sum = chi2 + zet.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(chi2, zet, &mut sum, n);

    // Step 2: out = alpha · sum.
    ctaylor_scalar_mul::<F>(&sum, alpha, out, n);

    // Step 3: out += 1 at CNST slot.
    out[0] = out[0] + F::new(1.0);
}

// ---------------------------------------------------------------------------
//  h — exchange polynomial.  Port of m0xy_fun.hpp:84-97:
//    gam1 = gamma(alpha, chi2, zet)
//    t1 = d[0]/gam1
//    t2 = (d[1]*chi2 + d[2]*zet) / gam1²
//    t3 = (chi2*(d[3]*chi2 + d[4]*zet) + d[5]*zet²) / gam1³
//    return t1 + t2 + t3
// ---------------------------------------------------------------------------

/// M0x exchange polynomial `h(d[6], α, χ², zet)`.
///
/// 6 scalar coefficients passed as `d0..d5: F`.
#[cube]
pub fn m0x_h<F: Float>(
    d0: F,
    d1: F,
    d2: F,
    d3: F,
    d4: F,
    d5: F,
    alpha: F,
    chi2: &Array<F>,
    zet: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: gam1 = 1 + alpha*(chi2+zet).
    let mut gam1 = Array::<F>::new(size);
    m0x_gamma::<F>(alpha, chi2, zet, &mut gam1, n);

    // Step 2: inv_gam1 = 1 / gam1.
    let mut inv_gam1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gam1, &mut inv_gam1, n);

    // Step 3: inv_gam1_sq = inv_gam1 · inv_gam1.
    let mut inv_gam1_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&inv_gam1, &inv_gam1, &mut inv_gam1_sq, n);

    // Step 4: inv_gam1_cu = inv_gam1_sq · inv_gam1.
    let mut inv_gam1_cu = Array::<F>::new(size);
    ctaylor_mul::<F>(&inv_gam1_sq, &inv_gam1, &mut inv_gam1_cu, n);

    // Step 5: t1 = d0 · inv_gam1.
    let mut t1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_gam1, d0, &mut t1, n);

    // Step 6: t2_num = d1·chi2 + d2·zet.
    let mut d1_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(chi2, d1, &mut d1_chi2, n);
    let mut d2_zet = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(zet, d2, &mut d2_zet, n);
    let mut t2_num = Array::<F>::new(size);
    ctaylor_add::<F>(&d1_chi2, &d2_zet, &mut t2_num, n);

    // Step 7: t2 = t2_num · inv_gam1_sq.
    let mut t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&t2_num, &inv_gam1_sq, &mut t2, n);

    // Step 8: t3_inner = d3·chi2 + d4·zet.
    let mut d3_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(chi2, d3, &mut d3_chi2, n);
    let mut d4_zet = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(zet, d4, &mut d4_zet, n);
    let mut t3_inner = Array::<F>::new(size);
    ctaylor_add::<F>(&d3_chi2, &d4_zet, &mut t3_inner, n);

    // Step 9: chi2_t3_inner = chi2 · (d3·chi2 + d4·zet).
    let mut chi2_t3_inner = Array::<F>::new(size);
    ctaylor_mul::<F>(chi2, &t3_inner, &mut chi2_t3_inner, n);

    // Step 10: zet_sq = zet · zet.
    let mut zet_sq = Array::<F>::new(size);
    ctaylor_mul::<F>(zet, zet, &mut zet_sq, n);

    // Step 11: d5_zet_sq = d5 · zet².
    let mut d5_zet_sq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&zet_sq, d5, &mut d5_zet_sq, n);

    // Step 12: t3_num = chi2_t3_inner + d5_zet_sq.
    let mut t3_num = Array::<F>::new(size);
    ctaylor_add::<F>(&chi2_t3_inner, &d5_zet_sq, &mut t3_num, n);

    // Step 13: t3 = t3_num · inv_gam1_cu.
    let mut t3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&t3_num, &inv_gam1_cu, &mut t3, n);

    // Step 14: out = t1 + t2 + t3 (left-to-right per ACC-06).
    let mut t1_p_t2 = Array::<F>::new(size);
    ctaylor_add::<F>(&t1, &t2, &mut t1_p_t2, n);
    ctaylor_add::<F>(&t1_p_t2, &t3, out, n);
}

// ---------------------------------------------------------------------------
//  fw — kinetic-energy enhancement factor (12-coefficient polynomial).
//  Port of m0xy_fun.hpp:106-128:
//    tau_lsda = pw91k_prefactor(rho)
//    t = tau_lsda / tau
//    w = (t - 1) / (t + 1)
//    fw = poly(w, 12, a)   // descending Horner: a[11]·w^11 + ... + a[0]
// ---------------------------------------------------------------------------

/// M0x kinetic-energy density enhancement factor `fw(a[12], rho, tau)`.
///
/// 12 scalar coefficients `a0..a11: F` for descending Horner per Pitfall P11.
#[cube]
pub fn m0x_fw<F: Float>(
    a0: F,
    a1: F,
    a2: F,
    a3: F,
    a4: F,
    a5: F,
    a6: F,
    a7: F,
    a8: F,
    a9: F,
    a10: F,
    a11: F,
    rho: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // tau_lsda = pw91k_prefactor(rho)
    let mut tau_lsda = Array::<F>::new(size);
    pw91_like::pw91k_prefactor::<F>(rho, &mut tau_lsda, n);

    // inv_tau = 1 / tau
    let mut inv_tau = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(tau, &mut inv_tau, n);

    // t = tau_lsda · inv_tau
    let mut t = Array::<F>::new(size);
    ctaylor_mul::<F>(&tau_lsda, &inv_tau, &mut t, n);

    // t_minus_1 = t with CNST -= 1
    let mut t_minus_1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t_minus_1[i] = t[i];
    }
    t_minus_1[0] = t_minus_1[0] - F::new(1.0);

    // t_plus_1 = t with CNST += 1
    let mut t_plus_1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t_plus_1[i] = t[i];
    }
    t_plus_1[0] = t_plus_1[0] + F::new(1.0);

    // inv_t_plus_1 = 1 / (t+1)
    let mut inv_t_plus_1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&t_plus_1, &mut inv_t_plus_1, n);

    // w = (t-1) / (t+1)
    let mut w = Array::<F>::new(size);
    ctaylor_mul::<F>(&t_minus_1, &inv_t_plus_1, &mut w, n);

    // poly(w, 12, a) — descending Horner per specmath.hpp:24-33:
    //   res = a11
    //   for k in 10..=0: res = res*w + a[k]
    //
    // Each step is an explicit Array intermediate (no fused FMA, ACC-06).

    // res = a11 (scalar -> Array via CNST slot)
    let mut res = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        res[i] = F::new(0.0);
    }
    res[0] = a11;

    // i=10
    let mut tmp1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&res, &w, &mut tmp1, n);
    tmp1[0] = tmp1[0] + a10;
    // i=9
    let mut tmp2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp1, &w, &mut tmp2, n);
    tmp2[0] = tmp2[0] + a9;
    // i=8
    let mut tmp3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp2, &w, &mut tmp3, n);
    tmp3[0] = tmp3[0] + a8;
    // i=7
    let mut tmp4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp3, &w, &mut tmp4, n);
    tmp4[0] = tmp4[0] + a7;
    // i=6
    let mut tmp5 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp4, &w, &mut tmp5, n);
    tmp5[0] = tmp5[0] + a6;
    // i=5
    let mut tmp6 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp5, &w, &mut tmp6, n);
    tmp6[0] = tmp6[0] + a5;
    // i=4
    let mut tmp7 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp6, &w, &mut tmp7, n);
    tmp7[0] = tmp7[0] + a4;
    // i=3
    let mut tmp8 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp7, &w, &mut tmp8, n);
    tmp8[0] = tmp8[0] + a3;
    // i=2
    let mut tmp9 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp8, &w, &mut tmp9, n);
    tmp9[0] = tmp9[0] + a2;
    // i=1
    let mut tmp10 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp9, &w, &mut tmp10, n);
    tmp10[0] = tmp10[0] + a1;
    // i=0
    ctaylor_mul::<F>(&tmp10, &w, out, n);
    out[0] = out[0] + a0;
}

// ---------------------------------------------------------------------------
//  chi² — reduced gradient.  Delegates to gga::shared::pw91_like::chi2.
// ---------------------------------------------------------------------------

/// M0x `chi²` reduced-gradient working variable. Delegate to `pw91_like::chi2`.
#[cube]
pub fn m0x_chi2<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    pw91_like::chi2::<F>(rho, grad2, out, n);
}

// ---------------------------------------------------------------------------
//  D_sigma — exchange-hole spin-decomposed enhancement.
//  Port of m0xy_fun.hpp:140-150:
//    return 1.0 - 0.125 * gaa / (na * taua);
// ---------------------------------------------------------------------------

/// M0x `Dsigma(rho, gaa, taua) = 1 - 0.125·gaa/(rho·tau)`.
#[cube]
pub fn m0x_Dsigma<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: rho_tau = rho · tau.
    let mut rho_tau = Array::<F>::new(size);
    ctaylor_mul::<F>(rho, tau, &mut rho_tau, n);

    // Step 2: inv_rho_tau = 1 / (rho · tau).
    let mut inv_rho_tau = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&rho_tau, &mut inv_rho_tau, n);

    // Step 3: ratio = grad2 · inv_rho_tau.
    let mut ratio = Array::<F>::new(size);
    ctaylor_mul::<F>(grad2, &inv_rho_tau, &mut ratio, n);

    // Step 4: scaled = -0.125 · ratio.
    ctaylor_scalar_mul::<F>(&ratio, F::cast_from(-0.125_f64), out, n);

    // Step 5: bump CNST by +1 → out = 1 - 0.125·gaa/(rho·tau).
    out[0] = out[0] + F::new(1.0);
}

// ---------------------------------------------------------------------------
//  g — correlation polynomial.
//  Port of m0xy_fun.hpp:164-176:
//    b = gamma_chi_squared / (1 + gamma_chi_squared)
//    g = poly(b, 5, c)   // descending Horner over 5 coefs
// ---------------------------------------------------------------------------

/// M0x correlation polynomial `g(c[5], gamma_chi_squared)`.
///
/// 5 scalar coefficients `c0..c4: F`.
#[cube]
pub fn m0x_g<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    c3: F,
    c4: F,
    gamma_chi_squared: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // denom = 1 + gamma_chi_squared (CNST-bump on a copy)
    let mut denom = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom[i] = gamma_chi_squared[i];
    }
    denom[0] = denom[0] + F::new(1.0);

    // inv_denom = 1 / (1 + gamma_chi_squared)
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // b = gamma_chi_squared · inv_denom
    let mut b = Array::<F>::new(size);
    ctaylor_mul::<F>(gamma_chi_squared, &inv_denom, &mut b, n);

    // poly(b, 5, c): res=c4; res=res*b+c3; res=res*b+c2; res=res*b+c1; res=res*b+c0
    let mut res = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        res[i] = F::new(0.0);
    }
    res[0] = c4;

    let mut tmp1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&res, &b, &mut tmp1, n);
    tmp1[0] = tmp1[0] + c3;

    let mut tmp2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp1, &b, &mut tmp2, n);
    tmp2[0] = tmp2[0] + c2;

    let mut tmp3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&tmp2, &b, &mut tmp3, n);
    tmp3[0] = tmp3[0] + c1;

    ctaylor_mul::<F>(&tmp3, &b, out, n);
    out[0] = out[0] + c0;
}

// ---------------------------------------------------------------------------
//  M06 correlation antiparallel + parallel branches.
//  Port of m0xy_fun.hpp:186-213.
//
//  m06_c_anti(c[5], d[6], chi_a², zet_a, chi_b², zet_b):
//    γ_anti = 0.0031
//    zet_ab = zet_a + zet_b
//    chi_ab² = chi_a² + chi_b²
//    return g(c, γ_anti · chi_ab²) + h(d, α_anti, chi_ab², zet_ab)
//
//  m06_c_para(c[5], d[6], chi², zet, Dsigma):
//    γ_para = 0.06
//    return (g(c, γ_para · chi²) + h(d, α_para, chi², zet)) · Dsigma
// ---------------------------------------------------------------------------

const M06_GAMMA_C_ANTI_F64: f64 = 0.0031_f64;
const M06_GAMMA_C_PARA_F64: f64 = 0.06_f64;

/// M06 correlation antiparallel `(g + h)` branch.
///
/// Takes 5 c-coefs + 6 d-coefs, plus chi_a², zet_a, chi_b², zet_b arrays.
#[cube]
pub fn m06_c_anti<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    c3: F,
    c4: F,
    d0: F,
    d1: F,
    d2: F,
    d3: F,
    d4: F,
    d5: F,
    chi_a2: &Array<F>,
    zet_a: &Array<F>,
    chi_b2: &Array<F>,
    zet_b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // zet_ab = zet_a + zet_b
    let mut zet_ab = Array::<F>::new(size);
    ctaylor_add::<F>(zet_a, zet_b, &mut zet_ab, n);

    // chi_ab2 = chi_a2 + chi_b2
    let mut chi_ab2 = Array::<F>::new(size);
    ctaylor_add::<F>(chi_a2, chi_b2, &mut chi_ab2, n);

    // gamma_chi2 = γ_anti · chi_ab2
    let mut gamma_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &chi_ab2,
        F::cast_from(M06_GAMMA_C_ANTI_F64),
        &mut gamma_chi2,
        n,
    );

    // g_term = g(c, gamma_chi2)
    let mut g_term = Array::<F>::new(size);
    m0x_g::<F>(c0, c1, c2, c3, c4, &gamma_chi2, &mut g_term, n);

    // h_term = h(d, α_anti, chi_ab2, zet_ab)
    let mut h_term = Array::<F>::new(size);
    m0x_h::<F>(
        d0,
        d1,
        d2,
        d3,
        d4,
        d5,
        F::cast_from(M0X_ALPHA_C_ANTIPARALLEL_F64),
        &chi_ab2,
        &zet_ab,
        &mut h_term,
        n,
    );

    // out = g_term + h_term
    ctaylor_add::<F>(&g_term, &h_term, out, n);
}

/// M06 correlation parallel `(g + h) · Dsigma` branch.
#[cube]
pub fn m06_c_para<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    c3: F,
    c4: F,
    d0: F,
    d1: F,
    d2: F,
    d3: F,
    d4: F,
    d5: F,
    chi2: &Array<F>,
    zet: &Array<F>,
    dsigma: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // gamma_chi2 = γ_para · chi2
    let mut gamma_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(chi2, F::cast_from(M06_GAMMA_C_PARA_F64), &mut gamma_chi2, n);

    // g_term = g(c, gamma_chi2)
    let mut g_term = Array::<F>::new(size);
    m0x_g::<F>(c0, c1, c2, c3, c4, &gamma_chi2, &mut g_term, n);

    // h_term = h(d, α_para, chi2, zet)
    let mut h_term = Array::<F>::new(size);
    m0x_h::<F>(
        d0,
        d1,
        d2,
        d3,
        d4,
        d5,
        F::cast_from(M0X_ALPHA_C_PARALLEL_F64),
        chi2,
        zet,
        &mut h_term,
        n,
    );

    // sum = g_term + h_term
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&g_term, &h_term, &mut sum, n);

    // out = sum · dsigma
    ctaylor_mul::<F>(&sum, dsigma, out, n);
}

// ---------------------------------------------------------------------------
//  M05 correlation antiparallel + parallel branches.
//  Port of m0xy_fun.hpp:222-242.
//
//  m05_c_anti(c[5], chi_a², chi_b²):
//    γ_anti = 0.0031
//    chi_ab² = chi_a² + chi_b²
//    return g(c, γ_anti · chi_ab²)
//
//  m05_c_para(c[5], chi², _zet, Dsigma):
//    γ_para = 0.06
//    return g(c, γ_para · chi²) · Dsigma
// ---------------------------------------------------------------------------

/// M05 correlation antiparallel `g`-only branch.
#[cube]
pub fn m05_c_anti<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    c3: F,
    c4: F,
    chi_a2: &Array<F>,
    chi_b2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // chi_ab2 = chi_a2 + chi_b2
    let mut chi_ab2 = Array::<F>::new(size);
    ctaylor_add::<F>(chi_a2, chi_b2, &mut chi_ab2, n);

    // gamma_chi2 = γ_anti · chi_ab2
    let mut gamma_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &chi_ab2,
        F::cast_from(M06_GAMMA_C_ANTI_F64),
        &mut gamma_chi2,
        n,
    );

    // out = g(c, gamma_chi2)
    m0x_g::<F>(c0, c1, c2, c3, c4, &gamma_chi2, out, n);
}

/// M05 correlation parallel `g · Dsigma` branch (zet unused per upstream comment).
#[cube]
pub fn m05_c_para<F: Float>(
    c0: F,
    c1: F,
    c2: F,
    c3: F,
    c4: F,
    chi2: &Array<F>,
    dsigma: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // gamma_chi2 = γ_para · chi2
    let mut gamma_chi2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(chi2, F::cast_from(M06_GAMMA_C_PARA_F64), &mut gamma_chi2, n);

    // g_term = g(c, gamma_chi2)
    let mut g_term = Array::<F>::new(size);
    m0x_g::<F>(c0, c1, c2, c3, c4, &gamma_chi2, &mut g_term, n);

    // out = g_term · dsigma
    ctaylor_mul::<F>(&g_term, dsigma, out, n);
}

// ---------------------------------------------------------------------------
//  UEG correlation parallel + antiparallel.
//  Port of m0xy_fun.hpp:250-256:
//    ueg_c_para(rho)   = pw92eps_polarized(rho) · rho
//    ueg_c_anti(d)     = pw92eps(d) · d.n - ueg_c_para(d.a) - ueg_c_para(d.b)
// ---------------------------------------------------------------------------

/// UEG correlation parallel: `pw92eps_polarized(rho) · rho`.
#[cube]
pub fn ueg_c_para<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(rho, &mut eps, n);

    ctaylor_mul::<F>(&eps, rho, out, n);
}

/// UEG correlation antiparallel: `pw92eps(d)·d.n - ueg_c_para(d.a) - ueg_c_para(d.b)`.
#[cube]
pub fn ueg_c_anti<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92_eps(d)
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // total = eps · d.n
    let mut total = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps, &d.n, &mut total, n);

    // para_a = ueg_c_para(d.a)
    let mut para_a = Array::<F>::new(size);
    ueg_c_para::<F>(&d.a, &mut para_a, n);

    // para_b = ueg_c_para(d.b)
    let mut para_b = Array::<F>::new(size);
    ueg_c_para::<F>(&d.b, &mut para_b, n);

    // tmp = total - para_a
    let mut tmp = Array::<F>::new(size);
    ctaylor_sub::<F>(&total, &para_a, &mut tmp, n);

    // out = tmp - para_b
    ctaylor_sub::<F>(&tmp, &para_b, out, n);
}

// ---------------------------------------------------------------------------
//  lsda_x — local spin-density approximation for exchange.
//  Port of m0xy_fun.hpp:260-262:
//    return -(3/2) · (3/(4π))^(1/3) · ρ^(4/3)
// ---------------------------------------------------------------------------

/// `-(3/2) · (3/(4π))^(1/3) · ρ^(4/3) = -0.930525736349100 · ρ^(4/3)`.
///
/// Cross-check: `(3/2) · (3/(4π))^(1/3) = 0.9305257363491002` in f64 — NOT the
/// standard Slater coefficient `(3/4) · (3/π)^(1/3) = 0.7385587663820223`. The
/// two differ by exactly `2^(1/3)`. See `xcfun-master/src/functionals/m0xy_fun.hpp:260-262`
/// for the literal C++ expression this reproduces.
const LSDA_X_COEFF_F64: f64 = -0.930_525_736_349_100_2_f64;

/// `lsda_x(rho)` — local spin-density approximation for exchange.
#[cube]
pub fn m0x_lsda_x<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // rho_43 = rho^(4/3)
    let mut rho_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(4.0_f64 / 3.0_f64), &mut rho_43, n);

    // out = LSDA_X_COEFF · rho_43
    ctaylor_scalar_mul::<F>(&rho_43, F::cast_from(LSDA_X_COEFF_F64), out, n);
}
