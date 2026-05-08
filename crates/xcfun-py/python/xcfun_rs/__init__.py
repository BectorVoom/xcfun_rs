"""xcfun_rs — Rust reimplementation of xcfun via PyO3.

This module re-exports the symbols from the native ``_native`` cdylib produced
by maturin and provides the Python-source ``XcfunError`` shim that grafts
``.code`` / ``.kind`` attributes (D-09; abi3 §5 workaround per 07-RESEARCH).

Plan 07-04 — exports ``Functional`` (PY-02) plus ``Mode`` and ``Vars``
IntEnums whose discriminants are byte-matched against ``xcfun-core``.
"""

from ._native import (  # type: ignore[attr-defined]
    Functional, Mode, Vars,
    XcfunError as _XcfunErrorBase,
    version, splash, authors, self_test, is_compatible_library,
    which_vars, which_mode,
    enumerate_parameters, enumerate_aliases,
    describe_short, describe_long,
)


# D-09 — attach .code and .kind attributes via a thin Python wrapper.
#
# CRITICAL — this Python-source shim exists because abi3-py310 forbids
# subclassing PyException at the C level until Python 3.12 (see
# 07-RESEARCH §5 CRITICAL FINDING). The Rust side `xc_to_py` raises with
# positional args (message, code, kind); Python's exception-matching rules
# catch the bare `_XcfunErrorBase` via this subclass; the caller accesses
# `.code` / `.kind` on the caught instance.
class XcfunError(_XcfunErrorBase):  # noqa: N818  (suffix-Error name is intentional)
    """Exception raised by xcfun_rs operations.

    Attributes:
        code: C ABI error code per Phase 5 D-08-A
              {0: ok, 1: InvalidOrder, 2: InvalidVars, 4: InvalidMode,
               6: InvalidVarsAndMode, -1: UnknownName / other}.
        kind: Rust ``XcError`` variant name as string
              (e.g. ``"InvalidVars"``, ``"WgpuNoF64"``).
    """

    code: int
    kind: str

    def __init__(self, *args):
        # Rust raises with positional args (msg, code, kind).
        if len(args) == 3:
            msg, self.code, self.kind = args
            super().__init__(msg)
        else:  # defensive — direct construction from Python
            super().__init__(*args)
            self.code = -1
            self.kind = "Unknown"


__version__ = version()

__all__ = [
    "Functional", "Mode", "Vars",
    "XcfunError",
    "version", "splash", "authors", "self_test", "is_compatible_library",
    "which_vars", "which_mode",
    "enumerate_parameters", "enumerate_aliases",
    "describe_short", "describe_long",
    "__version__",
]
