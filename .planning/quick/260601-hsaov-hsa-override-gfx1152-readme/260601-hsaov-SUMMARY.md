---
quick_id: 260601-hsaov
slug: hsa-override-gfx1152-readme
date: 2026-06-01
status: complete
mode: quick
commits:
  - d4e59d0
---

# Quick Task 260601-hsaov — Summary

## What changed

`crates/xcfun-gpu/README.md` now documents the `HSA_OVERRIDE_GFX_VERSION=11.0.0`
override for RDNA-3.5 integrated GPUs whose `gfx` target the installed HIP
compiler has no native code object for (e.g. **gfx1152 / Radeon 860M**, Ryzen AI
300-series), alongside the pre-existing RDNA-2 `10.3.0` guidance:

1. **Environment Variables table** — new `HSA_OVERRIDE_GFX_VERSION=11.0.0` row:
   coerces a too-new target to gfx1100's code object; without it the
   `HipRuntime` probe (`rocm_available()` / `Batch::open_rocm()`) bails. Same
   "export before first `auto_backend()` / `Batch::open_rocm()`" timing rule as
   the 10.3.0 row.
2. **ROCm Install (Linux)** — matching `export HSA_OVERRIDE_GFX_VERSION=11.0.0`
   line plus a troubleshooting paragraph: if `rocminfo` lists your `gfx` target
   but `open_rocm()` still bails at the probe, pick the override row matching
   your architecture generation.

## Why

Verified 2026-06-01 on the dev machine's gfx1152: with the override exported,
the tier-3 ROCm oracle (`cargo run -p validation --release --features hip --
--backend rocm --tier 3 --order 3`) reported **0 failing at strict 1e-13**
across the 17 known-clean exchange functionals. Without it, the probe bails.
The repo previously only documented the RDNA-2 case, so this was an undocumented
environment gotcha for newer AMD iGPUs.

## Verification

- `grep -c '11.0.0' crates/xcfun-gpu/README.md` → 2 (table row + install export).
- New rows render as a set with the existing 10.3.0 guidance; no other files
  touched in the deliverable commit.

## Commits

- `d4e59d0` — docs(xcfun-gpu): document HSA_OVERRIDE_GFX_VERSION=11.0.0 for gfx1152

## Notes / deviations

- Documentation-only; no code or test changes.
- Executed **inline** (orchestrator-as-executor) rather than via a
  worktree-isolated `gsd-executor`: the README edit was already applied in the
  working tree before the task was invoked, and a fresh worktree branched from
  HEAD would not have seen the uncommitted edit. The installed GSD SDK also
  lacks the `init.quick` query path the workflow template expects, so quick-task
  identifiers were derived from `generate-slug` / `current-timestamp` and the
  steps run directly.
- Committed on `master`, matching the project's established quick-task pattern
  (prior quick tasks committed directly to master). Not pushed.
