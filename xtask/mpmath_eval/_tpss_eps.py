"""mpmath substrate for the TPSS-C family at mp.prec=200.

Plan 06-N5 substrate. Shared by `tpssc.py`, `tpsslocc.py`, `revtpssc.py`.
Verbatim ports of:

    * `xcfun-master/src/functionals/pbec_eps.hpp:23-61` — A, H, phi,
      pbec_eps, pbec_eps_polarized.
    * `xcfun-master/src/functionals/tpssc_eps.hpp:22-31` — C with C0=0.53.
    * `xcfun-master/src/functionals/tpsslocc.cpp:19-90` — pbeloc_eps,
      pbeloc_eps_pola, C with C0=0.35, plus the energy body parts.
    * `xcfun-master/src/functionals/revtpssc_eps.hpp:24-80` — revtpssA,
      revtpssH, revtpss_beta, revtpss_pbec_eps, revtpss_pbec_eps_polarized,
      C with C0=0.59 + 0.9269*z² + 0.6225*z⁴ + 2.1540*z⁶.
    * `xcfun-master/src/functionals/constants.hpp:34-37` — param_gamma,
      param_beta_pbe_paper, param_beta_accurate, param_beta_gamma.

D-10 tau-clamp guard: `tau_clamp(tau, gnn, n) = max(tau, gnn / (8*n))`.
The Rust kernel applies `tau_clamped = ctaylor_max(d.tau, tau_w)` (Plan
04-10 Path-B) to silence the unphysical `tau < tau_w` regime where C++
suffers f64 cancellation. mpmath@200 mirrors the clamp scalarly via
the built-in `max` (mpmath supports `<` / `>` on `mp.mpf`);
multivariate_taylor extends it via mp.diff.

ALL arithmetic uses `mp.mpf` / mpmath transcendentals; no f64 intermediates.
"""
from __future__ import annotations
import mpmath as mp

from ._pw92eps import pw92eps, pw92eps_polarized


# ---------------------------------------------------------------------------
# Constants (constants.hpp:34-37)
# ---------------------------------------------------------------------------

PI2 = mp.pi * mp.pi
param_gamma = (1 - mp.log(2)) / PI2
param_beta_pbe_paper = mp.mpf("0.066725")
param_beta_accurate = mp.mpf("0.06672455060314922")
param_beta_gamma = param_beta_accurate / param_gamma

# d2 prefactor for PBE-correlation H term — matches PBEC_D2_PREFACTOR_F64
# in xcfun-kernels gga::shared::constants. Literal C++ form:
#   (1/12 * 3^(5/6) / pi^(-1/6))^2
_D2_PREFACTOR = mp.power(
    mp.mpf(1) / 12 * mp.power(mp.mpf(3), mp.mpf(5) / mp.mpf(6))
    / mp.power(mp.pi, mp.mpf(-1) / mp.mpf(6)),
    2,
)


# ---------------------------------------------------------------------------
# D-10 tau-clamp guard
# ---------------------------------------------------------------------------


def tau_clamp(tau, gnn, n):
    """tau_clamp(tau, gnn, n) = max(tau, gnn / (8*n)).

    The mpmath analog of the Rust ctaylor_max(d.tau, tau_w) D-10 guard.
    Applied scalarly to the per-point evaluation; multivariate_taylor
    extends via mp.diff at prec=200. C++ has NO equivalent guard — it
    returns the cancellation-affected value in the unphysical
    `tau ≪ tau_w` regime (per Phase 4 D-10).
    """
    tau = mp.mpf(tau)
    gnn = mp.mpf(gnn)
    n = mp.mpf(n)
    tau_w = gnn / (8 * n)
    # Built-in max works on mp.mpf via __lt__/__gt__; mpmath has no
    # dedicated fmax/max function. Behaviour matches `max(tau, tau_w)`.
    return tau if tau > tau_w else tau_w


# ---------------------------------------------------------------------------
# phi reorganised (pbec_eps.hpp:38-41 / tpsslocc.cpp:19-22)
# ---------------------------------------------------------------------------


def phi_reorganised(a, b, n):
    """phi(d) = 2^(-1/3) * n^(-2/3) * (sqrt(a^(4/3)) + sqrt(b^(4/3))).

    Equivalent to ((1+zeta)^(2/3) + (1-zeta)^(2/3))/2.
    """
    a = mp.mpf(a)
    b = mp.mpf(b)
    n = mp.mpf(n)
    n_m13 = mp.power(n, mp.mpf(-1) / mp.mpf(3))
    a43 = mp.power(a, mp.mpf(4) / mp.mpf(3))
    b43 = mp.power(b, mp.mpf(4) / mp.mpf(3))
    return mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3)) * n_m13 * n_m13 * (
        mp.sqrt(a43) + mp.sqrt(b43)
    )


# ---------------------------------------------------------------------------
# PBE-correlation A and H (pbec_eps.hpp:23-36)
# ---------------------------------------------------------------------------


def pbec_A(eps, u3):
    """A(eps, u3) per pbec_eps.hpp:23-27.

    NOTE: C++ uses expm1; at prec=200 mp.exp(-x) - 1 has no cancellation
    issue, so the simpler form is the algebraic ground truth.
    """
    eps = mp.mpf(eps)
    u3 = mp.mpf(u3)
    return param_beta_gamma / (mp.exp(-eps / (param_gamma * u3)) - 1)


def pbec_H(d2, eps, u3):
    """H(d2, eps, u3) per pbec_eps.hpp:29-36."""
    d2 = mp.mpf(d2)
    eps = mp.mpf(eps)
    u3 = mp.mpf(u3)
    d2A = d2 * pbec_A(eps, u3)
    return param_gamma * u3 * mp.log(
        1 + param_beta_gamma * d2 * (1 + d2A) / (1 + d2A * (1 + d2A))
    )


def pbec_eps(d):
    """pbec_eps(d) per pbec_eps.hpp:43-50.

    `d` is a dict with keys `a`, `b`, `n`, `gnn`. Uses pw92eps from
    `_pw92eps.py` for the unscreened LDA-c eps.
    """
    a = mp.mpf(d["a"])
    b = mp.mpf(d["b"])
    n = mp.mpf(d["n"])
    gnn = mp.mpf(d["gnn"])
    eps = pw92eps(a, b)
    u = phi_reorganised(a, b, n)
    d2 = _D2_PREFACTOR * gnn / (u * u * mp.power(n, mp.mpf(7) / mp.mpf(3)))
    return eps + pbec_H(d2, eps, mp.power(u, 3))


def pbec_eps_polarized(a, gaa):
    """pbec_eps_polarized(a, gaa) per pbec_eps.hpp:52-61.

    Fully spin-polarized branch: phi reduces to 2^(-1/3); eps comes from
    `pw92eps_polarized` (single-row PW92C arm).
    """
    a = mp.mpf(a)
    gaa = mp.mpf(gaa)
    eps = pw92eps_polarized(a)
    u = mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3))
    d2 = _D2_PREFACTOR * gaa / (u * u * mp.power(a, mp.mpf(7) / mp.mpf(3)))
    return eps + pbec_H(d2, eps, mp.power(u, 3))


# ---------------------------------------------------------------------------
# PBE-loc (TPSSLOCC) — pbeloc_eps + pbeloc_eps_pola (tpsslocc.cpp:24-61)
# ---------------------------------------------------------------------------


def _pbeloc_H(d2, eps, u3, beta):
    """Position-dependent-beta variant of pbec_H, used by pbeloc_eps."""
    d2 = mp.mpf(d2)
    eps = mp.mpf(eps)
    u3 = mp.mpf(u3)
    beta = mp.mpf(beta)
    bg = beta / param_gamma
    A = bg / (mp.exp(-eps / (param_gamma * u3)) - 1)
    d2A = d2 * A
    return param_gamma * u3 * mp.log(
        1 + bg * d2 * (1 + d2A) / (1 + d2A * (1 + d2A))
    )


def pbeloc_eps(d):
    """pbeloc_eps(d) per tpsslocc.cpp:24-41.

    PBE-loc correlation eps with position-dependent beta:
        beta = beta0 + aa * d2 * ff
        ff   = 1 - exp(-r_s²)
        beta0 = 0.0375; aa = 0.08
    """
    a = mp.mpf(d["a"])
    b = mp.mpf(d["b"])
    n = mp.mpf(d["n"])
    gnn = mp.mpf(d["gnn"])
    r_s = mp.mpf(d["r_s"])
    beta0 = mp.mpf("0.0375")
    aa = mp.mpf("0.08")
    u = phi_reorganised(a, b, n)
    u3 = mp.power(u, 3)
    d2 = _D2_PREFACTOR * gnn / (u * u * mp.power(n, mp.mpf(7) / mp.mpf(3)))
    ff = 1 - mp.exp(-r_s * r_s)
    beta = beta0 + aa * d2 * ff
    eps = pw92eps(a, b)
    return eps + _pbeloc_H(d2, eps, u3, beta)


def pbeloc_eps_pola(a, gaa):
    """pbeloc_eps_pola(a, gaa) per tpsslocc.cpp:43-61."""
    a = mp.mpf(a)
    gaa = mp.mpf(gaa)
    beta0 = mp.mpf("0.0375")
    aa = mp.mpf("0.08")
    u = mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3))
    u3 = mp.power(u, 3)
    d2 = _D2_PREFACTOR * gaa / (u * u * mp.power(a, mp.mpf(7) / mp.mpf(3)))
    rs = mp.power(3 / (4 * mp.pi), mp.mpf(1) / mp.mpf(3)) \
        * mp.power(a, mp.mpf(-1) / mp.mpf(3))
    ff = 1 - mp.exp(-rs * rs)
    beta = beta0 + aa * d2 * ff
    eps = pw92eps_polarized(a)
    return eps + _pbeloc_H(d2, eps, u3, beta)


# ---------------------------------------------------------------------------
# revTPSS — revtpss_beta + revtpss_pbec_eps (revtpssc_eps.hpp:43-68)
# ---------------------------------------------------------------------------


def revtpss_beta(dens):
    """revtpss_beta(dens) per revtpssc_eps.hpp:43-47.

    beta(rs) = beta_pbe_paper * (1 + 0.1 * r_s) / (1 + 0.1778 * r_s)
    """
    dens = mp.mpf(dens)
    r_s = mp.cbrt(3 / (4 * mp.pi * dens))
    return param_beta_pbe_paper * (1 + mp.mpf("0.1") * r_s) \
        / (1 + mp.mpf("0.1778") * r_s)


def _revtpss_H(d2, eps, u3, beta_tpss):
    """revtpssH(d2, eps, u3, beta_tpss) per revtpssc_eps.hpp:31-41."""
    d2 = mp.mpf(d2)
    eps = mp.mpf(eps)
    u3 = mp.mpf(u3)
    beta_tpss = mp.mpf(beta_tpss)
    beta_gamma = beta_tpss / param_gamma
    A = beta_gamma / (mp.exp(-eps / (param_gamma * u3)) - 1)
    d2A = d2 * A
    return param_gamma * u3 * mp.log(
        1 + beta_gamma * d2 * (1 + d2A) / (1 + d2A * (1 + d2A))
    )


def revtpss_pbec_eps(d):
    """revtpss_pbec_eps(d) per revtpssc_eps.hpp:49-57.

    Like pbec_eps but with position-dependent beta = revtpss_beta(d.n).
    """
    a = mp.mpf(d["a"])
    b = mp.mpf(d["b"])
    n = mp.mpf(d["n"])
    gnn = mp.mpf(d["gnn"])
    beta_tpss = revtpss_beta(n)
    eps = pw92eps(a, b)
    u = phi_reorganised(a, b, n)
    d2 = _D2_PREFACTOR * gnn / (u * u * mp.power(n, mp.mpf(7) / mp.mpf(3)))
    return eps + _revtpss_H(d2, eps, mp.power(u, 3), beta_tpss)


def revtpss_pbec_eps_polarized(a, gaa):
    """revtpss_pbec_eps_polarized(a, gaa) per revtpssc_eps.hpp:59-68."""
    a = mp.mpf(a)
    gaa = mp.mpf(gaa)
    eps = pw92eps_polarized(a)
    u = mp.power(mp.mpf(2), mp.mpf(-1) / mp.mpf(3))
    beta_tpss = revtpss_beta(a)
    d2 = _D2_PREFACTOR * gaa / (u * u * mp.power(a, mp.mpf(7) / mp.mpf(3)))
    return eps + _revtpss_H(d2, eps, mp.power(u, 3), beta_tpss)


# ---------------------------------------------------------------------------
# C(d) coefficients — three variants (tpssc / tpsslocc / revtpssc)
# ---------------------------------------------------------------------------


def _C_with_C0(d, C0):
    """Common body for tpssc_C / tpsslocc_C / revtpssc_C.

    All three share the xi² + ufunc(zeta, -4/3) composition; only the C0
    polynomial in zeta differs (caller passes the precomputed C0 value).
    """
    n = mp.mpf(d["n"])
    s = mp.mpf(d["s"])
    gss = mp.mpf(d["gss"])
    gns = mp.mpf(d["gns"])
    gnn = mp.mpf(d["gnn"])
    zeta = mp.mpf(d["zeta"])
    gzeta2 = (n * n * gss - 2 * n * s * gns + s * s * gnn) / mp.power(n, 4)
    xi2 = gzeta2 / (
        4 * mp.power(3 * PI2 * n, mp.mpf(2) / mp.mpf(3))
    )
    uf = mp.power(1 + zeta, mp.mpf(-4) / mp.mpf(3)) \
        + mp.power(1 - zeta, mp.mpf(-4) / mp.mpf(3))
    return C0 * mp.power(1 + mp.mpf("0.5") * xi2 * uf, -4)


def tpssc_C(d):
    """C(d) per tpssc_eps.hpp:22-31.

    C0 = 0.53 + 0.87*ζ² + 0.50*ζ⁴ + 2.26*ζ⁶  (NOTE: 0.53)
    """
    zeta = mp.mpf(d["zeta"])
    C0 = (
        mp.mpf("0.53")
        + mp.mpf("0.87") * zeta * zeta
        + mp.mpf("0.50") * mp.power(zeta, 4)
        + mp.mpf("2.26") * mp.power(zeta, 6)
    )
    return _C_with_C0(d, C0)


def tpsslocc_C(d):
    """C(d) per tpsslocc.cpp:63-69.

    C0 = 0.35 + 0.87*ζ² + 0.50*ζ⁴ + 2.26*ζ⁶  (NOTE: 0.35 vs tpssc's 0.53)
    """
    zeta = mp.mpf(d["zeta"])
    C0 = (
        mp.mpf("0.35")
        + mp.mpf("0.87") * zeta * zeta
        + mp.mpf("0.50") * mp.power(zeta, 4)
        + mp.mpf("2.26") * mp.power(zeta, 6)
    )
    return _C_with_C0(d, C0)


def revtpssc_C(d):
    """C(d) per revtpssc_eps.hpp:70-80.

    C0 = 0.59 + 0.9269*ζ² + 0.6225*ζ⁴ + 2.1540*ζ⁶
    """
    zeta = mp.mpf(d["zeta"])
    C0 = (
        mp.mpf("0.59")
        + mp.mpf("0.9269") * zeta * zeta
        + mp.mpf("0.6225") * mp.power(zeta, 4)
        + mp.mpf("2.1540") * mp.power(zeta, 6)
    )
    return _C_with_C0(d, C0)
