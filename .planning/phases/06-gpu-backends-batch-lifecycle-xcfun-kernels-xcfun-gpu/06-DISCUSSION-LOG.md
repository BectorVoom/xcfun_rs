# Phase 6: GPU Backends + Batch Lifecycle — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in `06-CONTEXT.md` — this log preserves the analysis.

**Date:** 2026-04-30
**Phase:** 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
**Mode:** discuss (interactive)
**Areas selected by user:** Phase scope strategy, D-19 forward triage + ground-truth policy, xcfun-gpu vs xcfun-eval crate boundary, Batch + dispatch ergonomics + zero-alloc

## Pre-discussion context loaded

- `.planning/PROJECT.md` — core value, constraints, key decisions
- `.planning/REQUIREMENTS.md` — 103 v1 requirements; Phase 6 owns 15 (RS-08, KER-01..06, GPU-01..08)
- `.planning/STATE.md` — Phase 5 sign-off summary; 30+ Phase-4 D-19 + 13 Phase-3 D-19 forwarded to Phase 6
- `.planning/research/SUMMARY.md` — 10-phase DAG collapsed into 7 standard-granularity phases; cubecl pre-release risk gates
- `.planning/phases/01..05/*-CONTEXT.md` — locked decisions inheritance (D-decisions D5/D6/D8/D17 from Phase 1; D-04/D-21/D-22/D-25 from Phase 2; D-19 forwards from Phases 3/4; D-13/D-17 from Phase 5)
- `docs/design/06-cubecl-strategy.md` — runtime matrix, batch lifecycle, kernel structure, numerical parity envelope
- `docs/design/07-accuracy-strategy.md` — ACC-04 statement of invariant, 4-tier test architecture, fixture format

## Codebase scout findings

- `crates/xcfun-gpu/src/lib.rs` — 73-byte stub; crate excluded from workspace `members`
- `crates/xcfun-eval/` — already cubecl-bearing; `for_tests::cpu_client()` `OnceLock<CpuClient>` pattern; per-point `eval_point_kernel` `#[cube(launch_unchecked)]` adapter
- `crates/xcfun-rs/src/functional.rs` — Phase 5 facade with documented `Box::leak` in `sync_weights_from_settings`; RS-08 stub
- Workspace `members` = `xcfun-ad`, `xcfun-core`, `xcfun-eval`, `xcfun-rs`, `xcfun-capi`, `xtask`, `validation`
- Workspace `exclude` = `crates/xcfun-gpu`, `crates/xcfun-python`
- `cubecl-cpu` already wired at `=0.10.0-pre.3`; cubecl-cuda / cubecl-hip / cubecl-metal / cubecl-wgpu absent

## Discussion turns

### Area selection (multiSelect)

**Q:** Which areas do you want to discuss for Phase 6?
**Selected:** all four (Phase scope strategy, D-19 forward triage + ground-truth policy, xcfun-gpu vs xcfun-eval crate boundary, Batch + dispatch ergonomics + zero-alloc).

### Area 1 — Phase scope strategy

**Q1.1:** How should Phase 6 scope be structured? (decimal sub-phases / wide / narrow / hybrid)
**Selected:** "Decimal sub-phases (Recommended)".

**Q1.2:** Phase 6 (proper) sign-off requires tier-3 parity GREEN on which set of functionals?
**Selected:** "All 78, strict 1e-13 — D-19 work blocks Phase 6". User flagged the contradiction with Q1.1; Q1.3 reconciles.

**Q1.3 (reconciliation):** How do "decimal sub-phases" + "strict 1e-13 blocks Phase 6" coexist?
**User clarification:** picked "Wide Phase 6, decimals are plans (Recommended)". Reasoning: prefers a single GSD phase with ~10–15 plans (decimal numbering for plan organisation, not separate phases); strict 1e-13 across 78 is the sign-off bar; unphysical regimes get kernel-level guards rather than stratum exclusion.

**Q1.4 (GPU target):** How should ROCm replace / coexist with CUDA in Phase 6?
**User input:** added "+metal(opt-in)" to the recommended option. Final selection: ROCm primary + CUDA + Metal as opt-in. Reason: user's dev environment is AMD; no CUDA hardware available locally. CUDA + Metal preserved as best-effort for the broader DFT user community. Saved to memory as `project_gpu_target.md`.

### Area 2 — D-19 forward triage + ground-truth policy

**Q2.1:** What is the ground-truth policy for Phase 6's strict 1e-13 tier-3 bar?
**Selected:** "Amend ACC-04: mpmath truth where C++ documents cancellation (Recommended)". Default reference stays C++ xcfun; per-functional ACC-04 amendment switches reference to mpmath at 200-digit precision for documented-cancellation regimes. Preserves algorithmic-identity contract; resolves LDAERFX without per-functional tolerance widening.

**Q2.2:** What lands in Plan 06-00 (substrate, before any GPU-runtime work)?
**Selected:** "Full substrate stack (Recommended)" — AD `ctaylor_compose` / `ctaylor_multo` specialisations for N ∈ {4, 5, 6}; in-kernel libm-hybrid `erf` at tightened precision; `tau ≥ tau_w` hard-clamp guard inside TPSS-correlation kernels; mpmath-truth fixture generator in `xtask`. Wave 0 of Phase 6.

**Q2.3:** How are post-substrate D-19 categories closed for the strict 1e-13 sign-off?
**Selected:** "Three cleanup plans, mpmath-only for excluded-spec (Recommended)". Plan 06-N1 = root-cause bisection for inherited Phase-3 forwards; Plan 06-N2 = mpmath fixtures for the 20 `excluded_by_upstream_spec` functionals (C++ aborts, cannot be reference); Plan 06-N3 = post-libm-hybrid sweep verifying small-magnitude residuals tighten to 1e-13.

### Area 3 — xcfun-gpu vs xcfun-eval crate boundary

**Q3.1:** Where do `Batch<R>`, `auto_backend()`, and the GPU buffer pool live?
**User input:** changed selection to "Full xcfun-kernels + xcfun-gpu split (per design doc)". Resurrects `docs/design/05-module-responsibilities.md` layout: `xcfun-kernels` = `#[cube]` kernel sources only; `xcfun-gpu` = runtime + Batch. Moves all 78 functional bodies + DensVarsDev + dispatch from `xcfun-eval` to `xcfun-kernels`. `xcfun-eval` becomes per-point dispatcher only. Saved to memory as `project_crate_layout.md`.

**Q3.2:** How should the xcfun-kernels migration sequence with the substrate work?
**Selected:** "Substrate first, then move (Recommended)". Plan 06-00 lands substrate in CURRENT `xcfun-eval/src/functionals/` tree. Plan 06-01 then `git mv` to `xcfun-kernels` after substrate GREEN. Avoids merge-conflicting substrate work with structural reorg; reorg-induced regressions are bisectable.

### Area 4 — Batch + dispatch ergonomics + zero-alloc

**Q4.1 (zero-alloc):** How does the strict zero-alloc per-point eval form work (Phase 5 D-13 forward; ~287 → 0 allocs/eval)?
**Selected:** "Pre-allocated reusable handle in Functional (Recommended)". `Functional` gains private mutable buffers sized at `eval_setup` time per `(vars, mode, order)`; interior mutability via `SyncUnsafeCell` (or `RefCell` if Send+Sync gate weakens). cubecl-cpu `CpuClient` stays in `for_tests`-promoted-to-production `OnceLock<CpuClient>`. One Functional per thread to avoid contention; documented in RS-10. Strict 0 allocs/eval after first call.

**Q4.2 (Wgpu f64):** On a Wgpu adapter without `wgpu::Features::SHADER_F64`, what does `Batch::open` do?
**Selected:** "Refuse with typed XcError::WgpuNoF64 (Recommended)". Adds typed variant `XcError::WgpuNoF64 { adapter_name: &'static str, requested_runtime: Backend }` (D-13-A preserves Phase 2 D-25 `Copy` constraint via `&'static str` payload). Compile-time `const _: () = assert!(size_of::<Scalar>() == 8);` in `xcfun-kernels` root.

## Decisions captured (D-01 ... D-18 + D-13-A) — see 06-CONTEXT.md

D-01..D-02: Phase scope (wide single phase, decimal plans, strict 1e-13 across 78 sign-off bar).
D-03..D-04: Ground-truth policy (mpmath amendment to ACC-04; xtask mpmath fixture generator).
D-05..D-07: GPU backend strategy (ROCm primary; CUDA + Metal opt-in; auto_backend priority order).
D-08..D-09: Crate boundary + migration order (full xcfun-kernels + xcfun-gpu split; substrate first then move).
D-10..D-11: Numerical / kernel guards (TPSS tau ≥ tau_w hard-clamp; libm-hybrid erf extension at order 3+).
D-12..D-13 + D-13-A: Batch + zero-alloc + typed Wgpu error (pre-allocated Functional handle; XcError::WgpuNoF64).
D-14..D-16: Dispatch threshold + buffer pool + eval_vec signature (64 / env XCFUN_MIN_BATCH_SIZE; powers-of-two doubling + monotonic generation counter; pitched signature matches xcfun.h).
D-17..D-18: Phase 5 substrate forwards (weights Vec refactor; LDA-vars=6 via DensVars-driven dispatch).

## Deferred ideas

Captured in 06-CONTEXT.md `<deferred>` section: stream-overlapped async GPU; Line<F> vectorisation; shared-memory reductions; stable cubecl 0.10 re-validation; xcfun-master patches; CUDA / Metal local validation; PyO3 / NumPy interop (Phase 7).

## Memory updates

- Created `project_gpu_target.md` — ROCm primary; CUDA + Metal opt-in; `auto_backend()` priority order.
- Created `project_crate_layout.md` — xcfun-kernels + xcfun-gpu split per design doc 05.
- Updated `MEMORY.md` index with both entries.
