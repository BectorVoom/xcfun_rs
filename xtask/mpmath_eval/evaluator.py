"""Generic Taylor-series AD chain at mp.prec=200 + functional dispatch.

Imports the per-functional `LOOKUP` table from the `functionals` sub-package
(see `functionals/__init__.py`). Plan 06-N2 fills the per-functional bodies;
Plan 06-00 lands the package skeleton with `NotImplementedError` stubs.
"""
from .functionals import LOOKUP


def eval_record(functional, vars_str, mode, order, inputs, prec):
    """Dispatch to the named functional's mpmath body and wrap the output
    into a JSONL-friendly record dict.
    """
    fn = LOOKUP[functional.lower()]
    output = fn(inputs, vars=vars_str, mode=mode, order=order)
    return {
        "functional": functional,
        "vars": vars_str,
        "mode": mode,
        "order": order,
        "input": [float(x) for x in inputs],
        "output": [float(x) for x in output],
        "mpmath_prec": prec,
        "source": "mpmath",
    }
