//! End-to-end FFI smoke test — verifies the `unsafe extern "C"` declarations
//! in `validation/src/ffi.rs` match xcfun-master's C ABI and that the
//! CppXcfun RAII lifecycle (new → set → eval_setup → input_length →
//! output_length → eval → drop) works for a known functional.
//!
//! Test case: SLATERX at (ρ_α, ρ_β) = (39, 38) order 0. Energy ≈ -241.948
//! (from `xcfun-master/src/functionals/slaterx.cpp:32`, which is the
//! upstream test_out[0] value).

use validation::ffi::CppXcfun;

#[test]
fn ffi_smoke_slaterx_xcfun_lifecycle() {
    let mut cpp = CppXcfun::new();

    // Activate slaterx with weight 1.0. C side is case-insensitive (strcasecmp).
    let status_set = cpp.set("slaterx", 1.0);
    assert_eq!(
        status_set, 0,
        "xcfun_set(slaterx, 1.0) failed: status={}",
        status_set
    );

    // vars = XC_A_B (2); mode = XC_PARTIAL_DERIVATIVES (1); order = 0.
    let status_setup = cpp.eval_setup(2, 1, 0);
    assert_eq!(
        status_setup, 0,
        "xcfun_eval_setup(XC_A_B, XC_PARTIAL_DERIVATIVES, 0) failed: status={}",
        status_setup
    );

    assert_eq!(cpp.input_length(), 2, "SLATERX input_length must be 2");
    assert_eq!(cpp.output_length(), 1, "order=0 output_length must be 1");

    let input = [39.0_f64, 38.0];
    let mut output = [0.0_f64; 1];
    cpp.eval(&input, &mut output);

    // Expected energy from xcfun-master/src/functionals/slaterx.cpp:32: -241.948147838.
    let expected = -241.948_147_838_f64;
    assert!(
        (output[0] - expected).abs() < 1e-7,
        "SLATERX energy at (39, 38): got {}, expected {} (upstream test_out[0])",
        output[0],
        expected
    );
}
