//! `cubecl-cuda` (NVIDIA) probe + `OnceLock<CudaClient>` cache.
//!
//! Phase 6 Plan 06-04 wires this module behind `feature = "cuda"` per
//! D-06 (CUDA opt-in best-effort; no NVIDIA hardware in dev environment).
//! The probe attempts `CudaRuntime::client(&CudaDevice::default())` once,
//! and additionally probes `client.properties().supports_type(
//! ElemType::Float(FloatKind::F64))` per W-7 (revision-1) before caching
//! the client. CUDA f64 is "always supported" on real hardware in
//! practice, but the cubecl-book feature matrix flags it as "?" — the
//! defensive probe prevents a silent f32 downgrade if a CI runner ever
//! presents a CUDA device that fails the gate.
//!
//! ## Probe semantics
//!
//! `cuda_available()` is conservative: a client init failure (no CUDA
//! toolkit on the loader path; no CUDA-capable GPU; driver mismatch)
//! returns `false` without panicking. The probe wraps the init in
//! `std::panic::catch_unwind` because some failure modes (e.g. dynamic-
//! link errors loading `libcuda.so` / `libcudart.so`) can manifest as
//! panics from inside `cubecl-cuda` before the `Result`-returning
//! constructors reach our control flow.
//!
//! On the happy path, the cached `CudaClient` is returned by
//! `cuda_client()` and consumed by `Batch::<CudaRuntime>::open_cuda()`
//! (mirrors the `Batch::<HipRuntime>::open_rocm()` analog from Plan
//! 06-03).
//!
//! ## API note (W-4 / W-10 revision-1)
//!
//! The plan's literal pattern `feature_enabled(Feature::Type(Elem::
//! Float(FloatKind::F64)))` is the cubecl-book documentation phrasing,
//! but in cubecl 0.10.0-pre.3 the equivalent runtime API is
//! `DeviceProperties::supports_type(impl Into<Type>)` where
//! `ElemType: Into<Type>` via the `From<T: Into<StorageType>> for Type`
//! chain. We use the actual API; semantic intent is identical (probe
//! whether the device's compiled feature set includes the f64 element
//! type).

use cubecl::Runtime;
use cubecl::ir::{ElemType, FloatKind};
use cubecl::prelude::ComputeClient;
use cubecl_cuda::{CudaDevice, CudaRuntime};
use std::sync::OnceLock;

/// Concrete `cubecl-cuda` compute client. Type alias keeps the module
/// API stable across cubecl 0.10-pre.N drift (the inner runtime type
/// is what changes shape between pre-releases).
pub type CudaClient = ComputeClient<CudaRuntime>;

/// Probe outcome cache. `Some(client)` => `cuda_available()` returned
/// `true`; `None` => probe failed (no CUDA toolkit, no GPU, init panic,
/// or f64 not supported on this device). We use `Option<CudaClient>` so
/// a probe failure caches the negative result and avoids re-running the
/// init on every `auto_backend()` call.
static CUDA_CLIENT: OnceLock<Option<CudaClient>> = OnceLock::new();

/// Probe whether CUDA is available on this machine AND the device
/// supports `f64`.
///
/// Returns `false` when:
/// - `cubecl-cuda` cannot find the CUDA runtime libraries (`libcuda.so`
///   or `libcudart.so` missing from the loader path),
/// - no NVIDIA GPU is visible to CUDA,
/// - the CUDA driver / runtime version mismatch causes init to panic,
/// - the device's compiled feature set does NOT include
///   `ElemType::Float(FloatKind::F64)` per W-7 defensive gate.
///
/// On a positive probe the `CudaClient` is cached for downstream
/// `Batch::<CudaRuntime>::open_cuda()`. A failed f64 gate caches `None`
/// so `auto_backend()` falls through to the next priority — and
/// `Batch::<CudaRuntime>::open_cuda()` returns `XcError::CudaNoF64`
/// when called explicitly.
pub fn cuda_available() -> bool {
    CUDA_CLIENT
        .get_or_init(|| {
            // Wrap in `catch_unwind` because `cubecl-cuda` can panic
            // during dynamic-link resolution of CUDA libs that are
            // missing or version-mismatched. We treat a panic as
            // "probe failed" rather than letting it propagate.
            let init = std::panic::catch_unwind(|| {
                let device = CudaDevice::default();
                CudaRuntime::client(&device)
            });
            match init {
                Ok(client) => {
                    // W-7 (revision-1) defensive f64 gate. If the device
                    // reports it cannot handle the f64 element type,
                    // refuse to cache the client — `auto_backend()`
                    // falls through to the next backend, and explicit
                    // `Batch::<CudaRuntime>::open_cuda` returns the
                    // typed `XcError::CudaNoF64` error.
                    if client
                        .properties()
                        .supports_type(ElemType::Float(FloatKind::F64))
                    {
                        Some(client)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .is_some()
}

/// Returns the cached `CudaClient`. **Panics** when `cuda_available()`
/// would return `false` — callers MUST gate on the probe first.
///
/// `Batch::<CudaRuntime>::open_cuda()` is the canonical caller; it
/// invokes `cuda_available()` directly and returns either
/// `XcError::CudaNoF64` (if the probe was reached but the f64 gate
/// failed) or `XcError::Runtime` (if the probe init itself failed)
/// rather than panicking.
pub fn cuda_client() -> &'static CudaClient {
    match CUDA_CLIENT.get() {
        Some(Some(c)) => c,
        Some(None) | None => panic!(
            "xcfun-gpu: cuda_client() called when cuda_available() == false. \
             Check that the CUDA toolkit is installed (`nvidia-smi` should \
             list a device), and that the device supports f64 (W-7 gate)."
        ),
    }
}

/// Construct the typed `XcError::CudaNoF64` payload for the active CUDA
/// adapter (D-13-A symmetric pattern; W-7 revision-1).
///
/// The `Runtime::name(&client)` accessor returns a `&'static str` — no
/// `Box::leak` is required for cubecl 0.10.0-pre.3 (verified by reading
/// `cubecl_cuda::CudaRuntime::name` which returns the literal `"cuda"`).
/// Used by `Batch::<CudaRuntime>::open_cuda` when the caller pre-selects
/// `Backend::Cuda` but the f64 probe failed.
pub fn cuda_no_f64_error(requested: crate::Backend) -> xcfun_core::XcError {
    // Best-effort recover the adapter name. If client construction
    // succeeds (the f64 probe failed but init succeeded), we get the
    // runtime's `&'static str` name. Otherwise fall back to a sentinel.
    let adapter_name: &'static str = match std::panic::catch_unwind(|| {
        let device = CudaDevice::default();
        let client = CudaRuntime::client(&device);
        CudaRuntime::name(&client)
    }) {
        Ok(name) => name,
        Err(_) => "<cuda init failed>",
    };
    xcfun_core::XcError::CudaNoF64 {
        adapter_name,
        requested_runtime: requested.into(),
    }
}
