//! NumPy strict zero-copy `eval_vec` (PY-03 + D-07 + D-08).
//!
//! Contract:
//!   - Input: 2-D `np.ndarray[np.float64]`, C-contiguous, shape (nr_points, inlen).
//!   - Output: a fresh 2-D `np.ndarray[np.float64]`, shape (nr_points, outlen),
//!     owning its data. Returned by value to Python.
//!   - On layout violation: raises `TypeError` (NOT XcfunError) per D-07.

use numpy::{PyArray2, PyArrayMethods, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use xcfun_rs::Functional as RsFunctional;

/// PY-03 / D-07 / D-08 — strict-zero-copy eval_vec.
///
/// Source for `is_c_contiguous` / `dtype` / `strides` / `flags`:
/// https://docs.rs/numpy/0.28.0/numpy/trait.PyUntypedArrayMethods.html
pub fn eval_vec_impl<'py>(
    py: Python<'py>,
    inner: &RsFunctional,
    densities: PyReadonlyArray2<'py, f64>,
) -> PyResult<Bound<'py, PyArray2<f64>>> {
    // ----- D-07 — layout validation. PyReadonlyArray2<f64> already enforces
    // 2-D + f64 dtype at the type level. The remaining check is C-contiguity:
    // numpy view-style transposes (densities.T) are O(1) stride-flips that
    // give Fortran-contiguous arrays. The user must explicitly
    // np.ascontiguousarray(densities) at the call site.
    if !densities.is_c_contiguous() {
        let strides = densities.strides();
        let dtype = densities.dtype();
        return Err(PyTypeError::new_err(format!(
            "xcfun_rs.eval_vec: densities must be float64 C-contiguous; \
             got dtype={}, strides={:?}, flags=non-C-contiguous",
            dtype, strides,
        )));
    }

    let shape = densities.shape();
    let nr_points = shape[0];
    let inlen = shape[1];
    let expected_inlen = inner.input_buffer_length();
    if inlen != expected_inlen {
        return Err(PyTypeError::new_err(format!(
            "xcfun_rs.eval_vec: densities axis-1 length {} does not match \
             functional input length {}",
            inlen, expected_inlen,
        )));
    }

    let outlen = inner.output_length().map_err(crate::errors::xc_to_py)?;

    // ----- D-08 — allocate output as fresh PyArray2<f64>. Source: rust-numpy
    // `PyArray2::zeros` (Bound<'py, PyArray2<f64>>).
    let out_array = PyArray2::<f64>::zeros(py, [nr_points, outlen], false);

    // ----- Hot path: release the GIL during the Rust eval_vec call.
    {
        let dens_view = densities.as_slice()?;
        let mut out_rw = out_array.readwrite();
        let out_view = out_rw.as_slice_mut()?;

        // The Rust eval_vec is Send+Sync (Phase 5 RS-10) — detach is safe.
        // PyO3 0.28 — `py.detach` replaces `py.allow_threads`.
        // Source: pyo3 guide §parallelism.md.
        py.detach(|| {
            inner.eval_vec(
                dens_view, inlen, // density_pitch == inlen for C-contiguous
                out_view, outlen, // out_pitch == outlen for fresh allocation
                nr_points,
            )
        })
        .map_err(crate::errors::xc_to_py)?;
    }

    Ok(out_array)
}
