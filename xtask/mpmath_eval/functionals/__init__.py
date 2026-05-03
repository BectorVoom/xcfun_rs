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

# Plan 06-N2 imports brx, brc, brxc, csc, blocx, scan, tw, vwk, pbelocc,
# zvpbesolc, zvpbeintc into this same package.

LOOKUP = {
    "ldaerfx": ldaerfx.eval_ldaerfx,
    "ldaerfc": ldaerfc.eval_ldaerfc,
    "ldaerfc_jt": ldaerfc_jt.eval_ldaerfc_jt,
    "tpssc": tpssc.eval_tpssc,
    "tpsslocc": tpsslocc.eval_tpsslocc,
    "revtpssc": revtpssc.eval_revtpssc,
    # Plan 06-N2 extends with the 20 excluded_by_upstream_spec entries.
}
