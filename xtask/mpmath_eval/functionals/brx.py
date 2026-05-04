"""mpmath port of xcfun-master/src/functionals/brx.cpp at mp.prec=200.

Becke-Roussel exchange functional. Plan 06-N2 Task 1a (W-6 revision-2 — BR
family).

# Algorithm
The C++ port (`brx.cpp:103-106`) reduces to:
    brx(d) = 0.5 * (a * polarized(a, gaa, lapa, 2*taua, jpaa)
                  + b * polarized(b, gbb, lapb, 2*taub, jpbb))
with `polarized(...)` involving the Newton-inverted `BR(z)` function:
    Q       = (lapa - 2*tau + (0.5*gaa + 2*jp) / na) / 6
    arg     = (1 / (2/3 * pi^(2/3))) * Q * na^(-5/3)
    x       = BR(arg)                              # solve BR_z(x) = arg
    b_norm  = cbrt(x^3 * exp(-x) / (8*pi*na))
    return -(1 - (1 + 0.5*x) * exp(-x)) / b_norm

# Why mpmath, not C++
The C++ harness aborts on the low-density tail via `tmath::sqrt_expand`
(plus the BR-Newton convergence is fragile near boundaries). mpmath at
prec=200 with `mp.findroot` solves the inversion to ~200-digit precision
deterministically — D-03 ACC-04 amendment, the sole ground truth.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names


# ---------------------------------------------------------------------------
# BR_z and Newton inversion (mpmath equivalents of brx.cpp:21-48)
# ---------------------------------------------------------------------------


def br_z(x):
    """BR_z(x) = (x - 2) / x * exp(2*x/3)."""
    return (x - mp.mpf(2)) / x * mp.exp(mp.mpf(2) / mp.mpf(3) * x)


def br_solve(z):
    """Return x satisfying BR_z(x) = z, via mpmath Newton iteration.

    Mirrors the seed-and-Newton scheme of `brx.cpp:30-48` but uses
    mp.findroot for the actual root-finding (which is convergent to
    full mp.prec, unlike the hand-coded loop with 1e-15 stopping).
    """
    z = mp.mpf(z)
    # Initial guess scheme from C++ (brx.cpp:32-39).
    if z < mp.mpf("-1e4"):
        x0 = mp.mpf(-2) / z
    elif z < mp.mpf(-2):
        x0 = (mp.sqrt(9 * z * z + 6 * z + 49) + 3 * z + 1) / 4
    elif z < mp.mpf(1):
        x0 = mp.mpf(2) * (z * mp.exp(mp.mpf(-4) / mp.mpf(3)) + 1)
    else:
        log_z = mp.log(z)
        x0 = mp.mpf("1.5") * log_z + mp.mpf("3.75") / (mp.mpf("1.5") + log_z)
    # Solve to full precision; allow a few Newton steps.
    return mp.findroot(lambda x: br_z(x) - z, x0)


def _polarized(na, gaa, lapa, taua_eff, jpaa):
    """polarized() helper from `brx.cpp:89-101`.

    Note: caller passes `2*taua` as `taua_eff` (Becke tau, no factor 1/2 —
    matches `brx.cpp:104` which feeds `2*d.taua`).
    """
    pi = mp.pi
    Q = (lapa - 2 * taua_eff + (mp.mpf("0.5") * gaa + 2 * jpaa) / na) / 6
    cf = mp.mpf(1) / (mp.mpf(2) / mp.mpf(3) * mp.power(pi, mp.mpf(2) / mp.mpf(3)))
    arg = cf * Q * mp.power(na, mp.mpf(-5) / mp.mpf(3))
    x = br_solve(arg)
    b_norm = mp.cbrt(mp.power(x, 3) * mp.exp(-x) / (8 * pi * na))
    # -(1 - (1 + 0.5*x)*exp(-x)) / b_norm
    return -(1 - (1 + mp.mpf("0.5") * x) * mp.exp(-x)) / b_norm


def _value_brx(*inputs, vars_str):
    """Scalar BR-X value from the unpacked input slots.

    Args:
        inputs: positional mp.mpf scalars matching slot_names(vars_str).
        vars_str: canonical Vars enum string.
    """
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
    pa = _polarized(a, gaa, lapa, 2 * taua, jpaa)
    pb = _polarized(b, gbb, lapb, 2 * taub, jpbb)
    return mp.mpf("0.5") * (a * pa + b * pb)


def eval_brx(inputs, vars, mode, order):
    """Evaluate BR-X at prec=200 and return the rev-gradlex Taylor vector.

    The `mode` argument is accepted for API compatibility with
    `xtask.mpmath_eval.evaluator.eval_record`; only the
    `partial_derivatives` mode is implemented at this tier (energy slot
    + multinomial Taylor coefficients per `xcfun_eval`'s output layout).
    """
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_brx(*args, vars_str=_vs),
        pt,
        order,
    )
