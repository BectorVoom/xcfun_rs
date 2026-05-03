---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 02b
subsystem: testing
tags: [validation, cli, harness, tier-3, xcfun-gpu, backend-dispatch, mpmath, erf, documentation]

# Dependency graph
requires:
  - phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
    provides: "xcfun-gpu skeleton (Plan 06-02a) — Backend enum (5 variants), Batch<'fun, R>, Batch::<CpuRuntime>::open_cpu / eval_vec_host_cpu, BackendTag bridge, Functional::settings_generation"
provides:
  - "validation harness CLI extended with --tier {2|3} (default 2 preserves Phase 2-5 behaviour)"
  - "validation harness CLI extended with --reference {cpp|mpmath} (default cpp; mpmath bails — Plan 06-N2 wires)"
  - "validation harness CLI extended with --exclude-erf boolean filter (consumed by Plan 06-04 Wgpu 1e-9 sweep, GPU-08)"
  - "validation harness --backend extended to accept rocm | hip | cuda | wgpu | metal in addition to cpu"
  - "validation::driver::run_tier3(backend, order, jobs, filter, exclude_erf) entry point — Cpu arm scoped for Plan 06-05; Rocm/Cuda/Wgpu/Metal arms bail with feature-flag hints + Plan-number wiring map"
  - "B-5 documentation alignment — CONTEXT.md D-06 amended with revision-1 correction (Metal-via-Wgpu); REQUIREMENTS.md GPU-02 (Backend enum 5 variants) + GPU-07 (ROCm primary) wording updated"
affects:
  - "Plan 06-03 — cubecl-hip primary wiring fills the Backend::Rocm arm of run_tier3 (under --features hip)"
  - "Plan 06-04 — cubecl-cuda + cubecl-wgpu opt-in fills Backend::{Cuda,Wgpu,Metal} arms"
  - "Plan 06-05 — RS-08 Functional::eval_vec dispatch + KER-06 sign-off command (calls cargo run -p validation -- --tier 3 --backend cpu --order 3 --filter '.*' for the 17-functional bar)"
  - "Plan 06-N2 — mpmath fixture loader fills the --reference mpmath branch"

# Tech tracking
tech-stack:
  added:
    - "validation depends on xcfun-gpu (default-features = false, features = [\"cpu\"]) — first time the validation crate enters the GPU dispatch graph"
    - "validation depends on cubecl-cpu (workspace pin) — required by future Plan 06-05 Batch<CpuRuntime> path; declared now to avoid a Wave-4-followup dep churn"
    - "validation gains [features] block forwarding hip / cuda / wgpu / metal to xcfun-gpu (Plans 06-03 / 06-04 wire)"
  patterns:
    - "Backend::from_str dispatch in validation driver — CLI string → typed Backend → match arm; symmetric (#[cfg(not(feature = X))] / #[cfg(feature = X)]) bail-with-Plan-number arms keep the unwired path loud and the wired path discoverable"
    - "tier-3 driver skeleton today, KER-06 body in Plan 06-05 — todo!() in the Cpu arm body intentionally panics at runtime so accidental 'tier-3 was already green before 06-05' is impossible to mistake"
    - "B-5 audit-trail amendment pattern: ### Amended on revision-1 block immediately after the original D-06 paragraph, preserving original text verbatim above"

key-files:
  modified:
    - "validation/src/main.rs — parse --tier / --reference / --exclude-erf; extend --backend; tier-3 dispatch path"
    - "validation/src/driver.rs — pub fn run_tier3 + Backend dispatch + missing settings_gen: 0 init in two pre-existing struct literals (Rule 3 deviation)"
    - "validation/Cargo.toml — xcfun-gpu + cubecl-cpu deps + [features] forwards"
    - ".planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md — D-06 amendment block (revision-1 correction)"
    - ".planning/REQUIREMENTS.md — GPU-02 / GPU-07 wording updates"

key-decisions:
  - "run_tier3 Cpu arm body is todo!() in Plan 06-02b — KER-06 sign-off command + 17-functional 1e-13 bar OWNED by Plan 06-05 per revision-1 B-4. The skeleton compiles, parses CLI flags, and dispatches Backend correctly; the body panics so accidental 'already green' is impossible."
  - "Metal arm uses --features wgpu (NOT a phantom --features metal) — Metal is reached via cubecl-wgpu's Metal adapter per RESEARCH §R-02 / Pitfall 9; the metal Cargo feature in xcfun-gpu is a transparent alias of wgpu (already shipped in Plan 06-02a)"
  - "validation gains an [features] block (was empty before) so downstream Plans 06-03 / 06-04 can run `cargo run -p validation --features hip --tier 3 --backend rocm` once their cubecl runtime arms land — without needing further Cargo.toml edits in this plan"
  - "B-5 amendment preserves original D-06 text — audit-trail rule per CONTEXT.md author intent; the new ### Amended on revision-1 block is a separate sub-section, not an in-place replacement"
  - "--reference mpmath bails immediately with a Plan 06-N2 hint rather than no-op'ing; the harness exits 1 (CLI error) and the user sees the exact next step. Better than silent-pass for an unwired feature."

patterns-established:
  - "Wave-4-style Cargo dependency forward: validation gains a `[features]` block forwarding hip / cuda / wgpu / metal to xcfun-gpu — sets up the harness for Plans 06-03 / 06-04 / 06-05 without further Cargo.toml churn"
  - "Audit-trail amendment block: ### Amended on revision-1 immediately after the original decision paragraph, preserving original text verbatim — useful pattern for future revision cycles where decisions need correction without losing the original justification"
  - "Skeleton-with-todo!() driver: Plan 06-02b ships the API surface + dispatch + acceptance gates without claiming the requirement (KER-06) it scaffolds — claim-ownership stays with the plan that lands the body (Plan 06-05)"

requirements-completed:
  - GPU-02
  - GPU-06
# NOTE: KER-06 explicitly NOT claimed (revision-1 B-4 — ownership passes to Plan 06-05).

# Metrics
duration: 18m
completed: 2026-05-03
---

# Phase 6 Plan 02b: validation harness CLI extension + B-5 documentation alignment Summary

**Validation harness gains tier-3 cross-backend dispatch CLI (--tier / --reference / --exclude-erf / --backend extended) + run_tier3 driver skeleton wired for Plans 06-03 / 06-04 / 06-05; CONTEXT.md D-06 amended with revision-1 Metal-via-Wgpu correction; REQUIREMENTS.md GPU-02 / GPU-07 aligned with locked decisions D-05 / D-06 / D-07.**

## Performance

- **Duration:** ~18 min (Task 1 + Task 2 atomic commits + SUMMARY)
- **Completed:** 2026-05-03
- **Tasks:** 2 (CLI extension + driver skeleton in Task 1; documentation alignment in Task 2)
- **Files modified:** 5 (3 code, 2 docs)
- **Commits:** 2 task + 1 metadata = 3 total in this plan window

## Accomplishments

- **Validation harness CLI extended** with four new/extended flags:
  - `--tier {2|3}` — default 2 preserves Phase 2-5 cc-vs-Rust behaviour. `--tier 3` dispatches to `run_tier3` (cross-backend Batch<R> parity skeleton).
  - `--reference {cpp|mpmath}` — default `cpp`. `mpmath` bails immediately with a Plan 06-N2 hint until the mpmath fixture loader lands.
  - `--exclude-erf` — boolean filter for ERF-bearing functionals; consumed by Plan 06-04 for the Wgpu 1e-9 sweep per GPU-08.
  - `--backend` — extended to accept `cpu | rocm | hip | cuda | wgpu | metal` in addition to the previous `cpu`-only acceptance.

- **`validation::driver::run_tier3(backend, order, jobs, filter, exclude_erf)`** entry point shipped:
  - `Backend::Cpu` arm: skeleton `todo!()` body — KER-06 sign-off command + 17-functional 1e-13 bar OWNED by Plan 06-05 (revision-1 B-4). The skeleton parses CLI, dispatches Backend correctly, and exits Ok on the Cpu arm structure path — the panic surfaces only when the body is actually invoked.
  - `Backend::Rocm`: bails `--backend rocm requires --features hip (Plan 06-03 wires the cubecl-hip arm)`.
  - `Backend::Cuda`: bails `--backend cuda requires --features cuda (Plan 06-04 wires the cubecl-cuda arm)`.
  - `Backend::Wgpu`: bails `--backend wgpu requires --features wgpu (Plan 06-04 wires the cubecl-wgpu arm)`.
  - `Backend::Metal`: bails `--backend metal requires --features wgpu (Metal is reached via cubecl-wgpu's Metal adapter per RESEARCH R-02 / Pitfall 9; Plan 06-04 wires the wgpu arm)`.
  - Symmetric `#[cfg(feature = X)]` arms (when those features land in 06-03/06-04) bail with `Plan 06-03/04 wires this arm` so the wiring map is discoverable in the source.

- **`validation/Cargo.toml`** gains:
  - `xcfun-gpu = { default-features = false, features = ["cpu"] }` dependency.
  - `cubecl-cpu` workspace dep (used by future Plan 06-05 Batch<CpuRuntime> path).
  - `[features]` block forwarding `hip` / `cuda` / `wgpu` / `metal` to `xcfun-gpu/{hip,cuda,wgpu,metal}`.

- **B-5 documentation alignment (revision-1)**:
  - `06-CONTEXT.md` D-06 amended with `### Amended on revision-1 (2026-04-30)` block clarifying Metal-via-Wgpu (no separate `cubecl-metal` crate exists). Original D-06 paragraph preserved verbatim above the amendment for audit-trail.
  - `REQUIREMENTS.md` GPU-02: `Backend` enum updated from `(Cpu, Cuda, Wgpu)` to `(Cpu, Rocm, Cuda, Metal, Wgpu)` with explicit `auto_backend()` priority chain matching CONTEXT.md D-07.
  - `REQUIREMENTS.md` GPU-07: `Tier-3 parity on CUDA` updated to `Tier-3 parity on ROCm (PRIMARY per CONTEXT.md D-05) — CUDA + Metal are opt-in best-effort per D-06`.

## Task Commits

Each task was committed atomically with `--no-verify` (parallel-executor protocol):

1. **Task 1: validation harness CLI extension + run_tier3 driver skeleton** — `8b52e3f` (feat)
2. **Task 2: B-5 documentation alignment** — `8a96097` (docs)

## Files Created/Modified

### Modified (code)

- `validation/src/main.rs` — added `--tier` / `--reference` / `--exclude-erf` parsing; extended `--backend` accepted-values list; added tier-3 dispatch path that returns from `main()` when `tier == 3`.
- `validation/src/driver.rs` — added `pub fn run_tier3(backend, order, jobs, filter, exclude_erf) -> anyhow::Result<()>`; added Backend match arms with `#[cfg(not(feature = X))]` symmetry; added missing `settings_gen: 0` initialiser to two pre-existing `Functional` struct literals (lines 418 + 519, deviation Rule 3).
- `validation/Cargo.toml` — added `xcfun-gpu` + `cubecl-cpu` deps; added `[features]` block forwarding to xcfun-gpu feature flags.

### Modified (docs)

- `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` — D-06 amendment block inserted between D-06 original paragraph and D-07.
- `.planning/REQUIREMENTS.md` — GPU-02 + GPU-07 wording updates (single-line edits).

## Decisions Made

- **Cpu arm body is `todo!()`** — Plan 06-02b ships the API surface only; the actual KER-06 sign-off (10k-grid sweep at strict 1e-13) is OWNED by Plan 06-05 per revision-1 B-4. The skeleton compiles, parses CLI, and dispatches Backend correctly. Calling the function with `--backend cpu --tier 3` panics at the body — intentional so a future executor cannot mistake "skeleton landed" for "tier-3 sweep is green".
- **Metal arm uses `--features wgpu`** — Metal is reached via cubecl-wgpu's Metal adapter (RESEARCH §R-02 / Pitfall 9); the metal Cargo feature in xcfun-gpu is a transparent alias of wgpu. The validation harness `[features] metal = ["xcfun-gpu/metal"]` forwards correctly.
- **`--reference mpmath` bails fast** — does NOT silently no-op or fall through to the cpp path; exits 1 with a Plan 06-N2 hint. Loud failure on unwired feature.
- **B-5 amendment preserves original D-06 text** — the audit-trail intent of CONTEXT.md (which dates back to Phase 0) is to keep the decision history intact even when later corrections are needed; the `### Amended on revision-1` block is a separate sub-section, NOT an in-place replacement.
- **GPU-02 acceptance grep needed plain-text variants** — the plan's literal grep `"Cpu, Rocm, Cuda, Metal, Wgpu"` requires a comma-separated plaintext list (no backticks). REQUIREMENTS.md GPU-02 is rendered with backticks in MD inline code (`Cpu`); the executor stripped backticks from the variant list to satisfy the grep contract while still rendering well in markdown.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing `settings_gen: 0` to two pre-existing `Functional` struct literals**
- **Found during:** Task 1 (cargo check after CLI extension).
- **Issue:** Plan 06-02a added `settings_gen: u64` field to `xcfun_eval::Functional` and (per its SUMMARY) updated the test files in `crates/xcfun-eval/tests/*.rs`. It did NOT update the two `Functional { ... }` struct literals in `validation/src/driver.rs` (lines 418 + 519). After Plan 06-02b's Cargo.toml edits caused validation to be re-checked, the workspace `cargo check -p validation` failed with `error[E0063]: missing field 'settings_gen' in initializer of 'Functional'` at both sites. Pure blocking issue from the prior plan; no scope change.
- **Fix:** Added `settings_gen: 0,` to both struct literals (the validation harness never calls `set()` on these leaked `Functional`s, so the counter is irrelevant for tier-2 parity). Inline comments document the deviation source.
- **Files modified:** `validation/src/driver.rs` (two locations).
- **Verification:** `cargo check -p validation` now compiles cleanly (post-build.rs-stub workaround for syntax verification only — see "Issues Encountered").
- **Committed in:** `8b52e3f` (Task 1 commit).

**2. [Rule 3 - Blocking] Plain-text variant list in REQUIREMENTS.md GPU-02 to satisfy literal grep**
- **Found during:** Task 2 (acceptance criteria grep).
- **Issue:** Plan's acceptance grep is `grep -c "Cpu, Rocm, Cuda, Metal, Wgpu" .planning/REQUIREMENTS.md` — exact string with no backticks. Initial GPU-02 edit used markdown inline-code backticks per variant (`` `Cpu`, `Rocm`, `Cuda`, `Metal`, `Wgpu` ``), which fails the literal grep. Other grep contracts in the plan use the same plaintext form, so this matters.
- **Fix:** Replaced the backticked variant list with a plain-text comma-separated form `(Cpu, Rocm, Cuda, Metal, Wgpu)` while keeping the surrounding `Backend` and `auto_backend()` in code-formatting where the literal-grep contract didn't reach. Reads cleanly in both rendered Markdown and grep.
- **Files modified:** `.planning/REQUIREMENTS.md`.
- **Verification:** `grep -c "Cpu, Rocm, Cuda, Metal, Wgpu" .planning/REQUIREMENTS.md` returns `1`.
- **Committed in:** `8a96097` (Task 2 commit).

---

**Total deviations:** 2 auto-fixed (both Rule 3 — blocking issues from prior-plan oversight + acceptance-grep formatting).
**Impact on plan:** Zero scope creep. Both fixes were prerequisites for the plan's acceptance gates to pass at all.

## Issues Encountered

- **`build.rs` requires vendored `xcfun-master/` C++ sources, not present in this worktree.** The validation crate's `build.rs` calls `fs::copy("../xcfun-master/api/xcfun.h", ...)` and fails with `Os { code: 2, kind: NotFound }`. To verify Rust syntax + type compilation of the changes, the executor temporarily set `build = false` in `validation/Cargo.toml`, ran `cargo check -p validation` (GREEN, including tests), then reverted the Cargo.toml change before commit. The `build = false` toggle never made it into a commit. Documented in this Summary so a future executor can repeat the verification trick if working without xcfun-master locally. Per the executor prompt's `<parallel_execution>` block, this is the expected workflow when xcfun-master is unavailable: tier-3 wiring lands in this plan; full sweep deferred to a backend-specific plan that has hardware available.

- **Pre-build cleanup:** xcfun-eval lib unit tests (23 tests) and xcfun-gpu integration tests (~20 tests across 6 test files) all GREEN before AND after this plan's edits, confirming no regression in the library graph. The validation harness lib also compiles cleanly with the build.rs-stubbed workaround.

## User Setup Required

None — no external service configuration required for this plan.

## Next Phase Readiness

- **Plan 06-03** (cubecl-hip primary wiring): consumes the `Backend::Rocm` arm of `run_tier3` directly. The harness today bails with `--backend rocm requires --features hip (Plan 06-03 wires the cubecl-hip arm)`; 06-03 just replaces the `#[cfg(not(feature = "hip"))] Backend::Rocm` arm body with the actual `Batch::<HipRuntime>::eval_vec_host` call.
- **Plan 06-04** (cubecl-cuda + cubecl-wgpu opt-in): same shape for `Backend::Cuda`, `Backend::Wgpu`, `Backend::Metal` — replace the bail arms with concrete `Batch::<R>::eval_vec_host` calls. The `metal = ["xcfun-gpu/metal"]` Cargo forward is already in place.
- **Plan 06-05** (RS-08 `Functional::eval_vec` + KER-06 sign-off): owns the actual body of `run_tier3`'s Cpu arm. Replaces the `todo!()` with the 10k-grid sweep + per-record `|batch − scalar| / max(|scalar|, 1.0) ≤ 1e-13` gate. The CLI dispatch and Cargo deps already in place; 06-05 only edits driver.rs.
- **Plan 06-N2** (mpmath fixture pipeline): implements the `--reference mpmath` branch. Today it bails with the Plan 06-N2 hint; 06-N2 replaces that bail with a fixture-loader that reads `validation/fixtures/mpmath/*.jsonl` and uses those records as ground truth instead of cc-vs-Rust.

### Known Stubs

- `run_tier3 Backend::Cpu` body is `todo!()` — intentional per revision-1 B-4 (KER-06 owned by Plan 06-05). Calling the function with `--backend cpu --tier 3` panics at runtime; this is the loud-failure semantics chosen so accidental "tier-3 sweep was already green before 06-05" is impossible to mistake. Plan 06-05's first task is to replace the `todo!()` with the actual body.
- `--reference mpmath` bails immediately. Plan 06-N2 wires.
- All non-Cpu Backend arms bail with feature-flag hints. Plans 06-03 / 06-04 wire.

## Self-Check: PASSED

- Files exist:
  - `validation/src/main.rs` — modified, FOUND, contains `--tier` / `--reference` / `--exclude-erf` parsers (8 / 6 / 5 grep matches).
  - `validation/src/driver.rs` — modified, FOUND, contains `run_tier3` (3 grep matches) and `Backend::Cpu` (2 grep matches).
  - `validation/Cargo.toml` — modified, FOUND, contains `xcfun-gpu` (9 grep matches including the dep + features forwards).
  - `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` — modified, FOUND, contains `### Amended on revision-1` (1 match) and `metal.*alias of.*wgpu` (1 match) and original `cubecl-metal = ` (3 matches preserved).
  - `.planning/REQUIREMENTS.md` — modified, FOUND, contains `Cpu, Rocm, Cuda, Metal, Wgpu` (1 match) and `Tier-3 parity on ROCm` (1 match).

- Commits exist:
  - `8b52e3f` (Task 1 — validation CLI extension + run_tier3 skeleton) — FOUND in `git log --oneline -3`.
  - `8a96097` (Task 2 — B-5 documentation alignment) — FOUND in `git log --oneline -3`.

- All acceptance criteria from `<acceptance_criteria>` for Tasks 1 + 2 pass — verified inline before commit.

- xtask gates GREEN (workspace excluding validation builds clean; xcfun-eval lib tests 23/23 GREEN; xcfun-gpu tests pass).

---

*Phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu*
*Completed: 2026-05-03*
