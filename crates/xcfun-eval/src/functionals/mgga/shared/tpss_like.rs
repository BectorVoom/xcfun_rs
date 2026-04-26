//! TPSS / revTPSS exchange + correlation enhancement helpers.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A. Exports the helper
//! shapes consumed by Wave 1 family kernels (TPSSX, TPSSC, REVTPSSX, REVTPSSC,
//! TPSSLOCC). Wave 1 (Plan 04-01) ports the FULL bodies.
//!
//! # Sources (all in `xcfun-master/src/functionals/`)
//! - `tpssx_eps.hpp:1-60` — `F_x`, `fx_unif`, `x` (TPSS exchange enhancement)
//! - `tpssc_eps.hpp:1-62` — `tpssc_eps`, `C`, `epsc_summax`, `epsc_revpkzb`
//! - `revtpssx_eps.hpp:1-65` — revTPSS exchange (different mu/c/e constants)
//! - `revtpssc_eps.hpp:1-111` — revTPSS correlation
//!
//! # Scope (Wave 0 substrate)
//!
//! Each `pub fn` below is a **SKELETON**: signature is final, body is a
//! placeholder `unimplemented!()` macro inside a `#[cube]` kernel-friendly
//! pattern. Wave 1 (04-01) replaces the skeleton bodies in-place.
//!
//! Why skeletons? The 32 metaGGA family kernels in Waves 1/2/3 will
//! evolve the helper shapes as fixture-gates surface algorithmic-identity
//! issues; locking the bodies in Wave 0 risks rework. Wave 0's job is to
//! create the module tree and verify it compiles.

// Match upstream C++ naming (`F_x` — capital F to follow the published
// formula notation) — algorithmic-identity rule.
#![allow(non_snake_case)]

use cubecl::prelude::*;

/// Uniform-density exchange energy `(-3/4) · (3/π)^(1/3) · ρ^(4/3)`.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `tpssx_eps.hpp:23-25`:
///
/// ```cpp
/// template <typename num> static num fx_unif(const num & d) {
///   return (-0.75 * pow(3 / PI, 1.0 / 3.0)) * pow(d, 4.0 / 3.0);
/// }
/// ```
#[cube]
pub fn tpss_fx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Wave-1 placeholder: zero out output. Replaced by full ctaylor_pow chain
    // in plan 04-01 Task 1 once the TPSS exchange body lands.
    let size = comptime!((1_u32 << n) as usize);
    let _ = rho;
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// TPSS exchange `x` numerator-of-enhancement composite.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `tpssx_eps.hpp:27-52` — the multi-line composite that
/// computes the `x` quantity feeding into `F_x = 1 + κ - κ/(1 + x/κ)`.
///
/// Inputs:
/// - `d_n`: density (CTaylor)
/// - `d_gnn`: |∇ρ|² (CTaylor)
/// - `d_tau`: kinetic energy density (CTaylor)
#[cube]
pub fn tpss_x<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // Wave-1 placeholder: zero out output.
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// TPSS exchange enhancement factor `F_x = 1 + κ - κ/(1 + x/κ)`.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `tpssx_eps.hpp:54-59`:
///
/// ```cpp
/// template <typename num>
/// static num F_x(const num & d_n, const num & d_gnn, const num & d_tau) {
///   const parameter kappa = 0.804;
///   num xpz = x(d_n, d_gnn, d_tau);
///   return 1 + kappa - kappa / (1 + xpz / kappa);
/// }
/// ```
#[cube]
pub fn tpss_F_x<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// TPSS correlation `tpssc_eps`. Top-level entry point for TPSSC kernel.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `tpssc_eps.hpp:56-61`. Reads multiple `DensVarsDev` fields:
/// `d.a`, `d.b`, `d.n`, `d.s`, `d.gaa`, `d.gbb`, `d.gnn`, `d.gns`, `d.gss`,
/// `d.tau`, `d.taua`, `d.taub`, `d.zeta`. The Wave-1 port refactors this
/// into the canonical `(d: &DensVarsDev<F>, out: &mut Array<F>, n)` shape.
#[cube]
pub fn tpss_eps<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// revTPSS exchange enhancement `F_x` (different κ/μ/c/e constants vs TPSS).
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `revtpssx_eps.hpp:51-56`.
#[cube]
pub fn revtpss_fx<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// revTPSS correlation `revtpssc_eps`. Top-level entry point for REVTPSSC.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Port target: `revtpssc_eps.hpp:105-110`.
#[cube]
pub fn revtpss_eps<F: Float>(
    d_n: &Array<F>,
    d_gnn: &Array<F>,
    d_tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = d_n;
    let _ = d_gnn;
    let _ = d_tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}
