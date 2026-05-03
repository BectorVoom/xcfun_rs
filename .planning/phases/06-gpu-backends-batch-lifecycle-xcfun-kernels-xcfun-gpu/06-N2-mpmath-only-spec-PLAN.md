---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N2
type: execute
wave: 9
depends_on:
  - 06-00
  - 06-01
files_modified:
  - xtask/mpmath_eval/functionals/__init__.py
  - xtask/mpmath_eval/functionals/brx.py
  - xtask/mpmath_eval/functionals/brc.py
  - xtask/mpmath_eval/functionals/brxc.py
  - xtask/mpmath_eval/functionals/csc.py
  - xtask/mpmath_eval/functionals/blocx.py
  - xtask/mpmath_eval/functionals/scanx.py
  - xtask/mpmath_eval/functionals/scanc.py
  - xtask/mpmath_eval/functionals/rscanx.py
  - xtask/mpmath_eval/functionals/rscanc.py
  - xtask/mpmath_eval/functionals/rppscanx.py
  - xtask/mpmath_eval/functionals/rppscanc.py
  - xtask/mpmath_eval/functionals/r2scanx.py
  - xtask/mpmath_eval/functionals/r2scanc.py
  - xtask/mpmath_eval/functionals/r4scanx.py
  - xtask/mpmath_eval/functionals/r4scanc.py
  - xtask/mpmath_eval/functionals/tw.py
  - xtask/mpmath_eval/functionals/vwk.py
  - xtask/mpmath_eval/functionals/pbelocc.py
  - xtask/mpmath_eval/functionals/zvpbesolc.py
  - xtask/mpmath_eval/functionals/zvpbeintc.py
  - xtask/mpmath_eval/ad_chain.py
  - xtask/mpmath_eval/densvars.py
  - validation/fixtures/mpmath/brx.jsonl
  - validation/fixtures/mpmath/brc.jsonl
  - validation/fixtures/mpmath/brxc.jsonl
  - validation/fixtures/mpmath/csc.jsonl
  - validation/fixtures/mpmath/blocx.jsonl
  - validation/fixtures/mpmath/scanx.jsonl
  - validation/fixtures/mpmath/scanc.jsonl
  - validation/fixtures/mpmath/rscanx.jsonl
  - validation/fixtures/mpmath/rscanc.jsonl
  - validation/fixtures/mpmath/rppscanx.jsonl
  - validation/fixtures/mpmath/rppscanc.jsonl
  - validation/fixtures/mpmath/r2scanx.jsonl
  - validation/fixtures/mpmath/r2scanc.jsonl
  - validation/fixtures/mpmath/r4scanx.jsonl
  - validation/fixtures/mpmath/r4scanc.jsonl
  - validation/fixtures/mpmath/tw.jsonl
  - validation/fixtures/mpmath/vwk.jsonl
  - validation/fixtures/mpmath/pbelocc.jsonl
  - validation/fixtures/mpmath/zvpbesolc.jsonl
  - validation/fixtures/mpmath/zvpbeintc.jsonl
  - validation/src/driver.rs
  - validation/src/main.rs
  - xtask/src/bin/regen_mpmath_fixtures.rs
autonomous: false
requirements:
  - ACC-04
must_haves:
  truths:
    - "mpmath ports of all 20 excluded_by_upstream_spec functionals (BR×3 + CSC + BLOCX + SCAN×10 + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC) populated in xtask/mpmath_eval/functionals/* — replacing the NotImplementedError stubs from Plan 06-00."
    - "(W-6 revision-2) Task 1 split into FAMILY sub-tasks — Task 1a (BR family — `brx`, `brc`, `brxc`), Task 1b (SCAN family — `scanx`, `scanc`, `rscanx`, `rscanc`, `rppscanx`, `rppscanc`, `r2scanx`, `r2scanc`, `r4scanx`, `r4scanc` — one .py module per functional, NOT one shared scan.py), Task 1c (kinetic-GGA family — `tw`, `vwk`), Task 1d (PBE-correlation variants + miscellaneous — `csc`, `blocx`, `pbelocc`, `zvpbesolc`, `zvpbeintc`). Each sub-task has its own RED→GREEN per-family smoke (~5-functional × 5-record `python3 -c \"from xtask.mpmath_eval.functionals.<name> import eval_<name>; ...\"` `<verify>`) running in single-digit seconds BEFORE Task 2 commits to full ~6-hour fixture regeneration."
    - "(W-5 revision-1) Task 2 split into THREE autonomous-friendly steps: (a) wiring + 5-functional × 5-record SMOKE generation [autonomous]; (b) full ~6h regeneration command DOCUMENTED for offline manual execution + commit fixtures separately [non-autonomous, MANUAL]; (c) `cargo xtask regen-mpmath-fixtures --check` drift gate runs in CI in single-digit seconds [autonomous]. Plan-level `autonomous: false` reflects step (b)."
    - "(W-11 revision-1) Per-functional record count REDUCED to ~30 (5 strata × 6 records) to keep individual JSONL files under ~40KB. Total: 20 × 30 = ~600 records (was ~2000). If reviewer prefers smaller count for code-review tractability, fixtures can be split into a follow-up data-only PR."
    - "(B-3 revision-2) The xtask/mpmath_eval/functionals/ Python package directory ALREADY EXISTS (created by Plan 06-00 Task 4 with per-functional NotImplementedError stub modules — see 06-00-PLAN.md Step A); this plan ONLY ADDS new module files into the existing package and ONLY REPLACES stub bodies in the 6 ACC-04-amended modules already shipped by Plan 06-00. NO mid-execution restructure (no `mv functionals.py functionals/`, no `mkdir xtask/mpmath_eval/functionals`); xtask/mpmath_eval/functionals.py never exists as a single file in this branch."
    - "JSONL fixture files committed at validation/fixtures/mpmath/<functional>.jsonl (~30 stratified records per functional × 20 = ~600 records total per W-11); each file is content-hash stamped via .sha256 (drift gate per Phase 2 D-21 pattern)."
    - "validation harness `--reference mpmath` flag dispatches per-record source: when functional is in the excluded_by_upstream_spec list, reference defaults to mpmath; otherwise C++ remains the default."
    - "Order-3 tier-2 sweep with `--reference mpmath` on the 20-functional set GREEN at strict 1e-13 (mpmath truth at prec=200 vs Rust output)."
    - "Phase 2 driver skip-list at `validation/src/driver.rs` for these 20 functionals REMOVED (or marked excluded_by_upstream_spec_RESOLVED)."
    - "ACC-04 amendment per D-03 explicitly applies here: C++ harness aborts on these density strata via `tmath::sqrt_expand`/`log_expand`/`pow_expand` — mpmath at prec=200 is the SOLE ground truth; algorithmic-identity contract preserved (Rust matches mpmath)."
  artifacts:
    - path: "xtask/mpmath_eval/functionals/scanx.py"
      provides: "mpmath port of SCAN-X exchange (one of 10 SCAN-family modules; one .py per functional per W-6)"
      contains: "eval_scanx"
    - path: "xtask/mpmath_eval/functionals/brx.py"
      provides: "mpmath port of BR-X (Becke-Roussel exchange) at prec=200"
      contains: "eval_brx"
    - path: "validation/fixtures/mpmath/scanx.jsonl"
      provides: "~30 mpmath truth records for SCANX at strict 1e-13"
      contains: "mpmath_prec.*200"
    - path: "validation/src/driver.rs"
      provides: "--reference mpmath dispatch + per-record source annotation in report.html"
      contains: "Reference::Mpmath\\|--reference"
  key_links:
    - from: "xtask/src/bin/regen_mpmath_fixtures.rs (Plan 06-00)"
      to: "xtask/mpmath_eval/functionals/<name>.py"
      via: "subprocess invocation per density record"
      pattern: "python3 -m xtask.mpmath_eval"
    - from: "validation/src/driver.rs::run_tier2"
      to: "validation/fixtures/mpmath/<functional>.jsonl"
      via: "fixture-loader path on --reference mpmath"
      pattern: "validation/fixtures/mpmath"
---

<objective>
Validate the **20 `excluded_by_upstream_spec` functionals** at strict 1e-13 using mpmath at 200-digit precision as the SOLE ground truth (per CONTEXT.md D-03 ACC-04 amendment + RESEARCH §"D-19 Bisection Methodology" Plan 06-N2).

Set:
- **BR family ×3** — BRX, BRC, BRXC (Phase 4 Plan 04-01; tier-1 GREEN, tier-2 marked `excluded_by_upstream_spec`)
- **CSC** (Plan 04-01)
- **BLOCX** (Plan 04-03; tier-1 GREEN)
- **SCAN family ×10** — SCANX, SCANC, RSCANX, RSCANC, RPPSCANX, RPPSCANC, R2SCANX, R2SCANC, R4SCANX, R4SCANC (Plans 04-02 + 04-09; commit `f968c32` skip-list extension)
- **TW** + **VWK** (Phase 2 Plan 02-05; no upstream test_in)
- **PBELOCC** + **ZVPBESOLC** + **ZVPBEINTC** (Phase 3 Plan 03-02; C++ pow_expand at zero aborts)

Why mpmath-only: C++ reference harness (`tmath::sqrt_expand` / `log_expand` / `pow_expand`) ABORTS on these density strata — see `xcfun-master/external/upstream/taylor/tmath.hpp:156` per `.planning/REQUIREMENTS.md` GGA-01 caveat. The C++ binary cannot be linked against `validation/` for these functionals' tier-2 records. mpmath at prec=200 (Plan 06-00 sidecar) is the sole reference per D-03 / D-04.

Per RESEARCH §"D-19 Bisection Methodology" Plan 06-N2 Step 3: cost is ~6 hours one-time mpmath fixture generation (~30 records per functional × 20 = ~600 evaluations × ~10s each at prec=200; W-11 reduction from original 2000). Run once; commit; replay forever via `cargo xtask regen-mpmath-fixtures --check`.

Three concrete deliverables:
1. **Populate `xtask/mpmath_eval/functionals/<name>.py`** — port each of the 20 functionals into mpmath. Plan 06-00 Task 4 already created `xtask/mpmath_eval/functionals/` AS A PYTHON PACKAGE DIRECTORY containing per-functional `NotImplementedError` stub modules (see 06-00 Task 4 Step A — the package layout has been in place since the very first Phase 6 commit). This plan ONLY replaces stub bodies with full mpmath implementations and adds NEW per-functional module files for the 20-set. NO `mv`, NO `mkdir`, NO restructure. **(W-6 revision-2)** Population is split into 4 family sub-tasks (Task 1a–1d), each with its own per-family RED→GREEN smoke that runs in single-digit seconds and surfaces transcription bugs uniformly across the family BEFORE Task 2's slow ~6h fixture regen.
2. **Generate + commit JSONL fixtures** — run `cargo xtask regen-mpmath-fixtures` to emit `validation/fixtures/mpmath/<functional>.jsonl` files (one per functional with ~30 stratified records covering bulk + regularize + polarised + gradient-stress per Phase 2 D-18 pattern; xoshiro seed `0x1234abcd` for determinism; ~15 + 15 hand-curated boundary records per RESEARCH §"Open Question 2"). Commit each `.jsonl` + `.sha256` stamp.
3. **Wire validation harness `--reference mpmath`** — `validation/src/driver.rs` reads JSONL fixtures from `validation/fixtures/mpmath/`; for functionals in the 20-set, the reference defaults to mpmath; per-record source annotated in `report.html`. Remove the Phase 2 `excluded_by_upstream_spec` skip-list entries for these 20 functionals (they're now covered).

Purpose: Satisfy the ROADMAP Phase 6 success criterion 1 (every functional has a `#[cube]` body that compiles unchanged for any cubecl Runtime) AND the implicit Phase 6 sign-off requirement that all 78 functionals tier-2 GREEN. The 20 set are part of the 78-functional contract per ROADMAP — Plan 06-N2 closes them via mpmath-only validation.

**Parallel-safe with 06-N1 + 06-N3** — touches independent functional sets (per CONTEXT.md D-01: "parallel is fine because they touch independent functional sets").

Output: 20 mpmath ports populated; 20 JSONL fixture files (~600 records total at W-11 cap); validation harness `--reference mpmath` flag; tier-2 with `--reference mpmath` GREEN at strict 1e-13 across the 20-functional set.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-00-substrate-PLAN.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@xtask/mpmath_eval/__main__.py
@xtask/mpmath_eval/evaluator.py
@xtask/mpmath_eval/functionals/__init__.py
@xtask/src/bin/regen_mpmath_fixtures.rs
@validation/src/driver.rs
@validation/src/main.rs
@xcfun-master/src/functionals/brx.cpp
@xcfun-master/src/functionals/csc.cpp
@xcfun-master/src/functionals/blocx.cpp
@xcfun-master/src/functionals/scanx.cpp
@xcfun-master/src/functionals/tw.cpp
@xcfun-master/src/functionals/vwk.cpp
@xcfun-master/src/functionals/pbelocc.cpp
</context>

<tasks>

<task type="auto">
  <name>Task 1a: BR family mpmath ports (brx, brc, brxc) + per-family RED→GREEN smoke</name>
  <files>xtask/mpmath_eval/functionals/__init__.py, xtask/mpmath_eval/functionals/brx.py, xtask/mpmath_eval/functionals/brc.py, xtask/mpmath_eval/functionals/brxc.py, xtask/mpmath_eval/ad_chain.py, xtask/mpmath_eval/densvars.py</files>
  <read_first>
    - xtask/mpmath_eval/__main__.py (Plan 06-00 — entry point invoked by Rust driver)
    - xtask/mpmath_eval/evaluator.py (Plan 06-00 — eval_record + LOOKUP dispatch)
    - xtask/mpmath_eval/functionals/__init__.py (Plan 06-00 — package layout already in place)
    - xtask/mpmath_eval/functionals/ldaerfx.py (Plan 06-00 — per-functional stub module shape to mirror)
    - xcfun-master/src/functionals/brx.cpp (full file — BR-exchange port reference; uses Newton inversion)
    - xcfun-master/src/functionals/brc.cpp (full file — BR-correlation port reference)
    - xcfun-master/src/functionals/brxc.cpp (full file — BR-XC composite)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-N2" + "mpmath_eval" (lines 102-115, 297-313)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"mpmath Sidecar Architecture" (lines 812-907)
  </read_first>
  <action>
**Step A — Confirm starting state (NO restructure; Plan 06-00 already shipped the package layout):**

Plan 06-00 Task 4 already created `xtask/mpmath_eval/functionals/` as a Python package directory containing per-functional `NotImplementedError` stub modules (see 06-00-PLAN.md Step A: "B-3 revision-1 — `xtask/mpmath_eval/functionals/` package directory (NOT a single `functionals.py` file)"). This task does NOT run `mv` or `mkdir`. Verify the starting state:

```bash
test -d xtask/mpmath_eval/functionals
test -f xtask/mpmath_eval/functionals/__init__.py
test -f xtask/mpmath_eval/functionals/ldaerfx.py   # Plan 06-00 stub
ls xtask/mpmath_eval/functionals.py                # MUST NOT exist (no single .py file)
```

**Step B — Add (or fill) the supporting modules at `xtask/mpmath_eval/`:**

Plan 06-00 Task 4 created `xtask/mpmath_eval/ad_chain.py` and `xtask/mpmath_eval/densvars.py` as `NotImplementedError` stubs. Replace stub bodies with full implementations:

```python
# xtask/mpmath_eval/ad_chain.py
"""Generic Taylor-series AD chain at mp.prec=200.

Provides a Taylor coefficient extractor for arbitrary scalar f: mp.mpf → mp.mpf
across orders 0..6. Uses mpmath.diff at prec=200; the result is exact to ~200
digits for analytic functions.
"""
import mpmath as mp

def taylor_coeffs(f, x0, order, prec=200):
    """Compute Taylor coefficients [f(x0), f'(x0), f''(x0)/2!, ..., f^(order)(x0)/order!].

    Each coefficient = f^(k)(x0) / k! per the multinomial-Taylor convention.
    """
    mp.mp.prec = prec
    out = []
    for k in range(order + 1):
        deriv = mp.diff(f, x0, n=k)
        out.append(deriv / mp.factorial(k))
    return out
```

```python
# xtask/mpmath_eval/densvars.py
"""DensVars equivalent at prec=200.

Mirrors xcfun-eval::DensVarsDev<F> field-order:
  n     = a + b
  s     = (a - b) / (a + b)        [zeta]
  gnn   = gaa + 2*gab + gbb
  gns   = (gaa - gbb) / n
  gss   = (gaa - 2*gab + gbb) / n^2
  taua, taub, lapla, laplb         [if Vars supplies them]
"""
import mpmath as mp

def build_densvars(inputs, vars):
    """Returns dict{n, s, gnn, gns, gss, ...} of mp.mpf based on Vars layout."""
    a, b = inputs[0], inputs[1]
    n = a + b
    s = (a - b) / n
    out = {'n': n, 's': s, 'a': a, 'b': b}
    if 'gaa' in vars or 'gab' in vars or 'gbb' in vars or vars in ['a_b_gaa_gab_gbb', 'A_B_GAA_GAB_GBB']:
        gaa, gab, gbb = inputs[2], inputs[3], inputs[4]
        out['gnn'] = gaa + 2*gab + gbb
        out['gns'] = (gaa - gbb) / n
        out['gss'] = (gaa - 2*gab + gbb) / (n * n)
        out['gaa'] = gaa
        out['gab'] = gab
        out['gbb'] = gbb
    # ... extend for tau, laplacian as needed for SCAN/TPSS/BR ...
    return out
```

**Step C — Create the BR family per-functional modules** (3 new files in the existing package):

Plan 06-00 only shipped stub modules for the 6 ACC-04-amended set (`ldaerfx.py`, `ldaerfc.py`, `ldaerfc_jt.py`, `tpssc.py`, `tpsslocc.py`, `revtpssc.py`). For the 20-set this plan adds NEW per-functional module files into the same already-existing package — one `.py` per functional, mirroring the 06-00 stub shape. The BR family adds three:

```python
# xtask/mpmath_eval/functionals/brx.py
"""BR-X (Becke-Roussel exchange) mpmath port per xcfun-master/src/functionals/brx.cpp.

C++ harness aborts on this via tmath::sqrt_expand at low-density tail
(metaGGA-class deps: KINETIC|LAPLACIAN|JP). BR-X uses Newton inversion
internally — implemented here at prec=200 for ground truth.
"""
import mpmath as mp
from ..ad_chain import taylor_coeffs
from ..densvars import build_densvars

def eval_brx(inputs, vars, mode, order):
    """Port of xcfun-master/src/functionals/brx.cpp at prec=200."""
    mp.mp.prec = 200
    d = build_densvars(inputs, vars)
    # ... port the BR-X body verbatim from brx.cpp ...
    # ... including Newton inversion at prec=200 ...
    # Return the order+1-length Taylor expansion as a List[mp.mpf].
    raise NotImplementedError("Port BR-X body from brx.cpp; uses Newton inversion")
```

Repeat the same shape for `brc.py` (eval_brc — BR-correlation body) and `brxc.py` (eval_brxc — BR-XC composite). Each file is ~50–150 LOC after porting the C++ body verbatim.

**Step D — Update `xtask/mpmath_eval/functionals/__init__.py` to register the 3 new BR entries:**

The Plan 06-00-shipped `__init__.py` currently registers the 6 ACC-04-amended modules. ADD the BR family imports + LOOKUP entries WITHOUT removing the existing 6:

```python
# xtask/mpmath_eval/functionals/__init__.py (extended; Plan 06-00 lines preserved, BR family appended)
"""mpmath ports of ACC-04-amended functionals (Plan 06-00) + excluded_by_upstream_spec
functionals (Plan 06-N2; populated incrementally by Tasks 1a–1d).

Each per-functional module exposes `eval_<name>(inputs, vars, mode, order)`.
"""
# Plan 06-00 ACC-04 stubs (kept):
from . import ldaerfx, ldaerfc, ldaerfc_jt, tpssc, tpsslocc, revtpssc
# Plan 06-N2 Task 1a (BR family):
from . import brx, brc, brxc
# Task 1b–1d will append: scanx, scanc, ..., r4scanc, tw, vwk, csc, blocx,
# pbelocc, zvpbesolc, zvpbeintc.

LOOKUP = {
    # Plan 06-00 ACC-04 set:
    'ldaerfx': ldaerfx.eval_ldaerfx,
    'ldaerfc': ldaerfc.eval_ldaerfc,
    'ldaerfc_jt': ldaerfc_jt.eval_ldaerfc_jt,
    'tpssc': tpssc.eval_tpssc,
    'tpsslocc': tpsslocc.eval_tpsslocc,
    'revtpssc': revtpssc.eval_revtpssc,
    # Plan 06-N2 Task 1a (BR family):
    'brx': brx.eval_brx,
    'brc': brc.eval_brc,
    'brxc': brxc.eval_brxc,
    # Task 1b–1d will append entries here.
}
```

**Step E — Per-family RED→GREEN smoke (W-6):**

Verify every BR module imports cleanly and the populated body returns a `List[mp.mpf]` of correct length WITHOUT raising `NotImplementedError`. The verify command exercises 3 functionals × ~5 records each in single-digit seconds:

```bash
python3 -c "
from xtask.mpmath_eval.functionals.brx import eval_brx
from xtask.mpmath_eval.functionals.brc import eval_brc
from xtask.mpmath_eval.functionals.brxc import eval_brxc
import mpmath; mpmath.mp.prec = 200
inputs = [mpmath.mpf('0.5'), mpmath.mpf('0.5'),
          mpmath.mpf('0.1'), mpmath.mpf('0.05'), mpmath.mpf('0.1'),
          mpmath.mpf('0.0'), mpmath.mpf('0.0'),     # lapl
          mpmath.mpf('0.05'), mpmath.mpf('0.05')]    # tau
for fn in (eval_brx, eval_brc, eval_brxc):
    out = fn(inputs, 'A_B_GAA_GAB_GBB_LAPLA_LAPLB_TAUA_TAUB', 'partial_derivatives', 2)
    assert len(out) >= 3, fn.__name__
    print(fn.__name__, 'OK', len(out))
"
```

**Forbidden:**
- Do NOT introduce Python deps beyond `mpmath` itself (per D-04).
- Do NOT use `float()` on the inputs internally — every intermediate must stay `mp.mpf` at prec=200.
- Do NOT raise `NotImplementedError` from any populated function in this task's commit — every BR entry MUST be filled.
- Do NOT run `mv` or `mkdir` on `xtask/mpmath_eval/functionals/` (B-3 revision-2 invariant — package directory pre-exists).
  </action>
  <verify>
    <automated>python3 -c "from xtask.mpmath_eval.functionals.brx import eval_brx; from xtask.mpmath_eval.functionals.brc import eval_brc; from xtask.mpmath_eval.functionals.brxc import eval_brxc; import mpmath; mpmath.mp.prec = 200; inputs=[mpmath.mpf('0.5'),mpmath.mpf('0.5'),mpmath.mpf('0.1'),mpmath.mpf('0.05'),mpmath.mpf('0.1'),mpmath.mpf('0.0'),mpmath.mpf('0.0'),mpmath.mpf('0.05'),mpmath.mpf('0.05')]; [print(fn.__name__, 'OK', len(fn(inputs, 'A_B_GAA_GAB_GBB_LAPLA_LAPLB_TAUA_TAUB', 'partial_derivatives', 2))) for fn in (eval_brx, eval_brc, eval_brxc)]"</automated>
  </verify>
  <acceptance_criteria>
    - 3 new BR family modules exist: `test -f xtask/mpmath_eval/functionals/brx.py && test -f xtask/mpmath_eval/functionals/brc.py && test -f xtask/mpmath_eval/functionals/brxc.py`.
    - `xtask/mpmath_eval/functionals/__init__.py` LOOKUP dict has at least 9 entries (6 from 06-00 + 3 from this task): `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 9"`.
    - Per-family smoke (3 functionals × 1 record each) completes in single-digit seconds.
    - **(B-3 revision-2)** `xtask/mpmath_eval/functionals.py` (single-file) does NOT exist: `test ! -e xtask/mpmath_eval/functionals.py`.
    - **(B-3 revision-2)** `xtask/mpmath_eval/functionals/` is a directory: `test -d xtask/mpmath_eval/functionals`.
    - No `mv` / `mkdir` of `xtask/mpmath_eval/functionals/` in this task's diff: `git diff --stat HEAD~1 -- xtask/mpmath_eval/functionals/` shows only ADDED `.py` files (and `__init__.py` extension), no rename.
    - No Python imports beyond `mpmath` in the runtime path: `grep -rE 'import (numpy|scipy|sympy)' xtask/mpmath_eval/functionals/{brx,brc,brxc}.py | wc -l` == 0.
  </acceptance_criteria>
  <done>BR family (brx, brc, brxc) populated at prec=200; LOOKUP extended; per-family smoke GREEN in single-digit seconds; no package restructure (B-3 invariant preserved).</done>
</task>

<task type="auto">
  <name>Task 1b: SCAN family mpmath ports (10 functionals; one .py per functional) + per-family RED→GREEN smoke</name>
  <files>xtask/mpmath_eval/functionals/__init__.py, xtask/mpmath_eval/functionals/scanx.py, xtask/mpmath_eval/functionals/scanc.py, xtask/mpmath_eval/functionals/rscanx.py, xtask/mpmath_eval/functionals/rscanc.py, xtask/mpmath_eval/functionals/rppscanx.py, xtask/mpmath_eval/functionals/rppscanc.py, xtask/mpmath_eval/functionals/r2scanx.py, xtask/mpmath_eval/functionals/r2scanc.py, xtask/mpmath_eval/functionals/r4scanx.py, xtask/mpmath_eval/functionals/r4scanc.py</files>
  <read_first>
    - xtask/mpmath_eval/functionals/__init__.py (Task 1a — extended LOOKUP)
    - xtask/mpmath_eval/functionals/brx.py (Task 1a — module shape + per-file docstring style to mirror)
    - xcfun-master/src/functionals/scanx.cpp (full file — SCAN-X port reference)
    - xcfun-master/src/functionals/scanc.cpp + scan_eps.hpp (correlation companion)
    - xcfun-master/src/functionals/SCAN_like_eps.hpp (shared substrate per Phase 4 commit f968c32)
    - xcfun-master/src/functionals/rscanx.cpp + rscanc.cpp (regularised-SCAN variants)
    - xcfun-master/src/functionals/rppscanx.cpp + rppscanc.cpp (rppSCAN variants)
    - xcfun-master/src/functionals/r2scanx.cpp + r2scanc.cpp (r²SCAN variants)
    - xcfun-master/src/functionals/r4scanx.cpp + r4scanc.cpp (r⁴SCAN variants)
  </read_first>
  <action>
**Step A — Create 10 per-functional SCAN modules** (W-6 revision-2 — one `.py` per functional, NOT one shared `scan.py`):

Per the 06-N2 truths block (W-6 revision-2): "SCAN family (10 functionals split as scanx/scanc/rscanx/rscanc/rppscanx/rppscanc/r2scanx/r2scanc/r4scanx/r4scanc — one .py module per functional, NOT one shared scan.py)". The C++ tree mirrors this layout (`xcfun-master/src/functionals/scanx.cpp`, `scanc.cpp`, etc.) — port one-for-one.

Each new module follows the BR family shape (Task 1a Step C). For example `scanx.py`:

```python
# xtask/mpmath_eval/functionals/scanx.py
"""SCAN-X (SCAN exchange) mpmath port per xcfun-master/src/functionals/scanx.cpp
+ SCAN_like_eps.hpp.

C++ harness aborts on these via tmath::sqrt_expand at low-density tail (17
sqrt() call-sites in the SCAN_like_eps shared substrate). mpmath at prec=200
is the sole reference per D-03 ACC-04 amendment.
"""
import mpmath as mp
from ..ad_chain import taylor_coeffs
from ..densvars import build_densvars

def eval_scanx(inputs, vars, mode, order):
    """Port of xcfun-master/src/functionals/scanx.cpp at prec=200."""
    mp.mp.prec = 200
    d = build_densvars(inputs, vars)
    # ... port the SCAN exchange body (SCAN_like_eps + scanx-specific exchange enhancement) ...
    raise NotImplementedError("Port SCAN-X body from scanx.cpp lines N..M")
```

Repeat for the other 9 SCAN-family functionals (`scanc.py`, `rscanx.py`, `rscanc.py`, `rppscanx.py`, `rppscanc.py`, `r2scanx.py`, `r2scanc.py`, `r4scanx.py`, `r4scanc.py`) — each module ports its corresponding `xcfun-master/src/functionals/<name>.cpp` body at prec=200. The `SCAN_like_eps` substrate is shared between SCAN-correlation variants; factor it into a private helper inside each correlation module rather than introducing a separate `scan_like_eps.py` (keeping one-file-per-functional invariant).

**Step B — Update `xtask/mpmath_eval/functionals/__init__.py` LOOKUP** (append 10 SCAN-family entries to the dict; preserve Plan 06-00 + Task 1a entries):

```python
# Append after the BR family imports/entries:
from . import (scanx, scanc, rscanx, rscanc, rppscanx, rppscanc,
               r2scanx, r2scanc, r4scanx, r4scanc)

LOOKUP.update({
    'scanx':    scanx.eval_scanx,
    'scanc':    scanc.eval_scanc,
    'rscanx':   rscanx.eval_rscanx,
    'rscanc':   rscanc.eval_rscanc,
    'rppscanx': rppscanx.eval_rppscanx,
    'rppscanc': rppscanc.eval_rppscanc,
    'r2scanx':  r2scanx.eval_r2scanx,
    'r2scanc':  r2scanc.eval_r2scanc,
    'r4scanx':  r4scanx.eval_r4scanx,
    'r4scanc':  r4scanc.eval_r4scanc,
})
```

(If editing the LOOKUP dict literal in-place is cleaner than `.update()`, do that — same end-state.)

**Step C — Per-family RED→GREEN smoke (W-6):**

Verify every SCAN module imports cleanly and the populated body returns a `List[mp.mpf]` of correct length. The smoke exercises 5 representative SCAN-family functionals × ~5 records each in single-digit seconds (full 10-functional round-trip is in Task 2's full sweep):

```bash
python3 -c "
from xtask.mpmath_eval.functionals.scanx   import eval_scanx
from xtask.mpmath_eval.functionals.scanc   import eval_scanc
from xtask.mpmath_eval.functionals.r2scanx import eval_r2scanx
from xtask.mpmath_eval.functionals.r4scanx import eval_r4scanx
from xtask.mpmath_eval.functionals.rppscanc import eval_rppscanc
import mpmath; mpmath.mp.prec = 200
inputs = [mpmath.mpf('0.5'), mpmath.mpf('0.5'),
          mpmath.mpf('0.1'), mpmath.mpf('0.05'), mpmath.mpf('0.1'),
          mpmath.mpf('0.0'), mpmath.mpf('0.0'),
          mpmath.mpf('0.05'), mpmath.mpf('0.05')]
for fn in (eval_scanx, eval_scanc, eval_r2scanx, eval_r4scanx, eval_rppscanc):
    out = fn(inputs, 'A_B_GAA_GAB_GBB_LAPLA_LAPLB_TAUA_TAUB', 'partial_derivatives', 2)
    assert len(out) >= 3, fn.__name__
    print(fn.__name__, 'OK', len(out))
"
```

**Forbidden:**
- Do NOT consolidate the 10 SCAN files into a single `scan.py` (W-6 revision-2 contract: one .py per functional).
- Same package-restructure prohibitions as Task 1a (B-3 revision-2).
  </action>
  <verify>
    <automated>python3 -c "from xtask.mpmath_eval.functionals.scanx import eval_scanx; from xtask.mpmath_eval.functionals.scanc import eval_scanc; from xtask.mpmath_eval.functionals.r2scanx import eval_r2scanx; from xtask.mpmath_eval.functionals.r4scanx import eval_r4scanx; from xtask.mpmath_eval.functionals.rppscanc import eval_rppscanc; import mpmath; mpmath.mp.prec = 200; inputs=[mpmath.mpf('0.5'),mpmath.mpf('0.5'),mpmath.mpf('0.1'),mpmath.mpf('0.05'),mpmath.mpf('0.1'),mpmath.mpf('0.0'),mpmath.mpf('0.0'),mpmath.mpf('0.05'),mpmath.mpf('0.05')]; [print(fn.__name__, 'OK', len(fn(inputs, 'A_B_GAA_GAB_GBB_LAPLA_LAPLB_TAUA_TAUB', 'partial_derivatives', 2))) for fn in (eval_scanx, eval_scanc, eval_r2scanx, eval_r4scanx, eval_rppscanc)]"</automated>
  </verify>
  <acceptance_criteria>
    - All 10 SCAN-family modules exist as separate files: `find xtask/mpmath_eval/functionals -name 'scanx.py' -o -name 'scanc.py' -o -name 'rscanx.py' -o -name 'rscanc.py' -o -name 'rppscanx.py' -o -name 'rppscanc.py' -o -name 'r2scanx.py' -o -name 'r2scanc.py' -o -name 'r4scanx.py' -o -name 'r4scanc.py' | wc -l` == 10.
    - **(W-6 revision-2)** No shared `scan.py` module exists: `test ! -e xtask/mpmath_eval/functionals/scan.py`.
    - LOOKUP dict has at least 19 entries (6 from 06-00 + 3 from Task 1a + 10 from this task): `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 19"`.
    - Per-family smoke (5 representative functionals × 1 record each) completes in single-digit seconds.
    - No `mv` / `mkdir` on `xtask/mpmath_eval/functionals/` (B-3 revision-2 invariant).
  </acceptance_criteria>
  <done>SCAN family (10 functionals) populated at prec=200; one .py per functional (W-6); LOOKUP extended; per-family smoke GREEN in single-digit seconds.</done>
</task>

<task type="auto">
  <name>Task 1c: Kinetic-GGA family mpmath ports (tw, vwk) + per-family RED→GREEN smoke</name>
  <files>xtask/mpmath_eval/functionals/__init__.py, xtask/mpmath_eval/functionals/tw.py, xtask/mpmath_eval/functionals/vwk.py</files>
  <read_first>
    - xtask/mpmath_eval/functionals/__init__.py (after Task 1a + 1b)
    - xtask/mpmath_eval/functionals/brx.py (module shape reference)
    - xcfun-master/src/functionals/tw.cpp (full file — Thomas-Weizsäcker kinetic functional)
    - xcfun-master/src/functionals/vwk.cpp (full file — Vela-Weizsäcker-Kohout kinetic functional)
  </read_first>
  <action>
**Step A — Create 2 per-functional kinetic-GGA modules** (`tw.py`, `vwk.py`) following the BR family shape (Task 1a Step C). Each ports `xcfun-master/src/functionals/tw.cpp` and `vwk.cpp` verbatim at `mp.mp.prec = 200`. These functionals lack upstream `test_in` reference data (Phase 2 Plan 02-05 finding) — mpmath truth at prec=200 is the sole reference.

```python
# xtask/mpmath_eval/functionals/tw.py
"""Thomas-Weizsäcker kinetic-GGA mpmath port per xcfun-master/src/functionals/tw.cpp.

No upstream test_in (Phase 2 Plan 02-05); mpmath at prec=200 is sole reference.
"""
import mpmath as mp
from ..ad_chain import taylor_coeffs
from ..densvars import build_densvars

def eval_tw(inputs, vars, mode, order):
    """Port of xcfun-master/src/functionals/tw.cpp at prec=200."""
    mp.mp.prec = 200
    d = build_densvars(inputs, vars)
    # ... port verbatim ...
    raise NotImplementedError("Port T-W body from tw.cpp")
```

Repeat the same shape for `vwk.py` (eval_vwk).

**Step B — Update LOOKUP** in `xtask/mpmath_eval/functionals/__init__.py` to append `'tw'` and `'vwk'` entries (preserving prior Tasks 1a + 1b entries).

**Step C — Per-family RED→GREEN smoke:**

```bash
python3 -c "
from xtask.mpmath_eval.functionals.tw  import eval_tw
from xtask.mpmath_eval.functionals.vwk import eval_vwk
import mpmath; mpmath.mp.prec = 200
inputs = [mpmath.mpf('0.5'), mpmath.mpf('0.5'),
          mpmath.mpf('0.1'), mpmath.mpf('0.05'), mpmath.mpf('0.1')]
for fn in (eval_tw, eval_vwk):
    out = fn(inputs, 'A_B_GAA_GAB_GBB', 'partial_derivatives', 2)
    assert len(out) >= 3, fn.__name__
    print(fn.__name__, 'OK', len(out))
"
```

**Forbidden:** Same as Task 1a (no package restructure; no non-mpmath imports).
  </action>
  <verify>
    <automated>python3 -c "from xtask.mpmath_eval.functionals.tw import eval_tw; from xtask.mpmath_eval.functionals.vwk import eval_vwk; import mpmath; mpmath.mp.prec = 200; inputs=[mpmath.mpf('0.5'),mpmath.mpf('0.5'),mpmath.mpf('0.1'),mpmath.mpf('0.05'),mpmath.mpf('0.1')]; [print(fn.__name__, 'OK', len(fn(inputs, 'A_B_GAA_GAB_GBB', 'partial_derivatives', 2))) for fn in (eval_tw, eval_vwk)]"</automated>
  </verify>
  <acceptance_criteria>
    - 2 new kinetic-GGA modules exist: `test -f xtask/mpmath_eval/functionals/tw.py && test -f xtask/mpmath_eval/functionals/vwk.py`.
    - LOOKUP dict has at least 21 entries (prior 19 + 2 from this task): `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 21"`.
    - Per-family smoke completes in single-digit seconds.
    - No package restructure (B-3 revision-2 invariant).
  </acceptance_criteria>
  <done>Kinetic-GGA family (tw, vwk) populated at prec=200; LOOKUP extended; per-family smoke GREEN.</done>
</task>

<task type="auto">
  <name>Task 1d: PBE-correlation variants + miscellaneous (csc, blocx, pbelocc, zvpbesolc, zvpbeintc) + per-family RED→GREEN smoke</name>
  <files>xtask/mpmath_eval/functionals/__init__.py, xtask/mpmath_eval/functionals/csc.py, xtask/mpmath_eval/functionals/blocx.py, xtask/mpmath_eval/functionals/pbelocc.py, xtask/mpmath_eval/functionals/zvpbesolc.py, xtask/mpmath_eval/functionals/zvpbeintc.py</files>
  <read_first>
    - xtask/mpmath_eval/functionals/__init__.py (after Tasks 1a + 1b + 1c)
    - xtask/mpmath_eval/functionals/brx.py (module shape reference)
    - xcfun-master/src/functionals/csc.cpp (CSC functional with kinetic JP dependency)
    - xcfun-master/src/functionals/blocx.cpp (BLOCX TPSS-shaped per CONTEXT D-01-A correction)
    - xcfun-master/src/functionals/pbelocc.cpp (full file — PBE-loc-correlation; C++ pow_expand at zero aborts)
    - xcfun-master/src/functionals/zvpbesolc.cpp (zvPBE-sol-correlation)
    - xcfun-master/src/functionals/zvpbeintc.cpp (zvPBE-int-correlation)
  </read_first>
  <action>
**Step A — Create 5 per-functional modules** (`csc.py`, `blocx.py`, `pbelocc.py`, `zvpbesolc.py`, `zvpbeintc.py`) following the BR family shape. Each ports its corresponding `xcfun-master/src/functionals/<name>.cpp` body verbatim at `mp.mp.prec = 200`. The PBE-correlation variants (`pbelocc`, `zvpbesolc`, `zvpbeintc`) trigger C++ `pow_expand` aborts at the zero-density boundary; mpmath at prec=200 is the sole reference.

```python
# xtask/mpmath_eval/functionals/pbelocc.py
"""PBE-loc-correlation mpmath port per xcfun-master/src/functionals/pbelocc.cpp.

C++ harness aborts on pow_expand at the zero-density boundary (Phase 3 Plan
03-02 finding). mpmath at prec=200 is sole reference per D-03.
"""
import mpmath as mp
from ..ad_chain import taylor_coeffs
from ..densvars import build_densvars

def eval_pbelocc(inputs, vars, mode, order):
    """Port of xcfun-master/src/functionals/pbelocc.cpp at prec=200."""
    mp.mp.prec = 200
    d = build_densvars(inputs, vars)
    # ... port verbatim ...
    raise NotImplementedError("Port PBELOCC body from pbelocc.cpp")
```

Repeat for `csc.py`, `blocx.py`, `zvpbesolc.py`, `zvpbeintc.py`.

**Step B — Update LOOKUP** in `__init__.py` to append the 5 new entries (preserving Tasks 1a + 1b + 1c entries) — final LOOKUP must have ≥ 26 keys (6 ACC-04 + 20 excluded_by_upstream_spec).

**Step C — Per-family RED→GREEN smoke (5 functionals × 1 record each):**

```bash
python3 -c "
from xtask.mpmath_eval.functionals.csc       import eval_csc
from xtask.mpmath_eval.functionals.blocx     import eval_blocx
from xtask.mpmath_eval.functionals.pbelocc   import eval_pbelocc
from xtask.mpmath_eval.functionals.zvpbesolc import eval_zvpbesolc
from xtask.mpmath_eval.functionals.zvpbeintc import eval_zvpbeintc
import mpmath; mpmath.mp.prec = 200
inputs = [mpmath.mpf('0.5'), mpmath.mpf('0.5'),
          mpmath.mpf('0.1'), mpmath.mpf('0.05'), mpmath.mpf('0.1')]
for fn in (eval_csc, eval_blocx, eval_pbelocc, eval_zvpbesolc, eval_zvpbeintc):
    out = fn(inputs, 'A_B_GAA_GAB_GBB', 'partial_derivatives', 2)
    assert len(out) >= 3, fn.__name__
    print(fn.__name__, 'OK', len(out))
"
```

**Step D — End-of-Task-1 invariant check** (final state of the package layout):

```bash
# 6 ACC-04 modules (Plan 06-00) + 3 BR (Task 1a) + 10 SCAN (Task 1b) + 2 kinetic (Task 1c) + 5 misc (Task 1d) = 26 modules.
test $(ls xtask/mpmath_eval/functionals/*.py | grep -v '__init__.py' | wc -l) -ge 26
python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 26"
test ! -e xtask/mpmath_eval/functionals.py    # B-3 revision-2: no single-file module exists
```

**Forbidden:** Same as Task 1a.
  </action>
  <verify>
    <automated>python3 -c "from xtask.mpmath_eval.functionals.csc import eval_csc; from xtask.mpmath_eval.functionals.blocx import eval_blocx; from xtask.mpmath_eval.functionals.pbelocc import eval_pbelocc; from xtask.mpmath_eval.functionals.zvpbesolc import eval_zvpbesolc; from xtask.mpmath_eval.functionals.zvpbeintc import eval_zvpbeintc; import mpmath; mpmath.mp.prec = 200; inputs=[mpmath.mpf('0.5'),mpmath.mpf('0.5'),mpmath.mpf('0.1'),mpmath.mpf('0.05'),mpmath.mpf('0.1')]; [print(fn.__name__, 'OK', len(fn(inputs, 'A_B_GAA_GAB_GBB', 'partial_derivatives', 2))) for fn in (eval_csc, eval_blocx, eval_pbelocc, eval_zvpbesolc, eval_zvpbeintc)]"</automated>
  </verify>
  <acceptance_criteria>
    - 5 new misc modules exist: `find xtask/mpmath_eval/functionals -name 'csc.py' -o -name 'blocx.py' -o -name 'pbelocc.py' -o -name 'zvpbesolc.py' -o -name 'zvpbeintc.py' | wc -l` == 5.
    - LOOKUP dict has ≥ 26 entries (6 ACC-04 + 20 excluded_by_upstream_spec): `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 26"`.
    - Per-family smoke completes in single-digit seconds.
    - **(B-3 revision-2)** Final invariant: `test ! -e xtask/mpmath_eval/functionals.py && test -d xtask/mpmath_eval/functionals` succeeds.
    - No new Python imports beyond `mpmath` across all 20 new modules: `grep -rE 'import (numpy|scipy|sympy)' xtask/mpmath_eval/functionals/ | wc -l` == 0.
  </acceptance_criteria>
  <done>PBE-corr-variants and miscellaneous family (csc, blocx, pbelocc, zvpbesolc, zvpbeintc) populated at prec=200; LOOKUP final state has ≥ 26 entries; package layout invariant preserved (no functionals.py single file); per-family smoke GREEN.</done>
</task>

<task type="auto">
  <name>Task 2: Generate JSONL fixtures + wire validation --reference mpmath flag + remove skip-list</name>
  <files>validation/fixtures/mpmath/brx.jsonl, validation/fixtures/mpmath/brc.jsonl, validation/fixtures/mpmath/brxc.jsonl, validation/fixtures/mpmath/csc.jsonl, validation/fixtures/mpmath/blocx.jsonl, validation/fixtures/mpmath/scanx.jsonl, validation/fixtures/mpmath/scanc.jsonl, validation/fixtures/mpmath/rscanx.jsonl, validation/fixtures/mpmath/rscanc.jsonl, validation/fixtures/mpmath/rppscanx.jsonl, validation/fixtures/mpmath/rppscanc.jsonl, validation/fixtures/mpmath/r2scanx.jsonl, validation/fixtures/mpmath/r2scanc.jsonl, validation/fixtures/mpmath/r4scanx.jsonl, validation/fixtures/mpmath/r4scanc.jsonl, validation/fixtures/mpmath/tw.jsonl, validation/fixtures/mpmath/vwk.jsonl, validation/fixtures/mpmath/pbelocc.jsonl, validation/fixtures/mpmath/zvpbesolc.jsonl, validation/fixtures/mpmath/zvpbeintc.jsonl, validation/src/driver.rs, validation/src/main.rs, xtask/src/bin/regen_mpmath_fixtures.rs</files>
  <read_first>
    - xtask/src/bin/regen_mpmath_fixtures.rs (Plan 06-00 — extend with 20-functional set)
    - validation/src/driver.rs (Plan 06-02 added run_tier3 + --reference flag stub; current state)
    - validation/src/main.rs (Plan 06-02 added --reference parsing; verify)
    - validation/src/fixtures.rs (10k-point xoshiro grid; reuse stratification for mpmath records)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-N2" (lines 104-109, 781-787)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"D-19 Bisection Methodology" Plan 06-N2 (lines 932-941)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"mpmath Fixture Cadence" (lines 1109-1115)
  </read_first>
  <action>
**Step A — Extend `xtask/src/bin/regen_mpmath_fixtures.rs`** to cover the 20-functional set:

```rust
// Plan 06-00 had:
let functionals = ["ldaerfx", "ldaerfc", "ldaerfc_jt", "tpssc", "tpsslocc", "revtpssc"];
// Plan 06-N2 extends:
let functionals = [
    // Plan 06-00 ACC-04 amendment set (LDAERF + TPSS):
    "ldaerfx", "ldaerfc", "ldaerfc_jt", "tpssc", "tpsslocc", "revtpssc",
    // Plan 06-N2 excluded_by_upstream_spec set:
    "brx", "brc", "brxc", "csc", "blocx",
    "scanx", "scanc", "rscanx", "rscanc", "rppscanx", "rppscanc",
    "r2scanx", "r2scanc", "r4scanx", "r4scanc",
    "tw", "vwk", "pbelocc", "zvpbesolc", "zvpbeintc",
];
```

Each functional gets ~30 stratified records (15 from xoshiro256++ seed `0x1234abcd` + 15 hand-curated boundary records covering the regimes where C++ aborts: `min(a, b) ∈ [1e-14, 1e-5]`, `|∇ρ|² > 1e3`, polarised ζ → 1.0). The hand-curated set is implementer's choice per CONTEXT discretion.

**(W-5 revision-1) Add a `--smoke` flag** that selects a 5-functional × 5-record subset for the autonomous CI lane (single-digit seconds total runtime):

```bash
cargo run -p xtask --bin regen-mpmath-fixtures -- --smoke
# 5 functionals × 5 records = 25 evaluations × ~10s each ≈ ~4 minutes (fast enough for CI;
# the executor may pick a smaller record count if needed to stay under 60s).
```

**(W-5 revision-1) Document the FULL ~6h regen as a MANUAL step** in 06-N2-SUMMARY.md (NOT in this plan's automated `<verify>`):

```bash
# MANUAL — execute offline (~6 hours wall-clock):
cargo run -p xtask --bin regen-mpmath-fixtures
# After completion, commit fixtures + .sha256 stamps in a dedicated PR.
```

Verify each fixture file (post-MANUAL run):
```bash
ls -la validation/fixtures/mpmath/
wc -l validation/fixtures/mpmath/*.jsonl   # ~30 lines per file
```

**Step B — Wire `--reference mpmath` in `validation/src/driver.rs`:**

Plan 06-02 added the `--reference` flag stub. Plan 06-N2 fills the implementation:

```rust
//! Phase 6 ACC-04 amendment per D-03 — when --reference mpmath is set, ground truth
//! comes from validation/fixtures/mpmath/<functional>.jsonl. Otherwise default to C++.

#[derive(Clone, Copy, Debug)]
enum Reference { Cpp, Mpmath }

impl Reference {
    fn from_str(s: &str) -> Self {
        match s {
            "cpp" => Reference::Cpp,
            "mpmath" => Reference::Mpmath,
            _ => panic!("--reference must be cpp or mpmath"),
        }
    }
}

const MPMATH_ONLY_FUNCTIONALS: &[&str] = &[
    "brx", "brc", "brxc", "csc", "blocx",
    "scanx", "scanc", "rscanx", "rscanc", "rppscanx", "rppscanc",
    "r2scanx", "r2scanc", "r4scanx", "r4scanc",
    "tw", "vwk", "pbelocc", "zvpbesolc", "zvpbeintc",
];

fn run_tier2_with_reference(reference: Reference, functional: &str, ...) -> anyhow::Result<()> {
    // Per-record reference selection:
    //   - If functional in MPMATH_ONLY_FUNCTIONALS → ALWAYS use mpmath (C++ aborts).
    //   - Else if reference == Reference::Mpmath → use mpmath.
    //   - Else → use C++ via cc-FFI.

    let effective = if MPMATH_ONLY_FUNCTIONALS.contains(&functional.to_lowercase().as_str()) {
        Reference::Mpmath
    } else { reference };

    match effective {
        Reference::Cpp => { /* existing cc-FFI path */ }
        Reference::Mpmath => {
            let path = format!("validation/fixtures/mpmath/{}.jsonl", functional.to_lowercase());
            let lines = std::fs::read_to_string(&path)?;
            for line in lines.lines() {
                let record: MpmathRecord = serde_json::from_str(line)?;
                // Run Rust eval at the same density input + compare to record.output at strict 1e-13.
                let mut rust_out = vec![0.0; record.output.len()];
                rust_eval(&record.input, &mut rust_out, ...)?;
                for i in 0..rust_out.len() {
                    let rel = (rust_out[i] - record.output[i]).abs() / record.output[i].abs().max(1.0);
                    if rel > 1e-13 {
                        anyhow::bail!("{} {}/{} mpmath rel_err {} > 1e-13", functional, record.input.len(), i, rel);
                    }
                }
            }
        }
    }
    Ok(())
}
```

**Step C — Remove Phase 4 skip-list entries for the 20 functionals:**

`validation/src/driver.rs` has a `SKIP_LIST` (or analogous constant) added at Phase 4 commit `f968c32` (per `.planning/REQUIREMENTS.md` MGGA-02 caveat): the SCAN family ×10 + BR×3 + CSC + BLOCX + TW + VWK + PBELOCC + ZVPBESOLC + ZVPBEINTC are skipped via `excluded_by_upstream_spec`. Plan 06-N2 RESOLVES the exclusion by routing these via `--reference mpmath`.

Find the skip-list:
```bash
git grep -n "excluded_by_upstream_spec\|SKIP_LIST" validation/src/
```

Remove or annotate as RESOLVED:
```rust
// PRE-Plan-06-N2:
const SKIP_LIST: &[&str] = &["brx", "brc", "brxc", "csc", "blocx", "scanx", /* ... */];

// POST-Plan-06-N2:
const SKIP_LIST: &[&str] = &[];   // Phase 6 Plan 06-N2 closed via mpmath fixtures
```

**Step D — Update `report.html` per-record source annotation:**

When emitting `report.jsonl` records, annotate the source: `{ ..., "source": "mpmath" or "cpp", "rel_err_threshold": 1e-13 or 1e-12 }`. The existing Phase 2 D-15 report format already supports per-record metadata.

**Step E — Verify (autonomous + manual splits):**

```bash
# (W-5 step a) AUTONOMOUS: SMOKE generation finishes in <60s.
cargo run -p xtask --bin regen-mpmath-fixtures -- --smoke

# (W-5 step c) AUTONOMOUS: drift gate runs on existing committed fixtures (re-hash; <30s).
cargo run -p xtask --bin regen-mpmath-fixtures -- --check    # exits 0 if no drift

# (W-5 step b) MANUAL — offline ~6h regen (documented in 06-N2-SUMMARY.md, NOT in CI):
cargo run -p xtask --bin regen-mpmath-fixtures

# AFTER manual regen + commit, run the tier-2 sweep with mpmath reference for the 20-set:
cargo run -p validation --release -- --backend cpu --tier 2 --order 3 --jobs 4 \
    --reference mpmath \
    --filter '^(brx|brc|brxc|csc|blocx|scanx|scanc|rscanx|rscanc|rppscanx|rppscanc|r2scanx|r2scanc|r4scanx|r4scanc|tw|vwk|pbelocc|zvpbesolc|zvpbeintc)$'
```

Expected: 0 failures at strict 1e-13 (mpmath truth at prec=200; Rust matches algorithmic-identity contract).

**Forbidden:**
- Do NOT commit fixture files larger than ~40KB each (W-11 cap; ~30 records × ~1KB = ~30KB target). If fixtures balloon, reduce record count or compress.
- Do NOT silently widen tolerance from 1e-13 to 1e-12 — escalate via PLANNING INCONCLUSIVE for user approval.
- Do NOT include the mpmath fixtures in `.gitignore`; they are committed source-of-truth (mirrors Phase 2 D-15 `report.jsonl` ungitignored pattern).
  </action>
  <verify>
    <automated>cargo run -p xtask --bin regen-mpmath-fixtures -- --smoke && cargo run -p xtask --bin regen-mpmath-fixtures -- --check</automated>
  </verify>
  <acceptance_criteria>
    - 20 JSONL fixture files exist under `validation/fixtures/mpmath/` (post-MANUAL regen): `find validation/fixtures/mpmath -name '*.jsonl' | wc -l` >= 20.
    - Each fixture file has a corresponding `.sha256` stamp: `find validation/fixtures/mpmath -name '*.jsonl.sha256' | wc -l` >= 20.
    - **(W-11 revision-1)** Each individual fixture file is < 40KB: `find validation/fixtures/mpmath -name '*.jsonl' -size +40k | wc -l` == 0.
    - **(W-11 revision-1)** Total record count across the 20 functionals is approximately ~600 (~30 records × 20 functionals): `cat validation/fixtures/mpmath/*.jsonl | wc -l` is between 500 and 700.
    - `grep -c '"--reference mpmath"\|Reference::Mpmath' validation/src/driver.rs` >= 1
    - `grep -c "MPMATH_ONLY_FUNCTIONALS\|brx.*brc.*brxc" validation/src/driver.rs` >= 1
    - **(B-3 revision-2)** Package layout invariant: `test -d xtask/mpmath_eval/functionals && test ! -e xtask/mpmath_eval/functionals.py` succeeds (this plan never restructures the package; Plan 06-00 ships it from day one).
    - **(W-5 revision-1)** SMOKE-only run completes in single-digit seconds: `cargo run -p xtask --bin regen-mpmath-fixtures -- --smoke` exits 0 within 60s.
    - **(W-5 revision-1)** Drift gate completes in single-digit seconds: `cargo run -p xtask --bin regen-mpmath-fixtures -- --check` exits 0 within 30s (it only re-hashes existing fixtures; does NOT regenerate).
    - Tier-2 with `--reference mpmath` on the 20 functionals exits 0 (manual verification command in 06-N2-SUMMARY.md after the ~6h offline regen).
  </acceptance_criteria>
  <done>20 mpmath ports populated in the existing `functionals/` package (B-3 audit-trail); 20 JSONL fixtures + .sha256 stamps committed (W-11 ~30 records/functional cap); validation `--reference mpmath` flag wired (consumes Plan 06-02b CLI); W-5 split: SMOKE generation autonomous in CI; full ~6h regeneration documented as MANUAL step in 06-N2-SUMMARY.md.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust driver ↔ Python sidecar | `Command::new("python3")` subprocess; argument list hardcoded in driver, no user input pass-through |
| mpmath port ↔ C++ source | Algorithmic-identity at prec=200; per-port hand-curated baseline test (per-family RED→GREEN smoke in Tasks 1a–1d) catches transcription bugs |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-MPMATH-PRECISION | low | mpmath at prec=200 reproducibility across mpmath versions | Plan 06-00 `xtask/mpmath_eval/README.md` documents `mpmath>=1.4,<2.0` floor; `--check` drift gate catches version drift |
| T-06-FIXTURE-BLOAT | medium | Committed JSONL fixtures balloon git history | W-11 caps per-file at ~40KB / ~30 records; total ~600 records ≈ ~600KB across 20 files (acceptable; mirrors Phase 2 D-15 report.jsonl pattern) |
| T-06-PORT-BUG | medium | Hand-translated mpmath ports may have transcription bugs | W-6 revision-2 family-grouped sub-tasks (1a–1d) each carry RED→GREEN smoke test BEFORE the full fixture run; per-family bugs surface in single-digit seconds, not after ~6h fixture run |
| T-06-MID-EXEC-RESTRUCTURE | medium | Original Plan 06-N2 had a `mv functionals.py functionals/` step mid-execution (B-3) | Resolved (B-3 revision-2): Plan 06-00 ships the package directory FROM DAY ONE; this plan only adds new modules; no `mv`/`mkdir` anywhere in the diff |
| T-06-LONG-RUN | medium | The ~6h fixture regeneration is too long for autonomous CI execution | W-5 split: SMOKE in CI, FULL run as documented manual step; drift gate (--check) is fast |
</threat_model>

<verification>
- All acceptance criteria GREEN per the automated commands.
- Plan 06-02b `--reference mpmath` flag consumed (no further validation/* CLI churn in this plan).
- No restructure of `xtask/mpmath_eval/` package layout (B-3 revision-2 invariant): `test ! -e xtask/mpmath_eval/functionals.py && test -d xtask/mpmath_eval/functionals` succeeds at every step.
- Per-family smoke tests pass (W-6 revision-2 — Tasks 1a–1d each have their own per-family `<verify>` block running in single-digit seconds).
- Per-fixture file size ≤ 40KB (W-11).
- Manual ~6h full regeneration command documented in 06-N2-SUMMARY.md for offline execution (W-5).
</verification>

<success_criteria>
- ACC-04 amendment per CONTEXT.md D-03 fully realised for the 20 excluded_by_upstream_spec set.
- ROADMAP Phase 6 implicit sign-off requirement (all 78 functionals tier-2 GREEN) advanced — the 20 set are the largest blocker.
- B-3 revision-2 / W-5 / W-6 revision-2 / W-11 from review iterations applied.
- Plans 06-N1 / 06-N3 remain parallel-safe — no cross-plan coupling introduced.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N2-SUMMARY.md` documenting:
- 20 mpmath ports landed in `xtask/mpmath_eval/functionals/` (B-3 revision-2 — package layout from 06-00 reused; no mid-execution restructure)
- 20 JSONL fixtures + .sha256 stamps committed (W-11 ~30 records/functional cap; per-file < 40KB)
- W-5 split: (a) SMOKE auto + (c) drift-gate auto in CI; (b) FULL ~6h regen command MANUAL — exact command + expected runtime documented here
- W-6 revision-2 family split: Task 1a (BR) / 1b (SCAN) / 1c (kinetic-GGA) / 1d (PBE-corr+misc) — per-family smoke verdicts
- validation `--reference mpmath` flag wired through `validation/src/driver.rs::run_tier2_with_reference` (consumes Plan 06-02b CLI extension)
- Phase 4 skip-list at `validation/src/driver.rs` REMOVED (excluded_by_upstream_spec entries closed)
- REQUIREMENTS.md GGA-03 / GGA-10 (CSC) / MGGA-02 (SCAN family) / MGGA-05 (BLOCX) updated from `~` to `Complete (with caveat)` where caveat = mpmath truth at prec=200
</output>
</content>
</invoke>