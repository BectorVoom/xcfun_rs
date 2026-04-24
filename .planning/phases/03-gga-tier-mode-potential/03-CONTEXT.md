# Phase 3: GGA Tier + `Mode::Potential` - Context

**Gathered:** 2026-04-24 (discuss-phase `--auto`, Claude auto-selected recommended defaults for 10 gray areas)
**Status:** Ready for planning
**Supersedes:** None (first context for Phase 3). Inherits locked decisions from Phase 1 (`01-CONTEXT.md` D-01..D-28) and Phase 2 (`02-CONTEXT.md` D-01..D-25).

<domain>
## Phase Boundary

Ship the **40 GGA functional bodies** across 10 families (the REQUIREMENTS.md GGA-01..GGA-10 listing expands to 40 `XC_*` IDs in the FunctionalId enum; ROADMAP's "45" is a soft count that folds in a few shared helpers) + implement `Mode::Potential` for the `CTaylor<f64, 2>` divergence construction + extend `Mode::PartialDerivatives` to orders 3..=4 — all layered on the Phase 2 cubecl-native substrate (`DensVarsDev<F>` + `#[cube] fn <name>_kernel<F, const N>` + `dispatch_kernel` + tier-2 parity harness).

A user can run:

```
cargo xtask validate --backend cpu --order 2 --filter 'gga'
```

and see **zero failures at 1e-12 relative error** against the C++ reference on the 10 000-point seeded grid, for every `(functional, vars, mode ∈ {PartialDerivatives, Potential}, order ∈ 0..=4, density point)` tuple that is legal per `eval_setup`.

**In scope:**
- **xcfun-ad primitives (Wave 0 preamble):** Add `expm1_expand` + `ctaylor_expm1` and `sqrtx_asinh_sqrtx` helper (composed op) — both required by the GGA C++ bodies but absent from the Phase 1 output.
- **GGA shared helpers (Wave 0):** port `pw91_like_x_internal::{chi2, S2, prefactor, pw91k_prefactor, pw91xk_enhancement}`, `pbex::{enhancement, enhancement_RPBE, energy_pbe_ab}`, PBEC/APBEC `pbec_eps::A_expm1`, OPTX helpers, B97 polynomial helpers (b97x.hpp / b97c.hpp / b97xc.hpp), and the KTX/BTK/CSC scalar constants.
- **DensVarsDev new Vars arms (Wave 0):** `XC_A_GAA` (inlen=2), `XC_N_GNN` (inlen=2), `XC_A_B_GAA_GAB_GBB` (inlen=5, already exists — verify GGA fields populate the derived gnn/gns/gss correctly for GGAs, not just TW/VWK), `XC_N_S_GNN_GNS_GSS` (inlen=5), plus the four 2ND_TAYLOR variants for `Mode::Potential`: `XC_A_2ND_TAYLOR` (inlen=10), `XC_A_B_2ND_TAYLOR` (inlen=20), `XC_N_2ND_TAYLOR` (inlen=10), `XC_N_S_2ND_TAYLOR` (inlen=20). Comptime if-chain extends the Plan 02-03 pattern.
- **40 GGA `#[cube] fn <name>_kernel<F, const N>` bodies** in `crates/xcfun-eval/src/functionals/gga/*.rs`:
  - **PBE family (12, GGA-01):** `XC_PBEX` (5), `XC_PBEC` (4), `XC_REVPBEX` (19), `XC_RPBEX` (20), `XC_PBESOLX` (74), `XC_PBEINTX` (72), `XC_PBEINTC` (71), `XC_SPBEC` (21), `XC_PBELOCC` (73), `XC_ZVPBESOLC` (69), `XC_ZVPBEINTC` (76), `XC_VWN_PBEC` (22).
  - **Becke family (4, GGA-02):** `XC_BECKEX` (6), `XC_BECKECORRX` (7), `XC_BECKESRX` (8), `XC_BECKECAMX` (9) — last two compose `ctaylor_erf` (inherit Phase-2 in-kernel `erf_precise` libm port).
  - **Becke–Roussel (3, GGA-03):** `XC_BRX` (10), `XC_BRC` (11), `XC_BRXC` (12).
  - **LYP (1, GGA-04):** `XC_LYPC` (16).
  - **OPTX (2, GGA-05):** `XC_OPTX` (17), `XC_OPTXCORR` (18).
  - **PW86/PW91 (4, GGA-06):** `XC_PW86X` (1), `XC_PW91X` (26), `XC_PW91C` (77), `XC_PW91K` (27).
  - **P86 (2, GGA-07):** `XC_P86C` (56), `XC_P86CORRC` (57).
  - **APBE (2, GGA-08):** `XC_APBEX` (68), `XC_APBEC` (67).
  - **B97 family (6, GGA-09):** `XC_B97X` (60), `XC_B97C` (61), `XC_B97_1X` (62), `XC_B97_1C` (63), `XC_B97_2X` (64), `XC_B97_2C` (65).
  - **KT/BTK/CSC (3, GGA-10 minus LB94):** `XC_KTX` (23), `XC_BTK` (58), `XC_CSC` (66).
- **`Mode::Potential`** (MODE-02): line-for-line port of `XCFunctional.cpp:637-790` divergence loop. New `#[cube] fn potential_lda_kernel<F>` (N=1, VAR0 seeding) and `potential_gga_kernel<F>` (N=2, three-direction `nabla · dE/dg` construction). Host-side reject for metaGGA `Dependency::LAPLACIAN | KINETIC` variants. `output_length`: 2 (nspin=1) or 3 (nspin=2) per `XCFunctional.cpp:482-490`.
- **Orders 3..=4 for `Mode::PartialDerivatives`** (MODE-01 extension beyond Phase 2's 0..=2 scope): the existing kernel signature `#[cube] fn <name>_kernel<F, const N>` is already generic over N; extension lands incrementally as each family's fixture-gate passes at order 2 first, then bumps to 4. No new API surface — just dispatcher `N` values.
- **`dispatch_kernel` extension:** add 40 comptime arms (one per new GGA functional id). Host-side `supports(id)` bitmap bumped from 11 to 51.
- **Registry extension:** rerun `xtask regen-registry` to populate 40 additional `FunctionalDescriptor` entries (test_in, test_out, test_threshold extracted from FUNCTIONAL macros). `ALIASES` remains empty this phase (Phase 4 owns the 46 aliases).
- **Tier-2 harness extension:** extend `validation/build.rs` to cc-compile the GGA `.cpp` + `.hpp` files incrementally per family wave; shrink `validation/c_stubs.cpp` (from 67 stubs to ~27). Grid generator already populates gradient strata (design intentionally preseeded in Plan 02-06 for Phase-3 reuse).
- **Phase 2 ACC-04 residual re-run:** dedicated capstone wave retests VWN3C/VWN5C/PW92C/PZ81C at order 2 after all GGA `build_densvars` redesign work lands, to check whether the redesign incidentally tightens the near-clamp drift. If still INCONCLUSIVE, forward to Phase 6 unchanged.

**Out of scope (downstream):**
- **LB94** — see D-07 below. LB94 uses a legacy C++ `setup_lb94` pattern (not `FUNCTIONAL` macro) and is NOT in the 78-entry FunctionalId enum; deferred to Phase 5 (facade + enum extension) or Phase 4 if it is sensibly expressible as an alias. REQUIREMENTS GGA-10 will be amended accordingly.
- **15 metaGGA bodies** (MGGA-01..05, Phase 4).
- **46 aliases including `camcompx` negative-weight canary** (ALIAS-01..06, Phase 4).
- **`Mode::Contracted` at any order** (MODE-03, Phase 4).
- **Orders 5..=6 for `Mode::Contracted`** (Phase 4).
- **Full 78-entry `FUNCTIONAL_DESCRIPTORS`** — Phase 3 takes populated entries from 35 → ~75; the remaining ~3 (metaGGA + LB94-style specials) land in Phase 4/5.
- **CUDA / Wgpu backends** — Phase 3 remains cubecl-cpu only (Phase 6).
- **C ABI (`xcfun-capi`, Phase 5); Python (`xcfun-py`, Phase 7).**
- **Full `Functional` API surface (RS-01..10)** — Phase 3 keeps the minimal dispatcher from Phase 2, extending only with the Mode::Potential dispatch path and the expanded `supports(id)` bitmap.
- **Criterion benchmarks** — Phase 3 ships `--release` parity only; PERF-01/02 are Phase 6.

</domain>

<decisions>
## Implementation Decisions

### GGA scope + wave strategy

- **D-01:** **Port 40 GGA functional IDs** (not 45 as ROADMAP loosely states). The authoritative count is the intersection of `REQUIREMENTS.md` GGA-01..GGA-10 explicit IDs and the Phase-2 `FunctionalId` enum = 40 entries. ROADMAP Phase 3 "Goal" sentence to be corrected by the planner. **Authorisation:** Auto-mode default; the correct count is derivable from enum inspection and is non-negotiable for dispatcher arm-count.
- **D-02:** **Wave 0 = substrate extension (xcfun-ad primitives + DensVarsDev Vars arms + GGA shared helpers).** Rationale: every GGA family depends on at least one of `expm1` / `sqrtx_asinh_sqrtx` / `pw91_like` / `pbex::enhancement`; without the substrate, per-family ports would duplicate helpers. Wave 0 is atomic (one commit per primitive / arm / helper) and MUST pass a fixture-gate before any functional wave begins. **Authorisation:** Auto-mode recommended default — Phase 2 established the "extract shared substrate before family ports" pattern (Plans 02-01 → 02-03).
- **D-03:** **Wave-based parallelism: Wave 1 = PBE+Becke+BR+LYP (20 functionals); Wave 2 = OPTX+PW86/PW91+P86+APBE (10); Wave 3 = B97+KT/BTK/CSC (9); Wave 4 = Mode::Potential (LDA + GGA divergence); Wave 5 = orders 3..=4 bump + tier-2 full-matrix run + Phase-2 ACC-04 residual re-run.** Inside each family wave, per-functional ports are embarrassingly parallel. Planner decides exact wave layout and may split / fuse waves based on the pattern-mapper output. **Authorisation:** Auto-mode default — mirrors Phase 2's Wave-2 parallel pattern (Plans 02-04 + 02-05 in parallel) at scale.
- **D-04:** **`dispatch_kernel` gets 40 new comptime arms.** Pattern identical to the 11 LDA arms already shipped (Plan 02-04 Wave-1B + Plan 02-05 Wave-1C). `supports(id)` bitmap bumped from 11 ids to 51 ids. No fn-pointer dispatch, no runtime registry — cubecl monomorphizes at launch site. **Authorisation:** Auto-mode default — direct extension of Phase 2 D-03 + D-21.

### xcfun-ad substrate extensions (Phase 1 output bump)

- **D-05:** **Add `expm1_expand` + `ctaylor_expm1` to `xcfun-ad`.** Used by PBEC (`pbec.cpp:23`), APBEC (`apbec.cpp:34`), SPBEC (`spbec.cpp:27`), PBEINTC (`pbeintc.cpp:34`), PBELOCC, TPSSLOCC (Phase 4 anticipates), and the REVPBEX enhancement formula (`pbex.hpp:43`). Mandatory for GGA-01/07/08 families. Port target: `xcfun-master/external/upstream/taylor/tmath.hpp` `expm1_expand` if present, else derive via `exp_expand` with a stable-bracket x → 0 branch (mirrors the Phase 2 LDAERFX `expm1`-stable-bracket Fix 1 pattern). Fixture-gate: generate golden coefficients via `xtask regen-ad-fixtures` and run `golden_expand`. **Authorisation:** Auto-mode default; not a Phase-1 regression — Phase 1 success criteria are met; this is an additive Phase-3 substrate extension explicitly called out in RESEARCH as missing.
- **D-06:** **Add `sqrtx_asinh_sqrtx` helper (composed op) to `xcfun-ad`.** Used by PW91X/PW91K enhancement formula (`pw9xx.hpp:87`). Upstream C++ implements it as a dedicated special function to avoid the non-differentiable `sqrt(x) * asinh(sqrt(x))` at `x=0`. Port as `#[cube] fn ctaylor_sqrtx_asinh_sqrtx<F, const N>(x, out, n)` composing `ctaylor_sqrt` + `ctaylor_asinh` + `ctaylor_mul`, with the `x → 0` limit handled by an `expand`-level stable-bracket matching the C++ implementation. **Authorisation:** Auto-mode default; GGA-06 cannot be ported without it.
- **D-07:** **No other xcfun-ad additions.** `ctaylor_exp / erf / asinh / sqrt / log / pow / powi_*/ atan / reciprocal` are sufficient for the remaining 38 GGA bodies. `ctaylor_cbrt` shipped in Phase 1 covers PW86X / PBEX prefactors. Any further primitive shortfall discovered at fixture-gate is escalated via PLANNING INCONCLUSIVE per Phase 1 D-03. **Authorisation:** Auto-mode default.

### GGA shared helpers (extracted, not inlined)

- **D-08:** **GGA shared helpers live in `crates/xcfun-eval/src/functionals/gga/shared/`** (one module per C++ `.hpp` helper cluster). Target modules:
  - `shared/pw91_like.rs` — `chi2`, `S2`, `prefactor`, `pw91k_prefactor`, `pw91xk_enhancement` (source: `pw9xx.hpp`).
  - `shared/pbex.rs` — `enhancement`, `enhancement_RPBE`, `energy_pbe_ab` (source: `pbex.hpp`). REVPBE uses the same `enhancement` with a different R parameter.
  - `shared/pbec_eps.rs` — `A_expm1_inner` used by PBEC/APBEC/SPBEC/PBEINTC (source: `pbec_eps.hpp`).
  - `shared/b97_poly.rs` — degree-4 polynomial combinators for B97 family (source: `b97x.hpp` / `b97c.hpp` / `b97xc.hpp`).
  - `shared/optx.rs` — OPTX/OPTXCORR enhancement with Becke cutoff (source: `optx.cpp` + `optxcorr.cpp` inline helpers).
  - `shared/constants.rs` — per-family scalar constants (R_pbe=0.804, R_revpbe=1.245, mu_pbe, kappa_revpbe, R_rpbe, B97 linear coefficients table, KTX constant a=0.006, BTK constants, CSC constants).
  Pattern mirrors Phase 2's `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs` + `pw92eps.rs` helper modules. **Authorisation:** Auto-mode default; avoids ~600 lines of helper duplication across PBE-family bodies.
- **D-09:** **Each shared helper is a `#[cube] fn` generic over `F: Float` with the single-launch kernel-signature convention** (inputs by `&Array<F>` or `&CTaylor<F, N>`, outputs by `&mut Array<F>`, `#[comptime] n: u32`). No host-side scalar stubs. **Authorisation:** Auto-mode default; matches Phase 1 D-09 + Phase 2 D-03.

### DensVarsDev new Vars arms

- **D-10:** **Add 7 new comptime arms in `build_densvars` (Wave 0).** Current arms: XC_A_B (id=2), XC_A_B_GAA_GAB_GBB (id=6). New arms (all required by at least one Phase-3 GGA or by `Mode::Potential`):
  - `XC_A_GAA` (id=4, inlen=2) — alpha-only GGA; unused by the GGA-01..10 listing but is in the canonical Vars table; implement for completeness + host-side rejection fallback.
  - `XC_N_GNN` (id=5, inlen=2) — total-density GGA variant; the "derived basis" form.
  - `XC_N_S_GNN_GNS_GSS` (id=7, inlen=5) — spin-resolved total-density GGA variant (derived basis).
  - `XC_A_2ND_TAYLOR` (id=26 per xcfun.h positional ordering, inlen=10) — Mode::Potential alpha-only.
  - `XC_A_B_2ND_TAYLOR` (id=27, inlen=20) — Mode::Potential spin-resolved.
  - `XC_N_2ND_TAYLOR` (id=28, inlen=10) — Mode::Potential total-density.
  - `XC_N_S_2ND_TAYLOR` (id=29, inlen=20) — Mode::Potential total + spin-resolved.
  The XC_A_B_GAA_GAB_GBB arm from Plan 02-05 already populates `gaa`, `gab`, `gbb`, `gnn`, `gns`, `gss` correctly — no changes required for GGA families that use id=6. **Authorisation:** Auto-mode default; exact Vars list verified against `xcfun-master/api/xcfun.h:86-121` + `densvars.hpp:35-218`.
- **D-11:** **Use explicit helper-function chains (never C-style fallthrough)** in every new variant arm, per CORE-05 + Phase 2 Pitfall P5 + Plan 02-05 Wave-1C-1 pattern. The `_2ND_TAYLOR` arms chain to the corresponding non-Taylor arm after populating the Taylor coefficients. **Authorisation:** Auto-mode default.
- **D-12:** **Pre-seeded CTaylor input layout per Plan 02-04 Wave-1B-14a amendment.** Each Vars arm reads `input: &Array<F>` as a flat `(inlen × (1<<N))` block of pre-seeded CTaylor coefficients (not raw scalars). Host-side `Functional::eval` packs the derivative-seed markers. The `_2ND_TAYLOR` variants pre-seed the 2nd-order Taylor expansion coefficients for the divergence construction in `Mode::Potential`. **Authorisation:** Auto-mode default; inherits locked decision from Phase 2.

### Mode::Potential implementation

- **D-13:** **Line-for-line port of `xcfun-master/src/XCFunctional.cpp:637-790`** into two host-side dispatch paths plus two `#[cube] fn` bodies:
  - **LDA potential path** (host): check `fun->depends & XC_GRADIENT == 0`; launch `potential_lda_kernel<F>` with `N=1` (CTaylor<F,1>); output layout `[energy, pot_alpha]` or `[energy, pot_alpha, pot_beta]` (2 or 3 doubles).
  - **GGA potential path** (host): check `fun->depends & XC_GRADIENT != 0` and `!(fun->depends & (XC_LAPLACIAN | XC_KINETIC))`; launch `potential_gga_kernel<F>` with `N=2` three times (d/dx, d/dy, d/dz); accumulate `-nabla · dE/dg` into the same output layout [energy, pot_alpha(, pot_beta)].
  - **metaGGA rejection** at `eval_setup` — returns `XcError::InvalidMode` for `fun->depends & (LAPLACIAN | KINETIC)` under Mode::Potential.
  - **Vars compatibility gate** at `eval_setup` — Mode::Potential with `fun->depends & XC_GRADIENT` rejects any vars not in `{XC_A_2ND_TAYLOR, XC_A_B_2ND_TAYLOR, XC_N_2ND_TAYLOR, XC_N_S_2ND_TAYLOR}` with `XcError::InvalidVars` (mirrors `XCFunctional.cpp:437-447`).
  **Authorisation:** Auto-mode default — algorithmic-identity contract requires operation-order identity with C++, so line-for-line is non-negotiable.
- **D-14:** **`Mode::Potential` parity threshold: strict 1e-12** on all 40 GGAs + the 11 Phase-2 LDAs (reran under the new mode), subject to the D-24 LDAERF 1e-7 override inheritance where the `erf_precise` libm port doesn't fully resolve the cancellation chain. No blanket Mode::Potential relaxation. If fixture-gate at Wave 4 shows tier-2 drift > 1e-12 on any GGA, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03 / Phase 2 D-19. **Authorisation:** Auto-mode default — follows Phase 2 D-19 locked precedent.
- **D-15:** **`output_length` returns 2 or 3 for Mode::Potential** matching `XCFunctional.cpp:482-490`:
  - Returns 2 when `fun->vars ∈ {XC_A, XC_A_2ND_TAYLOR}` (inlen = 1 or 10).
  - Returns 3 otherwise (spin-resolved).
  Encodes MODE-05. **Authorisation:** Auto-mode default — direct spec transcription.

### Orders 3..=4 for Mode::PartialDerivatives (MODE-01 extension)

- **D-16:** **Extend orders incrementally per family wave.** Each family's kernel passes tier-2 at orders 0/1/2 (Phase-2 scope) first, then extends to orders 3/4 in a dedicated sub-task. The kernel signature `#[cube] fn <name>_kernel<F, const N>` is already generic over N; extension requires only (a) regen-ad-fixtures at N=3 and N=4 for any composed op that doesn't already ship fixtures at that order, and (b) tier-2 harness invocation at `--order 4` per family. **Authorisation:** Auto-mode default; Phase 1 already validated CTaylor primitives at orders 0..=3 (AD-06 SC #4 states "orders 0..=3"; order 4 is inside the CTaylor<F, N<=7> bound).
- **D-17:** **Wave 5 runs the full-matrix tier-2 at `--order 4`** across all 51 functionals (11 LDA + 40 GGA) and commits an updated `validation/report.html` + `validation/report.jsonl` snapshot. Any order-3/4 residual outside 1e-12 is documented with D-19 INCONCLUSIVE status and forwarded to Phase 6 if unrecoverable. **Authorisation:** Auto-mode default.

### `erf`-bearing GGA tolerance

- **D-18:** **BECKESRX + BECKECAMX hold strict 1e-12** by inheriting the Phase-2 in-kernel libm-port `erf_precise` (Plan 02-06 commit `dca382a`). Phase 1's cubecl polyfill `Float::erf` (~1.3e-8 ULP) is NOT used on the numerical path. If Wave-1 fixture-gate shows drift > 1e-12 on either functional, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03 / Phase 2 D-19. **The D-24 LDAERF 1e-7 override does NOT extend to GGAs** — LDAERF's override was upstream-sourced (`ldaerfx.cpp:66`); GGA `erf` usage is not upstream-documented to suffer analogous cancellation. **Authorisation:** Auto-mode default — extends Phase 2 D-19 locked precedent.

### LB94 scope exclusion

- **D-19:** **LB94 deferred from Phase 3 to Phase 5 (or Phase 4 as an alias if feasible).** Rationale:
  1. LB94 uses the legacy C++ `setup_lb94` pattern (not `FUNCTIONAL` macro) in `xcfun-master/src/functionals/lb94.cpp:1-60`. It is NOT emitted by `xtask regen-registry`.
  2. LB94 is NOT in the 78-entry `FunctionalId` enum (Phase 2 locked at COUNT=78 per CORE-07 test).
  3. LB94 has no well-defined energy (per its own `.cpp` comment): "the LB94 energy is not well defined, here its …"; it is a GGA-modified LDA _potential_.
  4. Adding LB94 to Phase 3 forces: (a) extending `FunctionalId` enum to COUNT=79 with manual discriminant assignment, (b) hand-authoring a `FunctionalDescriptor` entry (regen-registry won't extract it), (c) special-case dispatch + host-side Mode::Potential-only enforcement.
  These changes are load-bearing on the Phase-5 facade (RS-01..10) + C-ABI (CAPI-01..07), which is where the full `Functional` API surface lands and where new enum variants are least disruptive.
  **Action items:** (a) planner updates REQUIREMENTS.md GGA-10 wording to note LB94 deferred; (b) planner adds a deferred-ideas entry pointing to this decision; (c) post-Phase-5, revisit as either a FunctionalId extension or an alias pattern. **Authorisation:** Auto-mode default; blocks no other Phase-3 deliverable; prevents Phase 3 from exfiltrating into facade work.

### Phase 2 ACC-04 residual forward-action

- **D-20:** **Wave 5 re-runs tier-2 at `--order 2` for VWN3C/VWN5C/PW92C/PZ81C** AFTER all GGA substrate + body work completes. Rationale: the Phase-2 SUMMARY noted the 1-3 ULP near-clamp drift at `min(a,b) ∈ [2e-14, 1e-11]` may be incidentally tightened by the GGA-era build_densvars redesign (new Vars arms may alter the regularize + derived-field numerical chain). Possible outcomes:
  - **All 4 turn GREEN at 1e-12:** upgrade ACC-04 from Partial → Complete in REQUIREMENTS.md; remove the "forwarded to Phase 3" annotation; close the issue.
  - **Still INCONCLUSIVE:** no-op — the Phase-2 annotation already forwards them to Phase 6 (libm-hybrid) as the final fallback.
  No regression allowed: tier-2 Order 0/1 at 1e-12 for those 4 LDAs must remain GREEN in the Wave-5 run. **Authorisation:** Auto-mode default — fulfils the Phase-2 SUMMARY's explicit "re-run tier-2 after" action item.
- **D-21:** **LDAERFX/LDAERFC/LDAERFC_JT are NOT retested** at Wave 5 — Phase 2 documented "Rust = mpmath ground truth; C++ itself diverges by 6.7%" at the LDAERFX failing point. Phase 3 build_densvars redesign cannot change this (the cancellation is in the LDAERFX kernel's bracket algebra, not in density-var construction). Forwarded to Phase 6 unchanged. **Authorisation:** Auto-mode default; Phase 2 SUMMARY is explicit.

### Validation harness extension (incremental per wave)

- **D-22:** **`validation/build.rs` extends incrementally per family wave** — after each wave's Rust bodies ship, append the corresponding C++ `.cpp` + `.hpp` files to the `cc::Build::file(...)` list + shrink `c_stubs.cpp` accordingly. The `xtask regen-registry` rerun at each wave emits fewer stubs as more functionals become non-stub.
- **D-23:** **Grid generator is unchanged from Plan 02-06.** The 10k-point stratified 70/30 grid with fixed seed `0x1234abcd` (xoshiro256++) already populates gradient strata (1000 points with `|∇ρ|² ∈ [1, 1e6]`) explicitly for Phase-3 reuse. The `XC_A_B_GAA_GAB_GBB`-variant inputs are synthesised from `(a, b, gradient_alpha_x, gradient_alpha_y, gradient_alpha_z, gradient_beta_x, gradient_beta_y, gradient_beta_z)` per `densvars.hpp:58-64`. **Authorisation:** Auto-mode default — Phase 2 design intent.
- **D-24:** **No new CI targets** — Phase 3 reuses the Phase 2 CI job set: `fmt`, `clippy`, `test` (tier-1 self-tests), `xtask validate --backend cpu --order 2` (tier-2 parity gate extended to 51 functionals; runtime budget ~60 s on the standard runner), `xtask regen-registry --check` (drift gate), `xtask check-no-mul-add` (grep gate extended to `crates/xcfun-eval/src/functionals/gga/**/*.rs`), `xtask check-no-fma` (asm gate unchanged). **Authorisation:** Auto-mode default.

### Error model additions

- **D-25:** **`XcError::InvalidMode` + `XcError::InvalidVars`** (already defined in Phase 2 per D-25) now become reachable in `eval_setup` via the Mode::Potential + metaGGA/non-2ND_TAYLOR Vars rejection paths described in D-13. No new error variants — just new rejection paths in the existing match arms. `XcError` remains 9 variants, `Copy + Clone + Debug + non_exhaustive + thiserror::Error`. **Authorisation:** Auto-mode default — direct reuse of Phase 2 error model.

### Claude's Discretion

- **Per-functional file layout:** one file per functional id (`pbex.rs`, `pbec.rs`, etc.) mirroring the LDA-tier layout from Phase 2 (`slaterx.rs`, `vwn3c.rs`, …). Planner may consolidate small-body functionals (e.g., OPTXCORR next to OPTX).
- **Helper-module granularity:** whether `shared/pbex.rs` fuses with `shared/pw91_like.rs` or stays separate — planner picks based on compile-unit cohesion.
- **Exact wave internal ordering** — e.g., which family in Wave 1 starts first, and whether BR/LYP can pipeline with PBE/Becke.
- **Kernel-name prefix** — `xcfun_eval_gga_<fn>_kernel` vs `xcfun_eval_<fn>_kernel` (mirror Phase 2 `xcfun_eval_<fn>_kernel` convention unless the cubecl compiler emits confusing errors).
- **`xtask regen-registry`'s handling of FUNCTIONAL macros with conditional compilation** — `pbex.cpp:33-49` wraps the test data in `#ifdef XCFUN_REF_PBEX_MU`; planner decides whether the extractor evaluates `#ifdef XCFUN_REF_PBEX_MU` as "undefined" (matching vendored default) or hand-patches. Phase 2 precedent: extractor ignores conditional compilation and the `xcfun-master/` vendor state is the source of truth.
- **Mode::Potential output-array layout details** — the `output[0] = energy` convention is explicit, but whether `output[1] = pot_alpha` or `output[1] = pot_beta` is dictated by `XCFunctional.cpp:666-669`; planner verifies the exact ordering at implementation.
- **B97 polynomial coefficient tables** — 6 functionals share 3 parameterisations (B97, B97-1, B97-2); planner decides whether to parametrise a single `#[cube] fn b97_kernel<F, const N>(d, coefs, out, n)` or split into 6 distinct fn bodies. Algorithmic-identity rule permits either; operation order must match C++.

### Folded Todos

None surfaced — STATE.md `Active TODOs` lists only "Plan Phase 3", which is this workflow itself.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth)

- `xcfun-master/src/XCFunctional.cpp:420-800` — `is_gga`, `is_metagga`, `eval_setup`, `output_length`, the full `eval` dispatch with PartialDerivatives orders 0..=4 + Mode::Potential LDA + GGA divergence loop. **The source of truth for D-13 and D-15.**
- `xcfun-master/src/XCFunctional.cpp:637-790` — Mode::Potential body (LDA + GGA). **Line-for-line port target for D-13.**
- `xcfun-master/src/XCFunctional.cpp:501-612` — PartialDerivatives output layout orders 0..=4. **The source of truth for D-16.**
- `xcfun-master/src/functionals/pbex.cpp`, `pbex.hpp`, `pbec.cpp`, `revpbex.cpp`, `rpbex.cpp`, `pbesolx.cpp`, `pbeintx.cpp`, `pbeintc.cpp`, `spbec.cpp`, `pbelocc.cpp`, `zvpbesolc.cpp`, `zvpbeintc.cpp`, `vwn_pbec.cpp` — PBE family (12 functionals, GGA-01).
- `xcfun-master/src/functionals/beckex.cpp`, `beckecorrx.cpp`, `beckesrx.cpp`, `beckecamx.cpp` — Becke family (4, GGA-02).
- `xcfun-master/src/functionals/brx.cpp`, `brc.cpp`, `brxc.cpp` — Becke–Roussel (3, GGA-03).
- `xcfun-master/src/functionals/lypc.cpp` — LYP (1, GGA-04).
- `xcfun-master/src/functionals/optx.cpp`, `optxcorr.cpp` — OPTX (2, GGA-05).
- `xcfun-master/src/functionals/pw86x.cpp`, `pw91x.cpp`, `pw91c.cpp`, `pw91k.cpp`, `pw9xx.hpp` — PW86/PW91 (4, GGA-06) + shared helpers.
- `xcfun-master/src/functionals/p86c.cpp`, `p86corrc.cpp` — P86 (2, GGA-07).
- `xcfun-master/src/functionals/apbex.cpp`, `apbec.cpp` — APBE (2, GGA-08).
- `xcfun-master/src/functionals/b97xc.cpp`, `b97-1xc.cpp`, `b97-2xc.cpp`, `b97x.hpp`, `b97c.hpp`, `b97xc.hpp` — B97 family (6, GGA-09).
- `xcfun-master/src/functionals/ktx.cpp`, `btk.cpp`, `cs.cpp` — KT/BTK/CSC (3, GGA-10 minus LB94).
- `xcfun-master/src/functionals/lb94.cpp` — LB94 (reference ONLY; D-19 defers to Phase 5).
- `xcfun-master/src/densvars.hpp:35-218` — switch-case densvars constructor + C-fallthrough pattern. **Port target for D-10 + D-11** (explicit helper-function chains replace fallthrough).
- `xcfun-master/src/functional.hpp` — FUNCTIONAL macro + ENERGY_FUNCTION expansion (7× fp{N} per functional); drives FUNCTIONAL_DESCRIPTORS shape.
- `xcfun-master/external/upstream/taylor/tmath.hpp` — `expm1_expand` port target for D-05.
- `xcfun-master/external/upstream/taylor/ctaylor_math.hpp` — `sqrtx_asinh_sqrtx` composition pattern for D-06.

### cubecl substrate (Phase 1 output — unchanged)

- `crates/xcfun-ad/src/{ctaylor.rs, ctaylor_rec/*, math.rs, tfuns.rs, expand/*}` — Phase 1 deliverable. D-05 + D-06 extend this surface by 2 new operations; D-07 commits to no further additions.
- `crates/xcfun-ad/src/index.rs` — CNST, VAR0..VAR7 constants used in new `_2ND_TAYLOR` variant arms (D-10) + Mode::Potential seeding (D-13).
- `cubecl_core::prelude::{Float, Array, CUBE}` + `cubecl_cpu::CpuRuntime` — same substrate as Phase 2.

### Phase 2 substrate (consumed unchanged except for additive extensions)

- `crates/xcfun-eval/src/density_vars.rs` — `DensVarsDev<F>` 22-field struct. No changes.
- `crates/xcfun-eval/src/density_vars/build.rs` — existing XC_A_B + XC_A_B_GAA_GAB_GBB arms. D-10 adds 7 new arms here.
- `crates/xcfun-eval/src/density_vars/regularize.rs` — unchanged; preserved by D-11 explicit chains.
- `crates/xcfun-eval/src/dispatch.rs` — `dispatch_kernel` + `supports(id)`. D-04 extends by 40 arms + bitmap bump.
- `crates/xcfun-eval/src/functional.rs` — `Functional` struct + `eval` method. D-13 adds Mode::Potential branch; D-15 adds `output_length` Mode::Potential arms.
- `crates/xcfun-eval/src/functionals/lda/*` — 11 LDA kernels (11 `.rs` files) unchanged; provide the pattern template for D-03 per-family waves.
- `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs`, `pw92eps.rs` — helper-module pattern template for D-08.

### Registry + validation (consumed, extended)

- `xtask regen-registry` — extraction driver (Plan 02-02 Wave-1A-2). Extend invocation set to populate 40 GGA FunctionalDescriptor entries.
- `validation/build.rs` — cc-compile C++ ref. D-22 extends the `cc::Build::file(...)` list per wave.
- `validation/c_stubs.cpp` — auto-generated from non-LDA functionals. Shrinks by 40 as GGAs go live.
- `validation/src/fixtures.rs` — 10k-point stratified grid generator (Plan 02-06). D-23 commits to no changes.
- `validation/report.html` + `validation/report.jsonl` — tier-2 verdict artifacts. D-17 commits an updated snapshot at Wave 5.

### Phase 1 + Phase 2 locked decisions (inherited verbatim)

- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` — 28 locked decisions. Of particular relevance: **D-01 (1e-12 strict on cubecl-cpu), D-02 (no mul_add), D-04 (`CTaylor` is `#[cube]` type), D-09 (Num retired → Float), D-23 (per-functional `#[cube]` bodies land in Phases 2–4)**.
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md` — 25 locked decisions. Of particular relevance: **D-01 (full cubecl-native), D-02 (DensVarsDev as CubeType), D-03 (single generic kernel), D-19 (strict 1e-12 no blanket relaxation), D-24 (LDAERF 1e-7 upstream-sourced override)**.
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-06-SUMMARY.md` + `02-07-SUMMARY.md` — Phase 2 capstone documenting ACC-04 residuals forwarded to Phase 3 (VWN/PW/PZ) and Phase 6 (LDAERF).

### Design brief (updates required per Phase 3)

- `docs/design/02-data-structures.md` §5 (`DensVars`) — needs update: new Vars arms listing (D-10); 2ND_TAYLOR variants for Potential mode.
- `docs/design/03-api-surface.md` — `Functional` API; Phase 3 ships Mode::Potential entry point.
- `docs/design/04-control-flow.md` — dispatcher control flow now includes Mode::Potential LDA + GGA divergence paths (D-13).
- `docs/design/05-module-responsibilities.md` §3 (xcfun-eval) — GGA body layout + shared helpers (D-08).
- `docs/design/06-cubecl-strategy.md` §3 — per-functional inner kernels GGA extension.
- `docs/design/07-accuracy-strategy.md` §4 (tier architecture) + §5 (fixtures) — unchanged; §6 (tolerance budget) may need a GGA row.
- `docs/design/08-error-model.md` — Mode::Potential rejection paths (D-13); no new variants (D-25).
- `docs/design/09-testing-strategy.md` — tier-1 GGA self-tests + tier-2 GGA parity + Wave 5 order-4 extension.
- `docs/design/11-process-and-milestones.md` §M4 — Phase 3 entry/exit criteria.
- `docs/design/12-design-decisions.md` — add Phase-3 decisions section.

### Research (pitfalls + phase mapping)

- `.planning/research/SUMMARY.md` "Implications for Roadmap" → Phase 3.
- `.planning/research/PITFALLS.md` **P2 (libm erf variance — BECKECAMX/BECKESRX drift risk, D-18), P3 (CTaylor layout — resolved), P5 (C-fallthrough — D-11 explicit chains), P8 (cubecl drift — `=0.10.0-pre.3` pin unchanged), P9 (*_expand miscopy — D-05/D-06 fixture-gate), P10 (silent NaN — `expm1` preconditions in D-05), P13 (registry drift — regen-registry rerun per D-22)**.
- `.planning/research/STACK.md` — cubecl `=0.10.0-pre.3`, `rand_xoshiro =0.8.0`.

### Project-level

- `.planning/PROJECT.md` — Core Value (1e-12 parity); "Out of Scope" (no f32 on numerical path; no fast-math).
- `.planning/REQUIREMENTS.md` GGA-01..GGA-10, MODE-01, MODE-02, MODE-05, ACC-04 (partial, D-20 re-run). GGA-10 wording needs an amendment noting LB94 deferred (D-19).
- `.planning/ROADMAP.md` Phase 3 — Goal + 5 Success Criteria. The "45 GGA functionals" figure needs correction to 40 (D-01).
- `.planning/STATE.md` — Accumulated Context + Session Continuity sections.
- `CLAUDE.md` — tech-stack pins (`cubecl =0.10.0-pre.3`, `rand_xoshiro =0.8.0`); f64-only numerical path; anyhow allowed only in validation/xtask/benches.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (from Phase 1 + Phase 2)

- **`crates/xcfun-ad/src/math.rs`** — 11 composed ops (reciprocal, sqrt, exp, log, pow, erf, asinh, atan, powi_0..=10). Phase 3 composes these directly in GGA kernels + adds 2 more (expm1, sqrtx_asinh_sqrtx per D-05/D-06).
- **`crates/xcfun-ad/src/ctaylor.rs`** — ctaylor_add, ctaylor_sub, ctaylor_scalar_mul, ctaylor_zero. GGA bodies compose these extensively.
- **`crates/xcfun-ad/src/ctaylor_rec/mul.rs`** — ctaylor_mul. Every GGA body multiplies CTaylor values.
- **`crates/xcfun-eval/src/density_vars.rs`** — `DensVarsDev<F>` 22-field struct; GGAs read `d.a`, `d.b`, `d.gaa`, `d.gab`, `d.gbb`, `d.gnn`, `d.gns`, `d.gss`, `d.n`, `d.s`, `d.zeta`, `d.r_s`, `d.n_m13`, `d.a_43`, `d.b_43` extensively. Already populated by the existing XC_A_B_GAA_GAB_GBB arm (Plan 02-05 Wave-1C-1).
- **`crates/xcfun-eval/src/functionals/lda/vwn_eps.rs`, `pw92eps.rs`** — helper-module template for D-08 GGA shared helpers. Each is a `#[cube] fn` composing ctaylor_* ops, generic over F, returning via `&mut Array<F>`.
- **`crates/xcfun-eval/src/dispatch.rs`** — 11-arm comptime if-chain pattern. D-04 extends by 40 arms; no structural change.
- **`crates/xcfun-eval/src/functional.rs`** — `Functional::eval` host-side dispatch. D-13 adds Mode::Potential branch; D-15 adds output_length arms.
- **`validation/src/` + `validation/build.rs`** — tier-2 harness. D-22 extends build.rs per wave; grid generator unchanged (D-23).
- **`xtask/src/regen_registry.rs`** — extractor driver. Wave 0 rerun populates 40 GGA FunctionalDescriptor entries.
- **`xcfun-master/src/functionals/`** — C++ reference; `cc`-compiled per `validation/build.rs`. 40 GGA `.cpp` + ~20 helper `.hpp` files are the port source.

### Established Patterns

- **Kernel signature: `#[cube] fn <name>_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32)`** — universal across Phase 2 LDAs; every GGA body adopts verbatim.
- **Kernel body structure:** 1:1 port of C++ `energy_fn(densvars<num> & d)` → Rust `#[cube] fn ... { let t1 = ...; ctaylor_mul(&a, &b, &mut t1, n); ... out[...] = ...; }`. Operation order preserved; mul_add banned (ACC-06); no FMA (Phase 1 `check-no-fma` asm gate).
- **Helper-module pattern** (Phase 2): complex algebra extracted into `lda/<helper>.rs` module (pw92eps.rs is 351 lines), re-used by multiple functional bodies. D-08 extends this to `gga/shared/<module>.rs`.
- **Algorithmic-identity port rule** (Phase 1 D-01 → Phase 2 D-03): verbatim port, preserve recursion structure, no reassociation, no SIMD intrinsics. Let-bindings mirror C++ intermediate names.
- **Fixture-gate escalation** (Phase 1 D-03 + Phase 2 D-19): strict 1e-12; escalate via PLANNING INCONCLUSIVE rather than widen. D-18 continues for GGA erf.
- **Atomic-commits per wave task** (Phase 2 D-09): extended to Phase 3 (planner decides the commit granularity; one commit per substrate extension, one commit per family wave's functional ports grouped by family).
- **`#[forbid(unsafe_code)]`** — xcfun-core + xcfun-eval crate-root attributes; Phase 3 preserves.

### Integration Points

- **Phase 2 → Phase 3:** xcfun-ad receives 2 new primitives (D-05, D-06) — additive, not a rework. xcfun-eval receives 7 new Vars arms (D-10), 40 new kernel fn bodies (D-03), new GGA shared helper modules (D-08), dispatch-kernel extension (D-04), Mode::Potential host-side + kernel paths (D-13, D-15).
- **Phase 3 → Phase 4 (metaGGA):** Wave 0's DensVarsDev Vars arms already cover the `_TAUA_TAUB` + `_LAPA_LAPB` set via the existing `tau`, `taua`, `taub`, `lapa`, `lapb` fields (Plan 02-03 provisioned them for this). Phase 4 adds the metaGGA-specific Vars arms `XC_A_GAA_TAUA`, `XC_A_B_GAA_GAB_GBB_TAUA_TAUB`, etc.; metaGGA kernels read `d.tau`/`d.taua`/`d.taub`/`d.lapa`/`d.lapb`.
- **Phase 3 → Phase 5 (facade + C ABI):** xcfun-rs::Functional re-exports from xcfun-eval::Functional with the full RS-01..10 surface; LB94 scope-creep defers to this phase per D-19.
- **Phase 3 → Phase 6 (GPU):** every new `#[cube] fn` GGA body compiles unchanged for CudaRuntime + WgpuRuntime (subject to f64 support). Phase 6 adds tier-3 parity at 1e-13 (CPU vs CUDA) and 1e-9 (CPU vs Wgpu with erf fallback). The erf_precise libm-port in Mode::Potential GGA divergence will need a Wgpu SHADER_F64 + WGSL f64 sanity check.

### No pre-phase code to remove

Phase 3 is purely additive. The Phase 1 revert pattern (Wave 0 of Plan 01-01) and Phase 2 surgical cleanup (Plan 02-01) have no Phase 3 equivalents — no stale `density_vars.rs`, no `Num` re-export, no enum renames.

### Known Hazards (from research + Phase 2 forensics)

- **P2 — libm erf variance on BECKECAMX/BECKESRX:** the erf_precise Phase-2 port tightened LDAERFX from 1e-7 (polyfill) to 1e-14 (libm-identical); apply verbatim. D-18 commits to this path with fixture-gate escalation if drift > 1e-12.
- **P5 — C-fallthrough in densvars.hpp:35-218:** the switch-case still has multiple fallthrough chains for the 2ND_TAYLOR variants. D-11 commits to explicit helper-function chains; any chain bug shows up immediately at the Wave-0 tier-1 self-test.
- **P9 — `*_expand` miscopy:** D-05 (`expm1`) and D-06 (`sqrtx_asinh_sqrtx`) are new ports at the same fixture-gate standard as Phase 1's inv/exp/log/pow/sqrt/cbrt. New golden coefficients required.
- **PBE-family R_revpbe/kappa_revpbe numerical precision:** `pbex.hpp:23` uses `const parameter R_revpbe = 1.245`; `revpbex.cpp` uses `kappa_revpbe = 0.804 * 1.245 / 0.804 = 1.245`. Verify the PBE numerator/denominator formula (`enhancement(R, ...) = 1 + R - R/t1`) preserves 1e-12 across the fixture grid — escalate if not.
- **B97 polynomial coefficient tables:** `b97xc.hpp` has multiple parameterisations. Extract + commit a constant table; no "magic number" inlining in kernel bodies.
- **PBEC β/γ constants:** `pbec.cpp` uses `beta_gamma / expm1(-eps / (gamma * u3))`. The `expm1` port (D-05) MUST preserve the `x → 0` stable-bracket; otherwise PBEC exhibits ~1e-14 drift at density regions where `u3 → 0`.

</code_context>

<specifics>
## Specific Ideas

- **Kernel-name prefix:** `xcfun_eval_gga_<fn>_kernel` (e.g., `xcfun_eval_gga_pbex_kernel`) so cubecl-compiler error messages cite the right crate + family. Phase 2 used `xcfun_eval_<fn>_kernel` (without the `lda` segment); consider retrofitting for consistency OR keep the Phase-2 naming and let planner decide.
- **File layout (mirror LDA):** `crates/xcfun-eval/src/functionals/gga/<family>/<fn>.rs` (e.g., `gga/pbe/pbex.rs`, `gga/becke/beckex.rs`). OR flat `gga/<fn>.rs` matching Phase 2's LDA flat layout — planner picks.
- **Shared helper module header:** each helper module gets the same 3-item doc header as LDA modules: (1) upstream `.hpp` line range, (2) formula in LaTeX, (3) preconditions.
- **PBE enhancement constant:** `mu = 0.066725 * pi^2 / 3` — keep as `const MU_PBE_F64: f64 = <precomputed>;` not computed at kernel entry, to avoid f32-rounding on `PI::<F>()`. Precompute against `std::f64::consts::PI`.
- **RPBE expm1-bracket opportunity:** `rpbex.cpp` uses `1 - R_pbe * expm1(-mu/R_pbe * S2)`. The `expm1` port from D-05 makes this a one-line kernel after the `S2` helper lands. If D-05 is done right, RPBEX is the shortest GGA body in the phase.
- **Mode::Potential output sign conventions:** `XCFunctional.cpp:666-669` assigns `output[j+1] = out.get(VAR0)` — the `+1` offset reserves `output[0]` for the energy. Preserve verbatim; don't "simplify" to `output[j] = ...`.
- **Regularize invariant preservation:** the new `_2ND_TAYLOR` Vars arms regularize only `out.a[CNST]` (+ `out.b[CNST]` for spin-resolved), preserving ALL higher-order Taylor coefficients (the 2ND_TAYLOR-seeded derivatives). Test: `tests/regularize_2nd_taylor.rs` verifies.
- **`test_in`/`test_out` test data for GGAs:** the FUNCTIONAL macro test data may be wrapped in `#ifdef XCFUN_REF_*_MU` conditionals (e.g., `pbex.cpp:33-49`). Regen-registry extracts the macro-body text; if conditionally compiled, the extractor MUST match the vendored default. Phase 2 precedent: extractor ignores `#ifdef` and trusts the vendor.
- **B97 parameter tables:** `b97x.hpp` / `b97c.hpp` / `b97xc.hpp` — consolidate into `shared/b97_params.rs` with a single `const B97_{X,C,1X,1C,2X,2C}_PARAMS: [f64; N] = [...];` table per functional. Kernel body reads from the table via `#[comptime]` index.

</specifics>

<deferred>
## Deferred Ideas

- **LB94** — see D-19. Deferred to Phase 5 (facade) or Phase 4 (alias-treatment). Blocks nothing this phase. REQUIREMENTS GGA-10 to be amended by the planner.
- **Mode::Contracted at orders 0..=6** — Phase 4 (MODE-03). Phase 3 leaves the Mode variant defined but `eval_setup` rejects with `XcError::InvalidMode`.
- **Orders 5..=6 for Mode::Contracted** — Phase 4 (MGGA).
- **15 metaGGA bodies + 46 aliases** — Phase 4.
- **Full `Functional` API surface (RS-01..10)** — Phase 5.
- **C ABI + cbindgen** — Phase 5 (CAPI-01..07).
- **Python bindings** — Phase 7.
- **CUDA / Wgpu backends** — Phase 6.
- **Criterion benches for GGA (PERF-01 — eval_vec on 100k GGA points within ±10% of C++ wall-clock)** — Phase 6.
- **Phase-6 libm-hybrid resolution for LDAERF bracket cancellation** — D-21 forwards unchanged; Phase 3 does not attempt.
- **Alias-ability of `Mode::Potential` for LDAs** — `Mode::Potential` for LDA is trivially the scalar derivative; whether to pre-compute or always via CTaylor<F,1> is a minor planner call; not user-visible.
- **Regression-gate for tier-2 order 0/1 on 11 LDAs** — D-20 requires no regression; planner adds a CI invariant assertion.

### Reviewed Todos (not folded)

None reviewed at this session (no pending todos surfaced via grep of STATE.md or `.planning/` trees; the only active TODO is "Plan Phase 3" which is this workflow itself).

</deferred>

---

*Phase: 03-gga-tier-mode-potential*
*Context gathered: 2026-04-24 (discuss `--auto`, 10 gray areas auto-resolved with recommended defaults logged inline as D-01..D-25)*
