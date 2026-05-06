"""mpmath port of xcfun-master/src/functionals/ldaerfc_jt.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment.

Body per `ldaerfc_jt.cpp:47-53`:
    denom = 1 + c1(r_s) * mu + c2(d) * mu^2
    return d.n * vwn5_eps(d) / denom

Range-separation mu = 0.4. The c1 / c2 / vwn5_eps_mp helpers live in
`_ldaerf_eps.py`.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._ldaerf_eps import c1, c2, vwn5_eps_mp


_MU = mp.mpf("0.4")


def _value_ldaerfc_jt(*inputs, vars_str):
    slots = slot_names(vars_str)
    d_in = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d_in.get("a", mp.mpf(0))
    b = d_in.get("b", mp.mpf(0))
    n = a + b
    zeta = (a - b) / n
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    d_dict = {"a": a, "b": b, "n": n, "zeta": zeta, "r_s": r_s}
    denom = 1 + c1(r_s) * _MU + c2(d_dict) * _MU * _MU
    return n * vwn5_eps_mp(d_dict) / denom


def eval_ldaerfc_jt(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_ldaerfc_jt(*args, vars_str=_vs),
        pt,
        order,
    )
