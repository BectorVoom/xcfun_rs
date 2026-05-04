"""mpmath port of xcfun-master/src/functionals/vonw.cpp at mp.prec=200.

von Weizsäcker kinetic-GGA functional (XC_VWK). Plan 06-N2 Task 1c. No
upstream test_in (Phase 2 Plan 02-05); mpmath at prec=200 is sole
reference per D-03 ACC-04 amendment.

Formula (verbatim port of `vonw.cpp:17-23`):
    vW_alpha(na, gaa) = gaa / (8 * na)
    vW(d) = vW_alpha(d.a, d.gaa) + vW_alpha(d.b, d.gbb)
Cross-checked against the Rust kernel at
`crates/xcfun-kernels/src/functionals/lda/vwk.rs`.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names


def _value_vwk(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    return gaa / (mp.mpf(8) * a) + gbb / (mp.mpf(8) * b)


def eval_vwk(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_vwk(*args, vars_str=_vs),
        pt,
        order,
    )
