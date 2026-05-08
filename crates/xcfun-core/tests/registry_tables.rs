//! Integration tests for the auto-generated registry tables (CORE-07,
//! CORE-08, CORE-09, CORE-10).
//!
//! Source of truth for these assertions:
//!   * xcfun-master/src/functionals/*.cpp — FUNCTIONAL macro payloads
//!   * xcfun-master/src/xcint.cpp:93-135  — xcint_vars[] table
//!   * xcfun-master/src/functionals/aliases.cpp — Phase-2 empty, Phase-4 populates

use xcfun_core::{ALIASES, Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, VARS_TABLE};

#[test]
fn descriptors_count_is_79() {
    // Plan 05-00 D-16: upstream 78 + Rust-only XC_LB94 stub.
    assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), 79);
    assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), FunctionalId::COUNT);
}

#[test]
fn lb94_descriptor_present() {
    // D-16 — descriptor exists at row 78; depends mask matches
    // setup_lb94 macro (xcfun-master/src/functionals/lb94.cpp:48-50).
    let lb94 = &FUNCTIONAL_DESCRIPTORS[78];
    assert_eq!(lb94.id, FunctionalId::XC_LB94);
    assert_eq!(lb94.name, "XC_LB94");
    assert!(lb94.depends.contains(Dependency::DENSITY));
    assert!(lb94.depends.contains(Dependency::GRADIENT));
    // No test data for LB94 (body is #if 0'd upstream).
    assert!(lb94.test_in.is_none());
    assert!(lb94.test_out.is_none());
}

#[test]
fn descriptors_match_functional_id_ordering() {
    for (i, desc) in FUNCTIONAL_DESCRIPTORS.iter().enumerate() {
        assert_eq!(
            desc.id as u32, i as u32,
            "descriptor at index {} has id discriminant {} (expected {})",
            i, desc.id as u32, i
        );
    }
}

#[test]
fn lda_descriptors_have_test_data() {
    // Seven LDA functionals with upstream `test_in`/`test_out`:
    //   SLATERX (id 0), VWN5C (3), PW92C (28), PZ81C (55), LDAERFX (13),
    //   LDAERFC (14), TFK (24).
    // (Upstream VWN3C / LDAERFC_JT / TW / VWK intentionally ship no test data
    // in their FUNCTIONAL macro — see `lda_descriptors_without_upstream_data`
    // below.)
    for id in [
        FunctionalId::XC_SLATERX,
        FunctionalId::XC_VWN5C,
        FunctionalId::XC_PW92C,
        FunctionalId::XC_PZ81C,
        FunctionalId::XC_TFK,
        FunctionalId::XC_LDAERFX,
        FunctionalId::XC_LDAERFC,
    ] {
        let desc = &FUNCTIONAL_DESCRIPTORS[id as usize];
        assert!(desc.test_in.is_some(), "{:?} should have test_in", id);
        assert!(desc.test_out.is_some(), "{:?} should have test_out", id);
        assert!(
            desc.test_threshold.is_some(),
            "{:?} should have test_threshold",
            id
        );
        assert!(desc.test_vars.is_some(), "{:?} should have test_vars", id);
    }
}

#[test]
fn ldaerf_uses_d24_threshold() {
    // D-24: upstream xcfun-master ships LDAERFX/LDAERFC at 1e-7.
    for id in [FunctionalId::XC_LDAERFX, FunctionalId::XC_LDAERFC] {
        let desc = &FUNCTIONAL_DESCRIPTORS[id as usize];
        assert_eq!(
            desc.test_threshold,
            Some(1e-7),
            "{:?} test_threshold should be 1e-7 (D-24, sourced from upstream)",
            id
        );
    }
}

#[test]
fn vars_table_count_is_31() {
    assert_eq!(VARS_TABLE.len(), 31);
}

#[test]
fn vars_table_xc_a_b_anchor() {
    // xcint.cpp:95 — XC_A_B is row 2 (after XC_A row 0, XC_N row 1).
    assert_eq!(VARS_TABLE[2].symbol, "XC_A_B");
    assert_eq!(VARS_TABLE[2].len, 2);
}

#[test]
fn vars_table_xc_a_b_gaa_gab_gbb_anchor() {
    // xcint.cpp — XC_A_B_GAA_GAB_GBB is row 6 (5-input GGA used by TW + VWK).
    assert_eq!(VARS_TABLE[6].symbol, "XC_A_B_GAA_GAB_GBB");
    assert_eq!(VARS_TABLE[6].len, 5);
}

#[test]
fn aliases_populated_in_phase_4() {
    // Phase 4 (Plan 04-04 D-04): 46 alias entries populated from
    // xcfun-master/src/functionals/aliases.cpp:17-138.
    assert_eq!(ALIASES.len(), 46);
}

#[test]
fn lda_descriptors_without_upstream_data() {
    // VWN3C, LDAERFC_JT, TW, VWK are LDA-tier (or LDA-kinetic) functionals
    // whose upstream FUNCTIONAL macro ends before the optional test
    // {test_vars, test_mode, test_order, test_threshold, test_in, test_out}
    // tail — so the registry row is a depends-populated stub with
    // test_in=None. Tier-2 vs C++ (Plan 02-06) covers them.
    for id in [
        FunctionalId::XC_VWN3C,
        FunctionalId::XC_LDAERFC_JT,
        FunctionalId::XC_TW,
        FunctionalId::XC_VWK,
    ] {
        let desc = &FUNCTIONAL_DESCRIPTORS[id as usize];
        assert!(
            desc.test_in.is_none(),
            "{:?} carries no upstream test_in",
            id
        );
        assert!(
            desc.test_out.is_none(),
            "{:?} carries no upstream test_out",
            id
        );
    }
}
