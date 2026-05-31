---
quick_id: 260601-hsaov
slug: hsa-override-gfx1152-readme
date: 2026-06-01
status: complete
mode: quick
---

# Quick Task 260601-hsaov: Document HSA_OVERRIDE_GFX_VERSION=11.0.0 for RDNA-3.5 iGPUs (gfx1152) in xcfun-gpu README

## Problem

`crates/xcfun-gpu/README.md` documents the RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0`
workaround but nothing for newer RDNA-3.5 integrated GPUs whose `gfx` target the
installed HIP compiler has no native code object for. On the dev machine's
**Radeon 860M (gfx1152, Ryzen AI 300-series)** the `HipRuntime` probe
(`rocm_available()` / `Batch::open_rocm()`) bails until
`HSA_OVERRIDE_GFX_VERSION=11.0.0` coerces the target to gfx1100's code object.

Verified 2026-06-01: with the override exported, the tier-3 ROCm oracle
(`--features hip --backend rocm --tier 3 --order 3`) reported **0 failing at
strict 1e-13** across the 17 known-clean exchange functionals. This is a real
environment gotcha that belongs next to the existing 10.3.0 guidance.

## Tasks

### Task 1 — Add the gfx1152 / 11.0.0 override to the README

- **files:** `crates/xcfun-gpu/README.md`
- **action:** Add a `HSA_OVERRIDE_GFX_VERSION=11.0.0` row to the Environment
  Variables table (covering RDNA-3.5 iGPUs not in the HIP target list, e.g.
  gfx1152 / Radeon 860M), add the matching `export` line + a probe-bail
  troubleshooting paragraph to the ROCm Install (Linux) section.
- **verify:** `grep -c '11.0.0' crates/xcfun-gpu/README.md` ≥ 2; the new table
  row and install note render alongside the existing 10.3.0 guidance.
- **done:** README documents both override cases as a set; no other files touched.

## must_haves

- **truths:** gfx1152 requires `HSA_OVERRIDE_GFX_VERSION=11.0.0` for the ROCm
  probe to launch; verified 0 failing at 1e-13 on 2026-06-01.
- **artifacts:** `crates/xcfun-gpu/README.md` (env-var table row + install note +
  troubleshooting paragraph).
- **key_links:** `crates/xcfun-gpu/README.md` Environment Variables table; ROCm
  Install (Linux) section.

## Notes

Documentation-only change. The edit was applied in the working tree prior to
invoking this quick task; the executor step commits it atomically.
