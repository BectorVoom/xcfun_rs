//! Error types for xcfun-core.
//!
//! `XcError` is `Copy + Clone + Debug + Send + Sync + #[non_exhaustive]` per CORE-04.
//! D-25: `UnknownName` is a unit variant -- the offending name is captured at the
//! call site (FFI boundary, dispatcher) and can be logged there; the error type
//! itself stays Copy-compatible.

use crate::enums::{Mode, Vars};
use crate::traits::Dependency;

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
