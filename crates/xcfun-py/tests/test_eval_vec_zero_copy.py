"""PY-03 — strict zero-copy `eval_vec` contract (D-07 + D-08).

Verifies:
  - Float64 C-contiguous input -> success path returns a fresh PyArray2<f64>.
  - Non-C-contiguous (transposed view) -> TypeError with the D-07 locked message.
  - Fortran-contiguous -> TypeError.
  - dtype != f64 -> TypeError (PyReadonlyArray2<f64> rejects at coercion).
  - Wrong axis-1 length -> TypeError.
  - Both small batch (nr_points=32, per-point fallback) and large batch (1024,
    Batch dispatch) paths exercised - Pitfall 8 - Phase 7 D-04 backend hidden.
"""
import numpy as np
import pytest
import xcfun_rs as xc


def _make_pbex(order: int = 0) -> xc.Functional:
    return xc.Functional(
        "pbex",
        vars=xc.Vars.A_B_GAA_GAB_GBB,
        mode=xc.Mode.PartialDerivatives,
        order=order,
    )


def _make_density(nr_points: int) -> np.ndarray:
    # 5 inputs per point for A_B_GAA_GAB_GBB.
    rng = np.random.default_rng(0xC0FFEE)
    rho_a = rng.uniform(0.2, 0.5, nr_points)
    rho_b = rng.uniform(0.1, 0.4, nr_points)
    gaa   = rng.uniform(0.05, 0.20, nr_points)
    gab   = rng.uniform(0.02, 0.10, nr_points)
    gbb   = rng.uniform(0.05, 0.20, nr_points)
    return np.ascontiguousarray(np.stack([rho_a, rho_b, gaa, gab, gbb], axis=1))


# --- Success path ---------------------------------------------------------


@pytest.mark.parametrize("nr_points", [32, 1024])  # Pitfall 8 - both dispatch routes
def test_eval_vec_success_returns_fresh_2d_f64_array(nr_points):
    f = _make_pbex(order=0)
    d = _make_density(nr_points)
    out = f.eval_vec(d)
    assert isinstance(out, np.ndarray)
    assert out.dtype == np.float64
    assert out.ndim == 2
    assert out.shape == (nr_points, f.output_length())
    # Fresh allocation: out should NOT alias the input buffer.
    assert out.ctypes.data != d.ctypes.data


# --- TypeError contract - non-C-contiguous --------------------------------


def test_transposed_array_raises_typeerror_with_d07_message():
    f = _make_pbex(order=0)
    d = _make_density(64)
    # densities.T is a stride-flip view - Fortran-contiguous, NOT C-contiguous.
    with pytest.raises(TypeError) as excinfo:
        _ = f.eval_vec(d.T)
    msg = str(excinfo.value)
    # D-07 message lock - exact substrings:
    assert "xcfun_rs.eval_vec" in msg
    assert "must be float64 C-contiguous" in msg
    assert "flags=non-C-contiguous" in msg


def test_fortran_contiguous_raises_typeerror():
    f = _make_pbex(order=0)
    d = np.asfortranarray(_make_density(64))
    with pytest.raises(TypeError):
        _ = f.eval_vec(d)


# --- TypeError contract - wrong dtype -------------------------------------


def test_f32_input_raises_typeerror():
    f = _make_pbex(order=0)
    d = _make_density(64).astype(np.float32)
    # PyReadonlyArray2<f64> rejects at the type coercion layer; the error is
    # a TypeError (or numpy.exceptions / pyo3 type error - accept any
    # subclass-of-TypeError).
    with pytest.raises((TypeError, ValueError)):
        _ = f.eval_vec(d)


# --- Shape validation ----------------------------------------------------


def test_wrong_axis1_length_raises_typeerror():
    f = _make_pbex(order=0)
    # PBEX wants 5 inputs per point; we pass 3.
    d = np.zeros((64, 3), dtype=np.float64)
    with pytest.raises(TypeError) as excinfo:
        _ = f.eval_vec(d)
    msg = str(excinfo.value)
    assert "axis-1 length 3" in msg
    assert "input length 5" in msg
