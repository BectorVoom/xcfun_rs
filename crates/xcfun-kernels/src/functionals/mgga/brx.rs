//! BRX, BRC, BRXC kernel functions — port of `xcfun-master/src/functionals/brx.cpp`.
//!
//! All three Becke-Roussel functionals share the `polarized` helper
//! (`br_like::polarized`) and use Vars `id=17` (XC_DENSITY | XC_GRADIENT |
//! XC_KINETIC | XC_LAPLACIAN | XC_JP, inlen=11).
//!
//! # Sources
//! - `xcfun-master/src/functionals/brx.cpp:103-136`.
//! - `crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs` — `polarized` helper.
//!
//! # Dispatch IDs
//! - XC_BRX  = 10
//! - XC_BRC  = 11
//! - XC_BRXC = 12
//!
//! # Caller convention for `taua`
//!
//! `brx.cpp` calls `polarized(d.a, d.gaa, d.lapa, 2*d.taua, d.jpaa)` — the
//! factor 2 is applied at the call site. We follow the same convention: call
//! `polarized` with a CTaylor of `2 * d.taua`, built inside each kernel.
//!
//! # Note on FIXME (Ekström D-26)
//! `brx.cpp:100` contains `// FIXME: use expm1`. Per CONTEXT D-26 we port
//! VERBATIM using `exp`, not `expm1`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::br_like::polarized;

// ---------------------------------------------------------------------------
//  BRX — XC_BRX (id=10).  Port of brx.cpp:103-106.
//
//  E = 0.5 * (d.a * polarized(d.a, d.gaa, d.lapa, 2*d.taua, d.jpaa)
//           + d.b * polarized(d.b, d.gbb, d.lapb, 2*d.taub, d.jpbb))
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
pub fn brx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Build 2*taua and 2*taub (passed to polarized as the `taua` argument).
    let mut taua2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taua, F::cast_from(2.0_f64), &mut taua2, n);

    let mut taub2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taub, F::cast_from(2.0_f64), &mut taub2, n);

    let mut ux_a = Array::<F>::new(size);
    polarized::<F>(&d.a, &d.gaa, &d.lapa, &taua2, &d.jpaa, &mut ux_a, n);

    let mut ux_b = Array::<F>::new(size);
    polarized::<F>(&d.b, &d.gbb, &d.lapb, &taub2, &d.jpbb, &mut ux_b, n);

    // E = 0.5 * (a * UXa + b * UXb)
    let mut a_uxa_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &ux_a, &mut a_uxa_raw, n);

    let mut b_uxb_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &ux_b, &mut b_uxb_raw, n);

    let mut sum_raw = Array::<F>::new(size);
    ctaylor_add::<F>(&a_uxa_raw, &b_uxb_raw, &mut sum_raw, n);

    ctaylor_scalar_mul::<F>(&sum_raw, F::cast_from(0.5_f64), out, n);
}

// ---------------------------------------------------------------------------
//  BRC — XC_BRC (id=11).  Port of brx.cpp:108-121.
//
//  parameter cab = 0.63, caa = 0.88
//  UXa = polarized(d.a, d.gaa, d.lapa, 2*d.taua, d.jpaa)
//  UXb = polarized(d.b, d.gbb, d.lapb, 2*d.taub, d.jpbb)
//  zaa = |caa * (2 / UXa)|
//  zbb = |caa * (2 / UXb)|
//  zab = |cab * (1/UXa + 1/UXb)|
//  ECopp = -0.8 * a * b * zab^2 * (1 - log(1 + zab) / zab)
//  ECaa  = -0.01 * a * (2*taua - (0.25*gaa + jpaa)/a) * zaa^4 * (1 - 2/zaa*log(1+zaa/2))
//  ECbb  = -0.01 * b * (2*taub - (0.25*gbb + jpbb)/b) * zbb^4 * (1 - 2/zbb*log(1+zbb/2))
//  return ECopp + ECaa + ECbb
//
//  Note: abs() on a CTaylor applies to the CNST slot (constant-coefficient).
//  Derivatives follow the sign of the constant value (chain-rule of |x|).
//  Port verbatim — abs is taken on the scalar value only; higher-order
//  coefficients inherit the sign of CNST (per T-04-01-01 threat model note).
// ---------------------------------------------------------------------------

/// Compute `|z|` for a CTaylor: negate all coefficients if CNST < 0.
///
/// Port of `abs()` usage in `brx.cpp:112-114`. Per CONTEXT T-04-01-01,
/// the `abs()` applies at the constant-coefficient slot; derivatives of
/// `|x|` are `sign(x) * d/dx(x)`, so we negate all slots when CNST < 0.
#[cube]
fn ctaylor_abs<F: Float>(z: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // Read CNST to determine sign.
    let sign_negative = z[0] < F::new(0.0);
    if sign_negative {
        #[unroll]
        for i in 0..size {
            out[i] = -z[i];
        }
    } else {
        #[unroll]
        for i in 0..size {
            out[i] = z[i];
        }
    }
}

/// Compute `ECopp = -0.8 * a * b * zab^2 * (1 - log(1+zab)/zab)`.
///
/// Port of `brx.cpp:115`.
#[cube]
fn brc_ecopp<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    zab: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // zab^2
    let mut zab2 = Array::<F>::new(size);
    ctaylor_mul::<F>(zab, zab, &mut zab2, n);

    // 1 + zab
    let mut one_plus_zab = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus_zab[i] = zab[i];
    }
    one_plus_zab[0] = one_plus_zab[0] + F::new(1.0);

    // log(1 + zab)
    let mut log1pz = Array::<F>::new(size);
    ctaylor_log::<F>(&one_plus_zab, &mut log1pz, n);

    // log(1+zab)/zab
    let mut inv_zab = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(zab, &mut inv_zab, n);

    let mut log1pz_div_zab_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&log1pz, &inv_zab, &mut log1pz_div_zab_raw, n);

    // 1 - log(1+zab)/zab
    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = -log1pz_div_zab_raw[i];
    }
    bracket[0] = bracket[0] + F::new(1.0);

    // a * b * zab^2 * bracket
    let mut ab_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(a, b, &mut ab_raw, n);

    let mut ab_zab2_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ab_raw, &zab2, &mut ab_zab2_raw, n);

    let mut ab_zab2_bkt_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ab_zab2_raw, &bracket, &mut ab_zab2_bkt_raw, n);

    // ECopp = -0.8 * a*b*zab^2*bracket
    ctaylor_scalar_mul::<F>(&ab_zab2_bkt_raw, F::cast_from(-0.8_f64), out, n);
}

/// Compute same-spin EC term: `-0.01 * rho * kinetic_factor * zs^4 * (1 - 2/zs * log(1+zs/2))`.
///
/// Port of `brx.cpp:116-119` for both ECaa and ECbb.
/// `rho` = d.a or d.b; `tau` = d.taua or d.taub; `g` = d.gaa or d.gbb;
/// `jp` = d.jpaa or d.jpbb; `zs` = zaa or zbb.
#[cube]
fn brc_ec_same_spin<F: Float>(
    rho: &Array<F>,
    tau: &Array<F>,
    g: &Array<F>,
    jp: &Array<F>,
    zs: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // kinetic_factor = 2*tau - (0.25*g + jp)/rho
    let mut g025 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(g, F::cast_from(0.25_f64), &mut g025, n);

    let mut g025_plus_jp = Array::<F>::new(size);
    ctaylor_add::<F>(&g025, jp, &mut g025_plus_jp, n);

    let mut inv_rho = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(rho, &mut inv_rho, n);

    let mut frac_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&g025_plus_jp, &inv_rho, &mut frac_raw, n);

    let mut tau2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(tau, F::cast_from(2.0_f64), &mut tau2, n);

    let mut kinfactor = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        kinfactor[i] = tau2[i] - frac_raw[i];
    }

    // zs^4
    let mut zs4 = Array::<F>::new(size);
    ctaylor_pow::<F>(zs, F::cast_from(4.0_f64), &mut zs4, n);

    // 1 - 2/zs * log(1 + zs/2)
    // = 1 - (2/zs) * log(1 + zs/2)
    let mut zs_half = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(zs, F::cast_from(0.5_f64), &mut zs_half, n);

    let mut one_plus_zs_half = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus_zs_half[i] = zs_half[i];
    }
    one_plus_zs_half[0] = one_plus_zs_half[0] + F::new(1.0);

    let mut log1_zs_half = Array::<F>::new(size);
    ctaylor_log::<F>(&one_plus_zs_half, &mut log1_zs_half, n);

    let mut inv_zs = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(zs, &mut inv_zs, n);

    let mut two_inv_zs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_zs, F::cast_from(2.0_f64), &mut two_inv_zs, n);

    let mut two_inv_zs_log_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&two_inv_zs, &log1_zs_half, &mut two_inv_zs_log_raw, n);

    let mut bracket = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket[i] = -two_inv_zs_log_raw[i];
    }
    bracket[0] = bracket[0] + F::new(1.0);

    // rho * kinfactor * zs4 * bracket
    let mut rho_kin_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(rho, &kinfactor, &mut rho_kin_raw, n);

    let mut rho_kin_zs4_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&rho_kin_raw, &zs4, &mut rho_kin_zs4_raw, n);

    let mut rho_kin_zs4_bkt_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&rho_kin_zs4_raw, &bracket, &mut rho_kin_zs4_bkt_raw, n);

    // EC_same = -0.01 * rho_kin_zs4_bkt
    ctaylor_scalar_mul::<F>(&rho_kin_zs4_bkt_raw, F::cast_from(-0.01_f64), out, n);
}

#[cube(launch_unchecked)]
pub fn brc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // Build 2*taua and 2*taub.
    let mut taua2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taua, F::cast_from(2.0_f64), &mut taua2, n);

    let mut taub2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taub, F::cast_from(2.0_f64), &mut taub2, n);

    let mut ux_a = Array::<F>::new(size);
    polarized::<F>(&d.a, &d.gaa, &d.lapa, &taua2, &d.jpaa, &mut ux_a, n);

    let mut ux_b = Array::<F>::new(size);
    polarized::<F>(&d.b, &d.gbb, &d.lapb, &taub2, &d.jpbb, &mut ux_b, n);

    // inv_UXa = 1/UXa, inv_UXb = 1/UXb
    let mut inv_uxa = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&ux_a, &mut inv_uxa, n);

    let mut inv_uxb = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&ux_b, &mut inv_uxb, n);

    // zaa = |caa * (2 / UXa)| = |caa * 2 * inv_UXa|
    // const caa = 0.88, cab = 0.63
    let mut zaa_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_uxa, F::cast_from(0.88_f64 * 2.0_f64), &mut zaa_raw, n);
    let mut zaa = Array::<F>::new(size);
    ctaylor_abs::<F>(&zaa_raw, &mut zaa, n);

    let mut zbb_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_uxb, F::cast_from(0.88_f64 * 2.0_f64), &mut zbb_raw, n);
    let mut zbb = Array::<F>::new(size);
    ctaylor_abs::<F>(&zbb_raw, &mut zbb, n);

    // zab = |cab * (1/UXa + 1/UXb)|
    let mut inv_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&inv_uxa, &inv_uxb, &mut inv_sum, n);
    let mut zab_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_sum, F::cast_from(0.63_f64), &mut zab_raw, n);
    let mut zab = Array::<F>::new(size);
    ctaylor_abs::<F>(&zab_raw, &mut zab, n);

    let mut ecopp = Array::<F>::new(size);
    brc_ecopp::<F>(&d.a, &d.b, &zab, &mut ecopp, n);

    let mut ecaa = Array::<F>::new(size);
    brc_ec_same_spin::<F>(&d.a, &d.taua, &d.gaa, &d.jpaa, &zaa, &mut ecaa, n);

    let mut ecbb = Array::<F>::new(size);
    brc_ec_same_spin::<F>(&d.b, &d.taub, &d.gbb, &d.jpbb, &zbb, &mut ecbb, n);

    let mut ec_aa_bb = Array::<F>::new(size);
    ctaylor_add::<F>(&ecaa, &ecbb, &mut ec_aa_bb, n);

    ctaylor_add::<F>(&ecopp, &ec_aa_bb, out, n);
}

// ---------------------------------------------------------------------------
//  BRXC — XC_BRXC (id=12).  Port of brx.cpp:123-136.
//
//  Same as BRC plus the exchange: 0.5*(UXa*d.a + UXb*d.b) + ECopp + ECaa + ECbb.
// ---------------------------------------------------------------------------

#[cube(launch_unchecked)]
pub fn brxc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut taua2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taua, F::cast_from(2.0_f64), &mut taua2, n);

    let mut taub2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.taub, F::cast_from(2.0_f64), &mut taub2, n);

    let mut ux_a = Array::<F>::new(size);
    polarized::<F>(&d.a, &d.gaa, &d.lapa, &taua2, &d.jpaa, &mut ux_a, n);

    let mut ux_b = Array::<F>::new(size);
    polarized::<F>(&d.b, &d.gbb, &d.lapb, &taub2, &d.jpbb, &mut ux_b, n);

    let mut inv_uxa = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&ux_a, &mut inv_uxa, n);

    let mut inv_uxb = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&ux_b, &mut inv_uxb, n);

    let mut zaa_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_uxa, F::cast_from(0.88_f64 * 2.0_f64), &mut zaa_raw, n);
    let mut zaa = Array::<F>::new(size);
    ctaylor_abs::<F>(&zaa_raw, &mut zaa, n);

    let mut zbb_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_uxb, F::cast_from(0.88_f64 * 2.0_f64), &mut zbb_raw, n);
    let mut zbb = Array::<F>::new(size);
    ctaylor_abs::<F>(&zbb_raw, &mut zbb, n);

    let mut inv_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&inv_uxa, &inv_uxb, &mut inv_sum, n);
    let mut zab_raw = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_sum, F::cast_from(0.63_f64), &mut zab_raw, n);
    let mut zab = Array::<F>::new(size);
    ctaylor_abs::<F>(&zab_raw, &mut zab, n);

    let mut ecopp = Array::<F>::new(size);
    brc_ecopp::<F>(&d.a, &d.b, &zab, &mut ecopp, n);

    let mut ecaa = Array::<F>::new(size);
    brc_ec_same_spin::<F>(&d.a, &d.taua, &d.gaa, &d.jpaa, &zaa, &mut ecaa, n);

    let mut ecbb = Array::<F>::new(size);
    brc_ec_same_spin::<F>(&d.b, &d.taub, &d.gbb, &d.jpbb, &zbb, &mut ecbb, n);

    // exchange term: 0.5 * (UXa*a + UXb*b)
    let mut uxa_a_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ux_a, &d.a, &mut uxa_a_raw, n);

    let mut uxb_b_raw = Array::<F>::new(size);
    ctaylor_mul::<F>(&ux_b, &d.b, &mut uxb_b_raw, n);

    let mut xsum_raw = Array::<F>::new(size);
    ctaylor_add::<F>(&uxa_a_raw, &uxb_b_raw, &mut xsum_raw, n);

    let mut ex = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&xsum_raw, F::cast_from(0.5_f64), &mut ex, n);

    // out = ex + ecopp + ecaa + ecbb
    let mut ec_aa_bb = Array::<F>::new(size);
    ctaylor_add::<F>(&ecaa, &ecbb, &mut ec_aa_bb, n);

    let mut ec_all = Array::<F>::new(size);
    ctaylor_add::<F>(&ecopp, &ec_aa_bb, &mut ec_all, n);

    ctaylor_add::<F>(&ex, &ec_all, out, n);
}
