"""mpmath port of xcfun-master/src/functionals/pbelocc.cpp at mp.prec=200.

PBE-loc correlation functional (Constantin/Fabiano/Laricchia/Della Sala,
PRB 86, 035130 (2012)). Plan 06-N2 Task 1d. C++ harness aborts on
`tmath::pow_expand` at the zero-density boundary (Phase 3 Plan 03-02).
mpmath at prec=200 is sole reference per D-03 ACC-04 amendment.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._pw92eps import pw92eps


def _phi(a, b, n):
    """phi(d) = 2^(-1/3) * n^(-2/3) * (sqrt(a^(4/3)) + sqrt(b^(4/3))).

    Per `pbelocc.cpp:20-22` — equivalent to ((1+zeta)^(2/3) + (1-zeta)^(2/3))/2.
    """
    n_m13 = mp.power(n, mp.mpf(-1) / mp.mpf(3))
    a43 = mp.power(a, mp.mpf(4) / mp.mpf(3))
    b43 = mp.power(b, mp.mpf(4) / mp.mpf(3))
    return mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3)) * n_m13 * n_m13 * (
        mp.sqrt(a43) + mp.sqrt(b43)
    )


def _value_pbelocc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gab = d.get("gab", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    n = a + b
    gnn = gaa + 2 * gab + gbb
    # constants
    PI2 = mp.pi * mp.pi
    param_gamma = (1 - mp.log(2)) / PI2  # `param_gamma = (1 - log(2)) / PI2`
    beta0 = mp.mpf("0.0375")
    aa = mp.mpf("0.08")
    u = _phi(a, b, n)
    u3 = u ** 3
    # d2 (the t^2 in PBE) — coefficient = (1/12 * 3^(5/6) / pi^(-1/6))^2
    coeff_t2 = mp.power(
        mp.mpf(1) / mp.mpf(12) * mp.power(mp.mpf(3), mp.mpf(5) / mp.mpf(6))
        / mp.power(mp.pi, mp.mpf(-1) / mp.mpf(6)),
        2,
    )
    d2 = coeff_t2 * gnn / (u * u * mp.power(n, mp.mpf(7) / mp.mpf(3)))
    # ff = 1 - exp(-r_s^2)
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    ff = 1 - mp.exp(-r_s * r_s)
    beta = beta0 + aa * d2 * ff
    bg = beta / param_gamma
    eps = pw92eps(a, b)
    A = bg / (mp.exp(-eps / (param_gamma * u3)) - 1)
    d2A = d2 * A
    H = param_gamma * u3 * mp.log(
        1 + bg * d2 * (1 + d2A) / (1 + d2A * (1 + d2A))
    )
    return n * (eps + H)


def eval_pbelocc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_pbelocc(*args, vars_str=_vs),
        pt,
        order,
    )
