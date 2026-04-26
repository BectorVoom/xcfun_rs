//! Minnesota M05/M06 family helpers (M0x kinetic-energy enhancement substrate).
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A. Ports the helper
//! shapes consumed by Wave 3 family kernels (M05X, M05C, M052X, M052XC,
//! M06X, M06C, M06LX, M06LC, M06HFX, M06HFC, M06X2X, M06X2C). Wave 3
//! (Plan 04-03) ports the FULL bodies.
//!
//! # Sources
//! - `xcfun-master/src/functionals/m0xy_fun.hpp:1-262` — full module port target.
//! - Reuses `pw92eps::pw92eps` from `crates/xcfun-eval/src/functionals/lda/pw92eps.rs`
//!   (Phase 2 deliverable, GREEN at strict 1e-12).
//! - Reuses `pw9xx::chi2` from `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs`
//!   (Phase 3 deliverable).
//!
//! # Wave 0 status
//!
//! Each `pub fn` below is a SKELETON — signature is final, body is a
//! placeholder. Wave 3 (Plan 04-03) replaces with FULL bodies including
//! M05/M06 12-coefficient parameter array Horner evaluation per
//! PATTERNS C.5 (`b97_poly` analog).

// Match upstream C++ naming (`Dsigma`, etc) — algorithmic-identity rule.
#![allow(non_snake_case)]

use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  zet — kinetic-energy density working variable.
//  Port of m0xy_fun.hpp:64-68.
// ---------------------------------------------------------------------------

/// `zet(rho, tau) = 2·tau / ρ^(5/3) - C_F · scalefactor_TF`.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
///
/// Port target: `m0xy_fun.hpp:64-68`.
#[cube]
pub fn m0x_zet<F: Float>(
    rho: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rho;
    let _ = tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  gamma — denominator function.  Port of m0xy_fun.hpp:73-76.
// ---------------------------------------------------------------------------

/// `gamma(α, χ², zet) = 1 + α·(χ² + zet)`.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m0x_gamma<F: Float>(
    alpha: F,
    chi2: &Array<F>,
    zet: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = alpha;
    let _ = chi2;
    let _ = zet;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  h — exchange polynomial.  Port of m0xy_fun.hpp:84-97.
// ---------------------------------------------------------------------------

/// M0x exchange polynomial `h(d[6], α, χ², zet)`.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
///
/// Port target: `m0xy_fun.hpp:84-97`.
#[cube]
pub fn m0x_h<F: Float>(
    d_coeffs: &Array<F>,
    alpha: F,
    chi2: &Array<F>,
    zet: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = d_coeffs;
    let _ = alpha;
    let _ = chi2;
    let _ = zet;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  fw — kinetic-energy enhancement factor (12-coefficient polynomial).
//  Port of m0xy_fun.hpp:106-...
// ---------------------------------------------------------------------------

/// M0x kinetic-energy density enhancement factor `fw(a[12], rho, tau)`.
///
/// 12-coefficient Horner polynomial — RESEARCH §"M05 family" Pitfall P11:
/// preserve descending-Horner order to match C++ `specmath.hpp:24-33`.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
///
/// Port target: `m0xy_fun.hpp:106-...`. Used by every M05/M06 exchange
/// kernel (12 functionals total).
#[cube]
pub fn m0x_fw<F: Float>(
    coeffs: &Array<F>,
    rho: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = coeffs;
    let _ = rho;
    let _ = tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  chi² — reduced gradient (uses pw91_like::chi2 from gga/shared).
// ---------------------------------------------------------------------------

/// M0x `chi²` reduced-gradient working variable.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03. The Wave-3 port
/// will delegate to `crate::functionals::gga::shared::pw91_like::chi2`
/// rather than re-implement (already GREEN at strict 1e-12 from Phase 3).
#[cube]
pub fn m0x_chi2<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rho;
    let _ = grad2;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  D_sigma — exchange-hole spin-decomposed enhancement.
// ---------------------------------------------------------------------------

/// M0x `D_sigma` — single-spin exchange-hole enhancement.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m0x_Dsigma<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rho;
    let _ = grad2;
    let _ = tau;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  g — correlation polynomial.
// ---------------------------------------------------------------------------

/// M0x correlation polynomial `g`.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m0x_g<F: Float>(
    coeffs: &Array<F>,
    chi2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = coeffs;
    let _ = chi2;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  M06 correlation antiparallel + parallel branches.
// ---------------------------------------------------------------------------

/// M06 correlation antiparallel branch.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m06_c_anti<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    gaa: &Array<F>,
    gbb: &Array<F>,
    taua: &Array<F>,
    taub: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = b;
    let _ = gaa;
    let _ = gbb;
    let _ = taua;
    let _ = taub;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// M06 correlation parallel branch.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m06_c_para<F: Float>(
    a: &Array<F>,
    gaa: &Array<F>,
    taua: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = gaa;
    let _ = taua;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  M05 correlation antiparallel + parallel branches.
// ---------------------------------------------------------------------------

/// M05 correlation antiparallel branch.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m05_c_anti<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    gaa: &Array<F>,
    gbb: &Array<F>,
    taua: &Array<F>,
    taub: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = b;
    let _ = gaa;
    let _ = gbb;
    let _ = taua;
    let _ = taub;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// M05 correlation parallel branch.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn m05_c_para<F: Float>(
    a: &Array<F>,
    gaa: &Array<F>,
    taua: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = gaa;
    let _ = taua;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  UEG correlation parallel + antiparallel.
// ---------------------------------------------------------------------------

/// UEG correlation parallel.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn ueg_c_para<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let _ = rho;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// UEG correlation antiparallel.
///
/// **WAVE-3 SKELETON** — full body lands in plan 04-03.
#[cube]
pub fn ueg_c_anti<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = b;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}
