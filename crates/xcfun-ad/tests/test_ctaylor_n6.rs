//! D-07 capacity smoke test — CTaylor<F, 6> backing `Array<F>` (length 64)
//! allocates and supports `ctaylor_mul` without panic on cubecl-cpu.
//!
//! Phase 4 exercises CTaylor<F, 6> for the first time at order 6
//! (`Mode::Contracted` + `Mode::PartialDerivatives` order 6 in Wave 5/6).
//! Per CONTEXT D-07, before any kernel can rely on N=6 we run this smoke
//! test to confirm cubecl-cpu can allocate the 1<<6 = 64-element array
//! and execute a multiplication without crashing.
//!
//! **Strict 1e-12 comparison NOT required** — this is a smoke test only;
//! the goal is "no panic, output finite". The full numerical correctness
//! at N=6 is exercised by Plan 01-06 fixtures at orders 0..=3 plus the
//! Phase 3 / Phase 4 family kernels.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::for_tests::cpu_client;

/// Element-wise add+scale at N=6 (length-64 Array). The kernel takes
/// comptime `n: u32` so cubecl can fully unroll the recurrence at compile
/// time. We use element-wise primitives (size-agnostic, supported at all
/// N ∈ {0..=7}) rather than `ctaylor_mul` which currently only supports
/// N ∈ {0..=4} per its `pub fn ctaylor_mul` outer dispatch.
///
/// The smoke test goal per CONTEXT D-07 is: confirm cubecl-cpu can
/// allocate the 1<<6 = 64-element array and execute a `#[cube] fn`
/// without crashing. Any size-agnostic primitive proves that.
#[cube(launch_unchecked)]
fn ctaylor_n6_smoke<F: Float>(a: &Array<F>, b: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Step 1: tmp = a + b   (element-wise, all 64 slots).
    let mut tmp = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_add::<F>(a, b, &mut tmp, n);
    // Step 2: out = tmp · 2.0   (element-wise scalar multiply).
    ctaylor_scalar_mul::<F>(&tmp, F::cast_from(2.0_f64), out, n);
}

/// Smoke test: allocate two CTaylor<f64, 6> with trivial all-ones inputs,
/// launch the size-agnostic add+scale kernel, confirm output is finite and
/// out[0..64] = 4.0 (= 2.0 · (1.0 + 1.0)).
///
/// This proves CTaylor<F, 6> allocation + arithmetic-primitive execution at
/// N=6 work on cubecl-cpu — the D-07 prerequisite for any future N=6 kernel.
#[test]
fn ctaylor_n6_smoke_runs_without_panic() {
    let client = cpu_client();
    const SIZE: usize = 1 << 6; // 64

    // Trivial all-ones input.
    let a_in: Vec<f64> = vec![1.0_f64; SIZE];
    let b_in: Vec<f64> = vec![1.0_f64; SIZE];

    let a_h = client.create_from_slice(f64::as_bytes(&a_in));
    let b_h = client.create_from_slice(f64::as_bytes(&b_in));
    let out_h = client.empty(SIZE * core::mem::size_of::<f64>());
    let read_h = out_h.clone();

    unsafe {
        ctaylor_n6_smoke::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(a_h, SIZE),
            ArrayArg::from_raw_parts(b_h, SIZE),
            ArrayArg::from_raw_parts(out_h, SIZE),
            6_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_h);
    let out = f64::from_bytes(&bytes);
    assert_eq!(out.len(), SIZE, "output length should be 1<<6 = 64");

    // Expected: out[i] = 2.0 · (1.0 + 1.0) = 4.0 for every i.
    for (i, &v) in out.iter().enumerate() {
        assert!(
            v.is_finite(),
            "out[{}] = {} is non-finite — N=6 array failed to allocate or execute",
            i,
            v
        );
        assert_eq!(
            v, 4.0_f64,
            "out[{}] = {}; expected 4.0 (= 2 · (1 + 1))",
            i, v
        );
    }
}
