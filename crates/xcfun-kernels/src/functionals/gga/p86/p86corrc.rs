//! XC_P86CORRC — P86 GGA correlation, gradient correction only. **GGA-07.**
//!
//! # Source
//! - `xcfun-master/src/functionals/p86c.cpp:50-52, 112-119`
//!
//! # Formula
//! ```cpp
//! p86c_corr(d) = exp(-Pg(d)) · Cg(r_s) · gnn / (n^(4/3) · dz(d))
//! ```
//!
//! Same gradient-correction term as P86C minus the `n · pz81eps(d)` LSDA part.
//! Sub-helpers (`Cg`, `Pg`, `dz`) are inlined here as private functions —
//! identical to the P86C versions but kept private to `p86corrc.rs` so changes
//! to either can be made independently without coupling.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use crate::density_vars::DensVarsDev;

const P86_CX: f64 = 0.001_667_f64;
const P86_BG: f64 = 7.389e-6_f64;
const P86_FG: f64 = 0.11_f64;
const P86_CINF: f64 = 0.004_235_f64;
/// `(9π)^(1/6)` — must match `gga::p86::p86c::P86_PI_EXPR`. Locked by
/// `tests::p86corrc_pi_expr_locked` (06-N7/07-00). Previous value
/// `1.745_050_359_752_853_5` was the same wrong literal as p86c.rs's;
/// fixing only one and not the other would silently leave half the
/// failures in place (P86CORRC's 496,355 fails vs P86C's 21).
const P86_PI_EXPR: f64 = 1.745_415_106_125_124_f64;
const P86_DBL_EPS: f64 = 2.220_446_049_250_313e-16_f64;
const P86_CBRT2: f64 = 1.259_921_049_894_873_2_f64;

#[cube]
fn cg<F: Float>(r: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut bg_r = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(r, F::cast_from(P86_BG), &mut bg_r, n);
    let mut inner_n = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_n[i] = bg_r[i];
    }
    inner_n[0] = inner_n[0] + F::cast_from(0.023_266_f64);
    let mut r_inner_n = Array::<F>::new(size);
    ctaylor_mul::<F>(r, &inner_n, &mut r_inner_n, n);
    let mut num = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        num[i] = r_inner_n[i];
    }
    num[0] = num[0] + F::cast_from(0.002_568_f64);

    let mut bg10k_r = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(r, F::cast_from(10_000.0_f64 * P86_BG), &mut bg10k_r, n);
    let mut inner_d1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_d1[i] = bg10k_r[i];
    }
    inner_d1[0] = inner_d1[0] + F::cast_from(0.472_f64);
    let mut r_id1 = Array::<F>::new(size);
    ctaylor_mul::<F>(r, &inner_d1, &mut r_id1, n);
    let mut inner_d2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        inner_d2[i] = r_id1[i];
    }
    inner_d2[0] = inner_d2[0] + F::cast_from(8.723_f64);
    let mut r_id2 = Array::<F>::new(size);
    ctaylor_mul::<F>(r, &inner_d2, &mut r_id2, n);
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = r_id2[i];
    }
    den[0] = den[0] + F::new(1.0);

    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num, &inv_den, &mut frac, n);
    #[unroll]
    for i in 0..size {
        out[i] = frac[i];
    }
    out[0] = out[0] + F::cast_from(P86_CX);
}

#[cube]
fn pg<F: Float>(d: &DensVarsDev<F>, cg_rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut gnn_eps = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        gnn_eps[i] = d.gnn[i];
    }
    gnn_eps[0] = gnn_eps[0] + F::cast_from(P86_DBL_EPS);
    let mut sgnn = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&gnn_eps, &mut sgnn, n);
    let mut n_76 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 6.0_f64), &mut n_76, n);
    let mut cg_n76 = Array::<F>::new(size);
    ctaylor_mul::<F>(cg_rs, &n_76, &mut cg_n76, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&cg_n76, &mut inv_den, n);
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&sgnn, &inv_den, &mut frac, n);
    const PG_PREFAC: f64 = P86_PI_EXPR * P86_FG * P86_CINF;
    ctaylor_scalar_mul::<F>(&frac, F::cast_from(PG_PREFAC), out, n);
}

#[cube]
fn dz<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut a_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.a, F::cast_from(5.0_f64 / 3.0_f64), &mut a_53, n);
    let mut b_53 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.b, F::cast_from(5.0_f64 / 3.0_f64), &mut b_53, n);
    let mut sum_53 = Array::<F>::new(size);
    ctaylor_add::<F>(&a_53, &b_53, &mut sum_53, n);
    let mut s53 = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&sum_53, &mut s53, n);
    let mut n_n56 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-5.0_f64 / 6.0_f64), &mut n_n56, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&s53, &n_n56, &mut prod, n);
    ctaylor_scalar_mul::<F>(&prod, F::cast_from(P86_CBRT2), out, n);
}

/// XC_P86CORRC kernel. 1:1 port of `p86c.cpp:50-52`.
#[cube]
pub fn p86corrc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut cg_rs = Array::<F>::new(size);
    cg::<F>(&d.r_s, &mut cg_rs, n);

    let mut pg_d = Array::<F>::new(size);
    pg::<F>(d, &cg_rs, &mut pg_d, n);

    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_pg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&pg_d, neg_one, &mut neg_pg, n);
    let mut e_neg_pg = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_pg, &mut e_neg_pg, n);

    let mut n_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(4.0_f64 / 3.0_f64), &mut n_43, n);

    let mut dz_d = Array::<F>::new(size);
    dz::<F>(d, &mut dz_d, n);

    let mut n43_dz = Array::<F>::new(size);
    ctaylor_mul::<F>(&n_43, &dz_d, &mut n43_dz, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n43_dz, &mut inv_den, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_den, &mut g_over_d, n);
    let mut cg_g = Array::<F>::new(size);
    ctaylor_mul::<F>(&cg_rs, &g_over_d, &mut cg_g, n);
    ctaylor_mul::<F>(&e_neg_pg, &cg_g, out, n);
}

#[cfg(test)]
mod tests {
    /// Regression lock for the P86CORRC copy of `(9π)^(1/6)`. Must
    /// match the value locked in `gga::p86::p86c::tests::p86_pi_expr_locked`.
    /// Run 25534837958 demonstrated that fixing only the p86c.rs copy
    /// reduced P86C's failures from 496,353 → 21 while leaving P86CORRC
    /// unchanged at 496,355 — both copies need to move together.
    #[test]
    fn p86corrc_pi_expr_locked() {
        let truth: f64 = 1.745_415_106_125_124_f64;
        assert_eq!(super::P86_PI_EXPR, truth);
    }
}
