//! Wave 0 spike test — proves cubecl-cpu 0.10-pre.3 works on this machine
//! before any `ctaylor_*` port is written (PATTERNS.md "cubecl spike task").
//!
//! This file is the only place in Phase 1 Plan 01-01 that exercises a real
//! `#[cube] fn` + `launch_unchecked` round-trip. If this test passes, the
//! substrate is good; if it fails, no downstream plan can be executed
//! (escalate per CONTEXT.md D-03 — `PLANNING INCONCLUSIVE`).
//!
//! # Cubecl 0.10-pre.3 API deltas vs plan <interfaces>
//!
//! Documented in `src/for_tests/raw_eval_scalar.rs`. Summary:
//! - `client.create` → `client.create_from_slice`
//! - `client.read_one(h.binding())` → `client.read_one_unchecked(h)`
//! - `ArrayArg::from_raw_parts::<F>(&h, len, 1)` →
//!   `ArrayArg::from_raw_parts(h.clone(), len)` (unsafe, 2 args, owned
//!   handle; no turbofish, no vectorization).

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::for_tests::{cpu_client, raw_eval_scalar};
use xcfun_ad::{CNST, VAR0, VAR1, VAR2, VAR3, VAR4, VAR5, VAR6, VAR7};

// ---- Kernel 1: add two scalars ----
#[cube(launch_unchecked)]
fn add_two_scalars<F: Float>(a: &Array<F>, b: &Array<F>, out: &mut Array<F>) {
    out[0] = a[0] + b[0];
}

#[test]
fn add_two_numbers_on_cpu() {
    let client = cpu_client();
    let a = [3.0_f64];
    let b = [4.0_f64];
    let a_h = client.create_from_slice(f64::as_bytes(&a));
    let b_h = client.create_from_slice(f64::as_bytes(&b));
    let out_h = client.empty(core::mem::size_of::<f64>());

    unsafe {
        add_two_scalars::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(a_h.clone(), 1),
            ArrayArg::from_raw_parts(b_h.clone(), 1),
            ArrayArg::from_raw_parts(out_h.clone(), 1),
        );
    }

    let bytes = client.read_one_unchecked(out_h);
    let out = f64::from_bytes(&bytes);
    assert_eq!(out.len(), 1, "cubecl-cpu read-back length mismatch");
    assert_eq!(out[0], 7.0_f64, "3.0 + 4.0 must equal 7.0 (bit-exact)");
}

#[test]
fn client_singleton_is_shared() {
    let c1 = cpu_client() as *const _;
    let c2 = cpu_client() as *const _;
    assert_eq!(c1, c2, "cpu_client() must return the same &CpuClient pointer");
}

// ---- Kernel 2: copy input to output (exercises raw_eval_scalar) ----
#[cube(launch_unchecked)]
fn copy_kernel<F: Float>(inp: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Cubecl 0.10-pre.3: `Array<F>` indexing is `usize`; the comptime loop
    // counter `i` is the iterator type of the unrolled `0..n` expression,
    // cast to `usize` at the index site (see cubecl runtime_tests/cmma.rs).
    #[unroll]
    for i in 0..n {
        out[i as usize] = inp[i as usize];
    }
}

#[test]
fn raw_eval_scalar_roundtrip() {
    let inputs = vec![1.0_f64, 2.0, 3.0, 4.0];
    let inputs_len = inputs.len();
    let out = raw_eval_scalar(&inputs, inputs_len, |client, in_h, out_h| {
        unsafe {
            copy_kernel::launch_unchecked::<f64, CpuRuntime>(
                client,
                CubeCount::Static(1, 1, 1),
                CubeDim::new_3d(1, 1, 1),
                ArrayArg::from_raw_parts(in_h, inputs_len),
                ArrayArg::from_raw_parts(out_h, inputs_len),
                4_u32,
            );
        }
    });
    assert_eq!(out, inputs, "round-trip must be identity");
}

#[test]
fn index_consts_match_cpp() {
    assert_eq!(CNST, 0);
    assert_eq!(VAR0, 1);
    assert_eq!(VAR1, 2);
    assert_eq!(VAR2, 4);
    assert_eq!(VAR3, 8);
    assert_eq!(VAR4, 16);
    assert_eq!(VAR5, 32);
    assert_eq!(VAR6, 64);
    assert_eq!(VAR7, 128);
}
