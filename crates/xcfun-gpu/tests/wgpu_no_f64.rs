//! GPU-06 — `XcError::WgpuNoF64` typed variant + `BackendTag` shadow.
//!
//! Phase 6 Plan 06-02a only declares the variant + shadow enum;
//! Plan 06-04 wires the actual `wgpu::Features::SHADER_F64` runtime
//! probe inside `Batch::open` (the `wgpu` feature flag isn't compiled
//! in this plan). The full `Batch::open(...)` → `Err(WgpuNoF64 { ... })`
//! integration assertion lives behind `#[ignore]` until 06-04 lights
//! up the wgpu arm.
//!
//! Today's gate: the variant exists, is `Copy + non_exhaustive`-
//! compatible, and Display formats the adapter name + requested
//! runtime — i.e., the shape Plan 06-04 will populate from
//! `wgpu::AdapterInfo::name`.

use xcfun_core::{BackendTag, XcError};

#[test]
fn wgpu_no_f64_constructible() {
    let err = XcError::WgpuNoF64 {
        adapter_name: "Apple M2",
        requested_runtime: BackendTag::Wgpu,
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("Apple M2"),
        "Display does not echo adapter name: {msg}"
    );
    assert!(
        msg.contains("Wgpu"),
        "Display does not echo requested_runtime: {msg}"
    );
    // `Copy` round-trip — same compile-time gate as
    // xcerror_copy_invariant, but local to this file so a future
    // payload regression here is caught even if the cross-crate test
    // is silenced.
    let _a = err;
    let _b = err;
}

#[test]
fn wgpu_no_f64_with_metal_tag() {
    // The Metal arm shares the SHADER_F64 contract; same typed error,
    // different `requested_runtime` payload.
    let err = XcError::WgpuNoF64 {
        adapter_name: "Apple A17",
        requested_runtime: BackendTag::Metal,
    };
    assert_eq!(err.as_c_code(), -1);
}

// Plan 06-04 un-ignores this test and wires the actual probe. Until
// then the wgpu feature isn't compiled into xcfun-gpu, so a real
// `Batch::open` integration check is impossible.
#[ignore = "Plan 06-04 wires the wgpu f64-probe in Batch::open"]
#[test]
fn batch_open_returns_wgpu_no_f64_when_probe_fails() {
    // Placeholder. When 06-04 lights this up the body becomes:
    //   let fun = ...;
    //   let result = Batch::<WgpuRuntime>::open(&fun, /* low-f64 device */);
    //   assert!(matches!(result, Err(XcError::WgpuNoF64 { .. })));
}
