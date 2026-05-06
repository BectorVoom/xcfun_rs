---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
reviewed: 2026-05-07T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - xtask/mpmath_eval/_ldaerf_eps.py
  - xtask/mpmath_eval/_pw92eps.py
  - xtask/mpmath_eval/_tpss_eps.py
  - xtask/mpmath_eval/functionals/ldaerfc.py
  - xtask/mpmath_eval/functionals/ldaerfc_jt.py
  - xtask/mpmath_eval/functionals/ldaerfx.py
  - xtask/mpmath_eval/functionals/revtpssc.py
  - xtask/mpmath_eval/functionals/tpssc.py
  - xtask/mpmath_eval/functionals/tpsslocc.py
  - xtask/src/bin/regen_mpmath_fixtures.rs
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 6 (Plan 06-N5): Code Review Report

**Reviewed:** 2026-05-07
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

The 10 files implement Plan 06-N5: mpmath verbatim ports of the LDAERF family
(ldaerfx, ldaerfc, ldaerfc_jt) and the TPSS-C family (tpssc, tpsslocc, revtpssc),
plus two private substrate modules (`_ldaerf_eps.py`, `_tpss_eps.py`) and an
env-var fix in the regen driver. The work targets the 1e-12 ground-truth
contract via `mp.prec = 200`.

**Numerical/algorithmic identity verified** by line-by-line cross-check against
the C++ reference (`xcfun-master/src/functionals/{ldaerfx,ldaerfc,ldaerfc_jt,
tpssc,tpsslocc,revtpssc}.cpp`, `tpssc_eps.hpp`, `revtpssc_eps.hpp`,
`pbec_eps.hpp`, `vwn.hpp`, `pw92eps.hpp`):

- LDAERFX: 4-branch dispatch (`a < 1e-9 / a < 100 / a < 1e9 / else`), bracket
  coefficients (`-3/8`, `sqrt(pi)*erf(1/(2a))`, `(2a-4a³)*exp(-1/(4a²))`,
  `-3a + 4a³`), and large-a expansion `-1/(96a²)` — all match.
- LDAERFC: Qrpa, dpol, g0f, ecorrlr (coe2..coe5, b06/b08, a1..a4 polynomial,
  `phi³ * Qrpa(...) + sum a_k * mu^k + (b0*mu)^8 * ec` numerator, `(1+(b0*mu)²)⁴`
  denominator) — all match.
- LDAERFC_JT: c1, c2 (with `g0 = f * ((gam+rs)^1.5 + bet) * exp(-a*sqrt(gam+rs))`),
  vwn5_eps (vwn_a/_b/_c/_x/_y/_z/_f, `(2^(1/3)-1)^(-1/2)` prefactor) — match.
- TPSS-C bodies: `eps_pkzb = pbec_eps * (1 + C*tauwtau²) - (1+C)*tauwtau²*epsc_summax`
  identical across tpssc/tpsslocc/revtpssc; outer factor uses `tauwtau³` for
  tpssc/tpsslocc and `tauwtau²` for revtpssc (per `revtpssc_eps.hpp:107`) — match.
- C0 polynomials: `0.53/0.87/0.50/2.26` (tpssc), `0.35/0.87/0.50/2.26` (tpsslocc),
  `0.59/0.9269/0.6225/2.1540` (revtpssc) — match.
- DD constants: `2.8` (tpssc, revtpssc) vs `4.5` (tpsslocc) — match.
- D-10 tau-clamp guard correctly applied **before** `tauwtau{2,3}` composition
  in all three TPSS-C ports (tpssc.py:48, tpsslocc.py:45, revtpssc.py:50).
- D-10 clamp body: `max(tau, gnn/(8n))` implemented via `tau if tau > tau_w else
  tau_w` — equivalent to Python's built-in `max()` (correct: mpmath has no
  `fmax`; the in-code comments correctly note `mp.fmax` does not exist).
- PW92 substrate: 3-row parameter table (`0.03109070/0.21370/7.59570/3.5876/...`)
  matches `pw92eps.hpp:42-44` exactly.

The remaining findings are quality and correctness concerns that do not
invalidate the algebraic ground-truth claim, but warrant attention before the
fixtures are regenerated and committed as load-bearing references.

## Warnings

### WR-01: Stratification mis-bins LAPA/LAPB slots in 9-slot TPSS-C layout

**File:** `xtask/src/bin/regen_mpmath_fixtures.rs:115-136`
**Issue:** `vars_for("tpssc"|"tpsslocc"|"revtpssc")` returns the 9-slot layout
`A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB`. In `density_grid`, the stratification
branch at lines 126-128 applies only when `!has_jp && slot_idx < 7`:

```rust
} else if !has_jp && slot_idx < 7 {
    // metaGGA-kinetic-only layouts: slots 5,6 = taua, taub.
    0.01 + 0.49 * next_unit(&mut rng)
}
```

For the 9-slot TPSS-C layout `has_jp == false` and slots 5,6 are LAPA, LAPB
(not taua, taub as the comment claims). Laplacians can be physically negative;
the stratification samples them in the kinetic-positive range `[0.01, 0.5]`.
While the TPSS-C functional bodies do not actually read `lapa`/`lapb` (they
read `taua`/`taub` from slots 7,8), the comment is wrong, the slot kind
inference is wrong for 9-slot layouts without JP, and the same code is reused
for any future 9-slot non-JP variants where laplacian sign matters (e.g., a
hypothetical metaGGA that consumes laplacians).

**Fix:** Make the slot-kind inference depend on `vars_str` rather than
`has_jp`:

```rust
let layout_has_lap = vars_str.contains("LAPA");
let layout_has_tau = vars_str.contains("TAUA");
// ... per-slot dispatch keyed by the layout, not has_jp ...
```

Or, more simply, parse the slot list once via `slot_names(vars_str)` and
dispatch on the slot name (`a`/`b`/`gaa`/`lapa`/`taua`/`jpaa`).

### WR-02: `next_unit` can return 1.0 exactly, violating its [0, 1) contract

**File:** `xtask/src/bin/regen_mpmath_fixtures.rs:111-114`
**Issue:** The closure converts `u64::MAX` (`18446744073709551615`) to `f64`,
which rounds **up** to `1.8446744073709552e19`. The same `u64::MAX` is the
divisor cast similarly. When `rng.next_u64()` returns `u64::MAX`, the result
is `1.0` exactly — but the doc comment says "Convert two u64 -> f64 in [0, 1)"
(the half-open interval). Likewise the `as f64` cast loses 11 mantissa bits,
so values near the top of the range are quantized in steps of ~2048.

```rust
let next_unit = |rng: &mut Xoshiro256PlusPlus| -> f64 {
    let raw = rng.next_u64();
    (raw as f64) / (u64::MAX as f64)
};
```

For the input grid this means a 1-in-2^53 chance per draw of exactly hitting
the upper boundary (e.g., `a = 1.0`), which is fine for fixtures but
contradicts the stated half-open invariant.

**Fix:** Take the upper 53 bits and divide by `2^53` (giving the half-open
[0, 1) interval that f64 can represent exactly), or use the standard
construction:

```rust
let next_unit = |rng: &mut Xoshiro256PlusPlus| -> f64 {
    // Take top 53 bits → exact f64 in [0, 1).
    ((rng.next_u64() >> 11) as f64) * (1.0 / (1u64 << 53) as f64)
};
```

### WR-03: D-10 clamp boundary is non-differentiable but ground truth is taken via `mp.diff` finite differences

**File:** `xtask/mpmath_eval/_tpss_eps.py:57-72`, `xtask/mpmath_eval/functionals/{tpssc,tpsslocc,revtpssc}.py` (all use `tau_clamp` then call `multivariate_taylor`)
**Issue:** `tau_clamp(tau, gnn, n)` is `max(tau, gnn/(8n))` — piecewise linear
with a kink at `tau == tau_w`. `multivariate_taylor` (in `ad_chain.py`) takes
partial derivatives via `mp.diff(f, pt, n=mi)`, which uses Richardson
extrapolation / finite differences around `pt`. At points on or near the
clamp boundary, the finite-difference stencil straddles the kink, and the
returned derivative values are not the limiting value from either side —
they are a stencil-dependent average.

The Rust kernel uses the smooth `ctaylor_max(d.tau, tau_w)` with finite-but-
nonzero Taylor coefficients on both sides; the mpmath ground truth at the
boundary will not match either Rust side cleanly, and at points strictly
inside the clamp regime (`tau ≪ tau_w`), partial derivatives w.r.t. `tau`
are exactly zero analytically — `mp.diff` may produce a tiny non-zero value
from rounding.

For the stratified grid (`tau ∈ [0.01, 0.5]`, `gnn/(8n)` reaches ~2.5), most
sampled points are deep inside the clamped regime, where the analytical
derivative w.r.t. tau is exactly 0. Practical impact: derivative slots in the
fixture for ∂/∂tau will be ~0 with stencil noise rather than exactly 0;
the validation harness must tolerate this when comparing against a Rust
implementation that produces identically-zero derivatives there.

**Fix:** Document the expected behaviour, and either (a) post-process the
fixture rows where `tau < tau_w` to set `∂/∂tau` derivatives to exact 0, or
(b) extend `multivariate_taylor` to accept a piecewise-defined function and
take one-sided derivatives on the active side of the clamp. Option (a) is
the cheaper path and matches what the Rust kernel will produce (a `select`
that zeros the gradient on the clamped branch is the natural ctaylor_max
behaviour). At minimum, add an assertion that Plan 06-N5 fixture rows whose
input `tau` is below `gnn/(8*n)` are flagged as "clamped" so downstream
parity checks don't compare derivative noise against analytic zero.

## Info

### IN-01: VWN5 prefactor diverges from C++ literal at the 14th significant digit

**File:** `xtask/mpmath_eval/_ldaerf_eps.py:325-327`
**Issue:** C++ `vwn.hpp:71` uses the f64 literal `1.92366105093154` for
`(2^(1/3) - 1)^(-1/2)`. The mpmath port computes the value at prec=200:

```python
_VWN_G_PREF = mp.power(mp.power(2, mp.mpf(1) / mp.mpf(3)) - 1, mp.mpf("-0.5"))
```

The exact value is `1.9236610509315367...`; the C++ literal `1.92366105093154`
is rounded to ~14 digits. Relative discrepancy ≈ 3e-15, well inside the 1e-12
contract, but this means the mpmath ground truth is slightly *more* accurate
than C++ for vwn5_eps and consequently for ldaerfc_jt.

The module docstring at `_ldaerf_eps.py:325-326` documents this choice. Flag
only as a heads-up: any 1e-12 discrepancy between mpmath and Rust f64 paths
that traces to `_VWN_G_PREF` is the C++-literal effect, not a bug in either
implementation.

**Fix:** No change required if the project accepts the documented behaviour.
Alternative: define `_VWN_G_PREF = mp.mpf("1.92366105093154")` to bit-track
C++ — but this would make mpmath the *less* accurate side, which inverts the
"mpmath@200 is the truth" project invariant. Recommend leaving as-is and
documenting the 3e-15 differential in `06-N5-SUMMARY.md`.

### IN-02: PBE-correlation `pbec_H` uses `exp(-x) - 1` instead of `expm1(-x)`

**File:** `xtask/mpmath_eval/_tpss_eps.py:101-109` (`pbec_A`), and
`_pbeloc_H` lines 165, `_revtpss_H` lines 236
**Issue:** C++ `pbec_eps.hpp:26` uses `expm1(-eps / (param_gamma * u3))` for
cancellation safety; the mpmath ports use `mp.exp(-eps/(param_gamma*u3)) - 1`.
At prec=200 the cancellation is harmless (the comment at lines 105-106 says
so), but the choice is inconsistent with what a future code-review of the
algorithm-identity claim would flag.

For the typical eps values (~−0.1 Hartree) and `param_gamma * u3 ~ 0.04`,
the argument to exp is large negative (~−2 to −10), making `exp(-x)` close to
zero and `exp(-x) - 1` close to −1 — no significant cancellation.

**Fix:** Replace `mp.exp(-x) - 1` with `mp.expm1(-x)` in `pbec_A`, `_pbeloc_H`,
`_revtpss_H`. mpmath provides `mp.expm1`. This is a clarity-and-parity fix
only; numerical impact is well below 1e-30.

### IN-03: Repeated `dpol(rs * pow(2/(1±z), 1/3))` calls in `ecorrlr` not factored

**File:** `xtask/mpmath_eval/_ldaerf_eps.py:174-202` (coe4 and coe5)
**Issue:** `coe4` and `coe5` both compute the same `dpol((1+z)/...)` and
`dpol((1-z)/...)` calls — four mpmath `dpol` invocations where two would
suffice. Each call performs a `mp.power(2/(1±z), 1/3)` and a `dpol` body
(itself doing `mp.power(rs, ...)` and several mpmath multiplies). At prec=200
each is non-trivial.

**Fix:** Factor out before the coe4/coe5 expressions:

```python
dpol_plus = dpol(r_s * mp.power(2 / (1 + z), mp.mpf(1) / mp.mpf(3)))
dpol_minus = dpol(r_s * mp.power(2 / (1 - z), mp.mpf(1) / mp.mpf(3)))
# ... use dpol_plus and dpol_minus in both coe4 and coe5 ...
```

Performance only; numerical result identical (mpmath is deterministic).
Performance issues are formally out of v1 review scope, but this also
improves readability.

### IN-04: `g0f(d.r_s)` called twice in `ecorrlr`

**File:** `xtask/mpmath_eval/_ldaerf_eps.py:170, 172`
**Issue:** `coe2` and `coe3` both call `g0f(r_s)` independently. Each call
evaluates a 4th-degree polynomial in r_s and an `mp.exp`. Same algorithmic
duplication as IN-03; same fix (precompute `g0_rs = g0f(r_s)` once).

**Fix:** Hoist:

```python
g0_rs = g0f(r_s)
coe2 = mp.mpf("-0.375") / rs3 * (1 - z2) * (g0_rs - mp.mpf("0.5"))
coe3 = -(1 - z2) * g0_rs / (mp.sqrt(2 * PI) * rs3)
```

### IN-05: 9-slot fixture-vars layout for tpssc/tpsslocc/revtpssc differs from C++ functional signature

**File:** `xtask/src/bin/regen_mpmath_fixtures.rs:80-82`
**Issue:** `vars_for("tpssc"|"tpsslocc"|"revtpssc")` returns
`A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB` (9 slots). The C++ functional bodies at
`tpssc.cpp:33`, `tpsslocc.cpp:104`, `revtpssc.cpp:33` declare
`XC_A_B_GAA_GAB_GBB_TAUA_TAUB` (7 slots). The TPSS-C bodies do not read
laplacian slots, so the energy and tau/density-derivative outputs of the
9-slot fixture will match those of the 7-slot C++ functional, but the fixture
will additionally contain ∂/∂lapa, ∂/∂lapb derivatives (= 0 analytically) that
have no C++ counterpart in the 7-slot output vector.

The validation harness must therefore know which slots to *skip* when
comparing tpssc fixtures against C++ tpssc output, or call C++ via a 9-slot
adapter. This appears to be an intentional Plan 06-N5 architecture choice
(see `densvars.py:VARS_SLOTS` enumeration), but is worth a one-line comment
in `vars_for` to flag the divergence and point to the harness side that
handles it.

**Fix:** Add a comment near `vars_for` lines 80-82:

```rust
// NOTE: TPSS-C uses 9-slot layout in the mpmath fixture even though the
// C++ functional declares the 7-slot XC_A_B_GAA_GAB_GBB_TAUA_TAUB layout.
// The two extra slots (LAPA, LAPB) are ignored by the functional body;
// validation harness strips ∂/∂lapa, ∂/∂lapb slots before parity check.
```

---

_Reviewed: 2026-05-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
