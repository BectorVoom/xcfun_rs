//! XC_PBEC — PBE correlation functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbec.cpp:40-47`
//!
//! # Formula
//! ```cpp
//! eps = pw92eps::pw92eps(d);
//! u   = phi(d);  // reorganised: 2^(-1/3) · n_m13² · (sqrt(a_43)+sqrt(b_43))
//! d2  = (1/12 · 3^(5/6) / π^(-1/6))² · gnn / (u² · n^(7/3));
//! return n · (eps + H(d2, eps, u³));
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_powi_3, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::PBEC_D2_PREFACTOR_F64;
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;

#[cube]
pub fn pbec_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92_eps(d).
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // u = phi_reorganised(n_m13, a_43, b_43).
    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);

    // u² and u³.
    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    // n^(7/3).
    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);

    // u² · n^(7/3).
    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom, n);

    // 1 / denom.
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);

    // gnn · inv_denom.
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom, &mut g_over_d, n);

    // d2 = PREFACTOR · g_over_d.
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // h = H(d2, eps, u³).
    let mut h = Array::<F>::new(size);
    pbec_eps::h_gga::<F>(&d2, &eps, &u3, &mut h, n);

    // sum = eps + h.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &h, &mut sum, n);

    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum, out, n);
}
