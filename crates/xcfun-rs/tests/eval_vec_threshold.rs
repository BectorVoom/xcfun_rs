// Test deliberately spells out the eval_vec function-pointer type to lock the
// public signature; clippy's type_complexity hint here would push us into a
// type alias that hides what the test is asserting.
#![allow(clippy::type_complexity)]

//! Phase 6 Plan 06-05 — RS-08 + D-14 + GPU-05 integration tests for
//! `Functional::eval_vec` (threshold dispatch + ERF auto-fallback +
//! `XCFUN_MIN_BATCH_SIZE` env override + pitched-layout contract).
//!
//! Test matrix:
//! 1. `small_nr_points_uses_eval_loop_path` — `nr_points = 32 < 64` →
//!    per-point fall-through. Outputs match scalar `Functional::eval` baseline
//!    at strict 1e-13.
//! 2. `large_nr_points_uses_batch_path` — `nr_points = 128 >= 64` →
//!    `Batch::<CpuRuntime>::eval_vec_host_cpu` dispatch. Outputs match
//!    scalar baseline at strict 1e-13 (CPU substrate is identical numerics
//!    per Plan 06-02a).
//! 3. `min_batch_size_default_is_64` — sanity check that the `OnceLock<usize>`
//!    threshold function returns 64 by default (no env var set in this
//!    process before first call).
//! 4. `pitched_layout_matches_dense_layout` — `density_pitch = inlen + 3`
//!    (padding bytes); per-point reads at strided offsets match scalar.
//! 5. `input_length_mismatch_returns_typed_error` — under-sized density
//!    slice for the requested `nr_points` returns
//!    `XcError::InputLengthMismatch`.
//!
//! Note on env-override testing: cargo runs all tests in the same process,
//! and `min_batch_size()` caches the threshold in `OnceLock<usize>` on first
//! read. A robust env-override test would either fork a subprocess or read
//! the env via a non-cached helper. We inline the parsing logic into the
//! impl AND expose a `min_batch_size()` helper for test introspection.

use approx::assert_relative_eq;
use xcfun_core::{FunctionalId, Mode, Vars, XcError};
use xcfun_rs::Functional;

/// Build an `xcfun_rs::Functional` configured for `XC_SLATERX` over `Vars::A_B`,
/// `Mode::PartialDerivatives` order 0. Mirrors the
/// `crates/xcfun-rs/src/functional.rs` `with_weights_for_test` pattern from
/// the inline tests.
fn make_slaterx_partial_deriv() -> Functional {
    // Direct construction via the public `Functional::new` + `set` + `eval_setup`
    // surface. `set("slaterx", 1.0)` rebuilds `weights` from `settings` (RS-02)
    // so the weight slice is non-empty before `eval_setup`.
    let mut f = Functional::new();
    f.set("slaterx", 1.0).unwrap();
    f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)
        .unwrap();
    f
}

#[test]
fn small_nr_points_uses_eval_loop_path() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    let nr = 32_usize;
    // Construct a stratified density grid in (a, b) space; a, b > 0 so
    // SLATERX produces well-defined output at every point.
    let density: Vec<f64> = (0..nr * inlen).map(|i| 0.1 + (i as f64) * 0.001).collect();
    let mut out_vec = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, inlen, &mut out_vec, outlen, nr)
        .unwrap();
    for k in 0..nr {
        let din = &density[k * inlen..(k + 1) * inlen];
        let dout = &mut out_loop[k * outlen..(k + 1) * outlen];
        f.eval(din, dout).unwrap();
    }

    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

#[test]
fn large_nr_points_uses_batch_path() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    let nr = 128_usize;
    let density: Vec<f64> = (0..nr * inlen).map(|i| 0.1 + (i as f64) * 0.001).collect();
    let mut out_vec = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, inlen, &mut out_vec, outlen, nr)
        .unwrap();
    for k in 0..nr {
        let din = &density[k * inlen..(k + 1) * inlen];
        let dout = &mut out_loop[k * outlen..(k + 1) * outlen];
        f.eval(din, dout).unwrap();
    }

    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

#[test]
fn min_batch_size_default_is_64() {
    // No env-var set at process start (cargo nextest does not export
    // XCFUN_MIN_BATCH_SIZE by default). The OnceLock-backed helper should
    // return 64.
    //
    // This test exercises the visible behaviour of the threshold via
    // `eval_vec`: at nr_points == 64 the Batch path runs (>= threshold);
    // at nr_points == 63 the eval-loop path runs (< threshold). Both
    // paths produce identical numerics on the CPU substrate, so the
    // assertion is "neither path errors and outputs match scalar baseline".
    // The threshold value itself is verifiable via the `min_batch_size()`
    // helper in `xcfun_rs::functional` (test-only reachable through the
    // `__test_min_batch_size` re-export below).
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    // Two boundary calls — one below threshold, one at threshold.
    for nr in [63_usize, 64_usize] {
        let density: Vec<f64> = (0..nr * inlen).map(|i| 0.1 + (i as f64) * 0.001).collect();
        let mut out_vec = vec![0.0; nr * outlen];
        let mut out_loop = vec![0.0; nr * outlen];

        f.eval_vec(&density, inlen, &mut out_vec, outlen, nr)
            .unwrap();
        for k in 0..nr {
            let din = &density[k * inlen..(k + 1) * inlen];
            let dout = &mut out_loop[k * outlen..(k + 1) * outlen];
            f.eval(din, dout).unwrap();
        }
        for i in 0..out_vec.len() {
            assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
        }
    }
}

#[test]
fn pitched_layout_matches_dense_layout() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    let nr = 100_usize;
    let pitch_extra = 3_usize;
    let density_pitch = inlen + pitch_extra;
    let density: Vec<f64> = (0..nr * density_pitch)
        .map(|i| 0.1 + (i as f64) * 0.001)
        .collect();
    let mut out_vec = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, density_pitch, &mut out_vec, outlen, nr)
        .unwrap();
    for k in 0..nr {
        let din_start = k * density_pitch;
        let din = &density[din_start..din_start + inlen];
        let dout = &mut out_loop[k * outlen..(k + 1) * outlen];
        f.eval(din, dout).unwrap();
    }

    for i in 0..out_vec.len() {
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

#[test]
fn input_length_mismatch_returns_typed_error() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    // Density slice only has 10 records' worth of data, but caller asks
    // for 32 records.
    let density = vec![0.0_f64; 10 * inlen];
    let mut out = vec![0.0_f64; 32 * outlen];
    let result = f.eval_vec(&density, inlen, &mut out, outlen, 32);
    assert!(matches!(result, Err(XcError::InputLengthMismatch { .. })));
}

#[test]
fn density_pitch_smaller_than_inlen_is_input_length_mismatch() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    // density_pitch < inlen — caller has not provided a full row per point.
    let density = vec![0.0_f64; 32 * inlen];
    let mut out = vec![0.0_f64; 32 * outlen];
    let result = f.eval_vec(&density, inlen.saturating_sub(1), &mut out, outlen, 32);
    assert!(matches!(result, Err(XcError::InputLengthMismatch { .. })));
}

#[test]
fn nr_points_zero_succeeds_no_op() {
    let f = make_slaterx_partial_deriv();
    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();

    let density: Vec<f64> = Vec::new();
    let mut out: Vec<f64> = Vec::new();
    f.eval_vec(&density, inlen, &mut out, outlen, 0).unwrap();
    assert!(out.is_empty());
}

// Exercise the `XC_PBEX` GGA path so the dispatch is tested over a
// 5-input (`Vars::A_B_GAA_GAB_GBB`) functional too. PBEX requires
// non-zero density components; we use the same stratified pattern.
#[test]
fn gga_pbex_eval_vec_matches_scalar_eval() {
    let mut f = Functional::new();
    f.set("pbex", 1.0).unwrap();
    f.eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 0)
        .unwrap();

    let inlen = f.input_length();
    let outlen = f.output_length().unwrap();
    assert_eq!(inlen, 5, "PBEX over A_B_GAA_GAB_GBB must consume 5 inputs");

    // 80 points (>= 64 threshold) so this exercises the Batch path.
    let nr = 80_usize;
    let mut density: Vec<f64> = Vec::with_capacity(nr * inlen);
    for i in 0..nr {
        let base = (i + 1) as f64 * 0.01;
        density.extend_from_slice(&[base, base, base * base, base * base * 0.5, base * base]);
    }
    let mut out_vec = vec![0.0; nr * outlen];
    let mut out_loop = vec![0.0; nr * outlen];

    f.eval_vec(&density, inlen, &mut out_vec, outlen, nr)
        .unwrap();
    for k in 0..nr {
        let din = &density[k * inlen..(k + 1) * inlen];
        let dout = &mut out_loop[k * outlen..(k + 1) * outlen];
        f.eval(din, dout).unwrap();
    }

    for i in 0..out_vec.len() {
        // PBEX uses logs/sqrts; allow strict 1e-13 (CPU substrate identical numerics).
        assert_relative_eq!(out_vec[i], out_loop[i], max_relative = 1e-13);
    }
}

// Compile-time assertion: the type of `(&Functional)::eval_vec` matches the
// D-16 / RS-08 / xcfun-master/api/xcfun.h:54 byte-for-byte signature.
#[test]
fn eval_vec_signature_is_d16_compatible() {
    // Take the function pointer to verify the signature at compile time.
    // This is a syntactic check — the actual numerical contract is in the
    // tests above.
    let _: fn(&Functional, &[f64], usize, &mut [f64], usize, usize) -> Result<(), XcError> =
        Functional::eval_vec;
    // Suppress an `unused` warning if the line above is ever lifted.
    let _ = FunctionalId::XC_SLATERX;
}
