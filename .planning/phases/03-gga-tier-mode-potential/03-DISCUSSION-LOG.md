# Phase 3: GGA Tier + `Mode::Potential` - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `03-CONTEXT.md` — this log preserves the alternatives considered
> and the rationale for each auto-selected default.

**Date:** 2026-04-24
**Phase:** 03-gga-tier-mode-potential
**Mode:** discuss `--auto`
**Areas discussed (auto-selected recommended defaults):** 10 gray areas / 25 decisions

---

## Area 1 — GGA scope count

| Option | Description | Selected |
|--------|-------------|----------|
| 45 functionals (per ROADMAP Phase 3 Goal text) | Match the ROADMAP loose count; stretch to 45 by counting shared helpers | |
| **40 functionals (per REQUIREMENTS GGA-01..GGA-10 explicit IDs ∩ FunctionalId enum)** | **Authoritative count from enum inspection; dispatcher arm-count must match** | **✓ (D-01)** |
| 39 functionals (exclude LB94) | LB94 is not in the enum; skip it numerically, keep ROADMAP happy | |

**Auto-mode rationale:** ROADMAP's "45" figure is soft; the REQUIREMENTS list is explicit and the FunctionalId enum is the unambiguous count. 40 includes LB94 as a scope item deferred separately (D-19); the 39 count conflates scope-exclusion with enum membership. ROADMAP Phase 3 sentence should be corrected by the planner.

---

## Area 2 — Wave 0 strategy (substrate vs family-first)

| Option | Description | Selected |
|--------|-------------|----------|
| **Wave 0 = substrate extension (xcfun-ad primitives + DensVarsDev Vars arms + GGA shared helpers)** | **Every GGA family depends on shared substrate; extract once upfront** | **✓ (D-02)** |
| Wave 0 = first family port (PBE) | Port PBE family first, extract helpers as needed | |
| No dedicated Wave 0 | Inline all helpers into per-functional bodies | |

**Auto-mode rationale:** Phase 2 established the "extract shared substrate before family ports" pattern (Plans 02-01 → 02-03: cleanup → gates → DensVarsDev scaffold, THEN per-LDA ports). Skipping Wave 0 duplicates ~600 lines of helpers across PBE-family bodies and risks per-family drift.

---

## Area 3 — Wave partitioning

| Option | Description | Selected |
|--------|-------------|----------|
| Single monolithic wave (40 functionals) | No parallelism | |
| **5 waves: Wave 1 (PBE+Becke+BR+LYP = 20), Wave 2 (OPTX+PW86/PW91+P86+APBE = 10), Wave 3 (B97+KT/BTK/CSC = 9), Wave 4 (Mode::Potential), Wave 5 (order-4 bump + ACC-04 re-run)** | **Family-grouped with capstone; mirrors Phase 2 Wave-2 parallel pattern** | **✓ (D-03)** |
| Per-functional wave (40 waves) | Maximum commit granularity but excessive overhead | |

**Auto-mode rationale:** Phase 2 Plans 02-04 + 02-05 ran in parallel; scaling this to 40 functionals requires family-grouping. Within each wave, per-functional ports are embarrassingly parallel. Mode::Potential depends on all GGA bodies, so it is a separate downstream wave. The ACC-04 re-run is a capstone gate.

---

## Area 4 — xcfun-ad primitive additions

| Option | Description | Selected |
|--------|-------------|----------|
| No additions (inline expm1 + sqrtx_asinh_sqrtx in per-functional bodies) | Violates algorithmic-identity rule (C++ has them as separate helpers) | |
| **Add `expm1_expand` + `ctaylor_expm1` (D-05) and `sqrtx_asinh_sqrtx` helper (D-06); commit to no further additions (D-07)** | **Mandatory: PBEC/APBEC/SPBEC/PBEINTC/RPBEX need expm1; PW91X/PW91K need sqrtx_asinh_sqrtx** | **✓ (D-05, D-06, D-07)** |
| Add expm1 + sqrtx_asinh_sqrtx + speculative future primitives | Scope-creep risk | |

**Auto-mode rationale:** grep of `xcfun-master/src/functionals/*.cpp` confirms `expm1` used in 6 GGA/MGGA functionals; `sqrtx_asinh_sqrtx` used in 4 GGAs. Phase 1 shipped 11 composed ops but NOT these two. Phase 1 is Complete so this is an additive extension, not a rework. D-07 caps further additions to prevent drift.

---

## Area 5 — GGA shared-helper location

| Option | Description | Selected |
|--------|-------------|----------|
| Inline helpers per-functional | Duplication across 12 PBE-family bodies | |
| **`crates/xcfun-eval/src/functionals/gga/shared/<module>.rs`** (one module per C++ `.hpp` cluster) | **Mirrors Phase-2 `lda/vwn_eps.rs`, `lda/pw92eps.rs` helper pattern** | **✓ (D-08, D-09)** |
| Shared helpers in a top-level `xcfun-eval::shared` crate-level module | Less cohesive; obscures GGA-locality | |

**Auto-mode rationale:** Phase 2 already has the pattern (`vwn_eps.rs` is 495 lines, `pw92eps.rs` is 351 lines, consumed by multiple LDA bodies). Extending it to `gga/shared/*` is structurally identical.

---

## Area 6 — DensVarsDev Vars arms

| Option | Description | Selected |
|--------|-------------|----------|
| Add only the 3 GGA Vars arms (XC_A_GAA, XC_N_GNN, XC_N_S_GNN_GNS_GSS) | Insufficient for Mode::Potential (needs 2ND_TAYLOR) | |
| **Add 7 arms: 3 GGA + 4 2ND_TAYLOR** (D-10) | **Both GGA PartialDerivatives and Mode::Potential have what they need** | **✓ (D-10, D-11, D-12)** |
| Add all remaining 24 arms (metaGGA + gradient-components) | Scope-creep; metaGGA Vars land in Phase 4 | |

**Auto-mode rationale:** 7 arms is the minimum for Phase 3's declared scope (GGA + Mode::Potential). Adding more pre-empts Phase 4 work. Adding fewer blocks Mode::Potential.

---

## Area 7 — Mode::Potential implementation approach

| Option | Description | Selected |
|--------|-------------|----------|
| Clean-room reimplementation (derive divergence construction from physics) | Violates algorithmic-identity rule | |
| **Line-for-line port of `XCFunctional.cpp:637-790`** | **Preserves operation order + 1e-12 parity** | **✓ (D-13, D-14, D-15)** |
| Separate LDA Potential kernel + deferred GGA Potential | Partial delivery; GGA MODE-02 is the headline requirement | |

**Auto-mode rationale:** The 1e-12 parity contract demands algorithmic identity with the C++ reference. D-13 commits to the line-for-line port; D-14 holds the 1e-12 threshold; D-15 transcribes `output_length` verbatim.

---

## Area 8 — erf-bearing GGA tolerance (BECKECAMX / BECKESRX)

| Option | Description | Selected |
|--------|-------------|----------|
| Hold 1e-12 using erf_precise libm port (Phase-2 solution) | Strict; inherit Phase-2 Fix 1 (commit `dca382a`) | |
| Relax to 1e-7 matching the D-24 LDAERF override | Unjustified — LDAERF override is upstream-sourced, GGA erf usage is not | |
| **Provisionally hold 1e-12; escalate via PLANNING INCONCLUSIVE if fixture-gate shows drift** | **Follows Phase 2 D-19 locked precedent** | **✓ (D-18)** |

**Auto-mode rationale:** Extending the 1e-7 override blindly would violate D-19 ("no blanket relaxation"). Phase 2's in-kernel libm-port `erf_precise` tightened LDAERFX from 1e-7 → 1e-14; applying it to GGAs should achieve 1e-12 without further override. If not, the escalation path is the correct response.

---

## Area 9 — LB94 scope

| Option | Description | Selected |
|--------|-------------|----------|
| Include LB94 in Phase 3 | Forces FunctionalId enum extension + manual descriptor + special-case dispatch; scope-creep into facade territory | |
| **Defer LB94 to Phase 5 (or Phase 4 alias-treatment)** | **LB94 uses legacy `setup_lb94` pattern, no FUNCTIONAL macro, not in the 78-entry enum** | **✓ (D-19)** |
| Defer LB94 entirely (drop from REQUIREMENTS) | Violates requirement coverage; REQUIREMENTS GGA-10 explicitly lists LB94 | |

**Auto-mode rationale:** LB94 has no well-defined energy per its own .cpp comment ("LB94 is basically LDA with a GGA modification _for the potential_"). It's structurally different from the other 39 GGAs. Facade layer (Phase 5) is where new enum variants are least disruptive and where the LB94's facade-level special-casing (potential-only, no energy) fits naturally.

---

## Area 10 — ACC-04 residual re-run (Phase 2 forward-action)

| Option | Description | Selected |
|--------|-------------|----------|
| Skip re-run (forward VWN/PW/PZ to Phase 6 unchanged) | Ignores the Phase-2 SUMMARY explicit forward-action | |
| **Dedicated Wave 5 re-runs tier-2 at `--order 2` for VWN3C/VWN5C/PW92C/PZ81C after all GGA substrate work completes** | **Fulfils Phase-2 SUMMARY action item; may upgrade ACC-04 Partial → Complete** | **✓ (D-20, D-21)** |
| Re-run at start of Phase 3 (before GGA substrate changes) | Wasted effort — the Phase-2 SUMMARY's hypothesis is that the GGA build_densvars redesign may tighten the drift; pre-redesign re-run tests nothing new | |

**Auto-mode rationale:** The Phase-2 SUMMARY is explicit that "VWN/PW/PZ near-clamp precision → Phase 3" is contingent on the GGA-era build_densvars redesign. Wave 5 runs AFTER all substrate work, checking the hypothesis directly. D-21 clarifies that LDAERF bracket cancellation cannot be resolved by build_densvars changes (the cancellation is in the kernel bracket algebra itself) and remains forwarded to Phase 6.

---

## Claude's Discretion (deferred to planner)

- Per-functional file layout (flat vs per-family subdirectories).
- Kernel-name prefix (`xcfun_eval_gga_<fn>_kernel` vs `xcfun_eval_<fn>_kernel`).
- Shared helper module granularity (e.g., whether to fuse `shared/pbex.rs` with `shared/pw91_like.rs`).
- Regen-registry handling of `#ifdef XCFUN_REF_*_MU` in FUNCTIONAL macros (Phase-2 precedent: trust vendor state).
- B97 polynomial coefficient table structure (single parameterised kernel vs 6 distinct bodies).
- Exact wave internal ordering within each family wave.

---

## Deferred Ideas (captured, not acted on in Phase 3)

- **LB94** → Phase 5 (facade) or Phase 4 (alias-treatment); REQUIREMENTS GGA-10 to be amended.
- **Mode::Contracted** → Phase 4.
- **Orders 5..=6** → Phase 4 (Contracted) / N/A (PartialDerivatives caps at 4).
- **Criterion GGA benches (PERF-01)** → Phase 6.
- **Phase 6 libm-hybrid resolution for LDAERF** → unchanged from Phase 2 forward.

---

## Auto-Resolved Summary

- **Total gray areas auto-resolved:** 10
- **Total decisions written to CONTEXT.md:** 25 (D-01..D-25)
- **Upstream locked decisions inherited:** 53 (28 from Phase 1 + 25 from Phase 2)
- **Claude's Discretion (planner decides):** 6
- **Deferred ideas preserved:** 11
- **Scope-creep redirected:** 1 (LB94 → Phase 5; noted but not acted on)

## External Research

Not performed this session — `--auto` mode uses codebase evidence (Phase 1/2 CONTEXT + locked decisions in STATE.md + xcfun-master C++ reference inspection + crates/xcfun-{ad,eval,core}/src inspection) as the decision substrate. Phase-3 research, if needed, is the planner's call at `/gsd-plan-phase 3` time via the `gsd-phase-researcher` agent.

---

*Generated: 2026-04-24 by `/gsd-discuss-phase 3 --auto`*
