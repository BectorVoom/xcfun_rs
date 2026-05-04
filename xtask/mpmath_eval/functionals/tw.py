"""mpmath port of xcfun-master/src/functionals/tw.cpp at mp.prec=200.

Thomas-Weizsäcker kinetic-GGA functional. Plan 06-N2 Task 1c (kinetic-GGA
family). No upstream test_in (Phase 2 Plan 02-05); mpmath at prec=200 is
sole reference per D-03 ACC-04 amendment.

Formula (verbatim port of `tw.cpp:20-22`):
    tw(d) = (1/8) * (gaa + gbb)^2 / n
where n = a + b. Cross-checked against the Rust kernel at
`crates/xcfun-kernels/src/functionals/lda/tw.rs:39-50` (1:1 port —
ctaylor_add → ctaylor_powi_2 → ctaylor_reciprocal → ctaylor_mul →
ctaylor_scalar_mul(0.125)).
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names


def _value_tw(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    n = a + b
    return mp.mpf("0.125") * (gaa + gbb) ** 2 / n


def eval_tw(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_tw(*args, vars_str=_vs),
        pt,
        order,
    )
