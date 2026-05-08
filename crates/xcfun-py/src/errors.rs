//! XcError → PyErr conversion (PY-05 + D-09 + D-10 + abi3 §5 workaround).
//!
//! The Rust side declares a bare `PyException` subclass via `create_exception!`
//! and raises with positional args `(message, code, kind)`. The Python source-side
//! `python/xcfun_rs/__init__.py` grafts a thin subclass that unpacks these args
//! into `.code: int` and `.kind: str` attributes.
//!
//! Why not `#[pyclass(extends = PyException)]`? abi3-py310 forbids subclassing
//! exception types until Python 3.12 — see 07-RESEARCH §5 CRITICAL FINDING and
//! Pitfall 1. The `create_exception!` macro produces a bare `PyException`
//! subclass that works on Python 3.10 / 3.11 / 3.12 / 3.13 under abi3-py310.
//!
//! D-10 — `WgpuNoF64` surfaces as a fixed message `"GPU adapter lacks f64
//! support"` with `adapter_name` / `requested_runtime` payload deliberately
//! dropped at the Python boundary (information-disclosure threat T-7-03-01).

use pyo3::PyErr;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use xcfun_core::XcError;

// The bare native exception class.
//
// First arg `_native` matches the cdylib module name (per pyproject.toml
// `[tool.maturin] module-name = "xcfun_rs._native"`). PyO3 uses this to set
// the exception's `__module__` so `repr(e)` shows `xcfun_rs._native.XcfunError`.
//
// The Python source-side `xcfun_rs/__init__.py` re-imports this class as
// `_XcfunErrorBase` and subclasses it to graft `.code: int` / `.kind: str`
// onto the user-facing `XcfunError`.
create_exception!(_native, XcfunError, PyException);

/// Convert a Rust [`XcError`] into a Python `XcfunError` `PyErr` instance.
///
/// Builds the exception with positional args `(message, code, kind)` so the
/// Python `__init__` shim ([`crates/xcfun-py/python/xcfun_rs/__init__.py`])
/// can unpack them onto `.code: int` and `.kind: str` attributes.
///
/// Sources:
/// - 07-RESEARCH Example B (full code template)
/// - D-09 — `.code` per Phase 5 D-08-A `as_c_code`; `.kind` = Rust variant name
/// - D-10 — `WgpuNoF64` message fixed; payload dropped at Python boundary
pub fn xc_to_py(err: XcError) -> PyErr {
    let code = err.as_c_code();

    // Enumerate every current variant explicitly — `XcError` is
    // `#[non_exhaustive]` so rustc requires a wildcard arm for cross-crate
    // matches, but enumerating the 12 known variants gives us the safety
    // gate for threat T-7-03-03: when xcfun-core adds a 13th variant, the
    // wildcard arm silently routes it to "Unknown" — a future xcfun-core
    // change MUST also extend this match (caught by the plan-level grep
    // AC requiring all 12 known variants enumerated, plus the dedicated
    // pytest in `test_xcfun_error.py` exercising the known kinds).
    let kind = match err {
        XcError::InvalidOrder { .. } => "InvalidOrder",
        XcError::InvalidVars { .. } => "InvalidVars",
        XcError::InvalidMode { .. } => "InvalidMode",
        XcError::InvalidVarsAndMode { .. } => "InvalidVarsAndMode",
        XcError::UnknownName => "UnknownName",
        XcError::InputLengthMismatch { .. } => "InputLengthMismatch",
        XcError::OutputLengthMismatch { .. } => "OutputLengthMismatch",
        XcError::NotConfigured => "NotConfigured",
        XcError::InvalidEncoding => "InvalidEncoding",
        XcError::Runtime => "Runtime",
        XcError::WgpuNoF64 { .. } => "WgpuNoF64",
        XcError::CudaNoF64 { .. } => "CudaNoF64",
        // `#[non_exhaustive]` forward-compat hatch. Reachable only if a future
        // xcfun-core release adds a new XcError variant; the safe fallback is
        // a generic "Unknown" kind. Adding the new variant to xcfun-core
        // SHOULD also be a matching addition here.
        _ => "Unknown",
    };

    // D-10 — `WgpuNoF64` message is a fixed string; `adapter_name` +
    // `requested_runtime` payload are deliberately dropped at the Python
    // boundary so host-info / GPU-fingerprint data never reaches Python.
    // Mitigates information-disclosure threat T-7-03-01.
    let msg = match err {
        XcError::WgpuNoF64 { .. } => "GPU adapter lacks f64 support".to_string(),
        other => format!("{}", other),
    };

    XcfunError::new_err((msg, code, kind))
}
