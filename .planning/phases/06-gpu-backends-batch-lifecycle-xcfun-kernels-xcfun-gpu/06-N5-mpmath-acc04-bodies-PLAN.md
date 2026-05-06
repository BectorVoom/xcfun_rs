---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N5
type: execute
wave: 1
gap_closure: true
depends_on: []
files_modified:
  - xtask/mpmath_eval/functionals/ldaerfx.py
  - xtask/mpmath_eval/functionals/ldaerfc.py
  - xtask/mpmath_eval/functionals/ldaerfc_jt.py
  - xtask/mpmath_eval/functionals/tpssc.py
  - xtask/mpmath_eval/functionals/tpsslocc.py
  - xtask/mpmath_eval/functionals/revtpssc.py
  - xtask/mpmath_eval/_ldaerf_eps.py
  - xtask/mpmath_eval/_tpss_eps.py
  - xtask/src/bin/regen_mpmath_fixtures.rs
autonomous: true
requirements: [ACC-04]
tags: [mpmath, ground-truth, ACC-04, gap-closure, ldaerf, tpssc, prec-200]
must_haves:
  truths:
    - "Each of the 6 mpmath stub files (ldaerfx, ldaerfc, ldaerfc_jt, tpssc, tpsslocc, revtpssc) imports cleanly and `eval_<name>(inputs, vars, mode, 0)` returns a list of mp.mpf — no NotImplementedError."
    - "The 3 LDAERF-family ports compute `0.5 * (per-spin(a, mu) + per-spin(b, mu))` with the 4-branch `a < 1e-9 / a < 100 / a < 1e9 / else` dispatch from `ldaerfx.cpp:34-47` (and the equivalent ecorrlr / vwn5-based forms for ldaerfc / ldaerfc_jt) at mp.prec=200."
    - "The 3 TPSS-C-family ports apply the `tau_clamped = max(tau, tau_w)` guard (D-10) before substituting into `tauwtau2` / `tauwtau3` so mpmath truth at the boundary matches the Rust kernel's intentional divergence from C++ in the `tau ≪ tau_w` regime."
    - "`cargo run --release -p xtask --bin regen-mpmath-fixtures -- --smoke` exits 0 for the smoke set AND a one-shot single-record invocation works for each of the 6 newly-filled functionals (no NotImplementedError, finite numerical output)."
    - "Python interpreter dispatch is no longer hard-coded to `python3.12` — `regen_mpmath_fixtures.rs` reads `XCFUN_MPMATH_PYTHON` env var with default `python3` (D-09 of the gap-closure plan; chosen Option C from the SUMMARY pause-note)."
    - "Driver invocation works end-to-end on the operator's primary `python3` (3.14) provided `mpmath` is installed there, AND falls back via env var to `python3.12` for legacy."
  artifacts:
    - path: "xtask/mpmath_eval/functionals/ldaerfx.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/ldaerfx.cpp` (esrx_ldaerfspin + lda_erfx) at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/functionals/ldaerfc.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/ldaerfc.cpp` (Qrpa + dpol + g0f + ecorrlr + ldaerfc) at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/functionals/ldaerfc_jt.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/ldaerfc_jt.cpp` (c1 + c2 + ldaerfc_jt + vwn5_eps reference) at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/functionals/tpssc.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/{tpssc.cpp, tpssc_eps.hpp, pbec_eps.hpp}` with tau_clamped guard at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/functionals/tpsslocc.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/tpsslocc.cpp` (pbeloc_eps + pbeloc_eps_pola + C + epsc_summax + epsc_revpkzb + energy with dd=4.5) with tau_clamped guard at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/functionals/revtpssc.py"
      provides: "mpmath verbatim port of `xcfun-master/src/functionals/{revtpssc.cpp, revtpssc_eps.hpp}` (revtpss_beta + revtpssA + revtpssH + revtpss_pbec_eps + epsc_summax + epsc_revpkzb + revtpssc_eps) with tau_clamped guard at prec=200"
      forbids: "NotImplementedError"
    - path: "xtask/mpmath_eval/_ldaerf_eps.py"
      provides: "Private substrate for the LDAERF family — `esrx_ldaerfspin(na, mu)` 4-branch helper, `Qrpa(x)`, `dpol(rs)`, `g0f(rs)`, `ecorrlr(d, mu, eps)`, `c1(rs)`, `c2(d)`, plus the small `vwn5_eps` mpmath equivalent shared by ldaerfc_jt"
    - path: "xtask/mpmath_eval/_tpss_eps.py"
      provides: "Private substrate for the TPSS-C family — `phi_reorganised(a, b, n)`, `pbec_eps(a, b, gnn, n)` (PBE correlation eps), `pbec_eps_polarized(a, gaa)`, `tpssc_C(d_dict)` with C0=0.53 coefficient, `tpsslocc_C(d_dict)` with C0=0.35 coefficient, `revtpssc_C(d_dict)` with C0=0.59 coefficient, `epsc_summax_with_eps_fn(d, eps_fn, eps_pola_fn)` parametric helper, `tau_clamp(tau, gnn, n)` returning `mp.fmax(tau, gnn/(8*n))` for the D-10 guard"
    - path: "xtask/src/bin/regen_mpmath_fixtures.rs"
      provides: "Modified driver: `python3.12` literal replaced with env-var-overridable `python3` default; `XCFUN_MPMATH_PYTHON` documented in module docstring"
      forbids: "literal `python3.12`"
  key_links:
    - from: "xtask/mpmath_eval/functionals/ldaerfx.py"
      to: "xtask/mpmath_eval/_ldaerf_eps.py::esrx_ldaerfspin"
      via: "from .._ldaerf_eps import esrx_ldaerfspin"
      pattern: "from \\.\\._ldaerf_eps import"
    - from: "xtask/mpmath_eval/functionals/ldaerfc.py"
      to: "xtask/mpmath_eval/_ldaerf_eps.py::ecorrlr"
      via: "from .._ldaerf_eps import ecorrlr"
      pattern: "from \\.\\._ldaerf_eps import.*ecorrlr"
    - from: "xtask/mpmath_eval/functionals/ldaerfc.py"
      to: "xtask/mpmath_eval/_pw92eps.py::pw92eps"
      via: "from .._pw92eps import pw92eps (eps base)"
      pattern: "from \\.\\._pw92eps import"
    - from: "xtask/mpmath_eval/functionals/ldaerfc_jt.py"
      to: "xtask/mpmath_eval/_ldaerf_eps.py::vwn5_eps"
      via: "from .._ldaerf_eps import vwn5_eps_mp"
      pattern: "from \\.\\._ldaerf_eps import.*vwn5"
    - from: "xtask/mpmath_eval/functionals/tpssc.py"
      to: "xtask/mpmath_eval/_tpss_eps.py::tau_clamp + pbec_eps + tpssc_C"
      via: "from .._tpss_eps import tau_clamp, pbec_eps, pbec_eps_polarized, tpssc_C"
      pattern: "from \\.\\._tpss_eps import"
    - from: "xtask/mpmath_eval/functionals/tpsslocc.py"
      to: "xtask/mpmath_eval/_tpss_eps.py::tau_clamp + pbeloc_eps + tpsslocc_C"
      via: "from .._tpss_eps import tau_clamp, pbeloc_eps, pbeloc_eps_polarized, tpsslocc_C"
      pattern: "from \\.\\._tpss_eps import"
    - from: "xtask/mpmath_eval/functionals/revtpssc.py"
      to: "xtask/mpmath_eval/_tpss_eps.py::tau_clamp + revtpss_pbec_eps + revtpssc_C"
      via: "from .._tpss_eps import tau_clamp, revtpss_pbec_eps, revtpss_pbec_eps_polarized, revtpssc_C"
      pattern: "from \\.\\._tpss_eps import"
    - from: "xtask/src/bin/regen_mpmath_fixtures.rs"
      to: "process::Command via XCFUN_MPMATH_PYTHON env"
      via: "std::env::var(\"XCFUN_MPMATH_PYTHON\").unwrap_or_else(|_| \"python3\".into())"
      pattern: "XCFUN_MPMATH_PYTHON"
---

<objective>
Fill the 6 ACC-04-amended mpmath sidecar functional bodies that Plan 06-N1
was scheduled to deliver but never did, and resolve the python-interpreter
dispatch issue so Plan 07-00 Task 0.2 (`cargo run --release -p xtask --bin
regen-mpmath-fixtures`) runs end-to-end clean.

The 6 stubs are exactly the ACC-04-amended set surfaced by Phase 4 D-19
forwards (LDAERF×3 erf-bracket-cancellation forwards + TPSS-C×3
AD-chain-divergence at `tau ≪ tau_w` forwards). Plan 06-00 substrate (Rust
side) shipped `erf_precise_taylor` and the `tau ≥ tau_w` clamp; Plan 06-N2
shipped 20 of 26 mpmath ports plus the `_pw92eps.py` and `_scan_like.py`
private substrates. This plan adds the remaining 6 mpmath ports plus two
new substrate modules (`_ldaerf_eps.py`, `_tpss_eps.py`) and one driver
fix.

Purpose: Unblock Phase 7 Wave 0 Task 0.2 (offline ~6h mpmath regen), close
ACC-04 amendment for the LDAERF + TPSS-C set, and document the canonical
python interpreter contract so the regen does not yak-shave on
host-Python version mismatches.

Output: 8 Python files (6 functionals filled + 2 new substrate modules)
+ 1 Rust file edit. The two substrate modules are the new pattern this
plan establishes — same shape as `_pw92eps.py` and `_scan_like.py` but
covering the LDAERF-family scalar-only `esrx_ldaerfspin` / `ecorrlr` /
`c1`/`c2` helpers and the TPSS-family `pbec_eps` / `pbeloc_eps` /
`revtpss_pbec_eps` / `C(d)` / `tau_clamp` helpers respectively.
</objective>

<execution_context>
@/home/user/Documents/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/user/Documents/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VERIFICATION.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-HUMAN-UAT.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N2-SUMMARY.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/deferred-items.md
@.planning/phases/07-python-bindings-release/07-00-SUMMARY.md
@CLAUDE.md

# C++ source-of-truth
@xcfun-master/src/functionals/ldaerfx.cpp
@xcfun-master/src/functionals/ldaerfc.cpp
@xcfun-master/src/functionals/ldaerfc_jt.cpp
@xcfun-master/src/functionals/tpssc.cpp
@xcfun-master/src/functionals/tpsslocc.cpp
@xcfun-master/src/functionals/revtpssc.cpp
@xcfun-master/src/functionals/tpssc_eps.hpp
@xcfun-master/src/functionals/revtpssc_eps.hpp
@xcfun-master/src/functionals/pbec_eps.hpp
@xcfun-master/src/functionals/vwn.hpp
@xcfun-master/src/functionals/constants.hpp
@xcfun-master/src/specmath.hpp

# Rust kernel — algorithmic identity targets
@crates/xcfun-kernels/src/functionals/lda/ldaerfx.rs
@crates/xcfun-kernels/src/functionals/lda/ldaerfc.rs
@crates/xcfun-kernels/src/functionals/lda/ldaerfc_jt.rs
@crates/xcfun-kernels/src/functionals/mgga/tpssc.rs
@crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs
@crates/xcfun-kernels/src/functionals/mgga/revtpssc.rs

# Existing mpmath substrate (already shipped — patterns to follow)
@xtask/mpmath_eval/__init__.py
@xtask/mpmath_eval/__main__.py
@xtask/mpmath_eval/evaluator.py
@xtask/mpmath_eval/ad_chain.py
@xtask/mpmath_eval/densvars.py
@xtask/mpmath_eval/_pw92eps.py
@xtask/mpmath_eval/_scan_like.py
@xtask/mpmath_eval/functionals/__init__.py
@xtask/mpmath_eval/functionals/blocx.py
@xtask/mpmath_eval/functionals/tw.py
@xtask/mpmath_eval/functionals/pbelocc.py
@xtask/mpmath_eval/functionals/brx.py

# Driver
@xtask/src/bin/regen_mpmath_fixtures.rs

<interfaces>
<!-- Pattern that every per-functional port follows (mirrors blocx.py / pbelocc.py / tw.py): -->
```python
"""mpmath port of xcfun-master/src/functionals/<name>.cpp at mp.prec=200."""
from __future__ import annotations
import mpmath as mp

from ..ad_chain import multivariate_taylor
from ..densvars import slot_names
# (optional) from .._<substrate> import <helpers>


def _value_<name>(*inputs, vars_str):
    slots = slot_names(vars_str)
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    a = d.get("a", mp.mpf(0))
    b = d.get("b", mp.mpf(0))
    # ... unpack the rest, compute scalar functional value at point ...
    return <scalar mp.mpf result>


def eval_<name>(inputs, vars, mode, order):
    mp.mp.prec = 200
    pt = tuple(mp.mpf(x) for x in inputs)
    return multivariate_taylor(
        lambda *args, _vs=vars: _value_<name>(*args, vars_str=_vs),
        pt,
        order,
    )
```
The `mode` argument is accepted for API compatibility but only
`partial_derivatives` is exercised at this tier.

<!-- Vars layouts (from xtask/src/bin/regen_mpmath_fixtures.rs::vars_for): -->
ldaerfx, ldaerfc, ldaerfc_jt   →  "A_B"  (2 slots: a, b)
tpssc, tpsslocc, revtpssc      →  "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB"  (9 slots)

<!-- _pw92eps.py exposes -->
def pw92eps(a, b) -> mp.mpf            # Spin-dependent PW92 correlation eps (already used by pbelocc.py).
def eopt(sqrtr, t) -> mp.mpf
def omega(zeta) -> mp.mpf
def _ufunc(x, a) -> mp.mpf

<!-- xcfun constants (from constants.hpp:34-37; mpmath ports must use these exact forms): -->
PI2          = mp.pi * mp.pi
param_gamma  = (1 - mp.log(2)) / PI2                         # 0.031090690869654895
param_beta_pbe_paper = mp.mpf("0.066725")                    # used by revtpss_beta only
param_beta_accurate  = mp.mpf("0.06672455060314922")
param_beta_gamma     = param_beta_accurate / param_gamma     # used by pbec_eps::A and ::H

<!-- ufunc(x, a) per specmath.hpp:35-37 -->
def ufunc(x, a) -> mp.mpf:
    return mp.power(1 + x, a) + mp.power(1 - x, a)
</interfaces>

# Why we need the two new substrate modules

The Plan 06-N2 substrate factoring rule (see `_pw92eps.py` and
`_scan_like.py`): when ≥2 functionals share a non-trivial helper, the
helper lives at `xtask/mpmath_eval/_<name>.py` (leading underscore at the
package root, NOT under `functionals/` — this preserves W-6 revision-2's
"one .py per functional" invariant on the `functionals/` directory).

* **`_ldaerf_eps.py`** is shared by `ldaerfx.py` (`esrx_ldaerfspin`),
  `ldaerfc.py` (`Qrpa`, `dpol`, `g0f`, `ecorrlr`), `ldaerfc_jt.py`
  (`c1`, `c2`, plus the inline `vwn5_eps` reference). Hosting these in
  one place keeps the 4-branch `a < 1e-9 / a < 100 / a < 1e9 / else`
  control flow, the Newton-style boundary handling, and the small
  `vwn5_eps_mp` mpmath equivalent (~50 LOC) factored out.
* **`_tpss_eps.py`** is shared by `tpssc.py` (`pbec_eps` + `pbec_eps_polarized` + `tpssc_C` with C0=0.53),
  `tpsslocc.py` (`pbeloc_eps` + `pbeloc_eps_pola` + `tpsslocc_C` with C0=0.35),
  `revtpssc.py` (`revtpss_pbec_eps` + `revtpss_pbec_eps_polarized` + `revtpssc_C` with C0=0.59).
  Also exports `tau_clamp(tau, gnn, n)` returning `mp.fmax(tau, gnn/(8*n))`
  — this is the mpmath analog of the Rust `ctaylor_max(d.tau, tau_w)`
  D-10 guard, applied scalarly to the SCALAR per-point evaluation
  (multivariate_taylor numerically diff-extends from the scalar).

# Why the LDAERF and TPSS-C bodies are scalar mpmath, not Taylor-symbolic

`ad_chain.multivariate_taylor` already wraps any `f(*scalar_args) -> mp.mpf`
into the rev-gradlex output vector via `mp.diff`. Each per-functional
module only needs a `_value_<name>` that returns the SCALAR functional
value at a point — multivariate_taylor handles the AD chain at prec=200.
This is the BLOCX / PBELOCC / TW / BRX pattern; we are NOT writing
hand-coded Taylor expansions.

</context>

<tasks>

<task type="auto">
  <name>Task 1: Fill the LDAERF family (3 functionals + _ldaerf_eps.py substrate)</name>

  <read_first>
    - xtask/mpmath_eval/functionals/ldaerfx.py (stub to fill)
    - xtask/mpmath_eval/functionals/ldaerfc.py (stub to fill)
    - xtask/mpmath_eval/functionals/ldaerfc_jt.py (stub to fill)
    - xcfun-master/src/functionals/ldaerfx.cpp (C++ source-of-truth: esrx_ldaerfspin lines 24-47, lda_erfx lines 49-52)
    - xcfun-master/src/functionals/ldaerfc.cpp (C++ source-of-truth: Qrpa lines 23-31, dpol lines 33-40, g0f lines 46-53, ecorrlr lines 55-104, ldaerfc lines 106-110)
    - xcfun-master/src/functionals/ldaerfc_jt.cpp (C++ source-of-truth: c1 lines 24-32, c2 lines 34-45, ldaerfc_jt lines 47-53)
    - xcfun-master/src/functionals/vwn.hpp (vwn5_eps lines 54-78 — needed by ldaerfc_jt::c2)
    - crates/xcfun-kernels/src/functionals/lda/ldaerfx.rs (Rust kernel — algorithmic-identity reference; note the expm1 STABLE rederivation Plan 02-06 Fix 1; mpmath at prec=200 does NOT need stable rederivation since cancellation is f64-only)
    - crates/xcfun-kernels/src/functionals/lda/ldaerfc.rs (Rust kernel — algorithmic-identity reference)
    - crates/xcfun-kernels/src/functionals/lda/ldaerfc_jt.rs (Rust kernel — algorithmic-identity reference)
    - xtask/mpmath_eval/_pw92eps.py (substrate pattern: shared mpmath helper at package root with leading underscore)
    - xtask/mpmath_eval/_scan_like.py (longer substrate example with 4-branch / parameter-table style)
    - xtask/mpmath_eval/functionals/blocx.py (port pattern reference — _value_<name>/eval_<name> + multivariate_taylor wrapper)
    - xtask/mpmath_eval/functionals/pbelocc.py (port pattern reference — uses pw92eps from substrate)
    - xtask/mpmath_eval/functionals/brx.py (port pattern reference — also uses 4-branch dispatch via mp.findroot)
    - xtask/mpmath_eval/densvars.py::slot_names (slot-name dispatch; vars="A_B" yields ["a", "b"])
    - xtask/mpmath_eval/ad_chain.py (multivariate_taylor — mode/order semantics + raw-partial-derivative output convention)
  </read_first>

  <files>
    xtask/mpmath_eval/_ldaerf_eps.py
    xtask/mpmath_eval/functionals/ldaerfx.py
    xtask/mpmath_eval/functionals/ldaerfc.py
    xtask/mpmath_eval/functionals/ldaerfc_jt.py
  </files>

  <action>
    Per ACC-04 (D-03 amendment) + D-11 libm-hybrid erf substrate, port
    the 3 LDAERF-family functional bodies to mpmath@prec=200. ALL
    arithmetic uses `mp.mpf` / mpmath transcendentals; the C++ stable-bracket
    expm1 rederivation in the Rust kernel is f64-only — at prec=200 the
    bracket cancellation is irrelevant (200-digit precision absorbs
    cancellation by construction). MPMATH PORTS USE THE ORIGINAL C++ FORMULA,
    NOT the stable-rederivation. This is critical because mpmath is meant
    to be the algebraic ground truth, not a transcription of the f64-stable
    workaround.

    Range-separation parameter `mu` is hard-coded to `mp.mpf("0.4")`
    (matches `RANGESEP_MU_F64 = 0.4` in `crates/xcfun-kernels/src/functionals/lda/ldaerfx.rs:51`,
    Phase 5 RS-01..10 left runtime-mu wiring deferred to Phase 7).

    --- Step 1.1: Create `xtask/mpmath_eval/_ldaerf_eps.py` ---

    NEW FILE. Follows the `_pw92eps.py` / `_scan_like.py` pattern.
    Module docstring cites `xcfun-master/src/functionals/ldaerfx.cpp:24-47`,
    `ldaerfc.cpp:23-104`, `ldaerfc_jt.cpp:24-45`, and `vwn.hpp:54-78`.

    Exports (with type signatures matching the C++ scalar prototypes):

    ```python
    # esrx_ldaerfspin(na, mu) -> mp.mpf
    # 4-branch port of ldaerfx.cpp:24-47 — VERBATIM (no stable rederivation):
    #   ckf = mp.mpf("3.093667726280136")
    #   rhoa = 2 * na
    #   akf = ckf * mp.cbrt(rhoa)
    #   a = mu / (2 * akf); a2 = a*a; a3 = a2*a
    #   if a < 1e-9: return -3/8 * rhoa * (24*rhoa/pi)^(1/3)
    #   elif a < 100:
    #     return -(rhoa * (24*rhoa/pi)^(1/3)) * (3/8 - a*(sqrt(pi)*erf(0.5/a)
    #            + (2*a - 4*a3) * exp(-0.25/a2) - 3*a + 4*a3))
    #   elif a < 1e9: return -(rhoa * (24*rhoa/pi)^(1/3)) / (96*a2)
    #   else: return mp.mpf(0)
    # Use mp.erf, mp.exp, mp.sqrt; cbrt via mp.cbrt or mp.power(.., 1/3).

    # Qrpa(x) per ldaerfc.cpp:23-31:
    #   Acoul = 2 * (log(2) - 1) / pi**2
    #   a2 = mp.mpf("5.84605"); c2 = mp.mpf("3.91744"); d2 = mp.mpf("3.44851")
    #   b2 = d2 - 3.0/(2*pi*Acoul) * (4/(9*pi))^(1/3)
    #   return Acoul * log((1 + x*(a2 + x*(b2 + c2*x))) / (1 + x*(a2 + d2*x)))

    # dpol(rs) per ldaerfc.cpp:33-40:
    #   cf = (9*pi/4)^(1/3)
    #   p2p = 0.04; p3p = 0.4319
    #   rs2 = rs*rs
    #   return 2^(5/3)/5 * cf**2 / rs2 * (1 + (p3p - 0.454555)*rs)
    #          / (1 + p3p*rs + p2p*rs2)

    # g0f(rs) per ldaerfc.cpp:46-53:
    #   C0f = 0.0819306; D0f = 0.752411; E0f = -0.0127713; F0f = 0.00185898
    #   return (1 + rs*(D0f - 0.7317 + rs*(C0f + rs*(E0f + F0f*rs))))
    #          * exp(-D0f * rs) / 2

    # ecorrlr(d_dict, mu, ec) per ldaerfc.cpp:55-104 — d_dict comes from
    # build_densvars; needs r_s, n, zeta. Port verbatim.
    # Exports the long expression assembled from coe2..coe5, b06, b08, a1..a4
    # and the final pow(phi,3)*Qrpa(...) sum.
    # CRITICAL: at the entry, take phi from
    #   phi = (mp.power(1 + zeta, 2/3) + mp.power(1 - zeta, 2/3)) / 2

    # c1(rs) per ldaerfc_jt.cpp:24-32 — 3-parameter ratio:
    #   u1 = 1.0270741452992294; u2 = -0.230160617208092; v1 = 0.6196884832404359
    #   return (u1*rs + u2*rs*rs) / (1 + v1*rs)

    # c2(d_dict) per ldaerfc_jt.cpp:34-45:
    #   a = 3.2581; f = 3.39530545262710070631; bet = 163.44; gam = 4.7125
    #   g0 = f * (gam + r_s)^1.5 + bet) * exp(-a*sqrt(gam + r_s))
    #   denom = 0.5*pi*n^2 * (g0 - 0.5)
    #   return n * vwn5_eps_mp(d_dict) / denom

    # vwn5_eps_mp(d_dict) per vwn.hpp:54-78 (non-XCFUN_VWN5_REF arm):
    #   Implements vwn_a/_b/_c/_x/_y/_z/_f as scalar mpmath helpers and
    #   composes them per vwn5_eps. The 4 parameter tables (para, ferro,
    #   inter) live as module-private constants per vwn.hpp:57-60.
    #   `pow(3 * pi**2, -1)` for inter[1] is a single mp.mpf.
    ```

    All numeric literals use `mp.mpf("...")` strings to avoid f64 rounding
    at module-load time. Cite the C++ line numbers in inline comments.

    --- Step 1.2: Fill `xtask/mpmath_eval/functionals/ldaerfx.py` ---

    REPLACE the `raise NotImplementedError(...)` body with:

    ```python
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


    _MU = mp.mpf("0.4")  # XC_RANGESEP_MU default; matches Rust RANGESEP_MU_F64.


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
    ```

    --- Step 1.3: Fill `xtask/mpmath_eval/functionals/ldaerfc.py` ---

    Same shape. The `_value_ldaerfc` builds a `d_dict` containing
    `n`, `zeta`, `r_s`, `a`, `b`, then calls `pw92eps(a, b)` (already in
    `_pw92eps.py`) for the unscreened LDA-c eps, then `ecorrlr(d_dict,
    mu=0.4, ec=eps)` from `_ldaerf_eps`, then returns
    `n * (eps - ecorrlr_result)` per `ldaerfc.cpp:108-109`.

    The d_dict construction mirrors `densvars.build_densvars(inputs, "A_B")`:

    ```python
    n = a + b
    zeta = (a - b) / n          # only used inside ecorrlr
    r_s = mp.power(3 / (n * 4 * mp.pi), mp.mpf(1) / mp.mpf(3))
    d_dict = {"a": a, "b": b, "n": n, "zeta": zeta, "r_s": r_s}
    ```

    --- Step 1.4: Fill `xtask/mpmath_eval/functionals/ldaerfc_jt.py` ---

    Same shape. Uses `c1(r_s)` and `c2(d_dict)` from `_ldaerf_eps`. Body
    per `ldaerfc_jt.cpp:47-53`:

    ```python
    denom = 1 + c1(r_s) * mu + c2(d_dict) * mu * mu
    result = n * vwn5_eps_mp(d_dict) / denom
    return result
    ```

    Vars layout for all three is `"A_B"` (2 slots: a, b). The
    multivariate_taylor wrapper extends scalarly to higher orders.

    --- DO NOT modify ---

    - `xtask/mpmath_eval/functionals/__init__.py` already imports + LOOKUP-registers
      these 3 modules. Filling the bodies is sufficient.
  </action>

  <verify>
    <automated>
# Stubs are gone:
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/ldaerfx.py
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/ldaerfc.py
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/ldaerfc_jt.py
# Substrate exists with required exports:
test -f xtask/mpmath_eval/_ldaerf_eps.py
grep -q "def esrx_ldaerfspin" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "def Qrpa" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "def ecorrlr" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "def c1" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "def c2" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "vwn5_eps" xtask/mpmath_eval/_ldaerf_eps.py
# Imports wire correctly:
grep -q "from \.\._ldaerf_eps import" xtask/mpmath_eval/functionals/ldaerfx.py
grep -q "from \.\._ldaerf_eps import" xtask/mpmath_eval/functionals/ldaerfc.py
grep -q "from \.\._ldaerf_eps import" xtask/mpmath_eval/functionals/ldaerfc_jt.py
# Smoke single-point eval (use whichever python3 has mpmath; the env-var
# default is set by Task 3, but for Task 1 verify we can fall back manually):
python3 -c "
import sys
sys.path.insert(0, '.')
from xtask.mpmath_eval.functionals.ldaerfx import eval_ldaerfx
from xtask.mpmath_eval.functionals.ldaerfc import eval_ldaerfc
from xtask.mpmath_eval.functionals.ldaerfc_jt import eval_ldaerfc_jt
import mpmath as mp
mp.mp.prec = 200
out_x = eval_ldaerfx([mp.mpf('1.1'), mp.mpf('1.0')], 'A_B', 'partial_derivatives', 0)
out_c = eval_ldaerfc([mp.mpf('1.1'), mp.mpf('1.0')], 'A_B', 'partial_derivatives', 0)
out_jt = eval_ldaerfc_jt([mp.mpf('1.1'), mp.mpf('1.0')], 'A_B', 'partial_derivatives', 0)
assert all(mp.isfinite(v) for v in out_x + out_c + out_jt), 'non-finite'
print('OK', float(out_x[0]), float(out_c[0]), float(out_jt[0]))
"
# (Tier-0 sanity: ldaerfx[0] should match -1.553573128702155 to ~1e-7
# per the C++ test_threshold; do NOT assert tighter — exact match
# requires order=2 + the full mpmath AD vector, which Task 3 verifies
# via `regen-mpmath-fixtures --smoke`.)
    </automated>
  </verify>

  <done>
    All 3 LDAERF stub files no longer raise NotImplementedError; each
    `eval_<name>(inputs, vars, mode, 0)` returns a list of finite
    mp.mpf values for at least one canonical test input (`{a=1.1, b=1.0}`
    for the LDAERFX test_in). `_ldaerf_eps.py` provides
    `esrx_ldaerfspin`, `Qrpa`, `dpol`, `g0f`, `ecorrlr`, `c1`, `c2`,
    `vwn5_eps_mp` as documented. Imports compile cleanly under
    `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP"`.
    Numerical sanity checks (energy-slot agreement to ≤ 1e-7 at the
    canonical LDAERFX test_in `{1.1, 1.0}` and a fixed-vector consistency
    check vs. the Rust kernel at one density point) optional in this task
    — strict 1e-13 verification belongs to Task 3 via the smoke regen lane.
  </done>
</task>

<task type="auto">
  <name>Task 2: Fill the TPSS-C family (3 functionals + _tpss_eps.py substrate with tau-clamp guard)</name>

  <read_first>
    - xtask/mpmath_eval/functionals/tpssc.py (stub to fill)
    - xtask/mpmath_eval/functionals/tpsslocc.py (stub to fill)
    - xtask/mpmath_eval/functionals/revtpssc.py (stub to fill)
    - xcfun-master/src/functionals/tpssc.cpp (C++ source-of-truth: tpssc lines 19-21)
    - xcfun-master/src/functionals/tpssc_eps.hpp (C++ source-of-truth: C lines 22-31, epsc_summax 33-42, epsc_revpkzb 44-54, tpssc_eps 56-61, dd=2.8)
    - xcfun-master/src/functionals/tpsslocc.cpp (C++ source-of-truth: phi 19-22, pbeloc_eps 24-41, pbeloc_eps_pola 43-61, C 63-69 with C0=0.35, epsc_summax 72-82, epsc_revpkzb 84-90, energy 92-97 with dd=4.5 — note the dd=4.5 vs tpssc's 2.8 is intentional)
    - xcfun-master/src/functionals/revtpssc.cpp (C++ source-of-truth: revtpssc lines 19-21)
    - xcfun-master/src/functionals/revtpssc_eps.hpp (C++ source-of-truth: revtpssA 24-29, revtpssH 31-41, revtpss_beta 43-47, revtpss_pbec_eps 49-57, revtpss_pbec_eps_polarized 59-68, C 70-80 with C0=0.59 + 0.9269*z² + 0.6225*z⁴ + 2.1540*z⁶, epsc_summax 82-91, epsc_revpkzb 93-103, revtpssc_eps 105-110 with dd=2.8 + tauwtau2 not tauwtau3 — note this is intentional in revtpssc_eps.hpp:105-109)
    - xcfun-master/src/functionals/pbec_eps.hpp (C++ source-of-truth: A 23-27, H 29-36, phi 38-41, pbec_eps 43-50, pbec_eps_polarized 52-61)
    - xcfun-master/src/functionals/constants.hpp:34-37 (param_gamma, param_beta_pbe_paper, param_beta_accurate, param_beta_gamma)
    - xcfun-master/src/specmath.hpp:35-37 (ufunc(x, a) = pow(1+x, a) + pow(1-x, a))
    - crates/xcfun-kernels/src/functionals/mgga/tpssc.rs (Rust kernel — D-10 tau_clamped pattern via build_tau_w + ctaylor_max)
    - crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs (Rust kernel — full tau_clamped flow into both inner epsc_revpkzb_with_tau and outer tauwtau3)
    - crates/xcfun-kernels/src/functionals/mgga/revtpssc.rs (Rust kernel — same pattern)
    - xtask/mpmath_eval/_pw92eps.py::pw92eps (substrate; consumed by every TPSS-C eps_pkzb branch)
    - xtask/mpmath_eval/functionals/blocx.py (port pattern reference — 9-slot vars unpack)
    - xtask/mpmath_eval/functionals/pbelocc.py (port pattern reference — exp/expm1/log composition with PBE-eps shape)
    - xtask/mpmath_eval/densvars.py::slot_names + build_densvars (slot-name dispatch for "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB")
  </read_first>

  <files>
    xtask/mpmath_eval/_tpss_eps.py
    xtask/mpmath_eval/functionals/tpssc.py
    xtask/mpmath_eval/functionals/tpsslocc.py
    xtask/mpmath_eval/functionals/revtpssc.py
  </files>

  <action>
    Per ACC-04 D-03 amendment + D-10 tau-clamp guard, port the 3 TPSS-C
    family functional bodies to mpmath@prec=200. The mpmath ports apply
    the D-10 clamp `tau = max(tau, gnn / (8*n))` BEFORE composing the
    `tauwtau2` / `tauwtau3` ratios, mirroring the Rust kernel's
    intentional divergence from C++ in the `tau ≪ tau_w` regime
    (verified algorithmically faithful by Phase 4 Plan 04-10 Path-B).

    The 9 input slots (vars="A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB") are
    unpacked via `densvars.slot_names`. Note that `lapa, lapb` are
    present in the input layout but UNUSED in the TPSS-C bodies (TPSS-C
    is gradient + kinetic but not laplacian — `XC_DENSITY | XC_GRADIENT
    | XC_KINETIC` per `tpssc.cpp:32`). Build the same `d_dict` as
    `densvars.build_densvars` augments: `n, s, zeta, gnn, gns, gss,
    r_s` plus the per-spin slots and `tau = taua + taub`.

    --- Step 2.1: Create `xtask/mpmath_eval/_tpss_eps.py` ---

    NEW FILE. Module docstring cites
    `xcfun-master/src/functionals/{tpssc.cpp, tpssc_eps.hpp, tpsslocc.cpp,
    revtpssc.cpp, revtpssc_eps.hpp, pbec_eps.hpp, constants.hpp}`.

    Constants section (use mp.mpf strings + the constants.hpp formulae):

    ```python
    PI2 = mp.pi * mp.pi
    param_gamma = (1 - mp.log(2)) / PI2
    param_beta_pbe_paper = mp.mpf("0.066725")
    param_beta_accurate = mp.mpf("0.06672455060314922")
    param_beta_gamma = param_beta_accurate / param_gamma
    # PBEC d2 prefactor — matches PBEC_D2_PREFACTOR_F64 in xcfun-kernels
    # gga::shared::constants. = (1/12 * 3^(5/6) / pi^(-1/6))^2 = (3^(5/6) * pi^(1/6) / 12)^2.
    # Compute symbolically at module load (mp.power resolves at prec=200).
    ```

    Exports (cite the C++ source line for each):

    ```python
    # phi_reorganised(a, b, n) per pbec_eps.hpp:38-41:
    #   = 2^(-1/3) * n_m13^2 * (sqrt(a^(4/3)) + sqrt(b^(4/3)))
    # where n_m13 = n^(-1/3); a_43 = a^(4/3); b_43 = b^(4/3).

    # tau_clamp(tau, gnn, n) -> mp.mpf:
    #   tau_w = gnn / (8*n)
    #   return mp.fmax(tau, tau_w)
    # This is the SCALAR D-10 guard. multivariate_taylor extends it
    # via mp.diff at prec=200; the C++ ctaylor_max compares CNST slots
    # only, which corresponds to scalar selection at the eval point.

    # pbec_A(eps, u3) per pbec_eps.hpp:23-27:
    #   return param_beta_gamma / (mp.exp(-eps / (param_gamma * u3)) - 1)
    # NOTE: C++ uses expm1; at prec=200 expm1 is identical to exp - 1
    # (no cancellation). Use mp.exp(-x) - 1; comment that expm1 is f64-only.

    # pbec_H(d2, eps, u3) per pbec_eps.hpp:29-36 with mpmath log:
    #   d2A = d2 * pbec_A(eps, u3)
    #   return param_gamma * u3 *
    #          mp.log(1 + param_beta_gamma * d2 * (1 + d2A) / (1 + d2A * (1 + d2A)))

    # pbec_eps(d_dict) per pbec_eps.hpp:43-50 (uses pw92eps + phi):
    #   eps = pw92eps(d_dict["a"], d_dict["b"])     # already in _pw92eps
    #   u = phi_reorganised(a, b, n)
    #   d2 = D2_PREFACTOR * gnn / (u^2 * n^(7/3))
    #   return eps + pbec_H(d2, eps, u**3)

    # pbec_eps_polarized(a, gaa) per pbec_eps.hpp:52-61:
    #   eps = pw92eps_polarized(a)   # need this: see below
    #   u = mp.power(2, -1/3)        # phi for fully polarized
    #   d2 = D2_PREFACTOR * gaa / (u^2 * a^(7/3))
    #   return eps + pbec_H(d2, eps, u**3)

    # pw92eps_polarized(a): pw92eps from _pw92eps.py with b -> 0 limit;
    # implement by calling pw92eps(a, mp.mpf("1e-300")) IS WRONG —
    # instead, write a dedicated helper that mirrors C++ pw92eps's
    # polarized arm where zeta=1 / 4 (zeta^4) = 1 / omega(1) is finite
    # (omega(1) = (2^(4/3) - 2)/(2*2^(1/3) - 2) = 2^(1/3)). Read
    # xcfun-master/src/functionals/pw92eps.hpp:48-61 for the
    # polarized formula. Or simpler: extend _pw92eps.py with a
    # dedicated `pw92eps_polarized(a)` for the spin-fully-polarized
    # ferro arm (eopt with PW92C_PARAMS[1] only, zeta=1, ec_para drops out).

    # tpssc_C(d_dict) per tpssc_eps.hpp:22-31. C0 = 0.53 + 0.87*ζ²
    #         + 0.50*ζ⁴ + 2.26*ζ⁶ (NOTE: 0.53!)
    #   gzeta2 = (n²*gss - 2*n*s*gns + s²*gnn) / n^4
    #   xi2 = gzeta2 / (4 * (3*pi²*n)^(2/3))
    #   uf = ufunc(zeta, -4/3) = (1+ζ)^(-4/3) + (1-ζ)^(-4/3)
    #   return C0 / (1 + 0.5*xi2*uf)^4

    # tpsslocc_C(d_dict) per tpsslocc.cpp:63-69. C0 = 0.35 + 0.87*ζ²
    #         + 0.50*ζ⁴ + 2.26*ζ⁶ (NOTE: 0.35 vs tpssc's 0.53!)
    # Same xi2 + uf composition.

    # revtpssc_C(d_dict) per revtpssc_eps.hpp:70-80. C0 = 0.59
    #         + 0.9269*ζ² + 0.6225*ζ⁴ + 2.1540*ζ⁶
    # Same xi2 + uf composition.

    # revtpss_beta(dens) per revtpssc_eps.hpp:43-47:
    #   r_s = mp.cbrt(3 / (4*pi*dens))
    #   return param_beta_pbe_paper * (1 + 0.1*r_s) / (1 + 0.1778*r_s)

    # revtpss_pbec_eps(d_dict) per revtpssc_eps.hpp:49-57. Like pbec_eps
    # but with beta = revtpss_beta(d.n) (position-dependent).

    # revtpss_pbec_eps_polarized(a, gaa) per revtpssc_eps.hpp:59-68.

    # pbeloc_eps(d_dict) per tpsslocc.cpp:24-41. PBE-loc eps with
    # position-dependent beta = beta0 + aa*d2*ff where ff = 1 - exp(-r_s²),
    # beta0 = 0.0375, aa = 0.08.

    # pbeloc_eps_pola(a, gaa) per tpsslocc.cpp:43-61 — same with the
    # internal r_s = (3/(4*pi))^(1/3) * a^(-1/3).
    ```

    All literals as mp.mpf strings; all power/log/exp via mpmath; no f64
    intermediates. Cite C++ line numbers in comments.

    --- Step 2.2: Fill `xtask/mpmath_eval/functionals/tpssc.py` ---

    REPLACE the `raise NotImplementedError(...)` body with a port of
    `tpssc.cpp:19-21`:

    ```python
    """mpmath port of xcfun-master/src/functionals/tpssc.cpp at mp.prec=200.

    Plan 06-N5 — fills the Plan 06-N2 stub. ACC-04 D-03 amendment +
    D-10 tau-clamp guard. The Rust kernel at
    crates/xcfun-kernels/src/functionals/mgga/tpssc.rs applies
    `tau_clamped = max(tau, tau_w)` (build_tau_w + ctaylor_max);
    this mpmath port mirrors that clamp scalarly via `_tpss_eps.tau_clamp`,
    then evaluates `n * tpssc_eps(d_with_clamped_tau)`.

    C++ has NO equivalent guard — it returns the cancellation-affected
    value in the unphysical `tau ≪ tau_w` regime. Per D-03, mpmath@200
    with the clamp is the truth at the boundary; C++ is no longer the
    truth in this regime (Plan 04-10 Path-B confirmed algorithmic
    faithfulness, isolated the divergence to f64 cancellation).
    """
    from __future__ import annotations
    import mpmath as mp

    from ..ad_chain import multivariate_taylor
    from ..densvars import slot_names
    from .._tpss_eps import (
        tau_clamp, pbec_eps, pbec_eps_polarized, tpssc_C,
    )
    from .._pw92eps import pw92eps as pw92eps_unscreened


    DD = mp.mpf("2.8")  # tpssc_eps.hpp:59 — DIFFERENT from tpsslocc's 4.5.


    def _value_tpssc(*inputs, vars_str):
        slots = slot_names(vars_str)
        d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
        a = d.get("a", mp.mpf(0))
        b = d.get("b", mp.mpf(0))
        gaa = d.get("gaa", mp.mpf(0))
        gab = d.get("gab", mp.mpf(0))
        gbb = d.get("gbb", mp.mpf(0))
        taua = d.get("taua", mp.mpf(0))
        taub = d.get("taub", mp.mpf(0))
        n = a + b
        s = a - b
        zeta = s / n
        gnn = gaa + 2 * gab + gbb
        gns = gaa - gbb
        gss = gaa - 2 * gab + gbb
        tau = taua + taub
        # D-10 clamp.
        tau_clamped = tau_clamp(tau, gnn, n)
        d_dict = {
            "a": a, "b": b, "n": n, "s": s, "zeta": zeta,
            "gnn": gnn, "gns": gns, "gss": gss,
            "gaa": gaa, "gab": gab, "gbb": gbb,
            "tau": tau_clamped,
        }
        # tpssc_eps body: epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3),
        # where epsc_revpkzb = pbec_eps * (1 + C*tauwtau2)
        #                    - (1 + C) * tauwtau2 * epsc_summax_pbe.
        epsc_pbe = pbec_eps(d_dict)
        epsc_pbe_a = pbec_eps_polarized(a, gaa)
        epsc_pbe_b = pbec_eps_polarized(b, gbb)
        epsc_summax = (a * mp.fmax(epsc_pbe, epsc_pbe_a)
                       + b * mp.fmax(epsc_pbe, epsc_pbe_b)) / n
        tauwtau = gnn / (8 * n * tau_clamped)
        tauwtau2 = tauwtau ** 2
        tauwtau3 = tauwtau ** 3
        C_val = tpssc_C(d_dict)
        epsc_revpkzb = (epsc_pbe * (1 + C_val * tauwtau2)
                        - (1 + C_val) * tauwtau2 * epsc_summax)
        eps = epsc_revpkzb * (1 + DD * epsc_revpkzb * tauwtau3)
        return n * eps


    def eval_tpssc(inputs, vars, mode, order):
        mp.mp.prec = 200
        pt = tuple(mp.mpf(x) for x in inputs)
        return multivariate_taylor(
            lambda *args, _vs=vars: _value_tpssc(*args, vars_str=_vs),
            pt,
            order,
        )
    ```

    --- Step 2.3: Fill `xtask/mpmath_eval/functionals/tpsslocc.py` ---

    Same shape but uses `pbeloc_eps`, `pbeloc_eps_pola`, `tpsslocc_C`
    from `_tpss_eps` (instead of `pbec_eps` + `pbec_eps_polarized` +
    `tpssc_C`). Constant `DD = mp.mpf("4.5")` per `tpsslocc.cpp:95`.
    Final return is `n * eps_pkzb * (1 + DD * eps_pkzb * tauwtau3)`
    per `tpsslocc.cpp:96`.

    --- Step 2.4: Fill `xtask/mpmath_eval/functionals/revtpssc.py` ---

    Same shape but uses `revtpss_pbec_eps`, `revtpss_pbec_eps_polarized`,
    `revtpssc_C` from `_tpss_eps`. Constant `DD = mp.mpf("2.8")` per
    `revtpssc_eps.hpp:108`. CRITICAL: the outer term is
    `(1 + DD * eps_pkzb * tauwtau2)`, NOT `tauwtau3` — see
    `revtpssc_eps.hpp:107-109` (different from tpsslocc's tauwtau3 body).
    Final return is `n * revtpssc_eps(d_with_clamped_tau)` per
    `revtpssc.cpp:19-21`.

    --- DO NOT modify ---

    - `xtask/mpmath_eval/functionals/__init__.py` already imports + LOOKUP-registers
      these 3 modules. Filling the bodies is sufficient.
  </action>

  <verify>
    <automated>
# Stubs are gone:
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/tpssc.py
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/tpsslocc.py
! grep -q "NotImplementedError" xtask/mpmath_eval/functionals/revtpssc.py
# Substrate exists with required exports:
test -f xtask/mpmath_eval/_tpss_eps.py
grep -q "def tau_clamp" xtask/mpmath_eval/_tpss_eps.py
grep -q "def pbec_eps" xtask/mpmath_eval/_tpss_eps.py
grep -q "def pbeloc_eps" xtask/mpmath_eval/_tpss_eps.py
grep -q "def revtpss_pbec_eps" xtask/mpmath_eval/_tpss_eps.py
grep -q "def tpssc_C" xtask/mpmath_eval/_tpss_eps.py
grep -q "def tpsslocc_C" xtask/mpmath_eval/_tpss_eps.py
grep -q "def revtpssc_C" xtask/mpmath_eval/_tpss_eps.py
grep -q "def revtpss_beta" xtask/mpmath_eval/_tpss_eps.py
# Different DD constants per the C++ sources:
grep -q '"2.8"' xtask/mpmath_eval/functionals/tpssc.py
grep -q '"4.5"' xtask/mpmath_eval/functionals/tpsslocc.py
grep -q '"2.8"' xtask/mpmath_eval/functionals/revtpssc.py
# tau_clamp wired into all 3:
grep -q "tau_clamp" xtask/mpmath_eval/functionals/tpssc.py
grep -q "tau_clamp" xtask/mpmath_eval/functionals/tpsslocc.py
grep -q "tau_clamp" xtask/mpmath_eval/functionals/revtpssc.py
# Smoke single-point eval (canonical test_in for tpssc per tpssc.cpp:37):
python3 -c "
import sys
sys.path.insert(0, '.')
from xtask.mpmath_eval.functionals.tpssc import eval_tpssc
from xtask.mpmath_eval.functionals.tpsslocc import eval_tpsslocc
from xtask.mpmath_eval.functionals.revtpssc import eval_revtpssc
import mpmath as mp
mp.mp.prec = 200
# tpssc test_in (tpssc.cpp:37): {1, 2, 3, 4, 5, 6, 7} on
# A_B_GAA_GAB_GBB_TAUA_TAUB (7 slots) — but our regen uses the 9-slot
# layout A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB. Pad with lapa=lapb=0:
inp = [mp.mpf(x) for x in (1, 2, 3, 4, 5, 0, 0, 6, 7)]
v = 'A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB'
out_c = eval_tpssc(inp, v, 'partial_derivatives', 0)
out_lc = eval_tpsslocc(inp, v, 'partial_derivatives', 0)
out_r = eval_revtpssc(inp, v, 'partial_derivatives', 0)
assert all(mp.isfinite(x) for x in out_c + out_lc + out_r), 'non-finite'
print('OK', float(out_c[0]), float(out_lc[0]), float(out_r[0]))
"
    </automated>
  </verify>

  <done>
    All 3 TPSS-C stub files no longer raise NotImplementedError; each
    `eval_<name>(inputs, vars, mode, 0)` returns finite mp.mpf values.
    The D-10 `tau_clamp` is applied scalarly before `tauwtau{2,3}`
    composition. `_tpss_eps.py` provides `tau_clamp`, `phi_reorganised`,
    `pbec_eps`, `pbec_eps_polarized`, `pbeloc_eps`, `pbeloc_eps_pola`,
    `revtpss_pbec_eps`, `revtpss_pbec_eps_polarized`, `revtpss_beta`,
    `tpssc_C`, `tpsslocc_C`, `revtpssc_C`. Imports compile cleanly.
    Strict 1e-13 verification belongs to Task 3 via the smoke regen lane;
    if intrinsic f64 drift exceeds 1e-13 (per the SCAN-family precedent
    in deferred-items.md), record observed tolerance in the SUMMARY and
    do NOT force 1e-13 retroactively.
  </done>
</task>

<task type="auto">
  <name>Task 3: Resolve python interpreter dispatch + end-to-end smoke regen verification</name>

  <read_first>
    - xtask/src/bin/regen_mpmath_fixtures.rs (driver — note line ~203 hardcodes "python3.12")
    - xtask/mpmath_eval/__main__.py (entry point Command target)
    - .planning/phases/07-python-bindings-release/07-00-SUMMARY.md (operator pause note + 3 options listed)
    - CLAUDE.md (no anyhow in libraries; xtask is app-boundary so anyhow is OK there — just confirm)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VERIFICATION.md (UAT #3 — the gap this task closes)
  </read_first>

  <files>
    xtask/src/bin/regen_mpmath_fixtures.rs
  </files>

  <action>
    Resolve the python-interpreter dispatch issue documented in the
    07-00-SUMMARY.md pause note (Option C — env-var override with
    `python3` default), then run end-to-end verification that all 6
    newly-filled functional bodies regen cleanly via the smoke lane.

    Per the SUMMARY pause note:

    > "Driver invokes python3.12 literally (xtask/src/bin/regen_mpmath_fixtures.rs:203),
    > not python3. The operator's primary python3 is 3.14 (user-local at
    > /home/user/.local/bin/python3); they had to install mpmath separately
    > into python3.12 to get past the import gate. Plan 06-N5 should
    > either lift python3.12 into a docs requirement, switch the driver
    > to python3 (and thus track whatever python3 --version ships in CI),
    > or expose the interpreter via a XCFUN_MPMATH_PYTHON env var."

    **Choice: Option C** (env-var override with `python3` default).
    Rationale documented inline in the driver:

    1. `python3` default keeps zero-config OOTB on systems where the
       primary `python3` has `mpmath` installed (most distros + the CI
       images we will use in Phase 7).
    2. `XCFUN_MPMATH_PYTHON` override lets the operator point at a
       non-default interpreter (e.g., `python3.12`, `pypy3`,
       `~/venv/bin/python`) without modifying source.
    3. Documenting both in the module-level docstring keeps the contract
       discoverable.

    --- Step 3.1: Edit `xtask/src/bin/regen_mpmath_fixtures.rs` ---

    Two changes inside `main()`:

    (a) BEFORE the `for fn_name in functionals` loop, resolve the
        interpreter once:

    ```rust
    let mpmath_python = std::env::var("XCFUN_MPMATH_PYTHON")
        .unwrap_or_else(|_| "python3".to_string());
    ```

    (b) Replace `Command::new("python3.12")` (around line 203) with
        `Command::new(&mpmath_python)`.

    Also append an `XCFUN_MPMATH_PYTHON` paragraph to the module-level
    docstring (the `//!` block at the top of the file). Suggested:

    ```rust
    //! ## Python interpreter selection
    //!
    //! The driver invokes `python3 -m xtask.mpmath_eval` by default. To
    //! point at a non-default interpreter (e.g., a venv or a
    //! version-pinned `python3.12` install with `mpmath`), set the
    //! `XCFUN_MPMATH_PYTHON` environment variable:
    //!
    //!   XCFUN_MPMATH_PYTHON=python3.12 cargo run --release -p xtask --bin regen-mpmath-fixtures -- --smoke
    //!
    //! The default `python3` matches `python3 --version` on the host
    //! shell. The interpreter MUST have `mpmath >= 1.4` installed —
    //! `pip install mpmath` or distro-equivalent.
    ```

    Do NOT change anything else (the python3.12 → python3 default is the
    only behavioural change; the env-var override is purely additive).

    --- Step 3.2: End-to-end smoke regen verification ---

    The default `--smoke` flag pool is `{brx, tw, pbelocc, blocx, scanx}`
    (per the existing `smoke_functionals()` constant — see line 162-164
    in the driver), which does NOT include the 6 ACC-04 set. To verify
    the 6 newly-filled bodies, do a full-set smoke that includes them.
    The simplest path: invoke the driver directly with the env override
    and visually confirm that ALL 6 ACC-04 functionals produce a record
    via a one-shot single-point invocation per functional. Then run the
    existing `--smoke` lane to confirm we have not broken the
    Plan-06-N2-merged smoke set (regression check).

    Verification goals:
    1. `cargo build --release -p xtask --bin regen-mpmath-fixtures` succeeds (the env-var change compiles).
    2. The literal `python3.12` no longer appears in the driver source.
    3. For each of the 6 ACC-04 functionals, a single-point invocation
       via `python3 -m xtask.mpmath_eval ...` returns a JSONL record
       with finite floats (no NotImplementedError, no panic).
    4. The pre-existing `--smoke` lane (5 functionals × 5 records) still
       passes — this catches regressions in `_pw92eps.py` /
       `_scan_like.py` if Tasks 1+2 accidentally modified them.
  </action>

  <verify>
    <automated>
# Driver compiles after the env-var change:
cargo build --release -p xtask --bin regen-mpmath-fixtures 2>&1 | tail -5
# python3.12 literal is gone:
! grep -q "python3.12" xtask/src/bin/regen_mpmath_fixtures.rs
# XCFUN_MPMATH_PYTHON env var is wired:
grep -q "XCFUN_MPMATH_PYTHON" xtask/src/bin/regen_mpmath_fixtures.rs
# Default falls back to python3:
grep -q '"python3"' xtask/src/bin/regen_mpmath_fixtures.rs
# Pre-existing --smoke lane (Plan-06-N2 set) still passes:
cargo run --release -p xtask --bin regen-mpmath-fixtures -- --smoke 2>&1 | tail -10
test -f target/mpmath_smoke/brx.jsonl
test -f target/mpmath_smoke/tw.jsonl
test -f target/mpmath_smoke/pbelocc.jsonl
test -f target/mpmath_smoke/blocx.jsonl
test -f target/mpmath_smoke/scanx.jsonl
# All 6 ACC-04 functionals produce a record via single-point invocation
# (uses XCFUN_MPMATH_PYTHON if set, otherwise python3):
PY="${XCFUN_MPMATH_PYTHON:-python3}"
for fn in ldaerfx ldaerfc ldaerfc_jt; do
  out=$($PY -m xtask.mpmath_eval --functional $fn --vars A_B \
        --mode partial_derivatives --order 0 --input "1.1,1.0" --prec 200 2>&1)
  echo "$out" | grep -q '"output":' || { echo "FAIL: $fn — $out"; exit 1; }
  echo "$out" | grep -q "NotImplementedError" && { echo "FAIL: $fn still stubbed"; exit 1; }
  echo "OK $fn"
done
for fn in tpssc tpsslocc revtpssc; do
  out=$($PY -m xtask.mpmath_eval --functional $fn \
        --vars A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB \
        --mode partial_derivatives --order 0 \
        --input "0.5,0.4,0.1,0.05,0.1,0.0,0.0,0.2,0.15" --prec 200 2>&1)
  echo "$out" | grep -q '"output":' || { echo "FAIL: $fn — $out"; exit 1; }
  echo "$out" | grep -q "NotImplementedError" && { echo "FAIL: $fn still stubbed"; exit 1; }
  echo "OK $fn"
done
echo "All 6 ACC-04 functionals respond to single-point invocation."
    </automated>
  </verify>

  <done>
    1. `xtask/src/bin/regen_mpmath_fixtures.rs` no longer hardcodes
       `python3.12`. Default interpreter is `python3`; override via
       `XCFUN_MPMATH_PYTHON` env var.
    2. Module-level docstring documents the env-var contract.
    3. `cargo build --release -p xtask --bin regen-mpmath-fixtures` succeeds.
    4. All 6 ACC-04 functionals (ldaerfx/ldaerfc/ldaerfc_jt/tpssc/
       tpsslocc/revtpssc) accept a single-point invocation via
       `python3 -m xtask.mpmath_eval` and return a JSONL record with
       finite floats — NO NotImplementedError surfaces anywhere.
    5. Pre-existing `--smoke` lane (5 functionals × 5 records:
       brx/tw/pbelocc/blocx/scanx) still passes — confirms no regression
       in `_pw92eps.py` / `_scan_like.py`.

    Intentionally OUT OF SCOPE:
    - The full ~6h MANUAL regen path (`cargo run --release -p xtask --bin
      regen-mpmath-fixtures` with no flags) — that is Plan 07-00 Task 0.2,
      now unblocked by 06-N5.
    - The strict 1e-13 sweep with `--reference mpmath` — also Plan 07-00
      Task 0.2 / Phase 6 HUMAN-UAT #3 follow-up.
    - Per-functional tolerance overrides for tpssc/revtpssc if intrinsic
      f64 drift exceeds 1e-13 — record observed tolerance in the
      Plan-06-N5 SUMMARY (mirrors the SCAN-family precedent in
      deferred-items.md), do not force 1e-13 retroactively.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| host shell ↔ mpmath sidecar | Rust driver invokes `python3 -m xtask.mpmath_eval` via std::process::Command; user-controlled env var XCFUN_MPMATH_PYTHON selects interpreter. |
| disk ↔ validation harness | Generated JSONL fixtures + .sha256 stamps land in `validation/fixtures/mpmath/` (full regen) or `target/mpmath_smoke/` (smoke); --check drift gate hashes them. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-06N5-01 | Tampering | XCFUN_MPMATH_PYTHON env var | accept | Build-time-only tooling under xtask/. Operator-controlled. Not part of any library/runtime path. Worst case: regen aborts on import error if interpreter lacks mpmath — fails loudly. No data exfiltration vector (we only pass arguments to a python module that does numerical work). |
| T-06N5-02 | Tampering | mpmath sidecar arithmetic correctness | mitigate | Numerical regression detection via the existing sha256 stamp + `--check` drift gate (driver lines 238-250). Any change to a substrate body (`_ldaerf_eps.py` / `_tpss_eps.py` / `_pw92eps.py`) that drifts the output will trip the stamp diff at next CI run. |
| T-06N5-03 | Information Disclosure | mpmath fixture content | accept | Outputs are functional values at arbitrary density grid points + their derivatives — no PII, no secrets, no business logic. Public domain DFT reference data. |
| T-06N5-04 | Denial of Service | full ~6h regen consuming CI budget | mitigate | The full-regen path is explicitly MANUAL/offline; CI uses `--check` (re-hashes existing fixtures, no python invocations) or `--smoke` (~seconds). Documented in the module docstring + Plan 06-N2 SUMMARY. |
| T-06N5-05 | Elevation of Privilege | python interpreter shell injection via XCFUN_MPMATH_PYTHON | mitigate | The env var resolves to a string passed directly to `Command::new`. No shell expansion (Command::new spawns the process directly, not via `sh -c`). Standard Rust process API hardening. |
</threat_model>

<verification>
## Phase-level checks (after all 3 tasks complete)

```bash
# 1. None of the 6 stubs raise NotImplementedError:
for fn in ldaerfx ldaerfc ldaerfc_jt tpssc tpsslocc revtpssc; do
  ! grep -q "NotImplementedError" "xtask/mpmath_eval/functionals/$fn.py" || \
    { echo "FAIL: $fn still stubbed"; exit 1; }
done

# 2. Both new substrate modules exist with their expected exports:
grep -q "esrx_ldaerfspin" xtask/mpmath_eval/_ldaerf_eps.py
grep -q "tau_clamp" xtask/mpmath_eval/_tpss_eps.py

# 3. python3.12 literal is gone:
! grep -q "python3.12" xtask/src/bin/regen_mpmath_fixtures.rs

# 4. End-to-end single-point invocation works for all 6 ACC-04 functionals
#    (see Task 3 verify block).

# 5. Pre-existing Plan-06-N2 smoke lane still passes (regression check):
cargo run --release -p xtask --bin regen-mpmath-fixtures -- --smoke
ls target/mpmath_smoke/{brx,tw,pbelocc,blocx,scanx}.jsonl   # all present

# 6. Workspace still builds (sanity — only xtask changed; library crates untouched):
cargo check --workspace
```

## Goal-backward derivation

* Truth #1: "Each of the 6 mpmath stub files imports cleanly and `eval_<name>` returns mp.mpf."
  — Verified by Tasks 1+2 stub-removal grep + the python3 -c smoke import in each task's verify.
* Truth #2: "LDAERF family follows the 4-branch dispatch from ldaerfx.cpp:34-47."
  — Verified by Task 1's `_ldaerf_eps.py::esrx_ldaerfspin` having the 4 branches; the existence
    test is structural via grep on substrate exports.
* Truth #3: "TPSS-C family applies tau_clamped guard before tauwtau composition."
  — Verified by Task 2's grep for `tau_clamp` in each tpssc/tpsslocc/revtpssc port + the
    substrate having the `tau_clamp(tau, gnn, n)` export.
* Truth #4: "Smoke regen exits 0 + each ACC-04 functional produces a JSONL record."
  — Verified by Task 3's full smoke run + 6 single-point invocations.
* Truth #5: "python interpreter dispatch is no longer hard-coded to python3.12."
  — Verified by Task 3's grep `! python3.12` + grep `XCFUN_MPMATH_PYTHON`.
* Truth #6: "Driver works end-to-end via env-var override."
  — Verified implicitly by Task 3 setting `PY="${XCFUN_MPMATH_PYTHON:-python3}"` in the
    smoke loop; if the operator sets `XCFUN_MPMATH_PYTHON=python3.12` it routes there.
</verification>

<success_criteria>
1. **All 6 mpmath stub bodies are filled.** Each `eval_<name>(inputs, vars, mode, order=0)` returns a list of finite mp.mpf values for at least one canonical test input. Specifically: `ldaerfx({a=1.1, b=1.0}, mu=0.4, vars="A_B", order=0)` returns approximately `[-1.553573128702155]` (matches the C++ test_threshold reference at LDAERFX `xcfun-master/src/functionals/ldaerfx.cpp:67-68` to within 1e-7 — relaxed tolerance reflects the documented C++ bracket cancellation; mpmath@200 is the truth).

2. **Two new substrate modules ship.** `xtask/mpmath_eval/_ldaerf_eps.py` provides `esrx_ldaerfspin`, `Qrpa`, `dpol`, `g0f`, `ecorrlr`, `c1`, `c2`, `vwn5_eps_mp`. `xtask/mpmath_eval/_tpss_eps.py` provides `tau_clamp`, `phi_reorganised`, `pbec_eps`, `pbec_eps_polarized`, `pbeloc_eps`, `pbeloc_eps_pola`, `revtpss_pbec_eps`, `revtpss_pbec_eps_polarized`, `revtpss_beta`, `tpssc_C`, `tpsslocc_C`, `revtpssc_C`. Both follow the leading-underscore-at-package-root convention established by `_pw92eps.py` / `_scan_like.py` (preserves W-6 revision-2's "one .py per functional" invariant on `functionals/`).

3. **D-10 tau-clamp guard is wired into all 3 TPSS-C ports.** Each port computes `tau_clamped = tau_clamp(tau, gnn, n) = max(tau, gnn/(8*n))` BEFORE composing `tauwtau{2,3}` ratios, mirroring the Rust kernel's intentional divergence from C++ in the unphysical `tau ≪ tau_w` regime.

4. **Driver works on the operator's primary python3 (3.14) AND on python3.12 via env var.** `cargo run --release -p xtask --bin regen-mpmath-fixtures -- --smoke` exits 0 with the default (= `python3`); `XCFUN_MPMATH_PYTHON=python3.12 cargo run ...` also exits 0. Module-level docstring documents both.

5. **End-to-end unblock for Plan 07-00 Task 0.2.** All 6 ACC-04 functionals (ldaerfx/ldaerfc/ldaerfc_jt/tpssc/tpsslocc/revtpssc) accept a single-point invocation via `python3 -m xtask.mpmath_eval` and return a JSONL record with finite floats. The full ~6h MANUAL regen and the subsequent strict 1e-13 sweep belong to Plan 07-00 / Phase 6 HUMAN-UAT #3 — they are now UNBLOCKED.

6. **No regression in Plan-06-N2 substrate.** The pre-existing `--smoke` lane (5 functionals × 5 records: brx/tw/pbelocc/blocx/scanx) still passes — confirms `_pw92eps.py` and `_scan_like.py` were not touched.

7. **No anyhow surfaces in any library crate.** Driver changes are confined to `xtask/src/bin/regen_mpmath_fixtures.rs` (already an anyhow consumer; xtask is app-boundary per CLAUDE.md). No new dependency added; no library Python deps introduced (the entire mpmath sidecar lives under `xtask/mpmath_eval/` per D-04).

8. **No fast-math, no f32, no `-Cfast-math` introduced.** Pure Python + Rust process spawn changes. CLAUDE.md numerical contract preserved.
</success_criteria>

<output>
After completion, create
`.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N5-SUMMARY.md`
following the template at `.claude/get-shit-done/templates/summary.md`.

Required SUMMARY fields:
- `phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu`
- `plan: N5`
- `subsystem: validation` (mpmath sidecar)
- `tags: [mpmath, ground-truth, ACC-04, gap-closure, ldaerf, tpssc, prec-200]`
- `requires: [{phase: 06-00, provides: "mpmath sidecar boot"}, {phase: 06-N2, provides: "20 mpmath ports + _pw92eps.py + _scan_like.py substrates"}]`
- `provides: ["6 ACC-04 mpmath ports filled", "_ldaerf_eps.py + _tpss_eps.py substrates", "XCFUN_MPMATH_PYTHON env-var override"]`
- `affects: [07-00]` (unblocks Plan 07-00 Task 0.2)
- `requirements-completed: [ACC-04]`
- Document any per-functional tolerance overrides observed during smoke
  invocation (especially for tpssc/revtpssc where Plan 04-10 Path-B
  noted f64 cancellation could bring intrinsic drift above 1e-13 even at
  prec=200 truth — record observed but DO NOT force).

The SUMMARY also documents the verification result of the unblock chain:
- BEFORE: regen aborts on functional #1 (ldaerfx) with NotImplementedError.
- AFTER: regen --smoke exits 0; all 6 ACC-04 functionals respond to single-point invocation.

Plan 07-00 Task 0.2 (the operator's pause point) can now resume via
`/gsd:execute-phase 7 --wave 0` (the orchestrator re-discovers Plan 07-00,
sees the SUMMARY exists but is paused, and continues from Task 0.2).
</output>
