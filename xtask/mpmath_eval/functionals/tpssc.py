"""mpmath port of xcfun-master/src/functionals/tpssc.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment + D-10
tau-clamp guard. The Rust kernel at
crates/xcfun-kernels/src/functionals/mgga/tpssc.rs applies
`tau_clamped = max(tau, tau_w)` (build_tau_w + ctaylor_max);
this mpmath port mirrors that clamp scalarly via `_tpss_eps.tau_clamp`,
then evaluates `n * tpssc_eps(d_with_clamped_tau)`.

C++ has NO equivalent guard — it returns the cancellation-affected
value in the unphysical `tau ≪ tau_w` regime. Per D-03, mpmath@200
with the clamp is the truth at the boundary; C++ is no longer the
truth in this regime (Plan 04-10 Path-B confirmed algorithmic
faithfulness, isolated the divergence to f64 cancellation).
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._tpss_eps import (
    tau_clamp, pbec_eps, pbec_eps_polarized, tpssc_C,
)


# tpssc_eps.hpp:59 — DIFFERENT from tpsslocc's 4.5.
DD = mp.mpf("2.8")


def _value_tpssc(*inputs, vars_str):
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
    # tpssc_eps body (tpssc_eps.hpp:33-61):
    #   epsc_revpkzb = pbec_eps * (1 + C*tauwtau2)
    #                  - (1 + C) * tauwtau2 * epsc_summax
    #   eps          = epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3)
    epsc_pbe = pbec_eps(d)
    epsc_pbe_a = pbec_eps_polarized(a, gaa)
    epsc_pbe_b = pbec_eps_polarized(b, gbb)
    # mpmath has no fmax — use Python's max (works on mp.mpf via __lt__).
    epsc_summax = (
        a * max(epsc_pbe, epsc_pbe_a)
        + b * max(epsc_pbe, epsc_pbe_b)
    ) / n
    tauwtau = gnn / (8 * n * tau_clamped)
    tauwtau2 = tauwtau * tauwtau
    tauwtau3 = tauwtau2 * tauwtau
    C_val = tpssc_C(d)
    epsc_revpkzb = (
        epsc_pbe * (1 + C_val * tauwtau2)
        - (1 + C_val) * tauwtau2 * epsc_summax
    )
    eps = epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3)
    return n * eps


def eval_tpssc(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_tpssc(*args, vars_str=_vs),
        pt,
        order,
    )
