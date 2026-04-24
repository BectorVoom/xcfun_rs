# Phase 3: GGA Tier + `Mode::Potential` — Research

**Researched:** 2026-04-24
**Domain:** GGA exchange–correlation functional ports + `Mode::Potential` divergence construction + `Mode::PartialDerivatives` orders 3..=4 extension, atop the Phase 2 cubecl-native substrate
**Confidence:** HIGH (all claims derived from in-tree C++ reference and committed Rust Phase 2 code; no training-data reliance)

---

## Summary

Phase 3 is an **additive port** — not an architectural pivot. Phase 2 already shipped the cubecl-native substrate (`DensVarsDev<F>` + `#[cube] fn <name>_kernel<F, const N>` + `dispatch_kernel` comptime if-chain + tier-2 parity harness), so Phase 3 stacks on top in three orthogonal layers:

1. **xcfun-ad substrate bump (2 new primitives — D-05/D-06):** `ctaylor_expm1` (used by 9 of 40 GGAs — all the PBEC/APBEC/SPBEC/PBEINTC/PBELOCC/ZVPBESOLC/ZVPBEINTC/RPBEX/PW91C "A = beta_gamma / expm1(-eps/(gamma·u³))" family, plus BECKESRX/BECKECAMX); and `ctaylor_sqrtx_asinh_sqrtx` (used by PW91X + PW91K + all 4 Becke family bodies — B88 enhancement's `1 + 6*d*sqrtx_asinh_sqrtx(chi2)` denominator). Both are line-for-line ports of `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:85-102` (expm1) and `:276-325` (sqrtx_asinh_sqrtx, with its [8,8] Padé at `|t.c[0]| < 0.5`).

2. **DensVarsDev Vars arms (Wave 0):** 7 new comptime arms in `build_densvars` — `XC_A_GAA` (id=4, inlen=2), `XC_N_GNN` (id=5, inlen=2), `XC_N_S_GNN_GNS_GSS` (id=7, inlen=5), and the four `_2ND_TAYLOR` variants (ids 27..30 per xcfun-core enum — **CONTEXT.md D-10 cites ids 26..29, off-by-one — planner must verify against `crates/xcfun-core/src/enums.rs:73-76`**). `XC_A_B_GAA_GAB_GBB` (id=6) already ships from Plan 02-05 Wave-1C-1 and is reused by 34 of the 40 GGAs unchanged.

3. **40 GGA kernel bodies + `Mode::Potential` + orders 3..=4:** Per-family `#[cube] fn` ports modelled on `lda/slaterx.rs`/`vwn3c.rs`, plus a line-for-line port of `XCFunctional.cpp:637-790` for the `-∇·(∂E/∂g)` divergence sum, plus bumping the host-side launch loop in `functional.rs` from orders 0..=2 to orders 0..=4.

**Primary recommendation:** The planner MUST gate every family wave behind a **Wave-0 fixture-gate** covering the two new xcfun-ad primitives at orders 0..=4 before any functional body lands. Three structural surprises in the C++ source require CONTEXT.md amendments — (a) BRX/BRC/BRXC + CSC are *not* pure GGAs (they declare `XC_KINETIC | XC_LAPLACIAN | XC_JP` — they are metaGGA-class dependencies and belong in Phase 4, NOT Phase 3 unless CONTEXT is amended); (b) LB94 source is entirely behind `#if 0` upstream (confirms D-19 to defer); (c) the `_2ND_TAYLOR` Vars discriminants in the Rust enum are 27..30, not 26..29 as CONTEXT.md asserts.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (25 total — D-01..D-25)

**Scope + wave strategy:**
- **D-01:** Port **40 GGA functional IDs** (not 45 as ROADMAP says) — the authoritative count is `REQUIREMENTS.md` GGA-01..GGA-10 ∩ `FunctionalId` enum. Planner to correct ROADMAP Phase 3 Goal wording to 40.
- **D-02:** **Wave 0 = substrate extension** (xcfun-ad primitives + DensVarsDev Vars arms + GGA shared helpers). Atomic (one commit per primitive / arm / helper) and MUST pass a fixture-gate before any functional wave begins.
- **D-03:** Wave-based parallelism. Wave 1 = PBE+Becke+BR+LYP (20 functionals); Wave 2 = OPTX+PW86/PW91+P86+APBE (10); Wave 3 = B97+KT/BTK/CSC (9); Wave 4 = Mode::Potential (LDA + GGA divergence); Wave 5 = orders 3..=4 + full-matrix tier-2 + Phase-2 ACC-04 re-run. Inside each wave, per-functional ports are embarrassingly parallel.
- **D-04:** `dispatch_kernel` gets **40 new comptime arms**. `supports(id)` bumps 11 → 51 ids. No fn-pointer dispatch, no runtime registry.

**xcfun-ad substrate extensions:**
- **D-05:** Add `expm1_expand` + `ctaylor_expm1` to `xcfun-ad`. Mandatory for GGA-01/07/08.
- **D-06:** Add `sqrtx_asinh_sqrtx` helper (composed op) to `xcfun-ad`. Mandatory for GGA-06 + GGA-02 (Becke family B88 enhancement).
- **D-07:** **No other xcfun-ad additions.** Any shortfall discovered at fixture-gate escalates via PLANNING INCONCLUSIVE.

**GGA shared helpers:**
- **D-08:** GGA shared helpers live in `crates/xcfun-eval/src/functionals/gga/shared/`: `pw91_like.rs`, `pbex.rs`, `pbec_eps.rs`, `b97_poly.rs`, `optx.rs`, `constants.rs`.
- **D-09:** Each shared helper is a `#[cube] fn` generic over `F: Float` with the single-launch kernel-signature convention. No host-side scalar stubs.

**DensVarsDev new Vars arms:**
- **D-10:** Add 7 new comptime arms in `build_densvars`: `XC_A_GAA` (id=4, inlen=2), `XC_N_GNN` (id=5, inlen=2), `XC_N_S_GNN_GNS_GSS` (id=7, inlen=5), `XC_A_2ND_TAYLOR` (**id=27 per xcfun-core enum, NOT 26 as CONTEXT states**, inlen=10), `XC_A_B_2ND_TAYLOR` (**id=28 per enum, NOT 27**, inlen=20), `XC_N_2ND_TAYLOR` (**id=29, NOT 28**, inlen=10), `XC_N_S_2ND_TAYLOR` (**id=30, NOT 29**, inlen=20). `XC_A_B_GAA_GAB_GBB` (id=6) already ships — no changes.
- **D-11:** Use **explicit helper-function chains** (never C-style fallthrough) per CORE-05 + Pitfall P5. The `_2ND_TAYLOR` arms chain to the corresponding non-Taylor arm after populating the Taylor coefficients.
- **D-12:** Pre-seeded CTaylor input layout per Plan 02-04 Wave-1B-14a amendment. Each Vars arm reads `input: &Array<F>` as a flat `(inlen × (1<<N))` block.

**Mode::Potential:**
- **D-13:** Line-for-line port of `XCFunctional.cpp:637-790`. LDA path: N=1 CTaylor, VAR0 seeding. GGA path: N=2 CTaylor, three directional launches (d/dx, d/dy, d/dz) summed to accumulate `-∇·(∂E/∂g)`. metaGGA rejection at `eval_setup` returns `XcError::InvalidMode`. Vars compat gate rejects non-`_2ND_TAYLOR` with `XcError::InvalidVars` mirroring `XCFunctional.cpp:437-447`.
- **D-14:** **Strict 1e-12** on all 40 GGAs + the 11 Phase-2 LDAs (rerun under Mode::Potential), subject to D-24 LDAERF 1e-7 override inheritance. No blanket Mode::Potential relaxation.
- **D-15:** `output_length` returns **2** for Mode::Potential when `fun->vars ∈ {XC_A, XC_A_2ND_TAYLOR}`, else **3**. Encodes MODE-05.

**Orders 3..=4:**
- **D-16:** Extend orders incrementally per family wave. Kernel signature already generic over N; extension requires (a) regen-ad-fixtures at N=3 and N=4, (b) tier-2 harness `--order 4`.
- **D-17:** Wave 5 runs the full-matrix tier-2 at `--order 4` across all 51 functionals.

**`erf` tolerance:**
- **D-18:** BECKESRX + BECKECAMX hold **strict 1e-12** by inheriting the Phase-2 in-kernel `erf_precise` libm port (commit `dca382a`). **D-24 LDAERF 1e-7 override does NOT extend to GGAs** — LDAERF's override was upstream-sourced (`ldaerfx.cpp:66`); GGA `erf` usage is not analogous.

**LB94 scope exclusion:**
- **D-19:** LB94 deferred from Phase 3 to Phase 5 (or Phase 4 as alias if feasible). **Reason:** `lb94.cpp` is entirely behind `#if 0 ... #endif` upstream, not in the 78-entry `FunctionalId` enum, uses legacy `setup_lb94` pattern (not FUNCTIONAL macro), and has no well-defined energy per its own source comment. Planner updates REQUIREMENTS GGA-10 wording.

**Phase 2 ACC-04 residual forward-action:**
- **D-20:** Wave 5 re-runs tier-2 `--order 2` for VWN3C/VWN5C/PW92C/PZ81C AFTER all GGA substrate + body work completes (build_densvars redesign may incidentally tighten the residuals).
- **D-21:** LDAERFX/LDAERFC/LDAERFC_JT are NOT retested — the cancellation is in the kernel's bracket algebra, not density-var construction. Forwarded to Phase 6 unchanged.

**Validation + CI:**
- **D-22:** `validation/build.rs` extends incrementally per family wave — append `.cpp`/`.hpp` files to `cc::Build::file(...)`, shrink `c_stubs.cpp` accordingly.
- **D-23:** Grid generator unchanged from Plan 02-06 (10k-point seed `0x1234abcd`; gradient strata already present).
- **D-24:** **No new CI targets.** Reuses Phase 2 CI: `fmt`, `clippy`, `test`, `xtask validate --backend cpu --order 2` (extended to 51 functionals), `xtask regen-registry --check`, `xtask check-no-mul-add` (scope extended), `xtask check-no-fma`.

**Error model:**
- **D-25:** `XcError::InvalidMode` + `XcError::InvalidVars` become reachable via new rejection paths — no new variants. `XcError` stays at 9 variants.

### Claude's Discretion (planner-owned)

- Per-functional file layout: one file per functional id mirroring LDA tier (`slaterx.rs`, `vwn3c.rs` ...), OR consolidate small bodies (e.g., OPTXCORR next to OPTX).
- Helper-module granularity (fuse `shared/pbex.rs` with `shared/pw91_like.rs` OR keep separate).
- Exact wave internal ordering (which family starts first; pipelining BR/LYP with PBE/Becke).
- Kernel-name prefix (`xcfun_eval_gga_<fn>_kernel` vs `xcfun_eval_<fn>_kernel`).
- Regen-registry handling of `#ifdef XCFUN_REF_PBEX_MU` conditional test data.
- Mode::Potential output-array layout details (`output[1] = pot_alpha` vs `pot_beta` exact ordering per `XCFunctional.cpp:666-669`).
- B97 kernel strategy: single parametrised `#[cube] fn b97_kernel<F, const N>(d, coefs, out, n)` vs 6 distinct fn bodies.

### Deferred Ideas (OUT OF SCOPE)

- **LB94** — Phase 5 (facade) or Phase 4 alias.
- **Mode::Contracted at any order** — Phase 4 (MODE-03).
- **Orders 5..=6** (any mode) — Phase 4.
- **15 metaGGA bodies + 46 aliases** — Phase 4.
- **Full `Functional` API surface (RS-01..10)** — Phase 5.
- **C ABI + cbindgen** — Phase 5 (CAPI-01..07).
- **Python bindings** — Phase 7.
- **CUDA / Wgpu backends** — Phase 6.
- **Criterion benches** (PERF-01/02) — Phase 6.
- **Phase-6 libm-hybrid for LDAERF** — D-21 forwards unchanged.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **GGA-01** | PBE family (12 IDs: XC_PBEX/PBEC/REVPBEX/RPBEX/PBESOLX/PBEINTX/PBEINTC/SPBEC/PBELOCC/ZVPBESOLC/ZVPBEINTC/VWN_PBEC) | §"File-by-File Mapping — PBE Family"; all 12 sources read and analysed (pbex.cpp, pbec.cpp, etc.); requires `ctaylor_expm1` (D-05), shared `pbex::enhancement`/`pw91_like_x_internal::S2`/`pbec_eps::A` (D-08) |
| **GGA-02** | Becke family (4 IDs: XC_BECKEX/BECKECORRX/BECKESRX/BECKECAMX) | §"File-by-File Mapping — Becke Family"; all 4 in `beckex.cpp`; requires `ctaylor_sqrtx_asinh_sqrtx` (D-06) + `ctaylor_erf` (Phase 1) + `ctaylor_expm1` (BECKESRX/BECKECAMX use it). BECKESRX/BECKECAMX read `XC_RANGESEP_MU` / `XC_CAM_ALPHA` / `XC_CAM_BETA` — parameter table support in `DensVarsDev.parent->settings`. |
| **GGA-03** | Becke–Roussel (3 IDs: XC_BRX/BRC/BRXC) | **CRITICAL:** `brx.cpp:138-157` declares `XC_DENSITY \| XC_GRADIENT \| XC_KINETIC \| XC_LAPLACIAN \| XC_JP` — these are NOT pure GGAs. They need `tau/lapa/lapb/jpaa/jpbb` from DensVarsDev. BR requires a Newton iteration via `taylor.hpp` (static double BR(double z) at brx.cpp:30-48, then `BR_taylor<T, Ndeg>` at :52-71 and ctaylor-adapting overload at :77-87). **Planner amendment needed: defer BRX/BRC/BRXC to Phase 4 OR plan a substantial BR-inverse-Newton extension to xcfun-ad.** |
| **GGA-04** | LYP correlation (XC_LYPC) | §"File-by-File Mapping — LYP"; pure GGA algebra, no new primitives needed. Uses `ctaylor_exp`, `ctaylor_pow`, `ctaylor_reciprocal` (all Phase 1). |
| **GGA-05** | OPTX family (2: XC_OPTX/OPTXCORR) | §"File-by-File Mapping — OPTX"; pure algebra with `ctaylor_pow`. Per-file layout: `gga/optx.rs` with both kernels (they share the `gamma=0.006` constant + same `g_xa2`/`g_xb2` construction). |
| **GGA-06** | PW86/PW91 (4: XC_PW86X/PW91X/PW91C/PW91K) | PW86X pure algebra (trivial). PW91X + PW91K share `pw91xk_enhancement` (pw9xx.hpp) which requires `ctaylor_sqrtx_asinh_sqrtx` (D-06) + `ctaylor_exp`. PW91C is the longest GGA body in the phase (87 lines of C++; four compound terms). |
| **GGA-07** | P86 correlation (2: XC_P86C/P86CORRC — both in `p86c.cpp`) | §"File-by-File Mapping — P86"; uses pz81eps (pz81c.hpp already ported Phase 2) — verify `pz81eps::pz81eps` is exported from `lda/pz81c.rs`. Uses DBL_EPSILON fudge at `gnn=0` — port as `F::cast_from(f64::EPSILON)`. |
| **GGA-08** | APBE (2: XC_APBEX/APBEC) | §"File-by-File Mapping — APBE"; requires `ctaylor_expm1` (D-05). Reuses pbex shared enhancement pattern with different mu=0.26, kappa=0.804. |
| **GGA-09** | B97 family (6: XC_B97X/B97C/B97_1X/B97_1C/B97_2X/B97_2C — in `b97xc.cpp`/`b97-1xc.cpp`/`b97-2xc.cpp`) | §"File-by-File Mapping — B97"; all 6 differ only by coefficient tables (`b97x::c_b97`/`c_b97_1`/`c_b97_2` + `b97c::c_b97`/`c_b97_1`/`c_b97_2`). Only 2 distinct enhancement algorithms (`b97xc::enhancement` degree-2 polynomial + `b97c::energy_b97c_par`/`antipar`). Reuse Phase-2 `pw92eps` for B97C LSDA baseline. |
| **GGA-10** | KT/BTK/CSC (3: XC_KTX/BTK/CSC — **LB94 deferred per D-19**) | §"File-by-File Mapping — KT/BTK/CSC"; KTX pure algebra. BTK pure algebra + `fudge=1e-24`. **CSC has same XC_KINETIC \| XC_LAPLACIAN \| XC_JP dependency as BRX** — planner amendment needed (defer CSC to Phase 4 OR extend DensVarsDev coverage). |
| **MODE-01** | `Mode::PartialDerivatives` orders 0..=4 | §"Orders 3..=4 Extension"; already CTaylor-ready at the kernel level (N generic). Host launch loop in `functional.rs:179-282` currently handles 0..=2; extension per D-16 adds 3..=4 arms following `XCFunctional.cpp:562-588` (order 3 seeding loop) + the `DOEVAL` macro for order 4. |
| **MODE-02** | `Mode::Potential` via `CTaylor<f64, 2>` divergence | §"Mode::Potential Algorithm"; line-for-line port of `XCFunctional.cpp:637-790` per D-13. Two new `#[cube] fn` entry points (`potential_lda_kernel` N=1, `potential_gga_kernel` N=2). |
| **MODE-05** | `Functional::output_length` correct for Potential (2 or 3 doubles) | §"output_length Mode::Potential"; direct port of `XCFunctional.cpp:482-490`, D-15. |
</phase_requirements>

---

## Project Constraints (from CLAUDE.md)

- **Accuracy:** 1e-12 relative error vs C++ xcfun on every `(functional, vars, mode, order, density point)` tuple. Non-negotiable.
- **f64 only** on the numerical path. No f32 in xcfun-eval kernels. `F: Float` bound with `F = f64` at launch (Phase 2 pattern).
- **No `.mul_add(`** in `crates/xcfun-eval/src/functionals/gga/**/*.rs` — `xtask check-no-mul-add` grep gate per ACC-06. Extends to gga/ subtree.
- **No FMA emission** — `xtask check-no-fma` asm gate unchanged (Phase 1 D-24).
- **No `-Cfast-math`, no reassociation flags.** RUSTFLAGS empty in CI; release profile has `-Cllvm-args=-fp-contract=off`.
- **`cubecl =0.10.0-pre.3` hard-pinned** — all 4 cubecl crates move in lockstep. `xtask check-cubecl-pin` gate.
- **`thiserror 2.0.18`** library errors. **`anyhow` only in validation/xtask/benches/examples** — never in xcfun-core/xcfun-eval/xcfun-ad. Enforced by `xtask check-no-anyhow`.
- **Edition 2024, MSRV 1.85.**
- **`#[forbid(unsafe_code)]`** at crate roots of xcfun-core + xcfun-eval. Phase 3 preserves. (`unsafe` is used ONLY inside `launch_unchecked` block in `functional.rs` Phase 2 code.)
- **MPL-2.0** licence.

---

## Architectural Responsibility Map

Phase 3 is a single-tier (xcfun-eval) feature with a small ripple into xcfun-ad (2 new primitives) and validation/ (incremental build.rs extension + c_stubs.cpp shrink). No cross-process or GPU concerns.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| `ctaylor_expm1` + `ctaylor_sqrtx_asinh_sqrtx` primitives | xcfun-ad (`crates/xcfun-ad/src/math.rs`, new `expand/expm1.rs` + new `composed::sqrtx_asinh_sqrtx`) | — | Pure Taylor-algebra operations — no density-var coupling. Mirror Phase 1 D-23 scoping: AD primitives live in xcfun-ad. |
| GGA shared helpers (`pw91_like`, `pbex`, `pbec_eps`, `b97_poly`, `optx`, `constants`) | xcfun-eval (`src/functionals/gga/shared/*.rs`) | — | These compose 2-3 xcfun-ad ops into a GGA-specific helper (enhancement factor, S², etc.). They are DENSITY-variable aware via `DensVarsDev<F>` — belong in xcfun-eval per Phase 2 D-04. |
| 40 GGA functional `#[cube] fn` bodies | xcfun-eval (`src/functionals/gga/<family>/*.rs`) | — | Per-functional body signature identical to Phase 2 LDA pattern. |
| `dispatch_kernel` 40-arm extension | xcfun-eval (`src/dispatch.rs`) | — | Direct extension of Phase 2 D-21 (comptime if-chain). |
| `DensVarsDev` 7-arm Vars extension | xcfun-eval (`src/density_vars/build.rs`) | — | Direct extension of Phase 2 Plan 02-05 Wave-1C-1 pattern. |
| `Mode::Potential` host-side launch + `#[cube] fn potential_{lda,gga}_kernel` | xcfun-eval (`src/functional.rs` + `src/functionals/potential.rs` new module) | — | Dispatch + per-point launch is host; the inner accumulator is a `#[cube] fn`. |
| Orders 3..=4 host launch loop extension | xcfun-eval (`src/functional.rs`) | — | Direct extension of the Phase 2 `launch_and_accumulate` match arms. |
| Incremental `cc::Build::file(...)` additions | validation/build.rs | — | Per family wave per D-22. |
| `c_stubs.cpp` auto-shrink | xtask (`regen-registry`) | validation/ | `regen-registry` re-run per wave shrinks stubs from 67 → ~27 by Phase 3 end. |
| Tier-2 `--order 4` full-matrix run | validation/ (Wave 5) | — | Existing driver; extends outer loop `for order in 0..=max_order` from 2 → 4. |
| Phase-2 ACC-04 residual re-run | validation/ (Wave 5) | — | Same driver, filtered to `VWN3C/VWN5C/PW92C/PZ81C`. |

---

## Standard Stack

Pinned in `CLAUDE.md`. Phase 3 introduces **no new dependencies** — pure additive ports on top of the Phase 1 + Phase 2 substrate.

### Core (unchanged from Phase 2)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cubecl` | `=0.10.0-pre.3` | `#[cube]` macro + `Array<F>` + `Float` trait + `CpuRuntime` | Phase 1+2 locked pin; 40 new `#[cube]` fns compile against it |
| `cubecl-cpu` | `=0.10.0-pre.3` | CpuRuntime | Phase 2 tier-1/tier-2 driver; 40 new kernels run on same backend |
| `thiserror` | `=2.0.18` | Library errors | `XcError` unchanged per D-25 |
| `bitflags` | `=2.11.1` (CLAUDE.md challenges Phase 2's `2.10.0` pin) | `Dependency` flags | Unchanged |
| `approx` | `=0.5.1` | `assert_relative_eq!` in tier-1 self-tests | Unchanged |

### GGA-specific (all already in the build)

| Module | Phase | Purpose | Used by Phase 3 |
|--------|-------|---------|-----------------|
| `xcfun_ad::math::ctaylor_pow` | Phase 1 | `ctaylor_pow(x, a)` — real-exponent power | Every PBE-family prefactor, every `pow(na, 4.0/3.0)` call |
| `xcfun_ad::math::ctaylor_exp` | Phase 1 | exp | PW91C (2 calls), P86C (`exp(-Pg(d))`), ZVPBESOLC/ZVPBEINTC (`exp(-alpha*v3*zw)`), PBELOCC (`exp(-r_s²)`) |
| `xcfun_ad::math::ctaylor_log` | Phase 1 | log | PBEC/APBEC/SPBEC/PBEINTC/PBELOCC/ZV*/VWN_PBEC `log(1 + beta_gamma·...)` terms; PW91C `log(1 + 2α(T²+A·T⁴)/...)`; LYPC nothing (uses exp); BRC/BRXC `log(1+z)` (deferred) |
| `xcfun_ad::math::ctaylor_sqrt` | Phase 1 | sqrt | PBEC `sqrt(d.a_43)`/`sqrt(d.b_43)` → phi(d); PW91C `sqrt(r)`; P86C `sqrt(fudge+d.gnn)` |
| `xcfun_ad::math::ctaylor_erf` | Phase 1 | erf (Phase 2 `erf_precise` libm port applied) | **BECKESRX, BECKECAMX** (D-18 strict 1e-12) |
| `xcfun_ad::math::ctaylor_asinh` | Phase 1 | asinh | Referenced indirectly through `sqrtx_asinh_sqrtx` (D-06). Direct: none of the 40 GGAs (rpbex uses `expm1`, not asinh). |
| `xcfun_ad::math::ctaylor_reciprocal` | Phase 1 | 1/x | Every division in every GGA body (no `ctaylor_div` — use `ctaylor_mul` + `ctaylor_reciprocal`) |
| `xcfun_ad::math::ctaylor_powi_{1..=10,0,neg1,neg2}` | Phase 1 | integer powers | PW91C `pow(d.zeta, 4)`; ZVPBESOLC `pow3(v)` = `ctaylor_powi_3` |
| `xcfun_ad::math::ctaylor_cbrt` | Phase 1 | cbrt | PW91C `cbrt(3π²ρ)`, P86C `cbrt(2) * sqrt(...)` |

### New (D-05 + D-06)

| New op | Source | Target location | Kernel-layer pattern |
|--------|--------|-----------------|----------------------|
| `expm1_expand<F>(t: &mut Array<F>, x0: F, #[comptime] n: u32)` | `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:84-102` ("only constant value is affected by cancellation; if `|t.c[0]| > 1e-3` → `tmp[0] -= 1` else `tmp[0] = 2*exp(x0/2)*sinh(x0/2)`") | `crates/xcfun-ad/src/expand/expm1.rs` (new file) | Port scheme: identical to `exp_expand` (Phase 1, `exp.rs`) for `i ≥ 1`; branch on `x0.abs() > 1e-3_f64` for `tmp[0]`. The sinh branch needs `F::cast_from(2.0_f64) * (x0/2).exp() * (x0/2).sinh()` — cubecl 0.10-pre.3 Float trait has both `.exp()` and `.sinh()`. |
| `ctaylor_expm1<F>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32)` | Same C++ source | `crates/xcfun-ad/src/math.rs` (add next to `ctaylor_exp`) | Standard 3-step pipeline: alloc scratch length n+1, call `expm1_expand::<F>(&mut scratch, x[0], n)`, call `ctaylor_compose::<F>(out, x, &scratch, n)`. |
| `ctaylor_sqrtx_asinh_sqrtx<F>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32)` | `xcfun-master/external/upstream/taylor/ctaylor_math.hpp:276-325` (branching on `fabs(t.c[0]) < 0.5`: if yes, shift [8,8] Padé polys P/Q, divide, compose; else `sqrt(t) * asinh(sqrt(t))` unstable form) | `crates/xcfun-ad/src/math.rs` (new fn; may need helper `crates/xcfun-ad/src/expand/sqrtx_asinh_sqrtx.rs` for the Padé branch) | **Composed op with TWO branches.** Unstable branch = `ctaylor_sqrt` → `ctaylor_asinh` → `ctaylor_mul`. Stable branch needs polynomial shift + inv_expand + multo — use `tfuns::shift`/`multo` (Phase 1 `tfuns.rs`). Constants P[9] + Q[9] are exact f64 literals from ctaylor_math.hpp:286-303. See §"sqrtx_asinh_sqrtx Port Strategy". |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| **Port BRX/BRC/BRXC in Phase 3** | Defer to Phase 4 | BR inversion requires `BR_taylor<T, Ndeg>` which uses `taylor.hpp` (a SEPARATE algebra from CTaylor) + a Newton iteration. This is 100+ LOC of new AD machinery. Also, BR* depend on `lapa`, `taua`, `jpaa` — metaGGA-class fields. Phase 4 scope (metaGGA) naturally covers this. **STRONG RECOMMENDATION: amend CONTEXT to defer.** |
| **Port CSC in Phase 3** | Defer to Phase 4 | `csc()` reads `d.taua`, `d.taub`, `d.jpaa`, `d.jpbb` — same metaGGA-class deps. **STRONG RECOMMENDATION: amend CONTEXT to defer.** |
| Single generic `#[cube] fn b97_kernel<F, const N>(d, ..., out, n)` | 6 separate fn bodies | Both satisfy algorithmic-identity (operation order identical). Single-fn saves ~400 LOC; 6-fn is trivial to audit. Planner's discretion per CONTEXT. **Recommendation: single generic, with comptime `#[comptime] coef_set: u32` choosing among {B97/B97-1/B97-2}.** |
| Port `expm1` via identity `exp(x)-1` | C++ upstream stable-bracket | Loses ~1e-14 precision near `x=0`. **Reject.** D-05 is explicit on the upstream port. |

---

## Architecture Patterns

### System Architecture Diagram

```
 host-side (xcfun-eval/Functional::eval)
       │
       │  input: &[f64]       ┌──────────────────────────────────────────┐
       │  vars, mode, order   │  Mode dispatch (D-13):                   │
       ├─────────────────────▶│  - PartialDerivatives (0..=4)            │
       │                      │  - Potential (LDA / GGA)                 │
       │                      │  - Contracted → InvalidMode (Phase 4)    │
       │                      └────────┬───────────────┬─────────────────┘
       │                               │               │
       │                   ┌───────────▼──┐      ┌─────▼──────────┐
       │                   │ Partial path │      │  Potential     │
       │                   │ (Plan 02-04  │      │  (D-13 path)   │
       │                   │  extended    │      │                │
       │                   │  0..=4)      │      │  LDA: N=1,     │
       │                   │              │      │  VAR0 seed     │
       │                   │ per (i,j...) │      │                │
       │                   │  launch loop │      │  GGA: N=2,     │
       │                   │              │      │  3× (d/dx,     │
       │                   │              │      │  d/dy, d/dz)   │
       │                   └──────┬───────┘      └──────┬─────────┘
       │                          │                     │
       │                          ▼                     ▼
       │                 ┌──────────────────────────────────────────┐
       │                 │   cubecl unsafe launch                   │
       │                 │   eval_point_kernel / potential_kernel   │
       │                 └──────┬───────────────────────────────────┘
       │                        │   (device-side: ~0 work in cubecl-cpu)
       ▼                        ▼
┌──────────────────────────────────────────────────────────────────────┐
│  #[cube] kernel (F: Float, #[comptime] id/vars/n)                    │
│                                                                      │
│  1. build_densvars(input, d, #[comptime] vars, #[comptime] n)        │
│     - Comptime if-chain over vars discriminant (9 arms total:        │
│       5 existing + 7 new — XC_A, XC_N, XC_A_B, XC_N_S,               │
│       XC_A_B_GAA_GAB_GBB [existing] + XC_A_GAA, XC_N_GNN,            │
│       XC_N_S_GNN_GNS_GSS, XC_A_2ND_TAYLOR, XC_A_B_2ND_TAYLOR,        │
│       XC_N_2ND_TAYLOR, XC_N_S_2ND_TAYLOR [new D-10])                 │
│     - regularize(out.a)  [+out.b if spin-resolved]                   │
│     - derived fields: zeta, r_s, n_m13, a_43, b_43                   │
│                                                                      │
│  2. dispatch_kernel(#[comptime] id, d, out, #[comptime] n)           │
│     - 51-arm comptime if-chain (11 LDA Phase 2 + 40 GGA Phase 3)     │
│     - Each arm calls functionals::{lda,gga}::<fn>::<fn>_kernel       │
│                                                                      │
│  3. <fn>_kernel — the functional body                                │
│     - Composes ctaylor_add/sub/mul/scalar_mul, ctaylor_pow,          │
│       ctaylor_exp/log/sqrt/cbrt/erf/expm1/sqrtx_asinh_sqrtx          │
│     - Reads d.a, d.b, d.gaa, d.gnn, d.r_s, ...                       │
│     - Writes out[0..(1<<n)]                                          │
└──────────────────────────────────────────────────────────────────────┘
```

**File-to-implementation mapping:** See §"File-by-File Mapping" below — one row per functional with primary source, shared helpers used, new primitives needed, recommended Rust file location.

### Recommended Project Structure

```
crates/xcfun-eval/src/
├── density_vars.rs                       # DensVarsDev struct — no change
├── density_vars/
│   ├── build.rs                          # EXTEND: 7 new arms (D-10)
│   └── regularize.rs                     # No change (D-22 preserved)
├── dispatch.rs                           # EXTEND: 40 new comptime arms (D-04)
├── functional.rs                         # EXTEND: Mode::Potential branch (D-13);
│                                         # orders 3/4 launch arms (D-16);
│                                         # output_length arms (D-15)
├── functionals/
│   ├── mod.rs                            # EXTEND: pub mod gga;
│   ├── lda/                              # UNCHANGED (Phase 2)
│   │   ├── slaterx.rs, vwn3c.rs, ...    # 11 LDA kernels
│   │   ├── vwn_eps.rs, pw92eps.rs      # LDA shared helpers
│   │   └── pz81c.rs                     # pz81eps used by GGA-07 P86C
│   └── gga/                              # NEW (Phase 3)
│       ├── mod.rs
│       ├── shared/                       # D-08 shared helpers
│       │   ├── mod.rs
│       │   ├── constants.rs              # R_pbe, R_revpbe, kappa, mu, CF, etc.
│       │   ├── pw91_like.rs              # chi2, S2, prefactor, pw91k_prefactor,
│       │   │                             #   pw91xk_enhancement
│       │   ├── pbex.rs                   # enhancement, enhancement_RPBE,
│       │   │                             #   energy_pbe_ab
│       │   ├── pbec_eps.rs               # A_expm1_inner, H, phi helper
│       │   ├── b97_poly.rs               # spin_dens_gradient_ab2, ux_ab, enhancement,
│       │   │                             #   energy_b97x_ab, energy_b97c_par,
│       │   │                             #   energy_b97c_antipar
│       │   └── optx.rs                   # shared g_xa2/g_xb2 construction
│       ├── pbe/                          # Wave 1 (12 functionals)
│       │   ├── mod.rs, pbex.rs, pbec.rs, revpbex.rs, rpbex.rs,
│       │   │   pbesolx.rs, pbeintx.rs, pbeintc.rs, spbec.rs,
│       │   │   pbelocc.rs, zvpbesolc.rs, zvpbeintc.rs, vwn_pbec.rs
│       ├── becke/                        # Wave 1 (4 functionals)
│       │   ├── mod.rs, beckex.rs, beckecorrx.rs, beckesrx.rs, beckecamx.rs
│       ├── lyp.rs                        # Wave 1 (1 functional — GGA-04)
│       ├── optx/                         # Wave 2 (2: optx.rs + optxcorr.rs OR fused)
│       ├── pw91/                         # Wave 2 (4: pw86x.rs, pw91x.rs, pw91c.rs, pw91k.rs)
│       ├── p86/                          # Wave 2 (2: p86c.rs, p86corrc.rs — both from p86c.cpp)
│       ├── apbe/                         # Wave 2 (2: apbex.rs, apbec.rs)
│       ├── b97/                          # Wave 3 (6: b97x.rs, b97c.rs,
│       │   │                             #   b97_1x.rs, b97_1c.rs, b97_2x.rs, b97_2c.rs)
│       └── kt/                           # Wave 3 (3: ktx.rs, btk.rs, csc.rs)
│                                         #   NOTE: csc.rs needs tau/lapa/jpa dep —
│                                         #   PLANNER: consider deferring to Phase 4
│
└── functionals/potential.rs               # NEW (Wave 4) — potential_lda_kernel +
                                           #   potential_gga_kernel (D-13)

crates/xcfun-ad/src/
├── math.rs                                # EXTEND: ctaylor_expm1 + ctaylor_sqrtx_asinh_sqrtx
└── expand/
    └── expm1.rs                           # NEW (D-05): expm1_expand
    └── sqrtx_asinh_sqrtx.rs               # (optional — inv_expand + shift are reusable from
                                           #   Phase 1 tfuns.rs; may inline in math.rs)
```

### Pattern 1: Per-functional kernel body

**What:** Each GGA functional has one `#[cube] fn <name>_kernel<F: Float>(d, out, #[comptime] n: u32)` that 1:1 ports the C++ `static num <name>(const densvars<num> &d)` function.

**When to use:** Every one of the 40 GGA bodies. No exceptions.

**Example — port pattern for `pbex.cpp:20-23` → `gga/pbe/pbex.rs`:**

```rust
//! XC_PBEX — PBE exchange. GGA-01.
//! # Source
//! - `xcfun-master/src/functionals/pbex.cpp:20-23` (body)
//! - `xcfun-master/src/functionals/pbex.hpp:26-39` (`pbex::enhancement`)
//! - `xcfun-master/src/functionals/pw9xx.hpp:51-63` (`pw91_like_x_internal::prefactor`)
//!
//! # Formula
//! E = pbex::energy_pbe_ab(R_pbe, d.a, d.gaa) + pbex::energy_pbe_ab(R_pbe, d.b, d.gbb)
//!   where energy_pbe_ab = prefactor(rho) * enhancement(R, rho, grad)
//!
//! # Preconditions
//! - d.a, d.b regularized (> 0) by `build_xc_a_b_gaa_gab_gbb` chain.
//! - d.gaa, d.gbb finite (input fixture).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pbex::energy_pbe_ab;
use crate::functionals::gga::shared::constants::R_PBE_F64;

#[cube]
pub fn pbex_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // C++: return energy_pbe_ab(R_pbe, d.a, d.gaa) + energy_pbe_ab(R_pbe, d.b, d.gbb);
    let size = comptime!((1_u32 << n) as usize);
    let mut ea = Array::<F>::new(size);
    let mut eb = Array::<F>::new(size);
    let r = F::cast_from(R_PBE_F64);
    energy_pbe_ab::<F>(r, &d.a, &d.gaa, &mut ea, n);
    energy_pbe_ab::<F>(r, &d.b, &d.gbb, &mut eb, n);
    ctaylor_add::<F>(&ea, &eb, out, n);
}
```

### Pattern 2: GGA shared helper module

**What:** Each `gga/shared/*.rs` ports a C++ namespace from `pbex.hpp` / `pw9xx.hpp` / `b97*.hpp` into a set of `#[cube] fn` helpers with single-launch-kernel signature.

**When to use:** Whenever two or more GGA functionals share non-trivial algebra.

**Example — `gga/shared/pw91_like.rs` (`S2` + `pw91xk_enhancement`):**

```rust
//! pw91_like shared helpers. 1:1 port of
//! `xcfun-master/src/functionals/pw9xx.hpp:38-94`.

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_scalar_mul;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_exp, ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrtx_asinh_sqrtx};
use crate::density_vars::DensVarsDev;

/// S²(ρ, γ) = γ / ρ^(8/3) * ( 6^(2/3) / (12·π^(2/3)) )²
///   Factor simplifies to: 1/(4*(3π²)^(2/3))  (precomputed f64 constant)
///
/// Source: `pw9xx.hpp:43-46`.
#[cube]
pub fn s2_helper<F: Float>(
    rho: &Array<F>,
    grad: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // C++: grad / pow(rho, 8.0/3.0) * pow(pow(6, 2/3) / (12*pow(PI, 2/3)), 2.0)
    //
    // Precompute prefactor: (6^(2/3))² / (12·π^(2/3))² = 6^(4/3) / 144·π^(4/3)
    //   = 6^(4/3) / (144 · π^(4/3))
    //   f64: 3.2396... / (144 · 4.6011...) ≈ 0.00489...
    //   Exact formula: 1 / (4 · (3π²)^(2/3))
    const S2_PREFACTOR_F64: f64 = 1.0_f64 / (4.0_f64 * 2.9615953242_f64.powf(2.0 / 3.0));

    let size = comptime!((1_u32 << n) as usize);
    let mut rho83 = Array::<F>::new(size);
    ctaylor_pow::<F>(rho, F::cast_from(-8.0_f64 / 3.0_f64), &mut rho83, n);
    let mut tmp = Array::<F>::new(size);
    ctaylor_mul::<F>(grad, &rho83, &mut tmp, n);
    ctaylor_scalar_mul::<F>(&tmp, F::cast_from(S2_PREFACTOR_F64), out, n);
}
```

### Anti-Patterns to Avoid

- **C-style fallthrough** in `build_densvars` arms. The C++ `densvars.hpp:35-218` chains cases like `XC_A_B_GAA_GAB_GBB → XC_A_B → break`; Rust match has no fallthrough. **D-11 mandates explicit helper-function chains** — e.g., `build_xc_a_b_gaa_gab_gbb` calls `build_xc_a_b` at the end (Phase 2 Plan 02-05 pattern).
- **Hand-inlining shared helpers.** A B97-family port inlining `b97xc::enhancement` 6 times would bloat the crate + break any constant-table fix. Use the shared helper pattern.
- **Inlining scalar constants inside `#[cube]` via `F::new(0.066725)`.** `F::new` takes f32 on cubecl 0.10-pre.3 — rounds f64 constants to ~1e-8. Use `F::cast_from(CONST_F64)` where `CONST_F64: f64`. All PBE/APBE/Becke constants are sensitive enough that f32 rounding breaks 1e-12 parity. Phase 2 pattern: `NEG_C_SLATER_F64`, `RS_PREFACTOR_F64` — follow it.
- **Using `PI` inside the kernel via `F::PI()`.** cubecl's `Float::PI()` is likely emitted as a float constant at codegen time. Use a precomputed `const X_PI_POW_F64: f64 = <precomputed against std::f64::consts::PI>`. Several GGA bodies multiply by `pow(PI, 1/3)` or `pow(PI, 2/3)` — precompute host-side.
- **Calling `ctaylor_div` (does not exist).** Phase 1 decision: `a / b` expressed as `ctaylor_mul(a, reciprocal_b, ...)`. Every division in every GGA body follows this pattern.
- **`ctaylor_pow(x, F::cast_from(2.0))` for integer powers.** Use `ctaylor_powi_2` (Phase 1). More precise (no `log(x)*2` detour) and matches C++ `pow2(x)` / `pow3(x)` helpers.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Taylor expansion of `exp(x)-1` | Custom `exp(x) - F::cast_from(1.0)` | `ctaylor_expm1` (D-05) | Loses ~10 ULP near `x=0` — PBEC exhibits `expm1(-eps/(γ·u³))` where the argument regularly approaches 0 at high densities. ULP-loss directly breaks 1e-12 parity. |
| `sqrt(x)·asinh(sqrt(x))` near `x=0` | `ctaylor_sqrt(x) · ctaylor_asinh(ctaylor_sqrt(x))` | `ctaylor_sqrtx_asinh_sqrtx` (D-06) | The composition is non-differentiable at `x=0` — derivatives of the AD polynomial blow up. Upstream uses [8,8] Padé approximant at `|t.c[0]| < 0.5`. **Branching required** — direct composition fails at `grad = 0` fixtures. |
| BR inversion (BRX/BRC/BRXC) | A new Newton-iteration scheme | `BR_taylor<T, Ndeg>` from `brx.cpp:52-71` — a SEPARATE Taylor algebra (`taylor.hpp`, not `ctaylor.hpp`) | **STRONGLY RECOMMEND DEFERRING TO PHASE 4.** The `taylor.hpp` module is a different data layout (dense polynomials, not bit-flag-indexed); porting requires a new xcfun-ad submodule. Phase 4 metaGGA scope aligns better (BRX declares XC_LAPLACIAN\|XC_KINETIC\|XC_JP). |
| Regen of derived fields `zeta, r_s, n_m13, a_43, b_43` | Reimplement in each Vars arm | Let `build_densvars` top-level populate them after the arm chain | Phase 2 pattern — already works. Tests in `regularize_invariant.rs` cover it. |
| The GGA `erf` polyfill | Cubecl's stock `Float::erf` (1.3e-8 polyfill) | Phase 2's in-kernel `erf_precise` libm port (commit `dca382a`) | D-18 locks this path. The polyfill is 5 orders of magnitude less precise than 1e-12 requires. |

**Key insight:** Every GGA family has at least ONE gotcha where naïve Rust would silently lose precision — `expm1` cancellation, Padé approximant for non-differentiable Taylor, B97 polynomial conditioning at high `ux`, PW91C's combinatorial log/exp stack. The xcfun C++ reference has already found and fixed all of these. The Rust port's ONLY legitimate path is line-for-line translation. Algorithmic creativity is the enemy of 1e-12 parity.

---

## Common Pitfalls

### Pitfall G1: `ctaylor_expm1` branching threshold mismatch
**What goes wrong:** The upstream port branches at `|x0| > 1e-3`. A naïve reformulation (say, `|x0| > 1e-6`) produces different cancellation behaviour at the tier-2 fixture grid's high-density PBE points.
**Why it happens:** The threshold `1e-3` is tuned to keep the constant-term relative error below f64 machine epsilon. Changing it imports cancellation error into the CTaylor series at every order.
**How to avoid:** Port the exact constant `1e-3_f64` (cast via `F::cast_from`). Add a fixture-gate at Wave-0 comparing 100 random `(x0, N)` tuples against mpmath at 200-digit precision; any diff > 1e-14 fails.
**Warning signs:** Tier-2 PBEC drift concentrated at `min(a,b) > 10` density regions — the `u³` denominator there drives `-eps/(γ·u³) → 0`, which is exactly the stable-bracket domain.

### Pitfall G2: `sqrtx_asinh_sqrtx` Padé branch polynomial-shift precision
**What goes wrong:** `tfuns<T, 8>::shift(pq, t.c[0])` with `t.c[0]` near 0.5 (the branch boundary) produces catastrophic cancellation in the shift's intermediate coefficients.
**Why it happens:** The [8,8] Padé polynomials P[9], Q[9] have coefficients spanning `[2.87e-2, 1.92e4]` — 6 orders of magnitude. Shifting them by `t.c[0] ~ 0.5` with naive Horner requires order-by-order cancellation management.
**How to avoid:** Port `tfuns<T, Ndeg>::shift` verbatim from `taylor_math.hpp` (already exists in Phase 1 `tfuns.rs`). Do NOT "simplify" by computing shifted polynomials algebraically; that loses the incremental-sum structure. Add a fixture-gate at 200 random `(x, N)` with `x ∈ [0, 0.5]` vs mpmath.
**Warning signs:** PW91X/PW91K tier-2 failing at the `grad² ∈ [1e-6, 1e-3]` fixture band — that's exactly the domain where `S²(rho, grad)` crosses the 0.5 Padé-boundary for typical densities.

### Pitfall G3: `Mode::Potential` C-fallthrough vs explicit chain in the `_2ND_TAYLOR` arms
**What goes wrong:** `densvars.hpp:111-147` has the `XC_N_2ND_TAYLOR → XC_N_NX_NY_NZ_TAUN → XC_N_NX_NY_NZ` chain with two C-style fallthroughs. Rust-side explicit chains must populate the Taylor-coefficient fields (lapa = 0.5*(d[4]+d[7]+d[9]) etc.) BEFORE calling the base arm's builder.
**Why it happens:** Phase 2 Pitfall P5 pattern extended. In the `_2ND_TAYLOR` case, the 10- or 20-slot input has a specific order (see `XCFunctional.cpp:679` comment: `n gx gy gz xx xy xz yy yz zz`) and the `lapa = 0.5*(d[4]+d[7]+d[9])` computation consumes slots [4]/[7]/[9] — the base `XC_N_NX_NY_NZ` arm then consumes slots [0..=3] plus writes `gnn = d[1]² + d[2]² + d[3]²`. Chain order matters.
**How to avoid:** Implement `build_xc_n_2nd_taylor` with explicit calls: (1) populate `lapa` from slots 4/7/9 (scalar, not CTaylor — these are 2nd-Taylor coefficients already); (2) call `build_xc_n_nx_ny_nz`. Test with fixture exercising both the `lapa` population and the downstream `gnn` computation.
**Warning signs:** Mode::Potential fixture shows `pot_alpha` matching but `pot_beta` diverging for spin-resolved cases — points to a missed fallthrough step for `XC_A_B_2ND_TAYLOR`.

### Pitfall G4: BECKESRX/BECKECAMX `erf` + `expm1` cancellation cascade
**What goes wrong:** BECKESRX body at `beckex.cpp:42-53` computes
```
num K = 2 * (cparam + (d*chi2)/(1 + 6*d*sqrtx_asinh_sqrtx(chi2)));
num a = mu * sqrt(K) / (6 * sqrt(PI) * pow(na, 1/3));
num b = expm1(-1 / (4*a*a));   // <-- cancellation zone
num c = 2*a*a*b + 0.5;
return -0.5*na43*K*(1 - 8/3*a*(sqrt(PI)*erf(1/(2*a)) + 2*a*(b - c)));
```
At large `a` (high density), `-1/(4a²) → 0`, so `expm1 → 0` (stable-bracket domain for D-05). But then `2a²·b → -1/2 + ...` → `c → 0 + ...`, and the `(b - c)` subtraction catastrophically cancels. Phase-1 cubecl polyfill erf made this drift beyond 1e-12 even without the cancellation. With `erf_precise` (D-18), the erf side is tight; the expm1-side cancellation remains as the residual risk.
**Why it happens:** Upstream code comment (`beckex.cpp:36-40`) explicitly flags it: *"the erf + something is basically the erf minus its asymptotic expansion. This is horribel for numerics"*. Upstream used `expm1` + cancellation-aware bracket to mitigate.
**How to avoid:** Port the bracket algebra VERBATIM. Do NOT substitute `expm1(x)` with `exp(x) - 1`. Do NOT pre-compute `c - b` as a separate simplification. Run tier-2 at Wave-1 fixture-gate; escalate via PLANNING INCONCLUSIVE if drift > 1e-12 (D-18 explicit).
**Warning signs:** Tier-2 BECKESRX failures concentrated at `(a, gaa)` ≈ `(0.39E+2, 0.81E+6)` — the xcfun-reference test point — where `a = mu·√K / (6·√π·a^(1/3))` is large and `-1/(4a²)` is small. If Wave-1 passes this point at 1e-12, the cancellation is under control.

### Pitfall G5: PBE R=0.804 vs REVPBE R=1.245 parameter swap
**What goes wrong:** A developer reading `revpbex.cpp` and `pbex.cpp` side-by-side may accidentally swap `R_pbe ↔ R_revpbe` in the shared `pbex::enhancement(R, rho, grad)` call — they share the enhancement function signature but differ ONLY in the first argument.
**Why it happens:** Both functions have identical body except for `R_pbe` vs `R_revpbe`. pbex.hpp:22-23 defines both constants together. Easy to cross-wire.
**How to avoid:** **Centralise R constants in `gga/shared/constants.rs`**:
```rust
pub const R_PBE_F64: f64 = 0.804_f64;
pub const R_REVPBE_F64: f64 = 1.245_f64;
pub const KAPPA_PBESOL_F64: f64 = 0.804_f64;  // = R_PBE
pub const KAPPA_APBE_F64: f64 = 0.804_f64;
pub const MU_PBE_F64: f64 = 0.219_514_972_764_517_1_f64;  // = 0.066725 * π² / 3
pub const MU_PBESOL_F64: f64 = 0.123_456_790_123_f64;
pub const MU_APBE_F64: f64 = 0.26_f64;
pub const ALPHA_PBEINT_F64: f64 = 0.197_f64;
```
Each PBE-family body imports the one it needs by name. No bare numeric literals in the kernel bodies.
**Warning signs:** REVPBEX passes tier-2 but PBEX fails (or vice versa). Debug: assert `R_PBE_F64 < 1.0 < R_REVPBE_F64`.

### Pitfall G6: B97 polynomial conditioning at order 4 (tier-2 `--order 4`)
**What goes wrong:** B97 enhancement `c0 + c1·ux + c2·ux²` where `ux = Γ·s²ᵢ / (1 + Γ·s²ᵢ)` is bounded in `[0, 1)`. At order 4, the 4th derivative of the quadratic wrt the density input accumulates `c2 · (ux')⁴ · (ux composition)` terms. `c2` for B97-2 is `-7.44060` — 10× larger than `c1`. At order 4 this gives an amplification factor of `|c2|⁴ ≈ 3056`, pushing the 4th-deriv term toward numerical condition limits.
**Why it happens:** Polynomial amplification of derivative magnitude, nothing more. But the 1e-12 threshold is `rel_err = |diff| / max(|ref|, 1.0)` so a 4th-derivative magnitude of 1e4 needs absolute accuracy of 1e-8 — the CTaylor product chain of 4 multiplications then caps us at 1e-12.
**How to avoid:** Port operation order EXACTLY. In particular, preserve the C++ operator precedence at `c_params[0] + c_params[1]*ux + c_params[2]*ux*ux` as three `ctaylor_add`/`ctaylor_scalar_mul` steps — do NOT reorder into Horner form `c_params[0] + ux*(c_params[1] + ux*c_params[2])`.
**Warning signs:** B97-2 (never B97 nor B97-1) shows tier-2 residuals at order 4 just above 1e-12 on the bulk grid, not the gradient-zero band.

### Pitfall G7: PW92C legacy constants + VWN_PBEC shared helper
**What goes wrong:** `vwn_pbec.cpp` uses `vwn::vwn5_eps(d)` instead of `pw92eps::pw92eps(d)`. VWN5 is in Phase 2's `lda/vwn_eps.rs`. PBEC uses pw92eps in Phase 2's `lda/pw92eps.rs`. Both need to be importable from xcfun-eval's gga/ subtree.
**Why it happens:** Hierarchical import — `gga/pbe/vwn_pbec.rs` → `functionals::lda::vwn_eps::vwn5_eps`. Currently `lda/mod.rs` is `pub mod vwn_eps;` with items `pub use` — verify during Wave 0.
**How to avoid:** Check `crates/xcfun-eval/src/functionals/lda/mod.rs` exports. Ensure `vwn5_eps` + `pw92_eps` are `pub fn`s or re-exported from the crate root. Add a compile-time dummy test in Wave 0 that imports both from a gga/ module.
**Warning signs:** `cargo build -p xcfun-eval` fails with "fn vwn5_eps not found in scope" when `vwn_pbec.rs` is first added.

### Pitfall G8: `regularize` on `_2ND_TAYLOR` Vars arms
**What goes wrong:** Each `_2ND_TAYLOR` arm has a SCALAR density input at slot 0 (or 10 for beta), but the slots [1..9] hold first- and second-Taylor coefficients that were constructed AT SEEDING TIME — they are NOT Taylor coefficients of the CTaylor polynomial used for AD. Calling `regularize(out.a)` clamps the `a[CNST]` constant term, but the `a[VAR0]`, `a[VAR1]`, `a[VAR0|VAR1]` slots are not density but *density derivatives*. Clamping them would destroy the 2nd-Taylor construction.
**Why it happens:** The 2ND_TAYLOR Vars encode a pre-computed Taylor polynomial of density around a point, for use by the Potential-mode divergence construction. The Rust kernel uses Taylor AD on top — two layers of Taylor expansion coexist.
**How to avoid:** `regularize` must clamp ONLY `a[CNST]` (the CTaylor constant term) — this IS Phase 2 D-22. D-11 explicit chains preserve this. **But additional test:** a new `tests/regularize_2nd_taylor.rs` (mentioned in CONTEXT.md <specifics>) verifies that after `build_xc_a_2nd_taylor`, all 1..9 slots of the INPUT data are propagated into the output `DensVarsDev` fields without mutation.
**Warning signs:** Mode::Potential tier-2 failures at "clamp stratum" fixtures — the 1000-point regularize-clamp subgrid. Expected: Phase 2 already has Fix 2 exclusion, but for `_2ND_TAYLOR` inputs the clamp triggers on `d[0]` only, not `d[1..=9]`.

### Pitfall G9: RPBEX is the shortest body IF D-05 lands correctly
**What goes wrong:** RPBEX = `prefactor(d.a) * enhancement_RPBE(d.a, d.gaa) + prefactor(d.b) * enhancement_RPBE(d.b, d.gbb)` where `enhancement_RPBE(rho, grad) = 1 - R_pbe * expm1(-mu/R_pbe * S2(rho, grad))`. With `ctaylor_expm1` from D-05 + shared `S2` from D-08, RPBEX is ≤ 15 Rust LOC. But if D-05 has a bug in the sinh-branch for `|x0| ≤ 1e-3`, RPBEX fails at every small-gradient fixture.
**Why it happens:** RPBEX arguments to `expm1` are `-μ/R·S² → 0` as `grad² → 0`. This is the EXACT stability-bracket domain.
**How to avoid:** Wave 0 fixture-gate for `ctaylor_expm1` MUST include the `x0 ∈ [-1e-3, +1e-3]` stratum explicitly. Use 200 random samples (xoshiro seed `0xdeadbeef`) and compare against mpmath at 200-digit precision. Any diff > 1e-15 at CNST OR derivative-term magnitude > f64 eps fails the gate.
**Warning signs:** RPBEX passes tier-2 at high densities but fails at low densities — the gradient-zero stratum is the diagnostic.

### Pitfall G10: cubecl 0.10-pre.3 pre-release risk (P8 from Phase 1 research)
**What goes wrong:** Phase 3 adds 40 new kernel compile-site monomorphisations on cubecl-cpu. With `N∈{0,1,2,3,4}` and `id∈{40 new ids}` and `vars∈{6+ new arms}`, the monomorphisation count jumps from 11·5·3 = 165 (Phase 2) to ~51·5·9 ≈ 2295. Pre-release cubecl may have memory/perf regressions at this scale.
**Why it happens:** Each `launch_unchecked::<F, CpuRuntime>` with new comptime values triggers a cubecl JIT compile. The match-on-(id, n) in `run_launch` will explode from ~33 arms to 255+ arms.
**How to avoid:** Expect `functional.rs`'s `run_launch` to grow substantially. Planner should consider splitting it per-order OR per-vars to keep each match arm manageable. Also: run `time cargo test -p xcfun-eval --features testing` at end of Wave 0 to baseline, and track any > 2× compile-time regression per wave.
**Warning signs:** CI `test` job exceeding 5 s (Phase 2 ACC-04 gate) — D-24 does NOT bump this threshold. If it does, that's a planning flag.

---

## Runtime State Inventory

Not applicable — Phase 3 is purely additive / code-only. No rename, refactor, or migration. No stored data, no live-service config, no OS-registered state, no secrets, no build artifacts carry state from Phase 2 that would go stale.

*(Category explicitly checked: no `.planning/phases/03-*/` runtime state to preserve beyond the CONTEXT + RESEARCH docs themselves. Existing Phase-2 artifacts (`validation/report.html`, `validation/report.jsonl`) will be OVERWRITTEN by Wave 5 tier-2 re-run — this is intended and not a migration concern.)*

---

## Code Examples

Verified patterns from the Phase 2 codebase + upstream C++:

### Example 1: `ctaylor_expm1` port (D-05)

**C++ source** (`xcfun-master/external/upstream/taylor/ctaylor_math.hpp:84-102`):
```cpp
template <class T, int Nvar>
static ctaylor<T, Nvar> expm1(const ctaylor<T, Nvar> & t) {
  T tmp[Nvar + 1];
  exp_expand<T, Nvar>(tmp, t.c[0]);
  if (fabs(t.c[0]) > 1e-3)
    tmp[0] -= 1;
  else
    tmp[0] = 2 * exp(t.c[0] / 2) * sinh(t.c[0] / 2);
  ctaylor<T, Nvar> res;
  ctaylor_rec<T, Nvar>::compose(res.c, t.c, tmp);
  return res;
}
```

**Rust port target** (`crates/xcfun-ad/src/math.rs` after D-05 lands):
```rust
use crate::expand::expm1::expm1_expand;

#[cube]
pub fn ctaylor_expm1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);
    expm1_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}
```

**And `expm1_expand`** (`crates/xcfun-ad/src/expand/expm1.rs` new file):
```rust
#[cube]
pub fn expm1_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // Start with exp(x0) in t[0] + cumulative factorial for t[1..]
    let mut ifac = F::new(1.0);
    t[0] = x0.exp();
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        ifac *= i_f;
        t[k] = t[0] / ifac;
    }
    // Stable-bracket correction for t[0] only (higher coeffs unchanged).
    //   C++ ctaylor_math.hpp:93-97:
    //     if (fabs(t.c[0]) > 1e-3)  tmp[0] -= 1
    //     else                       tmp[0] = 2 * exp(x0/2) * sinh(x0/2)
    let threshold = F::cast_from(1e-3_f64);
    let abs_x0 = x0.abs();
    if abs_x0 > threshold {
        t[0] = t[0] - F::new(1.0);
    } else {
        let half = x0 / F::new(2.0);
        t[0] = F::new(2.0) * half.exp() * half.sinh();
    }
}
```
*Caveat (MEDIUM confidence):* cubecl 0.10-pre.3's `Float` trait surface MUST support `.abs()`, `.exp()`, `.sinh()` — verify via Phase 1's `expand/` modules which use `.exp()` (`exp.rs:38`). If `.sinh()` is absent, replace with `(0.5 * half.exp()) - (0.5 * (-half).exp())` (identity sinh(x) = (e^x - e^-x)/2).

### Example 2: `ctaylor_sqrtx_asinh_sqrtx` port (D-06)

**C++ source** (`ctaylor_math.hpp:275-325`): already shown above — branches on `|t.c[0]| < 0.5` between [8,8] Padé and unstable-form `sqrt(t)*asinh(sqrt(t))`.

**Port strategy:**
1. **Unstable form** (`|x0| ≥ 0.5`): trivial — `ctaylor_sqrt(x) → s`; `ctaylor_asinh(s) → asinh_s`; `ctaylor_mul(s, asinh_s, out)`.
2. **Stable Padé form** (`|x0| < 0.5`): port `tfuns::shift` (Phase 1 `tfuns.rs` — already shipped for `sqrtx_asinh_sqrtx` itself? **verify**). Then: shift P[9] and Q[9] by `x0`; `inv_expand(tmp, pq[0])` on shifted Q; compose tmp with shifted Q; `multo(tmp, shifted_P)`; `ctaylor_compose(out, x, tmp)`.

**Recommendation:** Port in two sub-tasks inside Wave 0 — first the easy unstable-form sub-case (covers `grad² > 0.25 / S2_prefactor ≈ 51` points), then the Padé stable form. Gate each sub-task with 100-point fixture vs mpmath; once both gate-pass, assemble the outer branch.

### Example 3: Mode::Potential GGA divergence — line-for-line port (D-13)

**C++ source** (`XCFunctional.cpp:671-792`): two paths — `XC_A_2ND_TAYLOR|XC_N_2ND_TAYLOR` (nspin=1, 1 divergence block) and `XC_A_B_2ND_TAYLOR|XC_N_S_2ND_TAYLOR` (nspin=2, 2 blocks each with 3 directional launches).

**Algorithm (nspin=1 path, 3 directional launches sum into `out[VAR0|VAR1]`):**

```
# Input layout (inlen=10, 2nd Taylor coefficients):
#   [n, gx, gy, gz, nxx, nxy, nxz, nyy, nyz, nzz]
#   [0, 1,  2,  3,  4,    5,   6,   7,   8,   9  ]

Phase 1 (LDA part — already computed via Mode::Potential LDA path with N=1, VAR0 seeding):
  out[0] = energy
  out[1] = ∂E/∂n   (LDA-scalar part)

Phase 2 (GGA correction — this loop):
  Declare ttype = CTaylor<F, 2>
  Accumulator `out_accumulated`: 1 CTaylor<F,2> (reset to zero before loop)

  # d/dx:
  # Seed in[0], in[1], in[2], in[3] with CTaylor values;
  #   in[0] gets VAR0=input[1]  (∂n/∂x)
  #   in[1] gets VAR0=input[4]  (∂gx/∂x = ∂²n/∂x²)
  #   in[2] gets VAR0=input[5]  (∂gy/∂x = ∂²n/∂x∂y)
  #   in[3] gets VAR0=input[6]  (∂gz/∂x = ∂²n/∂x∂z)
  # in[4..=9] = 0.
  # in[1].set(VAR1, 1)  — mark gradient_x as the differentiation direction
  Build densvars<ttype2> d
  out_accumulated += fp2(d) (weighted sum of active functionals)

  # d/dy: analogous, consumes input[2] + input[5,7,8], seeds VAR1 on slot 2
  # d/dz: analogous, consumes input[3] + input[6,8,9], seeds VAR1 on slot 3
  ... (2 more launches)

Phase 3 (subtract divergence):
  output[1] -= out_accumulated.get(VAR0 | VAR1)
```

**Rust port shape** (`crates/xcfun-eval/src/functionals/potential.rs` new file, `potential_gga_kernel`):

```rust
#[cube]
pub fn potential_gga_kernel_dir<F: Float>(
    input: &Array<F>,
    d: &mut DensVarsDev<F>,
    out_acc: &mut Array<F>,   // length 1<<2 = 4; accumulator across 3 calls
    #[comptime] dir: u32,     // 0=x, 1=y, 2=z
    #[comptime] vars: u32,
    #[comptime] id: u32,
) {
    // Pack input into DensVarsDev's pre-seeded flat input buffer per the
    // direction-specific offset table. Then call build_densvars::<F>(...).
    //
    // For d/dx (dir=0):
    //   in[0] = CTaylor(input[0], VAR0 -> input[1])    (4 slots)
    //   in[1] = CTaylor(input[1], VAR0 -> input[4], VAR1 -> 1)
    //   in[2] = CTaylor(input[2], VAR0 -> input[5])
    //   in[3] = CTaylor(input[3], VAR0 -> input[6])
    //
    // For d/dy (dir=1): offsets {2, 5, 7, 8} + VAR1 on slot [2]
    // For d/dz (dir=2): offsets {3, 6, 8, 9} + VAR1 on slot [3]

    // ... build densvars + dispatch_kernel::<F>(id, d, local_out, 2)
    // ... ctaylor_add::<F>(out_acc, local_out, out_acc, 2)
}
```

**Host-side** `Functional::eval` (extend Phase 2 pattern):
```rust
Mode::Potential => {
    // 1. Reject metaGGA (D-13).
    if self.depends_on(Dependency::LAPLACIAN | Dependency::KINETIC) {
        return Err(XcError::InvalidMode { .. });
    }
    // 2. Reject non-_2ND_TAYLOR vars with XC_GRADIENT deps.
    if self.depends_on(Dependency::GRADIENT) && !matches!(self.vars,
        Vars::A_2ND_TAYLOR | Vars::A_B_2ND_TAYLOR
        | Vars::N_2ND_TAYLOR | Vars::N_S_2ND_TAYLOR) {
        return Err(XcError::InvalidVars { .. });
    }
    // 3. Run LDA path (N=1 launch, VAR0 seeding) → output[0] = energy, output[1..] = potential.
    // 4. If fun.depends_on(GRADIENT): run GGA 3-direction accumulation; subtract from output[1] (and [2] for spin).
    // 5. Output-length check: matches D-15.
}
```

Verification: every C++ line-number comment in the Rust port cites the corresponding `XCFunctional.cpp:NNN` line. Plan-checker inspects.

---

## File-by-File Mapping

Authoritative port-target table for the pattern-mapper sub-agent. Columns:
- **FunctionalId** — Rust enum variant / xcfun-core id
- **C++ source** — `.cpp` file (line range for FUNCTIONAL body)
- **C++ shared** — `.hpp` imports (port target in `gga/shared/*.rs`)
- **xcfun-ad ops** — composed ops used (D-05/D-06 additions flagged)
- **Vars arm** — which `DensVarsDev` Vars arm it uses (MUST be in D-10)
- **test_in present?** — whether FUNCTIONAL macro has test_in/test_out block
- **Recommended Rust file** — kernel body location
- **Closest LDA analog** — for pattern-mapper sub-agent reference

### PBE Family (GGA-01) — 12 functionals

| ID | C++ src | C++ shared | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|------------|--------------|------|-------|-----------|------------|
| XC_PBEX (5) | `pbex.cpp:20-50` | `pbex.hpp`, `pw9xx.hpp` | pow, scalar_mul | XC_A_B_GAA_GAB_GBB | YES (#ifdef XCFUN_REF_PBEX_MU) | `gga/pbe/pbex.rs` | slaterx.rs |
| XC_PBEC (4) | `pbec.cpp:40-47` | `pw92eps.hpp` (Phase-2), constants.hpp | **ctaylor_expm1 (D-05)**, log, pow, sqrt, mul | XC_A_B_GAA_GAB_GBB | YES | `gga/pbe/pbec.rs` | pw92c.rs (uses pw92eps via new cross-import) |
| XC_REVPBEX (19) | `revpbex.cpp:18-21` | `pbex.hpp` | pow, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/revpbex.rs` | slaterx.rs |
| XC_RPBEX (20) | `rpbex.cpp:18-23` | `pbex.hpp` (for enhancement_RPBE) | **ctaylor_expm1 (D-05)**, pow, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/rpbex.rs` | slaterx.rs |
| XC_PBESOLX (74) | `pbesolx.cpp:18-39` | `pw9xx.hpp` (S2) | pow, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/pbesolx.rs` | slaterx.rs |
| XC_PBEINTX (72) | `pbeintx.cpp:18-41` | `pw9xx.hpp` (S2) | pow, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/pbeintx.rs` | slaterx.rs |
| XC_PBEINTC (71) | `pbeintc.cpp:18-38` | `pw92eps.hpp` | **ctaylor_expm1 (D-05)**, log, pow | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/pbeintc.rs` | pw92c.rs |
| XC_SPBEC (21) | `spbec.cpp:21-45` | `vwn.hpp` (uses vwn5_eps) | **ctaylor_expm1 (D-05)**, log, pow, cbrt | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/spbec.rs` | vwn5c.rs |
| XC_PBELOCC (73) | `pbelocc.cpp:19-41` | `pw92eps.hpp` | **ctaylor_expm1 (D-05)**, log, pow, exp | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/pbelocc.rs` | pw92c.rs |
| XC_ZVPBESOLC (69) | `zvpbesolc.cpp:19-93` | `pw92eps.hpp` | **ctaylor_expm1**, log, pow, exp, ctaylor_powi_3 | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/zvpbesolc.rs` | pw92c.rs |
| XC_ZVPBEINTC (76) | `zvpbeint.cpp:19-55` | `pw92eps.hpp` | **ctaylor_expm1**, log, pow, exp | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/zvpbeintc.rs` | pw92c.rs |
| XC_VWN_PBEC (22) | `pbec.cpp:49-56 + FUNCTIONAL at :82-92` | `vwn.hpp`, `pw92eps.hpp` | **ctaylor_expm1 (D-05)**, log, pow | XC_A_B_GAA_GAB_GBB | NO | `gga/pbe/vwn_pbec.rs` | vwn5c.rs + pbec pattern |

**Shared helpers (`gga/shared/pbex.rs`):** `enhancement(R, rho, grad)`, `enhancement_RPBE(rho, grad)`, `energy_pbe_ab(R, rho, grad)`.

**Shared helpers (`gga/shared/pbec_eps.rs`):** `A_expm1(eps, u3)`, `H(d2, eps, u3)`, `phi(d)`.

### Becke Family (GGA-02) — 4 functionals

| ID | C++ src | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|--------------|------|-------|-----------|------------|
| XC_BECKEX (6) | `beckex.cpp:17-25, 75-77` | **ctaylor_sqrtx_asinh_sqrtx (D-06)**, pow | XC_A_B_GAA_GAB_GBB | YES | `gga/becke/beckex.rs` | slaterx.rs |
| XC_BECKECORRX (7) | `beckex.cpp:27-32, 79-81` | **ctaylor_sqrtx_asinh_sqrtx (D-06)**, pow | XC_A_B_GAA_GAB_GBB | YES | `gga/becke/beckecorrx.rs` | slaterx.rs |
| XC_BECKESRX (8) | `beckex.cpp:42-54, 83-86` | **ctaylor_sqrtx_asinh_sqrtx + ctaylor_erf + ctaylor_expm1 + ctaylor_sqrt + pow** | XC_A_B_GAA_GAB_GBB | NO | `gga/becke/beckesrx.rs` | ldaerfx.rs (erf pattern) |
| XC_BECKECAMX (9) | `beckex.cpp:56-73, 88-94` | **sqrtx_asinh_sqrtx + erf + expm1 + sqrt + pow** | XC_A_B_GAA_GAB_GBB | NO | `gga/becke/beckecamx.rs` | ldaerfx.rs |

**BECKESRX / BECKECAMX parameter reads:** `d.get_param(XC_RANGESEP_MU)`, `d.get_param(XC_CAM_ALPHA)`, `d.get_param(XC_CAM_BETA)`. **DensVarsDev must expose `parent: &XCFunctional`-equivalent.** Phase 2 `functional.rs` has `weights: &'static [(FunctionalId, f64)]` but no parameter-setting surface — **planner MUST design the parameter-read path** (Mode::Potential needs it too for range-separated functionals). Options: (a) add `parameters: [f64; 4]` field to `Functional`, make it a field of `DensVarsDev` via the launcher; (b) pass parameters as additional kernel args. **Recommendation: option (a)** — lets us avoid a 4-scalar scratch allocation per launch.

### Becke–Roussel (GGA-03) — 3 functionals — **⚠️ METAGGA-CLASS DEPS**

| ID | C++ src | Dependencies | Vars | Status |
|----|---------|--------------|------|--------|
| XC_BRX (10) | `brx.cpp:103-106, 138-143` | DENSITY \| GRADIENT \| **KINETIC \| LAPLACIAN \| JP** | XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB | **DEFER TO PHASE 4** |
| XC_BRC (11) | `brx.cpp:108-121, 145-150` | DENSITY \| GRADIENT \| **KINETIC \| LAPLACIAN \| JP** | XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB | **DEFER TO PHASE 4** |
| XC_BRXC (12) | `brx.cpp:123-136, 152-157` | DENSITY \| GRADIENT \| **KINETIC \| LAPLACIAN \| JP** | XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB | **DEFER TO PHASE 4** |

**⚠️ PLANNER AMENDMENT REQUIRED:**
1. BRX/BRC/BRXC read `d.lapa`, `d.taua`, `d.jpaa` — these are ALL metaGGA fields. Putting them in Phase 3 forces DensVarsDev to populate `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` (inlen=11, Vars discriminant 17).
2. BR inversion requires porting `taylor.hpp::BR_taylor<T, Ndeg>` — a SEPARATE Taylor algebra (dense polynomial vs bit-flag multilinear). This is a new xcfun-ad module, not a primitive.
3. No xcfun-core amendment has been requested for these Vars arms in the 7-arm D-10 set.

**Recommendation:** Amend CONTEXT.md §Deferred to add BRX/BRC/BRXC. GGA-03 pass-through on REQUIREMENTS becomes a Phase-4 requirement. Alternative: expand D-10 to include the JPAA/JPBB arm and port `BR_taylor` as part of Wave 0.

### LYP (GGA-04) — 1 functional

| ID | C++ src | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|--------------|------|-------|-----------|------------|
| XC_LYPC (16) | `lypc.cpp:18-39` | exp, pow, reciprocal | XC_A_B_GAA_GAB_GBB | YES | `gga/lyp.rs` | vwn5c.rs |

No shared helpers. LYP is self-contained.

### OPTX (GGA-05) — 2 functionals

| ID | C++ src | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|--------------|------|-------|-----------|------------|
| XC_OPTX (17) | `optx.cpp:18-26` | pow, reciprocal | XC_A_B_GAA_GAB_GBB | NO | `gga/optx/optx.rs` | slaterx.rs |
| XC_OPTXCORR (18) | `optxcorr.cpp:18-33` | pow, reciprocal | XC_A_B_GAA_GAB_GBB | NO | `gga/optx/optxcorr.rs` | slaterx.rs |

**Shared helper (`gga/shared/optx.rs`):** `g_xa2(rho, grad) = gamma * grad * pow(rho, -8/3)` + enhancement `pow(gx2,2) / pow(1+gx2,2)`.

### PW86/PW91 (GGA-06) — 4 functionals

| ID | C++ src | C++ shared | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|------------|--------------|------|-------|-----------|------------|
| XC_PW86X (1) | `pw86x.cpp:17-32` | — | pow (heavy) | XC_A_B_GAA_GAB_GBB | NO | `gga/pw91/pw86x.rs` | slaterx.rs |
| XC_PW91X (26) | `pw91x.cpp:18-24` | `pw9xx.hpp` (prefactor, pw91xk_enhancement) | **ctaylor_sqrtx_asinh_sqrtx (D-06)**, exp, pow | XC_A_B_GAA_GAB_GBB | YES | `gga/pw91/pw91x.rs` | slaterx.rs |
| XC_PW91C (77) | `pw91c.cpp:39-87` | constants.hpp | **ctaylor_expm1 (D-05)**, log, pow, sqrt, exp, cbrt, scalar_mul — **THE LONGEST BODY** | XC_A_B_GAA_GAB_GBB | YES | `gga/pw91/pw91c.rs` | pw92c.rs (shares algebra structure) |
| XC_PW91K (27) | `pw91k.cpp:21-28` | `pw9xx.hpp` (pw91k_prefactor, pw91xk_enhancement) | **ctaylor_sqrtx_asinh_sqrtx**, exp, pow | XC_A_B_GAA_GAB_GBB | NO | `gga/pw91/pw91k.rs` | tw.rs (kinetic analog) |

**Shared helper (`gga/shared/pw91_like.rs`):** `chi2`, `S2`, `prefactor`, `pw91k_prefactor`, `pw91xk_enhancement` — ported verbatim from `pw9xx.hpp`.

### P86 (GGA-07) — 2 functionals

| ID | C++ src | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|--------------|------|-------|-----------|------------|
| XC_P86C (56) | `p86c.cpp:45-48, 54-110` | pz81eps (from Phase 2 lda/pz81c.rs), exp, sqrt, pow, cbrt | XC_A_B_GAA_GAB_GBB | YES | `gga/p86/p86c.rs` | pz81c.rs |
| XC_P86CORRC (57) | `p86c.cpp:50-52, 112-119` | Same | XC_A_B_GAA_GAB_GBB | NO | `gga/p86/p86corrc.rs` | pz81c.rs |

Both in same C++ file. `DBL_EPSILON` fudge (`p86c.cpp:30`) → `F::cast_from(f64::EPSILON)`.

### APBE (GGA-08) — 2 functionals

| ID | C++ src | C++ shared | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|------------|--------------|------|-------|-----------|------------|
| XC_APBEX (68) | `apbex.cpp:18-38` | `pw9xx.hpp` (S2) | pow, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/apbe/apbex.rs` | slaterx.rs |
| XC_APBEC (67) | `apbec.cpp:19-38` | `pw92eps.hpp` | **ctaylor_expm1 (D-05)**, log, pow | XC_A_B_GAA_GAB_GBB | NO | `gga/apbe/apbec.rs` | pw92c.rs |

### B97 (GGA-09) — 6 functionals

| ID | C++ src | xcfun-ad ops | Vars | test? | Rust file | LDA analog |
|----|---------|--------------|------|-------|-----------|------------|
| XC_B97X (60) | `b97xc.cpp:20-23, 36-43` | pow, reciprocal, scalar_mul | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97x.rs` | slaterx.rs |
| XC_B97C (61) | `b97xc.cpp:25-34, 45-52` | pw92eps, pow, reciprocal | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97c.rs` | pw92c.rs |
| XC_B97_1X (62) | `b97-1xc.cpp:20-23, 36-43` | — (coef swap only) | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97_1x.rs` | slaterx.rs |
| XC_B97_1C (63) | `b97-1xc.cpp:25-34, 44-50` | pw92eps, pow, reciprocal | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97_1c.rs` | pw92c.rs |
| XC_B97_2X (64) | `b97-2xc.cpp:20-23, 36-43` | — | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97_2x.rs` | slaterx.rs |
| XC_B97_2C (65) | `b97-2xc.cpp:25-34, 45-52` | pw92eps, pow, reciprocal | XC_A_B_GAA_GAB_GBB | NO | `gga/b97/b97_2c.rs` | pw92c.rs |

**Shared helper (`gga/shared/b97_poly.rs`):**
- `spin_dens_gradient_ab2(gaa, a_43) = gaa / (a_43 · a_43)` (abs() → identity at positive densities per post-regularize invariant)
- `ux_ab(Γ, s²) = Γ·s² / (1 + Γ·s²)`
- `enhancement(Γ, [c0, c1, c2], s²) = c0 + c1·ux + c2·ux²`
- `energy_b97x_ab(Γ, c_params, a_43, gaa) = e_x_LSDA_ab(a_43) · enhancement(Γ, c, ...)` — where `e_x_LSDA_ab = -PREFACTOR · a_43` with `PREFACTOR_F64 = 0.9305257363491002`
- `energy_b97c_par(Γ, c, a, a_43, gaa, &mut e_LSDA)` — returns e_LSDA by out-ref
- `energy_b97c_antipar(Γ, c, d, e_LSDA_a, e_LSDA_b)` — uses pw92eps::pw92eps(d)

**Shared constants (`gga/shared/constants.rs`):**
```rust
pub const B97_GAMMA_X_F64: f64 = 0.004_f64;
pub const B97_GAMMA_C_PAR_F64: f64 = 0.2_f64;
pub const B97_GAMMA_C_ANTIPAR_F64: f64 = 0.006_f64;
pub const B97_X_COEF: [f64; 3]    = [0.8094_f64, 0.5073_f64, 0.7481_f64];
pub const B97_1X_COEF: [f64; 3]   = [0.789518_f64, 0.573805_f64, 0.660975_f64];
pub const B97_2X_COEF: [f64; 3]   = [0.827642_f64, 0.047840_f64, 1.76125_f64];
pub const B97_C_PAR_COEF: [f64; 3]     = [0.1737_f64, 2.3487_f64, -2.4868_f64];
pub const B97_C_ANTIPAR_COEF: [f64; 3] = [0.9454_f64, 0.7471_f64, -4.5961_f64];
pub const B97_1C_PAR_COEF: [f64; 3]     = [0.0820011_f64, 2.71681_f64, -2.87103_f64];
pub const B97_1C_ANTIPAR_COEF: [f64; 3] = [0.955689_f64, 0.788552_f64, -5.47869_f64];
pub const B97_2C_PAR_COEF: [f64; 3]     = [0.585808_f64, -0.691682_f64, 0.394796_f64];
pub const B97_2C_ANTIPAR_COEF: [f64; 3] = [0.999849_f64, 1.40626_f64, -7.44060_f64];
```

### KT/BTK/CSC (GGA-10 minus LB94) — 3 functionals

| ID | C++ src | Deps | Vars | test? | Status |
|----|---------|------|------|-------|--------|
| XC_KTX (23) | `ktx.cpp:18-24` | DENSITY \| GRADIENT | XC_A_B_GAA_GAB_GBB | YES | Rust file `gga/kt/ktx.rs` — LDA analog slaterx.rs — TRIVIAL (pure pow + reciprocal) |
| XC_BTK (58) | `btk.cpp:17-27` | DENSITY \| GRADIENT | XC_A_B_GAA_GAB_GBB | NO | Rust file `gga/kt/btk.rs` — LDA analog tfk.rs — TRIVIAL (pure pow) |
| XC_CSC (66) | `cs.cpp:17-27` | DENSITY \| GRADIENT \| **KINETIC \| LAPLACIAN \| JP** | **XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB** (inlen=11) | NO | **⚠️ DEFER TO PHASE 4** |

**⚠️ PLANNER AMENDMENT:** CSC has same metaGGA-class dependencies as BRX/BRC/BRXC. Defer to Phase 4 OR expand D-10.

### Summary: Revised Phase 3 scope (MY RECOMMENDATION)

**If planner accepts BR+CSC deferral amendment:**
- Phase 3 ships **36 functionals** (12 PBE + 4 Becke + 1 LYP + 2 OPTX + 4 PW86/91 + 2 P86 + 2 APBE + 6 B97 + 2 KT/BTK — CSC deferred — BR×3 deferred)
- Wave 1 = 17 (12 PBE + 4 Becke + 1 LYP)
- Wave 2 = 10 (2 OPTX + 4 PW86/91 + 2 P86 + 2 APBE)
- Wave 3 = 8 (6 B97 + 2 KT/BTK)
- Wave 4 = Mode::Potential (all 47 functionals: 11 LDA + 36 GGA)
- Wave 5 = orders 3..=4 full matrix

**If planner DOES NOT defer:** 40 functionals as CONTEXT states, but Wave 0 must also port `taylor.hpp::BR_taylor` + expand DensVarsDev to support `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` (inlen=11, Vars=17). This is a 2–3× Wave-0 scope increase.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase-1 cubecl `Float::erf` polyfill (~1.3e-8 ULP) | In-kernel `erf_precise` libm port (Phase 2 commit `dca382a`) | 2026-04-21 | D-18 applies to Phase 3 — BECKESRX/BECKECAMX inherit 1e-12 capability |
| Phase-1 polyfill `cbrt` (1e-7 budget) | Same (unchanged) | — | PW91C + P86C still use this; Phase 3 tier-2 must accept the documented 1e-7 budget for cbrt-heavy bodies (**test and escalate if breaks 1e-12**) |
| Separate `exp(x) - 1` hand-rolled | `ctaylor_expm1` (D-05, this phase) | 2026-04-24 (CONTEXT locked) | Enables 9 PBE-family GGAs |
| `sqrt(x)*asinh(sqrt(x))` composition (non-diff at x=0) | `ctaylor_sqrtx_asinh_sqrtx` Padé-branched (D-06, this phase) | 2026-04-24 | Enables 6 Becke + PW91 family GGAs |

**Deprecated / outdated:** none. Phase 3 is purely additive.

---

## Assumptions Log

Every claim tagged `[ASSUMED]` in this research needs user confirmation before discussion-phase / planner locks it.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | cubecl 0.10-pre.3 `Float` trait supports `.sinh()` | Example 1 (`expm1_expand`) | If false, fall back to sinh identity `(e^x - e^-x)/2` — one extra kernel step |
| A2 | The `parameters: [f64; 4]` design for BECKESRX/BECKECAMX parameter reads is optimal (vs kernel args) | §File-by-File → Becke | Low — either approach works, planner picks |
| A3 | Combined monomorphisation count ≈ 2295 is approximate | Pitfall G10 | Medium — if actual count is 3×, compile time may balloon; planner should profile Wave 0 |
| A4 | Mode::Potential LDA path reuses the N=1 launch from `functional.rs` orders-1 arm unchanged (just with VAR0 seeded alone, not VAR0+VAR1) | Example 3 | Low — straightforward adaptation |
| A5 | The B97 `abs(gaa)` call in `spin_dens_gradient_ab2` is a no-op after regularize — `gaa ≥ 0` is invariant by construction (`gaa = ∇ρ·∇ρ`) | §File-by-File → B97 | Very low — mathematically guaranteed |
| A6 | The VWN3C/VWN5C/PW92C/PZ81C near-clamp drift (ACC-04 Partial) is likely to be incidentally tightened by Phase-3 build_densvars redesign | Intro, D-20 rationale | Medium — optimistic assumption; Wave 5 re-run may reveal no change, in which case D-20 falls through to Phase-6 forwarding |
| A7 | The `S2_prefactor = 1/(4·(3π²)^(2/3))` precomputation is numerically equivalent to the upstream `pow(pow(6, 2/3)/(12·pow(PI, 2/3)), 2.0)` at f64 precision | §Example 2 | Low — algebraic identity; fixture-gate will verify |
| A8 | CSC + BRX/BRC/BRXC deferral to Phase 4 is the right call | §File-by-File (BR, CSC) | **High** — if planner chooses NOT to defer, Wave 0 balloons. But the research evidence (XC_KINETIC dependency declared in the FUNCTIONAL macros) supports the deferral objectively. |
| A9 | The `_2ND_TAYLOR` Vars discriminants are 27..30 (per current `crates/xcfun-core/src/enums.rs:73-76`), not 26..29 as CONTEXT D-10 states | §User Constraints D-10 | Low — easily verified by `grep`; planner must use the enum as source of truth |

**Assumptions that WILL NOT hold without verification:** A3 (compile-time), A6 (ACC-04 tightening), A8 (BR/CSC scope). Planner must probe all three during planning.

---

## Open Questions

1. **Parameter-reading API for BECKESRX/BECKECAMX/CAM family.**
   - What we know: Upstream reads `d.get_param(XC_RANGESEP_MU)` inline inside the C++ densvars.hpp functionals; `parent->settings[XC_RANGESEP_MU]` where `parent` is the XCFunctional struct.
   - What's unclear: Phase 2 `Functional` has no parameter-carrying field. Does Phase 3 extend Functional? Does DensVarsDev get a 4-scalar `parameters` field? Does `eval_point_kernel` take parameters as additional kernel args?
   - Recommendation: Add `parameters: [f64; 4]` field to `Functional` (RS-02 Phase 5 anticipates this); pack into `DensVarsDev` at launch time; kernel reads `d.parameters[0]` etc. Document in Wave 0 task design.

2. **cubecl 0.10-pre.3 comptime bool support for `#[cube] fn` branching.**
   - What we know: Phase 1 uses `if comptime!(exponent == 0) { ... }` in `ctaylor_powi` — runtime bool branching on comptime integers works.
   - What's unclear: can `expm1_expand` branch at runtime on `F::abs(x0) > F::cast_from(1e-3)` inside the same `#[cube]` fn? The `.abs()` comparison produces a runtime F bool.
   - Recommendation: Phase 1's `Float` trait almost certainly supports `.abs() > threshold` → runtime bool → `if` statement; verify at Wave 0 via a 5-line spike. If not supported, refactor `expm1_expand` to unconditionally compute both branches and select via arithmetic blending.

3. **`#[ifdef XCFUN_REF_PBEX_MU]` conditional test data in regen-registry.**
   - What we know: `pbex.cpp:33-49` wraps test data in `#ifdef XCFUN_REF_PBEX_MU`. Phase 2 precedent (`pw92c.cpp` legacy constants) ignored the ifdef.
   - What's unclear: does Phase 3's regen-registry invocation (with the extended scope) correctly handle the `#ifdef`?
   - Recommendation: Verify during Wave 0 substrate extension. If extractor picks up the test data incorrectly (e.g., emitting test_in as `None` instead of as the ifdef-vendored default), hand-patch the generated file until xtask is fixed.

4. **Compile-time monomorphisation blow-up in `run_launch`.**
   - What we know: Phase 2's `run_launch` has ~33 arms `(id, n)` for 11 LDAs at N∈{0,1,2}. Phase 3 adds 40 more IDs at 5 N values (0..=4) + 7 vars arms (up from 2) — roughly 40×5×9 = 1800 new arms.
   - What's unclear: whether cubecl 0.10-pre.3 handles 2000+ monomorphisations in one match. Peak memory during JIT may exceed the Phase-2 10-minute tier-2 runtime budget.
   - Recommendation: Split `run_launch` into per-Mode-per-order submatches. Profile Wave-0 baseline `cargo test -p xcfun-eval --features testing` vs Wave-2 final. Planner sets a hard warning if compile time > 2× Phase-2 baseline.

5. **Mode::PartialDerivatives order 3 and 4 output-layout length.**
   - What we know: `XCFunctional.cpp:562-588` (order 3 seeding loop) + `DOEVAL(4, _)` macro (order 4) produce output layouts per `taylorlen(inlen, order)` — `taylorlen(2, 4) = 15`, `taylorlen(5, 4) = 126`.
   - What's unclear: Whether the Phase-2 `taylorlen` function (in `xcfun-core`) was verified at orders 3/4 (Phase 2 only needed 0/1/2).
   - Recommendation: Add a unit test for `taylorlen(inlen, k)` at inlen∈{1,2,3,4,5,10,20} and k∈{0,1,2,3,4}. Verify against C++ `xcfun_output_length(fun)` values for each combination.

---

## Environment Availability

Phase 3 has no new external dependencies. Environment already probed in Phase 2 (cubecl-cpu works, cc works, xcfun-master vendored).

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| `cargo` (workspace build) | All kernel ports | ✓ | Rust 1.85 (per rust-toolchain.toml) | — |
| `cc` | validation/build.rs C++ compile | ✓ | ^1.2.60 per CLAUDE.md recommended bump | — |
| cubecl-cpu MLIR runtime | All kernel launches | ✓ (tested in Phase 1/2) | =0.10.0-pre.3 | — |
| GNU C++ (`g++`) | validation/ cc compile | ✓ (Phase 2 CI + local) | ≥ 9 | clang++ (cc-rs picks automatically) |
| xcfun-master vendored tree | validation + regen-registry + port sources | ✓ | commit content-hash pinned per D-18 (Phase 1) | — |
| `xtask regen-registry` | Extending FUNCTIONAL_DESCRIPTORS | ✓ | Plan 02-02 Wave-1A-2 (shipped) | — |
| `mpmath` (Python, 200-digit precision) | Wave 0 fixture-gate for expm1 + sqrtx_asinh_sqrtx ground truth | ⚠️ (was used at Phase 2 ad-hoc; not in CI) | Any recent (≥ 1.3) | Direct comparison against C++ upstream at N-specific orders |

**Missing dependencies:** none blocking. mpmath is "nice to have" for fixture-gate ground truth; Phase 2 didn't CI-automate it.

---

## Validation Architecture

`workflow.nyquist_validation: true` in `.planning/config.json`. Validation is enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | **Rust built-in `#[test]` + `cargo-nextest` (CI) + `approx` (assertions) + custom tier-2 `validation/` binary (cc-linked C++ reference)** |
| Config file | Cargo workspace (`Cargo.toml`); no pytest/jest config equivalent |
| Quick run command | `cargo test -p xcfun-eval --features testing --test self_tests` (tier-1, < 5 s gate) |
| Mid run command | `cargo nextest run -p xcfun-eval --features testing` (all xcfun-eval tests, parallel) |
| Full suite command | `cargo nextest run --workspace --all-features && cargo xtask validate --backend cpu --order 4 --filter gga` (tier-1 + tier-2 parity full matrix at orders 0..=4) |
| Phase gate command | `cargo nextest run --workspace --all-features && cargo xtask validate --backend cpu --order 4` (all 51 functionals tier-2 GREEN at 1e-12 strict, per-functional override for LDAERF D-24 / BECKESRX per D-18) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| **GGA-01** | 12 PBE-family kernels match C++ at 1e-12 | unit + tier-2 | `cargo test -p xcfun-eval --features testing --test self_tests pbex_`, tier-2 per functional | ❌ Wave 1 (new `gga/pbe/*.rs` files + self-test enumeration extended) |
| **GGA-02** | 4 Becke kernels match C++ at 1e-12 (D-18 strict) | unit + tier-2 | `cargo test ... beckex`, tier-2 | ❌ Wave 1 |
| **GGA-03** | 3 BR kernels (⚠️ recommend Phase-4 defer) | unit + tier-2 | n/a if deferred | ❌ Wave 1 (or deferred) |
| **GGA-04** | LYP | unit + tier-2 | `cargo test ... lypc`, tier-2 | ❌ Wave 1 |
| **GGA-05** | OPTX | unit + tier-2 | `cargo test ... optx`, tier-2 | ❌ Wave 2 |
| **GGA-06** | PW86/91 | unit + tier-2 | `cargo test ... pw91x`, tier-2 | ❌ Wave 2 |
| **GGA-07** | P86 | unit + tier-2 | `cargo test ... p86c`, tier-2 | ❌ Wave 2 |
| **GGA-08** | APBE | unit + tier-2 | `cargo test ... apbex`, tier-2 | ❌ Wave 2 |
| **GGA-09** | 6 B97 variants match C++ at 1e-12 (includes order-4 conditioning check) | unit + tier-2 | `cargo test ... b97_`, tier-2 | ❌ Wave 3 |
| **GGA-10** | 3 (KT/BTK — CSC deferred) | unit + tier-2 | `cargo test ... ktx`, tier-2 | ❌ Wave 3 |
| **MODE-01** | orders 3..=4 PartialDerivatives output layout matches C++ | tier-2 | `cargo xtask validate --backend cpu --order 4 --filter gga_` | ❌ Wave 5 |
| **MODE-02** | Mode::Potential GGA divergence construction matches C++ 1e-12 | tier-2 | `cargo xtask validate --backend cpu --mode potential --filter gga_` | ❌ Wave 4 (validation/ mode flag added) |
| **MODE-05** | `output_length` returns 2/3 for Potential | unit | `cargo test -p xcfun-eval output_length_potential` | ❌ Wave 4 (new unit test in `functional.rs`) |
| D-05 fixture-gate | `ctaylor_expm1` matches mpmath at 1e-14 (orders 0..=4) | unit (fixture-compare) | `cargo test -p xcfun-ad golden_expand::test_expm1` | ❌ Wave 0 (new golden fixture) |
| D-06 fixture-gate | `ctaylor_sqrtx_asinh_sqrtx` matches mpmath at 1e-14 | unit (fixture-compare) | `cargo test -p xcfun-ad golden_composed::test_sqrtx_asinh_sqrtx` | ❌ Wave 0 |

### Input Space Coverage

For each GGA family, Phase 3 coverage matrix:

**Bulk grid (7000 pts):** Existing Phase 2 stratum — (a, b) uniform in `[1e-3, 100]`², gradients uniform in `[0, (ρ^(4/3))²]`. Sufficient for all GGA bulk regions.

**Regularize-clamp (1000 pts, D-22 excluded):** Exercises the `min(a,b) ≤ 2e-14` path; Mode::Potential `_2ND_TAYLOR` arms on these grid points test the "density goes to clamp" limit — covered transparently.

**Polarised (1000 pts):** `|zeta| → 1` stratum. Critical for PBEC `phi(d) = (1/∛2) * n^(-2/3) * (a_43^(1/2) + b_43^(1/2))` at high polarisation. Existing.

**Gradient-zero (1000 pts):** `|∇ρ|² ∈ [1, 1e6]` → `|∇ρ|² → 0` via log-uniform with bottom at 0. **Key for GGAs** — tests the `chi² → 0` / `S² → 0` limit where PW91 Padé-approximant branches. Phase 2 already populated this stratum per Plan 02-06 D-23.

**Additional strata for Phase 3 (PLANNER TO DESIGN):**
- **erf-argument low (100 pts):** BECKESRX `a = μ·√K / (6·√π·ρ^(1/3))` — sample `a ∈ [0.1, 1.0]` where the cancellation is strongest.
- **B97 coefficient saturation (200 pts):** `ux = Γ·s² / (1 + Γ·s²) → 1` as `s² → ∞` — sample `s² ∈ [1e3, 1e6]`.
- **PBE-family R ratio (100 pts):** sample near the boundary where `enhancement(R, rho, grad)` crosses `F(S²) ≈ 1` for both R=0.804 and R=1.245 — tests R swap.

**Recommendation:** Extend `validation/src/fixtures.rs` Wave 5 with a new `gga_stratified_supplement(seed: u64) -> Vec<GridPoint>` adding 400 points (100+200+100) for the three new strata above, to augment the existing 10k grid without changing the seed 0x1234abcd for the 10k bulk.

### Boundary / Edge Cases

1. **Regularize-clamp strata (ρ → 0):** Density clamps to `XCFUN_TINY_DENSITY ≈ 1e-14`. Phase 2 Fix 2 excludes these from tier-2 verdict. Phase 3 extends the exclusion policy UNCHANGED.
2. **Gradient-zero limit (∇ρ → 0):** `sqrtx_asinh_sqrtx(chi²)` hits the Padé-branch boundary. GGA functionals must pass at 1e-12 here — this is the D-06 Wave-0 fixture-gate focus.
3. **High-density/low-gradient (PBE-family `S² → 0`):** `expm1(-eps / (γ·u³))` near `x=0` — D-05 stable-bracket domain. Fixture-gate must cover.
4. **erf-branch crossover (BECKESRX at `a ~ 1`):** Cancellation cascade peaks here. D-18 strict.
5. **Spin-zero (β=α, zeta=0):** `phi(d)` symmetry; `gns` vanishes exactly; tests DensVarsDev correctness.
6. **`_2ND_TAYLOR` slot 4..=9 population:** `lapa = 0.5*(d[4]+d[7]+d[9])` — off-by-one or wrong-slot errors in the arm implementation will show only on Mode::Potential, not PartialDerivatives. Unit tests in `tests/regularize_2nd_taylor.rs`.

### Property / Invariant Tests

Phase-3 specific invariants (augment Phase 1's AD-06 proptest suite):

1. **Spin-symmetry metamorphic test:** For every GGA, `energy(a, b, gaa, gab, gbb) == energy(b, a, gbb, gab, gaa)` (swap alpha ↔ beta). Proptest batch-per-property — 10k random inputs.
2. **Gradient-zero limit:** As `(gaa, gab, gbb) → (0, 0, 0)`, `E_GGA → E_LDA` (for same density). Proptest batch — tests PW91X, PBEX, Becke converge to Slater-like bodies at zero gradient.
3. **PBE-family enhancement monotonicity:** `enhancement(R_pbe, rho, grad) ≥ 1` for all valid inputs (by PBE construction: `F(S²) = 1 + R - R/(1 + μ·S²/R) ≥ 1`). Proptest invariant.
4. **B97 polynomial coefficient sum:** At `ux = 0`, `enhancement == c0`. Unit test per coefficient set.
5. **Mode::Potential consistency:** For LDA functionals (no GRADIENT), Mode::Potential output[1] = Mode::PartialDerivatives output[1] at order 1. Unit test on 11 LDAs (sanity check that the new path doesn't break Phase 2 LDA behaviour).
6. **Mode::Potential gauge invariance:** For any GGA, adding a divergence-free vector field to ∇ρ (e.g., circular gradient) leaves Mode::Potential `output[1] -= nabla · dE/dg` unchanged. Quick sanity check via finite-difference vs pure analytical.

### Metamorphic Tests

1. **Spin swap:** `(a, b) ↔ (b, a)` + `(gaa, gab, gbb) ↔ (gbb, gab, gaa)` — output[0] unchanged, output[1] ↔ output[2]. For all 40 GGAs.
2. **Gradient scaling:** `∇ρ → α·∇ρ` → `|∇ρ|² → α²·|∇ρ|²`. Functionals like PBEX are closed-form in `S²(ρ, grad)` — should produce predictable scaling.
3. **Density scaling:** `ρ → λρ` gives known scaling relations for Slater-like bodies (E_x → λ^(4/3) · E_x).
4. **Zero-spin limit (b = a):** `phi(d) = (2/2)^(1/3) · n^(-2/3) · 2·sqrt(a^(4/3)) = ...` — verify phi reaches 1 at equal-spin.

### Fixture Datasets

| Dataset | Size | Seed | Source | Phase |
|---------|------|------|--------|-------|
| **Phase 2 stratified 10k grid** | 10 000 pts × 5 scalars (A_B_GAA_GAB_GBB) | xoshiro256++ 0x1234abcd | `validation/src/fixtures.rs` | Phase 2 — UNCHANGED |
| **Phase 3 supplemental 400 pts** | 400 pts × 5 scalars (three new strata) | xoshiro256++ 0xdeadbeef | new in Phase 3 Wave 5 | Planner adds |
| **Phase 3 `_2ND_TAYLOR` fixtures** | 200 pts × 10 or 20 scalars (for Mode::Potential testing) | xoshiro256++ 0xfaceb00c | new in Phase 3 Wave 4 | Planner adds |
| **Wave-0 expm1 fixture** | 500 pts × (x0, n) pairs | xoshiro256++ 0xcafebabe | new in Phase 3 Wave 0 | Planner adds; golden = mpmath 200-digit |
| **Wave-0 sqrtx_asinh_sqrtx fixture** | 500 pts × (x, n) pairs | xoshiro256++ 0xb16b00b5 | new in Phase 3 Wave 0 | Planner adds; golden = mpmath |
| **Upstream test_in/test_out** | Per-functional (12 of 40 GGAs have upstream data — GGA-01/02/04/06/07/10) | — | `xtask regen-registry` extraction | Extended incrementally |

### Sampling Rates

- **Per task commit:** `cargo test -p xcfun-eval --features testing --test self_tests` (tier-1 sub-30-second gate). Filter by family once gated.
- **Per wave merge:** `cargo xtask validate --backend cpu --order 2 --filter <family>` (tier-2, 10–30 seconds per family wave).
- **Phase gate:** `cargo xtask validate --backend cpu --order 4` (full matrix; 60+ seconds with 51 functionals + 4 orders; accept 5-minute budget in CI per D-24).
- **Tier-1 order coverage per family:** 0, 1, 2 until the family's tier-2 passes; then bump to 3 and 4 in Wave 5. Don't run order 3/4 tier-1 until Wave 5.
- **Tier-2 order coverage per family:** 0, 1 in each wave's commit (quick); 2 at wave-end; 3 and 4 in Wave 5 only.

### Wave 0 Gaps

Before any family wave:
- [ ] `crates/xcfun-ad/src/expand/expm1.rs` — new; D-05 expand body.
- [ ] `crates/xcfun-ad/src/math.rs` — extend with `ctaylor_expm1` + `ctaylor_sqrtx_asinh_sqrtx`.
- [ ] `crates/xcfun-ad/tests/golden_expand.rs` — add expm1 test module.
- [ ] `crates/xcfun-ad/tests/golden_composed.rs` — add sqrtx_asinh_sqrtx test module.
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` — extend to emit 500 expm1 + 500 sqrtx_asinh_sqrtx fixtures (mpmath ground-truth).
- [ ] `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` + 6 new shared helper modules (pbex, pbec_eps, pw91_like, b97_poly, optx, constants) — D-08.
- [ ] `crates/xcfun-eval/src/density_vars/build.rs` — 7 new `build_xc_<variant>` functions (D-10).
- [ ] `crates/xcfun-eval/src/density_vars/build.rs` — 7 new comptime if-chain arms dispatching to the above.
- [ ] `crates/xcfun-eval/src/functional.rs` — 4 `_2ND_TAYLOR` vars match arms in `run_launch`; extend (id, n) match to handle N=1 for Mode::Potential LDA path.
- [ ] `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` — new test file per CONTEXT specifics.
- [ ] `crates/xcfun-eval/src/functionals/gga/mod.rs` — placeholder with `pub mod shared; pub mod pbe;` etc.
- [ ] (Potential amendment path) `crates/xcfun-ad/src/taylor.rs` + `crates/xcfun-ad/src/br_inverse.rs` — if BR/CSC NOT deferred, port `taylor.hpp` + `BR_taylor` (Newton-inverse). **Recommend deferring.**

If deferral accepted: 10 Wave-0 gaps total. If not: ~14 gaps, with roughly 2× Wave-0 scope.

---

## Security Domain

`security_enforcement` not explicitly set → default ON. Phase 3 security posture:

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Not applicable — library code, no auth surface |
| V3 Session Management | no | Not applicable |
| V4 Access Control | no | Not applicable |
| V5 Input Validation | yes | `Functional::eval` validates input length, order range, Mode/Vars compatibility. Phase 2 already enforces; Phase 3 extends to Mode::Potential arms per D-13 (`XcError::InvalidMode`, `XcError::InvalidVars`). No new input-validation surface beyond rejection paths. |
| V6 Cryptography | no | Not applicable — numerical library, no cryptographic operations |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| **Panic in kernel** | Denial-of-Service | cubecl 0.10-pre.3 rejects `assert!` inside `#[cube]` — Phase 1 uses host-side guards (precondition documentation). Phase 3 continues this: `regularize` in `build_densvars` clamps density to `XCFUN_TINY_DENSITY` before kernel sees it; all `sqrt`/`log`/`pow` preconditions met via regularize. |
| **Input buffer overflow** | Information Disclosure / Tampering | `Functional::eval` checks `input.len() == expected_inlen` + `output.len() == expected_outlen` before launch. Phase 2 pattern preserved; Phase 3 extends for Mode::Potential's `output.len() ∈ {2, 3}`. |
| **unsafe-code audit** | Tampering | `#[forbid(unsafe_code)]` in xcfun-core and xcfun-eval crate roots. The existing `unsafe` is ONLY around `launch_unchecked` in `functional.rs` — Phase 3 preserves this scoping. |
| **Stale generated code (c_stubs.cpp, FUNCTIONAL_DESCRIPTORS)** | Spoofing (via drift) | `xtask regen-registry --check` drift-gate per QG-07. Phase 3 extends sources. |
| **Integer overflow in monomorphisation match** | Denial-of-Service at compile time | 2000+ arm match may hit `rustc` match-arm limits. Mitigation: split match per-Mode as noted in Pitfall G10. |

---

## Sources

### Primary (HIGH confidence — in-tree or official)

- **`xcfun-master/src/functionals/*.cpp` and `*.hpp`** (authoritative algorithmic source) — read all 40 GGA source files + 6 shared .hpp files directly. Every algorithm in §File-by-File Mapping cites a specific line range.
- **`xcfun-master/src/XCFunctional.cpp:420-800`** — eval_setup + Mode::Potential body + Mode::PartialDerivatives layout (orders 0..=4). Line-for-line port target for D-13.
- **`xcfun-master/src/densvars.hpp:35-218`** — authoritative Vars arm definitions; port target for D-10.
- **`xcfun-master/external/upstream/taylor/ctaylor_math.hpp:85-102` (expm1), `:276-325` (sqrtx_asinh_sqrtx)** — D-05/D-06 port targets.
- **`xcfun-master/src/xcint.cpp:93-135`** — VARS_TABLE authoritative — 31 rows. Verified `_2ND_TAYLOR` rows 28..31 (0-indexed 27..30 — one-off CONTEXT.md).
- **`crates/xcfun-core/src/enums.rs:36-76`** — authoritative Rust Vars enum discriminants. Phase-3 references.
- **`crates/xcfun-eval/src/functionals/lda/{slaterx,pw92c,pz81c,vwn3c,vwn5c,tfk,tw,vwk,ldaerfx,ldaerfc,ldaerfc_jt}.rs`** — 11 LDA kernel template.
- **`crates/xcfun-eval/src/density_vars/build.rs`** — Phase 2 comptime if-chain pattern template.
- **`crates/xcfun-eval/src/dispatch.rs`** — Phase 2 11-arm dispatch pattern template.
- **`crates/xcfun-eval/src/functional.rs:170-484`** — Phase 2 launch_and_accumulate pattern template.
- **`crates/xcfun-ad/src/math.rs`** — Phase 1 composed-op pattern (ctaylor_exp, ctaylor_log, etc).
- **`crates/xcfun-ad/src/expand/*.rs`** — Phase 1 expand pattern (template for expm1_expand).
- **`validation/src/driver.rs` + `validation/build.rs`** — Phase 2 tier-2 harness (extension target).
- **`.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md`** — 25 Phase-2 locked decisions inherited.
- **`.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`** — 28 Phase-1 locked decisions inherited.
- **`.planning/STATE.md` (lines 100-135, 165-189)** — Phase-2 accumulated decisions + ACC-04 residual status.

### Secondary (MEDIUM confidence — configuration / workspace-scanned)

- `Cargo.toml` (workspace root) + per-crate Cargo.toml manifests — stack versions.
- `CLAUDE.md` lines 1–290 — pinned versions + constraints (authoritative for project, NOT cross-verified against training knowledge).
- `.planning/REQUIREMENTS.md` lines 47-64, 67-72, 136-142 — Phase 3 requirements + ACC-04 status.
- `.planning/ROADMAP.md:100-111` — Phase 3 success criteria.
- `xcfun-master/api/xcfun.h:86-121` — Vars enum xcfun-API canonical.

### Tertiary (LOW — not used for hard claims)

- No WebSearch / WebFetch used. All claims come from in-repo sources.

---

## Metadata

**Confidence breakdown:**

- **Standard stack:** HIGH — Phase 3 adds zero dependencies; all pins inherited from CLAUDE.md and are valid at 2026-04-24.
- **Architecture patterns:** HIGH — Phase 2 patterns directly reused; no novel constructs except the Mode::Potential dispatch (line-for-line port of concrete C++).
- **File-by-File Mapping:** HIGH for 37 functionals (pure GGA); MEDIUM for BRX/BRC/BRXC/CSC due to the metaGGA-dep finding requiring CONTEXT amendment.
- **Common pitfalls:** HIGH for G1 (expm1 threshold), G2 (Padé shift), G5 (R parameter swap), G7 (legacy constants), G8 (regularize invariant). MEDIUM for G3 (_2ND_TAYLOR C-fallthrough — implementation detail), G4 (BECKESRX cancellation — inherited from upstream comment), G6 (B97 order-4 — extrapolated from coefficient magnitudes), G9 (RPBEX shortest — algebraic), G10 (cubecl monomorphisation — extrapolation from Phase 2).
- **Assumptions Log:** HIGH-integrity — 9 assumptions explicitly called out, 3 flagged as needing probing (A3, A6, A8).

**Research date:** 2026-04-24
**Valid until:** 2026-05-24 (30 days — Phase 3 planning/execution should stay within this window; cubecl 0.10-pre.4 release or any xcfun-master vendor bump invalidates and requires re-research).

---

*Phase: 03-gga-tier-mode-potential*
*Researched: 2026-04-24 (standalone `/gsd-research-phase 3` — integrated with 25 locked decisions in 03-CONTEXT.md)*
*Inherits: 53 locked decisions (28 Phase-1 + 25 Phase-2)*
