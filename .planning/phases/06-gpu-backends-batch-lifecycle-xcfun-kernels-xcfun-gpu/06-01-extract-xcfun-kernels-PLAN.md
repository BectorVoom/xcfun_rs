---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 01
type: execute
wave: 2
depends_on:
  - 06-00
files_modified:
  - Cargo.toml
  - crates/xcfun-kernels/Cargo.toml
  - crates/xcfun-kernels/src/lib.rs
  - crates/xcfun-kernels/src/functionals/
  - crates/xcfun-kernels/src/density_vars/
  - crates/xcfun-kernels/src/density_vars.rs
  - crates/xcfun-kernels/src/dispatch.rs
  - crates/xcfun-kernels/tests/
  - crates/xcfun-eval/Cargo.toml
  - crates/xcfun-eval/src/lib.rs
  - crates/xcfun-eval/src/functional.rs
  - validation/src/driver.rs
  - xtask/src/bin/check_no_mul_add.rs
  - xtask/src/bin/check_no_anyhow.rs
autonomous: true
requirements:
  - KER-01
  - KER-02
  - KER-05
must_haves:
  truths:
    - "crates/xcfun-kernels/ exists as workspace member with 78 functional bodies + DensVarsDev<F> + dispatch_kernel — per D-08 design-doc-05 split."
    - "xcfun-kernels has NO direct dependency on cubecl-cpu / cubecl-hip / cubecl-cuda / cubecl-wgpu — only `cubecl` core (per D-08 contract: never instantiates a runtime)."
    - "xcfun-eval depends on xcfun-kernels; retains only Functional + eval_point_kernel + cpu_client substrate (per D-08)."
    - "Tier-1 self-tests for all 78 functionals pass after the move (`cargo nextest run -p xcfun-kernels --features testing --test self_tests` GREEN)."
    - "validation harness still cc-links xcfun-master + dispatches via xcfun_kernels::dispatch::run_launch (path rename only, no algorithmic change)."
    - "(W-8 revision-1) NO re-export shim in xcfun-eval/src/lib.rs — the 11 existing test files in crates/xcfun-eval/tests/ migrate their imports to xcfun_kernels::* directly (test count < 15 → clean migration preferred over long-lived re-export)."
    - "xtask check-no-mul-add scope EXTENDED to `crates/xcfun-kernels/src/functionals/**/*.rs`; xtask check-no-anyhow allowlist EXTENDED to xcfun-kernels."
  artifacts:
    - path: "crates/xcfun-kernels/Cargo.toml"
      provides: "Workspace-member crate manifest (cubecl core only, no runtime deps)"
      contains: "name = \"xcfun-kernels\""
    - path: "crates/xcfun-kernels/src/lib.rs"
      provides: "Crate root with pub mod functionals; pub mod density_vars; pub mod dispatch;"
      contains: "pub mod functionals"
    - path: "crates/xcfun-kernels/src/dispatch.rs"
      provides: "FunctionalId-keyed dispatch_kernel (78-arm comptime if-chain)"
      contains: "dispatch_kernel"
    - path: "crates/xcfun-eval/src/functional.rs"
      provides: "Functional struct + eval_point_kernel; imports rewired to xcfun_kernels::*"
      contains: "use xcfun_kernels"
    - path: "Cargo.toml"
      provides: "Workspace members updated to include xcfun-kernels"
      contains: "xcfun-kernels"
  key_links:
    - from: "crates/xcfun-eval/src/functional.rs::eval_point_kernel"
      to: "crates/xcfun-kernels/src/dispatch.rs::dispatch_kernel"
      via: "use xcfun_kernels::dispatch::dispatch_kernel"
      pattern: "xcfun_kernels::dispatch"
    - from: "validation/src/driver.rs::run_launch"
      to: "crates/xcfun-kernels/src/dispatch.rs"
      via: "use xcfun_kernels::dispatch"
      pattern: "xcfun_kernels"
    - from: "xtask/src/bin/check_no_mul_add.rs"
      to: "crates/xcfun-kernels/src/functionals"
      via: "scope path expansion in PINNED_DIRS or analogous constant"
      pattern: "xcfun-kernels"
---

<objective>
Resurrect the original docs/design/05-module-responsibilities.md crate layout by extracting `crates/xcfun-kernels/` from `crates/xcfun-eval/` (per D-08). Pure structural reorg — **zero algebraic deltas** because Plan 06-00 (Wave 1) already landed every algebraic change in the current xcfun-eval tree (per D-09).

Migration scope (per CONTEXT.md D-08):
- Move `crates/xcfun-eval/src/functionals/` → `crates/xcfun-kernels/src/functionals/` (78 kernel bodies in `lda/`, `gga/`, `mgga/`).
- Move `crates/xcfun-eval/src/density_vars/` + `density_vars.rs` → `crates/xcfun-kernels/src/density_vars/` + `density_vars.rs` (DensVarsDev<F> + build_densvars + regularize).
- Move `crates/xcfun-eval/src/dispatch.rs` → `crates/xcfun-kernels/src/dispatch.rs` (FunctionalId-keyed comptime if-chain).
- Move tier-1 self-tests `crates/xcfun-eval/tests/self_tests.rs` (and other tests that touch kernel internals) → `crates/xcfun-kernels/tests/`.

`xcfun-kernels` Cargo.toml exposes `pub mod functionals; pub mod density_vars; pub mod dispatch;`. Depends on `xcfun-ad` + `xcfun-core` + `cubecl` core ONLY (no `cubecl-cpu` / `cubecl-hip` / etc — never instantiates a runtime per D-08).

`xcfun-eval` shrinks to: `Functional` struct, `eval_point_kernel` `#[cube(launch_unchecked)]` adapter, `for_tests::cpu_client()` substrate. Continues to depend on `cubecl-cpu` (validation substrate). Imports rewired from `crate::functionals::*` → `xcfun_kernels::functionals::*` etc.

`validation/` driver: import path updates (`xcfun_eval::dispatch::run_launch` → `xcfun_kernels::dispatch::run_launch`).

xtask gate scopes:
- `xtask check-no-mul-add` — scope extends to `crates/xcfun-kernels/src/functionals/**/*.rs` (per Phase 2 D-13 + Pitfall extension).
- `xtask check-no-anyhow` — allowlist extends to include `xcfun-kernels` in the enforced library-graph set.

Purpose: Achieve the design-doc-05 §3 architecture (kernel bodies in their own crate; runtime concerns in xcfun-gpu; per-point validation substrate in xcfun-eval). Per D-09, by landing all substrate work in Plan 06-00 first, this reorg has zero algebraic deltas — any post-mv tier-1 / tier-2 regression is unambiguously a "move bug" (path rename), not a "substrate bug".

Output: New `crates/xcfun-kernels/` crate (Cargo.toml + src/lib.rs + 78+ moved files). `xcfun-eval` shrunk. Workspace `Cargo.toml` updated. xtask gates updated. Tier-1 self-tests GREEN post-move. tier-2 LDA+GGA quick sweep GREEN post-move.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-00-substrate-PLAN.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@Cargo.toml
@crates/xcfun-eval/Cargo.toml
@crates/xcfun-eval/src/lib.rs
@crates/xcfun-eval/src/functional.rs
@crates/xcfun-eval/src/dispatch.rs
@crates/xcfun-eval/src/density_vars.rs
@validation/src/driver.rs
@xtask/src/bin/check_no_mul_add.rs
@xtask/src/bin/check_no_anyhow.rs

<interfaces>
<!-- Existing imports the executor needs to rewire post-move. -->

From crates/xcfun-eval/src/functional.rs (current — top of file):
```rust
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_core::{ALIASES, Dependency, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, ParameterId, Vars, XcError, taylorlen};

use crate::density_vars::DensVarsDev;
use crate::density_vars::build::build_densvars;
use crate::density_vars::DensVarsDevLaunch;
use crate::dispatch;
use crate::dispatch::dispatch_kernel;

#[cube(launch_unchecked)]
fn eval_point_kernel<F: Float>(
    input: &Array<F>, d: &mut DensVarsDev<F>, out: &mut Array<F>,
    #[comptime] id: u32, #[comptime] vars: u32, #[comptime] n: u32,
) {
    build_densvars::<F>(input, d, vars, n);
    dispatch_kernel::<F>(id, d, out, n);
}
```
After move: `use crate::density_vars::*` → `use xcfun_kernels::density_vars::*`; `use crate::dispatch::*` → `use xcfun_kernels::dispatch::*`.

From current Workspace Cargo.toml:
```toml
[workspace]
members = [
    "crates/xcfun-ad",
    "crates/xcfun-core",
    "crates/xcfun-eval",
    "crates/xcfun-rs",
    "crates/xcfun-capi",
    "xtask",
    "validation",
]
exclude = [
    "crates/xcfun-gpu",
    "crates/xcfun-python",
]
```
After: `members` adds `"crates/xcfun-kernels"`. `exclude` unchanged in this plan (Plan 06-02 promotes `xcfun-gpu` to members).
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: git mv functionals + density_vars + dispatch into new xcfun-kernels crate</name>
  <files>Cargo.toml, crates/xcfun-kernels/Cargo.toml, crates/xcfun-kernels/src/lib.rs, crates/xcfun-kernels/src/functionals/, crates/xcfun-kernels/src/density_vars/, crates/xcfun-kernels/src/density_vars.rs, crates/xcfun-kernels/src/dispatch.rs, crates/xcfun-kernels/tests/, crates/xcfun-eval/Cargo.toml, crates/xcfun-eval/src/lib.rs, crates/xcfun-eval/src/functional.rs</files>
  <read_first>
    - Cargo.toml (workspace root — verify members + exclude blocks)
    - crates/xcfun-eval/Cargo.toml (current functionals owner)
    - crates/xcfun-eval/src/lib.rs (current re-export pattern)
    - crates/xcfun-eval/src/functional.rs (current import paths to rewire)
    - crates/xcfun-eval/src/dispatch.rs (the file to git-mv)
    - crates/xcfun-eval/src/density_vars.rs (the file to git-mv)
    - crates/xcfun-eval/src/functionals/ (78 files to git-mv — verify count via `find crates/xcfun-eval/src/functionals -name '*.rs' | wc -l`)
    - crates/xcfun-eval/tests/ (find tests touching kernel internals to migrate)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md §"Plan 06-01" (lines 26-39, 329-441)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Runtime State Inventory" (lines 583-599)
  </read_first>
  <action>
**Step A — Create `crates/xcfun-kernels/Cargo.toml`:**

```toml
[package]
name = "xcfun-kernels"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "Per-functional #[cube] kernel bodies + DensVarsDev + dispatch table for xcfun_rs (runtime-agnostic)"
license = "MPL-2.0"

[features]
default = []
testing = ["xcfun-eval/testing", "dep:cubecl-cpu", "dep:approx", "dep:bincode", "dep:serde"]

[dependencies]
xcfun-core = { path = "../xcfun-core" }
xcfun-ad   = { path = "../xcfun-ad" }
cubecl     = { workspace = true }
thiserror  = { workspace = true }

# Tier-1 self-tests need cubecl-cpu — gated behind `testing`.
cubecl-cpu = { workspace = true, optional = true }

[dev-dependencies]
approx     = { workspace = true }
bincode    = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
xcfun-eval = { path = "../xcfun-eval", features = ["testing"] }   # for cpu_client substrate
```

**No `cubecl-cpu` in default features** per D-08 (kernel bodies don't instantiate runtimes). Tests run via `--features testing` which pulls in cubecl-cpu through xcfun-eval re-export.

**Step B — git-mv tree:**

```bash
# Create destination
mkdir -p crates/xcfun-kernels/src crates/xcfun-kernels/tests

# Move source tree (preserve git history)
git mv crates/xcfun-eval/src/functionals  crates/xcfun-kernels/src/functionals
git mv crates/xcfun-eval/src/density_vars crates/xcfun-kernels/src/density_vars
git mv crates/xcfun-eval/src/density_vars.rs crates/xcfun-kernels/src/density_vars.rs
git mv crates/xcfun-eval/src/dispatch.rs  crates/xcfun-kernels/src/dispatch.rs

# Move tier-1 self-tests + any test that imports kernel internals
git mv crates/xcfun-eval/tests/self_tests.rs crates/xcfun-kernels/tests/self_tests.rs
# Move other tests as needed (regularize_invariant, regularize_mgga_invariant, tpss_tau_clamp from Plan 06-00):
git mv crates/xcfun-eval/tests/regularize_invariant.rs  crates/xcfun-kernels/tests/regularize_invariant.rs
git mv crates/xcfun-eval/tests/regularize_mgga_invariant.rs crates/xcfun-kernels/tests/regularize_mgga_invariant.rs
git mv crates/xcfun-eval/tests/tpss_tau_clamp.rs crates/xcfun-kernels/tests/tpss_tau_clamp.rs
# IMPORTANT: leave tests that use Functional surface (cubecl_spike.rs, cubecl_densvars_spike.rs, alias_canary.rs) in xcfun-eval/tests/ — they exercise the eval entry point.
```

**Step C — Create `crates/xcfun-kernels/src/lib.rs`:**

```rust
//! # xcfun-kernels
//!
//! Per-functional `#[cube] fn` kernel bodies + DensVarsDev<F> + dispatch_kernel.
//! **Runtime-agnostic** — never instantiates a `cubecl::Runtime`; depends only
//! on `cubecl` core. Per Phase 6 D-08 (resurrects design-doc-05 §3).
//!
//! Consumers:
//!   - `xcfun-eval` for per-point CPU validation (via cubecl-cpu).
//!   - `xcfun-gpu` for batch GPU dispatch (via cubecl-hip / cubecl-cuda / cubecl-wgpu).
//!
//! Phase 6 Pitfall 2 — compile-time f64 invariant:
//!   the cubecl-wgpu Wgpu backend silently downgrades to f32 if the device
//!   lacks `wgpu::Features::SHADER_F64`. Compile-time guard:
const _: () = assert!(core::mem::size_of::<f64>() == 8);

pub mod functionals;
pub mod density_vars;
pub mod dispatch;
```

**Step D — Update `crates/xcfun-eval/Cargo.toml`** to depend on xcfun-kernels:

```toml
[dependencies]
xcfun-core    = { path = "../xcfun-core" }
xcfun-ad      = { path = "../xcfun-ad" }
xcfun-kernels = { path = "../xcfun-kernels" }   # NEW per Plan 06-01 D-08
cubecl        = { workspace = true }
cubecl-cpu    = { workspace = true, optional = true }
thiserror     = { workspace = true }
```

**Step E — Rewire `crates/xcfun-eval/src/lib.rs` and `crates/xcfun-eval/src/functional.rs`:**

In `crates/xcfun-eval/src/lib.rs`: remove `pub mod functionals; pub mod density_vars; pub mod dispatch;`. Add re-exports for backward compat:

```rust
//! xcfun-eval — Functional struct + per-point eval entry point + cubecl-cpu validation substrate.
//! Phase 6 D-08: kernel bodies + DensVarsDev + dispatch_kernel migrated to xcfun-kernels.
//!
//! (W-8 revision-1) NO re-export of xcfun_kernels::*. The 11 existing test files in
//! crates/xcfun-eval/tests/ that imported xcfun_eval::functionals::* / dispatch / density_vars
//! are migrated to xcfun_kernels::* as part of this plan (test count < 15 — small enough
//! that a clean import migration is preferable to a long-lived re-export shim).

pub mod functional;
pub mod for_tests;
```

In `crates/xcfun-eval/src/functional.rs`, update the `use` block:
```rust
// OLD:
//   use crate::density_vars::DensVarsDev;
//   use crate::density_vars::build::build_densvars;
//   use crate::density_vars::DensVarsDevLaunch;
//   use crate::dispatch;
//   use crate::dispatch::dispatch_kernel;
// NEW:
use xcfun_kernels::density_vars::DensVarsDev;
use xcfun_kernels::density_vars::build::build_densvars;
use xcfun_kernels::density_vars::DensVarsDevLaunch;
use xcfun_kernels::dispatch;
use xcfun_kernels::dispatch::dispatch_kernel;
```

**Step F — Workspace `Cargo.toml` update:**

```toml
[workspace]
members = [
    "crates/xcfun-ad",
    "crates/xcfun-core",
    "crates/xcfun-kernels",   # NEW Plan 06-01
    "crates/xcfun-eval",
    "crates/xcfun-rs",
    "crates/xcfun-capi",
    "xtask",
    "validation",
]
exclude = [
    "crates/xcfun-gpu",      # Plan 06-02 promotes
    "crates/xcfun-python",   # Phase 7
]
```

Order matters for `cargo build` clarity — xcfun-kernels listed before xcfun-eval (its dependent).

**Step G — Run codebase-wide grep + fixup of import paths:**

```bash
git grep -nE 'xcfun_eval::(functionals|density_vars|dispatch)' -- '*.rs' '!crates/xcfun-eval/**' '!crates/xcfun-kernels/**'
```

Each hit becomes `xcfun_kernels::*`. Expected hits per RESEARCH §"Runtime State Inventory":
- `crates/xcfun-rs/src/functional.rs` — likely 0-2 hits.
- `crates/xcfun-capi/src/lib.rs` — likely 0 hits (uses xcfun_rs facade).
- `validation/src/driver.rs` — likely 1-2 hits (`run_launch`, `dispatch::supports`).
- `xtask` — 0 hits (xtask doesn't import functional bodies).

For each, replace `xcfun_eval::functionals::` → `xcfun_kernels::functionals::`, etc.

**Step H — Per-test path adjustments inside moved test files (W-8: migrate ALL, no re-export shim):**

Each test in `crates/xcfun-kernels/tests/*.rs` that previously did `use xcfun_eval::functionals::*` now does `use xcfun_kernels::functionals::*` (or `use crate::functionals::*` since they're now in the same crate). Same for density_vars and dispatch.

**(W-8 revision-1) Also migrate the 11 test files that STAY in `crates/xcfun-eval/tests/`** (alias_canary.rs, contracted_cross_mode.rs, cubecl_densvars_spike.rs, pack_ctaylor_inputs.rs, potential_gga.rs, potential_lda.rs, potential_parity.rs, regularize_2nd_taylor.rs, regularize_invariant.rs, regularize_mgga_invariant.rs, self_tests.rs) so any `use xcfun_eval::functionals::*` / `dispatch` / `density_vars` becomes `use xcfun_kernels::functionals::*`. Add `xcfun-kernels = { path = "../xcfun-kernels" }` to `crates/xcfun-eval/[dev-dependencies]` if not already present.

After migration, `crates/xcfun-eval/src/lib.rs` exposes ONLY `Functional`, `eval_point_kernel`, `for_tests::cpu_client` — NOT a `pub use xcfun_kernels::*` shim. The tests import `xcfun_kernels::*` directly.

The cubecl-cpu launcher in tests still imports `cubecl_cpu::CpuRuntime` and `xcfun_eval::for_tests::cpu_client` (the cpu_client substrate STAYS in xcfun-eval per D-08).

**Step I — Verification:**

```bash
cargo clean -p xcfun-eval
cargo build --workspace
cargo nextest run --workspace --tests
```

All workspace tests must pass — there should be ZERO algebraic deltas because Plan 06-00 already shipped substrate work in the current xcfun-eval tree. Any test failure here is a "move bug" (path rename) and must be fixed before this plan completes.
  </action>
  <verify>
    <automated>cargo build --workspace && cargo nextest run --workspace --tests</automated>
  </verify>
  <acceptance_criteria>
    - `crates/xcfun-kernels/Cargo.toml` exists with `name = "xcfun-kernels"`.
    - `crates/xcfun-kernels/src/lib.rs` exists with `pub mod functionals;` `pub mod density_vars;` `pub mod dispatch;`.
    - `find crates/xcfun-kernels/src/functionals -name '*.rs' | wc -l` >= 78
    - `find crates/xcfun-eval/src -name 'functionals' -type d` returns empty (functionals/ moved out).
    - `find crates/xcfun-eval/src -name 'dispatch.rs'` returns empty (moved out).
    - `find crates/xcfun-eval/src -name 'density_vars.rs'` returns empty (moved out).
    - `grep -c '"crates/xcfun-kernels"' Cargo.toml` >= 1 (workspace members)
    - `grep -c '"crates/xcfun-gpu"' Cargo.toml` >= 1 (still in `exclude`; Plan 06-02 promotes)
    - `grep -c "xcfun-kernels.*path" crates/xcfun-eval/Cargo.toml` >= 1
    - `grep -c "use xcfun_kernels" crates/xcfun-eval/src/functional.rs` >= 1
    - **(W-8 revision-1)** No re-export shim in xcfun-eval lib.rs: `grep -c "pub use xcfun_kernels" crates/xcfun-eval/src/lib.rs` == 0
    - **(W-8 revision-1)** All 11 test files in crates/xcfun-eval/tests/ that touch kernel internals import via xcfun_kernels::*: `grep -rE 'use xcfun_eval::(functionals|density_vars|dispatch)' crates/xcfun-eval/tests/ | wc -l` == 0
    - `grep -E 'xcfun_eval::(functionals|density_vars|dispatch)' -r --include='*.rs' . | grep -v 'crates/xcfun-eval' | grep -v 'crates/xcfun-kernels' | wc -l` == 0
    - `cargo build --workspace` exits 0.
    - `cargo nextest run --workspace --tests` exits 0 (no regressions; per D-09 zero algebraic deltas means tier-1/tier-2 must remain GREEN).
    - `cargo nextest run -p xcfun-kernels --features testing --test self_tests` exits 0 (tier-1 self-tests at new location).
  </acceptance_criteria>
  <done>xcfun-kernels crate exists as workspace member; all kernel bodies + DensVarsDev + dispatch moved verbatim; xcfun-eval shrunk and depends on xcfun-kernels; workspace builds and full test suite passes.</done>
</task>

<task type="auto">
  <name>Task 2: Update xtask gates + validation harness for new crate boundary</name>
  <files>xtask/src/bin/check_no_mul_add.rs, xtask/src/bin/check_no_anyhow.rs, validation/src/driver.rs, validation/Cargo.toml</files>
  <read_first>
    - xtask/src/bin/check_no_mul_add.rs (full file — current scope/dirs constant)
    - xtask/src/bin/check_no_anyhow.rs (full file — current allowlist + crates-walk)
    - validation/src/driver.rs (find `run_launch` + dispatch imports)
    - validation/Cargo.toml (verify dependency on xcfun-eval; may need xcfun-kernels added)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Cross-cutting xtask gate updates" (lines 117-122)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Project Constraints" (lines 1129-1138)
  </read_first>
  <action>
**Step A — Extend `xtask/src/bin/check_no_mul_add.rs`:**

Find the constant defining the scope (likely `const SCAN_DIRS: &[&str] = &["crates/xcfun-eval/src/functionals"];` or similar). Append `crates/xcfun-kernels/src/functionals` to the array. Final state:

```rust
const SCAN_DIRS: &[&str] = &[
    "crates/xcfun-eval/src/functionals",      // legacy path; will be empty post Plan 06-01 but kept for safety
    "crates/xcfun-kernels/src/functionals",   // Phase 6 Plan 06-01: new home for 78 functional bodies
];
```

(If SCAN_DIRS only had the eval path, replace it; if it has multiple paths, append.)

**Step B — Extend `xtask/src/bin/check_no_anyhow.rs`:**

Find the enforced library-graph crate set (likely a `const LIBRARY_CRATES: &[&str]` or analogous). Add `xcfun-kernels`. The current set should include `xcfun-ad`, `xcfun-core`, `xcfun-eval`, `xcfun-rs`, `xcfun-capi` (per Phase 2 QG-01). Add:

```rust
const LIBRARY_CRATES: &[&str] = &[
    "xcfun-ad",
    "xcfun-core",
    "xcfun-kernels",   // Phase 6 Plan 06-01
    "xcfun-eval",
    "xcfun-rs",
    "xcfun-capi",
    // Plan 06-02 will add "xcfun-gpu"
];
```

If the gate uses `cargo metadata` to walk all workspace crates and excludes `validation` / `xtask` / dev-deps, then no change may be needed — `xcfun-kernels` joining `members` automatically lands in scope. Read the file first to confirm which model the gate uses, then make the corresponding change.

**Step C — Update `validation/src/driver.rs`:**

Search for imports of `xcfun_eval::dispatch` or `xcfun_eval::functionals` or `xcfun_eval::density_vars`. For each:
```rust
// OLD:
//   use xcfun_eval::dispatch::run_launch;
//   use xcfun_eval::dispatch::supports;
// NEW:
use xcfun_kernels::dispatch::run_launch;
use xcfun_kernels::dispatch::supports;
```

(If the validation harness imports via the `xcfun_eval::*` re-export added in Plan 06-01 Task 1 Step E, the import lines may continue to work without change — but prefer direct `xcfun_kernels::*` paths for clarity.)

**Step D — Update `validation/Cargo.toml`:**

Add `xcfun-kernels` as a direct dep if `validation` previously consumed `xcfun_eval::dispatch::*`:

```toml
[dependencies]
xcfun-core    = { path = "../crates/xcfun-core" }
xcfun-eval    = { path = "../crates/xcfun-eval" }
xcfun-kernels = { path = "../crates/xcfun-kernels" }   # NEW Plan 06-01
xcfun-rs      = { path = "../crates/xcfun-rs" }
# ... rest of existing deps ...
```

**Step E — Verify gates run GREEN:**

```bash
cargo run -p xtask --bin check-no-mul-add
cargo run -p xtask --bin check-no-anyhow
cargo run -p xtask --bin check-cubecl-pin
cargo run -p xtask --bin check-no-fma  # if exists
cargo build -p validation --release
```

All must exit 0. The `check-no-mul-add` gate must scan the new `xcfun-kernels/src/functionals/` location and confirm no `mul_add(` calls (Plan 06-00 substrate work landed without introducing any).
  </action>
  <verify>
    <automated>cargo run -p xtask --bin check-no-mul-add && cargo run -p xtask --bin check-no-anyhow && cargo run -p xtask --bin check-cubecl-pin && cargo build -p validation --release</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "xcfun-kernels" xtask/src/bin/check_no_mul_add.rs` >= 1
    - `grep -c "xcfun-kernels\|xcfun_kernels" xtask/src/bin/check_no_anyhow.rs` >= 1 OR (if gate uses cargo-metadata-driven crate walk) gate runs GREEN with xcfun-kernels in scope by virtue of workspace membership.
    - `grep -E 'xcfun_eval::(dispatch|functionals|density_vars)' validation/src/driver.rs | wc -l` == 0 (all rewired to xcfun_kernels::*)
    - `grep -c "xcfun-kernels" validation/Cargo.toml` >= 1
    - `cargo run -p xtask --bin check-no-mul-add` exits 0.
    - `cargo run -p xtask --bin check-no-anyhow` exits 0.
    - `cargo run -p xtask --bin check-cubecl-pin` exits 0.
    - `cargo build -p validation --release` exits 0.
  </acceptance_criteria>
  <done>xtask check-no-mul-add scope extended; xtask check-no-anyhow allowlist extended; validation driver imports rewired; validation builds and all xtask gates GREEN.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-kernels crate boundary | Per D-08 contract: never depends on cubecl-cpu / cubecl-hip / cubecl-cuda / cubecl-wgpu directly — only `cubecl` core |
| xcfun-eval ↔ xcfun-kernels | xcfun-eval depends on xcfun-kernels (one-way); kernels never depend on eval |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-CUBECL-DRIFT | high | New crate `xcfun-kernels` adding `cubecl` dep without lockstep pin would drift | `xcfun-kernels/Cargo.toml` uses `cubecl = { workspace = true }` (inherits `=0.10.0-pre.3` workspace pin); xtask check-cubecl-pin runs in Step E |
| T-06-FAST-MATH | high | Move could accidentally introduce `mul_add` somewhere if executor isn't careful | xtask check-no-mul-add scope extension to `xcfun-kernels/src/functionals/` covers post-move scope; gate runs in Task 2 verify |
| T-06-ANYHOW | medium | xcfun-kernels accidentally depending on anyhow would violate library-graph rule | xtask check-no-anyhow allowlist extension covers; gate runs in Task 2 verify |
| T-06-MOVE-BUG | medium | Any algebraic delta during the move would mask itself as a "substrate bug" — but per D-09, Plan 06-00 already shipped all substrate, so any test failure post-move IS a move-bug, bisectable by `git diff` against the move | Per-D-09 sequencing; full workspace test run in Task 1 verify; tier-2 LDA+GGA quick sweep recommended in success criteria |
| T-06-RUNTIME-LEAK | medium | Adding cubecl-cpu (or hip/cuda/wgpu) to xcfun-kernels Cargo.toml [dependencies] would violate D-08 | Cargo.toml `[dependencies]` block in Task 1 Step A explicitly omits all runtime crates; cubecl-cpu only via `[features].testing` opt-in for tier-1 self-tests |
</threat_model>

<verification>
- All 2 tasks GREEN per their automated commands.
- `cargo build --workspace` succeeds.
- `cargo nextest run --workspace --tests` succeeds (no regressions).
- Per D-09: tier-2 LDA+GGA quick sweep validates no algebraic regression: `cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga' --jobs 4`.
- D-08 boundary preserved: `grep -E 'cubecl-(cpu|hip|cuda|wgpu)' crates/xcfun-kernels/Cargo.toml | grep -v 'optional = true.*testing\|features.*testing'` — should show only `optional = true` deps gated behind `testing` feature, NEVER as default deps.
- xtask gates all GREEN.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 1 advanced: 78 `#[cube]` body home is `xcfun-kernels` (per design-doc-05 §3); compiles unchanged for any cubecl Runtime (cubecl-cpu still GREEN at this point; cubecl-hip / cubecl-cuda / cubecl-wgpu wired in Plans 06-03 / 06-04).
- KER-01 / KER-02 / KER-05 advanced: kernel bodies migrated; DensVarsDev field-order parity preserved (verbatim move); `#[cube]` purity preserved by macro.
- Plan 06-02 unblocked: `xcfun-gpu` can now `use xcfun_kernels::dispatch::dispatch_kernel` and instantiate `Batch<R>` over the kernel bodies in their new home.
- Phase 6 invariants preserved: cubecl pin lockstep; no anyhow in library graph; no mul_add in functional bodies.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-01-SUMMARY.md` documenting:
- Workspace topology change: `members += xcfun-kernels`
- Files moved (78 functional bodies + density_vars/ + dispatch.rs + 4 tier-1 test files)
- Import rewires (count of `xcfun_eval::*` → `xcfun_kernels::*` across rs/capi/validation/xtask)
- xtask gate scope updates (check-no-mul-add, check-no-anyhow)
- Per-D-09 confirmation: zero algebraic deltas (full workspace test suite GREEN; tier-2 LDA+GGA quick sweep GREEN)
- `cargo clean -p xcfun-eval && cargo clean -p xcfun-kernels` recommendation (per RESEARCH §"Runtime State Inventory" Build artifacts row)
</output>
