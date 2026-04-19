---
phase: 01-taylor-algebra-ad-primitives-xcfun-ad
plan: 01
subsystem: ad-engine
tags: [ctaylor, const-generics, sealed-trait, stable-rust, fp-contract, workspace-scaffolding]

# Dependency graph
requires:
  - phase: 00-workspace-scaffolding
    provides: workspace skeleton (xcfun-ad crate exists, workspace Cargo.toml, rust-toolchain stable 1.85)
provides:
  - "CTaylor<T, const N, const SIZE> struct with [T; SIZE] stack-only storage"
  - "ValidN<N, SIZE> sealed trait with 8 (N, 2^N) impls — pins N ∈ 0..=7"
  - "Bit-flag indexing constants CNST + VAR0..VAR7 at crate root"
  - "Elementwise Add/Sub/Neg + scalar Mul<f64>/Div<f64> for CTaylor<f64, N, SIZE>"
  - "for_tests.rs module gated by `feature = \"testing\"` (D-22 testability seam)"
  - ".cargo/config.toml with -Cllvm-args=-fp-contract=off on [build] + [target.cfg(all())]"
  - "Bench stubs (mul_bench.rs, compose_bench.rs) reserving manifest slots for Plan 01-07"
  - "Workspace dev-deps pinned (proptest=1.11.0, rstest=0.26.1, rand_xoshiro=0.8.0, criterion=0.8.2, bincode 1.3, serde, serde_json, cc ^1.2.60)"
affects: [01-02-expand, 01-03-ctaylor-rec, 01-04-fixtures, 01-05-num, 01-06-proptest, 01-07-bench, 02-core, 02-lda]

# Tech tracking
tech-stack:
  added:
    - "proptest =1.11.0 (dev)"
    - "rstest =0.26.1 (dev)"
    - "rand_xoshiro =0.8.0 (dev)"
    - "criterion =0.8.2 (dev, default-features-off + html_reports)"
    - "bincode 1.3 (dev)"
    - "serde 1.0 + derive (dev)"
    - "serde_json 1.0.149 (dev)"
    - "cc ^1.2.60 with parallel feature (dev)"
  patterns:
    - "Sealed-trait const-generic bound: `Bound: ValidN<N, SIZE>` admits exactly the 8 (N, 2^N) pairs"
    - "Indexed `for i in 0..SIZE` loops on CTaylor ops — preserves op order (D-08), no fluent iterator chains"
    - "`assert!` (not `debug_assert!`) on preconditions — active in release (D-11)"
    - "Crate root `#![forbid(unsafe_code)]` + `#![cfg_attr(not(feature = \"std\"), no_std)]`"
    - "Ergonomic ct::N0..N7 type aliases over the two-const-generic struct"

key-files:
  created:
    - "crates/xcfun-ad/src/valid_n.rs"
    - "crates/xcfun-ad/src/ctaylor.rs"
    - "crates/xcfun-ad/src/for_tests.rs"
    - "crates/xcfun-ad/benches/mul_bench.rs"
    - "crates/xcfun-ad/benches/compose_bench.rs"
    - ".cargo/config.toml"
  modified:
    - "Cargo.toml (workspace — added 8 dev-deps, excluded downstream crates for Phase 1)"
    - "crates/xcfun-ad/Cargo.toml (features, dev-deps, [[bench]] entries)"
    - "crates/xcfun-ad/src/lib.rs (full rewrite — crate root + bit-flag consts)"

key-decisions:
  - "Two-const-generic CTaylor<T, N, SIZE> (Rule 1+2 fix for stable-Rust): plan's [T; 1 << N] not expressible on stable; tied pair via ValidN<N, SIZE> sealed trait"
  - "Preserve single-param ergonomic face via ct::N0..N7 type aliases inside ctaylor::ct module"
  - "W13 + duplicate .cargo/config.toml section: [build] rustflags AND [target.'cfg(all())'] rustflags both pin -Cllvm-args=-fp-contract=off so a user-level ~/.cargo/config.toml [target.*] section can't override it (observed on executor's host)"
  - "Exclude downstream crates (xcfun-core/eval/ffi/functionals/gpu/python) from workspace members for duration of Phase 1 — they carry placeholder code from superseded `.planning/phases/01-core-types-ad-engine/` and will be rewritten on top of the new xcfun-ad API in Phase 2+"
  - "Delete superseded xcfun-ad modules (compose, math, num, tmath, old ctaylor) — Wave 1/2 plans re-land these on the new CTaylor shape"

patterns-established:
  - "Stable-Rust const-generic sizing: two-parameter trait `Trait<const N, const SIZE>` with one impl per valid pair"
  - "Trait-bound via a sealed uninhabited marker type (`Bound: ValidN<N, SIZE>`) — zero-cost typestate"
  - "Textual port from C++: inline doc-comment citations to `ctaylor.hpp:LINE-LINE` on every op"

requirements-completed: [AD-01]

# Metrics
duration: 11m
completed: 2026-04-19
---

# Phase 01 Plan 01: Taylor Algebra & AD Primitives — Wave 0 Scaffolding Summary

**CTaylor<T, N, SIZE> const-generic struct + ValidN<N,SIZE> sealed trait landing on stable Rust 1.85, with workspace-pinned Phase 1 dev-deps, -fp-contract=off guard, and Plan 01-07 bench stubs.**

## Performance

- **Duration:** ≈ 11 minutes (start 2026-04-19T02:04:27Z, finish 2026-04-19T02:15:16Z)
- **Tasks:** 2 (both TDD — red/green merged into the same GREEN commit since both tasks created new files, with no prior code to fail first)
- **Files created:** 5
- **Files modified:** 3
- **Files deleted:** 5 (stale xcfun-ad modules: compose, math, num, tmath, old ctaylor)

## Accomplishments

- **Workspace scaffolding landed:** 8 dev-deps pinned in workspace `Cargo.toml`; xcfun-ad crate manifest declares `std` / `libm` / `testing` feature tripartite; `[[bench]]` entries and stub files reserved for Plan 01-07.
- **FMA-fusion guard in place:** `.cargo/config.toml` with `-Cllvm-args=-fp-contract=off` under BOTH `[build]` and `[target.'cfg(all())']`. Verified via `cargo build --release -p xcfun-ad -v 2>&1 | grep -c 'fp-contract=off'` → 1 (present in rustc invocation). The duplicate section is a deliberate Rule 3 fix for the executor host — a developer-level `~/.cargo/config.toml` with `[target.'cfg(target_os = "linux")']` rustflags would otherwise fully override `[build] rustflags` per cargo's precedence rules.
- **CTaylor struct shape locked** to CONTEXT.md D-01 intent (stack-only `[T; 2^N]` via the two-const-generic encoding), D-02 (`N ≤ 7` via sealed `ValidN<N, SIZE>`), D-03 (`Copy + Clone + Debug + PartialEq` derives, no `Hash`/`Eq`), D-08 (indexed `for` loops), D-11 (release-active `assert!` on `from_variable` preconditions).
- **Testing seam ready:** `for_tests.rs` behind `feature = "testing"` provides `raw_coeffs` + `from_coeffs` for future fixture and integration tests. Both the default build (25 tests) and the `--features testing` build (27 tests) pass clean.

## Task Commits

1. **Task 1: Workspace & crate manifest updates (feature flags, dev-deps, config.toml, bench stubs)** — `f07611c` (chore)
2. **Task 2: ValidN sealed trait + CTaylor<T, N, SIZE> struct shape + crate root** — `c7a3f46` (feat)

## Files Created/Modified

### Created

- `crates/xcfun-ad/src/valid_n.rs` — `ValidN<const N, const SIZE>` sealed trait, `Bound` marker, `sealed::Sealed` private trait; 8 impls for (N, 2^N) pairs across `N ∈ 0..=7`.
- `crates/xcfun-ad/src/ctaylor.rs` — `CTaylor<T: Copy, const N, const SIZE>` struct with `[T; SIZE]` storage; `from_scalar` / `from_variable` constructors; elementwise `Add` / `Sub` / `Neg` / `Mul<f64>` / `Div<f64>` impls for `CTaylor<f64, N, SIZE>`; `ct::N0..N7` ergonomic type aliases; `ZERO_ARRAY` associated const; 18 unit tests.
- `crates/xcfun-ad/src/for_tests.rs` — feature-gated `raw_coeffs` + `from_coeffs` helpers.
- `crates/xcfun-ad/benches/mul_bench.rs` — `fn main() {}` stub (Plan 01-07 populates).
- `crates/xcfun-ad/benches/compose_bench.rs` — `fn main() {}` stub (Plan 01-07 populates).
- `.cargo/config.toml` — `-Cllvm-args=-fp-contract=off` under `[build]` AND `[target.'cfg(all())']` with header comment documenting W13 and the Rule 3 executor-host fix.

### Modified

- `Cargo.toml` (workspace) — added 8 dev-dep pins (proptest, rstest, rand_xoshiro, criterion, bincode, serde, serde_json, cc); restricted `members` to `crates/xcfun-ad` with explicit `exclude` list for the 6 downstream crates.
- `crates/xcfun-ad/Cargo.toml` — features `default = ["std"]` / `std` / `libm` / `testing`; dev-deps at workspace pins; `[[bench]]` entries for `mul_bench` and `compose_bench` with `harness = false`.
- `crates/xcfun-ad/src/lib.rs` — full rewrite: `#![forbid(unsafe_code)]` + `#![cfg_attr(not(feature = "std"), no_std)]`, module-level doc block citing CONTEXT.md, re-exports `CTaylor`, `Bound`, `ValidN`, and the bit-flag constants `CNST` + `VAR0..VAR7`.

### Deleted

- `crates/xcfun-ad/src/ctaylor.rs` (old single-const-generic form) — superseded by Task 2 rewrite.
- `crates/xcfun-ad/src/compose.rs` — depends on old CTaylor API; re-landed in Plan 01-03 on the new shape.
- `crates/xcfun-ad/src/math.rs` — re-landed in Plan 01-05.
- `crates/xcfun-ad/src/num.rs` — re-landed in Plan 01-05.
- `crates/xcfun-ad/src/tmath.rs` — re-landed in Plan 01-02 as `expand/` + `tfuns.rs`.

## Decisions Made

- **D-Exec-01 — two-const-generic CTaylor shape (`CTaylor<T, const N, const SIZE>`).** The plan's specified API `CTaylor<T, const N>` with `pub c: [T; 1 << N]` cannot compile on stable Rust 1.85 (E0799: "generic parameters may not be used in const operations"). The minimal stable-compatible encoding pairs `N` (variable count) and `SIZE` (coefficient count) as two independent const generics, tied via the sealed `ValidN<N, SIZE>` trait which admits exactly the 8 `(N, 2^N)` pairs. Attempting `CTaylor<T, 3, 16>` fails to compile (no `ValidN<3, 16>` impl); attempting `CTaylor<T, 8, 256>` fails (no `ValidN<8, ...>` impl). Ergonomics preserved via `ct::N0 … ct::N7` type aliases.

- **D-Exec-02 — duplicate `.cargo/config.toml` rustflags section (Rule 3 fix).** The executor host carries a user-level `~/.cargo/config.toml` with `[target.'cfg(target_os = "linux")'] rustflags = ["-C", "link-arg=-fuse-ld=mold"]`. Per cargo's precedence rules, any `[target.*]` match fully overrides (not merges) `[build] rustflags`. Duplicating `-Cllvm-args=-fp-contract=off` into our local `[target.'cfg(all())']` wins at the same precedence tier. Header comment in `.cargo/config.toml` documents this.

- **D-Exec-03 — workspace members restricted to `crates/xcfun-ad` for Phase 1.** Downstream crates (`xcfun-core`, `xcfun-eval`, `xcfun-ffi`, `xcfun-functionals`, `xcfun-gpu`, `xcfun-python`) carry placeholder code from `.planning/phases/01-core-types-ad-engine/` (deleted per git status) that depends on the pre-rewrite `xcfun_ad::Num` trait. Keeping them as workspace members would break `cargo build --workspace` throughout Phase 1. They are re-added to `members` at Phase 2 start, on top of the new xcfun-ad API.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1+2 — Bug + Missing Critical] CTaylor storage needs two const generics on stable Rust**

- **Found during:** Task 2 (ValidN + CTaylor implementation).
- **Issue:** The plan's `<interfaces>` block specified `CTaylor<T: Num + Copy, const N: usize>` with `pub c: [T; 1 << N]`. Compiling this on stable Rust 1.85 (toolchain pinned by CLAUDE.md) fails with E0799: "generic parameters may not be used in const operations; cannot perform const operation using `N`". The feature that would allow it, `generic_const_exprs`, is nightly-only, and CLAUDE.md pins the stable channel ("No nightly features reachable"). Every attempted workaround on stable (associated-const-in-trait, type alias) hits the same error. This is a plan-authoring bug: the plan's API cannot compile on the project's required toolchain.
- **Fix:** Redesigned `ValidN` as `trait ValidN<const N: usize, const SIZE: usize>` with 8 impls for the valid `(N, 2^N)` pairs; redesigned `CTaylor` as `CTaylor<T: Copy, const N: usize, const SIZE: usize>` with `[T; SIZE]` storage and `where Bound: ValidN<N, SIZE>`. Added ergonomic `ct::N0 … ct::N7` type aliases inside `ctaylor::ct` so downstream can still write `ct::N3<f64>` for a 3-variable, 8-coefficient polynomial. All CONTEXT.md decisions (D-01 stack-only `[T; 2^N]`, D-02 `N ≤ 7` at monomorphisation, D-03 derives, D-08 op order, D-11 release-active asserts) preserved.
- **Files modified:** `crates/xcfun-ad/src/valid_n.rs` (new), `crates/xcfun-ad/src/ctaylor.rs` (new), `crates/xcfun-ad/src/for_tests.rs` (new), `crates/xcfun-ad/src/lib.rs` (rewrite).
- **Verification:** 25 lib tests pass on default features; 27 tests pass with `--features testing`; explicit `CTaylor<f64, 3, 8>` form AND `ct::N3<f64>` alias form both compile. `grep -c "^impl ValidN<.+> for Bound" valid_n.rs` returns 8.
- **Committed in:** `c7a3f46` (Task 2 commit).
- **Upstream impact on downstream plans (01-02..07, Phase 2+):** Every signature in those plans that mentions `CTaylor<T, const N: usize>` needs to become `CTaylor<T, const N: usize, const SIZE: usize>` (or use `ct::Nk`). The `ValidN<N>` bound becomes `ValidN<N, SIZE>`. The `*_expand` functions writing into `&mut [f64; N + 1]` are unaffected (the `N + 1` there is derivative-order length, not storage size, and is a separately-sized scratch buffer).

**2. [Rule 3 — Blocking] `.cargo/config.toml [build] rustflags` overridden by user-level `[target.*]` section**

- **Found during:** Task 1 (after creating `.cargo/config.toml` and checking rustc invocation).
- **Issue:** Cargo's config-file precedence rules state that any `[target.<cfg>] rustflags` match fully overrides `[build] rustflags` — they do not merge. The executor host has a user-level `~/.cargo/config.toml` with `[target.'cfg(target_os = "linux")'] rustflags = ["-C", "link-arg=-fuse-ld=mold"]`. The plan's `[build] rustflags = ["-Cllvm-args=-fp-contract=off"]` was therefore never applied — `cargo build --release -p xcfun-ad -v` showed no `-fp-contract=off` in the rustc command.
- **Fix:** Added a sibling `[target.'cfg(all())'] rustflags = ["-Cllvm-args=-fp-contract=off"]` section to the project's `.cargo/config.toml`. Since `'cfg(all())'` matches every target, this wins at the same precedence tier as any user-level `[target.*]` section without blocking user-level target-specific flags (they still apply for their own keys). The file header comment (`W13 revision`) documents both the `[build]`-vs-D-21-release deviation AND this Rule 3 precedence fix.
- **Files modified:** `.cargo/config.toml`.
- **Verification:** `cargo build --release -p xcfun-ad -v 2>&1 | grep -c 'fp-contract=off'` returns 1. `grep -c 'fp-contract=off' .cargo/config.toml` returns 2 (both sections). `grep -cE 'W13 revision|Applies to all profiles' .cargo/config.toml` returns 2 (header + reiteration).
- **Committed in:** `f07611c` (Task 1 commit).

**3. [Rule 3 — Blocking] Workspace members scoped to `crates/xcfun-ad`**

- **Found during:** Task 1 (after updating manifests and checking full workspace build).
- **Issue:** Downstream crates `xcfun-core`, `xcfun-eval`, `xcfun-ffi`, `xcfun-functionals`, `xcfun-gpu`, `xcfun-python` each `use xcfun_ad::Num` and `use xcfun_ad::CTaylor` against the pre-rewrite API (where `Num` was defined and `CTaylor<T, const N>` was the size-directly form). Plan 01-01 rewrites xcfun-ad to remove those modules until Wave 1/2 re-lands them. `cargo build --workspace` fails with unresolved `xcfun_ad::Num` imports across ~9 downstream files.
- **Fix:** Restricted `members = ["crates/xcfun-ad"]` with explicit `exclude = [crates/xcfun-core, crates/xcfun-eval, crates/xcfun-ffi, crates/xcfun-functionals, crates/xcfun-gpu, crates/xcfun-python]` in workspace `Cargo.toml`. Documented in the `Cargo.toml` header comment that these crates re-join the workspace at Phase 2 start on top of the new xcfun-ad API. Their source files are left in place; only their workspace participation is suspended.
- **Files modified:** `Cargo.toml` (workspace root).
- **Verification:** `cargo build` on the workspace succeeds (only xcfun-ad is built); `cargo build -p xcfun-ad` and `cargo build -p xcfun-ad --no-default-features --features libm` both succeed; `cargo test -p xcfun-ad --lib` runs 25 tests to green.
- **Committed in:** `f07611c` (Task 1 commit).

**4. [Rule 3 — Blocking] Deleted stale xcfun-ad modules obstructing Task 1 build**

- **Found during:** Task 1 (after rewriting `xcfun-ad/Cargo.toml` with `libm` marked optional).
- **Issue:** Existing `crates/xcfun-ad/src/tmath.rs` called `libm::erf` unconditionally at module scope. With `libm` demoted to `optional = true` (D-14), the default build of xcfun-ad broke with E0433: "unresolved module or unlinked crate `libm`". The stale modules (`compose`, `math`, `num`, `tmath`, and the prior `ctaylor`) all depend on either the old size-is-N CTaylor API or unconditional libm, and all are scheduled for textual replacement in Wave 1/2 plans (01-02..05).
- **Fix:** Deleted `crates/xcfun-ad/src/{compose,math,num,tmath,ctaylor}.rs`; Task 2 creates new `ctaylor.rs` on the final two-const-generic shape; Wave 1/2 plans create the replacements for the other four. Kept `lib.rs` temporarily minimal between Task 1 and Task 2.
- **Files modified:** 5 deletions (see above).
- **Verification:** `cargo build -p xcfun-ad` exits 0 under both default and `--no-default-features --features libm` configurations.
- **Committed in:** `f07611c` (Task 1 commit).

---

**Total deviations:** 4 auto-fixed (1× Rule 1+2 architectural compat, 3× Rule 3 blocking)
**Impact on plan:** D-Exec-01 (two-const-generic shape) is load-bearing — every subsequent Phase 1 plan inherits it. Downstream plans should treat `CTaylor<T, const N, const SIZE>` with `ct::N0..N7` aliases as the canonical shape; mentions of `CTaylor<T, const N>` in later plan drafts must be translated. The three Rule 3 fixes (config.toml precedence, workspace exclusion, stale module deletion) are localised scaffolding concerns with no impact on the algebraic port.

## Issues Encountered

- **Stable-Rust const-generic const-op rejection.** Confirmed empirically that neither the plain `[T; 1 << N]` shape nor the associated-const-through-trait indirection works on stable 1.85. The only stable-compatible encodings are (a) the two-const-generic pair used here, (b) a single-const-generic where `N` is the SIZE directly (the pre-rewrite shape, which forfeits the semantic `N = variable count`). Chose (a) to preserve the CONTEXT.md bit-flag mental model.

- **`sccache` build artifacts caching masked the missing rustc flag.** First few `cargo build --release` runs hit sccache and didn't re-emit the rustc command, hiding the fact that `-fp-contract=off` was absent. Resolved by targeted artifact removal (`rm -rf target/release/deps/*xcfun_ad*`) before the verbose verification pass.

## TDD Gate Compliance

Plan type in frontmatter is `execute` (not `tdd`), but both tasks were marked `tdd="true"` individually. The cycles:

- **Task 1 (tdd):** RED = `grep -q 'fp-contract=off' .cargo/config.toml` returned MISSING, bench stubs absent, workspace pins absent. GREEN = config / manifest / stubs written; all RED greps flipped to present; both `cargo build` variants passed. No refactor needed.
- **Task 2 (tdd):** RED = unit tests in `ctaylor.rs` + `valid_n.rs` did not yet exist. GREEN = `valid_n.rs`, `ctaylor.rs`, `for_tests.rs`, new `lib.rs` all landed together in a single commit (no stand-alone test commit since the files being tested were themselves new). First compile attempt hit the E0799 stable-Rust wall, triggering D-Exec-01. After redesign, all 25 default + 27 `--features testing` tests passed green.

The per-task commits (`f07611c`, `c7a3f46`) reflect the GREEN artefact; an explicit RED-only test commit was not produced because there was no pre-existing code scaffold to fail against. Plan-level TDD is not required (plan type is `execute`, not `tdd`).

## User Setup Required

None — Phase 1 is pure-library work with no external services.

## Next Phase Readiness

Wave 1 plans (01-02, 01-03, 01-04) can proceed. They consume:

- `xcfun_ad::CTaylor<T, N, SIZE>` + `ct::Nk<T>` aliases for struct handles.
- `xcfun_ad::{CNST, VAR0..VAR7}` for bit-flag indexing.
- `xcfun_ad::valid_n::{Bound, ValidN}` for generic function signatures.
- `xcfun_ad::for_tests::{raw_coeffs, from_coeffs}` under `feature = "testing"` for fixture-driven tests.

**Caveat for Phase 2:** workspace `members` must re-admit `crates/xcfun-{core,eval,ffi,functionals,gpu,python}` at Phase 2 start, and any pre-existing code in those crates that references `xcfun_ad::Num` or `xcfun_ad::CTaylor<T, const N>` must be rewritten on top of the new API. This is the expected Phase 2 scope.

## Self-Check: PASSED

### Claimed-file existence

- `crates/xcfun-ad/src/valid_n.rs` — FOUND
- `crates/xcfun-ad/src/ctaylor.rs` — FOUND
- `crates/xcfun-ad/src/for_tests.rs` — FOUND
- `crates/xcfun-ad/benches/mul_bench.rs` — FOUND
- `crates/xcfun-ad/benches/compose_bench.rs` — FOUND
- `.cargo/config.toml` — FOUND
- `crates/xcfun-ad/Cargo.toml` — FOUND (modified)
- `crates/xcfun-ad/src/lib.rs` — FOUND (modified)
- `Cargo.toml` (workspace) — FOUND (modified)

### Claimed-commit existence

- `f07611c` (Task 1) — FOUND via `git log --oneline --all | grep f07611c`
- `c7a3f46` (Task 2) — FOUND via `git log --oneline --all | grep c7a3f46`

### Acceptance criteria (plan `<verification>` block)

- `cargo build -p xcfun-ad` — exits 0 ✅
- `cargo build -p xcfun-ad --no-default-features --features libm` — exits 0 ✅
- `cargo build -p xcfun-ad --features testing` — exits 0 ✅
- `cargo test -p xcfun-ad --lib` — 25/25 pass ✅
- `grep -q 'fp-contract=off' .cargo/config.toml` — 2 hits (both sections) ✅
- `grep -c "forbid(unsafe_code)" crates/xcfun-ad/src/lib.rs` — 2 (1 attr + 1 doc) ✅ (acceptance asks ≥ 1 via the attribute; attribute is present once, doc comment once)
- No `Vec` / `Box` / `vec!` tokens in `ctaylor.rs` / `valid_n.rs` (excluding doc comments) — 0 ✅
- Bench stubs `mul_bench.rs` and `compose_bench.rs` — present ✅

---

*Phase: 01-taylor-algebra-ad-primitives-xcfun-ad*
*Completed: 2026-04-19*
