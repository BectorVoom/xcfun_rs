"""mpmath port of xcfun-master/src/functionals/SCAN_like_eps.hpp at mp.prec=200.

Plan 06-N2 substrate. Shared by the 10 SCAN-family modules (`scanx`,
`scanc`, `rscanx`, `rscanc`, `rppscanx`, `rppscanc`, `r2scanx`,
`r2scanc`, `r4scanx`, `r4scanc`).

Verbatim port of `SCAN_eps` namespace at prec=200. The C++ harness
aborts on the low-density tail (17 sqrt() call-sites in the substrate);
mpmath at prec=200 is the sole reference per D-03 ACC-04 amendment.

# Settings table (xcfun-master/src/functionals/{SCAN,r,r2,r4,rpp}{x,c}.cpp)
| Variant   | IALPHA | IINTERP | IDELFX |
|-----------|--------|---------|--------|
| SCAN      | 0      | 0       | 0      |
| rSCAN     | 1      | 1       | 0      |
| rppSCAN   | 2      | 1       | 0      |
| r2SCAN    | 2      | 1       | 1      |
| r4SCAN    | 2      | 1       | 2      |

# AD differentiability caveat
The `if alpha < 1.0` branch in `SCAN_X_Fx` (and the analogous branch in
`SCAN_C`) makes the SCAN energy NON-smooth at alpha=1 (the boundary
between the "single-orbital" and "slowly-varying" branches). C++ xcfun
uses CTaylor-symbolic-AD which builds the Taylor expansion within whichever
branch the seed point lies in. mpmath.diff uses Richardson finite
differences and, near alpha~1, will sample across the branch boundary
and return inaccurate derivatives. The fixture grid stays away from
alpha=1 by construction (regularize ensures min density stratum and
non-pathological tau values), so this is a fixture-stratification
concern rather than a port bug.
"""
from __future__ import annotations
import mpmath as mp


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

PI2 = mp.pi * mp.pi


def _fx_unif(d):
    """fx_unif(d) per SCAN_like_eps.hpp:71-73."""
    return mp.mpf("-0.75") * mp.power(3 / mp.pi, mp.mpf(1) / mp.mpf(3)) \
        * mp.power(d, mp.mpf(4) / mp.mpf(3))


# ---------------------------------------------------------------------------
# SCAN exchange (single-spin) — SCAN_like_eps.hpp:75-251
# ---------------------------------------------------------------------------


def _scan_x_fx(p, alpha, ETA, IINTERP, IDELFX):
    """SCAN_X_Fx(p, alpha, ETA, IINTERP, IDELFX) per SCAN_like_eps.hpp:132-251."""
    A1 = mp.mpf("4.9479")
    K1 = mp.mpf("0.065")
    K0 = mp.mpf("0.174")
    MU = mp.mpf(10) / mp.mpf(81)

    IE_PARAMS = (
        mp.mpf("1.0"),
        mp.mpf("-0.667"),
        mp.mpf("-0.4445555"),
        mp.mpf("-0.663086601049"),
        mp.mpf("1.451297044490"),
        mp.mpf("-0.887998041597"),
        mp.mpf("0.234528941479"),
        mp.mpf("-0.023185843322"),
    )
    CFX1 = mp.mpf("0.667")
    CFX2 = mp.mpf("0.8")
    CFDX1 = mp.mpf("1.24")

    DX_DAMP4_P = mp.mpf("0.232")
    DX_DAMP4_A = mp.mpf("0.232")
    D_DAMP2 = mp.mpf("0.361")
    B1 = mp.mpf("0.156632")
    B2 = mp.mpf("0.12083")
    B3 = mp.mpf("0.5")
    B4 = MU * MU / K1 - mp.mpf("0.112654")

    ALPHA_GE = mp.mpf(20) / mp.mpf(27) + ETA * mp.mpf(5) / mp.mpf(3)

    oma = 1 - alpha
    # Interpolation
    if IINTERP == 0:
        # SCAN: smooth-junction in alpha
        if alpha < 1:
            ief = mp.exp(-CFX1 * alpha / oma)
        else:
            ief = -CFDX1 * mp.exp(CFX2 / oma)
    elif IINTERP == 1:
        # rSCAN: polynomial in [0, 2.5] sandwiched between two exp arms
        if alpha < mp.mpf("1e-13"):
            ief = mp.exp(-CFX1 * alpha / oma)
        elif alpha < mp.mpf("2.5"):
            ief = mp.mpf(0)
            for i in range(8):
                ief = ief + IE_PARAMS[i] * mp.power(alpha, i)
        else:
            ief = -CFDX1 * mp.exp(CFX2 / oma)
    else:
        raise ValueError(f"Unknown IINTERP {IINTERP}")

    h0x = 1 + K0

    if IDELFX == 0:
        # 2nd + 4th order GE corrections (SCAN, rSCAN-base path)
        wfac = B4 * p * p * mp.exp(-B4 * p / MU)
        vfac = B1 * p + B2 * oma * mp.exp(-B3 * oma * oma)
        yfac = MU * p + wfac + vfac * vfac
        h1x = 1 + K1 - K1 / (1 + yfac / K1)
        del_f2 = mp.mpf(0)
        C2 = mp.mpf(0)
    elif IDELFX in (1, 2):
        # 2nd-order GE for r2SCAN / r4SCAN
        del_f2 = mp.mpf(0)
        for i in range(1, 8):
            del_f2 = del_f2 + i * IE_PARAMS[i]
        C2 = -del_f2 * (1 - h0x)
        damp = mp.exp(-(p * p) / mp.power(D_DAMP2, 4))
        h1x = 1 + K1 - K1 / (1 + p * (MU + ALPHA_GE * C2 * damp) / K1)
    else:
        raise ValueError(f"Unknown IDELFX {IDELFX}")

    # gx scaling
    gx = 1 - mp.exp(-A1 / mp.power(p, mp.mpf(1) / mp.mpf(4)))

    # 4th-order gradient enhancement (r4SCAN only)
    del_fx = mp.mpf(0)
    if IDELFX == 2:
        eta_term = ETA * mp.mpf(3) / mp.mpf(4) + mp.mpf(2) / mp.mpf(3)
        del_f4 = mp.mpf(0)
        for i in range(1, 8):
            del_f4 = del_f4 + i * (i - 1) * IE_PARAMS[i]
        C_aa = mp.mpf(73) / mp.mpf(5000) - mp.mpf("0.5") * del_f4 * (h0x - 1)
        C_pa = (
            mp.mpf(511) / mp.mpf(13500)
            - mp.mpf(73) / mp.mpf(1500) * ETA
            - del_f2 * (ALPHA_GE * C2 + MU)
        )
        C_pp = (
            mp.mpf(146) / mp.mpf(2025) * eta_term * eta_term
            - mp.mpf(73) / mp.mpf(405) * eta_term
            + (ALPHA_GE * C2 + MU) * (ALPHA_GE * C2 + MU) / K1
        )
        order_1 = C2 * (oma - ALPHA_GE * p)
        t1 = order_1 + C_aa * oma * oma + C_pa * p * oma + C_pp * p * p
        damp_4_t1 = 2 * alpha * alpha / (1 + mp.power(alpha, 4))
        damp_4_t2 = mp.exp(
            -(oma * oma) / (DX_DAMP4_A * DX_DAMP4_A)
            - (p * p) / mp.power(DX_DAMP4_P, 4)
        )
        damp_4 = damp_4_t1 * damp_4_t2
        del_fx = t1 * damp_4

    return (h1x + ief * (h0x - h1x) + del_fx) * gx


def _get_scan_fx(d_n, d_g, d_tau, IALPHA, IINTERP, IDELFX):
    """get_SCAN_Fx(d_n, d_g, d_tau, IALPHA, IINTERP, IDELFX) per SCAN_like_eps.hpp:75-130."""
    ETA = mp.mpf("1.0e-3")
    TAU_R = mp.mpf("1.0e-4")
    A_REG = mp.mpf("1.0e-3")

    tauw = d_g / (8 * d_n)

    if IALPHA == 1:
        tauUnif = (
            mp.mpf(3) / mp.mpf(10) * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
            * mp.power(d_n, mp.mpf(5) / mp.mpf(3))
        ) + TAU_R
    else:
        tauUnif = (
            mp.mpf("0.3") * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
            * mp.power(d_n, mp.mpf(5) / mp.mpf(3))
        )

    if IALPHA == 0:
        # alpha (SCAN). The 1e-14 tolerance branch is a numerical safety
        # for d_tau == tauw exactly; mpmath at prec=200 hits the branch
        # only at the literal exact equality, so we mirror the C++ guard.
        if abs(d_tau - tauw) > mp.mpf("1.0e-14"):
            alpha = (d_tau - tauw) / tauUnif
        else:
            alpha = mp.mpf(0)
    elif IALPHA == 1:
        # alpha' (rSCAN)
        a0 = (d_tau - tauw) / tauUnif
        alpha = mp.power(a0, 3) / (a0 * a0 + A_REG)
    elif IALPHA == 2:
        # \bar{alpha} (r2SCAN, r4SCAN)
        if abs(d_tau - tauw) > mp.mpf("1.0e-14"):
            alpha = (d_tau - tauw) / (tauUnif + ETA * tauw)
        else:
            alpha = mp.mpf(0)
    else:
        raise ValueError(f"Unknown IALPHA {IALPHA}")

    if abs(d_g) > mp.mpf("1.0e-16"):
        p = d_g / (
            4 * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
            * mp.power(d_n, mp.mpf(8) / mp.mpf(3))
        )
    else:
        p = mp.mpf("1e-16") / (
            4 * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
            * mp.power(d_n, mp.mpf(8) / mp.mpf(3))
        )

    return _scan_x_fx(p, alpha, ETA, IINTERP, IDELFX)


# Public API for SCAN-X variants:
def scan_exchange_value(a, b, gaa, gbb, taua, taub, IALPHA, IINTERP, IDELFX):
    """SCAN exchange energy density per SCANx.cpp:19-28.

    Returns 0.5 * (Fx_a * fx_unif(2a) + Fx_b * fx_unif(2b)).
    """
    Fx_a = _get_scan_fx(2 * a, 4 * gaa, 2 * taua, IALPHA, IINTERP, IDELFX)
    Fx_b = _get_scan_fx(2 * b, 4 * gbb, 2 * taub, IALPHA, IINTERP, IDELFX)
    eu_a = _fx_unif(2 * a)
    eu_b = _fx_unif(2 * b)
    return mp.mpf("0.5") * (Fx_a * eu_a + Fx_b * eu_b)


# ---------------------------------------------------------------------------
# SCAN correlation — SCAN_like_eps.hpp:253-461
# ---------------------------------------------------------------------------


def _ufunc(x, a):
    return mp.power(1 + x, a) + mp.power(1 - x, a)


def _gcor2(P, rs, sqrtrs):
    """gcor2(P, rs, sqrtrs) returning (GG, GGRS) per lines 499-521."""
    A, A1, B1, B2, B3, B4 = P
    Q0 = -2 * A * (1 + A1 * rs)
    Q0RS = -2 * A * A1
    Q1 = (
        2 * A * sqrtrs
        * (B1 + sqrtrs * (B2 + sqrtrs * (B3 + B4 * sqrtrs)))
    )
    Q1RS = A * (
        2 * B2 + B1 / sqrtrs + 3 * B3 * sqrtrs + 4 * B4 * rs
    )
    Q2 = mp.log(1 + 1 / Q1)
    Q2RS = -Q1RS / ((1 + 1 / Q1) * Q1 * Q1)
    GG = Q0 * Q2
    GGRS = Q0 * Q2RS + Q2 * Q0RS
    return GG, GGRS


def _get_lsda1(rs, sqrtrs, zeta):
    """get_lsda1(...) returning (eclda1, d_eclda1_drs) per lines 463-497."""
    GAM = mp.mpf("0.51984209978974632953442121455650")
    FZZ = mp.mpf(8) / (9 * GAM)
    p_eu = (
        mp.mpf("0.03109070"), mp.mpf("0.213700"), mp.mpf("7.59570"),
        mp.mpf("3.58760"), mp.mpf("1.63820"), mp.mpf("0.492940"),
    )
    p_ep = (
        mp.mpf("0.015545350"), mp.mpf("0.205480"), mp.mpf("14.11890"),
        mp.mpf("6.19770"), mp.mpf("3.36620"), mp.mpf("0.625170"),
    )
    p_alfm = (
        mp.mpf("0.01688690"), mp.mpf("0.111250"), mp.mpf("10.3570"),
        mp.mpf("3.62310"), mp.mpf("0.880260"), mp.mpf("0.496710"),
    )
    eu, deudrs = _gcor2(p_eu, rs, sqrtrs)
    ep, depdrs = _gcor2(p_ep, rs, sqrtrs)
    alfm, dalfmdrs = _gcor2(p_alfm, rs, sqrtrs)
    z3 = zeta * zeta * zeta
    z4 = zeta * z3
    f = (_ufunc(zeta, mp.mpf(4) / mp.mpf(3)) - 2) / GAM
    eclda1 = eu * (1 - f * z4) + ep * f * z4 - alfm * f * (1 - z4) / FZZ
    d_eclda1_drs = (
        (1 - z4 * f) * deudrs
        + z4 * f * depdrs
        - (1 - z4) * f * dalfmdrs / FZZ
    )
    return eclda1, d_eclda1_drs


def _lda_0(rs, B1C, B2C, B3C):
    return -B1C / (1 + B2C * mp.sqrt(rs) + B3C * rs)


def _scan_ec0(rs, s, zeta, B1C, B2C, B3C):
    """scan_ec0(rs, s, zeta, B1C, B2C, B3C) per lines 355-376."""
    CHI_LD = mp.mpf("0.12802585262625815")
    eclda = _lda_0(rs, B1C, B2C, B3C)
    dx_z = _ufunc(zeta, mp.mpf(4) / mp.mpf(3)) / 2
    gc_z = (1 - mp.mpf("2.363") * (dx_z - 1)) * (1 - mp.power(zeta, 12))
    w0 = mp.exp(-eclda / B1C) - 1
    ginf = 1 / mp.power(1 + 4 * CHI_LD * s * s, mp.mpf(1) / mp.mpf(4))
    h0 = B1C * mp.log(1 + w0 * (1 - ginf))
    return (eclda + h0) * gc_z


def _scan_ec1(rs, s, zeta, IE_PARAMS, ETA, B1C, B2C, B3C, IDELEC):
    """scan_ec1(rs, s, zeta, ie_params, eta, b1c, b2c, b3c, idelec) per lines 387-461."""
    BETA_MB = mp.mpf("0.066725")
    AFACTOR = mp.mpf("0.1")
    BFACTOR = mp.mpf("0.1778")
    GAMMA = mp.mpf("0.031090690869655")
    AFIX_T = mp.sqrt(mp.pi / 4) * mp.power(9 * mp.pi / 4, mp.mpf(1) / mp.mpf(6))
    D_DAMP2 = mp.mpf("0.361")
    dx_z = _ufunc(zeta, mp.mpf(4) / mp.mpf(3)) / 2
    gc_z = (1 - mp.mpf("2.363") * (dx_z - 1)) * (1 - mp.power(zeta, 12))
    phi = _ufunc(zeta, mp.mpf(2) / mp.mpf(3)) / 2
    phi3 = phi ** 3
    sqrtrs = mp.sqrt(rs)
    eclda0 = _lda_0(rs, B1C, B2C, B3C)
    eclsda1, d_eclsda1_drs = _get_lsda1(rs, sqrtrs, zeta)
    t = AFIX_T * s / (sqrtrs * phi)
    w1 = mp.exp(-eclsda1 / (GAMMA * phi3)) - 1
    beta = BETA_MB * (1 + AFACTOR * rs) / (1 + BFACTOR * rs)
    y = beta / (GAMMA * w1) * t * t
    if IDELEC == 0:
        del_y = mp.mpf(0)
    elif IDELEC in (1, 2):
        p = s * s
        ds_z = _ufunc(zeta, mp.mpf(5) / mp.mpf(3)) / 2
        del_f2 = mp.mpf(0)
        for i in range(1, 8):
            del_f2 = del_f2 + i * IE_PARAMS[i]
        eclsda0 = eclda0 * gc_z
        d_eclsda0_drs = (
            gc_z
            * (B3C + B2C / (2 * sqrtrs))
            * eclda0 * eclda0 / B1C
        )
        t1 = del_f2 / (27 * GAMMA * ds_z * phi3 * w1)
        t2 = 20 * rs * (d_eclsda0_drs - d_eclsda1_drs)
        t3 = 45 * ETA * (eclsda0 - eclsda1)
        k = t1 * (t2 - t3)
        damp = mp.exp(-(p * p) / mp.power(D_DAMP2, 4))
        del_y = k * p * damp
    else:
        raise ValueError(f"Unknown IDELEC {IDELEC}")
    g_y = 1 / mp.power(1 + 4 * (y - del_y), mp.mpf(1) / mp.mpf(4))
    h1 = GAMMA * phi3 * mp.log(1 + w1 * (1 - g_y))
    return eclsda1 + h1


def scan_correlation_value(a, b, gaa, gab, gbb, taua, taub, IALPHA, IINTERP, IDELEC):
    """SCAN correlation energy density per SCAN_like_eps.hpp:253-353."""
    CFC1 = mp.mpf("0.64")
    CFC2 = mp.mpf("1.5")
    CFDC1 = mp.mpf("0.7")
    IE_PARAMS = (
        mp.mpf("1.0"),
        mp.mpf("-0.64"),
        mp.mpf("-0.4352"),
        mp.mpf("-1.535685604549"),
        mp.mpf("3.061560252175"),
        mp.mpf("-1.915710236206"),
        mp.mpf("0.516884468372"),
        mp.mpf("-0.051848879792"),
    )
    ETA = mp.mpf("1.0e-3")
    TAU_R = mp.mpf("1.0e-4")
    A_REG = mp.mpf("1.0e-3")
    B1C = mp.mpf("0.0285764")
    B2C = mp.mpf("0.0889")
    B3C = mp.mpf("0.125541")
    n = a + b
    s_spin = a - b
    zeta = s_spin / n
    gnn = gaa + 2 * gab + gbb
    tau = taua + taub
    rs = mp.power(4 * mp.pi * n / 3, mp.mpf(-1) / mp.mpf(3))
    if abs(rs) > mp.mpf("1.0e-16"):
        sqrtrs = mp.sqrt(rs)
    else:
        sqrtrs = mp.mpf(0)
    ds_z = _ufunc(zeta, mp.mpf(5) / mp.mpf(3)) / 2
    s = mp.sqrt(gnn) / (
        2 * mp.power(3 * PI2, mp.mpf(1) / mp.mpf(3))
        * mp.power(n, mp.mpf(4) / mp.mpf(3))
    )
    tueg_con = mp.mpf(3) / mp.mpf(10) * mp.power(3 * PI2, mp.mpf(2) / mp.mpf(3))
    if IALPHA == 1:
        tueg = (tueg_con * mp.power(n, mp.mpf(5) / mp.mpf(3)) + TAU_R) * ds_z
    else:
        tueg = tueg_con * mp.power(n, mp.mpf(5) / mp.mpf(3)) * ds_z
    tauw = gnn / (8 * n)
    if IALPHA == 0:
        if abs(tau - tauw) > mp.mpf("1.0e-14"):
            alpha = (tau - tauw) / tueg
        else:
            alpha = mp.mpf(0)
    elif IALPHA == 1:
        a0 = (tau - tauw) / tueg
        alpha = mp.power(a0, 3) / (a0 * a0 + A_REG)
    elif IALPHA == 2:
        if abs(tau - tauw) > mp.mpf("1.0e-14"):
            alpha = (tau - tauw) / (tueg + ETA * tauw)
        else:
            alpha = mp.mpf(0)
    else:
        raise ValueError(f"Unknown IALPHA {IALPHA}")

    oma = 1 - alpha
    if IINTERP == 0:
        if alpha < 1:
            ief = mp.exp(-CFC1 * alpha / oma)
        else:
            ief = -CFDC1 * mp.exp(CFC2 / oma)
    elif IINTERP == 1:
        if alpha < mp.mpf("1e-13"):
            ief = mp.exp(-CFC1 * alpha / oma)
        elif alpha < mp.mpf("2.5"):
            ief = mp.mpf(0)
            for i in range(8):
                ief = ief + IE_PARAMS[i] * mp.power(alpha, i)
        else:
            ief = -CFDC1 * mp.exp(CFC2 / oma)
    else:
        raise ValueError(f"Unknown IINTERP {IINTERP}")

    ec0 = _scan_ec0(rs, s, zeta, B1C, B2C, B3C)
    ec1 = _scan_ec1(rs, s, zeta, IE_PARAMS, ETA, B1C, B2C, B3C, IDELEC)
    eps_c = (ec1 + ief * (ec0 - ec1)) * n
    return eps_c
