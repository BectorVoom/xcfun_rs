//! XC_PW91C — Perdew-Wang 1991 correlation. **GGA-06.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pw91c.cpp:18-87`
//!
//! # Formula (the longest GGA body — ~80 LOC mirror of pw91c.cpp:39-87)
//! ```cpp
//! uf(d, p) = (a^p + b^p) · (2/n)^p
//! Gc(r, A, a1, b1, b2, b3, b4, p) =
//!   -2·A·(1 + a1·r) · log(1 + 0.5/(A·(sqrt(r)·(b1+sqrt(r)·(b2+sqrt(r)·b3)) + b4·r^(p+1))))
//!
//! fz   = (uf(d, 4/3) - 2) / (2·2^(1/3) - 2)
//! Ac   = Gc(r_s, Aa, a1a, b1a, b2a, b3a, b4a, pa=1)
//! EcP  = Gc(r_s, c0p, a1p, b1p, b2p, b3p, b4p, pe=1)
//! EcF  = Gc(r_s, c0f, a1f, b1f, b2f, b3f, b4f, pe=1)
//! Ec   = EcP - Ac·fz·(1 - ζ^4)/d2fz0 + (EcF - EcP)·fz·ζ^4
//! kF   = cbrt(3π²·n)
//! ks   = sqrt(4) · sqrt(kF / π)   = 2 · sqrt(kF/π)
//! gs   = 0.5 · uf(d, 2/3)
//! T2   = 0.25 · gnn / (gs · ks · n)²
//! ν    = 16 · cbrt(3π²) / π                  (precomputed scalar)
//! β    = ν · Cc0
//! A    = (2α/β) / expm1(-2αEc / (gs³·β²))
//! Cc   = 1/1000 · ((2.568 + r_s·(23.266 + 0.007389·r_s)) /
//!                  (1 + r_s·(8.723 + r_s·(0.472 + r_s·0.07389)))) - Cx
//! H0   = 0.5·gs³·β²/α · log(1 + 2α·(T2 + A·T2²) / (β·(1 + A·T2·(1 + A·T2))))
//! H1   = ν·(Cc - Cc0 - 3/7·Cx)·gs³·T2 · exp(-100·gs⁴·ks²·T2/kF²)
//! return n·(Ec + H0 + H1)
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_cbrt, ctaylor_exp, ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_2,
    ctaylor_powi_3, ctaylor_powi_4, ctaylor_reciprocal, ctaylor_sqrt,
};

use crate::density_vars::DensVarsDev;

// pw91c parameters (verbatim from pw91c.cpp:39-60)
const PW91C_PA: f64 = 1.0_f64;
const PW91C_AA: f64 = 0.016_887_f64;
const PW91C_A1A: f64 = 0.111_25_f64;
const PW91C_B1A: f64 = 10.357_f64;
const PW91C_B2A: f64 = 3.623_1_f64;
const PW91C_B3A: f64 = 0.880_26_f64;
const PW91C_B4A: f64 = 0.496_71_f64;

const PW91C_PE: f64 = 1.0_f64;
const PW91C_C0P: f64 = 0.031_091_f64;
const PW91C_A1P: f64 = 0.213_70_f64;
const PW91C_B1P: f64 = 7.595_7_f64;
const PW91C_B2P: f64 = 3.587_6_f64;
const PW91C_B3P: f64 = 1.638_2_f64;
const PW91C_B4P: f64 = 0.492_94_f64;

const PW91C_C0F: f64 = 0.015_545_f64;
const PW91C_A1F: f64 = 0.205_48_f64;
const PW91C_B1F: f64 = 14.118_9_f64;
const PW91C_B2F: f64 = 6.197_7_f64;
const PW91C_B3F: f64 = 3.366_2_f64;
const PW91C_B4F: f64 = 0.625_17_f64;

const PW91C_D2FZ0: f64 = 1.709_921_f64;

const PW91C_ALPHA: f64 = 0.09_f64;
const PW91C_CC0: f64 = 0.004_235_f64;
const PW91C_CX: f64 = -0.001_667_f64;
/// `nu = 16 · cbrt(3π²) / π` precomputed at higher precision then
/// rounded to the f64-nearest. The previous literal `15.755_926_546_290_507`
/// was hand-derived at insufficient precision (off by ~4e-7 relative);
/// locked by `tests::pw91c_nu_locked` (06-N7/07-00).
const PW91C_NU: f64 = 15.755_920_349_483_143_f64;
/// `beta = nu · Cc0`.
const PW91C_BETA: f64 = PW91C_NU * PW91C_CC0;

/// `2 · 2^(1/3) - 2 = 2^(4/3) - 2`. Used as the `fz` denominator.
/// f64-nearest of the algebraic truth. Previous literal
/// `0.519_842_099_789_746_3` was off by 1 ULP; locked by
/// `tests::pw91c_fz_denom_locked` (06-N7/07-00).
const PW91C_FZ_DENOM: f64 = 0.519_842_099_789_746_4_f64;

// uf(d, p) = (a^p + b^p) · (2/n)^p.
#[cube]
fn uf<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    n_ct: &Array<F>,
    p: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut a_p = Array::<F>::new(size);
    ctaylor_pow::<F>(a, p, &mut a_p, n);
    let mut b_p = Array::<F>::new(size);
    ctaylor_pow::<F>(b, p, &mut b_p, n);
    let mut sum_p = Array::<F>::new(size);
    ctaylor_add::<F>(&a_p, &b_p, &mut sum_p, n);
    // 2/n.
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(n_ct, &mut inv_n, n);
    let mut two_over_n = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_n, F::new(2.0), &mut two_over_n, n);
    // (2/n)^p.
    let mut two_n_p = Array::<F>::new(size);
    ctaylor_pow::<F>(&two_over_n, p, &mut two_n_p, n);
    // out = sum_p · (2/n)^p.
    ctaylor_mul::<F>(&sum_p, &two_n_p, out, n);
}

// Gc(r, A, a1, b1, b2, b3, b4, p) = -2A·(1+a1·r) ·
//   log(1 + 0.5/(A · (sqrt(r)·(b1 + sqrt(r)·(b2 + sqrt(r)·b3)) + b4 · r^(p+1))))
#[cube]
fn gc<F: Float>(
    r: &Array<F>,
    a_p: F,
    a1: F,
    b1: F,
    b2: F,
    b3: F,
    b4: F,
    p: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // sqrt_r = sqrt(r).
    let mut sqrt_r = Array::<F>::new(size);
    ctaylor_sqrt::<F>(r, &mut sqrt_r, n);
    // b3·sqrt_r.
    let mut b3_sr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_r, b3, &mut b3_sr, n);
    // b2 + b3·sqrt_r (CNST-bump).
    let mut inner1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner1[i] = b3_sr[i];
    }
    inner1[0] = inner1[0] + b2;
    // sqrt_r · inner1.
    let mut sr_in1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&sqrt_r, &inner1, &mut sr_in1, n);
    // b1 + sr_in1.
    let mut inner2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner2[i] = sr_in1[i];
    }
    inner2[0] = inner2[0] + b1;
    // sqrt_r · inner2.
    let mut sr_in2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&sqrt_r, &inner2, &mut sr_in2, n);
    // r^(p+1).
    let mut r_p1 = Array::<F>::new(size);
    ctaylor_pow::<F>(r, p + F::new(1.0), &mut r_p1, n);
    // b4 · r^(p+1).
    let mut b4_rp1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&r_p1, b4, &mut b4_rp1, n);
    // bracket = sr_in2 + b4_rp1.
    let mut bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&sr_in2, &b4_rp1, &mut bracket, n);
    // a_bracket = a_p · bracket.
    let mut a_bracket = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&bracket, a_p, &mut a_bracket, n);
    // inv_a_bracket = 1 / a_bracket.
    let mut inv_ab = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&a_bracket, &mut inv_ab, n);
    // half_inv = 0.5 · inv_ab.
    let mut half_inv = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_ab, F::new(0.5), &mut half_inv, n);
    // 1 + half_inv.
    let mut log_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        log_arg[i] = half_inv[i];
    }
    log_arg[0] = log_arg[0] + F::new(1.0);
    // lg = log(log_arg).
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&log_arg, &mut lg, n);
    // a1_r = a1 · r.
    let mut a1_r = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(r, a1, &mut a1_r, n);
    // 1 + a1_r.
    let mut one_a1r = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_a1r[i] = a1_r[i];
    }
    one_a1r[0] = one_a1r[0] + F::new(1.0);
    // prod = (1+a1·r) · log(...).
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_a1r, &lg, &mut prod, n);
    // out = -2A · prod.
    let neg_2a = F::new(0.0) - F::new(2.0) * a_p;
    ctaylor_scalar_mul::<F>(&prod, neg_2a, out, n);
}

/// XC_PW91C kernel. 1:1 port of `pw91c.cpp:39-87`.
#[cube]
pub fn pw91c_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // fz = (uf(d, 4/3) - 2) / (2·2^(1/3) - 2).
    let mut uf43 = Array::<F>::new(size);
    uf::<F>(&d.a, &d.b, &d.n, F::cast_from(4.0_f64 / 3.0_f64), &mut uf43, n);
    let mut fz_num = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        fz_num[i] = uf43[i];
    }
    fz_num[0] = fz_num[0] - F::new(2.0);
    let mut fz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&fz_num, F::cast_from(1.0_f64 / PW91C_FZ_DENOM), &mut fz, n);

    // Ac = Gc(r_s, Aa, a1a, b1a, b2a, b3a, b4a, pa=1).
    let mut ac = Array::<F>::new(size);
    gc::<F>(
        &d.r_s,
        F::cast_from(PW91C_AA),
        F::cast_from(PW91C_A1A),
        F::cast_from(PW91C_B1A),
        F::cast_from(PW91C_B2A),
        F::cast_from(PW91C_B3A),
        F::cast_from(PW91C_B4A),
        F::cast_from(PW91C_PA),
        &mut ac,
        n,
    );
    // EcP = Gc(r_s, c0p, a1p, b1p, b2p, b3p, b4p, pe=1).
    let mut ec_p = Array::<F>::new(size);
    gc::<F>(
        &d.r_s,
        F::cast_from(PW91C_C0P),
        F::cast_from(PW91C_A1P),
        F::cast_from(PW91C_B1P),
        F::cast_from(PW91C_B2P),
        F::cast_from(PW91C_B3P),
        F::cast_from(PW91C_B4P),
        F::cast_from(PW91C_PE),
        &mut ec_p,
        n,
    );
    // EcF.
    let mut ec_f = Array::<F>::new(size);
    gc::<F>(
        &d.r_s,
        F::cast_from(PW91C_C0F),
        F::cast_from(PW91C_A1F),
        F::cast_from(PW91C_B1F),
        F::cast_from(PW91C_B2F),
        F::cast_from(PW91C_B3F),
        F::cast_from(PW91C_B4F),
        F::cast_from(PW91C_PE),
        &mut ec_f,
        n,
    );

    // ζ^4.
    let mut zeta_4 = Array::<F>::new(size);
    ctaylor_powi_4::<F>(&d.zeta, &mut zeta_4, n);
    // 1 - ζ^4.
    let mut one_minus_z4 = Array::<F>::new(size);
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&zeta_4, neg_one, &mut one_minus_z4, n);
    one_minus_z4[0] = one_minus_z4[0] + F::new(1.0);
    // Ac · fz.
    let mut ac_fz = Array::<F>::new(size);
    ctaylor_mul::<F>(&ac, &fz, &mut ac_fz, n);
    // (Ac·fz) · (1-ζ^4).
    let mut ac_fz_omz4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&ac_fz, &one_minus_z4, &mut ac_fz_omz4, n);
    // / d2fz0.
    let mut term2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &ac_fz_omz4,
        F::cast_from(1.0_f64 / PW91C_D2FZ0),
        &mut term2,
        n,
    );
    // EcF - EcP.
    let mut ecf_ecp = Array::<F>::new(size);
    ctaylor_sub::<F>(&ec_f, &ec_p, &mut ecf_ecp, n);
    // (EcF-EcP) · fz.
    let mut ecf_ecp_fz = Array::<F>::new(size);
    ctaylor_mul::<F>(&ecf_ecp, &fz, &mut ecf_ecp_fz, n);
    // · ζ^4.
    let mut term3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&ecf_ecp_fz, &zeta_4, &mut term3, n);
    // EcP - term2.
    let mut ec_minus_t2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&ec_p, &term2, &mut ec_minus_t2, n);
    // Ec = ec_minus_t2 + term3.
    let mut ec = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_minus_t2, &term3, &mut ec, n);

    // kF = cbrt(3π² · n).
    const THREE_PI_SQ: f64 = 29.608_813_203_268_074_f64;
    let mut three_pi_sq_n = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.n, F::cast_from(THREE_PI_SQ), &mut three_pi_sq_n, n);
    let mut kf = Array::<F>::new(size);
    // 06-N7/07-00 — use ctaylor_cbrt (libm-cbrt-precision via Newton
    // refinement) instead of ctaylor_pow(x, 1/3). Matches C++ pw91c.cpp:67
    // `cbrt(3 * M_PI * M_PI * d.n)` which routes through tmath.hpp:172-178
    // cbrt_expand (uses libm cbrt for the seed), not pow.
    ctaylor_cbrt::<F>(&three_pi_sq_n, &mut kf, n);

    // ks = 2 · sqrt(kF/π).
    const INV_PI: f64 = 0.318_309_886_183_790_67_f64;
    let mut kf_over_pi = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&kf, F::cast_from(INV_PI), &mut kf_over_pi, n);
    let mut sqrt_kop = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&kf_over_pi, &mut sqrt_kop, n);
    let mut ks = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_kop, F::new(2.0), &mut ks, n);

    // gs = 0.5 · uf(d, 2/3).
    let mut uf23 = Array::<F>::new(size);
    uf::<F>(&d.a, &d.b, &d.n, F::cast_from(2.0_f64 / 3.0_f64), &mut uf23, n);
    let mut gs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf23, F::new(0.5), &mut gs, n);

    // T2 = 0.25 · gnn / (gs·ks·n)².
    let mut gs_ks = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs, &ks, &mut gs_ks, n);
    let mut gs_ks_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs_ks, &d.n, &mut gs_ks_n, n);
    let mut den_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&gs_ks_n, &mut den_sq, n);
    let mut inv_dsq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den_sq, &mut inv_dsq, n);
    let mut gnn_over_dsq = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_dsq, &mut gnn_over_dsq, n);
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&gnn_over_dsq, F::new(0.25), &mut t2, n);

    // gs³, gs⁴.
    let mut gs3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&gs, &mut gs3, n);
    let mut gs4 = Array::<F>::new(size);
    ctaylor_powi_4::<F>(&gs, &mut gs4, n);

    // A = (2α/β) / expm1(-2αEc / (gs³·β²)).
    // Compute -2αEc / (gs³·β²).
    let mut alpha_ec = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ec, F::cast_from(PW91C_ALPHA), &mut alpha_ec, n);
    // β² scalar precomputed.
    const BETA_SQ: f64 = PW91C_BETA * PW91C_BETA;
    let mut gs3_betasq = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&gs3, F::cast_from(BETA_SQ), &mut gs3_betasq, n);
    let mut inv_g3b2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gs3_betasq, &mut inv_g3b2, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&alpha_ec, &inv_g3b2, &mut frac, n);
    // arg = -2 · frac.
    let mut arg_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&frac, F::new(-2.0), &mut arg_a, n);
    let mut em1_a = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg_a, &mut em1_a, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1_a, &mut inv_em1, n);
    const TWO_ALPHA_OVER_BETA: f64 = 2.0_f64 * PW91C_ALPHA / PW91C_BETA;
    let mut a_pw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_em1, F::cast_from(TWO_ALPHA_OVER_BETA), &mut a_pw, n);

    // Cc = 1/1000 · ((2.568 + r_s·(23.266 + 0.007389·r_s)) /
    //                (1 + r_s·(8.723 + r_s·(0.472 + r_s·0.07389)))) - Cx.
    const CC_NUM_C0: f64 = 2.568_f64;
    const CC_NUM_C1: f64 = 23.266_f64;
    const CC_NUM_C2: f64 = 0.007_389_f64;
    const CC_DEN_C1: f64 = 8.723_f64;
    const CC_DEN_C2: f64 = 0.472_f64;
    const CC_DEN_C3: f64 = 0.073_89_f64;
    // numerator: 2.568 + r_s·(23.266 + 0.007389·r_s)
    let mut c2_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(CC_NUM_C2), &mut c2_rs, n);
    let mut inner_n = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_n[i] = c2_rs[i];
    }
    inner_n[0] = inner_n[0] + F::cast_from(CC_NUM_C1);
    let mut rs_inner_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &inner_n, &mut rs_inner_n, n);
    let mut cc_num = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        cc_num[i] = rs_inner_n[i];
    }
    cc_num[0] = cc_num[0] + F::cast_from(CC_NUM_C0);
    // denominator: 1 + r_s·(8.723 + r_s·(0.472 + r_s·0.07389))
    let mut c3_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.r_s, F::cast_from(CC_DEN_C3), &mut c3_rs, n);
    let mut inner_d1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_d1[i] = c3_rs[i];
    }
    inner_d1[0] = inner_d1[0] + F::cast_from(CC_DEN_C2);
    let mut rs_id1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &inner_d1, &mut rs_id1, n);
    let mut inner_d2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_d2[i] = rs_id1[i];
    }
    inner_d2[0] = inner_d2[0] + F::cast_from(CC_DEN_C1);
    let mut rs_id2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.r_s, &inner_d2, &mut rs_id2, n);
    let mut cc_den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        cc_den[i] = rs_id2[i];
    }
    cc_den[0] = cc_den[0] + F::new(1.0);
    let mut inv_ccd = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&cc_den, &mut inv_ccd, n);
    let mut frac_cc = Array::<F>::new(size);
    ctaylor_mul::<F>(&cc_num, &inv_ccd, &mut frac_cc, n);
    let mut cc = Array::<F>::new(size);
    // 06-N7/07-00 — `F::new(x)` takes f32; `F::new(0.001)` parses 0.001
    // as f32 first (=0.0010000000474974513 when promoted to f64), 4.75e-8
    // relative away from the f64 truth `0.001`. Use F::cast_from(0.001_f64)
    // to preserve f64 precision. Identified as a contributor to PW91C's
    // systematic order-0 offset against C++.
    ctaylor_scalar_mul::<F>(&frac_cc, F::cast_from(0.001_f64), &mut cc, n);
    cc[0] = cc[0] - F::cast_from(PW91C_CX);

    // H0 = 0.5 · gs³ · β²/α · log(1 + 2α·(T2 + A·T2²) / (β·(1 + A·T2·(1 + A·T2)))).
    let mut t2_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&t2, &mut t2_sq, n);
    let mut a_t2sq = Array::<F>::new(size);
    ctaylor_mul::<F>(&a_pw, &t2_sq, &mut a_t2sq, n);
    let mut t2_plus_at2sq = Array::<F>::new(size);
    ctaylor_add::<F>(&t2, &a_t2sq, &mut t2_plus_at2sq, n);
    let mut a_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&a_pw, &t2, &mut a_t2, n);
    let mut one_at2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_at2[i] = a_t2[i];
    }
    one_at2[0] = one_at2[0] + F::new(1.0);
    let mut at2_one_at2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&a_t2, &one_at2, &mut at2_one_at2, n);
    let mut den_h0 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den_h0[i] = at2_one_at2[i];
    }
    den_h0[0] = den_h0[0] + F::new(1.0);
    let mut beta_den_h0 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&den_h0, F::cast_from(PW91C_BETA), &mut beta_den_h0, n);
    let mut inv_bdh = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&beta_den_h0, &mut inv_bdh, n);
    let mut frac_h0 = Array::<F>::new(size);
    ctaylor_mul::<F>(&t2_plus_at2sq, &inv_bdh, &mut frac_h0, n);
    let mut twoalpha_frac = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &frac_h0,
        F::cast_from(2.0_f64 * PW91C_ALPHA),
        &mut twoalpha_frac,
        n,
    );
    let mut log_arg_h0 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        log_arg_h0[i] = twoalpha_frac[i];
    }
    log_arg_h0[0] = log_arg_h0[0] + F::new(1.0);
    let mut log_h0 = Array::<F>::new(size);
    ctaylor_log::<F>(&log_arg_h0, &mut log_h0, n);
    const HALF_BETASQ_OVER_ALPHA: f64 = 0.5_f64 * BETA_SQ / PW91C_ALPHA;
    let mut h0_pre = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&gs3, F::cast_from(HALF_BETASQ_OVER_ALPHA), &mut h0_pre, n);
    let mut h0 = Array::<F>::new(size);
    ctaylor_mul::<F>(&h0_pre, &log_h0, &mut h0, n);

    // H1 = ν·(Cc - Cc0 - 3/7·Cx)·gs³·T2 · exp(-100·gs⁴·ks²·T2/kF²).
    // (Cc - Cc0 - 3/7·Cx)
    let mut cc_minus = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        cc_minus[i] = cc[i];
    }
    cc_minus[0] = cc_minus[0] - F::cast_from(PW91C_CC0 + 3.0_f64 / 7.0_f64 * PW91C_CX);
    // gs³·T2.
    let mut gs3_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs3, &t2, &mut gs3_t2, n);
    // pre = ν · cc_minus · gs³ · T2.
    let mut nu_cc = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&cc_minus, F::cast_from(PW91C_NU), &mut nu_cc, n);
    let mut nu_cc_gs3t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&nu_cc, &gs3_t2, &mut nu_cc_gs3t2, n);
    // exp_arg = -100 · gs⁴ · ks² · T2 / kF².
    let mut ks_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&ks, &mut ks_sq, n);
    let mut kf_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&kf, &mut kf_sq, n);
    let mut inv_kf_sq = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&kf_sq, &mut inv_kf_sq, n);
    let mut gs4_kssq = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs4, &ks_sq, &mut gs4_kssq, n);
    let mut gs4_kssq_t2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs4_kssq, &t2, &mut gs4_kssq_t2, n);
    let mut frac_h1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gs4_kssq_t2, &inv_kf_sq, &mut frac_h1, n);
    let mut exp_arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&frac_h1, F::new(-100.0), &mut exp_arg, n);
    let mut e_h1 = Array::<F>::new(size);
    ctaylor_exp::<F>(&exp_arg, &mut e_h1, n);
    let mut h1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&nu_cc_gs3t2, &e_h1, &mut h1, n);

    // sum = Ec + H0 + H1.
    let mut ec_h0 = Array::<F>::new(size);
    ctaylor_add::<F>(&ec, &h0, &mut ec_h0, n);
    let mut sum_eh = Array::<F>::new(size);
    ctaylor_add::<F>(&ec_h0, &h1, &mut sum_eh, n);

    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum_eh, out, n);
}

#[cfg(test)]
mod tests {
    /// Regression lock for `PW91C_NU = 16 · cbrt(3π²) / π`. The previous
    /// value `15.755_926_546_290_507_f64` was incorrect — the f64-nearest
    /// of the truth is `15.755_920_349_483_143`. The constant was
    /// hand-derived at insufficient precision; the bug contributed to
    /// 69% record-level FAIL of PW91C in Phase 7 Plan 07-00 Task 0.3.
    #[test]
    fn pw91c_nu_locked() {
        let truth: f64 = 15.755_920_349_483_143_f64;
        assert_eq!(super::PW91C_NU, truth);
    }

    /// 1-ULP correction to `PW91C_FZ_DENOM = 2·2^(1/3) - 2`. f64-nearest
    /// is `0.519_842_099_789_746_4`, not `..._3`.
    #[test]
    fn pw91c_fz_denom_locked() {
        let truth: f64 = 0.519_842_099_789_746_4_f64;
        assert_eq!(super::PW91C_FZ_DENOM, truth);
    }
}

