"""mpmath port of xcfun-master/src/functionals/tpsslocc.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment + D-10
tau-clamp guard. See `tpssc.py` for the rationale.

Body per `tpsslocc.cpp:84-97`:
    epsc_revpkzb = pbeloc_eps * (1 + C*tauwtau2) - (1+C) * tauwtau2 * epsc_summax
    energy = n * epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3)
    DD = 4.5  (different from tpssc and revtpssc, both 2.8)
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._tpss_eps import (
    tau_clamp, pbeloc_eps, pbeloc_eps_pola, tpsslocc_C,
)


# tpsslocc.cpp:95 — DIFFERENT from tpssc/revtpssc (both 2.8).
DD = mp.mpf("4.5")


def _value_tpsslocc(*inputs, vars_str):
    slots = slot_names(vars_str)
    d_in = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d_in.get("a", mp.mpf(0))
    b = d_in.get("b", mp.mpf(0))
    gaa = d_in.get("gaa", mp.mpf(0))
    gab = d_in.get("gab", mp.mpf(0))
    gbb = d_in.get("gbb", mp.mpf(0))
    taua = d_in.get("taua", mp.mpf(0))
    taub = d_in.get("taub", mp.mpf(0))
    n = a + b
    s = a - b
    zeta = s / n
    gnn = gaa + 2 * gab + gbb
    gns = gaa - gbb
    gss = gaa - 2 * gab + gbb
    tau = taua + taub
    # r_s — needed by pbeloc_eps for its position-dependent ff = 1 - exp(-r_s²).
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    # D-10 clamp.
    tau_clamped = tau_clamp(tau, gnn, n)
    d = {
        "a": a, "b": b, "n": n, "s": s, "zeta": zeta,
        "gnn": gnn, "gns": gns, "gss": gss,
        "gaa": gaa, "gab": gab, "gbb": gbb,
        "tau": tau_clamped, "r_s": r_s,
    }
    epsc_pbeloc = pbeloc_eps(d)
    epsc_pbeloc_a = pbeloc_eps_pola(a, gaa)
    epsc_pbeloc_b = pbeloc_eps_pola(b, gbb)
    # mpmath has no fmax — use Python's max (works on mp.mpf via __lt__).
    epsc_summax = (
        a * max(epsc_pbeloc, epsc_pbeloc_a)
        + b * max(epsc_pbeloc, epsc_pbeloc_b)
    ) / n
    tauwtau = gnn / (8 * n * tau_clamped)
    tauwtau2 = tauwtau * tauwtau
    tauwtau3 = tauwtau2 * tauwtau
    C_val = tpsslocc_C(d)
    epsc_revpkzb = (
        epsc_pbeloc * (1 + C_val * tauwtau2)
        - (1 + C_val) * tauwtau2 * epsc_summax
    )
    return n * epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3)


def eval_tpsslocc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_tpsslocc(*args, vars_str=_vs),
        pt,
        order,
    )
