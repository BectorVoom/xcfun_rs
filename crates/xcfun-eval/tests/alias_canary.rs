//! Plan 04-04 Task 2 — alias engine canary tests.
//!
//! Verifies `Functional::set/get` implementing the 3-case recursion from
//! `xcfun-master/src/XCFunctional.cpp:369-405`:
//!   case 1 — functional name match → settings[id] += value (additive)
//!   case 2 — parameter name match  → settings[id]  = value (overwrite)
//!   case 3 — alias    name match   → recurse over alias.terms with
//!                                    value * term_weight
//!
//! Acceptance criteria 1-7 from PLAN 04-04 Task 4.2 §acceptance_criteria.

use xcfun_core::{ALIASES, FunctionalId, ParameterId, XcError};
use xcfun_eval::functional::Functional;

#[test]
fn test_b3lyp_trace() {
    // aliases.cpp:42-48 — b3lyp = 0.80*slaterx + 0.72*beckecorrx
    //                          + 0.81*lypc + 0.19*vwn5c + 0.20*exx
    let mut f = Functional::new();
    f.set("b3lyp", 1.0).unwrap();
    assert!((f.get("slaterx").unwrap() - 0.80).abs() < 1e-15);
    assert!((f.get("beckecorrx").unwrap() - 0.72).abs() < 1e-15);
    assert!((f.get("lypc").unwrap() - 0.81).abs() < 1e-15);
    assert!((f.get("vwn5c").unwrap() - 0.19).abs() < 1e-15);
    // exx is a parameter — overwrite semantics, single-shot value.
    assert!((f.get("exx").unwrap() - 0.20).abs() < 1e-15);
}

#[test]
fn test_b3lyp_additive_accumulation() {
    // Two consecutive functional sets must accumulate per
    // XCFunctional.cpp:373: `fun->settings[item] += value;`.
    // After set("b3lyp", 1.0) -> settings[SLATERX] == 0.80.
    // After set("slaterx", 0.5) -> settings[SLATERX] == 1.30.
    let mut f = Functional::new();
    f.set("b3lyp", 1.0).unwrap();
    f.set("slaterx", 0.5).unwrap();
    let w = f.get("slaterx").unwrap();
    assert!(
        (w - 1.30).abs() < 1e-15,
        "additive accumulation broken: expected 1.30, got {}",
        w
    );
}

#[test]
fn test_exx_parameter_overwrite() {
    // Parameter case — XCFunctional.cpp:381 does `=`, not `+=`.
    // set("b3lyp", 0.5) recurses set("exx", 0.5 * 0.20) = set("exx", 0.10).
    // Per parameter overwrite semantics, settings[XC_EXX] = 0.10
    // (NOT 0.0 + 0.10; not 0.20 + 0.10; the result is exactly 0.10).
    let mut f = Functional::new();
    f.set("b3lyp", 0.5).unwrap();
    let exx = f.get("exx").unwrap();
    assert!(
        (exx - 0.10).abs() < 1e-15,
        "parameter overwrite broken: expected 0.10, got {}",
        exx
    );
}

#[test]
fn test_camcompx_negative_weight() {
    // aliases.cpp:119-125 — camcompx terms include {"beckecamx", -1.0}.
    // set("camcompx", 0.37) → settings[BECKECAMX] += 0.37 * -1.0 = -0.37.
    let mut f = Functional::new();
    f.set("camcompx", 0.37).unwrap();
    let w = f.get("beckecamx").unwrap();
    assert!(
        (w - (-0.37)).abs() < 1e-15,
        "negative-weight propagation broken: expected -0.37, got {}",
        w
    );
}

#[test]
fn test_camcompx_parameter_overwrite_with_value_weight() {
    // C++ FIXME at XCFunctional.cpp:393: parameters ARE multiplied by value
    // when going through an alias. set("camcompx", 0.37) overwrites
    // settings[XC_RANGESEP_MU] = 0.37 * 0.33 = 0.1221 (not the default 0.4).
    let mut f = Functional::new();
    f.set("camcompx", 0.37).unwrap();
    let mu = f.get("rangesep_mu").unwrap();
    assert!(
        (mu - (0.37 * 0.33)).abs() < 1e-15,
        "alias parameter overwrite broken: expected 0.1221, got {}",
        mu
    );
}

#[test]
fn test_case_insensitive() {
    // D-04-B: case-insensitive lookup — "B3LYP" and "b3lyp" must produce
    // identical settings.
    let mut f1 = Functional::new();
    let mut f2 = Functional::new();
    f1.set("b3lyp", 1.0).unwrap();
    f2.set("B3LYP", 1.0).unwrap();
    for i in 0..82 {
        assert_eq!(
            f1.settings[i], f2.settings[i],
            "case-insensitive lookup broken at settings[{}]",
            i
        );
    }
}

#[test]
fn test_parameter_defaults_after_new() {
    // Functional::new() seeds settings[78..=81] with the defaults from
    // common_parameters.cpp:17-29.
    let f = Functional::new();
    assert_eq!(f.get("rangesep_mu").unwrap(), 0.4);
    assert_eq!(f.get("exx").unwrap(), 0.0);
    assert_eq!(f.get("cam_alpha").unwrap(), 0.19);
    assert_eq!(f.get("cam_beta").unwrap(), 0.46);
    // Direct settings[] reads — same values via index.
    assert_eq!(f.settings[ParameterId::XC_RANGESEP_MU as usize], 0.4);
    assert_eq!(f.settings[ParameterId::XC_EXX as usize], 0.0);
    assert_eq!(f.settings[ParameterId::XC_CAM_ALPHA as usize], 0.19);
    assert_eq!(f.settings[ParameterId::XC_CAM_BETA as usize], 0.46);
}

#[test]
fn test_functional_settings_zeroed_after_new() {
    // settings[0..78] must initialize to 0.0 — only parameters get defaults.
    let f = Functional::new();
    for i in 0..78 {
        assert_eq!(
            f.settings[i], 0.0,
            "functional slot {} should be zero after new()",
            i
        );
    }
}

#[test]
fn test_all_46_aliases_resolve() {
    // No alias entry must trigger UnknownName when invoked with value=1.0.
    // Aliases that share a name with a functional (OPTX, PBEX) route to the
    // functional case (lookup-priority order); both still return Ok.
    for alias in ALIASES.iter() {
        let mut f = Functional::new();
        let res = f.set(alias.name, 1.0);
        assert!(
            res.is_ok(),
            "alias '{}' returned {:?}",
            alias.name,
            res.err()
        );
    }
}

#[test]
fn test_unknown_name_returns_error() {
    let mut f = Functional::new();
    let res = f.set("definitely_not_a_functional_xyz", 1.0);
    assert!(matches!(res, Err(XcError::UnknownName)));
}

#[test]
fn test_get_unknown_name_returns_error() {
    let f = Functional::new();
    let res = f.get("definitely_not_a_functional_xyz");
    assert!(matches!(res, Err(XcError::UnknownName)));
}

#[test]
fn test_get_alias_returns_unknown_name() {
    // C++ xcfun_get (XCFunctional.cpp:407-419) returns -1 for alias names.
    // Aliases are NOT readable via get(); only functional and parameter names.
    let f = Functional::new();
    // Use an alias name that is NOT also a functional or parameter name.
    // "blyp", "b3lyp", "camcompx" are pure aliases (no FunctionalId conflict).
    let res = f.get("blyp");
    assert!(
        matches!(res, Err(XcError::UnknownName)),
        "alias 'blyp' should not be readable via get(), got {:?}",
        res
    );
    let res = f.get("camcompx");
    assert!(matches!(res, Err(XcError::UnknownName)));
    let res = f.get("b3lyp");
    assert!(matches!(res, Err(XcError::UnknownName)));
}

#[test]
fn test_set_functional_directly_additive() {
    // Direct functional set is additive (case 1 of xcfun_set).
    let mut f = Functional::new();
    f.set("slaterx", 0.5).unwrap();
    f.set("slaterx", 0.3).unwrap();
    let w = f.get("slaterx").unwrap();
    assert!((w - 0.8).abs() < 1e-15);
}

#[test]
fn test_set_parameter_directly_overwrite() {
    // Direct parameter set is overwrite (case 2 of xcfun_set).
    let mut f = Functional::new();
    f.set("exx", 0.25).unwrap();
    f.set("exx", 0.15).unwrap();
    assert_eq!(f.get("exx").unwrap(), 0.15);
}

#[test]
fn test_set_via_xc_prefix_aliased_functional() {
    // Both bare and XC_-prefixed names must work for functional case 1.
    let mut f1 = Functional::new();
    let mut f2 = Functional::new();
    f1.set("slaterx", 1.0).unwrap();
    f2.set("XC_SLATERX", 1.0).unwrap();
    assert_eq!(f1.settings[FunctionalId::XC_SLATERX as usize], 1.0);
    assert_eq!(f2.settings[FunctionalId::XC_SLATERX as usize], 1.0);
}

#[test]
fn test_set_via_xc_prefix_aliased_parameter() {
    // Both bare and XC_-prefixed names must work for parameter case 2.
    let mut f1 = Functional::new();
    let mut f2 = Functional::new();
    f1.set("rangesep_mu", 0.5).unwrap();
    f2.set("XC_RANGESEP_MU", 0.5).unwrap();
    assert_eq!(f1.get("rangesep_mu").unwrap(), 0.5);
    assert_eq!(f2.get("rangesep_mu").unwrap(), 0.5);
}
