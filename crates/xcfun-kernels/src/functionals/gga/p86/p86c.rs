//! XC_P86C — P86 GGA correlation. **GGA-07.**
//!
//! # Source
//! - `xcfun-master/src/functionals/p86c.cpp:18-48`
//!
//! # Formula
//! ```cpp
//! Cg(r) = Cx + (0.002568 + r·(0.023266 + Bg·r)) /
//!              (1 + r·(8.723 + r·(0.472 + 10000·Bg·r)))
//! Pg(d) = (9π)^(1/6) · Fg · Cinf · sqrt(DBL_EPS + gnn) /
//!         (Cg(r_s) · n^(7/6))
//! dz(d) = cbrt(2) · sqrt(a^(5/3) + b^(5/3)) · n^(-5/6)
//! p86c(d) = n · pz81eps(d) + exp(-Pg(d)) · Cg(r_s) · gnn / (n^(4/3) · dz(d))
//! ```
//! Constants:
//!   Cx = 0.001667, Bg = 7.389e-6, Fg = 0.11, Cinf = 0.004235.
//!   DBL_EPS = std::numeric_limits<double>::epsilon() = 2.220446049250313e-16
//!   (9π)^(1/6) = 1.74505...
//!
//! P86 cross-tier import (W8): `pz81_eps` must be `pub` in `lda::pz81c`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use crate::density_vars::DensVarsDev;
use crate::functionals::lda::pz81c;

const P86_CX: f64 = 0.001_667_f64;
const P86_BG: f64 = 7.389e-6_f64;
const P86_FG: f64 = 0.11_f64;
const P86_CINF: f64 = 0.004_235_f64;
/// `(9π)^(1/6)` precomputed in f64. 9·π = 28.27433..., ^(1/6) = 1.7450503...
const P86_PI_EXPR: f64 = 1.745_050_359_752_853_5_f64;
const P86_DBL_EPS: f64 = 2.220_446_049_250_313e-16_f64;
/// `cbrt(2.0) = 1.2599210498948732`.
const P86_CBRT2: f64 = 1.259_921_049_894_873_2_f64;

// Cg(r) = Cx + (0.002568 + r·(0.023266 + Bg·r)) /
//                (1 + r·(8.723 + r·(0.472 + 10000·Bg·r)))
#[cube]
fn cg<F: Float>(r: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // numerator: 0.002568 + r·(0.023266 + Bg·r)
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

    // denominator: 1 + r·(8.723 + r·(0.472 + 10000·Bg·r))
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

// Pg(d) = pi_expr · Fg · Cinf · sqrt(DBL_EPS + gnn) / (Cg(r_s) · n^(7/6))
#[cube]
fn pg<F: Float>(d: &DensVarsDev<F>, cg_rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // gnn + DBL_EPS (CNST-bump).
    let mut gnn_eps = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        gnn_eps[i] = d.gnn[i];
    }
    gnn_eps[0] = gnn_eps[0] + F::cast_from(P86_DBL_EPS);
    // sqrt.
    let mut sgnn = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&gnn_eps, &mut sgnn, n);
    // n^(7/6).
    let mut n_76 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 6.0_f64), &mut n_76, n);
    // Cg · n^(7/6).
    let mut cg_n76 = Array::<F>::new(size);
    ctaylor_mul::<F>(cg_rs, &n_76, &mut cg_n76, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&cg_n76, &mut inv_den, n);
    // sgnn · inv_den.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&sgnn, &inv_den, &mut frac, n);
    // pi_expr · Fg · Cinf · frac.
    const PG_PREFAC: f64 = P86_PI_EXPR * P86_FG * P86_CINF;
    ctaylor_scalar_mul::<F>(&frac, F::cast_from(PG_PREFAC), out, n);
}

// dz(d) = cbrt(2) · sqrt(a^(5/3) + b^(5/3)) · n^(-5/6)
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

// gga_correction(d) = exp(-Pg(d)) · Cg(r_s) · gnn / (n^(4/3) · dz(d))
#[cube]
fn p86_gga_correction<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Cg(r_s).
    let mut cg_rs = Array::<F>::new(size);
    cg::<F>(&d.r_s, &mut cg_rs, n);

    // Pg(d).
    let mut pg_d = Array::<F>::new(size);
    pg::<F>(d, &cg_rs, &mut pg_d, n);

    // exp(-Pg).
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_pg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&pg_d, neg_one, &mut neg_pg, n);
    let mut e_neg_pg = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_pg, &mut e_neg_pg, n);

    // n^(4/3).
    let mut n_43 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(4.0_f64 / 3.0_f64), &mut n_43, n);

    // dz(d).
    let mut dz_d = Array::<F>::new(size);
    dz::<F>(d, &mut dz_d, n);

    // n^(4/3) · dz.
    let mut n43_dz = Array::<F>::new(size);
    ctaylor_mul::<F>(&n_43, &dz_d, &mut n43_dz, n);
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&n43_dz, &mut inv_den, n);
    // gnn / den.
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_den, &mut g_over_d, n);
    // Cg · g_over_d.
    let mut cg_g = Array::<F>::new(size);
    ctaylor_mul::<F>(&cg_rs, &g_over_d, &mut cg_g, n);
    // out = exp(-Pg) · cg_g.
    ctaylor_mul::<F>(&e_neg_pg, &cg_g, out, n);
}

/// XC_P86C kernel. 1:1 port of `p86c.cpp:45-48`.
/// `p86c(d) = n · pz81eps(d) + gga_correction(d)`.
#[cube]
pub fn p86c_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eps_lda = pz81_eps(d).
    let mut eps_lda = Array::<F>::new(size);
    pz81c::pz81_eps::<F>(d, &mut eps_lda, n);

    // term1 = n · eps_lda.
    let mut term1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.n, &eps_lda, &mut term1, n);

    // term2 = gga correction.
    let mut term2 = Array::<F>::new(size);
    p86_gga_correction::<F>(d, &mut term2, n);

    // out = term1 + term2.
    ctaylor_add::<F>(&term1, &term2, out, n);
}

#[cfg(test)]
mod tests {
    /// Regression lock for `P86_PI_EXPR = (9π)^(1/6)`. The previous value
    /// `1.745_050_359_752_853_5_f64` was incorrect by ~2e-4 relative —
    /// the f64-nearest of `(9π)^(1/6)` is `1.745_415_106_125_124`. This
    /// constant feeds the exp(-Pg) term in both XC_P86C and XC_P86CORRC
    /// (they share `pg`) and produced 59% record-level FAIL for both
    /// in Phase 7 Plan 07-00 Task 0.3.
    #[test]
    fn p86_pi_expr_locked() {
        let truth: f64 = 1.745_415_106_125_124_f64;
        assert_eq!(super::P86_PI_EXPR, truth);
    }
}
