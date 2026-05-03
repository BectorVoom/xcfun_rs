"""Phase 6 D-04 — mpmath sidecar package marker.

This package is invoked from Rust's `xtask/src/bin/regen_mpmath_fixtures.rs`
via `python3 -m xtask.mpmath_eval`. It is NOT a runtime/library dependency:
no `xcfun-*` library crate imports Python.

Reproducibility: mpmath >= 1.4, mp.prec = 200 (set in `__main__.py`).
"""
