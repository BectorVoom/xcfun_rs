//! Functional dispatcher — match-on-FunctionalId resolves the registry
//! circular-dep (xcfun-core has no fp-table; xcfun-eval owns dispatch).
//!
//! See RESEARCH.md §"Registry Shape + Circular-Dep Resolution" for rationale.
//!
//! Phase 2 dispatch arms (11 LDAs) + Phase 3 Wave-2 GGAs (17) + Wave-3 GGAs (10) +
//! Phase 3 Wave-4 GGAs (8):
//!   id ==  0 → XC_SLATERX     (Plan 02-04)
//!   id ==  1 → XC_PW86X       (Plan 03-03 — Wave 3)
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
//!   id == 17 → XC_OPTX        (Plan 03-03 — Wave 3)
//!   id == 18 → XC_OPTXCORR    (Plan 03-03 — Wave 3)
//!   id == 19 → XC_REVPBEX     (Plan 03-02 — Wave 2)
//!   id == 20 → XC_RPBEX       (Plan 03-02 — Wave 2)
//!   id == 21 → XC_SPBEC       (Plan 03-02 — Wave 2)
//!   id == 22 → XC_VWN_PBEC    (Plan 03-02 — Wave 2)
//!   id == 23 → XC_KTX         (Plan 03-04 — Wave 4)
//!   id == 24 → XC_TFK         (Plan 02-04)
//!   id == 25 → XC_TW          (Plan 02-05)
//!   id == 26 → XC_PW91X       (Plan 03-03 — Wave 3)
//!   id == 27 → XC_PW91K       (Plan 03-03 — Wave 3)
//!   id == 28 → XC_PW92C       (Plan 02-04)
//!   id == 55 → XC_PZ81C       (Plan 02-04)
//!   id == 56 → XC_P86C        (Plan 03-03 — Wave 3)
//!   id == 57 → XC_P86CORRC    (Plan 03-03 — Wave 3)
//!   id == 58 → XC_BTK         (Plan 03-04 — Wave 4)
//!   id == 59 → XC_VWK         (Plan 02-05)
//!   id == 60 → XC_B97X        (Plan 03-04 — Wave 4)
//!   id == 61 → XC_B97C        (Plan 03-04 — Wave 4)
//!   id == 62 → XC_B97_1X      (Plan 03-04 — Wave 4)
//!   id == 63 → XC_B97_1C      (Plan 03-04 — Wave 4)
//!   id == 64 → XC_B97_2X      (Plan 03-04 — Wave 4)
//!   id == 65 → XC_B97_2C      (Plan 03-04 — Wave 4)
//!   id == 67 → XC_APBEC       (Plan 03-03 — Wave 3)
//!   id == 68 → XC_APBEX       (Plan 03-03 — Wave 3)
//!   id == 69 → XC_ZVPBESOLC   (Plan 03-02 — Wave 2)
//!   id == 71 → XC_PBEINTC     (Plan 03-02 — Wave 2)
//!   id == 72 → XC_PBEINTX     (Plan 03-02 — Wave 2)
//!   id == 73 → XC_PBELOCC     (Plan 03-02 — Wave 2)
//!   id == 74 → XC_PBESOLX     (Plan 03-02 — Wave 2)
//!   id == 76 → XC_ZVPBEINTC   (Plan 03-02 — Wave 2)
//!   id == 77 → XC_PW91C       (Plan 03-03 — Wave 3)
//!   id == 41 → XC_TPSSC       (Plan 04-01 — Wave 1)
//!   id == 42 → XC_TPSSX       (Plan 04-01 — Wave 1)
//!   id == 43 → XC_REVTPSSC    (Plan 04-01 — Wave 1)
//!   id == 44 → XC_REVTPSSX    (Plan 04-01 — Wave 1)
//!   id == 75 → XC_TPSSLOCC    (Plan 04-01 — Wave 1)
//!   id == 45 → XC_SCANC       (Plan 04-02 — Wave 2)
//!   id == 46 → XC_SCANX       (Plan 04-02 — Wave 2)
//!   id == 47 → XC_RSCANC      (Plan 04-02 — Wave 2)
//!   id == 48 → XC_RSCANX      (Plan 04-02 — Wave 2)
//!   id == 49 → XC_RPPSCANC    (Plan 04-02 — Wave 2)
//!   id == 50 → XC_RPPSCANX    (Plan 04-02 — Wave 2)
//!   id == 51 → XC_R2SCANC     (Plan 04-02 — Wave 2)
//!   id == 52 → XC_R2SCANX     (Plan 04-02 — Wave 2)
//!   id == 53 → XC_R4SCANC     (Plan 04-02 — Wave 2)
//!   id == 54 → XC_R4SCANX     (Plan 04-02 — Wave 2)

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
    } else if comptime!(id == 1) {
        // XC_PW86X
        crate::functionals::gga::pw91::pw86x::pw86x_kernel::<F>(d, out, n);
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
    } else if comptime!(id == 17) {
        // XC_OPTX
        crate::functionals::gga::optx::optx::optx_kernel::<F>(d, out, n);
    } else if comptime!(id == 18) {
        // XC_OPTXCORR
        crate::functionals::gga::optx::optxcorr::optxcorr_kernel::<F>(d, out, n);
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
    } else if comptime!(id == 23) {
        // XC_KTX (Wave 4)
        crate::functionals::gga::kt::ktx::ktx_kernel::<F>(d, out, n);
    } else if comptime!(id == 24) {
        // XC_TFK
        crate::functionals::lda::tfk::tfk_kernel::<F>(d, out, n);
    } else if comptime!(id == 25) {
        // XC_TW
        crate::functionals::lda::tw::tw_kernel::<F>(d, out, n);
    } else if comptime!(id == 26) {
        // XC_PW91X
        crate::functionals::gga::pw91::pw91x::pw91x_kernel::<F>(d, out, n);
    } else if comptime!(id == 27) {
        // XC_PW91K
        crate::functionals::gga::pw91::pw91k::pw91k_kernel::<F>(d, out, n);
    } else if comptime!(id == 28) {
        // XC_PW92C
        crate::functionals::lda::pw92c::pw92c_kernel::<F>(d, out, n);
    } else if comptime!(id == 55) {
        // XC_PZ81C
        crate::functionals::lda::pz81c::pz81c_kernel::<F>(d, out, n);
    } else if comptime!(id == 56) {
        // XC_P86C
        crate::functionals::gga::p86::p86c::p86c_kernel::<F>(d, out, n);
    } else if comptime!(id == 57) {
        // XC_P86CORRC
        crate::functionals::gga::p86::p86corrc::p86corrc_kernel::<F>(d, out, n);
    } else if comptime!(id == 58) {
        // XC_BTK (Wave 4)
        crate::functionals::gga::kt::btk::btk_kernel::<F>(d, out, n);
    } else if comptime!(id == 59) {
        // XC_VWK
        crate::functionals::lda::vwk::vwk_kernel::<F>(d, out, n);
    } else if comptime!(id == 60) {
        // XC_B97X (Wave 4)
        crate::functionals::gga::b97::b97x::b97x_kernel::<F>(d, out, n);
    } else if comptime!(id == 61) {
        // XC_B97C (Wave 4)
        crate::functionals::gga::b97::b97c::b97c_kernel::<F>(d, out, n);
    } else if comptime!(id == 62) {
        // XC_B97_1X (Wave 4)
        crate::functionals::gga::b97::b97_1x::b97_1x_kernel::<F>(d, out, n);
    } else if comptime!(id == 63) {
        // XC_B97_1C (Wave 4)
        crate::functionals::gga::b97::b97_1c::b97_1c_kernel::<F>(d, out, n);
    } else if comptime!(id == 64) {
        // XC_B97_2X (Wave 4)
        crate::functionals::gga::b97::b97_2x::b97_2x_kernel::<F>(d, out, n);
    } else if comptime!(id == 65) {
        // XC_B97_2C (Wave 4)
        crate::functionals::gga::b97::b97_2c::b97_2c_kernel::<F>(d, out, n);
    } else if comptime!(id == 67) {
        // XC_APBEC
        crate::functionals::gga::apbe::apbec::apbec_kernel::<F>(d, out, n);
    } else if comptime!(id == 68) {
        // XC_APBEX
        crate::functionals::gga::apbe::apbex::apbex_kernel::<F>(d, out, n);
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
    } else if comptime!(id == 77) {
        // XC_PW91C
        crate::functionals::gga::pw91::pw91c::pw91c_kernel::<F>(d, out, n);
    } else if comptime!(id == 41) {
        // XC_TPSSC (Plan 04-01 Wave 1)
        crate::functionals::mgga::tpssc::tpssc_kernel::<F>(d, out, n);
    } else if comptime!(id == 42) {
        // XC_TPSSX (Plan 04-01 Wave 1)
        crate::functionals::mgga::tpssx::tpssx_kernel::<F>(d, out, n);
    } else if comptime!(id == 43) {
        // XC_REVTPSSC (Plan 04-01 Wave 1)
        crate::functionals::mgga::revtpssc::revtpssc_kernel::<F>(d, out, n);
    } else if comptime!(id == 44) {
        // XC_REVTPSSX (Plan 04-01 Wave 1)
        crate::functionals::mgga::revtpssx::revtpssx_kernel::<F>(d, out, n);
    } else if comptime!(id == 75) {
        // XC_TPSSLOCC (Plan 04-01 Wave 1)
        crate::functionals::mgga::tpsslocc::tpsslocc_kernel::<F>(d, out, n);
    } else if comptime!(id == 10) {
        // XC_BRX (Plan 04-01 Wave 1)
        crate::functionals::mgga::brx::brx_kernel::<F>(d, out, n);
    } else if comptime!(id == 11) {
        // XC_BRC (Plan 04-01 Wave 1)
        crate::functionals::mgga::brx::brc_kernel::<F>(d, out, n);
    } else if comptime!(id == 12) {
        // XC_BRXC (Plan 04-01 Wave 1)
        crate::functionals::mgga::brx::brxc_kernel::<F>(d, out, n);
    } else if comptime!(id == 66) {
        // XC_CSC (Plan 04-01 Wave 1)
        crate::functionals::mgga::csc::csc_kernel::<F>(d, out, n);
    } else if comptime!(id == 45) {
        // XC_SCANC (Plan 04-02 Wave 2)
        crate::functionals::mgga::scanc::scanc_kernel::<F>(d, out, n);
    } else if comptime!(id == 46) {
        // XC_SCANX (Plan 04-02 Wave 2)
        crate::functionals::mgga::scanx::scanx_kernel::<F>(d, out, n);
    } else if comptime!(id == 47) {
        // XC_RSCANC (Plan 04-02 Wave 2)
        crate::functionals::mgga::rscanc::rscanc_kernel::<F>(d, out, n);
    } else if comptime!(id == 48) {
        // XC_RSCANX (Plan 04-02 Wave 2)
        crate::functionals::mgga::rscanx::rscanx_kernel::<F>(d, out, n);
    } else if comptime!(id == 49) {
        // XC_RPPSCANC (Plan 04-02 Wave 2)
        crate::functionals::mgga::rppscanc::rppscanc_kernel::<F>(d, out, n);
    } else if comptime!(id == 50) {
        // XC_RPPSCANX (Plan 04-02 Wave 2)
        crate::functionals::mgga::rppscanx::rppscanx_kernel::<F>(d, out, n);
    } else if comptime!(id == 51) {
        // XC_R2SCANC (Plan 04-02 Wave 2)
        crate::functionals::mgga::r2scanc::r2scanc_kernel::<F>(d, out, n);
    } else if comptime!(id == 52) {
        // XC_R2SCANX (Plan 04-02 Wave 2)
        crate::functionals::mgga::r2scanx::r2scanx_kernel::<F>(d, out, n);
    } else if comptime!(id == 53) {
        // XC_R4SCANC (Plan 04-02 Wave 2)
        crate::functionals::mgga::r4scanc::r4scanc_kernel::<F>(d, out, n);
    } else if comptime!(id == 54) {
        // XC_R4SCANX (Plan 04-02 Wave 2)
        crate::functionals::mgga::r4scanx::r4scanx_kernel::<F>(d, out, n);
    }
}

/// Host-side guard: returns true if `id` has an implemented kernel arm in
/// `dispatch_kernel`. Called by `Functional::eval` BEFORE launching, so stubs
/// (67 - 17 = 50 non-implemented) return `XcError::NotConfigured`.
///
/// Phase 2 ships 11 LDA ids; Phase 3 plan 03-02 adds 17 GGA ids; plan 03-03
/// adds 10 more GGAs (OPTX×2 + PW86/91×4 + P86×2 + APBE×2); plan 03-04 adds
/// 8 more GGAs (B97×6 + KTX + BTK):
///   {23, 58, 60, 61, 62, 63, 64, 65}.
/// Phase 4 plan 04-01 Wave 1 adds 5 metaGGA ids (TPSS family):
///   {41, 42, 43, 44, 75}.
/// Phase 4 plan 04-01 Task 2 adds BR family + CSC (4):
///   {10, 11, 12, 66}.
/// Phase 4 plan 04-02 Wave 2 adds SCAN family (10):
///   {45, 46, 47, 48, 49, 50, 51, 52, 53, 54}.
/// Total: 65 functional ids supported.
pub fn supports(id: FunctionalId) -> bool {
    matches!(
        id as u32,
        // Phase 2 LDAs (11)
        0 | 2 | 3 | 13 | 14 | 15 | 24 | 25 | 28 | 55 | 59
        // Phase 3 Wave-2 GGAs (17)
        | 4 | 5 | 6 | 7 | 8 | 9 | 16 | 19 | 20 | 21 | 22
        | 69 | 71 | 72 | 73 | 74 | 76
        // Phase 3 Wave-3 GGAs (10)
        | 1 | 17 | 18 | 26 | 27 | 56 | 57 | 67 | 68 | 77
        // Phase 3 Wave-4 GGAs (8: B97 family + KTX + BTK)
        | 23 | 58 | 60 | 61 | 62 | 63 | 64 | 65
        // Phase 4 Wave-1 metaGGAs: TPSS family (5)
        | 41 | 42 | 43 | 44 | 75
        // Phase 4 Wave-1 carryovers: BR family (3) + CSC (1)
        | 10 | 11 | 12 | 66
        // Phase 4 Wave-2: SCAN family (10)
        | 45 | 46 | 47 | 48 | 49 | 50 | 51 | 52 | 53 | 54
    )
}
