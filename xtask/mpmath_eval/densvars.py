"""DensVars equivalent at mp.prec=200.

Phase 6 Plan 06-N2 substrate. Mirrors the host-side input -> DensVars
unpacking in `crates/xcfun-eval/src/density_vars/build.rs` so each
per-functional port can read `d['n']`, `d['gnn']`, `d['taua']`, etc.,
rather than re-deriving from the raw `inputs` list.

The unpacking convention follows `xcfun-master/api/xcfun.h:88-122` and
the C++ `densvars` template at `xcfun-master/src/densvars.hpp`. Rust's
build path is at `crates/xcfun-eval/src/density_vars/build.rs:60-220`.

Scope:
    * Provides ONLY the slot names referenced by the 20 mpmath ports
      shipped in this plan (BR family + SCAN family + TW + VWK + CSC +
      BLOCX + PBELOCC + ZVPBESOLC + ZVPBEINTC).
    * Handles the canonical metaGGA layouts:
        - XC_A_B_GAA_GAB_GBB                              (5)
        - XC_A_B_GAA_GAB_GBB_LAPA_LAPB                    (7)
        - XC_A_B_GAA_GAB_GBB_TAUA_TAUB                    (7)
        - XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB          (9)
        - XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB(11)
"""
from __future__ import annotations
import mpmath as mp


# Canonical Vars name -> ordered slot list.
# Slot names use the lowercase short form ('a', 'b', 'gaa', 'lapa', 'taua',
# 'jpaa', etc.) consistent with `crates/xcfun-eval/src/density_vars/`.
VARS_SLOTS = {
    "A_B": ["a", "b"],
    "XC_A_B": ["a", "b"],
    "A_B_GAA_GAB_GBB": ["a", "b", "gaa", "gab", "gbb"],
    "XC_A_B_GAA_GAB_GBB": ["a", "b", "gaa", "gab", "gbb"],
    "A_B_GAA_GAB_GBB_LAPA_LAPB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb",
    ],
    "XC_A_B_GAA_GAB_GBB_LAPA_LAPB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb",
    ],
    "A_B_GAA_GAB_GBB_TAUA_TAUB": [
        "a", "b", "gaa", "gab", "gbb", "taua", "taub",
    ],
    "XC_A_B_GAA_GAB_GBB_TAUA_TAUB": [
        "a", "b", "gaa", "gab", "gbb", "taua", "taub",
    ],
    "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb", "taua", "taub",
    ],
    "XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb", "taua", "taub",
    ],
    "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb", "taua", "taub",
        "jpaa", "jpbb",
    ],
    "XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB": [
        "a", "b", "gaa", "gab", "gbb", "lapa", "lapb", "taua", "taub",
        "jpaa", "jpbb",
    ],
}


def slot_names(vars_str):
    """Return ordered list of slot names for a Vars enum string."""
    if vars_str not in VARS_SLOTS:
        raise KeyError(
            f"densvars: unsupported Vars '{vars_str}'. "
            f"Add to VARS_SLOTS in xtask/mpmath_eval/densvars.py."
        )
    return list(VARS_SLOTS[vars_str])


def build_densvars(inputs, vars_str):
    """Build a DensVars-equivalent dict from raw inputs + Vars enum name.

    Mirrors the field-order of `crates/xcfun-eval/src/density_vars/`:
        n     = a + b
        s     = (a - b) / n          (zeta)
        gnn   = gaa + 2*gab + gbb
        gns   = (gaa - gbb)
        gss   = (gaa - 2*gab + gbb)
        ...
    Slots not present in the chosen Vars layout default to mp.mpf(0).

    Args:
        inputs: list of mp.mpf values matching len(slot_names(vars_str)).
        vars_str: canonical Vars enum string (e.g.,
                  'A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB').

    Returns:
        dict with all per-spin and combined slots populated as mp.mpf.
    """
    slots = slot_names(vars_str)
    if len(inputs) != len(slots):
        raise ValueError(
            f"densvars: expected {len(slots)} inputs for {vars_str}, "
            f"got {len(inputs)}"
        )
    d = {name: mp.mpf(val) for name, val in zip(slots, inputs)}
    # Defaults for missing slots — keeps per-functional code uniform.
    for name in (
        "a", "b", "gaa", "gab", "gbb",
        "lapa", "lapb", "taua", "taub",
        "jpaa", "jpbb",
    ):
        d.setdefault(name, mp.mpf(0))
    a, b = d["a"], d["b"]
    n = a + b
    d["n"] = n
    # zeta = s / n; guard near-zero n
    if n != 0:
        d["s"] = (a - b) / n
        d["zeta"] = d["s"]
    else:
        d["s"] = mp.mpf(0)
        d["zeta"] = mp.mpf(0)
    gaa, gab, gbb = d["gaa"], d["gab"], d["gbb"]
    d["gnn"] = gaa + 2 * gab + gbb
    d["gns"] = gaa - gbb
    d["gss"] = gaa - 2 * gab + gbb
    # Spin-summed laplacian / kinetic, used by some metaGGAs.
    d["lapn"] = d["lapa"] + d["lapb"]
    d["taun"] = d["taua"] + d["taub"]
    return d
