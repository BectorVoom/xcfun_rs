//! D-02 verification spike — confirm `#[derive(CubeType, CubeLaunch)]` works
//! for nested struct types on cubecl-cpu 0.10-pre.3.
//!
//! If this test fails to compile or run, the planner MUST escalate
//! `PLANNING INCONCLUSIVE` per CONTEXT D-02 (fallback: monolithic Array<F>
//! of length 22 * (1<<N) with comptime offset helpers).
//!
//! # Cubecl 0.10-pre.3 API notes (from xcfun-ad/tests/cubecl_spike.rs)
//!
//! - `ArrayArg::from_raw_parts(handle.clone(), len)` — 2 args, no turbofish.
//! - `client.create_from_slice(bytes)` / `client.read_one_unchecked(handle)`.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_eval::for_tests::cpu_client;

#[derive(CubeType, CubeLaunch)]
struct Trio<F: Float> {
    a: Array<F>,
    b: Array<F>,
    c: Array<F>,
}

#[cube(launch_unchecked)]
fn fill_trio<F: Float>(out: &mut Trio<F>) {
    out.a[0] = F::new(1.0);
    out.b[0] = F::new(2.0);
    out.c[0] = out.a[0] + out.b[0];
}

#[test]
fn trio_struct_compiles_and_runs() {
    let client = cpu_client();
    let a_h = client.empty(core::mem::size_of::<f64>());
    let b_h = client.empty(core::mem::size_of::<f64>());
    let c_h = client.empty(core::mem::size_of::<f64>());

    unsafe {
        fill_trio::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            TrioLaunch::new(
                ArrayArg::from_raw_parts(a_h.clone(), 1),
                ArrayArg::from_raw_parts(b_h.clone(), 1),
                ArrayArg::from_raw_parts(c_h.clone(), 1),
            ),
        );
    }

    let bytes = client.read_one_unchecked(c_h);
    let result = f64::from_bytes(&bytes);
    assert_eq!(
        result[0], 3.0,
        "Trio<F> with #[derive(CubeType, CubeLaunch)] should support kernel-arg + field access"
    );
}
