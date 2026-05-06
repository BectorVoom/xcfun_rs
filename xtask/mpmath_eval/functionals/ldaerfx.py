"""mpmath port of xcfun-master/src/functionals/ldaerfx.cpp at mp.prec=200.

Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment: the
LDAERFX bracket cancellation in `ldaerfx.cpp:39-41` cancels at f64 but
resolves at prec=200; this module is the ground truth. Cross-checked
against Rust kernel at crates/xcfun-kernels/src/functionals/lda/ldaerfx.rs
(which uses an algebraically-identical expm1-stable rederivation per
Plan 02-06 Fix 1 — that workaround is f64-only and intentionally NOT
reflected here; mpmath@200 evaluates the original C++ formula directly).

Test in (ldaerfx.cpp:67): {1.1, 1.0}; expected energy slot
-1.553573128702155 at order=2 with vars=A_B and mu=0.4.
"""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
from .._ldaerf_eps import esrx_ldaerfspin


# XC_RANGESEP_MU default; matches Rust RANGESEP_MU_F32 = 0.4 at
# crates/xcfun-kernels/src/functionals/lda/ldaerfx.rs:74. Phase 5
# RS-01..10 deferred runtime-mu wiring to Phase 7.
_MU = mp.mpf("0.4")


def _value_ldaerfx(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    return mp.mpf("0.5") * (esrx_ldaerfspin(a, _MU) + esrx_ldaerfspin(b, _MU))


def eval_ldaerfx(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_ldaerfx(*args, vars_str=_vs),
        pt,
        order,
    )
