//! Error types for xcfun-core.
//!
//! `XcError` is `Copy + Clone + Debug + Send + Sync + #[non_exhaustive]` per CORE-04.
//! D-25: `UnknownName` is a unit variant -- the offending name is captured at the
//! call site (FFI boundary, dispatcher) and can be logged there; the error type
//! itself stays Copy-compatible.

use crate::enums::{Mode, Vars};
use crate::traits::Dependency;

/// Subset of `xcfun-gpu::Backend` used by `XcError` typed runtime errors.
///
/// Lives in `xcfun-core` to avoid the layering inversion that would arise if
/// `xcfun-core` (the foundational crate) depended on `xcfun-gpu` (which sits
/// above it). Phase 6 Plan 06-02a CONTEXT D-13 / D-13-A; W-7 from
/// revision-1. `xcfun-gpu::Backend` provides `From`/`Into` to convert
/// between the two enums.
///
/// Field ordering MUST mirror `xcfun-gpu::Backend` so the discriminants stay
/// in agreement when both enums are in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendTag {
    Cpu,
    Rocm,
    Cuda,
    Metal,
    Wgpu,
}

/// Errors returned by xcfun library operations.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[non_exhaustive]
pub enum XcError {
    #[error("invalid derivative order {order} for mode {mode:?} with {n_vars} input variables")]
    InvalidOrder {
        order: u32,
        mode: Mode,
        n_vars: usize,
    },

    #[error("variable type {vars:?} does not provide required dependencies {required:?}")]
    InvalidVars {
        vars: Vars,
        required: Dependency,
    },

    #[error("mode {mode:?} is not supported for functionals with dependencies {depends:?}")]
    InvalidMode {
        mode: Mode,
        depends: Dependency,
    },

    /// Combined `XC_EVARS | XC_EMODE` (= 6) returned by
    /// `xcfun-master/src/XCFunctional.cpp:441-443` when `Mode::Potential`
    /// is requested for a GGA-tier functional set whose `Vars` is not
    /// one of the `_2ND_TAYLOR` arms.
    #[error("vars {vars:?} and mode {mode:?} both invalid for dependencies {depends:?}")]
    InvalidVarsAndMode {
        vars: Vars,
        mode: Mode,
        depends: Dependency,
    },

    #[error("unknown functional name")]
    UnknownName,

    #[error("input length {got} does not match expected {expected}")]
    InputLengthMismatch { expected: usize, got: usize },

    #[error("output length {got} does not match expected {expected}")]
    OutputLengthMismatch { expected: usize, got: usize },

    #[error("functional not configured: call eval_setup() before eval()")]
    NotConfigured,

    #[error("invalid input encoding")]
    InvalidEncoding,

    #[error("runtime error during kernel launch")]
    Runtime,

    /// Phase 6 Plan 06-02a (D-13 / D-13-A) — Wgpu device lacks `SHADER_F64`.
    ///
    /// `adapter_name` is `&'static str` (NOT `String`) to preserve Phase 2
    /// D-25 `Copy` + `non_exhaustive`. The runtime wrapper that builds this
    /// variant is responsible for `Box::leak`-promoting the upstream
    /// `wgpu::AdapterInfo::name: String` once at construction (justified —
    /// one-time panic-on-misconfiguration message). `requested_runtime` is
    /// a `BackendTag` shadow enum (also `Copy`).
    ///
    /// Plan 06-04 wires the actual `wgpu::Features::SHADER_F64` runtime
    /// probe; this plan declares the variant so 06-04 only adds runtime
    /// probe code, not enum shape.
    #[error(
        "Wgpu adapter '{adapter_name}' lacks SHADER_F64; cannot launch {requested_runtime:?} (D-13/D-13-A)"
    )]
    WgpuNoF64 {
        adapter_name: &'static str,
        requested_runtime: BackendTag,
    },

    /// Phase 6 Plan 06-02a (W-7 revision-1) — symmetric typed error for the
    /// CUDA f64 probe. CUDA f64 is reportedly always supported on real
    /// hardware, but the cubecl-book feature matrix flags CUDA f64 as "?";
    /// this defensive typed error lets the dispatcher refuse to launch on a
    /// device that fails the runtime probe rather than silently producing
    /// f32-precision output.
    ///
    /// Plan 06-04 wires the actual probe + payload construction.
    #[error(
        "CUDA adapter '{adapter_name}' lacks f64 device support; cannot launch {requested_runtime:?} (W-7)"
    )]
    CudaNoF64 {
        adapter_name: &'static str,
        requested_runtime: BackendTag,
    },
}

impl XcError {
    /// C ABI error code per CAPI-05 + Phase 5 D-08-A. Mirrors the
    /// `XC_E*` constants in `xcfun-master/src/XCFunctional.hpp:40-46`:
    /// `XC_EORDER=1, XC_EVARS=2, XC_EMODE=4`. The combined
    /// `XC_EVARS | XC_EMODE` (= 6) is produced by `Functional::eval_setup`
    /// when `Mode::Potential` is requested for a GGA tier whose
    /// `Vars` lacks the `_2ND_TAYLOR` shape (XCFunctional.cpp:441-443).
    ///
    /// Variants without a direct upstream `XC_E*` mapping
    /// (UnknownName, NotConfigured, Runtime, InputLengthMismatch,
    /// OutputLengthMismatch, InvalidEncoding) all map to `-1`,
    /// mirroring the C++ pattern of returning `-1` from
    /// `xcfun_set` / `xcfun_get` for unknown names. LB94's
    /// `XcError::Runtime` (returned by `Functional::eval` because
    /// the upstream lb94.cpp is `#if 0`'d) maps to `-1`.
    pub fn as_c_code(&self) -> i32 {
        match self {
            Self::InvalidOrder { .. } => 1,         // XC_EORDER
            Self::InvalidVars { .. } => 2,          // XC_EVARS
            Self::InvalidMode { .. } => 4,          // XC_EMODE
            Self::InvalidVarsAndMode { .. } => 6,   // XC_EVARS | XC_EMODE
            // Phase 6 Plan 06-02a — no upstream `XC_E*` mapping for the GPU
            // device-feature errors; `-1` matches the `UnknownName` /
            // `Runtime` precedent (returned by `xcfun_set` / `xcfun_get`
            // for any non-recognised input).
            Self::WgpuNoF64 { .. } => -1,
            Self::CudaNoF64 { .. } => -1,
            _ => -1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    // CORE-04 compile-test: XcError MUST be Copy + Send + Sync.
    assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug);

    #[test]
    fn invalid_order_display() {
        let err = XcError::InvalidOrder {
            order: 5,
            mode: Mode::PartialDerivatives,
            n_vars: 2,
        };
        let msg = format!("{err}");
        assert!(msg.contains("invalid derivative order 5"), "got: {msg}");
    }

    #[test]
    fn unknown_name_display_drops_payload() {
        let err = XcError::UnknownName;
        let msg = format!("{err}");
        assert_eq!(msg, "unknown functional name");
    }

    #[test]
    fn not_configured_display() {
        let err = XcError::NotConfigured;
        let msg = format!("{err}");
        assert!(msg.contains("not configured"), "got: {msg}");
    }

    #[test]
    fn xc_error_is_copy() {
        let err = XcError::NotConfigured;
        let _copy = err; // Compiles iff XcError: Copy
        let _again = err; // Use after copy still works
    }

    // ----- Plan 05-00 Task 0.2: as_c_code mapping (D-08-A, CAPI-05) -----

    #[test]
    fn as_c_code_invalid_order() {
        assert_eq!(
            XcError::InvalidOrder { order: 5, mode: Mode::PartialDerivatives, n_vars: 2 }
                .as_c_code(),
            1,
        );
    }

    #[test]
    fn as_c_code_invalid_vars() {
        assert_eq!(
            XcError::InvalidVars { vars: Vars::A, required: Dependency::GRADIENT }
                .as_c_code(),
            2,
        );
    }

    #[test]
    fn as_c_code_invalid_mode() {
        assert_eq!(
            XcError::InvalidMode { mode: Mode::Potential, depends: Dependency::KINETIC }
                .as_c_code(),
            4,
        );
    }

    #[test]
    fn as_c_code_invalid_vars_and_mode() {
        assert_eq!(
            XcError::InvalidVarsAndMode {
                vars: Vars::A_B,
                mode: Mode::Potential,
                depends: Dependency::GRADIENT,
            }
            .as_c_code(),
            6,
        );
    }

    #[test]
    fn as_c_code_unknown_name() {
        assert_eq!(XcError::UnknownName.as_c_code(), -1);
    }

    #[test]
    fn as_c_code_not_configured() {
        assert_eq!(XcError::NotConfigured.as_c_code(), -1);
    }

    #[test]
    fn as_c_code_runtime() {
        assert_eq!(XcError::Runtime.as_c_code(), -1);
    }

    #[test]
    fn as_c_code_input_length_mismatch() {
        assert_eq!(
            XcError::InputLengthMismatch { expected: 2, got: 1 }.as_c_code(),
            -1,
        );
    }

    #[test]
    fn as_c_code_output_length_mismatch() {
        assert_eq!(
            XcError::OutputLengthMismatch { expected: 2, got: 1 }.as_c_code(),
            -1,
        );
    }

    #[test]
    fn as_c_code_invalid_encoding() {
        assert_eq!(XcError::InvalidEncoding.as_c_code(), -1);
    }

    #[test]
    fn xc_error_still_copy() {
        // Plan 05-00 Task 0.2 — InvalidVarsAndMode carries Copy fields
        // (Vars, Mode, Dependency are all Copy), so XcError remains Copy.
        fn _assert_copy<T: Copy>() {}
        _assert_copy::<XcError>();
    }
}
