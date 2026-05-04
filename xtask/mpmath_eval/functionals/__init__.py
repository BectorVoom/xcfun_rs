"""mpmath ports of ACC-04-amended functionals.

Plan 06-00 ships the 6 boundary functionals (LDAERF family + TPSS-correlation
family) as `NotImplementedError` stubs — so that the package layout is in
place and `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP"`
imports cleanly. Plan 06-N2 fills the bodies AND adds the 20
`excluded_by_upstream_spec` set (BR×3 + SCAN×10 + CSC + BLOCX + TW + VWK +
PBELOCC + ZVPBESOLC + ZVPBEINTC) into this same package directory.

Each per-functional module exposes `eval_<name>(inputs, vars, mode, order)`
taking mp.mpf inputs and returning a list of mp.mpf outputs.
"""
from . import ldaerfx, ldaerfc, ldaerfc_jt, tpssc, tpsslocc, revtpssc

# Plan 06-N2 Task 1a — BR family (excluded_by_upstream_spec set, part 1):
from . import brx, brc, brxc

# Plan 06-N2 Task 1c — Kinetic-GGA family (tw, vwk):
from . import tw, vwk

# Tasks 1b + 1d will append: scanx, scanc, ..., r4scanc, csc, blocx,
# pbelocc, zvpbesolc, zvpbeintc.

LOOKUP = {
    "ldaerfx": ldaerfx.eval_ldaerfx,
    "ldaerfc": ldaerfc.eval_ldaerfc,
    "ldaerfc_jt": ldaerfc_jt.eval_ldaerfc_jt,
    "tpssc": tpssc.eval_tpssc,
    "tpsslocc": tpsslocc.eval_tpsslocc,
    "revtpssc": revtpssc.eval_revtpssc,
    # Plan 06-N2 Task 1a — BR family:
    "brx": brx.eval_brx,
    "brc": brc.eval_brc,
    "brxc": brxc.eval_brxc,
    # Plan 06-N2 Task 1c — Kinetic-GGA family:
    "tw": tw.eval_tw,
    "vwk": vwk.eval_vwk,
    # Plan 06-N2 Tasks 1b + 1d will append the remaining 15 entries here.
}
