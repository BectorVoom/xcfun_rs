# Phase 2: Core Foundations + LDA Tier + Parity Harness — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `02-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 02-core-foundations-lda-tier-parity-harness
**Mode:** discuss (interactive)
**Areas discussed:** Architecture (cubecl-native vs scalar host), Pre-pivot xcfun-core code disposition, Phase 0 prerequisites + registry codegen, Validation harness + functional shape + test data

---

## Area 1 — Architecture: cubecl-native vs scalar host

| Option | Description | Selected |
|--------|-------------|----------|
| A. Full cubecl-native | All DensVars + functional bodies #[cube] from day one; scalar eval = 1-thread cubecl-cpu launch | ✓ |
| B. Hybrid: scalar host f64 + cubecl AD | Two impls per functional (host f64 + #[cube] AD) | |
| C. Restore host Num shim for f64-only | Narrow Num trait for f64; still need separate #[cube] for AD | |
| D. Other / discuss further | | |

**User's choice:** A. Full cubecl-native.
**Notes:** Matches Phase 1 D-23 (per-functional #[cube] bodies move forward into Phases 2–4). Single source of truth; no drift between scalar and AD paths. ~10μs launch overhead for scalar use accepted.

---

| Option | Description | Selected |
|--------|-------------|----------|
| DensVarsDev<F,N> as #[cube] type | 29 CTaylor<F,N> fields; #[cube] fn build_densvars; fallback to monolithic Array<F> if nesting unsupported | ✓ |
| Monolithic Array<F> with comptime offsets | Single Array<F> of length 29*(1<<N) | |
| Inline per-functional (no shared DensVars) | Each kernel computes only fields it needs | |
| Defer — research output | Planner picks based on researcher findings | |

**User's choice:** DensVarsDev<F,N> as #[cube] type.
**Notes:** Researcher MUST verify cubecl 0.10-pre.3 nested-#[cube]-type support. Documented fallback to monolithic Array<F> of length 29*(1<<N).

---

| Option | Description | Selected |
|--------|-------------|----------|
| Single generic #[cube] fn per functional | slaterx_kernel<F, N>; cubecl monomorphizes | ✓ |
| Per-N specializations (slaterx_n0..slaterx_n6) | 7× source explosion | |
| Trait-object dispatch table at runtime | fn-pointer entries | |
| Other / discuss further | | |

**User's choice:** Single generic #[cube] fn per functional.
**Notes:** Matches Phase 1 ctaylor_mul pattern. 11 × 5 ≈ 55 instantiations for Phase 2 LDA coverage.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Bring in the existing xcfun-eval crate | Re-include in workspace; hosts launcher; xcfun-core stays cubecl-free | ✓ |
| Put launcher in xcfun-core | Pulls cubecl into types-only crate | |
| Tests-only launcher in xcfun-core::for_tests | Defer prod launcher to Phase 5 | |
| Other / discuss further | | |

**User's choice:** Bring in the existing xcfun-eval crate.
**Notes:** xcfun-eval already exists as placeholder. Depends on xcfun-core + xcfun-ad + cubecl + cubecl-cpu. Tier-1 + tier-2 harness route through xcfun-eval; Phase 5/6 layers xcfun-rs + xcfun-gpu on top. Design 05 §2 boundary needs updating.

---

## Area 2 — Pre-pivot xcfun-core code disposition

| Option | Description | Selected |
|--------|-------------|----------|
| Surgical: rewrite obsolete files, keep sound ones | Delete density_vars.rs; fix lib.rs; audit + fix rest; atomic Wave-0 cleanup commits per file | ✓ |
| Wholesale revert (Phase 1 D-21 pattern) | git revert the pre-pivot commits | |
| Wholesale delete + rewrite from scratch | rm crates/xcfun-core/src/* | |
| Keep everything, fix in-place per requirement | No cleanup wave | |

**User's choice:** Surgical: rewrite obsolete files, keep sound ones.
**Notes:** density_vars.rs (826 lines) is fully obsolete under Decision A (D-01). lib.rs `pub use xcfun_ad::Num` is broken (Num retired Phase 1 D-09). Other files need audit but mostly sound.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Keep A_B_GAA_GAB_GBB (screaming snake case) | Matches C header | ✓ |
| Rewrite to UpperCamelCase (ABGaaGabGbb) | Rust convention | |
| Both — canonical Vars + VarsC_* in xcfun-capi | Two names per variant | |

**User's choice:** Keep A_B_GAA_GAB_GBB.
**Notes:** Grep-friendly with C header; add #[allow(non_camel_case_types)] shield.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Rename EvalMode → Mode + add Unset=0 + #[repr(u32)] | Matches CORE-02 + C header exactly | ✓ |
| Keep EvalMode name + add Unset | Diverges from REQUIREMENTS | |
| Keep 3-variant EvalMode, drop Unset | REQUIREMENTS amendment required | |

**User's choice:** Rename EvalMode → Mode + add Unset=0 variant.
**Notes:** Unset=0, PartialDerivatives=1, Potential=2, Contracted=3.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Rename VarType → Vars | Matches REQUIREMENTS + design + C header | ✓ |
| Keep VarType | Amendment required | |

**User's choice:** Rename VarType → Vars.
**Notes:** Mechanical rename through xcfun-core + xcfun-eval + tests.

---

| Option | Description | Selected |
|--------|-------------|----------|
| One atomic cleanup commit per file | Discrete Wave-0 tasks for bisect usability | ✓ |
| Single mega-commit | Faster, harder to bisect | |
| Wholesale rewrite | Contradicts prior 'surgical' answer | |

**User's choice:** One atomic cleanup commit per file.
**Notes:** Wave-0 tasks (a-e): workspace members, delete density_vars, rewrite lib.rs, rename types, audit remaining files.

---

## Area 3 — Phase 0 prerequisites + registry codegen

| Option | Description | Selected |
|--------|-------------|----------|
| Fold Phase 0 prereqs into Phase 2 Wave 0/1 | Absorb QG-01/02/06/07, ACC-05/06, CORE-10 | ✓ |
| Block: plan Phase 0 first | Strict dependency ordering | |
| Minimal absorption (only CORE-10 + ACC-06) | Other QG gates later | |
| Hand-write tables, defer CORE-10 | Fastest path but redundant work | |

**User's choice:** Fold Phase 0 prereqs into Phase 2 Wave 0/1.
**Notes:** Absorbed: CORE-10, ACC-05, ACC-06, QG-01, QG-02, QG-06, QG-07. Deferred to future Phase 0 plan: QG-03 (cargo-deny), QG-04/05 (clippy+fmt), QG-08 (atomic-commits gate).

---

| Option | Description | Selected |
|--------|-------------|----------|
| xtask parses xcfun-master C++ headers directly | cc-compiled C++ extractor scrapes FUNCTIONAL macros | ✓ |
| xtask generates from Rust DSL | Hand-curated per-functional Rust descriptors | |
| Hybrid: codegen metadata via C++, Rust source for rest | Two-source model | |
| Defer — researcher output | | |

**User's choice:** xtask parses xcfun-master C++ headers directly.
**Notes:** Single source of truth; automatic drift detection via content-hash (CORE-10 + QG-07 rolled together). Output: crates/xcfun-core/src/registry/generated/*.rs.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Generate LDA + full VARS_TABLE | 11 LDA descriptors + all 31 vars; ALIASES empty | ✓ |
| Generate full registry upfront (all 78 + 46 + 31) | Stub fps for non-LDA | |
| LDA only + minimal VARS_TABLE (only LDA-needed vars) | Fails CORE-09's 31-entry spec | |

**User's choice:** Generate LDA + full VARS_TABLE.
**Notes:** CORE-09 complete in Phase 2; CORE-07/08 deferred (stubs for non-LDA; full population in Phase 3/4). Multiple incremental regen-registry commits across phases.

---

| Option | Description | Selected |
|--------|-------------|----------|
| xtask check-no-mul-add grep gate | Lightweight grep matching Phase 1 check-no-fma pattern | ✓ |
| Custom clippy lint (dylint) | AST-aware, heavier | |
| #![forbid(...)] crate-level | Too broad | |

**User's choice:** xtask check-no-mul-add grep gate.
**Notes:** Target: crates/xcfun-eval/src/functionals/**/*.rs (adjusted per D-04 — functional bodies live in xcfun-eval, not xcfun-core). Regex `\.mul_add\s*\(` with //-comment filter. Exit status 2 on match.

---

## Area 4 — Validation harness + functional shape + test data

| Option | Description | Selected |
|--------|-------------|----------|
| validation/ binary crate | Design 05 §8 + ACC-01 literal spec | ✓ |
| xtask validate subcommand | Heavier xtask, no new crate | |
| xcfun-eval::tests/parity.rs | Integration test, no report.html | |
| Defer to researcher | | |

**User's choice:** validation/ binary crate.
**Notes:** Depends on xcfun-eval + anyhow + cc + serde_json + approx + rand_xoshiro. build.rs compiles xcfun-master/src/*.cpp (minus GGA/metaGGA for Phase 2) into a static lib. Ships in Wave 2 after Wave-0 cleanup + Wave-1 xcfun-core types.

---

| Option | Description | Selected |
|--------|-------------|----------|
| report.html + report.jsonl | ACC-03 literal spec; on-demand grid regeneration | ✓ |
| Committed JSONL fixtures | ~40MB git blob | |
| Hybrid: golden set + live grid | Two comparison paths | |
| bincode (Phase 1 precedent) | Violates ACC-03 literal spec | |

**User's choice:** report.html + report.jsonl.
**Notes:** No committed fixtures. 10k-point grid regenerated from fixed xoshiro seed (0x1234abcd) per run. CI runs validate on demand/merge-gate, not per-commit.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Extract via xtask regen-registry | Same C++ extractor scrapes test_in/test_out | ✓ |
| Hand-port test arrays | Transcription error risk | |
| Run C++ self_test at build time | Brittle | |

**User's choice:** Extract via xtask regen-registry.
**Notes:** `test_in: Option<&'static [f64]>`, `test_out: Option<&'static [f64]>`, `test_threshold: Option<f64>` fields in FunctionalDescriptor. None for stubs. Content-hash drift detection via QG-07.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Write into &mut CTaylor<F,N> out arg | Matches Phase 1 ctaylor_* convention | ✓ |
| Return CTaylor<F,N> | Less common in Phase 1 code | |
| Defer to researcher/planner | | |

**User's choice:** Write into &mut CTaylor<F,N> out arg.
**Notes:** Signature `#[cube] fn <name>_kernel<F: Float, const N: u32>(d: &DensVarsDev<F,N>, out: &mut CTaylor<F,N>)`. Callers allocate out on kernel stack.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Stratified 70/30 bulk + stress | 7k uniform + 3k stress corners | ✓ |
| Uniform across full range | Misses regularize edge cases | |
| Mirror xcfun-master's test points + jitter | Too few points, jitter isn't principled | |
| Defer to researcher | | |

**User's choice:** Stratified 70/30 bulk + stress.
**Notes:** 7k uniform (n ∈ [1e-5, 10], |ζ| ∈ [0, 0.95]); 3k across stress regions (ρ→0 regularize, |ζ|→±1, high |∇ρ|²). Fixed xoshiro seed.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Strict 1e-12 for all 11 LDAs — no relaxation | LDAERF investigated at fixture gate | ✓ |
| Allow 1e-9 for LDAERFX/C/C_JT (match Phase 1 erf relaxation) | Violates ACC-02 literal spec | |
| 1e-12 non-erf; investigate erf at fixture gate | Evidence-based | |

**User's choice:** Strict 1e-12 for all 11 LDAs — no relaxation.
**Notes:** Phase 1 Plan 01-06 relaxed cbrt/erf/gauss at coefficient layer to 1e-7 — that's upstream polyfill precision. End-to-end LDAERF vs C++ MAY still hold 1e-12 via composition; researcher instruments at fixture gate. If unavoidable drift > 1e-12, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03 (never silently widen).

---

## Claude's Discretion (per CONTEXT.md)

- File layout under xcfun-eval/src/functionals/lda/
- DensVarsDev<F,N> accessor pattern
- FunctionalDescriptor stub shape
- C++ extractor implementation (regex vs libclang)
- Grid distribution shape within each stratum
- Functional::eval error semantics narrow-slice for Phase 2
- Wave layout (planner decides based on DAG)

## Deferred Ideas (per CONTEXT.md)

- Mode::Potential + Mode::Contracted (Phase 3/4)
- Orders 3..=6 (Phase 3/4)
- GGA + metaGGA bodies (Phase 3/4)
- 46 aliases (Phase 4)
- Full Functional API (Phase 5), C ABI (Phase 5), Python (Phase 7), CUDA/Wgpu (Phase 6)
- XcError::as_c_code (Phase 5)
- PW92C legacy-constants feature (Phase 0 or Wave-1 escalation)
- QG-03/04/05/08 (future Phase 0)
- Criterion benches (Phase 6)
- regen-registry for GGA/MGGA/aliases (Phase 3/4 re-runs)
