//! SCAN-family exchange + correlation enhancement helpers.
//!
//! Phase 4 plan 04-02 Wave 2 — FULL BODIES replacing the Wave-0 skeletons.
//!
//! Port of `xcfun-master/src/functionals/SCAN_like_eps.hpp` (522 LOC).
//!
//! # IALPHA/IINTERP/IDELFX comptime dispatch (per C++ signature)
//!
//! The three comptime parameters map to the three C++ integer arguments:
//! - `ialpha`  (0=SCAN alpha, 1=rSCAN alpha', 2=r2/r4SCAN alpha-bar)
//! - `iinterp` (0=SCAN interpolation, 1=rSCAN polynomial interpolation)
//! - `idelfx`  (0=SCAN 4th-order GE, 1=rSCAN 2nd-order GE, 2=r4SCAN 4th-order GE)
//!
//! Per-functional mapping (from C++ .cpp files):
//! - SCAN:    get_SCAN_Fx(…, 0,0,0)  SCAN_C(…, 0,0,0)
//! - rSCAN:   get_SCAN_Fx(…, 1,1,0)  SCAN_C(…, 1,1,0)
//! - r++SCAN: get_SCAN_Fx(…, 2,1,0)  SCAN_C(…, 2,1,0)
//! - r2SCAN:  get_SCAN_Fx(…, 2,1,1)  SCAN_C(…, 2,1,1)
//! - r4SCAN:  get_SCAN_Fx(…, 2,1,2)  SCAN_C(…, 2,1,2)
//!
//! # Sources
//! - `SCAN_like_eps.hpp:71-73`   — fx_unif
//! - `SCAN_like_eps.hpp:75-130`  — get_SCAN_Fx (alpha computation)
//! - `SCAN_like_eps.hpp:132-251` — SCAN_X_Fx (enhancement factor)
//! - `SCAN_like_eps.hpp:253-353` — SCAN_C (correlation top-level)
//! - `SCAN_like_eps.hpp:355-376` — scan_ec0
//! - `SCAN_like_eps.hpp:378-385` — lda_0
//! - `SCAN_like_eps.hpp:387-461` — scan_ec1
//! - `SCAN_like_eps.hpp:463-497` — get_lsda1
//! - `SCAN_like_eps.hpp:499-521` — gcor2
//! - `specmath.hpp:35-37`        — ufunc(x,a) = (1+x)^a + (1-x)^a

#![allow(non_snake_case)]

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_powi_2, ctaylor_powi_3, ctaylor_powi_4,
    ctaylor_reciprocal, ctaylor_sqrt,
};

// ---------------------------------------------------------------------------
//  Constants from SCAN_like_eps.hpp
// ---------------------------------------------------------------------------

// Exchange constants
const A1_F64: f64 = 4.9479;
const K1_F64: f64 = 0.065;
const K0_F64: f64 = 0.174;
const MU_F64: f64 = 10.0 / 81.0;
// IE_PARAMS exchange: used in rSCAN interpolation and del_f2/del_f4 computation
const IE_PARAMS_X: [f64; 8] = [
    1.0,
    -0.667,
    -0.4445555,
    -0.663086601049,
    1.451297044490,
    -0.887998041597,
    0.234528941479,
    -0.023185843322,
];
const CFX1_F64: f64 = 0.667;
const CFX2_F64: f64 = 0.8;
const CFDX1_F64: f64 = 1.24;
const D_DAMP2_X_F64: f64 = 0.361;
const DX_DAMP4_P_F64: f64 = 0.232;
const DX_DAMP4_A_F64: f64 = 0.232;
const B1_F64: f64 = 0.156632;
const B2_F64: f64 = 0.12083;
const B3_F64: f64 = 0.5;
// B4 = MU*MU/K1 - 0.112654
const B4_F64: f64 = MU_F64 * MU_F64 / K1_F64 - 0.112654;
// ALPHA_GE = 20/27 + ETA * 5/3  (ETA = 1e-3)
const ALPHA_GE_F64: f64 = 20.0 / 27.0 + 1.0e-3 * 5.0 / 3.0;
// del_f2 = sum_{i=1}^{7} i * IE_PARAMS_X[i]
const DEL_F2_X: f64 = {
    let mut s = 0.0;
    let mut i = 1_usize;
    while i < 8 {
        s += (i as f64) * IE_PARAMS_X[i];
        i += 1;
    }
    s
};
// del_f4 = sum_{i=1}^{7} i*(i-1) * IE_PARAMS_X[i]
const DEL_F4_X: f64 = {
    let mut s = 0.0;
    let mut i = 1_usize;
    while i < 8 {
        s += (i as f64) * ((i as f64) - 1.0) * IE_PARAMS_X[i];
        i += 1;
    }
    s
};

// Correlation constants
const CFC1_F64: f64 = 0.64;
const CFC2_F64: f64 = 1.5;
const CFDC1_F64: f64 = 0.7;
// IE_PARAMS correlation (different from exchange)
const IE_PARAMS_C: [f64; 8] = [
    1.0,
    -0.64,
    -0.4352,
    -1.535685604549,
    3.061560252175,
    -1.915710236206,
    0.516884468372,
    -0.051848879792,
];
// del_f2 for correlation
const DEL_F2_C: f64 = {
    let mut s = 0.0;
    let mut i = 1_usize;
    while i < 8 {
        s += (i as f64) * IE_PARAMS_C[i];
        i += 1;
    }
    s
};
const B1C_F64: f64 = 0.0285764;
const B2C_F64: f64 = 0.0889;
const B3C_F64: f64 = 0.125541;
const CHI_LD_F64: f64 = 0.12802585262625815;
const BETA_MB_F64: f64 = 0.066725;
const AFACTOR_F64: f64 = 0.1;
const BFACTOR_F64: f64 = 0.1778;
const GAMMA_F64: f64 = 0.031090690869655;
// AFIX_T = sqrt(π/4) * (9π/4)^(1/6)
const AFIX_T_F64: f64 = 1.2277228507842887; // sqrt(PI/4) * (9*PI/4)^(1/6)
const D_DAMP2_C_F64: f64 = 0.361;

// gcor2 coefficient sets (indices A=0, A1=1, B1=2, B2=3, B3=4, B4=5)
const P_EU: [f64; 6] = [0.03109070, 0.213700, 7.59570, 3.58760, 1.63820, 0.492940];
const P_EP: [f64; 6] = [0.015545350, 0.205480, 14.11890, 6.19770, 3.36620, 0.625170];
const P_ALFM: [f64; 6] = [0.01688690, 0.111250, 10.3570, 3.62310, 0.880260, 0.496710]; // matches C++ p_alfm[6]

// GAM for get_lsda1 (PW92 spin-stiffness)
const GAM_F64: f64 = 0.51984209978974632953442121455650;
const FZZ_F64: f64 = 8.0 / (9.0 * GAM_F64);

// Derived constants for gcor2 coefficient sets (avoids NativeExpand issues inside #[cube])
// EU set (p_set=0)
const EU_A: f64 = P_EU[0];
const EU_A1: f64 = P_EU[1];
const EU_B1: f64 = P_EU[2];
const EU_B2: f64 = P_EU[3];
const EU_B3: f64 = P_EU[4];
const EU_B4: f64 = P_EU[5];
const EU_Q0_CONST: f64 = -2.0 * EU_A;
const EU_Q0RS: f64 = -2.0 * EU_A * EU_A1;
const EU_2A: f64 = 2.0 * EU_A;
const EU_2B2: f64 = 2.0 * EU_B2;
const EU_3B3: f64 = 3.0 * EU_B3;
const EU_4B4: f64 = 4.0 * EU_B4;
// EP set (p_set=1)
const EP_A: f64 = P_EP[0];
const EP_A1: f64 = P_EP[1];
const EP_B1: f64 = P_EP[2];
const EP_B2: f64 = P_EP[3];
const EP_B3: f64 = P_EP[4];
const EP_B4: f64 = P_EP[5];
const EP_Q0_CONST: f64 = -2.0 * EP_A;
const EP_Q0RS: f64 = -2.0 * EP_A * EP_A1;
const EP_2A: f64 = 2.0 * EP_A;
const EP_2B2: f64 = 2.0 * EP_B2;
const EP_3B3: f64 = 3.0 * EP_B3;
const EP_4B4: f64 = 4.0 * EP_B4;
// ALFM set (p_set=2)
const ALFM_A: f64 = P_ALFM[0];
const ALFM_A1: f64 = P_ALFM[1];
const ALFM_B1: f64 = P_ALFM[2];
const ALFM_B2: f64 = P_ALFM[3];
const ALFM_B3: f64 = P_ALFM[4];
const ALFM_B4: f64 = P_ALFM[5];
const ALFM_Q0_CONST: f64 = -2.0 * ALFM_A;
const ALFM_Q0RS: f64 = -2.0 * ALFM_A * ALFM_A1;
const ALFM_2A: f64 = 2.0 * ALFM_A;
const ALFM_2B2: f64 = 2.0 * ALFM_B2;
const ALFM_3B3: f64 = 3.0 * ALFM_B3;
const ALFM_4B4: f64 = 4.0 * ALFM_B4;

// ---------------------------------------------------------------------------
//  Derived constants (computed from primitives above; used inside #[cube] via
//  F::cast_from() to avoid NativeExpand conflicts inside the macro).
// ---------------------------------------------------------------------------

// SCAN_X_Fx / get_SCAN_Fx derived
const H0X_VAL: f64 = 1.0 + K0_F64;
const H1X_CNST: f64 = 1.0 + K1_F64;
// C2 = -DEL_F2_X * (1 - H0X_VAL) = DEL_F2_X * K0_F64
const C2_X: f64 = -DEL_F2_X * (1.0 - H0X_VAL);
// damp denominators
const DAMP_DENOM_X: f64 = D_DAMP2_X_F64 * D_DAMP2_X_F64 * D_DAMP2_X_F64 * D_DAMP2_X_F64;
const DAMP_DENOM_C: f64 = D_DAMP2_C_F64 * D_DAMP2_C_F64 * D_DAMP2_C_F64 * D_DAMP2_C_F64;
// r4SCAN 4th-order GE coefficients (IDELFX=2)
const ETA_VAL: f64 = 1.0e-3;
const ETA_TERM: f64 = ETA_VAL * 3.0 / 4.0 + 2.0 / 3.0;
const C_AA: f64 = 73.0 / 5000.0 - 0.5 * DEL_F4_X * (H0X_VAL - 1.0);
const C_PA: f64 =
    511.0 / 13500.0 - 73.0 / 1500.0 * ETA_VAL - DEL_F2_X * (ALPHA_GE_F64 * C2_X + MU_F64);
const C_PP: f64 = 146.0 / 2025.0 * ETA_TERM * ETA_TERM - 73.0 / 405.0 * ETA_TERM
    + (ALPHA_GE_F64 * C2_X + MU_F64) * (ALPHA_GE_F64 * C2_X + MU_F64) / K1_F64;
const DAMP4_A_SQ: f64 = DX_DAMP4_A_F64 * DX_DAMP4_A_F64;
const DAMP4_P4: f64 = DX_DAMP4_P_F64 * DX_DAMP4_P_F64 * DX_DAMP4_P_F64 * DX_DAMP4_P_F64;
// ALPHA_GE * C2_X + MU used in both C_PA and C_PP — inline in those consts for now
#[allow(dead_code)]
const AGE_C2_MU: f64 = ALPHA_GE_F64 * C2_X + MU_F64;

// ---------------------------------------------------------------------------
//  ufunc(x, p) = (1+x)^p + (1-x)^p  (specmath.hpp:35-37)
// ---------------------------------------------------------------------------

/// `ufunc(zeta, p) = (1+zeta)^p + (1-zeta)^p`.
#[cube]
pub fn ufunc<F: Float>(zeta: &Array<F>, p: F, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // pz = 1 + zeta
    let mut pz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        pz[i] = zeta[i];
    }
    pz[0] = pz[0] + F::new(1.0);
    // mz = -(zeta) + 1 = 1 - zeta
    let mut mz = Array::<F>::new(size);
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(zeta, neg_one, &mut mz, n);
    mz[0] = mz[0] + F::new(1.0);
    // pz^p
    let mut pzp = Array::<F>::new(size);
    ctaylor_pow::<F>(&pz, p, &mut pzp, n);
    // mz^p
    let mut mzp = Array::<F>::new(size);
    ctaylor_pow::<F>(&mz, p, &mut mzp, n);
    // out = pzp + mzp
    ctaylor_add::<F>(&pzp, &mzp, out, n);
}

// ---------------------------------------------------------------------------
//  fx_unif — uniform exchange.  Port of SCAN_like_eps.hpp:71-73.
// ---------------------------------------------------------------------------

/// `fx_unif(d) = (-3/4) * (3/π)^(1/3) * d^(4/3)`.
#[cube]
pub fn scan_fx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // (-3/4)*(3/π)^(1/3) = -0.7385587663820223
    const COEFF: f64 = -0.738_558_766_382_022_3_f64;
    let size = comptime!((1_u32 << n) as usize);
    let mut rho_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(4.0_f64 / 3.0_f64), &mut rho_43, n);
    ctaylor_scalar_mul::<F>(&rho_43, F::cast_from(COEFF), out, n);
}

// ---------------------------------------------------------------------------
//  gcor2 — PW92 correlation helper.  Port of SCAN_like_eps.hpp:499-521.
// ---------------------------------------------------------------------------

/// `gcor2(P, rs, sqrtrs, &GG, &GGRS)`.
/// Uses fixed P set selected by comptime index (0=EU, 1=EP, 2=ALFM).
#[cube]
pub fn gcor2<F: Float>(
    rs: &Array<F>,
    sqrtrs: &Array<F>,
    gg: &mut Array<F>,
    ggrs: &mut Array<F>,
    #[comptime] p_set: u32,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // All coefficients selected via if comptime!() using module-level consts.
    // pa=A, pa1=A1, pb1..pb4=B1..B4 (vary per p_set).
    // Derived: Q0_CONST=-2A, Q0RS=-2*A*A1, 2A, 2B2, 3B3, 4B4.

    // Q0 = (-2*A) + (-2*A*A1)*rs
    let mut q0 = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(EU_Q0RS), &mut q0, n);
        q0[0] = q0[0] + F::cast_from(EU_Q0_CONST);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(EP_Q0RS), &mut q0, n);
        q0[0] = q0[0] + F::cast_from(EP_Q0_CONST);
    } else {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(ALFM_Q0RS), &mut q0, n);
        q0[0] = q0[0] + F::cast_from(ALFM_Q0_CONST);
    }
    // q0rs_val scalar (used at end for GGRS)
    let q0rs_val = if comptime!(p_set == 0) {
        F::cast_from(EU_Q0RS)
    } else if comptime!(p_set == 1) {
        F::cast_from(EP_Q0RS)
    } else {
        F::cast_from(ALFM_Q0RS)
    };

    // Q1 = 2*A*sqrtrs*(B1 + sqrtrs*(B2 + sqrtrs*(B3 + B4*sqrtrs)))
    // inner: B3 + B4*sqrtrs
    let mut inner = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(EU_B4), &mut inner, n);
        inner[0] = inner[0] + F::cast_from(EU_B3);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(EP_B4), &mut inner, n);
        inner[0] = inner[0] + F::cast_from(EP_B3);
    } else {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(ALFM_B4), &mut inner, n);
        inner[0] = inner[0] + F::cast_from(ALFM_B3);
    }
    // (B2 + inner*sqrtrs)
    let mut inner2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&inner, sqrtrs, &mut inner2_raw, n);
    let mut inner2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner2[i] = inner2_raw[i];
    }
    if comptime!(p_set == 0) {
        inner2[0] = inner2[0] + F::cast_from(EU_B2);
    } else if comptime!(p_set == 1) {
        inner2[0] = inner2[0] + F::cast_from(EP_B2);
    } else {
        inner2[0] = inner2[0] + F::cast_from(ALFM_B2);
    }
    // (B1 + inner2*sqrtrs)
    let mut inner3_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&inner2, sqrtrs, &mut inner3_raw, n);
    let mut inner3 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner3[i] = inner3_raw[i];
    }
    if comptime!(p_set == 0) {
        inner3[0] = inner3[0] + F::cast_from(EU_B1);
    } else if comptime!(p_set == 1) {
        inner3[0] = inner3[0] + F::cast_from(EP_B1);
    } else {
        inner3[0] = inner3[0] + F::cast_from(ALFM_B1);
    }
    // Q1 = 2*A*sqrtrs*inner3
    let mut q1_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtrs, &inner3, &mut q1_raw, n);
    let mut q1 = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(&q1_raw, F::cast_from(EU_2A), &mut q1, n);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(&q1_raw, F::cast_from(EP_2A), &mut q1, n);
    } else {
        ctaylor_scalar_mul::<F>(&q1_raw, F::cast_from(ALFM_2A), &mut q1, n);
    }

    // Q1RS = A*(2*B2 + B1/sqrtrs + 3*B3*sqrtrs + 4*B4*rs)
    // B1/sqrtrs = B1 * sqrtrs^(-1)
    let mut inv_sqrtrs = Array::<F>::new(size);
    ctaylor_pow::<F>(sqrtrs, F::cast_from(-1.0_f64), &mut inv_sqrtrs, n);
    let mut b1_inv_sqrtrs = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(&inv_sqrtrs, F::cast_from(EU_B1), &mut b1_inv_sqrtrs, n);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(&inv_sqrtrs, F::cast_from(EP_B1), &mut b1_inv_sqrtrs, n);
    } else {
        ctaylor_scalar_mul::<F>(&inv_sqrtrs, F::cast_from(ALFM_B1), &mut b1_inv_sqrtrs, n);
    }
    // 2*B2
    let mut q1rs_acc = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        q1rs_acc[i] = b1_inv_sqrtrs[i];
    }
    if comptime!(p_set == 0) {
        q1rs_acc[0] = q1rs_acc[0] + F::cast_from(EU_2B2);
    } else if comptime!(p_set == 1) {
        q1rs_acc[0] = q1rs_acc[0] + F::cast_from(EP_2B2);
    } else {
        q1rs_acc[0] = q1rs_acc[0] + F::cast_from(ALFM_2B2);
    }
    // +3*B3*sqrtrs
    let mut three_b3_sqrtrs = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(EU_3B3), &mut three_b3_sqrtrs, n);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(EP_3B3), &mut three_b3_sqrtrs, n);
    } else {
        ctaylor_scalar_mul::<F>(sqrtrs, F::cast_from(ALFM_3B3), &mut three_b3_sqrtrs, n);
    }
    let mut q1rs_acc2 = Array::<F>::new(size);
    ctaylor_add::<F>(&q1rs_acc, &three_b3_sqrtrs, &mut q1rs_acc2, n);
    // +4*B4*rs
    let mut four_b4_rs = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(EU_4B4), &mut four_b4_rs, n);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(EP_4B4), &mut four_b4_rs, n);
    } else {
        ctaylor_scalar_mul::<F>(rs, F::cast_from(ALFM_4B4), &mut four_b4_rs, n);
    }
    let mut q1rs_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&q1rs_acc2, &four_b4_rs, &mut q1rs_sum, n);
    // Q1RS = A * q1rs_sum
    let mut q1rs = Array::<F>::new(size);
    if comptime!(p_set == 0) {
        ctaylor_scalar_mul::<F>(&q1rs_sum, F::cast_from(EU_A), &mut q1rs, n);
    } else if comptime!(p_set == 1) {
        ctaylor_scalar_mul::<F>(&q1rs_sum, F::cast_from(EP_A), &mut q1rs, n);
    } else {
        ctaylor_scalar_mul::<F>(&q1rs_sum, F::cast_from(ALFM_A), &mut q1rs, n);
    }

    // Q2 = log(1 + 1/Q1)
    let mut inv_q1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&q1, &mut inv_q1, n);
    let mut one_plus_inv_q1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus_inv_q1[i] = inv_q1[i];
    }
    one_plus_inv_q1[0] = one_plus_inv_q1[0] + F::new(1.0);
    let mut q2 = Array::<F>::new(size);
    ctaylor_log::<F>(&one_plus_inv_q1, &mut q2, n);

    // Q2RS = -Q1RS / ((1 + 1/Q1) * Q1^2)
    // = -Q1RS / (Q1^2 + Q1)
    let mut q1_sq = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&q1, &mut q1_sq, n);
    let mut q1_sq_plus_q1 = Array::<F>::new(size);
    ctaylor_add::<F>(&q1_sq, &q1, &mut q1_sq_plus_q1, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&q1_sq_plus_q1, &mut inv_denom, n);
    let mut q2rs_neg = Array::<F>::new(size);
    ctaylor_mul::<F>(&q1rs, &inv_denom, &mut q2rs_neg, n);
    // q2rs = -q2rs_neg
    let mut q2rs = Array::<F>::new(size);
    let neg = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&q2rs_neg, neg, &mut q2rs, n);

    // GG = Q0 * Q2
    ctaylor_mul::<F>(&q0, &q2, gg, n);

    // GGRS = Q0 * Q2RS + Q2 * Q0RS
    // Q0RS is scalar: q0rs_val
    let mut q0_q2rs = Array::<F>::new(size);
    ctaylor_mul::<F>(&q0, &q2rs, &mut q0_q2rs, n);
    let mut q2_q0rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&q2, q0rs_val, &mut q2_q0rs, n);
    ctaylor_add::<F>(&q0_q2rs, &q2_q0rs, ggrs, n);
}

// ---------------------------------------------------------------------------
//  get_lsda1 — PW92 LSDA.  Port of SCAN_like_eps.hpp:463-497.
// ---------------------------------------------------------------------------

/// Compute `eclsda1` and `d_eclsda1_drs` via PW92 spin-interpolation.
/// Both output arrays must be pre-allocated size `1 << n`.
#[cube]
pub fn get_lsda1<F: Float>(
    rs: &Array<F>,
    zeta: &Array<F>,
    sqrtrs: &Array<F>,
    eps: &mut Array<F>,
    eps_rs: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eu, deudrs
    let mut eu = Array::<F>::new(size);
    let mut deudrs = Array::<F>::new(size);
    gcor2::<F>(rs, sqrtrs, &mut eu, &mut deudrs, 0_u32, n);
    // ep, depdrs
    let mut ep = Array::<F>::new(size);
    let mut depdrs = Array::<F>::new(size);
    gcor2::<F>(rs, sqrtrs, &mut ep, &mut depdrs, 1_u32, n);
    // alfm, dalfmdrs
    let mut alfm = Array::<F>::new(size);
    let mut dalfmdrs = Array::<F>::new(size);
    gcor2::<F>(rs, sqrtrs, &mut alfm, &mut dalfmdrs, 2_u32, n);

    // z3 = zeta^3, z4 = zeta^4
    let mut z3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(zeta, &mut z3, n);
    let mut z4 = Array::<F>::new(size);
    ctaylor_powi_4::<F>(zeta, &mut z4, n);

    // f = (ufunc(zeta, 4/3) - 2) / GAM
    let mut uf_43 = Array::<F>::new(size);
    ufunc::<F>(zeta, F::cast_from(4.0_f64 / 3.0_f64), &mut uf_43, n);
    let mut uf_43_m2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        uf_43_m2[i] = uf_43[i];
    }
    uf_43_m2[0] = uf_43_m2[0] - F::new(2.0);
    let mut f = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf_43_m2, F::cast_from(1.0 / GAM_F64), &mut f, n);

    // z4 * f
    let mut z4f = Array::<F>::new(size);
    ctaylor_mul::<F>(&z4, &f, &mut z4f, n);

    // (1 - z4*f)
    let mut one_m_z4f = Array::<F>::new(size);
    let neg = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&z4f, neg, &mut one_m_z4f, n);
    one_m_z4f[0] = one_m_z4f[0] + F::new(1.0);

    // (1 - z4)
    let mut one_m_z4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z4, neg, &mut one_m_z4, n);
    one_m_z4[0] = one_m_z4[0] + F::new(1.0);

    // eclda1 = eu*(1 - f*z4) + ep*f*z4 - alfm*f*(1 - z4)/FZZ
    let mut term1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&eu, &one_m_z4f, &mut term1, n);
    let mut ep_z4f = Array::<F>::new(size);
    ctaylor_mul::<F>(&ep, &z4f, &mut ep_z4f, n);
    let mut term12 = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &ep_z4f, &mut term12, n);
    let mut f_one_m_z4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&f, &one_m_z4, &mut f_one_m_z4, n);
    let mut alfm_f_1mz4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&alfm, &f_one_m_z4, &mut alfm_f_1mz4, n);
    let mut alfm_f_1mz4_fzz = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &alfm_f_1mz4,
        F::cast_from(1.0 / FZZ_F64),
        &mut alfm_f_1mz4_fzz,
        n,
    );
    ctaylor_sub::<F>(&term12, &alfm_f_1mz4_fzz, eps, n);

    // d_eclda1_drs = (1-z4*f)*deudrs + z4*f*depdrs - (1-z4)*f*dalfmdrs/FZZ
    let mut dterm1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&one_m_z4f, &deudrs, &mut dterm1, n);
    let mut dterm2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z4f, &depdrs, &mut dterm2, n);
    let mut dterm12 = Array::<F>::new(size);
    ctaylor_add::<F>(&dterm1, &dterm2, &mut dterm12, n);
    let mut dalfm_fzz = Array::<F>::new(size);
    ctaylor_mul::<F>(&f_one_m_z4, &dalfmdrs, &mut dalfm_fzz, n);
    let mut dalfm_fzz_scaled = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &dalfm_fzz,
        F::cast_from(1.0 / FZZ_F64),
        &mut dalfm_fzz_scaled,
        n,
    );
    ctaylor_sub::<F>(&dterm12, &dalfm_fzz_scaled, eps_rs, n);
}

// ---------------------------------------------------------------------------
//  lda_0 — simple LDA baseline.  Port of SCAN_like_eps.hpp:378-385.
//  lda_0(rs) = -B1C / (1 + B2C*sqrt(rs) + B3C*rs)
// ---------------------------------------------------------------------------

#[cube]
pub fn lda_0<F: Float>(rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // sqrt(rs)
    let mut sqrtrs = Array::<F>::new(size);
    ctaylor_sqrt::<F>(rs, &mut sqrtrs, n);
    // B2C*sqrt(rs)
    let mut b2c_sqrtrs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrtrs, F::cast_from(B2C_F64), &mut b2c_sqrtrs, n);
    // B3C*rs
    let mut b3c_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::cast_from(B3C_F64), &mut b3c_rs, n);
    // 1 + B2C*sqrt(rs) + B3C*rs
    let mut denom = Array::<F>::new(size);
    ctaylor_add::<F>(&b2c_sqrtrs, &b3c_rs, &mut denom, n);
    denom[0] = denom[0] + F::new(1.0);
    // -B1C / denom
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    ctaylor_scalar_mul::<F>(&inv_denom, F::cast_from(-B1C_F64), out, n);
}

// ---------------------------------------------------------------------------
//  scan_ec0 — paramagnetic correlation.  Port of SCAN_like_eps.hpp:355-376.
// ---------------------------------------------------------------------------

#[cube]
pub fn scan_ec0<F: Float>(
    rs: &Array<F>,
    s: &Array<F>,
    zeta: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eclda = lda_0(rs)
    let mut eclda = Array::<F>::new(size);
    lda_0::<F>(rs, &mut eclda, n);

    // dx_z = ufunc(zeta, 4/3) / 2
    let mut uf_43 = Array::<F>::new(size);
    ufunc::<F>(zeta, F::cast_from(4.0_f64 / 3.0_f64), &mut uf_43, n);
    let mut dx_z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf_43, F::cast_from(0.5), &mut dx_z, n);

    // gc_z = (1 - 2.363*(dx_z - 1)) * (1 - zeta^12)
    // dx_z - 1
    let mut dx_z_m1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        dx_z_m1[i] = dx_z[i];
    }
    dx_z_m1[0] = dx_z_m1[0] - F::new(1.0);
    // 2.363*(dx_z-1)
    let mut scaled = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&dx_z_m1, F::cast_from(2.363), &mut scaled, n);
    // 1 - 2.363*(dx_z-1)
    let mut factor1 = Array::<F>::new(size);
    let neg = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&scaled, neg, &mut factor1, n);
    factor1[0] = factor1[0] + F::new(1.0);
    // zeta^12
    let mut z12 = Array::<F>::new(size);
    ctaylor_pow::<F>(zeta, F::cast_from(12.0_f64), &mut z12, n);
    // 1 - zeta^12
    let mut factor2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z12, neg, &mut factor2, n);
    factor2[0] = factor2[0] + F::new(1.0);
    // gc_z = factor1 * factor2
    let mut gc_z = Array::<F>::new(size);
    ctaylor_mul::<F>(&factor1, &factor2, &mut gc_z, n);

    // w0 = exp(-eclda/B1C) - 1
    let mut eclda_over_b1c = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eclda, F::cast_from(-1.0 / B1C_F64), &mut eclda_over_b1c, n);
    let mut exp_term = Array::<F>::new(size);
    ctaylor_exp::<F>(&eclda_over_b1c, &mut exp_term, n);
    let mut w0 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        w0[i] = exp_term[i];
    }
    w0[0] = w0[0] - F::new(1.0);

    // ginf = 1 / (1 + 4*CHI_LD*s^2)^(1/4)
    let mut s2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(s, &mut s2, n);
    let mut scaled_s2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2, F::cast_from(4.0 * CHI_LD_F64), &mut scaled_s2, n);
    let mut denom_ginf = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        denom_ginf[i] = scaled_s2[i];
    }
    denom_ginf[0] = denom_ginf[0] + F::new(1.0);
    // denom^(1/4)
    let mut denom_ginf_14 = Array::<F>::new(size);
    ctaylor_pow::<F>(&denom_ginf, F::cast_from(0.25_f64), &mut denom_ginf_14, n);
    let mut ginf = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_ginf_14, &mut ginf, n);

    // h0 = B1C * log(1 + w0*(1 - ginf))
    // (1 - ginf)
    let mut one_m_ginf = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ginf, neg, &mut one_m_ginf, n);
    one_m_ginf[0] = one_m_ginf[0] + F::new(1.0);
    // w0*(1-ginf)
    let mut w0_1mg = Array::<F>::new(size);
    ctaylor_mul::<F>(&w0, &one_m_ginf, &mut w0_1mg, n);
    // 1 + w0*(1-ginf)
    let mut log_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        log_arg[i] = w0_1mg[i];
    }
    log_arg[0] = log_arg[0] + F::new(1.0);
    let mut log_val = Array::<F>::new(size);
    ctaylor_log::<F>(&log_arg, &mut log_val, n);
    let mut h0 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&log_val, F::cast_from(B1C_F64), &mut h0, n);

    // out = (eclda + h0) * gc_z
    let mut eclda_h0 = Array::<F>::new(size);
    ctaylor_add::<F>(&eclda, &h0, &mut eclda_h0, n);
    ctaylor_mul::<F>(&eclda_h0, &gc_z, out, n);
}

// ---------------------------------------------------------------------------
//  scan_ec1 — coupling-constant integrated correlation.
//  Port of SCAN_like_eps.hpp:387-461.
// ---------------------------------------------------------------------------

#[cube]
pub fn scan_ec1<F: Float>(
    rs: &Array<F>,
    s: &Array<F>,
    zeta: &Array<F>,
    sqrtrs: &Array<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // dx_z = ufunc(zeta, 4/3) / 2
    let mut uf_43 = Array::<F>::new(size);
    ufunc::<F>(zeta, F::cast_from(4.0_f64 / 3.0_f64), &mut uf_43, n);
    let mut dx_z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf_43, F::cast_from(0.5), &mut dx_z, n);

    // gc_z = (1 - 2.363*(dx_z-1)) * (1 - zeta^12)
    let mut dx_z_m1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        dx_z_m1[i] = dx_z[i];
    }
    dx_z_m1[0] = dx_z_m1[0] - F::new(1.0);
    let mut scaled_dx = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&dx_z_m1, F::cast_from(2.363), &mut scaled_dx, n);
    let neg = F::new(0.0) - F::new(1.0);
    let mut factor1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&scaled_dx, neg, &mut factor1, n);
    factor1[0] = factor1[0] + F::new(1.0);
    let mut z12 = Array::<F>::new(size);
    ctaylor_pow::<F>(zeta, F::cast_from(12.0_f64), &mut z12, n);
    let mut factor2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&z12, neg, &mut factor2, n);
    factor2[0] = factor2[0] + F::new(1.0);
    let mut gc_z = Array::<F>::new(size);
    ctaylor_mul::<F>(&factor1, &factor2, &mut gc_z, n);

    // phi = ufunc(zeta, 2/3) / 2
    let mut uf_23 = Array::<F>::new(size);
    ufunc::<F>(zeta, F::cast_from(2.0_f64 / 3.0_f64), &mut uf_23, n);
    let mut phi = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf_23, F::cast_from(0.5), &mut phi, n);

    // phi3 = phi^3
    let mut phi3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&phi, &mut phi3, n);

    // eclda0 = lda_0(rs)
    let mut eclda0 = Array::<F>::new(size);
    lda_0::<F>(rs, &mut eclda0, n);

    // eclsda1, d_eclsda1_drs via get_lsda1
    let mut eclsda1 = Array::<F>::new(size);
    let mut d_eclsda1_drs = Array::<F>::new(size);
    get_lsda1::<F>(rs, zeta, sqrtrs, &mut eclsda1, &mut d_eclsda1_drs, n);

    // t = AFIX_T * s / (sqrtrs * phi)
    // sqrtrs * phi
    let mut sqrtrs_phi = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtrs, &phi, &mut sqrtrs_phi, n);
    let mut inv_sqrtrs_phi = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&sqrtrs_phi, &mut inv_sqrtrs_phi, n);
    let mut t = Array::<F>::new(size);
    ctaylor_mul::<F>(s, &inv_sqrtrs_phi, &mut t, n);
    let mut t_scaled = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&t, F::cast_from(AFIX_T_F64), &mut t_scaled, n);

    // w1 = exp(-eclsda1/(GAMMA*phi3)) - 1
    let mut gamma_phi3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&phi3, F::cast_from(GAMMA_F64), &mut gamma_phi3, n);
    let mut inv_gamma_phi3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gamma_phi3, &mut inv_gamma_phi3, n);
    let mut eclsda1_over = Array::<F>::new(size);
    ctaylor_mul::<F>(&eclsda1, &inv_gamma_phi3, &mut eclsda1_over, n);
    let mut neg_eclsda1_over = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&eclsda1_over, neg, &mut neg_eclsda1_over, n);
    let mut exp_w1 = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_eclsda1_over, &mut exp_w1, n);
    let mut w1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        w1[i] = exp_w1[i];
    }
    w1[0] = w1[0] - F::new(1.0);

    // beta = BETA_MB * (1 + AFACTOR*rs) / (1 + BFACTOR*rs)
    let mut af_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::cast_from(AFACTOR_F64), &mut af_rs, n);
    let mut num_beta = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        num_beta[i] = af_rs[i];
    }
    num_beta[0] = num_beta[0] + F::new(1.0);
    let mut bf_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rs, F::cast_from(BFACTOR_F64), &mut bf_rs, n);
    let mut den_beta = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den_beta[i] = bf_rs[i];
    }
    den_beta[0] = den_beta[0] + F::new(1.0);
    let mut inv_den_beta = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den_beta, &mut inv_den_beta, n);
    let mut ratio_beta = Array::<F>::new(size);
    ctaylor_mul::<F>(&num_beta, &inv_den_beta, &mut ratio_beta, n);
    let mut beta = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ratio_beta, F::cast_from(BETA_MB_F64), &mut beta, n);

    // y = beta / (GAMMA * w1) * t^2
    let mut t2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&t_scaled, &mut t2, n);
    let mut gamma_w1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&w1, F::cast_from(GAMMA_F64), &mut gamma_w1, n);
    let mut inv_gamma_w1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gamma_w1, &mut inv_gamma_w1, n);
    let mut beta_over = Array::<F>::new(size);
    ctaylor_mul::<F>(&beta, &inv_gamma_w1, &mut beta_over, n);
    let mut y = Array::<F>::new(size);
    ctaylor_mul::<F>(&beta_over, &t2, &mut y, n);

    // del_y: depends on idelec
    let mut del_y = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        del_y[i] = F::new(0.0);
    }

    if comptime!(idelec == 1 || idelec == 2) {
        // IDELEC 1 or 2: 2nd order GE correction for rSCAN interpolation
        // p = s^2
        let mut p_corr = Array::<F>::new(size);
        ctaylor_powi_2::<F>(s, &mut p_corr, n);

        // ds_z = ufunc(zeta, 5/3) / 2
        let mut uf_53 = Array::<F>::new(size);
        ufunc::<F>(zeta, F::cast_from(5.0_f64 / 3.0_f64), &mut uf_53, n);
        let mut ds_z = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&uf_53, F::cast_from(0.5), &mut ds_z, n);

        // eclsda0 = eclda0 * gc_z
        let mut eclsda0 = Array::<F>::new(size);
        ctaylor_mul::<F>(&eclda0, &gc_z, &mut eclsda0, n);

        // d_eclsda0_drs = gc_z * (B3C + B2C/(2*sqrtrs)) * eclda0^2 / B1C
        let mut inv_sqrtrs_2 = Array::<F>::new(size);
        ctaylor_pow::<F>(sqrtrs, F::cast_from(-1.0_f64), &mut inv_sqrtrs_2, n);
        let mut b2c_over_2sqrtrs = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &inv_sqrtrs_2,
            F::cast_from(B2C_F64 * 0.5),
            &mut b2c_over_2sqrtrs,
            n,
        );
        let mut b3c_plus = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            b3c_plus[i] = b2c_over_2sqrtrs[i];
        }
        b3c_plus[0] = b3c_plus[0] + F::cast_from(B3C_F64);
        let mut eclda0_sq = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&eclda0, &mut eclda0_sq, n);
        let mut frac_part = Array::<F>::new(size);
        ctaylor_mul::<F>(&b3c_plus, &eclda0_sq, &mut frac_part, n);
        let mut frac_scaled = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&frac_part, F::cast_from(1.0 / B1C_F64), &mut frac_scaled, n);
        let mut d_eclsda0_drs = Array::<F>::new(size);
        ctaylor_mul::<F>(&gc_z, &frac_scaled, &mut d_eclsda0_drs, n);

        // t1 = del_f2 / (27 * GAMMA * ds_z * phi3 * w1)
        // ds_z * phi3
        let mut ds_phi3 = Array::<F>::new(size);
        ctaylor_mul::<F>(&ds_z, &phi3, &mut ds_phi3, n);
        // ds_z * phi3 * w1
        let mut ds_phi3_w1 = Array::<F>::new(size);
        ctaylor_mul::<F>(&ds_phi3, &w1, &mut ds_phi3_w1, n);
        // 27 * GAMMA * ds_z * phi3 * w1
        let mut denom_t1 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &ds_phi3_w1,
            F::cast_from(27.0 * GAMMA_F64),
            &mut denom_t1,
            n,
        );
        let mut inv_denom_t1 = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom_t1, &mut inv_denom_t1, n);
        let mut t1 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&inv_denom_t1, F::cast_from(DEL_F2_C), &mut t1, n);

        // t2 = 20*rs*(d_eclsda0_drs - d_eclsda1_drs)
        let mut diff_drs = Array::<F>::new(size);
        ctaylor_sub::<F>(&d_eclsda0_drs, &d_eclsda1_drs, &mut diff_drs, n);
        let mut t2_mul_rs = Array::<F>::new(size);
        ctaylor_mul::<F>(rs, &diff_drs, &mut t2_mul_rs, n);
        let mut t2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&t2_mul_rs, F::cast_from(20.0), &mut t2, n);

        // t3 = 45*ETA*(eclsda0 - eclsda1)
        let mut diff_ec = Array::<F>::new(size);
        ctaylor_sub::<F>(&eclsda0, &eclsda1, &mut diff_ec, n);
        let mut t3 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&diff_ec, F::cast_from(45.0 * 1.0e-3), &mut t3, n);

        // k = t1 * (t2 - t3)
        let mut t2_m_t3 = Array::<F>::new(size);
        ctaylor_sub::<F>(&t2, &t3, &mut t2_m_t3, n);
        let mut k = Array::<F>::new(size);
        ctaylor_mul::<F>(&t1, &t2_m_t3, &mut k, n);

        // damp = exp(-p^2 / DAMP_DENOM_C)  (DAMP_DENOM_C = D_DAMP2_C^4)
        let mut p2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&p_corr, &mut p2, n);
        let mut neg_p2_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &p2,
            F::cast_from(-1.0_f64 / DAMP_DENOM_C),
            &mut neg_p2_over,
            n,
        );
        let mut damp = Array::<F>::new(size);
        ctaylor_exp::<F>(&neg_p2_over, &mut damp, n);

        // del_y = k * p * damp
        let mut k_p = Array::<F>::new(size);
        ctaylor_mul::<F>(&k, &p_corr, &mut k_p, n);
        ctaylor_mul::<F>(&k_p, &damp, &mut del_y, n);
    }

    // g_y = 1 / (1 + 4*(y - del_y))^(1/4)
    let mut y_m_dely = Array::<F>::new(size);
    ctaylor_sub::<F>(&y, &del_y, &mut y_m_dely, n);
    let mut four_y_m_dely = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&y_m_dely, F::cast_from(4.0), &mut four_y_m_dely, n);
    let mut one_plus_4y = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus_4y[i] = four_y_m_dely[i];
    }
    one_plus_4y[0] = one_plus_4y[0] + F::new(1.0);
    let mut one_plus_4y_14 = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_plus_4y, F::cast_from(0.25_f64), &mut one_plus_4y_14, n);
    let mut g_y = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_plus_4y_14, &mut g_y, n);

    // h1 = GAMMA * phi3 * log(1 + w1*(1 - g_y))
    let mut one_m_gy = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_y, neg, &mut one_m_gy, n);
    one_m_gy[0] = one_m_gy[0] + F::new(1.0);
    let mut w1_1mgy = Array::<F>::new(size);
    ctaylor_mul::<F>(&w1, &one_m_gy, &mut w1_1mgy, n);
    let mut log_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        log_arg[i] = w1_1mgy[i];
    }
    log_arg[0] = log_arg[0] + F::new(1.0);
    let mut log_val = Array::<F>::new(size);
    ctaylor_log::<F>(&log_arg, &mut log_val, n);
    let mut gamma_phi3_log = Array::<F>::new(size);
    ctaylor_mul::<F>(&phi3, &log_val, &mut gamma_phi3_log, n);
    let mut h1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&gamma_phi3_log, F::cast_from(GAMMA_F64), &mut h1, n);

    // out = eclsda1 + h1
    ctaylor_add::<F>(&eclsda1, &h1, out, n);
}

// ---------------------------------------------------------------------------
//  SCAN_X_Fx — exchange enhancement factor.
//  Port of SCAN_like_eps.hpp:132-251.
// ---------------------------------------------------------------------------

/// SCAN-family exchange enhancement factor `F_x(p, alpha)`.
/// All three comptime params come from the caller's `get_SCAN_Fx`.
#[cube]
fn SCAN_X_Fx<F: Float>(
    p: &Array<F>,
    alpha: &Array<F>,
    #[comptime] iinterp: u32,
    #[comptime] idelfx: u32,
    #[comptime] n: u32,
) -> Array<F> {
    let size = comptime!((1_u32 << n) as usize);
    let neg = F::new(0.0) - F::new(1.0);

    // oma = 1 - alpha
    let mut oma = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(alpha, neg, &mut oma, n);
    oma[0] = oma[0] + F::new(1.0);

    // Interpolation function ief
    let mut ief = Array::<F>::new(size);
    if comptime!(iinterp == 0) {
        // SCAN: if alpha < 1 => exp(-CFX1*alpha/oma), else => -CFDX1*exp(CFX2/oma)
        // Use CNST slot to branch
        let alpha_cnst = alpha[0];
        if alpha_cnst < F::new(1.0) {
            // exp(-CFX1 * alpha / oma)
            let mut alpha_over_oma = Array::<F>::new(size);
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            ctaylor_mul::<F>(alpha, &inv_oma, &mut alpha_over_oma, n);
            let mut neg_cfx1_aoo = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(
                &alpha_over_oma,
                F::cast_from(-CFX1_F64),
                &mut neg_cfx1_aoo,
                n,
            );
            ctaylor_exp::<F>(&neg_cfx1_aoo, &mut ief, n);
        } else {
            // -CFDX1 * exp(CFX2/oma)
            let mut cfx2_over_oma = Array::<F>::new(size);
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            ctaylor_scalar_mul::<F>(&inv_oma, F::cast_from(CFX2_F64), &mut cfx2_over_oma, n);
            let mut exp_val = Array::<F>::new(size);
            ctaylor_exp::<F>(&cfx2_over_oma, &mut exp_val, n);
            ctaylor_scalar_mul::<F>(&exp_val, F::cast_from(-CFDX1_F64), &mut ief, n);
        }
    } else {
        // rSCAN (iinterp == 1): polynomial in alpha, with 3 branches
        let alpha_cnst = alpha[0];
        if alpha_cnst < F::new(1.0e-13) {
            // exp(-CFX1*alpha/oma)
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut aoo = Array::<F>::new(size);
            ctaylor_mul::<F>(alpha, &inv_oma, &mut aoo, n);
            let mut neg_cfx1 = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&aoo, F::cast_from(-CFX1_F64), &mut neg_cfx1, n);
            ctaylor_exp::<F>(&neg_cfx1, &mut ief, n);
        } else if alpha_cnst < F::new(2.5) {
            // Polynomial: sum_{i=0}^{7} IE_PARAMS_X[i] * alpha^i
            // Start with IE_PARAMS_X[7]*alpha^7 and use Horner's method
            // alpha^0 contribution
            ief[0] = F::cast_from(IE_PARAMS_X[0]);
            #[unroll]
            for i in 1..size {
                ief[i] = F::new(0.0);
            }
            // alpha^1 term
            let mut alpha_i = Array::<F>::new(size);
            #[unroll]
            for i in 0..size {
                alpha_i[i] = alpha[i];
            }
            let mut term = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&alpha_i, F::cast_from(IE_PARAMS_X[1]), &mut term, n);
            let mut ief_raw = Array::<F>::new(size);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^2
            let mut alpha2 = Array::<F>::new(size);
            ctaylor_powi_2::<F>(alpha, &mut alpha2, n);
            ctaylor_scalar_mul::<F>(&alpha2, F::cast_from(IE_PARAMS_X[2]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^3
            let mut alpha3 = Array::<F>::new(size);
            ctaylor_powi_3::<F>(alpha, &mut alpha3, n);
            ctaylor_scalar_mul::<F>(&alpha3, F::cast_from(IE_PARAMS_X[3]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^4
            let mut alpha4 = Array::<F>::new(size);
            ctaylor_powi_4::<F>(alpha, &mut alpha4, n);
            ctaylor_scalar_mul::<F>(&alpha4, F::cast_from(IE_PARAMS_X[4]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^5
            let mut alpha5 = Array::<F>::new(size);
            let mut alpha5_raw = Array::<F>::new(size);
            ctaylor_mul::<F>(&alpha4, alpha, &mut alpha5_raw, n);
            #[unroll]
            for i in 0..size {
                alpha5[i] = alpha5_raw[i];
            }
            ctaylor_scalar_mul::<F>(&alpha5, F::cast_from(IE_PARAMS_X[5]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^6
            let mut alpha6 = Array::<F>::new(size);
            ctaylor_mul::<F>(&alpha5, alpha, &mut alpha6, n);
            ctaylor_scalar_mul::<F>(&alpha6, F::cast_from(IE_PARAMS_X[6]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
            // alpha^7
            let mut alpha7 = Array::<F>::new(size);
            ctaylor_mul::<F>(&alpha6, alpha, &mut alpha7, n);
            ctaylor_scalar_mul::<F>(&alpha7, F::cast_from(IE_PARAMS_X[7]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_raw, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_raw[i];
            }
        } else {
            // -CFDX1 * exp(CFX2/oma)
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut cfx2_oma = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&inv_oma, F::cast_from(CFX2_F64), &mut cfx2_oma, n);
            let mut exp_val = Array::<F>::new(size);
            ctaylor_exp::<F>(&cfx2_oma, &mut exp_val, n);
            ctaylor_scalar_mul::<F>(&exp_val, F::cast_from(-CFDX1_F64), &mut ief, n);
        }
    }

    // h0x = 1 + K0  (module-level const H0X_VAL)
    // h1x depends on IDELFX
    let mut h1x = Array::<F>::new(size);
    if comptime!(idelfx == 0) {
        // SCAN: 2nd and 4th order GE corrections
        // wfac = B4*p^2 * exp(-B4*p/MU)
        let mut p2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(p, &mut p2, n);
        let mut b4p2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p2, F::cast_from(B4_F64), &mut b4p2, n);
        let mut neg_b4p_mu = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(p, F::cast_from(-B4_F64 / MU_F64), &mut neg_b4p_mu, n);
        let mut exp_b4 = Array::<F>::new(size);
        ctaylor_exp::<F>(&neg_b4p_mu, &mut exp_b4, n);
        let mut wfac = Array::<F>::new(size);
        ctaylor_mul::<F>(&b4p2, &exp_b4, &mut wfac, n);
        // vfac = B1*p + B2*oma*exp(-B3*oma^2)
        let mut b1p = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(p, F::cast_from(B1_F64), &mut b1p, n);
        let mut oma2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&oma, &mut oma2, n);
        let mut neg_b3_oma2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&oma2, F::cast_from(-B3_F64), &mut neg_b3_oma2, n);
        let mut exp_oma = Array::<F>::new(size);
        ctaylor_exp::<F>(&neg_b3_oma2, &mut exp_oma, n);
        let mut b2_oma_exp = Array::<F>::new(size);
        ctaylor_mul::<F>(&oma, &exp_oma, &mut b2_oma_exp, n);
        let mut b2_oma_exp_scaled = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&b2_oma_exp, F::cast_from(B2_F64), &mut b2_oma_exp_scaled, n);
        let mut vfac = Array::<F>::new(size);
        ctaylor_add::<F>(&b1p, &b2_oma_exp_scaled, &mut vfac, n);
        // yfac = MU*p + wfac + vfac^2
        let mut mu_p = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(p, F::cast_from(MU_F64), &mut mu_p, n);
        let mut mu_p_wfac = Array::<F>::new(size);
        ctaylor_add::<F>(&mu_p, &wfac, &mut mu_p_wfac, n);
        let mut vfac2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&vfac, &mut vfac2, n);
        let mut yfac = Array::<F>::new(size);
        ctaylor_add::<F>(&mu_p_wfac, &vfac2, &mut yfac, n);
        // h1x = 1 + K1 - K1/(1 + yfac/K1)
        let mut yfac_k1 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&yfac, F::cast_from(1.0 / K1_F64), &mut yfac_k1, n);
        let mut denom_h1 = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            denom_h1[i] = yfac_k1[i];
        }
        denom_h1[0] = denom_h1[0] + F::new(1.0);
        let mut inv_denom_h1 = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom_h1, &mut inv_denom_h1, n);
        let mut k1_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&inv_denom_h1, F::cast_from(K1_F64), &mut k1_over, n);
        ctaylor_scalar_mul::<F>(&k1_over, neg, &mut h1x, n);
        h1x[0] = h1x[0] + F::cast_from(H1X_CNST);
    } else {
        // idelfx == 1 or 2: rSCAN 2nd order GE corrections (C2_X = DEL_F2_X * K0_F64)
        // damp = exp(-p^2 / DAMP_DENOM_X)
        let mut p2_h = Array::<F>::new(size);
        ctaylor_powi_2::<F>(p, &mut p2_h, n);
        let mut neg_p2_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &p2_h,
            F::cast_from(-1.0_f64 / DAMP_DENOM_X),
            &mut neg_p2_over,
            n,
        );
        let mut damp = Array::<F>::new(size);
        ctaylor_exp::<F>(&neg_p2_over, &mut damp, n);
        // h1x = 1 + K1 - K1/(1 + p*(MU + ALPHA_GE*C2_X*damp)/K1)
        let mut alpha_ge_c2_damp = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &damp,
            F::cast_from(ALPHA_GE_F64 * C2_X),
            &mut alpha_ge_c2_damp,
            n,
        );
        let mut mu_plus = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            mu_plus[i] = alpha_ge_c2_damp[i];
        }
        mu_plus[0] = mu_plus[0] + F::cast_from(MU_F64);
        let mut p_times = Array::<F>::new(size);
        ctaylor_mul::<F>(p, &mu_plus, &mut p_times, n);
        let mut p_times_k1 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p_times, F::cast_from(1.0 / K1_F64), &mut p_times_k1, n);
        let mut denom_h1 = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            denom_h1[i] = p_times_k1[i];
        }
        denom_h1[0] = denom_h1[0] + F::new(1.0);
        let mut inv_denom_h1 = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom_h1, &mut inv_denom_h1, n);
        let mut k1_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&inv_denom_h1, F::cast_from(K1_F64), &mut k1_over, n);
        ctaylor_scalar_mul::<F>(&k1_over, neg, &mut h1x, n);
        h1x[0] = h1x[0] + F::cast_from(H1X_CNST);
    }

    // gx = 1 - exp(-A1/p^(1/4))
    let mut p_14 = Array::<F>::new(size);
    ctaylor_pow::<F>(p, F::cast_from(0.25_f64), &mut p_14, n);
    let mut inv_p14 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&p_14, &mut inv_p14, n);
    let mut neg_a1_over = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_p14, F::cast_from(-A1_F64), &mut neg_a1_over, n);
    let mut exp_gx = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_a1_over, &mut exp_gx, n);
    let mut gx = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&exp_gx, neg, &mut gx, n);
    gx[0] = gx[0] + F::new(1.0);

    // del_fx (only for idelfx == 2, r4SCAN)
    let mut del_fx = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        del_fx[i] = F::new(0.0);
    }
    if comptime!(idelfx == 2) {
        // All coefficients are module-level consts: C2_X, C_AA, C_PA, C_PP,
        // DAMP4_A_SQ, DAMP4_P4 (all derived from DEL_F2_X, DEL_F4_X, H0X_VAL, etc.)

        // order_1 = C2_X*(oma - ALPHA_GE*p)
        let mut alpha_ge_p = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(p, F::cast_from(ALPHA_GE_F64), &mut alpha_ge_p, n);
        let mut oma_m_age_p = Array::<F>::new(size);
        ctaylor_sub::<F>(&oma, &alpha_ge_p, &mut oma_m_age_p, n);
        let mut order_1 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&oma_m_age_p, F::cast_from(C2_X), &mut order_1, n);

        // C_AA * oma^2
        let mut oma2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&oma, &mut oma2, n);
        let mut c_aa_oma2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&oma2, F::cast_from(C_AA), &mut c_aa_oma2, n);

        // C_PA * p * oma
        let mut p_oma = Array::<F>::new(size);
        ctaylor_mul::<F>(p, &oma, &mut p_oma, n);
        let mut c_pa_p_oma = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p_oma, F::cast_from(C_PA), &mut c_pa_p_oma, n);

        // C_PP * p^2
        let mut p2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(p, &mut p2, n);
        let mut c_pp_p2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p2, F::cast_from(C_PP), &mut c_pp_p2, n);

        // t1 = order_1 + C_AA*oma^2 + C_PA*p*oma + C_PP*p^2
        let mut t1_raw = Array::<F>::new(size);
        ctaylor_add::<F>(&order_1, &c_aa_oma2, &mut t1_raw, n);
        let mut t1_raw2 = Array::<F>::new(size);
        ctaylor_add::<F>(&t1_raw, &c_pa_p_oma, &mut t1_raw2, n);
        let mut t1 = Array::<F>::new(size);
        ctaylor_add::<F>(&t1_raw2, &c_pp_p2, &mut t1, n);

        // damp_4_t1 = 2*alpha^2 / (1 + alpha^4)
        let mut alpha2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(alpha, &mut alpha2, n);
        let mut two_alpha2 = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&alpha2, F::cast_from(2.0), &mut two_alpha2, n);
        let mut alpha4 = Array::<F>::new(size);
        ctaylor_powi_4::<F>(alpha, &mut alpha4, n);
        let mut one_plus_a4 = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            one_plus_a4[i] = alpha4[i];
        }
        one_plus_a4[0] = one_plus_a4[0] + F::new(1.0);
        let mut inv_1_plus_a4 = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&one_plus_a4, &mut inv_1_plus_a4, n);
        let mut damp_4_t1 = Array::<F>::new(size);
        ctaylor_mul::<F>(&two_alpha2, &inv_1_plus_a4, &mut damp_4_t1, n);

        // damp_4_t2 = exp(-oma^2/DAMP4_A_SQ - p^2/DAMP4_P4)
        let mut oma2_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &oma2,
            F::cast_from(-1.0_f64 / DAMP4_A_SQ),
            &mut oma2_over,
            n,
        );
        let mut p2_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p2, F::cast_from(-1.0_f64 / DAMP4_P4), &mut p2_over, n);
        let mut sum_exp = Array::<F>::new(size);
        ctaylor_add::<F>(&oma2_over, &p2_over, &mut sum_exp, n);
        let mut damp_4_t2 = Array::<F>::new(size);
        ctaylor_exp::<F>(&sum_exp, &mut damp_4_t2, n);

        // damp_4 = damp_4_t1 * damp_4_t2
        let mut damp_4 = Array::<F>::new(size);
        ctaylor_mul::<F>(&damp_4_t1, &damp_4_t2, &mut damp_4, n);

        // del_fx = t1 * damp_4
        ctaylor_mul::<F>(&t1, &damp_4, &mut del_fx, n);
    }

    // fx = (h1x + ief*(h0x - h1x) + del_fx) * gx
    // ief*(h0x - h1x): first compute h0x - h1x  (h0x = H0X_VAL)
    let mut h0x_m_h1x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&h1x, neg, &mut h0x_m_h1x, n);
    h0x_m_h1x[0] = h0x_m_h1x[0] + F::cast_from(H0X_VAL);
    let mut ief_h0x_h1x = Array::<F>::new(size);
    ctaylor_mul::<F>(&ief, &h0x_m_h1x, &mut ief_h0x_h1x, n);
    // h1x + ief*(h0x-h1x) + del_fx
    let mut inner1 = Array::<F>::new(size);
    ctaylor_add::<F>(&h1x, &ief_h0x_h1x, &mut inner1, n);
    let mut inner2 = Array::<F>::new(size);
    ctaylor_add::<F>(&inner1, &del_fx, &mut inner2, n);
    // * gx
    let mut result = Array::<F>::new(size);
    ctaylor_mul::<F>(&inner2, &gx, &mut result, n);
    result
}

// ---------------------------------------------------------------------------
//  get_SCAN_Fx — exchange entry point.  Port of SCAN_like_eps.hpp:75-130.
// ---------------------------------------------------------------------------

/// SCAN-family exchange enhancement factor entry point.
///
/// Comptime params: `ialpha` (0/1/2), `iinterp` (0/1), `idelfx` (0/1/2).
/// Returns `F_x(d_n, d_g, d_tau)` (scalar factor; caller multiplies by `fx_unif`).
#[cube]
pub fn get_SCAN_Fx<F: Float>(
    d_n: &Array<F>,
    d_g: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] ialpha: u32,
    #[comptime] iinterp: u32,
    #[comptime] idelfx: u32,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let _neg = F::new(0.0) - F::new(1.0);

    // tauw = d_g / (8 * d_n)
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(d_n, &mut inv_n, n);
    let mut tauw_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(d_g, &inv_n, &mut tauw_raw, n);
    let mut tauw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&tauw_raw, F::cast_from(1.0 / 8.0), &mut tauw, n);

    // tauUnif
    const COEFF_53: f64 = 0.3 * 9.575_178_436_213_78; // 0.3*(3π²)^(2/3)
    let mut n_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(d_n, F::cast_from(5.0_f64 / 3.0_f64), &mut n_53, n);
    let mut tau_unif = Array::<F>::new(size);
    if comptime!(ialpha == 1) {
        // tauUnif = 0.3*(3π²)^(2/3)*n^(5/3) + TAU_R  (TAU_R=1e-4)
        ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_53), &mut tau_unif, n);
        tau_unif[0] = tau_unif[0] + F::cast_from(1.0e-4_f64);
    } else {
        ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_53), &mut tau_unif, n);
    }

    // alpha — branches on ialpha
    let mut alpha = Array::<F>::new(size);
    if comptime!(ialpha == 0) {
        // SCAN: alpha = (tau - tauw) / tauUnif, guarded by abs check on CNST
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(d_tau, &tauw, &mut tau_m_tauw, n);
        let diff_cnst = tau_m_tauw[0];
        if diff_cnst < F::new(0.0) {
            // abs(diff) > 1e-14 → use 0 (below threshold → alpha=0)
            // diff is negative, so |diff| > 0 but we need to check magnitude
            let abs_diff = F::new(0.0) - diff_cnst;
            if abs_diff > F::new(1.0e-14) {
                let mut inv_tu = Array::<F>::new(size);
                ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tu, n);
                ctaylor_mul::<F>(&tau_m_tauw, &inv_tu, &mut alpha, n);
            } else {
                #[unroll]
                for i in 0..size {
                    alpha[i] = F::new(0.0);
                }
            }
        } else if diff_cnst > F::new(1.0e-14) {
            let mut inv_tu = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tu, n);
            ctaylor_mul::<F>(&tau_m_tauw, &inv_tu, &mut alpha, n);
        } else {
            #[unroll]
            for i in 0..size {
                alpha[i] = F::new(0.0);
            }
        }
    } else if comptime!(ialpha == 1) {
        // rSCAN: alpha' = (tau-tauw)/tauUnif, then alpha'^3/(alpha'^2 + A_REG)
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(d_tau, &tauw, &mut tau_m_tauw, n);
        let mut inv_tu = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tu, n);
        let mut alpha_raw = Array::<F>::new(size);
        ctaylor_mul::<F>(&tau_m_tauw, &inv_tu, &mut alpha_raw, n);
        // alpha'^3 / (alpha'^2 + A_REG)
        let mut a3 = Array::<F>::new(size);
        ctaylor_powi_3::<F>(&alpha_raw, &mut a3, n);
        let mut a2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&alpha_raw, &mut a2, n);
        let mut a2_plus = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            a2_plus[i] = a2[i];
        }
        a2_plus[0] = a2_plus[0] + F::cast_from(1.0e-3_f64); // A_REG
        let mut inv_a2_plus = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&a2_plus, &mut inv_a2_plus, n);
        ctaylor_mul::<F>(&a3, &inv_a2_plus, &mut alpha, n);
    } else {
        // ialpha == 2: bar-alpha = (tau - tauw) / (tauUnif + ETA*tauw)
        // ETA = 1e-3
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(d_tau, &tauw, &mut tau_m_tauw, n);
        let diff_cnst = tau_m_tauw[0];
        let mut eta_tauw = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&tauw, F::cast_from(1.0e-3_f64), &mut eta_tauw, n);
        let mut denom_alpha = Array::<F>::new(size);
        ctaylor_add::<F>(&tau_unif, &eta_tauw, &mut denom_alpha, n);
        if diff_cnst < F::new(0.0) {
            let abs_diff = F::new(0.0) - diff_cnst;
            if abs_diff > F::new(1.0e-14) {
                let mut inv_da = Array::<F>::new(size);
                ctaylor_reciprocal::<F>(&denom_alpha, &mut inv_da, n);
                ctaylor_mul::<F>(&tau_m_tauw, &inv_da, &mut alpha, n);
            } else {
                #[unroll]
                for i in 0..size {
                    alpha[i] = F::new(0.0);
                }
            }
        } else if diff_cnst > F::new(1.0e-14) {
            let mut inv_da = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&denom_alpha, &mut inv_da, n);
            ctaylor_mul::<F>(&tau_m_tauw, &inv_da, &mut alpha, n);
        } else {
            #[unroll]
            for i in 0..size {
                alpha[i] = F::new(0.0);
            }
        }
    }

    // p = d_g / (4*(3π²)^(2/3) * d_n^(8/3))
    // If d_g CNST near 0, use 1e-16 instead (per C++ guard)
    const FOUR_3PI2_23: f64 = 38.283_120_002_509_214_f64; // 4*(3π²)^(2/3)
    let dg_cnst = d_g[0];
    let mut p = Array::<F>::new(size);
    {
        let mut n_m83 = Array::<F>::new(size);
        ctaylor_pow::<F>(d_n, F::cast_from(-8.0_f64 / 3.0_f64), &mut n_m83, n);
        let inv4_3pi2 = 1.0 / FOUR_3PI2_23;
        if dg_cnst > F::new(1.0e-16) || dg_cnst < F::new(-1.0e-16) {
            let mut p_raw = Array::<F>::new(size);
            ctaylor_mul::<F>(d_g, &n_m83, &mut p_raw, n);
            ctaylor_scalar_mul::<F>(&p_raw, F::cast_from(inv4_3pi2), &mut p, n);
        } else {
            // use 1e-16 as numerator (scalar guard)
            ctaylor_scalar_mul::<F>(&n_m83, F::cast_from(1.0e-16 * inv4_3pi2), &mut p, n);
        }
    }

    // Fx = SCAN_X_Fx(p, alpha, iinterp, idelfx)
    let fx = SCAN_X_Fx::<F>(&p, &alpha, iinterp, idelfx, n);
    #[unroll]
    for i in 0..size {
        out[i] = fx[i];
    }
}

// ---------------------------------------------------------------------------
//  r2SCAN_C (SCAN_C) — correlation entry point.
//  Port of SCAN_like_eps.hpp:253-353.
// ---------------------------------------------------------------------------

/// SCAN-family correlation energy density.
/// Takes DensVarsDev directly — uses pre-computed d.n, d.gnn, d.zeta, d.tau.
/// Comptime params: `ialpha`, `iinterp`, `idelec` (same meanings as exchange).
#[cube]
pub fn r2SCAN_C<F: Float>(
    d: &crate::density_vars::DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] ialpha: u32,
    #[comptime] iinterp: u32,
    #[comptime] idelec: u32,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let neg = F::new(0.0) - F::new(1.0);

    // Use pre-computed derived quantities from DensVarsDev:
    //   d.n    = a + b
    //   d.gnn  = gaa + 2*gab + gbb  (correct cross-term included)
    //   d.zeta = (a - b) / n
    //   d.tau  = taua + taub
    let n_dens = &d.n;
    let gnn = &d.gnn;
    let zeta = &d.zeta;
    let tau = &d.tau;
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(n_dens, &mut inv_n, n);

    // rs = (4*π*n/3)^(-1/3)
    // = (4*π/3)^(-1/3) * n^(-1/3)
    // (4π/3)^(-1/3) = (4.18879...)^(-1/3) = 0.6204...
    const FOUR_PI_3_INV13: f64 = 0.620_350_490_899_400_16_f64; // (4π/3)^(-1/3)
    let mut n_m13 = Array::<F>::new(size);
    ctaylor_pow::<F>(n_dens, F::cast_from(-1.0_f64 / 3.0_f64), &mut n_m13, n);
    let mut rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n_m13, F::cast_from(FOUR_PI_3_INV13), &mut rs, n);
    // sqrtrs
    let rs_cnst = rs[0];
    let mut sqrtrs = Array::<F>::new(size);
    if rs_cnst > F::new(1.0e-16) {
        ctaylor_sqrt::<F>(&rs, &mut sqrtrs, n);
    } else {
        #[unroll]
        for i in 0..size {
            sqrtrs[i] = F::new(0.0);
        }
    }

    // ds_z = ufunc(zeta, 5/3) / 2
    let mut uf_53 = Array::<F>::new(size);
    ufunc::<F>(zeta, F::cast_from(5.0_f64 / 3.0_f64), &mut uf_53, n);
    let mut ds_z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&uf_53, F::cast_from(0.5), &mut ds_z, n);

    // s = sqrt(gnn) / (2*(3π²)^(1/3) * n^(4/3))
    // (3π²)^(1/3) = 3.093...
    // 2*(3π²)^(1/3) = 6.187...
    const TWO_3PI2_13: f64 = 6.187_335_472_212_163_f64;
    let mut sqrt_gnn = Array::<F>::new(size);
    ctaylor_sqrt::<F>(gnn, &mut sqrt_gnn, n);
    let mut n_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(n_dens, F::cast_from(4.0_f64 / 3.0_f64), &mut n_43, n);
    let mut denom_s = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n_43, F::cast_from(TWO_3PI2_13), &mut denom_s, n);
    let mut inv_denom_s = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_s, &mut inv_denom_s, n);
    let mut s = Array::<F>::new(size);
    ctaylor_mul::<F>(&sqrt_gnn, &inv_denom_s, &mut s, n);

    // tueg = 0.3*(3π²)^(2/3)*n^(5/3)*ds_z  (or + TAU_R * ds_z if ialpha==1)
    const COEFF_53: f64 = 2.871_234_000_188_191_f64; // 0.3*(3π²)^(2/3)
    let mut n_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(n_dens, F::cast_from(5.0_f64 / 3.0_f64), &mut n_53, n);
    let mut tueg_base = Array::<F>::new(size);
    if comptime!(ialpha == 1) {
        // tueg_con * n^(5/3) + TAU_R
        let mut tc = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_53), &mut tc, n);
        tc[0] = tc[0] + F::cast_from(1.0e-4_f64);
        ctaylor_mul::<F>(&tc, &ds_z, &mut tueg_base, n);
    } else {
        let mut tc = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&n_53, F::cast_from(COEFF_53), &mut tc, n);
        ctaylor_mul::<F>(&tc, &ds_z, &mut tueg_base, n);
    }

    // tauw = gnn / (8*n)
    let mut tauw_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(gnn, &inv_n, &mut tauw_raw, n);
    let mut tauw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&tauw_raw, F::cast_from(1.0 / 8.0), &mut tauw, n);

    // alpha — branches on ialpha
    let mut alpha = Array::<F>::new(size);
    if comptime!(ialpha == 0) {
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(tau, &tauw, &mut tau_m_tauw, n);
        let diff_cnst = tau_m_tauw[0];
        if diff_cnst < F::new(0.0) {
            let abs_diff = F::new(0.0) - diff_cnst;
            if abs_diff > F::new(1.0e-14) {
                let mut inv_tueg = Array::<F>::new(size);
                ctaylor_reciprocal::<F>(&tueg_base, &mut inv_tueg, n);
                ctaylor_mul::<F>(&tau_m_tauw, &inv_tueg, &mut alpha, n);
            } else {
                #[unroll]
                for i in 0..size {
                    alpha[i] = F::new(0.0);
                }
            }
        } else if diff_cnst > F::new(1.0e-14) {
            let mut inv_tueg = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&tueg_base, &mut inv_tueg, n);
            ctaylor_mul::<F>(&tau_m_tauw, &inv_tueg, &mut alpha, n);
        } else {
            #[unroll]
            for i in 0..size {
                alpha[i] = F::new(0.0);
            }
        }
    } else if comptime!(ialpha == 1) {
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(tau, &tauw, &mut tau_m_tauw, n);
        let mut inv_tueg = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&tueg_base, &mut inv_tueg, n);
        let mut alpha_raw = Array::<F>::new(size);
        ctaylor_mul::<F>(&tau_m_tauw, &inv_tueg, &mut alpha_raw, n);
        let mut a3 = Array::<F>::new(size);
        ctaylor_powi_3::<F>(&alpha_raw, &mut a3, n);
        let mut a2 = Array::<F>::new(size);
        ctaylor_powi_2::<F>(&alpha_raw, &mut a2, n);
        let mut a2_reg = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            a2_reg[i] = a2[i];
        }
        a2_reg[0] = a2_reg[0] + F::cast_from(1.0e-3_f64);
        let mut inv_a2_reg = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&a2_reg, &mut inv_a2_reg, n);
        ctaylor_mul::<F>(&a3, &inv_a2_reg, &mut alpha, n);
    } else {
        // ialpha == 2
        let mut tau_m_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(tau, &tauw, &mut tau_m_tauw, n);
        let diff_cnst = tau_m_tauw[0];
        let mut eta_tauw = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&tauw, F::cast_from(1.0e-3_f64), &mut eta_tauw, n);
        let mut denom_alpha = Array::<F>::new(size);
        ctaylor_add::<F>(&tueg_base, &eta_tauw, &mut denom_alpha, n);
        if diff_cnst < F::new(0.0) {
            let abs_diff = F::new(0.0) - diff_cnst;
            if abs_diff > F::new(1.0e-14) {
                let mut inv_da = Array::<F>::new(size);
                ctaylor_reciprocal::<F>(&denom_alpha, &mut inv_da, n);
                ctaylor_mul::<F>(&tau_m_tauw, &inv_da, &mut alpha, n);
            } else {
                #[unroll]
                for i in 0..size {
                    alpha[i] = F::new(0.0);
                }
            }
        } else if diff_cnst > F::new(1.0e-14) {
            let mut inv_da = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&denom_alpha, &mut inv_da, n);
            ctaylor_mul::<F>(&tau_m_tauw, &inv_da, &mut alpha, n);
        } else {
            #[unroll]
            for i in 0..size {
                alpha[i] = F::new(0.0);
            }
        }
    }

    // Interpolation function ief (correlation)
    let mut oma = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&alpha, neg, &mut oma, n);
    oma[0] = oma[0] + F::new(1.0);

    let mut ief = Array::<F>::new(size);
    if comptime!(iinterp == 0) {
        let alpha_cnst = alpha[0];
        if alpha_cnst < F::new(1.0) {
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut aoo = Array::<F>::new(size);
            ctaylor_mul::<F>(&alpha, &inv_oma, &mut aoo, n);
            let mut neg_cfc1_aoo = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&aoo, F::cast_from(-CFC1_F64), &mut neg_cfc1_aoo, n);
            ctaylor_exp::<F>(&neg_cfc1_aoo, &mut ief, n);
        } else {
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut cfc2_oma = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&inv_oma, F::cast_from(CFC2_F64), &mut cfc2_oma, n);
            let mut exp_val = Array::<F>::new(size);
            ctaylor_exp::<F>(&cfc2_oma, &mut exp_val, n);
            ctaylor_scalar_mul::<F>(&exp_val, F::cast_from(-CFDC1_F64), &mut ief, n);
        }
    } else {
        // iinterp == 1: rSCAN polynomial
        let alpha_cnst = alpha[0];
        if alpha_cnst < F::new(1.0e-13) {
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut aoo = Array::<F>::new(size);
            ctaylor_mul::<F>(&alpha, &inv_oma, &mut aoo, n);
            let mut neg_cfc1_aoo = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&aoo, F::cast_from(-CFC1_F64), &mut neg_cfc1_aoo, n);
            ctaylor_exp::<F>(&neg_cfc1_aoo, &mut ief, n);
        } else if alpha_cnst < F::new(2.5) {
            ief[0] = F::cast_from(IE_PARAMS_C[0]);
            #[unroll]
            for i in 1..size {
                ief[i] = F::new(0.0);
            }
            // alpha^1
            let mut term = Array::<F>::new(size);
            let mut ief_tmp = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&alpha, F::cast_from(IE_PARAMS_C[1]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^2
            let mut a2 = Array::<F>::new(size);
            ctaylor_powi_2::<F>(&alpha, &mut a2, n);
            ctaylor_scalar_mul::<F>(&a2, F::cast_from(IE_PARAMS_C[2]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^3
            let mut a3 = Array::<F>::new(size);
            ctaylor_powi_3::<F>(&alpha, &mut a3, n);
            ctaylor_scalar_mul::<F>(&a3, F::cast_from(IE_PARAMS_C[3]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^4
            let mut a4 = Array::<F>::new(size);
            ctaylor_powi_4::<F>(&alpha, &mut a4, n);
            ctaylor_scalar_mul::<F>(&a4, F::cast_from(IE_PARAMS_C[4]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^5
            let mut a5 = Array::<F>::new(size);
            ctaylor_mul::<F>(&a4, &alpha, &mut a5, n);
            ctaylor_scalar_mul::<F>(&a5, F::cast_from(IE_PARAMS_C[5]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^6
            let mut a6 = Array::<F>::new(size);
            ctaylor_mul::<F>(&a5, &alpha, &mut a6, n);
            ctaylor_scalar_mul::<F>(&a6, F::cast_from(IE_PARAMS_C[6]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
            // alpha^7
            let mut a7 = Array::<F>::new(size);
            ctaylor_mul::<F>(&a6, &alpha, &mut a7, n);
            ctaylor_scalar_mul::<F>(&a7, F::cast_from(IE_PARAMS_C[7]), &mut term, n);
            ctaylor_add::<F>(&ief, &term, &mut ief_tmp, n);
            #[unroll]
            for i in 0..size {
                ief[i] = ief_tmp[i];
            }
        } else {
            let mut inv_oma = Array::<F>::new(size);
            ctaylor_reciprocal::<F>(&oma, &mut inv_oma, n);
            let mut cfc2_oma = Array::<F>::new(size);
            ctaylor_scalar_mul::<F>(&inv_oma, F::cast_from(CFC2_F64), &mut cfc2_oma, n);
            let mut exp_val = Array::<F>::new(size);
            ctaylor_exp::<F>(&cfc2_oma, &mut exp_val, n);
            ctaylor_scalar_mul::<F>(&exp_val, F::cast_from(-CFDC1_F64), &mut ief, n);
        }
    }

    // ec0 = scan_ec0(rs, s, zeta)
    let mut ec0 = Array::<F>::new(size);
    scan_ec0::<F>(&rs, &s, zeta, &mut ec0, n);

    // ec1 = scan_ec1(rs, s, zeta, sqrtrs, idelec)
    let mut ec1 = Array::<F>::new(size);
    scan_ec1::<F>(&rs, &s, zeta, &sqrtrs, &mut ec1, idelec, n);

    // eps_c = (ec1 + ief*(ec0 - ec1)) * n
    let mut ec0_m_ec1 = Array::<F>::new(size);
    ctaylor_sub::<F>(&ec0, &ec1, &mut ec0_m_ec1, n);
    let mut ief_ec0_ec1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&ief, &ec0_m_ec1, &mut ief_ec0_ec1, n);
    let mut eps_c_over_n = Array::<F>::new(size);
    ctaylor_add::<F>(&ec1, &ief_ec0_ec1, &mut eps_c_over_n, n);
    ctaylor_mul::<F>(&eps_c_over_n, n_dens, out, n);
}
