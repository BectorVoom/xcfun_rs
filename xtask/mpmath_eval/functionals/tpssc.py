"""mpmath port of xcfun-master/src/functionals/tpssc.cpp at mp.prec=200.

Plan 06-N2 populates the body. The TPSS-correlation family is ACC-04-amended
because tau << tau_w in the unphysical regime drives f64-rounding cancellation
to ~1e+27 magnitudes; mpmath at 200-digit precision is the truth at the
boundary. See `06-CONTEXT.md` D-10 / D-03 for the design rationale.
"""


def eval_tpssc(inputs, vars, mode, order):
    raise NotImplementedError("Plan 06-N2 populates this body")
