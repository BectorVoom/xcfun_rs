"""mpmath port of xcfun-master/src/functionals/ldaerfx.cpp at mp.prec=200.

Plan 06-N2 populates with mpmath verbatim port of the LDAERFX exchange
functional (range-separated LDA exchange via erf bracket). For Plan 06-00
substrate work, this stub is enough — fixture generation in Plan 06-00
Task 4 only validates the SUBPROCESS WIRING (Rust driver → python3 -m
xtask.mpmath_eval), not the functional bodies.
"""


def eval_ldaerfx(inputs, vars, mode, order):
    raise NotImplementedError("Plan 06-N2 populates this body")
