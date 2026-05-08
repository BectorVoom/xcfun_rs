//! Plan 04-04 Task 1 — RED tests for the parameter + alias registries
//! and the `ParameterId` enum.
//!
//! These tests assert the contracts spelled out in
//! `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-04-alias-parameters-PLAN.md`
//! Task 4.1 `<behavior>`. They MUST fail before implementation and pass after
//! Task 4.1's GREEN step.
//!
//! Source-of-truth references (verbatim):
//!   - `xcfun-master/src/functionals/list_of_functionals.hpp:99-105` — parameter
//!     discriminants 78..=81 (XC_RANGESEP_MU, XC_EXX, XC_CAM_ALPHA, XC_CAM_BETA).
//!   - `xcfun-master/src/functionals/common_parameters.cpp:17-29` — defaults
//!     {0.4, 0.0, 0.19, 0.46}.
//!   - `xcfun-master/src/functionals/aliases.cpp:17-138` — 46 alias entries.

use xcfun_core::{ALIASES, PARAMETERS, ParameterEntry, ParameterId};

// -----------------------------------------------------------------------
//  ParameterId discriminants — Plan 04-04 §behavior tests 1-4.
// -----------------------------------------------------------------------

#[test]
fn parameter_id_discriminants_match_cpp() {
    // list_of_functionals.hpp:99-104:
    //   enum xc_parameter {
    //     XC_RANGESEP_MU = XC_NR_FUNCTIONALS,  // = 78
    //     XC_EXX,                              // 79
    //     XC_CAM_ALPHA,                        // 80
    //     XC_CAM_BETA,                         // 81
    //     XC_NR_PARAMETERS_AND_FUNCTIONALS    // 82
    //   };
    assert_eq!(ParameterId::XC_RANGESEP_MU as u32, 78);
    assert_eq!(ParameterId::XC_EXX as u32, 79);
    assert_eq!(ParameterId::XC_CAM_ALPHA as u32, 80);
    assert_eq!(ParameterId::XC_CAM_BETA as u32, 81);
}

#[test]
fn parameter_id_from_name_case_insensitive() {
    // D-04-B: case-insensitive lookup. C++ `xcint_lookup_parameter` uses
    // `strcasecmp` against the symbol minus its `XC_` prefix.
    assert_eq!(
        ParameterId::from_name("rangesep_mu"),
        Some(ParameterId::XC_RANGESEP_MU)
    );
    assert_eq!(
        ParameterId::from_name("RANGESEP_MU"),
        Some(ParameterId::XC_RANGESEP_MU)
    );
    assert_eq!(
        ParameterId::from_name("XC_RANGESEP_MU"),
        Some(ParameterId::XC_RANGESEP_MU)
    );
    assert_eq!(ParameterId::from_name("exx"), Some(ParameterId::XC_EXX));
    assert_eq!(ParameterId::from_name("EXX"), Some(ParameterId::XC_EXX));
    assert_eq!(ParameterId::from_name("XC_EXX"), Some(ParameterId::XC_EXX));
    assert_eq!(
        ParameterId::from_name("cam_alpha"),
        Some(ParameterId::XC_CAM_ALPHA)
    );
    assert_eq!(
        ParameterId::from_name("Cam_Alpha"),
        Some(ParameterId::XC_CAM_ALPHA)
    );
    assert_eq!(
        ParameterId::from_name("XC_CAM_ALPHA"),
        Some(ParameterId::XC_CAM_ALPHA)
    );
    assert_eq!(
        ParameterId::from_name("cam_beta"),
        Some(ParameterId::XC_CAM_BETA)
    );
    assert_eq!(
        ParameterId::from_name("XC_CAM_BETA"),
        Some(ParameterId::XC_CAM_BETA)
    );
    assert_eq!(ParameterId::from_name("not_a_parameter"), None);
}

#[test]
fn parameter_id_default_values_match_cpp() {
    // common_parameters.cpp:17, 19-21, 23-25, 27-29.
    assert_eq!(ParameterId::XC_RANGESEP_MU.default_value(), 0.4);
    assert_eq!(ParameterId::XC_EXX.default_value(), 0.0);
    assert_eq!(ParameterId::XC_CAM_ALPHA.default_value(), 0.19);
    assert_eq!(ParameterId::XC_CAM_BETA.default_value(), 0.46);
}

// -----------------------------------------------------------------------
//  PARAMETERS registry — Plan 04-04 §behavior test 7.
// -----------------------------------------------------------------------

#[test]
fn parameters_registry_has_4_entries() {
    assert_eq!(PARAMETERS.len(), 4);
}

#[test]
fn parameters_registry_defaults_in_order() {
    // Order matches list_of_functionals.hpp:99-104.
    assert_eq!(PARAMETERS[0].id, ParameterId::XC_RANGESEP_MU);
    assert_eq!(PARAMETERS[0].default, 0.4);
    assert_eq!(PARAMETERS[1].id, ParameterId::XC_EXX);
    assert_eq!(PARAMETERS[1].default, 0.0);
    assert_eq!(PARAMETERS[2].id, ParameterId::XC_CAM_ALPHA);
    assert_eq!(PARAMETERS[2].default, 0.19);
    assert_eq!(PARAMETERS[3].id, ParameterId::XC_CAM_BETA);
    assert_eq!(PARAMETERS[3].default, 0.46);
}

#[test]
fn parameters_registry_struct_shape() {
    // ParameterEntry has the 4 expected fields (id, name, description, default).
    let p: &ParameterEntry = &PARAMETERS[0];
    assert!(!p.name.is_empty());
    assert!(!p.description.is_empty());
}

#[test]
fn parameter_names_match_cpp_strip_xc_prefix() {
    // C++ `pardat_db<P>::d.name = pardat_db<P>::symbol + 3;` — strips "XC_".
    // So lookup names are "RANGESEP_MU", "EXX", "CAM_ALPHA", "CAM_BETA"
    // (case-insensitive).
    let names: Vec<&str> = PARAMETERS.iter().map(|p| p.name).collect();
    assert!(names.iter().any(|n| n.eq_ignore_ascii_case("rangesep_mu")));
    assert!(names.iter().any(|n| n.eq_ignore_ascii_case("exx")));
    assert!(names.iter().any(|n| n.eq_ignore_ascii_case("cam_alpha")));
    assert!(names.iter().any(|n| n.eq_ignore_ascii_case("cam_beta")));
}

// -----------------------------------------------------------------------
//  ALIASES registry — Plan 04-04 §behavior tests 5, 6.
// -----------------------------------------------------------------------

#[test]
fn aliases_registry_has_46_entries() {
    // aliases.cpp:17-138 — exactly 46 alias entries.
    assert_eq!(
        ALIASES.len(),
        46,
        "expected 46 aliases, got {}",
        ALIASES.len()
    );
}

#[test]
fn aliases_contains_camcompx_with_negative_beckecamx() {
    // aliases.cpp:119-125 — camcompx canary, negative-weight term.
    let alias = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("camcompx"))
        .expect("camcompx alias missing");
    let neg = alias
        .components
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("beckecamx"))
        .expect("camcompx must contain a beckecamx term");
    assert_eq!(
        neg.1, -1.0,
        "beckecamx term in camcompx must have weight -1.0"
    );
}

#[test]
fn aliases_contains_b3lyp_with_documented_weights() {
    // aliases.cpp:42-48 — b3lyp alias.
    let alias = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("b3lyp"))
        .expect("b3lyp alias missing");
    let by_name = |needle: &str| -> f64 {
        alias
            .components
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(needle))
            .map(|(_, w)| *w)
            .unwrap_or_else(|| panic!("b3lyp missing term {}", needle))
    };
    assert_eq!(by_name("slaterx"), 0.80);
    assert_eq!(by_name("beckecorrx"), 0.72);
    assert_eq!(by_name("lypc"), 0.81);
    assert_eq!(by_name("vwn5c"), 0.19);
    assert_eq!(by_name("exx"), 0.20);
}

#[test]
fn aliases_contains_kt2_with_full_precision_weights() {
    // aliases.cpp:26-28 — kt2 with non-trivial precision (1.07173, 0.576727).
    let alias = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("kt2"))
        .expect("kt2 alias missing");
    let by_name = |needle: &str| -> f64 {
        alias
            .components
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(needle))
            .map(|(_, w)| *w)
            .unwrap_or_else(|| panic!("kt2 missing term {}", needle))
    };
    assert_eq!(by_name("slaterx"), 1.07173);
    assert_eq!(by_name("ktx"), -0.006);
    assert_eq!(by_name("vwn5c"), 0.576727);
}

#[test]
fn aliases_contains_camb3lyp_with_parameter_terms() {
    // aliases.cpp:83-91 — camb3lyp uses parameter terms (cam_alpha, cam_beta,
    // rangesep_mu) intermingled with functional terms.
    let alias = ALIASES
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("camb3lyp"))
        .expect("camb3lyp alias missing");
    let by_name = |needle: &str| -> f64 {
        alias
            .components
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(needle))
            .map(|(_, w)| *w)
            .unwrap_or_else(|| panic!("camb3lyp missing term {}", needle))
    };
    assert_eq!(by_name("cam_alpha"), 0.19);
    assert_eq!(by_name("cam_beta"), 0.46);
    assert_eq!(by_name("rangesep_mu"), 0.33);
    assert_eq!(by_name("beckecamx"), 1.0);
    assert_eq!(by_name("vwn5c"), 0.19);
    assert_eq!(by_name("lypc"), 0.81);
    assert_eq!(by_name("exx"), 1.0);
}

#[test]
fn aliases_all_terms_resolve_to_known_names() {
    // Every alias term name must resolve to either a FunctionalId or a
    // ParameterId. If neither, the alias table or the lookup tables drifted.
    for alias in ALIASES.iter() {
        for (term_name, _) in alias.components.iter() {
            let fid = xcfun_core::FunctionalId::from_name(term_name);
            let pid = ParameterId::from_name(term_name);
            assert!(
                fid.is_some() || pid.is_some(),
                "alias '{}' references unknown term '{}'",
                alias.name,
                term_name
            );
        }
    }
}
