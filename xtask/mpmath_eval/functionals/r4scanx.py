"""mpmath port of xcfun-master/src/functionals/r4SCANx.cpp at mp.prec=200.

r4SCAN exchange. Plan 06-N2 Task 1b. Settings: IALPHA=2, IINTERP=1, IDELFX=2.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._scan_like import scan_exchange_value


def _value_r4scanx(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    return scan_exchange_value(
        d.get("a", mp.mpf(0)),
        d.get("b", mp.mpf(0)),
        d.get("gaa", mp.mpf(0)),
        d.get("gbb", mp.mpf(0)),
        d.get("taua", mp.mpf(0)),
        d.get("taub", mp.mpf(0)),
        IALPHA=2, IINTERP=1, IDELFX=2,
    )


def eval_r4scanx(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_r4scanx(*args, vars_str=_vs),
        pt,
        order,
    )
