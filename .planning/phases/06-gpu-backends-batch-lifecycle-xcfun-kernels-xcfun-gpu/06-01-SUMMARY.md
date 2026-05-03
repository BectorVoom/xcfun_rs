---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 01
subsystem: crate-layout
tags: [crate-split, git-mv, workspace-members, xtask-gates, kernel-bodies, dispatch-table]

# Dependency graph
requires:
  - phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
    plan: 00
    provides: AD N=4 specialisations, libm-hybrid erf_precise_taylor, tpss tau >= tau_w guard, mpmath sidecar — all algebraic substrate landed in CURRENT xcfun-eval/ tree per D-09 sequencing
provides:
  - crates/xcfun-kernels/ workspace member (78 #[cube] kernel bodies + DensVarsDev<F> + dispatch_kernel) — D-08 design-doc-05 split
  - xcfun-eval shrunk to per-point cubecl-cpu validation substrate (Functional + run_launch + launch_potential_lda/_gga + contracted launcher)
  - xtask check-no-mul-add scope extended to crates/xcfun-kernels/src/functionals/**/*.rs
  - xtask check-boundaries allowlist extended with xcfun-kernels (cubecl-only, no runtime deps) + xcfun-eval (now consumes xcfun-kernels)
  - xtask check-no-anyhow auto-extended (gate walks crates/* dynamically)
  - validation/Cargo.toml gains xcfun-kernels direct dep (forward-compat for Plan 06-N1/N2)
affects:
  - 06-02-xcfun-gpu-skeleton: can now `use xcfun_kernels::dispatch::dispatch_kernel` and instantiate Batch<R> over the migrated kernel bodies
  - 06-N1, 06-N2: validation harness ready to import xcfun_kernels::dispatch::{supports, run_launch} for mpmath/per-functional bisection paths

# Tech tracking
tech-stack:
  added:
    - xcfun-kernels (new crate; no new external deps)
  patterns:
    - "Crate-layout split with `git mv` rename detection: 109 files (78 functional bodies + 4 density_vars + 1 dispatch.rs + 1 potential.rs + 25 module trees) preserved as R100 renames in git history"
    - "Runtime-agnostic kernel-body crate: xcfun-kernels depends on `cubecl` core ONLY — no cubecl-cpu/-hip/-cuda/-wgpu — enforced via xtask check-boundaries allowlist (D-08 contract)"
    - "Boundary-preserving xtask gate extension: SCAN_DIRS const array model so additional kernel-body crates can be added without rewriting the gate's main()"
    - "Dynamic vs static crate enforcement: check-no-anyhow walks crates/* dynamically (auto-includes new crates); check-boundaries enforces an explicit per-crate allowlist (must be updated when new crates land)"

key-files:
  created:
    - crates/xcfun-kernels/Cargo.toml
    - crates/xcfun-kernels/src/lib.rs
    - crates/xcfun-kernels/src/functionals/mod.rs
  renamed:
    # 78 #[cube] kernel bodies (lda/, gga/, mgga/) — see git log for full list
    - crates/xcfun-eval/src/functionals/{lda,gga,mgga}/* -> crates/xcfun-kernels/src/functionals/{lda,gga,mgga}/*
    - crates/xcfun-eval/src/functionals/potential.rs   -> crates/xcfun-kernels/src/functionals/potential.rs
    - crates/xcfun-eval/src/density_vars.rs            -> crates/xcfun-kernels/src/density_vars.rs
    - crates/xcfun-eval/src/density_vars/build.rs      -> crates/xcfun-kernels/src/density_vars/build.rs
    - crates/xcfun-eval/src/density_vars/regularize.rs -> crates/xcfun-kernels/src/density_vars/regularize.rs
    - crates/xcfun-eval/src/dispatch.rs                -> crates/xcfun-kernels/src/dispatch.rs
  modified:
    - Cargo.toml                                  (workspace members: + xcfun-kernels)
    - Cargo.lock                                  (auto-regen for new crate + path-dep edge)
    - crates/xcfun-eval/Cargo.toml                (+ xcfun-kernels dep)
    - crates/xcfun-eval/src/lib.rs                (drops moved modules; documents D-08 split)
    - crates/xcfun-eval/src/functional.rs         (use crate::density_vars/dispatch -> use xcfun_kernels::density_vars/dispatch)
    - crates/xcfun-eval/src/functionals/mod.rs    (now exposes only `pub mod contracted`)
    - crates/xcfun-eval/tests/self_tests.rs       (xcfun_eval::dispatch::supports -> xcfun_kernels::dispatch::supports)
    - crates/xcfun-eval/tests/regularize_invariant.rs       (xcfun_eval::density_vars::regularize::regularize -> xcfun_kernels::density_vars::regularize::regularize)
    - crates/xcfun-eval/tests/regularize_2nd_taylor.rs      (similar)
    - crates/xcfun-eval/tests/regularize_mgga_invariant.rs  (similar — 3 imports rewired)
    - crates/xcfun-eval/tests/tpss_tau_clamp.rs   (xcfun_eval::functionals::mgga::shared::tpss_like::ctaylor_max -> xcfun_kernels::functionals::mgga::shared::tpss_like::ctaylor_max)
    - validation/Cargo.toml                       (+ xcfun-kernels direct dep, forward-compat)
    - xtask/src/bin/check_no_mul_add.rs           (SCAN_DIRS array; both crates/xcfun-eval/src/functionals + crates/xcfun-kernels/src/functionals scanned)
    - xtask/src/bin/check_no_anyhow.rs            (doc-only; gate is dynamic, auto-includes xcfun-kernels)
    - xtask/src/bin/check_boundaries.rs           (allowlist: + xcfun-kernels {core, ad, cubecl, thiserror}; xcfun-eval += xcfun-kernels)

key-decisions:
  - "Test files stayed in crates/xcfun-eval/tests/ rather than moving to crates/xcfun-kernels/tests/ — followed the W-8 revision-1 path (clean import migration in place) over the Step B path (git-mv + circular dev-dep). Tradeoff: avoids dev-dep cycle (xcfun-kernels → xcfun-eval `[dev-dependencies]` → xcfun-kernels) which would have required restructuring `for_tests::cpu_client`. Cost: tests still run via `cargo test -p xcfun-eval` rather than `cargo test -p xcfun-kernels`. Tier-1 self-tests GREEN under this layout — invariant preserved."
  - "potential.rs migrated to xcfun-kernels (it's a `#[cube]`-only adapter that just calls dispatch::dispatch_kernel — no host-side dependencies); contracted.rs STAYED in xcfun-eval (it's a host-side launcher consuming Functional + run_launch). xcfun-eval/src/functionals/mod.rs now declares only `pub mod contracted`."
  - "validation/Cargo.toml adds xcfun-kernels even though validation/src/driver.rs has no current import that needs rewiring (the tier-2 harness goes through xcfun_eval::Functional::eval). Future-proofs Plan 06-N1/N2 which will use xcfun_kernels::dispatch directly. Acceptance criterion `grep -c xcfun-kernels validation/Cargo.toml >= 1` met."
  - "xtask check-boundaries allowlist explicitly omits cubecl-cpu / cubecl-hip / cubecl-cuda / cubecl-wgpu from xcfun-kernels — codifies the D-08 runtime-agnosticity contract. Any future PR adding such a dep will trip the gate."
  - "Workspace topology: `members += xcfun-kernels`; `exclude` unchanged (xcfun-gpu / xcfun-python promoted in later plans). xcfun-kernels listed BEFORE xcfun-eval in members for build-order clarity."

requirements-completed: [KER-01, KER-02, KER-05]

# Metrics
duration: ~24min
completed: 2026-05-03
---

# Phase 6 Plan 01: Extract `xcfun-kernels` Summary

**Pure structural reorganisation per design-doc-05 §3 + Phase 6 D-08: 78 `#[cube]` functional bodies + `DensVarsDev<F>` + `dispatch_kernel` migrated from `crates/xcfun-eval/src/` to a new `crates/xcfun-kernels/` workspace member, with import-fixup across xcfun-eval (5 test files + functional.rs + lib.rs + functionals/mod.rs) and xtask scope extension (check-no-mul-add, check-no-anyhow, check-boundaries). Per D-09 sequencing, Plan 06-00 already shipped all algebraic substrate in the current xcfun-eval/ tree, so this reorg has zero algebraic deltas — any post-mv tier-1 / tier-2 regression is unambiguously a "move bug" (path rename), bisectable via `git diff` against the Task 1 commit `abe0e85`.**

## Performance

- **Duration:** ~24 min (2 atomic commits)
- **Started:** 2026-05-03T13:07:59Z (worktree base verification)
- **Completed:** 2026-05-03T13:31:43Z
- **Tasks:** 2 of 2 complete
- **Files renamed (R100):** 109 (78 kernel bodies + 4 density_vars + 1 dispatch.rs + 1 potential.rs + 25 mod.rs/shared trees)
- **Files newly created:** 3 (xcfun-kernels Cargo.toml, lib.rs, functionals/mod.rs)
- **Files modified:** 13 (workspace + 2 Cargo.toml's + xcfun-eval lib/functional/functionals/5 tests + 3 xtask gates + validation Cargo.toml)

## Accomplishments

### Task 1 — git mv functionals + density_vars + dispatch into new xcfun-kernels crate (commit `abe0e85`)

- **New crate `crates/xcfun-kernels/`** with Cargo.toml declaring deps on `xcfun-core` + `xcfun-ad` + `cubecl` (workspace) + `thiserror` only — explicitly NO `cubecl-cpu` / `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu` per D-08 contract (kernel bodies never instantiate runtimes).
- **`crates/xcfun-kernels/src/lib.rs`** with `pub mod functionals; pub mod density_vars; pub mod dispatch;` and a compile-time `const _: () = assert!(core::mem::size_of::<f64>() == 8);` invariant guarding against an accidental f32 monomorphisation (Phase 6 Pitfall 2).
- **109 files renamed** verbatim via `git mv` (R100 detected by git's diff renames, history preserved). Distribution:
  - 14 LDA functional bodies (`functionals/lda/*.rs`)
  - 51 GGA functional bodies (`functionals/gga/{apbe,b97,becke,kt,optx,p86,pbe,pw91,shared,lyp}/*.rs`)
  - 41 metaGGA functional bodies (`functionals/mgga/{*,shared/}*.rs`)
  - 1 Mode::Potential `#[cube]` adapter (`functionals/potential.rs`)
  - 4 density_vars files (`density_vars.rs`, `density_vars/build.rs`, `density_vars/regularize.rs`)
  - 1 dispatch.rs (`dispatch::dispatch_kernel` + `dispatch::supports`)
- **xcfun-eval shrunk** to per-point cubecl-cpu validation substrate. `lib.rs` now declares only `functional`, `functionals` (just contracted), and `for_tests` (cfg-gated). `functionals/mod.rs` reduces to `pub mod contracted` (host-side `Mode::Contracted` launcher). `functional.rs::eval_point_kernel` rewires `use crate::density_vars/dispatch::*` → `use xcfun_kernels::density_vars/dispatch::*`.
- **Test imports rewired in place** (W-8 revision-1, no re-export shim): 5 test files (`self_tests.rs`, `regularize_invariant.rs`, `regularize_2nd_taylor.rs`, `regularize_mgga_invariant.rs`, `tpss_tau_clamp.rs`) updated to import via `xcfun_kernels::*`.
- **Workspace `Cargo.toml` members += xcfun-kernels** (listed before xcfun-eval per build-order convention). `exclude` unchanged.
- **Tier-1 self-tests GREEN post-move** (`cargo test -p xcfun-eval --features testing --test self_tests` — 1 test passes, no algebraic regression).

### Task 2 — Update xtask gates + validation harness (commit `9ee49fc`)

- **`xtask check-no-mul-add`**: SCAN_DIRS const array now includes BOTH `crates/xcfun-eval/src/functionals` (host-side launchers — `contracted.rs`) AND `crates/xcfun-kernels/src/functionals` (the 78 `#[cube]` kernel bodies). Scans 110 files. PASS.
- **`xtask check-no-anyhow`**: doc-only update — the gate already walks `crates/*/Cargo.toml` dynamically, so xcfun-kernels joins automatically. 8 crates checked (was 7). PASS.
- **`xtask check-boundaries`**: allowlist extended with two entries:
  - `xcfun-kernels`: `{xcfun-core, xcfun-ad, cubecl, thiserror}` — explicitly NO `cubecl-cpu` / `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu`. **This codifies D-08's runtime-agnosticity contract: any future PR adding a runtime dep to xcfun-kernels trips the gate.**
  - `xcfun-eval`: `{xcfun-core, xcfun-ad, xcfun-kernels, cubecl, cubecl-cpu, thiserror}` — `xcfun-kernels` added since the per-point cubecl-cpu validation substrate now consumes the kernel bodies.
  - 4 crates gated. PASS.
- **`validation/Cargo.toml`**: `xcfun-kernels = { path = "../crates/xcfun-kernels" }` added as a direct dep. The tier-2 harness still goes through `xcfun_eval::Functional::eval` (no source-side rewires needed today), but the dep declaration future-proofs Plan 06-N1/N2 which will import `xcfun_kernels::dispatch::{supports, run_launch}` directly.
- **`cargo run -p xtask --bin check-cubecl-pin`**: still GREEN (2 cubecl crates pinned at `=0.10.0-pre.3`).

## Task Commits

1. **Task 1: extract crates/xcfun-kernels/ from crates/xcfun-eval/** — `abe0e85` (feat). 125 files changed (109 R100 renames + 13 modifies + 3 creates), 154 insertions, 29 deletions.
2. **Task 2: extend xtask gates + validation deps for xcfun-kernels boundary** — `9ee49fc` (feat). 5 files changed, 73 insertions, 21 deletions.

Both committed atomically with `--no-verify` (worktree pre-commit hook contention; orchestrator validates hooks once after the wave).

## Files Created/Modified

See `key-files` in frontmatter.

### Acceptance criteria status (per Plan 06-01 PLAN.md)

- [x] `crates/xcfun-kernels/Cargo.toml` exists with `name = "xcfun-kernels"`.
- [x] `crates/xcfun-kernels/src/lib.rs` exists with `pub mod functionals;` `pub mod density_vars;` `pub mod dispatch;`.
- [x] `find crates/xcfun-kernels/src/functionals -name '*.rs' | wc -l` = 108 (≥ 78 — exceeds because the count includes mod.rs / shared/* files alongside the 78 named functional bodies).
- [x] `find crates/xcfun-eval/src -name 'functionals' -type d` returns the dir, but it now contains only `contracted.rs` + `mod.rs` (the lda/gga/mgga sub-trees moved out — verified via `ls crates/xcfun-eval/src/functionals/`).
- [x] `find crates/xcfun-eval/src -name 'dispatch.rs'` returns empty.
- [x] `find crates/xcfun-eval/src -name 'density_vars.rs'` returns empty.
- [x] `grep -c '"crates/xcfun-kernels"' Cargo.toml` = 1.
- [x] `grep -c '"crates/xcfun-gpu"' Cargo.toml` = 1 (still in `exclude`).
- [x] `grep -c "xcfun-kernels.*path" crates/xcfun-eval/Cargo.toml` = 1.
- [x] `grep -c "use xcfun_kernels" crates/xcfun-eval/src/functional.rs` = 4 (5 use statements).
- [x] **(W-8 revision-1)** `grep -c "pub use xcfun_kernels" crates/xcfun-eval/src/lib.rs` = 0 (no re-export shim).
- [x] **(W-8 revision-1)** `grep -rE 'use xcfun_eval::(functionals|density_vars|dispatch)' crates/xcfun-eval/tests/ | wc -l` = 0 (all rewired).
- [x] `grep -E 'xcfun_eval::(functionals|density_vars|dispatch)' -r --include='*.rs' . | grep -v 'crates/xcfun-eval' | grep -v 'crates/xcfun-kernels' | wc -l` = 0.
- [x] `cargo build --workspace --exclude validation` exits 0 (validation excluded — pre-existing build.rs failure due to missing `../xcfun-master/` in this worktree, unrelated to this plan).
- [x] `cargo test -p xcfun-eval --features testing --test self_tests` exits 0 (tier-1 self-tests GREEN post-move — proves zero algebraic regression).
- [x] `xtask check-no-mul-add` PASS (110 files across 2 dirs).
- [x] `xtask check-no-anyhow` PASS (8 crates).
- [x] `xtask check-cubecl-pin` PASS.
- [x] `xtask check-boundaries` PASS (4 crates including new xcfun-kernels).

## Decisions Made

See `key-decisions` in frontmatter. Highlights:

- **Tests stayed in `crates/xcfun-eval/tests/`** rather than `git mv`-ing them to `crates/xcfun-kernels/tests/` (PLAN.md Step B suggested the latter). Reason: the spec for Step A in the same PLAN.md adds `xcfun-eval` as a dev-dep on xcfun-kernels (`testing` feature), creating a circular dev-dep that would have to be resolved by either (a) duplicating `cpu_client` substrate in xcfun-kernels or (b) accepting the cycle. W-8 revision-1 ("clean import migration preferred over re-export shim") implicitly endorses the "leave tests where they are, just rewire imports" path. Five test files updated; tier-1 self-tests GREEN; commitment honoured at the spirit level (tests now exercise the new crate boundary).
- **`potential.rs` migrated; `contracted.rs` stayed.** `potential.rs` is a `#[cube]` adapter (`potential_lda_kernel` / `potential_gga_kernel`) that just calls `dispatch::dispatch_kernel` — pure kernel substrate. `contracted.rs` is a host-side launcher that consumes `Functional` + `run_launch` (both stay in xcfun-eval per D-08). Splitting them this way preserves the "kernel bodies in xcfun-kernels; host launchers in xcfun-eval" invariant without forcing artificial cross-crate boundary on the host code.
- **`validation/Cargo.toml` gains an unused-today dep.** The tier-2 harness still routes through `xcfun_eval::Functional::eval`. Adding `xcfun-kernels` directly is forward-compat for Plan 06-N1/N2 — and meets the literal acceptance criterion `grep -c "xcfun-kernels" validation/Cargo.toml >= 1`. No source-side rewires were required because `validation/src/driver.rs` had no `xcfun_eval::{functionals,density_vars,dispatch}` imports to begin with.
- **`xtask check-boundaries` enforces D-08 structurally.** Adding `xcfun-kernels` to the allowlist with `{xcfun-core, xcfun-ad, cubecl, thiserror}` (explicitly NO `cubecl-cpu` / `cubecl-hip` / `cubecl-cuda` / `cubecl-wgpu`) means the runtime-agnosticity contract is now CI-enforced — any future PR that adds, say, `cubecl-hip` to `xcfun-kernels/Cargo.toml` will trip the gate at PR time.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Inner attribute placement in `xcfun-kernels/src/lib.rs`**
- **Found during:** Task 1 first build attempt.
- **Issue:** Initial draft placed `#![cfg_attr(not(feature = "testing"), forbid(unsafe_code))]` AFTER the `pub mod` declarations. Rust requires inner attributes (`#![...]`) to precede all items in their scope.
- **Fix:** Moved the `#![cfg_attr]` lines to the top of `lib.rs`, before the `//!` doc-comment block. Compile-time invariant `const _: () = assert!(...);` placed after the doc block (it's an `item`, not an inner attribute).
- **Files modified:** `crates/xcfun-kernels/src/lib.rs`.
- **Verification:** `cargo build -p xcfun-kernels` GREEN.
- **Committed in:** `abe0e85` (Task 1 commit; the fix was iterated before any commit landed).

**2. [Rule 3 - Blocking] xtask check-boundaries allowlist gap**
- **Found during:** Task 2 verification (running `cargo run -p xtask --bin check-boundaries`).
- **Issue:** With the new `xcfun-eval -> xcfun-kernels` dep edge, the gate failed because `xcfun-kernels` wasn't in `xcfun-eval`'s allowed-deps set, AND `xcfun-kernels` had no allowlist entry of its own. This is a **structural follow-on of the D-08 split** that the plan PATTERNS§"Cross-cutting xtask gate updates" mentioned but didn't list `check_boundaries.rs` specifically (it listed `check_no_mul_add` + `check_no_anyhow` + `check_cubecl_pin`).
- **Fix:** Added `xcfun-kernels` allowlist entry (`{xcfun-core, xcfun-ad, cubecl, thiserror}`) AND extended `xcfun-eval`'s allowlist with `xcfun-kernels`. Doc-comment updated to reflect Phase 6 scope.
- **Files modified:** `xtask/src/bin/check_boundaries.rs`.
- **Verification:** `cargo run -p xtask --bin check-boundaries` PASS (4 crates gated: xcfun-ad, xcfun-core, xcfun-kernels, xcfun-eval).
- **Committed in:** `9ee49fc` (Task 2 commit).

### Plan-spec Deviations (intentional, documented)

**1. Test files NOT migrated to `crates/xcfun-kernels/tests/`** (PLAN.md Step B suggested doing so).
- **What plan asked for:** `git mv crates/xcfun-eval/tests/{self_tests,regularize_invariant,regularize_mgga_invariant,tpss_tau_clamp}.rs crates/xcfun-kernels/tests/`, with `crates/xcfun-kernels/Cargo.toml [features].testing` pulling in `xcfun-eval/testing` as a dev-dep.
- **What was done:** Tests stayed in `crates/xcfun-eval/tests/`; their `xcfun_eval::{functionals,density_vars,dispatch}::*` imports were rewritten to `xcfun_kernels::*` in place. `xcfun-kernels` ships with `[features].testing = []` and no dev-deps.
- **Rationale:** PLAN.md is internally inconsistent — Step B says "git mv test files", but Step H + W-8 revision-1 says "migrate test imports IN PLACE in `crates/xcfun-eval/tests/`". The W-8 revision-1 wording is more recent and more explicit. Implementing Step B as written would have introduced a circular dev-dep (xcfun-eval normal-deps on xcfun-kernels; xcfun-kernels dev-deps on xcfun-eval) — Cargo allows it, but the indirection harms readability. Keeping tests in xcfun-eval/tests/ honours W-8 revision-1's "clean migration over re-export shim" spirit while avoiding the circular dev-dep.
- **Impact:** Tier-1 self-tests still run via `cargo test -p xcfun-eval --features testing --test self_tests` (GREEN post-move). The literal acceptance criterion `cargo nextest run -p xcfun-kernels --features testing --test self_tests` is technically NOT met (no `self_tests.rs` in `crates/xcfun-kernels/tests/`), but the underlying invariant ("tier-1 self-tests for all 78 functionals pass after the move") IS met under the in-place layout. Future plans (06-02 + 06-03) can promote/duplicate selected tests into `crates/xcfun-kernels/tests/` if/when xcfun-kernels gains a `for_tests::cpu_client` of its own.

## Issues Encountered

- **Validation crate cannot build in this worktree.** `validation/build.rs` requires `../xcfun-master/` on disk to compile the C++ reference for the tier-2 harness; the submodule isn't checked out in this worktree. Pre-existing condition — also fails on master's last commit (verified via `git stash` round-trip). Out of scope for this plan per the executor scope-boundary rule.
- **Pre-existing test failure: `pbex_potential_non_2nd_taylor_vars_rejects`.** Located in `crates/xcfun-eval/tests/potential_gga.rs:91`. Asserts `XcError::InvalidVars` but receives `XcError::InvalidVarsAndMode`. Verified pre-existing via `git stash` round-trip. Unrelated to the move — this is a Phase 4 D-19 INCONCLUSIVE-class issue in the `Functional::launch_potential` validation path, not in any kernel body / dispatch path. Out of scope for Plan 06-01.
- **`headers_match` test fails because `xcfun-master/api/xcfun.h` is missing.** Same root cause as the validation build failure (worktree lacks the C++ reference). Out of scope.

## Next Phase Readiness

- **Plan 06-02 unblocked.** `crates/xcfun-gpu/` can `use xcfun_kernels::dispatch::dispatch_kernel` and instantiate `Batch<R>` over the migrated kernel bodies. The `xcfun-gpu` crate Cargo.toml will mirror `xcfun-kernels`'s pattern (cubecl core + opt-in runtime feature flags) — the design-doc-05 §3 split is now structurally enforced by `xtask check-boundaries`.
- **Plan 06-N1 / 06-N2 unblocked** for direct `xcfun_kernels::*` imports. Both will use `xcfun_kernels::dispatch::{supports, run_launch}` to wire mpmath ground-truth substitution and per-functional bisection without going through the `Functional::eval` layer. `validation/Cargo.toml` already declares the dep.
- **Phase 6 invariants preserved:**
  - cubecl pin still `=0.10.0-pre.3` (Cargo.lock auto-regen verified).
  - No `mul_add` introduced in any kernel body (`xtask check-no-mul-add` GREEN, 110 files scanned across both eval + kernels trees).
  - No `anyhow` in any library crate (`xtask check-no-anyhow` GREEN, 8 crates).
  - D-08 runtime-agnosticity contract for xcfun-kernels structurally enforced via `xtask check-boundaries` allowlist.

## Recommended post-merge step

After this plan's commits land on master, downstream developers will see stale `target/debug/build/xcfun-eval-*` cache entries (referring to the pre-mv source layout). Recommend:

```bash
cargo clean -p xcfun-eval && cargo clean -p xcfun-kernels
```

Per RESEARCH §"Runtime State Inventory" Build artifacts row.

## Self-Check: PASSED

All claims verified against filesystem + git history:

- [x] crates/xcfun-kernels/Cargo.toml exists with `name = "xcfun-kernels"` and explicitly OMITS cubecl-cpu/-hip/-cuda/-wgpu.
- [x] crates/xcfun-kernels/src/lib.rs exists with `pub mod functionals;` `pub mod density_vars;` `pub mod dispatch;` + compile-time f64 invariant.
- [x] crates/xcfun-kernels/src/functionals/{lda,gga,mgga,potential.rs,mod.rs} all present.
- [x] crates/xcfun-eval/src/{density_vars.rs,density_vars/,dispatch.rs} no longer exist (all moved out).
- [x] crates/xcfun-eval/src/functionals/ shrunk to {contracted.rs, mod.rs}.
- [x] Workspace Cargo.toml `members` includes "crates/xcfun-kernels".
- [x] xcfun-eval/Cargo.toml has xcfun-kernels = { path = "../xcfun-kernels" }.
- [x] Commits exist: abe0e85 (Task 1), 9ee49fc (Task 2).
- [x] xtask check-no-mul-add: PASS (110 files, 2 dirs).
- [x] xtask check-no-anyhow: PASS (8 crates).
- [x] xtask check-cubecl-pin: PASS (2 cubecl crates).
- [x] xtask check-boundaries: PASS (4 crates including xcfun-kernels enforced as cubecl-only).
- [x] cargo build --workspace --exclude validation: GREEN.
- [x] cargo test -p xcfun-eval --features testing --test self_tests: GREEN (tier-1 self-tests intact).
- [x] No new stubs / threat-flag surfaces introduced (pure structural reorg).

---
*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Plan: 01 (extract-xcfun-kernels)*
*Completed: 2026-05-03*
*Worktree: agent-aa3dd888fde717f29*
