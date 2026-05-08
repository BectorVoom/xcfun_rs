//! NumPy strict zero-copy `eval_vec` (PY-03 + D-07 + D-08).
//!
//! Phase 7 Plan 07-04 stub — full body lands in Plan 07-05. This stub returns
//! `NotImplementedError` so `Functional.eval_vec` is callable from Python at
//! Plan 07-04 sign-off (the type-checker passes; the runtime call raises a
//! clear pytest.skip-able error).

use numpy::{PyArray2, PyReadonlyArray2};
use pyo3::prelude::*;
use xcfun_rs::Functional as RsFunctional;

pub fn eval_vec_impl<'py>(
    _py: Python<'py>,
    _inner: &RsFunctional,
    _densities: PyReadonlyArray2<'py, f64>,
) -> PyResult<Bound<'py, PyArray2<f64>>> {
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "Functional.eval_vec stub — full implementation lands in Plan 07-05",
    ))
}
