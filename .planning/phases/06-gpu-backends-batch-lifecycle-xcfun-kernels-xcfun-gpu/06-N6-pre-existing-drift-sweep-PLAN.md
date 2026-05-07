---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N6
type: execute
wave: 1
gap_closure: true
depends_on: []
files_modified:
  # Tentative; expanded per task. fmt task touches dozens of files in
  # crates/xcfun-ad/src/. clippy task touches the named functional files.
  # MSRV task touches Cargo.toml + Cargo.lock + CLAUDE.md. xcfun-master
  # task touches .github/workflows/ci.yml plus possibly a vendor dir.
  - Cargo.toml
  - Cargo.lock
  - CLAUDE.md
  - .github/workflows/ci.yml
  - crates/xcfun-ad/src/   # mass-fmt
  - crates/xcfun-kernels/src/functionals/mgga/tpss_like.rs
  - crates/xcfun-kernels/src/functionals/mgga/m0x_like.rs
  - crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs
  - crates/xcfun-kernels/src/functionals/mgga/scan_like.rs
  - crates/xcfun-kernels/src/functionals/gga/br_like.rs
  - crates/xcfun-kernels/src/functionals/gga/brx.rs
autonomous: false
requirements: []
tags: [drift, ci, fmt, clippy, msrv, gap-closure, follow-up-from-06-N5]
must_haves:
  truths:
    - "GitHub Actions CI master pipeline reaches GREEN end-to-end on `cargo build --workspace`, `cargo nextest run --workspace`, `cargo fmt --all -- --check`, and `cargo clippy --workspace --all-targets -- -D warnings` with no `continue-on-error` escape valves."
    - "MSRV contract is internally consistent: either Cargo.toml's `rust-version` matches what `Cargo.lock` actually requires, or transitive deps in Cargo.lock are pinned (via `cargo update --precise`) to the highest versions still compatible with the documented MSRV."
    - "The `validation` crate and `xcfun-capi::headers_match::capi_header_matches_xcfun_master` test reach a deterministic state on CI — either by vendoring a pinned snapshot of the necessary `xcfun-master/` subset, by adding a CI step that clones `xcfun-master/` at the documented HEAD `a89b783`, or by feature-gating the affected build/test paths so they only run when `xcfun-master/` is present."
    - "No `continue-on-error: true` remains on the fmt or clippy jobs in `.github/workflows/ci.yml` after this plan completes."
  artifacts:
    - path: ".github/workflows/ci.yml"
      provides: "Strict CI pipeline — fmt/clippy/build/test/smoke all hard-fail on regression; xcfun-master/ availability handled deterministically (vendored, cloned, or gated)"
      forbids: "continue-on-error: true on any required job"
    - path: "Cargo.toml"
      provides: "MSRV (`rust-version`) consistent with Cargo.lock's actual rustc requirement"
    - path: "Cargo.lock"
      provides: "Transitive dep versions consistent with the workspace MSRV; if MSRV is held at 1.85, transitive deps are downgraded to MSRV-compatible versions; if MSRV is bumped, the bump is documented in CLAUDE.md"
  key_links:
    - from: ".github/workflows/ci.yml"
      to: "deterministic xcfun-master/ resolution"
      via: "either vendored `vendor/xcfun-master-snapshot/` checked into the repo, OR a `git clone https://github.com/dftlibs/xcfun.git --depth 1 -b a89b783 xcfun-master` step before the build job"
      pattern: "xcfun-master|vendor/xcfun-master"
    - from: "Cargo.toml"
      to: "Cargo.lock"
      via: "rust-version field matches actual MSRV of all transitive deps"
      pattern: "rust-version = "
---

<objective>
Sweep the four pre-existing repo-drift items that GitHub Actions CI surfaced
when Plan 06-N5's first CI run executed against a clean Linux runner. None
of these items were caused by 06-N5 — all four predate it — but 06-N5's
CI bring-up was the first time they were exposed. Address them as a
follow-up gap-closure plan so CI can hard-gate the master branch from
this point on.

The four items, in order of severity:

  1. **MSRV vs Cargo.lock mismatch (HIGH).** `Cargo.toml` pins
     `rust-version = "1.85"` (per CLAUDE.md "required for current
     const-generic features"), but the current `Cargo.lock` transitively
     pulls in `cubecl-zspace 0.10.0-pre.3` (rustc 1.92), `darling 0.23`
     (1.88), `icu_*@2.2.0` (1.86), `sysinfo 0.38.4` (1.88), and
     `tracel-llvm` / `tracel-mlir-rs` (1.87). Local builds work because
     the developer's local rustc is newer than the documented MSRV.
     CI on stable works as a workaround but the MSRV contract is broken.

  2. **`xcfun-master/` not available on CI (HIGH).** The `validation/`
     crate's `build.rs` compiles `xcfun-master/src/**/*.cpp` for the
     parity-test harness, and the `xcfun-capi::headers_match` test
     reads `xcfun-master/api/xcfun.h` for the cbindgen drift check.
     `xcfun-master/` is `.gitignore`d and only present locally when the
     operator manually checks it out (per CLAUDE.md: "xcfun-master
     restored at HEAD `a89b783`"). On CI, neither the `validation` build
     nor the `headers_match` test can succeed. 06-N5 worked around this
     by adding `--exclude validation` to the CI build/test commands and
     accepting `headers_match` as a known failure; that workaround is
     unsustainable.

  3. **fmt drift (MEDIUM).** `cargo fmt --check` reports diffs across
     dozens of files in `crates/xcfun-ad/src/` (notably `ctaylor.rs`,
     `ctaylor_rec/{compose,mul,multo}.rs`, `expand/{br_inverse,cbrt}.rs`,
     plus more). Mechanical fix; large diff. Currently advisory in CI.

  4. **clippy unused-import / dead-code (MEDIUM).** Six pre-existing
     warnings flagged by VS Code rustc and confirmed by `cargo clippy`:

       - `crates/xcfun-kernels/src/functionals/mgga/tpss_like.rs:19` — unused `ctaylor_exp`
       - `crates/xcfun-kernels/src/functionals/mgga/m0x_like.rs:35-36` — unused `M0X_ALPHA_X_F64`, `M0X_SCALEFACTOR_TF_F64`
       - `crates/xcfun-kernels/src/functionals/mgga/m0x_like.rs:44` — dead `M0X_CF_F64`
       - `crates/xcfun-kernels/src/functionals/gga/br_like.rs:28` — unused `ctaylor_log`
       - `crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs:19` — unused `ctaylor_sqrt`
       - `crates/xcfun-kernels/src/functionals/gga/brx.rs:29` — unused `ctaylor_exp`
       - `crates/xcfun-kernels/src/functionals/mgga/scan_like.rs:209` — dead `AGE_C2_MU`

     Per-file triage required: distinguish "stale import after refactor
     → delete" from "intentionally retained for future kernel body →
     `#[allow(unused_imports)]` with comment".

**Why this is `autonomous: false`:** Items 1 and 2 are *design decisions*
that only the operator can make — bumping MSRV vs downgrading lockfile
is a project-policy call (CLAUDE.md says 1.85 is required), and choosing
between vendoring vs cloning vs feature-gating xcfun-master/ is an ops
call (vendoring adds repo bloat; cloning adds CI runtime; feature-gating
loses the cbindgen drift check on CI). Items 3 and 4 are mechanical
once the policy is set.
</objective>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N5-SUMMARY.md
@.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-REVIEW.md
@CLAUDE.md
@.github/workflows/ci.yml
@Cargo.toml
@validation/build.rs
@validation/Cargo.toml
@crates/xcfun-capi/tests/headers_match.rs
@crates/xcfun-kernels/src/functionals/mgga/tpss_like.rs
@crates/xcfun-kernels/src/functionals/mgga/m0x_like.rs
@crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs
@crates/xcfun-kernels/src/functionals/mgga/scan_like.rs
@crates/xcfun-kernels/src/functionals/gga/br_like.rs
@crates/xcfun-kernels/src/functionals/gga/brx.rs

<reference_ci_runs>
- 25526643150 (first run, MSRV-1.85 toolchain, all jobs failed)
- 25526734591 (stable toolchain, build still failed on validation/xcfun-master)
- 25526864865 (`--exclude validation`, build+smoke green, 2/375 nextest failures)
- f61afa2 (pbex test fix rolled into 06-N5; expected to fix 1 of the 2 nextest failures)
</reference_ci_runs>
</context>

<tasks>

## Task 1 — MSRV-vs-Cargo.lock policy decision (CHECKPOINT — operator input required)

**Two paths**, both compatible with the 1e-12 accuracy contract:

  **Path A — Hold MSRV at 1.85; downgrade transitive deps.**
  Use `cargo update --precise <max-msrv-1.85-compatible-version>` for
  each offending dep:
    - `cubecl-zspace` → highest 0.10.0-pre.X (or alternative 0.9.X) compatible with 1.85
    - `darling`, `darling_core`, `darling_macro` → 0.22.X
    - `icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`, `icu_properties`, `icu_properties_data`, `icu_provider` → 2.1.X
    - `sysinfo` → 0.38.3 or 0.37.X
    - `tracel-llvm`, `tracel-llvm-bundler`, `tracel-mlir-rs`, `tracel-mlir-rs-macros`, `tracel-mlir-sys`, `tracel-tblgen-rs` → highest 1.85-compatible
  Risk: Some deps may not have an MSRV-1.85-compatible release at all, in
  which case Path A degenerates into vendoring.

  **Path B — Bump MSRV to 1.92.**
  Update `Cargo.toml` `rust-version`, update CLAUDE.md (Tech Stack
  section) to document the bump, ensure `rust-toolchain.toml` (if added)
  pins to 1.92+. Risk: Loses the documented "MSRV 1.85" compatibility
  promise to downstream library consumers.

**Operator chooses A or B.** Then implement the chosen path; verify
`cargo build --workspace --locked` succeeds on the chosen toolchain.

**Acceptance:**
- `cargo build --workspace --locked` on the chosen MSRV exits 0
- CI `cargo build` job uses `--locked` again (drop the workaround comment from 06-N5)

---

## Task 2 — xcfun-master/ on CI strategy (CHECKPOINT — operator input required)

**Three paths**, listed by long-term sustainability:

  **Path A — Vendor a pinned subset.**
  Add `vendor/xcfun-master-snapshot/` to the repo, containing only the
  files the validation crate's build.rs and headers_match test need
  (api/xcfun.h, src/**/*.cpp, src/**/*.hpp, etc.) at HEAD `a89b783`.
  Pros: Reproducible, offline-capable, fast CI. Cons: Repo bloat
  (xcfun-master/ ~10MB), license review (MPL-2.0 inheritance ok per
  CLAUDE.md), update workflow needed when xcfun-master/ moves.

  **Path B — Add a CI clone step.**
  Pre-build step: `git clone https://github.com/dftlibs/xcfun.git --depth 1 ../xcfun-master && git -C ../xcfun-master checkout a89b783`.
  Pros: No repo bloat, single source of truth. Cons: CI depends on
  upstream availability, ~2-5 sec extra per run, network egress.

  **Path C — Feature-gate.**
  Make `validation` build and `headers_match` test conditional on
  presence of `XCFUN_MASTER_DIR` env var or a `cfg(local_xcfun_master)`
  feature. Pros: No bloat, no CI dep. Cons: Loses cbindgen drift gate
  on CI (the whole point of headers_match is catching upstream xcfun.h
  drift; gating it defeats that).

**Recommendation:** Path B (clone step) for CI; keep Path C as a
fallback for offline development. Decision is operator's.

**Acceptance:**
- `validation` crate is no longer `--exclude`d in CI
- `xcfun-capi::headers_match::capi_header_matches_xcfun_master` runs
  and passes on CI
- 06-N5's `--exclude validation` workaround is removed from
  `.github/workflows/ci.yml`

---

## Task 3 — cargo fmt --all (mechanical)

```bash
cargo fmt --all
```

Review the diff to confirm no semantic changes (rustfmt is
deterministic and only adjusts whitespace + line breaks). Commit as
single chunk with explicit `style:` prefix:

```
style: cargo fmt --all (sweep pre-existing drift surfaced by 06-N5 CI)
```

**Acceptance:**
- `cargo fmt --all -- --check` exits 0 (no diffs)
- CI fmt job loses `continue-on-error: true` and remains green

---

## Task 4 — clippy unused-import / dead-code triage (per-file)

For each of the 7 warning sites, determine intent:

  - If the import / constant was added for a kernel body that hasn't
    been written yet (placeholder for future work) — annotate with
    `#[allow(unused_imports)]` or `#[allow(dead_code)]` and a one-line
    comment explaining what's expected to consume it.
  - If the import / constant is genuinely stale (refactor leftover) —
    delete it.

**Files (each gets a small commit):**

  1. `crates/xcfun-kernels/src/functionals/mgga/tpss_like.rs:19` — `ctaylor_exp`
  2. `crates/xcfun-kernels/src/functionals/mgga/m0x_like.rs:35-36, :44` — `M0X_*`
  3. `crates/xcfun-kernels/src/functionals/gga/br_like.rs:28` — `ctaylor_log`
  4. `crates/xcfun-kernels/src/functionals/mgga/tpsslocc.rs:19` — `ctaylor_sqrt`
  5. `crates/xcfun-kernels/src/functionals/gga/brx.rs:29` — `ctaylor_exp`
  6. `crates/xcfun-kernels/src/functionals/mgga/scan_like.rs:209` — `AGE_C2_MU`

**Acceptance:**
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0
- CI clippy job loses `continue-on-error: true` and remains green

---

## Task 5 — Tighten CI

After Tasks 1-4 are landed, restore the strict CI configuration:

  - Re-add `--locked` to `cargo build` and `cargo nextest run`
  - Drop `--exclude validation` from build / clippy / nextest
  - Drop `continue-on-error: true` from `fmt` and `clippy` jobs
  - Drop the explanatory NOTE comments about the workarounds

**Acceptance:**
- One green CI run on master with all 5 jobs strict
- The `06-N5` mpmath-smoke job continues to pass

</tasks>

<deviation_rules>
- If Path A (hold MSRV at 1.85) turns out to be impossible because at
  least one dep lacks an MSRV-1.85-compatible release, escalate to
  operator before silently switching to Path B.
- If Path B (CI clone step) for xcfun-master is selected and the upstream
  repo URL is wrong (`https://github.com/dftlibs/xcfun.git` is a guess
  based on convention; verify before pushing the workflow change),
  escalate to operator with the corrected URL.
- Task 4 per-file triage: when in doubt, prefer `#[allow]` + comment
  over deletion. Deletion is one-way; allow + comment surfaces the
  intent for the next refactor.
</deviation_rules>

<success_criteria>
- [ ] Task 1 — MSRV decision made, implementation landed, `--locked` build green
- [ ] Task 2 — xcfun-master/ strategy chosen, validation + headers_match green on CI
- [ ] Task 3 — `cargo fmt --all -- --check` exits 0
- [ ] Task 4 — `cargo clippy --workspace --all-targets -- -D warnings` exits 0
- [ ] Task 5 — Strict CI configuration restored, one green run on master
- [ ] All commits atomic; each task lands as 1-3 commits with clear prefixes (`style:`, `chore:`, `fix:`, `ci:`)
- [ ] 06-N6-SUMMARY.md created with which paths were chosen for Tasks 1+2 and the rationale
</success_criteria>
