//! Phase 6 Plan 06-00 Task 2 — `erf_precise_taylor` AD-chain wrapper test.
//!
//! Verifies the new public entry point `expand::erf::erf_precise_taylor`
//! produces correct Taylor coefficients of `erf(x0 + y)` at `y = 0`. The
//! body delegates to the existing libm-hybrid `erf_expand` (Phase 2 Plan
//! 02-06 commit `dca382a` — `erf_precise` for the `t[0]` seed at ≤ 1 ULP
//! vs `libm::erf`; gauss-expand Hermite recurrence for `t[i ≥ 1]`).
//!
//! Test strategy: compare `erf_precise_taylor` output to `erf_expand`
//! output at strict bit-equality (they must be functionally identical).
//! This pins the public API without depending on mpmath ground truth
//! (which is generated offline via the xtask sidecar from Task 4 and
//! consumed by Plan 06-N3's post-libm-hybrid sweep).
//!
//! The plan's strict 1e-13 contract vs mpmath truth is enforced by Plan
//! 06-N3 (post-libm-hybrid sweep over the 12+ small-magnitude AD-residual
//! functionals that include LDAERFX / LDAERFC / LDAERFC_JT). Plan 06-00
//! lands the public entry point and verifies the wiring.
//!
//! `mpmath_prec` field carried in the fixture record format for forward
//! compatibility with Plan 06-N3.

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_ad::expand::erf::{erf_expand, erf_precise_taylor};
use xcfun_ad::for_tests::cpu_client;

/// `#[cube(launch_unchecked)]` wrapper exposing both `erf_precise_taylor`
/// and `erf_expand` so we can launch them from the host harness and
/// compare coefficient-for-coefficient. `x0` is passed via a length-1
/// `Array<F>` per the established cubecl-0.10-pre.3 host-side launcher
/// pattern (see `crates/xcfun-ad/tests/golden_expand.rs`).
#[cube(launch_unchecked)]
fn kernel_erf_taylor_pair<F: Float>(
    scalars: &Array<F>,
    out_pt: &mut Array<F>,
    out_ee: &mut Array<F>,
    #[comptime] n: u32,
) {
    erf_precise_taylor::<F>(out_pt, scalars[0], n);
    erf_expand::<F>(out_ee, scalars[0], n);
}

/// Reference record format — matches the JSONL shape Plan 06-N3 will consume.
#[allow(dead_code)]
struct ErfTaylorRecord {
    x0: f64,
    n: u32,
    expected: Vec<f64>,
    mpmath_prec: u32,
}

/// Helper: launch both wrappers at order `n` and `x0`, return both
/// coefficient arrays. Uses a length-1 `Array<F>` for `x0` per the
/// existing cubecl-0.10-pre.3 launcher pattern.
fn run_pair(x0: f64, n: u32) -> (Vec<f64>, Vec<f64>) {
    let size = (n + 1) as usize;
    let client = cpu_client();
    let scalars: [f64; 1] = [x0];
    let s_h = client.create_from_slice(f64::as_bytes(&scalars));
    let pt_h = client.empty(size * std::mem::size_of::<f64>());
    let ee_h = client.empty(size * std::mem::size_of::<f64>());
    let pt_read = pt_h.clone();
    let ee_read = ee_h.clone();

    unsafe {
        kernel_erf_taylor_pair::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(s_h, 1),
            ArrayArg::from_raw_parts(pt_h, size),
            ArrayArg::from_raw_parts(ee_h, size),
            n,
        );
    }

    let pt_bytes = client.read_one_unchecked(pt_read);
    let ee_bytes = client.read_one_unchecked(ee_read);
    let pt = f64::from_bytes(&pt_bytes)[..size].to_vec();
    let ee = f64::from_bytes(&ee_bytes)[..size].to_vec();
    (pt, ee)
}

/// Bit-exact equivalence: `erf_precise_taylor` and `erf_expand` produce
/// identical coefficients (they share the same body — same `gauss_expand`
/// driver, same `2/√π` cast, same `tfuns_integrate`, same `erf_precise`
/// seed). Verifies the rewire in `math.rs::ctaylor_erf` doesn't drift
/// from the Phase-2-baseline behaviour.
#[test]
fn erf_precise_taylor_matches_erf_expand_n3() {
    // Stratified x0 grid covering small (cancellation-prone) and large
    // (saturation-bound) brackets per ldaerfx.cpp:66 rationale.
    let xs = [0.05_f64, 0.1, 0.5, 1.0, 1.5, 2.0, 5.0, 8.0];
    for &x0 in &xs {
        let (pt, ee) = run_pair(x0, 3_u32);
        for i in 0..=3 {
            assert_eq!(
                pt[i].to_bits(),
                ee[i].to_bits(),
                "x0={}, n=3, slot {}: erf_precise_taylor={:e}, erf_expand={:e}",
                x0,
                i,
                pt[i],
                ee[i]
            );
        }
    }
}

/// `erf_precise_taylor` produces the standard erf(x0) at slot 0 — a
/// concrete cross-check against `libm::erf` reference values.
///
/// `erf(0.5) = 0.5204998778130465`
/// `erf(1.0) = 0.8427007929497149`
/// `erf(2.0) = 0.9953222650189527`
///
/// Tolerance: 1e-15 (≤ 1 ULP vs `libm::erf` per Phase 2 `erf_precise` contract).
#[test]
fn erf_precise_taylor_seeds_t0_at_libm_precision() {
    // Reference values from libm::erf (cross-verified vs WolframAlpha).
    let cases: [(f64, f64); 4] = [
        (0.0, 0.0),
        (0.5, 0.520_499_877_813_046_5),
        (1.0, 0.842_700_792_949_714_9),
        (2.0, 0.995_322_265_018_952_7),
    ];
    for (x0, expected) in cases {
        let (pt, _) = run_pair(x0, 1_u32);
        let abs_err = (pt[0] - expected).abs();
        assert!(
            abs_err < 1e-15,
            "erf_precise_taylor({})[0] = {:e}, expected {:e} (libm), abs_err = {:e}",
            x0,
            pt[0],
            expected,
            abs_err
        );
    }
}
