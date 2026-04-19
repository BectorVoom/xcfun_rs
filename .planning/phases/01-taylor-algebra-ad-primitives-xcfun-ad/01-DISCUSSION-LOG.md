# Phase 1: Taylor Algebra & AD Primitives - Discussion Log (cubecl pivot)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `01-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 01-taylor-algebra-ad-primitives-xcfun-ad
**Areas discussed:** Numerical parity contract, CTaylor type design, Existing-work disposition, Phase 6 fate, Scalar-eval call pattern, `*_expand` representation, Property-test strategy, cubecl-cpu runtime lifetime, `F: Float` generic bound, GPU validation timing, Planning-doc update timing
**Mode:** discuss (TUI prompts via AskUserQuestion)
**Outcome:** Pivot from in-house hand-Rust `CTaylor` to cubecl-native `xcfun-ad`. 11 areas resolved, all user-selected to recommended option. Previous CONTEXT.md (12-decision auto-mode capture) superseded.

**Pivot precondition:** User confirmed "C-confirmed — Yes, proceed with the full pivot" via prior conversational turn (rejecting Interpretations A/B/Reconsider) and answered the blocker question with "still Taylor polynomial AD, just expressed in `#[cube]` form, validated on cubecl-cpu, GPU after support." This discussion-phase locks the implementation choices that fall out of that decision.

**Supersedes:** 2026-04-19 AM auto-mode log (10 hand-Rust gray areas resolved with recommended defaults). All 10 prior decisions VOID under the cubecl pivot.

---

## Numerical parity contract

| Option | Description | Selected |
|--------|-------------|----------|
| 1e-12 strict | Keep PROJECT.md core value. Verify cubecl-cpu preserves operation order. Pivot fails fast if 1e-12 unachievable. | ✓ |
| 1e-9 relaxed | Accept cubecl reordering. Easier to ship; PROJECT.md Core Value statement changes. | |
| Hybrid 1e-12 / 1e-9 | Tight on orders 0–2, loose on 3–6. Two parity gates in CI. | |
| Defer to researcher | Researcher surveys cubecl reordering behavior first. | |

**User's choice:** 1e-12 strict
**Notes:** Locks D-01 through D-03. Pivot accepts the risk of failing at the fixture-gate rather than silently widening tolerance.

---

## CTaylor type design

| Option | Description | Selected |
|--------|-------------|----------|
| `#[derive(CubeType)]` host struct + `#[cube]` free fns | Host Rust struct holds `[F; 1<<N]`; ops are `#[cube] fn`. | |
| Pure `#[cube]` type with cubecl `Array<F>` storage | No host struct. Storage is `Array<F>` allocated in kernel scope. Single source of truth. | ✓ |
| Two parallel types: host `CTaylor` + device `CTaylorDev` | Doubles maintenance; weakens single-source rationale. | |
| Defer — researcher prototypes both | Compile-test both options before deciding. | |

**User's choice:** Pure `#[cube]` type with cubecl `Array<F>` storage
**Notes:** Locks D-04 through D-08. Eliminates the planned Phase 6 `CTaylorDev` type — Phase 1's `CTaylor` already runs on any cubecl runtime.

---

## Existing-work disposition

| Option | Description | Selected |
|--------|-------------|----------|
| Revert all 01-* commits, replan from scratch | `git revert f07611c c7a3f46 2db557c`, drop crate src tree, rebuild on cubecl. | ✓ |
| Keep crate scaffolding, rewrite source files | Save half-day plumbing. Risks inconsistent toolchain config. | |
| Keep both — port hand-Rust as validation oracle | Doubles codebase, locks hand-Rust into long-term maintenance. | |

**User's choice:** Revert all 01-* commits, replan from scratch
**Notes:** Locks D-21, D-22. Wave 0 of new plan must include the `git revert` task as first action. Reverts are non-destructive (new commits, original history preserved). 01-01-SUMMARY.md kept with "SUPERSEDED BY CUBECL PIVOT" header.

---

## Phase 6 fate

| Option | Description | Selected |
|--------|-------------|----------|
| Keep Phase 6, narrow scope | Per-functional `#[cube]` bodies move to Phases 2–4; Phase 6 owns batch lifecycle, CUDA, Wgpu. | ✓ |
| Delete Phase 6, fold into Phases 2–4 | Each tier ships CPU + CUDA + Wgpu. Roadmap shrinks to 6 phases. Bigger per-phase scope. | |
| Defer — decide later | Discuss-phase scope is Phase 1 only. | |

**User's choice:** Keep Phase 6, narrow scope
**Notes:** Locks D-23, D-24. ROADMAP.md Phase 6 "Goal" and Phases 2–4 Success Criteria need editorial updates per D-27.

---

## Scalar-eval call pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Every scalar eval launches a 1-thread kernel | `eval(point)` calls `eval_vec(&[point])`. ~10 μs overhead. Single source of truth. | ✓ |
| Bypass cubecl for scalar — keep Rust scalar interpreter | Microsecond-fast scalar, but two `CTaylor::mul` impls to keep bit-identical. | |
| Caller-side accumulator that flushes when full | Stateful API. Bad fit for numerical library. | |

**User's choice:** Every scalar eval launches a 1-thread kernel
**Notes:** Locks D-15, D-16. Test/property-test design must account for kernel-launch overhead — addressed by D-18's batch-per-property pattern.

---

## `*_expand` representation

| Option | Description | Selected |
|--------|-------------|----------|
| Port each as `#[cube] fn` writing into cubecl `Array<F, 8>` | Algorithmic identity end-to-end on cubecl. Mirrors `tmath.hpp` line-for-line. | ✓ |
| Keep as host `#[inline] fn` writing `&mut [f64; 8]`, `#[cube]` calls via comptime | Only runs on host f64. Breaks once Phase 6 enables CUDA/Wgpu. | |
| Defer to researcher | Researcher prototypes one `*_expand` first. | |

**User's choice:** Port each as `#[cube] fn` writing into cubecl `Array<F, 8>`
**Notes:** Locks D-12 through D-14. Composed elementary fns (`reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`) follow the same pattern.

---

## Property-test strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Batch each property into one kernel call of 10,000 inputs | proptest generates 10k upfront, single kernel evaluates all. ~100 ms total. | ✓ |
| Reduce iteration count to 1,000 | 10× fewer iterations. Easier to write; weaker statistical coverage. | |
| Keep 10k and accept multi-second wall-clock | Simplest; cargo test slows. | |

**User's choice:** Batch each property into one kernel call of 10,000 inputs
**Notes:** Locks D-18. Preserves the original ≥10 000 iteration count from the design strategy without paying 10,000× kernel-launch overhead.

---

## cubecl-cpu runtime lifetime in tests

| Option | Description | Selected |
|--------|-------------|----------|
| `OnceLock<CpuClient>` in `xcfun-ad::for_tests`, shared across threads | Init once per binary. Matches cubecl idiom. Minimal overhead. | ✓ |
| Per-test fresh client | Cleaner isolation; pays init cost (~100 ms) per test. | |
| Defer to researcher | Researcher checks tracel-ai/cubecl examples. | |

**User's choice:** `OnceLock<CpuClient>` in `xcfun-ad::for_tests`, shared across threads
**Notes:** Locks D-17. Helper module also exposes `raw_eval_scalar<F>` per D-16.

---

## `F: Float` generic bound

| Option | Description | Selected |
|--------|-------------|----------|
| Generic over `F: Float` | All `#[cube] fn` take `<F: Float>`. f64 today, f32 reachable for Phase 6 spikes. | ✓ |
| Hardcode f64 | Simpler signatures; forks the source for Phase 6 Wgpu research. | |
| Generic but lock test runner to f64 only | Public API generic; `cargo test` only runs f64. Middle ground. | |

**User's choice:** Generic over `F: Float`
**Notes:** Locks D-09 through D-11. f32 ban moves UPSTREAM to `xcfun-rs::Functional` (not the AD crate's responsibility).

---

## GPU validation timing

| Option | Description | Selected |
|--------|-------------|----------|
| Defer to Phase 6 — unchanged from current roadmap | Phase 1 cubecl-cpu only. CUDA + Wgpu in Phase 6. Matches "(gpu after support)". | ✓ |
| Phase 1 also smoke-tests CUDA when `--features cuda` | Single "cubecl runs on CUDA" smoke test. Catches catastrophic regressions. | |
| Decide at Phase 6 planning | Note as deferred. | |

**User's choice:** Defer to Phase 6 — unchanged from current roadmap
**Notes:** Locks D-25, D-26. `xcfun-ad` exposes only the `cpu` feature; CUDA/Wgpu features live in Phase 6 backend crates.

---

## Planning-doc update timing

| Option | Description | Selected |
|--------|-------------|----------|
| Now, before plan-phase runs | Rewrite STATE.md decisions, ROADMAP.md Phase 1 + Phase 6, REQUIREMENTS.md AD-01..06 immediately. Stale docs would mislead the planner. | ✓ |
| Let plan-phase do it as part of replanning | Planner reads conflicting locked decisions; risks confused questions. | |
| Defer — rewrite during plan-phase verification loop | Reactive; risks double work. | |

**User's choice:** Now, before plan-phase runs
**Notes:** Locks D-27, D-28. STATE.md, ROADMAP.md, REQUIREMENTS.md updated as part of this discuss-phase commit. `docs/design/06,07,12-*.md` follow-on rewrites become planner tasks (D-28).

---

## Claude's Discretion

- Module layout for `*_expand` (one mod or one mod per fn)
- Internal `cpu_client()` naming and exact module path
- Criterion bench directory layout
- Error variant names for kernel-launch failures in tests

## Deferred Ideas

- f32 instantiation (kept generic, exercised only in Phase 6 spikes)
- SIMD inside `CTaylor::mul` (cubecl-cpu vectorizes via MLIR; revisit if benches show headroom)
- Nested `CTaylor<CTaylor<F, M>, N>` (not required)
- Hand-Rust CTaylor as redundant oracle (explicitly rejected per D-21)
- Criterion regression gate (v2 — PERF-01)
