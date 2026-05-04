"""mpmath port of xcfun-master/src/functionals/brx.cpp::brc at mp.prec=200.

Becke-Roussel correlation functional (with JP dependence). Plan 06-N2 Task
1a (W-6 revision-2 — BR family).

The BR-correlation body (`brx.cpp:108-121`) layers an opposite-spin and
two same-spin terms on top of the BR exchange polarized() helper. Constants
cab=0.63, caa=0.88 are inlined per the C++ source. mpmath at prec=200
serves as the sole ground truth (D-03 ACC-04 amendment).
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .brx import _polarized


def _value_brc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    gaa = d.get("gaa", mp.mpf(0))
    gbb = d.get("gbb", mp.mpf(0))
    lapa = d.get("lapa", mp.mpf(0))
    lapb = d.get("lapb", mp.mpf(0))
    taua = d.get("taua", mp.mpf(0))
    taub = d.get("taub", mp.mpf(0))
    jpaa = d.get("jpaa", mp.mpf(0))
    jpbb = d.get("jpbb", mp.mpf(0))
    cab = mp.mpf("0.63")
    caa = mp.mpf("0.88")
    UXa = _polarized(a, gaa, lapa, 2 * taua, jpaa)
    UXb = _polarized(b, gbb, lapb, 2 * taub, jpbb)
    zaa = abs(caa * (mp.mpf(2) / UXa))
    zbb = abs(caa * (mp.mpf(2) / UXb))
    zab = abs(cab * (mp.mpf(1) / UXa + mp.mpf(1) / UXb))
    ECopp = mp.mpf("-0.8") * a * b * zab * zab * (1 - mp.log(1 + zab) / zab)
    ECaa = (
        mp.mpf("-0.01")
        * a
        * (2 * taua - (mp.mpf("0.25") * gaa + jpaa) / a)
        * mp.power(zaa, 4)
        * (1 - mp.mpf(2) / zaa * mp.log(1 + zaa / 2))
    )
    ECbb = (
        mp.mpf("-0.01")
        * b
        * (2 * taub - (mp.mpf("0.25") * gbb + jpbb) / b)
        * mp.power(zbb, 4)
        * (1 - mp.mpf(2) / zbb * mp.log(1 + zbb / 2))
    )
    return ECopp + ECaa + ECbb


def eval_brc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_brc(*args, vars_str=_vs),
        pt,
        order,
    )
