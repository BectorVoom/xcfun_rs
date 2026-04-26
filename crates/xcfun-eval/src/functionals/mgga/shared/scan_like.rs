//! SCAN-family exchange + correlation enhancement helpers.
//!
//! Phase 4 plan 04-00 Wave 0 substrate per CONTEXT D-01-A. This module is
//! the largest single helper in Phase 4 — it ports `SCAN_like_eps.hpp`
//! (522 LOC) covering the SCAN, rSCAN, r++SCAN, r2SCAN, r4SCAN families.
//!
//! # IDELEC comptime dispatch (PATTERNS C.4)
//!
//! Every SCAN-family functional differs from its siblings only in the
//! regularisation flavour applied to `α`, `p`, and the gradient-expansion
//! coefficients. The C++ source uses runtime `int IDELEC, int IINTERP,
//! int IDELFX` arguments to switch behaviour; in our cubecl port these
//! become `#[comptime] u32` parameters so each call site monomorphises
//! the kernel into a single SCAN variant — no runtime branching inside
//! the kernel.
//!
//! IDELEC mapping (per PATTERNS):
//! - 0 = SCAN     (Sun-Ruzsinszky-Perdew 2015)
//! - 1 = rSCAN    (Bartok-Yates 2019)
//! - 2 = r++SCAN  (Furness-Kaplan-Ning-Perdew-Sun, in prep)
//! - 3 = r2SCAN   (Furness-Kaplan-Ning-Perdew-Sun JPCL accepted)
//! - 4 = r4SCAN   (Furness-Kaplan-Ning-Perdew-Sun, in prep)
//!
//! # Wave 0 status
//!
//! Each `pub fn` exported below is a SKELETON — signature is final, body
//! is a placeholder zero-fill. Wave 2 (Plan 04-02) ports the FULL bodies
//! per `SCAN_like_eps.hpp` line-for-line. The IDELEC dispatch shape is
//! locked here so SCAN family kernels in 04-02 plug in directly.
//!
//! # Sources (all in `xcfun-master/src/functionals/`)
//! - `SCAN_like_eps.hpp:1-522` — full module port target.
//! - `SCAN_like_eps.hpp:36-69`  — function declarations (matches Rust signatures here).
//! - `SCAN_like_eps.hpp:71-73`  — `fx_unif`.
//! - `SCAN_like_eps.hpp:75-...` — `get_SCAN_Fx` (with IDELEC).
//! - `SCAN_like_eps.hpp:386-461` — full SCAN exchange enhancement.
//! - `SCAN_like_eps.hpp:500-519` — `gcor2` (PW92 helper).

// Match upstream C++ naming exactly (algorithmic-identity rule). Upstream
// uses `get_SCAN_Fx`, `r2SCAN_C` etc — preserve case to ease cross-reference.
#![allow(non_snake_case)]

use cubecl::prelude::*;

// ---------------------------------------------------------------------------
//  fx_unif — uniform-density exchange.  Port of SCAN_like_eps.hpp:71-73.
// ---------------------------------------------------------------------------

/// Uniform exchange energy density `(-3/4)·(3/π)^(1/3)·ρ^(4/3)`.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:71-73` (identical to TPSS `fx_unif`).
#[cube]
pub fn scan_fx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let _ = rho;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

// ---------------------------------------------------------------------------
//  get_SCAN_Fx — SCAN-family exchange enhancement factor with IDELEC dispatch.
//  Port of SCAN_like_eps.hpp:75-... (large multi-branch body).
// ---------------------------------------------------------------------------

/// SCAN-family exchange enhancement factor `F_x`.
///
/// Branches on comptime `idelec` selecting the regularisation variant:
/// - 0 = SCAN     (un-regularised, Sun-Ruzsinszky-Perdew 2015)
/// - 1 = rSCAN    (Bartok-Yates 2019; smooths α regularisation)
/// - 2 = r++SCAN  (regularised + restored)
/// - 3 = r2SCAN   (full GE correction; the recommended r-family default)
/// - 4 = r4SCAN   (4th-order GE correction; HIGH-RISK precision per
///                 CONTEXT D-11 watch list)
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02. The current body
/// dispatches on `idelec` and delegates to a per-variant placeholder so
/// the comptime-dispatch shape is verified at compile time; each branch
/// will be filled in plan 04-02 Task N where N corresponds to the variant.
///
/// Port target: `SCAN_like_eps.hpp:75-461` (full multi-variant body).
#[cube]
pub fn get_SCAN_Fx<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] iinterp: u32,
    #[comptime] idelfx: u32,
    #[comptime] n: u32,
) {
    let _ = rho;
    let _ = grad2;
    let _ = tau;
    let _ = iinterp;
    let _ = idelfx;
    let size = comptime!((1_u32 << n) as usize);

    if comptime!(idelec == 0) {
        // SCAN path (un-regularised). Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 1) {
        // rSCAN path (smoothed α). Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 2) {
        // r++SCAN path. Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 3) {
        // r2SCAN path (GE correction; recommended default). Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 4) {
        // r4SCAN path (4th-order GE; HIGH-RISK polynomial precision).
        // Filled in plan 04-02 with explicit-let-binding (Rule-1 for ACC-06).
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else {
        // Unsupported idelec: leave out unchanged (zero). Caller MUST pass
        // 0..=4; host-side dispatch already enforces this at the kernel
        // launch level.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
//  r2SCAN_C — SCAN-family correlation entry point with IDELEC dispatch.
//  Port of SCAN_like_eps.hpp top-level correlation routine.
// ---------------------------------------------------------------------------

/// SCAN-family correlation energy density.
///
/// Branches on comptime `idelec` (same mapping as `get_SCAN_Fx`).
/// Reads the standard metaGGA Vars `id=13` slots: `a, b, gaa, gab, gbb,
/// taua, taub` plus their derived combinations (`gnn`, `gss`, `tau`).
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp` (correlation top-level + scan_ec0 / scan_ec1
/// helpers below).
#[cube]
pub fn r2SCAN_C<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    gaa: &Array<F>,
    gbb: &Array<F>,
    taua: &Array<F>,
    taub: &Array<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] iinterp: u32,
    #[comptime] idelfx: u32,
    #[comptime] n: u32,
) {
    let _ = a;
    let _ = b;
    let _ = gaa;
    let _ = gbb;
    let _ = taua;
    let _ = taub;
    let _ = iinterp;
    let _ = idelfx;
    let size = comptime!((1_u32 << n) as usize);

    if comptime!(idelec == 0) {
        // SCAN correlation. Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 1) {
        // rSCAN correlation. Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 2) {
        // r++SCAN correlation. Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 3) {
        // r2SCAN correlation (recommended default). Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else if comptime!(idelec == 4) {
        // r4SCAN correlation. Filled in plan 04-02.
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    } else {
        #[unroll]
        for i in 0..size {
            out[i] = F::new(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
//  Sub-helpers for SCAN_like_eps correlation.
//  Port of SCAN_like_eps.hpp:46-69 declarations.
// ---------------------------------------------------------------------------

/// SCAN correlation `ec0` — single-spin (paramagnetic) branch.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:48-53` declaration + body (later in file).
#[cube]
pub fn scan_ec0<F: Float>(
    rs: &Array<F>,
    s2: &Array<F>,
    zeta: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rs;
    let _ = s2;
    let _ = zeta;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// SCAN correlation `ec1` — full coupling-constant integrated correlation.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:57-65` declaration.
#[cube]
pub fn scan_ec1<F: Float>(
    rs: &Array<F>,
    s2: &Array<F>,
    zeta: &Array<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] n: u32,
) {
    let _ = rs;
    let _ = s2;
    let _ = zeta;
    let _ = idelec;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// SCAN correlation `lda_0` — LSDA baseline used as fallback.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:55` declaration.
#[cube]
pub fn lda_0<F: Float>(rs: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let _ = rs;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}

/// PW92 correlation helper `gcor2` — outputs `GG` and `GGRS` together.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:69` declaration + `:500-519` body.
///
/// Out-parameter pattern: cubecl `#[cube] fn` cannot return tuples cleanly,
/// so `GG` and `GGRS` are passed as separate `&mut Array<F>` outputs.
#[cube]
pub fn gcor2<F: Float>(
    rs: &Array<F>,
    sqrtrs: &Array<F>,
    gg: &mut Array<F>,
    ggrs: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rs;
    let _ = sqrtrs;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        gg[i] = F::new(0.0);
        ggrs[i] = F::new(0.0);
    }
}

/// LSDA-1 helper — outputs `eps` and `eps_rs` together.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: `SCAN_like_eps.hpp:67` declaration.
///
/// Out-parameter pattern (same reason as `gcor2`).
#[cube]
pub fn get_lsda1<F: Float>(
    rs: &Array<F>,
    zeta: &Array<F>,
    sqrtrs: &Array<F>,
    eps: &mut Array<F>,
    eps_rs: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = rs;
    let _ = zeta;
    let _ = sqrtrs;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        eps[i] = F::new(0.0);
        eps_rs[i] = F::new(0.0);
    }
}

/// `ufunc(zeta, p)` helper — `pow(1+zeta, p) + pow(1-zeta, p)`.
///
/// **WAVE-2 SKELETON** — full body lands in plan 04-02.
///
/// Port target: this helper is shared with `tpssc_eps.hpp` and is also
/// declared in `SCAN_like_eps.hpp` (used by spin-interpolation kernels).
#[cube]
pub fn ufunc<F: Float>(
    zeta: &Array<F>,
    p: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let _ = zeta;
    let _ = p;
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out[i] = F::new(0.0);
    }
}
