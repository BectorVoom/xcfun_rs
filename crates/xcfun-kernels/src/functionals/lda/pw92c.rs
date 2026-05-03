//! PW92 LDA correlation functional. **LDA-04.**
//!
//! # Source
//! - `xcfun-master/src/functionals/pw92c.cpp:18-20` (`pw92eps::pw92eps(d) * d.n`)
//! - `xcfun-master/src/functionals/pw92eps.hpp:48-61` (epsilon helper, accurate constants
//!    per RESEARCH §"PW92C Legacy Constants")
//!
//! # Phase 2 PW92C Legacy Constants (RESEARCH §"PW92C Legacy Constants")
//! Ships ACCURATE constants only (XCFUN_REF_PW92C undefined). No Cargo feature flag.

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use super::pw92eps::pw92_eps;
use crate::density_vars::DensVarsDev;

/// PW92 correlation kernel. 1:1 port of `pw92c.cpp:18-20`.
#[cube]
pub fn pw92c_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // C++: return pw92eps::pw92eps(d) * d.n;
    //   ctaylor_mul is commutative but we preserve C++ operand order (eps * n).
    let mut eps = Array::<F>::new(comptime!((1_u32 << n) as usize));
    pw92_eps::<F>(d, &mut eps, n);
    ctaylor_mul::<F>(&eps, &d.n, out, n);
}
