# Phase 2: Core Foundations + LDA Tier + Parity Harness — Research

**Researched:** 2026-04-20
**Domain:** Rust cubecl-native port of xcfun C++ DFT XC functional library — `xcfun-core` types/registry + `xcfun-eval` cubecl launcher + 11 LDA `#[cube] fn` bodies + tier-2 parity harness
**Confidence:** HIGH (stack/architecture/code-context); MEDIUM (D-19 LDAERF tolerance — propagation analysis is desk-only, not yet measured); HIGH (D-02 cubecl nesting — verified via Context7)

---

<user_constraints>
## User Constraints (from 02-CONTEXT.md)

### Locked Decisions

**Architecture (cubecl-native baseline)**
- **D-01:** **Full cubecl-native.** All `DensVars` construction and per-functional bodies are `#[cube] fn` from day one. No host-side `DensVars<T: Num>` struct, no host-scalar `fn slaterx<T: Num>(d) -> T`. Scalar `Functional::eval(point)` is a 1-thread `cubecl-cpu` launch (Phase 1 D-15/D-16).
- **D-02:** **`DensVarsDev<F: Float, const N: u32>` as a `#[cube]` type** holding 29 `CTaylor<F, N>` fields. Built by `#[cube] fn build_densvars<F, N>(input, #[comptime] vars: u32) -> DensVarsDev<F, N>`. Researcher MUST verify cubecl 0.10-pre.3 supports nesting `#[cube]` types. Documented fallback: monolithic `Array<F>` of length `29 * (1 << N)` with `#[comptime]` offset helpers. Planner escalates `PLANNING INCONCLUSIVE` if neither pattern meets 1e-12.
- **D-03:** **Single generic `#[cube] fn` per functional.** Signature: `#[cube] fn <name>_kernel<F: Float, const N: u32>(d: &DensVarsDev<F, N>, out: &mut CTaylor<F, N>)`. cubecl monomorphizes per `(F, N)` at launch. 11 LDA `#[cube] fn`s in `xcfun-eval::functionals::lda::*`.
- **D-04:** **`xcfun-eval` is the cubecl launcher and functional-body home.** Depends on `xcfun-core` + `xcfun-ad` + `cubecl =0.10.0-pre.3` + `cubecl-cpu =0.10.0-pre.3` + `thiserror`. `xcfun-core` stays cubecl-free (types + registry tables only).

**Pre-pivot scaffold disposition**
- **D-05:** Surgical rewrite. Wave-0: re-include xcfun-core + xcfun-eval in workspace `members`; delete `crates/xcfun-core/src/density_vars.rs` (825 lines, obsolete under D-01); rewrite `lib.rs` (drop `pub use xcfun_ad::Num`); audit + fix `enums.rs` (rename + add `Unset`), `error.rs` (CORE-04 compliance), `traits.rs` (keep `Dependency` bitflags; remove stale `Functional`/`TestData`), `functional_id.rs` (verify 78 entries match `xcfun.h`), `constants.rs` (unchanged), `test_data.rs` (retain or delete based on post-codegen role).
- **D-06:** Keep `A_B_GAA_GAB_GBB`-style screaming-snake-case variant names (matches C header). `#[allow(non_camel_case_types)]` on `Vars`. Design 02 §3's UpperCamelCase superseded by REQUIREMENTS CORE-01 (discriminants matching xcfun.h exactly).
- **D-07:** Rename `EvalMode → Mode`, add `Unset = 0` variant, `#[repr(u32)]`. Final variants: `Unset = 0`, `PartialDerivatives = 1`, `Potential = 2`, `Contracted = 3`. Matches CORE-02 + C header.
- **D-08:** Rename `VarType → Vars`. Matches CORE-01 + design 02 §3 + C header.
- **D-09:** Wave-0 atomicity = one commit per cleanup task: (a) workspace `members` re-include, (b) delete `density_vars.rs`, (c) rewrite `lib.rs`, (d) rename `EvalMode→Mode`/`VarType→Vars`/add `Unset`, (e) audit + fix each remaining file.

**Phase 0 prerequisite absorption**
- **D-10:** Phase 2 absorbs CORE-10 (xtask regen-registry + --check), ACC-05 (RUSTFLAGS empty + `-Cllvm-args=-fp-contract=off` — verify already in place from Phase 1), ACC-06 (mul_add ban — target is `xcfun-eval/src/functionals/*.rs` per D-04 adjustment), QG-01 (xtask check-no-anyhow), QG-02 (xtask check-boundaries — basic), QG-06 (cargo metadata cubecl `=0.10.0-pre.3` pin), QG-07 (registry content-hash drift). Deferred: QG-03 (cargo-deny), QG-04/05 (clippy + fmt full cleanup), QG-08 (atomic-commits CI gate).
- **D-11:** CORE-10 via cc-compiled C++ extractor. Scrapes `FUNCTIONAL(XC_*)` macros, `aliases.cpp`, `xcint.cpp` lines 93-135. Emits `crates/xcfun-core/src/registry/generated/*.rs`. `regen-registry --check` reruns + diffs against committed.
- **D-12:** Registry codegen scope = LDA only + full VARS_TABLE. 11 LDA entries populated; 67 GGA/metaGGA = `FunctionalDescriptor::stub()`; full 31-entry VARS_TABLE (CORE-09 complete); ALIASES = empty slice (no LDA-only aliases — CORE-08 deferred to Phase 4). LDA-10's `XC_VWK` confirmed in `xcfun-master/src/functionals/vonw.cpp` (file is `vonw.cpp`, FUNCTIONAL is `XC_VWK`).
- **D-13:** ACC-06 via `xtask check-no-mul-add` grep gate. Same idiom as Phase 1 `check-no-fma`. Target: `crates/xcfun-eval/src/functionals/**/*.rs` (NOT xcfun-core per D-04).

**Validation harness**
- **D-14:** `validation/` binary crate at workspace root. Deps: xcfun-eval, anyhow, cc (build-dep), approx, serde_json, rand_xoshiro, tracing-subscriber. `build.rs` compiles `xcfun-master/src/**/*.cpp` minus `functionals/{gga,metagga}/*.cpp` for Phase 2. Invoked via `cargo xtask validate -- <args>`. ONE place anyhow is permitted in library-adjacent graph. Ships in Wave 2.
- **D-15:** Report = `report.html` + `report.jsonl` (one record per `(functional, vars, mode, order, point_idx, element_idx)`). NO committed fixtures — grid regenerated from fixed `rand_xoshiro` seed `0x1234abcd`. C++ toolchain required for CI's validate run (on-demand/merge-gate, not per-commit).
- **D-16:** Tier-1 self-tests source `test_in`/`test_out` from xtask-generated registry. Tier-1 uses upstream `desc.test_threshold` (typically 1e-7 to 1e-11 — VARIES per functional, see Critical Findings); tier-2 uses strict 1e-12.
- **D-17:** Per-functional kernel sig: `#[cube] fn <name>_kernel<F: Float, const N: u32>(d: &DensVarsDev<F, N>, out: &mut CTaylor<F, N>)`. Caller allocates `out` before call.
- **D-18:** 10k-point grid = stratified 70/30: 7000 uniform bulk (`n ∈ [1e-5, 10.0]`, `|s/n| ∈ [0, 0.95]`); 1000 regularize stress (`ρ ∈ [1e-14, 1e-5]`); 1000 polarised limit (`|ζ| ∈ [0.95, 1.0]`); 1000 gradient stress (`|∇ρ|² ∈ [1, 1e6]`, N/A for LDA but reused Phase 3+). Fixed seed `0x1234abcd` (xoshiro256++). Generator in `validation/src/fixtures.rs`.
- **D-19:** **Strict 1e-12 for all 11 LDAs — no blanket relaxation.** LDAERF chain composes `erf_expand` (Phase 1 relaxed cbrt/erf/gauss expand to 1e-7). Researcher MUST instrument the LDAERF chain against C++ at the fixture gate; if rel-error reliably exceeds 1e-12 on cubecl-cpu, planner escalates `PLANNING INCONCLUSIVE` per Phase 1 D-03. Per-functional overrides only with user approval.

**Derivable from cascade (locked)**
- **D-20:** Workspace members in Wave 0: `["crates/xcfun-ad", "crates/xcfun-core", "crates/xcfun-eval", "xtask", "validation"]` (`validation` added in Wave 2). Keep `xcfun-functionals`, `xcfun-gpu`, `xcfun-ffi`, `xcfun-python` excluded.
- **D-21:** Functional dispatcher + minimal `Functional` struct in `xcfun-eval`. Carries `weights`, `vars`, `mode`, `order`. `eval(input, out) -> Result<(), XcError>` calls cubecl-cpu launch per `(FunctionalId, weight)` pair.
- **D-22:** `regularize` on `#[cube] CTaylor` modifies only `Array<F>[0]` (CNST). Mirror C++ `set_constant`. Verified by unit test (CORE-06).
- **D-23:** Tier-2 order scope in Phase 2: `Mode::PartialDerivatives` orders 0..=2 only. SC #5 says "--order 2". Orders 3..=4 in Phase 3.

### Claude's Discretion

- Exact file layout under `xcfun-eval/src/functionals/lda/` — one module per functional vs. consolidation.
- `DensVarsDev<F, N>` exposes per-field accessors vs. `#[comptime]` field-index constants.
- `FunctionalDescriptor` struct shape for stub entries (67 non-LDA): marker enum value, `Option<...>` fp table, or `panic!` stub.
- C++ extractor implementation language (regex vs. libclang).
- Grid generator distribution shape inside each stratum.
- `Functional::eval` error semantics in narrow Phase 2 slice — which `XcError` variants surface.
- Wave layout and parallelization — planner picks based on dependency DAG.

### Deferred Ideas (OUT OF SCOPE)

- `Mode::Potential` + `Mode::Contracted` — Phase 3 (MODE-02) + Phase 4 (MODE-03). Phase 2 leaves variants defined; eval_setup rejects them with `XcError::InvalidMode`.
- Orders 3..=4 for `Mode::PartialDerivatives` — Phase 3.
- Orders 5..=6 for `Mode::Contracted` — Phase 4 (MGGA).
- GGA + metaGGA bodies (45 + 15) — Phases 3 and 4.
- 46 aliases (CORE-08, ALIAS-01..06) — Phase 4. No LDA-only aliases.
- Full `Functional` API surface (RS-01..10) — Phase 5.
- C ABI + cbindgen — Phase 5. Python — Phase 7. CUDA / Wgpu — Phase 6.
- `XcError::as_c_code` mapping — Phase 5.
- **PW92C legacy-constants Cargo feature** — Phase 0 concern. If not resolved before Wave-1 PW92C port (LDA-04), Wave-1 planner escalates; fallback ships PW92C matching vendored `xcfun-master/` default + adds feature flag in future Phase 0 cleanup plan.
- QG-03 cargo-deny, QG-04/05 clippy + fmt full cleanup, QG-08 atomic-commits CI gate — future Phase 0 cleanup.
- Criterion benches for LDA — not in Phase 2 SC; Phase 6.
- `regen-registry` for GGA/MGGA/aliases — extractor is general-purpose in Phase 2 but committed output covers only LDA + VARS_TABLE; Phase 3/4 re-run extends.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **CORE-01** | `Vars` enum 31 variants, discriminants matching `xcfun.h` exactly | xcfun.h:86-122 catalogued; existing `enums.rs::VarType` already lists all 31 with `#[repr(u32)]` and matching discriminants — rename to `Vars`, keep names verbatim. |
| **CORE-02** | `Mode` enum: `Unset`, `PartialDerivatives`, `Potential`, `Contracted`; `#[repr(u32)]` matching C ABI | xcfun.h:35-41 confirms `XC_MODE_UNSET = 0`. Wave-0 task per D-07. |
| **CORE-03** | `Dependency` bitflags `#[repr(transparent)] u8` via bitflags 2; `DENSITY|GRADIENT|LAPLACIAN|KINETIC|JP` matching C header | Existing `traits.rs::Dependency` uses `u32`; xcint.hpp:46-50 uses `int` (32-bit). Existing layout works; planner verifies u32 matches against C header. |
| **CORE-04** | `XcError` is `thiserror`-derived `Copy + Send + Sync` enum with `#[non_exhaustive]` and 9 variants: `InvalidOrder`, `InvalidVars`, `InvalidMode`, `UnknownName`, `InputLengthMismatch`, `OutputLengthMismatch`, `NotConfigured`, `InvalidEncoding`, `Runtime` | Existing `error.rs::XcError` has 7 variants (missing `InvalidVars` — currently `InsufficientVars`; missing `InvalidEncoding`, `Runtime`). Wave-1 task to extend + add `#[non_exhaustive]`. NOTE: `Copy` requires no `String` fields — `UnknownName(String)` must change to `UnknownName(&'static str)` or be encoded numerically. |
| **CORE-05** | `DensVarsDev` (per D-01/D-02) populates 29 raw + derived fields for each of 31 `Vars`; helper-function-chain (no fallthrough). REQUIREMENTS wording says `DensVars<T: Num>` but D-01 supersedes — Phase 2 ships cubecl `DensVarsDev` only. | densvars.hpp lines 35-218 catalogued; 29 fields (a, b, gaa, gab, gbb, n, s, gnn, gns, gss, tau, taua, taub, lapa, lapb, zeta, r_s, n_m13, a_43, b_43, jpaa, jpbb — that's 22; plus 7 future-use raw input slots reserved per design 02 §5). 12 case arms (some with fallthrough chains). |
| **CORE-06** | `regularize` modifies only `c[CNST]` — higher-order coefficients preserved (unit test) | densvars.hpp:22-25: `if (x < XCFUN_TINY_DENSITY) x.set(0, XCFUN_TINY_DENSITY)`. Already locked by D-22. |
| **CORE-07** | `FUNCTIONAL_DESCRIPTORS` static array with 78 entries stored in `.rodata`, no runtime init | D-12: 11 LDA populated + 67 stubs. Use `static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; 78] = [...]`. Stub entries can hold `name`, `Option<&'static [fn ptr; 7]>` = None, `test_in: None`, etc. |
| **CORE-08** | `ALIASES` 46 entries | DEFERRED to Phase 4 per D-12. Phase 2 ships an empty slice. |
| **CORE-09** | `VARS_TABLE` 31 entries | xcint.cpp lines 93-135 (verified above). 31 rows of `{symbol, len, provides}`. |
| **LDA-01..LDA-10** | 11 LDA `#[cube] fn`s ported, registered, self-test passes | All 11 functional bodies sampled (slaterx.cpp, vwn3c.cpp, vwn5c.cpp, pw92c.cpp, pz81c.cpp, ldaerfx.cpp, ldaerfc.cpp, ldaerfc_jt.cpp, tfk.cpp, tw.cpp, vonw.cpp). LDA-10 covers BOTH `XC_TW` (in tw.cpp) and `XC_VWK` (in vonw.cpp). |
| **MODE-04** | `Functional::input_length` returns `VARS_TABLE[vars].len as usize` | Existing `enums.rs::input_len` already implements this for all 31 variants. Adapter needed once `VARS_TABLE` is the source of truth (post-codegen). |
| **ACC-01** | Validation harness links xcfun-master via cc, evaluates every supported `(functional, vars, mode, order)` tuple | D-14 + Wave 2. cc 1.2.60 + parallel feature already in workspace deps. |
| **ACC-02** | For every tuple × every output element × every grid point: `|rust - cpp| / max(|cpp|, 1.0) ≤ 1e-12` on CPU | The numerator definition matches `approx::assert_relative_eq!` semantics. Implemented via `validation/` Wave 2. |
| **ACC-03** | Validation harness emits `report.html` + `report.jsonl`; failing element blocks merge | D-15. JSONL via `serde_json`; HTML hand-written from a per-functional max-rel-error matrix. |
| **ACC-04** | Tier-1 self-tests run every `FUNCTIONAL_DESCRIPTORS[id].test_in/test_out` on `cargo test` in under 5s | D-16. 11 functionals × ~5ms cubecl-cpu launch overhead = ~55ms. Well under 5s budget. |
| **CORE-10 (absorbed)** | `xtask regen-registry` regenerates `registry/generated/*.rs`; `--check` blocks on drift | D-11 + D-12. See § "CORE-10 Extractor Recommendation" below. |
| **ACC-05 (absorbed)** | RUSTFLAGS empty + release profile contains `-Cllvm-args=-fp-contract=off` | Already in `.cargo/config.toml` from Phase 1. Wave-0 verifies still present + extends `xtask check-no-fma` scope to xcfun-eval functional bodies. |
| **ACC-06 (absorbed)** | Lint bans `mul_add` inside `xcfun-core/src/functionals/*.rs` | D-13: target adjusted to `xcfun-eval/src/functionals/*.rs` per D-04. |
| **QG-01 (absorbed)** | `cargo xtask check-no-anyhow` passes (no library crate depends on anyhow) | New xtask binary modeled on `check-no-fma` pattern: parses each crate's Cargo.toml under `crates/`, fails if `anyhow` appears in `[dependencies]` (NOT `[dev-dependencies]`). |
| **QG-02 (absorbed)** | `cargo xtask check-boundaries` passes — basic version | New xtask binary: enforces "xcfun-core depends only on `thiserror`, `bitflags`"; "xcfun-eval depends on `xcfun-core`, `xcfun-ad`, cubecl, cubecl-cpu, thiserror"; "validation may depend on anyhow"; via `cargo metadata` JSON. |
| **QG-06 (absorbed)** | `cargo metadata` CI assertion verifies `cubecl =0.10.0-pre.3` | New xtask binary: parses `cargo metadata --format-version 1` JSON, asserts `cubecl == 0.10.0-pre.3` (and same for `cubecl-cpu`). |
| **QG-07 (absorbed)** | Registry content-hash drift detection: `xtask regen-registry --check` fails CI if generated file is stale | D-11. The `--check` mode regenerates to a temp dir, computes SHA-256 of each `generated/*.rs`, diffs against committed `generated/*.rs.sha256` stamp file. |
</phase_requirements>

## Research Summary

Phase 2 is a **structural** phase (types + registry + 11 LDA functional bodies + parity harness), not a numerical-discovery phase. Phase 1 already shipped the full `CTaylor<F, N>` cubecl-native AD substrate (`ctaylor_mul`, `ctaylor_compose`, `*_expand`, composed `ctaylor_*`); Phase 2 stitches them into 11 LDA `#[cube] fn`s by porting `xcfun-master/src/functionals/{slaterx, vwn3c, vwn5c, pw92c, pz81c, ldaerfx, ldaerfc, ldaerfc_jt, tfk, tw, vonw}.cpp` body-for-body, plus a 29-field `DensVarsDev<F, N>` `#[cube]` type whose builder ports `densvars.hpp:35-218` with C-fallthrough flattened into a helper-chain.

**Three load-bearing risks for the planner to resolve before Wave-1:** (1) **D-02 — VERIFIED** — cubecl 0.10-pre.3's `#[derive(CubeType, CubeLaunch)]` explicitly supports nested `#[cube]` types and methods; the planned 29-field `DensVarsDev` is a direct fit. (2) **D-19 — INCONCLUSIVE without execution** — desk analysis shows the LDAERF families pass cubecl through Phase 1 `erf_expand` whose coefficient parity was relaxed to 1e-7. End-to-end propagated rel-error vs C++ has not been measured; quantitative analysis below estimates a *worst-case* of 1e-7 rel-error in `ldaerfx` output (NOT 1e-12) — but **upstream xcfun's own `test_threshold` for LDAERFX is 1e-7** (matching ours), suggesting tier-1 will pass. Tier-2's strict 1e-12 against C++ remains uncertain until measured. **Planner ships Wave-2 LDAERF gate WITH the documented escalation path: if measured rel-error > 1e-12 on the bulk grid, escalate `PLANNING INCONCLUSIVE` and require user approval for per-functional override.** (3) **`tw` and `vwk` are NOT pure LDAs** — both depend on `XC_DENSITY | XC_GRADIENT` and use `XC_A_B_GAA_GAB_GBB` (5-input GGA vars). They share the LDA *tier* in REQUIREMENTS LDA-09/LDA-10 but require gradient-bearing `DensVarsDev` fields. The planner MUST seed Wave-1 `build_densvars` with the GGA-grade `XC_A_B_GAA_GAB_GBB` arm or LDA-09/LDA-10 will not compile.

**Primary recommendation:** Run Wave-0 as five sequential atomic commits (D-09 order); Wave-1 in parallel — Wave-1A (xcfun-core types + registry codegen + xtask QG gates), Wave-1B (xcfun-eval `DensVarsDev` + `build_densvars` + 9 pure LDA `#[cube] fn`s), Wave-1C (LDA-09/LDA-10 GGA-needing kinetic functionals once Wave-1B's `build_densvars` covers `XC_A_B_GAA_GAB_GBB`); Wave-2 sequential — `validation/` cc-link of xcfun-master subset → 10k-point grid generator → tier-2 harness → SC #5 gate run. **One commit per task per D-09.**

---

## Critical Findings

- **CTX cubecl `#[derive(CubeType, CubeLaunch)]` supports nested types and methods (HIGH).** Verified via Context7 query against `/tracel-ai/cubecl` `cubecl-book/src/language-support/struct.md`. The exact pattern needed by D-02 (struct holding 29 nested cubecl-aware fields, `&mut` access in kernel, methods on the struct) is documented and exemplified. See § "D-02 cubecl Nesting Decision". `[VERIFIED: Context7 /tracel-ai/cubecl cubecl-book/src/language-support/struct.md]`

- **Phase 1 already lowered scratch `Array::<F>::new(comptime!(len))` inside `#[cube] fn` bodies on cubecl-cpu (HIGH).** `crates/xcfun-ad/src/math.rs` (committed Plan 01-06) allocates a length-`(n+1)` `Array<F>` inside every composed `ctaylor_*` body and the asm-gate (`xtask check-no-fma`) passes. This means scratch buffers for the 29-field DensVarsDev approach are already a known-working idiom. `[VERIFIED: crates/xcfun-ad/src/math.rs:89-95]`

- **`xcfun-master` upstream `test_threshold` for LDAERFX/LDAERFC is 1e-7, not 1e-11 (HIGH).** Confirmed by direct read of `xcfun-master/src/functionals/ldaerfx.cpp:66` (`1e-7`) and `ldaerfc.cpp:124` (`1e-7`). For LDAERFC_JT no test data is provided (block ends at `ENERGY_FUNCTION(ldaerfc_jt)` line 64 with no `XC_PARTIAL_DERIVATIVES, …` payload). LDA-08 thus has no upstream tier-1 fixture; Phase 2 tier-1 must skip LDAERFC_JT in the self-test loop. Tier-2 (vs C++ runtime evaluation) still tests it. `[VERIFIED: xcfun-master/src/functionals/ldaerfx.cpp:66, ldaerfc.cpp:124, ldaerfc_jt.cpp:64]`

- **`tw` and `vwk` are kinetic-GGA, not pure-LDA (HIGH).** `xcfun-master/src/functionals/tw.cpp:28` declares `XC_DENSITY | XC_GRADIENT` and uses `XC_A_B_GAA_GAB_GBB` (5-input). `vonw.cpp:28` (file is `vonw.cpp`, FUNCTIONAL is `XC_VWK`) does the same. Phase 2 LDA-09/LDA-10 tier-1+tier-2 require Wave-1 `build_densvars` to populate the `XC_A_B_GAA_GAB_GBB` arm. Pure-LDA arms `XC_A_B`/`XC_N_S` cover only LDA-01..LDA-08. `[VERIFIED: xcfun-master/src/functionals/tw.cpp:28-30, vonw.cpp:25-29]`

- **PW92C `XCFUN_REF_PW92C` define controls TWO sources of arithmetic divergence (HIGH).** `pw92eps.hpp:38-41` switches the `omega` denominator (`(2*pow(2,1/3)-2)` vs `0.5198421`) AND `pw92eps.hpp:53-58` switches the prefactor `c` (`8.0/(9.0*(2*pow(2,1/3)-2))` vs `1.709921`). Vendored `config.hpp:35-36` ships `XCFUN_REF_PW92C` UNDEFINED → accurate constants are the default. Phase 2 plan recommendation: ship the accurate constants directly; defer the legacy-constants feature flag entirely (no production C++ build of xcfun ships with REF_PW92C defined; the macro is a developer-time fork-comparison knob). See § "PW92C Legacy Constants". `[VERIFIED: xcfun-master/src/functionals/pw92eps.hpp:36-58, src/config.hpp:35-36]`

- **`XCFunctional.cpp` order=2 dispatcher writes outputs in a specific layout (HIGH).** `XCFunctional.cpp:589-612` (case 2): outer loop `i ∈ [0, inlen)` sets `VAR0 = 1` on `in[i]`; inner loop `j ∈ [i, inlen)` sets `VAR1 = 1` on `in[j]`; output indices are `output[k++]` for second derivatives starting at `k = inlen + 1`, then `output[i+1] = out.get(VAR0)` for first derivatives, then `output[0] = out.get(CNST)` for energy. Output length = `1 + inlen + inlen*(inlen+1)/2 = taylor_len(inlen, 2)`. For `XC_A_B` (inlen=2): output length = 1+2+3 = 6 (matches slaterx test_out which has exactly 6 entries). `[VERIFIED: xcfun-master/src/XCFunctional.cpp:589-612]`

- **xcfun-master vendored at 71 .cpp files in `src/functionals/` (MEDIUM).** Plus `src/{XCFunctional.cpp, xcint.cpp}` = 73 total .cpp files. Phase 2 D-14 says "minus `functionals/{gga,metagga}/*.cpp` for Phase 2"; the actual functionals dir is FLAT (no gga/metagga subdirs) — every functional .cpp lives directly in `src/functionals/`. The Wave-2 cc-build set must enumerate explicitly which .cpp files to include (not just exclude a directory). See § "Validation Harness Bring-Up". `[VERIFIED: ls /home/chemtech/workspace/xcfun_rs/xcfun-master/src/functionals/*.cpp | wc -l = 71]`

- **The 78-entry FunctionalId list in REQUIREMENTS exactly matches `list_of_functionals.hpp` (HIGH).** Counted: 78 entries from `XC_SLATERX` to `XC_PW91C` before `XC_NR_FUNCTIONALS` sentinel. The existing `crates/xcfun-core/src/functional_id.rs` already enumerates 78 with `COUNT = 78` and matching names. Wave-0 task: cross-verify discriminant ORDERING matches xcfun.h enum (the existing rust enum reorders by family for readability — D-12 + CORE-07 require *xcfun.h* ordering for 78 entries to map to fp-table indices via `as u32`). `[VERIFIED: xcfun-master/src/functionals/list_of_functionals.hpp + crates/xcfun-core/src/functional_id.rs:11-102]`

- **Phase 1 `for_tests::cpu_client()` `OnceLock<ComputeClient<CpuRuntime>>` pattern is reusable (HIGH).** `crates/xcfun-ad/src/for_tests/cpu_client.rs:21-32` is the canonical 1-thread launcher pattern. Phase 2 `xcfun-eval` mirrors this: `xcfun_eval::for_tests::cpu_client()` returns `&'static ComputeClient<CpuRuntime>`. Per-D-15 scalar `Functional::eval(point)` IS a 1-thread launch via this client. `[VERIFIED: crates/xcfun-ad/src/for_tests/cpu_client.rs]`

- **Phase 1 ACC-05 already in `.cargo/config.toml` (HIGH).** Both `[build] rustflags = ["-Cllvm-args=-fp-contract=off"]` and `[target.'cfg(all())'] rustflags = ["-Cllvm-args=-fp-contract=off"]` are present (Phase 1 W13 revision). Wave-0 task: verify still present + run `xtask check-no-fma` against new `xcfun-eval` symbols once they exist. `[VERIFIED: /home/chemtech/workspace/xcfun_rs/.cargo/config.toml]`

---

## D-02 cubecl Nesting Decision

### Verdict: **FULL NESTING via `#[derive(CubeType, CubeLaunch)]`** (HIGH confidence)

cubecl 0.10-pre.3 explicitly supports the pattern Phase 2 needs. The cubecl-book `language-support/struct.md` shows:

```rust
// Source: github.com/tracel-ai/cubecl/blob/main/cubecl-book/src/language-support/struct.md
// [VERIFIED via Context7]

#[derive(CubeType, CubeLaunch)]
pub struct Pair<T: CubeLaunch> {
    pub left: T,
    pub right: T,
}

#[cube(launch_unchecked)]
pub fn kernel_struct_mut(output: &mut Pair<Array<f32>>) {
    output.left[UNIT_POS] = 42.0;
    output.right[UNIT_POS] = 3.14;
}

#[cube]
impl Pair<Array<f32>> {
    pub fn sum(&self, index: u32) -> f32 {
        self.left[index] + self.right[index]
    }
}
```

This composition pattern (struct holding `Array<F>` fields, generic over a `CubeLaunch` parameter, with both kernel-arg and method usage) is exactly what `DensVarsDev<F, N>` requires. Confirmed: Phase 1 already exercises a simpler version of this — `crates/xcfun-ad/src/math.rs:89-95` creates `Array::<F>::new(comptime!((n+1) as usize))` as kernel-local scratch and the `xtask check-no-fma` asm-gate passes (PASS recorded in Phase 1 sign-off, commit `2882b58`).

### Recommended `DensVarsDev<F, N>` skeleton

Two implementation patterns are both viable; recommendation is **Pattern A** for clarity and stub-safety:

#### Pattern A: One `Array<F>` of length `1 << N` per field (recommended)

```rust
// crates/xcfun-eval/src/density_vars.rs
use cubecl::prelude::*;

/// Device-side densvars container holding 22 named CTaylor fields.
/// Each field is a length-`1 << N` Array<F>, indexed via the bit-flag
/// scheme from xcfun_ad::index (CNST=0, VAR0=1, VAR1=2, ...).
///
/// 1:1 port of `xcfun-master/src/densvars.hpp:35-244` field set.
/// Field order matches the C++ struct member ordering for code-review
/// parity. (Renames to fully-qualified names per CORE-05.)
#[derive(CubeType, CubeLaunch)]
pub struct DensVarsDev<F: Float> {
    // Raw inputs (extracted from input array by the variant builder)
    pub a: Array<F>,
    pub b: Array<F>,
    pub gaa: Array<F>,
    pub gab: Array<F>,
    pub gbb: Array<F>,
    pub n: Array<F>,
    pub s: Array<F>,
    // Derived
    pub gnn: Array<F>,
    pub gns: Array<F>,
    pub gss: Array<F>,
    pub tau: Array<F>,
    pub taua: Array<F>,
    pub taub: Array<F>,
    pub lapa: Array<F>,
    pub lapb: Array<F>,
    pub zeta: Array<F>,    // s/n
    pub r_s: Array<F>,     // (3/(4*pi))^(1/3) * n^(-1/3)
    pub n_m13: Array<F>,   // pow(n, -1/3)
    pub a_43: Array<F>,    // pow(a, 4/3)
    pub b_43: Array<F>,    // pow(b, 4/3)
    pub jpaa: Array<F>,
    pub jpbb: Array<F>,
}

#[cube]
pub fn build_densvars_xc_a_b<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    // Port of densvars.hpp:65-72 (XC_A_B arm).
    // a = d[0]; regularize(a); b = d[1]; regularize(b); n = a + b; s = a - b;
    ctaylor_from_scalar::<F>(input[0], &mut out.a, n);
    ctaylor_regularize::<F>(&mut out.a, n);
    ctaylor_from_scalar::<F>(input[1], &mut out.b, n);
    ctaylor_regularize::<F>(&mut out.b, n);
    ctaylor_add::<F>(&out.a, &out.b, &mut out.n, n);
    ctaylor_sub::<F>(&out.a, &out.b, &mut out.s, n);
    // Derived (densvars.hpp:213-217):
    ctaylor_div::<F>(&out.s, &out.n, &mut out.zeta, n);
    // r_s, n_m13, a_43, b_43 follow via ctaylor_pow with constant exponents.
    ctaylor_pow::<F>(&out.n, F::new(-1.0/3.0), &mut out.n_m13, n);
    // r_s = pow(3/(n*4*PI), 1/3) — composes a constant 3/(4*PI) with n_m13.
    // …
    // (All other fields zero-initialised by the prelude in DensVarsDev::default
    // path or via an explicit ctaylor_zero call — see densvars.hpp default-zero
    // initialiser pattern at lines 223-244.)
}
```

#### Pattern B: Monolithic `Array<F>` of length `22 * (1 << N)` with `#[comptime]` offsets (FALLBACK)

```rust
const FIELD_A: u32      = 0;
const FIELD_B: u32      = 1;
const FIELD_GAA: u32    = 2;
// … 22 total field IDs …

#[cube]
pub fn dv_get<F: Float>(
    storage: &Array<F>,
    #[comptime] field: u32,
    #[comptime] coeff: u32,
    #[comptime] n: u32,
) -> F {
    let stride = comptime!(1u32 << n);
    storage[(field * stride + coeff) as usize]
}
```

**Pattern A is recommended** because (a) it's the documented cubecl idiom, (b) field-by-field access compiles to the same LLVM IR as Pattern B's offset arithmetic on cubecl-cpu (no overhead measured in Phase 1's Plan 01-06 composed-fn benches), and (c) Pattern A enables the 11 LDA `#[cube] fn`s to read `d.a`, `d.b`, `d.n` directly — matching the C++ source line-for-line per the algorithmic-identity rule (Pitfall P3 prevention).

**Pattern B is the contingency** if Pattern A causes a cubecl monomorphization explosion. We expect monomorphization to stay bounded because Phase 2 instantiates `DensVarsDev<f64>` only at `N ∈ {0, 1, 2}` (per D-23, `Mode::PartialDerivatives` orders 0..=2 = `N=0`, `N=1`, `N=2`). 22 fields × 3 N values × `f64` only = 66 monomorphized type instantiations. cubecl-cpu has handled the Phase 1 11 `#[cube] fn` × 5 N values × f64 case without compile-time complaints (Plan 01-07 sign-off).

### Compile probe (recommended Wave-0 task)

A 30-line Wave-0 task can confirm the pattern compiles before Wave-1 commits to either:

```rust
// crates/xcfun-eval/tests/cubecl_densvars_spike.rs
#[derive(CubeType, CubeLaunch)]
struct Trio<F: Float> {
    a: Array<F>,
    b: Array<F>,
    c: Array<F>,
}

#[cube(launch_unchecked)]
fn fill_trio<F: Float>(out: &mut Trio<F>) {
    out.a[0] = F::new(1.0);
    out.b[0] = F::new(2.0);
    out.c[0] = out.a[0] + out.b[0];
}
```

If this compiles + runs, Pattern A is locked. Total Wave-0 cost ~10 minutes of dev time.

---

## D-19 LDAERF Tolerance Analysis

### Verdict: **PROVISIONAL 1e-12 — measure-then-decide gate at Wave-2** (MEDIUM confidence; planner must include the escalation path)

### What Phase 1 actually relaxed

Phase 1 Plan 01-06 (commit `1a5a744`) relaxed `golden_expand.rs` and `golden_composed.rs` per-cell tolerance for `cbrt_expand`, `erf_expand`, and `gauss_expand` to **1e-7 rel / 1.5e-7 abs on `t[0]`** (the leading coefficient). All other expand fns hold strict 1e-12. The 1e-7 number is upstream's polyfill precision: cubecl's `Float::erf` on cubecl-cpu lowers to a polyfill (`2/√π` constant computed in f32, ~1.3e-8 ULP error), and `cbrt` similarly uses an f32-precision `1/3` exponent path inside the JIT.

**The relaxation is at the `*_expand` *coefficient* layer, not the final scalar output.** A coefficient error of 1e-7 in the Taylor series at `x[CNST]` does not directly map to a 1e-7 error in the final scalar `e[n] = ε(n) * n`.

### Propagation analysis for `ldaerfx`

`ldaerfx.cpp:24-48` (`esrx_ldaerfspin`) is a single-spin-channel function dominated by:

```cpp
// Branch A (a < 1e-9):  -3/8 * rhoa * pow(24*rhoa/PI, 1/3)
// Branch B (1e-9 ≤ a < 100):
//   -(rhoa * pow(24*rhoa/PI, 1/3)) *
//     (3/8 - a*(sqrt(PI)*erf(0.5/a) + (2*a - 4*a^3)*exp(-1/(4*a^2)) - 3*a + 4*a^3))
// Branch C (100 ≤ a < 1e9):  -(rhoa * pow(24*rhoa/PI, 1/3)) / (96*a^2)
// Branch D (a ≥ 1e9):  0
```

The error chain:
1. `pow(24*rhoa/PI, 1/3)` — uses `pow` not `cbrt`. Phase 1 `pow_expand` is 1e-12 strict (NOT relaxed). Output has 1e-12 rel-error in coefficients.
2. `erf(0.5/a)` — Phase 1 `erf_expand` t[0] error ~1.3e-8. **Source of the load-bearing drift.**
3. `exp(-1/(4*a^2))` — Phase 1 `exp_expand` is 1e-12 strict.
4. The bracketed expression is multiplied by `a` before subtraction from `3/8`. `a` is bounded; for typical bulk-density inputs, `μ = 0.4` and `n ~ 1`, `kf ~ 3.09`, `a = μ/(2*kf) ~ 0.065` — small, so Branch B applies and `0.5/a ~ 7.7`.
5. At `0.5/a = 7.7`, `erf(7.7) ≈ 1.0 - 2e-26` (saturated), so the absolute coefficient error from `erf_expand` is bounded by 1e-7 × abs(prefactor) ≈ 1e-7. The product `a * 1e-7 ≈ 6.5e-9`.
6. The full bracketed group ≈ 0.375 - 6.5e-9, and multiplied by `-rhoa * pow(24*rhoa/PI, 1/3) ≈ -1.5` (for `rhoa = 1`), gives outputs ~ -0.56 with absolute error ~ 1e-8 → **rel-error ~ 2e-8**.

**Worst case for `ldaerfx` ≈ 2e-8 rel-error from cubecl's polyfilled `erf`.** This is two orders of magnitude WORSE than the 1e-12 contract.

### What this means for Phase 2

- **Tier-1 self-test PASSES** because upstream's own `test_threshold` for LDAERFX is 1e-7 (`xcfun-master/src/functionals/ldaerfx.cpp:66`). Tier-1 uses `desc.test_threshold` per D-16.
- **Tier-2 vs C++ runtime evaluation FAILS at 1e-12.** The C++ reference uses libm `erf` (1e-15+ accuracy); cubecl-cpu uses a polyfill (1e-7 accuracy). The output drift is determined by libm vs polyfill, not by anything Phase 2 implements.
- **The fix is at the `erf_expand` layer, not the LDA layer.** Phase 1 D-03 mandated escalation rather than tolerance widening, but the escalation path was deferred. Phase 2 surfaces the question concretely.

### Recommended planner action (THREE viable paths)

| # | Approach | Cost | Risk | Recommendation |
|---|----------|------|------|----------------|
| 1 | **Per-functional tier-2 override**: LDAERFX/LDAERFC/LDAERFC_JT use 1e-7 rel-error gate; remaining 8 LDAs hold 1e-12. | Low | Document a known divergence; user must approve. | Recommended IF user signs off on the 1e-7 override (it matches upstream xcfun's own threshold). |
| 2 | **Replace cubecl polyfilled `erf` with a host libm-call**: rewrite `erf_expand` to do the leading `erf(x[0])` call host-side BEFORE kernel launch, pass the f64 result into the kernel as a scalar. Inside-kernel computes only the higher-order coefficients (which use `exp(-x*x)`-style identities and don't need `erf` again). | MEDIUM (rewrite Phase 1 erf_expand path) | Requires Phase 1 amendment + new fixture regen + new asm-gate run. | Defer to Phase 6 (when the same problem hits CUDA/Wgpu). For Phase 2 cubecl-cpu, recommend Approach 1. |
| 3 | **Halt Phase 2 until libm-quality `erf` is wired**: planner escalates `PLANNING INCONCLUSIVE` blocking LDA-06/07/08 until D-19 is resolved upstream. | High (multi-day stall on Phase 2 progress) | LDA-01..05, 09, 10 wait. | Reject — too disruptive. |

**Planner recommendation:** Adopt Approach 1 with the explicit user-approval task in Wave-2 sequence. Tier-1 passes immediately (matches upstream test_threshold). Tier-2 LDAERF parity gate runs at 1e-7 with documented divergence note in `report.html`. Phase 6 revisits with libm-call hybrid when CUDA `erf` deviation also enters scope.

**This is NOT silent tolerance widening** — it's a per-functional override sourced from upstream xcfun's own self-test threshold. The 1e-12 contract holds for the 8 non-LDAERF LDAs.

---

## CORE-10 Extractor Recommendation

### Recommended approach: **Hybrid — `cc::Build` + minimal C++ extractor printing JSONL records**

Three options compared:

| Option | Build deps | Implementation cost | Robustness | CI cost |
|--------|-----------|---------------------|------------|---------|
| **(a) Pure-C++ with regex/string parse** | C++17 compiler + cc 1.2.60 | LOW (~150 lines C++) | Brittle to macro spacing, comment-stripping edge cases | C++ compile (~5s) — required ANY way for Wave-2 cc-link |
| (b) C++ with libclang | libclang.so + cc + bindgen | HIGH (~600 lines C++ + AST traversal) | Robust to formatting | LARGE (~30s libclang-bringup); requires libclang on CI runner |
| (c) Pure-Rust regex over .cpp text | regex crate | MEDIUM (~250 lines Rust) | Brittle to macro spacing; no preprocessor expansion | NO C++ toolchain needed for regen |

**Recommendation: option (a)**. Here's why:

1. **C++ toolchain is already required** for Wave-2 `validation/build.rs` cc-compile of xcfun-master (D-14). Adding a 150-line C++ extractor adds zero new toolchain dependency.
2. **The macro pattern is regular.** `FUNCTIONAL(XC_*) = { ... };` follows a strict format documented at `functional.hpp:20-24`. A 5-line regex catches every instance:
   ```regex
   ^FUNCTIONAL\(([A-Z_0-9]+)\)\s*=\s*\{(.*?)\};
   ```
3. **The extractor reads xcfun's own headers**, so types/macros (`XC_DENSITY`, `XC_PARTIAL_DERIVATIVES`, etc.) resolve via the same `#include` chain as the production C++ build.
4. **Hash-based drift detection works at the source-file level**, not at the parsed-output level — we SHA-256 each `xcfun-master/src/functionals/*.cpp` and store the hash alongside the generated `.rs` file. Format mismatch is caught by the diff in `--check` mode (option (a)'s output is deterministic).

### Skeleton

```cpp
// xtask/assets/regen_registry/extractor.cpp
//
// Compiled by xtask::regen_registry. Outputs JSONL on stdout, one record
// per FUNCTIONAL(XC_*) macro instantiation:
//   {"id":"XC_SLATERX","short_desc":"...","long_desc":"...","depends":1,
//    "test_vars":"XC_A_B","test_mode":"XC_PARTIAL_DERIVATIVES",
//    "test_order":2,"test_threshold":1e-11,
//    "test_in":[39.0, 38.0],
//    "test_out":[-241.948, -4.207, ...]}
//
// xtask reads stdout, generates crates/xcfun-core/src/registry/generated/
// {FUNCTIONAL_DESCRIPTORS.rs, VARS_TABLE.rs, ALIASES.rs, *.sha256}.

#include <iostream>
#include <regex>
#include <fstream>
#include <sstream>
// Use only stdlib + xcfun-master headers (no external deps).

int main(int argc, char** argv) {
    // Walk xcfun-master/src/functionals/*.cpp, regex-extract FUNCTIONAL macros,
    // emit JSONL records.
    //
    // Comment-stripping is conservative: strip line-comments (`// …`) before
    // regex match; block comments (`/* … */`) handled by extending the regex
    // to (?s) dotall and skipping `/\*.*?\*/` chunks.
}
```

### CI install footprint

- **Linux x86_64 runners (canonical):** g++ 11+ already in `ubuntu:22.04` base image. No install needed.
- **macOS:** `clang++` from XCode CLI tools. Already present on every macos-latest runner.
- **Windows:** MSVC via cargo-built-in `cc 1.2.60` parallel mode (the `parallel` feature is already enabled in xtask Cargo.toml).
- **No libclang** required (vs option (b)) — saves a ~30MB CI install + 60s cold setup.

The `--check` mode workflow:
1. Run extractor against current `xcfun-master/src/`.
2. Generate generated/*.rs to a temp dir.
3. Compute SHA-256 of each generated file.
4. Diff against committed `generated/*.rs.sha256`.
5. Non-zero diff → CI fails with a message pointing at `cargo run -p xtask --bin regen-registry`.

---

## Validation Harness Bring-Up

### Minimum cc-compile set for the LDA tier

The validation harness needs the C++ symbols: `xcfun_new`, `xcfun_set`, `xcfun_eval_setup`, `xcfun_input_length`, `xcfun_output_length`, `xcfun_eval`, `xcfun_delete` — exposed by `xcfun-master/api/xcfun.h`. Implementations live in `xcfun-master/src/XCFunctional.cpp`.

The Wave-2 minimum cc-build set:

| File | Why required | Lines |
|------|--------------|-------|
| `XCFunctional.cpp` | `xcfun_new`/`xcfun_eval`/`xcfun_eval_setup` + `dispatcher` | ~800 |
| `xcint.cpp` | `xcint_vars[]` table; functional symbol setup | ~150 |
| `functionals/aliases.cpp` | `aliases_array[]` (xcfun_new pulls from this) | ~250 |
| `functionals/common_parameters.cpp` | XC_RANGESEP_MU + EXX + CAM defaults (LDAERFX needs RANGESEP_MU) | ~50 |
| **The 11 LDA functional .cpp** files: `slaterx.cpp, vwn3.cpp, vwn5c.cpp, pw92c.cpp, pz81c.cpp, ldaerfx.cpp, ldaerfc.cpp, ldaerfc_jt.cpp, tfk.cpp, tw.cpp, vonw.cpp` | Provide the `FUNCTIONAL(XC_*)` template specializations referenced by `xcint_setup_functional_helper` | ~500 total |

**Critical: `xcint.cpp` references EVERY functional via `xcint_functional_setup_helper` template recursion** through `XC_NR_FUNCTIONALS`. The recursion walks the compile-time enum range, expecting `fundat_db<XC_*>::d` specializations for every ID. **Missing specializations cause link failure.**

### Workaround for the GGA/MGGA exclusion (REQUIRED)

The simplest fix: provide **stub `FUNCTIONAL(XC_*)` instantiations** for the 67 non-LDA IDs in a `validation/c_stubs.cpp` file. Each stub is 2 lines:

```cpp
// validation/c_stubs.cpp — stubs for Phase 2 cc-compile.
// Every non-LDA functional ID needs a fundat_db specialization or xcint.cpp
// won't link.
#include "functional.hpp"

template <typename num> static num stub_unimpl(const densvars<num> &) { return num(0); }

FUNCTIONAL(XC_PW86X) = {"stub", "stub", XC_DENSITY|XC_GRADIENT, ENERGY_FUNCTION(stub_unimpl)};
FUNCTIONAL(XC_PBEX)  = {"stub", "stub", XC_DENSITY|XC_GRADIENT, ENERGY_FUNCTION(stub_unimpl)};
// … 65 more ID stubs …
```

This file is auto-generated by the xtask extractor (it knows which 67 functionals are non-LDA — it reads `list_of_functionals.hpp` and excludes the 11 LDA IDs). No manual maintenance.

### `cc::Build` setup in `validation/build.rs`

```rust
// validation/build.rs
fn main() -> std::io::Result<()> {
    let xcfun_root = "../xcfun-master";
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .flag("-fno-fast-math")
        .flag("-ffp-contract=off")  // ACC-05 — match Rust-side flag
        .define("XCFUN_MAX_ORDER", "6")
        .include(format!("{}/api", xcfun_root))
        .include(format!("{}/src", xcfun_root))
        .include(format!("{}/src/functionals", xcfun_root))
        .include(format!("{}/external/upstream/taylor", xcfun_root))
        .file(format!("{}/src/XCFunctional.cpp", xcfun_root))
        .file(format!("{}/src/xcint.cpp", xcfun_root))
        .file(format!("{}/src/functionals/aliases.cpp", xcfun_root))
        .file(format!("{}/src/functionals/common_parameters.cpp", xcfun_root));
    // 11 LDA files
    for f in &["slaterx", "vwn3", "vwn5c", "pw92c", "pz81c",
               "ldaerfx", "ldaerfc", "ldaerfc_jt",
               "tfk", "tw", "vonw"] {
        build.file(format!("{}/src/functionals/{}.cpp", xcfun_root, f));
    }
    // Stubs for the other 67.
    build.file("c_stubs.cpp");
    build.compile("xcfun_cpp_lda");
    Ok(())
}
```

### Cold compile time estimate

Phase 1 Plan 01-05 fixture driver compiled `xcfun-master/external/upstream/taylor/` (~3 .hpp files) in ~2s with g++. The Phase 2 set is ~14 .cpp files + headers. Estimated cold time:

- **Linux x86_64 g++ -O2 with `cc::parallel`:** ~10-15s (4-core parallel)
- **macOS clang -O2:** ~8-12s
- **Windows MSVC /O2:** ~20-25s

Acceptable for the on-demand validate job per D-15.

### FFI shim shape

```rust
// validation/src/ffi.rs
unsafe extern "C" {
    pub fn xcfun_new() -> *mut std::ffi::c_void;
    pub fn xcfun_delete(fun: *mut std::ffi::c_void);
    pub fn xcfun_set(fun: *mut std::ffi::c_void, name: *const i8, value: f64) -> i32;
    pub fn xcfun_eval_setup(fun: *mut std::ffi::c_void, vars: u32, mode: u32, order: i32) -> i32;
    pub fn xcfun_input_length(fun: *const std::ffi::c_void) -> i32;
    pub fn xcfun_output_length(fun: *const std::ffi::c_void) -> i32;
    pub fn xcfun_eval(fun: *const std::ffi::c_void, density: *const f64, result: *mut f64);
}

pub struct CppXcfun { handle: *mut std::ffi::c_void }
impl CppXcfun {
    pub fn new() -> Self { Self { handle: unsafe { xcfun_new() } } }
    pub fn set(&mut self, name: &str, v: f64) -> i32 { /* CString + xcfun_set */ }
    pub fn eval_setup(&mut self, vars: u32, mode: u32, order: i32) -> i32 { /* … */ }
    pub fn eval(&self, input: &[f64], output: &mut [f64]) { /* … */ }
}
impl Drop for CppXcfun { fn drop(&mut self) { unsafe { xcfun_delete(self.handle) }; } }
```

### Cold-call cost — 110k calls × 11 functionals

For Phase 2 LDA (11 functionals × 10k grid points × 3 orders × ~1 vars per functional ≈ 330k C++ evals + 330k Rust evals): per-call C++ overhead ~5μs × 330k = 1.65s; per-call Rust cubecl-cpu launch ~10μs × 330k = 3.3s. Total tier-2 run ≈ 5s, comfortable for an on-demand job. **No batching required.**

---

## Mode::PartialDerivatives Output Layout

Verified from `xcfun-master/src/XCFunctional.cpp:493-617` (cases 0/1/2 of the dispatcher):

### Output length formula

`output_length(vars, Mode::PartialDerivatives, order) = taylor_len(input_len(vars), order)` where `taylor_len(n, k) = C(n+k, k)` (already implemented in `crates/xcfun-core/src/lib.rs:33-41`).

### Element ordering (ORDER ≤ 2)

For `Mode::PartialDerivatives` and `inlen = input_length(vars)`:

| Order | Output length | Index 0 | Indices 1..=inlen | Indices inlen+1..end |
|-------|---------------|---------|-------------------|----------------------|
| 0 | 1 | Energy `out.get(CNST)` | — | — |
| 1 | 1 + inlen | Energy `out.get(CNST)` | First derivatives — `output[i+1] = d/d(input[i])` for `i ∈ 0..inlen` | — |
| 2 | 1 + inlen + inlen*(inlen+1)/2 | Energy | First derivatives | Second derivatives, packed as upper-triangular by `(i, j)` with `i ≤ j`: `output[k++] = d²/d(input[i])d(input[j])` for `i ∈ 0..inlen` and `j ∈ i..inlen`, starting at `k = inlen + 1` |

### Verification table for the 11 LDA functionals (Phase 2 scope)

| Functional | `test_vars` | `inlen` | order=0 outlen | order=1 outlen | order=2 outlen |
|------------|------------|---------|----------------|----------------|----------------|
| XC_SLATERX | XC_A_B | 2 | 1 | 3 | 6 |
| XC_VWN3C | (no test_vars in macro — uses caller-set vars) | (varies) | (varies) | (varies) | (varies) |
| XC_VWN5C | XC_A_B | 2 | 1 | 3 | 6 |
| XC_PW92C | XC_A_B | 2 | 1 | 3 | 6 |
| XC_PZ81C | XC_A_B | 2 | 1 | 3 | 6 |
| XC_LDAERFX | XC_A_B | 2 | 1 | 3 | 6 |
| XC_LDAERFC | XC_A_B | 2 | 1 | 3 | 6 |
| XC_LDAERFC_JT | (no test_vars) | (varies) | (varies) | (varies) | (varies) |
| XC_TFK | XC_A_B | 2 | 1 | 3 | 6 |
| XC_TW | XC_A_B_GAA_GAB_GBB | 5 | 1 | 6 | 21 |
| XC_VWK | (no test_vars in macro) | (varies) | (varies) | (varies) | (varies) |

- **Order 0:** `[energy]`
- **Order 1, inlen=2:** `[energy, ∂/∂a, ∂/∂b]`
- **Order 2, inlen=2:** `[energy, ∂/∂a, ∂/∂b, ∂²/∂a², ∂²/∂a∂b, ∂²/∂b²]` ← matches the slaterx test_out 6-element layout exactly.
- **Order 2, inlen=5 (TW):** `[energy, ∂/∂a, ∂/∂b, ∂/∂gaa, ∂/∂gab, ∂/∂gbb, ∂²/∂a², ∂²/∂a∂b, ∂²/∂a∂gaa, ∂²/∂a∂gab, ∂²/∂a∂gbb, ∂²/∂b², ∂²/∂b∂gaa, ∂²/∂b∂gab, ∂²/∂b∂gbb, ∂²/∂gaa², ∂²/∂gaa∂gab, ∂²/∂gaa∂gbb, ∂²/∂gab², ∂²/∂gab∂gbb, ∂²/∂gbb²]`

### Implementation strategy for the Rust dispatcher

For order 0, allocate `CTaylor<F, 0>` (length 1) — read `out[0]` as the energy.
For order 1 with `inlen` even, use `CTaylor<F, 2>` (length 4) and run inlen/2 launches per the C++ pattern at `XCFunctional.cpp:514-535`. For odd `inlen`, the trailing input gets its own `CTaylor<F, 1>` launch (lines 537-555).
For order 2, use `CTaylor<F, 2>` (length 4) and run inlen × (inlen+1) / 2 launches in a nested loop per `XCFunctional.cpp:589-612`.

Per-functional kernel signature is the same regardless of order — the dispatcher decides `N`.

---

## Registry Shape + Circular-Dep Resolution

### The problem

`xcfun-eval` depends on `xcfun-core`. The fp-table holds function pointers to `#[cube] fn`s in `xcfun-eval`. If `xcfun-core::FunctionalDescriptor` has type `[fn(&DensVarsDev<F, N>, &mut CTaylor<F, N>); 7]`, that field forces `xcfun-core` to import `DensVarsDev` and `CTaylor` from `xcfun-eval` → circular dep, won't compile.

### Resolution: registry-as-data lives in xcfun-core, dispatch lives in xcfun-eval

`xcfun-core::FunctionalDescriptor` carries ONLY:

```rust
// crates/xcfun-core/src/registry/mod.rs
pub struct FunctionalDescriptor {
    pub id: FunctionalId,
    pub name: &'static str,
    pub short_description: &'static str,
    pub long_description: &'static str,
    pub depends: Dependency,
    /// Upstream `test_vars` from the FUNCTIONAL macro (None = no test data).
    pub test_vars: Option<Vars>,
    /// Upstream `test_mode` from the macro.
    pub test_mode: Option<Mode>,
    /// Upstream `test_order` from the macro.
    pub test_order: Option<u32>,
    /// Upstream `test_threshold` from the macro (LDAs use 1e-11; LDAERF uses 1e-7).
    pub test_threshold: Option<f64>,
    /// Static slice of f64 inputs (slice points into .rodata).
    pub test_in: Option<&'static [f64]>,
    /// Static slice of f64 expected outputs.
    pub test_out: Option<&'static [f64]>,
    /// Per-order Taylor length present in test_out (0 if no test data).
    pub test_outlen: u32,
}

pub static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; 78] = [
    // 11 LDA entries, fully populated (test_in/test_out from regen-registry):
    FunctionalDescriptor {
        id: FunctionalId::SlaterX,
        name: "XC_SLATERX",
        short_description: "Slater LDA exchange",
        long_description: "...",
        depends: Dependency::DENSITY,
        test_vars: Some(Vars::A_B),
        test_mode: Some(Mode::PartialDerivatives),
        test_order: Some(2),
        test_threshold: Some(1e-11),
        test_in: Some(&SLATERX_TEST_IN),
        test_out: Some(&SLATERX_TEST_OUT),
        test_outlen: 6,
    },
    // … 10 more LDA entries …
    // 67 stubs for non-LDA:
    FunctionalDescriptor::stub(FunctionalId::PbeX, "XC_PBEX", Dependency::DENSITY.union(Dependency::GRADIENT)),
    // … 66 more stubs …
];

impl FunctionalDescriptor {
    pub const fn stub(id: FunctionalId, name: &'static str, depends: Dependency) -> Self {
        Self { id, name, short_description: "(stub — not implemented in Phase 2)",
               long_description: "", depends,
               test_vars: None, test_mode: None, test_order: None,
               test_threshold: None, test_in: None, test_out: None, test_outlen: 0 }
    }
}
```

**The `FunctionalDescriptor` carries NO function pointers.** The fp dispatch happens entirely inside `xcfun-eval::dispatch_functional`:

```rust
// crates/xcfun-eval/src/dispatch.rs
use xcfun_core::{FunctionalId, Vars, Mode, XcError};
use crate::density_vars::DensVarsDev;
use crate::functionals::lda;

pub fn dispatch_kernel<F: cubecl::prelude::Float, const N: u32>(
    id: FunctionalId,
    d: &DensVarsDev<F>,
    out: &mut Array<F>,  // Reads the bit-flag-indexed CTaylor coefficients from the kernel.
) -> Result<(), XcError> {
    match id {
        FunctionalId::SlaterX => { lda::slaterx::slaterx_kernel::<F, N>(d, out); Ok(()) }
        FunctionalId::Vwn3C   => { lda::vwn3c::vwn3c_kernel::<F, N>(d, out); Ok(()) }
        // … 9 more LDA arms …
        // Stubs panic — they should not be dispatched in Phase 2:
        _ => Err(XcError::NotConfigured),  // Or a more specific "stub not implemented" variant.
    }
}
```

`xcfun-eval::Functional::eval` calls `dispatch_kernel` per `(FunctionalId, weight)` tuple, summing weighted outputs into the result buffer. No fp-table lookup; the match is the dispatch.

**Why this works:** `cargo metadata` shows `xcfun-eval → xcfun-core` (one-way dep). `xcfun-core` ships static data + the FunctionalId enum; `xcfun-eval` ships the dispatcher and the kernels. No type from `xcfun-eval` appears in any `xcfun-core` signature.

**Stub safety:** the 67 non-LDA dispatch arms return `XcError::NotConfigured` (or a new variant `XcError::StubNotImplemented`). Tier-1 self-tests skip stubs by checking `desc.test_in.is_some()`. Tier-2 harness similarly only loops over the 11 populated entries.

---

## build_densvars Pattern

### Strategy: per-variant `#[cube] fn build_<vars>` chain (matches D-22 + Pitfall P5 prevention)

C++ `densvars.hpp:35-218` uses C-style switch fallthrough. Phase 2 D-22 + CORE-05 forbid fallthrough; require explicit helper-function chains. The builder signature:

```rust
// crates/xcfun-eval/src/density_vars.rs

#[cube]
pub fn build_densvars<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] vars: u32,  // The Vars discriminant as a comptime value.
    #[comptime] n: u32,     // CTaylor depth (0..=2 for Phase 2).
) {
    // Initialise all 22 fields to zero. The variant builders only touch the
    // fields they need; uninitialised reads would be UB. ctaylor_zero is a
    // 1:1 port of the C++ default constructor (densvars.hpp:223-244).
    ctaylor_zero::<F>(&mut out.a, n);
    ctaylor_zero::<F>(&mut out.b, n);
    // … 20 more zero calls — fully unrolled by cubecl ; all 22 zeros …

    // Variant dispatch via comptime if-chain.
    if comptime!(vars == VARS_XC_A) {
        build_xc_a::<F>(input, out, n);
    } else if comptime!(vars == VARS_XC_N) {
        build_xc_n::<F>(input, out, n);
    } else if comptime!(vars == VARS_XC_A_B) {
        build_xc_a_b::<F>(input, out, n);
    } else if comptime!(vars == VARS_XC_N_S) {
        build_xc_n_s::<F>(input, out, n);
    } else if comptime!(vars == VARS_XC_A_B_GAA_GAB_GBB) {
        build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);  // Required for TW + VWK!
    }
    // 26 more variant arms (Phase 3+).
    // (Falls through with all-zeros DensVarsDev if vars not in the comptime-known set —
    // host launcher rejects unsupported vars before launch.)

    // Derived fields (densvars.hpp:213-217), computed once after the variant arm.
    // These are the SAME 4 lines for every variant — they read from out.n, out.a, out.b
    // which the variant arms have populated.
    let mut zeta_tmp = Array::<F>::new(comptime!((1u32 << n) as usize));
    ctaylor_div::<F>(&out.s, &out.n, &mut zeta_tmp, n);
    ctaylor_copy::<F>(&zeta_tmp, &mut out.zeta, n);
    ctaylor_pow::<F>(&out.n, F::new(-1.0/3.0), &mut out.n_m13, n);
    ctaylor_pow::<F>(&out.a, F::new(4.0/3.0), &mut out.a_43, n);
    ctaylor_pow::<F>(&out.b, F::new(4.0/3.0), &mut out.b_43, n);
    // r_s = (3/(4*PI))^(1/3) * n_m13 — composed with constant prefactor:
    let prefactor = F::new(0.6203504908994001);  // (3/(4*PI))^(1/3) = constants::RS_PREFACTOR
    ctaylor_scalar_mul::<F>(&out.n_m13, prefactor, &mut out.r_s, n);
}

// Per-variant arm: XC_A_B (the LDA workhorse for 8 of 11 functionals).
// 1:1 port of densvars.hpp:65-72.
#[cube]
fn build_xc_a_b<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    // a = d[0]; regularize(a);
    ctaylor_from_scalar::<F>(input[0], &mut out.a, n);
    ctaylor_regularize::<F>(&mut out.a, n);
    // b = d[1]; regularize(b);
    ctaylor_from_scalar::<F>(input[1], &mut out.b, n);
    ctaylor_regularize::<F>(&mut out.b, n);
    // n = a + b; s = a - b;
    ctaylor_add::<F>(&out.a, &out.b, &mut out.n, n);
    ctaylor_sub::<F>(&out.a, &out.b, &mut out.s, n);
}

// Per-variant arm: XC_A_B_GAA_GAB_GBB (required for TW, VWK).
// 1:1 port of densvars.hpp:58-72 (the GGA chain that falls through to XC_A_B).
#[cube]
fn build_xc_a_b_gaa_gab_gbb<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    // First: gaa = d[2]; gab = d[3]; gbb = d[4];
    ctaylor_from_scalar::<F>(input[2], &mut out.gaa, n);
    ctaylor_from_scalar::<F>(input[3], &mut out.gab, n);
    ctaylor_from_scalar::<F>(input[4], &mut out.gbb, n);
    // gnn = gaa + 2*gab + gbb (left-to-right; ACC-06 forbids mul_add)
    let mut t1 = Array::<F>::new(comptime!((1u32 << n) as usize));
    let mut t2 = Array::<F>::new(comptime!((1u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&out.gab, F::new(2.0), &mut t1, n);  // t1 = 2*gab
    ctaylor_add::<F>(&out.gaa, &t1, &mut t2, n);                  // t2 = gaa + 2*gab
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gnn, n);             // gnn = (gaa+2*gab)+gbb
    // gss = gaa - 2*gab + gbb
    ctaylor_sub::<F>(&out.gaa, &t1, &mut t2, n);
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gss, n);
    // gns = gaa - gbb
    ctaylor_sub::<F>(&out.gaa, &out.gbb, &mut out.gns, n);
    // Then chain into XC_A_B (the C++ fallthrough target):
    build_xc_a_b::<F>(input, out, n);
}
```

### Per-variant chain table (Phase 2 needs 5 of 31 arms)

| Vars discriminant | Phase 2 functionals | Helper fn called | Chains to |
|---|---|---|---|
| `XC_A` | (none — all LDAs use A_B/N_S in tests) | `build_xc_a` | (none — ends) |
| `XC_N` | (none — Phase 2 LDA test_vars = A_B) | `build_xc_n` | (none — ends) |
| `XC_A_B` | SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK | `build_xc_a_b` | (none — ends) |
| `XC_N_S` | (none in Phase 2 LDA tests) | `build_xc_n_s` | (none — ends) |
| `XC_A_B_GAA_GAB_GBB` | **TW, VWK** | `build_xc_a_b_gaa_gab_gbb` | chains to `build_xc_a_b` |

**Decision: Phase 2 ships 5 variant arms. The remaining 26 arms (for GGA/MGGA in Phases 3/4) panic at host-launcher level (returning `XcError::InvalidVars`).**

### Field zero-init: required for stub safety

C++ uses `T x{static_cast<T>(0)}` member initialisers (densvars.hpp:223-244). Rust `#[derive(CubeType, CubeLaunch)]` does not auto-zero. The Rust port MUST explicitly zero every field via `ctaylor_zero` before the variant arm runs (or the unset fields contain whatever cubecl-cpu's `Array::new` left in memory). Cubecl's `Array::new` does NOT guarantee zero-init.

---

## PW92C Legacy Constants

### Current state

`xcfun-master/src/config.hpp:35-36` ships `XCFUN_REF_PW92C` UNDEFINED by default:

```cpp
// Use #define XCFUN_REF_PW92C to use inaccurate constants in
// PW92C. This matches the reference implementation.
```

Two sources of arithmetic divergence within `pw92eps.hpp` are gated on this define:
1. `pw92eps.hpp:36-41` — `omega(z)` denominator: accurate `(2*pow(2,1/3)-2)` vs reference `0.5198421` (~10⁻¹¹ rel-error).
2. `pw92eps.hpp:53-58` — `pw92eps` prefactor `c`: accurate `8.0/(9.0*(2*pow(2,1/3)-2))` vs reference `1.709921`.

The Wave-2 cc-build does NOT define `XCFUN_REF_PW92C`, so the C++ reference compiles with accurate constants. Phase 2 Rust port must MATCH this — use accurate constants.

### Recommendation: ship accurate constants only; defer feature flag

**Phase 2 LDA-04 ports `pw92c.cpp` + `pw92eps.hpp` with accurate constants directly.** No Cargo feature flag in Phase 2.

**Why defer the feature flag:**
1. The feature flag exists in C++ for fork-comparison purposes (matching an earlier xcfun release's output). Modern xcfun (vendored at `xcfun-master/`) does not ship with REF_PW92C defined.
2. Tier-2 parity gate compiles xcfun-master/ with accurate constants → tests Rust accurate against C++ accurate → 1e-12 achievable.
3. The flag becomes useful only if downstream consumers explicitly request "compare against legacy xcfun-1.x output" — a v2 use case (PERF/INT requirements), not v1.

**Wave-1 LDA-04 port skeleton:**

```rust
// crates/xcfun-eval/src/functionals/lda/pw92c.rs

// Accurate PW92C constants — the current xcfun-master default
// (XCFUN_REF_PW92C undefined). See `xcfun-master/src/functionals/pw92eps.hpp:36-58`.
const PW92C_PARAMS: [[f64; 7]; 3] = [
    [0.03109070, 0.21370, 7.59570, 3.5876, 1.63820, 0.49294, 1.0],
    [0.01554535, 0.20548, 14.1189, 6.1977, 3.36620, 0.62517, 1.0],
    [0.01688690, 0.11125, 10.3570, 3.6231, 0.88026, 0.49671, 1.0],
];
const PW92C_C: f64 = 1.0 / 0.518467692492554; // = 8.0 / (9.0 * (2 * pow(2, 1/3) - 2))
                                              // computed once with libm pow → matches C++ accurate path
// (Verify the value at unit-test time against the C++ driver's runtime print.)

// Planner action: at LDA-04 wave end, escalate via `PLANNING INCONCLUSIVE` IF and ONLY IF
// tier-2 PW92C parity fails > 1e-12 on the bulk grid (would imply a constant-table mismatch
// or a porting error). The escalation path matches D-19's pattern.
```

**No Cargo feature flag needed** for Phase 2 v1. Document in the long-form Wave-1 LDA-04 plan that the feature flag is a v2 forward-compat option, not a Phase 2 requirement.

---

## Wave-0 Commit Order

D-09 mandates one commit per cleanup task. The exact 5-commit Wave-0 sequence (with dependency arrows):

```
Wave-0a: Workspace members re-include
  - Edit: /Cargo.toml
    Change `members = ["crates/xcfun-ad", "xtask"]`
    to     `members = ["crates/xcfun-ad", "crates/xcfun-core", "crates/xcfun-eval", "xtask"]`
    (validation/ added in Wave 2)
  - Update `exclude = […]` to remove xcfun-core, xcfun-eval (keep ffi/functionals/gpu/python).
  - Verify: `cargo check --workspace` passes (existing xcfun-core compiles in isolation
    BUT only IF xcfun-core's lib.rs::pub use xcfun_ad::Num; line is fixed first → see Wave-0c)

  ⚠️ DEPENDENCY: This commit alone WILL NOT BUILD until Wave-0c removes the broken
  Num re-export. Either:
    Option α — Wave-0a + Wave-0c as ONE commit (gives up D-09 atomicity for the broken-build
              window). NOT RECOMMENDED.
    Option β — Wave-0a edits Cargo.toml ONLY; xcfun-core stays in `exclude`; Wave-0b inlines
              the lib.rs fix; Wave-0c moves xcfun-core into `members`.
              RECOMMENDED.

  Final Wave-0a content (Option β): Cargo.toml `[workspace.dependencies]` adjustment ONLY
  (e.g., add `cubecl-cpu` to be available for xcfun-eval). No members change.

Wave-0b: Delete crates/xcfun-core/src/density_vars.rs (825 lines, obsolete under D-01)
  - rm crates/xcfun-core/src/density_vars.rs
  - Edit: crates/xcfun-core/src/lib.rs
    Remove `pub mod density_vars;` and `pub use density_vars::DensityVars;`
  - Verify: `cargo check -p xcfun-ad` still passes (xcfun-core still excluded; this commit
    is a file deletion in an excluded crate).

Wave-0c: Rewrite crates/xcfun-core/src/lib.rs (drop broken `pub use xcfun_ad::Num;`)
  - Edit: crates/xcfun-core/src/lib.rs — remove the `pub use xcfun_ad::Num;` line (line 27),
    remove the `pub mod density_vars;` line (line 12) and `pub use density_vars::DensityVars;`
    (line 20).
  - Keep: constants/enums/error/functional_id/test_data/traits + taylorlen + tests.
  - Verify: `cd crates/xcfun-core && cargo check` passes (single-crate build, since still
    in workspace exclude).

Wave-0d: Rename EvalMode → Mode (+ Unset variant) and VarType → Vars
  - Edit: crates/xcfun-core/src/enums.rs
    - Rename `pub enum EvalMode` → `pub enum Mode`
    - Add `Unset = 0,` as first variant; renumber: PartialDerivatives = 1, Potential = 2, Contracted = 3
    - Add `#[repr(u32)]` on Mode
    - Rename `pub enum VarType` → `pub enum Vars`
    - Add `#[allow(non_camel_case_types)]` (variant names are SCREAMING_SNAKE_CASE)
  - Edit: crates/xcfun-core/src/error.rs — replace EvalMode/VarType references with Mode/Vars
  - Edit: crates/xcfun-core/src/lib.rs — `pub use enums::{Mode, Vars}` (replaced `EvalMode, VarType`)
  - Verify: `cargo check -p xcfun-core` passes (still in exclude); existing tests recompile.

Wave-0e: Audit + fix remaining xcfun-core src files (error.rs CORE-04 compliance, traits.rs
         remove stale Functional/TestData)
  - Edit: crates/xcfun-core/src/error.rs
    - Add `#[non_exhaustive]` on XcError
    - Add `Copy` derive (NOTE: requires removing `String` from variants — change UnknownName
      to take `&'static str` or encode as a numeric ID; CORE-04 mandates Copy + Send + Sync
      so this is non-negotiable)
    - Add 9-variant set: InvalidOrder, InvalidVars (rename from InsufficientVars),
      InvalidMode (rename from UnsupportedMode), UnknownName (with &'static str, not String),
      InputLengthMismatch, OutputLengthMismatch, NotConfigured, InvalidEncoding, Runtime
  - Edit: crates/xcfun-core/src/traits.rs
    - Keep `Dependency` bitflags (CORE-03 compliance). Cross-check matches xcfun-master/src/xcint.hpp:46-50:
      `#define XC_DENSITY 1; XC_GRADIENT 2; XC_LAPLACIAN 4; XC_KINETIC 8; XC_JP 16` (matches!)
    - REMOVE `pub trait Functional` and `pub struct TestData` (functional trait moves to
      xcfun-eval per D-04; test data lives in FUNCTIONAL_DESCRIPTORS post-codegen).
  - Edit: crates/xcfun-core/src/functional_id.rs
    - Cross-check 78 entries match xcfun-master/src/functionals/list_of_functionals.hpp ORDERING
      (the existing rust enum reorders by family — fix to match xcfun.h exactly so `as u32`
      indexes the fp-table correctly per CORE-07). Keep COUNT = 78.
  - Edit: crates/xcfun-core/src/constants.rs — verify TINY_DENSITY = 1e-14, RS_PREFACTOR
    matches the C++ pow(3/(4*PI), 1/3) value. NO CHANGES expected (per Phase 1 design).
  - Edit: crates/xcfun-core/src/test_data.rs — DELETE (CORE-07 mandates test data lives in
    FUNCTIONAL_DESCRIPTORS, not a separate trait).
  - Edit: crates/xcfun-core/src/lib.rs — remove test_data mod reference; remove Functional/TestData
    re-exports; pub use functional_id::FunctionalId only.
  - Verify: `cargo check -p xcfun-core` passes; existing xcfun-core tests pass.

Wave-0f: Re-include xcfun-core in workspace members
  - Edit: /Cargo.toml
    `members = ["crates/xcfun-ad", "crates/xcfun-core", "xtask"]`
    Remove "crates/xcfun-core" from `exclude`.
  - Verify: `cargo build --workspace` passes (xcfun-eval still excluded; will be added when
    its src/lib.rs is non-empty).
  - This is the gate commit that confirms Wave-0a..e are coherent.

Wave-0g (deferred to Wave-1 if scope permits): Add xcfun-eval to workspace members + skeleton lib.rs
  - Either: count this as part of Wave-1's first task (when xcfun-eval gets actual content).
    RECOMMENDED — keeps Wave-0 minimal.
  - Or: ship a no-op xcfun-eval lib.rs in Wave-0 that just re-exports xcfun_core. Adds
    nothing useful; defer.
```

**Total Wave-0 = 6 commits (a..f). Each is < 50 LOC. Each leaves the workspace in a buildable state.** This satisfies D-09 atomicity + git-bisect usability.

---

## Grid Generator Spec

### Stratified 70/30 distribution recipe (D-18)

```rust
// validation/src/fixtures.rs
use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};
use rand_core::RngCore;

pub const GRID_SEED: u64 = 0x1234abcd;
pub const TOTAL_POINTS: usize = 10_000;

#[derive(Clone, Copy)]
pub struct GridPoint {
    pub n: f64,        // total density
    pub s: f64,        // spin density (s = n_a - n_b; |s| ≤ n)
    pub gnn: f64,      // |∇n|² (Phase 3+ uses)
    pub gns: f64,
    pub gss: f64,
    pub gaa: f64,      // for XC_A_B_GAA_GAB_GBB derivation (TW, VWK)
    pub gab: f64,
    pub gbb: f64,
    // Future (Phase 3/4): tau, lapa, lapb, jpaa, jpbb, gradient components.
}

impl GridPoint {
    /// Convert (n, s) to (a, b) via a = (n+s)/2, b = (n-s)/2.
    pub fn ab_from_ns(&self) -> (f64, f64) { ((self.n + self.s) * 0.5, (self.n - self.s) * 0.5) }
}

pub fn generate_grid() -> Vec<GridPoint> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(GRID_SEED);
    let mut out = Vec::with_capacity(TOTAL_POINTS);
    // Stratum 1: 7000 uniform bulk (n ∈ [1e-5, 10.0], |s/n| ∈ [0, 0.95])
    out.extend(generate_bulk(&mut rng, 7000));
    // Stratum 2: 1000 regularize stress (ρ ∈ [1e-14, 1e-5]) — tests Pitfall P10 (NaN canary)
    out.extend(generate_regularize_stress(&mut rng, 1000));
    // Stratum 3: 1000 polarised limit (|ζ| ∈ [0.95, 1.0]) — tests pow_expand at near-singular ρ
    out.extend(generate_polarised(&mut rng, 1000));
    // Stratum 4: 1000 gradient stress (|∇ρ|² ∈ [1, 1e6]) — N/A for LDA but reused Phase 3+
    out.extend(generate_gradient_stress(&mut rng, 1000));
    out
}

// Stratum 1 distribution (bulk density):
//   n ~ log-uniform on [1e-5, 10.0]      — covers 6 decades; matches ρ-distribution
//                                          of typical molecular bulks (e.g., CH4 valence:
//                                          ρ ~ 0.01..0.5; H2O cusp: ρ ~ 1..5)
//   |s/n| ~ uniform on [0, 0.95]          — uniform spin polarisation
//   gnn, gss, gns ← derived from gradient stratum (gradient stress computed separately)
//
// Stratum 2 (regularize stress):
//   n ~ log-uniform on [1e-14, 1e-5]
//   s = 0 (spin-unpolarised) — emphasis on the regularize boundary, not on polarisation
//
// Stratum 3 (polarised):
//   n ~ log-uniform on [1e-3, 10.0]
//   |ζ| = |s/n| ~ uniform on [0.95, 1.0] (occasional zeta = 0.99999 to test the
//                                          (1 - zeta^4) cancellation form in vwn5_eps)
//
// Stratum 4 (gradient — Phase 3+):
//   n ~ log-uniform on [1e-3, 10.0]
//   |∇ρ|² ~ log-uniform on [1, 1e6]       — covers six decades of gradient magnitude
//   gaa = |∇ρ|² × cos(θ)²; gbb = |∇ρ|² × sin(θ)²; gab = θ-correlated mixing (random)

fn generate_bulk(rng: &mut Xoshiro256PlusPlus, count: usize) -> Vec<GridPoint> {
    (0..count).map(|_| {
        let u = next_uniform_01(rng);
        let n = 1e-5 * (10.0_f64.powf(6.0 * u));   // log-uniform on [1e-5, 1e1]
        let z_abs = 0.95 * next_uniform_01(rng);
        let z_sign = if next_uniform_01(rng) < 0.5 { -1.0 } else { 1.0 };
        let s = z_sign * z_abs * n;
        GridPoint { n, s, gnn: 0.0, gns: 0.0, gss: 0.0, gaa: 0.0, gab: 0.0, gbb: 0.0 }
    }).collect()
}
```

### `rand_xoshiro =0.8.0` reproducibility verification

`rand_xoshiro 0.8` documents Xoshiro256++ as a deterministic PRNG with byte-identical output for the same `seed_from_u64`. The crate uses pure Rust integer arithmetic (no platform-specific intrinsics) — output is identical across x86_64 / aarch64 / Linux / macOS / Windows.

**Phase 1 already exercises `rand_xoshiro =0.8.0`** in proptest_algebra.rs (commit `3514217`); 110k iterations × 11 properties green on cubecl-cpu confirms platform reproducibility.

For Pitfall P8-style cubecl drift concern: the grid generator runs OUTSIDE cubecl kernels (host-side Rust), so it's not affected by MLIR JIT lowering. The grid is generated once per harness invocation and fed into both the Rust cubecl-cpu kernel AND the C++ runtime via FFI — same f64 input bit-pattern, two evaluators, compare outputs.

### Grid record persistence

Per D-15, NO committed fixtures. The grid is regenerated per harness run. This saves ~1 MB per stratum × 4 strata × ~100 bytes/record ≈ 400 KB of repo bloat per phase.

---

## Validation Architecture

> Workflow `nyquist_validation` is enabled (config.json shows `"nyquist_validation": true`). Section is required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `proptest =1.11.0` + `approx =0.5.1` (already pinned in workspace) |
| Config file | `crates/xcfun-eval/Cargo.toml` (created in Wave-0g/Wave-1) + `validation/Cargo.toml` (Wave-2) |
| Quick run command | `cargo test -p xcfun-eval --lib` (tier-1 self-tests, < 5s per ACC-04) |
| Full suite command | `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter 'lda'` (tier-2 parity, ~5-15s including C++ build amortisation) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CORE-01 | `Vars` enum 31 variants matching xcfun.h discriminants | unit | `cargo test -p xcfun-core --lib enums::tests::var_type_cpp_ordering` | ✅ (existing in xcfun-core/src/enums.rs:266-272 — needs rename) |
| CORE-02 | `Mode` enum with `Unset = 0` repr u32 | unit | `cargo test -p xcfun-core --lib enums::tests::eval_mode_has_4_variants` | ❌ Wave-0d |
| CORE-03 | `Dependency` bitflags 5 entries | unit | `cargo test -p xcfun-core --lib traits::tests::dependency_bits` | ✅ (xcfun-core/src/traits.rs:60-65) |
| CORE-04 | `XcError` 9-variant `Copy + Send + Sync` `#[non_exhaustive]` | unit + compile-test | `cargo test -p xcfun-core --lib error::tests::*` + `static_assertions::assert_impl_all!(XcError: Copy, Send, Sync)` | ❌ Wave-0e |
| CORE-05 | `DensVarsDev` populates 22 fields per 5 Vars arms | integration | `cargo test -p xcfun-eval --test densvars_field_parity` (compares each populated field against C++ densvars dump per Vars) | ❌ Wave-1B |
| CORE-06 | `regularize` modifies only `c[CNST]` | unit | `cargo test -p xcfun-eval --lib density_vars::tests::regularize_preserves_derivatives` | ❌ Wave-1B |
| CORE-07 | `FUNCTIONAL_DESCRIPTORS` 78 entries in `.rodata` | unit | `cargo test -p xcfun-core --test descriptors_count` (asserts `FUNCTIONAL_DESCRIPTORS.len() == 78`) | ❌ Wave-1A (post-codegen) |
| CORE-09 | `VARS_TABLE` 31 entries, len + provides match xcint.cpp | unit | `cargo test -p xcfun-core --test vars_table_parity` | ❌ Wave-1A (post-codegen) |
| LDA-01..05, 07, 09 (8 strict-1e-12 LDAs) | tier-1 self-test passes at upstream test_threshold | integration | `cargo test -p xcfun-eval --test self_tests` (one parametric test per descriptor with test_in.is_some()) | ❌ Wave-1B |
| LDA-06, 07, 08 (3 LDAERF) | tier-1 passes at upstream 1e-7 threshold; tier-2 may require per-functional override | integration | `cargo test -p xcfun-eval --test self_tests` + tier-2 conditional gate | ❌ Wave-1B + Wave-2 |
| LDA-10 (TW + VWK) | tier-1 passes via XC_A_B_GAA_GAB_GBB build path | integration | `cargo test -p xcfun-eval --test self_tests` (TW has fixture; VWK has no test_in — tier-2 covers it) | ❌ Wave-1C |
| MODE-04 | `input_length` matches VARS_TABLE | unit | `cargo test -p xcfun-eval --lib functional::tests::input_length` | ❌ Wave-1B |
| ACC-01..04 | 10k-point seeded grid × 11 LDA × 3 orders parity at 1e-12 | full system | `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` produces zero failures | ❌ Wave-2 |
| ACC-05 (absorbed) | RUSTFLAGS empty + -fp-contract=off in release | xtask | `cargo run -p xtask --bin check-no-fma` (existing) + extend scope to xcfun-eval | ✅ Phase 1 + extension Wave-0 |
| ACC-06 (absorbed) | `mul_add` ban in xcfun-eval/src/functionals/ | xtask | `cargo run -p xtask --bin check-no-mul-add` | ❌ Wave-1A |
| QG-01 (absorbed) | check-no-anyhow xtask | xtask | `cargo run -p xtask --bin check-no-anyhow` | ❌ Wave-1A |
| QG-02 (absorbed) | check-boundaries xtask | xtask | `cargo run -p xtask --bin check-boundaries` | ❌ Wave-1A |
| QG-06 (absorbed) | cargo metadata cubecl pin | xtask | `cargo run -p xtask --bin check-cubecl-pin` | ❌ Wave-1A |
| QG-07 (absorbed) | regen-registry --check drift gate | xtask | `cargo run -p xtask --bin regen-registry -- --check` | ❌ Wave-1A |

### Sampling Rate

- **Per task commit:** `cargo test -p <crate-name> --lib` (≈ 1-5s per crate; the modified crate's unit tests).
- **Per wave merge:** `cargo test --workspace` (≈ 30s including all unit + integration tests across 4 crates).
- **Phase gate (Wave-2 SC #5):** `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` reports zero failures across all 11 LDA × 10k grid × 3 orders.

### What classes of failures the harness CAN catch

1. **Algorithmic divergence** between cubecl-cpu and C++ — caught at every grid point per output element. ANY 1e-12 violation in any (functional, vars, mode, order, point, element) tuple fails the merge.
2. **MLIR FMA injection** — `xtask check-no-fma` asm-grep gate (Phase 1 inheritance, extended Wave-0).
3. **Registry drift** — `xtask regen-registry --check` SHA-256 stamp comparison.
4. **PW92C constant mismatch** — IF Rust port uses wrong constant table, 7000-point bulk grid catches at ~1e-6 to 1e-4 (well above 1e-12).
5. **Regularize off-path** — 1000-point regularize-stress stratum exercises ρ ∈ [1e-14, 1e-5] explicitly; Pitfall P10 NaN canary.
6. **Densvars fallthrough bug (Pitfall P5)** — per-variant CORE-05 unit test (compares each of 22 fields populated against C++ densvars trace).
7. **`mul_add` introduction** — `xtask check-no-mul-add` grep gate.
8. **anyhow leakage into library graph** — `xtask check-no-anyhow` grep gate.

### What the harness CANNOT catch

1. **Rare grid corners** the seeded grid doesn't sample. Mitigation: seed `0x1234abcd` is fixed; if a corner case is found in the wild, add it as a deterministic seed alongside the random grid.
2. **CUDA / Wgpu drift** — Phase 6's responsibility (KER-06, GPU-07, GPU-08). Phase 2 is cubecl-cpu only.
3. **Order > 2 on PartialDerivatives** — Phase 3 (MODE-01 extends to orders 3..=4).
4. **Mode::Potential / Mode::Contracted** — Phase 3/4. Phase 2 eval_setup rejects them with `XcError::InvalidMode`.
5. **Cross-platform libm divergence** (Pitfall P2, MEDIUM impact) — Linux x86_64 glibc is the canonical CI runner. macOS/Windows runs accepted at the same 1e-12 bar but documented as known-flake risk.

### `report.html` schema

```
┌─────────────────────────────────────────────────────────────┐
│ XCFun Tier-2 Parity Report — 2026-04-20T22:30:15Z          │
│ Backend: CpuRuntime (cubecl-cpu =0.10.0-pre.3)              │
│ Reference: xcfun-master/ SHA-256 abc123… (vendored)         │
│ Total tuples evaluated: 11 LDA × 10000 points × 3 orders    │
│ ────────────────────────────────────────────────────────── │
│ Functional × Mode × Order matrix (max rel-error per cell)   │
│                                                             │
│              | order=0 | order=1 | order=2 | TOLERANCE      │
│   slaterx    | 1.2e-15 | 8.4e-15 | 2.1e-14 | 1e-12 ✓ GREEN  │
│   vwn5c      | 6.7e-15 | 9.0e-15 | 1.1e-14 | 1e-12 ✓ GREEN  │
│   pw92c      | 4.5e-14 | 1.7e-13 | 8.9e-13 | 1e-12 ✓ GREEN  │
│   ldaerfx    | 2.3e-08 | 4.1e-08 | 7.8e-08 | 1e-7  ✓ GREEN  │
│   ldaerfc    | 1.8e-08 | 3.2e-08 | 5.4e-08 | 1e-7  ✓ GREEN  │
│   ldaerfc_jt | 2.1e-08 | 4.7e-08 | 8.3e-08 | 1e-7  ✓ GREEN  │
│   …                                                         │
└─────────────────────────────────────────────────────────────┘

Color coding: rel-err < 1e-13 GREEN, 1e-13..tolerance YELLOW, > tolerance RED
```

### `report.jsonl` schema

```jsonl
{"functional":"slaterx","vars":"XC_A_B","mode":"PartialDerivatives","order":2,"point_idx":0,"element_idx":0,"input":[39.0,38.0],"rust":-241.948,"cpp":-241.948,"abs_err":3.5e-13,"rel_err":1.4e-15,"pass":true}
{"functional":"slaterx","vars":"XC_A_B","mode":"PartialDerivatives","order":2,"point_idx":0,"element_idx":1,"input":[39.0,38.0],"rust":-4.207,"cpp":-4.207,"abs_err":1.2e-14,"rel_err":2.9e-15,"pass":true}
…
```

**Failed records** are kept in the JSONL for post-mortem; max-rel-error per (functional, mode, order) cell is rolled up into the HTML matrix.

### CI integration

- **Per-commit (every PR):** `cargo test --workspace` runs tier-1 self-tests (5s) + xtask gates (10s).
- **Pre-merge (manual/required):** `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` (15-30s including C++ cc-build amortised).
- **Nightly:** Full validate run + `cargo run -p xtask --bin regen-registry -- --check` + `cargo run -p xtask --bin check-no-fma`.

---

## Phase-2 Pitfalls

Beyond inherited PITFALLS.md (P1, P2, P3, P4, P5, P9, P10, P11, P12, P13):

### Pitfall PHASE2-A: cubecl monomorphization explosion

**What goes wrong:** `DensVarsDev<F>` × `(F, N)` instantiations × 11 functional kernels × 5 build_densvars variant arms could produce thousands of monomorphizations. Each cubecl `#[cube] fn` generates an MLIR module + a static dispatch entry; cubecl-cpu compiles on first launch.

**Why it happens:** Phase 2 instantiates `(f64, N=0)`, `(f64, N=1)`, `(f64, N=2)` — only 3 N values. Per the dispatcher (`XCFunctional.cpp:500-617`):
- Order 0 uses `CTaylor<F, 0>`
- Order 1 uses `CTaylor<F, 2>` for the bulk of inputs and `CTaylor<F, 1>` for the trailing if odd inlen
- Order 2 uses `CTaylor<F, 2>`

So Phase 2 instantiates `N ∈ {0, 1, 2}` only. Total: 11 functionals × 3 N × 1 F (=f64) = 33 kernels. Plus `build_densvars` × 5 variant arms × 3 N = 15 builder kernels = ~48 total. Bounded.

**How to avoid:** Phase 1 already handled 11 `#[cube] fn` × 5 N values (Plan 01-07 sign-off). Phase 2's 48 is comparable. Document at Wave-1 Plan that monomorphization stays bounded; if a future GGA wave (Phase 3) blows the budget, switch to comptime-dispatched single-kernel-per-functional.

**Warning signs:** `cargo build` for xcfun-eval takes > 30s; cubecl error mentioning "too many specializations".

### Pitfall PHASE2-B: FFI shim overhead on tier-2

**What goes wrong:** Tier-2 launches 110k C++ FFI calls + 110k cubecl-cpu kernel launches. If each FFI call has > 100μs cold-call overhead, tier-2 takes minutes instead of seconds.

**Why it happens:** Cold cubecl-cpu kernel launch is ~10-20μs; first-launch JIT cost is ~50ms (amortised across the 10k-point batch); FFI extern "C" call overhead is ~1μs.

**How to avoid:** Already amortised — 11 functionals × 3 orders × 10k points = 330k Rust launches × 10μs = 3.3s; equivalent C++ FFI = 0.33s. Total tier-2 wall-time ~5-8s. NO BATCHING NEEDED in Phase 2. Phase 3 GGAs may push to 60s if needed; revisit batch optimisation at that point.

**Warning signs:** Tier-2 takes > 60s on a clean build. (Initial first-run JIT amortisation can add ~10s; subsequent runs in same process should not exceed 8s.)

### Pitfall PHASE2-C: regen-registry running in CI without C++ toolchain

**What goes wrong:** `cargo run -p xtask --bin regen-registry --check` requires a C++ compiler. CI runners that skip C++ install fail the gate.

**Why it happens:** `xtask` is a Rust-only crate; cc 1.2.60 is the build-dep. The C++ toolchain is at the OS level.

**How to avoid:**
1. Document in CI YAML that the `regen-registry` job requires `g++` / `clang++`.
2. The PR-time CI uses pre-baked containers (ubuntu:22.04 has g++ 11 pre-installed).
3. The `--check` mode is fast (regenerates to temp dir + SHA-256 diff); only runs on PRs that touch `xcfun-master/src/functionals/*.cpp` (path filter in CI YAML).

**Warning signs:** CI runner installation failure on `regen-registry` step → check YAML for missing apt-get install g++ step.

### Pitfall PHASE2-D: `tw` and `vwk` mistakenly grouped with pure LDAs in Wave-1B

**What goes wrong:** A planner who reads "11 LDAs" without checking each .cpp file groups TW + VWK with the 9 pure-density LDAs in Wave-1B. The pure-LDA `XC_A_B` builder doesn't populate `gaa, gab, gbb`. TW kernel reads `d.gaa + d.gbb` → reads zero → returns 0 for the von Weizsäcker energy.

**Why it happens:** REQUIREMENTS LDA-09 / LDA-10 say "kinetic energy functionals" without distinguishing by Vars; the LDA TIER label suggests pure density. Reality: `tw.cpp` and `vonw.cpp` use `XC_A_B_GAA_GAB_GBB`.

**How to avoid:** Wave-1B ports the 8 pure-density LDAs (SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK) using the `XC_A_B` builder. Wave-1C ports TW + VWK separately, requiring the `XC_A_B_GAA_GAB_GBB` builder arm (which itself chains to `XC_A_B`). LDAERFC_JT ports as part of Wave-1B but with no tier-1 fixture.

**Warning signs:** TW or VWK tier-1 self-test returns 0 instead of the expected energy. (TW has no upstream test_in/test_out — caught only at tier-2 against C++.)

### Pitfall PHASE2-E: Existing `XcError::UnknownName(String)` blocks `Copy` derive

**What goes wrong:** CORE-04 mandates `XcError: Copy + Send + Sync`. Existing `error.rs:32` has `UnknownName(String)`. `String` is not `Copy`. The Wave-0e edit must change this OR the derive fails.

**Why it happens:** Phase 1 design predates the CORE-04 strict `Copy` requirement; the existing variant accepts `String` for ergonomic name lookup error messages.

**How to avoid:**
- **Option α:** `UnknownName(&'static str)` — requires the lookup name to come from a static table, breaking dynamic name lookup at the FFI boundary. NOT VIABLE.
- **Option β:** `UnknownName` as a unit variant with the name printed via `Display::fmt` based on context — loses the offending name in the error message. ACCEPTABLE.
- **Option γ:** Drop `Copy` requirement for now — REVERT decision in CORE-04 wording. NOT VIABLE (CORE-04 is explicit).
- **Option δ:** `UnknownName` carries a `[u8; 32]` fixed-size buffer with truncation. Unusual but Copy-compatible. POSSIBLE.

**Recommended:** Option β (unit variant). The name is captured at the FFI boundary (xcfun_set, xcfun_eval_setup); xcfun-core doesn't need to carry it. The test message ("set returned XcError::UnknownName for input 'foo'") is composable at the call site without needing to embed in the error type.

**Warning signs:** `cargo build -p xcfun-core` fails with "the trait `Copy` is not implemented for `String`" after Wave-0e.

---

## Recommended Plan Decomposition

The planner is final authority. This is a sketch.

### Wave 0 — Atomic cleanup (sequential, 6 commits)
- 0a: Cargo.toml workspace deps add cubecl-cpu (xcfun-eval prereq); no member changes
- 0b: Delete crates/xcfun-core/src/density_vars.rs
- 0c: Rewrite crates/xcfun-core/src/lib.rs (drop broken Num re-export)
- 0d: Rename EvalMode → Mode (+ Unset), VarType → Vars (#[repr(u32)], #[allow(non_camel_case_types)])
- 0e: Audit + fix error.rs (CORE-04: 9 variants + Copy + #[non_exhaustive]), traits.rs (remove Functional/TestData), functional_id.rs (verify xcfun.h ordering), constants.rs (verify), test_data.rs (delete)
- 0f: Re-include xcfun-core in workspace members; verify cargo build --workspace passes

### Wave 1A — Registry codegen + xtask QG gates (parallel with 1B/1C)
- 1A-1: Implement `xtask regen-registry` with cc-compiled C++ extractor + JSONL output + `--check` SHA-256 gate (CORE-10 / QG-07)
- 1A-2: Generate `crates/xcfun-core/src/registry/generated/{FUNCTIONAL_DESCRIPTORS.rs, VARS_TABLE.rs, ALIASES.rs}` (11 LDA + 67 stubs + 31 vars + 0 aliases)
- 1A-3: Re-export FUNCTIONAL_DESCRIPTORS, VARS_TABLE from xcfun-core lib.rs (CORE-07, CORE-09)
- 1A-4: Implement `xtask check-no-mul-add` (ACC-06 — pattern from check-no-fma)
- 1A-5: Implement `xtask check-no-anyhow` (QG-01)
- 1A-6: Implement `xtask check-boundaries` basic version (QG-02)
- 1A-7: Implement `xtask check-cubecl-pin` (QG-06 — `cargo metadata` JSON parse)
- 1A-8: Verify `xtask check-no-fma` extends to xcfun-eval functional bodies (ACC-05 inheritance)

### Wave 1B — xcfun-eval infrastructure + 8 pure-LDA functionals (parallel with 1A)
- 1B-1: Add xcfun-eval to workspace members; bring up `crates/xcfun-eval/src/lib.rs` skeleton + `for_tests::cpu_client()` (mirror Phase 1 pattern)
- 1B-2: Implement `DensVarsDev<F>` `#[cube]` type with 22 named fields (CORE-05 part 1)
- 1B-3: Implement `build_densvars` dispatcher + `build_xc_a_b` arm (CORE-05 part 2)
- 1B-4: Implement `regularize` `#[cube] fn` modifying only `c[CNST]` (CORE-06)
- 1B-5: Implement `Functional` struct + `Functional::eval(input, output) → Result<(), XcError>` minimal slice with cubecl-cpu launch (D-21, MODE-04)
- 1B-6: Implement `dispatch_kernel` match-based dispatcher (registry circular-dep resolution)
- 1B-7: Implement `xcfun_eval::functionals::lda::slaterx` `#[cube] fn slaterx_kernel` (LDA-01)
- 1B-8: Implement vwn3c, vwn5c (LDA-02, LDA-03)
- 1B-9: Implement pw92c (LDA-04) — confirm at port-time PW92C constants match accurate C++ defaults; escalate if measured rel-error > 1e-12
- 1B-10: Implement pz81c (LDA-05)
- 1B-11: Implement ldaerfx, ldaerfc (LDA-06, LDA-07) — provisional 1e-7 tier-2 override per § "D-19 LDAERF Tolerance Analysis"; user approval required at Wave-2
- 1B-12: Implement ldaerfc_jt (LDA-08) — no upstream test_in; tier-2 covers
- 1B-13: Implement tfk (LDA-09 part — pure density)
- 1B-14: Tier-1 self-test infrastructure: parametric test loop over `FUNCTIONAL_DESCRIPTORS.iter().filter(|d| d.test_in.is_some())` (ACC-04); per-functional asserts at upstream test_threshold

### Wave 1C — Kinetic-GGA (sequential after 1B-3, parallel with rest of 1B)
- 1C-1: Extend `build_densvars` with `build_xc_a_b_gaa_gab_gbb` arm (CORE-05 part 3 — required for TW, VWK)
- 1C-2: Implement tw kernel (LDA-09 part — kinetic GGA via XC_A_B_GAA_GAB_GBB)
- 1C-3: Implement vwk kernel (LDA-10) using vonw.cpp's `vW = gaa/(8*a) + gbb/(8*b)` formula

### Wave 2 — Validation harness (sequential, depends on Wave 1A+1B+1C complete)
- 2-1: Bring up `validation/` crate (binary) + `validation/build.rs` cc-compile of XCFunctional.cpp + xcint.cpp + aliases.cpp + common_parameters.cpp + 11 LDA + auto-generated `c_stubs.cpp`
- 2-2: Implement FFI shim (`CppXcfun` wrapper around xcfun_new/xcfun_set/xcfun_eval_setup/xcfun_input_length/xcfun_output_length/xcfun_eval/xcfun_delete)
- 2-3: Implement grid generator (`validation/src/fixtures.rs`) per § "Grid Generator Spec" — 4 strata × seed 0x1234abcd
- 2-4: Implement tier-2 driver: loop over (functional, vars, mode, order, point_idx, element_idx); compute Rust output via xcfun-eval, C++ output via FFI; compute rel-error
- 2-5: Implement report writers (`report.jsonl` via serde_json; `report.html` hand-written matrix)
- 2-6: Implement `xtask validate` thin wrapper that delegates to `validation` binary + parses CLI flags (`--backend cpu --order N --filter regex`)
- 2-7: Run SC #5 gate: `cargo run -p xtask --bin validate -- --backend cpu --order 2 --filter lda` reports zero failures (with documented LDAERF 1e-7 override). User approval gate: review max-rel-error matrix; sign off on per-functional overrides.
- 2-8: Wire CI integration (per-commit tier-1 + nightly tier-2 + pre-merge tier-2 with C++ build cache)

### Wave 3 — Documentation + Sign-off (sequential, fast)
- 3-1: Update `docs/design/02-data-structures.md` §5 (DensVarsDev as `#[cube]` type)
- 3-2: Update `docs/design/05-module-responsibilities.md` §2 (xcfun-eval owns dispatcher; xcfun-core owns types + registry)
- 3-3: Update `docs/design/06-cubecl-strategy.md` §3 (functional bodies are `#[cube] fn`)
- 3-4: Update `docs/design/09-testing-strategy.md` (tier-1 + tier-2 patterns landed)
- 3-5: Mark CORE-01..09, LDA-01..10, MODE-04, ACC-01..04, CORE-10, ACC-05, ACC-06, QG-01, QG-02, QG-06, QG-07 as `[x]` in REQUIREMENTS.md
- 3-6: Mark Phase 2 as Complete in ROADMAP.md and STATE.md
- 3-7: Update STATE.md "Decisions added this phase" block

**Estimated wave durations** (based on Phase 1 actuals: 7 plans / 1 day):
- Wave 0: 1-2 hours (sequential cleanup)
- Wave 1A: 4-6 hours (xtask infrastructure + codegen)
- Wave 1B: 6-10 hours (8 pure-LDA + tier-1 infrastructure)
- Wave 1C: 2-3 hours (TW + VWK)
- Wave 2: 4-6 hours (validation harness + cc-build amortisation)
- Wave 3: 1-2 hours (docs + sign-off)
- **Total Phase 2: 18-29 hours of focused dev time**, distributable across 3-5 day calendar window with parallelisation.

---

## Open Questions for Planner

1. **LDAERF tolerance override authorisation (BLOCKING for Wave-2 SC #5).** Per § "D-19 LDAERF Tolerance Analysis", three LDAs (LDAERFX, LDAERFC, LDAERFC_JT) cannot meet 1e-12 against C++ on cubecl-cpu due to cubecl's `Float::erf` polyfill (~1e-8 rel-err). Options: (a) ship per-functional 1e-7 override matching upstream xcfun's own self-test threshold (recommended); (b) replace cubecl polyfill with libm-call hybrid (Phase 6 scope creep); (c) halt LDA-06/07/08 pending upstream cubecl `erf` improvement. **Planner must pick + escalate to user before Wave-2 begins. Recommend (a).**

2. **`XcError::UnknownName` Copy compliance (BLOCKING for Wave-0e).** Per § "Pitfall PHASE2-E", existing `UnknownName(String)` is incompatible with CORE-04's `Copy` requirement. Options: (a) drop the variant's payload (lose name in error message); (b) use fixed-size `[u8; 32]` buffer (unusual but Copy-compatible); (c) revisit CORE-04 to drop `Copy` (rejected per CORE-04 explicit wording). **Recommend (a).**

3. **`test_data.rs` retention vs deletion (Wave-0e tactical).** With CORE-07 mandating test data lives in `FUNCTIONAL_DESCRIPTORS.test_in/test_out`, `crates/xcfun-core/src/test_data.rs` (currently 2 lines, near-empty) becomes dead code. Recommend deletion at Wave-0e. (Trivial — flagged for visibility.)

4. **`functional_id.rs` discriminant ordering rewrite (Wave-0e tactical).** The existing `FunctionalId` enum reorders by family for readability (LDA / GGA Exchange / GGA Correlation / Kinetic / etc.). xcfun.h's `list_of_functionals.hpp` orders by historical insertion (SlaterX, Pw86X, Vwn3C, Vwn5C, PbeC, PbeX, BeckeX, …). For CORE-07 + CORE-10 to work, `FunctionalId as u32` MUST equal the C `xcfun_functional_id` discriminant (so the regen-registry's generated entries land at the correct array indices). **Wave-0e includes a non-trivial rewrite to match xcfun.h ordering exactly.**

5. **regen-registry bootstrap chicken-and-egg (Wave-1A planning).** `xtask regen-registry` requires xcfun-master to compile. xcfun-master compilation requires cc + a C++ toolchain. Wave-1A regen-registry runs once at Wave-1A-2 to populate the committed `generated/*.rs`. After that, the committed files are checked in; regen-registry --check verifies the SHA-256 stamps. **Issue:** if a developer doesn't have a C++ toolchain installed locally, they cannot re-run regen-registry. **Recommendation:** document in `CONTRIBUTING.md` (yet to be created in Phase 0) that re-running regen-registry requires `g++` or `clang++`; the `--check` gate runs on CI runners that DO have a toolchain.

6. **Cubecl `Array::new` zero-init semantics (Wave-1B tactical, may need a minor probe).** Pattern A in § "D-02 cubecl Nesting Decision" relies on explicit `ctaylor_zero` calls before each variant arm (densvars.hpp default-initialiser semantics). If cubecl's `#[derive(CubeType, CubeLaunch)]` already zero-inits Array fields, the explicit zeros become a perf no-op. If not, they're load-bearing. **Recommend: 5-line spike test to confirm in Wave-0g (or first task of Wave-1B-2). Either way, the explicit zeros are defensive and correct.**

7. **`ALIASES` empty slice declaration shape (Wave-1A tactical).** CORE-08 requires 46 entries (Phase 4); D-12 says Phase 2 ships an empty slice. Planner: declare `pub static ALIASES: &[Alias] = &[];` in xcfun-core/src/registry/generated/aliases.rs so Phase 4 simply re-runs regen-registry to populate. (Trivial; flagged for completeness.)

---

## Sources

### Primary (HIGH confidence)
- **xcfun-master/src/densvars.hpp** lines 22-244 (`regularize`, `densvars<T>` template, 12 case arms with C-fallthrough, 22 fields, 4 derived fields)
- **xcfun-master/src/functional.hpp** lines 20-28 (`FUNCTIONAL` macro definition, `ENERGY_FUNCTION` 7× expansion via `FOR_EACH(XCFUN_MAX_ORDER, EN, FUN)`)
- **xcfun-master/src/xcint.hpp** lines 28-62 (`XC_MAX_INVARS=20`, FOR_EACH/REP macros, XC_DENSITY=1 etc., `functional_data` struct, `vars_data` struct)
- **xcfun-master/src/xcint.cpp** lines 93-135 (31-row `xcint_vars[]` table — VARS_TABLE source-of-truth)
- **xcfun-master/api/xcfun.h** lines 35-122 (`xcfun_mode` 5-variant enum with UNSET=0; `xcfun_vars` 31 variants with VARS_UNSET=-1; full C ABI surface for FFI shim)
- **xcfun-master/src/config.hpp** lines 22-50 (TINY_DENSITY=1e-14, XCFUN_REF_PW92C undefined by default, ireal_t = double)
- **xcfun-master/src/XCFunctional.cpp** lines 493-617 (PartialDerivatives dispatcher orders 0/1/2 — output element ordering verified)
- **xcfun-master/src/functionals/list_of_functionals.hpp** (78 entries XC_SLATERX..XC_PW91C ordered by historical insertion — drives FunctionalId discriminant assignment)
- **xcfun-master/src/functionals/{slaterx, vwn3, vwn5c, pw92c, pz81c, ldaerfx, ldaerfc, ldaerfc_jt, tfk, tw, vonw}.cpp** (11 LDA bodies — sampled, confirmed test_threshold values)
- **xcfun-master/src/functionals/aliases.cpp** lines 17-80 (alias source-of-truth — 46 entries; verified no LDA-only aliases for Phase 2)
- **xcfun-master/src/functionals/pw92eps.hpp** lines 36-58 (PW92C_PARAMS table + accurate vs reference constant gating on XCFUN_REF_PW92C)
- **xcfun-master/src/functionals/vwn.hpp** lines 19-91 (VWN3/VWN5 helpers — used by VWN3C/VWN5C/LDAERFC_JT)
- **xcfun-master/src/CMakeLists.txt** + **functionals/CMakeLists.txt** (cc-build target structure for Wave-2 reference)
- **crates/xcfun-ad/src/{lib.rs, ctaylor.rs, ctaylor_rec/, math.rs, expand/, index.rs, for_tests/cpu_client.rs}** (Phase 1 output — `#[cube] fn` pattern + OnceLock client + scratch allocation idiom)
- **crates/xcfun-ad/tests/cubecl_spike.rs** (cubecl-cpu launch_unchecked working example)
- **crates/xcfun-core/src/{lib.rs, enums.rs, error.rs, traits.rs, functional_id.rs, constants.rs, density_vars.rs, test_data.rs}** (pre-pivot scaffold — disposition assessed per D-05)
- **crates/xcfun-eval/src/lib.rs** + **Cargo.toml** (current state — 1-line stub, dependencies declared but no implementation)
- **xtask/src/bin/check_no_fma.rs** (Phase 1 pattern: cargo rustc --emit=asm + symbol-grep gate — direct template for check-no-mul-add)
- **/Cargo.toml** (workspace members + exclude state — confirms 5-of-7 crates excluded; cubecl-cpu in workspace deps; cc 1.2.60 in workspace deps)
- **/.cargo/config.toml** (ACC-05 already in place: `-Cllvm-args=-fp-contract=off` in [build] AND [target.'cfg(all())'])
- **/.planning/research/PITFALLS.md** (P1, P3, P5, P10, P11, P12, P13 directly relevant to Phase 2)
- **/.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md** (Phase 1 D-01 1e-12 strict; D-02 no mul_add; D-04 cubecl Array storage; D-09 Float trait; D-15 1-thread launch; D-23 functional bodies move forward)
- **/.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-07-PLAN.md** (xtask check-no-fma pattern; batch-per-property + criterion idioms)

### Secondary (HIGH confidence — Context7-verified)
- **CubeCL book** `language-support/struct.md` (via Context7 `/tracel-ai/cubecl`) — `#[derive(CubeType, CubeLaunch)]` supports nested types, methods on cube structs, comptime fields, `&mut` kernel arguments
- **CubeCL book** `core-features/comptime.md` (via Context7) — `#[unroll]` on comptime range, `#[comptime]` parameter on `#[cube] fn`, `comptime!()` macro
- **CubeCL book** `language-support/trait.md` (via Context7) — `#[cube] trait` with `'static + Send + Sync` bounds; associated types

### Tertiary (LOW confidence — propagation analysis, not measured)
- **D-19 LDAERF rel-error estimate ~2e-8 from cubecl polyfilled erf** — desk analysis only; actual Wave-2 measurement may show different magnitude depending on kf/μ regime sampled by the bulk grid stratum

## Metadata

**Confidence breakdown:**
- D-02 cubecl nesting verdict: HIGH (Context7 + Phase 1 precedent)
- D-19 LDAERF tolerance: MEDIUM (desk propagation analysis only — REQUIRES Wave-2 measurement before user sign-off)
- Standard stack inheritance: HIGH (Phase 1 already exercises everything)
- Architecture (Pattern A `DensVarsDev`): HIGH (cubecl docs + Phase 1 idiom)
- CORE-10 extractor approach: HIGH (cc + regex pattern is straightforward)
- Validation harness bring-up: HIGH (cc 1.2.60 + xcfun-master vendored)
- Mode::PartialDerivatives output layout: HIGH (verified XCFunctional.cpp line-by-line)
- Wave-0 commit order: HIGH (D-09 + dependency analysis)
- Grid generator spec: HIGH (rand_xoshiro 0.8 reproducibility verified by Phase 1 use)
- PW92C feature flag: HIGH (config.hpp explicit; ship default, defer flag)
- Pitfalls (PHASE2 A-E): MEDIUM-HIGH (PHASE2-D specifically caught by direct .cpp read; others are inferred from cubecl + Phase 1 patterns)

**Research date:** 2026-04-20
**Valid until:** 2026-05-20 (30 days for stable phase port; D-19 propagation analysis re-validate at Wave-2 measurement)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | cubecl `Array::new(comptime!(len))` does NOT zero-init the buffer (defensive `ctaylor_zero` is required) | D-02 + build_densvars Pattern + Open Question 6 | Defensive zeros are a perf no-op if wrong (~22 instructions per kernel call); not load-bearing. |
| A2 | LDAERF rel-error vs C++ on cubecl-cpu is ~2e-8 (worst case) — derived from desk propagation, NOT measured | D-19 LDAERF Tolerance Analysis | If actual measurement shows < 1e-12, no override needed (good news). If > 2e-8, the override magnitude needs adjustment. Either way, Wave-2's measure-then-decide gate handles it. |
| A3 | cc-compile of XCFunctional.cpp + xcint.cpp + 11 LDA + 67 stubs takes 10-15s on Linux x86_64 g++ -O2 with parallel | Validation Harness Bring-Up | Underestimate by 2× would push Wave-2 first-run to 30s; still acceptable. |
| A4 | Cubecl monomorphization for `(F=f64, N ∈ {0,1,2}) × 11 functionals × 5 build_densvars arms = ~48 kernels` stays under cubecl-cpu's compile-time budget | Pitfall PHASE2-A | Phase 1 handled comparable monomorphization (Plan 01-07). If Phase 2 hits a budget surprise, fall back to comptime-dispatch single-kernel-per-functional. |
| A5 | `xcfun_eval` C ABI return values for `xcfun_eval_setup` and `xcfun_eval` propagate correctly via the FFI shim (`extern "C"` + `*mut`/`*const` raw pointers + Box leak/reclaim) | Validation Harness Bring-Up | If FFI shim mishandles pointer ownership → segfault or memory leak. Standard cbindgen-style pattern; low risk. |
| A6 | The 5 `build_densvars` arms (`XC_A, XC_N, XC_A_B, XC_N_S, XC_A_B_GAA_GAB_GBB`) cover ALL Vars values needed by Phase 2's 11 LDA functional `test_in`s | build_densvars Pattern | If a 6th arm is required (e.g., LDAERFC_JT actually needs XC_N_S internally) the planner adds it without restructuring. |
| A7 | `xcint.cpp::xcint_assure_setup()` requires every functional's `fundat_db<XC_*>::d` specialization at link time → `validation/c_stubs.cpp` provides 67 stubs | Validation Harness Bring-Up | If link-time symbol resolution is more lazy than expected, stubs may not be needed (smaller cc-compile). Worst case: extra 67 trivial declarations, harmless. |
| A8 | `rand_xoshiro 0.8.0` produces byte-identical seed sequences across Linux x86_64 / macOS aarch64 / Windows MSVC for `seed_from_u64(0x1234abcd)` | Grid Generator Spec | If platform-divergent, the grid changes per platform → tier-2 reproducibility breaks. xoshiro algorithm is pure integer arithmetic, so this risk is very low; but flagged for the planner to add a 3-platform CI smoke test verifying seed determinism. |

---

## RESEARCH COMPLETE

Phase 2 research is complete. The planner can now create PLAN.md files. Two open questions (Q1 LDAERF override authorisation; Q2 XcError UnknownName Copy compliance) require user decisions before Wave-0e and Wave-2 begin — recommend the planner surfaces these as upfront `RESOLVE-BEFORE-WAVE-START` items.

**Critical reminders for the planner:**
- Phase 1 D-09 (Float trait) and D-04 (cubecl Array storage) are inviolable — no host-scalar `<T: Num>` paths.
- D-19 LDAERF override is the most likely source of `PLANNING INCONCLUSIVE` escalation; build the user-approval gate into Wave-2 explicitly.
- D-09 atomic-commits applies to Wave-0 (6 commits, sequential) AND continues into Wave-1+ (one commit per task).
- TW and VWK are kinetic-GGA (not pure LDA) — Wave-1C must use the GGA densvars builder.
- `XCFUN_REF_PW92C` ships UNDEFINED in vendored xcfun-master; Phase 2 ports the accurate constants directly; Cargo feature flag deferred to a future cleanup phase.
