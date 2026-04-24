//! Functional dispatcher — match-on-FunctionalId resolves the registry
//! circular-dep (xcfun-core has no fp-table; xcfun-eval owns dispatch).
//!
//! See RESEARCH.md §"Registry Shape + Circular-Dep Resolution" for rationale.
//!
//! Phase 2 dispatch arms (11 LDAs) + Phase 3 Wave-2 GGAs (17):
//!   id ==  0 → XC_SLATERX     (Plan 02-04)
//!   id ==  2 → XC_VWN3C       (Plan 02-04)
//!   id ==  3 → XC_VWN5C       (Plan 02-04)
//!   id ==  4 → XC_PBEC        (Plan 03-02 — Wave 2)
//!   id ==  5 → XC_PBEX        (Plan 03-02 — Wave 2)
//!   id ==  6 → XC_BECKEX      (Plan 03-02 — Wave 2)
//!   id ==  7 → XC_BECKECORRX  (Plan 03-02 — Wave 2)
//!   id ==  8 → XC_BECKESRX    (Plan 03-02 — Wave 2)
//!   id ==  9 → XC_BECKECAMX   (Plan 03-02 — Wave 2)
//!   id == 13 → XC_LDAERFX     (Plan 02-04)
//!   id == 14 → XC_LDAERFC     (Plan 02-04)
//!   id == 15 → XC_LDAERFC_JT  (Plan 02-04)
//!   id == 16 → XC_LYPC        (Plan 03-02 — Wave 2)
//!   id == 19 → XC_REVPBEX     (Plan 03-02 — Wave 2)
//!   id == 20 → XC_RPBEX       (Plan 03-02 — Wave 2)
//!   id == 21 → XC_SPBEC       (Plan 03-02 — Wave 2)
//!   id == 22 → XC_VWN_PBEC    (Plan 03-02 — Wave 2)
//!   id == 24 → XC_TFK         (Plan 02-04)
//!   id == 25 → XC_TW          (Plan 02-05)
//!   id == 28 → XC_PW92C       (Plan 02-04)
//!   id == 55 → XC_PZ81C       (Plan 02-04)
//!   id == 59 → XC_VWK         (Plan 02-05)
//!   id == 69 → XC_ZVPBESOLC   (Plan 03-02 — Wave 2)
//!   id == 71 → XC_PBEINTC     (Plan 03-02 — Wave 2)
//!   id == 72 → XC_PBEINTX     (Plan 03-02 — Wave 2)
//!   id == 73 → XC_PBELOCC     (Plan 03-02 — Wave 2)
//!   id == 74 → XC_PBESOLX     (Plan 03-02 — Wave 2)
//!   id == 76 → XC_ZVPBEINTC   (Plan 03-02 — Wave 2)

use cubecl::prelude::*;
use xcfun_core::FunctionalId;

use crate::density_vars::DensVarsDev;

/// Comptime-dispatched per-functional kernel call. Each arm corresponds to a
/// `FunctionalId` discriminant (xcfun.h historical-insertion ordering).
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
    } else if comptime!(id == 4) {
        // XC_PBEC
        crate::functionals::gga::pbe::pbec::pbec_kernel::<F>(d, out, n);
    } else if comptime!(id == 5) {
        // XC_PBEX
        crate::functionals::gga::pbe::pbex::pbex_kernel::<F>(d, out, n);
    } else if comptime!(id == 6) {
        // XC_BECKEX
        crate::functionals::gga::becke::beckex::beckex_kernel::<F>(d, out, n);
    } else if comptime!(id == 7) {
        // XC_BECKECORRX
        crate::functionals::gga::becke::beckecorrx::beckecorrx_kernel::<F>(d, out, n);
    } else if comptime!(id == 8) {
        // XC_BECKESRX
        crate::functionals::gga::becke::beckesrx::beckesrx_kernel::<F>(d, out, n);
    } else if comptime!(id == 9) {
        // XC_BECKECAMX
        crate::functionals::gga::becke::beckecamx::beckecamx_kernel::<F>(d, out, n);
    } else if comptime!(id == 13) {
        // XC_LDAERFX
        crate::functionals::lda::ldaerfx::ldaerfx_kernel::<F>(d, out, n);
    } else if comptime!(id == 14) {
        // XC_LDAERFC
        crate::functionals::lda::ldaerfc::ldaerfc_kernel::<F>(d, out, n);
    } else if comptime!(id == 15) {
        // XC_LDAERFC_JT
        crate::functionals::lda::ldaerfc_jt::ldaerfc_jt_kernel::<F>(d, out, n);
    } else if comptime!(id == 16) {
        // XC_LYPC
        crate::functionals::gga::lyp::lypc_kernel::<F>(d, out, n);
    } else if comptime!(id == 19) {
        // XC_REVPBEX
        crate::functionals::gga::pbe::revpbex::revpbex_kernel::<F>(d, out, n);
    } else if comptime!(id == 20) {
        // XC_RPBEX
        crate::functionals::gga::pbe::rpbex::rpbex_kernel::<F>(d, out, n);
    } else if comptime!(id == 21) {
        // XC_SPBEC
        crate::functionals::gga::pbe::spbec::spbec_kernel::<F>(d, out, n);
    } else if comptime!(id == 22) {
        // XC_VWN_PBEC
        crate::functionals::gga::pbe::vwn_pbec::vwn_pbec_kernel::<F>(d, out, n);
    } else if comptime!(id == 24) {
        // XC_TFK
        crate::functionals::lda::tfk::tfk_kernel::<F>(d, out, n);
    } else if comptime!(id == 25) {
        // XC_TW
        crate::functionals::lda::tw::tw_kernel::<F>(d, out, n);
    } else if comptime!(id == 28) {
        // XC_PW92C
        crate::functionals::lda::pw92c::pw92c_kernel::<F>(d, out, n);
    } else if comptime!(id == 55) {
        // XC_PZ81C
        crate::functionals::lda::pz81c::pz81c_kernel::<F>(d, out, n);
    } else if comptime!(id == 59) {
        // XC_VWK
        crate::functionals::lda::vwk::vwk_kernel::<F>(d, out, n);
    } else if comptime!(id == 69) {
        // XC_ZVPBESOLC
        crate::functionals::gga::pbe::zvpbesolc::zvpbesolc_kernel::<F>(d, out, n);
    } else if comptime!(id == 71) {
        // XC_PBEINTC
        crate::functionals::gga::pbe::pbeintc::pbeintc_kernel::<F>(d, out, n);
    } else if comptime!(id == 72) {
        // XC_PBEINTX
        crate::functionals::gga::pbe::pbeintx::pbeintx_kernel::<F>(d, out, n);
    } else if comptime!(id == 73) {
        // XC_PBELOCC
        crate::functionals::gga::pbe::pbelocc::pbelocc_kernel::<F>(d, out, n);
    } else if comptime!(id == 74) {
        // XC_PBESOLX
        crate::functionals::gga::pbe::pbesolx::pbesolx_kernel::<F>(d, out, n);
    } else if comptime!(id == 76) {
        // XC_ZVPBEINTC
        crate::functionals::gga::pbe::zvpbeintc::zvpbeintc_kernel::<F>(d, out, n);
    }
}

/// Host-side guard: returns true if `id` has an implemented kernel arm in
/// `dispatch_kernel`. Called by `Functional::eval` BEFORE launching, so stubs
/// (67 - 17 = 50 non-implemented) return `XcError::NotConfigured`.
///
/// Phase 2 ships 11 LDA ids; Phase 3 plan 03-02 adds 17 GGA ids:
///   {4, 5, 6, 7, 8, 9, 16, 19, 20, 21, 22, 69, 71, 72, 73, 74, 76}.
/// Total: 28 functional ids supported.
pub fn supports(id: FunctionalId) -> bool {
    matches!(
        id as u32,
        // Phase 2 LDAs (11)
        0 | 2 | 3 | 13 | 14 | 15 | 24 | 25 | 28 | 55 | 59
        // Phase 3 Wave-2 GGAs (17)
        | 4 | 5 | 6 | 7 | 8 | 9 | 16 | 19 | 20 | 21 | 22
        | 69 | 71 | 72 | 73 | 74 | 76
    )
}
