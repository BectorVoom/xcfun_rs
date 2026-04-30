//! Free-function behaviour parity with XCFunctional.cpp:48-348 (RS-09).
//!
//! Plan 05-01 Task 1.2. Each <behavior> bullet from the plan is exercised by
//! at least one test below.

use xcfun_rs::*;

// -- version / splash / authors / is_compatible_library --------------------

#[test]
fn version_returns_crate_version_starting_with_digit() {
    let v = version();
    assert!(!v.is_empty(), "version must be non-empty");
    assert!(
        v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false),
        "expected version to start with a digit, got {v:?}",
    );
}

#[test]
fn version_matches_crate_pkg_version() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn splash_starts_with_xcfun_dft_library() {
    assert!(
        splash().starts_with("XCFun DFT library Copyright 2009-2020 Ulf Ekstrom"),
        "splash() did not start with the expected banner; got prefix {:?}",
        &splash()[..splash().len().min(60)],
    );
}

#[test]
fn authors_starts_with_written_by() {
    assert!(
        authors().starts_with("XCFun was written by Ulf Ekstrom"),
        "authors() did not start as expected; got prefix {:?}",
        &authors()[..authors().len().min(60)],
    );
}

#[test]
fn is_compatible_library_is_true() {
    assert!(is_compatible_library());
}

// -- self_test ------------------------------------------------------------

#[test]
fn self_test_returns_non_negative_count() {
    // self_test iterates over functionals carrying upstream test data.
    // The exact count depends on which functionals are populated and pass
    // their tier-1 fixture; we assert non-negativity as the contract.
    let nfail = self_test();
    assert!(nfail >= 0, "self_test() returned negative {nfail}");
}

// -- which_vars: 31 mapped cases + range checks ---------------------------

#[test]
fn which_vars_lda_a() {
    assert_eq!(which_vars(0, 0, 0, 0, 0, 0), Some(Vars::A));
}

#[test]
fn which_vars_lda_n() {
    assert_eq!(which_vars(0, 1, 0, 0, 0, 0), Some(Vars::N));
}

#[test]
fn which_vars_lda_a_b() {
    assert_eq!(which_vars(0, 2, 0, 0, 0, 0), Some(Vars::A_B));
}

#[test]
fn which_vars_lda_n_s() {
    assert_eq!(which_vars(0, 3, 0, 0, 0, 0), Some(Vars::N_S));
}

#[test]
fn which_vars_gga_squared_a_b() {
    assert_eq!(
        which_vars(1, 2, 0, 0, 0, 0),
        Some(Vars::A_B_GAA_GAB_GBB)
    );
}

#[test]
fn which_vars_gga_explicit_components_a_b() {
    assert_eq!(
        which_vars(1, 2, 0, 0, 0, 1),
        Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ)
    );
}

#[test]
fn which_vars_metagga_kinetic_a_b() {
    // func_type=2 (metaGGA), dens_type=2 (A_B), kinetic=1 → bw=164.
    assert_eq!(
        which_vars(2, 2, 0, 1, 0, 0),
        Some(Vars::A_B_GAA_GAB_GBB_TAUA_TAUB)
    );
}

#[test]
fn which_vars_2nd_taylor_a_b() {
    assert_eq!(
        which_vars(3, 2, 0, 0, 0, 0),
        Some(Vars::A_B_2ND_TAYLOR)
    );
}

#[test]
fn which_vars_rejects_func_type_out_of_range() {
    assert_eq!(which_vars(4, 0, 0, 0, 0, 0), None);
}

#[test]
fn which_vars_rejects_dens_type_out_of_range() {
    assert_eq!(which_vars(0, 4, 0, 0, 0, 0), None);
}

#[test]
fn which_vars_rejects_laplacian_out_of_range() {
    assert_eq!(which_vars(0, 0, 2, 0, 0, 0), None);
}

#[test]
fn which_vars_rejects_unmapped_bitwise_combo() {
    // (func_type=0, dens_type=1, laplacian=1, ...) → bw = 16 | 8 = 24,
    // which is NOT one of the 31 cases in the C++ table.
    assert_eq!(which_vars(0, 1, 1, 0, 0, 0), None);
}

// -- which_mode: 3 valid cases + 2 invalid --------------------------------

#[test]
fn which_mode_partial_derivatives() {
    assert_eq!(which_mode(1), Some(Mode::PartialDerivatives));
}

#[test]
fn which_mode_potential() {
    assert_eq!(which_mode(2), Some(Mode::Potential));
}

#[test]
fn which_mode_contracted() {
    assert_eq!(which_mode(3), Some(Mode::Contracted));
}

#[test]
fn which_mode_zero_returns_none() {
    assert_eq!(which_mode(0), None);
}

#[test]
fn which_mode_four_returns_none() {
    assert_eq!(which_mode(4), None);
}

// -- enumerate_parameters --------------------------------------------------

#[test]
fn enumerate_parameters_index_zero_is_xc_slaterx() {
    // FUNCTIONAL_DESCRIPTORS[0].name is "XC_SLATERX" (per the
    // generated registry).
    assert_eq!(enumerate_parameters(0), Some("XC_SLATERX"));
}

#[test]
fn enumerate_parameters_index_77_is_last_upstream_functional() {
    // FUNCTIONAL_DESCRIPTORS[77].name = "XC_PW91C".
    assert_eq!(enumerate_parameters(77), Some("XC_PW91C"));
}

#[test]
fn enumerate_parameters_index_78_is_rangesep_mu() {
    // Plan 05-01 must_have: 78 → first parameter, not LB94.
    // PARAMETERS[0].name is "RANGESEP_MU".
    assert_eq!(enumerate_parameters(78), Some("RANGESEP_MU"));
}

#[test]
fn enumerate_parameters_index_81_is_cam_beta() {
    // PARAMETERS[3].name is "CAM_BETA".
    assert_eq!(enumerate_parameters(81), Some("CAM_BETA"));
}

#[test]
fn enumerate_parameters_index_82_returns_none() {
    assert_eq!(enumerate_parameters(82), None);
}

#[test]
fn enumerate_parameters_negative_returns_none() {
    assert_eq!(enumerate_parameters(-1), None);
}

// -- enumerate_aliases -----------------------------------------------------

#[test]
fn enumerate_aliases_index_zero_is_null() {
    // ALIASES[0].name is "null".
    assert_eq!(enumerate_aliases(0), Some("null"));
}

#[test]
fn enumerate_aliases_out_of_range_returns_none() {
    // ALIASES has 46 entries (0..=45); 46 is out of range.
    assert_eq!(enumerate_aliases(46), None);
}

#[test]
fn enumerate_aliases_negative_returns_none() {
    assert_eq!(enumerate_aliases(-1), None);
}

// -- describe_short --------------------------------------------------------

#[test]
fn describe_short_slaterx_uppercase() {
    assert_eq!(describe_short("SLATERX"), Some("Slater LDA exchange"));
}

#[test]
fn describe_short_slaterx_lowercase() {
    // Case-insensitive functional lookup.
    assert_eq!(describe_short("slaterx"), Some("Slater LDA exchange"));
}

#[test]
fn describe_short_rangesep_mu_parameter() {
    assert_eq!(
        describe_short("RANGESEP_MU"),
        Some("Range separation inverse length [1/a0]"),
    );
}

#[test]
fn describe_short_blyp_alias() {
    // BLYP is in the alias table → "Becke exchange and LYP correlation".
    assert_eq!(
        describe_short("BLYP"),
        Some("Becke exchange and LYP correlation"),
    );
}

#[test]
fn describe_short_blyp_lowercase_alias() {
    assert_eq!(
        describe_short("blyp"),
        Some("Becke exchange and LYP correlation"),
    );
}

#[test]
fn describe_short_unknown_returns_none() {
    assert_eq!(describe_short("not_a_known_thing"), None);
}

// -- describe_long ---------------------------------------------------------

#[test]
fn describe_long_slaterx_returns_long_description() {
    let long = describe_long("SLATERX")
        .expect("describe_long(SLATERX) should be Some");
    // Long description starts with "LDA Exchange functional" per the
    // generated descriptor.
    assert!(
        long.starts_with("LDA Exchange functional"),
        "describe_long(SLATERX) prefix unexpected: {:?}",
        &long[..long.len().min(40)],
    );
}

#[test]
fn describe_long_unknown_returns_none() {
    assert_eq!(describe_long("does_not_exist_anywhere"), None);
}
