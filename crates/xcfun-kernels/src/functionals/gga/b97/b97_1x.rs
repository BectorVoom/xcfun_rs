//! XC_B97_1X — B97-1 GGA exchange. **GGA-09 (B97_1X, id=62).**
//!
//! # Source
//! - `xcfun-master/src/functionals/b97-1xc.cpp:20-23`   (b97_1x_en aggregator)
//! - `xcfun-master/src/functionals/b97x.hpp` + `b97xc.hpp` (shared helpers)
//!
//! Identical algebraic structure as B97X — only the coefficient table differs:
//! `B97_1X_COEF = [0.789518, 0.573805, 0.660975]` (vs B97X's `[0.8094, ...]`).
//! Per-functional body kept distinct (not parameterised) for audit-friendliness.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::b97_poly;
use crate::functionals::gga::shared::constants::{B97_1X_COEF, B97_GAMMA_X_F64};

/// `s2_ab(gaa, a_43) = (gaa / a_43) / a_43` — C++ left-associative div chain.
#[cube]
fn s2_ab<F: Float>(gaa: &Array<F>, a_43: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut inv_a43 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(a_43, &mut inv_a43, n);
    let mut first_div = Array::<F>::new(size);
    ctaylor_mul::<F>(gaa, &inv_a43, &mut first_div, n);
    ctaylor_mul::<F>(&first_div, &inv_a43, out, n);
}

/// Per-spin B97-1 exchange energy.
#[cube]
fn energy_b97x_ab<F: Float>(
    gamma: F,
    c0: F,
    c1: F,
    c2: F,
    rho_43: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    let mut s2 = Array::<F>::new(size);
    s2_ab::<F>(grad2, rho_43, &mut s2, n);

    let mut u = Array::<F>::new(size);
    b97_poly::ux_ab::<F>(gamma, &s2, &mut u, n);

    let mut enh = Array::<F>::new(size);
    b97_poly::b97_enhancement::<F>(c0, c1, c2, &u, &mut enh, n);

    const NEG_PREFACTOR_F64: f64 = -0.930_525_736_349_100_2_f64;
    let mut lsda = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, F::cast_from(NEG_PREFACTOR_F64), &mut lsda, n);

    ctaylor_mul::<F>(&lsda, &enh, out, n);
}

/// XC_B97_1X kernel. 1:1 port of `b97-1xc.cpp:20-23`.
#[cube]
pub fn b97_1x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let gamma = F::cast_from(B97_GAMMA_X_F64);
    let c0 = F::cast_from(B97_1X_COEF[0]);
    let c1 = F::cast_from(B97_1X_COEF[1]);
    let c2 = F::cast_from(B97_1X_COEF[2]);

    let mut e_alpha = Array::<F>::new(size);
    energy_b97x_ab::<F>(gamma, c0, c1, c2, &d.a_43, &d.gaa, &mut e_alpha, n);

    let mut e_beta = Array::<F>::new(size);
    energy_b97x_ab::<F>(gamma, c0, c1, c2, &d.b_43, &d.gbb, &mut e_beta, n);

    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
