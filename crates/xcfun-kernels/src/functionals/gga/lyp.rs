//! XC_LYPC — LYP correlation. GGA-04.
//!
//! # Source
//! - `xcfun-master/src/functionals/lypc.cpp:18-39`
//!
//! # Formula
//! ```cpp
//! const parameter A = 0.04918;  B = 0.132;  C = 0.2533;  Dd = 0.349;
//! using xcfun_constants::CF;   // = (3/10) · (3π²)^(2/3)
//! icbrtn = pow(d.n, -1/3);
//! P = 1 / (1 + Dd · icbrtn);
//! omega = exp(-C · icbrtn) · P · pow(d.n, -11/3);
//! delta = icbrtn · (C + Dd · P);
//! n2 = d.n²;
//! return -A · ( 4·a·b·P/n
//!         + B·omega · ( a·b · ( 2^(11/3)·CF·(a^(8/3)+b^(8/3))
//!                              + (47-7·delta)·gnn/18
//!                              - (2.5 - delta/18)·(gaa+gbb)
//!                              - (delta-11)/9·(a·gaa + b·gbb)/n
//!                            )
//!                       - 2/3·n²·gnn + (2/3·n² - a²)·gbb + (2/3·n² - b²)·gaa
//!                     )
//!     );
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_powi_2, ctaylor_reciprocal};

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::constants::{
    LYP_A_F64, LYP_B_F64, LYP_C_F64, LYP_CF_F64, LYP_D_F64,
};

/// `2^(11/3)` precomputed in f64.
const TWO_11_3_F64: f64 = 12.699_208_415_745_595_f64;

#[cube]
pub fn lypc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);

    // icbrtn = ρ^(-1/3).
    let mut icbrtn = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-1.0_f64 / 3.0_f64), &mut icbrtn, n);

    // 1 + Dd·icbrtn.
    let mut dd_ic = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&icbrtn, F::cast_from(LYP_D_F64), &mut dd_ic, n);
    let mut one_plus = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        one_plus[i] = dd_ic[i];
    }
    one_plus[0] = one_plus[0] + F::new(1.0);
    // P = 1 / (1 + Dd·icbrtn).
    let mut p = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&one_plus, &mut p, n);

    // exp(-C · icbrtn).
    let mut neg_c_ic = Array::<F>::new(size);
    let neg_one = F::new(0.0) - F::new(1.0);
    ctaylor_scalar_mul::<F>(&icbrtn, neg_one * F::cast_from(LYP_C_F64), &mut neg_c_ic, n);
    let mut exp_term = Array::<F>::new(size);
    ctaylor_exp::<F>(&neg_c_ic, &mut exp_term, n);

    // ρ^(-11/3).
    let mut n_m113 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.n, F::cast_from(-11.0_f64 / 3.0_f64), &mut n_m113, n);

    // omega = exp(-C·ic) · P · ρ^(-11/3).
    let mut exp_p = Array::<F>::new(size);
    ctaylor_mul::<F>(&exp_term, &p, &mut exp_p, n);
    let mut omega = Array::<F>::new(size);
    ctaylor_mul::<F>(&exp_p, &n_m113, &mut omega, n);

    // delta = icbrtn · (C + Dd·P).
    let mut dd_p = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&p, F::cast_from(LYP_D_F64), &mut dd_p, n);
    let mut c_plus_ddp = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        c_plus_ddp[i] = dd_p[i];
    }
    c_plus_ddp[0] = c_plus_ddp[0] + F::cast_from(LYP_C_F64);
    let mut delta = Array::<F>::new(size);
    ctaylor_mul::<F>(&icbrtn, &c_plus_ddp, &mut delta, n);

    // n2 = ρ².
    let mut n2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.n, &mut n2, n);

    // a·b.
    let mut ab = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &d.b, &mut ab, n);

    // 4·a·b·P/n: 4·ab·P · (1/n).
    let mut ab_p = Array::<F>::new(size);
    ctaylor_mul::<F>(&ab, &p, &mut ab_p, n);
    let mut inv_n = Array::<F>::new(size);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, n);
    let mut ab_p_inv_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&ab_p, &inv_n, &mut ab_p_inv_n, n);
    let mut four_ab_p_n = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&ab_p_inv_n, F::new(4.0), &mut four_ab_p_n, n);

    // a^(8/3) + b^(8/3).
    let mut a83 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.a, F::cast_from(8.0_f64 / 3.0_f64), &mut a83, n);
    let mut b83 = Array::<F>::new(size);
    ctaylor_pow::<F>(&d.b, F::cast_from(8.0_f64 / 3.0_f64), &mut b83, n);
    let mut a83_b83 = Array::<F>::new(size);
    ctaylor_add::<F>(&a83, &b83, &mut a83_b83, n);

    // 2^(11/3)·CF·(a^(8/3)+b^(8/3)).
    let mut term1 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &a83_b83,
        F::cast_from(TWO_11_3_F64) * F::cast_from(LYP_CF_F64),
        &mut term1,
        n,
    );

    // (47 - 7·delta) · gnn / 18.
    let mut seven_delta = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&delta, F::new(7.0), &mut seven_delta, n);
    let mut neg_seven_delta = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&seven_delta, neg_one, &mut neg_seven_delta, n);
    let mut bracket1 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket1[i] = neg_seven_delta[i];
    }
    bracket1[0] = bracket1[0] + F::new(47.0);
    let mut bracket1_gnn = Array::<F>::new(size);
    ctaylor_mul::<F>(&bracket1, &d.gnn, &mut bracket1_gnn, n);
    let mut term2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&bracket1_gnn, F::new(1.0) / F::new(18.0), &mut term2, n);

    // (2.5 - delta/18) · (gaa + gbb).
    let mut delta_18 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&delta, F::new(1.0) / F::new(18.0), &mut delta_18, n);
    let mut neg_delta_18 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&delta_18, neg_one, &mut neg_delta_18, n);
    let mut bracket2 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket2[i] = neg_delta_18[i];
    }
    bracket2[0] = bracket2[0] + F::new(2.5);
    let mut gaa_gbb = Array::<F>::new(size);
    ctaylor_add::<F>(&d.gaa, &d.gbb, &mut gaa_gbb, n);
    let mut term3 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bracket2, &gaa_gbb, &mut term3, n);

    // (delta - 11)/9 · (a·gaa + b·gbb)/n.
    let mut a_gaa = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.a, &d.gaa, &mut a_gaa, n);
    let mut b_gbb = Array::<F>::new(size);
    ctaylor_mul::<F>(&d.b, &d.gbb, &mut b_gbb, n);
    let mut sum_g = Array::<F>::new(size);
    ctaylor_add::<F>(&a_gaa, &b_gbb, &mut sum_g, n);
    let mut sum_g_inv_n = Array::<F>::new(size);
    ctaylor_mul::<F>(&sum_g, &inv_n, &mut sum_g_inv_n, n);
    // (delta - 11)/9 = delta/9 - 11/9.
    let mut delta_9 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&delta, F::new(1.0) / F::new(9.0), &mut delta_9, n);
    let mut bracket3 = Array::<F>::new(size);
    #[unroll]
    for i in 0..size {
        bracket3[i] = delta_9[i];
    }
    bracket3[0] = bracket3[0] - F::new(11.0) / F::new(9.0);
    let mut term4 = Array::<F>::new(size);
    ctaylor_mul::<F>(&bracket3, &sum_g_inv_n, &mut term4, n);

    // inner_paren = term1 + term2 - term3 - term4.
    let mut sum12 = Array::<F>::new(size);
    ctaylor_add::<F>(&term1, &term2, &mut sum12, n);
    let mut sum12_minus_3 = Array::<F>::new(size);
    ctaylor_sub::<F>(&sum12, &term3, &mut sum12_minus_3, n);
    let mut inner_paren = Array::<F>::new(size);
    ctaylor_sub::<F>(&sum12_minus_3, &term4, &mut inner_paren, n);

    // ab · inner_paren.
    let mut ab_inner = Array::<F>::new(size);
    ctaylor_mul::<F>(&ab, &inner_paren, &mut ab_inner, n);

    // -2/3·n²·gnn.
    let mut n2_gnn = Array::<F>::new(size);
    ctaylor_mul::<F>(&n2, &d.gnn, &mut n2_gnn, n);
    let mut neg_two_thirds_n2_gnn = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(
        &n2_gnn,
        neg_one * F::new(2.0) / F::new(3.0),
        &mut neg_two_thirds_n2_gnn,
        n,
    );

    // (2/3·n² - a²)·gbb.
    let mut two_thirds_n2 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&n2, F::new(2.0) / F::new(3.0), &mut two_thirds_n2, n);
    let mut a2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.a, &mut a2, n);
    let mut bracket4 = Array::<F>::new(size);
    ctaylor_sub::<F>(&two_thirds_n2, &a2, &mut bracket4, n);
    let mut bracket4_gbb = Array::<F>::new(size);
    ctaylor_mul::<F>(&bracket4, &d.gbb, &mut bracket4_gbb, n);

    // (2/3·n² - b²)·gaa.
    let mut b2 = Array::<F>::new(size);
    ctaylor_powi_2::<F>(&d.b, &mut b2, n);
    let mut bracket5 = Array::<F>::new(size);
    ctaylor_sub::<F>(&two_thirds_n2, &b2, &mut bracket5, n);
    let mut bracket5_gaa = Array::<F>::new(size);
    ctaylor_mul::<F>(&bracket5, &d.gaa, &mut bracket5_gaa, n);

    // outer_inner = ab·inner_paren + (-2/3·n²·gnn) + bracket4_gbb + bracket5_gaa.
    let mut s1 = Array::<F>::new(size);
    ctaylor_add::<F>(&ab_inner, &neg_two_thirds_n2_gnn, &mut s1, n);
    let mut s2 = Array::<F>::new(size);
    ctaylor_add::<F>(&s1, &bracket4_gbb, &mut s2, n);
    let mut outer = Array::<F>::new(size);
    ctaylor_add::<F>(&s2, &bracket5_gaa, &mut outer, n);

    // B · omega · outer.
    let mut omega_outer = Array::<F>::new(size);
    ctaylor_mul::<F>(&omega, &outer, &mut omega_outer, n);
    let mut b_omega_outer = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&omega_outer, F::cast_from(LYP_B_F64), &mut b_omega_outer, n);

    // bracket_total = 4·a·b·P/n + B·omega·outer.
    let mut bracket_total = Array::<F>::new(size);
    ctaylor_add::<F>(&four_ab_p_n, &b_omega_outer, &mut bracket_total, n);

    // out = -A · bracket_total.
    ctaylor_scalar_mul::<F>(&bracket_total, neg_one * F::cast_from(LYP_A_F64), out, n);
}
