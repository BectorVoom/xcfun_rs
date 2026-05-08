---
phase: 07-python-bindings-release
plan: 08
subsystem: release-tooling
tags: [xtask, cargo-publish, topological-sort, kahns-algorithm, crates-io, d-15, pitfall-4, pitfall-5, release-publish]

# Dependency graph
requires:
  - phase: 07-python-bindings-release
    provides: "Plan 07-07 release.yml workflow + maturin sdist/wheel matrix; Plan 07-00 D-15 lock (Rust crates publish locally; only maturin publish runs in CI)"
provides:
  - "xtask release-publish binary — topological cargo publish driver for the 7 Rust library crates (xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi)"
  - "Pitfall 5 lock — per-crate `cargo publish --dry-run` before any bare `cargo publish`, preventing partial-publish + yank-irreversibility cascade"
  - "Pitfall 4 lock — index-propagation polling between successive publishes, capped at 5 min; prevents the next crate's resolve from racing crates.io's sparse index"
  - "Resume idempotency — `--from <crate>` flag for partially-published runs + skip-if-already-published-at-version (cargo search lookup)"
  - "NEVER_PUBLISH allowlist — xtask + validation + xcfun-py + xcfun-python (legacy alias) are filtered from the publish set"
affects: [07-09, 07-10, post-v0.1.0-release]

# Tech tracking
tech-stack:
  added: []  # Reused existing xtask deps (anyhow, serde_json) — no new crates
  patterns:
    - "xtask binary using `cargo metadata --format-version 1` shell-out + `serde_json::Value` parse (mirrors check_cubecl_pin.rs)"
    - "Kahn's topological sort with deterministic alphabetical tiebreak via BTreeMap key-order iteration"
    - "Modern cargo metadata workspace_members are PackageId strings (`path+file:///...#0.1.0`) — match by `id` field, not by space-splitting the workspace_members entry"
    - "Driver flags: --dry-run is DEFAULT; --execute is opt-in; --execute + --dry-run mutually exclusive; --from + --skip for resume + debug"
    - "Integration tests via `env!(\"CARGO_BIN_EXE_release-publish\")` — Cargo auto-exposes binary paths to integration tests"

key-files:
  created:
    - "xtask/src/bin/release_publish.rs (350 LoC) — topological cargo publish driver"
    - "xtask/tests/release_publish.rs (152 LoC) — 6 integration tests covering Tests 1–5 from plan §<behavior>"
  modified:
    - "xtask/Cargo.toml — added [[bin]] name = \"release-publish\" entry after regen-mpmath-fixtures"

key-decisions:
  - "Match workspace members by `id` (PackageId string) instead of parsing `workspace_members` entries — modern cargo (≥ 1.77) uses `path+file:///abs/path#0.1.0` form, not the legacy `name version source` triple"
  - "Exclude `kind = \"dev\"` deps from the topological graph — dev-dependencies don't affect publish ordering since `cargo publish` skips them"
  - "BTreeMap-key-order tiebreak ⇒ deterministic alphabetical order; for v0.1.0 the two zero-in-degree roots (xcfun-ad, xcfun-core) sort `xcfun-ad` first, matching the plan's expected order"
  - "`cargo publish --dry-run` failures in dry-run-only mode are warnings, not fatals — the user is checking publishability across all crates in one pass; in --execute mode they remain hard errors"
  - "Final maturin step is PRINTED, not invoked — D-15 + Open Question 2 keep maturin publish CI-driven (release.yml Plan 07-09); xtask is local-driven"

patterns-established:
  - "Pre-release publish driver pattern: dry-run-by-default, opt-in execute, idempotent resume, network-fault-tolerant search lookup"
  - "Workspace dep DAG extraction: shell-out cargo metadata, parse via serde_json::Value, build edges from packages.dependencies filtered by `name ∈ workspace_members`"

requirements-completed: []  # plan frontmatter requirements: []

# Metrics
duration: ~25min
completed: 2026-05-08
---

# Phase 07 Plan 08: xtask release-publish Driver Summary

**Local-only `cargo run -p xtask --bin release-publish` driver that walks the workspace dep DAG, computes a topological cargo publish order over the 7 Rust library crates, runs `cargo publish --dry-run` per crate before any actual publish (Pitfall 5 yank lock), and polls the crates.io index between successive publishes (Pitfall 4 propagation race) — completing the D-15 release flow primitive.**

## Performance

- **Duration:** ~25 min (single TDD task, RED→GREEN, no REFACTOR needed)
- **Completed:** 2026-05-08
- **Tasks:** 1 (the plan defines a single TDD task spanning binary + Cargo.toml entry + tests)
- **Files created:** 2 (release_publish.rs, tests/release_publish.rs)
- **Files modified:** 1 (xtask/Cargo.toml)

## Accomplishments

- Topological cargo publish driver: 350 LoC of Rust + 152 LoC of integration tests, all in xtask (app-boundary; anyhow allowed per CLAUDE.md)
- Verified topological output for v0.1.0: `xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi` (7 crates, deterministic alphabetical tiebreak)
- 6/6 integration tests pass; covers help output, topological order, NEVER_PUBLISH exclusion, --from resume, and unknown-flag rejection
- Pitfall 4 (`wait_for_index`) and Pitfall 5 (per-crate `cargo publish --dry-run` before bare publish) wired with the correct ordering
- Idempotent: skip-if-already-published-at-version via `cargo search` + alphabetical Kahn's algorithm

## Task Commits

Plan 07-08 has a single TDD task; the RED→GREEN cycle produced 2 commits:

1. **RED gate** — `632b2b5` — `test(07-08): add failing release-publish integration tests (RED gate)`
2. **GREEN gate** — `e901ad4` — `feat(07-08): xtask release-publish — topological cargo publish driver (D-15 + Pitfalls 4/5)`

No REFACTOR commit was needed — the GREEN implementation is self-contained and well-factored (separate functions for arg parsing, topo sort, search lookup, polling, command spawning).

## Files Created/Modified

- **CREATED** `xtask/src/bin/release_publish.rs` — driver implementation (~350 LoC):
  - `main()` — arg parsing + driver loop
  - `topological_publish_order()` — Kahn's algorithm over cargo metadata
  - `is_published_at_version()` — `cargo search` idempotency check
  - `wait_for_index()` — Pitfall 4 polling (5-min cap)
  - `run()` — Command::status helper with non-zero-exit ⇒ Err
  - `print_help()` — usage text
  - Constants: `NEVER_PUBLISH`, `POLL_INTERVAL = 5s`, `POLL_TIMEOUT = 300s`, `PUBLISH_COOLDOWN = 30s`
- **CREATED** `xtask/tests/release_publish.rs` — 6 integration tests (152 LoC):
  - `test_help_exits_zero`
  - `test_dry_run_prints_topological_order`
  - `test_excludes_never_publish_crates`
  - `test_from_skips_earlier_crates`
  - `test_unknown_flag_fails`
  - `test_from_unknown_crate_fails`
- **MODIFIED** `xtask/Cargo.toml` — appended `[[bin]] name = "release-publish"` block after `regen-mpmath-fixtures`

## Decisions Made

- **PackageId-based workspace member match**: see Deviations §1
- **dev-dep exclusion from topological graph**: `cargo metadata` emits `dependencies[].kind = "dev"` for dev-deps; including them would inflate in-degree counts and could cycle (e.g., a crate's tests depending on a sibling). Since `cargo publish` skips dev-deps during registry resolution, they're correctly omitted from publish ordering.
- **BTreeMap iteration as alphabetical tiebreak**: rather than calling `.min_by_key()` on each iteration of Kahn's, the in-degree map IS a BTreeMap whose `iter()` yields keys in alphabetical order. The first zero-in-degree key encountered is the next publish target. Cleaner and faster than recomputing min on each step.
- **Soft failure for dry-run-only `cargo publish --dry-run`**: in --dry-run mode, a per-crate dry-run failure (typically because the crate's deps aren't yet on crates.io) is informational, not fatal — print + continue so the operator sees ALL failures in one pass. In --execute mode, it stays a hard error.
- **Final maturin step is PRINTED only**: D-15 + Open Question 2 lock maturin publish behind CI (release.yml). xtask refusing to invoke `maturin publish` is intentional — it forces operators to either commit the tag (which triggers CI) or run maturin locally with explicit credentials.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Plan's `workspace_members` parser broken on modern cargo**

- **Found during:** Task 8.1 (initial draft based on plan pseudocode)
- **Issue:** The plan's `topological_publish_order()` draft (07-08-PLAN.md lines 302–307) parses workspace_members entries via `s.split_whitespace().next()`, expecting the legacy `name version (path+file://path)` format. Modern cargo (≥ 1.77, confirmed on the worktree's resolved metadata) emits PackageId strings like `path+file:///abs/path#0.1.0` with NO whitespace. `split_whitespace().next()` returns the entire URL string, which never matches the bare crate name in `pkg["name"]`. The result: every workspace member fails the `workspace_members.contains(&name)` check, the `crates` map stays empty, and `topological_publish_order()` returns `Ok(vec![])` — silently producing a 0-crate "publish order" with no error.
- **Fix:** Match workspace members by exact `id` field (the full PackageId string) instead of by parsed name. Two-pass approach:
  1. First pass: build `member_ids: HashSet<String>` from `workspace_members` (just the strings), then for each package whose `id` is in `member_ids`, record its `name` in `workspace_names: HashSet<String>`.
  2. Second pass: filter dep edges via `workspace_names.contains(&dep_name)`.
- **Files modified:** xtask/src/bin/release_publish.rs (the entire `topological_publish_order` function)
- **Verification:** Integration test `test_dry_run_prints_topological_order` asserts the 7-crate order matches `[xcfun-ad, xcfun-core, xcfun-kernels, xcfun-eval, xcfun-gpu, xcfun-rs, xcfun-capi]` exactly — would have failed under the plan's draft (zero crates).
- **Committed in:** e901ad4

**2. [Rule 2 - Missing Critical] Excluded `kind = "dev"` deps from the dependency graph**

- **Found during:** Task 8.1 (writing the second pass of dep extraction)
- **Issue:** The plan's draft doesn't filter `kind`. Including dev-deps would (a) inflate in-degree counts, potentially making the graph cyclic, and (b) misrepresent publish ordering — `cargo publish` skips dev-deps during registry resolution.
- **Fix:** `if d["kind"].as_str() == Some("dev") { continue; }` in the dep-edge loop.
- **Files modified:** xtask/src/bin/release_publish.rs
- **Verification:** Topological output matches the expected 7-crate order. If dev-deps were included, e.g., xcfun-core's dev-deps would add edges that don't reflect publish reality.
- **Committed in:** e901ad4

**3. [Rule 1 - Bug] Edition-2024 reference-pattern in Kahn's filter**

- **Found during:** Task 8.1 (first compile attempt)
- **Issue:** The plan's draft uses `.filter(|(_, &deg)| deg == 0)`. Edition 2024 enforces stricter pattern matching: a non-reference pattern matching a reference type implicitly borrows; mixing `(_, &deg)` inside that triggers `error: cannot explicitly dereference within an implicitly-borrowing pattern`.
- **Fix:** `.filter(|&(_, &deg)| deg == 0)` (explicit reference pattern on the outer tuple).
- **Files modified:** xtask/src/bin/release_publish.rs (line 333)
- **Verification:** `cargo build -p xtask --bin release-publish` exits 0.
- **Committed in:** e901ad4

**4. [Rule 2 - Missing Critical] Soft-error for `cargo publish --dry-run` in dry-run-only mode**

- **Found during:** Task 8.1 (testing the binary against the actual workspace)
- **Issue:** Plan draft treats `cargo publish --dry-run` failure as fatal in all modes. But in dry-run-only operation, a single failure aborts the loop — and since most failures are "dep not yet on crates.io", the first crate's success but second crate's failure (because its dep IS on crates.io but not at the bumped version yet) would leave the operator with no information about crates 3-7. Operators want to see ALL the publishability failures in one pass.
- **Fix:** In dry-run-only mode (`dry_run = true && execute = false`), a per-crate `cargo publish --dry-run` failure prints `[warn] dry-run failed for {crate}: {error}` and continues. In `--execute` mode it remains a hard error (we MUST not proceed past a failed dry-run when about to actually publish).
- **Files modified:** xtask/src/bin/release_publish.rs (`main()` loop)
- **Verification:** Manual run with `--from xcfun-eval --skip xcfun-rs` confirmed warning-and-continue behavior; tests pass.
- **Committed in:** e901ad4

---

**Total deviations:** 4 auto-fixed (3 bugs in plan draft + 1 missing critical correctness)
**Impact on plan:** All deviations are correctness fixes for the plan's pseudocode; the public surface (CLI flags, behavior, success criteria) matches the plan exactly. No scope creep. The plan's draft was a useful skeleton; CARGO/Edition realities required these touch-ups.

## Issues Encountered

- **rustfmt edition mismatch in standalone invocation**: `rustfmt --check` without `--edition 2024` flags Edition-2024-style import sort as drift. `cargo fmt -p xtask -- --check` honours the package edition, but doesn't accept extra flags after `--`. Workaround: use `rustfmt --edition 2024 --check` for explicit verification. Output is clean. The repo has no `rustfmt.toml`, but all existing xtask binaries use Edition 2024 sort, so my code matches the project convention.

## Verification

All acceptance criteria from plan §<verify> + §<acceptance_criteria> pass:

- `cargo build -p xtask --release --bin release-publish` exits 0
- `grep -F 'name = "release-publish"' xtask/Cargo.toml` → 1 line
- `grep -F 'src/bin/release_publish.rs' xtask/Cargo.toml` → 1 line
- `test -f xtask/src/bin/release_publish.rs` exits 0
- `grep -F 'NEVER_PUBLISH' xtask/src/bin/release_publish.rs` → 4 lines (definition + 3 use sites)
- `grep -F '"xtask"' / '"validation"' / '"xcfun-py"'` → each 1 line in NEVER_PUBLISH list
- `grep -F '"metadata"' xtask/src/bin/release_publish.rs` → 1 line (cargo metadata invocation)
- `grep -F 'topological_publish_order' xtask/src/bin/release_publish.rs` → 2 lines
- `grep -F 'is_published_at_version'` → 3 lines
- `grep -F 'wait_for_index'` → 2 lines (Pitfall 4)
- `grep -F '--dry-run'` → 12 lines (≥ 2 required by plan — Pitfall 5)
- `grep -F '--execute'` → 9 lines (≥ 1 required)
- `grep -F '--from'` → 6 lines (≥ 1 required, resume idempotency)
- `cargo run -q -p xtask --bin release-publish -- --help` exits 0 and prints "release-publish"
- `cargo run -q -p xtask --bin release-publish -- --dry-run | grep -E '^  xcfun-' | wc -l` → 7 (the 7 expected crates)
- `cargo test -p xtask --test release_publish` → 6/6 passed
- `cargo run -q -p xtask --bin check-no-anyhow` → PASS (8 library crates checked; xtask is app-boundary)

## User Setup Required

None — release-publish is operator-driven local tooling. To use it for the v0.1.0 release:

```bash
# 1. Verify everything is publishable (network-safe, no credential use)
cargo run -p xtask --bin release-publish -- --dry-run

# 2. Actually publish (requires ~/.cargo/credentials.toml with crates.io token)
cargo run -p xtask --bin release-publish -- --execute

# 3. After all 7 Rust crates are visible on crates.io, push the tag — release.yml
#    builds + uploads the maturin sdist + wheels (Plan 07-07 / 07-09).
```

## Next Phase Readiness

- The D-15 release flow primitive is complete: `xtask release-publish` (LOCAL, this plan) + release.yml maturin matrix (CI, Plan 07-07).
- Plan 07-09 (publish-pypi job in release.yml) and Plan 07-10 (final phase wrap) are unblocked.
- v0.1.0 publish flow is now executable end-to-end (modulo the operator running the driver).

## Self-Check: PASSED

**Files created exist:**
- `xtask/src/bin/release_publish.rs` — FOUND
- `xtask/tests/release_publish.rs` — FOUND
- `.planning/phases/07-python-bindings-release/07-08-SUMMARY.md` — FOUND (this file)

**Files modified contain expected content:**
- `xtask/Cargo.toml` contains `name = "release-publish"` and `path = "src/bin/release_publish.rs"` — FOUND

**Commits exist in git log:**
- `632b2b5` (RED gate) — FOUND
- `e901ad4` (GREEN gate) — FOUND

## Threat Surface Scan

No new security-relevant surface introduced. The driver:
- Reads `~/.cargo/credentials.toml` indirectly via `cargo publish` — same trust boundary as any other cargo invocation.
- Talks to `cargo search` (read-only) and `cargo publish` (write, gated on --execute).
- All STRIDE entries from the plan's `<threat_model>` are covered:
  - T-7-08-01 mitigated: `--dry-run` is default; per-crate `cargo publish --dry-run` runs before bare publish
  - T-7-08-02 mitigated: `--from <crate>` resume + `is_published_at_version` idempotent skip
  - T-7-08-03 mitigated: `wait_for_index` polls with explicit timeout error
  - T-7-08-04 accepted: stale Cargo.lock surfaces via `cargo publish` itself

No new threat flags.

## TDD Gate Compliance

The plan task is `tdd="true"`. Gate sequence verified:

1. **RED gate** — commit `632b2b5` is `test(07-08): ...`
2. **GREEN gate** — commit `e901ad4` is `feat(07-08): ...`
3. **REFACTOR gate** — not needed (single-pass GREEN was already factored)

RED commit precedes GREEN commit ✓; both are present ✓; type prefixes are correct ✓.

---
*Phase: 07-python-bindings-release*
*Plan: 08*
*Completed: 2026-05-08*
