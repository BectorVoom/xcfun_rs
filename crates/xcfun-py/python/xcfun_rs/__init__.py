"""xcfun_rs — Rust reimplementation of xcfun via PyO3.

This module re-exports the symbols from the native `_native` cdylib produced
by maturin. The `XcfunError` shim and `Functional` / `Mode` / `Vars` re-exports
are wired in Phase 7 Plan 07-03 / 07-04 / 07-05; this Plan 07-02 only ships
the 11 module-level free functions.
"""

from ._native import (  # type: ignore[attr-defined]
    version, splash, authors, self_test, is_compatible_library,
    which_vars, which_mode,
    enumerate_parameters, enumerate_aliases,
    describe_short, describe_long,
)

__version__ = version()

__all__ = [
    "version", "splash", "authors", "self_test", "is_compatible_library",
    "which_vars", "which_mode",
    "enumerate_parameters", "enumerate_aliases",
    "describe_short", "describe_long",
    "__version__",
]
