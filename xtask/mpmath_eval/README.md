# xtask.mpmath_eval — mpmath sidecar (Phase 6 D-04)

Python module providing 200-digit-precision ground truth for ACC-04-amended
functionals (LDAERF family, TPSS-correlation family) and Plan 06-N2's
20 `excluded_by_upstream_spec` set (BR×3 + CSC + BLOCX + SCAN×10 + TW + VWK
+ PBELOCC + ZVPBESOLC + ZVPBEINTC).

## Reproducibility

- Python: `>= 3.10` (project local: 3.14.4 — verified 2026-05-03)
- mpmath: `>= 1.4, < 2.0`
- mp.prec = 200 (set in `__main__.py`); deterministic for fixed input

## Invocation

```bash
python3 -m xtask.mpmath_eval --functional ldaerfx --vars XC_A_B \
    --mode PartialDerivatives --order 3 --input 0.5,0.5 --prec 200
```

Emits one JSONL record on stdout. Used exclusively by
`xtask/src/bin/regen_mpmath_fixtures.rs`.

## NOT a runtime/library dep

`cargo build` of any `xcfun-*` library crate (xcfun-ad, xcfun-core,
xcfun-kernels post-Plan-06-01, xcfun-eval, xcfun-rs, xcfun-capi, xcfun-gpu)
does NOT require Python3. This module lives in the `xtask/` build-tools tree
and is invoked via `subprocess::Command::new("python3")` ONLY at fixture
regeneration time.

## Package layout

```
xtask/mpmath_eval/
├── __init__.py        # package marker — does NOT import mpmath
├── __main__.py        # CLI entry — imports mpmath inside main()
├── evaluator.py       # LOOKUP dispatch + JSONL record assembly
├── ad_chain.py        # generic Taylor-coefficient helper (Plan 06-N2 fills)
├── densvars.py        # DensVars mirror at mp.prec=200 (Plan 06-N2 fills)
├── README.md          # this file
└── functionals/
    ├── __init__.py    # exports LOOKUP dict mapping name → eval_<name>
    ├── ldaerfx.py     # Plan 06-00 stub; Plan 06-N2 populates body
    ├── ldaerfc.py     # ditto
    ├── ldaerfc_jt.py  # ditto
    ├── tpssc.py       # ditto
    ├── tpsslocc.py    # ditto
    └── revtpssc.py    # ditto
```

Plan 06-N2 will add per-functional modules for the 20 `excluded_by_upstream_spec`
set into the existing `functionals/` directory — no package restructure needed.

## Dependencies (out-of-band)

To regenerate fixtures, install mpmath in a dev venv:

```bash
python3 -m pip install --user 'mpmath>=1.4,<2.0'
```

The `cargo build -p xtask --bin regen-mpmath-fixtures` smoke check does NOT
exercise the Python side — it only verifies the Rust driver compiles.
