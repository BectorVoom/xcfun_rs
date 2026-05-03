//! XC_TPSSC — TPSS correlation functional. MGGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/tpssc.cpp:19-21`
//! - `xcfun-master/src/functionals/tpssc_eps.hpp`
//!
//! # Formula:
//! ```cpp
//! num eps = tpssc_eps::tpssc_eps(d);
//! return d.n * eps;
//! ```
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::tpss_like;

#[cube]
pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Phase 6 D-10 — hard-clamp tau to tau_w to guard the unphysical regime
    // where the von Weizsäcker bound is violated by f64-rounding cancellation.
    // Plan 04-10 Path-B confirmed algorithmically faithful port; the
    // divergence (`tauwtau3 ≈ 1e+27` amplifying ULP differences in
    // `eps_pkzb * (1 + dd * eps_pkzb * tauwtau3)`) is pure f64 cancellation
    // in the `tau ≪ tau_w` regime. ACC-04 mpmath amendment (D-03) covers
    // boundary verification; C++ has no equivalent guard.
    let mut tau_w = Array::<F>::new(size);
    tpss_like::build_tau_w::<F>(d, &mut tau_w, n);
    let mut tau_clamped = Array::<F>::new(size);
    tpss_like::ctaylor_max::<F>(&d.tau, &tau_w, &mut tau_clamped, n);

    let mut eps = Array::<F>::new(size);
    tpss_like::tpss_eps_full_with_tau::<F>(d, &tau_clamped, &mut eps, n);

    // out = d.n * eps
    ctaylor_mul::<F>(&d.n, &eps, out, n);
}
