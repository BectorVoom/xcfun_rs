//! PBE exchange-family enhancement helpers — 1:1 port of
//! `xcfun-master/src/functionals/pbex.hpp:26-52`.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbex.hpp:26-39`  — `enhancement(R, ρ, |∇ρ|²)`
//! - `xcfun-master/src/functionals/pbex.hpp:41-46`  — `enhancement_RPBE(ρ, |∇ρ|²)`
//! - `xcfun-master/src/functionals/pbex.hpp:48-52`  — `energy_pbe_ab(R, ρ, |∇ρ|²)`
//!
//! # Formulas
//! ```text
//! enhancement(R, ρ, grad²) = 1 + R - R / (1 + (μ/R) · S²(ρ, grad²))
//! enhancement_RPBE(ρ, grad²) = 1 - R_pbe · (expm1((-μ/R_pbe) · S²(ρ, grad²)))
//! energy_pbe_ab(R, ρ, grad²) = prefactor(ρ) · enhancement(R, ρ, grad²)
//! ```
//! where `μ = 0.066725 · π² / 3` (default `XCFUN_REF_PBEX_MU` undefined) and
//! `R ∈ {R_PBE = 0.804, R_REVPBE = 1.245, R_RPBE = 0.804}`.
//!
//! # Preconditions
//! - `rho[0] > 0` (post-regularize — CNST coefficient above `TINY_DENSITY`).
//! - `grad2[0] >= 0` (gradient squared is non-negative).
//!
//! # Wave 1 status (03-01)
//! - `enhancement` — **FULL BODY** (primary consumer this plan).
//! - `energy_pbe_ab` — **FULL BODY** (used by future PBEX/REVPBEX/PBESOLX kernels).
//! - `enhancement_RPBE` — **SKELETON** (body lands in 03-02 Task 1 Step A, RPBEX consumer).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_scalar_mul, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;

use super::constants::{MU_PBE_F64, NEG_C_SLATER_F64};
use super::pw91_like;

/// `enhancement(R, ρ, |∇ρ|²)` — shared PBE / REVPBE / PBESOL exchange
/// enhancement factor. `R` is passed as an `F`-typed scalar so PBEX (R = 0.804)
/// and REVPBE (R = 1.245) share the same kernel — caller supplies the κ value.
///
/// Formula (port of `pbex.hpp:26-39`):
/// ```text
/// S²   = pw91_like::s2(ρ, grad²)
/// t1   = 1 + (μ / R) · S²
/// enh  = 1 + R - R / t1
/// ```
///
/// Operation order (strict left-to-right, no `mul_add` per ACC-06):
///   1. `s2      = pw91_like::s2(ρ, grad²)`
///   2. `scaled  = (μ/R) · s2`                         (scalar_mul)
///   3. `t1      = scaled + F::new(1.0) ⊕ slot 0`       (add-to-CNST via tmp + scalar_mul)
///   4. `inv_t1  = 1 / t1`                              (ctaylor_reciprocal)
///   5. `rr      = R · inv_t1`                          (scalar_mul)
///   6. `out     = (F::new(1.0) + R) ⊕ slot 0 − rr`    (scalar constant − rr)
///
/// NOTE: adding a scalar constant to a CTaylor touches **only** the CNST slot
/// per the bit-flag indexing; we do it via an explicit `tmp[0] += 1` since there
/// is no `ctaylor_add_scalar` primitive in the Phase-1/2 API surface.
#[cube]
pub fn enhancement<F: Float>(
    r: F,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Step 1: s2_val = pw91_like::s2(rho, grad2).
    let mut s2_val = Array::<F>::new(size);
    pw91_like::s2::<F>(rho, grad2, &mut s2_val, n);

    // Step 2: scaled = (MU_PBE / R) * s2_val.
    let mu_over_r = F::cast_from(MU_PBE_F64) / r;
    let mut scaled = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2_val, mu_over_r, &mut scaled, n);

    // Step 3: t1 = 1 + scaled. Copy scaled then bump CNST by 1 (bit-flag indexed).
    let mut t1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        t1[i] = scaled[i];
    }
    t1[0] = t1[0] + F::new(1.0);

    // Step 4: inv_t1 = 1 / t1.
    let mut inv_t1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&t1, &mut inv_t1, n);

    // Step 5: rr = R * inv_t1.
    let mut rr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_t1, r, &mut rr, n);

    // Step 6: out = (1 + R) - rr. Copy (-rr) then bump CNST by (1 + R).
    // First compute neg_rr = -rr via scalar_mul by -1; then add `1 + R` to CNST.
    // To avoid a neg helper we use ctaylor_sub against a zero buffer, BUT the
    // cleanest Phase-2 path is scalar_mul by F::new(-1.0) then add-to-CNST.
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut neg_rr = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&rr, neg_one, &mut neg_rr, n);
    #[unroll]
    for i in 0..size {
        out[i] = neg_rr[i];
    }
    out[0] = out[0] + F::new(1.0) + r;
}

/// `enhancement_RPBE(ρ, |∇ρ|²)` — RPBE exchange enhancement factor.
///
/// Port of `pbex.hpp:41-46`:
/// ```cpp
/// return 1 - R_pbe · expm1((-μ / R_pbe) · S²(ρ, grad²));
/// ```
///
/// SKELETON — full body lands in 03-02 Task 1 Step A (RPBEX consumer).
#[cube]
pub fn enhancement_rpbe<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // SKELETON — full body lands in 03-02 Task 1 Step A.
    let _ = rho;
    let _ = grad2;
    ctaylor_zero::<F>(out, n);
}

/// `energy_pbe_ab(R, ρ, |∇ρ|²) = prefactor(ρ) · enhancement(R, ρ, grad²)`.
///
/// Port of `pbex.hpp:48-52`. **FULL BODY**.
///
/// The upstream `prefactor(rho)` = `-C_SLATER · ρ^(4/3)` algebraically
/// collapses to `NEG_C_SLATER_F64 · rho_43`. This body uses the caller-supplied
/// `rho_43 = ρ^(4/3)` array (from `DensVarsDev::a_43` or `b_43`) to avoid a
/// redundant `ctaylor_pow` — matching how `slaterx.rs` consumes `d.a_43`/`d.b_43`.
///
/// Operation order (strict left-to-right, no `mul_add` per ACC-06):
///   1. `enh       = enhancement(R, ρ, grad²)`          (this module)
///   2. `neg_pref  = NEG_C_SLATER · rho_43`              (scalar_mul)
///   3. `out       = neg_pref · enh`                    (ctaylor_mul)
#[cube]
pub fn energy_pbe_ab<F: Float>(
    r: F,
    rho_43: &Array<F>,
    rho: &Array<F>,
    grad2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // enh = enhancement(R, rho, grad2).
    let mut enh = Array::<F>::new(size);
    enhancement::<F>(r, rho, grad2, &mut enh, n);

    // neg_pref = NEG_C_SLATER · rho_43.
    let mut neg_pref = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(rho_43, F::cast_from(NEG_C_SLATER_F64), &mut neg_pref, n);

    // out = neg_pref · enh.
    ctaylor_mul::<F>(&neg_pref, &enh, out, n);
}
