"""mpmath port of xcfun-master/src/functionals/zvpbesolc.cpp at mp.prec=200.

zvPBEsol correlation functional (Constantin/Fabiano/Della Sala, JCP 137,
194105 (2012)). Plan 06-N2 Task 1d. C++ harness aborts on `tmath::pow_expand`
at the zero-density boundary; mpmath at prec=200 is sole reference per D-03
ACC-04 amendment.

The C++ source (third version, `zvpbesolc.cpp:74-86`) replaces |zeta|^omega
(omega=4.5) with a smooth polynomial fit:
    zw = (0.462757 + 1.30129*z^2 - 1.59546*z^4 + 1.19635*z^6 - 0.36519*z^8) * z^4
This avoids `abs()` in the AD path; ported verbatim.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._pw92eps import pw92eps


def _phi(a, b, n):
    n_m13 = mp.power(n, mp.mpf(-1) / mp.mpf(3))
    a43 = mp.power(a, mp.mpf(4) / mp.mpf(3))
    b43 = mp.power(b, mp.mpf(4) / mp.mpf(3))
    return mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3)) * n_m13 * n_m13 * (
        mp.sqrt(a43) + mp.sqrt(b43)
    )


def _zw_fit(z):
    """Polynomial fit to |z|^4.5 used in zvpbesolc/zvpbeintc (third version)."""
    z2 = z * z
    z4 = z2 * z2
    z6 = z4 * z2
    z8 = z4 * z4
    return (
        mp.mpf("0.462757")
        + mp.mpf("1.30129") * z2
        - mp.mpf("1.59546") * z4
        + mp.mpf("1.19635") * z6
        - mp.mpf("0.36519") * z8
    ) * z4


def _value_zvpbesolc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gab = d.get("gab", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    n = a + b
    gnn = gaa + 2 * gab + gbb
    zeta = (a - b) / n
    PI2 = mp.pi * mp.pi
    param_gamma = (1 - mp.log(2)) / PI2
    beta = mp.mpf("0.046")
    alpha = mp.mpf("1.8")
    bg = beta / param_gamma
    eps = pw92eps(a, b)
    u = _phi(a, b, n)
    u3 = u ** 3
    coeff_t2 = mp.power(
        mp.mpf(1) / mp.mpf(12) * mp.power(mp.mpf(3), mp.mpf(5) / mp.mpf(6))
        / mp.power(mp.pi, mp.mpf(-1) / mp.mpf(6)),
        2,
    )
    d2 = coeff_t2 * gnn / (u * u * mp.power(n, mp.mpf(7) / mp.mpf(3)))
    tt = mp.sqrt(d2)
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    v = tt * u * mp.power(r_s / 3, mp.mpf(-1) / mp.mpf(6))
    v3 = v ** 3
    zw = _zw_fit(zeta)
    ff = mp.exp(-alpha * v3 * zw)
    A = bg / (mp.exp(-eps / (param_gamma * u3)) - 1)
    d2A = d2 * A
    H = param_gamma * u3 * mp.log(
        1 + bg * d2 * (1 + d2A) / (1 + d2A * (1 + d2A))
    )
    return n * (eps + ff * H)


def eval_zvpbesolc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_zvpbesolc(*args, vars_str=_vs),
        pt,
        order,
    )
