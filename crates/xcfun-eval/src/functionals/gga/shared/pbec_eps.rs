//! PBE correlation epsilon helpers — 1:1 port target of
//! `xcfun-master/src/functionals/pbec.cpp:20-33` + `pbec_eps.hpp` shape.
//!
//! # Purpose
//! Inner helpers shared by PBEC / APBEC / SPBEC / PBEINTC / PBELOCC / ZVPBESOLC /
//! ZVPBEINTC / VWN_PBEC / PW91C — the β/γ correlation algebra around
//! `expm1(-ε/(γ·u³))` and the spin-polarisation factor φ(ζ).
//!
//! # Source
//! - `xcfun-master/src/functionals/pbec.cpp:20-24`  — `A(ε, u³)` template
//! - `xcfun-master/src/functionals/pbec.cpp:26-33`  — `H(d², ε, u³)` template
//! - `xcfun-master/src/functionals/pbec.cpp:35-38`  — `phi(d)` reorganised form
//!
//! # Critical port rule (Known Hazard §PBEC β/γ)
//! Preserve operation order around `expm1`: compute `expm1(-ε/(γ·u³))` **first**
//! as a `ctaylor_expm1` on the scaled argument, then `ctaylor_reciprocal`, then
//! `scalar_mul` by `β_gamma`. Do NOT algebraically simplify to
//! `β_gamma / (exp(...) - 1)` — that loses the x → 0 stable-bracket from D-05.
//!
//! # Status
//! All three helpers ship as **FULL BODIES** in plan 03-02 (W3 conversion).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use super::constants::{PBEC_BETA_GAMMA_F64, PBEC_GAMMA_F64};

/// `A_expm1_inner(ε, u³) = β_gamma / expm1(-ε / (γ·u³))` — the A term of the
/// PBEC H gradient correction. Port of `pbec.cpp:20-24`:
/// ```cpp
/// return param_beta_gamma / expm1(-eps / (param_gamma * u3));
/// ```
///
/// **FULL BODY** (Wave 2, plan 03-02 — W3 conversion). Operation order
/// (no algebraic simplification per Known Hazard §PBEC β/γ):
///   1. `gu3       = γ · u3`                         (scalar_mul)
///   2. `inv_gu3   = 1 / gu3`                        (ctaylor_reciprocal)
///   3. `arg       = -ε · inv_gu3`                   (ctaylor_mul + scalar_mul -1)
///   4. `em1       = expm1(arg)`                     (ctaylor_expm1, D-05)
///   5. `inv_em1   = 1 / em1`                        (ctaylor_reciprocal)
///   6. `out       = β_γ · inv_em1`                  (scalar_mul)
#[cube]
pub fn a_expm1_inner<F: Float>(
    eps: &Array<F>,
    u3: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // Step 1: gu3 = γ · u3.
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    // Step 2: inv_gu3 = 1 / gu3.
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    // Step 3a: prod = ε · inv_gu3.
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(eps, &inv_gu3, &mut prod, n);
    // Step 3b: arg = -prod.
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    // Step 4: em1 = expm1(arg).
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    // Step 5: inv_em1 = 1 / em1.
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);
    // Step 6: out = β_γ · inv_em1.
    ctaylor_scalar_mul::<F>(&inv_em1, F::cast_from(PBEC_BETA_GAMMA_F64), out, n);
}

/// `h_gga(d², ε, u³)` — PBEC gradient correction H term, port of
/// `pbec.cpp:26-33`:
/// ```cpp
/// num d2A = d2 * A(eps, u3);
/// return param_gamma * u3 *
///        log(1 + param_beta_gamma * d2 * (1 + d2A) / (1 + d2A * (1 + d2A)));
/// ```
///
/// **FULL BODY** (Wave 2, plan 03-02 — W3 conversion). Operation order:
///   1. `a       = a_expm1_inner(ε, u³)`         (this module)
///   2. `d2a     = d² · a`                       (mul)
///   3. `one_d2a = 1 + d2a`                       (CNST-bump)
///   4. `inner   = d2a · one_d2a`                 (mul)
///   5. `den     = 1 + inner`                     (CNST-bump)
///   6. `num1    = β_γ · d²`                      (scalar_mul)
///   7. `num2    = num1 · one_d2a`                (mul)
///   8. `inv_den = 1 / den`                       (reciprocal)
///   9. `frac    = num2 · inv_den`                (mul)
///   10. `arg    = 1 + frac`                      (CNST-bump)
///   11. `lg     = log(arg)`                      (ctaylor_log)
///   12. `gu3    = γ · u³`                        (scalar_mul)
///   13. `out    = gu3 · lg`                      (mul)
#[cube]
pub fn h_gga<F: Float>(
    d2: &Array<F>,
    eps: &Array<F>,
    u3: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // Step 1: a = A(ε, u³).
    let mut a = Array::<F>::new(size);
    a_expm1_inner::<F>(eps, u3, &mut a, n);
    // Step 2: d2a = d² · a.
    let mut d2a = Array::<F>::new(size);
    ctaylor_mul::<F>(d2, &a, &mut d2a, n);
    // Step 3: one_d2a = 1 + d2a (copy then CNST-bump).
    let mut one_d2a = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2a[i] = d2a[i];
    }
    one_d2a[0] = one_d2a[0] + F::new(1.0);
    // Step 4: inner = d2a · one_d2a.
    let mut inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2a, &one_d2a, &mut inner, n);
    // Step 5: den = 1 + inner.
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = inner[i];
    }
    den[0] = den[0] + F::new(1.0);
    // Step 6: num1 = β_γ · d².
    let mut num1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(d2, F::cast_from(PBEC_BETA_GAMMA_F64), &mut num1, n);
    // Step 7: num2 = num1 · one_d2a.
    let mut num2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&num1, &one_d2a, &mut num2, n);
    // Step 8: inv_den = 1 / den.
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    // Step 9: frac = num2 · inv_den.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num2, &inv_den, &mut frac, n);
    // Step 10: arg = 1 + frac.
    let mut arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        arg[i] = frac[i];
    }
    arg[0] = arg[0] + F::new(1.0);
    // Step 11: lg = log(arg).
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&arg, &mut lg, n);
    // Step 12: gu3 = γ · u³.
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    // Step 13: out = gu3 · lg.
    ctaylor_mul::<F>(&gu3, &lg, out, n);
}

/// `phi_reorganised(n_m13, a_43, b_43) = 2^(-1/3) · n_m13² · (sqrt(a_43) + sqrt(b_43))`
/// — algebraically identical to `phi(ζ)` but computed in the form used by the
/// C++ `pbec.cpp:35-38` to preserve 1e-12 operation-order identity.
///
/// **FULL BODY** (Wave 2, plan 03-02). Operation order matches C++ verbatim:
///   1. `sa  = sqrt(a_43)`                         (ctaylor_sqrt)
///   2. `sb  = sqrt(b_43)`                         (ctaylor_sqrt)
///   3. `sab = sa + sb`                            (ctaylor_add)
///   4. `nsq = n_m13 · n_m13`                       (ctaylor_mul)
///   5. `prod = nsq · sab`                          (ctaylor_mul)
///   6. `out  = 2^(-1/3) · prod`                    (scalar_mul)
#[cube]
pub fn phi_reorganised<F: Float>(
    n_m13: &Array<F>,
    a_43: &Array<F>,
    b_43: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut sa = Array::<F>::new(size);
    ctaylor_sqrt::<F>(a_43, &mut sa, n);
    let mut sb = Array::<F>::new(size);
    ctaylor_sqrt::<F>(b_43, &mut sb, n);
    let mut sab = Array::<F>::new(size);
    ctaylor_add::<F>(&sa, &sb, &mut sab, n);
    let mut nsq = Array::<F>::new(size);
    ctaylor_mul::<F>(n_m13, n_m13, &mut nsq, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&nsq, &sab, &mut prod, n);
    // 2^(-1/3) precomputed in f64.
    const TWO_NEG_13_F64: f64 = 0.793_700_525_984_099_8_f64;
    ctaylor_scalar_mul::<F>(&prod, F::cast_from(TWO_NEG_13_F64), out, n);
}

/// `phi(ζ) = ½ · ((1+ζ)^(2/3) + (1-ζ)^(2/3))` — PBEC spin-polarisation factor.
///
/// **FULL BODY** (Wave 2, plan 03-02 — W3 conversion). Operation order:
///   1. `pz   = 1 + ζ`                            (copy ζ + CNST-bump)
///   2. `mz   = 1 - ζ`                            (neg ζ + CNST-bump)
///   3. `pz23 = pow(pz, 2/3)`                     (ctaylor_pow)
///   4. `mz23 = pow(mz, 2/3)`                     (ctaylor_pow)
///   5. `sum  = pz23 + mz23`                      (add)
///   6. `out  = ½ · sum`                          (scalar_mul)
///
/// Note: this is the canonical `phi(ζ)` from `pbec.cpp:35-38` written in its
/// algebraically-direct form rather than the reorganised form using `n_m13`,
/// `a_43`, `b_43`. Either is valid; this form is slightly more direct for
/// arbitrary `ζ`.
#[cube]
pub fn phi<F: Float>(
    zeta: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // Step 1: pz = 1 + ζ.
    let mut pz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        pz[i] = zeta[i];
    }
    pz[0] = pz[0] + F::new(1.0);
    // Step 2: mz = 1 - ζ.
    let mut mz = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        mz[i] = F::new(0.0) - zeta[i];
    }
    mz[0] = mz[0] + F::new(1.0);
    // Step 3: pz23 = pow(pz, 2/3).
    let mut pz23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&pz, F::cast_from(2.0_f64 / 3.0_f64), &mut pz23, n);
    // Step 4: mz23 = pow(mz, 2/3).
    let mut mz23 = Array::<F>::new(size);
    ctaylor_pow::<F>(&mz, F::cast_from(2.0_f64 / 3.0_f64), &mut mz23, n);
    // Step 5: sum = pz23 + mz23.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&pz23, &mz23, &mut sum, n);
    // Step 6: out = ½ · sum.
    ctaylor_scalar_mul::<F>(&sum, F::cast_from(0.5_f64), out, n);
}
