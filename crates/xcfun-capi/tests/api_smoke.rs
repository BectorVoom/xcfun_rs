//! Phase 5 D-08 — Rust-side smoke test exercising every C entry point
//! through its FFI signature. Plan 05-04 adds the actual C-source
//! golden test (`tests/c_abi.c`).

use std::ffi::{CStr, CString};

use xcfun_capi::*;

fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[test]
fn xcfun_new_and_delete_null_safe() {
    let fun = xcfun_new();
    assert!(!fun.is_null());
    xcfun_delete(fun);
    // NULL-safe — must not abort.
    xcfun_delete(std::ptr::null_mut());
}

#[test]
fn xcfun_version_returns_digit_string() {
    let p = xcfun_version();
    assert!(!p.is_null());
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
    assert!(!s.is_empty());
    assert!(s.chars().next().unwrap().is_ascii_digit(), "got {s:?}");
}

#[test]
fn xcfun_splash_and_authors_non_null() {
    let p = xcfun_splash();
    assert!(!p.is_null());
    let p2 = xcfun_authors();
    assert!(!p2.is_null());
}

#[test]
fn xcfun_test_returns_non_negative() {
    let n = xcfun_test();
    assert!(n >= 0, "expected non-negative, got {n}");
}

#[test]
fn xcfun_is_compatible_library_returns_true() {
    assert!(xcfun_is_compatible_library());
}

#[test]
fn xcfun_which_vars_a_b_returns_two() {
    // (func_type=0=LDA, dens_type=2=A_B, ...) → XC_A_B = 2.
    let v = xcfun_which_vars(0, 2, 0, 0, 0, 0);
    assert_eq!(v, 2);
}

#[test]
fn xcfun_which_mode_potential_returns_two() {
    let m = xcfun_which_mode(2);
    assert_eq!(m, 2);
}

#[test]
fn xcfun_enumerate_parameters_in_range_non_null_out_of_range_null() {
    let p0 = xcfun_enumerate_parameters(0);
    assert!(!p0.is_null());
    let p82 = xcfun_enumerate_parameters(82);
    assert!(p82.is_null());
    let pneg = xcfun_enumerate_parameters(-1);
    assert!(pneg.is_null());
}

#[test]
fn xcfun_enumerate_aliases_in_range_non_null_out_of_range_null() {
    let p0 = xcfun_enumerate_aliases(0);
    assert!(!p0.is_null());
    let p46 = xcfun_enumerate_aliases(46);
    assert!(p46.is_null());
    let pneg = xcfun_enumerate_aliases(-1);
    assert!(pneg.is_null());
}

#[test]
fn xcfun_describe_short_known_and_unknown() {
    let known = cstr("SLATERX");
    let p = xcfun_describe_short(known.as_ptr());
    assert!(!p.is_null());
    let unknown = cstr("not_a_known_thing_at_all_xyz");
    let p2 = xcfun_describe_short(unknown.as_ptr());
    assert!(p2.is_null());
}

#[test]
fn xcfun_describe_long_known() {
    let known = cstr("SLATERX");
    let p = xcfun_describe_long(known.as_ptr());
    assert!(!p.is_null());
}

#[test]
fn xcfun_full_happy_path_lda() {
    let fun = xcfun_new();
    let name = cstr("slaterx");
    assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);

    // Vars::A_B = 2; Mode::PartialDerivatives = 1; order = 0.
    assert_eq!(xcfun_eval_setup(fun, 2, 1, 0), 0);
    assert_eq!(xcfun_input_length(fun), 2);
    assert_eq!(xcfun_output_length(fun), 1);

    let density: [f64; 2] = [0.5, 0.5];
    let mut result: [f64; 1] = [0.0];
    xcfun_eval(fun, density.as_ptr(), result.as_mut_ptr());
    assert!(
        result[0].abs() > 1e-9,
        "expected non-zero result, got {}",
        result[0]
    );

    let mut got: f64 = 0.0;
    assert_eq!(xcfun_get(fun, name.as_ptr(), &mut got as *mut f64), 0);
    assert_eq!(got, 1.0);

    xcfun_delete(fun);
}

#[test]
fn xcfun_set_unknown_returns_minus_one() {
    let fun = xcfun_new();
    let bad = cstr("not_a_known_name");
    assert_eq!(xcfun_set(fun, bad.as_ptr(), 1.0), -1);
    xcfun_delete(fun);
}

#[test]
fn xcfun_user_eval_setup_lda_a_b() {
    let fun = xcfun_new();
    let name = cstr("slaterx");
    assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
    // order=0, func_type=0 (LDA), dens_type=2 (A_B), mode_type=1
    // (PartialDerivatives), all flags 0.
    assert_eq!(xcfun_user_eval_setup(fun, 0, 0, 2, 1, 0, 0, 0, 0), 0);
    xcfun_delete(fun);
}

#[test]
fn xcfun_eval_vec_writes_all_points() {
    let fun = xcfun_new();
    let name = cstr("slaterx");
    assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
    assert_eq!(xcfun_eval_setup(fun, 2, 1, 0), 0);

    // 4 points × 2 inputs = 8 doubles, density_pitch = 2.
    let density: [f64; 8] = [0.5, 0.5, 0.6, 0.6, 0.7, 0.7, 0.8, 0.8];
    // 4 points × 1 output = 4 doubles, result_pitch = 1.
    let mut result: [f64; 4] = [0.0; 4];
    xcfun_eval_vec(fun, 4, density.as_ptr(), 2, result.as_mut_ptr(), 1);
    for v in result.iter() {
        assert!(v.abs() > 1e-9, "expected non-zero, got {v}");
    }
    xcfun_delete(fun);
}

#[test]
fn xcfun_is_gga_pbex_true() {
    let fun = xcfun_new();
    let name = cstr("pbex");
    assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
    assert!(xcfun_is_gga(fun));
    assert!(!xcfun_is_metagga(fun));
    xcfun_delete(fun);
}

#[test]
fn xcfun_is_metagga_tpssx_true() {
    let fun = xcfun_new();
    let name = cstr("tpssx");
    assert_eq!(xcfun_set(fun, name.as_ptr(), 1.0), 0);
    assert!(xcfun_is_metagga(fun));
    xcfun_delete(fun);
}

// -- abort tests, gated --
#[test]
#[ignore = "calls die_with -> abort; runs via `cargo test -- --ignored`"]
fn xcfun_eval_setup_invalid_vars_aborts() {
    let fun = xcfun_new();
    // 99 is out of range for vars; should die_with.
    let _ = xcfun_eval_setup(fun, 99, 1, 0);
    unreachable!("xcfun_eval_setup with vars=99 must abort");
}
