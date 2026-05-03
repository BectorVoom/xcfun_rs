//! XC_BLOCX — BLOC exchange functional. MGGA-05.
//!
//! # Source
//! - `xcfun-master/src/functionals/blocx.cpp:18-53`
//!
//! # Formula (port of `blocx.cpp:49-53`):
//! ```cpp
//! enea = energy_blocx(2*d.a, 4*d.gaa, 2*d.taua);
//! eneb = energy_blocx(2*d.b, 4*d.gbb, 2*d.taub);
//! return (enea + eneb) / 2;
//! ```
//!
//! BLOCX is **independent of BRX** (verified by RESEARCH §"BLOCX") — the body
//! delegates to `shared::blocx::blocx_energy` with no Newton-iteration step.
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::blocx as blocx_shared;

#[cube]
pub fn blocx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Spin-scaling: 2·a, 4·gaa, 2·taua  (and similarly for beta).
    let mut two_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.a, F::cast_from(2.0_f64), &mut two_a, n);
    let mut four_gaa = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.gaa, F::cast_from(4.0_f64), &mut four_gaa, n);
    let mut two_taua = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taua, F::cast_from(2.0_f64), &mut two_taua, n);

    let mut enea = Array::<F>::new(size);
    blocx_shared::blocx_energy::<F>(&two_a, &four_gaa, &two_taua, &mut enea, n);

    let mut two_b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.b, F::cast_from(2.0_f64), &mut two_b, n);
    let mut four_gbb = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.gbb, F::cast_from(4.0_f64), &mut four_gbb, n);
    let mut two_taub = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taub, F::cast_from(2.0_f64), &mut two_taub, n);

    let mut eneb = Array::<F>::new(size);
    blocx_shared::blocx_energy::<F>(&two_b, &four_gbb, &two_taub, &mut eneb, n);

    // out = (enea + eneb) / 2
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&enea, &eneb, &mut sum, n);
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.5_f64), out, n);
}
