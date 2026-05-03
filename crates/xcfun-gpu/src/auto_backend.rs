//! `auto_backend()` — runtime backend selector.
//!
//! Phase 6 CONTEXT D-07 priority chain:
//!
//! ```text
//! XCFUN_FORCE_BACKEND
//!   ↓ (env unset)
//! ROCm  if cubecl-hip probe succeeds                  (Plan 06-03)
//!   ↓
//! CUDA  if cubecl-cuda probe succeeds                 (Plan 06-04)
//!   ↓
//! Metal if cubecl-wgpu Metal-with-f64 probe succeeds  (Plan 06-04)
//!   ↓
//! Wgpu  if cubecl-wgpu shader-f64 probe succeeds      (Plan 06-04)
//!   ↓
//! Cpu   (fallback — always available)
//! ```
//!
//! Plan 06-02a wires the env-var override + the cascading probe shape.
//! All non-CPU probes return `false` in 06-02a; Plans 06-03 / 06-04 flip
//! them on by replacing the stubs in `runtime/{hip,cuda,wgpu}.rs`.

use crate::Backend;

/// Select the highest-priority backend supported on this build + device.
///
/// Reads `XCFUN_FORCE_BACKEND` first; an unrecognised value PANICS so a
/// misconfigured CI job fails loudly rather than silently picking the
/// wrong backend. Recognised values are case-insensitive: `cpu`, `rocm`,
/// `hip`, `cuda`, `metal`, `wgpu` (matches `Backend::from_str`).
pub fn auto_backend() -> Backend {
    if let Ok(force) = std::env::var("XCFUN_FORCE_BACKEND") {
        return Backend::from_str(&force).unwrap_or_else(|| {
            panic!(
                "XCFUN_FORCE_BACKEND={force:?} unrecognised \
                 (expected one of: cpu | rocm | hip | cuda | metal | wgpu)"
            )
        });
    }

    #[cfg(feature = "hip")]
    if crate::runtime::hip::rocm_available() {
        return Backend::Rocm;
    }

    #[cfg(feature = "cuda")]
    if crate::runtime::cuda::cuda_available() {
        return Backend::Cuda;
    }

    #[cfg(feature = "wgpu")]
    if crate::runtime::wgpu::metal_with_f64_available() {
        return Backend::Metal;
    }

    #[cfg(feature = "wgpu")]
    if crate::runtime::wgpu::wgpu_with_shader_f64_available() {
        return Backend::Wgpu;
    }

    Backend::Cpu
}
