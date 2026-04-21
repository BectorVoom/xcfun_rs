//! PW92 epsilon helpers — homogeneous electron gas correlation energy.
//! 1:1 port of `xcfun-master/src/functionals/pw92eps.hpp` (accurate branch only).
//!
//! # Source
//! - `xcfun-master/src/functionals/pw92eps.hpp:19-61`
//!
//! # Phase 2 PW92C Legacy Constants Decision (RESEARCH §"PW92C Legacy Constants")
//!
//! `xcfun-master/src/config.hpp` ships `XCFUN_REF_PW92C` UNDEFINED by default.
//! Phase 2 matches that and ships the *accurate* constants directly:
//! - `omega(z)` denominator: `2 * pow(2, 1/3) - 2 = 0.5198421...` (NOT the legacy `0.5198421` literal)
//! - `pw92eps` prefactor `c`: `8.0 / (9.0 * (2 * pow(2, 1/3) - 2)) = 1.70992093...`
//!   (NOT the legacy `1.709921` literal)
//!
//! No Cargo feature flag in Phase 2. The legacy-constants flag is a v2 forward-compat
//! option (PERF/INT requirements), not a Phase 2 requirement.
//!
//! Pitfall P12 escalation path: if tier-2 PW92C parity fails > 1e-12 on the bulk grid,
//! escalate `PLANNING INCONCLUSIVE` per Phase 1 D-03 + CONTEXT D-19.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_log, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use crate::density_vars::DensVarsDev;

// ---------------------------------------------------------------------------
// Accurate PW92 constants (XCFUN_REF_PW92C undefined branch, the default).
// ---------------------------------------------------------------------------

/// Accurate PW92 omega denominator. `2 * pow(2.0_f64, 1.0/3.0) - 2.0` evaluated
/// at f64 precision = 0.5198420997897463. The legacy reference value is 0.5198421.
const PW92_OMEGA_DENOM_F64: f64 = 0.5198420997897463_f64;

/// Accurate PW92 prefactor `c = 8 / (9 * omega_denom) = 1.7099209341613654`.
/// The legacy reference value is 1.709921.
const PW92_C_F64: f64 = 1.7099209341613654_f64;

// PW92 PARAMS table (`pw92eps.hpp:41-44`) — paramagnetic / ferromagnetic / spin-stiffness.
// Each row: [A, alpha, beta1, beta2, beta3, beta4, p]. `eopt` reads indices 0..=5; the
// 7th entry (p=1.0) is unused here (legacy generic-formula artefact).
//
// Stored as f64 and cast via F::cast_from at kernel-time for 1e-11 tier-1 parity.
const PW92_PARA_T0: f64 = 0.03109070_f64;
const PW92_PARA_T1: f64 = 0.21370_f64;
const PW92_PARA_T2: f64 = 7.59570_f64;
const PW92_PARA_T3: f64 = 3.5876_f64;
const PW92_PARA_T4: f64 = 1.63820_f64;
const PW92_PARA_T5: f64 = 0.49294_f64;

const PW92_FERRO_T0: f64 = 0.01554535_f64;
const PW92_FERRO_T1: f64 = 0.20548_f64;
const PW92_FERRO_T2: f64 = 14.1189_f64;
const PW92_FERRO_T3: f64 = 6.1977_f64;
const PW92_FERRO_T4: f64 = 3.36620_f64;
const PW92_FERRO_T5: f64 = 0.62517_f64;

const PW92_SS_T0: f64 = 0.01688690_f64;
const PW92_SS_T1: f64 = 0.11125_f64;
const PW92_SS_T2: f64 = 10.3570_f64;
const PW92_SS_T3: f64 = 3.6231_f64;
const PW92_SS_T4: f64 = 0.88026_f64;
const PW92_SS_T5: f64 = 0.49671_f64;

// ---------------------------------------------------------------------------
//  Inner helper: eopt(sqrtr, t) — port of pw92eps.hpp:20-25.
//
//  C++:
//    return -2 * t[0] * (1 + t[1] * sqrtr * sqrtr) *
//           log(1 + 0.5 / (t[0] *
//                          (sqrtr *
//                           (t[2] + sqrtr * (t[3] + sqrtr * (t[4] + t[5] * sqrtr))))));
//
//  Operation order (innermost first):
//    horner = t[5] * sqrtr
//    horner = horner + t[4]
//    horner = sqrtr * horner
//    horner = horner + t[3]
//    horner = sqrtr * horner
//    horner = horner + t[2]
//    horner = sqrtr * horner
//    horner = t[0] * horner           // parenthesisation matches C++: t[0]*(sqrtr*(...))
//    inv    = 1 / horner
//    arg    = 0.5 * inv
//    arg    = 1 + arg
//    log_v  = log(arg)
//    outer  = sqrtr * sqrtr           // sqrtr² = r_s (but the C++ expression is t[1]*sqrtr*sqrtr)
//    outer  = t[1] * outer            // t[1] * r_s
//    outer  = 1 + outer
//    outer  = outer * log_v
//    outer  = -2 * t[0] * outer
// ---------------------------------------------------------------------------

#[cube]
#[allow(clippy::too_many_arguments)]
fn eopt<F: Float>(
    sqrtr: &Array<F>,
    t0: F,
    t1: F,
    t2: F,
    t3: F,
    t4: F,
    t5: F,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Horner chain (innermost outwards): starts with `t[5]*sqrtr + t[4]`.
    let mut h = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(sqrtr, t5, &mut h, n);
    // h += t[4] (add scalar to CNST coef via 1-coef CTaylor)
    let mut add_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut add_const, n);
    add_const[0] = t4;
    let mut h1 = Array::<F>::new(size);
    ctaylor_add::<F>(&h, &add_const, &mut h1, n);

    // h2 = sqrtr * h1
    let mut h2 = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtr, &h1, &mut h2, n);
    // h2 += t[3]
    ctaylor_zero::<F>(&mut add_const, n);
    add_const[0] = t3;
    let mut h3 = Array::<F>::new(size);
    ctaylor_add::<F>(&h2, &add_const, &mut h3, n);

    // h4 = sqrtr * h3
    let mut h4 = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtr, &h3, &mut h4, n);
    // h4 += t[2]
    ctaylor_zero::<F>(&mut add_const, n);
    add_const[0] = t2;
    let mut h5 = Array::<F>::new(size);
    ctaylor_add::<F>(&h4, &add_const, &mut h5, n);

    // h6 = sqrtr * h5
    let mut h6 = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtr, &h5, &mut h6, n);

    // horner = t[0] * h6
    let mut horner = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&h6, t0, &mut horner, n);

    // inv = 1 / horner
    let mut inv = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&horner, &mut inv, n);

    // arg = 0.5 * inv
    let mut arg_inner = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv, F::new(0.5_f32), &mut arg_inner, n);

    // arg = 1 + arg_inner  (CNST += 1)
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut arg = Array::<F>::new(size);
    ctaylor_add::<F>(&arg_inner, &one_const, &mut arg, n);

    // log_v = log(arg)
    let mut log_v = Array::<F>::new(size);
    ctaylor_log::<F>(&arg, &mut log_v, n);

    // rs_like = sqrtr * sqrtr  (= r_s, but computed via the C++ op order)
    let mut rs_like = Array::<F>::new(size);
    ctaylor_mul::<F>(sqrtr, sqrtr, &mut rs_like, n);

    // t1_rs = t[1] * rs_like
    let mut t1_rs = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rs_like, t1, &mut t1_rs, n);

    // outer_sum = 1 + t1_rs
    let mut outer_sum = Array::<F>::new(size);
    ctaylor_add::<F>(&t1_rs, &one_const, &mut outer_sum, n);

    // outer = outer_sum * log_v
    let mut outer = Array::<F>::new(size);
    ctaylor_mul::<F>(&outer_sum, &log_v, &mut outer, n);

    // out = -2 * t[0] * outer
    //   step 1: t0_outer = t[0] * outer
    //   step 2: out = -2 * t0_outer
    let mut t0_outer = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&outer, t0, &mut t0_outer, n);
    ctaylor_scalar_mul::<F>(&t0_outer, F::new(-2.0), out, n);
}

// ---------------------------------------------------------------------------
//  omega(z) — port of pw92eps.hpp:32-39.
//
//  Accurate branch (XCFUN_REF_PW92C undefined):
//    return (ufunc(z, 4/3) - 2) / (2 * pow(2, 1/3) - 2);
//
//  where ufunc(z, a) = (1+z)^a + (1-z)^a (specmath.hpp:35).
// ---------------------------------------------------------------------------

#[cube]
fn omega_zeta<F: Float>(zeta: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // ufunc(zeta, 4/3) = (1+zeta)^(4/3) + (1-zeta)^(4/3)
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut one_plus = Array::<F>::new(size);
    ctaylor_add::<F>(zeta, &one_const, &mut one_plus, n);
    let mut one_minus = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, zeta, &mut one_minus, n);

    let four_thirds = F::cast_from(4.0_f64 / 3.0_f64);
    let mut pow_plus = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_plus, four_thirds, &mut pow_plus, n);
    let mut pow_minus = Array::<F>::new(size);
    ctaylor_pow::<F>(&one_minus, four_thirds, &mut pow_minus, n);

    let mut ufz = Array::<F>::new(size);
    ctaylor_add::<F>(&pow_plus, &pow_minus, &mut ufz, n);

    // (ufunc - 2)
    let mut two_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut two_const, n);
    two_const[0] = F::new(2.0);
    let mut numer = Array::<F>::new(size);
    ctaylor_sub::<F>(&ufz, &two_const, &mut numer, n);

    // divide by omega_denom = 2*2^(1/3) - 2  (multiply by 1/denom as a scalar)
    // Use 1/denom as a precomputed f32 to avoid a spurious reciprocal kernel on a
    // constant. 1/0.5198421 = 1.9236610509315363... → f32 = 1.923_661_f32.
    // 1/omega_denom = 1/(2*2^(1/3) - 2) = 1.9236610509315363 — f64 precision.
    let inv_denom = F::cast_from(1.9236610509315363_f64);
    let _ = PW92_OMEGA_DENOM_F64;
    ctaylor_scalar_mul::<F>(&numer, inv_denom, out, n);
}

// ---------------------------------------------------------------------------
//  zeta^4 helper — used twice in pw92eps.
// ---------------------------------------------------------------------------

#[cube]
fn pow4<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut x2 = Array::<F>::new(size);
    ctaylor_mul::<F>(x, x, &mut x2, n);
    ctaylor_mul::<F>(&x2, &x2, out, n);
}

// ---------------------------------------------------------------------------
//  pw92eps — port of pw92eps.hpp:48-61.
//
//  C++ (accurate branch):
//    num zeta4   = pow(d.zeta, 4);
//    num omegav  = omega(d.zeta);
//    num sqrtr   = sqrt(d.r_s);
//    num e0      = eopt(sqrtr, TUVWXYP[0]);               // paramagnetic
//    return e0 - eopt(sqrtr, TUVWXYP[2]) * omegav * (1 - zeta4) / c
//              + (eopt(sqrtr, TUVWXYP[1]) - e0) * omegav * zeta4;
// ---------------------------------------------------------------------------

#[cube]
pub fn pw92_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // zeta4 = zeta^4
    let mut zeta4 = Array::<F>::new(size);
    pow4::<F>(&d.zeta, &mut zeta4, n);

    // omegav = omega(d.zeta)
    let mut omegav = Array::<F>::new(size);
    omega_zeta::<F>(&d.zeta, &mut omegav, n);

    // sqrtr = sqrt(r_s)
    let mut sqrtr = Array::<F>::new(size);
    ctaylor_sqrt::<F>(&d.r_s, &mut sqrtr, n);

    // e0 = eopt(sqrtr, PARA)
    let mut e0 = Array::<F>::new(size);
    eopt::<F>(
        &sqrtr,
        F::cast_from(PW92_PARA_T0),
        F::cast_from(PW92_PARA_T1),
        F::cast_from(PW92_PARA_T2),
        F::cast_from(PW92_PARA_T3),
        F::cast_from(PW92_PARA_T4),
        F::cast_from(PW92_PARA_T5),
        &mut e0,
        n,
    );

    // e_ss = eopt(sqrtr, SS)   (alpha_c-like spin-stiffness)
    let mut e_ss = Array::<F>::new(size);
    eopt::<F>(
        &sqrtr,
        F::cast_from(PW92_SS_T0),
        F::cast_from(PW92_SS_T1),
        F::cast_from(PW92_SS_T2),
        F::cast_from(PW92_SS_T3),
        F::cast_from(PW92_SS_T4),
        F::cast_from(PW92_SS_T5),
        &mut e_ss,
        n,
    );

    // e_f = eopt(sqrtr, FERRO)
    let mut e_f = Array::<F>::new(size);
    eopt::<F>(
        &sqrtr,
        F::cast_from(PW92_FERRO_T0),
        F::cast_from(PW92_FERRO_T1),
        F::cast_from(PW92_FERRO_T2),
        F::cast_from(PW92_FERRO_T3),
        F::cast_from(PW92_FERRO_T4),
        F::cast_from(PW92_FERRO_T5),
        &mut e_f,
        n,
    );

    // term_a = e_ss * omegav
    let mut term_a0 = Array::<F>::new(size);
    ctaylor_mul::<F>(&e_ss, &omegav, &mut term_a0, n);
    // term_a = term_a0 * (1 - zeta4)
    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut one_m_zeta4 = Array::<F>::new(size);
    ctaylor_sub::<F>(&one_const, &zeta4, &mut one_m_zeta4, n);
    let mut term_a1 = Array::<F>::new(size);
    ctaylor_mul::<F>(&term_a0, &one_m_zeta4, &mut term_a1, n);
    // term_a = term_a1 / c  (multiply by 1/c as scalar)
    // 1 / 1.7099209341613654 = 0.584822362263464...
    // 1/c = 1/1.7099209341613654 = 0.5848223622134647 — f64 precision.
    let inv_c = F::cast_from(0.5848223622134647_f64);
    let _ = PW92_C_F64;
    let mut term_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&term_a1, inv_c, &mut term_a, n);

    // term_b = (e_f - e0) * omegav * zeta4
    let mut ef_m_e0 = Array::<F>::new(size);
    ctaylor_sub::<F>(&e_f, &e0, &mut ef_m_e0, n);
    let mut tb0 = Array::<F>::new(size);
    ctaylor_mul::<F>(&ef_m_e0, &omegav, &mut tb0, n);
    let mut term_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&tb0, &zeta4, &mut term_b, n);

    // out = e0 - term_a + term_b
    let mut tmp = Array::<F>::new(size);
    ctaylor_sub::<F>(&e0, &term_a, &mut tmp, n);
    ctaylor_add::<F>(&tmp, &term_b, out, n);
}
