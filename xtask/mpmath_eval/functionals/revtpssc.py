"""mpmath port of xcfun-master/src/functionals/revtpssc.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment + D-10
tau-clamp guard. See `tpssc.py` for the rationale.

Body per `revtpssc_eps.hpp:105-110`:
    epsc_revpkzb = revtpss_pbec_eps * (1 + C*tauwtau2)
                   - (1+C) * tauwtau2 * epsc_summax
    eps          = epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau2)
    DD = 2.8

CRITICAL: the outer term uses `tauwtau2` (NOT `tauwtau3` as in tpssc /
tpsslocc) — see `revtpssc_eps.hpp:107-109`. This is intentional in the
revTPSS reference and faithfully reflects the C++ source.

Final wrapper per `revtpssc.cpp:19-21`: `n * revtpssc_eps(d_clamped)`.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._tpss_eps import (
    tau_clamp, revtpss_pbec_eps, revtpss_pbec_eps_polarized, revtpssc_C,
)


# revtpssc_eps.hpp:108 — same as tpssc, different from tpsslocc (4.5).
DD = mp.mpf("2.8")


def _value_revtpssc(*inputs, vars_str):
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
    # D-10 clamp.
    tau_clamped = tau_clamp(tau, gnn, n)
    d = {
        "a": a, "b": b, "n": n, "s": s, "zeta": zeta,
        "gnn": gnn, "gns": gns, "gss": gss,
        "gaa": gaa, "gab": gab, "gbb": gbb,
        "tau": tau_clamped,
    }
    epsc_pbe = revtpss_pbec_eps(d)
    epsc_pbe_a = revtpss_pbec_eps_polarized(a, gaa)
    epsc_pbe_b = revtpss_pbec_eps_polarized(b, gbb)
    # mpmath has no fmax — use Python's max (works on mp.mpf via __lt__).
    epsc_summax = (
        a * max(epsc_pbe, epsc_pbe_a)
        + b * max(epsc_pbe, epsc_pbe_b)
    ) / n
    tauwtau = gnn / (8 * n * tau_clamped)
    tauwtau2 = tauwtau * tauwtau
    C_val = revtpssc_C(d)
    epsc_revpkzb = (
        epsc_pbe * (1 + C_val * tauwtau2)
        - (1 + C_val) * tauwtau2 * epsc_summax
    )
    # NOTE: outer term uses tauwtau2, NOT tauwtau3 (revtpssc_eps.hpp:109).
    eps = epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau2)
    return n * eps


def eval_revtpssc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_revtpssc(*args, vars_str=_vs),
        pt,
        order,
    )
