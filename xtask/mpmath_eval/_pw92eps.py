"""mpmath port of xcfun-master/src/functionals/pw92eps.hpp at mp.prec=200.

Plan 06-N2 substrate. Shared by the PBE-correlation family modules
(`pbelocc`, `zvpbesolc`, `zvpbeintc`) and any future correlation
functional that needs the PW92 epsilon expression.

Verbatim port of:
    pw92eps::eopt(sqrtr, t)            (lines 20-25)
    pw92eps::omega(z)                  (lines 32-39, non-XCFUN_REF_PW92C arm)
    pw92eps::pw92eps(d)                (lines 48-61, non-XCFUN_REF_PW92C arm)
The XCFUN_REF_PW92C variant is NOT compiled by default (config.hpp default)
so the algorithmic-identity path uses the non-REF arm.
"""
from __future__ import annotations
import mpmath as mp


# PW92C parameter table (pw92eps.hpp:41-44).
PW92C_PARAMS = (
    (mp.mpf("0.03109070"), mp.mpf("0.21370"), mp.mpf("7.59570"),
     mp.mpf("3.5876"), mp.mpf("1.63820"), mp.mpf("0.49294"), mp.mpf(1)),
    (mp.mpf("0.01554535"), mp.mpf("0.20548"), mp.mpf("14.1189"),
     mp.mpf("6.1977"), mp.mpf("3.36620"), mp.mpf("0.62517"), mp.mpf(1)),
    (mp.mpf("0.01688690"), mp.mpf("0.11125"), mp.mpf("10.3570"),
     mp.mpf("3.6231"), mp.mpf("0.88026"), mp.mpf("0.49671"), mp.mpf(1)),
)


def eopt(sqrtr, t):
    """eopt(sqrtr, t) per pw92eps.hpp:20-25."""
    inner = sqrtr * (t[2] + sqrtr * (t[3] + sqrtr * (t[4] + t[5] * sqrtr)))
    return -2 * t[0] * (1 + t[1] * sqrtr * sqrtr) * mp.log(
        1 + mp.mpf("0.5") / (t[0] * inner)
    )


def _ufunc(x, a):
    """ufunc(x, a) = (1+x)^a + (1-x)^a per specmath.hpp:35-37."""
    return mp.power(1 + x, a) + mp.power(1 - x, a)


def omega(zeta):
    """omega(z) per pw92eps.hpp:32-39 (non-XCFUN_REF_PW92C arm)."""
    return (_ufunc(zeta, mp.mpf(4) / mp.mpf(3)) - 2) / (
        2 * mp.power(2, mp.mpf(1) / mp.mpf(3)) - 2
    )


def pw92eps(a, b):
    """pw92eps applied to alpha + beta densities at prec=200.

    Equivalent to the C++ `pw92eps::pw92eps(densvars<num> &)` for
    XC_A_B input layout. Returns the PW92 correlation per-electron
    energy (in Hartree).
    """
    a = mp.mpf(a)
    b = mp.mpf(b)
    n = a + b
    s = a - b
    zeta = s / n
    # r_s = (3 / (4*pi*n))^(1/3)
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    # c = 8 / (9 * (2 * 2^(1/3) - 2))
    c = mp.mpf(8) / (9 * (2 * mp.power(2, mp.mpf(1) / mp.mpf(3)) - 2))
    zeta4 = mp.power(zeta, 4)
    omegaval = omega(zeta)
    sqrtr = mp.sqrt(r_s)
    e0 = eopt(sqrtr, PW92C_PARAMS[0])
    e1 = eopt(sqrtr, PW92C_PARAMS[1])
    e2 = eopt(sqrtr, PW92C_PARAMS[2])
    return e0 - e2 * omegaval * (1 - zeta4) / c + (e1 - e0) * omegaval * zeta4
