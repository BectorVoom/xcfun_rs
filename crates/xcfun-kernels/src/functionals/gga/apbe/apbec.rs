//! XC_APBEC — APBE correlation functional. **GGA-08.**
//!
//! # Source
//! - `xcfun-master/src/functionals/apbec.cpp:18-38`
//!
//! # Formula
//! ```cpp
//! using xcfun_constants::param_gamma;
//! const parameter beta = 0.079030523241;   // APBE-specific β (NOT PBE β)
//! num bg = beta / param_gamma;
//! num eps = pw92eps::pw92eps(d);
//! num u = phi(d);   // reorganised: 2^(-1/3) · n_m13² · (sqrt(a_43)+sqrt(b_43))
//! num u3 = pow3(u);
//! num d2 = (1/12 · 3^(5/6) / π^(-1/6))² · gnn / (u² · n^(7/3));
//! num A = bg / expm1(-eps / (param_gamma · u³));
//! num d2A = d2 · A;
//! num H = param_gamma · u³ · log(1 + bg · d2 · (1 + d2A) / (1 + d2A · (1 + d2A)));
//! return n · (eps + H);
//! ```
//!
//! Difference from PBEC: APBEC uses an APBE-specific β (`0.079030523241`) vs.
//! PBE β (`0.066724550603`). The shared `pbec_eps::a_expm1_inner` and `h_gga`
//! hardcode `PBEC_BETA_GAMMA_F64 = β_PBE / γ`. For APBEC we re-implement the
//! H-formula inline with the APBE-specific β/γ ratio.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_expm1, ctaylor_log, ctaylor_pow, ctaylor_powi_3, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{PBEC_D2_PREFACTOR_F64, PBEC_GAMMA_F64};
use crate::functionals::gga::shared::pbec_eps;
use crate::functionals::lda::pw92eps;

/// APBE-specific β value per `apbec.cpp:21`.
const APBE_BETA: f64 = 0.079_030_523_241_f64;
/// β / γ ratio precomputed in f64.
const APBE_BETA_GAMMA: f64 = APBE_BETA / PBEC_GAMMA_F64;

/// XC_APBEC kernel. 1:1 port of `apbec.cpp:18-38`.
#[cube]
pub fn apbec_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // eps = pw92_eps(d).
    let mut eps = Array::<F>::new(size);
    pw92eps::pw92_eps::<F>(d, &mut eps, n);

    // u = phi_reorganised(n_m13, a_43, b_43).
    let mut u = Array::<F>::new(size);
    pbec_eps::phi_reorganised::<F>(&d.n_m13, &d.a_43, &d.b_43, &mut u, n);

    // u² and u³.
    let mut u2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u, &u, &mut u2, n);
    let mut u3 = Array::<F>::new(size);
    ctaylor_powi_3::<F>(&u, &mut u3, n);

    // n^(7/3).
    let mut n_73 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(7.0_f64 / 3.0_f64), &mut n_73, n);

    // u² · n^(7/3).
    let mut den_d2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&u2, &n_73, &mut den_d2, n);

    // 1 / denom.
    let mut inv_d2 = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den_d2, &mut inv_d2, n);

    // gnn · inv.
    let mut g_over_d = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.gnn, &inv_d2, &mut g_over_d, n);

    // d2 = PREFACTOR · g_over_d.
    let mut d2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&g_over_d, F::cast_from(PBEC_D2_PREFACTOR_F64), &mut d2, n);

    // A = bg / expm1(-eps / (γ·u³)).
    // Step: gu3 = γ · u³.
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
    let mut a_apbe = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&inv_em1, F::cast_from(APBE_BETA_GAMMA), &mut a_apbe, n);

    // d2A = d² · A.
    let mut d2a = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2, &a_apbe, &mut d2a, n);
    // 1 + d2A.
    let mut one_d2a = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_d2a[i] = d2a[i];
    }
    one_d2a[0] = one_d2a[0] + F::new(1.0);
    // d2A · (1+d2A).
    let mut inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&d2a, &one_d2a, &mut inner, n);
    // 1 + inner.
    let mut den = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        den[i] = inner[i];
    }
    den[0] = den[0] + F::new(1.0);
    // num1 = bg · d².
    let mut num1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d2, F::cast_from(APBE_BETA_GAMMA), &mut num1, n);
    // num2 = num1 · (1+d2A).
    let mut num2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&num1, &one_d2a, &mut num2, n);
    // inv_den = 1 / den.
    let mut inv_den = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&den, &mut inv_den, n);
    // frac = num2 · inv_den.
    let mut frac = Array::<F>::new(size);
    ctaylor_mul::<F>(&num2, &inv_den, &mut frac, n);
    // 1 + frac.
    let mut log_arg = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        log_arg[i] = frac[i];
    }
    log_arg[0] = log_arg[0] + F::new(1.0);
    let mut lg = Array::<F>::new(size);
    ctaylor_log::<F>(&log_arg, &mut lg, n);
    // gu3_lg = γ · u³ · lg.
    let mut h = Array::<F>::new(size);
    ctaylor_mul::<F>(&gu3, &lg, &mut h, n);

    // sum = eps + H.
    let mut sum = Array::<F>::new(size);
    ctaylor_add::<F>(&eps, &h, &mut sum, n);

    // out = n · sum.
    ctaylor_mul::<F>(&d.n, &sum, out, n);
}
