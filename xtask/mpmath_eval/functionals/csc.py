"""mpmath port of xcfun-master/src/functionals/cs.cpp at mp.prec=200.

Colle-Salvetti correlation functional (XC_CSC). Plan 06-N2 Task 1d.
Cross-checked against the Rust kernel at
`crates/xcfun-kernels/src/functionals/mgga/csc.rs` (which uses the
shared `mgga::shared::cs` substrate). C++ harness aborts on the
low-density tail via `tmath::sqrt_expand`; mpmath at prec=200 is the
sole reference per D-03 ACC-04 amendment.

Formula (verbatim port of `cs.cpp:17-27`, parameters a=b=c=d=1):
    gamma = 2 * (1 - (a^2 + b^2) / n^2)
    curv  = a*taua + b*taub - (1/8)*gnn - (jpaa + jpbb)
    n_m13 = n^(-1/3)
    return -1 * gamma * (n + 2 * n^(-5/3) * curv * exp(-n_m13))
                       / (1 + n_m13)
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names


def _value_csc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gab = d.get("gab", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    taua = d.get("taua", mp.mpf(0))
    taub = d.get("taub", mp.mpf(0))
    jpaa = d.get("jpaa", mp.mpf(0))
    jpbb = d.get("jpbb", mp.mpf(0))
    n = a + b
    gnn = gaa + 2 * gab + gbb
    n_m13 = mp.power(n, mp.mpf(-1) / mp.mpf(3))
    gamma = 2 * (1 - (a * a + b * b) / (n * n))
    curv = a * taua + b * taub - mp.mpf("0.125") * gnn - (jpaa + jpbb)
    return -gamma * (
        n + 2 * mp.power(n, mp.mpf(-5) / mp.mpf(3)) * curv * mp.exp(-n_m13)
    ) / (1 + n_m13)


def eval_csc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_csc(*args, vars_str=_vs),
        pt,
        order,
    )
