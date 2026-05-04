"""mpmath port of xcfun-master/src/functionals/blocx.cpp at mp.prec=200.

BLOC exchange functional (Constantin/Fabiano/Della Sala, JCTC 9, 2256
(2013)). Plan 06-N2 Task 1d.

The C++ port (`blocx.cpp`) uses an `energy_blocx(d_n, d_gnn, d_tau)` helper
applied to (2*a, 4*gaa, 2*taua) and (2*b, 4*gbb, 2*taub) and averaged.
The C++ harness aborts via `tmath::sqrt_expand` on the low-density tail
(BLOCX has internal `sqrt(1 + b*alpha*(alpha-1))` and others); mpmath at
prec=200 is sole reference per D-03 ACC-04 amendment.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names


# blocx.cpp:21-25 — local parameters.
_KAPPA = mp.mpf("0.804")
_MU = mp.mpf("0.21951")
_BB = mp.mpf("0.40")
_EE = mp.mpf("1.537")
_CC = mp.mpf("1.59096")


def _energy_blocx(d_n, d_gnn, d_tau):
    """Per-spin BLOC exchange. Verbatim port of `blocx.cpp:18-50`."""
    PI2 = mp.pi * mp.pi
    p0 = mp.mpf(1) / (
        4 * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
        * mp.power(d_n, mp.mpf(8) / mp.mpf(3))
    )
    p = d_gnn * p0  # s^2
    tauw = d_gnn / (mp.mpf(8) * d_n)
    z = tauw / d_tau
    z2 = z * z
    tau_unif = mp.mpf("0.3") * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3)) \
        * mp.power(d_n, mp.mpf(5) / mp.mpf(3))
    alpha = (d_tau - tauw) / tau_unif
    q_b = (
        mp.mpf(9) / mp.mpf(20) * (alpha - 1) / mp.sqrt(1 + _BB * alpha * (alpha - 1))
        + 2 * p / 3
    )
    ff = 4 - mp.mpf("3.3") * z
    zf = mp.exp(mp.log(z) * ff)  # z^ff
    tmp1 = p * (mp.mpf(10) / mp.mpf(81) + _CC * zf / mp.power(1 + z2, 2))
    tmp2 = mp.mpf(146) * q_b * q_b / mp.mpf(2025)
    tmp3 = (
        -mp.mpf(73) / mp.mpf(405)
        * q_b
        * d_gnn
        * mp.sqrt(
            mp.mpf("0.5") * mp.mpf("0.6") * mp.mpf("0.6")
            * mp.power(8 * d_n * d_tau, -2)
            + mp.mpf("0.5") * p0 * p0
        )
    )
    tmp4 = mp.power(p * mp.mpf(10) / mp.mpf(81), 2) / _KAPPA
    tmp5 = (
        2 * mp.sqrt(_EE) * mp.mpf("0.6") * mp.mpf("0.6") * z2
        * mp.mpf(10) / mp.mpf(81)
        + _EE * _MU * mp.power(p, 3)
    )
    tmp6 = tmp1 + tmp2 + tmp3 + tmp4 + tmp5
    x = tmp6 / mp.power(1 + mp.sqrt(_EE) * p, 2)
    Fx = 1 + _KAPPA - _KAPPA / (1 + x / _KAPPA)
    lda = (
        mp.mpf("-0.75") * mp.power(3 / mp.pi, mp.mpf(1) / mp.mpf(3))
        * mp.power(d_n, mp.mpf(4) / mp.mpf(3))
    )
    return lda * Fx


def _value_blocx(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    taua = d.get("taua", mp.mpf(0))
    taub = d.get("taub", mp.mpf(0))
    enea = _energy_blocx(2 * a, 4 * gaa, 2 * taua)
    eneb = _energy_blocx(2 * b, 4 * gbb, 2 * taub)
    return (enea + eneb) / 2


def eval_blocx(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_blocx(*args, vars_str=_vs),
        pt,
        order,
    )
