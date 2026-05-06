"""mpmath substrate for the LDAERF family at mp.prec=200.

Plan 06-N5 substrate. Shared by `ldaerfx.py`, `ldaerfc.py`, `ldaerfc_jt.py`.
Verbatim ports of:

    * `xcfun-master/src/functionals/ldaerfx.cpp:24-47` — `esrx_ldaerfspin`
      (the 4-branch range-separated LDA exchange per-spin helper).
    * `xcfun-master/src/functionals/ldaerfc.cpp:23-104` — Qrpa, dpol, g0f,
      ecorrlr (the LDA-erf correlation primitives + the long ecorrlr body).
    * `xcfun-master/src/functionals/ldaerfc_jt.cpp:24-45` — c1, c2 (the
      Toulouse short-range-LDA correlation prefactors).
    * `xcfun-master/src/functionals/vwn.hpp:19-78` — vwn5_eps and the
      scalar vwn_a/_b/_c/_x/_y/_z/_f helpers needed by ldaerfc_jt::c2.

ACC-04 D-03 amendment: the LDAERFX bracket cancellation in
`ldaerfx.cpp:39-41` cancels in f64 but resolves at prec=200; this module
is the ground truth. The Rust kernel uses an algebraically-identical
expm1-stable rederivation (Plan 02-06 Fix 1) — that workaround is f64-only
and intentionally NOT reflected here.

ALL arithmetic uses `mp.mpf` / mpmath transcendentals; no f64 intermediates.
"""
from __future__ import annotations
import mpmath as mp


# ---------------------------------------------------------------------------
# LDAERFX — esrx_ldaerfspin (ldaerfx.cpp:24-47)
# ---------------------------------------------------------------------------

# ckf = (3 * pi^2)^(1/3) ≈ 3.093667726280136 — the C++ literal.
_CKF = mp.mpf("3.093667726280136")


def esrx_ldaerfspin(na, mu):
    """esrx_ldaerfspin(na, mu) per ldaerfx.cpp:24-47.

    4-branch port — VERBATIM (no stable rederivation):
      * a < 1e-9     : limit for small a
      * a < 100      : intermediate-a bracket formula
      * a < 1e9      : large-a expansion
      * else         : limit for large a (returns 0)

    All branches use the C++ formula directly; mpmath@200 absorbs the
    bracket cancellation that the f64 path of the Rust kernel must work
    around via expm1 rederivation.
    """
    na = mp.mpf(na)
    mu = mp.mpf(mu)
    rhoa = 2 * na  # spin-scaling
    akf = _CKF * mp.cbrt(rhoa)
    a = mu / (2 * akf)
    a2 = a * a
    a3 = a2 * a
    # rhoa * (24 * rhoa / pi)^(1/3) — the common LDA prefactor.
    lda_pref = rhoa * mp.power(24 * rhoa / mp.pi, mp.mpf(1) / mp.mpf(3))
    if a < mp.mpf("1e-9"):
        # Limit for small a (uniform-electron-gas LDA-X).
        return mp.mpf("-0.375") * lda_pref
    elif a < 100:
        # Intermediate values of a — bracket formula from ldaerfx.cpp:39-41.
        bracket = (
            mp.mpf("0.375")
            - a * (
                mp.sqrt(mp.pi) * mp.erf(mp.mpf("0.5") / a)
                + (2 * a - 4 * a3) * mp.exp(mp.mpf("-0.25") / a2)
                - 3 * a + 4 * a3
            )
        )
        return -lda_pref * bracket
    elif a < mp.mpf("1e9"):
        # Expansion for large a (ldaerfx.cpp:43-44).
        return -lda_pref / (96 * a2)
    else:
        # Limit for large a (ldaerfx.cpp:46-47).
        return mp.mpf(0)


# ---------------------------------------------------------------------------
# LDAERFC — Qrpa, dpol, g0f, ecorrlr (ldaerfc.cpp:23-104)
# ---------------------------------------------------------------------------


def Qrpa(x):
    """Qrpa(x) per ldaerfc.cpp:23-31."""
    x = mp.mpf(x)
    PI = mp.pi
    Acoul = 2 * (mp.log(2) - 1) / (PI * PI)
    a2 = mp.mpf("5.84605")
    c2 = mp.mpf("3.91744")
    d2 = mp.mpf("3.44851")
    b2 = d2 - 3 / (2 * PI * Acoul) * mp.power(4 / (9 * PI), mp.mpf(1) / mp.mpf(3))
    return Acoul * mp.log(
        (1 + x * (a2 + x * (b2 + c2 * x))) / (1 + x * (a2 + d2 * x))
    )


def dpol(rs):
    """dpol(rs) per ldaerfc.cpp:33-40."""
    rs = mp.mpf(rs)
    cf = mp.power(9 * mp.pi / 4, mp.mpf(1) / mp.mpf(3))
    p2p = mp.mpf("0.04")
    p3p = mp.mpf("0.4319")
    rs2 = rs * rs
    return (
        mp.power(2, mp.mpf(5) / mp.mpf(3)) / 5
        * cf * cf / rs2
        * (1 + (p3p - mp.mpf("0.454555")) * rs)
        / (1 + p3p * rs + p2p * rs2)
    )


def g0f(x):
    """g0f(x) per ldaerfc.cpp:46-53.

    On-top pair-distribution function (Gori-Giorgi & Perdew, PRB 64, 155102).
    The argument `x` is r_s in the LDAERFC call sites.
    """
    x = mp.mpf(x)
    C0f = mp.mpf("0.0819306")
    D0f = mp.mpf("0.752411")
    E0f = mp.mpf("-0.0127713")
    F0f = mp.mpf("0.00185898")
    return (
        (1 + x * (D0f - mp.mpf("0.7317") + x * (C0f + x * (E0f + F0f * x))))
        * mp.exp(-D0f * x) / 2
    )


def ecorrlr(d, mu, ec):
    """ecorrlr(d, mu, ec) per ldaerfc.cpp:55-104.

    Long correlation expression assembled from coe2..coe5, b06, b08, a1..a4
    and the final pow(phi, 3) * Qrpa(...) sum. `d` is a dict with at least
    `r_s` and `zeta`; `mu` is the range-separation parameter; `ec` is the
    PW92 correlation eps at this density point.
    """
    mu = mp.mpf(mu)
    ec = mp.mpf(ec)
    r_s = mp.mpf(d["r_s"])
    zeta = mp.mpf(d["zeta"])

    PI = mp.pi
    alpha_const = mp.power(4 / (9 * PI), mp.mpf(1) / mp.mpf(3))
    cf = 1 / alpha_const  # C++: const parameter cf = 1/alpha;
    # phi = ((1+zeta)^(2/3) + (1-zeta)^(2/3)) / 2
    phi = (
        mp.power(1 + zeta, mp.mpf(2) / mp.mpf(3))
        + mp.power(1 - zeta, mp.mpf(2) / mp.mpf(3))
    ) / 2

    # cc parameters from the fit (ldaerfc.cpp:60-67).
    adib = mp.mpf("0.784949")
    q1a = mp.mpf("-0.388")
    q2a = mp.mpf("0.676")
    q3a = mp.mpf("0.547")
    t1a = mp.mpf("-4.95")
    t2a = mp.mpf("1.0")
    t3a = mp.mpf("0.31")

    b0 = adib * r_s
    rs2 = r_s * r_s
    rs3 = rs2 * r_s

    d2anti = (q1a * r_s + q2a * rs2) * mp.exp(-q3a * r_s) / rs2
    d3anti = (t1a * r_s + t2a * rs2) * mp.exp(-t3a * r_s) / rs3

    z = zeta
    z2 = zeta * zeta
    coe2 = mp.mpf("-0.375") / rs3 * (1 - z2) * (g0f(r_s) - mp.mpf("0.5"))

    coe3 = -(1 - z2) * g0f(r_s) / (mp.sqrt(2 * PI) * rs3)

    coe4 = (
        mp.mpf(-9) / mp.mpf(64) / rs3
        * (
            mp.power((1 + z) / 2, 2) * dpol(
                r_s * mp.power(2 / (1 + z), mp.mpf(1) / mp.mpf(3))
            )
            + mp.power((1 - z) / 2, 2) * dpol(
                r_s * mp.power(2 / (1 - z), mp.mpf(1) / mp.mpf(3))
            )
            + (1 - z * z) * d2anti
            - mp.power(cf, 2) / 10
            * (
                mp.power(1 + z, mp.mpf(8) / mp.mpf(3))
                + mp.power(1 - z, mp.mpf(8) / mp.mpf(3))
            ) / rs2
        )
    )
    coe5 = (
        mp.mpf(-9) / mp.mpf(40) / (mp.sqrt(2 * PI) * rs3)
        * (
            mp.power((1 + z) / 2, 2) * dpol(
                r_s * mp.power(2 / (1 + z), mp.mpf(1) / mp.mpf(3))
            )
            + mp.power((1 - z) / 2, 2) * dpol(
                r_s * mp.power(2 / (1 - z), mp.mpf(1) / mp.mpf(3))
            )
            + (1 - z2) * d3anti
        )
    )

    b06 = mp.power(b0, 6)
    b08 = mp.power(b0, 8)
    a1 = 4 * b06 * coe3 + b08 * coe5
    a2 = 4 * b06 * coe2 + b08 * coe4 + 6 * mp.power(b0, 4) * ec
    a3 = b08 * coe3
    a4 = b06 * (mp.power(b0, 2) * coe2 + 4 * ec)

    numer = (
        mp.power(phi, 3) * Qrpa(mu * mp.sqrt(r_s) / phi)
        + a1 * mp.power(mu, 3)
        + a2 * mp.power(mu, 4)
        + a3 * mp.power(mu, 5)
        + a4 * mp.power(mu, 6)
        + mp.power(b0 * mu, 8) * ec
    )
    denom = mp.power(1 + mp.power(b0 * mu, 2), 4)
    return numer / denom


# ---------------------------------------------------------------------------
# LDAERFC_JT — c1, c2, vwn5_eps_mp (ldaerfc_jt.cpp:24-45 + vwn.hpp:54-78)
# ---------------------------------------------------------------------------


def c1(rs):
    """c1(rs) per ldaerfc_jt.cpp:24-32."""
    rs = mp.mpf(rs)
    u1 = mp.mpf("1.0270741452992294")
    u2 = mp.mpf("-0.230160617208092")
    v1 = mp.mpf("0.6196884832404359")
    rs2 = rs * rs
    return (u1 * rs + u2 * rs2) / (1 + v1 * rs)


def c2(d):
    """c2(d) per ldaerfc_jt.cpp:34-45.

    `d` is a dict containing `n` and `r_s`. Uses `vwn5_eps_mp(d)` (the
    full-spin VWN5 correlation eps) per the C++ call site at line 42.
    """
    a = mp.mpf("3.2581")
    f = mp.mpf("3.39530545262710070631")
    bet = mp.mpf("163.44")
    gam = mp.mpf("4.7125")
    n = mp.mpf(d["n"])
    r_s = mp.mpf(d["r_s"])
    g0 = f * (mp.power(gam + r_s, mp.mpf("1.5")) + bet) * mp.exp(-a * mp.sqrt(gam + r_s))
    n2 = n * n
    denominator = mp.mpf("0.5") * mp.pi * n2 * (g0 - mp.mpf("0.5"))
    return n * vwn5_eps_mp(d) / denominator


# VWN5 parameter rows (vwn.hpp:57-60, non-XCFUN_VWN5_REF arm).
# Inter[1] is computed at module load via mpmath; everything else is the
# literal C++ value as an mp.mpf string.
_VWN_PARA = (
    mp.mpf("-0.10498"),
    mp.mpf("0.0621814"),
    mp.mpf("3.72744"),
    mp.mpf("12.9352"),
)
_VWN_FERRO = (
    mp.mpf("-0.325"),
    mp.mpf("0.0310907"),
    mp.mpf("7.06042"),
    mp.mpf("18.0578"),
)
_VWN_INTER = (
    mp.mpf("-0.0047584"),
    -mp.power(3 * mp.pi * mp.pi, mp.mpf(-1)),
    mp.mpf("1.13107"),
    mp.mpf("13.0045"),
)


def _vwn_a(p):
    """vwn_a(p) per vwn.hpp:21-23."""
    return p[0] * p[2] / (p[0] * p[0] + p[0] * p[2] + p[3]) - 1


def _vwn_b(p):
    """vwn_b(p) per vwn.hpp:25-27."""
    return 2 * (p[0] * p[2] / (p[0] * p[0] + p[0] * p[2] + p[3]) - 1) + 2


def _vwn_c(p):
    """vwn_c(p) per vwn.hpp:29-34."""
    sq = mp.sqrt(4 * p[3] - p[2] * p[2])
    return 2 * p[2] * (
        1 / sq
        - p[0] / ((p[0] * p[0] + p[0] * p[2] + p[3]) * sq / (p[2] + 2 * p[0]))
    )


def _vwn_x(s, p):
    return s * s + p[2] * s + p[3]


def _vwn_y(s, p):
    return s - p[0]


def _vwn_z(s, p):
    return mp.sqrt(4 * p[3] - p[2] * p[2]) / (2 * s + p[2])


def _vwn_f(s, p):
    """vwn_f(s, p) per vwn.hpp:48-52."""
    return mp.mpf("0.5") * p[1] * (
        2 * mp.log(s)
        + _vwn_a(p) * mp.log(_vwn_x(s, p))
        - _vwn_b(p) * mp.log(_vwn_y(s, p))
        + _vwn_c(p) * mp.atan(_vwn_z(s, p))
    )


def _ufunc(x, a):
    """ufunc(x, a) = (1+x)^a + (1-x)^a per specmath.hpp:35-37."""
    return mp.power(1 + x, a) + mp.power(1 - x, a)


# Constant is (2^(1/3) - 1)^(-1/2) — the literal 1.92366105093154 in
# vwn.hpp:71 (recomputed here at prec=200 to avoid f64 rounding).
_VWN_G_PREF = mp.power(mp.power(2, mp.mpf(1) / mp.mpf(3)) - 1, mp.mpf("-0.5"))


def vwn5_eps_mp(d):
    """vwn5_eps(d) per vwn.hpp:54-78 (non-XCFUN_VWN5_REF arm).

    `d` is a dict with `r_s` and `zeta`. Returns the per-electron VWN5
    correlation eps at the given density point.
    """
    r_s = mp.mpf(d["r_s"])
    zeta = mp.mpf(d["zeta"])
    s = mp.sqrt(r_s)
    g = _VWN_G_PREF * (_ufunc(zeta, mp.mpf(4) / mp.mpf(3)) - 2)
    zeta4 = mp.power(zeta, 4)
    dd = g * (
        (_vwn_f(s, _VWN_FERRO) - _vwn_f(s, _VWN_PARA)) * zeta4
        + _vwn_f(s, _VWN_INTER) * (1 - zeta4)
        * (mp.mpf(9) / 4 * (mp.power(2, mp.mpf(1) / mp.mpf(3)) - 1))
    )
    return _vwn_f(s, _VWN_PARA) + dd
