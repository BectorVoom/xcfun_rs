//! `cubecl-wgpu` (Vulkan / DX12 / Metal / WebGPU) probe + cached client.
//!
//! Phase 6 Plan 06-04 wires this module behind `feature = "wgpu"` per
//! D-06 (Wgpu portable fallback; relaxed 1e-9 tolerance per D-02). The
//! probe attempts `WgpuRuntime::client(&WgpuDevice::default())` once,
//! and additionally probes `client.properties().supports_type(
//! ElemType::Float(FloatKind::F64))` to enforce the SHADER_F64 contract
//! — refusing devices without f64 support rather than silently
//! downgrading to f32 (Pitfall 2 from RESEARCH; Phase 6 D-13/D-13-A).
//!
//! ## Pitfall 5: WGSL has no f64 type
//!
//! cubecl-wgpu's WGSL backend emits 32-bit code regardless of the
//! requested element type — only the SPIR-V backend honours f64
//! properly. The `supports_type` probe queries the per-backend
//! `Features` set populated at adapter init: WGSL adapters report
//! `f64` as unsupported (see `cubecl-wgpu/src/backend/wgsl.rs:103`
//! gates the `register_type_usage(F64, ...)` on Vulkan-with-f64 only).
//!
//! ## Apple Silicon caveat (D-06)
//!
//! Apple Silicon GPUs (M1/M2/M3/A17) lack hardware f64. cubecl-wgpu's
//! Metal backend on Apple Silicon will report `supports_type(F64) ==
//! false`, the probe returns `false`, `auto_backend()` falls through
//! to `Cpu`, and explicit `Batch::<WgpuRuntime>::open_wgpu()` callers
//! get `XcError::WgpuNoF64`.
//!
//! ## API note (W-4 / W-10 revision-1)
//!
//! The plan's literal pattern `feature_enabled(Feature::Type(Elem::
//! Float(FloatKind::F64)))` is the cubecl-book documentation phrasing
//! from before the 0.10.0-pre.3 API rename. The current public API on
//! `DeviceProperties` is `supports_type(impl Into<Type>)` — semantically
//! identical (both query the per-backend `Features` table for the f64
//! element type) but uses the documented stable accessor instead of an
//! API path that no longer exists in 0.10.0-pre.3.

use cubecl::Runtime;
use cubecl::ir::{ElemType, FloatKind};
use cubecl::prelude::ComputeClient;
use cubecl_wgpu::{WgpuDevice, WgpuRuntime};
use std::sync::OnceLock;

/// Concrete `cubecl-wgpu` compute client.
pub type WgpuClient = ComputeClient<WgpuRuntime>;

/// Probe outcome cache. `Some(client)` when both `WgpuRuntime::client`
/// init succeeds AND the device passes the f64 gate; `None` otherwise.
static WGPU_CLIENT: OnceLock<Option<WgpuClient>> = OnceLock::new();

/// Initialise + probe the default Wgpu adapter once. Returns
/// `Some(client)` only if the device's compiled feature set includes
/// `ElemType::Float(FloatKind::F64)`.
fn init_wgpu_with_f64() -> Option<WgpuClient> {
    let init = std::panic::catch_unwind(|| {
        let device = WgpuDevice::default();
        WgpuRuntime::client(&device)
    });
    let client = init.ok()?;
    if client
        .properties()
        .supports_type(ElemType::Float(FloatKind::F64))
    {
        Some(client)
    } else {
        None
    }
}

/// Probe whether the default Wgpu adapter is available AND reports
/// `SHADER_F64` (i.e. the device features table contains the f64
/// element type).
///
/// Returns `false` on:
/// - no Wgpu-compatible adapter (no Vulkan/Metal/DX12/WebGPU at all),
/// - WGSL-only backend (Pitfall 5 — register_type_usage(F64) not gated),
/// - Apple Silicon (no hardware f64),
/// - any panic during adapter init (driver mismatch, missing libs).
///
/// On a positive probe the `WgpuClient` is cached for downstream
/// `Batch::<WgpuRuntime>::open_wgpu()`. The cache is shared with
/// [`metal_with_f64_available`] — both end up using the same default
/// adapter which the OS resolves to Vulkan / Metal / DX12 as
/// appropriate.
pub fn wgpu_with_shader_f64_available() -> bool {
    WGPU_CLIENT
        .get_or_init(init_wgpu_with_f64)
        .is_some()
}

/// Apple Silicon Metal path — same probe as
/// [`wgpu_with_shader_f64_available`], gated on macOS.
///
/// On macOS the default Wgpu adapter resolves to the Metal backend; on
/// Linux/Windows this returns `false` so `auto_backend()` distinguishes
/// the Metal arm from the generic Wgpu arm. Per CONTEXT D-06 / D-07
/// priority chain, `auto_backend()` checks Metal first (macOS-specific)
/// before falling through to the Wgpu generic path.
pub fn metal_with_f64_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        wgpu_with_shader_f64_available()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Returns the cached `WgpuClient`. **Panics** when
/// `wgpu_with_shader_f64_available()` would return `false` — callers
/// MUST gate on the probe first.
///
/// `Batch::<WgpuRuntime>::open_wgpu()` is the canonical caller; it
/// invokes the probe directly and returns `XcError::WgpuNoF64` rather
/// than panicking.
pub fn wgpu_client() -> &'static WgpuClient {
    match WGPU_CLIENT.get() {
        Some(Some(c)) => c,
        Some(None) | None => panic!(
            "xcfun-gpu: wgpu_client() called when \
             wgpu_with_shader_f64_available() == false. The default Wgpu \
             adapter either is unavailable or lacks f64 support \
             (SHADER_F64). Apple Silicon GPUs and WGSL-only Vulkan \
             drivers are common offenders; see crates/xcfun-gpu/README.md."
        ),
    }
}

/// Construct the typed `XcError::WgpuNoF64` payload (D-13/D-13-A).
///
/// Used by `Batch::<WgpuRuntime>::open_wgpu` when the caller pre-selects
/// `Backend::Wgpu` or `Backend::Metal` but the f64 probe failed. The
/// `Runtime::name(&client)` accessor returns a `&'static str` derived
/// from compile-time backend identification (e.g. `"wgpu<spirv>"`,
/// `"wgpu<wgsl>"`, `"wgpu<msl>"`); no `Box::leak` is required because
/// the upstream API already exposes a `'static` lifetime.
///
/// On the path where Wgpu init itself failed (no adapter at all), we
/// fall back to a sentinel `&'static str` so the error variant is
/// always constructible.
pub fn wgpu_no_f64_error(requested: crate::Backend) -> xcfun_core::XcError {
    let adapter_name: &'static str =
        match std::panic::catch_unwind(|| {
            let device = WgpuDevice::default();
            let client = WgpuRuntime::client(&device);
            WgpuRuntime::name(&client)
        }) {
            Ok(name) => name,
            Err(_) => "<wgpu init failed>",
        };
    xcfun_core::XcError::WgpuNoF64 {
        adapter_name,
        requested_runtime: requested.into(),
    }
}
