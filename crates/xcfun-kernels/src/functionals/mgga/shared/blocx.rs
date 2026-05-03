//! BLOCX (B-LOC eXchange) helper.
//!
//! FULL BODY port of `xcfun-master/src/functionals/blocx.cpp:18-46` (Wave 3,
//! plan 04-03). Replaces Wave-0 SKELETON.
//!
//! BLOCX is **independent of BRX** (despite the misleading "BLOC" name);
//! the C++ port at `blocx.cpp:18-46` uses a TPSS-shaped enhancement structure
//! with a `tauw / d_tau` ratio and a `z^f` polynomial. No Newton, no `BR(...)`
//! call — just `pow`, `sqrt`, `log`, `exp` over scalar/CTaylor expressions.
//!
//! # Source
//! - `xcfun-master/src/functionals/blocx.cpp:18-46` — `energy_blocx` body.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use super::constants::{BLOCX_B_F64, BLOCX_C_F64, BLOCX_E_F64, BLOCX_KAPPA_F64, BLOCX_MU_F64};

// `(3·π²)^(2/3)` — appears in p0 and tau_unif. Computed once at module load.
// 3·π² = 29.608813...; (3·π²)^(2/3) = 9.570780...
const THREE_PI2_TWO_THIRDS_F64: f64 = 9.570_780_000_627_304_f64;

// `1 / (4 · (3π²)^(2/3))` — p0 prefactor.
const P0_PREFACTOR_F64: f64 = 1.0_f64 / (4.0_f64 * THREE_PI2_TWO_THIRDS_F64);

// `0.3 · (3π²)^(2/3)` — tau_unif prefactor.
const TAU_UNIF_PREFACTOR_F64: f64 = 0.3_f64 * THREE_PI2_TWO_THIRDS_F64;

// `sqrt(BLOCX_E_F64) = sqrt(1.537)`.
const BLOCX_SQRT_E_F64: f64 = 1.239_758_040_909_596_f64;

// `-0.75 · (3/π)^(1/3)` — lda_x coefficient.
const NEG_LDA_X_COEFF_F64: f64 = -0.738_558_766_382_022_3_f64;

// `2 · sqrt(e) · 0.6 · 0.6 · 10 / 81` — coefficient of `z2` in tmp5.
// 2 · 1.239758040909596 · 0.36 · 10 / 81 = 0.110200714747519...
const TMP5_Z2_COEFF_F64: f64 = 2.0_f64 * BLOCX_SQRT_E_F64 * 0.36_f64 * 10.0_f64 / 81.0_f64;

// `BLOCX_E_F64 · BLOCX_MU_F64` — coefficient of `p³` in tmp5.
const TMP5_P3_COEFF_F64: f64 = BLOCX_E_F64 * BLOCX_MU_F64;

// `0.5 · 0.6 · 0.6` — coefficient of `(8·d_n·d_tau)^(-2)` in tmp3 sqrt.
const TMP3_INNER_FIRST_F64: f64 = 0.5_f64 * 0.6_f64 * 0.6_f64;

// `(10 / 81)^2 / kappa` — coefficient of `p²` in tmp4.
const TMP4_P2_COEFF_F64: f64 =
    (10.0_f64 / 81.0_f64) * (10.0_f64 / 81.0_f64) / BLOCX_KAPPA_F64;

/// BLOCX exchange energy density `energy_blocx(d_n, d_gnn, d_tau)`.
///
/// Port target: `xcfun-master/src/functionals/blocx.cpp:18-46`. The body
/// follows the TPSS exchange structure with one notable difference: the
/// `c · z² / (1+z²)²` term in TPSS becomes `c · z^f / (1+z²)²` here, where
/// `f = 4 - 3.3·z`.
///
/// Used by the single MGGA-05 functional `XC_BLOCX`.
#[cube]
pub fn blocx_energy<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // -----------------------------------------------------------------
    //  p0 = 1 / (4 · (3π²)^(2/3) · d_n^(8/3))
    //     = P0_PREFACTOR_F64 · d_n^(-8/3)
    // -----------------------------------------------------------------
    let mut p0 = Array::<F>::new(size);
    {
        let mut dn_m83 = Array::<F>::new(size);
        ctaylor_pow::<F>(d_n, F::cast_from(-8.0_f64 / 3.0_f64), &mut dn_m83, n);
        ctaylor_scalar_mul::<F>(&dn_m83, F::cast_from(P0_PREFACTOR_F64), &mut p0, n);
    }

    // p = d_gnn · p0
    let mut p = Array::<F>::new(size);
    ctaylor_mul::<F>(d_gnn, &p0, &mut p, n);

    // -----------------------------------------------------------------
    //  tauw = d_gnn / (8 · d_n)
    // -----------------------------------------------------------------
    let mut tauw = Array::<F>::new(size);
    {
        // inv_dn = 1 / d_n
        let mut inv_dn = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(d_n, &mut inv_dn, n);
        // tmp = d_gnn · inv_dn
        let mut tmp = Array::<F>::new(size);
        ctaylor_mul::<F>(d_gnn, &inv_dn, &mut tmp, n);
        // tauw = (1/8) · tmp
        ctaylor_scalar_mul::<F>(&tmp, F::cast_from(1.0_f64 / 8.0_f64), &mut tauw, n);
    }

    // -----------------------------------------------------------------
    //  z = tauw / d_tau,   z2 = z²
    // -----------------------------------------------------------------
    let mut z = Array::<F>::new(size);
    {
        let mut inv_tau = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(d_tau, &mut inv_tau, n);
        ctaylor_mul::<F>(&tauw, &inv_tau, &mut z, n);
    }
    let mut z2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&z, &z, &mut z2, n);

    // -----------------------------------------------------------------
    //  tau_unif = TAU_UNIF_PREFACTOR · d_n^(5/3)
    // -----------------------------------------------------------------
    let mut tau_unif = Array::<F>::new(size);
    {
        let mut dn_53 = Array::<F>::new(size);
        ctaylor_pow::<F>(d_n, F::cast_from(5.0_f64 / 3.0_f64), &mut dn_53, n);
        ctaylor_scalar_mul::<F>(
            &dn_53,
            F::cast_from(TAU_UNIF_PREFACTOR_F64),
            &mut tau_unif,
            n,
        );
    }

    // -----------------------------------------------------------------
    //  alpha = (d_tau - tauw) / tau_unif
    // -----------------------------------------------------------------
    let mut alpha = Array::<F>::new(size);
    {
        let mut tau_minus_tauw = Array::<F>::new(size);
        ctaylor_sub::<F>(d_tau, &tauw, &mut tau_minus_tauw, n);
        let mut inv_tau_unif = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&tau_unif, &mut inv_tau_unif, n);
        ctaylor_mul::<F>(&tau_minus_tauw, &inv_tau_unif, &mut alpha, n);
    }

    // -----------------------------------------------------------------
    //  q_b = (9/20) · (α-1) / sqrt(1 + b·α·(α-1)) + 2·p/3
    // -----------------------------------------------------------------
    let mut q_b = Array::<F>::new(size);
    {
        // alpha_m1 = alpha - 1
        let mut alpha_m1 = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            alpha_m1[i] = alpha[i];
        }
        alpha_m1[0] = alpha_m1[0] - F::new(1.0);

        // alpha_alpha_m1 = alpha · (alpha-1)
        let mut alpha_alpha_m1 = Array::<F>::new(size);
        ctaylor_mul::<F>(&alpha, &alpha_m1, &mut alpha_alpha_m1, n);

        // b_term = b · alpha · (alpha-1)
        let mut b_term = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &alpha_alpha_m1,
            F::cast_from(BLOCX_B_F64),
            &mut b_term,
            n,
        );

        // sqrt_arg = 1 + b_term
        let mut sqrt_arg = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            sqrt_arg[i] = b_term[i];
        }
        sqrt_arg[0] = sqrt_arg[0] + F::new(1.0);

        // sqrt_val = sqrt(1 + b·α·(α-1))
        let mut sqrt_val = Array::<F>::new(size);
        ctaylor_sqrt::<F>(&sqrt_arg, &mut sqrt_val, n);

        // inv_sqrt = 1 / sqrt_val
        let mut inv_sqrt = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&sqrt_val, &mut inv_sqrt, n);

        // ratio = (alpha-1) · inv_sqrt
        let mut ratio = Array::<F>::new(size);
        ctaylor_mul::<F>(&alpha_m1, &inv_sqrt, &mut ratio, n);

        // first_term = (9/20) · ratio
        let mut first_term = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &ratio,
            F::cast_from(9.0_f64 / 20.0_f64),
            &mut first_term,
            n,
        );

        // second_term = (2/3) · p
        let mut second_term = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p, F::cast_from(2.0_f64 / 3.0_f64), &mut second_term, n);

        // q_b = first_term + second_term
        ctaylor_add::<F>(&first_term, &second_term, &mut q_b, n);
    }

    // -----------------------------------------------------------------
    //  ff = 4 - 3.3·z       (so ff is a CTaylor)
    //  zf = exp(log(z) · ff)
    // -----------------------------------------------------------------
    let mut ff = Array::<F>::new(size);
    {
        let mut neg33z = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&z, F::cast_from(-3.3_f64), &mut neg33z, n);
        #[unroll]
        for i in 0..size {
            ff[i] = neg33z[i];
        }
        ff[0] = ff[0] + F::new(4.0);
    }

    let mut zf = Array::<F>::new(size);
    {
        let mut log_z = Array::<F>::new(size);
        ctaylor_log::<F>(&z, &mut log_z, n);
        let mut log_z_ff = Array::<F>::new(size);
        ctaylor_mul::<F>(&log_z, &ff, &mut log_z_ff, n);
        ctaylor_exp::<F>(&log_z_ff, &mut zf, n);
    }

    // -----------------------------------------------------------------
    //  tmp1 = p · (10/81 + c · zf / (1+z²)²)
    // -----------------------------------------------------------------
    let mut tmp1 = Array::<F>::new(size);
    {
        // one_plus_z2 = 1 + z²
        let mut one_plus_z2 = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            one_plus_z2[i] = z2[i];
        }
        one_plus_z2[0] = one_plus_z2[0] + F::new(1.0);

        // one_plus_z2_sq = (1+z²)²
        let mut one_plus_z2_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&one_plus_z2, &one_plus_z2, &mut one_plus_z2_sq, n);

        // inv_denom = 1 / (1+z²)²
        let mut inv_denom = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&one_plus_z2_sq, &mut inv_denom, n);

        // c_zf = c · zf
        let mut c_zf = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&zf, F::cast_from(BLOCX_C_F64), &mut c_zf, n);

        // c_zf_over_denom = c·zf / (1+z²)²
        let mut c_zf_over_denom = Array::<F>::new(size);
        ctaylor_mul::<F>(&c_zf, &inv_denom, &mut c_zf_over_denom, n);

        // bracket = 10/81 + c_zf_over_denom (CNST-bump on copy)
        let mut bracket = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            bracket[i] = c_zf_over_denom[i];
        }
        bracket[0] = bracket[0] + F::cast_from(10.0_f64 / 81.0_f64);

        // tmp1 = p · bracket
        ctaylor_mul::<F>(&p, &bracket, &mut tmp1, n);
    }

    // -----------------------------------------------------------------
    //  tmp2 = 146 · q_b² / 2025
    // -----------------------------------------------------------------
    let mut tmp2 = Array::<F>::new(size);
    {
        let mut qb_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&q_b, &q_b, &mut qb_sq, n);
        ctaylor_scalar_mul::<F>(
            &qb_sq,
            F::cast_from(146.0_f64 / 2025.0_f64),
            &mut tmp2,
            n,
        );
    }

    // -----------------------------------------------------------------
    //  tmp3 = -73/405 · q_b · d_gnn · sqrt((0.5·0.36)·(8·d_n·d_tau)^(-2) + 0.5·p0²)
    // -----------------------------------------------------------------
    let mut tmp3 = Array::<F>::new(size);
    {
        // d_n_d_tau = d_n · d_tau
        let mut d_n_d_tau = Array::<F>::new(size);
        ctaylor_mul::<F>(d_n, d_tau, &mut d_n_d_tau, n);

        // 8·d_n·d_tau
        let mut eight_dn_dtau = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&d_n_d_tau, F::cast_from(8.0_f64), &mut eight_dn_dtau, n);

        // (8·d_n·d_tau)^(-2) = ctaylor_pow(eight_dn_dtau, -2.0)
        let mut inv_eight_dn_dtau_sq = Array::<F>::new(size);
        ctaylor_pow::<F>(
            &eight_dn_dtau,
            F::cast_from(-2.0_f64),
            &mut inv_eight_dn_dtau_sq,
            n,
        );

        // first = 0.18 · (8·d_n·d_tau)^(-2)
        let mut first = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &inv_eight_dn_dtau_sq,
            F::cast_from(TMP3_INNER_FIRST_F64),
            &mut first,
            n,
        );

        // p0_sq = p0²
        let mut p0_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&p0, &p0, &mut p0_sq, n);

        // second = 0.5 · p0²
        let mut second = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p0_sq, F::cast_from(0.5_f64), &mut second, n);

        // sqrt_arg = first + second
        let mut sqrt_arg = Array::<F>::new(size);
        ctaylor_add::<F>(&first, &second, &mut sqrt_arg, n);

        // sqrt_val
        let mut sqrt_val = Array::<F>::new(size);
        ctaylor_sqrt::<F>(&sqrt_arg, &mut sqrt_val, n);

        // qb_dgnn = q_b · d_gnn
        let mut qb_dgnn = Array::<F>::new(size);
        ctaylor_mul::<F>(&q_b, d_gnn, &mut qb_dgnn, n);

        // qb_dgnn_sqrt = qb_dgnn · sqrt_val
        let mut qb_dgnn_sqrt = Array::<F>::new(size);
        ctaylor_mul::<F>(&qb_dgnn, &sqrt_val, &mut qb_dgnn_sqrt, n);

        // tmp3 = -73/405 · qb_dgnn_sqrt
        ctaylor_scalar_mul::<F>(
            &qb_dgnn_sqrt,
            F::cast_from(-73.0_f64 / 405.0_f64),
            &mut tmp3,
            n,
        );
    }

    // -----------------------------------------------------------------
    //  tmp4 = (p · 10/81)² / kappa = (10/81)² / kappa · p²
    // -----------------------------------------------------------------
    let mut tmp4 = Array::<F>::new(size);
    {
        let mut p_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&p, &p, &mut p_sq, n);
        ctaylor_scalar_mul::<F>(&p_sq, F::cast_from(TMP4_P2_COEFF_F64), &mut tmp4, n);
    }

    // -----------------------------------------------------------------
    //  tmp5 = 2·sqrt(e)·0.36·z2·10/81 + e·mu·p³
    // -----------------------------------------------------------------
    let mut tmp5 = Array::<F>::new(size);
    {
        // first = TMP5_Z2_COEFF · z2
        let mut first = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&z2, F::cast_from(TMP5_Z2_COEFF_F64), &mut first, n);

        // p_sq = p · p
        let mut p_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&p, &p, &mut p_sq, n);

        // p_cu = p · p²
        let mut p_cu = Array::<F>::new(size);
        ctaylor_mul::<F>(&p, &p_sq, &mut p_cu, n);

        // second = TMP5_P3_COEFF · p³
        let mut second = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p_cu, F::cast_from(TMP5_P3_COEFF_F64), &mut second, n);

        ctaylor_add::<F>(&first, &second, &mut tmp5, n);
    }

    // -----------------------------------------------------------------
    //  tmp6 = tmp1 + tmp2 + tmp3 + tmp4 + tmp5  (left-to-right per ACC-06)
    // -----------------------------------------------------------------
    let mut tmp6 = Array::<F>::new(size);
    {
        let mut s12 = Array::<F>::new(size);
        ctaylor_add::<F>(&tmp1, &tmp2, &mut s12, n);
        let mut s123 = Array::<F>::new(size);
        ctaylor_add::<F>(&s12, &tmp3, &mut s123, n);
        let mut s1234 = Array::<F>::new(size);
        ctaylor_add::<F>(&s123, &tmp4, &mut s1234, n);
        ctaylor_add::<F>(&s1234, &tmp5, &mut tmp6, n);
    }

    // -----------------------------------------------------------------
    //  x = tmp6 / (1 + sqrt(e) · p)²
    // -----------------------------------------------------------------
    let mut x = Array::<F>::new(size);
    {
        // sqrt_e_p = sqrt(e) · p
        let mut sqrt_e_p = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(&p, F::cast_from(BLOCX_SQRT_E_F64), &mut sqrt_e_p, n);

        // one_plus = 1 + sqrt(e)·p
        let mut one_plus = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            one_plus[i] = sqrt_e_p[i];
        }
        one_plus[0] = one_plus[0] + F::new(1.0);

        // denom_sq = (1 + sqrt(e)·p)²
        let mut denom_sq = Array::<F>::new(size);
        ctaylor_mul::<F>(&one_plus, &one_plus, &mut denom_sq, n);

        // inv_denom = 1 / denom_sq
        let mut inv_denom = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom_sq, &mut inv_denom, n);

        ctaylor_mul::<F>(&tmp6, &inv_denom, &mut x, n);
    }

    // -----------------------------------------------------------------
    //  Fx = 1 + kappa - kappa / (1 + x/kappa)
    // -----------------------------------------------------------------
    let mut fx = Array::<F>::new(size);
    {
        // x_over_kappa = (1/kappa) · x
        let mut x_over_kappa = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &x,
            F::cast_from(1.0_f64 / BLOCX_KAPPA_F64),
            &mut x_over_kappa,
            n,
        );

        // denom = 1 + x/kappa
        let mut denom = Array::<F>::new(size);
        #[unroll]
        for i in 0..size {
            denom[i] = x_over_kappa[i];
        }
        denom[0] = denom[0] + F::new(1.0);

        // inv_denom = 1/(1+x/kappa)
        let mut inv_denom = Array::<F>::new(size);
        ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

        // kappa_over = kappa · inv_denom
        let mut kappa_over = Array::<F>::new(size);
        ctaylor_scalar_mul::<F>(
            &inv_denom,
            F::cast_from(BLOCX_KAPPA_F64),
            &mut kappa_over,
            n,
        );

        // fx = -kappa_over with CNST + (1+kappa)
        let neg_one = F::new(0.0) - F::new(1.0);
        ctaylor_scalar_mul::<F>(&kappa_over, neg_one, &mut fx, n);
        fx[0] = fx[0] + F::new(1.0) + F::cast_from(BLOCX_KAPPA_F64);
    }

    // -----------------------------------------------------------------
    //  lda = NEG_LDA_X_COEFF · d_n^(4/3)
    // -----------------------------------------------------------------
    let mut lda = Array::<F>::new(size);
    {
        let mut dn_43 = Array::<F>::new(size);
        ctaylor_pow::<F>(d_n, F::cast_from(4.0_f64 / 3.0_f64), &mut dn_43, n);
        ctaylor_scalar_mul::<F>(&dn_43, F::cast_from(NEG_LDA_X_COEFF_F64), &mut lda, n);
    }

    // -----------------------------------------------------------------
    //  return lda · Fx
    // -----------------------------------------------------------------
    ctaylor_mul::<F>(&lda, &fx, out, n);
}
