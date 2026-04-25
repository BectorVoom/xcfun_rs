# Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered and the auto-mode resolution rationale.

**Date:** 2026-04-25
**Phase:** 04-metagga-tier-mode-contracted-aliases
**Mode:** `--auto` (Claude auto-selected recommended defaults; no AskUserQuestion calls; single pass per workflow auto-mode pass cap)
**Areas discussed:** Scope inventory & wave strategy; xcfun-ad BR-Newton-inverse primitive; DensVarsDev metaGGA Vars arms; Alias engine multiplicative semantics; Parameter table; Mode::Contracted DOEVAL port; CTaylor<F, 6> capacity; Dispatch + supports() bitmap; Tier-2 metaGGA tolerance; LB94 disposition; Phase 3 D-19 forward inheritance.

---

## Area 1 — Scope inventory & wave strategy

| Option | Description | Selected |
|--------|-------------|----------|
| 28 metaGGA only | MGGA-01..05 only; defer BR family + CSC to a 4.5 patch phase | |
| 28 metaGGA + 4 Phase-3 carryovers (BRX/BRC/BRXC + CSC) | Pick up Phase 3 D-01-A deferrals; total 32 functional bodies | ✓ |
| 32 + LB94 | Add LB94 here as an alias attempt | |

**User's choice (auto):** Option 2 — 32 functional bodies (28 metaGGA + 4 carryovers).
**Notes:** Phase 3 D-01-A explicitly deferred BR family + CSC to Phase 4 because they need the inlen=11 Vars arm + BR Newton-inverse algebra. Including them here keeps Phase 4 atomic ("functional-tier closeout"). LB94 examined separately in Area 10. Wave layout = 6 waves: Wave 0 (substrate), Wave 1 (TPSS+BR+CSC), Wave 2 (SCAN×10), Wave 3 (M0x+BLOCX), Wave 4 (alias engine + parameters), Wave 5 (Mode::Contracted), Wave 6 (full-matrix tier-2 + sign-off).

---

## Area 2 — xcfun-ad BR-Newton-inverse primitive

| Option | Description | Selected |
|--------|-------------|----------|
| Add `ctaylor_br_inverse` only | Single new xcfun-ad primitive for BR family + BLOCX; rest composes from existing | ✓ |
| Add `ctaylor_br_inverse` + `ctaylor_div` | Take the opportunity to add true division | |
| Defer BR primitive entirely | Push BR family to Phase 5; ship metaGGA-only Phase 4 | |

**User's choice (auto):** Option 1 — single new primitive.
**Notes:** D-02 specifies port target as `xcfun-master/src/functionals/brx.cpp:25-72` (linear-method polynomial inverter). C++ Newton-Raphson with 4-branch initial guess + 20-iter cap + 1e-15 convergence is the algorithmic-identity contract. `ctaylor_div` deferred unless a fixture-gate failure surfaces a need (Phase 1 D-03 escalation). All metaGGA bodies decompose into existing `exp/log/pow/sqrt/cbrt/atan/asinh/erf/expm1` primitives.

---

## Area 3 — DensVarsDev metaGGA Vars arms

| Option | Description | Selected |
|--------|-------------|----------|
| Implement only metaGGA-active arms (ids 12, 13, 16, 17) | Minimal set covering the FUNCTIONAL macros | |
| Implement all 11 metaGGA Vars arms (ids 8..=18 + 23..=26) | Full coverage of the canonical Vars table for completeness + 31-Vars goal | ✓ |

**User's choice (auto):** Option 2 — all 11 arms.
**Notes:** PROJECT.md "Active" lists "All 31 Vars combinations from xcfun-master/src/xcint.cpp" as a v1 deliverable. Phase 4 is the completion phase for the Vars table. The id=17 (JPAA_JPBB) arm is the load-bearing one Phase 3 D-01-A flagged. Pattern mirrors Phase 3 D-10 + Plan 03-01 (explicit chains, no fallthrough, regularize-only-CNST invariant). Source of truth: `xcfun-master/src/densvars.hpp:35-218` + `crates/xcfun-core/src/enums.rs:73-108`.

---

## Area 4 — Alias engine multiplicative semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Faithful FIXME-preserving port | Bit-for-bit match of C++ `xcfun_set` recursion including the EXX-FIXME wart | ✓ |
| Principled fix (rewrite EXX handling) | Correct the FIXME at L390 — only weight functionals, not parameters | |

**User's choice (auto):** Option 1 — faithful FIXME-preserving port.
**Notes:** Algorithmic-identity rule (Phase 1 D-01 → Phase 2 D-03 → Phase 3 D-08) requires bit-for-bit parity with C++ at 1e-12. Any "fix" violates this. Phase 4 inherits the wart; if upstream xcfun ever patches the FIXME, vendor-bump propagates the fix. D-04 implements `xcfun_set` recursion with: (1) functional → settings += value (additive accumulation), (2) parameter → settings = value (overwrite), (3) alias → recurse with `value * weight`. Negative weights propagate via sign in `value * weight`. Case-insensitive name lookup via `eq_ignore_ascii_case`.

---

## Area 5 — Parameter table

| Option | Description | Selected |
|--------|-------------|----------|
| Single `settings: [f64; 82]` array (matches C++) | Functionals + parameters share storage; ParameterId enum 78..=81 | ✓ |
| Split arrays | `functionals: [f64; 78]` + `parameters: [f64; 4]` (cleaner Rust) | |

**User's choice (auto):** Option 1 — single shared array.
**Notes:** C-ABI compatibility (Phase 5) requires exactly the C++ memory layout. C++ `XC_NR_PARAMETERS_AND_FUNCTIONALS = 82` aligns at the int level. Defaults from `common_parameters.cpp:17-27`: RANGESEP_MU=0.4, EXX=0.0, CAM_ALPHA=0.19, CAM_BETA=0.46. `ParameterId` enum sits next to `FunctionalId` in `xcfun-core::enums.rs` with `#[repr(u32)]` discriminants 78..=81 matching the C++ `list_of_functionals.hpp:100-103` extension.

---

## Area 6 — Mode::Contracted DOEVAL port

| Option | Description | Selected |
|--------|-------------|----------|
| Comptime-monomorphized `contracted_kernel<F, const ORDER>` × 7 | One body, comptime ORDER 0..=6; mirrors C++ DOEVAL macro | ✓ |
| Runtime per-order branch | Single kernel with runtime ORDER parameter | |
| Per-functional Contracted kernels (78 × 7 = 546 functions) | Inline Contracted into each functional body | |

**User's choice (auto):** Option 1 — comptime monomorphization.
**Notes:** The C++ DOEVAL macro is itself comptime-monomorphized via `FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )`; mirroring the structure preserves algorithmic identity. cubecl's `#[comptime]` machinery already supports this (Phase 1 / Phase 3 patterns). Output: `1 << order` doubles per active functional, accumulated with weights, written to `output[i] = out.get(i)`. Order cap = 6 (matches `XCFUN_MAX_ORDER`); `eval_setup` returns `XcError::InvalidOrder` for order > 6.

---

## Area 7 — CTaylor<F, 6> capacity

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse Phase 1 CTaylor<F, N> for N=6 (1<<6 = 64 storage) | Phase 1 AD-01 declares N ∈ 0..=7; just exercise it | ✓ |
| Define new CTaylor6 specialization | Avoid generic-N overhead at order 6 | |
| Cap Mode::Contracted at order 4 | Match Phase 3 PartialDerivatives cap | |

**User's choice (auto):** Option 1 — reuse existing generic CTaylor.
**Notes:** Phase 1 AD-01 explicitly declared CTaylor<F, N> valid for N ∈ 0..=7. Phase 1 AD-06 ran property tests at orders 0..=3, and the type construction is generic; orders 5/6 are within the contracted range. Wave 0 fixture-gate (single `cargo test test_ctaylor_n6` smoke test) confirms before any kernel exercises it. Memory budget per kernel invocation: `XC_MAX_INVARS=20 × 64 = 1280 doubles = 10 KB on stack` — within cubecl-cpu budget. Capping at order 4 violates MODE-03 success criterion ("Mode::Contracted at orders 0..=6").

---

## Area 8 — Dispatch + supports() bitmap

| Option | Description | Selected |
|--------|-------------|----------|
| Extend existing `dispatch_kernel` comptime if-chain | 36 new arms for functionals + 7 for Mode::Contracted host-side | ✓ |
| Convert dispatch to fn-pointer table | Runtime FunctionalId → fn lookup | |
| Per-mode separate dispatchers | Split PartialDerivatives / Potential / Contracted | |

**User's choice (auto):** Option 1 — extend comptime if-chain.
**Notes:** Direct extension of Phase 3 D-04 + Phase 2 D-21. cubecl monomorphizes at launch site; fn-pointers would defeat the single-source GPU compatibility (Phase 6). `supports(id)` bitmap bumps from 51 to 87. If `u128` bitmap exceeds capacity (87 < 128, fits), fall back to `[u64; 2]` per planner judgment.

---

## Area 9 — Tier-2 metaGGA tolerance

| Option | Description | Selected |
|--------|-------------|----------|
| Strict 1e-12 default; per-functional override only with upstream documentation | Inherit Phase 3 D-18 + Phase 2 D-19 framework | ✓ |
| Blanket 1e-9 for metaGGA tier (high-degree polynomials sensitive) | Pre-emptively relaxed to absorb expected drift | |

**User's choice (auto):** Option 1 — strict 1e-12 default.
**Notes:** Phase 2 D-19 + Phase 3 D-18 explicitly committed to no blanket relaxation. M06/R4SCAN/BLOCX flagged as high-risk in CONTEXT.md "Known Hazards"; if their fixture-gate fails strict 1e-12, they get D-19 INCONCLUSIVE entries forwarded to Phase 6 (mirroring Phase 3 protocol — 13 entries already there). Mode::Contracted is a re-packaging of Taylor coefficients; if PartialDerivatives passes 1e-12, Contracted at the same order trivially passes.

---

## Area 10 — LB94 disposition

| Option | Description | Selected |
|--------|-------------|----------|
| Add LB94 as a Phase 4 special case (extend FunctionalId enum to 79) | Resolve here rather than carrying forward | |
| Defer LB94 to Phase 5 (final answer) | Confirmed not alias-feasible; structural change belongs in facade phase | ✓ |

**User's choice (auto):** Option 2 — defer to Phase 5.
**Notes:** D-13 examined `xcfun-master/src/functionals/lb94.cpp:1-60` and confirmed:
1. Uses legacy `setup_lb94` pattern (NOT FUNCTIONAL macro).
2. Has no well-defined energy (per its own comment).
3. Is NOT in the 78-entry FunctionalId enum.
All three reasons make it structurally incompatible with the alias engine (which composes energies multiplicatively). Phase 5 owns LB94 — either as a FunctionalId extension to COUNT=79 or as a special-case dispatch branch. Phase 4 does NOT re-attempt the alias-feasibility check.

---

## Area 11 — Phase 3 D-19 forward inheritance

| Option | Description | Selected |
|--------|-------------|----------|
| Inherit unchanged — leave 13 entries forwarded to Phase 6 | Phase 4 budget goes to metaGGA + alias delivery | ✓ |
| Attempt resolution in Phase 4 | Spend wave budget on libm-hybrid for 5+3+5 = 13 GGA functionals | |

**User's choice (auto):** Option 1 — inherit unchanged.
**Notes:** Phase 3 D-18 explicitly forwarded those 13 D-19 entries (5 Wave-3 PW86X/APBEX/APBEC/P86C/PW91C + 3 Wave-4 B97C/B97_1C/B97_2C + 5 from full-matrix SPBEC/PBEINTC/PW91K/P86CORRC/BECKESRX) to Phase 6 because Phase 6 has the libm-hybrid + asm-spot-check tooling that addresses libm/port-order drifts. Phase 4 has neither the tooling nor the budget; spending Phase-4 effort here dilutes metaGGA + alias delivery and produces partial-progress at best. Stay forwarded.

---

## Auto-Resolved (single pass)

All 11 gray areas auto-resolved with recommended defaults per workflow `--auto` mode pass cap (single pass; no re-reads of CONTEXT.md to find "gaps"). Decisions logged inline above and in CONTEXT.md as D-01..D-14.

## Deferred Ideas

Captured in CONTEXT.md `<deferred>` section. Highlights:
- LB94 — Phase 5.
- Mode::Potential for metaGGAs — never in scope (algorithmic-identity).
- Mode::PartialDerivatives orders 5..=6 — not in C++ reference.
- GPU backends + tier-3 parity — Phase 6.
- Criterion benchmarks — Phase 6.
- Phase 6 libm-hybrid resolution for the 13 Phase-3 D-19 forwards — inherited unchanged.
- Phase-3 HUMAN-UAT items — owned by `/gsd-verify-work 3`, not Phase 4.

---

*Audit log gathered: 2026-04-25 (discuss `--auto`)*
