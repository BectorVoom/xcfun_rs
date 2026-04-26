//! CSC (Colle-Salvetti correlation) helper.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A. Single-body
//! inline helper for the CSC functional carryover from Phase 3 (D-01-A).
//!
//! # Source
//! - `xcfun-master/src/functionals/cs.cpp:17-27` — `csc(d)` energy body.
//!
//! # Reads (DensVarsDev `id=17` slots)
//! - `d.a`, `d.b`, `d.n`, `d.taua`, `d.taub`, `d.gnn`, `d.jpaa`, `d.jpbb`,
//!   `d.n_m13` (derived n^(-1/3)).
//!
//! # Formula (port of `cs.cpp:17-27`)
//!
//! ```cpp
//! template <typename num> static num csc(const densvars<num> & d) {
//!   parameter a = 1.0;
//!   parameter b = 1.0;
//!   parameter c = 1.0;
//!   parameter dpar = 1.0;
//!   num gamma = 2 * (1 - (d.a*d.a + d.b*d.b) / (d.n*d.n));
//!   num curv = d.a*d.taua + d.b*d.taub - (1.0/8.0)*d.gnn - (d.jpaa + d.jpbb);
//!   return -a * gamma *
//!          (d.n + 2*b*pow(d.n, -5.0/3.0) * curv * exp(-c*d.n_m13)) /
//!          (1 + dpar*d.n_m13);
//! }
//! ```
//!
//! # Wave 0 status
//!
//! Skeleton signature; FULL body lands in plan 04-01 Wave 1 (CSC ships
//! alongside the BR family per Phase-3 D-01-A carryover scope).

use cubecl::prelude::*;

/// CSC correlation energy density.
///
/// **WAVE-1 SKELETON** — full body lands in plan 04-01.
///
/// Reads `DensVarsDev<F>` fields directly. Wave 1 will refactor this into
/// the canonical `(d: &DensVarsDev<F>, out: &mut Array<F>, n)` signature
/// once `DensVarsDev` access patterns from CSC's id=17 Vars stabilise.
#[cube]
pub fn csc_energy<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    n_density: &Array<F>,
    taua: &Array<F>,
    taub: &Array<F>,
    gnn: &Array<F>,
    jpaa: &Array<F>,
    jpbb: &Array<F>,
    n_m13: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = b;
    let _ = n_density;
    let _ = taua;
    let _ = taub;
    let _ = gnn;
    let _ = jpaa;
    let _ = jpbb;
    let _ = n_m13;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}
