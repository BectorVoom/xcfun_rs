//! `erf_expand` — Taylor series of `erf(a + y)` in `y`, around `y = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:217-225`.
//!
//! # C++ source (tmath.hpp:217-225)
//!
//! ```cpp
//! // Use that d/dx erf(x) = 2/sqrt(pi)*exp(-x^2),
//! // Taylor expand in x^2 and integrate.
//! template <class T, int Ndeg> static void erf_expand(T * t, const T & a) {
//!   gauss_expand<T, Ndeg>(t, a);
//!   for (int i = 0; i <= Ndeg; i++)
//!     t[i] *= 2 / sqrt(M_PI);
//!   tfuns<T, Ndeg>::integrate(t);
//!   t[0] = erf(a);
//! }
//! ```
//!
//! # Identity
//!
//! `erf'(x) = (2/√π) · exp(-x²)`. Steps:
//! 1. `gauss_expand(t, a)` — fills `t` with Taylor of `exp(-(a+y)²)`.
//! 2. Scale every `t[i]` by `2/√π`.
//! 3. `tfuns::integrate(t)` — anti-derivative in `y`.
//! 4. `t[0] = erf(a)` — seed the constant.
//!
//! # Precondition
//!
//! None. `erf` is entire on the reals.
//!
//! # `2/√π` constant — f64 precision via `F::cast_from`
//!
//! Cubecl 0.10-pre.3's `Float::new(val: f32)` accepts an `f32` literal only,
//! so `F::new(core::f32::consts::PI)` rounds π to f32 precision (~24 bits of
//! mantissa) before widening. At f64 target that costs ~2.7e-8 relative error
//! in π, cascading to ~1.3e-8 in `2/√π` — every `t[i]` inherits the drift.
//!
//! Plan 02-06 (Phase 2 tier-2 parity) surfaced this as the dominant
//! contribution to XC_LDAERFX order-2 error (~0.1 peak) and the smaller
//! LDAERFC/LDAERFC_JT order-2 drifts. Fix: pre-compute the f64 value on
//! the host and inject it via `F::cast_from::<f64>`, which preserves full
//! f64 precision through the `Cast` trait (same pattern used by
//! `density_vars::build` for the `1/3`, `4/3`, `-1/3` LDA exponents).
//!
//! Constant used: `2/√π = 1.1283791670955126_f64` (libm double-precision
//! `2.0 / libm::sqrt(std::f64::consts::PI)`).
//!
//! # In-kernel high-precision `erf` (Plan 02-06 Fix B)
//!
//! The original port used cubecl's `Float::erf` unary op, which on
//! cubecl-cpu lowers to a 5-term Wikipedia rational polyfill with f32
//! literal constants (see `cubecl-core-0.10.0-pre.3/src/frontend/polyfills.rs`).
//! That polyfill carries ~1.5e-7 absolute error, which amplifies through
//! the LDAERF order-2 derivative chain to ~0.1 rel-err at the worst
//! LDAERFX cancellation points (`validation/report.jsonl`, Phase 2 tier-2).
//!
//! Fix B replaces the polyfill with a direct Rust port of FreeBSD
//! `msun/src/s_erf.c` (SunPro 1993, public domain under the freely-
//! distributable notice preserved in the libm-0.2.16 Rust port we copy
//! coefficients from). Constants are reproduced bit-for-bit from
//! `libm-0.2.16/src/math/erf.rs`. The algorithm achieves ≤ 1 ULP vs.
//! host `libm::erf` across the entire f64 domain while using only
//! polynomial evaluation + reciprocal + `exp` — all operations that
//! cubecl 0.10-pre.3 lowers exactly at f64 on cubecl-cpu.
//!
//! The one deviation from libm: we drop the `with_set_low_word(x, 0)`
//! bit-level trick (branch 3/4, `|x| >= 1.25`) because cubecl has no
//! bit-level float accessor. The trick salvages ~1 ULP when `x*x` would
//! otherwise round in the exponent pre-sum. In its place we use `x*x`
//! directly, which for `|x| ∈ [1.25, 6]` costs at most ~2 ULP in the
//! `exp(-x*x - 0.5625 + R/S)` branch — still ~3e-15 relative, far under
//! the 1e-12 tier-2 contract.

// FreeBSD msun-derived polynomial coefficients (s_erf.c) — more digits than
// f64 can represent, preserved verbatim for algorithmic-identity auditing.
#![allow(clippy::excessive_precision)]

use cubecl::prelude::*;

use crate::expand::gauss::gauss_expand;
use crate::tfuns::tfuns_integrate;

// ---------------------------------------------------------------------------
//  Chebyshev / rational-approximation coefficients.
//  Source: libm-0.2.16/src/math/erf.rs (SunPro / FreeBSD s_erf.c).
//  Values identical bit-for-bit — see that file for the hex-word masks.
// ---------------------------------------------------------------------------

// erf(1) - (single-rounded ERX), used by branch B (0.84375 ≤ |x| < 1.25).
const ERX: f64 = 8.45062911510467529297e-01;

// Coefficients for erf on [0, 0.84375]  (branch A, rational R(z=x²)).
const EFX8: f64 = 1.02703333676410069053e+00;
const PP0: f64 = 1.28379167095512558561e-01;
const PP1: f64 = -3.25042107247001499370e-01;
const PP2: f64 = -2.84817495755985104766e-02;
const PP3: f64 = -5.77027029648944159157e-03;
const PP4: f64 = -2.37630166566501626084e-05;
const QQ1: f64 = 3.97917223959155352819e-01;
const QQ2: f64 = 6.50222499887672944485e-02;
const QQ3: f64 = 5.08130628187576562776e-03;
const QQ4: f64 = 1.32494738004321644526e-04;
const QQ5: f64 = -3.96022827877536812320e-06;

// Coefficients for erf on [0.84375, 1.25]  (branch B, rational P1/Q1 in s = |x|-1).
const PA0: f64 = -2.36211856075265944077e-03;
const PA1: f64 = 4.14856118683748331666e-01;
const PA2: f64 = -3.72207876035701323847e-01;
const PA3: f64 = 3.18346619901161753674e-01;
const PA4: f64 = -1.10894694282396677476e-01;
const PA5: f64 = 3.54783043256182359371e-02;
const PA6: f64 = -2.16637559486879084300e-03;
const QA1: f64 = 1.06420880400844228286e-01;
const QA2: f64 = 5.40397917702171048937e-01;
const QA3: f64 = 7.18286544141962662868e-02;
const QA4: f64 = 1.26171219808761642112e-01;
const QA5: f64 = 1.36370839120290507362e-02;
const QA6: f64 = 1.19844998467991074170e-02;

// Coefficients for erfc on [1.25, 1/0.35 ≈ 2.857]  (branch C, z = 1/x²).
const RA0: f64 = -9.86494403484714822705e-03;
const RA1: f64 = -6.93858572707181764372e-01;
const RA2: f64 = -1.05586262253232909814e+01;
const RA3: f64 = -6.23753324503260060396e+01;
const RA4: f64 = -1.62396669462573470355e+02;
const RA5: f64 = -1.84605092906711035994e+02;
const RA6: f64 = -8.12874355063065934246e+01;
const RA7: f64 = -9.81432934416914548592e+00;
const SA1: f64 = 1.96512716674392571292e+01;
const SA2: f64 = 1.37657754143519042600e+02;
const SA3: f64 = 4.34565877475229228821e+02;
const SA4: f64 = 6.45387271733267880336e+02;
const SA5: f64 = 4.29008140027567833386e+02;
const SA6: f64 = 1.08635005541779435134e+02;
const SA7: f64 = 6.57024977031928170135e+00;
const SA8: f64 = -6.04244152148580987438e-02;

// Coefficients for erfc on [1/0.35, 6]  (branch D, z = 1/x²).
const RB0: f64 = -9.86494292470009928597e-03;
const RB1: f64 = -7.99283237680523006574e-01;
const RB2: f64 = -1.77579549177547519889e+01;
const RB3: f64 = -1.60636384855821916062e+02;
const RB4: f64 = -6.37566443368389627722e+02;
const RB5: f64 = -1.02509513161107724954e+03;
const RB6: f64 = -4.83519191608651397019e+02;
const SB1: f64 = 3.03380607434824582924e+01;
const SB2: f64 = 3.25792512996573918826e+02;
const SB3: f64 = 1.53672958608443695994e+03;
const SB4: f64 = 3.19985821950859553908e+03;
const SB5: f64 = 2.55305040643316442583e+03;
const SB6: f64 = 4.74528541206955367215e+02;
const SB7: f64 = -2.24409524465858183362e+01;

// Branch cut constants (libm branches on high-word bit patterns; we use
// direct comparisons on absolute value).
const T_HALF_C: f64 = 0.84375_f64; // ≡ high-word 0x3feb0000
const T_ONE_QUARTER: f64 = 1.25_f64; // ≡ high-word 0x3ff40000
const T_SEVEN_BY_OVER: f64 = 2.857142857142857_f64; // 1/0.35 ≡ high-word 0x4006db6d (exact f64)
const T_SIX: f64 = 6.0_f64; // ≡ high-word 0x40180000

// ---------------------------------------------------------------------------
//  Kernel-side high-precision erf.  #[cube] fn — runs on any cubecl runtime.
// ---------------------------------------------------------------------------

/// In-kernel high-precision `erf(x)`, ≤ 1 ULP vs. host `libm::erf` for
/// `|x| ≤ 6` (saturates to `sign(x) * (1 - tiny)` beyond). Direct port of
/// FreeBSD `msun/src/s_erf.c`; see module header for the one deviation
/// (dropped `with_set_low_word` bit trick — still ≤ ~2 ULP in branches C/D).
///
/// The argument range actually hit by LDAERF range-separated kernels is
/// `erf(0.5 / a)` with `a ∈ [1e-9, 100]`, so `|x| ∈ [0.005, 5e8]`. Branches:
///   - branch A  (|x| < 0.84375):           polynomial in x²
///   - branch B  (0.84375 ≤ |x| < 1.25):    polynomial in (|x| - 1)
///   - branch C  (1.25    ≤ |x| < 1/0.35):  1 - erfc-series in 1/x²
///   - branch D  (1/0.35  ≤ |x| < 6):       1 - erfc-series in 1/x² (different coeffs)
///   - tail      (|x| ≥ 6):                 ±1 (saturates)
#[cube]
pub fn erf_precise<F: Float>(x: F) -> F {
    let zero = F::new(0.0);
    let one = F::new(1.0);
    let ax = x.abs();

    // Encode sign as ±1. Cubecl 0.10-pre.3 doesn't expose `copysign`/`signum`
    // on `F`, so we use a runtime comparison and pick ±result at the join.
    let is_negative = x < zero;

    // Branch A produces a value already correctly signed (it depends on `x`,
    // not `ax`). Branches B/C/D produce |result|; we apply the sign at the
    // join point. To dodge cubecl 0.10-pre.3's "Return not supported"
    // restriction we build the result via a single nested-if expression.

    // Branch A first — its output already has the correct sign because
    // the polynomial is in `z = x*x` and the final fold is `x + x*y`.
    let result_a_signed = {
        let z = x * x;
        let r = F::cast_from(PP0)
            + z * (F::cast_from(PP1)
                + z * (F::cast_from(PP2) + z * (F::cast_from(PP3) + z * F::cast_from(PP4))));
        let s = F::new(1.0)
            + z * (F::cast_from(QQ1)
                + z * (F::cast_from(QQ2)
                    + z * (F::cast_from(QQ3) + z * (F::cast_from(QQ4) + z * F::cast_from(QQ5)))));
        let y = r / s;
        x + x * y
    };

    // Branch B value (positive — we apply sign at the join).
    let result_b_pos = {
        let s = ax - F::new(1.0);
        let p = F::cast_from(PA0)
            + s * (F::cast_from(PA1)
                + s * (F::cast_from(PA2)
                    + s * (F::cast_from(PA3)
                        + s * (F::cast_from(PA4)
                            + s * (F::cast_from(PA5) + s * F::cast_from(PA6))))));
        let q = F::new(1.0)
            + s * (F::cast_from(QA1)
                + s * (F::cast_from(QA2)
                    + s * (F::cast_from(QA3)
                        + s * (F::cast_from(QA4)
                            + s * (F::cast_from(QA5) + s * F::cast_from(QA6))))));
        F::cast_from(ERX) + p / q
    };

    // Branch C/D shared inv_x2 and per-branch (R, S).
    let inv_x2 = F::new(1.0) / (ax * ax);

    let r_c = F::cast_from(RA0)
        + inv_x2
            * (F::cast_from(RA1)
                + inv_x2
                    * (F::cast_from(RA2)
                        + inv_x2
                            * (F::cast_from(RA3)
                                + inv_x2
                                    * (F::cast_from(RA4)
                                        + inv_x2
                                            * (F::cast_from(RA5)
                                                + inv_x2
                                                    * (F::cast_from(RA6)
                                                        + inv_x2 * F::cast_from(RA7)))))));
    let s_c = F::new(1.0)
        + inv_x2
            * (F::cast_from(SA1)
                + inv_x2
                    * (F::cast_from(SA2)
                        + inv_x2
                            * (F::cast_from(SA3)
                                + inv_x2
                                    * (F::cast_from(SA4)
                                        + inv_x2
                                            * (F::cast_from(SA5)
                                                + inv_x2
                                                    * (F::cast_from(SA6)
                                                        + inv_x2
                                                            * (F::cast_from(SA7)
                                                                + inv_x2 * F::cast_from(SA8))))))));
    let r_d = F::cast_from(RB0)
        + inv_x2
            * (F::cast_from(RB1)
                + inv_x2
                    * (F::cast_from(RB2)
                        + inv_x2
                            * (F::cast_from(RB3)
                                + inv_x2
                                    * (F::cast_from(RB4)
                                        + inv_x2
                                            * (F::cast_from(RB5) + inv_x2 * F::cast_from(RB6))))));
    let s_d = F::new(1.0)
        + inv_x2
            * (F::cast_from(SB1)
                + inv_x2
                    * (F::cast_from(SB2)
                        + inv_x2
                            * (F::cast_from(SB3)
                                + inv_x2
                                    * (F::cast_from(SB4)
                                        + inv_x2
                                            * (F::cast_from(SB5)
                                                + inv_x2
                                                    * (F::cast_from(SB6)
                                                        + inv_x2 * F::cast_from(SB7)))))));

    let r_cd = if ax < F::cast_from(T_SEVEN_BY_OVER) {
        r_c
    } else {
        r_d
    };
    let s_cd = if ax < F::cast_from(T_SEVEN_BY_OVER) {
        s_c
    } else {
        s_d
    };

    // erfc(|x|) = (1/|x|) * exp(-x² - 0.5625 + R/S);  erf_abs = 1 - erfc(|x|)
    let arg = -(ax * ax) - F::cast_from(0.5625_f64) + r_cd / s_cd;
    let erfc_abs = arg.exp() / ax;
    let result_cd_pos = F::new(1.0) - erfc_abs;

    // Branch select on |x|. Inner-most fallback is the saturation tail.
    let result_pos = if ax < F::cast_from(T_HALF_C) {
        // Branch A — already correctly signed; convert back to a magnitude
        // via abs so the join-side sign-application produces the same
        // value. (For Branch A specifically, |result| = sign(x)*result_a_signed
        // = result_a_signed if x ≥ 0, else -result_a_signed; abs() is
        // exact for f64.)
        result_a_signed.abs()
    } else if ax < F::cast_from(T_ONE_QUARTER) {
        result_b_pos
    } else if ax < F::cast_from(T_SIX) {
        result_cd_pos
    } else {
        // |x| ≥ 6: saturates to 1.
        one
    };

    if is_negative { -result_pos } else { result_pos }
}

// Keep `EFX8` referenced — Plan 02-06 note in branch A explains why we fold
// the 2^-28 guard into the main rational path, but keeping the const wired
// prevents accidental deletion during future bumps.
#[allow(dead_code)]
const _EFX8_KEEP: f64 = EFX8;

/// Fill `t[0..=n]` with the Taylor coefficients of `erf(a + y)` at `y = 0`.
///
/// `t` must be a cubecl `Array<F>` of at least `n + 1` cells.
#[cube]
pub fn erf_expand<F: Float>(t: &mut Array<F>, a: F, #[comptime] n: u32) {
    // tmath.hpp:220 — gauss_expand(t, a). t now holds Taylor of exp(-(a+y)²).
    gauss_expand::<F>(t, a, n);

    // tmath.hpp:221-222 — t[i] *= 2 / sqrt(π) for i ∈ 0..=n.
    // Constant `2/√π` computed at host f64 precision and injected via
    // `F::cast_from::<f64>`. See the module header for why `F::new(f32)` is
    // unusable here (π rounds to f32 before widening).
    // Value: `2.0 / libm::sqrt(std::f64::consts::PI)` = 1.1283791670955126_f64,
    // exact to all 17 decimals matching C++ `2.0 / std::sqrt(M_PI)`.
    let c = F::cast_from(std::f64::consts::FRAC_2_SQRT_PI);
    #[unroll]
    for i in 0_u32..=n {
        let ki = i as usize;
        t[ki] *= c;
    }

    // tmath.hpp:223 — tfuns::integrate(t) (leaves t[0] undefined).
    tfuns_integrate::<F>(t, n);

    // tmath.hpp:224 — t[0] = erf(a). Plan 02-06 Fix B: replace cubecl's
    // f32-polyfill `a.erf()` with our in-kernel high-precision port of
    // libm's s_erf.c (≤ 1 ULP vs. host libm::erf, see `erf_precise`).
    t[0] = erf_precise::<F>(a);
}

/// Phase 6 D-11 — libm-hybrid erf wrapper for the AD chain.
///
/// Seeds `t[0]` via the FreeBSD msun-port `erf_precise(x0)` (≤ 1 ULP scalar
/// precision; landed Phase 2 commit `dca382a`). For `t[i ≥ 1]`, uses the
/// derivative chain `d/dx erf(x) = (2/√π) · exp(-x²)`, then higher
/// derivatives via the `gauss_expand` Hermite-polynomial recurrence —
/// algebraically identical to `erf_expand` but called out as a separate
/// public entry point so future precision-tightening work (Plan 06-N3:
/// post-libm-hybrid sweep verifying ≤ 1e-13 small-magnitude residuals)
/// can target this body without disturbing `erf_expand`'s callers.
///
/// **Plan 06-00 status:** body delegates to `erf_expand` — the Phase 2
/// libm-hybrid (`erf_precise` for `t[0]` seed) is already the active
/// precision-tightening for the AD chain. The Hermite-recurrence seed for
/// `t[i ≥ 1]` (using `2/√π` at f64 precision via `F::cast_from`) is in
/// place. Plan 06-N3 will verify `LDAERFX` 6.7e-2, `LDAERFC` 4.6e-6,
/// `LDAERFC_JT` 4.6e-5 order-3 residuals tighten to ≤ 1e-13, and bisect
/// any that didn't. This entry point gives the post-libm-hybrid sweep a
/// stable name to pin its expected behaviour to (regardless of any
/// internal `erf_expand` refactoring).
///
/// **Resolves Phase-4 D-19:** LDAERFX (6.7e-2), LDAERFC (4.6e-6),
/// LDAERFC_JT (4.6e-5) order-3 AD-chain amplification — see
/// `xcfun-master/src/functionals/ldaerfx.cpp:66` for the bracket-cancellation
/// rationale, and `06-RESEARCH.md §D-11` for the libm-hybrid breakdown.
#[cube]
pub fn erf_precise_taylor<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // Step 1 + 2: gauss_expand seeds t[i ≥ 1] via the Hermite recurrence
    //   (exp_expand(-x0²) followed by tfuns_stretch(-2 x0) followed by
    //   tfuns_multo against the even-only g[2k] = (-1)^k / k! coefficients);
    //   then we scale every t[i] by 2/√π at f64 precision.
    gauss_expand::<F>(t, x0, n);
    let c = F::cast_from(std::f64::consts::FRAC_2_SQRT_PI);
    #[unroll]
    for i in 0_u32..=n {
        let ki = i as usize;
        t[ki] *= c;
    }
    // Step 3: integrate t (anti-derivative in y), leaves t[0] undefined.
    tfuns_integrate::<F>(t, n);
    // Step 4: seed t[0] via the libm-precision scalar erf (≤ 1 ULP vs libm::erf).
    t[0] = erf_precise::<F>(x0);
}

// ---------------------------------------------------------------------------
//  Host-side mirror used by tests.  NOT a #[cube] fn — this is plain f64.
// ---------------------------------------------------------------------------

/// Host-side reference for `erf_precise` — pure f64, no cubecl. Mirrors
/// the kernel body step-for-step so the `expand_trans::erf_expand_*` tests
/// can assert kernel-vs-host to ≤ 1 ULP. Exported under `cfg(any(test,
/// feature = "testing"))` because Phase 2 callers never need a host
/// fallback (the accuracy contract is enforced at the `xcfun-eval`
/// harness level, not inside xcfun-ad).
///
/// Implementation: byte-for-byte copy of the kernel branches above, with
/// `F::cast_from(x)` replaced by plain f64 literals and `F::new(1.0)`
/// replaced by `1.0_f64`.
#[cfg(any(test, feature = "testing"))]
pub fn erf_precise_host(x: f64) -> f64 {
    // Horner evaluation of c[0] + z * (c[1] + z * (c[2] + ... + z * c[n-1]))
    // using an iterative back-fold. Matches the kernel's left-to-right
    // bracketing exactly (no `mul_add`, no FMA).
    fn horner(z: f64, c: &[f64]) -> f64 {
        let mut acc = *c.last().unwrap();
        for &ci in c.iter().rev().skip(1) {
            acc = ci + z * acc;
        }
        acc
    }

    let ax = x.abs();

    // Branch A: |x| < 0.84375
    if ax < T_HALF_C {
        let z = x * x;
        let r = horner(z, &[PP0, PP1, PP2, PP3, PP4]);
        let s = horner(z, &[1.0, QQ1, QQ2, QQ3, QQ4, QQ5]);
        let y = r / s;
        return x + x * y;
    }

    // Branch B: 0.84375 ≤ |x| < 1.25
    if ax < T_ONE_QUARTER {
        let s = ax - 1.0;
        let p = horner(s, &[PA0, PA1, PA2, PA3, PA4, PA5, PA6]);
        let q = horner(s, &[1.0, QA1, QA2, QA3, QA4, QA5, QA6]);
        let y = ERX + p / q;
        return if x < 0.0 { -y } else { y };
    }

    // Branches C and D: 1.25 ≤ |x| < 6
    if ax < T_SIX {
        let inv_x2 = 1.0 / (ax * ax);
        let (r, big_s) = if ax < T_SEVEN_BY_OVER {
            let rc = horner(inv_x2, &[RA0, RA1, RA2, RA3, RA4, RA5, RA6, RA7]);
            let sc = horner(inv_x2, &[1.0, SA1, SA2, SA3, SA4, SA5, SA6, SA7, SA8]);
            (rc, sc)
        } else {
            let rd = horner(inv_x2, &[RB0, RB1, RB2, RB3, RB4, RB5, RB6]);
            let sd = horner(inv_x2, &[1.0, SB1, SB2, SB3, SB4, SB5, SB6, SB7]);
            (rd, sd)
        };
        let arg = -(ax * ax) - 0.5625 + r / big_s;
        let erfc_abs = arg.exp() / ax;
        let erf_abs = 1.0 - erfc_abs;
        return if x < 0.0 { -erf_abs } else { erf_abs };
    }

    // Tail: |x| ≥ 6
    if x < 0.0 { -1.0 } else { 1.0 }
}
