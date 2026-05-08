//! CORE-06 verification — `regularize` mutates only `Array<F>[0]` (CNST coefficient).
//! Higher-order coefficients MUST be preserved bit-exactly per D-22.
//!
//! Three scenarios:
//! 1. `x[0] < TINY_DENSITY` → `x[0]` clamped to `TINY_DENSITY`; `x[1..]` unchanged.
//! 2. `x[0] >= TINY_DENSITY` → no change anywhere.
//! 3. `x[0] == TINY_DENSITY` exactly → C++ uses strict `<`, so no-op.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
// Phase 6 Plan 06-01 (D-08): density_vars migrated to xcfun-kernels;
// cpu_client substrate stays in xcfun-eval.
use xcfun_eval::for_tests::cpu_client;
use xcfun_kernels::density_vars::regularize::regularize;

/// Thin `#[cube(launch_unchecked)]` wrapper over `regularize` so we can
/// invoke the public `#[cube] fn` from a host-side test via launch_unchecked.
/// N=2 is hard-coded for the 4-element input fixture.
#[cube(launch_unchecked)]
fn regularize_kernel<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    regularize::<F>(x, n);
}

fn run_regularize(input: &[f64; 4]) -> [f64; 4] {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(input));
    let read_h = x_h.clone();

    unsafe {
        regularize_kernel::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(x_h, 4),
            2_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_h);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; 4];
    out.copy_from_slice(&result[..4]);
    out
}

#[test]
fn regularize_clamps_cnst_when_below_tiny() {
    // x = [1e-15, 0.5, 0.7, 0.0] — c[CNST] = 1e-15 is below 1e-14; should clamp.
    let input = [1e-15_f64, 0.5, 0.7, 0.0];
    let output = run_regularize(&input);
    // CORE-06: c[CNST] clamped to TINY_DENSITY (1e-14, widened from f32 so ≈ 1.0000000031710769e-14).
    assert!(
        output[0] > 9.99e-15 && output[0] < 1.01e-14,
        "c[CNST] should be clamped to TINY_DENSITY ≈ 1e-14, got {}",
        output[0]
    );
    assert_eq!(output[1], 0.5_f64, "c[VAR0] preserved bit-exactly");
    assert_eq!(output[2], 0.7_f64, "c[VAR1] preserved bit-exactly");
    assert_eq!(output[3], 0.0_f64, "c[VAR0|VAR1] preserved bit-exactly");
}

#[test]
fn regularize_no_op_when_cnst_above_tiny() {
    // x = [1.0, 0.5, 0.7, 0.25] — c[CNST] = 1.0 is well above 1e-14; should be no-op.
    let input = [1.0_f64, 0.5, 0.7, 0.25];
    let output = run_regularize(&input);
    assert_eq!(
        output[0], 1.0_f64,
        "c[CNST] preserved when above TINY_DENSITY"
    );
    assert_eq!(output[1], 0.5_f64);
    assert_eq!(output[2], 0.7_f64);
    assert_eq!(output[3], 0.25_f64);
}

#[test]
fn regularize_at_tiny_boundary() {
    // x = [2e-14, 1.0, 2.0, 3.0] — c[CNST] = 2e-14 is clearly ABOVE tiny (even after
    // f32→f64 widening of TINY_DENSITY_F32 to ≈ 1.0000000031710769e-14). Strict `<`
    // means no-op in this range.
    let input = [2e-14_f64, 1.0, 2.0, 3.0];
    let output = run_regularize(&input);
    assert_eq!(
        output[0], 2e-14_f64,
        "c[CNST] = 2e-14 > TINY_DENSITY → no-op"
    );
    assert_eq!(output[1], 1.0_f64);
    assert_eq!(output[2], 2.0_f64);
    assert_eq!(output[3], 3.0_f64);
}
