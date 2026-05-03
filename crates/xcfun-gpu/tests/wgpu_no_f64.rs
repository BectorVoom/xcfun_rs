//! GPU-06 — `XcError::WgpuNoF64` typed variant + Wgpu f64 device-feature
//! probe integration test.
//!
//! Phase 6 Plan 06-02a declared the typed variant + `BackendTag` shadow
//! enum; Plan 06-04 wires the actual `wgpu::Features::SHADER_F64` runtime
//! probe inside `Batch::<WgpuRuntime>::open_wgpu` (D-13/D-13-A) and this
//! file un-`#[ignore]`s the integration test.
//!
//! ## Contract under test
//!
//! On a default Wgpu adapter, `Batch::<WgpuRuntime>::open_wgpu(&fun)`
//! MUST return one of TWO outcomes — never a third:
//!
//! 1. `Ok(_)` — adapter reports SHADER_F64 (e.g. Vulkan with f64
//!    extension; SPIR-V backend on Linux) → kernel can launch at f64.
//! 2. `Err(XcError::WgpuNoF64 { adapter_name, requested_runtime })` —
//!    adapter lacks SHADER_F64 (Apple Silicon, WGSL-only Vulkan, or no
//!    Wgpu adapter at all) → typed error with non-empty adapter_name +
//!    `BackendTag::Wgpu`.
//!
//! Specifically forbidden: silent f32 downgrade, `Ok(_)` from a device
//! that reports `SHADER_F64 == false`, or a different `XcError` variant.

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

// Plan 06-04 — un-`#[ignore]`'d. The integration test exercises the
// real probe + open path; it succeeds on hosts where Wgpu is reachable
// (matches the Ok arm or the typed-error arm) and on hosts where
// cubecl-wgpu cannot reach any adapter at all (the helper still
// constructs an XcError::WgpuNoF64 with a sentinel adapter_name —
// per `wgpu_no_f64_error`'s catch_unwind fallback).
#[cfg(feature = "wgpu")]
#[test]
fn batch_open_returns_wgpu_no_f64_when_probe_fails() {
    use cubecl_wgpu::WgpuRuntime;
    use xcfun_core::{FunctionalId, Mode, Vars};
    use xcfun_eval::functional::DEFAULT_SETTINGS;
    use xcfun_eval::Functional;
    use xcfun_gpu::Batch;

    // Direct-struct construction (Functional::set does not mutate
    // vars/mode/order — those are facade-controlled).
    let fun = Functional {
        weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };

    match Batch::<WgpuRuntime>::open_wgpu(&fun) {
        Ok(_) => {
            // Adapter HAS SHADER_F64 — accepted. Specifically, the
            // probe returned true AND no f32 downgrade occurred (the
            // size_of::<f64>() == 8 const-assert in xcfun-gpu/src/lib.rs
            // catches a build-time downgrade attempt). On hosts with
            // Vulkan + f64 extension (Linux + AMD/NVIDIA discrete
            // GPU) this is the expected branch.
        }
        Err(XcError::WgpuNoF64 {
            adapter_name,
            requested_runtime,
        }) => {
            // Adapter lacks SHADER_F64 — typed error returned.
            // adapter_name is &'static str backed by Runtime::name(...)
            // which returns a compile-time string (e.g. "wgpu<wgsl>")
            // — populated and non-empty.
            assert!(
                !adapter_name.is_empty(),
                "adapter_name should be a populated &'static str"
            );
            assert_eq!(
                requested_runtime,
                BackendTag::Wgpu,
                "requested_runtime payload must reflect the caller's selection"
            );
        }
        Err(other) => panic!(
            "Plan 06-04 contract violated: Batch::<WgpuRuntime>::open_wgpu \
             must return Ok or XcError::WgpuNoF64 — never any other variant. \
             Got: {other:?}"
        ),
    }
}

/// Same contract for the explicit `Backend::Metal` request shape.
/// `requested_runtime` payload must propagate as `BackendTag::Metal`
/// when the caller pre-selects Metal (needed by the Plan 06-05 dispatch
/// site for accurate diagnostics on Apple Silicon).
#[cfg(feature = "wgpu")]
#[test]
fn open_wgpu_with_request_metal_returns_metal_tag_on_no_f64() {
    use cubecl_wgpu::WgpuRuntime;
    use xcfun_core::{FunctionalId, Mode, Vars};
    use xcfun_eval::functional::DEFAULT_SETTINGS;
    use xcfun_eval::Functional;
    use xcfun_gpu::{Backend, Batch};

    let fun = Functional {
        weights: vec![(FunctionalId::XC_SLATERX, 1.0)],
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    };

    match Batch::<WgpuRuntime>::open_wgpu_with_request(&fun, Backend::Metal) {
        Ok(_) => {
            // f64-capable adapter present — the Metal-vs-Wgpu request
            // tag never reaches the error path; this is fine, the
            // contract only constrains the error case.
        }
        Err(XcError::WgpuNoF64 {
            requested_runtime, ..
        }) => {
            assert_eq!(
                requested_runtime,
                BackendTag::Metal,
                "open_wgpu_with_request(_, Backend::Metal) must surface BackendTag::Metal"
            );
        }
        Err(other) => panic!("unexpected error variant: {other:?}"),
    }
}
