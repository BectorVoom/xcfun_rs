//! `build_densvars` — comptime-dispatched density-variables builder.
//!
//! Phase 2 Wave-1B-3 ships:
//! - `build_densvars` top-level dispatcher (comptime if-chain over Vars discriminants)
//! - `build_xc_a_b` (8 of 11 LDA functionals: SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK)
//! - Derived-field section (zeta, r_s, n_m13, a_43, b_43) common to all variants
//!
//! Plan 02-05 Wave-1C-1 extends with `build_xc_a_b_gaa_gab_gbb` for TW + VWK (kinetic-GGAs).
//! Phase 3+ adds the remaining 26 variant arms.
//!
//! Sources:
//! - `xcfun-master/src/densvars.hpp:35-218` (switch-case constructor with C-fallthrough)
//! - `xcfun-master/src/densvars.hpp:213-217` (derived-field section — same 5 lines for every variant)
//! - CORE-05 + Pitfall P5 (no fallthrough; explicit helper-function chains)

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_reciprocal};

use super::DensVarsDev;
use super::regularize::regularize;

/// `(3 / (4 * π))^(1/3)` — Wigner-Seitz radius prefactor. Matches
/// `xcfun-core::constants::RS_PREFACTOR = 0.6203504908994001`.
///
/// NOTE: cubecl 0.10-pre.3 `F::new` takes `f32`; the magnitude 0.6203504908994001
/// is representable in f32 with ≤ 6e-9 absolute error. If Plan 02-04/05 fixture
/// gate exposes rel-error > 1e-12 attributable to this widening, replace with
/// `F::cast_from(RS_PREFACTOR_F64)` where `RS_PREFACTOR_F64: f64`.
const RS_PREFACTOR_F32: f32 = 0.620_350_5_f32;

/// Top-level density-variables builder. Comptime-dispatches on `vars` (the
/// `Vars` discriminant as u32) into per-variant helper chains, then fills
/// the 5 common derived fields (zeta, r_s, n_m13, a_43, b_43).
///
/// Phase 2 supports 5 variant arms (XC_A=0, XC_N=1, XC_A_B=2, XC_N_S=3,
/// XC_A_B_GAA_GAB_GBB=6). Wave-1B-3 ships only XC_A_B; Plan 02-05 Wave-1C-1
/// adds XC_A_B_GAA_GAB_GBB. Other Phase-2 arms (XC_A, XC_N, XC_N_S) are
/// host-side rejected by `Functional::eval` BEFORE launch.
#[cube]
pub fn build_densvars<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] vars: u32,
    #[comptime] n: u32,
) {
    // Defensive zero-init: cubecl's Array does not auto-zero per RESEARCH §"build_densvars Pattern".
    // Plan 02-05 Wave-1C-1's gradient builder reads/writes the same DensVarsDev fields, so
    // unsetting these would corrupt cross-variant invariants. 22 ctaylor_zero calls — fully
    // unrolled by cubecl since N is comptime.
    ctaylor_zero::<F>(&mut out.a, n);
    ctaylor_zero::<F>(&mut out.b, n);
    ctaylor_zero::<F>(&mut out.gaa, n);
    ctaylor_zero::<F>(&mut out.gab, n);
    ctaylor_zero::<F>(&mut out.gbb, n);
    ctaylor_zero::<F>(&mut out.n, n);
    ctaylor_zero::<F>(&mut out.s, n);
    ctaylor_zero::<F>(&mut out.gnn, n);
    ctaylor_zero::<F>(&mut out.gns, n);
    ctaylor_zero::<F>(&mut out.gss, n);
    ctaylor_zero::<F>(&mut out.tau, n);
    ctaylor_zero::<F>(&mut out.taua, n);
    ctaylor_zero::<F>(&mut out.taub, n);
    ctaylor_zero::<F>(&mut out.lapa, n);
    ctaylor_zero::<F>(&mut out.lapb, n);
    // B2 (plan 03-01): lapn + laps added for Mode::Potential on total/spin density.
    ctaylor_zero::<F>(&mut out.lapn, n);
    ctaylor_zero::<F>(&mut out.laps, n);
    ctaylor_zero::<F>(&mut out.zeta, n);
    ctaylor_zero::<F>(&mut out.r_s, n);
    ctaylor_zero::<F>(&mut out.n_m13, n);
    ctaylor_zero::<F>(&mut out.a_43, n);
    ctaylor_zero::<F>(&mut out.b_43, n);
    ctaylor_zero::<F>(&mut out.jpaa, n);
    ctaylor_zero::<F>(&mut out.jpbb, n);

    // Variant dispatch (comptime if-chain). Phase 2 ships XC_A_B = 2 and
    // XC_A_B_GAA_GAB_GBB = 6 (Plan 02-05 Wave-1C-1 Pitfall PHASE2-D fix).
    // Phase 3 plan 03-01 adds 7 new arms per D-10 (corrected discriminants
    // per D-10-A: _2ND_TAYLOR = 27..30, not 26..29). Per D-11, each arm
    // uses an explicit helper-function chain (never C-style fallthrough).
    if comptime!(vars == 2) {
        // XC_A_B (densvars.hpp:65-72). 8 of 11 LDAs use this arm.
        build_xc_a_b::<F>(input, out, n);
    } else if comptime!(vars == 6) {
        // XC_A_B_GAA_GAB_GBB (densvars.hpp:58-72). 2 LDAs (LDA-09 part 2 TW,
        // LDA-10 VWK) — Pitfall PHASE2-D fix (see build_xc_a_b_gaa_gab_gbb).
        build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
    }
    // ----- Phase 3 additions (D-10 + D-10-A; discriminants per enums.rs:36-76) -----
    else if comptime!(vars == 0) {
        // XC_A — single-spin density (Wave 2 use; W4 low-level arm).
        build_xc_a::<F>(input, out, n);
    } else if comptime!(vars == 1) {
        // XC_N — total density (Wave 2 use; W4 low-level arm).
        build_xc_n::<F>(input, out, n);
    } else if comptime!(vars == 3) {
        // XC_N_S — total + spin density (Wave 2 use; W4 low-level arm).
        build_xc_n_s::<F>(input, out, n);
    } else if comptime!(vars == 4) {
        build_xc_a_gaa::<F>(input, out, n);
    } else if comptime!(vars == 5) {
        build_xc_n_gnn::<F>(input, out, n);
    } else if comptime!(vars == 7) {
        build_xc_n_s_gnn_gns_gss::<F>(input, out, n);
    } else if comptime!(vars == 27) {
        build_xc_a_2nd_taylor::<F>(input, out, n);
    } else if comptime!(vars == 28) {
        build_xc_a_b_2nd_taylor::<F>(input, out, n);
    } else if comptime!(vars == 29) {
        build_xc_n_2nd_taylor::<F>(input, out, n);
    } else if comptime!(vars == 30) {
        build_xc_n_s_2nd_taylor::<F>(input, out, n);
    }
    // (Other arms guarded by host-side Functional::eval pre-launch check.)

    // Derived fields (densvars.hpp:213-217) — common to every variant, run after the variant arm.
    // zeta = s / n  →  zeta = s * (1/n)
    let mut inv_n = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_reciprocal::<F>(&out.n, &mut inv_n, n);
    ctaylor_mul::<F>(&out.s, &inv_n, &mut out.zeta, n);

    // n_m13 = pow(n, -1/3)
    //   Use F::cast_from(f64) for the exponent to preserve 1/3 precision to 1e-16
    //   rather than f32's ~1e-7 — critical for the 1e-11 tier-1 threshold.
    ctaylor_pow::<F>(&out.n, F::cast_from(-1.0_f64 / 3.0_f64), &mut out.n_m13, n);

    // a_43 = pow(a, 4/3); b_43 = pow(b, 4/3)
    ctaylor_pow::<F>(&out.a, F::cast_from(4.0_f64 / 3.0_f64), &mut out.a_43, n);
    ctaylor_pow::<F>(&out.b, F::cast_from(4.0_f64 / 3.0_f64), &mut out.b_43, n);

    // r_s = RS_PREFACTOR * n_m13
    //   RS_PREFACTOR = 0.6203504908994001 — f64 precision for 1e-11 parity.
    let _ = RS_PREFACTOR_F32;
    ctaylor_scalar_mul::<F>(&out.n_m13, F::cast_from(0.6203504908994001_f64), &mut out.r_s, n);
}

/// `XC_A_B` variant arm — populates `a`, `b`, `n`, `s` from a pre-seeded
/// flat CTaylor coefficient input of length `2 * (1 << n)`.
///
/// **Plan 02-04 Wave-1B-14a amendment:** input layout changed from 2 scalars
/// `[a_scalar, b_scalar]` to a flat pre-seeded CTaylor coefficient block:
/// - `input[0..(1<<n)]`         = coefficients of `a` (CTaylor<F, n>)
/// - `input[(1<<n)..(2*(1<<n))]` = coefficients of `b` (CTaylor<F, n>)
///
/// Host-side `Functional::eval` packs the derivative-seed markers (VAR0=1 on
/// input slot i, VAR1=1 on input slot j) into the flat input BEFORE launch,
/// so the kernel receives pre-seeded Taylor polynomials. This is required for
/// computing partial derivatives via the single-launch Taylor-series approach
/// (RESEARCH §"Mode::PartialDerivatives Output Layout").
///
/// 1:1 port of `xcfun-master/src/densvars.hpp:65-72` — the original scalar
/// `a = d[0]` is generalised to a Taylor-coefficient copy (preserving all
/// seeded derivative markers). `regularize` still clamps only `a[CNST]`
/// (CORE-06 + D-22), so the derivative coefficients are preserved.
///
/// Used by 8 LDAs: SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK.
/// Pitfall PHASE2-D: TW + VWK use XC_A_B_GAA_GAB_GBB (Plan 02-05 Wave-1C-1), NOT this arm.
#[cube]
pub fn build_xc_a_b<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy pre-seeded coefficients of `a` from input[0..size] into out.a.
    #[unroll]
    for i in 0..size {
        out.a[i] = input[i];
    }
    // regularize(a) — clamps out.a[CNST] to >= TINY_DENSITY; derivative coeffs unchanged.
    regularize::<F>(&mut out.a, n);

    // Copy pre-seeded coefficients of `b` from input[size..2*size] into out.b.
    #[unroll]
    for i in 0..size {
        out.b[i] = input[size + i];
    }
    regularize::<F>(&mut out.b, n);

    // n = a + b; s = a - b;
    ctaylor_add::<F>(&out.a, &out.b, &mut out.n, n);
    ctaylor_sub::<F>(&out.a, &out.b, &mut out.s, n);
}

/// `XC_A_B_GAA_GAB_GBB` variant arm — populates `gaa`, `gab`, `gbb` from the
/// gradient-bearing input slots, derives `gnn`, `gss`, `gns`, then EXPLICITLY
/// chains to `build_xc_a_b` to populate `a`, `b`, `n`, `s` (replacing the
/// C-style fallthrough at `xcfun-master/src/densvars.hpp:65-72` per CORE-05 +
/// Pitfall P5).
///
/// 1:1 port of `xcfun-master/src/densvars.hpp:58-72`:
///
/// ```cpp
/// case XC_A_B_GAA_GAB_GBB:
///     gaa = d[2]; gab = d[3]; gbb = d[4];
///     gnn = gaa + 2 * gab + gbb;
///     gss = gaa - 2 * gab + gbb;
///     gns = gaa - gbb;
/// case XC_A_B:               // <-- C-style fallthrough (explicit chain here)
///     a = d[0]; regularize(a);
///     b = d[1]; regularize(b);
///     n = a + b; s = a - b;
///     break;
/// ```
///
/// Input layout (inlen=5, pre-seeded CTaylor per Plan 02-04 Wave-1B-14a amendment):
///   - `input[0..(1<<n)]`               = coefficients of `a`
///   - `input[(1<<n)..(2<<n)]`          = coefficients of `b`
///   - `input[(2<<n)..(3<<n)]`          = coefficients of `gaa`
///   - `input[(3<<n)..(4<<n)]`          = coefficients of `gab`
///   - `input[(4<<n)..(5<<n)]`          = coefficients of `gbb`
///
/// Used by 2 LDAs: LDA-09 part 2 (TW, tw.cpp) and LDA-10 (VWK, vonw.cpp).
///
/// # Pitfall PHASE2-D
/// TW + VWK declare `XC_DENSITY | XC_GRADIENT` and REQUIRE this arm — the pure-density
/// `XC_A_B` arm leaves `gaa = gbb = 0` (defensive zero-init from Plan 02-03 Wave-1B-3),
/// so TW/VWK would silently return zero if driven through the wrong builder.
#[cube]
pub fn build_xc_a_b_gaa_gab_gbb<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy pre-seeded coefficients of `gaa` from input[2*size..3*size] into out.gaa.
    #[unroll]
    for i in 0..size {
        out.gaa[i] = input[2 * size + i];
    }
    // Copy pre-seeded coefficients of `gab` from input[3*size..4*size] into out.gab.
    #[unroll]
    for i in 0..size {
        out.gab[i] = input[3 * size + i];
    }
    // Copy pre-seeded coefficients of `gbb` from input[4*size..5*size] into out.gbb.
    #[unroll]
    for i in 0..size {
        out.gbb[i] = input[4 * size + i];
    }

    // gnn = gaa + 2*gab + gbb   (left-to-right, no mul_add per ACC-06)
    let mut t1 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut t2 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&out.gab, F::cast_from(2.0_f64), &mut t1, n); // t1 = 2*gab
    ctaylor_add::<F>(&out.gaa, &t1, &mut t2, n); // t2 = gaa + 2*gab
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gnn, n); // gnn = (gaa + 2*gab) + gbb

    // gss = gaa - 2*gab + gbb   (reuse t1 = 2*gab; reset t2)
    ctaylor_sub::<F>(&out.gaa, &t1, &mut t2, n); // t2 = gaa - 2*gab
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gss, n); // gss = (gaa - 2*gab) + gbb

    // gns = gaa - gbb
    ctaylor_sub::<F>(&out.gaa, &out.gbb, &mut out.gns, n);

    // EXPLICIT chain to XC_A_B (replaces C fallthrough at densvars.hpp:65-72).
    build_xc_a_b::<F>(input, out, n);
}

// ----------------------------------------------------------------------------
//  Phase 3 plan 03-01 additions (W4 + D-10 + D-10-A).
//
//  All arms follow the Phase 2 pattern: slot-copy from pre-seeded CTaylor input,
//  regularize where appropriate, derive any gradient / Laplacian / spin fields,
//  then EXPLICITLY chain to the lower-variant builder (replaces C-style
//  fallthrough at densvars.hpp per D-11 + Pitfall P5).
// ----------------------------------------------------------------------------

/// `XC_A` variant arm (vars=0, inlen=1) — single-spin α-density only.
///
/// Input layout (pre-seeded CTaylor per D-12): `input[0..(1<<n)]` = coeffs of `a`.
///
/// Derives: `b = 0`, `n = a`, `s = 0`. Does NOT populate `gaa/gbb/gab` (caller's
/// responsibility if those are needed — the XC_A variant provides density only).
///
/// W4 resolution: added in plan 03-01 since the Phase-2 build.rs had only
/// `build_xc_a_b` / `build_xc_a_b_gaa_gab_gbb` — this low-level arm is the
/// chain target for `build_xc_a_gaa` and `build_xc_a_2nd_taylor`.
#[cube]
pub fn build_xc_a<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out.a[i] = input[i];
    }
    regularize::<F>(&mut out.a, n);
    // b = 0 (single-spin); n = a; s = 0 (since b = 0).
    #[unroll]
    for i in 0..size {
        out.b[i] = F::new(0.0);
        out.n[i] = out.a[i];
        out.s[i] = F::new(0.0);
    }
}

/// `XC_N` variant arm (vars=1, inlen=1) — total density only, closed-shell.
///
/// Input layout (pre-seeded CTaylor per D-12): `input[0..(1<<n)]` = coeffs of `n`.
///
/// Derives: `a = b = n/2` (closed-shell), `s = 0`.
///
/// W4 resolution: chain target for `build_xc_n_gnn` and `build_xc_n_2nd_taylor`.
#[cube]
pub fn build_xc_n<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out.n[i] = input[i];
    }
    regularize::<F>(&mut out.n, n);
    // a = b = n/2; s = 0.
    let half = F::cast_from(0.5_f64);
    ctaylor_scalar_mul::<F>(&out.n, half, &mut out.a, n);
    #[unroll]
    for i in 0..size {
        out.b[i] = out.a[i];
        out.s[i] = F::new(0.0);
    }
}

/// `XC_N_S` variant arm (vars=3, inlen=2) — total + spin density, open-shell.
///
/// Input layout (pre-seeded CTaylor per D-12):
/// - `input[0..size]`      = coeffs of `n`
/// - `input[size..2*size]` = coeffs of `s`
///
/// Derives: `a = (n + s)/2`, `b = (n − s)/2`.
///
/// W4 resolution: chain target for `build_xc_n_s_gnn_gns_gss` and
/// `build_xc_n_s_2nd_taylor`.
#[cube]
pub fn build_xc_n_s<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out.n[i] = input[i];
        out.s[i] = input[size + i];
    }
    regularize::<F>(&mut out.n, n);
    // a = (n + s)/2; b = (n − s)/2.
    let half = F::cast_from(0.5_f64);
    let mut sum = Array::<F>::new(size);
    let mut diff = Array::<F>::new(size);
    ctaylor_add::<F>(&out.n, &out.s, &mut sum, n);
    ctaylor_sub::<F>(&out.n, &out.s, &mut diff, n);
    ctaylor_scalar_mul::<F>(&sum, half, &mut out.a, n);
    ctaylor_scalar_mul::<F>(&diff, half, &mut out.b, n);
}

/// `XC_A_GAA` variant arm (vars=4, inlen=2) — single-spin α-density + gradient².
///
/// Input layout:
/// - `input[0..size]`      = coeffs of `a`
/// - `input[size..2*size]` = coeffs of `gaa`
///
/// Explicit chain to `build_xc_a` (replaces C fallthrough at densvars.hpp ~90).
#[cube]
pub fn build_xc_a_gaa<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out.gaa[i] = input[size + i];
    }
    // Explicit chain: build_xc_a populates a, b, n, s from input[0..size].
    build_xc_a::<F>(input, out, n);
}

/// `XC_N_GNN` variant arm (vars=5, inlen=2) — total-density + |∇n|².
///
/// Input layout:
/// - `input[0..size]`      = coeffs of `n`
/// - `input[size..2*size]` = coeffs of `gnn`
///
/// Explicit chain to `build_xc_n` (replaces C fallthrough at densvars.hpp ~95).
#[cube]
pub fn build_xc_n_gnn<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    #[unroll]
    for i in 0..size {
        out.gnn[i] = input[size + i];
    }
    // Explicit chain: build_xc_n populates n, a, b, s from input[0..size].
    build_xc_n::<F>(input, out, n);
}

/// `XC_N_S_GNN_GNS_GSS` variant arm (vars=7, inlen=5) — total + spin density
/// with full gradient inner products.
///
/// Input layout (5 slots):
/// - `input[0..size]`        = coeffs of `n`
/// - `input[size..2*size]`   = coeffs of `s`
/// - `input[2*size..3*size]` = coeffs of `gnn`
/// - `input[3*size..4*size]` = coeffs of `gns`
/// - `input[4*size..5*size]` = coeffs of `gss`
///
/// Derives `gaa`, `gab`, `gbb` from inversions:
/// - `gaa = (gnn + 2·gns + gss) / 4`
/// - `gbb = (gnn − 2·gns + gss) / 4`
/// - `gab = (gnn − gss) / 4`
///
/// Explicit chain to `build_xc_n_s` (replaces C fallthrough at densvars.hpp ~105).
#[cube]
pub fn build_xc_n_s_gnn_gns_gss<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy pre-seeded gnn, gns, gss from slots 2/3/4.
    #[unroll]
    for i in 0..size {
        out.gnn[i] = input[2 * size + i];
        out.gns[i] = input[3 * size + i];
        out.gss[i] = input[4 * size + i];
    }

    // gaa = (gnn + 2·gns + gss) / 4
    let mut t1 = Array::<F>::new(size);
    let mut t2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&out.gns, F::cast_from(2.0_f64), &mut t1, n); // t1 = 2·gns
    ctaylor_add::<F>(&out.gnn, &t1, &mut t2, n); // t2 = gnn + 2·gns
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&t2, &out.gss, &mut sum, n); // sum = gnn + 2·gns + gss
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.25_f64), &mut out.gaa, n);

    // gbb = (gnn − 2·gns + gss) / 4
    ctaylor_sub::<F>(&out.gnn, &t1, &mut t2, n); // t2 = gnn − 2·gns
    ctaylor_add::<F>(&t2, &out.gss, &mut sum, n); // sum = gnn − 2·gns + gss
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.25_f64), &mut out.gbb, n);

    // gab = (gnn − gss) / 4
    let mut diff = Array::<F>::new(size);
    ctaylor_sub::<F>(&out.gnn, &out.gss, &mut diff, n);
    ctaylor_scalar_mul::<F>(&diff, F::cast_from(0.25_f64), &mut out.gab, n);

    // Explicit chain: build_xc_n_s populates n, s, a, b from input[0..2*size].
    build_xc_n_s::<F>(input, out, n);
}

/// `XC_A_2ND_TAYLOR` variant arm (vars=27, inlen=10) — α-density with full
/// 2nd-order spatial Taylor expansion at a point.
///
/// Input slot layout (`XCFunctional.cpp:679` comment `"n gx gy gz xx xy xz yy yz zz"`):
///   slot 0: a  (= density)
///   slot 1: a_x   (∂a/∂x)
///   slot 2: a_y   (∂a/∂y)
///   slot 3: a_z   (∂a/∂z)
///   slot 4: a_xx  (∂²a/∂x²)
///   slot 5: a_xy  (∂²a/∂x∂y)
///   slot 6: a_xz  (∂²a/∂x∂z)
///   slot 7: a_yy  (∂²a/∂y²)
///   slot 8: a_yz  (∂²a/∂y∂z)
///   slot 9: a_zz  (∂²a/∂z²)
///
/// Derives:
///   `gaa  = a_x² + a_y² + a_z²`
///   `lapa = 0.5 · (a_xx + a_yy + a_zz)`     (C++ xcfun factor, densvars.hpp ~90)
///
/// Pitfall G8 invariant: `regularize` clamps ONLY `a[CNST]`. Higher-order
/// 2nd-Taylor coefficients in slots 1..9 carry derivative-seed markers and
/// MUST pass through unchanged — verified by `regularize_2nd_taylor.rs`.
///
/// Explicit chain to `build_xc_a` (replaces C fallthrough at XCFunctional.cpp:675-695).
#[cube]
pub fn build_xc_a_2nd_taylor<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy gradient components + 2nd-order spatial partials from slots 1..9.
    let mut gx = Array::<F>::new(size);
    let mut gy = Array::<F>::new(size);
    let mut gz = Array::<F>::new(size);
    let mut nxx = Array::<F>::new(size);
    let mut nyy = Array::<F>::new(size);
    let mut nzz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        gx[i] = input[size + i];
        gy[i] = input[2 * size + i];
        gz[i] = input[3 * size + i];
        nxx[i] = input[4 * size + i];
        nyy[i] = input[7 * size + i];
        nzz[i] = input[9 * size + i];
    }

    // gaa = gx² + gy² + gz²  (no mul_add per ACC-06)
    let mut gx2 = Array::<F>::new(size);
    let mut gy2 = Array::<F>::new(size);
    let mut gz2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&gx, &gx, &mut gx2, n);
    ctaylor_mul::<F>(&gy, &gy, &mut gy2, n);
    ctaylor_mul::<F>(&gz, &gz, &mut gz2, n);
    let mut gxy2 = Array::<F>::new(size);
    ctaylor_add::<F>(&gx2, &gy2, &mut gxy2, n);
    ctaylor_add::<F>(&gxy2, &gz2, &mut out.gaa, n);

    // lapa = 0.5 · (nxx + nyy + nzz)
    let mut s1 = Array::<F>::new(size);
    let mut lap_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&nxx, &nyy, &mut s1, n);
    ctaylor_add::<F>(&s1, &nzz, &mut lap_sum, n);
    ctaylor_scalar_mul::<F>(&lap_sum, F::cast_from(0.5_f64), &mut out.lapa, n);

    // Explicit chain: build_xc_a populates a (slot 0), b=0, n=a, s=0.
    build_xc_a::<F>(input, out, n);
}

/// `XC_A_B_2ND_TAYLOR` variant arm (vars=28, inlen=20) — α + β densities with
/// full 2nd-order spatial Taylor expansion (double the `XC_A_2ND_TAYLOR` work).
///
/// Input slot layout: α-channel slots 0..9 + β-channel slots 10..19
/// (each channel follows the `XCFunctional.cpp:679` layout).
///
/// Derives:
///   `gaa  = ax² + ay² + az²`           (from α slots 1..3)
///   `gbb  = bx² + by² + bz²`           (from β slots 11..13)
///   `gab  = ax·bx + ay·by + az·bz`     (cross-spin inner product)
///   `lapa = 0.5 · (axx + ayy + azz)`   (from α slots 4, 7, 9)
///   `lapb = 0.5 · (bxx + byy + bzz)`   (from β slots 14, 17, 19)
///
/// Explicit chain to `build_xc_a_b` (populates `a`, `b`, `n`, `s` from slots 0 and 10;
/// note the 10-slot stride differs from `build_xc_a_b`'s `size`-stride — the chain
/// still works because `build_xc_a_b` reads `input[0..size]` and `input[size..2*size]`
/// but only the α and β CNST are clamped + combined; the bulk derivative coefficients
/// in the α/β 10-slot blocks are NOT the same as what `build_xc_a_b` wants).
///
/// **KEY:** the chain call here uses `input` as-is; `build_xc_a_b` will read
/// slots 0 and 10 correctly IFF `size == 10` (N=3, so `1 << 3 = 8`? — no,
/// this chain only works when size is set so the per-input-slot CTaylor size
/// matches). The upstream `XCFunctional.cpp:675-760` handles this via a
/// separate `d[0..=9]` / `d[10..=19]` 20-slot Taylor-seeded block. For Wave 1
/// scaffolding purposes, we replicate the C++ structure but note: the
/// inner `build_xc_a_b::<F>(input, out, n)` call packs α from `input[0..size]`
/// and β from `input[size..2*size]` — which IS consistent with the 2ND_TAYLOR
/// convention because the host packs α and β with offsets `0` and `size`
/// (size = 10 comes from `inlen` via `taylorlen`, but the per-variable
/// CTaylor coefficient array is of length `1 << n` per input slot). Wave 5
/// (plan 03-05) formalises the Mode::Potential launch layout for these vars.
#[cube]
pub fn build_xc_a_b_2nd_taylor<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // --- α-channel (slots 0..9) ---
    let mut ax = Array::<F>::new(size);
    let mut ay = Array::<F>::new(size);
    let mut az = Array::<F>::new(size);
    let mut axx = Array::<F>::new(size);
    let mut ayy = Array::<F>::new(size);
    let mut azz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        ax[i] = input[size + i];
        ay[i] = input[2 * size + i];
        az[i] = input[3 * size + i];
        axx[i] = input[4 * size + i];
        ayy[i] = input[7 * size + i];
        azz[i] = input[9 * size + i];
    }
    // gaa = ax² + ay² + az²
    let mut ax2 = Array::<F>::new(size);
    let mut ay2 = Array::<F>::new(size);
    let mut az2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&ax, &ax, &mut ax2, n);
    ctaylor_mul::<F>(&ay, &ay, &mut ay2, n);
    ctaylor_mul::<F>(&az, &az, &mut az2, n);
    let mut gxy2 = Array::<F>::new(size);
    ctaylor_add::<F>(&ax2, &ay2, &mut gxy2, n);
    ctaylor_add::<F>(&gxy2, &az2, &mut out.gaa, n);
    // lapa = 0.5 · (axx + ayy + azz)
    let mut s1 = Array::<F>::new(size);
    let mut lap_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&axx, &ayy, &mut s1, n);
    ctaylor_add::<F>(&s1, &azz, &mut lap_sum, n);
    ctaylor_scalar_mul::<F>(&lap_sum, F::cast_from(0.5_f64), &mut out.lapa, n);

    // --- β-channel (slots 10..19) ---
    let mut bx = Array::<F>::new(size);
    let mut by = Array::<F>::new(size);
    let mut bz = Array::<F>::new(size);
    let mut bxx = Array::<F>::new(size);
    let mut byy = Array::<F>::new(size);
    let mut bzz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bx[i] = input[11 * size + i];
        by[i] = input[12 * size + i];
        bz[i] = input[13 * size + i];
        bxx[i] = input[14 * size + i];
        byy[i] = input[17 * size + i];
        bzz[i] = input[19 * size + i];
    }
    // gbb = bx² + by² + bz²
    let mut bx2 = Array::<F>::new(size);
    let mut by2 = Array::<F>::new(size);
    let mut bz2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bx, &bx, &mut bx2, n);
    ctaylor_mul::<F>(&by, &by, &mut by2, n);
    ctaylor_mul::<F>(&bz, &bz, &mut bz2, n);
    let mut gxyb2 = Array::<F>::new(size);
    ctaylor_add::<F>(&bx2, &by2, &mut gxyb2, n);
    ctaylor_add::<F>(&gxyb2, &bz2, &mut out.gbb, n);
    // lapb = 0.5 · (bxx + byy + bzz)
    let mut s2 = Array::<F>::new(size);
    let mut lap_sumb = Array::<F>::new(size);
    ctaylor_add::<F>(&bxx, &byy, &mut s2, n);
    ctaylor_add::<F>(&s2, &bzz, &mut lap_sumb, n);
    ctaylor_scalar_mul::<F>(&lap_sumb, F::cast_from(0.5_f64), &mut out.lapb, n);

    // --- Cross-spin gradient inner product ---
    // gab = ax·bx + ay·by + az·bz
    let mut axbx = Array::<F>::new(size);
    let mut ayby = Array::<F>::new(size);
    let mut azbz = Array::<F>::new(size);
    ctaylor_mul::<F>(&ax, &bx, &mut axbx, n);
    ctaylor_mul::<F>(&ay, &by, &mut ayby, n);
    ctaylor_mul::<F>(&az, &bz, &mut azbz, n);
    let mut gab_xy = Array::<F>::new(size);
    ctaylor_add::<F>(&axbx, &ayby, &mut gab_xy, n);
    ctaylor_add::<F>(&gab_xy, &azbz, &mut out.gab, n);

    // Explicit chain: build_xc_a_b reads slots 0 and 10 (α-CNST, β-CNST) and
    // derives n, s. NOTE: for 2ND_TAYLOR the α lives at slot 10*size (not size).
    // We replicate the Phase-2 pattern by directly copying α and β CNST blocks.
    #[unroll]
    for i in 0..size {
        out.a[i] = input[i];
        out.b[i] = input[10 * size + i];
    }
    regularize::<F>(&mut out.a, n);
    regularize::<F>(&mut out.b, n);
    ctaylor_add::<F>(&out.a, &out.b, &mut out.n, n);
    ctaylor_sub::<F>(&out.a, &out.b, &mut out.s, n);
}

/// `XC_N_2ND_TAYLOR` variant arm (vars=29, inlen=10) — total density with
/// full 2nd-order spatial Taylor expansion.
///
/// Input slot layout (same as `XC_A_2ND_TAYLOR` but for `n`):
///   `[n, nx, ny, nz, nxx, nxy, nxz, nyy, nyz, nzz]`
///
/// Derives:
///   `gnn = nx² + ny² + nz²`
///   `lapn = 0.5 · (nxx + nyy + nzz)`   (B2 consumer — populates `out.lapn`)
///
/// Explicit chain to `build_xc_n` (populates `n` from slot 0, derives a, b, s).
#[cube]
pub fn build_xc_n_2nd_taylor<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy gradient + 2nd-order spatial partials from slots 1..9.
    let mut nx = Array::<F>::new(size);
    let mut ny = Array::<F>::new(size);
    let mut nz = Array::<F>::new(size);
    let mut nxx = Array::<F>::new(size);
    let mut nyy = Array::<F>::new(size);
    let mut nzz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        nx[i] = input[size + i];
        ny[i] = input[2 * size + i];
        nz[i] = input[3 * size + i];
        nxx[i] = input[4 * size + i];
        nyy[i] = input[7 * size + i];
        nzz[i] = input[9 * size + i];
    }

    // gnn = nx² + ny² + nz²
    let mut nx2 = Array::<F>::new(size);
    let mut ny2 = Array::<F>::new(size);
    let mut nz2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&nx, &nx, &mut nx2, n);
    ctaylor_mul::<F>(&ny, &ny, &mut ny2, n);
    ctaylor_mul::<F>(&nz, &nz, &mut nz2, n);
    let mut gxy2 = Array::<F>::new(size);
    ctaylor_add::<F>(&nx2, &ny2, &mut gxy2, n);
    ctaylor_add::<F>(&gxy2, &nz2, &mut out.gnn, n);

    // lapn = 0.5 · (nxx + nyy + nzz)  (B2 — populates lapn by name)
    let mut s1 = Array::<F>::new(size);
    let mut lap_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&nxx, &nyy, &mut s1, n);
    ctaylor_add::<F>(&s1, &nzz, &mut lap_sum, n);
    ctaylor_scalar_mul::<F>(&lap_sum, F::cast_from(0.5_f64), &mut out.lapn, n);

    // Explicit chain: build_xc_n populates n, a, b, s from slot 0.
    build_xc_n::<F>(input, out, n);
}

/// `XC_N_S_2ND_TAYLOR` variant arm (vars=30, inlen=20) — total + spin density
/// with full 2nd-order spatial Taylor expansion per channel.
///
/// Input slot layout: α-channel (for `n`) slots 0..9 + β-channel (for `s`)
/// slots 10..19.
///
/// Derives:
///   `gnn  = nx² + ny² + nz²`                 (from n-channel slots 1..3)
///   `gss  = sx² + sy² + sz²`                 (from s-channel slots 11..13)
///   `gns  = nx·sx + ny·sy + nz·sz`           (cross inner product)
///   `lapn = 0.5 · (nxx + nyy + nzz)`         (B2 — from n-channel slots 4, 7, 9)
///   `laps = 0.5 · (sxx + syy + szz)`         (B2 — from s-channel slots 14, 17, 19)
///
/// Explicit chain to `build_xc_n_s` (populates `n` from slot 0, `s` from slot 10,
/// derives `a = (n+s)/2`, `b = (n-s)/2`).
#[cube]
pub fn build_xc_n_s_2nd_taylor<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // --- n-channel (slots 0..9) ---
    let mut nx = Array::<F>::new(size);
    let mut ny = Array::<F>::new(size);
    let mut nz = Array::<F>::new(size);
    let mut nxx = Array::<F>::new(size);
    let mut nyy = Array::<F>::new(size);
    let mut nzz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        nx[i] = input[size + i];
        ny[i] = input[2 * size + i];
        nz[i] = input[3 * size + i];
        nxx[i] = input[4 * size + i];
        nyy[i] = input[7 * size + i];
        nzz[i] = input[9 * size + i];
    }
    // gnn
    let mut nx2 = Array::<F>::new(size);
    let mut ny2 = Array::<F>::new(size);
    let mut nz2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&nx, &nx, &mut nx2, n);
    ctaylor_mul::<F>(&ny, &ny, &mut ny2, n);
    ctaylor_mul::<F>(&nz, &nz, &mut nz2, n);
    let mut gxy2 = Array::<F>::new(size);
    ctaylor_add::<F>(&nx2, &ny2, &mut gxy2, n);
    ctaylor_add::<F>(&gxy2, &nz2, &mut out.gnn, n);
    // lapn
    let mut s1 = Array::<F>::new(size);
    let mut lap_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&nxx, &nyy, &mut s1, n);
    ctaylor_add::<F>(&s1, &nzz, &mut lap_sum, n);
    ctaylor_scalar_mul::<F>(&lap_sum, F::cast_from(0.5_f64), &mut out.lapn, n);

    // --- s-channel (slots 10..19) ---
    let mut sx = Array::<F>::new(size);
    let mut sy = Array::<F>::new(size);
    let mut sz = Array::<F>::new(size);
    let mut sxx = Array::<F>::new(size);
    let mut syy = Array::<F>::new(size);
    let mut szz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        sx[i] = input[11 * size + i];
        sy[i] = input[12 * size + i];
        sz[i] = input[13 * size + i];
        sxx[i] = input[14 * size + i];
        syy[i] = input[17 * size + i];
        szz[i] = input[19 * size + i];
    }
    // gss
    let mut sx2 = Array::<F>::new(size);
    let mut sy2 = Array::<F>::new(size);
    let mut sz2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&sx, &sx, &mut sx2, n);
    ctaylor_mul::<F>(&sy, &sy, &mut sy2, n);
    ctaylor_mul::<F>(&sz, &sz, &mut sz2, n);
    let mut sxys2 = Array::<F>::new(size);
    ctaylor_add::<F>(&sx2, &sy2, &mut sxys2, n);
    ctaylor_add::<F>(&sxys2, &sz2, &mut out.gss, n);
    // laps
    let mut sa = Array::<F>::new(size);
    let mut lap_sums = Array::<F>::new(size);
    ctaylor_add::<F>(&sxx, &syy, &mut sa, n);
    ctaylor_add::<F>(&sa, &szz, &mut lap_sums, n);
    ctaylor_scalar_mul::<F>(&lap_sums, F::cast_from(0.5_f64), &mut out.laps, n);

    // --- Cross inner product: gns = nx·sx + ny·sy + nz·sz ---
    let mut nxsx = Array::<F>::new(size);
    let mut nysy = Array::<F>::new(size);
    let mut nzsz = Array::<F>::new(size);
    ctaylor_mul::<F>(&nx, &sx, &mut nxsx, n);
    ctaylor_mul::<F>(&ny, &sy, &mut nysy, n);
    ctaylor_mul::<F>(&nz, &sz, &mut nzsz, n);
    let mut gns_xy = Array::<F>::new(size);
    ctaylor_add::<F>(&nxsx, &nysy, &mut gns_xy, n);
    ctaylor_add::<F>(&gns_xy, &nzsz, &mut out.gns, n);

    // Explicit chain: populate n (slot 0), s (slot 10), regularize n; derive a, b.
    #[unroll]
    for i in 0..size {
        out.n[i] = input[i];
        out.s[i] = input[10 * size + i];
    }
    regularize::<F>(&mut out.n, n);
    let half = F::cast_from(0.5_f64);
    let mut sum_ns = Array::<F>::new(size);
    let mut diff_ns = Array::<F>::new(size);
    ctaylor_add::<F>(&out.n, &out.s, &mut sum_ns, n);
    ctaylor_sub::<F>(&out.n, &out.s, &mut diff_ns, n);
    ctaylor_scalar_mul::<F>(&sum_ns, half, &mut out.a, n);
    ctaylor_scalar_mul::<F>(&diff_ns, half, &mut out.b, n);
}
