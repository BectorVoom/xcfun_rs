//! `cubecl-hip` (ROCm/AMD) probe + `OnceLock<HipClient>` cache.
//!
//! Phase 6 Plan 06-03 wires this module behind `feature = "hip"` per
//! D-05 (ROCm primary). The probe attempts to construct a `HipClient`
//! via `HipRuntime::client(&AmdDevice::default())` once; success caches
//! the client for downstream `Batch::<HipRuntime>::open()` callers.
//!
//! ## RDNA-2 caveat (RESEARCH §"Pitfall 3")
//!
//! AMD RX 6000-series GPUs (gfx1031/1032/1033) are not officially supported
//! by ROCm. cubecl-hip will compile and the runtime will load, but kernel
//! launches fail with a cryptic "code object load failed" error. The
//! workaround documented upstream by both AMD and `cubecl-hip/README.md`:
//!
//! ```bash
//! export HSA_OVERRIDE_GFX_VERSION=10.3.0
//! ```
//!
//! coerces RDNA-2 to RDNA-3 PTX. This must be set in the **process
//! environment before `auto_backend()` (or any direct `Batch::open`) is
//! called**. The user-facing documentation lives in
//! `crates/xcfun-gpu/README.md`.
//!
//! ## Probe semantics
//!
//! `rocm_available()` is conservative: a client init failure (no `/opt/rocm`
//! installed; no compatible AMD GPU; HIP driver mismatch) returns `false`
//! without panicking. The probe wraps the init in `std::panic::catch_unwind`
//! because some failure modes (e.g. dynamic-link errors loading
//! `libamd_comgr.so`) can manifest as panics from inside `cubecl-hip-sys`
//! before the `Result`-returning constructors reach our control flow.
//!
//! On the happy path, the cached `HipClient` is returned by `hip_client()`
//! and consumed by `Batch::<HipRuntime>::open_rocm()` (mirrors the
//! `Batch::<CpuRuntime>::open_cpu()` analog from Plan 06-02a).

use cubecl::Runtime;
use cubecl::prelude::ComputeClient;
use cubecl_hip::{AmdDevice, HipRuntime};
use std::sync::OnceLock;

/// Concrete `cubecl-hip` compute client. Type alias keeps the module
/// API stable across cubecl 0.10-pre.N drift (the inner runtime type
/// is what changes shape between pre-releases).
pub type HipClient = ComputeClient<HipRuntime>;

/// Probe outcome cache. `Some(client)` => `rocm_available()` returned
/// `true`; `None` => probe failed (no ROCm runtime, no GPU, init panic).
/// We use `Option<HipClient>` so a probe failure caches the negative
/// result and avoids re-running the init on every `auto_backend()` call
/// (the second-call cost matters because Plan 06-05 / RS-08 may call
/// `auto_backend()` once per `eval_vec` invocation depending on caller
/// behaviour).
static HIP_CLIENT: OnceLock<Option<HipClient>> = OnceLock::new();

/// Probe whether ROCm/HIP is available on this machine.
///
/// Returns `false` when:
/// - `cubecl-hip` cannot find the ROCm runtime libraries (`/opt/rocm`
///   missing or `libamd_comgr.so` not on the loader path),
/// - no AMD GPU is visible to HIP,
/// - the HIP driver / runtime version mismatch causes init to panic.
///
/// On a positive probe the `HipClient` is cached for downstream
/// `Batch::<HipRuntime>::open_rocm()`. Caller is responsible for setting
/// `HSA_OVERRIDE_GFX_VERSION=10.3.0` on RDNA-2 hardware **before** the
/// first call to this function — the env var is consulted by HIP's
/// runtime loader at client construction time.
pub fn rocm_available() -> bool {
    HIP_CLIENT
        .get_or_init(|| {
            // Wrap in `catch_unwind` because cubecl-hip-sys can panic
            // during dynamic-link resolution of ROCm libs that are
            // missing or version-mismatched. We treat a panic as
            // "probe failed" rather than letting it propagate to the
            // priority-chain caller (which would crash the host
            // binary on a CI runner without ROCm installed).
            std::panic::catch_unwind(|| {
                let device = AmdDevice::default();
                HipRuntime::client(&device)
            })
            .ok()
        })
        .is_some()
}

/// Returns the cached `HipClient`. **Panics** when `rocm_available()`
/// would return `false` — callers MUST gate on the probe first. The
/// panic message points at the canonical fix (RDNA-2 env var).
///
/// `Batch::<HipRuntime>::open_rocm()` is the canonical caller; it
/// invokes `rocm_available()` directly and returns `XcError::Runtime`
/// rather than panicking when the probe fails.
pub fn hip_client() -> &'static HipClient {
    match HIP_CLIENT.get() {
        Some(Some(c)) => c,
        Some(None) | None => panic!(
            "xcfun-gpu: hip_client() called when rocm_available() == false. \
             Check that ROCm is installed (`/opt/rocm/bin/rocminfo` should \
             list a gfx target) and, on RX 6000-series GPUs, that \
             `HSA_OVERRIDE_GFX_VERSION=10.3.0` is set in the process \
             environment."
        ),
    }
}
