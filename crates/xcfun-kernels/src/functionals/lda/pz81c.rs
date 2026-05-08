//! Perdew-Zunger 1981 LDA correlation functional. **LDA-05.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pz81c.cpp:18-20` (`pz81eps::pz81eps(d) * d.n`)
//! - `xcfun-master/src/functionals/pz81c.hpp:22-46` (pz81eps + fz + Eld + Ehd helpers)
//!
//! # Formula
//! Two regions per spin channel:
//!   - `r_s < 1`: Ehd branch — `c[1] + log(rs)*(c[0] + rs*c[2]) + c[3]*rs`
//!   - `r_s >= 1`: Eld branch — `c[0] / (1 + c[1]*sqrt(rs) + c[2]*rs)`
//!
//! Spin-interpolated via `fz(d) = (2^(4/3)*(a_43+b_43)*n_m13/n - 2) / (2*2^(1/3) - 2)`.
//!
//! # Implementation note — runtime if-else over ctaylor scalar
//!
//! C++ `if (1 > d.r_s)` switches on the CNST coefficient of `d.r_s`. The cleanest
//! cubecl lowering evaluates BOTH branches and selects via an arithmetic mask
//! based on `d.r_s[0]` compared to 1.0. This preserves Taylor-correctness at the
//! coefficient level: the chosen branch's coefficients pass through unchanged,
//! including all derivatives — because the branch test is on a scalar and each
//! branch is differentiable separately.
//!
//! For Phase 2 tier-1 (test input `(0.048, 0.025)` → `r_s ≈ 1.48`), the Eld
//! (low-density, r_s >= 1) branch is selected.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_log, ctaylor_reciprocal, ctaylor_sqrt};

use crate::density_vars::DensVarsDev;

// ---------------------------------------------------------------------------
// PZ81 parameter rows (pz81c.hpp:38-41).
// Row 0: low-density unpolarized  {gamma=-0.1423, beta1=1.0529, beta2=0.3334, _=0}
// Row 1: low-density polarized    {gamma=-0.0843, beta1=1.3981, beta2=0.2611, _=0}
// Row 2: high-density unpolarized {A=0.0311,  B=-0.048,  C=0.0020, D=-0.0116}
// Row 3: high-density polarized   {A=0.01555, B=-0.0269, C=0.0007, D=-0.0048}
// ---------------------------------------------------------------------------

// Stored as f64 and cast via F::cast_from at kernel-time for 1e-11 tier-1 parity.
const PZ81_LD_UNPOL_GAMMA: f64 = -0.1423_f64;
const PZ81_LD_UNPOL_B1: f64 = 1.0529_f64;
const PZ81_LD_UNPOL_B2: f64 = 0.3334_f64;

const PZ81_LD_POL_GAMMA: f64 = -0.0843_f64;
const PZ81_LD_POL_B1: f64 = 1.3981_f64;
const PZ81_LD_POL_B2: f64 = 0.2611_f64;

const PZ81_HD_UNPOL_A: f64 = 0.0311_f64;
const PZ81_HD_UNPOL_B: f64 = -0.048_f64;
const PZ81_HD_UNPOL_C: f64 = 0.0020_f64;
const PZ81_HD_UNPOL_D: f64 = -0.0116_f64;

const PZ81_HD_POL_A: f64 = 0.01555000000_f64;
const PZ81_HD_POL_B: f64 = -0.0269_f64;
const PZ81_HD_POL_C: f64 = 0.0007_f64;
const PZ81_HD_POL_D: f64 = -0.0048_f64;

// Constants for fz (pz81c.hpp:24-25): p = 2^(4/3), q = 2*2^(1/3) - 2.
const PZ81_FZ_P: f64 = 2.5198420997897464_f64; // 2^(4/3)
// 1/q where q = 0.5198420997897463 — reused from pw92eps.
const PZ81_FZ_INV_Q: f64 = 1.9236610509315363_f64;

// ---------------------------------------------------------------------------
//  fz(d) = (p * (a_43 + b_43) * n_m13 / n - 2) / q
//
//  Operation order (matches C++ pz81c.hpp:26):
//    step 1: sum_43 = a_43 + b_43
//    step 2: prod1  = p * sum_43          (scalar_mul)
//    step 3: prod2  = prod1 * n_m13
//    step 4: inv_n  = 1 / n
//    step 5: prod3  = prod2 * inv_n
//    step 6: diff   = prod3 - 2           (sub scalar 2 from CNST)
//    step 7: out    = diff / q = diff * (1/q)
// ---------------------------------------------------------------------------

#[cube]
fn fz<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut sum_43 = Array::<F>::new(size);
    ctaylor_add::<F>(&d.a_43, &d.b_43, &mut sum_43, n);
    let mut prod1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sum_43, F::cast_from(PZ81_FZ_P), &mut prod1, n);
    let mut prod2 = Array::<F>::new(size);
    ctaylor_mul::<F>(&prod1, &d.n_m13, &mut prod2, n);
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    let mut prod3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&prod2, &inv_n, &mut prod3, n);

    let mut two_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut two_const, n);
    two_const[0] = F::new(2.0);
    let mut diff = Array::<F>::new(size);
    ctaylor_sub::<F>(&prod3, &two_const, &mut diff, n);

    ctaylor_scalar_mul::<F>(&diff, F::cast_from(PZ81_FZ_INV_Q), out, n);
}

// ---------------------------------------------------------------------------
//  Eld(x, CB1B2) = CB1B2[0] / (1 + CB1B2[1]*sqrt(x) + CB1B2[2]*x)
//
//  Operation order:
//    sqrt_x   = sqrt(x)
//    b1_sqrt  = CB1B2[1] * sqrt_x
//    b2_x     = CB1B2[2] * x
//    denom1   = 1 + b1_sqrt    (CNST += 1)
//    denom    = denom1 + b2_x
//    inv      = 1 / denom
//    out      = CB1B2[0] * inv
// ---------------------------------------------------------------------------

#[cube]
fn eld<F: Float>(x: &Array<F>, gamma: F, b1: F, b2: F, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut sqrt_x = Array::<F>::new(size);
    ctaylor_sqrt::<F>(x, &mut sqrt_x, n);

    let mut b1_sqrt = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&sqrt_x, b1, &mut b1_sqrt, n);

    let mut b2_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, b2, &mut b2_x, n);

    let mut one_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut one_const, n);
    one_const[0] = F::new(1.0);
    let mut denom1 = Array::<F>::new(size);
    ctaylor_add::<F>(&b1_sqrt, &one_const, &mut denom1, n);
    let mut denom = Array::<F>::new(size);
    ctaylor_add::<F>(&denom1, &b2_x, &mut denom, n);

    let mut inv = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&denom, &mut inv, n);
    ctaylor_scalar_mul::<F>(&inv, gamma, out, n);
}

// ---------------------------------------------------------------------------
//  Ehd(x, c) = c[1] + log(x)*(c[0] + x*c[2]) + c[3]*x
//
//  Operation order (C++ pz81c.hpp:33-35 left-to-right):
//    log_x    = log(x)
//    x_c2     = c[2] * x
//    inner    = c[0] + x_c2     (CNST += c[0])
//    prod     = log_x * inner
//    c3_x     = c[3] * x
//    tmp      = c[1] + prod     (CNST += c[1] on prod)
//    out      = tmp + c3_x
// ---------------------------------------------------------------------------

#[cube]
fn ehd<F: Float>(x: &Array<F>, c0: F, c1: F, c2: F, c3: F, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut log_x = Array::<F>::new(size);
    ctaylor_log::<F>(x, &mut log_x, n);

    let mut x_c2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, c2, &mut x_c2, n);

    let mut c0_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut c0_const, n);
    c0_const[0] = c0;
    let mut inner = Array::<F>::new(size);
    ctaylor_add::<F>(&c0_const, &x_c2, &mut inner, n);

    let mut prod = Array::<F>::new(size);
    ctaylor_mul::<F>(&log_x, &inner, &mut prod, n);

    let mut c3_x = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(x, c3, &mut c3_x, n);

    let mut c1_const = Array::<F>::new(size);
    ctaylor_zero::<F>(&mut c1_const, n);
    c1_const[0] = c1;
    let mut tmp = Array::<F>::new(size);
    ctaylor_add::<F>(&c1_const, &prod, &mut tmp, n);

    ctaylor_add::<F>(&tmp, &c3_x, out, n);
}

// ---------------------------------------------------------------------------
//  pz81_eps(d) — port of pz81c.hpp:37-46.
//
//  C++:
//    if (1 > d.r_s)
//      return Ehd(d.r_s, c[2]) + (Ehd(d.r_s, c[3]) - Ehd(d.r_s, c[2])) * fz(d);
//    else
//      return Eld(d.r_s, c[0]) + (Eld(d.r_s, c[1]) - Eld(d.r_s, c[0])) * fz(d);
//
//  Runtime branch on r_s[0] — evaluated via cubecl's Float comparison (same
//  idiom as `regularize.rs`). Each branch is a separate ctaylor computation;
//  at kernel lowering cubecl-cpu emits a scalar conditional that takes one
//  branch at a time (no select-on-ctaylor-coefficients needed because each
//  branch writes the full `out` independently).
// ---------------------------------------------------------------------------

#[cube]
pub fn pz81_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    let mut fz_val = Array::<F>::new(size);
    fz::<F>(d, &mut fz_val, n);

    if d.r_s[0] < F::new(1.0) {
        // High-density branch (Ehd): c[2] = unpolarized, c[3] = polarized.
        let mut ehd_unpol = Array::<F>::new(size);
        ehd::<F>(
            &d.r_s,
            F::cast_from(PZ81_HD_UNPOL_A),
            F::cast_from(PZ81_HD_UNPOL_B),
            F::cast_from(PZ81_HD_UNPOL_C),
            F::cast_from(PZ81_HD_UNPOL_D),
            &mut ehd_unpol,
            n,
        );
        let mut ehd_pol = Array::<F>::new(size);
        ehd::<F>(
            &d.r_s,
            F::cast_from(PZ81_HD_POL_A),
            F::cast_from(PZ81_HD_POL_B),
            F::cast_from(PZ81_HD_POL_C),
            F::cast_from(PZ81_HD_POL_D),
            &mut ehd_pol,
            n,
        );
        let mut diff = Array::<F>::new(size);
        ctaylor_sub::<F>(&ehd_pol, &ehd_unpol, &mut diff, n);
        let mut prod = Array::<F>::new(size);
        ctaylor_mul::<F>(&diff, &fz_val, &mut prod, n);
        ctaylor_add::<F>(&ehd_unpol, &prod, out, n);
    } else {
        // Low-density branch (Eld): c[0] = unpolarized, c[1] = polarized.
        let mut eld_unpol = Array::<F>::new(size);
        eld::<F>(
            &d.r_s,
            F::cast_from(PZ81_LD_UNPOL_GAMMA),
            F::cast_from(PZ81_LD_UNPOL_B1),
            F::cast_from(PZ81_LD_UNPOL_B2),
            &mut eld_unpol,
            n,
        );
        let mut eld_pol = Array::<F>::new(size);
        eld::<F>(
            &d.r_s,
            F::cast_from(PZ81_LD_POL_GAMMA),
            F::cast_from(PZ81_LD_POL_B1),
            F::cast_from(PZ81_LD_POL_B2),
            &mut eld_pol,
            n,
        );
        let mut diff = Array::<F>::new(size);
        ctaylor_sub::<F>(&eld_pol, &eld_unpol, &mut diff, n);
        let mut prod = Array::<F>::new(size);
        ctaylor_mul::<F>(&diff, &fz_val, &mut prod, n);
        ctaylor_add::<F>(&eld_unpol, &prod, out, n);
    }
}

/// PZ81 correlation kernel. 1:1 port of `pz81c.cpp:18-20`:
/// `return pz81eps::pz81eps(d) * d.n;`
#[cube]
pub fn pz81c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let mut eps = Array::<F>::new(comptime!((1_u32 << n) as usize));
    pz81_eps::<F>(d, &mut eps, n);
    ctaylor_mul::<F>(&eps, &d.n, out, n);
}
