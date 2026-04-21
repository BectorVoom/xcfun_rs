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
//! Plan 02-04 uncomments 9 of 11 arms (all pure-density XC_A_B LDAs);
//! Plan 02-05 uncomments the remaining two (TW + VWK, kinetic-GGAs via XC_A_B_GAA_GAB_GBB).

use cubecl::prelude::*;
use xcfun_core::FunctionalId;

use crate::density_vars::DensVarsDev;

/// Comptime-dispatched per-functional kernel call. Each arm corresponds to a
/// `FunctionalId` discriminant (xcfun.h historical-insertion ordering).
///
/// Plan 02-04 fills in the 9 pure-density LDA arms (XC_A_B variant); Plan 02-05
/// adds TW + VWK (XC_A_B_GAA_GAB_GBB). Phase 3+ extends with GGA/MGGA.
#[cube]
#[allow(unused_variables)]
pub fn dispatch_kernel<F: Float>(
    #[comptime] id: u32,
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    if comptime!(id == 0) {
        // XC_SLATERX
        crate::functionals::lda::slaterx::slaterx_kernel::<F>(d, out, n);
    } else if comptime!(id == 2) {
        // XC_VWN3C
        crate::functionals::lda::vwn3c::vwn3c_kernel::<F>(d, out, n);
    } else if comptime!(id == 3) {
        // XC_VWN5C
        crate::functionals::lda::vwn5c::vwn5c_kernel::<F>(d, out, n);
    } else if comptime!(id == 13) {
        // XC_LDAERFX
        crate::functionals::lda::ldaerfx::ldaerfx_kernel::<F>(d, out, n);
    } else if comptime!(id == 14) {
        // XC_LDAERFC
        crate::functionals::lda::ldaerfc::ldaerfc_kernel::<F>(d, out, n);
    } else if comptime!(id == 15) {
        // XC_LDAERFC_JT
        crate::functionals::lda::ldaerfc_jt::ldaerfc_jt_kernel::<F>(d, out, n);
    } else if comptime!(id == 24) {
        // XC_TFK
        crate::functionals::lda::tfk::tfk_kernel::<F>(d, out, n);
    } else if comptime!(id == 25) {
        // XC_TW (Plan 02-05 Wave-1C-2 — kinetic-GGA via XC_A_B_GAA_GAB_GBB)
        crate::functionals::lda::tw::tw_kernel::<F>(d, out, n);
    } else if comptime!(id == 28) {
        // XC_PW92C
        crate::functionals::lda::pw92c::pw92c_kernel::<F>(d, out, n);
    } else if comptime!(id == 55) {
        // XC_PZ81C
        crate::functionals::lda::pz81c::pz81c_kernel::<F>(d, out, n);
    } else if comptime!(id == 59) {
        // XC_VWK (Plan 02-05 Wave-1C-3 — kinetic-GGA via XC_A_B_GAA_GAB_GBB)
        crate::functionals::lda::vwk::vwk_kernel::<F>(d, out, n);
    }
}

/// Host-side guard: returns true if `id` has an implemented kernel arm in
/// `dispatch_kernel`. Called by `Functional::eval` BEFORE launching, so stubs
/// (67 non-LDA + any not-yet-implemented LDA) return `XcError::NotConfigured`.
///
/// Phase 2 final (Plan 02-05 Wave-1C-4) ships all 11 LDA ids:
///   XC_SLATERX (0), XC_VWN3C (2), XC_VWN5C (3), XC_LDAERFX (13),
///   XC_LDAERFC (14), XC_LDAERFC_JT (15), XC_TFK (24), XC_TW (25),
///   XC_PW92C (28), XC_PZ81C (55), XC_VWK (59).
pub fn supports(id: FunctionalId) -> bool {
    matches!(
        id as u32,
        0 | 2 | 3 | 13 | 14 | 15 | 24 | 25 | 28 | 55 | 59
    )
}
