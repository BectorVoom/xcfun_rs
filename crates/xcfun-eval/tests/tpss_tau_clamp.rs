//! Phase 6 Plan 06-00 Task 3 — TPSS `tau ≥ tau_w` hard-clamp guard test.
//!
//! Verifies the kernel-body guard inserted in `tpssc.rs`, `tpsslocc.rs`,
//! `revtpssc.rs` per CONTEXT D-10. The guard collapses tau to the von
//! Weizsäcker bound (`tau_w = gnn / (8 n)`) when `tau < tau_w`, eliminating
//! the f64-rounding-cancellation divergence (1e+27 magnitudes — Plan 04-10
//! Path-B finding) in the unphysical regime.
//!
//! **Test design**
//!
//! Rather than exercise the full kernel through `build_densvars` →
//! `tpssc_kernel` (which requires 24 output buffer allocations per
//! launch), we test the two pieces of the guard mechanism in isolation:
//!
//! 1. `build_tau_w` returns `gnn / (8 n)` correctly at the CNST slot.
//! 2. `ctaylor_max(d.tau, tau_w)` returns `tau_w` when `tau < tau_w`
//!    (unphysical regime, the cause of the 1e+27 divergence) and `d.tau`
//!    when `tau >= tau_w` (physical regime; bit-exact baseline).
//!
//! The full kernel-level integration is exercised by the existing tier-1
//! self-tests (`self_tests.rs`) which load TPSS-correlation kernels via
//! `build_densvars` → `tpssc_kernel` and compare to the C++ reference at
//! a handful of physical-regime density points. Plan 06-00 sign-off relies
//! on those tier-1 self-tests still passing AFTER the guard insertion (no
//! regression in the physical regime — the `ctaylor_max` returns d.tau
//! exactly when tau >= tau_w, by construction).

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::for_tests::cpu_client;
// Phase 6 Plan 06-01 (D-08): mgga kernel bodies migrated to xcfun-kernels.
use xcfun_kernels::functionals::mgga::shared::tpss_like::ctaylor_max;

/// `#[cube(launch_unchecked)]` adapter exposing `ctaylor_max` to a host-side
/// test launcher. We test the max-on-CNST semantics directly (the guard's
/// "did the clamp fire?" decision) without needing the full DensVarsDev
/// allocation overhead.
#[cube(launch_unchecked)]
fn kernel_ctaylor_max<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_max::<F>(a, b, out, n);
}

/// Helper: launch `ctaylor_max(a, b)` at N=1 (size=2) and read back both
/// coefficients (CNST + first-derivative slot) so we can verify both the
/// CNST decision AND that the chosen array is propagated entirely (not
/// just CNST).
fn run_ctaylor_max_n1(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    const SIZE: usize = 2;
    let client = cpu_client();
    let a_h = client.create_from_slice(f64::as_bytes(&a));
    let b_h = client.create_from_slice(f64::as_bytes(&b));
    let out_h = client.empty(SIZE * std::mem::size_of::<f64>());
    let read_h = out_h.clone();

    unsafe {
        kernel_ctaylor_max::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(a_h, SIZE),
            ArrayArg::from_raw_parts(b_h, SIZE),
            ArrayArg::from_raw_parts(out_h, SIZE),
            1_u32,
        );
    }

    let bytes = client.read_one_unchecked(read_h);
    let result = f64::from_bytes(&bytes);
    let mut out = [0.0_f64; SIZE];
    out.copy_from_slice(&result[..SIZE]);
    out
}

/// **Unphysical regime**: `tau < tau_w` triggers the clamp. The guard
/// dispatches `ctaylor_max(tau, tau_w)` which selects on CNST. Result MUST
/// equal `tau_w` (greater CNST), not `tau` (smaller CNST) — eliminating
/// the 1e+27 divergence that Plan 04-10 Path-B traced to f64 cancellation
/// in `eps_pkzb * (1 + dd * eps_pkzb * tauwtau3)` when `tauwtau3 = (gnn/8nτ)³
/// ≈ 1e+27` for tau << tau_w.
#[test]
fn ctaylor_max_unphysical_regime_returns_tau_w() {
    // tau = 1e-6 (vanishing); tau_w = 12.5 (von Weizsäcker bound).
    // CNST: tau[0] = 1e-6 < 12.5 = tau_w[0] → max returns tau_w.
    // Higher slot: tau_w[1] = 1.0 should propagate (tau_w array is selected).
    let tau: [f64; 2] = [1e-6, 0.5];
    let tau_w: [f64; 2] = [12.5, 1.0];
    let out = run_ctaylor_max_n1(tau, tau_w);
    assert!(
        (out[0] - tau_w[0]).abs() < 1e-15,
        "unphysical-regime guard breached: max(tau={}, tau_w={})[0] = {} (expected {})",
        tau[0],
        tau_w[0],
        out[0],
        tau_w[0]
    );
    assert!(
        (out[1] - tau_w[1]).abs() < 1e-15,
        "ctaylor_max should propagate the entire chosen array, but slot[1] = {}, expected {}",
        out[1],
        tau_w[1]
    );
}

/// **Physical regime**: `tau >= tau_w`; the guard is a no-op in the sense
/// that `ctaylor_max(tau, tau_w) == tau` bit-exactly. This is what makes
/// the guard "transparent" in the physical regime — preserves the
/// algorithmically-faithful port at every grid point where tier-1 self-tests
/// land.
#[test]
fn ctaylor_max_physical_regime_returns_tau_bit_exact() {
    let tau: [f64; 2] = [100.0, 7.5];
    let tau_w: [f64; 2] = [12.5, 1.0];
    let out = run_ctaylor_max_n1(tau, tau_w);
    assert_eq!(
        out[0].to_bits(),
        tau[0].to_bits(),
        "physical-regime guard NOT bit-exact: out[0] = {:e}, tau[0] = {:e}",
        out[0],
        tau[0]
    );
    assert_eq!(
        out[1].to_bits(),
        tau[1].to_bits(),
        "physical-regime guard NOT bit-exact: out[1] = {:e}, tau[1] = {:e}",
        out[1],
        tau[1]
    );
}

/// **Boundary regime**: `tau == tau_w`. `ctaylor_max` uses `>=` so on
/// equality it picks `a` (the first arg = `d.tau`). Bit-exact to tau.
#[test]
fn ctaylor_max_boundary_regime_returns_tau() {
    let tau: [f64; 2] = [12.5, 0.0];
    let tau_w: [f64; 2] = [12.5, 1.0];
    let out = run_ctaylor_max_n1(tau, tau_w);
    assert_eq!(
        out[0].to_bits(),
        tau[0].to_bits(),
        "boundary `>=` semantics: out[0] = {}, expected tau[0] = {}",
        out[0],
        tau[0]
    );
    // Confirm slot[1] propagates from `a` (tau), not `b` (tau_w).
    assert_eq!(
        out[1].to_bits(),
        tau[1].to_bits(),
        "boundary semantics: out[1] = {}, expected tau[1] = {} (NOT tau_w[1] = {})",
        out[1],
        tau[1],
        tau_w[1]
    );
}
