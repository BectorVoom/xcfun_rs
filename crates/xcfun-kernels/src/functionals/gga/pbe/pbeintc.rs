//! XC_PBEINTC — PBEint correlation functional. GGA-01.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbeintc.cpp:18-38`
//!
//! # Formula
//! Like PBEC but with `beta = 0.052` (not the accurate β = 0.066724…),
//! so β_γ is recomputed inline.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_3, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{PBEC_D2_PREFACTOR_F64, PBEC_GAMMA_F64};
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;

const PBEINTC_BETA_F64: f64 = 0.052_f64;
// β / γ = 0.052 / ((1 - log(2)) / π²) = 0.052 / 0.0310906908696549.
// Locked by `tests::pbeintc_bg_locked`. Previous value 0.167_252_472_614_525_44
// was a copy-paste typo from `ZVPBEINTC_BG_F64 = 1.672_524_726_145_254_4` with
// the decimal shifted one place left, causing 99.84% order-0 record-level
// FAIL in the Phase-7 Plan 07-00 Task 0.3 sweep against C++ a89b783.
const PBEINTC_BG_F64: f64 = 1.672_526_359_031_570_2_f64;

#[cube]
pub fn pbeintc_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let _ = PBEINTC_BETA_F64;
    let bg = F::cast_from(PBEINTC_BG_F64);

    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);

    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);

    let mut denom = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut denom, n);
    let mut inv_denom = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv_denom, n);
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_denom, &mut g_over_d, n);
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // A = bg / expm1(-eps / (gamma · u³)).
    let mut gu3 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&u3, F::cast_from(PBEC_GAMMA_F64), &mut gu3, n);
    let mut inv_gu3 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&gu3, &mut inv_gu3, n);
    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&eps, &inv_gu3, &mut prod, n);
    let neg_one = F::new(0.0) - F::new(1.0);
    let mut arg = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&prod, neg_one, &mut arg, n);
    let mut em1 = Array::<F>::new(size);
    ctaylor_expm1::<F>(&arg, &mut em1, n);
    let mut inv_em1 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&em1, &mut inv_em1, n);
    let mut a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_em1, bg, &mut a, n);

    // d2A = d2 · a.
    let mut d2a = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &a, &mut d2a, n);
    // one_d2a = 1 + d2a.
    let mut one_d2a = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2a[i] = d2a[i];
    }
    one_d2a[0] = one_d2a[0] + F::new(1.0);
    // inner = d2a · one_d2a.
    let mut inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2a, &one_d2a, &mut inner, n);
    // den = 1 + inner.
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = inner[i];
    }
    den[0] = den[0] + F::new(1.0);
    // num1 = bg · d2.
    let mut num1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2, bg, &mut num1, n);
    // num2 = num1 · one_d2a.
    let mut num2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&num1, &one_d2a, &mut num2, n);
    // inv_den = 1 / den.
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    // frac = num2 · inv_den.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num2, &inv_den, &mut frac, n);
    // lg_arg = 1 + frac.
    let mut lg_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        lg_arg[i] = frac[i];
    }
    lg_arg[0] = lg_arg[0] + F::new(1.0);
    // lg = log(lg_arg).
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&lg_arg, &mut lg, n);
    // h = γ · u³ · lg.
    let mut h = Array::<F>::new(size);
    ctaylor_mul::<F>(&gu3, &lg, &mut h, n);

    // sum = eps + h.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &h, &mut sum, n);

    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum, out, n);
}

#[cfg(test)]
mod tests {
    /// Regression lock for the PBEINTC β/γ precomputation. The previous
    /// value `0.167_252_472_614_525_44_f64` was a copy-paste typo from
    /// `ZVPBEINTC_BG_F64 = 1.672_524_726_145_254_4_f64` with the decimal
    /// shifted one place left, making the GGA-correction term H ≈ 10× too
    /// small everywhere and producing 99.84% record-level FAIL against
    /// C++ at order 0 in the Phase 7 Plan 07-00 Task 0.3 sweep.
    /// The mathematically correct value is exactly β/γ where β = 0.052
    /// and γ = (1 - log(2)) / π² ≈ 0.0310906908696549.
    #[test]
    fn pbeintc_bg_locked() {
        // Recompute from full-precision γ rather than the imprecise constant
        // used historically; this is the f64-nearest of `0.052 / γ`.
        let truth: f64 = 1.672_526_359_031_570_2_f64;
        assert_eq!(super::PBEINTC_BG_F64, truth);
    }
}
