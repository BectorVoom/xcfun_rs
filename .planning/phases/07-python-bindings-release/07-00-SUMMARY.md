---
phase: 07-python-bindings-release
plan: 00
status: partial-checkpoint-human-action
subsystem: validation-substrate
tags: [HUMAN-UAT-clearance, br-prefactor-typo, mpmath-fixture-regen, blocking-v0.1.0]
dependency_graph:
  requires:
    - "Phase 6 sign-off (xcfun-master HEAD a89b783 restored)"
    - "Phase 6 Plan 06-N2 26-functional manual lane (mpmath-only spec)"
    - "Phase 6 Plan 06-N1 11-functional inherited Phase-3 D-19 forwards"
    - "Phase 6 Plan 06-N3 18-functional small-magnitude AD-residual forwards"
  provides:
    - "BR_Q_PREFACTOR_F64 = 0.699_291_115_553_117_4_f64 (D-14 #6 cleared)"
    - "Regression lock test br_q_prefactor_locked in xcfun-kernels"
  affects:
    - "BRX / BRC / BRXC mpmath smoke parity (downstream tier-1 / tier-2)"
    - "Plan 07-00 Task 0.2 / 0.3 / 0.4 still pending"
tech_stack:
  added: []
  patterns:
    - "TDD RED→GREEN with #[cfg(test)] regression lock"
key_files:
  created: []
  modified:
    - "crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs"
decisions:
  - "Honor D-14 #6 verbatim (mpmath@200 truth value)"
  - "Lock the corrected constant with a regression test in br_like.rs (Threat T-7-00-01 mitigation)"
metrics:
  duration: ~30min (Task 0.1 only; Tasks 0.2/0.3/0.4 still pending)
  completed_date: "2026-05-06 (Task 0.1 only)"
---

# Phase 7 Plan 00: Clear 4 blocking Phase-6 HUMAN-UAT items + BR_Q_PREFACTOR_F64 typo fix — Summary (PARTIAL)

**One-liner:** Task 0.1 GREEN — `BR_Q_PREFACTOR_F64` corrected to mpmath@200 truth `0.699_291_115_553_117_4_f64`, regression-locked. Tasks 0.2 / 0.3 / 0.4 pending (Task 0.2 is a ~6h offline mpmath regen requiring human-action checkpoint per the plan; Task 0.3 awaits Task 0.2 cleanup).

## Status: PARTIAL — checkpoint reached at Task 0.2

This SUMMARY is committed because the orchestrator force-removes the worktree after the executor returns. A continuation agent (spawned after the operator's "approved" signal on Task 0.2) will overwrite this file with the full Task 0.2 / 0.3 / 0.4 results.

## Tasks Completed in This Run

### Task 0.1: Fix BR_Q_PREFACTOR_F64 typo (D-14 #6) — GREEN

**Behavior:**
- TDD pair: RED commit added `#[cfg(test)] mod tests { fn br_q_prefactor_locked() }` asserting `BR_Q_PREFACTOR_F64 == 0.699_291_115_553_117_4_f64`. At RED commit the constant was still the typo, so the test failed (verified). GREEN commit corrected the constant; the test passes.
- Tier-1 self-tests pass: `cargo test -p xcfun-eval --test self_tests --features testing` — `tier1_self_tests_pass ... ok` (27.4 s wall-clock; iterates over `FUNCTIONAL_DESCRIPTORS` including the BR family).

**Files modified:** `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs` (line 37 constant change + new `#[cfg(test)] mod tests` block at the bottom of the file).

**Commits (on `worktree-agent-a9b0fa8ce9c70f5bf`):**
- `1156257` — `test(06-N4/07-00): add BR_Q_PREFACTOR_F64 regression lock (RED)` — RED gate, test fails because constant is still the typo.
- `0e399a8` — `fix(06-N4/07-00): correct BR_Q_PREFACTOR_F64 to mpmath@200 truth (1/((2/3)·π^(2/3))) (GREEN)` — GREEN gate, constant corrected, test passes.

**Acceptance criteria evidence:**
- `grep -F '0.699_291_115_553_117_4_f64' crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs` → 2 matches (line 37 const definition + line 338 test assertion). The plan's literal "exactly one match" is superseded by the plan's behavior section which explicitly requires the test assertion containing the same literal — 2 matches is the correct outcome.
- `grep -F '0.699_390_040_064_282_6' crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs` → 0 matches (old typo eliminated; the doc comment that originally referenced it was reworded to avoid the literal).
- `cargo test -p xcfun-kernels br_q_prefactor_locked --lib` → `1 passed; 0 failed` (8.9 s wall-clock).
- `cargo test -p xcfun-eval --test self_tests --features testing` → `1 passed; 0 failed` (27.4 s; tier-1 self-tests covering the BR family still pass).
- Two commits on `worktree-agent-a9b0fa8ce9c70f5bf`, each touching only `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs`.

**TDD Gate Compliance:** RED test commit (`1156257`) precedes GREEN fix commit (`0e399a8`); REFACTOR not needed (the GREEN edit was a single literal change + an inline doc comment reword to keep `grep` clean).

## Tasks Pending (Not Started)

- **Task 0.2** (`checkpoint:human-action`, blocking) — ~6h offline mpmath fixture regen via `cargo run --release -p xtask --bin regen-mpmath-fixtures` + `--reference mpmath` strict-1e-13 sweep on 13 functionals + commit + flip 06-HUMAN-UAT.md item #3 result line. The plan explicitly forbids the executor from running this — the operator owns the 6h job. **This is the checkpoint return point.**
- **Task 0.3** (auto) — order-3 strict-1e-12 sweep on the 29 functionals (Plan 06-N1 11 + Plan 06-N3 18); flip 06-HUMAN-UAT.md §4 + §5. Awaits Task 0.2.
- **Task 0.4** (`checkpoint:human-verify`, blocking) — final operator confirmation that items 3/4/5/6 are all closed. Awaits Task 0.3.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Doc comment containing literal of old typo broke `grep` acceptance gate**
- **Found during:** Task 0.1 acceptance check.
- **Issue:** The TDD test block I added contained a doc comment that quoted the prior typo literal `0.699_390_040_064_282_6_f64` to explain what the regression lock guards against. The plan's automated acceptance gate requires `grep -F '0.699_390_040_064_282_6' ... == 0`, so the doc comment falsely tripped it.
- **Fix:** Reworded the doc comment to describe the typo qualitatively without quoting the literal. Net effect: `grep -F '0.699_390_040_064_282_6'` returns 0; the regression-lock semantics are unchanged.
- **Files modified:** `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs` (test-block doc comment).
- **Commit:** Folded into GREEN commit `0e399a8` (single hand-edit; not worth a separate commit).

### Critical Workflow Incident — Operator Action Required

**Severity:** HIGH. The executor accidentally landed a commit on the main repo's `master` branch (a protected ref) before discovering the cwd-management bug below. Per `<destructive_git_prohibition>` and `<task_commit_protocol>` step 0 (#2924), the executor MUST NOT self-recover by `git update-ref refs/heads/master <prior-tip>` — that prohibition is absolute. **The operator must decide how to handle this.**

**What happened:**
1. The agent runs in worktree `/home/user/Documents/workspace/xcfun_rs/.claude/worktrees/agent-a9b0fa8ce9c70f5bf` with branch `worktree-agent-a9b0fa8ce9c70f5bf`.
2. The agent's first attempt at Task 0.1 used **absolute paths to the main repo** (`/home/user/Documents/workspace/xcfun_rs/crates/xcfun-kernels/...`) for the `Edit` tool, instead of the worktree-prefixed path (`/home/user/Documents/workspace/xcfun_rs/.claude/worktrees/agent-a9b0fa8ce9c70f5bf/crates/xcfun-kernels/...`).
3. Subsequent `cd /home/user/Documents/workspace/xcfun_rs && git ...` commands operated on the main repo (master branch HEAD), not the worktree branch.
4. The first RED commit `eff753a` landed on `master` in the **main repo**: `master: b91324d → eff753a`.
5. The agent then noticed the issue, halted self-recovery, and re-did Task 0.1 correctly inside the worktree (commits `1156257` + `0e399a8` on `worktree-agent-a9b0fa8ce9c70f5bf`, against the proper base `b91324d`).

**Current state:**
- Main repo `master` HEAD is `eff753a` (the orphan RED commit on top of `b91324d`). It contains ONLY the test-block addition, no fix; if checked out and built, the regression test will FAIL because `master`'s `br_like.rs` still has the typo. The commit is otherwise valid (compiles modulo the test failure).
- Worktree `worktree-agent-a9b0fa8ce9c70f5bf` HEAD is `0e399a8` based on `b91324d` (correct). It contains both the RED test and the GREEN fix as a clean TDD pair.
- No push has happened.
- No concurrent activity on `master` — `git reflog show master` shows only my accidental commit since `b91324d`.

**Recovery options (operator decides):**
- **Option A (recommended):** Operator runs `git -C /home/user/Documents/workspace/xcfun_rs reset --hard b91324d` to remove the orphan RED commit from `master`. The full TDD pair will land on `master` cleanly when the worktree is merged. Safe: no concurrent commits exist past `b91324d` on master per reflog.
- **Option B:** Leave `eff753a` on `master` and let the worktree merge land on top of it. Result: `master` will have RED then RED+GREEN (the worktree's `1156257` is identical in content to `eff753a` except for SHA), producing a malformed merge. **NOT recommended.**
- **Option C:** Operator hand-cherry-picks `0e399a8` onto `master` directly, then drops the worktree. Then `eff753a` (RED on master) plus `0e399a8` (GREEN on master) form a valid TDD pair on master. The worktree commits are orphaned. Clean but bypasses the merge flow.

The agent did NOT attempt any of these — per the absolute prohibition on `git update-ref refs/heads/<protected>` and `git reset --hard` outside the agent-startup `<worktree_branch_check>` step.

**Why surfaced as deviation (not Rule 4 architectural):** This is an executor-internal workflow incident, not an architectural decision about the plan. But it requires operator action before any subsequent merge of this worktree into `master`. Documented here for full transparency.

**Lesson learned:** Inside a Claude Code worktree, all bash commands and Edit/Write tool calls MUST use absolute paths prefixed with the worktree directory (`/home/user/Documents/workspace/xcfun_rs/.claude/worktrees/agent-<id>/...`). The system-reminder env says cwd is the worktree, and per-bash-call cwd resets to the worktree, but `cd /home/user/Documents/workspace/xcfun_rs` explicitly traverses out of the worktree into the main repo.

## Authentication Gates

None.

## Threat Flags

None — Task 0.1 is a single-constant correctness fix; no new trust boundary, network surface, or schema change.

## Self-Check

**Files claimed created/modified:**
- `crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs` — verified at `/home/user/Documents/workspace/xcfun_rs/.claude/worktrees/agent-a9b0fa8ce9c70f5bf/crates/xcfun-kernels/src/functionals/mgga/shared/br_like.rs`, modified as expected.

**Commits claimed:**
- `1156257` — `git log --oneline | grep 1156257` returns the RED commit. FOUND.
- `0e399a8` — `git log --oneline | grep 0e399a8` returns the GREEN commit. FOUND.

## Self-Check: PASSED (for Task 0.1; remaining tasks pending checkpoint return)
