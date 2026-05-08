"""PY-02 — Functional class method coverage.

Smoke + behavioral tests for every method exposed by the Python ``Functional``
class. Parity (1e-12 vs Rust facade) is the job of ``test_parity.py`` (Plan
07-05). These tests run after ``maturin develop --release`` rebuilds the
``xcfun_rs._native`` cdylib.
"""
import numpy as np
import pytest
import xcfun_rs as xc


# --- Construction --------------------------------------------------------


def test_construct_no_kwargs_leaves_unconfigured():
    # Functional("slaterx") sets the slaterx weight but does NOT call
    # eval_setup. input_length / output_length should reflect the
    # unconfigured state per crates/xcfun-rs/src/functional.rs:
    # input_length == 0 when vars == Vars::Unset (the default after `new`),
    # and output_length raises NotConfigured before eval_setup has run.
    f = xc.Functional("slaterx")
    assert isinstance(f, xc.Functional)
    assert f.is_gga() is False
    assert f.is_metagga() is False
    with pytest.raises(xc.XcfunError) as e:
        _ = f.output_length()
    assert e.value.kind == "NotConfigured"


def test_construct_eager_with_all_kwargs_runs_eval_setup():
    f = xc.Functional(
        "slaterx",
        vars=xc.Vars.A_B,
        mode=xc.Mode.PartialDerivatives,
        order=0,
    )
    # input_length is 2 for Vars.A_B (rho_a + rho_b)
    assert f.input_length() == 2
    # output_length for order 0 = 1 scalar (the energy density)
    assert f.output_length() == 1


def test_construct_invalid_name_raises_unknown_name():
    with pytest.raises(xc.XcfunError) as e:
        xc.Functional("__not_a_real_functional__")
    assert e.value.kind == "UnknownName"
    assert e.value.code == -1


def test_construct_invalid_combo_raises_xcfun_error():
    # PBEX is GGA — Vars.A_B (LDA, no gradient inputs) + Mode.PartialDerivatives
    # is invalid. Phase 5 D-08-A: InvalidVars maps to code 2,
    # InvalidMode → 4, InvalidVarsAndMode → 6. Either kind/code can fire
    # depending on which check trips first.
    with pytest.raises(xc.XcfunError) as e:
        xc.Functional(
            "pbex",
            vars=xc.Vars.A_B,
            mode=xc.Mode.PartialDerivatives,
            order=2,
        )
    assert e.value.kind in ("InvalidVars", "InvalidVarsAndMode", "InvalidMode")
    assert e.value.code in (2, 4, 6)


# --- configure escape hatch ----------------------------------------------


def test_configure_escape_hatch_sets_up_after_construction():
    f = xc.Functional("slaterx")
    f.configure(vars=xc.Vars.A_B, mode=xc.Mode.PartialDerivatives, order=0)
    assert f.input_length() == 2
    assert f.output_length() == 1


# --- set / get -----------------------------------------------------------


def test_set_returns_none_in_place_mutation():
    f = xc.Functional("slaterx")
    rv = f.set("exx", 0.25)
    assert rv is None  # D-06 — Pythonic in-place mutator
    assert f.get("exx") == pytest.approx(0.25)


def test_set_unknown_name_raises_xcfun_error():
    f = xc.Functional("slaterx")
    with pytest.raises(xc.XcfunError) as e:
        f.set("__nope__", 1.0)
    assert e.value.kind == "UnknownName"


def test_get_unknown_name_raises_xcfun_error():
    f = xc.Functional("slaterx")
    with pytest.raises(xc.XcfunError) as e:
        f.get("__nope__")
    assert e.value.kind == "UnknownName"


def test_alias_additive_composition_smoke():
    # Phase 4 ALIAS-06: set("b3lyp", 1.0) then set("slaterx", 0.5) accumulates.
    # We only smoke-test that no exception is raised; parity is enforced by
    # the validation harness, not pytest.
    f = xc.Functional("b3lyp")
    f.set("slaterx", 0.5)
    # After b3lyp + 0.5 slaterx, b3lyp's slaterx contribution is augmented.
    # `get("slaterx")` returns the cumulative weight.
    w = f.get("slaterx")
    assert w > 0.5  # at least 0.5 from the manual set; b3lyp adds more


# --- is_gga / is_metagga -------------------------------------------------


def test_is_gga_for_pbex():
    f = xc.Functional("pbex")
    assert f.is_gga() is True
    assert f.is_metagga() is False


def test_is_metagga_for_tpssx():
    f = xc.Functional("tpssx")
    assert f.is_metagga() is True


# --- Mode / Vars IntEnum runtime values ----------------------------------


def test_mode_enum_int_values():
    # eq_int — Mode.PartialDerivatives == 1 evaluates True.
    assert xc.Mode.Unset == 0
    assert xc.Mode.PartialDerivatives == 1
    assert xc.Mode.Potential == 2
    assert xc.Mode.Contracted == 3


def test_vars_enum_load_bearing_discriminants():
    # Spot-check the discriminants that show up across the codebase.
    assert xc.Vars.A == 0
    assert xc.Vars.A_B == 2
    assert xc.Vars.A_B_GAA_GAB_GBB == 6
    assert xc.Vars.A_B_2ND_TAYLOR == 28
    assert xc.Vars.N_S_2ND_TAYLOR == 30


# --- eval (per-point) ----------------------------------------------------


def test_eval_per_point_writes_into_out():
    f = xc.Functional(
        "slaterx",
        vars=xc.Vars.A_B,
        mode=xc.Mode.PartialDerivatives,
        order=0,
    )
    density = np.array([0.3, 0.2], dtype=np.float64)
    out = np.zeros(1, dtype=np.float64)
    rv = f.eval(density, out)
    assert rv is None
    # SLATERX returns negative energy density at this density (LDA exchange).
    assert out[0] < 0


# --- input_length / output_length ----------------------------------------


def test_input_length_for_a_b_gaa_gab_gbb():
    f = xc.Functional(
        "pbex",
        vars=xc.Vars.A_B_GAA_GAB_GBB,
        mode=xc.Mode.PartialDerivatives,
        order=2,
    )
    # A_B_GAA_GAB_GBB = (rho_a, rho_b, gaa, gab, gbb) = 5 inputs
    assert f.input_length() == 5
