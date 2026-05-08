//! XC_B97_1C — B97-1 GGA correlation. **GGA-09 (B97_1C, id=63).**
//!
//! # Source
//! - `xcfun-master/src/functionals/b97-1xc.cpp:25-34`   (b97_1c_en aggregator)
//! - `xcfun-master/src/functionals/b97c.hpp` + `b97xc.hpp` (shared helpers)
//!
//! Identical structure as B97C — only coefficient tables differ:
//!   `B97_1C_PAR_COEF     = [0.0820011, 2.71681, -2.87103]`
//!   `B97_1C_ANTIPAR_COEF = [0.955689, 0.788552, -5.47869]`

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::b97_poly;
use crate::functionals::gga::shared::constants::{
    B97_1C_ANTIPAR_COEF, B97_1C_PAR_COEF, B97_GAMMA_C_ANTIPAR_F64, B97_GAMMA_C_PAR_F64,
};
use crate::functionals::lda::pw92eps;

#[cube]
fn s2_ab2<F: Float>(gaa: &Array<F>, a_43: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut inv_a43 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(a_43, &mut inv_a43, n);
    let mut first_div = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_a43, &mut first_div, n);
    ctaylor_mul::<F>(&first_div, &inv_a43, out, n);
}

#[cube]
fn energy_b97c_par<F: Float>(
    rho: &Array<F>,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    e_lsda: &mut Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps_pol = Array::<F>::new(size);
    pw92eps::pw92eps_polarized::<F>(rho, &mut eps_pol, n);
    ctaylor_mul::<F>(&eps_pol, rho, e_lsda, n);

    let mut s2 = Array::<F>::new(size);
    s2_ab2::<F>(grad2, rho_43, &mut s2, n);

    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(F::cast_from(B97_GAMMA_C_PAR_F64), &s2, &mut u, n);

    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(
        F::cast_from(B97_1C_PAR_COEF[0]),
        F::cast_from(B97_1C_PAR_COEF[1]),
        F::cast_from(B97_1C_PAR_COEF[2]),
        &u,
        &mut enh,
        n,
    );

    ctaylor_mul::<F>(e_lsda, &enh, out, n);
}

#[cube]
fn energy_b97c_antipar<F: Float>(
    d: &DensVarsDev<F>,
    e_lsda_a: &Array<F>,
    e_lsda_b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut eps_total = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps_total, n);
    let mut eps_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps_total, &d.n, &mut eps_n, n);
    let mut after_a = Array::<F>::new(size);
    ctaylor_sub::<F>(&eps_n, e_lsda_a, &mut after_a, n);
    let mut e_lsda = Array::<F>::new(size);
    ctaylor_sub::<F>(&after_a, e_lsda_b, &mut e_lsda, n);

    let mut s2_a = Array::<F>::new(size);
    s2_ab2::<F>(&d.gaa, &d.a_43, &mut s2_a, n);
    let mut s2_b = Array::<F>::new(size);
    s2_ab2::<F>(&d.gbb, &d.b_43, &mut s2_b, n);
    let mut s2_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&s2_a, &s2_b, &mut s2_sum, n);
    let mut s2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2_sum, F::new(0.5), &mut s2, n);

    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(F::cast_from(B97_GAMMA_C_ANTIPAR_F64), &s2, &mut u, n);

    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(
        F::cast_from(B97_1C_ANTIPAR_COEF[0]),
        F::cast_from(B97_1C_ANTIPAR_COEF[1]),
        F::cast_from(B97_1C_ANTIPAR_COEF[2]),
        &u,
        &mut enh,
        n,
    );

    ctaylor_mul::<F>(&e_lsda, &enh, out, n);
}

/// XC_B97_1C kernel. 1:1 port of `b97-1xc.cpp:25-34`.
#[cube]
pub fn b97_1c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut e_lsda_a = Array::<F>::new(size);
    let mut e_par_a = Array::<F>::new(size);
    energy_b97c_par::<F>(&d.a, &d.a_43, &d.gaa, &mut e_lsda_a, &mut e_par_a, n);

    let mut e_lsda_b = Array::<F>::new(size);
    let mut e_par_b = Array::<F>::new(size);
    energy_b97c_par::<F>(&d.b, &d.b_43, &d.gbb, &mut e_lsda_b, &mut e_par_b, n);

    let mut tmp = Array::<F>::new(size);
    ctaylor_add::<F>(&e_par_a, &e_par_b, &mut tmp, n);

    let mut e_anti = Array::<F>::new(size);
    energy_b97c_antipar::<F>(d, &e_lsda_a, &e_lsda_b, &mut e_anti, n);

    ctaylor_add::<F>(&tmp, &e_anti, out, n);
}
