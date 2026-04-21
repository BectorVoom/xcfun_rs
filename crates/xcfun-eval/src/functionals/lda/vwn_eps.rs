//! Shared VWN3/VWN5 epsilon helpers. 1:1 port of `xcfun-master/src/functionals/vwn.hpp`.
//!
//! VWN parametrisation: paramagnetic / ferromagnetic / spin-stiffness fits to
//! the homogeneous electron gas correlation energy, interpolated by ζ.
//! See: S. H. Vosko, L. Wilk, M. Nusair, Can. J. Phys. 58, 1200 (1980).
//!
//! # Source
//! - `xcfun-master/src/functionals/vwn.hpp:19-91`
//!
//! # vwn_f(s, p) formula (vwn.hpp:48-52)
//!
//! ```cpp
//! vwn_f(s, p) = 0.5 * p[1] * (2*log(s) + vwn_a*log(vwn_x(s,p)) - vwn_b*log(vwn_y(s,p))
//!                             + vwn_c*atan(vwn_z(s,p)))
//! vwn_x(s, p) = s*s + p[2]*s + p[3]
//! vwn_y(s, p) = s - p[0]
//! vwn_z(s, p) = sqrt(4*p[3] - p[2]*p[2]) / (2*s + p[2])
//! vwn_a(p)    = p[0]*p[2] / (p[0]^2 + p[0]*p[2] + p[3]) - 1
//! vwn_b(p)    = 2*vwn_a(p) + 2
//! vwn_c(p)    = 2*p[2]*(1/sqrt(4*p[3] - p[2]^2)
//!               - p[0] / ((p[0]^2 + p[0]*p[2] + p[3]) * sqrt(4*p[3] - p[2]^2) / (p[2] + 2*p[0])))
//! ```
//!
//! # Port strategy
//!
//! Each vwn_f(para/ferro/inter) call in C++ has a distinct f64 parameter set; the
//! helper `vwn_a/b/c` values are derived from p[] and are f64 constants at C++
//! compile time. In Rust, we precompute them as Rust `const` f64 values and
//! pass into `F::new(f32 cast)` inside a dedicated helper per parameter set.
//! This preserves the C++ operation order inside each `vwn_f_*` helper while
//! avoiding comptime f32 scalar-parameter plumbing that cubecl 0.10-pre.3's
//! `#[cube]` signature does not widely exercise.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{
    ctaylor_atan, ctaylor_log, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt,
};

use crate::density_vars::DensVarsDev;

// ---------------------------------------------------------------------------
// VWN5 parameter sets (vwn.hpp:55-60 — #ifndef XCFUN_VWN5_REF branch).
// The second element (p1) is multiplied by 2 relative to Molpro manual per the
// `ulfek:` comment at vwn.hpp:55.
//
// All derived values (vwn_a, vwn_b, vwn_c, z_num) precomputed in f64 and cast
// via `F::cast_from` at kernel-time for 1e-11 tier-1 parity. f32 literals
// introduce ~1e-7 rel-error, which violates the 1e-11 threshold.
// ---------------------------------------------------------------------------
// VWN5 paramagnetic: p = [-0.10498, 0.0621814, 3.72744, 12.9352]
const VWN5_PARA_P0: f64 = -0.10498_f64;
const VWN5_PARA_P1: f64 = 0.0621814_f64;
const VWN5_PARA_P2: f64 = 3.72744_f64;
const VWN5_PARA_P3: f64 = 12.9352_f64;
const VWN5_PARA_VWN_A: f64 = -1.0311676086789439_f64;
const VWN5_PARA_VWN_B: f64 = -0.06233521735788772_f64;
const VWN5_PARA_VWN_C: f64 = 1.2474243062431214_f64;
const VWN5_PARA_Z_NUM: f64 = 6.15199081975908_f64;

// VWN5 ferromagnetic: p = [-0.325, 0.0310907, 7.06042, 18.0578]
const VWN5_FERRO_P0: f64 = -0.325_f64;
const VWN5_FERRO_P1: f64 = 0.0310907_f64;
const VWN5_FERRO_P2: f64 = 7.06042_f64;
const VWN5_FERRO_P3: f64 = 18.0578_f64;
const VWN5_FERRO_VWN_A: f64 = -1.1446006101852073_f64;
const VWN5_FERRO_VWN_B: f64 = -0.2892012203704146_f64;
const VWN5_FERRO_VWN_C: f64 = 3.3766620352569046_f64;
const VWN5_FERRO_Z_NUM: f64 = 4.730926909560114_f64;

// VWN5 spin-interpolation: p = [-0.0047584, -pow(3π²,-1), 1.13107, 13.0045]
// p1 = -1/(3π²) = -0.03377372788077926
const VWN5_INTER_P0: f64 = -0.0047584_f64;
const VWN5_INTER_P1: f64 = -0.03377372788077926_f64;
const VWN5_INTER_P2: f64 = 1.13107_f64;
const VWN5_INTER_P3: f64 = 13.0045_f64;
const VWN5_INTER_VWN_A: f64 = -1.000414033794282_f64;
const VWN5_INTER_VWN_B: f64 = -0.0008280675885639077_f64;
const VWN5_INTER_VWN_C: f64 = 0.31770800474394145_f64;
const VWN5_INTER_Z_NUM: f64 = 7.123108917818118_f64;

// VWN3 paramagnetic: p = [-0.4092860, 0.0621814, 13.0720, 42.7198] (vwn.hpp:82)
const VWN3_PARA_P0: f64 = -0.4092860_f64;
const VWN3_PARA_P1: f64 = 0.0621814_f64;
const VWN3_PARA_P2: f64 = 13.0720_f64;
const VWN3_PARA_P3: f64 = 42.7198_f64;
const VWN3_PARA_VWN_A: f64 = -1.142530524167984_f64;
const VWN3_PARA_VWN_B: f64 = -0.2850610483359679_f64;
const VWN3_PARA_VWN_C: f64 = 660.0678961137954_f64;
const VWN3_PARA_Z_NUM: f64 = 0.0448998886415768_f64;

// VWN3 ferromagnetic: p = [-0.7432940, 0.0310907, 20.1231, 101.578] (vwn.hpp:83)
const VWN3_FERRO_P0: f64 = -0.7432940_f64;
const VWN3_FERRO_P1: f64 = 0.0310907_f64;
const VWN3_FERRO_P2: f64 = 20.1231_f64;
const VWN3_FERRO_P3: f64 = 101.578_f64;
const VWN3_FERRO_VWN_A: f64 = -1.1715824994145076_f64;
const VWN3_FERRO_VWN_B: f64 = -0.3431649988290153_f64;
const VWN3_FERRO_VWN_C: f64 = 39.80727547405608_f64;
const VWN3_FERRO_Z_NUM: f64 = 1.1716852777089715_f64;

// `1.92366105093154` from vwn.hpp:71 and :86 — constant `(2^(1/3) - 1)^(-1/2)`
// used in both vwn3_eps and vwn5_eps as the prefactor of (ufunc(zeta, 4/3) - 2).
const VWN_ZETA_FACTOR: f64 = 1.92366105093154_f64;

// 9/4 * (2^(1/3) - 1) — used in vwn5 spin-interpolation formula (vwn.hpp:76).
// 9/4 * (2^(1/3) - 1) = 2.25 * 0.2599210498948732 = 0.5848223622134647
const VWN5_INTER_FACTOR: f64 = 0.5848223622134647_f64;

// ---------------------------------------------------------------------------
//  Internal: vwn_f_inline — operation-order port of vwn.hpp:48-52 for one
//  parameter set. Inlined per parameter set via a wrapper below.
// ---------------------------------------------------------------------------

/// Shared operation-order body for `vwn_f(s, p)`. The parameters are passed
/// as F::new(f32) runtime constants; each caller supplies the correct set.
///
/// C++ (vwn.hpp:48-52):
/// ```cpp
/// return 0.5 * p[1] * (2 * log(s)
///                      + vwn_a(p) * log(vwn_x(s, p))
///                      - vwn_b(p) * log(vwn_y(s, p))
///                      + vwn_c(p) * atan(vwn_z(s, p)));
/// ```
#[cube]
#[allow(clippy::too_many_arguments)]
fn vwn_f_body<F: Float>(
    s: &Array<F>,
    p0: F,
    p2: F,
    p3: F,
    vwn_a: F,
    vwn_b: F,
    vwn_c: F,
    z_num: F,
    half_p1: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // vwn_x(s, p) = s*s + p[2]*s + p[3]
    //   step 1: s2 = s * s
    //   step 2: p2_s = p[2] * s  (scalar_mul by p2)
    //   step 3: x = s2 + p2_s
    //   step 4: x = x + p[3]     (add p3 to constant term via ctaylor_scalar_add-style: we use add with a p3-scalar)
    let mut s2 = Array::<F>::new(size);
    ctaylor_mul::<F>(s, s, &mut s2, n);
    let mut p2_s = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(s, p2, &mut p2_s, n);
    let mut x_tmp = Array::<F>::new(size);
    ctaylor_add::<F>(&s2, &p2_s, &mut x_tmp, n);
    // x = x_tmp + p3  (add p3 only to CNST coefficient — the rest of x_tmp is
    // unchanged; emulate via a constant-only CTaylor that we allocate here).
    let mut p3_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut p3_const, n);
    p3_const[0] = p3;
    let mut x = Array::<F>::new(size);
    ctaylor_add::<F>(&x_tmp, &p3_const, &mut x, n);

    // vwn_y(s, p) = s - p[0]
    //   y = s - p0  (subtract p0 from CNST)
    let mut p0_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut p0_const, n);
    p0_const[0] = p0;
    let mut y = Array::<F>::new(size);
    ctaylor_sub::<F>(s, &p0_const, &mut y, n);

    // vwn_z(s, p) = z_num / (2*s + p[2])
    //   step 1: two_s = 2 * s
    //   step 2: denom = two_s + p[2]
    //   step 3: inv_denom = 1 / denom
    //   step 4: z = z_num * inv_denom
    let mut two_s = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(s, F::new(2.0), &mut two_s, n);
    let mut p2_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut p2_const, n);
    p2_const[0] = p2;
    let mut denom_z = Array::<F>::new(size);
    ctaylor_add::<F>(&two_s, &p2_const, &mut denom_z, n);
    let mut inv_denom_z = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom_z, &mut inv_denom_z, n);
    let mut z = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_denom_z, z_num, &mut z, n);

    // log_s = log(s)
    let mut log_s = Array::<F>::new(size);
    ctaylor_log::<F>(s, &mut log_s, n);
    // log_x = log(x)
    let mut log_x = Array::<F>::new(size);
    ctaylor_log::<F>(&x, &mut log_x, n);
    // log_y = log(y)
    let mut log_y = Array::<F>::new(size);
    ctaylor_log::<F>(&y, &mut log_y, n);
    // atan_z = atan(z)
    let mut atan_z = Array::<F>::new(size);
    ctaylor_atan::<F>(&z, &mut atan_z, n);

    // bracket terms (match C++ left-to-right order):
    //   term1 = 2 * log(s)
    //   term2 = vwn_a * log(x)
    //   term3 = -vwn_b * log(y)
    //   term4 = vwn_c * atan(z)
    let mut term1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&log_s, F::new(2.0), &mut term1, n);
    let mut term2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&log_x, vwn_a, &mut term2, n);
    let mut term3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&log_y, vwn_b, &mut term3, n);
    let mut term4 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&atan_z, vwn_c, &mut term4, n);

    // sum = term1 + term2 - term3 + term4  (C++ left-to-right associativity)
    let mut sum12 = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &term2, &mut sum12, n);
    let mut sum123 = Array::<F>::new(size);
    ctaylor_sub::<F>(&sum12, &term3, &mut sum123, n);
    let mut bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&sum123, &term4, &mut bracket, n);

    // out = half_p1 * bracket   (half_p1 = 0.5 * p[1])
    ctaylor_scalar_mul::<F>(&bracket, half_p1, out, n);
}

// ---------------------------------------------------------------------------
//  Per-parameter-set wrappers — each fills F::new(f32) constants for vwn_f_body.
// ---------------------------------------------------------------------------

#[cube]
fn vwn_f_vwn5_para<F: Float>(s: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    vwn_f_body::<F>(
        s,
        F::cast_from(VWN5_PARA_P0),
        F::cast_from(VWN5_PARA_P2),
        F::cast_from(VWN5_PARA_P3),
        F::cast_from(VWN5_PARA_VWN_A),
        F::cast_from(VWN5_PARA_VWN_B),
        F::cast_from(VWN5_PARA_VWN_C),
        F::cast_from(VWN5_PARA_Z_NUM),
        F::cast_from(VWN5_PARA_P1 * 0.5_f64),
        out,
        n,
    );
}

#[cube]
fn vwn_f_vwn5_ferro<F: Float>(s: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    vwn_f_body::<F>(
        s,
        F::cast_from(VWN5_FERRO_P0),
        F::cast_from(VWN5_FERRO_P2),
        F::cast_from(VWN5_FERRO_P3),
        F::cast_from(VWN5_FERRO_VWN_A),
        F::cast_from(VWN5_FERRO_VWN_B),
        F::cast_from(VWN5_FERRO_VWN_C),
        F::cast_from(VWN5_FERRO_Z_NUM),
        F::cast_from(VWN5_FERRO_P1 * 0.5_f64),
        out,
        n,
    );
}

#[cube]
fn vwn_f_vwn5_inter<F: Float>(s: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    vwn_f_body::<F>(
        s,
        F::cast_from(VWN5_INTER_P0),
        F::cast_from(VWN5_INTER_P2),
        F::cast_from(VWN5_INTER_P3),
        F::cast_from(VWN5_INTER_VWN_A),
        F::cast_from(VWN5_INTER_VWN_B),
        F::cast_from(VWN5_INTER_VWN_C),
        F::cast_from(VWN5_INTER_Z_NUM),
        F::cast_from(VWN5_INTER_P1 * 0.5_f64),
        out,
        n,
    );
}

#[cube]
fn vwn_f_vwn3_para<F: Float>(s: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    vwn_f_body::<F>(
        s,
        F::cast_from(VWN3_PARA_P0),
        F::cast_from(VWN3_PARA_P2),
        F::cast_from(VWN3_PARA_P3),
        F::cast_from(VWN3_PARA_VWN_A),
        F::cast_from(VWN3_PARA_VWN_B),
        F::cast_from(VWN3_PARA_VWN_C),
        F::cast_from(VWN3_PARA_Z_NUM),
        F::cast_from(VWN3_PARA_P1 * 0.5_f64),
        out,
        n,
    );
}

#[cube]
fn vwn_f_vwn3_ferro<F: Float>(s: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    vwn_f_body::<F>(
        s,
        F::cast_from(VWN3_FERRO_P0),
        F::cast_from(VWN3_FERRO_P2),
        F::cast_from(VWN3_FERRO_P3),
        F::cast_from(VWN3_FERRO_VWN_A),
        F::cast_from(VWN3_FERRO_VWN_B),
        F::cast_from(VWN3_FERRO_VWN_C),
        F::cast_from(VWN3_FERRO_Z_NUM),
        F::cast_from(VWN3_FERRO_P1 * 0.5_f64),
        out,
        n,
    );
}

// ---------------------------------------------------------------------------
//  ufunc(x, a) = (1+x)^a + (1-x)^a  — port of specmath.hpp:35-37.
// ---------------------------------------------------------------------------

/// `ufunc(x, a) = (1+x)^a + (1-x)^a`. Used by vwn3_eps/vwn5_eps with a = 4/3.
///
/// Port target: `xcfun-master/src/specmath.hpp:35-37`.
///
/// Operation order:
///   1. one_plus  = x + 1  (add scalar 1 to CNST via ctaylor_add with a 1-valued CTaylor)
///   2. one_minus = 1 - x  (sub with 1-valued CTaylor)
///   3. pow_plus  = (1+x)^a
///   4. pow_minus = (1-x)^a
///   5. out       = pow_plus + pow_minus
#[cube]
fn ufunc_4_3<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);

    let mut one_plus = Array::<F>::new(size);
    ctaylor_add::<F>(x, &one_const, &mut one_plus, n);
    let mut one_minus = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, x, &mut one_minus, n);

    let four_thirds = F::cast_from(4.0_f64 / 3.0_f64);
    let mut pow_plus = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_plus, four_thirds, &mut pow_plus, n);
    let mut pow_minus = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_minus, four_thirds, &mut pow_minus, n);

    ctaylor_add::<F>(&pow_plus, &pow_minus, out, n);
}

// ---------------------------------------------------------------------------
//  pow_4(x) = x^4 (used for zeta^4 in vwn5_eps). Matches `pow(d.zeta, 4)`.
// ---------------------------------------------------------------------------

#[cube]
fn pow4<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut x2 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, x, &mut x2, n);
    ctaylor_mul::<F>(&x2, &x2, out, n);
}

// ---------------------------------------------------------------------------
//  vwn3_eps — port of vwn.hpp:80-90.
// ---------------------------------------------------------------------------

/// VWN3 correlation energy per electron.
///
/// C++ source (vwn.hpp:80-90):
/// ```cpp
/// const parameter para[]  = {-0.4092860, 0.0621814, 13.0720, 42.7198};
/// const parameter ferro[] = {-0.7432940, 0.0310907, 20.1231, 101.578};
/// num s = sqrt(d.r_s);
/// num g = 1.92366105093154 * (ufunc(d.zeta, 4.0/3.0) - 2);
/// num dd = g * (vwn_f(s, ferro) - vwn_f(s, para));
/// return (vwn_f(s, para) + dd);
/// ```
#[cube]
pub fn vwn3_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // s = sqrt(r_s)
    let mut s = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&d.r_s, &mut s, n);

    // ufunc(zeta, 4/3) - 2  (the "- 2" is scalar-subtract from CNST)
    let mut ufz = Array::<F>::new(size);
    ufunc_4_3::<F>(&d.zeta, &mut ufz, n);
    let mut two_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut two_const, n);
    two_const[0] = F::new(2.0);
    let mut ufz_m2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&ufz, &two_const, &mut ufz_m2, n);

    // g = 1.92366105093154 * (ufunc(zeta, 4/3) - 2)
    let mut g = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ufz_m2, F::cast_from(VWN_ZETA_FACTOR), &mut g, n);

    // vwn_f(s, para) and vwn_f(s, ferro)
    let mut f_para = Array::<F>::new(size);
    vwn_f_vwn3_para::<F>(&s, &mut f_para, n);
    let mut f_ferro = Array::<F>::new(size);
    vwn_f_vwn3_ferro::<F>(&s, &mut f_ferro, n);

    // dd = g * (f_ferro - f_para)
    let mut f_diff = Array::<F>::new(size);
    ctaylor_sub::<F>(&f_ferro, &f_para, &mut f_diff, n);
    let mut dd = Array::<F>::new(size);
    ctaylor_mul::<F>(&g, &f_diff, &mut dd, n);

    // out = f_para + dd
    ctaylor_add::<F>(&f_para, &dd, out, n);
}

// ---------------------------------------------------------------------------
//  vwn5_eps — port of vwn.hpp:54-78.
// ---------------------------------------------------------------------------

/// VWN5 correlation energy per electron.
///
/// C++ source (vwn.hpp:54-78):
/// ```cpp
/// const parameter para[]  = {-0.10498, 0.0621814, 3.72744, 12.9352};
/// const parameter ferro[] = {-0.325, 0.0310907, 7.06042, 18.0578};
/// const parameter inter[] = {-0.0047584, -pow(3*M_PI*M_PI, -1.0), 1.13107, 13.0045};
/// num s     = sqrt(d.r_s);
/// num g     = 1.92366105093154 * (ufunc(d.zeta, 4/3) - 2);
/// num zeta4 = pow(d.zeta, 4);
/// num dd = g * ((vwn_f(s, ferro) - vwn_f(s, para)) * zeta4 +
///               vwn_f(s, inter) * (1 - zeta4) * (9/4 * (2^(1/3) - 1)));
/// return (vwn_f(s, para) + dd);
/// ```
#[cube]
pub fn vwn5_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // s = sqrt(r_s)
    let mut s = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&d.r_s, &mut s, n);

    // g = 1.92366105093154 * (ufunc(zeta, 4/3) - 2)
    let mut ufz = Array::<F>::new(size);
    ufunc_4_3::<F>(&d.zeta, &mut ufz, n);
    let mut two_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut two_const, n);
    two_const[0] = F::new(2.0);
    let mut ufz_m2 = Array::<F>::new(size);
    ctaylor_sub::<F>(&ufz, &two_const, &mut ufz_m2, n);
    let mut g = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ufz_m2, F::cast_from(VWN_ZETA_FACTOR), &mut g, n);

    // zeta4 = pow(zeta, 4) — exact x^4 via pow4 helper (avoids series expansion drift)
    let mut zeta4 = Array::<F>::new(size);
    pow4::<F>(&d.zeta, &mut zeta4, n);

    // vwn_f values
    let mut f_para = Array::<F>::new(size);
    vwn_f_vwn5_para::<F>(&s, &mut f_para, n);
    let mut f_ferro = Array::<F>::new(size);
    vwn_f_vwn5_ferro::<F>(&s, &mut f_ferro, n);
    let mut f_inter = Array::<F>::new(size);
    vwn_f_vwn5_inter::<F>(&s, &mut f_inter, n);

    // (f_ferro - f_para) * zeta4
    let mut f_diff = Array::<F>::new(size);
    ctaylor_sub::<F>(&f_ferro, &f_para, &mut f_diff, n);
    let mut term_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&f_diff, &zeta4, &mut term_a, n);

    // (1 - zeta4)
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut one_m_zeta4 = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, &zeta4, &mut one_m_zeta4, n);

    // f_inter * (1 - zeta4) * (9/4 * (2^(1/3) - 1))
    let mut finter_one_m = Array::<F>::new(size);
    ctaylor_mul::<F>(&f_inter, &one_m_zeta4, &mut finter_one_m, n);
    let mut term_b = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&finter_one_m, F::cast_from(VWN5_INTER_FACTOR), &mut term_b, n);

    // bracket = term_a + term_b
    let mut bracket = Array::<F>::new(size);
    ctaylor_add::<F>(&term_a, &term_b, &mut bracket, n);

    // dd = g * bracket
    let mut dd = Array::<F>::new(size);
    ctaylor_mul::<F>(&g, &bracket, &mut dd, n);

    // out = f_para + dd
    ctaylor_add::<F>(&f_para, &dd, out, n);
}
