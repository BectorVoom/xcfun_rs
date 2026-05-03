//! `Backend` enum — runtime discriminator. Phase 6 CONTEXT D-07 priority
//! order: env `XCFUN_FORCE_BACKEND` → ROCm → CUDA → Metal-with-f64 →
//! Wgpu-with-f64 → CPU.
//!
//! Five variants are declared even though Plans 06-03 / 06-04 are what
//! actually wire `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu` — keeping the
//! enum shape stable across plans avoids churn in downstream `match`
//! arms (06-05 / 06-06 / 06-N1..N3).
//!
//! Note: `cubecl-metal` does NOT exist as a separate crate (RESEARCH
//! §"Pitfall 9" / R-02). The `Metal` variant is reached via the Metal
//! backend of `cubecl-wgpu`. The `metal` feature flag in Cargo.toml is a
//! transparent alias for `wgpu`.

/// Runtime discriminator. Ordering follows CONTEXT D-07 priority chain
/// (Cpu first as the always-available substrate; Rocm before Cuda per
/// D-05 ROCm-primary policy).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Backend {
    /// `cubecl-cpu` substrate. Always available; tier-3 strict-1e-13
    /// gate per ACC-04 / D-02.
    Cpu,
    /// `cubecl-hip` — D-05 primary GPU backend (AMD/ROCm).
    Rocm,
    /// `cubecl-cuda` — opt-in NVIDIA backend per D-06.
    Cuda,
    /// `cubecl-wgpu` Metal backend — opt-in Apple Silicon path. Apple
    /// Silicon GPUs lack hardware f64; the runtime probe in Plan 06-04
    /// must refuse to launch when the device fails the f64 check.
    Metal,
    /// `cubecl-wgpu` generic backend (Vulkan / DX12 / WebGPU). Validated
    /// at the relaxed 1e-9 envelope per D-02 / Pitfall 5.
    Wgpu,
}

impl Backend {
    /// Parse the `XCFUN_FORCE_BACKEND` env-var value. Case-insensitive;
    /// returns `None` when the value matches no recognised variant.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "cpu" => Some(Backend::Cpu),
            // Accept both `rocm` and the cubecl crate name `hip` for the
            // ROCm-primary backend.
            "rocm" | "hip" => Some(Backend::Rocm),
            "cuda" => Some(Backend::Cuda),
            "metal" => Some(Backend::Metal),
            "wgpu" => Some(Backend::Wgpu),
            _ => None,
        }
    }
}

// Bidirectional conversion to/from the `xcfun-core::BackendTag` shadow
// enum. `BackendTag` lives in `xcfun-core` to avoid the layering
// inversion that would arise if `xcfun-core` (foundation) depended on
// `xcfun-gpu` (consumer). Phase 6 Plan 06-02a CONTEXT D-13.
impl From<Backend> for xcfun_core::BackendTag {
    fn from(b: Backend) -> Self {
        match b {
            Backend::Cpu => xcfun_core::BackendTag::Cpu,
            Backend::Rocm => xcfun_core::BackendTag::Rocm,
            Backend::Cuda => xcfun_core::BackendTag::Cuda,
            Backend::Metal => xcfun_core::BackendTag::Metal,
            Backend::Wgpu => xcfun_core::BackendTag::Wgpu,
        }
    }
}

impl From<xcfun_core::BackendTag> for Backend {
    fn from(t: xcfun_core::BackendTag) -> Self {
        match t {
            xcfun_core::BackendTag::Cpu => Backend::Cpu,
            xcfun_core::BackendTag::Rocm => Backend::Rocm,
            xcfun_core::BackendTag::Cuda => Backend::Cuda,
            xcfun_core::BackendTag::Metal => Backend::Metal,
            xcfun_core::BackendTag::Wgpu => Backend::Wgpu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_round_trip() {
        assert_eq!(Backend::from_str("cpu"), Some(Backend::Cpu));
        assert_eq!(Backend::from_str("CPU"), Some(Backend::Cpu));
        assert_eq!(Backend::from_str("rocm"), Some(Backend::Rocm));
        assert_eq!(Backend::from_str("hip"), Some(Backend::Rocm));
        assert_eq!(Backend::from_str("cuda"), Some(Backend::Cuda));
        assert_eq!(Backend::from_str("metal"), Some(Backend::Metal));
        assert_eq!(Backend::from_str("wgpu"), Some(Backend::Wgpu));
        assert_eq!(Backend::from_str("unrecognised"), None);
        assert_eq!(Backend::from_str(""), None);
    }

    #[test]
    fn backend_tag_conversion_is_total() {
        for b in [
            Backend::Cpu,
            Backend::Rocm,
            Backend::Cuda,
            Backend::Metal,
            Backend::Wgpu,
        ] {
            let t: xcfun_core::BackendTag = b.into();
            let round: Backend = t.into();
            assert_eq!(round, b);
        }
    }
}
