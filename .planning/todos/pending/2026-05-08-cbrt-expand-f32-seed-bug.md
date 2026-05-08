---
created: 2026-05-08T00:00:00.000Z
title: Fix xcfun-ad cbrt_expand f32-seed bug (cbrt_expand_x0_0_1, cbrt_expand_x0_10)
area: numerics
priority: medium
files:
  - crates/xcfun-ad/src/expand/cbrt.rs:63
  - crates/xcfun-ad/src/expand/cbrt.rs:78
  - crates/xcfun-ad/tests/expand_primary.rs:438
  - crates/xcfun-ad/tests/expand_primary.rs:453
---

## Problem

Two `xcfun-ad` parity tests fail on `cargo test`:

- `expand_primary :: cbrt_expand_x0_0_1` (test fn at `crates/xcfun-ad/tests/expand_primary.rs:438`)
- `expand_primary :: cbrt_expand_x0_10`  (test fn at `crates/xcfun-ad/tests/expand_primary.rs:453`)

Documented out-of-scope in 06-N6 STATE and project memory. **This is independent of** the cubecl `F::new(f64)` → silent f32 truncation bug that closed Plan 07-00 Task 0.3 (PW91C 541k → 1.8k via commit `df57c90`).

The `crates/xcfun-ad/src/expand/cbrt.rs` source already uses `F::cast_from(...)` correctly at the obvious sites (line 63: `y0 = x0.powf(F::cast_from(1.0_f64 / 3.0_f64))`; line 78: `let i_f = F::cast_from(i);`). The Newton-iteration seed and the order-1 path land on the right type. So the failing test points (`x0 = 0.1` and `x0 = 10`) drift from a different mechanism — likely the host-side reference (`host_cbrt` at `expand_primary.rs:195`) vs the cube kernel (`kernel_cbrt_expand` at `expand_primary.rs:78`) diverging for non-dyadic seeds; or a residual `F::new` call upstream of the runtime.

The third sibling test, `cbrt_expand_x0_1` (line 445, `x0 = 1.0` — exactly representable, dyadic), passes. The two failures both use non-dyadic decimals as starting points. That's the same fingerprint as the cubecl `F::new(f64)` truncation pitfall (project memory `cubecl_F_new_f32_pitfall.md`), so the first thing to check is whether anything inside the cube-side call chain still routes a non-dyadic literal through `F::new(...)` instead of `F::cast_from(...)`.

## Solution

1. **Reproduce in isolation**:
   ```bash
   cargo test -p xcfun-ad --test expand_primary cbrt_expand_x0_0_1 -- --nocapture
   cargo test -p xcfun-ad --test expand_primary cbrt_expand_x0_10  -- --nocapture
   ```
   Capture the rel-error magnitudes and the slot indices that fail.

2. **Bisect the algorithm**:
   - Compare host-only output (`host_cbrt`) vs the cube-kernel output (`run_cbrt_expand`) side-by-side at `x0 = 0.1` and `x0 = 10` — which leg is wrong?
   - Grep the call graph for any remaining `F::new(<f64-literal>)` or `F::new(constant)` calls in the cbrt path and the `kernel_cbrt_expand` body. Replace with `F::cast_from(...)` per the project memory rule.
   - If the host leg is wrong, the seed `y0 = x0.powf(F::cast_from(1.0/3.0))` is suspect — `(1.0/3.0)` is computed in f64 then `cast_from` to F. On `F = f64` host, that's a no-op so should match Newton convergence; on `F = f32` cube-side it loses ~7 decimals on the seed which then needs Newton iterations to recover, and the test tolerance may not allow for the extra refinement step.

3. **Pick a fix**:
   - If the issue is seed precision, increase Newton iterations conditionally on `F::IS_F32` or relax the test tolerance for the cube-side leg only (the host-side / f64 path must stay at strict 1e-12).
   - If the issue is a stray `F::new` upstream, swap to `F::cast_from`.
   - If neither — fall back to a relaxed 1e-9 tolerance for these two cases with a comment pointing to project memory `cubecl_F_new_f32_pitfall.md` and this todo.

4. **Verify**: `cargo test -p xcfun-ad --test expand_primary` clean; broader `cargo test -p xcfun-ad` clean.

## Context

- Cross-reference: project memory `cubecl_F_new_f32_pitfall.md` — fingerprint matches (non-dyadic decimal + cube-side path).
- Discovered during `/gsd:explore resolve phase 07-00 test and cargo test failed` (2026-05-08).
- Out of scope: bp86 c_abi fixture rot (separate todo, `2026-05-08-regenerate-c-abi-expected-json-after-06-n7-substrate-fixes.md`); PW91C 1.8k validation-harness residual (Phase 7+ workstream).
- These tests are tier-1 self-tests in `xcfun-ad`, not part of the validation harness against C++. Closing them tightens internal AD invariants.
