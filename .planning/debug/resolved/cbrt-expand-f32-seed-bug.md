---
slug: cbrt-expand-f32-seed-bug
status: resolved
trigger: "cbrt_expand f32-seed bug — Plan 07-00 follow-up, cargo test failures in xcfun-ad expand_primary"
created: 2026-05-08
updated: 2026-05-08
---

# Debug Session: cbrt_expand f32-seed bug

## Symptoms

- **Failing tests** (`cargo test -p xcfun-ad --test expand_primary`):
  - `cbrt_expand_x0_0_1`: got `0.4641588833612779`, expected `0.46415887274404843`, rel_err `1.06e-8`
  - `cbrt_expand_x0_10`:  got `2.154434690031884`,  expected `2.154434739312699`,  rel_err `2.29e-8`
- `cbrt_expand_x0_1` passes (because `1^anything ≈ 1` regardless of the 1/3 approximation).
- All other expand tests (`inv`, `exp`, `log`, `pow`, `sqrt`) pass.
- Workspace builds clean; no compile errors. Other crates' tests not directly affected.

## Reproduction

```
cargo test -p xcfun-ad --test expand_primary cbrt_expand
```

Both `cbrt_expand_x0_0_1` and `cbrt_expand_x0_10` panic at the `assert_close` rel-error check (1e-13 gate); `cbrt_expand_x0_1` passes.

## Initial Hypothesis (high confidence)

The **kernel** (`crates/xcfun-ad/src/expand/cbrt.rs:63-69`) was fixed in Plan 07-00 Task 0.3 (commits `92b1a4f` + `1edb1b0`):

```rust
let y0 = x0.powf(F::cast_from(1.0_f64 / 3.0_f64));   // f64 1/3, not f32-truncated
let y0_sq = y0 * y0;
let y1 = (F::new(2.0) * y0 + x0 / y0_sq) / F::new(3.0);   // Newton iter 1
let y1_sq = y1 * y1;
t[0] = (F::new(2.0) * y1 + x0 / y1_sq) / F::new(3.0);   // Newton iter 2 — converges to libm cbrt
```

The kernel now produces correctly-rounded cbrt to ≤1 ULP.

The **test reference** (`crates/xcfun-ad/tests/expand_primary.rs:195-213`) was NOT updated to track. It still emulates the OLD kernel:

```rust
let one_third_f32: f32 = 1.0_f32 / 3.0_f32;
t[0] = x0.powf(one_third_f32 as f64);   // f32-truncated 1/3, no Newton — old buggy seed
```

**Numerical verification (host f64 simulation):**
| x0  | kernel (Newton-refined) | host_cbrt (old f32 seed) | f64::cbrt | kernel vs cbrt | host vs cbrt |
|-----|------------------------|--------------------------|-----------|----------------|---------------|
| 0.1 | 0.4641588833612779     | 0.46415887274404843      | 0.4641588833612779 | 0           | 2.29e-8       |
| 10  | 2.154434690031884      | 2.154434739312699        | 2.154434690031884 | 0           | ~2.29e-8      |

The kernel matches `f64::cbrt` to 0 rel error. The host reference's rel-error matches the test failure magnitude exactly. **Confirmed root cause.**

## Proposed Fix

Update `host_cbrt` (and its accompanying doc comment) in `crates/xcfun-ad/tests/expand_primary.rs:195-213` to mirror the **current** kernel: f64 1/3 + 2 Newton iterations. The test crate's stated invariant is "use the same recurrence the kernel uses (same operation order) on host f64" (lines 20-22). The host reference simply drifted out of sync with the kernel update.

After applying:
- Run `cargo test -p xcfun-ad --test expand_primary` → all 18 tests should pass.
- Run `cargo test --workspace` → confirm no other regressions.

## Current Focus

- hypothesis: "Test reference `host_cbrt` is stale — still emulates pre-Plan-07-00 kernel (f32-truncated 1/3, no Newton refinement); kernel is now correct so the diff = 2e-8 surfaces."
- test: "Update host_cbrt to match kernel's `powf(f64 1/3) + 2 Newton iterations` recurrence; rerun expand_primary."
- expecting: "All 18 expand_primary tests pass; rel_err < 1e-13."
- next_action: "Apply fix to expand_primary.rs:195-213; cargo test -p xcfun-ad --test expand_primary."
- specialist_hint: "rust-numerical"

## Evidence

- timestamp: 2026-05-08
  finding: "Numerical sim in /tmp confirms kernel matches f64::cbrt exactly; host_cbrt drifts ~2e-8."
  source: "Inline rustc -e simulation; both code paths reproduced in isolation."

- timestamp: 2026-05-08
  finding: "Plan 07-00 SUMMARY.md (status: complete) lists item #7 — `cbrt_expand` f32 division + Newton-refinement seed — as fixed in kernel via commits 92b1a4f + 1edb1b0."
  source: ".planning/phases/07-python-bindings-release/07-00-SUMMARY.md:104"

- timestamp: 2026-05-08
  finding: "STATE.md explicitly notes: 'The two cbrt_expand_x0_{0_1,10} test failures in xcfun-ad's expand_primary.rs are pre-existing f32-truncation gotchas (documented in project memory) and out of scope for 06-N6.'"
  source: ".planning/STATE.md (2026-05-08 latest entry)"

- timestamp: 2026-05-08
  finding: "Project memory entry confirms cubecl F::new takes f32 (not f64) — silent truncation for non-dyadic literals; use F::cast_from(x_f64). The kernel uses cast_from correctly; the host_cbrt reference does not."
  source: "memory/cubecl_F_new_f32_pitfall.md"

## Eliminated

(none — single hypothesis with high confidence)

## Resolution

**Root cause:** `host_cbrt` reference fn in `crates/xcfun-ad/tests/expand_primary.rs:195-213` was emulating the pre-Plan-07-00 kernel (f32-truncated 1/3, no Newton refinement). The kernel was updated in Plan 07-00 Task 0.3 (commits `92b1a4f` + `1edb1b0`) to use a Newton-refined recurrence — `powf(F::cast_from(1.0_f64/3.0_f64))` seed plus two iterations of `y_{k+1} = (2·y_k + x0/y_k²) / 3`, converging to ≤ 1 ULP correctly-rounded cbrt. The test reference drifted out of sync; the residual `~2e-8` rel-error matched exactly the f32-precision approximation of 1/3 vs. the f64 value.

**Fix:** Mirrored the kernel's recurrence in `host_cbrt`, line-for-line in operation order. Replaced the `(1.0_f32/3.0_f32) as f64` seed with `1.0_f64/3.0_f64` then two host-f64 Newton iterations matching the kernel's `(2·y + x0/y²) / 3` schedule. Updated the doc comment to reference Plan 07-00 Task 0.3 + the kernel file/lines so future updates stay in lockstep.

**Verification:**
- `cargo test -p xcfun-ad --test expand_primary --features testing` → 18/18 pass (previously 16/18).
- `cargo test --workspace --no-fail-fast` → 402 passed, 0 failed, 6 ignored. No regressions.

**Files touched:** `crates/xcfun-ad/tests/expand_primary.rs` (host_cbrt + doc comment, lines 195-221).

**Commit:** `fix(07-00-followup): align host_cbrt reference with Newton-refined kernel`.
