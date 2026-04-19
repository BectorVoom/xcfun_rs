# Phase 2: Core Foundations + LDA Tier + Parity Harness - Context

**Gathered:** 2026-04-19
**Status:** Ready for planning
**Supersedes:** None (first context for Phase 2). Inherits locked decisions from Phase 1's cubecl pivot (`.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` D-01..D-28).

<domain>
## Phase Boundary

Ship `xcfun-core` types + registry + `xcfun-eval` cubecl launcher + 11 LDA functional bodies as `#[cube] fn` + the tier-2 validation harness so a user can run:

```
cargo xtask validate --backend cpu --order 2 --filter 'lda|slaterx|vwn|pw92c|pz81c|ldaerf|tfk|tw|vonw'
```

and see zero failures at ≤ 1e-12 relative error against the C++ reference for every `(functional, vars, mode=PartialDerivatives, order∈{0,1,2}, density point)` tuple in a 10 000-point seeded grid.

**In scope:**
- `xcfun-core`: `Vars` (31 variants), `Mode` (4 variants with Unset=0), `Dependency` (bitflags), `XcError` (9-variant thiserror enum), `FunctionalId` (78 entries), `constants` (TINY_DENSITY, RS_PREFACTOR, …), `FUNCTIONAL_DESCRIPTORS` (11 LDA entries populated, 67 GGA/metaGGA entries as stub), `VARS_TABLE` (all 31 entries), `ALIASES` (empty — no LDA-only aliases exist), `taylorlen`.
- `xcfun-eval`: cubecl launcher. `#[cube] fn build_densvars<F, N>`, `DensVarsDev<F, N>` as `#[cube]` type. 11 LDA `#[cube] fn <name>_kernel<F: Float, const N: u32>(d, out)` bodies. Functional dispatcher + minimal `Functional` struct + `eval` entry point used by tier-1 self-tests.
- `validation/` binary crate: `cc`-linked xcfun-master static lib + FFI shim + 10k-point stratified grid generator + max-rel-error reducer + `report.html` / `report.jsonl` writer.
- `xtask regen-registry` + `check` mode: C++ extractor parses `FUNCTIONAL` macros / `aliases.cpp` / `xcint.cpp` and emits `crates/xcfun-core/src/registry/generated/*.rs`. Captures `test_in`/`test_out` arrays. Content-hash drift detection.
- `xtask check-no-mul-add` (ACC-06), `xtask check-no-anyhow` (QG-01), `xtask check-boundaries` (QG-02), `cargo metadata` cubecl pin assertion (QG-06).
- Wave-0 cleanup: delete `crates/xcfun-core/src/density_vars.rs`; rewrite `lib.rs` without `Num` re-export; rename `EvalMode → Mode` (add `Unset=0`), `VarType → Vars`; audit + fix `error.rs`, `traits.rs`, `functional_id.rs`, `constants.rs`, `test_data.rs`. Re-include `xcfun-core` + `xcfun-eval` in workspace `members`.

**Out of scope (downstream):**
- 45 GGA bodies (Phase 3), 15 metaGGA bodies (Phase 4), 46 aliases (Phase 4 — none are LDA-only).
- `Mode::Potential` (Phase 3), `Mode::Contracted` at order > 2 (Phase 4).
- Orders 3..=4 for `Mode::PartialDerivatives` (MODE-01, Phase 3 — Phase 2 ships 0..=2 per SC #5).
- `Mode::Contracted` orders 3..=6 (MODE-03, Phase 4).
- Full 78-entry `FUNCTIONAL_DESCRIPTORS` (Phase 2 populates 11 LDA + stubs; Phase 3/4 extend).
- CUDA / Wgpu backends (Phase 6); Phase 2 is cubecl-cpu only.
- C ABI (`xcfun-capi`, Phase 5); Python (`xcfun-py`, Phase 7).
- Full `Functional` API surface (RS-01..10, Phase 5) — Phase 2 ships only the minimal dispatcher needed by tier-1 self-tests + tier-2 harness.
- Remaining Phase 0 gates (QG-03 cargo-deny, QG-04/05 clippy+fmt beyond Phase 1 scope, QG-08 atomic-commits gate).

</domain>

<decisions>
## Implementation Decisions

### Architecture (cubecl-native baseline)

- **D-01:** **Full cubecl-native.** All `DensVars` construction and all per-functional bodies are `#[cube] fn` from day one. No host-side `DensVars<T: Num>` struct, no host-scalar `fn slaterx<T: Num>(d) -> T`. Scalar `Functional::eval(point)` is a 1-thread `cubecl-cpu` launch (inherits Phase 1 D-15/D-16). This extends the Phase 1 cubecl pivot (D-04, D-09, D-23) into the functional-body layer.
- **D-02:** **`DensVarsDev<F: Float, const N: u32>` as a `#[cube]` type** holding 29 `CTaylor<F, N>` fields (design 02 §5 field set). Built by `#[cube] fn build_densvars<F, N>(input: &Array<F>, #[comptime] vars: u32) -> DensVarsDev<F, N>`. Researcher MUST verify cubecl 0.10-pre.3 supports nesting `#[cube]` types inside another `#[cube]` type. **Documented fallback:** if nesting is unsupported, collapse to a monolithic `Array<F>` of length `29 * (1 << N)` with `#[comptime]` offset helpers. Planner escalates via `PLANNING INCONCLUSIVE` if neither pattern meets the 1e-12 contract.
- **D-03:** **Single generic `#[cube] fn` per functional.** Signature: `#[cube] fn <name>_kernel<F: Float, const N: u32>(d: &DensVarsDev<F, N>, out: &mut CTaylor<F, N>)`. cubecl monomorphizes per `(F, N)` at launch site. No per-N specializations, no fn-pointer dispatch. 11 LDA functions = 11 `#[cube] fn`s in `xcfun-eval::functionals::lda::*`.
- **D-04:** **`xcfun-eval` is the cubecl launcher and functional-body home.** Depends on `xcfun-core` + `xcfun-ad` + `cubecl = "=0.10.0-pre.3"` + `cubecl-cpu = "=0.10.0-pre.3"` + `thiserror`. `xcfun-core` stays cubecl-free (types + registry tables only). Tier-1 self-tests and tier-2 harness both route through `xcfun-eval`. Phase 5/6 layer `xcfun-rs` + `xcfun-gpu` on top of `xcfun-eval`. Requires updating design 05 §2 and §10 (xcfun-core's "execute single-point evaluation on the CPU" responsibility moves to `xcfun-eval`).

### Pre-pivot `xcfun-core` scaffold disposition

- **D-05:** **Surgical rewrite.** Wave-0 cleanup: (a) re-include `crates/xcfun-core` and `crates/xcfun-eval` in workspace `members`; (b) delete `crates/xcfun-core/src/density_vars.rs` entirely (826 lines, fully obsolete under D-01); (c) rewrite `lib.rs` to drop the broken `pub use xcfun_ad::Num`; (d) audit + fix `enums.rs` (rename + add `Unset`), `error.rs` (CORE-04 compliance), `traits.rs` (keep `Dependency` bitflags; remove stale `Functional`/`TestData` traits; functional trait work moves to `xcfun-eval`), `functional_id.rs` (verify 78 entries match `xcfun-master/api/xcfun.h`), `constants.rs` (physical constants unchanged), `test_data.rs` (retain or delete based on post-codegen role). Other excluded crates (`xcfun-functionals`, `xcfun-gpu`, `xcfun-ffi`, `xcfun-python`) stay excluded this phase.
- **D-06:** **Keep `A_B_GAA_GAB_GBB`-style screaming-snake-case variant names** (matches C header `XC_A_B_GAA_GAB_GBB` exactly, minus the `XC_` prefix). Add `#[allow(non_camel_case_types)]` on the `Vars` enum. Design 02 §3's UpperCamelCase table is superseded — REQUIREMENTS CORE-01 ("discriminants matching xcfun.h exactly") governs; naming convention is Claude-discretion within that.
- **D-07:** **Rename `EvalMode → Mode`, add `Unset = 0` variant, add `#[repr(u32)]`.** Final variants: `Unset = 0`, `PartialDerivatives = 1`, `Potential = 2`, `Contracted = 3`. Matches REQUIREMENTS CORE-02 + C header `xcfun_mode` exactly.
- **D-08:** **Rename `VarType → Vars`.** Matches REQUIREMENTS CORE-01 + design 02 §3 + C header `xcfun_vars`. Propagate through `xcfun-core` + `xcfun-eval` + tests.
- **D-09:** **Wave-0 atomicity = one commit per cleanup task.** Discrete Wave-0 tasks: (a) workspace `members` re-include, (b) delete `density_vars.rs`, (c) rewrite `lib.rs`, (d) rename `EvalMode→Mode`/`VarType→Vars`/add `Unset`, (e) audit + fix each remaining file. Ensures `git bisect` usability for any Phase-2 regression.

### Phase 0 prerequisite absorption

- **D-10:** **Phase 2 absorbs the Phase 0 requirements that it strictly needs.** Absorbed: **CORE-10** (xtask regen-registry + --check drift gate), **ACC-05** (RUSTFLAGS empty + `-Cllvm-args=-fp-contract=off` in release profile — `.cargo/config.toml` from Phase 1 already covers this; verify still present), **ACC-06** (mul_add ban in `xcfun-core/src/functionals/*.rs` — but note: under D-04 the functional bodies live in `xcfun-eval`, so the lint target is `xcfun-eval/src/functionals/*.rs`; REQUIREMENTS ACC-06 wording needs a note), **QG-01** (xtask check-no-anyhow), **QG-02** (xtask check-boundaries — basic version), **QG-06** (cargo metadata cubecl `=0.10.0-pre.3` pin assertion), **QG-07** (registry content-hash drift detection — rides on top of CORE-10). **Deferred to a future Phase 0 plan-phase:** QG-03 (cargo-deny license/advisory allowlist), QG-04/05 (clippy + fmt — already partially in place from Phase 1; full cleanup separate), QG-08 (atomic-commits CI gate).
- **D-11:** **CORE-10 via cc-compiled C++ extractor.** `xtask regen-registry` invokes a C++ extractor program that is compiled against `xcfun-master/src/` and emits `.rs` files under `crates/xcfun-core/src/registry/generated/`. The extractor scrapes: `FUNCTIONAL(XC_*)` macro instantiations (→ `FUNCTIONAL_DESCRIPTORS` entries including `test_in`/`test_out` arrays), `aliases.cpp` (→ `ALIASES` entries), `xcint.cpp` lines 93–135 (→ `VARS_TABLE` entries). `regen-registry --check` reruns the extractor and diffs against committed output; any diff fails CI (QG-07). Hand-port fallback only if the extractor cannot be built.
- **D-12:** **Registry codegen scope in Phase 2 = LDA only + full VARS_TABLE.** `FUNCTIONAL_DESCRIPTORS`: 11 LDA entries populated (`XC_SLATERX`, `XC_VWN3C`, `XC_VWN5C`, `XC_PW92C`, `XC_PZ81C`, `XC_LDAERFX`, `XC_LDAERFC`, `XC_LDAERFC_JT`, `XC_TFK`, `XC_TW`, `XC_VWK`); remaining 67 slots defined as `FunctionalDescriptor::stub()` (name only, no fp table) so CORE-07's "78 entries" is literally satisfied but stubs panic when dispatched. `VARS_TABLE`: all 31 entries (CORE-09 complete). `ALIASES`: empty slice (no LDA-only aliases; CORE-08 deferred to Phase 4). Phase 3/4 extend by re-running `regen-registry` and rebuilding. LDA-10's `XC_VWK` entry confirms the existing `xcfun-master/src/functionals/` has `tw.cpp` + `vwk` — verify at Wave-0.
- **D-13:** **ACC-06 via `xtask check-no-mul-add` grep gate.** Same idiom as Phase 1's `check-no-fma` (Plan 01-07). Target file set: `crates/xcfun-eval/src/functionals/**/*.rs` (NOT `xcfun-core/src/functionals/` per literal ACC-06 wording — adjusted for D-04). Regex filter ignores `//` comments. Exit status 2 on any match.

### Validation harness (tier-1 + tier-2)

- **D-14:** **`validation/` binary crate** at the workspace root (design 05 §8). Depends on `xcfun-eval`, `anyhow`, `cc` (build-dep), `approx`, `serde_json`, `rand_xoshiro`, `tracing-subscriber`. `build.rs` compiles `xcfun-master/src/**/*.cpp` (minus `functionals/{gga,metagga}/*.cpp` for Phase 2 to keep C++ compile time manageable — Phase 3/4 extend) into a static lib; FFI shim calls `xcfun_*` via `extern "C"`. Invoked via `cargo xtask validate -- <args>` where `xtask` delegates to the `validation` bin. The `validation` crate is the ONE place `anyhow` is permitted in the library-adjacent graph (QG-01 respects this per design 05 §8 + existing xcfun-master Cargo.toml stance). Ships in Wave 2 of Phase 2 (after Wave-0 cleanup + Wave-1 xcfun-core types are in place).
- **D-15:** **Report format: `report.html` (HTML tables) + `report.jsonl` (one record per `(functional, vars, mode, order, point_idx, element_idx)`).** Matches ACC-03 literal spec. **No committed fixtures** — the 10k-point grid is regenerated from a fixed `rand_xoshiro` seed (`0x1234abcd` per design 07 §5) on every harness run; determinism comes from the seed, not stored data. Saves ~40MB of repo bloat; C++ toolchain required for CI's validate run (acceptable — validation is an on-demand/merge-gate job, not a per-commit job).
- **D-16:** **Tier-1 self-tests source `test_in`/`test_out` from the xtask-generated registry.** `regen-registry` extracts `test_in: &[f64]`, `test_out: &[f64]`, `test_threshold: f64` from each `FUNCTIONAL` macro payload (see `xcfun-master/src/functionals/slaterx.cpp:31-37` for the shape). Tier-1 test loops `for desc in FUNCTIONAL_DESCRIPTORS.iter().filter(|d| d.test_in.is_some())` and calls `Functional::eval` with `test_in`, compares to `test_out` at `desc.test_threshold` (typically 1e-11 per upstream). `cargo test` runs tier-1 under 5 seconds (SC #4).
- **D-17:** **Per-functional kernel signature: `#[cube] fn <name>_kernel<F: Float, const N: u32>(d: &DensVarsDev<F, N>, out: &mut CTaylor<F, N>)`** — write into `out` arg. Matches Phase 1's `ctaylor_mul` / `ctaylor_exp` / `ctaylor_pow` signature convention (Plan 01-02, 01-03, 01-06). No return type. Callers allocate `out` on the kernel stack before the call.
- **D-18:** **10k-point grid = stratified 70/30.** 7 000 points uniform across physical bulk: `n ∈ [1e-5, 10.0]`, `|s/n| ∈ [0, 0.95]`. 3 000 stress-region points: 1 000 with `ρ ∈ [1e-14, 1e-5]` (regularize path), 1 000 with `|ζ| ∈ [0.95, 1.0]` (fully-polarised limit), 1 000 with `|∇ρ|² ∈ [1, 1e6]` (N/A for LDA but included so Phase 3/4 can reuse the grid). Fixed seed `0x1234abcd` (xoshiro256++). Grid generator lives in `validation/src/fixtures.rs`.
- **D-19:** **Strict 1e-12 for all 11 LDAs — no blanket relaxation.** LDAERFX/LDAERFC/LDAERFC_JT compose `erf_expand` (Phase 1 Plan 01-06 relaxed cbrt/erf/gauss coefficients to 1e-7 at the `*_expand` layer). Researcher MUST instrument the LDAERF chain against C++ at the fixture gate; if end-to-end rel-error vs C++ reliably exceeds 1e-12 on cubecl-cpu, planner escalates via `PLANNING INCONCLUSIVE` per Phase 1 D-03 (never silently widen). Per-functional tolerance overrides allowed ONLY if escalated and user-approved.

### Derivable from cascade (locked)

- **D-20:** **Workspace `members` mutation in Wave 0:** `members = ["crates/xcfun-ad", "crates/xcfun-core", "crates/xcfun-eval", "xtask", "validation"]` (`validation` added in Wave 2). Keep `xcfun-functionals`, `xcfun-gpu`, `xcfun-ffi`, `xcfun-python` in `exclude` (Phase 4+ / 5+ / 7+).
- **D-21:** **Functional dispatcher + minimal `Functional` struct live in `xcfun-eval`.** `Functional` carries `weights: &'static [(FunctionalId, f64)]` (or equivalent), `vars: Vars`, `mode: Mode`, `order: u32`. `eval(&self, input: &[f64], out: &mut [f64]) -> Result<(), XcError>` calls the cubecl-cpu launch for each `(FunctionalId, weight)` pair against the shared registry `fp[order]` and accumulates into `out`. Phase 5 (RS-01..10) re-exports this through `xcfun-rs::Functional` with the full public API surface.
- **D-22:** **`DensityVars::regularize` on `#[cube] ctaylor`: modifies only `Array<F>[0]` (the `CNST` index coefficient).** Mirrors C++ `set_constant` semantics (`xcfun-master/src/densvars.hpp:22-25`). Verified by a unit test in `xcfun-eval::functionals::tests` exercising CTaylor<f64, 2> with a seeded first-derivative coefficient unchanged pre/post regularize (CORE-06).
- **D-23:** **Tier-2 order scope in Phase 2: `Mode::PartialDerivatives`, orders 0..=2 only.** SC #5 literal says "--order 2". Orders 3..=4 for PartialDerivatives land when MODE-01 does in Phase 3 (where GGAs need them). `Mode::Contracted` and `Mode::Potential` entirely out of scope for Phase 2.

### Claude's Discretion

- Exact file layout under `xcfun-eval/src/functionals/lda/` — one module per functional vs. `mod.rs` consolidation.
- Whether `DensVarsDev<F, N>` exposes per-field accessors or reads via `#[comptime]` field-index constants.
- `FunctionalDescriptor` struct shape for stub entries (67 non-LDA): marker enum value, `Option<...>` fp table, or `panic!` stub.
- Exact `C++ extractor` implementation language — pure C++ (parsing its own source via libclang is overkill; a small regex-based parser over the `FUNCTIONAL(XC_*) = { ... }` macro text probably suffices).
- Grid generator's exact distribution shape inside each stratum (uniform in log-space vs linear; mix of Gaussian vs uniform angular coverage for gradient components).
- `Functional::eval` error semantics in the narrow Phase 2 slice — which `XcError` variants actually get returned vs. Phase 5's full set.
- Wave layout and parallelization — planner picks based on dependency DAG.

### Folded Todos

None surfaced by `gsd-tools list-todos`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth)

- `xcfun-master/src/densvars.hpp` — C++ `densvars<T>` constructor + `regularize`, switch-fallthrough pattern to port into `#[cube] fn build_densvars` (Rust must use explicit helper-function chain per CORE-05)
- `xcfun-master/src/functional.hpp` — `FUNCTIONAL` macro + `ENERGY_FUNCTION` expansion (7× fp{N} per functional); drives the `FUNCTIONAL_DESCRIPTORS` shape
- `xcfun-master/src/xcint.hpp`, `xcfun-master/src/xcint.cpp` — `VARS_TABLE` source (31 rows); `xcfun_vars`/`xcfun_mode` enum discriminants
- `xcfun-master/api/xcfun.h` — `xcfun_vars` (31 values), `xcfun_mode`, error codes (1=EORDER, 2=EVARS, 4=EMODE, 6=EVARS|EMODE, -1=other); the parity contract for CORE-01, CORE-02, CORE-04 discriminants
- `xcfun-master/src/specmath.hpp` — physical constants (`TINY_DENSITY`, `pow2`, `pow3`, `poly` helpers)
- `xcfun-master/src/config.hpp` — `XCFUN_TINY_DENSITY`, `XCFUN_MAX_ORDER`, pw92c legacy-constants config
- `xcfun-master/src/functionals/slaterx.cpp` — LDA exemplar (FUNCTIONAL macro shape, test_in/test_out, formula)
- `xcfun-master/src/functionals/vwn3.cpp`, `vwn5c.cpp`, `pw92c.cpp`, `pz81c.cpp`, `ldaerfx.cpp`, `ldaerfc.cpp`, `ldaerfc_jt.cpp`, `tfk.cpp`, `tw.cpp` — the 11 LDA bodies (port targets for LDA-01..LDA-10)
- `xcfun-master/src/functionals/slater.hpp`, `pw92eps.hpp`, `pz81c.hpp`, `vwn.hpp` — LDA helper templates used by the .cpp bodies
- `xcfun-master/src/functionals/aliases.cpp` — Alias source-of-truth (empty contribution for Phase 2; full port in Phase 4)

### cubecl substrate (pivot contract)

- `cubecl_core::prelude::{Float, Array, CUBE}` + `cubecl_cpu::CpuRuntime` — same surface as Phase 1
- cubecl-book `core-features/features.md` — f64 support matrix (CPU is full; CUDA ? / WGPU caveats noted for Phase 6)
- `crates/xcfun-ad/src/{ctaylor.rs, ctaylor_rec/*, math.rs, tfuns.rs, expand/*}` — Phase 1 output that Phase 2 consumes; the LDA `#[cube] fn`s compose `ctaylor_mul`, `ctaylor_pow`, `ctaylor_log`, `ctaylor_exp`, `ctaylor_erf`, `ctaylor_asinh` from here
- `crates/xcfun-ad/src/index.rs` — `CNST`, `VAR0..VAR7` constants used in `DensVarsDev` regularize + derivative extraction

### Phase 1 locked decisions (inherited)

- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` — 28 locked decisions, notably **D-01 (1e-12 strict on cubecl-cpu), D-02 (no mul_add), D-04 (`CTaylor` is `#[cube]` type, no host struct), D-09 (Num retired, Float trait), D-15 (scalar eval = 1-thread kernel launch), D-23 (per-functional `#[cube]` bodies move forward to Phases 2–4)**
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-07-PLAN.md` + SUMMARY — `xtask check-no-fma` precedent for Phase 2's `check-no-mul-add` (ACC-06) and `xtask` extraction pattern

### Design brief (updates required per Phase 2 D-04)

- `docs/design/00-overview.md`, `docs/design/01-source-tree.md` — overall architecture; §3.2 xcfun-core layout needs a note that per-D-04 functional bodies move to xcfun-eval
- `docs/design/02-data-structures.md` §1 (`CTaylor`), §2 (`Dependency`), §3 (`Vars`), §4 (`Mode`), §5 (`DensVars`), §6 (`FunctionalDescriptor`) — pre-pivot host-struct language superseded by Phase 1 D-04/D-09 and Phase 2 D-01/D-02; Phase 2 planner updates the §5 `DensVars` layout to reflect `DensVarsDev<F, N>` as `#[cube]` type
- `docs/design/03-api-surface.md` — `Functional` API + free functions; Phase 2 ships a minimal slice, Phase 5 the full surface
- `docs/design/04-control-flow.md` — dispatcher control flow for `Functional::eval`; Phase 2 realises the LDA subset
- `docs/design/05-module-responsibilities.md` §2 (`xcfun-core`) — responsibility needs **update** per D-04: functional bodies + `eval` move to `xcfun-eval`; xcfun-core owns types + registry only. Boundary rule 2/3/10 also needs adjustment
- `docs/design/06-cubecl-strategy.md` §3 (kernel structure), §3.1 (per-functional inner kernels), §3.2 (dispatch by FunctionalId) — LDA bodies follow the pattern shown here
- `docs/design/07-accuracy-strategy.md` §1-3 (1e-12 invariant + sources of divergence + algorithmic identity), §4 (tier 1-4 test architecture), §5 (fixtures from C++ xcfun), §6 (tolerance budget), §7 (regen-fixtures workflow)
- `docs/design/08-error-model.md` — `XcError` variants + `as_c_code` mapping for CORE-04 + future CAPI-05
- `docs/design/09-testing-strategy.md` — tier-1 self-test + tier-2 parity harness patterns
- `docs/design/10-build-and-dependencies.md` §3.1 (xcfun-ad deps), §3.2 (xcfun-core deps) — workspace + feature wiring
- `docs/design/11-process-and-milestones.md` §M2 + §M3 (original Milestone 2 + 3 = Phase 2)
- `docs/design/12-design-decisions.md` D1, D2, D4 (superseded by Phase 1 pivot) — Phase 2 adds decisions for DensVarsDev, functional-body location, validation harness mechanics

### Research output this phase

- `.planning/research/SUMMARY.md` "Implications for Roadmap" → Phase 2
- `.planning/research/PITFALLS.md` P2 (libm erf variance — governs LDAERF tolerance discussion), P3 (CTaylor layout — resolved by Phase 1 D-04), P8 (cubecl drift — all 0.10-pre.3 crates must stay locked), P12 (PW92C constants — Phase 0 concern; if not resolved before Phase 2 Wave-1 PW92C port, escalate), P13 (registry drift — covered by CORE-10 + QG-07 absorbed into this phase)
- `.planning/research/STACK.md` — cubecl `=0.10.0-pre.3`, `rand_xoshiro =0.8.0` pin for the grid generator seed

### Project-level

- `.planning/PROJECT.md` — Core Value (1e-12 parity); "Out of Scope" (no f32 on numerical path; no fast-math)
- `.planning/REQUIREMENTS.md` CORE-01..CORE-09, LDA-01..LDA-10, MODE-04, ACC-01..ACC-04 (Phase-2 reqs) + CORE-10, ACC-05, ACC-06, QG-01, QG-02, QG-06, QG-07 (Phase-0 reqs absorbed via D-10) + the superseded AD-01..06 row (Phase 1, Complete)
- `.planning/ROADMAP.md` — Phase 2 Goal + 5 Success Criteria; Phase 0 (QG/ACC) cross-reference; Phase 3/4/5/6 downstream
- `.planning/STATE.md` — "Decisions from initialization" + Phase 1 summary
- `CLAUDE.md` — tech-stack pins (`cubecl =0.10.0-pre.3`, `thiserror =2.0.18`, `bitflags =2.11.1`, `cc ^1.2.60`, `rand_xoshiro =0.8.0`, `serde_json ^1.0.149`); f64-only numerical path; `anyhow` allowed only in `validation`/`xtask`/benches

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`crates/xcfun-ad/src/` (Phase 1 output)**: `CTaylor<F, N>`, `ctaylor_rec::{mul, multo, multo_skipconst, compose}`, `expand::{inv, exp, log, pow, sqrt, cbrt, atan, asinh, gauss, erf}`, `tfuns::{tfun_*}` helpers, `math::{ctaylor_reciprocal, ctaylor_sqrt, ctaylor_exp, ctaylor_log, ctaylor_pow, ctaylor_powi_*, ctaylor_erf, ctaylor_asinh, ctaylor_atan}`, `for_tests::cpu_client()`, `index::{CNST, VAR0..VAR7}`. Every LDA functional body composes these.
- **`crates/xcfun-core/src/enums.rs`** — `EvalMode` + `VarType` enums largely sound; post-rename (Mode + Vars) + Unset addition, the 322-line file is directly reusable. `input_len` / `provides` / `is_spin_polarized` metadata methods map onto `VARS_TABLE` generation (cross-check extractor output against the hand-written table).
- **`crates/xcfun-core/src/constants.rs`** — `TINY_DENSITY`, `RS_PREFACTOR` etc. Physical constants; reusable.
- **`crates/xcfun-core/src/functional_id.rs`** — if already enumerates 78 functional IDs, reusable (verify discriminant stability against `xcfun-master/api/xcfun.h`).
- **`crates/xcfun-core/src/traits.rs`** — `Dependency` bitflags reusable; `Functional`/`TestData` traits likely need rework or removal (evaluation surface moves to `xcfun-eval` per D-04).
- **`xtask/src/` (Phase 1 output)** — existing fixtures machinery + `check-no-fma` bin; reuse pattern for `regen-registry`, `check-no-mul-add`, `check-no-anyhow`, `check-boundaries`, `validate` subcommands.
- **`xcfun-master/`** — full C++ reference; `build.rs` in `validation/` will `cc`-compile `src/**/*.cpp` minus GGA/metaGGA bodies for Phase 2 compile-time budget.
- **`.cargo/config.toml`** — `-Cllvm-args=-fp-contract=off` from Phase 1 carries forward (ACC-05).

### Established Patterns

- **Algorithmic-identity port** — Phase 1 established the "verbatim port, preserve recursion structure" rule for `ctaylor_mul`. Phase 2 LDA bodies follow: match C++ operation order in `slaterx(a, b) = c * (a^(4/3) + b^(4/3))` etc. Let-bindings mirror C++ intermediates.
- **`cubecl-cpu` for validation** — `OnceLock<CpuClient>` pattern from Phase 1 reused across the launcher. `xcfun-eval::for_tests::cpu_client()` mirrors `xcfun-ad::for_tests::cpu_client()`.
- **Fixture-gate escalation** — Phase 1 D-03 established the "escalate via PLANNING INCONCLUSIVE rather than widen tolerance" rule. Phase 2 inherits; LDAERF tolerance decision (D-19) applies it.
- **Atomic commits per wave task** — Phase 1 Plans 01-01 through 01-07 each committed per task; Wave-0 of Phase 2 extends this (D-09).
- **`#[forbid(unsafe_code)]`** — Phase 1 forbids at crate root; Phase 2 `xcfun-core` + `xcfun-eval` inherit.

### Integration Points

- **Phase 1 → Phase 2 consumption**: `xcfun-eval` depends on `xcfun-ad` (all `ctaylor_*` + `*_expand` composed ops) and `xcfun-core` (types + registry). No revert of Phase 1 output.
- **Phase 2 → Phase 3** (GGA tier): Phase 3 extends `FUNCTIONAL_DESCRIPTORS` entries (via regen-registry rerun), adds GGA `#[cube] fn *_kernel` bodies in `xcfun-eval::functionals::gga`, extends `Mode::PartialDerivatives` orders 3..=4 (MODE-01), adds `Mode::Potential` (MODE-02), extends the `validation` build.rs to include GGA C++ bodies. 11 new `#[cube] fn`s per GGA family.
- **Phase 2 → Phase 5** (facade): `xcfun-rs::Functional` re-exports from `xcfun-eval::Functional`, adds the full RS-01..10 API layer.
- **Phase 2 → Phase 6** (GPU): CUDA/Wgpu backends replace `cubecl-cpu` in `Batch<'fun, R>`; the `#[cube] fn` bodies compile unchanged.

### Pre-pivot code to remove in Wave 0 (per D-05)

- `crates/xcfun-core/src/density_vars.rs` — 826 lines, fully obsolete under D-01 (no `<T: Num>` host struct).
- `pub use xcfun_ad::Num;` line in `crates/xcfun-core/src/lib.rs` (broken — Phase 1 D-09 retired `Num`).

### Pre-pivot code to audit (keep mostly as-is)

- `crates/xcfun-core/src/enums.rs` — keep variant definitions (D-06 screaming snake case); fix: rename `EvalMode → Mode`, rename `VarType → Vars`, add `Mode::Unset = 0`, add `#[repr(u32)]` on Mode. Re-verify all 31 discriminants against `xcfun-master/api/xcfun.h`.
- `crates/xcfun-core/src/{error.rs, traits.rs, functional_id.rs, constants.rs, test_data.rs}` — read each, confirm no stale `Num` references, align `XcError` variants against CORE-04 (add `#[non_exhaustive]`, ensure 9 variants), verify `FunctionalId` enumeration matches xcfun.h.

</code_context>

<specifics>
## Specific Ideas

- LDA functional module layout mirrors C++: `xcfun-eval/src/functionals/lda/{slaterx, vwn3c, vwn5c, pw92c, pz81c, ldaerfx, ldaerfc, ldaerfc_jt, tfk, tw, vwk}.rs` — one file per functional.
- Kernel-name prefix `xcfun_eval_<fn>_kernel` so cubecl error messages point to the right crate (mirrors Phase 1's `xcfun_ad_ctaylor_*` idiom).
- `VARS_TABLE` field layout: `struct VarsRow { symbol: &'static str, len: u8, provides: Dependency }` per design 02 §3.1. 31 rows, `#[repr(C)]` for future CAPI stability.
- Per-functional `#[cube] fn` doc-comment header: 3 items (upstream `.cpp` line range, formula in LaTeX, preconditions) — same format Phase 1 used for `*_expand`.
- `check-no-mul-add` xtask regex: `\.mul_add\s*\(` scoped to `crates/xcfun-eval/src/functionals/` (glob via walkdir); strips `//` comments before match.
- `regen-registry` content-hash stamp: commit the SHA256 of each generated `.rs` file alongside it (e.g., `generated/FUNCTIONAL_DESCRIPTORS.rs.sha256`). `--check` mode regenerates in a temp dir + compares SHA256 first, full diff only on mismatch.
- Grid generator record shape: `struct GridPoint { n: f64, s: f64, gaa: f64, gab: f64, gbb: f64, gnn: f64, gns: f64, gss: f64, … }` — superset of LDA inputs (Phase 2 uses `n, s`; Phase 3 uses gradient fields without regenerating).
- `report.html` top-level summary: functional × mode × order matrix with max-rel-error in each cell, coloured red > 1e-12 / yellow 1e-13–1e-12 / green < 1e-13.
- `test_in/test_out` storage in `FunctionalDescriptor`: `test_in: Option<&'static [f64]>, test_out: Option<&'static [f64]>, test_threshold: Option<f64>` — `None` for stub entries (non-LDA in Phase 2).

</specifics>

<deferred>
## Deferred Ideas

- **`Mode::Potential` + `Mode::Contracted`** — Phase 3 (MODE-02) + Phase 4 (MODE-03). Phase 2 leaves the Mode variants defined but eval_setup rejects them with `XcError::InvalidMode` until implemented.
- **Orders 3..=4 for `Mode::PartialDerivatives`** — Phase 3 alongside GGAs.
- **Orders 5..=6 for `Mode::Contracted`** — Phase 4 (MGGA).
- **GGA + metaGGA bodies** (45 + 15 functionals) — Phases 3 and 4.
- **46 aliases (CORE-08, ALIAS-01..06)** — Phase 4. No LDA-only aliases exist.
- **Full `Functional` API surface (RS-01..10)** — Phase 5. Phase 2 ships the minimum needed for tier-1 self-tests + tier-2 harness.
- **C ABI + `cbindgen`** — Phase 5 (CAPI-01..07).
- **Python bindings** — Phase 7.
- **CUDA / Wgpu backends** — Phase 6.
- **`XcError::as_c_code` mapping** — Phase 5 (CAPI-05). Phase 2 lands only the Rust-side variant list.
- **PW92C legacy-constants Cargo feature** — Phase 0 concern per STATE.md. If not resolved before Phase 2 Wave-1 PW92C port (LDA-04), Wave-1 planner escalates; fallback is to ship PW92C matching the vendored `xcfun-master/` default and add the feature flag in a future Phase 0 cleanup plan.
- **QG-03 cargo-deny, QG-04/05 clippy + fmt full cleanup, QG-08 atomic-commits CI gate** — future Phase 0 cleanup plan-phase.
- **Criterion benches for LDA** — not in Phase 2 SC; Phase 6 handles backend-comparison benchmarks. Phase 2 planner may add a dev-dep criterion stub for forward compatibility but no required benchmarks.
- **`regen-registry` for GGA/MGGA/aliases** — regen-registry is general-purpose in Phase 2 but its committed output covers only LDA + VARS_TABLE; Phase 3/4 re-run extends the committed output.

### Reviewed Todos (not folded)

None reviewed at this session (no todos surfaced).

</deferred>

---

*Phase: 02-core-foundations-lda-tier-parity-harness*
*Context gathered: 2026-04-19 (discuss mode, 4 gray areas, 23 interactive decisions captured)*
