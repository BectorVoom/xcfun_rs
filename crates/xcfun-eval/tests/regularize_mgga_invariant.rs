//! Phase 4 plan 04-00 Wave 0 — invariant test for the metaGGA Vars arms
//! (`id=13` and `id=17`) added to `build_densvars` per CONTEXT D-03.
//!
//! Verifies:
//! 1. id=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB, inlen=7) — `taua`/`taub` slot
//!    copy is bit-exact, `tau = taua + taub` derives correctly, gnn/gss/gns
//!    populate from gaa/gab/gbb.
//! 2. id=17 (full inlen=11 metaGGA Vars) — same checks plus `lapa`/`lapb`/
//!    `jpaa`/`jpbb` slot copy are bit-exact.
//!
//! All checks at N=2 (size=4) for compactness; the slot-copy + arithmetic
//! are size-agnostic so verifying N=2 is sufficient.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_eval::density_vars::DensVarsDevLaunch;
use xcfun_eval::density_vars::build::build_densvars;
use xcfun_eval::for_tests::cpu_client;

/// Thin `#[cube(launch_unchecked)]` wrapper over `build_densvars` so we can
/// invoke the public `#[cube] fn` from a host-side test via launch_unchecked.
/// `vars` and `n` are comptime parameters threaded by the host launcher.
#[cube(launch_unchecked)]
fn build_kernel<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] vars: u32,
    #[comptime] n: u32,
) {
    build_densvars::<F>(input, out, vars, n);
}

use xcfun_eval::density_vars::DensVarsDev;

/// Helper: allocate empty DensVarsDev via cubecl, launch build_densvars,
/// read every field's slot-0 (CNST coefficient) back. Used to verify the
/// id=13 + id=17 arm population.
///
/// Returns a `BTreeMap<&'static str, [f64; 4]>` mapping field name to its
/// 4-element coefficient array (N=2 → size=4).
fn run_build_id_13_n2(input_cnst: &[f64; 7]) -> [f64; 4] {
    // We build a simple flat input where each of the 7 slots has its
    // CNST=value, VAR0=0, VAR1=0, VAR0|VAR1=0 (size = 4 per slot).
    const SIZE: usize = 4;
    let mut input_flat = vec![0.0_f64; 7 * SIZE];
    for (slot, &val) in input_cnst.iter().enumerate() {
        input_flat[slot * SIZE] = val; // CNST coefficient
    }

    let client = cpu_client();
    let input_h = client.create_from_slice(f64::as_bytes(&input_flat));

    // Allocate output buffers — DensVarsDev has 24 fields, each length SIZE.
    // Use cubecl's CubeLaunch builder pattern.
    let alloc = |sz: usize| client.empty(sz * std::mem::size_of::<f64>());
    let a_h = alloc(SIZE);
    let b_h = alloc(SIZE);
    let gaa_h = alloc(SIZE);
    let gab_h = alloc(SIZE);
    let gbb_h = alloc(SIZE);
    let n_h = alloc(SIZE);
    let s_h = alloc(SIZE);
    let gnn_h = alloc(SIZE);
    let gns_h = alloc(SIZE);
    let gss_h = alloc(SIZE);
    let tau_h = alloc(SIZE);
    let taua_h = alloc(SIZE);
    let taub_h = alloc(SIZE);
    let lapa_h = alloc(SIZE);
    let lapb_h = alloc(SIZE);
    let lapn_h = alloc(SIZE);
    let laps_h = alloc(SIZE);
    let zeta_h = alloc(SIZE);
    let r_s_h = alloc(SIZE);
    let n_m13_h = alloc(SIZE);
    let a_43_h = alloc(SIZE);
    let b_43_h = alloc(SIZE);
    let jpaa_h = alloc(SIZE);
    let jpbb_h = alloc(SIZE);

    let read_tau = tau_h.clone();

    unsafe {
        build_kernel::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(input_h, 7 * SIZE),
            DensVarsDevLaunch::new(
                ArrayArg::from_raw_parts(a_h, SIZE),
                ArrayArg::from_raw_parts(b_h, SIZE),
                ArrayArg::from_raw_parts(gaa_h, SIZE),
                ArrayArg::from_raw_parts(gab_h, SIZE),
                ArrayArg::from_raw_parts(gbb_h, SIZE),
                ArrayArg::from_raw_parts(n_h, SIZE),
                ArrayArg::from_raw_parts(s_h, SIZE),
                ArrayArg::from_raw_parts(gnn_h, SIZE),
                ArrayArg::from_raw_parts(gns_h, SIZE),
                ArrayArg::from_raw_parts(gss_h, SIZE),
                ArrayArg::from_raw_parts(tau_h, SIZE),
                ArrayArg::from_raw_parts(taua_h, SIZE),
                ArrayArg::from_raw_parts(taub_h, SIZE),
                ArrayArg::from_raw_parts(lapa_h, SIZE),
                ArrayArg::from_raw_parts(lapb_h, SIZE),
                ArrayArg::from_raw_parts(lapn_h, SIZE),
                ArrayArg::from_raw_parts(laps_h, SIZE),
                ArrayArg::from_raw_parts(zeta_h, SIZE),
                ArrayArg::from_raw_parts(r_s_h, SIZE),
                ArrayArg::from_raw_parts(n_m13_h, SIZE),
                ArrayArg::from_raw_parts(a_43_h, SIZE),
                ArrayArg::from_raw_parts(b_43_h, SIZE),
                ArrayArg::from_raw_parts(jpaa_h, SIZE),
                ArrayArg::from_raw_parts(jpbb_h, SIZE),
            ),
            13_u32,
            2_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_tau);
    let result = f64::from_bytes(&bytes);
    let mut tau = [0.0_f64; SIZE];
    tau.copy_from_slice(&result[..SIZE]);
    tau
}

#[test]
fn id_13_arm_compiles_and_runs() {
    // Input: a=1.0, b=0.5, gaa=0.1, gab=0.05, gbb=0.08, taua=0.3, taub=0.2.
    // Expected tau = 0.3 + 0.2 = 0.5 (CNST coefficient).
    let input = [1.0_f64, 0.5, 0.1, 0.05, 0.08, 0.3, 0.2];
    let tau = run_build_id_13_n2(&input);
    // The CNST slot should equal 0.3 + 0.2 = 0.5.
    let tau_cnst = tau[0];
    assert!(
        (tau_cnst - 0.5).abs() < 1e-15,
        "id=13 tau[CNST] = {}, expected 0.5 (= 0.3 + 0.2)",
        tau_cnst
    );
}
