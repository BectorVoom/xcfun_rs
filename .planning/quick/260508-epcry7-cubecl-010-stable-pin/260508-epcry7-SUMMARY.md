---
quick_id: 260508-epcry7
status: complete
date: 2026-05-08
---

# Quick Task 260508-epcry7: Update cubecl to =0.10.0 stable

## What was done

Bumped all 5 cubecl-family workspace pins from `=0.10.0-pre.3` to `=0.10.0` (stable).
cubecl 0.10.0 was published 2026-05-07 across all five crates (cubecl, cubecl-cpu, cubecl-hip,
cubecl-cuda, cubecl-wgpu) — none yanked.

## Files changed

- `Cargo.toml` — 5 cubecl-* pins updated to `"=0.10.0"`
- `xtask/src/bin/check_cubecl_pin.rs` — `REQUIRED_VERSION` updated to `"0.10.0"`; pre-release language removed from doc comments and error messages
- `CLAUDE.md` — all `0.10.0-pre.3` occurrences replaced; TL;DR table, Technology Stack, Version Compatibility, Key Version Constraints, Risk Assessment, Sources, and Confidence sections updated to reflect stable status

## Verification results

- `cargo metadata` resolves all 15 cubecl-* crates at 0.10.0 — no conflicts
- `cargo run -p xtask --bin check-cubecl-pin` → **PASS (5 cubecl crate(s) pinned at 0.10.0)**
- `cargo check -p xcfun-kernels -p xcfun-ad -p xcfun-eval` → **Finished** (8.52s, warnings are pre-existing unused-import noise, unrelated to bump)

## Remaining risk

The 1e-12 numerical parity contract should be verified by re-running the Tier 2 validation
harness (`cargo test -p validation`) against the C++ xcfun reference. This is a CPU-only
check that does not require GPU hardware. CLAUDE.md recommends this for every cubecl bump.
