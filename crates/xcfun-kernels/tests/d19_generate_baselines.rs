//! Phase 6 Plan 06-N1 (B-6 revision-1) — fixture baseline generator.
//!
//! Test runner for capturing the 11 per-functional D-19 baseline jsonl
//! fixtures. The captured `expected_energy` value is the **current Rust
//! kernel's output** at commit time (a self-consistency regression
//! detector — see `tests/common/mod.rs` for the substrate-gap caveat).
//!
//! # Usage
//!
//! Captures all 11 fixtures in one launch:
//!
//! ```bash
//! D19_REGEN=1 cargo test -p xcfun-kernels --features testing \
//!     --test d19_generate_baselines -- --include-ignored
//! ```
//!
//! The test is `#[ignore]`'d by default to keep it out of normal runs.
//! Without `D19_REGEN=1` the test exits as a no-op.
//!
//! When `xcfun-master/` is restored and a C++ baseline becomes available,
//! this generator should be retired in favor of the standard
//! `cargo run -p validation -- --backend cpu --order 3 --filter ...`
//! → fixture extraction pipeline.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use std::fs::File;
use std::io::{BufWriter, Write};
use xcfun_ad::for_tests::cpu_client;
use xcfun_kernels::density_vars::DensVarsDev;
use xcfun_kernels::density_vars::DensVarsDevLaunch;
use xcfun_kernels::density_vars::build::build_densvars;

// 11 inherited-Phase-3 D-19 forwards' kernels.
use xcfun_kernels::functionals::gga::apbe::apbec::apbec_kernel;
use xcfun_kernels::functionals::gga::b97::b97_1c::b97_1c_kernel;
use xcfun_kernels::functionals::gga::b97::b97_2c::b97_2c_kernel;
use xcfun_kernels::functionals::gga::b97::b97c::b97c_kernel;
use xcfun_kernels::functionals::gga::becke::beckesrx::beckesrx_kernel;
use xcfun_kernels::functionals::gga::p86::p86c::p86c_kernel;
use xcfun_kernels::functionals::gga::p86::p86corrc::p86corrc_kernel;
use xcfun_kernels::functionals::gga::pbe::pbeintc::pbeintc_kernel;
use xcfun_kernels::functionals::gga::pbe::spbec::spbec_kernel;
use xcfun_kernels::functionals::gga::pw91::pw91c::pw91c_kernel;
use xcfun_kernels::functionals::gga::pw91::pw91k::pw91k_kernel;

/// Vars=A_B_GAA_GAB_GBB.
const VARS: u32 = 6;

/// Curated 6-record density grid covering the strata where Phase-4 reported
/// the largest order-3 max_rel_err (per `04-VERIFICATION.md` distribution):
///   - upstream P86C / PW91C test_in records (physical regime, low magnitude)
///   - mid-density polarised regime
///   - high-density polarised regime
///   - low-density gradient-stress regime
///   - very-low density (regularize-clamp boundary, but ABOVE TINY_DENSITY)
///   - asymmetric-spin gradient regime
///
/// Each record is `[a, b, gaa, gab, gbb]` with all 5 components > 0.
const GRID: [[f64; 5]; 6] = [
    // Upstream P86C test_in record (low-density physical regime).
    [4.8e-2, 2.5e-2, 4.6e-3, 4.4e-3, 4.1e-3],
    // Upstream PW91C test_in record (mid-density physical regime).
    [1.3e-1, 9.5e-2, 1.5e-1, 1.8e-1, 2.2e-1],
    // Mid-density polarised regime.
    [0.5, 0.4, 0.1, 0.05, 0.08],
    // High-density polarised regime (gradient-stress).
    [0.2, 0.18, 0.4, 0.3, 0.45],
    // Asymmetric-spin gradient regime.
    [0.15, 0.05, 0.08, 0.04, 0.12],
    // Low-density above regularize-clamp boundary.
    [1.0e-6, 8.0e-7, 5.0e-7, 4.0e-7, 6.0e-7],
];

// ---------------------------------------------------------------------------
//  One `#[cube(launch_unchecked)]` adapter per functional. Each runs
//  `build_densvars` + `<NAME>_kernel` at comptime VARS=6, N=0 → out_len=1.
// ---------------------------------------------------------------------------

macro_rules! adapter {
    ($adapter_name:ident, $kernel_ident:ident) => {
        #[cube(launch_unchecked)]
        fn $adapter_name<F: Float>(
            input: &Array<F>,
            d: &mut DensVarsDev<F>,
            out: &mut Array<F>,
            #[comptime] vars: u32,
            #[comptime] n: u32,
        ) {
            build_densvars::<F>(input, d, vars, n);
            $kernel_ident::<F>(d, out, n);
        }
    };
}

adapter!(adapter_pbeintc, pbeintc_kernel);
adapter!(adapter_beckesrx, beckesrx_kernel);
adapter!(adapter_p86c, p86c_kernel);
adapter!(adapter_p86corrc, p86corrc_kernel);
adapter!(adapter_pw91c, pw91c_kernel);
adapter!(adapter_spbec, spbec_kernel);
adapter!(adapter_apbec, apbec_kernel);
adapter!(adapter_b97c, b97c_kernel);
adapter!(adapter_b97_1c, b97_1c_kernel);
adapter!(adapter_b97_2c, b97_2c_kernel);
adapter!(adapter_pw91k, pw91k_kernel);

// One Rust-side launch helper per functional. The helper takes the cubecl
// adapter's `launch_unchecked` fn as input via type-erased closure (the
// `launch_unchecked` symbol resolves through the `use` import) — but cubecl's
// generated launchers are private modules with non-Copy fn-types, so we
// instead replicate the 50 lines of launch boilerplate inside each helper
// via macro. This keeps the same launch_unchecked signature visible at
// each call site for lint clarity.
macro_rules! launch_helper {
    ($helper_name:ident, $adapter_ident:ident) => {
        fn $helper_name(input: &[f64; 5]) -> f64 {
            let client = cpu_client();
            let in_h = client.create_from_slice(f64::as_bytes(input));
            let arr_cnt = 1_usize; // N=0 → 2^0 = 1 coefficient
            let array_len = arr_cnt * std::mem::size_of::<f64>();
            let mk = || client.empty(array_len);
            let h: [_; 24] = [
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
                mk(),
            ];
            let out_h = client.empty(std::mem::size_of::<f64>());
            let read_h = out_h.clone();

            #[allow(unsafe_code)]
            unsafe {
                $adapter_ident::launch_unchecked::<f64, CpuRuntime>(
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
                    ArrayArg::from_raw_parts(out_h, 1_usize),
                    VARS,
                    0_u32,
                );
            }

            let bytes = client.read_one_unchecked(read_h);
            let result = f64::from_bytes(&bytes);
            result[0]
        }
    };
}

launch_helper!(run_pbeintc, adapter_pbeintc);
launch_helper!(run_beckesrx, adapter_beckesrx);
launch_helper!(run_p86c, adapter_p86c);
launch_helper!(run_p86corrc, adapter_p86corrc);
launch_helper!(run_pw91c, adapter_pw91c);
launch_helper!(run_spbec, adapter_spbec);
launch_helper!(run_apbec, adapter_apbec);
launch_helper!(run_b97c, adapter_b97c);
launch_helper!(run_b97_1c, adapter_b97_1c);
launch_helper!(run_b97_2c, adapter_b97_2c);
launch_helper!(run_pw91k, adapter_pw91k);

fn write_fixture(name: &str, run: impl Fn(&[f64; 5]) -> f64) {
    // Cargo sets cwd to the package root; the workspace root is two `..` up
    // (workspace/xcfun-kernels → workspace/crates/xcfun-kernels).
    let path = format!("../../validation/fixtures/d19_n1/{}_baseline.jsonl", name);
    let f = File::create(&path).unwrap_or_else(|e| panic!("create {}: {}", path, e));
    let mut w = BufWriter::new(f);
    for input in GRID.iter() {
        let energy = run(input);
        writeln!(
            w,
            r#"{{"input":[{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}],"expected_energy":{:.17e}}}"#,
            input[0], input[1], input[2], input[3], input[4], energy
        )
        .unwrap();
    }
    w.flush().unwrap();
    eprintln!("wrote fixture: {}", path);
}

#[test]
#[ignore = "regen-only — run with D19_REGEN=1 to capture baselines"]
fn d19_regenerate_all_baselines() {
    if std::env::var("D19_REGEN").ok().as_deref() != Some("1") {
        eprintln!("D19_REGEN!=1; skipping (use -- --include-ignored + D19_REGEN=1)");
        return;
    }

    write_fixture("pbeintc", run_pbeintc);
    write_fixture("beckesrx", run_beckesrx);
    write_fixture("p86c", run_p86c);
    write_fixture("p86corrc", run_p86corrc);
    write_fixture("pw91c", run_pw91c);
    write_fixture("spbec", run_spbec);
    write_fixture("apbec", run_apbec);
    write_fixture("b97c", run_b97c);
    write_fixture("b97_1c", run_b97_1c);
    write_fixture("b97_2c", run_b97_2c);
    write_fixture("pw91k", run_pw91k);
}
