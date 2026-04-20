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
}
