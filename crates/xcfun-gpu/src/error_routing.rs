//! ERF auto-fallback routing per GPU-05.
//!
//! Phase 6 CONTEXT D-13 / Pitfall 5: cubecl-wgpu's WGSL backend has no
//! f64 type; even when the Wgpu device reports `SHADER_F64`, range-
//! separated functionals carrying `Dependency::ERF` would emit 32-bit
//! WGSL shader code and break the 1e-12 contract. Same issue applies
//! to the Metal backend on Apple Silicon (no hardware f64).
//!
//! The mitigation: on `Wgpu` and `Metal`, ERF-bearing functionals fall
//! back to the CPU substrate. The strict 1e-13 contract is preserved on
//! the CPU path; the ROCm/CUDA paths handle ERF natively at f64.
//!
//! This module is consumed by Plan 06-05 (`xcfun-rs::Functional::eval_vec`)
//! which checks `must_fall_back_to_cpu` before dispatching to a
//! non-CPU `Batch<R>`. Plan 06-02a only declares the helper; downstream
//! plans wire the dispatch site.

use crate::Backend;
use xcfun_core::traits::Dependency;

/// `true` iff this backend cannot run an ERF-bearing functional at
/// f64 precision (Wgpu / Metal); the caller must route to the CPU
/// substrate instead.
///
/// `Backend::Cpu` always returns `false`. `Backend::Rocm` and
/// `Backend::Cuda` return `false` because their cubecl backends emit
/// PTX / GCN with native f64 `erf`. `Backend::Wgpu` and `Backend::Metal`
/// return `true` whenever the functional set contains
/// `Dependency::ERF`.
pub fn must_fall_back_to_cpu(deps: Dependency, backend: Backend) -> bool {
    deps.contains(Dependency::ERF) && matches!(backend, Backend::Wgpu | Backend::Metal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_never_falls_back() {
        assert!(!must_fall_back_to_cpu(Dependency::ERF, Backend::Cpu));
        assert!(!must_fall_back_to_cpu(
            Dependency::DENSITY | Dependency::ERF,
            Backend::Cpu,
        ));
    }

    #[test]
    fn rocm_and_cuda_handle_erf_natively() {
        assert!(!must_fall_back_to_cpu(Dependency::ERF, Backend::Rocm));
        assert!(!must_fall_back_to_cpu(Dependency::ERF, Backend::Cuda));
    }

    #[test]
    fn wgpu_and_metal_with_erf_fall_back() {
        assert!(must_fall_back_to_cpu(Dependency::ERF, Backend::Wgpu));
        assert!(must_fall_back_to_cpu(Dependency::ERF, Backend::Metal));
    }

    #[test]
    fn wgpu_and_metal_without_erf_do_not_fall_back() {
        assert!(!must_fall_back_to_cpu(Dependency::DENSITY, Backend::Wgpu));
        assert!(!must_fall_back_to_cpu(
            Dependency::DENSITY | Dependency::GRADIENT,
            Backend::Metal,
        ));
    }
}
