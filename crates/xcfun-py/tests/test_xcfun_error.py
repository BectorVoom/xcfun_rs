"""PY-05 + D-09 + abi3 §5 workaround end-to-end lock.

Verifies that:
  (a) ``xcfun_rs.XcfunError`` is the Python subclass that grafts ``.code`` /
      ``.kind``,
  (b) direct Python construction works in both 1-arg and 3-arg shapes,
  (c) Rust-raised XcfunError triggers the catch path with ``.code`` /
      ``.kind`` accessible on the caught instance — the END-TO-END abi3 §5 lock.

This test MUST pass on Python 3.10 / 3.11 / 3.12 / 3.13. The CI matrix
``.github/workflows/release.yml`` (Plan 07-07) runs the wheel-installed tests
on each Python version reachable via abi3-py310.
"""
import pytest
import xcfun_rs as xc


# --- Direct Python-source construction --------------------------------------


def test_direct_construction_one_arg_defaults():
    e = xc.XcfunError("explosions")
    assert isinstance(e, Exception)
    assert isinstance(e, xc.XcfunError)
    assert str(e) == "explosions"
    assert e.code == -1  # defensive default per __init__.py shim
    assert e.kind == "Unknown"


def test_direct_construction_three_args_unpacks_to_code_and_kind():
    e = xc.XcfunError("msg", 2, "InvalidVars")
    assert isinstance(e, xc.XcfunError)
    assert e.code == 2  # Phase 5 D-08-A as_c_code: InvalidVars -> 2
    assert e.kind == "InvalidVars"


def test_subclass_relationship_to_base_native_exception():
    # The user-facing class is the Python subclass; its base is the bare
    # _native exception (re-exported from the cdylib).
    from xcfun_rs._native import XcfunError as _Base  # type: ignore[attr-defined]
    assert _Base in xc.XcfunError.__mro__


# --- End-to-end abi3 §5 lock — Rust raises, Python catches ------------------


def test_unknown_functional_name_raises_xcfun_error_with_unknown_name_kind():
    """RS-02 — ``Functional::set("garbage", 1.0)`` returns ``Err(XcError::UnknownName)``.

    After 07-04 lands, ``f.set("garbage", 1.0)`` translates to PyErr via
    ``errors::xc_to_py`` — caught here as ``xc.XcfunError`` with ``code=-1``,
    ``kind="UnknownName"``.

    Until Plan 07-04 lands ``Functional``, this test must be marked
    ``pytest.skip`` if ``xc.Functional`` does not exist yet.
    """
    if not hasattr(xc, "Functional"):
        pytest.skip(
            "Functional class lands in Plan 07-04; "
            "xc_to_py path not yet reachable from Python"
        )

    f = xc.Functional("slaterx")  # constructs OK; no eval_setup
    with pytest.raises(xc.XcfunError) as excinfo:
        f.set("__definitely_not_a_functional__", 1.0)
    e = excinfo.value
    assert e.code == -1   # UnknownName maps to -1 per as_c_code
    assert e.kind == "UnknownName"


def test_invalid_eval_setup_raises_with_correct_kind_and_code():
    """RS-05 — order > XCFUN_MAX_ORDER raises XcError::InvalidOrder.

    Same skip pattern as test_unknown_functional — pre-07-04 there is no
    Python entry point that triggers this. The test serves as a forward-
    commitment: when 07-04 lands, this test must pass without modification.
    """
    if (not hasattr(xc, "Functional")
            or not hasattr(xc, "Vars")
            or not hasattr(xc, "Mode")):
        pytest.skip(
            "Functional/Vars/Mode land in Plan 07-04; "
            "this lock is forward-committed."
        )

    f = xc.Functional("slaterx")
    # Order 99 is well above XCFUN_MAX_ORDER = 6; expect InvalidOrder (code=1).
    with pytest.raises(xc.XcfunError) as excinfo:
        f.eval_setup(xc.Vars.A_B, xc.Mode.PartialDerivatives, 99)
    e = excinfo.value
    assert e.code == 1
    assert e.kind == "InvalidOrder"


# --- Anti-pattern guard (Pitfall 1) -----------------------------------------


def test_no_pyclass_extends_pyexception_anti_pattern():
    """Lock the abi3 §5 workaround — verify the bare native class is a
    direct PyException subclass, NOT the result of pyclass extends.

    The bare class has __module__ = 'xcfun_rs._native' (set by
    create_exception!) and inherits Exception. If a future PyO3 / abi3 change
    re-introduces the C-pyclass-extends shape, this test still passes (the
    property is the same), but Pitfall 1 says we should ALSO verify via grep
    at the source level — that grep is in Task 3.1's acceptance criteria,
    not here.
    """
    from xcfun_rs._native import XcfunError as _Base  # type: ignore[attr-defined]
    assert issubclass(_Base, Exception)
    assert _Base.__module__ == "xcfun_rs._native"
