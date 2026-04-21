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
    ctaylor_zero::<F>(&mut out.zeta, n);
    ctaylor_zero::<F>(&mut out.r_s, n);
    ctaylor_zero::<F>(&mut out.n_m13, n);
    ctaylor_zero::<F>(&mut out.a_43, n);
    ctaylor_zero::<F>(&mut out.b_43, n);
    ctaylor_zero::<F>(&mut out.jpaa, n);
    ctaylor_zero::<F>(&mut out.jpbb, n);

    // Variant dispatch (comptime if-chain). Phase 2 ships XC_A_B = 2 and
    // XC_A_B_GAA_GAB_GBB = 6 (Plan 02-05 Wave-1C-1 Pitfall PHASE2-D fix).
    if comptime!(vars == 2) {
        // XC_A_B (densvars.hpp:65-72). 8 of 11 LDAs use this arm.
        build_xc_a_b::<F>(input, out, n);
    } else if comptime!(vars == 6) {
        // XC_A_B_GAA_GAB_GBB (densvars.hpp:58-72). 2 LDAs (LDA-09 part 2 TW,
        // LDA-10 VWK) — Pitfall PHASE2-D fix (see build_xc_a_b_gaa_gab_gbb).
        build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
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
