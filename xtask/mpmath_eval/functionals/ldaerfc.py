"""mpmath port of xcfun-master/src/functionals/ldaerfc.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment.

Body per `ldaerfc.cpp:106-110`:
    eps = pw92eps(d)
    return n * (eps - ecorrlr(d, mu, eps))

The `ecorrlr` substrate lives in `_ldaerf_eps.py`. The PW92 base eps is
imported from `_pw92eps.py` (already shipped by Plan 06-N2). Range-
separation mu = 0.4 (matches Rust RANGESEP_MU).
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._pw92eps import pw92eps
from .._ldaerf_eps import ecorrlr


_MU = mp.mpf("0.4")


def _value_ldaerfc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d_in = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d_in.get("a", mp.mpf(0))
    b = d_in.get("b", mp.mpf(0))
    n = a + b
    zeta = (a - b) / n
    # r_s = (3 / (4 * pi * n))^(1/3) — densvars formula at densvars.hpp:214.
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    d_dict = {"a": a, "b": b, "n": n, "zeta": zeta, "r_s": r_s}
    eps = pw92eps(a, b)
    return n * (eps - ecorrlr(d_dict, _MU, eps))


def eval_ldaerfc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_ldaerfc(*args, vars_str=_vs),
        pt,
        order,
    )
