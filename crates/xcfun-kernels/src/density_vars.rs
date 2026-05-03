//! Device-side densvars container — `#[derive(CubeType, CubeLaunch)]` struct
//! holding 24 named `Array<F>` fields (22 from Phase 2 + `lapn` + `laps`
//! added in Phase 3 plan 03-01 per B2 resolution). 1:1 port of the field
//! set in `xcfun-master/src/densvars.hpp:223-244` per CORE-05 + D-02.
//!
//! Each `Array<F>` is a length-`(1 << N)` CTaylor coefficient array
//! (bit-flag-indexed per `xcfun-ad::index::{CNST, VAR0..VAR7}`). Plan 02-04/02-05
//! `#[cube] fn <name>_kernel<F, const N: u32>` bodies read these fields directly:
//! `d.a`, `d.b`, `d.gaa`, `d.n`, `d.s`, `d.zeta`, `d.r_s`, `d.n_m13`, `d.a_43`, `d.b_43`, etc.

use cubecl::prelude::*;

pub mod build;
pub mod regularize;

/// Device-side density-variables container. 24 named CTaylor<F, N> fields
/// (22 from Phase 2 + `lapn` + `laps` added in Phase 3 plan 03-01 per B2),
/// each backed by an `Array<F>` of length `1 << N` (bit-flag-indexed).
///
/// Field order matches `xcfun-master/src/densvars.hpp:223-244` C++ struct
/// declaration order (raw inputs first, then derived fields).
#[derive(CubeType, CubeLaunch)]
pub struct DensVarsDev<F: Float> {
    // Raw inputs (extracted by `build_densvars` per-variant arms)
    /// alpha-spin density (input or derived)
    pub a: Array<F>,
    /// beta-spin density (input or derived)
    pub b: Array<F>,
    /// |∇ρ_α|² (alpha-spin gradient norm squared)
    pub gaa: Array<F>,
    /// ∇ρ_α · ∇ρ_β (cross-spin gradient inner product)
    pub gab: Array<F>,
    /// |∇ρ_β|² (beta-spin gradient norm squared)
    pub gbb: Array<F>,
    /// total density n = a + b
    pub n: Array<F>,
    /// spin density s = a − b
    pub s: Array<F>,
    // Derived gradient combinations
    /// |∇n|² = gaa + 2*gab + gbb
    pub gnn: Array<F>,
    /// ∇n · ∇s = gaa − gbb
    pub gns: Array<F>,
    /// |∇s|² = gaa − 2*gab + gbb
    pub gss: Array<F>,
    // Kinetic energy density terms (Phase 4 use)
    /// total kinetic energy density τ = τ_α + τ_β
    pub tau: Array<F>,
    /// alpha-spin kinetic energy density τ_α
    pub taua: Array<F>,
    /// beta-spin kinetic energy density τ_β
    pub taub: Array<F>,
    // Laplacian terms (Phase 4 use)
    /// alpha-spin Laplacian ∇²ρ_α
    pub lapa: Array<F>,
    /// beta-spin Laplacian ∇²ρ_β
    pub lapb: Array<F>,
    /// total-density Laplacian ∇²n = lapa + lapb (B2 — added by plan 03-01 for
    /// `N_2ND_TAYLOR` / `N_S_2ND_TAYLOR` under Mode::Potential)
    pub lapn: Array<F>,
    /// spin-density Laplacian ∇²s = lapa − lapb (B2 — added by plan 03-01 for
    /// `N_S_2ND_TAYLOR`)
    pub laps: Array<F>,
    // Spin-polarisation + radius
    /// spin polarisation ζ = s / n
    pub zeta: Array<F>,
    /// Wigner-Seitz radius r_s = (3/(4π))^(1/3) · n^(−1/3)
    pub r_s: Array<F>,
    /// n^(−1/3)
    pub n_m13: Array<F>,
    /// a^(4/3)
    pub a_43: Array<F>,
    /// b^(4/3)
    pub b_43: Array<F>,
    // Current-density terms (Phase 4+ use)
    /// j_α paramagnetic current squared
    pub jpaa: Array<F>,
    /// j_β paramagnetic current squared
    pub jpbb: Array<F>,
}
