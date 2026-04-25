---
phase: 01-taylor-algebra-ad-primitives-xcfun-ad
plan: 01
subsystem: ad-engine
tags: [cubecl, cubecl-cpu, taylor-algebra, ad, scaffolding, xtask, mlir-jit, pivot]

# Dependency graph
requires:
  - phase: 00-discovery
    provides: "C++ reference vendored at xcfun-master/; docs/design/ brief; CLAUDE.md tech-stack pins"
provides:
  - "Pre-pivot hand-Rust commits (217af4d, f07611c, c7a3f46, 1b95fe3, 2db557c) reverted per D-21/D-22 (non-destructive — original SHAs remain in history)"
  - "Workspace Cargo.toml pinned to cubecl =0.10.0-pre.3 + cubecl-cpu =0.10.0-pre.3 (lockstep pins per CLAUDE.md risk note)"
  - "xcfun-ad crate manifest: features default=[cpu], cpu=[dep:cubecl-cpu] (D-26), testing=[] (D-22)"
  - "Crate root src/lib.rs with #![forbid(unsafe_code)] + module stubs"
  - "src/index.rs with host-visible u32 consts CNST/VAR0..VAR7 verbatim from ctaylor.hpp:12-20"
  - "src/for_tests/cpu_client.rs — OnceLock<CpuClient> singleton per D-17 (CpuClient = ComputeClient<CpuRuntime>)"
  - "src/for_tests/raw_eval_scalar.rs — 1-thread kernel launcher helper per D-16"
  - "tests/cubecl_spike.rs — 4 spike tests, all green (add_two_numbers_on_cpu, client_singleton_is_shared, raw_eval_scalar_roundtrip, index_consts_match_cpp)"
  - "xtask workspace member: src/main.rs + src/bin/regen_ad_fixtures.rs skeleton + assets/regen_ad_fixtures/driver.cpp C++ stub"
  - "Planning-artifact hygiene: 01-RESEARCH.md SUPERSEDED banner; 01-VALIDATION.md nyquist_compliant=true + 6/6 sign-off ticks; stale @01-RESEARCH.md refs scrubbed from 01-02/01-03 plans"
affects: [01-02-ctaylor-rec, 01-03-expand, 01-04-tfuns, 01-05-fixtures, 01-06-math, 01-07-bench, 02-core, 06-gpu]

# Tech tracking
tech-stack:
  added:
    - "cubecl =0.10.0-pre.3 (kernel DSL)"
    - "cubecl-cpu =0.10.0-pre.3 (MLIR-JIT CpuRuntime backend)"
    - "bincode 1.3 / serde / serde_json (fixture wire format — dev-deps)"
    - "cc ^1.2.60 (xtask C++ driver compile)"
    - "anyhow 1.0.102 (xtask app-boundary; not in library graph)"
  patterns:
    - "Pure #[cube] AD layer — no parallel hand-Rust scalar implementation (D-01)"
    - "OnceLock<CpuClient> singleton for test binaries (D-17)"
    - "Launcher closures take owned Handle per cubecl 0.10-pre.3 API (owned-handle pattern)"
    - "Feature-gated testing seam: `testing` feature + `#[cfg(feature = \"testing\")]` on for_tests module (D-22)"

key-files:
  created:
    - "crates/xcfun-ad/src/lib.rs"
    - "crates/xcfun-ad/src/index.rs"
    - "crates/xcfun-ad/src/for_tests/mod.rs"
    - "crates/xcfun-ad/src/for_tests/cpu_client.rs"
    - "crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs"
    - "crates/xcfun-ad/tests/cubecl_spike.rs"
    - "xtask/Cargo.toml"
    - "xtask/src/main.rs"
    - "xtask/src/bin/regen_ad_fixtures.rs"
    - "xtask/assets/regen_ad_fixtures/driver.cpp"
  modified:
    - "Cargo.toml (workspace)"
    - "Cargo.lock"
    - "crates/xcfun-ad/Cargo.toml"
    - ".planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-01-SUMMARY.md (SUPERSEDED banner)"
    - ".planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-RESEARCH.md (SUPERSEDED banner)"
    - ".planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-VALIDATION.md (nyquist=true, sign-off ticks)"
    - ".planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-02-PLAN.md (stale @ref → NOTE)"
    - ".planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-03-PLAN.md (stale @ref → NOTE)"

key-decisions:
  - "CpuClient is `ComputeClient<CpuRuntime>` — `Runtime` trait has no `type Client` in cubecl 0.10-pre.3 (plan <interfaces> delta)"
  - "raw_eval_scalar takes owned Handle in launcher closure — cubecl 0.10-pre.3 `ArrayArg::from_raw_parts` consumes handle (not &handle)"
  - "Kernels use `CubeDim::new_3d(1,1,1)` — `CubeDim::new(x,y,z)` signature repurposed as `new<R>(&client, working_units)` in 0.10-pre.3"
  - "SUMMARY file named 01-01-CUBECL-PIVOT-SUMMARY.md per plan mandate (D-22 preserves pre-pivot 01-01-SUMMARY.md as historical)"

patterns-established:
  - "Pattern: cubecl 0.10-pre.3 API delta commenting — every API-surface divergence from the plan's <interfaces> block is documented inline in the file that adapts to the real API (cpu_client.rs, raw_eval_scalar.rs, cubecl_spike.rs module headers)"
  - "Pattern: non-destructive revert per D-22 — `git revert --no-commit` chain + preserve-list via `git checkout HEAD -- <path>` + single additive revert commit. Original SHAs remain in history."

requirements-completed: [AD-01]

# Metrics
duration: 14min
completed: 2026-04-19
---

# Phase 1 Plan 01: Cubecl-cpu Substrate Proof + for_tests Harness Summary

**Cubecl-cpu 0.10-pre.3 MLIR-JIT substrate probed green on this host; xcfun-ad scaffolded clean for the cubecl-native rewrite; xtask member wired with C++ driver stub.**

## Performance

- **Duration:** ~14 min
- **Started:** 2026-04-19T08:40:46Z
- **Completed:** 2026-04-19T08:54:31Z
- **Tasks:** 4
- **Files created:** 10
- **Files modified:** 7 (Cargo.toml, Cargo.lock, crates/xcfun-ad/Cargo.toml, 4 planning docs)

## Accomplishments

- Reverted 5 pre-pivot commits (217af4d → f07611c → c7a3f46 → 1b95fe3 → 2db557c) non-destructively per D-21/D-22. `git show c7a3f46` still resolves.
- Pinned cubecl =0.10.0-pre.3 + cubecl-cpu =0.10.0-pre.3 at the workspace level; `cargo check --workspace --all-targets` builds clean.
- Added `xtask` workspace member with `regen-ad-fixtures` binary skeleton and a C++ driver stub (`driver.cpp`) that references the three taylor headers (`ctaylor.hpp`, `ctaylor_math.hpp`, `tmath.hpp`).
- Wrote the repo's **first `#[cube] fn` kernel** in `tests/cubecl_spike.rs` — `add_two_scalars::<f64>` lowers through cubecl-cpu's MLIR JIT and returns `7.0` for `3.0 + 4.0`. CONTEXT.md D-01/D-03 substrate risk retired on this machine.
- Established the `for_tests` harness (OnceLock<CpuClient>, raw_eval_scalar helper) that every downstream plan (01-02..01-07) will consume.
- Closed the planning-artifact hygiene gap flagged by the checker: `01-RESEARCH.md` carries a SUPERSEDED banner; `01-VALIDATION.md` has `nyquist_compliant: true` + all 6 sign-off checkboxes ticked; stale `@01-RESEARCH.md` references in 01-02/01-03 replaced with inline NOTE pointers.

## Task Commits

Each task was committed atomically:

1. **Pre-execute housekeeping** — `a4f4fc5` (docs: planning cleanup to give `git revert` a clean base)
2. **Task 1: Revert pre-pivot + mark SUMMARY superseded** — `6d13128` (revert: 5-commit combined revert)
3. **Task 2: Workspace/xcfun-ad manifests + xtask skeleton** — `0f599bd` (chore: cubecl workspace scaffolding)
4. **Task 3: lib.rs + index.rs + for_tests harness + spike tests** — `038c8af` (feat: first working #[cube] fn)
5. **Task 4: Planning-artifact hygiene pass** — `4d6022b` (chore: SUPERSEDED banners, nyquist flag, @ref scrub)

## Files Created/Modified

### Created

- `crates/xcfun-ad/src/lib.rs` — crate root; `#![forbid(unsafe_code)]`; re-exports CNST/VAR0..VAR7
- `crates/xcfun-ad/src/index.rs` — host-visible bit-flag index constants ported from `ctaylor.hpp:12-20`
- `crates/xcfun-ad/src/for_tests/mod.rs` — `#[cfg(feature = "testing")]` module root for test helpers
- `crates/xcfun-ad/src/for_tests/cpu_client.rs` — `OnceLock<CpuClient>` singleton (D-17)
- `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` — 1-thread kernel launcher wrapper (D-16)
- `crates/xcfun-ad/tests/cubecl_spike.rs` — 4 spike tests proving the cubecl-cpu substrate
- `xtask/Cargo.toml` — app-boundary workspace member (anyhow + cc allowed)
- `xtask/src/main.rs` — dispatch stub
- `xtask/src/bin/regen_ad_fixtures.rs` — Plan 05 wires this up; Plan 01 ships the skeleton
- `xtask/assets/regen_ad_fixtures/driver.cpp` — minimal C++ main that includes the three taylor headers
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-01-CUBECL-PIVOT-SUMMARY.md` — this document

### Modified

- `Cargo.toml` — workspace members = [crates/xcfun-ad, xtask]; 10 library + dev pinned deps; cubecl =0.10.0-pre.3 lockstep pin
- `Cargo.lock` — regenerated against new pins
- `crates/xcfun-ad/Cargo.toml` — cubecl-native crate manifest; libm retired
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-01-SUMMARY.md` — prepended SUPERSEDED banner (D-22 historical preservation)
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-RESEARCH.md` — prepended SUPERSEDED banner
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-VALIDATION.md` — `nyquist_compliant: true`; 6/6 sign-off checkboxes ticked; Approval stamp
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-02-PLAN.md` — stale P3/P11 @refs replaced with inline NOTE
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-03-PLAN.md` — stale pow_expand Example 2 @ref replaced with inline NOTE

## Decisions Made

- **CpuClient type alias:** `pub type CpuClient = ComputeClient<CpuRuntime>;` — the plan's assumption of `<CpuRuntime as Runtime>::Client` doesn't exist in cubecl 0.10-pre.3 (the `Runtime` trait carries only `Compiler`/`Server`/`Device` as associated types). Clients are always `ComputeClient<R>`.
- **raw_eval_scalar launcher signature:** `impl FnOnce(&CpuClient, Handle, Handle)` (owned handles). Forced by `ArrayArg::from_raw_parts(handle, len)` which takes `Handle` by value. The helper clones the output handle before handing it off, so it can still call `read_one_unchecked` after the launcher returns.
- **CubeDim constructor:** Use `CubeDim::new_3d(1,1,1)` / `new_1d(1)`. The plan's `CubeDim::new(x,y,z)` no longer exists — `new` was repurposed as `new<R>(&client, working_units)` in 0.10-pre.3.
- **Feature-gate plumbing:** `default = ["cpu"]` on `xcfun-ad/Cargo.toml`; tests run with `--features "cpu testing"`. Keeps cubecl-cpu out of `cargo check` without features (though default brings it in).
- **SUMMARY filename:** `01-01-CUBECL-PIVOT-SUMMARY.md` (plan-mandated distinct name per D-22); the pre-pivot `01-01-SUMMARY.md` is preserved with its SUPERSEDED banner.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cubecl 0.10-pre.3 API surface differs from the plan's `<interfaces>` block**

- **Found during:** Task 3 (cubecl spike implementation)
- **Issue:** The plan's interfaces assumed `client.create(...)`, `client.read_one(h.binding())`, `ArrayArg::from_raw_parts::<F>(&h, len, vec)`, `CubeDim::new(x,y,z)`, and `<CpuRuntime as Runtime>::Client`. None of these match the actual cubecl 0.10-pre.3 API.
- **Fix:** Adopted the real API (see Decisions Made). Documented every delta inline in the module headers of `for_tests/raw_eval_scalar.rs` and `tests/cubecl_spike.rs`, per the plan's explicit fallback guidance ("adjust the exact call shape, and record the delta as a comment").
- **Files modified:** `src/for_tests/cpu_client.rs`, `src/for_tests/raw_eval_scalar.rs`, `tests/cubecl_spike.rs`
- **Verification:** `cargo test -p xcfun-ad --features "cpu testing" --test cubecl_spike` — 4 passed, 0 failed.
- **Committed in:** `038c8af` (Task 3 commit)

**2. [Rule 3 - Blocking] Pre-execute housekeeping commit (dirty-tree resolution)**

- **Found during:** Pre-Task-1 setup (dirty working tree before revert chain)
- **Issue:** The working tree at executor start had deleted superseded phase dirs (`01-core-types-ad-engine/`, `02-lda-functionals-validation-pipeline/`), modified planning docs (STATE.md, config.json, 01-01/05/07 PLAN.md), and a new `01-PATTERNS.md` — none of which were committed. `git revert` would inherit conflicts across unrelated planning changes.
- **Fix:** Committed those pending changes as a housekeeping commit (`a4f4fc5`) BEFORE running the Task 1 revert chain, per the executor prompt's explicit instruction. This gives the revert a clean base.
- **Files modified:** `.planning/STATE.md`, `.planning/config.json`, 01-core-types deletion, 02-lda deletion, Phase 1 plan edits + new PATTERNS.md
- **Verification:** `git status --porcelain` empty before Task 1; `git revert --no-commit` chain ran with only the expected (preserved-via-checkout) conflicts.
- **Committed in:** `a4f4fc5` (pre-execute; not a task commit per se — explicitly called out in the executor prompt)

---

**Total deviations:** 2 auto-fixed (2 blocking).
**Impact on plan:** Both deviations necessary to run the plan. Zero scope creep — the plan's `<interfaces>` block explicitly authorised deviation when the real API differs ("adjust the exact call shape, and record the delta as a comment"). The pre-execute housekeeping was explicitly mandated by the `<sequential_execution>` block in the executor prompt.

## Issues Encountered

- **Revert chain conflicts on STATE.md / ROADMAP.md / REQUIREMENTS.md / 01-*-PLAN.md + 01-RESEARCH.md + 01-VALIDATION.md:** Expected per plan — these files must NOT be reverted (they reflect the post-pivot planning state). Resolved each conflict with `git checkout HEAD -- <path>` before continuing the chain.
- **Revert of `f07611c` removed `.cargo/config.toml`:** Expected — that commit originally *added* `.cargo/config.toml`. Restored via `git checkout HEAD -- .cargo/config.toml` per plan step 3. Also re-added the `compose.rs`/`math.rs`/`num.rs`/`tmath.rs` files that the commit had deleted; removed them via `git rm -f`.
- **Task 3 initial compile failed on `inp[i]` where `i: u32`:** Cubecl 0.10-pre.3's `Array<F>` indexing requires `usize`. Fixed by casting: `out[i as usize] = inp[i as usize]` (pattern from `cubecl-core/src/runtime_tests/cmma.rs`).

## User Setup Required

None — Phase 1 Plan 01 is 100% autonomous. No external services. No API keys. No dashboard configuration.

## Next Phase Readiness

**Ready for Plan 01-02 (ctaylor + ctaylor_rec #[cube] ports):**

- `for_tests::cpu_client()` and `for_tests::raw_eval_scalar(...)` are importable from test bodies with `--features "cpu testing"`.
- `cubecl::prelude::{Array, Float, CubeCount, CubeDim, ArrayArg}` is the established import set for `#[cube] fn` bodies.
- Pattern for `launch_unchecked` calls (owned Handle + `.clone()` into `ArrayArg::from_raw_parts`) is encoded in `cubecl_spike.rs` as a template.
- `.cargo/config.toml` `-Cllvm-args=-fp-contract=off` survives and applies to cubecl-cpu's MLIR JIT via the standard rustflags path (confirmed by `grep -c fp-contract=off .cargo/config.toml` returning 2 — both `[build]` and `[target.'cfg(all())']` blocks intact).

**Substrate risk status (CONTEXT.md D-01/D-03):** PASSED on this host. The 1e-12 parity contract is still unproven at the numerical level (that lands in Plan 01-05's golden-fixture harness); this plan only proves the JIT + launch cycle works.

**No blockers.**

---
*Phase: 01-taylor-algebra-ad-primitives-xcfun-ad*
*Plan: 01 (cubecl pivot Wave 0)*
*Completed: 2026-04-19*

## Self-Check: PASSED

- [x] `crates/xcfun-ad/src/lib.rs` exists
- [x] `crates/xcfun-ad/src/index.rs` exists
- [x] `crates/xcfun-ad/src/for_tests/cpu_client.rs` exists
- [x] `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs` exists
- [x] `crates/xcfun-ad/tests/cubecl_spike.rs` exists
- [x] `xtask/Cargo.toml` exists
- [x] `xtask/src/main.rs` exists
- [x] `xtask/src/bin/regen_ad_fixtures.rs` exists
- [x] `xtask/assets/regen_ad_fixtures/driver.cpp` exists
- [x] Commit `a4f4fc5` resolves (pre-execute housekeeping)
- [x] Commit `6d13128` resolves (Task 1 revert)
- [x] Commit `0f599bd` resolves (Task 2 workspace)
- [x] Commit `038c8af` resolves (Task 3 spike)
- [x] Commit `4d6022b` resolves (Task 4 hygiene)
- [x] Pre-pivot commit `c7a3f46` still resolves (`git show c7a3f46` ok — D-22)
- [x] `cargo check --workspace --all-targets` exits 0
- [x] `cargo test -p xcfun-ad --features "cpu testing" --test cubecl_spike` exits 0 (4/4 passing)
- [x] `head -5 01-RESEARCH.md` contains `SUPERSEDED 2026-04-19`
- [x] `head -3 01-01-SUMMARY.md` contains `SUPERSEDED BY CUBECL PIVOT`
- [x] `grep -q '^nyquist_compliant: true$' 01-VALIDATION.md` returns 1 match
- [x] No raw `@01-RESEARCH.md` directive in any `01-0[2-7]-PLAN.md` (only NOTE replacements)
