//! Phase 6 Plan 06-N1 (B-6 revision-1) — D-19 regression detector for BECKESRX.
//!
//! Loads `validation/fixtures/d19_n1/beckesrx_baseline.jsonl` and asserts the
//! current Rust `beckesrx_kernel` output (energy / CNST coefficient at N=0)
//! matches the captured baseline within `common::REL_TOL`.
//!
//! See `tests/common/mod.rs` for the substrate-gap caveat (xcfun-master
//! missing in this worktree → fixtures captured from the kernel itself, not
//! from the C++ reference). This is a regression detector NOT a parity gate;
//! it becomes a parity gate after fixture re-baseline against C++ truth.

#![cfg(feature = "testing")]

mod common;
use common::*;

use approx::assert_relative_eq;
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::for_tests::cpu_client;
use xcfun_kernels::density_vars::DensVarsDev;
use xcfun_kernels::density_vars::DensVarsDevLaunch;
use xcfun_kernels::density_vars::build::build_densvars;
use xcfun_kernels::functionals::gga::becke::beckesrx::beckesrx_kernel;

#[cube(launch_unchecked)]
fn adapter<F: Float>(
    input: &Array<F>,
    d: &mut DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] vars: u32,
    #[comptime] n: u32,
) {
    build_densvars::<F>(input, d, vars, n);
    beckesrx_kernel::<F>(d, out, n);
}

fn run_energy(input: &[f64; 5]) -> f64 {
    let client = cpu_client();
    let in_h = client.create_from_slice(f64::as_bytes(input));
    let arr_cnt = 1_usize; // N=0 → 2^0 = 1 coefficient
    let array_len = arr_cnt * std::mem::size_of::<f64>();
    let mk = || client.empty(array_len);
    let h: [_; 24] = [
        mk(), mk(), mk(), mk(), mk(), mk(), mk(), mk(),
        mk(), mk(), mk(), mk(), mk(), mk(), mk(), mk(),
        mk(), mk(), mk(), mk(), mk(), mk(), mk(), mk(),
    ];
    let out_len = 1_usize;
    let out_h = client.empty(out_len * std::mem::size_of::<f64>());
    let read_h = out_h.clone();

    #[allow(unsafe_code)]
    unsafe {
        adapter::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(in_h, input.len()),
            DensVarsDevLaunch::new(
                ArrayArg::from_raw_parts(h[0].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[1].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[2].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[3].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[4].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[5].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[6].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[7].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[8].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[9].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[10].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[11].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[12].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[13].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[14].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[15].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[16].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[17].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[18].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[19].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[20].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[21].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[22].clone(), arr_cnt),
                ArrayArg::from_raw_parts(h[23].clone(), arr_cnt),
            ),
            ArrayArg::from_raw_parts(out_h, out_len),
            VARS_A_B_GAA_GAB_GBB,
            0_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_h);
    let result = f64::from_bytes(&bytes);
    result[0]
}

#[test]
fn d19_beckesrx_baseline_regression() {
    let path = fixture_path("beckesrx");
    let records = load_fixture(&path);
    assert!(
        records.len() >= 5 && records.len() <= 10,
        "fixture {} should hold 5-10 records, has {}",
        path,
        records.len()
    );
    for (i, rec) in records.iter().enumerate() {
        let actual = run_energy(&rec.input);
        assert_relative_eq!(
            actual,
            rec.expected_energy,
            max_relative = REL_TOL
        );
        eprintln!(
            "rec[{}]: input={:?} actual={:.17e} expected={:.17e}",
            i, rec.input, actual, rec.expected_energy
        );
    }
}
