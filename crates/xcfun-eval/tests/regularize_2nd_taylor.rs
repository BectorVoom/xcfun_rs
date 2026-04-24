//! Pitfall G8 + Wave-0 gap 5 verification — `regularize` clamps ONLY slot 0
//! (CNST) even when higher-order slots carry 2nd-Taylor-seeded derivative
//! coefficients. The 2ND_TAYLOR Vars arms (27..30) depend on this invariant:
//! their per-point launches pack α's (or n's) 10 spatial-Taylor coefficients
//! into slots 0..9 of a CTaylor<F, 3> (size=8) or CTaylor<F, 4> (size=16)
//! block, and the CNST-only clamp MUST NOT zero the higher-order partials.
//!
//! See `.planning/phases/03-gga-tier-mode-potential/03-RESEARCH.md` Pitfall G8
//! and `.planning/phases/03-gga-tier-mode-potential/03-VALIDATION.md` gap 5.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_eval::density_vars::regularize::{TINY_DENSITY_F64, regularize};
use xcfun_eval::for_tests::cpu_client;

/// Thin `#[cube(launch_unchecked)]` wrapper over `regularize` so we can invoke
/// the public `#[cube] fn` from a host-side test. N=3 (size=8) exercises the
/// same coefficient layout as the `_2ND_TAYLOR` Vars-arm bulk under
/// `Mode::Potential`.
#[cube(launch_unchecked)]
fn regularize_kernel<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    regularize::<F>(x, n);
}

fn run_regularize(input: &[f64; 8]) -> [f64; 8] {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(input));
    let read_h = x_h.clone();

    unsafe {
        regularize_kernel::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(x_h, 8),
            3_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_h);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; 8];
    out.copy_from_slice(&result[..8]);
    out
}

/// Core Pitfall G8 assertion: CNST slot (0) is clamped to `TINY_DENSITY_F64`
/// when below the floor; slots 1..7 are preserved bit-exactly. This is the
/// invariant that lets `_2ND_TAYLOR` Vars arms safely seed 2nd-order spatial
/// derivatives into the Taylor-coefficient array without `regularize` wiping
/// them on density-clamp iterations.
#[test]
fn regularize_preserves_2nd_taylor_coefficients() {
    // Slot 0 below the clamp floor → should be clamped.
    // Slots 1..7 carry nonzero 2nd-Taylor-seeded coefficients that MUST survive.
    let input: [f64; 8] = [1e-15, 0.3, 0.5, 1.1, 0.7, 0.9, -0.4, 2.25];
    let output = run_regularize(&input);

    // Slot 0: clamped to TINY_DENSITY_F64. The runtime F::cast_from(1e-14_f64)
    // under F=f64 is bit-exact, so equality holds.
    assert!(
        output[0] >= TINY_DENSITY_F64 && output[0] <= TINY_DENSITY_F64 * 1.000_001,
        "slot 0 (CNST) should be clamped to TINY_DENSITY_F64; got {}",
        output[0]
    );
    // Slots 1..7: byte-identical to input.
    for i in 1..8 {
        assert_eq!(
            output[i], input[i],
            "slot {i} must be preserved (input {} → output {})",
            input[i], output[i]
        );
    }
}

/// Cross-check: when slot 0 is above the floor, `regularize` is a no-op on
/// every slot. This confirms the branch doesn't accidentally write to
/// higher-order slots in either direction.
#[test]
fn regularize_leaves_above_clamp_unchanged() {
    let input: [f64; 8] = [1.0, 0.3, 0.5, 1.1, 0.7, 0.9, -0.4, 2.25];
    let output = run_regularize(&input);
    for i in 0..8 {
        assert_eq!(output[i], input[i], "slot {i} must be unchanged above floor");
    }
}
