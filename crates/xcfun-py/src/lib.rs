//! xcfun_rs — Python bindings for the xcfun_rs Rust facade.
//!
//! This is the PyO3 0.28.3 cdylib. The Python module name is `xcfun_rs._native`
//! (set by `[tool.maturin] module-name = "xcfun_rs._native"` in pyproject.toml).
//! The user-facing `xcfun_rs` package is the Python source dir at
//! `python/xcfun_rs/`, which re-exports the symbols defined here.
//!
//! Phase 7 Plan 07-02 — module skeleton + 11 free fns (PY-01 + PY-04).
//! Functional + Mode + Vars + XcfunError land in Plans 07-03 / 07-04 / 07-05.

#![allow(non_local_definitions)] // PyO3 macros emit non-local trait impls

use pyo3::prelude::*;

mod functional;

use functional::free_fns::{
    authors, describe_long, describe_short, enumerate_aliases, enumerate_parameters,
    is_compatible_library, self_test, splash, version, which_mode, which_vars,
};

/// PyO3 entry point. The `_native` name MUST match the
/// `[tool.maturin] module-name` value in `pyproject.toml`.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // ----- Class registrations land in Plans 07-03 / 07-04 / 07-05.
    // m.add_class::<functional::Functional>()?;          // Plan 07-04 (PY-02)
    // m.add_class::<functional::Mode>()?;                // Plan 07-04
    // m.add_class::<functional::Vars>()?;                // Plan 07-04
    // m.add("XcfunError", m.py().get_type::<errors::XcfunError>())?; // Plan 07-03 (PY-05)

    // ----- PY-04 — 11 module-level free functions.
    m.add_function(wrap_pyfunction!(version,                 m)?)?;
    m.add_function(wrap_pyfunction!(splash,                  m)?)?;
    m.add_function(wrap_pyfunction!(authors,                 m)?)?;
    m.add_function(wrap_pyfunction!(self_test,               m)?)?;
    m.add_function(wrap_pyfunction!(is_compatible_library,   m)?)?;
    m.add_function(wrap_pyfunction!(which_vars,              m)?)?;
    m.add_function(wrap_pyfunction!(which_mode,              m)?)?;
    m.add_function(wrap_pyfunction!(enumerate_parameters,    m)?)?;
    m.add_function(wrap_pyfunction!(enumerate_aliases,       m)?)?;
    m.add_function(wrap_pyfunction!(describe_short,          m)?)?;
    m.add_function(wrap_pyfunction!(describe_long,           m)?)?;

    Ok(())
}
