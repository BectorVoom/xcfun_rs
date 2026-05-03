//! XC_M06X2X — M06-2X exchange functional. MGGA-03.
//!
//! # Source
//! - `xcfun-master/src/functionals/m06x2x.cpp:23-41`
//!
//! # Formula
//! Per `m06x2x.cpp:20-22` the `param_d[6]` array is all zero, so the
//! `lsda_x() * h()` term drops out (as `h()=0`). The body reduces to:
//! ```cpp
//! return (pbex::energy_pbe_ab(R_pbe, d.a, d.gaa) * fw(param_a, d.a, d.taua) +
//!         pbex::energy_pbe_ab(R_pbe, d.b, d.gbb) * fw(param_a, d.b, d.taub));
//! ```
//!
//! Vars: `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pbex;
use crate::functionals::mgga::shared::m0x_like;

const R_PBE_F64: f64 = 0.804_f64;

// param_a[12] from m06x2x.cpp:26-37.
const M06X2X_A0_F64: f64 = 4.600_000e-1_f64;
const M06X2X_A1_F64: f64 = -2.206_052e-1_f64;
const M06X2X_A2_F64: f64 = -9.431_788e-2_f64;
const M06X2X_A3_F64: f64 = 2.164_494e0_f64;
const M06X2X_A4_F64: f64 = -2.556_466e0_f64;
const M06X2X_A5_F64: f64 = -1.422_133e1_f64;
const M06X2X_A6_F64: f64 = 1.555_044e1_f64;
const M06X2X_A7_F64: f64 = 3.598_078e1_f64;
const M06X2X_A8_F64: f64 = -2.722_754e1_f64;
const M06X2X_A9_F64: f64 = -3.924_093e1_f64;
const M06X2X_A10_F64: f64 = 1.522_808e1_f64;
const M06X2X_A11_F64: f64 = 1.522_227e1_f64;

#[cube]
pub fn m06x2x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let r = F::cast_from(R_PBE_F64);

    let mut pbe_a = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(r, &d.a_43, &d.a, &d.gaa, &mut pbe_a, n);

    let mut fw_a = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M06X2X_A0_F64),
        F::cast_from(M06X2X_A1_F64),
        F::cast_from(M06X2X_A2_F64),
        F::cast_from(M06X2X_A3_F64),
        F::cast_from(M06X2X_A4_F64),
        F::cast_from(M06X2X_A5_F64),
        F::cast_from(M06X2X_A6_F64),
        F::cast_from(M06X2X_A7_F64),
        F::cast_from(M06X2X_A8_F64),
        F::cast_from(M06X2X_A9_F64),
        F::cast_from(M06X2X_A10_F64),
        F::cast_from(M06X2X_A11_F64),
        &d.a,
        &d.taua,
        &mut fw_a,
        n,
    );

    let mut term_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbe_a, &fw_a, &mut term_a, n);

    let mut pbe_b = Array::<F>::new(size);
    pbex::energy_pbe_ab::<F>(r, &d.b_43, &d.b, &d.gbb, &mut pbe_b, n);

    let mut fw_b = Array::<F>::new(size);
    m0x_like::m0x_fw::<F>(
        F::cast_from(M06X2X_A0_F64),
        F::cast_from(M06X2X_A1_F64),
        F::cast_from(M06X2X_A2_F64),
        F::cast_from(M06X2X_A3_F64),
        F::cast_from(M06X2X_A4_F64),
        F::cast_from(M06X2X_A5_F64),
        F::cast_from(M06X2X_A6_F64),
        F::cast_from(M06X2X_A7_F64),
        F::cast_from(M06X2X_A8_F64),
        F::cast_from(M06X2X_A9_F64),
        F::cast_from(M06X2X_A10_F64),
        F::cast_from(M06X2X_A11_F64),
        &d.b,
        &d.taub,
        &mut fw_b,
        n,
    );

    let mut term_b = Array::<F>::new(size);
    ctaylor_mul::<F>(&pbe_b, &fw_b, &mut term_b, n);

    ctaylor_add::<F>(&term_a, &term_b, out, n);
}
