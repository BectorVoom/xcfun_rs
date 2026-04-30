---
phase: 05
plan: 00
plan_id: 05-00-topology-foundation
subsystem: workspace-topology + xcfun-core error surface + LB94 registry stub
tags: [topology, error-mapping, registry-codegen, capi-foundation]
requires:
  - Phase 4 baseline workspace (xcfun-core / xcfun-eval / validation members)
  - xtask regen-registry binary (Phase 2 Plan 02-02)
provides:
  - workspace topology aligned with Phase 5 D-01/D-02/D-04 (xcfun-rs + xcfun-capi members; xcfun-functionals deleted)
  - XcError::InvalidVarsAndMode variant + as_c_code() i32 mapping (CAPI-05, D-08-A)
  - eval_setup emits the combined GGA-non-2nd-Taylor Mode::Potential variant (XCFunctional.cpp:441-443)
  - FunctionalId::XC_LB94 = 78 + descriptor stub at row 78 (D-16); validation/c_stubs.cpp untouched (LB94 absent in upstream)
affects:
  - Cargo.toml (workspace members/exclude lists)
  - crates/xcfun-capi (renamed from xcfun-ffi)
  - crates/xcfun-rs (new facade skeleton)
  - crates/xcfun-core/src/{error.rs, functional_id.rs}
  - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (regenerated)
  - crates/xcfun-eval/src/functional.rs::eval_setup
  - crates/xcfun-eval/src/functionals/mgga/mod.rs (comment fix)
  - xtask/src/bin/regen_registry.rs (FUNCTIONAL_IDS + LB94 special-cases)
tech-stack:
  added: []
  patterns:
    - "git mv-based crate rename preserves history (xcfun-ffi -> xcfun-capi)"
    - "Rust-only registry id with explicit upstream-C++ exclusion via xtask emit_c_stubs_cpp skip-set (LB94 pattern)"
    - "C ABI error code mapping via thiserror enum method (XcError::as_c_code)"
key-files:
  created:
    - crates/xcfun-rs/Cargo.toml
    - crates/xcfun-rs/src/lib.rs
    - crates/xcfun-capi/Cargo.toml (renamed from xcfun-ffi/Cargo.toml)
    - crates/xcfun-capi/src/lib.rs (renamed from xcfun-ffi/src/lib.rs)
  modified:
    - Cargo.toml
    - crates/xcfun-core/src/error.rs
    - crates/xcfun-core/src/functional_id.rs
    - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (auto-regenerated)
    - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs.sha256
    - crates/xcfun-core/tests/registry_tables.rs
    - crates/xcfun-eval/src/functional.rs
    - crates/xcfun-eval/src/functionals/mgga/mod.rs
    - xtask/src/bin/regen_registry.rs
  deleted:
    - crates/xcfun-functionals/ (8 files — Cargo.toml + src/lib.rs + 6 LDA stubs)
decisions:
  - id: D-01 (Phase 5 CONTEXT)
    description: "Renamed xcfun-ffi crate to xcfun-capi via git mv (history preserved); body unchanged."
  - id: D-02 (Phase 5 CONTEXT)
    description: "Created empty xcfun-rs facade crate skeleton; Plan 05-01 will fill body."
  - id: D-04 (Phase 5 CONTEXT)
    description: "Deleted xcfun-functionals stub crate (dead post-cubecl-pivot artifact)."
  - id: D-08-A (Phase 5 CONTEXT)
    description: "Added XcError::InvalidVarsAndMode variant + as_c_code() method mapping to upstream XC_E* (1/2/4/6/-1)."
  - id: D-16 (Phase 5 CONTEXT)
    description: "LB94 registry stub at FunctionalId 78 with Dependency::DENSITY|GRADIENT; validation/c_stubs.cpp excludes LB94."
metrics:
  duration: ~35m
  completed_date: 2026-04-30
---

# Phase 5 Plan 00: Topology Foundation Summary

Establish workspace topology, the C ABI error-code surface, and LB94 registry coverage that all downstream Phase 5 plans depend on.

## One-line summary

Renamed xcfun-ffi to xcfun-capi, deleted xcfun-functionals, scaffolded xcfun-rs, added XcError::InvalidVarsAndMode + as_c_code mapping, wired eval_setup to emit the combined error, and back-filled the LB94 (id=78) registry stub.

## Tasks completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 0.1 | Workspace topology — rename xcfun-ffi → xcfun-capi, delete xcfun-functionals, register xcfun-rs | `fe1d9b1` | Cargo.toml, crates/xcfun-capi/{Cargo.toml,src/lib.rs}, crates/xcfun-rs/{Cargo.toml,src/lib.rs}, crates/xcfun-functionals/* (deleted) |
| 0.2 | XcError — InvalidVarsAndMode variant + as_c_code() (D-08-A, CAPI-05) | `2c6ee59` | crates/xcfun-core/src/error.rs |
| 0.3 | eval_setup — emit InvalidVarsAndMode for combined-error case (D-08-A) | `7a307c1` | crates/xcfun-eval/src/functional.rs |
| 0.4 | LB94 descriptor add-back per D-16 — extend FunctionalId, regen FUNCTIONAL_DESCRIPTORS, exclude from c_stubs | `5690b76` | crates/xcfun-core/src/functional_id.rs, xtask/src/bin/regen_registry.rs, crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (+ .sha256), crates/xcfun-core/tests/registry_tables.rs, crates/xcfun-eval/src/functionals/mgga/mod.rs |

## Net deltas

### Workspace topology (Task 0.1)

- `git mv crates/xcfun-ffi crates/xcfun-capi` (preserves git history).
- `git rm -r crates/xcfun-functionals` (8 files: Cargo.toml + src/lib.rs + 6 LDA-tier stubs).
- New `crates/xcfun-rs/Cargo.toml` declares `name = "xcfun-rs"`, depends on
  `xcfun-core`, `xcfun-eval` (with `features = ["testing"]`), and `thiserror`;
  `static_assertions` as a dev-dep.
- New `crates/xcfun-rs/src/lib.rs` — single `#![forbid(unsafe_code)]` line plus header.
- Workspace `Cargo.toml` `members` now lists `xcfun-rs` and `xcfun-capi`; the
  deleted/renamed crates are removed from both `members` and `exclude` arrays.
  `xcfun-gpu` and `xcfun-python` remain excluded (Phase 6+/7+).
- Verified: `cargo metadata --no-deps` lists xcfun-capi + xcfun-rs as packages,
  no xcfun-ffi or xcfun-functionals; `cargo check -p xcfun-rs -p xcfun-capi`
  exits 0; both directories absent from disk.

### XcError extension (Task 0.2)

- 1 new variant: `InvalidVarsAndMode { vars: Vars, mode: Mode, depends: Dependency }`,
  inserted between `InvalidMode` and `UnknownName`. Carries Copy fields only,
  preserving `XcError: Copy + Clone + Send + Sync`.
- 1 new method: `pub fn as_c_code(&self) -> i32`. Mapping verbatim from
  `xcfun-master/src/XCFunctional.hpp:40-46` (`XC_EORDER=1, XC_EVARS=2,
  XC_EMODE=4`) plus the combined `XC_EVARS|XC_EMODE = 6`. All other variants
  (UnknownName / NotConfigured / Runtime / InputLengthMismatch /
  OutputLengthMismatch / InvalidEncoding) map to `-1`.
- 11 new unit tests cover every code (1, 2, 4, 6, -1 across all variants);
  `xc_error_still_copy` doc-tests the Copy preservation.
- `cargo test -p xcfun-core --lib error::tests` → 15 passed.

### eval_setup combined-error wiring (Task 0.3)

- Replaced the single `_ => return Err(XcError::InvalidVars { vars, required: GRADIENT })`
  arm at the `Mode::Potential + Dependency::GRADIENT + non-_2ND_TAYLOR` branch with
  `_ => return Err(XcError::InvalidVarsAndMode { vars, mode, depends: deps })`.
  This mirrors `XCFunctional.cpp:441-443` returning `XC_EVARS | XC_EMODE` (= 6).
- Updated existing test `eval_setup_rejects_gga_non_2nd_taylor_potential` to
  assert the new combined variant carries the right (vars, mode, depends).
- Added new test `eval_setup_emits_combined_error_when_gga_potential_with_lda_vars`
  pinpointing the PBEX + Vars::A_B case.
- All other `eval_setup` arms (metaGGA → InvalidMode, laplacian → InvalidMode,
  Mode::Unset → NotConfigured, Mode::Contracted order > 6 → InvalidOrder,
  GGA + 2ND_TAYLOR → Ok) remain byte-identical.
- `cargo test -p xcfun-eval --features testing --lib functional::tests` → 23 passed.

**One-line confirmation:** `eval_setup`'s combined-error branch is reachable and
observable as `Err(XcError::InvalidVarsAndMode { .. })` — verified at
`crates/xcfun-eval/src/functional.rs::eval_setup` (Mode::Potential GGA-non-2ND_TAYLOR arm).

### FunctionalId / FUNCTIONAL_DESCRIPTORS / c_stubs (Task 0.4)

- 1 new variant: `FunctionalId::XC_LB94 = 78`. Documentation comment notes
  the upstream lb94.cpp `#if 0` status and the C-ABI uses string names (no
  numeric collision).
- `FunctionalId::COUNT` bumped to 79 (was 78).
- `from_name` extended with `"LB94"` arm; case-insensitive lookup unchanged.
- 3 new tests in functional_id::tests:
  - `count_is_79` (replaces `count_is_78`)
  - `lb94_discriminant_is_78` (NEW)
  - `from_name_round_trip` extended with three LB94 variants
- 1 new test in registry_tables: `lb94_descriptor_present` asserting
  row 78 carries `FunctionalId::XC_LB94`, `name = "XC_LB94"`,
  `depends = DENSITY | GRADIENT`, no test data.
- `descriptors_count_is_78` renamed to `descriptors_count_is_79`.
- xtask `FUNCTIONAL_IDS` array extended with `"XC_LB94"` (78th entry, fully
  documented in-line). `emit_c_stubs_cpp` now explicitly skips LB94 with a
  `if *id == "XC_LB94" { continue; }` guard. `emit_functional_descriptors_rs`
  special-cases LB94 with `Dependency::DENSITY.union(Dependency::GRADIENT)`
  (extracted from `setup_lb94` macro at `xcfun-master/src/functionals/lb94.cpp:48-50`).
- `FUNCTIONAL_DESCRIPTORS.rs` regenerated via `cargo run -p xtask --bin
  regen-registry`. Now 79 rows; row 78 is the LB94 stub. `.sha256` stamp
  updated.
- `validation/c_stubs.cpp` unchanged (already empty of FUNCTIONAL stubs after
  Phase 4 ports landed; the regen step's LB94 exclusion confirms no drift).
  Verified `! grep -F "FUNCTIONAL(XC_LB94)" validation/c_stubs.cpp` → exits 0
  (LB94 absent).
- `crates/xcfun-eval/src/functionals/mgga/mod.rs` header comment fixed: the
  earlier "LB94 (id=66) deferred to Phase 5" claim was factually wrong — id 66
  is XC_CSC. New text reflects LB94 at id=78 with `XcError::Runtime` eval path.
- `cargo run -p xtask --bin regen-registry -- --check` exits 0 (no drift).
- `cargo test -p xcfun-core --test registry_tables` → 10 passed.
- `cargo test -p xcfun-core --test parameter_and_alias_registry parameter_id_discriminants_match_cpp` → 1 passed (XC_RANGESEP_MU stays at 78 — XC_LB94 absent in upstream C++ enum, so no collision).

**c_stubs.cpp confirmation:** validation/c_stubs.cpp does NOT contain
`FUNCTIONAL(XC_LB94)` — both because the file is hand-tuned (all 67 stubs
were progressively removed by Phase 3 and Phase 4 plans as native ports
landed) and because the xtask `emit_c_stubs_cpp` now explicitly skips LB94
to keep the regenerator in sync with the upstream C++ enum (which lacks XC_LB94
because lb94.cpp is `#if 0`'d).

**parameter_id_discriminants_match_cpp confirmation:** The C++-tied parameter
discriminant test still passes unchanged — `ParameterId::XC_RANGESEP_MU as u32 == 78`,
`XC_EXX == 79`, `XC_CAM_ALPHA == 80`, `XC_CAM_BETA == 81`. These are independent
of the Rust-side `FunctionalId::XC_LB94` because they live in a different enum
type, and in the upstream C++ `list_of_functionals.hpp` XC_LB94 does not exist.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] xcfun-master gitignored, missing in worktree**

- **Found during:** Task 0.4 (running `cargo run -p xtask --bin regen-registry`)
- **Issue:** `xcfun-master/` is gitignored in this project (per `.gitignore`),
  so it does not exist in the parallel-execution git worktree. The xtask
  regen-registry binary requires this directory to invoke its C++ extractor.
- **Fix:** Created a relative symlink
  `xcfun-master -> /home/chemtech/workspace/xcfun_rs/xcfun-master` in the
  worktree root. The symlink is itself gitignored (matched by the existing
  `xcfun-master` entry), so it does not appear in `git status`.
- **Files modified:** none committed; symlink only.
- **Commit:** n/a (symlink not tracked)

**2. [Rule 3 — Blocking] regen-registry write-mode overwrote validation/c_stubs.cpp**

- **Found during:** Task 0.4 (after running `cargo run -p xtask --bin regen-registry`)
- **Issue:** The `validation/c_stubs.cpp` committed in HEAD is hand-tuned —
  Phase 3 / Phase 4 plans progressively removed all 67 non-LDA `FUNCTIONAL(...)`
  stubs as native Rust ports landed. The regen-registry binary in write mode
  always emits the full set (now 67 with LB94 skipped). Running the regen
  binary destroyed the hand-tuned file.
  Notably, the `c_stubs.cpp.sha256` stamp records the regen output's hash
  (5072...) which differs from the actual on-disk file's hash (4bf8...),
  so `regen-registry --check` continues to pass even when the hand-tuned
  file is committed: the drift gate compares the regen output's sha against
  the stamp, not against the file.
- **Fix:** `git checkout HEAD -- validation/c_stubs.cpp` to restore the
  hand-tuned file. The new `c_stubs.cpp.sha256` (= regen output hash with
  LB94 skip in place) is byte-identical to the committed stamp, because the
  regenerator's LB94 exclusion produces the same 67-stub output it always did.
  Drift check exits 0.
- **Files modified:** none (file restored to HEAD state, stamp unchanged).
- **Commit:** n/a (no diff after restoration)

**3. [Rule 1 — Bug fix] regen_registry.rs docstrings stale**

- **Found during:** Task 0.4 (post-edit grep)
- **Issue:** `xtask/src/bin/regen_registry.rs` had two `"78 functionals"` /
  `"78-entry"` docstring constants that the regen binary embeds verbatim
  into the generated `FUNCTIONAL_DESCRIPTORS.rs`. Without updating these,
  the generated file would still claim 78 entries.
- **Fix:** Updated to `"79 functionals"` and `"79-entry registry table
  indexed by FunctionalId as usize (78 upstream + LB94 stub per Phase 5 D-16)"`.
- **Files modified:** xtask/src/bin/regen_registry.rs.
- **Commit:** Folded into Task 0.4 commit `5690b76`.

No Rule 4 (architectural) deviations were needed.

## Self-Check

- All 4 tasks executed and committed individually.
- Workspace builds clean (`cargo check --workspace` exits 0).
- All targeted tests pass:
  - `cargo test -p xcfun-core --lib error::tests` → 15/15.
  - `cargo test -p xcfun-core --lib functional_id::tests` → 5/5.
  - `cargo test -p xcfun-core --test registry_tables` → 10/10.
  - `cargo test -p xcfun-core --test parameter_and_alias_registry` → 13/13.
  - `cargo test -p xcfun-eval --features testing --lib functional::tests` → 23/23.
  - `cargo run -p xtask --bin regen-registry -- --check` → exit 0.
- Topology constraints met: xcfun-ffi + xcfun-functionals deleted from disk,
  xcfun-rs + xcfun-capi listed as workspace packages.

## Self-Check: PASSED

All claimed files exist, all claimed commits exist:

- `fe1d9b1`: feat(05-00): rename xcfun-ffi -> xcfun-capi, delete xcfun-functionals, add xcfun-rs
- `2c6ee59`: feat(05-00): add XcError::InvalidVarsAndMode variant + as_c_code() method
- `7a307c1`: feat(05-00): eval_setup emits InvalidVarsAndMode for combined-error case
- `5690b76`: feat(05-00): add LB94 (id=78) registry stub per Phase 5 D-16

All 4 verified via `git log --oneline | grep <hash>` and corresponding files
verified via `[ -f <path> ]` and content `grep` checks.
