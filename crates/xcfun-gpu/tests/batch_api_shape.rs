//! GPU-01 — compile-time gate: `Batch<'fun, R>` exposes the contract API
//! (`reserve`, `upload_density`, `launch`, `download_result`,
//! `eval_vec_host`) AND the lifetime is bound to `&'fun
//! xcfun_eval::Functional` (W-3 revision-1, NOT `&xcfun_rs::Functional`).
//!
//! cubecl's `Runtime: 'static + Send + Sync` so a `Batch<'_, CpuRuntime>`
//! must inherit `Send` (the `&'fun Functional` borrow is `Send` because
//! `Functional: Send + Sync` per Phase 5 D-17).
//!
//! Phase 6 Plan 06-02a: only the CPU arm is wired. Plans 06-03 / 06-04
//! flip the same compile-time gate on for `HipRuntime` / `CudaRuntime` /
//! `WgpuRuntime` once they're behind their feature flags.

#![cfg(feature = "cpu")]

use cubecl_cpu::CpuRuntime;
use static_assertions::assert_impl_all;
use xcfun_gpu::Batch;

// `Batch<'_, CpuRuntime>` must be Send (cubecl::Runtime: 'static + Send + Sync;
// the &'fun Functional reference is Send because Functional is Send+Sync per
// Phase 5 RS-10).
assert_impl_all!(Batch<'static, CpuRuntime>: Send);

#[test]
fn batch_api_surface_exists() {
    // Compile-only smoke check — confirms each of the GPU-01 methods
    // exists with a callable signature. We don't actually call them
    // here (the generic bodies return `XcError::Runtime` until Plans
    // 06-03 / 06-04 wire concrete launches), but referencing each
    // method (or its first-class fn item) ensures the symbol is
    // reachable from the public API.
    //
    // The `let _ = ... as ...;` cast pattern doesn't work for methods
    // whose signature carries inferred lifetimes; we use the lighter
    // method-pointer pattern (`let _: fn(...) -> ...`) instead. If a
    // method is renamed or its signature changes incompatibly, this
    // file fails to compile.
    fn _assert_methods_exist<'fun>(b: &mut Batch<'fun, CpuRuntime>) {
        let _r: () = b.reserve(0);
        // Each call is unreachable at runtime (we never invoke the
        // function), but the borrow checker still requires lifetime
        // soundness — the inputs are constructed inside the closure.
        let dummy_in: &[f64] = &[];
        let dummy_out: &mut [f64] = &mut [];
        let _u: Result<(), xcfun_core::XcError> = b.upload_density(dummy_in, 0, 0);
        let _l: Result<(), xcfun_core::XcError> = b.launch(0);
        let _d: Result<(), xcfun_core::XcError> = b.download_result(dummy_out, 0, 0);
    }
    // Ensure the generic associated fn is reachable from outside.
    fn _assert_eval_vec_host_exists(fun: &xcfun_eval::Functional) {
        let mut out: Vec<f64> = Vec::new();
        let _e: Result<(), xcfun_core::XcError> =
            Batch::<CpuRuntime>::eval_vec_host(fun, &[], 0, &mut out, 0, 0);
    }
    // Discard so the helpers are flagged "used" but never invoked.
    let _ = _assert_methods_exist as fn(&mut Batch<'_, CpuRuntime>);
    let _ = _assert_eval_vec_host_exists as fn(&xcfun_eval::Functional);
}
