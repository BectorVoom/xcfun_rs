//! Phase 6 Plan 06-02a — compile-time gate: `XcError` MUST stay `Copy + Send +
//! Sync` after the addition of the `WgpuNoF64` (D-13/D-13-A) and `CudaNoF64`
//! (W-7 revision-1) typed variants.
//!
//! Both new variants carry `&'static str + BackendTag` payloads (NOT `String`
//! / NOT a non-Copy type) precisely to preserve this invariant. If a future
//! variant ever adds a non-Copy field, this test breaks at compile time.
//!
//! Phase 2 D-25 sets the `Copy + non_exhaustive` contract; this gate catches
//! regressions there as well.

use static_assertions::assert_impl_all;
use xcfun_core::{BackendTag, XcError};

assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug);
assert_impl_all!(BackendTag: Copy, Clone, Send, Sync, std::fmt::Debug);

#[test]
fn wgpu_no_f64_round_trips_copy() {
    // Construct the new variant and copy it twice; ensures the variant is
    // actually Copy at runtime (the assert_impl_all above is the static
    // gate; this is a smoke check).
    let err = XcError::WgpuNoF64 {
        adapter_name: "test-adapter",
        requested_runtime: BackendTag::Wgpu,
    };
    let _a = err;
    let _b = err;
    let msg = format!("{err}");
    assert!(msg.contains("test-adapter"), "got: {msg}");
    assert!(msg.contains("Wgpu"), "got: {msg}");
}

#[test]
fn cuda_no_f64_round_trips_copy() {
    let err = XcError::CudaNoF64 {
        adapter_name: "fictitious-cuda-device",
        requested_runtime: BackendTag::Cuda,
    };
    let _a = err;
    let _b = err;
    let msg = format!("{err}");
    assert!(msg.contains("fictitious-cuda-device"), "got: {msg}");
    assert!(msg.contains("Cuda"), "got: {msg}");
}

#[test]
fn wgpu_no_f64_as_c_code_is_minus_one() {
    let err = XcError::WgpuNoF64 {
        adapter_name: "x",
        requested_runtime: BackendTag::Wgpu,
    };
    assert_eq!(err.as_c_code(), -1);
}

#[test]
fn cuda_no_f64_as_c_code_is_minus_one() {
    let err = XcError::CudaNoF64 {
        adapter_name: "x",
        requested_runtime: BackendTag::Cuda,
    };
    assert_eq!(err.as_c_code(), -1);
}
