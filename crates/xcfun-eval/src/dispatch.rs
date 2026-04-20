//! Functional dispatcher — match-on-FunctionalId resolves the registry
//! circular-dep (xcfun-core has no fp-table; xcfun-eval owns dispatch).
//!
//! See RESEARCH.md §"Registry Shape + Circular-Dep Resolution" for rationale.
//!
//! Phase 2 dispatch arms (11 LDAs):
//!   id ==  0 → XC_SLATERX     (Plan 02-04)
//!   id ==  2 → XC_VWN3C       (Plan 02-04)
//!   id ==  3 → XC_VWN5C       (Plan 02-04)
//!   id == 13 → XC_LDAERFX     (Plan 02-04)
//!   id == 14 → XC_LDAERFC     (Plan 02-04)
//!   id == 15 → XC_LDAERFC_JT  (Plan 02-04)
//!   id == 24 → XC_TFK         (Plan 02-04)
//!   id == 25 → XC_TW          (Plan 02-05)
//!   id == 28 → XC_PW92C       (Plan 02-04)
//!   id == 55 → XC_PZ81C       (Plan 02-04)
//!   id == 59 → XC_VWK         (Plan 02-05)
//!
//! Plans 02-04 and 02-05 EXTEND this file to add the actual kernel calls;
//! this plan ships the file with all 11 arms commented out (host-side
//! `Functional::eval` returns `XcError::NotConfigured` until kernels exist).

use cubecl::prelude::*;
use xcfun_core::FunctionalId;

use crate::density_vars::DensVarsDev;

/// Comptime-dispatched per-functional kernel call. Each arm corresponds to a
/// `FunctionalId` discriminant (xcfun.h historical-insertion ordering).
///
/// Plan 02-04/02-05 fill in the LDA kernel calls. Phase 3+ extends with GGA/MGGA.
///
/// The `#[allow(unused_variables)]` is needed because the body is empty until
/// Plans 02-04/02-05 uncomment the arms — until then all four parameters are
/// syntactically unused.
#[cube]
#[allow(unused_variables)]
pub fn dispatch_kernel<F: Float>(
    #[comptime] id: u32,
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // PHASE 2 LDA arms — Plans 02-04 + 02-05 uncomment as kernels land:
    //
    // if comptime!(id == 0) {            // XC_SLATERX
    //     crate::functionals::lda::slaterx::slaterx_kernel::<F>(d, out, n);
    // } else if comptime!(id == 2) {     // XC_VWN3C
    //     crate::functionals::lda::vwn3c::vwn3c_kernel::<F>(d, out, n);
    // } else if comptime!(id == 3) {     // XC_VWN5C
    //     crate::functionals::lda::vwn5c::vwn5c_kernel::<F>(d, out, n);
    // } else if comptime!(id == 13) {    // XC_LDAERFX
    //     crate::functionals::lda::ldaerfx::ldaerfx_kernel::<F>(d, out, n);
    // } else if comptime!(id == 14) {    // XC_LDAERFC
    //     crate::functionals::lda::ldaerfc::ldaerfc_kernel::<F>(d, out, n);
    // } else if comptime!(id == 15) {    // XC_LDAERFC_JT
    //     crate::functionals::lda::ldaerfc_jt::ldaerfc_jt_kernel::<F>(d, out, n);
    // } else if comptime!(id == 24) {    // XC_TFK
    //     crate::functionals::lda::tfk::tfk_kernel::<F>(d, out, n);
    // } else if comptime!(id == 25) {    // XC_TW (Plan 02-05)
    //     crate::functionals::lda::tw::tw_kernel::<F>(d, out, n);
    // } else if comptime!(id == 28) {    // XC_PW92C
    //     crate::functionals::lda::pw92c::pw92c_kernel::<F>(d, out, n);
    // } else if comptime!(id == 55) {    // XC_PZ81C
    //     crate::functionals::lda::pz81c::pz81c_kernel::<F>(d, out, n);
    // } else if comptime!(id == 59) {    // XC_VWK (Plan 02-05)
    //     crate::functionals::lda::vwk::vwk_kernel::<F>(d, out, n);
    // }
}

/// Host-side guard: returns true if `id` has an implemented kernel arm in
/// `dispatch_kernel`. Called by `Functional::eval` BEFORE launching, so stubs
/// (67 non-LDA + any not-yet-implemented LDA) return `XcError::NotConfigured`.
///
/// Plan 02-03 ships an EMPTY allowlist; Plan 02-04 extends to 9 ids (SLATERX,
/// VWN3C, VWN5C, LDAERFX, LDAERFC, LDAERFC_JT, TFK, PW92C, PZ81C); Plan 02-05
/// extends to 11 ids (adds TW, VWK).
pub fn supports(_id: FunctionalId) -> bool {
    // Plan 02-04 will add: XC_SLATERX, XC_VWN3C, XC_VWN5C, XC_LDAERFX, XC_LDAERFC,
    //                      XC_LDAERFC_JT, XC_TFK, XC_PW92C, XC_PZ81C.
    // Plan 02-05 will add: XC_TW, XC_VWK.
    false
}
