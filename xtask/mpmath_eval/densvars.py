"""DensVars equivalent at mp.prec=200.

Plan 06-N2 will mirror `crates/xcfun-eval/src/density_vars/build.rs`
construction logic in mpmath-precision arithmetic so functional bodies
can pull `n`, `gnn`, `tau`, etc. directly from the built struct rather
than re-deriving in each per-functional module.
"""


def build_densvars(inputs, vars):
    """Build a DensVars-equivalent dict from raw inputs + Vars enum name."""
    raise NotImplementedError("Plan 06-N2 populates this body")
