//! KER-06 / GPU-01 — `Batch::<CpuRuntime>::eval_vec_host_cpu` matches the
//! per-point `Functional::eval` over a small grid.
//!
//! Plan 06-02a smoke test: the CPU host implementation iterates the
//! pitched density grid and dispatches each row to `Functional::eval`.
//! Every output row must equal the scalar `Functional::eval` output for
//! the same input within a tight tolerance — they share the same
//! cubecl-cpu launch path under the hood, so the tolerance is `0.0`
//! (bit-equal) here. Plan 06-05 RS-08 dispatch consumes this substrate
//! when `nr_points >= XCFUN_MIN_BATCH_SIZE` (default 64).
//!
//! 100 random density points × XC_SLATERX × order 0 — the smallest
//! end-to-end exercise that proves the host loop works.

#![cfg(feature = "cpu")]

use cubecl_cpu::CpuRuntime;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;
use xcfun_gpu::Batch;

static SLATERX_WEIGHTS: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];

#[test]
fn eval_vec_host_cpu_matches_scalar_eval_at_order_0() {
    let fun = Functional {
        weights: SLATERX_WEIGHTS,
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };
    let inlen = Functional::input_length(fun.vars);
    let outlen = Functional::output_length(fun.vars, fun.mode, fun.order)
        .expect("output_length");
    assert_eq!(inlen, 2);
    assert_eq!(outlen, 1);

    // 100 points with simple-but-non-trivial density values.
    let nr_points: usize = 100;
    let mut density = Vec::with_capacity(nr_points * inlen);
    for p in 0..nr_points {
        // `p / 100` keeps points in (0, 1); avoid 0 so the kernel
        // doesn't exercise the regularize-floor regime which is not
        // the focus of this smoke test.
        let rho_a = 0.01_f64 + (p as f64) * 0.0099_f64;
        let rho_b = 0.005_f64 + (p as f64) * 0.0049_f64;
        density.push(rho_a);
        density.push(rho_b);
    }

    let mut out_batch = vec![0.0_f64; nr_points * outlen];
    Batch::<CpuRuntime>::eval_vec_host_cpu(
        &fun,
        &density,
        inlen,
        &mut out_batch,
        outlen,
        nr_points,
    )
    .expect("eval_vec_host_cpu");

    // Reference: scalar Functional::eval per point.
    let mut out_scalar = vec![0.0_f64; nr_points * outlen];
    for p in 0..nr_points {
        let din = &density[p * inlen..p * inlen + inlen];
        let dout = &mut out_scalar[p * outlen..p * outlen + outlen];
        fun.eval(din, dout).expect("scalar eval");
    }

    for p in 0..nr_points {
        let b = out_batch[p * outlen];
        let s = out_scalar[p * outlen];
        // Both go through the same cubecl-cpu launch path; bit-equal
        // is the right contract here.
        assert_eq!(
            b, s,
            "point {p}: batch={b:.17e} != scalar={s:.17e}",
        );
    }
}
