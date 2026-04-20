//! Error types for xcfun-core.

use crate::enums::{Mode, Vars};
use crate::traits::Dependency;

/// Errors returned by xcfun library operations.
#[derive(Debug, thiserror::Error)]
pub enum XcError {
    #[error("invalid derivative order {order} for mode {mode:?} with {n_vars} input variables")]
    InvalidOrder {
        order: u32,
        mode: Mode,
        n_vars: usize,
    },

    #[error("variable type {vars:?} does not provide required dependencies {required:?}")]
    InsufficientVars {
        vars: Vars,
        required: Dependency,
    },

    #[error("mode {mode:?} is not supported for functionals with dependencies {depends:?}")]
    UnsupportedMode {
        mode: Mode,
        depends: Dependency,
    },

    #[error("functional not configured: call eval_setup() before eval()")]
    NotConfigured,

    #[error("unknown functional or parameter name: {0:?}")]
    UnknownName(String),

    #[error("input length {got} does not match expected {expected}")]
    InputLengthMismatch { expected: usize, got: usize },

    #[error("output length {got} does not match expected {expected}")]
    OutputLengthMismatch { expected: usize, got: usize },
}

impl XcError {
    /// Map error to C FFI error code.
    pub fn ffi_code(&self) -> i32 {
        match self {
            XcError::InvalidOrder { .. } => 1,
            XcError::InsufficientVars { .. } => 2,
            XcError::UnsupportedMode { .. } => 4,
            _ => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn not_configured_display() {
        let err = XcError::NotConfigured;
        let msg = format!("{err}");
        assert!(msg.contains("not configured"), "got: {msg}");
    }

    #[test]
    fn ffi_codes() {
        assert_eq!(
            XcError::InvalidOrder {
                order: 5,
                mode: Mode::PartialDerivatives,
                n_vars: 2
            }
            .ffi_code(),
            1
        );
        assert_eq!(XcError::NotConfigured.ffi_code(), 8);
    }
}
