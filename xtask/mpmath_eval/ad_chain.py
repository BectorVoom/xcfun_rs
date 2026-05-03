"""Generic Taylor-series AD chain at mp.prec=200.

Plan 06-N2 will fill `taylor_coeffs` with a generic numeric-derivative
helper (mpmath.diff at fixed prec). Plan 06-00 lands the skeleton.
"""


def taylor_coeffs(f, x0, order, prec=200):
    """Return [f(x0), f'(x0), f''(x0)/2!, ..., f^(order)(x0)/order!] as a
    list of mpmath.mpf values.
    """
    raise NotImplementedError("Plan 06-N2 populates this body")
