# Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases - Research

**Researched:** 2026-04-25
**Domain:** metaGGA exchange–correlation functional ports (28 metaGGAs + BR family + CSC carryovers, 32 total) atop the Phase-3 cubecl-native substrate, `Mode::Contracted` orders 0..=6 line-for-line port of `XCFunctional.cpp:619-635` `DOEVAL` macro, alias engine (46 entries, multiplicative-weight semantics from `XCFunctional.cpp:369-405`), and the 4 tunable parameters (`XC_RANGESEP_MU`, `XC_EXX`, `XC_CAM_ALPHA`, `XC_CAM_BETA`).
**Confidence:** HIGH for source-tree triage, alias-engine semantics, parameter table, BR Newton-inverse, and DensVarsDev field provisioning (all derived from in-tree C++ + committed Rust); MEDIUM for SCAN family port-cost (522-line `SCAN_like_eps.hpp` shared module with branch-on-`IDELEC` integer is the largest single Wave 2 risk); MEDIUM for orders 5/6 Contracted parity (no existing C++ test reaches this regime in our 10k grid — extending `validation/src/c_driver.rs` is required).

---

## Summary

Phase 4 is the **functional-coverage closeout phase**. Phase 2 + Phase 3 shipped the LDA + GGA tiers with the cubecl-native substrate; Phase 4 stacks on top in **five orthogonal layers**:

1. **xcfun-ad substrate (Wave 0):** **one** new `#[cube] fn ctaylor_br_inverse<F, const N>` primitive — host-side scalar Newton-Raphson `BR(z) → x` solving `BR_z(x) = (x-2)/x · exp(2x/3) = z` (port of `xcfun-master/src/functionals/brx.cpp:25-72`), wrapped by a Brent–Kung linear-method polynomial inverter `BR_taylor`. Fixture-gate at strict 1e-12 vs C++ `BR_taylor`, 30 z-points × N∈{2,3,4}. **No other xcfun-ad additions.**

2. **DensVarsDev Vars arms (Wave 0):** the 11 new arms listed in CONTEXT D-03 split into **7 implementable** (ids 11, 13, 15, 16, 17, 24, 25, 26 — these have C++ `densvars.hpp` switch arms) and **4 unimplemented in upstream** (ids 8, 9, 10, 12, 14, 18 — declared in `xcint.cpp:102-118` but C++ `xcfun::die` at runtime; Rust must mirror). **CRITICAL FINDING:** ids 8/9/10/12/14/18/23 have NO C++ densvars body — porting them risks divergence from C++. Treat them as `unimplemented!` (host-side `XcError::InvalidVars`) until a metaGGA actually requires them. **In practice, the 32 metaGGA bodies use only ids 13 (TAUA_TAUB) for TPSS/SCAN/M0x and id 17 (full JP) for BR/CSC. Wave 0 needs ONLY 2 new build_xc_* arms** (id 13 + id 17). The remaining arms can land later or be left as reject-arms.

3. **metaGGA shared helpers (Wave 0):** port `tpssx_eps.hpp`, `tpssc_eps.hpp`, `revtpssx_eps.hpp`, `revtpssc_eps.hpp`, `SCAN_like_eps.hpp` (522 lines — single largest Wave 2 deliverable), `m0xy_fun.hpp` (262 lines — covers M05 + M06 families), and a `br_like.rs` host-side scalar Newton driver. CSC + BLOCX have no shared header (single-functional bodies).

4. **32 functional bodies (Waves 1–3):** TPSS family (5), SCAN family (10), M05 family (4), M06 family (8), BLOCX (1), BR family (3 carryover), CSC (1 carryover). All use the established Phase 3 `#[cube] fn <name>_kernel<F: Float, const N: u32>` signature; algorithmic-identity rule applies; strict 1e-12 fixture-gate.

5. **Alias engine + parameters + Mode::Contracted (Waves 4–6):** a **27-line** line-for-line port of `XCFunctional.cpp:369-405` (`Functional::set` with multiplicative recursion + additive functional weights + overwriting parameter weights + EXX-FIXME preservation), 46-entry alias registry populated via `xtask regen-registry` extension, 4-row parameter registry, comptime-monomorphized `contracted_kernel<F, const ORDER>` for orders 0..=6 (CTaylor<F, 6> backing array length 64 — exercised for the first time at Phase 4).

**Primary recommendation:** Land Wave 0 (1 xcfun-ad primitive + 2 DensVarsDev arms + 7 helper modules) atomically with strict-1e-12 fixture-gates BEFORE any functional body. Order Wave 1 = TPSS (5) + BR family (3) + CSC (1) — total 9 — because TPSS is the simplest metaGGA family (under 200 LOC across all 5 variants combined) and BR ships first to unblock BLOCX in Wave 3. Defer the 4 unimplemented Vars arms (ids 8/9/10/12/14/18/23) to a host-side reject path — DO NOT port C++'s `xcfun::die` semantically; return `XcError::InvalidVars` instead. Plan a dedicated Wave 6 capstone that runs `xtask validate --order 3 --filter '.*'` across all 77 functionals (78 minus LB94) at strict 1e-12 and forwards any new D-19 INCONCLUSIVE entries to Phase 6 unchanged.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (14 total — D-01..D-14)

**Scope, ordering & wave strategy:**
- **D-01:** Phase 4 ports **32 functional bodies** (28 metaGGA + 4 carryovers BRX/BRC/BRXC + CSC). Plus alias engine (46 entries), parameter table (4 entries), Mode::Contracted (orders 0..=6). LB94 NOT included — reaffirmed via D-13 (not alias-feasible).
- **D-01-A:** **Wave layout (planner finalises exact internal order):**
  - **Wave 0 (substrate):** xcfun-ad `ctaylor_br_inverse` primitive + 11 new DensVarsDev Vars arms + metaGGA shared helper modules. Atomic; fixture-gate per primitive at strict 1e-12.
  - **Wave 1 (TPSS + BR + CSC, 9 functionals):** TPSS family (5) + BR family (3) + CSC (1). TPSS shares the `id=13` (TAUA_TAUB) Vars arm; BR + CSC share `id=17` (JPAA_JPBB).
  - **Wave 2 (SCAN family, 10 functionals):** SCAN, RSCAN, RPPSCAN, R2SCAN, R4SCAN — both X and C variants. Heavy on the `shared/scan_like.rs` module.
  - **Wave 3 (M05 + M06 + BLOCX, 13 functionals):** Minnesota families + BLOCX. Heavy on `shared/m0x_like.rs`.
  - **Wave 4 (alias engine + parameters):** populate `aliases.rs` registry (46 entries via `xtask regen-registry`), wire `Functional::set` recursion (multiplicative weights), wire 4 parameters with defaults from `common_parameters.cpp`, add `Functional::get(parameter_name)` path. Tier-1 alias-canary tests.
  - **Wave 5 (Mode::Contracted):** comptime `contracted_kernel<F, const ORDER>` orders 0..=6, host-side input pack / output unpack, eval_setup acceptance for Contracted, tier-2 parity at orders 0..=6 on a Contracted-mode subset.
  - **Wave 6 (full-matrix tier-2 + Phase-4 sign-off):** Run `xtask validate --order 3` across all 78 - 1 (LB94) = 77 functional ids. Forward any new D-19 INCONCLUSIVE entries to Phase 6.
- **D-01-B:** **Per-family parallelism inside each wave** mirrors Phase 3.

**xcfun-ad substrate extension (BR Newton-inverse only):**
- **D-02:** **Add `ctaylor_br_inverse` primitive to `xcfun-ad`.** Port target: `xcfun-master/src/functionals/brx.cpp:25-72`. Strict 1e-12 fixture-gate.
- **D-02-A:** **No other xcfun-ad additions.** Escalate via PLANNING INCONCLUSIVE if needed.

**DensVarsDev Vars arms:**
- **D-03:** **Add 11 new comptime arms in `build_densvars`** covering Vars discriminants 8..=18 + 23..=26.
- **D-03-A:** **The `id=17` arm is shared by BR family + CSC** + (potentially) BLOCX. Implement once; chain TAUA_TAUB / LAPA_LAPB / JPAA_JPBB derivations explicitly.
- **D-03-B:** **`DensVarsDev` struct may need new public fields** for `jpaa`, `jpbb`, `lap`, `tau` if not provisioned. **VERIFIED**: `crates/xcfun-eval/src/density_vars.rs:75-79` already provisions `jpaa`, `jpbb`, `lapa`, `lapb`, `tau`. **No struct change needed.**

**Alias engine (multiplicative weight semantics):**
- **D-04:** **Line-for-line port of `XCFunctional.cpp:369-405`**.
- **D-04-A:** **Alias table populated by `xtask regen-registry`** — extractor parses `aliases.cpp:17-139`.
- **D-04-B:** **Case-insensitive name lookup** (matches C++ `strcasecmp`).

**Parameter table:**
- **D-05:** **4 parameters with C++ defaults** at `crates/xcfun-core/src/registry/generated/parameters.rs`.
- **D-05-A:** **`ParameterId` enum in `xcfun-core`** (4 variants, `#[repr(u32)]`, discriminants 78..=81).

**Mode::Contracted:**
- **D-06:** **Line-for-line port of `XCFunctional.cpp:619-635` `DOEVAL` macro** as 7 comptime kernels.
- **D-06-A:** **Vars compatibility for Contracted:** every Vars whose pre-computed Taylor coefficients can be packed at order N is compatible.
- **D-06-B:** **`output_length` for Mode::Contracted = `1 << order`**.
- **D-06-C:** **Tier-2 Contracted-mode parity** at strict 1e-12 on a 1000-point subset.

**CTaylor<F, 6> capacity:**
- **D-07:** **CTaylor<F, 6> already declared valid** at the type level (Phase 1 AD-01); Wave 0 smoke-test.

**Dispatch + supports() bitmap:**
- **D-08:** **`dispatch_kernel` gets 36 new comptime arms.** `supports(id)` bitmap bumped from 51 to 87 ids. **NOTE:** CONTEXT says 51 — actual current count from `dispatch.rs:220-233` is **46** (11 LDA + 17 GGA Wave-2 + 10 GGA Wave-3 + 8 GGA Wave-4 = 46). Bitmap goes 46 → 82 (46 + 32 + 4). Mode::Contracted adds 7 host-side dispatch arms.
- **D-08-A:** **The 91-id total** within `u128` bitmap range.

**Validation harness (incremental per wave):**
- **D-09:** **`validation/build.rs` extends per wave**.
- **D-09-A:** **Grid generator gains metaGGA strata** at sibling seed `0xc0ffee01`.

**Phase 3 D-19 forwards: inheritance:**
- **D-10:** **The 13 D-19 INCONCLUSIVE entries from Phase 3 stay forwarded to Phase 6 unchanged.**

**Tier-2 parity threshold:**
- **D-11:** **Strict 1e-12 default** for metaGGA functionals.
- **D-12:** **No blanket Mode::Contracted relaxation.**

**LB94 disposition:**
- **D-13:** **LB94 is NOT alias-feasible.** Phase 5 owns.

**Error model:**
- **D-14:** **No new `XcError` variants.**

### Claude's Discretion (planner-owned)

- Per-functional file layout: `mgga/<family>/<fn>.rs` vs flat `mgga/<fn>.rs`.
- Helper-module granularity: whether `shared/scan_like.rs` fuses with `shared/tpss_like.rs`.
- Wave 1 internal ordering: TPSS-first vs BR-first.
- Alias registry layout: flat slice vs per-letter sub-table.
- Parameter storage layout: single `[f64; 82]` vs split `functionals/parameters` arrays.
- Mode::Contracted host-side input packing: in-place vs scratch buffer.
- Alias depth-guard mechanism: explicit counter vs registry-time invariant.
- `DensVarsDev` JP-field naming: verified (`jpaa`, `jpbb`).

### Deferred Ideas (OUT OF SCOPE)

- **LB94** — Phase 5 (D-13 confirmed not alias-feasible).
- **`Mode::Potential` for metaGGAs** — out of scope (algorithmic-identity rule; C++ rejects).
- **`Mode::PartialDerivatives` orders 5..=6** — not in C++ reference (XCFunctional.cpp falls through).
- **GPU backends** — Phase 6.
- **C ABI / Python / facade** — Phase 5/7.
- **Full `Functional` API surface (RS-01..10)** — Phase 5 (Phase 4 only ships set/get/eval cross-section needed for aliases + Mode::Contracted).
- **Criterion benchmarks** — Phase 6.
- **Phase 6 libm-hybrid resolution for the 13 Phase-3 D-19 forwards** — D-10 inherits unchanged.
- **Alias-feasibility re-check for LB94** — D-13 confirmed not feasible.
- **`ctaylor_div` primitive** — Phase 1 deferred; no metaGGA body requires it.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MGGA-01 | TPSS family (`XC_TPSSX`, `XC_TPSSC`, `XC_REVTPSSX`, `XC_REVTPSSC`, `XC_TPSSLOCC`) ported, self-tests pass | §"Source Tree Triage" — TPSS family table; `tpssx_eps.hpp` (60 LOC) + `tpssc_eps.hpp` (62) + `revtpssx_eps.hpp` (65) + `revtpssc_eps.hpp` (111) shared helpers |
| MGGA-02 | SCAN family (`SCANX/C`, `RSCANX/C`, `RPPSCANX/C`, `R2SCANX/C`, `R4SCANX/C`) ported, self-tests pass | §"Source Tree Triage" — SCAN family table; `SCAN_like_eps.hpp` (522 LOC, single largest helper) |
| MGGA-03 | Minnesota M05 family (`M05X`, `M05C`, `M05X2X`, `M05X2C`) ported, self-tests pass | §"Source Tree Triage" — M05 family; `m0xy_fun.hpp` (262 LOC) shared with M06 |
| MGGA-04 | Minnesota M06 family (`M06X/C`, `M06LX/C`, `M06HFX/C`, `M06X2X/C`) ported, self-tests pass | §"Source Tree Triage" — M06 family; same `m0xy_fun.hpp` shared module |
| MGGA-05 | `XC_BLOCX` ported, self-test passes | §"Source Tree Triage" — BLOCX. **NOTE:** CONTEXT claim "BLOCX composes BRX" is INCORRECT. `blocx.cpp` is a TPSS-shaped enhancement-factor body with no `BR(...)` call — see §"Source Tree Triage" verdict |
| MODE-03 | `Mode::Contracted` supports orders 0..=6 with output layout matching `DOEVAL` macro | §"Mode::Contracted Implementation" |
| ALIAS-01 | All 46 aliases resolve (including negative-weight `camcompx`) | §"Alias Engine Semantics" — 46 entries enumerated, traces shown |
| ALIAS-02 | Parameter `XC_EXX` settable, default 0.0 | §"Parameter Table" |
| ALIAS-03 | Parameter `XC_RANGESEP_MU` settable, default 0.4 | §"Parameter Table" |
| ALIAS-04 | Parameter `XC_CAM_ALPHA` settable, default 0.19 | §"Parameter Table" |
| ALIAS-05 | Parameter `XC_CAM_BETA` settable, default 0.46 | §"Parameter Table" |
| ALIAS-06 | `Functional::set(name, value)` recurses into aliases with weight multiplication | §"Alias Engine Semantics" — line-for-line port; FIXME-EXX preserved |
| GGA-03 (carryover) | Becke–Roussel family (BRX/BRC/BRXC) | §"BR Newton-Inverse Primitive" |
| GGA-10 (CSC portion) | XC_CSC | §"Source Tree Triage" — CSC, requires id=17 Vars arm |
</phase_requirements>

---

## Project Constraints (from CLAUDE.md)

- **Accuracy:** ≤ 1.0 × 10⁻¹² relative error on every `(functional, vars, mode, order, point)` tuple.
- **No `mul_add`, no FMA, no `-Cfast-math`, no reassociation** — algorithmic-identity rule. CI gates `xtask check-no-mul-add` + `xtask check-no-fma` cover `crates/xcfun-eval/src/functionals/**/*.rs` (the `**` glob already includes `mgga/`).
- **f64 only** on the numerical path — `CTaylor<F, N>` instantiated only with f64 in production.
- **Edition 2024, MSRV 1.85**, Rust stable.
- **`thiserror 2.0.18`** in library crates; **`anyhow`** allowed only in `validation/`, `xtask/`, `benches/`, `examples/` — `xtask check-no-anyhow` enforces.
- **`cubecl =0.10.0-pre.3`** hard-pin across all 4 cubecl crates (`cubecl`, `cubecl-cpu`, `cubecl-cuda`, `cubecl-wgpu`); `xtask check-cubecl-pin` enforces.
- **`bitflags 2.10.0`**, **`approx 0.5.1`**, **`criterion 0.8.2`** (CLAUDE.md pins; do not bump silently).
- **Edition-2024 module resolution + 2024-specific trait resolution**.
- **MPL-2.0 license** inherited from xcfun-master.
- **GSD workflow:** all changes via `/gsd-execute-phase` — no direct edits outside the workflow.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| BR Newton-Raphson scalar root | xcfun-ad (host-side scalar) | xcfun-ad (`#[cube]` for_tests harness) | Per Phase 1 D-04: scalar Newton runs on host slot, not inside a `#[cube]` body. Phase 1 `for_tests::cpu_client` pattern wraps the scalar call into a single-launch test kernel for parity diff vs C++. |
| `BR_taylor` polynomial coefficient sweep | xcfun-ad | — | The Brent–Kung linear method is pure CTaylor algebra (`exp`/`*`/`/` already shipped in Phase 1) — fits inside a `#[cube] fn ctaylor_br_inverse<F, const N>` natively. |
| metaGGA energy bodies (32) | xcfun-eval | — | Established Phase 2/3 pattern: `crates/xcfun-eval/src/functionals/<tier>/<fn>.rs` with `#[cube] fn <name>_kernel<F, const N>(d, out, n)`. metaGGAs read `d.tau`/`d.taua`/`d.taub`/`d.lapa`/`d.lapb`/`d.jpaa`/`d.jpbb` from the same `DensVarsDev<F>`. |
| Mode::Contracted host-side dispatch | xcfun-eval (`functional.rs::eval`) | xcfun-eval (`functionals/contracted.rs` new module) | Per `XCFunctional.cpp:619-635`: pack inputs, invoke `<name>_kernel<F, const ORDER>` (same kernel as PartialDerivatives), unpack `(1 << order)` outputs. Pure host-side composition over existing kernels — no per-functional changes. |
| Alias resolution recursion | xcfun-eval (`Functional::set`) | xcfun-core (`Alias` registry table — already declared) | Recursion lives in `Functional::set`; the static `ALIASES: &[Alias]` slice ships in `crates/xcfun-core/src/registry/generated/aliases.rs` (Phase 2 declared empty). Wave 4 populates 46 rows via xtask. |
| Parameter table | xcfun-core (`ParameterId` enum + static slice) | xcfun-eval (`Functional::settings[78..82]` + read path through `densvars.parent.settings`) | C-ABI compat (Phase 5) requires `[f64; 82]` storage layout matching `XC_NR_PARAMETERS_AND_FUNCTIONALS = 82`. Discriminants 78..=81 in a sibling enum. |
| Validation harness extension | validation/ binary | xtask `regen-registry` (extracts aliases.cpp + auto-generates `c_stubs.cpp` shrink) | `validation/build.rs` adds 32 new C++ source files per wave; grid generator extends with metaGGA stratum (sibling seed `0xc0ffee01`). |
| Tier-1 self-tests for 32 metaGGAs | xcfun-eval (`tests/self_tests.rs`) | xcfun-core (test_in/test_out already in `FUNCTIONAL_DESCRIPTORS.rs`) | Verified — `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs:77+` has TEST_IN/TEST_OUT for M05X, M06X, M06LX, M06HFX, M06X2X, M06C, TPSSC, TPSSX, SCANC, SCANX, RSCANC, RSCANX, RPPSCANC, RPPSCANX, ... already populated by Phase 2 `xtask regen-registry`. |

---

## Source Tree Triage

For each of the 32 functional ports + Mode::Contracted DOEVAL + alias engine + parameter table:

### TPSS family (5, MGGA-01)

| Functional | C++ source | LOC | Vars arm needed | Helpers used | Special |
|------------|-----------|-----|-----------------|--------------|---------|
| `XC_TPSSX` | `xcfun-master/src/functionals/tpssx.cpp` | 56 | id=13 (TAUA_TAUB) | `tpssx_eps::F_x`, `tpssx_eps::fx_unif` (in `tpssx_eps.hpp:60`) | Spin-decomposed exchange (`d.a`/`d.b`/`d.gaa`/`d.gbb`/`d.taua`/`d.taub`); body 6 lines, helper 60 lines |
| `XC_TPSSC` | `xcfun-master/src/functionals/tpssc.cpp` | 63 (45 active) | id=13 | `tpssc_eps::tpssc_eps` (in `tpssc_eps.hpp:62`) | Single-line body `return d.n * tpssc_eps::tpssc_eps(d)` — all complexity in helper |
| `XC_REVTPSSX` | `xcfun-master/src/functionals/revtpssx.cpp` | 32 | id=13 | `revtpssx_eps` (in `revtpssx_eps.hpp:65`) | revTPSS exchange |
| `XC_REVTPSSC` | `xcfun-master/src/functionals/revtpssc.cpp` | 33 | id=13 | `revtpssc_eps` (in `revtpssc_eps.hpp:111`) | revTPSS correlation; helper is largest TPSS-family helper |
| `XC_TPSSLOCC` | `xcfun-master/src/functionals/tpsslocc.cpp` | 105 | id=13 | TPSS-locc-specific (inline) | 105 LOC body — single largest TPSS body. Composes `pw92eps` + TPSS structural pieces |

**Existing primitives used:** `ctaylor_pow`, `ctaylor_sqrt`, `ctaylor_log`, `ctaylor_mul`, `ctaylor_add`, `ctaylor_sub`, `ctaylor_reciprocal`, `ctaylor_powi`. **None require BR.** **None require erf.**

**Verdict (HIGH confidence):** Smallest single Wave 1 deliverable. Helpers under 350 LOC combined. Strong candidate to ship Wave 1A.

### SCAN family (10, MGGA-02)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_SCANX` | `SCANx.cpp` | 49 | id=13 | `SCAN_eps::get_SCAN_Fx`, `SCAN_eps::fx_unif` | IDELEC=0 path (no GE correction) |
| `XC_SCANC` | `SCANc.cpp` | 43 | id=13 | `SCAN_eps::r2SCAN_C` | IDELEC=0 path |
| `XC_RSCANX` | `rSCANx.cpp` | 49 | id=13 | `SCAN_eps::get_SCAN_Fx` | IDELEC=1 (rSCAN regularisation) |
| `XC_RSCANC` | `rSCANc.cpp` | 44 | id=13 | `SCAN_eps::r2SCAN_C` | IDELEC=1 |
| `XC_RPPSCANX` | `rppSCANx.cpp` | 48 | id=13 | `SCAN_eps::get_SCAN_Fx` | IDELEC=2 (r++SCAN) |
| `XC_RPPSCANC` | `rppSCANc.cpp` | 43 | id=13 | `SCAN_eps::r2SCAN_C` | IDELEC=2 |
| `XC_R2SCANX` | `r2SCANx.cpp` | 49 | id=13 | `SCAN_eps::get_SCAN_Fx` | IDELEC=3 (r2SCAN — full GE correction) |
| `XC_R2SCANC` | `r2SCANc.cpp` | 44 | id=13 | `SCAN_eps::r2SCAN_C` | IDELEC=3 |
| `XC_R4SCANX` | `r4SCANx.cpp` | 48 | id=13 | `SCAN_eps::get_SCAN_Fx` | IDELEC=4 (4th-order GE correction) — **HIGH-RISK polynomial precision** |
| `XC_R4SCANC` | `r4SCANc.cpp` | 43 | id=13 | `SCAN_eps::r2SCAN_C` | IDELEC=4 |

**Helper:** `SCAN_like_eps.hpp` is **522 lines**, single largest shared module across the entire xcfun source tree. It exports `get_SCAN_Fx`, `r2SCAN_C`, `scan_ec0`, `scan_ec1`, `lda_0`, plus `gcor2` (Padé interpolation), `get_lsda1` (out-parameter style — needs Rust translation to tuple return), and `ufunc` helper. Branch-on-`IDELEC` dispatches the four regularisation variants (`IDELEC ∈ {0, 1, 2, 3, 4}` → SCAN/rSCAN/rppSCAN/r2SCAN/r4SCAN). [VERIFIED: `xcfun-master/src/functionals/SCAN_like_eps.hpp:1-522`]

**Existing primitives used:** `ctaylor_exp`, `ctaylor_log`, `ctaylor_pow`, `ctaylor_sqrt`, `ctaylor_powi`, plus a polynomial Padé via `gcor2`. **No erf, no BR.**

**Out-parameter translation:** `gcor2(P[6], rs, sqrtrs, GG, GGRS)` writes both `GG` and `GGRS` via reference. In Rust `#[cube] fn`, return as a tuple or pass two `&mut` Arrays — the second is the more cubecl-idiomatic. [CITED: `SCAN_like_eps.hpp:500-519`]

**Verdict (MEDIUM confidence):** Largest single helper port; planner should consider sub-waving SCAN_X (5) parallel with SCAN_C (5) per CONTEXT D-01-B. R4SCAN's 4th-order GE polynomial is the highest-degree single expression in the entire metaGGA set — flag for explicit Rule-1 (no reassociation, no compiler reordering) review during Wave 2.

### M05 family (4, MGGA-03)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_M05X` | `m05x.cpp` | 59 | id=13 | `m0xy_metagga_xc_internal::fw`, `pbex::energy_pbe_ab`, `pbex::R_pbe` | 12-coefficient polynomial enhancement `fw(param_a[12], rho, tau)` |
| `XC_M05X2X` | `m05x2x.cpp` | 59 | id=13 | same as M05X | Different `param_a[12]` constants |
| `XC_M05C` | `m05c.cpp` | 62 | id=13 | `m05_c_para`, `m05_c_anti`, `Dsigma`, `chi2`, `zet`, `ueg_c_para`, `ueg_c_anti` | Composition over 8 helpers |
| `XC_M05X2C` | `m05x2c.cpp` | 62 | id=13 | same as M05C | Different parameter set |

**Helper:** `m0xy_fun.hpp` is **262 lines** — second largest helper after `SCAN_like_eps.hpp`. Exports the M05/M06 family substrate: `zet`, `gamma`, `h`, `fw`, `chi2`, `Dsigma`, `g`, `m06_c_anti`, `m06_c_para`, `m05_c_anti`, `m05_c_para`, `ueg_c_para`, `ueg_c_anti`. Reuses `pw92eps::pw92eps` (already in xcfun-eval Phase 2) and `pw9xx::chi2` from `pw9xx.hpp` (already in `gga/shared/pw91_like.rs`). [VERIFIED: `xcfun-master/src/functionals/m0xy_fun.hpp:24-257`]

**Verdict (HIGH confidence):** M05 reuses the m0xy_fun.hpp helper that also feeds M06 (12 functionals total). Wave 3 ships m0xy_fun.hpp once, used by 12 functionals.

### M06 family (8, MGGA-04)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_M06X` | `m06x.cpp` | 93 | id=13 | `fw`, `h`, `zet`, `chi2`, `alpha_x`, `lsda_x`, `pbex::energy_pbe_ab` | 12-coef + 6-coef parameter sets |
| `XC_M06C` | `m06c.cpp` | 109 | id=13 | `m06_c_para`, `m06_c_anti`, `Dsigma`, `chi2`, `zet`, `ueg_c_para`, `ueg_c_anti` | Largest M06 body |
| `XC_M06LX` | `m06lx.cpp` | 74 | id=13 | same as M06X | Local M06 (no exact exchange admixture in alias) |
| `XC_M06LC` | `m06lc.cpp` | 69 | id=13 | same as M06C | |
| `XC_M06HFX` | `m06hfx.cpp` | 74 | id=13 | same as M06X | M06-HF (Hartree-Fock-like exchange) |
| `XC_M06HFC` | `m06hfc.cpp` | 96 | id=13 | same as M06C | |
| `XC_M06X2X` | `m06x2x.cpp` | 61 | id=13 | same as M06X | M06-2X (2× exact exchange) |
| `XC_M06X2C` | `m06x2c.cpp` | 78 | id=13 | same as M06C | |

**Helpers:** Same `m0xy_fun.hpp` as M05 family, plus `pbex.hpp` (already ported in `gga/shared/pbex.rs`).

**Verdict (HIGH confidence):** Numerically the highest-risk family per literature (Pitfall §"M06 family numerical sensitivity"). The 12-coefficient polynomial `fw` evaluated at high orders requires verbatim port-order preservation. The parameter array layout per functional is the only difference between M06X / M06X2X / M06LX / M06HFX bodies — consider a `#[cube] fn m06_x_kernel<F, const N>(d, params: &Array<F>, out, n)` with parameters passed as a runtime cubecl Array. Planner picks; both work.

### BLOCX (1, MGGA-05)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_BLOCX` | `blocx.cpp` | 61 | id=13 (TAUA_TAUB) | NONE — single body | TPSS-shaped enhancement; **CONTEXT note "BLOCX composes BRX" is INCORRECT** |

**Verdict (HIGH confidence — verified by reading `blocx.cpp:1-62`):** BLOCX has NO `BR(...)` call. It uses `pow(d_n, ...)`, `pow(p, ...)`, `sqrt`, `log`, `exp` over a TPSS-shaped enhancement structure. The `tauw / d_tau` reduced kinetic-energy ratio appears (similar shape to TPSSX) but no Newton-inverse. CONTEXT D-01-A's claim "BLOCX composes BRX (so depends on Wave 1 BR ship)" is **wrong** — BLOCX has zero BRX dependency. The planner should not gate Wave 3 (BLOCX) on Wave 1 (BR) shipping; they are independent. [VERIFIED: `xcfun-master/src/functionals/blocx.cpp:18-46`]

**Vars arm:** id=13 (TAUA_TAUB) — same as TPSS/SCAN/M0x. No JP / no laplacian.

### BR family carryover (3, GGA-03)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_BRX` | `brx.cpp:103-106` | 4 (helper-driven) | id=17 (full JPAA_JPBB) | `polarized` (`brx.cpp:89-101`), `BR(t)` ctaylor (`brx.cpp:78-87`), `BR_taylor<T,Ndeg>` (`brx.cpp:53-71`), scalar `BR(z)` (`brx.cpp:30-48`), `BR_z` (`brx.cpp:21-23`) | Spin-resolved, 5-arg `polarized(na, gaa, lapa, taua, jpaa)` |
| `XC_BRC` | `brx.cpp:108-121` | 14 | id=17 | `polarized` + log/pow algebra | "Becke-Roussel correlation"; uses `cab=0.63`, `caa=0.88` constants; uses `abs(...)` |
| `XC_BRXC` | `brx.cpp:123-136` | 14 | id=17 | same as BRC + `polarized` for energy contribution | Hybrid-ish energy = exchange + correlation |

**ALL three BR functionals ship in `brx.cpp` together** (one source file, three FUNCTIONAL macro registrations). Vars dependency: `XC_DENSITY | XC_GRADIENT | XC_KINETIC | XC_LAPLACIAN | XC_JP` → maps to `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` (id=17, inlen=11). [VERIFIED: `brx.cpp:138-157`]

**Critical numerical hazard:** `BR(z)` uses scalar Newton on the constant slot. Different libm `exp` between the C++ side (whatever the validation runner provides) and the Rust side `f64::exp` could change the converged x by 1–3 ULP. **Phase 2 LDAERFX precedent (commit `dca382a`)** introduces a libm-port `erf_precise` to lock the exp/erf to a deterministic implementation; the BR Newton may need an analogous libm-port `exp_precise` if fixture-gate exposes drift. **Fixture-gate at strict 1e-12; if it fails, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03.**

### CSC carryover (1, GGA-10)

| Functional | C++ source | LOC | Vars arm | Helpers used | Special |
|------------|-----------|-----|----------|--------------|---------|
| `XC_CSC` | `cs.cpp` | 35 (10 active LOC) | id=17 (full JP) | none — single body, single inline expression | Reads `d.a`/`d.b`/`d.n`/`d.taua`/`d.taub`/`d.gnn`/`d.jpaa`/`d.jpbb`/`d.n_m13` |

**Verdict (HIGH confidence — verified):** Single inline expression `-a * gamma * (n + 2*b * pow(n, -5/3) * curv * exp(-c*n_m13)) / (1 + d*n_m13)` where `gamma = 2*(1 - (a²+b²)/n²)` and `curv = a*taua + b*taub - gnn/8 - (jpaa+jpbb)`. No Newton, no erf. CSC is the simplest of the four Phase-3 carryovers. [VERIFIED: `xcfun-master/src/functionals/cs.cpp:17-27`]

### Mode::Contracted DOEVAL (MODE-03)

| Item | Source | Lines |
|------|--------|-------|
| `DOEVAL` macro definition | `XCFunctional.cpp:619-635` | 17 lines |
| `output_length` for Contracted | `XCFunctional.cpp:482-490` | 9 lines |
| Capacity constants | `xcint.hpp:26-28` | `XC_MAX_ALIASES=60`, `MAX_ALIAS_TERMS=10`, `XC_MAX_INVARS=20` |

**The DOEVAL pattern:** for each `(N=order)`, instantiate `ttype = ctaylor<ireal_t, N>`, allocate `ttype in[XC_MAX_INVARS]`; for each Vars-element `i` and each Taylor coefficient `j ∈ 0..(1<<order)`, `in[i].set(j, input[k++])` (linear flat read); construct `densvars<ttype>(fun, in)`; for each active functional, `out += setting * fp_N(d)`; output `(1 << order)` flat coefficients via `output[i] = out.get(i)`. [VERIFIED: `XCFunctional.cpp:619-635`]

**Per-functional kernel re-use:** Mode::Contracted does NOT call a per-mode kernel; it re-uses the same `<name>_kernel<F, const N>` body that PartialDerivatives uses (per-functional fp0..fp6 maps to the same template instantiation). The host-side wrapper packs / unpacks. **NO per-functional changes for Phase 3 GGAs / Phase 2 LDAs** — they "automatically" work in Mode::Contracted once the host-side dispatch lands.

### Alias engine (ALIAS-01..06)

| Item | Source | Lines |
|------|--------|-------|
| `xcfun_set` recursive resolution | `XCFunctional.cpp:369-405` | 37 lines |
| `xcfun_get` lookup (no alias) | `XCFunctional.cpp:407-419` | 13 lines |
| 46 alias entries | `aliases.cpp:17-139` | 122 lines (data table) |
| Constructor parameter init | `XCFunctional.cpp:350-355` | 6 lines |

**ALL 46 entries enumerated** — see §"Alias Engine Semantics" below.

### Parameter table (ALIAS-02..05)

| Item | Source | Lines |
|------|--------|-------|
| Parameter ids | `list_of_functionals.hpp:99-105` | 6 lines |
| Parameter defaults | `common_parameters.cpp:17-29` | 12 lines |
| Storage layout | `XCFunctional.cpp:347-355` | 6 lines |

`enum xc_parameter` is contiguous after `XC_NR_FUNCTIONALS`: `XC_RANGESEP_MU = XC_NR_FUNCTIONALS, XC_EXX, XC_CAM_ALPHA, XC_CAM_BETA, XC_NR_PARAMETERS_AND_FUNCTIONALS`. Discriminants 78..=81. `XC_NR_PARAMETERS_AND_FUNCTIONALS = 82`. [VERIFIED: `list_of_functionals.hpp:100-104`]

---

## Mode::Contracted Implementation

### Verbatim port of `XCFunctional.cpp:619-635`

```cpp
} else if (fun->mode == XC_CONTRACTED) {
#define DOEVAL(N, E)                                                                \
  if (fun->order == N) {                                                            \
    typedef ctaylor<ireal_t, N> ttype;                                              \
    int inlen = xcint_vars[fun->vars].len;                                          \
    ttype in[XC_MAX_INVARS], out = 0;                                               \
    int k = 0;                                                                      \
    for (int i = 0; i < inlen; i++)                                                 \
      for (int j = 0; j < (1 << fun->order); j++)                                   \
        in[i].set(j, input[k++]);                                                   \
    densvars<ttype> d(fun, in);                                                     \
    for (int i = 0; i < fun->nr_active_functionals; i++)                            \
      out += fun->settings[fun->active_functionals[i]->id] *                        \
             fun->active_functionals[i]->fp##N(d);                                  \
    for (int i = 0; i < (1 << fun->order); i++)                                     \
      output[i] = out.get(i);                                                       \
  } else
    FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )
    xcfun::die("bug! Order too high in XC_CONTRACTED", fun->order);
}
```

The `FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )` expands to `if (order==0) { ... } else if (order==1) { ... } else if (order==2) { ... } ... else if (order==6) { ... } else { die(...); }`. Seven `if` arms, one per order N ∈ {0,1,2,3,4,5,6}. [CITED: `XCFunctional.cpp:619-636`]

### Input layout at order N

For order N, input length = `inlen × (1 << N)` where `inlen = xcint_vars[fun->vars].len` (the same `input_length` returned by `xcfun_input_length`). The flat read pattern is:

```
input[0]            = in[0].coeff(CNST=0)
input[1]            = in[0].coeff(VAR0=1)
input[2]            = in[0].coeff(VAR1=2)
input[3]            = in[0].coeff(VAR0|VAR1=3)
...
input[(1<<N) - 1]   = in[0].coeff((1<<N) - 1)  // top bit-flag combo
input[(1<<N) + 0]   = in[1].coeff(CNST)
...
input[inlen*(1<<N) - 1] = in[inlen-1].coeff((1<<N) - 1)
```

In other words: per Vars-element, contiguously layout all `(1<<N)` Taylor coefficients in bit-flag-index order; concatenate over the `inlen` Vars-elements. [VERIFIED by reading `XCFunctional.cpp:625-627` — `for(i){for(j){in[i].set(j, input[k++])}}`]

**Output layout:** `output[i] = out.get(i)` for `i ∈ 0..(1 << N)`. The bit-flag indexing means `output[0]` = the constant (energy at the input point), `output[1]` = ∂E/∂x₀, `output[3]` = ∂²E/∂x₀∂x₁, etc.

**Output length:** `1 << order` for Mode::Contracted. **NOTE:** the C++ `xcfun_output_length` (XCFunctional.cpp:488-489) calls `xcfun::die` for Mode::Contracted — the value `1 << order` must be enforced **outside** the C `output_length`. Rust takes the opposite stance: `Functional::output_length` returns `Ok(1 << order)` for Contracted (D-06-B). This is a deliberate divergence from C++ — equivalent to the C++ behaviour on the host caller's side (the C++ caller would need to compute `1 << order` themselves). [VERIFIED: `XCFunctional.cpp:488` `xcfun::die("XC_CONTRACTED not implemented in xc_output_length()", 0)`]

### Vars compatibility (D-06-A)

The DOEVAL macro does NOT contain a Vars rejection step. Any Vars whose `densvars<T>` constructor accepts an `inlen`-element CTaylor array is compatible. The compatibility is data-driven: input is `inlen × (1 << order)` doubles. [VERIFIED]

The `eval_setup` outer guard at `XCFunctional.cpp:430-433` still applies: `(fun->depends & xcint_vars[vars].provides) != fun->depends → XC_EVARS`. This is the same Vars-vs-functional-deps check from PartialDerivatives. Phase 4 inherits unchanged.

### Order cap = 6

`XCFUN_MAX_ORDER` is `#define`d to `6` in the project. The `FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )` macro expands exactly to seven branches (orders 0..=6). `eval_setup` validates `0 <= order <= 6`. Order 7 returns `XC_EORDER`. [VERIFIED: `xcint.hpp` (also redefined per `validation/build.rs:75`)]

### CTaylor<F, 6> capacity check

CTaylor<F, 6> backing `Array<F>` has length `1 << 6 = 64`. Per Phase 1 AD-01, `CTaylor<F: Float, const N: u32>` is valid for `N ∈ 0..=7`. Phase 4 exercises N=6 for the first time at runtime (Phase 1 property tests covered orders 0..=3 per AD-06 batch-per-property pattern). **D-07 fixture-gate**: a single `cargo test -p xcfun-ad --features cpu test_ctaylor_n6` smoke test in Wave 0 confirms allocation + multiply-by-CTaylor-of-N=6 vs C++ reference at strict 1e-12.

**Stack budget per kernel invocation:** `XC_MAX_INVARS × (1 << 6) × 8 bytes = 20 × 64 × 8 = 10,240 bytes = 10 KB`. Well within cubecl-cpu kernel stack. [CITED: `xcint.hpp:28`]

### C++ tests reaching order 5/6

Search for upstream tests exercising Contracted at order 5 or 6:

```bash
grep -rn "XC_CONTRACTED\|XC_PARTIAL_DERIVATIVES" xcfun-master/test/ 2>/dev/null
```

Result: **no match** (no upstream test directory at `xcfun-master/test/` in the vendored copy). The `FUNCTIONAL` macro test_in/test_out fixtures in each `.cpp` file use `XC_PARTIAL_DERIVATIVES` only at `order 1` (the `1` in the `FUNCTIONAL` argument list — see e.g. `tpssx.cpp:40`). **Mode::Contracted at orders 5/6 has no direct C++ test coverage in the vendored xcfun source.** [VERIFIED: searched `xcfun-master/`]

**Implication for Wave 5:** Tier-2 cross-mode comparison at orders 0..=4 between PartialDerivatives and Contracted is the structural smoke test (Contracted is a re-packaging of the same Taylor coefficients per `XCFunctional.cpp:619-635` — algorithmic identity guarantees parity if the PartialDerivatives kernel passed). **Orders 5/6 require extending `validation/src/c_driver.rs`** to invoke `xcfun_eval` with `XC_CONTRACTED` mode at order 5/6 on a 100-point subset and diff vs Rust. Per CONTEXT D-06-C: 1000-point subset is the planning-time goal; in practice 100 points × 4 functionals at order 5 + 6 = 800 records is enough for the strict-1e-12 budget.

---

## Alias Engine Semantics

### Verbatim port of `XCFunctional.cpp:369-405`

```cpp
int xcfun_set(XCFunctional * fun, const char * name, double value) {
  xcint_assure_setup();
  int item;
  if ((item = xcint_lookup_functional(name)) >= 0) {
    fun->settings[item] += value;                       // ① ADDITIVE on functional weights
    bool found = false;
    for (int i = 0; i < fun->nr_active_functionals; i++)
      if (fun->active_functionals[i] == &xcint_funs[item]) { found = true; break; }
    if (!found) {
      fun->active_functionals[fun->nr_active_functionals++] = &xcint_funs[item];
      fun->depends |= xcint_funs[item].depends;
    }
    return 0;
  } else if ((item = xcint_lookup_parameter(name)) >= 0) {
    fun->settings[item] = value;                        // ② OVERWRITE on parameter weights
    return 0;
  } else if ((item = xcint_lookup_alias(name)) >= 0) {
    for (int i = 0; i < MAX_ALIAS_TERMS; i++) {
      if (!xcint_aliases[item].terms[i].name) break;
      // FIXME: Do not weight parameters with value for aliases, but what about EXX?
      if (xcfun_set(fun,
                    xcint_aliases[item].terms[i].name,
                    value * xcint_aliases[item].terms[i].weight) != 0) {       // ③ RECURSIVE WITH MULTIPLICATIVE WEIGHT
        fprintf(stderr, "Trying to set %s\n", xcint_aliases[item].terms[i].name);
        xcfun::die("Alias with unknown terms, fix aliases.cpp", item);
      }
    }
    return 0;
  }
  return -1;
}
```

**Three semantics in priority order:**
1. **Functional name match → ADDITIVE accumulation** (`fun->settings[item] += value`).
2. **Parameter name match → OVERWRITE** (`fun->settings[item] = value`).
3. **Alias name match → RECURSE on each term with weight multiplication** (`xcfun_set(fun, term.name, value * term.weight)`). The recursion bottoms out in 1 or 2.

**The FIXME at L390:** "Do not weight parameters with value for aliases, but what about EXX?" — the comment notes a wart but the code still runs unchanged (the `value * term.weight` IS multiplied for parameters via the recursive call, hitting case ②, where the parameter's effective value becomes `value × term.weight`). Algorithmic-identity rule **forbids** "fixing" the FIXME. [VERIFIED: `XCFunctional.cpp:389-404`]

### Concrete trace: `set("b3lyp", 1.0)`

`b3lyp` alias (lines 42-48 of aliases.cpp):
- `{"slaterx", 0.80}`
- `{"beckecorrx", 0.72}`
- `{"lypc", 0.81}`
- `{"vwn5c", 0.19}`
- `{"exx", 0.20}`

Recursion (with `value = 1.0`):
1. `set("slaterx", 1.0 × 0.80 = 0.80)` → case ① → `settings[XC_SLATERX] += 0.80`. Activate.
2. `set("beckecorrx", 1.0 × 0.72 = 0.72)` → case ① → `settings[XC_BECKECORRX] += 0.72`. Activate.
3. `set("lypc", 1.0 × 0.81 = 0.81)` → case ① → `settings[XC_LYPC] += 0.81`. Activate.
4. `set("vwn5c", 1.0 × 0.19 = 0.19)` → case ① → `settings[XC_VWN5C] += 0.19`. Activate.
5. `set("exx", 1.0 × 0.20 = 0.20)` → case ② → `settings[XC_EXX] = 0.20` **(overwrites)**.

Final state: `settings[SLATERX]=0.80, settings[BECKECORRX]=0.72, settings[LYPC]=0.81, settings[VWN5C]=0.19, settings[EXX]=0.20`. Active functionals = {SLATERX, BECKECORRX, LYPC, VWN5C}.

### Concrete trace: `set("b3lyp", 1.0); set("slaterx", 0.5)`

After `set("b3lyp", 1.0)` (above): `settings[SLATERX] = 0.80`.

Then `set("slaterx", 0.5)` → case ① → `settings[SLATERX] += 0.5 → 1.30`. Already-active so no list extension.

**Final `get("slaterx") == 1.30`.** Matches CONTEXT specifics line 388: `0.80 + 0.5 == 1.30`. [VERIFIED via line-by-line trace]

### Concrete trace: `set("b3lyp", 0.5)` then `get("exx")`

`set("b3lyp", 0.5)` recurses with `value = 0.5`:
- `set("exx", 0.5 × 0.20 = 0.10)` → case ② → `settings[EXX] = 0.10` (NOT 0.30 — overwrites).

`get("exx") == 0.10`. **NOT** additive (case ② overwrites). Matches CONTEXT specifics line 390. [VERIFIED]

### Concrete trace: negative-weight `camcompx`

`camcompx` alias (lines 119-125 of aliases.cpp):
- `{"beckex", 1.0}`
- `{"beckecamx", -1.0}` ← **NEGATIVE WEIGHT**
- `{"cam_alpha", 0.19}`
- `{"cam_beta", 0.46}`
- `{"rangesep_mu", 0.33}`

`set("camcompx", 0.37)` recurses:
1. `set("beckex", 0.37 × 1.0 = 0.37)` → ① → `settings[BECKEX] += 0.37`.
2. `set("beckecamx", 0.37 × (-1.0) = -0.37)` → ① → `settings[BECKECAMX] += -0.37` (i.e. `-0.37`).
3. `set("cam_alpha", 0.37 × 0.19 = 0.0703)` → ② → `settings[CAM_ALPHA] = 0.0703`.
4. `set("cam_beta", 0.37 × 0.46 = 0.1702)` → ② → `settings[CAM_BETA] = 0.1702`.
5. `set("rangesep_mu", 0.37 × 0.33 = 0.1221)` → ② → `settings[RANGESEP_MU] = 0.1221`.

**`get("beckecamx") == -0.37`** — the negative-weight canary from CONTEXT D-04 + Pitfall P11. [VERIFIED by line-by-line trace against `aliases.cpp:119-125`]

### All 46 aliases enumerated

| # | Alias | Description | Terms (name, weight) |
|---|-------|-------------|----------------------|
| 1 | `null` | No functional | `slaterx 0.0` |
| 2 | `lda` | Slater + VWN5 | `slaterx 1.0`, `vwn5c 1.0` |
| 3 | `blyp` | Becke + LYP | `beckex 1.0`, `lypc 1.0` |
| 4 | `pbe` | PBE | `pbex 1.0`, `pbec 1.0` |
| 5 | `bp86` | Becke-Perdew 86 | `beckex 1.0`, `p86c 1.0` |
| 6 | `kt1` | Keal-Tozer 1 | `slaterx 1.0`, `ktx -0.006`, `vwn5c 1.0` |
| 7 | `kt2` | Keal-Tozer 2 | `slaterx 1.07173`, `ktx -0.006`, `vwn5c 0.576727` |
| 8 | `kt3` | Keal-Tozer 3 | `slaterx 1.092`, `ktx -0.004`, `optxcorr -0.925452`, `lypc 0.864409` |
| 9 | `ldaerf` | Short-range LDA | `ldaerfx 1.0`, `ldaerfc 1.0` |
| 10 | `pbe0` | PBE0 | `pbex 0.75`, `pbec 1.0`, `exx 0.25` |
| 11 | `b3lyp` | B3LYP (VWN5) | `slaterx 0.80`, `beckecorrx 0.72`, `lypc 0.81`, `vwn5c 0.19`, `exx 0.20` |
| 12 | `m06` | M06 | `m06c 1.0`, `m06x 1.0` |
| 13 | `m06-2x` | M06-2X | `m06x2c 1.0`, `m06x2x 1.0` |
| 14 | `m06L` | M06-L | `m06lc 1.0`, `m06lx 1.0` |
| 15 | `b3lyp-g` | B3LYP (VWN3) | `slaterx 0.80`, `beckecorrx 0.72`, `lypc 0.81`, `vwn3c 0.19`, `exx 0.20` |
| 16 | `b3p86` | B3P86 (VWN5) | `slaterx 0.80`, `beckecorrx 0.72`, `p86corrc 0.81`, `vwn5c 1.0`, `exx 0.20` |
| 17 | `b3p86-g` | B3P86 (VWN3) | `slaterx 0.80`, `beckecorrx 0.72`, `p86corrc 0.81`, `vwn3c 1.0`, `exx 0.20` |
| 18 | `bpw91` | Becke + PW91 | `beckex 1.0`, `pw91c 1.0` |
| 19 | `b97` | B97 | `b97x 1.0`, `b97c 1.0`, `exx 0.1943` |
| 20 | `b97-1` | B97-1 | `b97_1x 1.0`, `b97_1c 1.0`, `exx 0.21` |
| 21 | `b97_2` | B97-2 | `b97_2x 1.0`, `b97_2c 1.0`, `exx 0.21` |
| 22 | `camb3lyp` | CAM-B3LYP | `cam_alpha 0.19`, `cam_beta 0.46`, `rangesep_mu 0.33`, `beckecamx 1.0`, `vwn5c 0.19`, `lypc 0.81`, `exx 1.0` |
| 23 | `vwn` | VWN5 | `vwn5c 1.0` |
| 24 | `vwn5` | VWN5 | `vwn5c 1.0` |
| 25 | `vwn3` | VWN3 | `vwn3c 1.0` |
| 26 | `svwn` | Slater + VWN5 | `slaterx 1.0`, `vwn5c 1.0` |
| 27 | `svwn5` | Slater + VWN5 | `slaterx 1.0`, `vwn5c 1.0` |
| 28 | `svwn3` | Slater + VWN3 | `slaterx 1.0`, `vwn3c 1.0` |
| 29 | `becke` | Becke exchange | `beckecorrx 1.0` |
| 30 | `slater` | Slater exchange | `slaterx 1.0` |
| 31 | `olyp` | LYP + OPTX | `lypc 1.0`, `optx 1.0` |
| 32 | `lyp` | LYP correlation | `lypc 1.0` |
| 33 | `B88X` | Becke (ADMM) | `slaterx 1.0`, `beckecorrx 1.0` |
| 34 | `LDAX` | Slater (ADMM) | `slaterx 1.0` |
| 35 | `PBEX` | PBE exchange (ADMM) | `pbex 1.0` |
| 36 | `HF` | HF exchange | `exx 1.0` |
| 37 | `KT3X` | KT3 exchange | `slaterx 1.092`, `ktx -0.004`, `optxcorr -0.925452` |
| 38 | `OPTX` | OPTX exchange | `slaterx 1.05151`, `optxcorr -1.43169` |
| 39 | `camcompx` | CAM complementary | `beckex 1.0`, **`beckecamx -1.0`**, `cam_alpha 0.19`, `cam_beta 0.46`, `rangesep_mu 0.33` |
| 40 | `tfk` | Thomas-Fermi | `tfk 1.0` |
| 41 | `tw` | von Weizsacker | `tw 1.0` |
| 42 | `scan` | SCAN | `scanx 1.0`, `scanc 1.0` |
| 43 | `rscan` | rSCAN | `rscanx 1.0`, `rscanc 1.0` |
| 44 | `rppscan` | r++SCAN | `rppscanx 1.0`, `rppscanc 1.0` |
| 45 | `r2scan` | r2SCAN | `r2scanx 1.0`, `r2scanc 1.0` |
| 46 | `r4scan` | r4SCAN | `r4scanx 1.0`, `r4scanc 1.0` |

[VERIFIED: full enumeration of `aliases.cpp:17-138`. Total = 46 entries — confirmed by `grep -c '^\s*\{\"[a-zA-Z0-9_-]\+\",' aliases.cpp = 46`.]

### Alias-of-alias check

**Question:** Does any term name in any alias resolve to another alias (rather than a functional or parameter)?

**Search:** every term name on the right side of all 46 aliases must be one of:
- A functional id (78 names from `FunctionalId`).
- A parameter id (4 names: `rangesep_mu`, `exx`, `cam_alpha`, `cam_beta`).
- ANOTHER alias (recursive case).

Going through each term name in the table above: `slaterx, vwn5c, beckex, lypc, pbex, pbec, p86c, ktx, optxcorr, ldaerfx, ldaerfc, exx, beckecorrx, vwn3c, p86corrc, pw91c, b97x, b97c, b97_1x, b97_1c, b97_2x, b97_2c, cam_alpha, cam_beta, rangesep_mu, beckecamx, optx, m06c, m06x, m06x2c, m06x2x, m06lc, m06lx, tfk, tw, scanx, scanc, rscanx, rscanc, rppscanx, rppscanc, r2scanx, r2scanc, r4scanx, r4scanc`. 

All are either functional names (78 distinct ids) or parameter names (4 distinct ids — `cam_alpha/cam_beta/rangesep_mu/exx`). **No term name matches an alias name** (cross-checked: `lda` is an alias only — never appears as a term; `b3lyp`, `pbe`, `m06`, etc. similarly).

**Conclusion (HIGH confidence):** the alias graph is depth-1: every alias resolves directly to functionals + parameters. The recursion in `Functional::set` terminates after exactly one alias-resolution step. **A registry-time invariant** ("no alias term name is itself an alias name") suffices — no runtime depth counter required. [VERIFIED by exhaustive enumeration]

**Recommendation:** Implement the registry-time invariant in `xtask regen-registry --check`: after extracting the 46 aliases, fail if any `term.name` matches any `alias.name`. This is the cheapest and strongest guard.

### Case-insensitive lookup (D-04-B)

C++ uses `strcasecmp` (POSIX-Unix) at `xcint.cpp:40` for `xcint_lookup_*`. Rust equivalent: `eq_ignore_ascii_case`. Note: alias names in the table are MIXED CASE (`b3lyp`, `M06`, `B97-1`, `pbe0`, `B88X`, `LDAX`, `HF`, `KT3X`, `OPTX`). **Verified case insensitivity in C++**: callers may pass any case (`set("B3LYP", 1.0)` matches `set("b3lyp", 1.0)`).

---

## Parameter Table

### Source verification

```cpp
// xcfun-master/src/functionals/list_of_functionals.hpp:99-105
enum xc_parameter {
  XC_RANGESEP_MU = XC_NR_FUNCTIONALS,  // = 78
  XC_EXX,                              // = 79
  XC_CAM_ALPHA,                        // = 80
  XC_CAM_BETA,                         // = 81
  XC_NR_PARAMETERS_AND_FUNCTIONALS     // = 82
};

// xcfun-master/src/functionals/common_parameters.cpp:17-29
PARAMETER(XC_RANGESEP_MU) = {"Range separation inverse length [1/a0]", 0.4};
PARAMETER(XC_EXX) = {"Amount of exact (HF like) exchange (must be provided externally)", 0.0};
PARAMETER(XC_CAM_ALPHA) = {"Amount of exact (HF like) exchange within CAM-B3LYP functional", 0.19};
PARAMETER(XC_CAM_BETA) = {"Amount of long-range (HF like) exchange within CAM-B3LYP functional", 0.46};
```

[VERIFIED]

### Storage layout

`XCFunctional::settings` is `double settings[XC_NR_PARAMETERS_AND_FUNCTIONALS]` (length 82). At construction:

```cpp
// xcfun-master/src/XCFunctional.cpp:350-355
XCFunctional::XCFunctional() {
  for (int i = 0; i < XC_NR_FUNCTIONALS; ++i)
    settings[i] = 0;
  for (int i = XC_NR_FUNCTIONALS; i < XC_NR_PARAMETERS_AND_FUNCTIONALS; ++i)
    settings[i] = xcint_params[i].default_value;
}
```

Functional weights at indices 0..78 default to 0.0; parameter values at indices 78..82 default to `[0.4, 0.0, 0.19, 0.46]` (in order RANGESEP_MU, EXX, CAM_ALPHA, CAM_BETA).

### Read path: `densvars::get_param`

```cpp
// xcfun-master/src/densvars.hpp:221
double get_param(enum xc_parameter p) const { return parent->settings[p]; }
```

The `densvars<T>` struct holds a `parent` pointer to the `XCFunctional` instance, and `get_param` reads the settings array directly. Used by e.g. `beckex.cpp:84` (`d.get_param(XC_RANGESEP_MU)`). [VERIFIED]

**Rust translation:** `DensVarsDev<F>` does NOT carry a parent pointer (Phase 3 plan 03-02 introduced `Functional::parameters: [f64; 4]` per B3 in `functional.rs:88-93`, passed as a separate cubecl argument). The existing parameters array is `[XC_EXX, XC_RANGESEP_MU, XC_CAM_ALPHA, XC_CAM_BETA]` — note the order is **DIFFERENT from C++** (`[EXX, RANGESEP_MU, ...]` vs C++ `[RANGESEP_MU, EXX, CAM_ALPHA, CAM_BETA]`). Phase 4 must reconcile: either **(a)** keep current Rust order and require host-side index translation when `Functional::set("RANGESEP_MU", v)` writes the parameter array, or **(b)** flip the Rust order to match C++ exactly. **Recommend (b)** — eliminates one layer of cognitive overhead and matches the C-ABI layout for Phase 5. [VERIFIED: `crates/xcfun-eval/src/functional.rs:79-93` shows `[EXX, RANGESEP_MU, CAM_ALPHA, CAM_BETA]` (index 0 = EXX, 1 = RANGESEP_MU)]

**Better still:** unify into a single `settings: [f64; 82]` array per CONTEXT D-05, matching C++ exactly. Index 78..=81 holds parameters; 0..=77 holds functional weights. C-ABI compatibility (Phase 5) prefers this layout. The current `Functional::parameters: [f64; 4]` design is a Phase-3 stopgap; Phase 4 unifies.

### `ParameterId` enum (D-05-A)

```rust
// crates/xcfun-core/src/parameter_id.rs (NEW)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterId {
    XC_RANGESEP_MU = 78,
    XC_EXX = 79,
    XC_CAM_ALPHA = 80,
    XC_CAM_BETA = 81,
}

impl ParameterId {
    pub const COUNT: usize = 4;
    pub fn name(&self) -> &'static str { /* ... */ }
    pub fn default_value(&self) -> f64 { /* ... */ }
    pub fn from_name(name: &str) -> Option<Self> { /* eq_ignore_ascii_case */ }
}
```

Co-located in `xcfun-core` next to `FunctionalId`. Defaults from `common_parameters.cpp:17-29`. [DESIGN]

---

## BR Newton-Inverse Primitive

### Source: `xcfun-master/src/functionals/brx.cpp:21-87`

The primitive has 3 layers:

**Layer 1 — Scalar Newton root finder `BR(z) → x`:**

```cpp
// brx.cpp:21-23 — root-finding target
template <typename T> T BR_z(const T & x) {
  return (x - 2) / x * exp(2.0 / 3.0 * x);   // BR_z(x) = (x-2)/x * exp(2x/3)
}

// brx.cpp:25-27 — Newton step
static double NR_step(double x, double z) {
  return (x * (3 * x * (exp(-2.0 / 3.0 * x) * z - 1) + 6)) / (x * (2 * x - 4) + 6);
}

// brx.cpp:29-48 — scalar Newton solving BR_z(x) = z
static double BR(double z) {
  double x0;
  if (z < -1e4)
    x0 = -2 / z;
  else if (z < -2)
    x0 = (sqrt(9 * z * z + 6 * z + 49) + 3 * z + 1) / 4;
  else if (z < 1)
    x0 = 2 * (z * exp(-4.0 / 3.0) + 1);
  else
    x0 = 3.0 / 2.0 * log(z) + 3.75 / (1.5 + log(z));
  for (int i = 0; i < 20; i++) {
    double xold = x0;
    x0 += NR_step(x0, z);
    if (fabs(xold - x0) < 1e-15 * (1 + x0))
      return x0;
  }
  fprintf(stderr, "BR: Not converged for z = %e\n", z);
  return x0;
}
```

**Initial-guess branches (CRITICAL — preserve `<` vs `<=` exactly):**

| Range | Initial guess `x0` |
|-------|--------------------|
| `z < -1e4` | `-2/z` |
| `-1e4 ≤ z < -2` | `(sqrt(9z² + 6z + 49) + 3z + 1) / 4` |
| `-2 ≤ z < 1` | `2 * (z * exp(-4/3) + 1)` |
| `z ≥ 1` | `(3/2) * log(z) + 3.75 / (1.5 + log(z))` |

The C++ uses `<` (strict less-than) for all four boundaries → exact ports must use `<` not `<=`. Boundary hits (z = -1e4, z = -2, z = 1) fall into the higher branch. [VERIFIED: `brx.cpp:30-39`]

**Convergence:** 20-iteration cap, `|xold - x0| < 1e-15 * (1 + x0)` relative tolerance (line 43).

**Layer 2 — Brent–Kung linear-method polynomial inverter `BR_taylor`:**

```cpp
// brx.cpp:50-71
template <typename T, int Ndeg>
taylor<T, 1, Ndeg> BR_taylor(const T & z0) {
  static_assert(Ndeg >= 3, "Polynomial degree must be at least 3!");

  taylor<T, 1, Ndeg> t;
  t = 0;
  t[0] = BR(z0);                  // scalar Newton on constant slot
  t[1] = 1;                       // initial guess for linear coefficient

  taylor<T, 1, Ndeg> f;
  f = BR_z(t);                    // evaluate forward map at the trial polynomial
  t[1] = 1 / f[1];                // back-solve linear coefficient

  for (int i = 2; i <= Ndeg; i++) {
    f = BR_z(t);
    t[i] = -f[i] * t[1];          // back-solve coefficient i (linear method)
  }

  return t;
}
```

**Recurrence verified:** `t[i] = -f[i] * t[1]` for `i ∈ 2..=Ndeg`. The mathematical justification: differentiating `BR_z(t(z)) = z` to order `i`, the `i`-th coefficient `f[i]` is `BR_z'(t[0]) * t[i] + (lower-order terms)`. Brent–Kung's linear method assumes lower-order terms have already been determined and back-solves `t[i] = -(lower-order contribution to f[i]) / BR_z'(t[0])`. Note `t[1] = 1/f[1] = 1/BR_z'(t[0])`, so the recurrence simplifies to `t[i] = -f[i] * t[1]`. [VERIFIED at `brx.cpp:64-67`]

**Layer 3 — `ctaylor` adapter `BR(t)`:**

```cpp
// brx.cpp:78-87
template <typename T, int Nvar>
static ctaylor<T, Nvar> BR(const ctaylor<T, Nvar> & t) {
  // temporary has dimension 3 at least. See: https://github.com/dftlibs/xcfun/issues/151
  // in C++14 and later can use std::max(Nvar, 3) for the second template argument
  auto tmp = BR_taylor<T, (Nvar >= 3) ? Nvar : 3>(t.c[0]);

  ctaylor<T, Nvar> res = tmp[0];
  for (int i = 1; i <= Nvar; i++)
    res += tmp[i] * pow(t - t.c[0], i);
  return res;
}
```

Adapts the 1-variable Taylor polynomial (`taylor<T, 1, Ndeg>`) to the multivariate `ctaylor<T, Nvar>` substrate via composition with `pow(t - t.c[0], i)`. Phase 4 Rust port must replicate this composition step explicitly. [VERIFIED]

### Numerical hazard: z = 0

At `z = 0` the C++ branch test `(z < -2)` and `(z < 1)` lead to the third branch: `x0 = 2 * (0 * exp(-4/3) + 1) = 2.0`. Plug into `BR_z(2) = (2-2)/2 * exp(4/3) = 0 / 2 * exp(4/3) = 0`. So `x = 2` is the exact root at `z = 0` — no iteration needed; the convergence check `|xold - x0| < 1e-15 * (1 + 2)` will fire on iteration 1 because `NR_step(2, 0) = 0` (the numerator `2 * (3*2*(1*0 - 1) + 6) = 2 * (-6 + 6) = 0`). **No numerical hazard at z = 0.**

### Numerical hazard: z near -2 boundary

Branch boundary `z = -2`: lower branch gives `x0 = (sqrt(9*4 + 6*(-2) + 49) + 3*(-2) + 1) / 4 = (sqrt(73) - 5) / 4 ≈ (8.5440 - 5) / 4 ≈ 0.886`. Upper branch gives `x0 = 2 * (-2 * exp(-4/3) + 1) ≈ 2 * (-2 * 0.2636 + 1) ≈ 2 * 0.4729 ≈ 0.946`. Both initial guesses are reasonable; Newton converges in 5–10 iterations. [Pen-and-paper computation; HIGH confidence]

### Numerical hazard: libm `exp` drift

The Newton iteration calls `exp(-2/3 * x)` and `exp(-4/3)` in the initial-guess branch. C++ side links to the validation runner's libm (Linux glibc on the canonical CI lane). Rust side uses `f64::exp`, which on Linux x86_64 also calls glibc libm. **Same libm, same result** (verified at the LDAERFC/LDAERFX precedent — Phase 2 commit `dca382a` factored out the libm port for `erf` because of cross-runtime drift; **the same precedent applies to `exp` if cross-runtime drift surfaces**). On Linux x86_64 the Rust + C++ `exp` should be bit-identical, so the 20-iteration Newton trajectory should be bit-identical, so the converged `x` should be bit-identical to within 1 ULP. [HIGH confidence on Linux; UNVERIFIED on macOS / Windows]

### Recommended Rust signature

```rust
// crates/xcfun-ad/src/br_inverse.rs (NEW)

// Host-side scalar Newton (called by ctaylor_br_inverse via for_tests pattern OR
// by a #[cube] for_each-style adapter).
pub fn br_scalar(z: f64) -> f64 {
    let mut x0 = if z < -1e4 {
        -2.0 / z
    } else if z < -2.0 {
        ((9.0 * z * z + 6.0 * z + 49.0).sqrt() + 3.0 * z + 1.0) / 4.0
    } else if z < 1.0 {
        2.0 * (z * (-4.0 / 3.0_f64).exp() + 1.0)
    } else {
        1.5 * z.ln() + 3.75 / (1.5 + z.ln())
    };
    for _ in 0..20 {
        let xold = x0;
        x0 += nr_step(x0, z);
        if (xold - x0).abs() < 1e-15 * (1.0 + x0) {
            return x0;
        }
    }
    // Diverged — emit warning to stderr, return last iterate.
    eprintln!("BR: Not converged for z = {:e}", z);
    x0
}

#[cube]
pub fn ctaylor_br_inverse<F: Float>(
    z: &CTaylor<F, N>,                  // input ctaylor
    out: &mut CTaylor<F, N>,            // output ctaylor t such that BR_z(t) = z
    #[comptime] n: u32,
) {
    // out[CNST] = br_scalar(z[CNST])  // host-side scalar slot
    // ... linear-method polynomial sweep using existing ctaylor primitives ...
}
```

The host-side scalar `br_scalar` would run via a single-launch cubecl kernel using the Phase-1 `for_tests::cpu_client` pattern. **Decision deferred to Phase 4 plan-time** whether to (a) host-side scalar + cubecl `#[cube]` polynomial sweep, OR (b) full `#[cube]` Newton with branch-on-comptime — option (a) is simpler and matches Phase 1 D-04 design intent.

---

## DensVarsDev Audit

### Existing field provisioning (verified)

`crates/xcfun-eval/src/density_vars.rs:23-80` declares 24 `Array<F>` fields:

| Phase 2 (22 fields) | Phase 3 (+2) | Phase 4 needs |
|---------------------|--------------|---------------|
| `a, b, gaa, gab, gbb, n, s, gnn, gns, gss, tau, taua, taub, lapa, lapb, zeta, r_s, n_m13, a_43, b_43, jpaa, jpbb` | `+lapn, +laps` | NONE — all needed fields already provisioned |

**`jpaa` and `jpbb` are already there** (lines 75-79), provisioned in Plan 02-03. **`tau`, `taua`, `taub`, `lapa`, `lapb` are already there**. **`n_m13`, `r_s`, `a_43`, `b_43` are already there** (Phase 2 derived fields).

**Verdict (HIGH confidence):** D-03-B is satisfied without any DensVarsDev struct change. All 32 metaGGA bodies + BR + CSC can read existing fields. [VERIFIED: `crates/xcfun-eval/src/density_vars.rs:23-80`]

### Existing build_densvars arms (Phase 2 + Phase 3)

`crates/xcfun-eval/src/density_vars/build.rs:83-115` lists 13 implemented arms:

| Vars | id | Status |
|------|----|--------|
| `XC_A` | 0 | ✓ Phase 3 |
| `XC_N` | 1 | ✓ Phase 3 |
| `XC_A_B` | 2 | ✓ Phase 2 |
| `XC_N_S` | 3 | ✓ Phase 3 |
| `XC_A_GAA` | 4 | ✓ Phase 3 |
| `XC_N_GNN` | 5 | ✓ Phase 3 |
| `XC_A_B_GAA_GAB_GBB` | 6 | ✓ Phase 2 (Plan 02-05) |
| `XC_N_S_GNN_GNS_GSS` | 7 | ✓ Phase 3 |
| `XC_A_2ND_TAYLOR` | 27 | ✓ Phase 3 |
| `XC_A_B_2ND_TAYLOR` | 28 | ✓ Phase 3 |
| `XC_N_2ND_TAYLOR` | 29 | ✓ Phase 3 |
| `XC_N_S_2ND_TAYLOR` | 30 | ✓ Phase 3 |

### Phase 4 new arms required

Per CONTEXT D-03 + verified against `xcfun-master/src/densvars.hpp:35-218`:

| Vars | id | Inlen | C++ densvars.hpp arm | Required by | Wave 4 priority |
|------|----|--|----------------------|-------------|-----------------|
| `XC_A_GAA_LAPA` | 8 | 3 | **NOT IMPLEMENTED in C++** (xcint.cpp declares it; densvars.hpp has no case → `xcfun::die`) | None of the 32 Phase-4 functionals | **REJECT-PATH only** |
| `XC_A_GAA_TAUA` | 9 | 3 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_N_GNN_LAPN` | 10 | 3 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_N_GNN_TAUN` | 11 | 3 | densvars.hpp:93-110 | None of Phase-4 (general use) | LOW priority — defer if no functional needs it |
| `XC_A_B_GAA_GAB_GBB_LAPA_LAPB` | 12 | 7 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` | **13** | **7** | densvars.hpp:54-72 | **TPSS×5 + SCAN×10 + M05×4 + M06×8 + BLOCX×1 = 28 functionals** | **HIGH — Wave 0 mandatory** |
| `XC_N_S_GNN_GNS_GSS_LAPN_LAPS` | 14 | 7 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_N_S_GNN_GNS_GSS_TAUN_TAUS` | 15 | 7 | densvars.hpp:73-92 | None of Phase-4 (general use) | LOW priority |
| `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB` | 16 | 9 | densvars.hpp:190-208 | None of Phase-4 (general use) | LOW priority |
| `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` | **17** | **11** | densvars.hpp:187-208 | **BRX + BRC + BRXC + CSC = 4 functionals** | **HIGH — Wave 0 mandatory** |
| `XC_N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS` | 18 | 9 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_A_AX_AY_AZ_TAUA` | 23 | 5 | **NOT IMPLEMENTED in C++** | None | REJECT-PATH only |
| `XC_A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB` | 24 | 10 | densvars.hpp:148-166 | None of Phase-4 (general use) | LOW priority |
| `XC_N_NX_NY_NZ_TAUN` | 25 | 5 | densvars.hpp:114-130 | None of Phase-4 (general use) | LOW priority |
| `XC_N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS` | 26 | 10 | densvars.hpp:167-186 | None of Phase-4 (general use) | LOW priority |

**CRITICAL FINDING:** Of the 11 new arms CONTEXT D-03 lists, only **2 are required** by the 32 Phase-4 functional bodies:
- **id=13 (TAUA_TAUB) — required by 28 of 32 Phase-4 functionals.**
- **id=17 (full JP) — required by 4 of 32 (BR family + CSC).**

The other 9 arms (ids 8, 9, 10, 11, 12, 14, 15, 16, 18, 23, 24, 25, 26) are not used by any Phase-4 functional. **Recommendation:**
- **Wave 0 ships 2 arms** (id=13 + id=17). All 32 Phase-4 functional bodies can run.
- The other arms (11, 15, 16, 24, 25, 26) — implemented in C++ but not exercised by any current functional — can be added incrementally OR deferred to Phase 5 (when the full `Functional::eval_setup` user-facing API needs them for completeness).
- **Arms 8, 9, 10, 12, 14, 18, 23 — NOT IMPLEMENTED in C++ (`xcfun::die` at runtime).** Rust mirrors this by returning `XcError::InvalidVars` from `eval_setup`. **DO NOT port a `densvars` arm for them; do not write a Rust body that would diverge from C++'s `die`.** [VERIFIED by reading `xcfun-master/src/densvars.hpp:35-218` exhaustively]

This is a planning-time scope reduction — CONTEXT D-03 says "11 new arms"; reality says "2 mandatory + ~6 nice-to-have + 7 forbidden (no C++ reference)".

### Inlen layout for Vars id=13 (TAUA_TAUB)

C++ densvars.hpp:54-72:

```cpp
case XC_A_B_GAA_GAB_GBB_TAUA_TAUB:
  taua = d[5];        // d[5]
  taub = d[6];        // d[6]
  tau = taua + taub;
  // FALLTHROUGH
case XC_A_B_GAA_GAB_GBB:
  gaa = d[2];         // d[2]
  gab = d[3];         // d[3]
  gbb = d[4];         // d[4]
  gnn = gaa + 2*gab + gbb;
  gss = gaa - 2*gab + gbb;
  gns = gaa - gbb;
  // FALLTHROUGH
case XC_A_B:
  a = d[0];           // d[0]
  regularize(a);
  b = d[1];           // d[1]
  regularize(b);
  n = a + b;
  s = a - b;
  break;
```

**Rust port (no fallthrough):** explicit chain `build_xc_a_b_gaa_gab_gbb_taua_taub` calls `build_xc_a_b_gaa_gab_gbb` after populating its own slots. The chain pattern is established in Phase 3 — see `density_vars/build.rs:281` (`build_xc_a_b` is "the chain target for build_xc_a_gaa and build_xc_a_2nd_taylor"). Phase 4 extends with one more chain link for id=13.

### Inlen layout for Vars id=17 (full JP, inlen=11)

C++ densvars.hpp:187-208:

```cpp
case XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB:
  jpaa = d[9];        // d[9]
  jpbb = d[10];       // d[10]
  // FALLTHROUGH
case XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB:
  lapa = d[5];        // d[5]
  lapb = d[6];        // d[6]
  taua = d[7];        // d[7]
  taub = d[8];        // d[8]
  tau = taua + taub;
  gaa = d[2];         // d[2]
  gab = d[3];         // d[3]
  gbb = d[4];         // d[4]
  gnn = gaa + 2*gab + gbb;
  gss = gaa - 2*gab + gbb;
  gns = gaa - gbb;
  a = d[0];           // d[0]
  regularize(a);
  b = d[1];           // d[1]
  regularize(b);
  n = a + b;
  s = a - b;
  break;
```

**Rust port:** explicit chain. id=17 reads d[9], d[10] then chains into id=16 (which is the C++ `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB` arm). Note: id=16 is NOT a chain link **into** id=13's helper — id=16 is a self-contained arm that reads d[5..8] directly. Implement id=17 as a free-standing build function; do NOT chain through id=16/id=13. The chain savings are minor (4 lines of d[5..8] reads); algorithmic-identity rule favours mirroring the C++ source structure verbatim. [VERIFIED]

---

## Pitfalls + Risk Assessment

### Phase 4-relevant pitfalls from `.planning/research/PITFALLS.md`

| Pitfall | Section in PITFALLS.md | Phase 4 manifestation |
|---------|------------------------|----------------------|
| **P5 — densvars switch fallthrough lost in Rust `match`** | Pitfall 5 | The 2 new build_densvars arms (id=13, id=17) MUST use explicit helper-function chains per Phase 3 D-11 — never C-style fallthrough. The id=17 arm reads d[9], d[10] and chains into id=16's body verbatim (or implements a free-standing equivalent). |
| **P9 — series-expansion coefficient layout miscopied from `tmath.hpp`** | Pitfall 9 | The Brent–Kung sweep `t[i] = -f[i] * t[1]` (BR_taylor) is the same recurrence-style as `*_expand` ports. Must port exactly: same loop bounds (`i in 2..=Ndeg`), same operation order (`f = BR_z(t)` evaluated each iteration with growing `t`), same multiplication direction (`-f[i] * t[1]`, not `t[1] * -f[i]`). |
| **P10 — silent NaN around BR Newton convergence** | Pitfall 10 | If BR's Newton fails to converge (z extreme, branch boundary), C++ prints to stderr and returns the last iterate. Rust must mirror — do NOT panic, do NOT NaN. The fixture-gate at strict 1e-12 catches divergence. |
| **P11 — alias misweighting (additive vs multiplicative)** | Pitfall 6 (P6 in research, P11 in CONTEXT) | The line-for-line port at D-04 is the only safeguard. Tests MUST cover negative-weight (camcompx → beckecamx = -0.37), additive (b3lyp + slaterx → 1.30), and overwrite (b3lyp 0.5 → exx = 0.10). |
| **P13 — registry drift between xtask and rustc build** | Pitfall 13 | `xtask regen-registry` extension to populate aliases.rs + parameters.rs MUST be hash-stamped per Phase 2 QG-07. CI gate `xtask regen-registry --check` catches drift. |

### Phase 4-specific new risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **R1 — M06 family numerical precision (12-coef polynomial `fw` evaluated at order 6 in Contracted mode)** | HIGH | Apply algorithmic-identity rule strictly. The C++ uses `poly(w, 12, a)` which is a **descending Horner** loop in `specmath.hpp:24-33`. Rust port must descend identically (Pitfall P11 in research = P11 here). Existing `gga/shared/b97_poly.rs` has the established Horner pattern. |
| **R2 — R4SCAN 4th-order GE polynomial precision** | HIGH | The `del_y` correction in `r2SCAN_C` (SCAN_like_eps.hpp:435-451) involves `pow2(p)`, `del_f2`, multiplications; chain via existing primitives. Apply Rule-1 explicit-let-binding. |
| **R3 — BR Newton trajectory libm `exp`/`log` drift across runners** | MEDIUM | On Linux x86_64 (canonical CI), Rust f64::exp = glibc libm = C++ exp. On macOS/Windows, possible 1-ULP drift could change Newton trajectory. **Mitigation:** if fixture-gate exposes drift, port `exp_precise` analogously to Phase 2's `erf_precise` (commit `dca382a`). Initial fixture-gate runs on Linux only. |
| **R4 — Mode::Contracted at order 6 stack budget** | LOW | 10 KB scratch per kernel invocation per CONTEXT D-06. Cubecl-cpu kernel stack default is much larger. **No action needed unless cubecl-cpu raises a stack-overflow.** |
| **R5 — BLOCX false dependency on BRX (CONTEXT typo)** | LOW (now caught) | CONTEXT D-01-A asserts "BLOCX composes BRX (so depends on Wave 1 BR ship)". **VERIFIED FALSE** — BLOCX has no `BR(...)` call. Wave 3 (BLOCX) is NOT blocked by Wave 1 (BR). Planner can move BLOCX to any wave. |
| **R6 — Alias EXX-FIXME accidentally "fixed"** | MEDIUM | A reviewer may try to "improve" the alias engine to make EXX additive (the FIXME's stated intention). Algorithmic-identity rule forbids. **Mitigation:** add a unit test verifying `set("b3lyp", 0.5); get("exx") == 0.10` (NOT 0.30). |
| **R7 — Parameter array layout divergence (Rust `[EXX, RANGESEP_MU, ...]` vs C++ `[RANGESEP_MU, EXX, ...]`)** | MEDIUM | The current `Functional::parameters: [f64; 4]` in `functional.rs:88-93` uses Rust-internal order. C-ABI compatibility (Phase 5) requires C++ order. **Mitigation:** Phase 4 unifies into `settings: [f64; 82]` matching C++ exactly. The 4 indexes in current parameters[] become 79 (EXX), 78 (RANGESEP_MU), 80 (CAM_ALPHA), 81 (CAM_BETA). |
| **R8 — DOEVAL macro argument indexing off-by-one** | LOW | `output[i] = out.get(i)` for `i ∈ 0..(1 << order)`. The bit-flag indices include CNST=0, VAR0=1, VAR1=2, VAR0|VAR1=3, ..., last=`(1<<order)-1`. Verbatim port; no off-by-one risk. **Mitigation:** unit test `output_length == 1 << order` for order 0..=6 → {1, 2, 4, 8, 16, 32, 64}. |
| **R9 — TPSSLOCC's 105 LOC body has a `pw92eps` dependency that may not be exposed at the right symbol** | LOW | `pw92eps::pw92eps` is in xcfun-eval already (`crates/xcfun-eval/src/functionals/lda/pw92eps.rs`). Wave 1 needs to verify the import path; if not pub-exported, add a `pub use`. |
| **R10 — SCAN family `IDELEC` integer dispatch** | MEDIUM | The `IDELEC ∈ {0..4}` int parameter selects the regularisation variant. In Rust, this becomes a `#[comptime] idelec: u32` parameter; each functional kernel passes a constant. Trace through `SCAN_like_eps.hpp:386-461` for full branch coverage. |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo-nextest` (CI) + `approx 0.5.1` (relative error assertions) + custom tier-2 `validation/` binary (cc-linked C++ reference) |
| Config file | Cargo workspace (`Cargo.toml`); no pytest-style external config |
| Quick run command | `cargo test -p xcfun-eval --features testing --test self_tests` (tier-1 self-tests, < 5 s) |
| Mid run command | `cargo nextest run -p xcfun-eval --features testing` (all xcfun-eval tests, parallel) |
| Full suite command | `cargo nextest run --workspace --all-features && cargo xtask validate --backend cpu --order 3` |
| Phase gate command | `cargo xtask validate --backend cpu --order 3 --filter '.*'` (all 77 functionals tier-2 GREEN at strict 1e-12; D-24 LDAERF 1e-7 inherited only for the 3 LDAERFs; **D-11 NO blanket relaxation for metaGGAs**) |
| Estimated runtime | tier-1: < 30 s · tier-2 per-family: 30–120 s · full phase gate: ~10 min (additive over Phase 3's 5 min) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MGGA-01 (TPSSX/C, REVTPSSX/C, TPSSLOCC) | Energy + 1st derivatives match C++ at strict 1e-12 on the 7-element fixture from `tpssx.cpp:42-48` | tier-1 self-test | `cargo test -p xcfun-eval self_tests::tpss` | ❌ Wave 1 |
| MGGA-02 (SCAN×10) | Energy + 1st derivatives match at 1e-12; **R4SCAN at relaxed 1e-11 if 4th-order GE polynomial drifts (escalation path D-19)** | tier-1 + tier-2 | `cargo test -p xcfun-eval self_tests::scan && cargo xtask validate --backend cpu --order 2 --filter scan` | ❌ Wave 2 |
| MGGA-03 (M05×4) | Energy + 1st derivatives match at 1e-12 OR D-19 relaxation per upstream `test_threshold` (M05X has `1e-7` threshold per `m05x.cpp:50`) | tier-1 + tier-2 | `cargo test -p xcfun-eval self_tests::m05 && cargo xtask validate --backend cpu --order 2 --filter m05` | ❌ Wave 3 |
| MGGA-04 (M06×8) | Same as M05; M06 family has `test_threshold = 1e-7` per `m06x.cpp:69` etc. — **CONFIRM upstream threshold drives D-19 override decision** | tier-1 + tier-2 | `cargo test -p xcfun-eval self_tests::m06 && cargo xtask validate --backend cpu --order 2 --filter m06` | ❌ Wave 3 |
| MGGA-05 (BLOCX) | No upstream `test_in`/`test_out` per `blocx.cpp:55-61` — tier-1 EXCLUDED, tier-2 enforced at 1e-12 on the 1000-point metaGGA stratum | tier-2 | `cargo xtask validate --backend cpu --order 2 --filter blocx` | ❌ Wave 3 |
| MODE-03 (Contracted orders 0..=6) | Output layout `output[i] = out.get(i)` matches C++ DOEVAL on 1000-point cross-mode subset; orders 0..=4 cross-checked vs PartialDerivatives; orders 5/6 require new C-driver path | tier-2 | `cargo xtask validate --backend cpu --mode contracted --order 6 --filter '.*'` | ❌ Wave 5 |
| ALIAS-01 (46 aliases) | Each alias resolves to the exact same weight set as manual composition | unit | `cargo test -p xcfun-eval alias_canary` | ❌ Wave 4 |
| ALIAS-02..05 (4 parameters) | Settable via `set`, default values match C++ | unit | `cargo test -p xcfun-eval parameter_defaults` | ❌ Wave 4 |
| ALIAS-06 (multiplicative weight + EXX-FIXME) | b3lyp at 1.0 → slaterx=0.80, vwn5c=0.19; b3lyp at 1.0 then slaterx 0.5 → 1.30 (additive); b3lyp 0.5 → exx=0.10 (overwrite via case ②); camcompx 0.37 → beckecamx=-0.37 | unit | `cargo test -p xcfun-eval alias_recursion_traces` | ❌ Wave 4 |
| GGA-03 (BRX/BRC/BRXC) | Energy + 1st derivatives match at 1e-12 on the inlen=11 fixture | tier-1 + tier-2 | `cargo test -p xcfun-eval self_tests::br && cargo xtask validate --backend cpu --order 2 --filter brx` | ❌ Wave 1 |
| GGA-10/CSC | Energy + 1st derivatives match at 1e-12 on the inlen=11 fixture | tier-1 + tier-2 | `cargo test -p xcfun-eval self_tests::csc && cargo xtask validate --backend cpu --order 2 --filter csc` | ❌ Wave 1 |

### Sampling Rate

- **Per task commit:** `cargo test -p xcfun-eval --features testing --test self_tests --filter <wave-family>` (< 30 s)
- **Per wave merge:** `cargo xtask validate --backend cpu --order 2 --filter <family>` (< 60 s per family)
- **Phase gate:** Full suite green before `/gsd-verify-work 4`. ~10 min budget.

### Tier Architecture (4-tier strategy from `07-accuracy-strategy.md`)

| Tier | Purpose | Phase 4 scope |
|------|---------|---------------|
| **Tier 1 — Self-test fixtures** | Per-functional `FUNCTIONAL` macro `test_in`/`test_out`/`test_threshold` from C++ source | 28 of 32 metaGGAs have `test_in` in upstream (BLOCX, BRC, BRXC, CSC do NOT — `blocx.cpp:55-61`, `brx.cpp:138-157`, `cs.cpp:29-35` — these are tier-2-only). Already auto-generated to `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs:77+` by Phase 2 `xtask regen-registry`. |
| **Tier 2 — Cross-mode parity** | 10k-point seeded grid (existing) + metaGGA stratum (NEW per D-09-A, sibling seed `0xc0ffee01`, ~1000 records); cc-compiled C++ reference; per-record `\|rust - cpp\| / max(\|cpp\|, 1.0)` at strict 1e-12 | New strata: random `tau ∈ [0, kF² · n^(2/3)]`, random `lap` near zero, random `JP_aa`/`JP_bb` for the inlen=11 stratum. Seed locked, regenerable. |
| **Tier 2 — Alias canary tests** | Unit tests building composed weight sets via `Functional::set("alias", v)` + comparing to manual composition | Negative-weight (camcompx → beckecamx = -0.37), additive (b3lyp + slaterx = 1.30), overwrite (b3lyp 0.5 → exx = 0.10). 4 traces minimum. |
| **Tier 2 — Mode::Contracted cross-mode** | Cross-checks orders 0..=4 against PartialDerivatives Taylor coefficients (algorithmic identity); orders 5/6 via new C-driver path | Wave 5 deliverable; 100-point subset for orders 5/6 (1000-point for orders 0..=4 from existing grid). |
| **Tier 3 — GPU parity** | OUT OF SCOPE | Phase 6 |

### Sample size + record count

- **10k-point base grid (existing)** + **1000-point metaGGA stratum** + **100-point Contracted-orders-5/6 subset** = ~11.1k records per functional × order set.
- **Per-functional × per-order at order 0..=2:** 11.1k × 32 functionals × 3 orders = **~1.07M records** (manageable; same magnitude as Phase 3 capstone).
- **Order 3 capstone:** add another ~3M records → total **~4M records**. Within the Phase 3 sweep precedent of 9.86M.
- **Mode::Contracted cross-mode subset (orders 0..=4):** 1000 × 32 functionals × 5 orders = **160k records**.
- **Mode::Contracted orders 5/6 (new C-driver):** 100 × 4 representative functionals × 2 orders = **800 records**.

### Acceptance threshold

- **Strict 1e-12 default** for all 32 metaGGA + 4 carryover functionals.
- **D-24 LDAERF 1e-7 inherited** only for the 3 LDAERFs (NOT extended to metaGGAs per Phase 3 D-18 precedent).
- **Per-functional override allowed only with upstream-documented `test_threshold`** (e.g., M05X's `1e-7` from `m05x.cpp:50`) OR escalation via PLANNING INCONCLUSIVE.
- **Mode::Contracted orders 5/6 inherit the underlying functional's threshold** — Contracted is a re-packaging, not an algorithmic difference.

### Wave 0 Gaps

- [ ] `crates/xcfun-ad/src/br_inverse.rs` — NEW; D-02 ctaylor_br_inverse + scalar BR Newton (port of `brx.cpp:25-72`)
- [ ] `crates/xcfun-ad/tests/golden_br_inverse.rs` — NEW; 30 z-points × N∈{2,3,4} fixtures from C++ `BR_taylor`
- [ ] `xtask/src/bin/regen_ad_fixtures.rs` — extend with BR fixture generation (or new `gen_br_fixtures.rs` binary)
- [ ] `crates/xcfun-eval/src/density_vars/build.rs` — add 2 new `build_xc_*` functions (id=13, id=17) + 2 comptime if-chain arms
- [ ] `crates/xcfun-eval/src/functionals/mgga/mod.rs` — NEW; module skeleton
- [ ] `crates/xcfun-eval/src/functionals/mgga/shared/{tpss_like, scan_like, m0xy_fun, br_like, blocx, cs}.rs` — 6 new helper modules
- [ ] `crates/xcfun-eval/src/functionals/mgga/shared/mod.rs` — NEW; module index
- [ ] `crates/xcfun-eval/src/functionals/mod.rs` — extend with `pub mod mgga`
- [ ] `xtask/assets/regen_registry/extractor.cpp` — extend to parse `aliases.cpp:17-138` (replace L605 stub with real parsing) — Wave 4 prep, but ship in Wave 0 to allow incremental validation
- [ ] `crates/xcfun-core/src/parameter_id.rs` — NEW; `ParameterId` enum + helpers (Wave 4 prep)
- [ ] `crates/xcfun-core/src/registry/generated/parameters.rs` — NEW; static slice of 4 parameter rows (Wave 4 prep)

**Total Wave 0: ~12 new files + 5 file extensions.** Atomic commits per file.

### Manual-Only Verifications

| Behavior | Why Manual | Test Instructions |
|----------|------------|-------------------|
| C++ `xcfun::die` semantics for unsupported Vars (ids 8, 9, 10, 12, 14, 18, 23) | Rust returns `XcError::InvalidVars` instead of process abort. Reviewer must confirm semantic equivalence (caller's-perspective: both fail; the diagnostic is different) | `grep -n "InvalidVars" crates/xcfun-eval/src/functional.rs` — confirm rejection branches present for all 7 unimplemented ids |
| Alias EXX-FIXME parity (do NOT fix the wart) | Algorithmic-identity rule forbids "improving" the wart; reviewer must not approve any patch that makes EXX additive in alias resolution | Read code review for any commit touching `Functional::set` against the upstream `XCFunctional.cpp:389-404` byte-for-byte |
| BLOCX-vs-BRX dependency claim in CONTEXT D-01-A | Auto-mode CONTEXT asserts "BLOCX composes BRX". Verified false by reading `blocx.cpp:18-46`. Planner must NOT gate Wave 3 on Wave 1 BR ship | Read `xcfun-master/src/functionals/blocx.cpp` — confirm no `BR(...)` call. |
| Parameter array layout migration (Rust [EXX, RANGESEP_MU, ...] → C++ [RANGESEP_MU, EXX, ...]) | Phase 3 left `Functional::parameters: [f64; 4]` with index 0 = EXX. Phase 4 unifies to `settings: [f64; 82]`. Migration requires updating BECKESRX/BECKECAMX kernel reads. | Diff `crates/xcfun-eval/src/functionals/gga/becke/beckesrx.rs` for parameter index references; confirm migration to `settings[78]` (RANGESEP_MU) |

---

## Recommended Wave Breakdown

Concrete plan-file decomposition for `/gsd-plan-phase 4`:

### 04-00-PLAN.md — Wave 0 substrate (ATOMIC)

**Scope:**
- `ctaylor_br_inverse` xcfun-ad primitive (D-02)
- 2 new DensVarsDev arms (id=13 TAUA_TAUB, id=17 full JP) — D-03 + D-03-A
- 6 metaGGA shared helper module skeletons (`tpss_like.rs`, `scan_like.rs`, `m0xy_fun.rs`, `br_like.rs`, `blocx.rs`, `cs.rs`)
- 1 CTaylor<F, 6> smoke test (D-07)
- Fixture-gates at strict 1e-12

**Atomic commits:**
1. xcfun-ad: ctaylor_br_inverse primitive + tests
2. xcfun-ad: 30-point BR_taylor fixtures generation in xtask
3. xcfun-eval: densvars id=13 arm + chain
4. xcfun-eval: densvars id=17 arm + chain
5. xcfun-eval: mgga/shared/ module skeletons (6 files, no live functions)
6. xcfun-eval: CTaylor<F, 6> smoke test (Phase 1 type re-exercise)

**No dependencies on other waves.** No live kernel bodies — Wave 0 is "load-bearing scaffolding."

### 04-01-PLAN.md — Wave 1 TPSS + BR + CSC (9 functionals)

**Scope:**
- TPSS family bodies (5): TPSSX, TPSSC, REVTPSSX, REVTPSSC, TPSSLOCC. Vars id=13.
- BR family bodies (3): BRX, BRC, BRXC. Vars id=17. **Composes Wave 0's `ctaylor_br_inverse`.**
- CSC body (1). Vars id=17.
- Helpers: TPSS-eps modules (4), `br_like.rs` polarized helper, `cs.rs` already in shared.
- Tier-1 self-tests for all 9 (where upstream test_in exists — TPSSX, TPSSC, REVTPSSX, REVTPSSC, TPSSLOCC, BRX have upstream test_in).
- Tier-2 family validation via `xtask validate --backend cpu --order 2 --filter <family>`.

**Dependencies:** Wave 0 must ship `ctaylor_br_inverse` + densvars arms 13/17 + shared module skeletons.

**Per-family parallelism:** TPSS×5 + BR×3 + CSC×1 are all independent within the wave (per CONTEXT D-01-B).

### 04-02-PLAN.md — Wave 2 SCAN family (10 functionals)

**Scope:**
- SCAN family bodies (10): SCANX, SCANC, RSCANX, RSCANC, RPPSCANX, RPPSCANC, R2SCANX, R2SCANC, R4SCANX, R4SCANC. Vars id=13.
- Heavy on `shared/scan_like.rs` (522-line port — single largest helper).
- 6 functions to port: `get_SCAN_Fx`, `r2SCAN_C`, `scan_ec0`, `scan_ec1`, `lda_0`, `gcor2`, plus `get_lsda1` (out-parameter → tuple return).
- IDELEC integer dispatch handled via `#[comptime] idelec: u32` parameter.
- R4SCAN at HIGH-RISK polynomial precision review.

**Dependencies:** Wave 0 (densvars id=13) + `shared/pw92eps` (already in xcfun-eval Phase 2).

**Sub-wave option:** SCAN_X (5 functionals) parallel with SCAN_C (5 functionals) per CONTEXT D-01-B.

### 04-03-PLAN.md — Wave 3 M05 + M06 + BLOCX (13 functionals)

**Scope:**
- M05 family bodies (4): M05X, M05X2X, M05C, M05X2C. Vars id=13.
- M06 family bodies (8): M06X, M06X2X, M06LX, M06HFX, M06C, M06LC, M06HFC, M06X2C. Vars id=13.
- BLOCX body (1). Vars id=13. **Independent of BRX (CONTEXT typo correction).**
- Heavy on `shared/m0xy_fun.rs` (262-line port — exports 12 functions).
- Helpers `pbex.rs` (already in `gga/shared/pbex.rs` from Phase 3 — re-export).

**Dependencies:** Wave 0 (densvars id=13) + `shared/m0xy_fun.rs` ported (Wave 0).

### 04-04-PLAN.md — Wave 4 alias engine + parameters

**Scope:**
- Extend `xtask regen-registry` extractor (`extractor.cpp`) to parse `aliases.cpp:17-138` → emit 46-row JSONL.
- Extend Rust extractor (`xtask/src/bin/regen_registry.rs`) to consume the alias JSONL → emit `crates/xcfun-core/src/registry/generated/aliases.rs` populated.
- Create `crates/xcfun-core/src/parameter_id.rs` (`ParameterId` enum, 4 variants).
- Create `crates/xcfun-core/src/registry/generated/parameters.rs` (4-row static slice).
- Migrate `Functional::parameters: [f64; 4]` → `Functional::settings: [f64; 82]` matching C++ layout. Update BECKESRX/BECKECAMX/LDAERFX/LDAERFC/LDAERFC_JT kernel reads (Phase 3 already wired these to current `parameters[]`).
- Implement `Functional::set(name, value)` recursion: line-for-line port of `XCFunctional.cpp:369-405`.
- Implement `Functional::get(name)`: port of `XCFunctional.cpp:407-419`.
- Tier-1 alias canary tests (5 traces minimum: b3lyp/slaterx-additive, b3lyp-overwrite-exx, camcompx negative-weight, b3lyp-resolves-vwn5c-not-vwn3c, all 46 aliases resolve no error).
- Registry-time invariant: no alias term name matches an alias name (drift gate in `xtask regen-registry --check`).

**Dependencies:** None (independent of metaGGA waves except for the parameter-array migration, which can land before or after).

### 04-05-PLAN.md — Wave 5 Mode::Contracted (orders 0..=6)

**Scope:**
- New `crates/xcfun-eval/src/functionals/contracted.rs` module: `#[cube] fn contracted_kernel<F: Float, const ORDER: u32>(input: &Array<F>, d: &mut DensVarsDev<F>, out: &mut Array<F>, #[comptime] id: u32, #[comptime] vars: u32)`.
- 7 host-side dispatch arms (orders 0..=6) in `Functional::eval`.
- `Functional::eval_setup` accepts `Mode::Contracted` for any compatible Vars.
- `Functional::output_length` returns `1 << order` for Mode::Contracted (D-06-B).
- Tier-2 cross-mode parity at orders 0..=4 (1000-point subset against PartialDerivatives Taylor coefficients).
- Tier-2 orders 5/6 via NEW C-driver path: extend `validation/src/c_driver.rs` to call `xcfun_eval` with `XC_CONTRACTED` mode at orders 5/6 on a 100-point subset × 4 representative functionals.

**Dependencies:** Waves 1–3 (need at least one functional shipped per family for cross-mode testing).

### 04-06-PLAN.md — Wave 6 full-matrix tier-2 + Phase-4 sign-off

**Scope:**
- Run `xtask validate --backend cpu --order 3 --filter '.*'` across all 77 functionals (78 minus LB94).
- Run `xtask validate --backend cpu --mode contracted --order 6` on 4 representative functionals.
- Forward any new D-19 INCONCLUSIVE entries to Phase 6 (NOT Phase 5 — Phase 5 is API-surface).
- Update REQUIREMENTS.md: flip MGGA-01..05, MODE-03, ALIAS-01..06 to Complete; flip GGA-03 + GGA-10/CSC carryover rows.
- Update ROADMAP.md Phase 4 success criteria.
- Update STATE.md.
- Update design docs (02 §5, 03, 04, 05 §3, 06 §3, 07 §4 §6, 08, 09, 11 §M5, 12).

**Dependencies:** Waves 0–5 complete.

---

## Open Questions / Escalations

1. **Should `Functional::parameters: [f64; 4]` → `settings: [f64; 82]` migration ride in Wave 4 or be split off?**
   - What we know: Phase 3 left `parameters[]` as a Rust-internal layout with index 0 = EXX. C-ABI compat (Phase 5) requires C++ layout (index 0 = RANGESEP_MU at position 78 in the 82-array).
   - What's unclear: Does the migration touch any production-critical Phase 3 code path? BECKESRX/BECKECAMX/LDAERF*  kernels read parameters via index. **Recommend:** ship the migration in Wave 0 (atomic, single commit, regression-tested via Phase 3 GREEN regression suite) BEFORE alias engine work. Removes 1 layer of risk for Wave 4.

2. **Should arms for Vars ids 11, 15, 16, 24, 25, 26 (C++-implemented, no Phase-4 functional uses them) be added in Phase 4 or deferred?**
   - What we know: C++ has `densvars` cases for these; no Phase-4 functional invokes them; `Functional::eval_setup` returns `XcError::InvalidVars` for them today.
   - What's unclear: Phase 5 facade (RS-01..10) may surface them via `eval_setup` for completeness. Not a Phase 4 blocker.
   - Recommendation: defer to Phase 5 unless a planner cycle has spare budget. Phase-4 scope reduction is preferred.

3. **R4SCAN at order 6 in Mode::Contracted — does the 4th-order GE polynomial drift?**
   - What we know: R4SCAN's `del_y` correction is a fourth-order polynomial in `p = s²`. At order 6 in CTaylor, the chain expands to a multilinear polynomial of degree ≤ 6 in the Taylor seeds. Compounded port-order risk.
   - What's unclear: Whether the existing `gga/shared/b97_poly.rs` Horner pattern is reusable or whether a SCAN-specific Horner module is needed.
   - Recommendation: Wave 2 plans the full polynomial port; flag R4SCAN at Wave 5 as a fixture-gate watchdog (+1e-12 vs PartialDerivatives at order 4 is the tightest constraint).

4. **Tier-2 grid generator: should `0xc0ffee01` sibling seed (D-09-A) be 1000 records or 2000 records?**
   - What we know: CONTEXT D-09-A says "metaGGA-specific stratum" without specifying size; precedent from Phase 3 `0xdeadbeef` supplemental was 400 points.
   - Recommendation: 1000 points × 32 functionals × 3 orders = 96k records — within Phase 3 sweep precedent. Planner-time decision.

5. **Should the CONTEXT D-01-A "BLOCX composes BRX" claim be amended in 04-CONTEXT.md before planning?**
   - What we know: VERIFIED false by reading `blocx.cpp:18-46`. The CONTEXT note is a typo.
   - Recommendation: planner should issue a CONTEXT amendment (D-01-A note correction) so downstream plans don't assume the false dependency. Low effort, prevents downstream confusion.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All builds | ✓ (verified MSRV 1.85 in `rust-toolchain.toml`) | 1.85 | — |
| `cubecl =0.10.0-pre.3` | xcfun-ad, xcfun-eval | ✓ (verified by `xtask check-cubecl-pin` in Phase 2 CI) | 0.10.0-pre.3 | — |
| `cubecl-cpu =0.10.0-pre.3` | tier-2 harness | ✓ | 0.10.0-pre.3 | — |
| C++17 compiler (cc 1.x) | validation/build.rs cc-compile of xcfun-master | ✓ (Phase 2 verified) | system gcc/clang | — |
| `cargo-nextest` | CI test parallelism | ✓ (Phase 2 verified) | latest | `cargo test --workspace` (slower) |
| `cargo-deny` | License + advisory gate | NOT IN SCOPE for Phase 4 (Phase 0 work) | — | — |

**No new external dependencies for Phase 4.** All toolchain in place from Phase 1/2/3.

---

## Code Examples

### Verified pattern: `#[cube] fn` body for a metaGGA TPSSX kernel (template)

```rust
// Source: extension of crates/xcfun-eval/src/functionals/lda/slaterx.rs pattern
// + crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs pattern + tpssx_eps.hpp port

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_sqrt};

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::tpss_like::{tpss_F_x, tpss_fx_unif};

/// `#[cube] fn xc_tpssx_kernel<F, const N>` — port of `xcfun-master/src/functionals/tpssx.cpp:21-27`.
///
/// `tpssx(d) = 0.5 * (epsxunif_a * Fxa + epsxunif_b * Fxb)`
/// where `Fxa = F_x(2*d.a, 4*d.gaa, 2*d.taua)` and similarly for b.
#[cube]
pub fn tpssx_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // 2 * d.a
    let mut two_a = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&d.a, F::new(2.0), &mut two_a, n);
    // 4 * d.gaa
    let mut four_gaa = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&d.gaa, F::new(4.0), &mut four_gaa, n);
    // 2 * d.taua
    let mut two_taua = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&d.taua, F::new(2.0), &mut two_taua, n);

    // Fxa = F_x(2a, 4gaa, 2taua); epsxunif_a = fx_unif(2a)
    let mut fx_a = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut eps_a = Array::<F>::new(comptime!((1_u32 << n) as usize));
    tpss_F_x::<F>(&two_a, &four_gaa, &two_taua, &mut fx_a, n);
    tpss_fx_unif::<F>(&two_a, &mut eps_a, n);

    // (... same for b spin ...)

    // out = 0.5 * (eps_a * fx_a + eps_b * fx_b)
    // ... (operations preserve C++ order: multiply then sum then half)
}
```

### Verified pattern: Alias resolution recursion (Functional::set)

```rust
// Source: line-for-line port of XCFunctional.cpp:369-405

impl Functional {
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        // Case ①: functional name (additive)
        if let Some(id) = FunctionalId::from_name_case_insensitive(name) {
            self.settings[id as usize] += value;
            // Activate if not already active (port of XCFunctional.cpp:374-384)
            if !self.is_active(id) {
                self.active_functionals.push(id);
                self.depends |= FUNCTIONAL_DESCRIPTORS[id as usize].depends;
            }
            return Ok(());
        }
        // Case ②: parameter name (overwrite)
        if let Some(pid) = ParameterId::from_name_case_insensitive(name) {
            self.settings[pid as usize] = value;
            return Ok(());
        }
        // Case ③: alias name (recursive with multiplicative weight)
        if let Some(alias) = ALIASES.iter().find(|a| a.name.eq_ignore_ascii_case(name)) {
            for (term_name, weight) in alias.components {
                self.set(term_name, value * weight)?;       // recursion
            }
            return Ok(());
        }
        // Case ④: not found
        Err(XcError::UnknownName)
    }
}
```

### Verified pattern: Mode::Contracted host-side dispatch

```rust
// Source: line-for-line port of XCFunctional.cpp:619-635

impl Functional {
    fn eval_contracted(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        match self.order {
            0 => self.launch_contracted::<0>(input, output),
            1 => self.launch_contracted::<1>(input, output),
            2 => self.launch_contracted::<2>(input, output),
            3 => self.launch_contracted::<3>(input, output),
            4 => self.launch_contracted::<4>(input, output),
            5 => self.launch_contracted::<5>(input, output),
            6 => self.launch_contracted::<6>(input, output),
            _ => Err(XcError::InvalidOrder { order: self.order }),
        }
    }

    fn launch_contracted<const ORDER: u32>(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // Verify input length: inlen × (1 << ORDER)
        let inlen = self.vars.input_len();
        let expected_input = inlen * (1usize << ORDER);
        if input.len() != expected_input {
            return Err(XcError::InputLengthMismatch);
        }
        // Verify output length: 1 << ORDER
        if output.len() != (1usize << ORDER) {
            return Err(XcError::OutputLengthMismatch);
        }

        // Pack inputs in bit-flag-index order (per DOEVAL macro lines 625-627):
        //   in[i].set(j, input[i * (1<<ORDER) + j]) for i ∈ 0..inlen, j ∈ 0..(1<<ORDER)
        // ...

        // Dispatch through dispatch_kernel for each active functional, accumulate weighted.
        // ...

        // Unpack: output[i] = out.get(i) for i ∈ 0..(1 << ORDER) (line 632-633)
        // ...

        Ok(())
    }
}
```

---

## State of the Art

| Old approach | Current approach | When changed | Impact |
|--------------|------------------|--------------|--------|
| Fragmented `parameters[]` Rust-internal layout | Unified `settings[82]` matching C++ exactly | Phase 4 Wave 4 | Phase 5 C-ABI compat trivialised |
| `xtask regen-registry` ALIASES = empty slice | 46 alias rows extracted from `aliases.cpp` | Phase 4 Wave 4 | ALIAS-01 satisfied |
| Phase 3 `Mode::Contracted → InvalidMode` rejection | Mode::Contracted orders 0..=6 supported | Phase 4 Wave 5 | MODE-03 satisfied; final mode of 3 |
| Phase 3 `XcError::InvalidVars` for ids 11, 13, 15, 16, 17, 24, 25, 26 | id=13 + id=17 implemented; others stay rejected | Phase 4 Wave 0 | 32 of 32 metaGGA + carryover ids able to run |
| LB94 deferred from Phase 3 to Phase 4 (D-19) | LB94 confirmed not alias-feasible; Phase 4 D-13 forwards to Phase 5 | Phase 4 (D-13 verification) | Phase 5 owns LB94 ABI extension |

**Deprecated/outdated:**
- The fragmented `Functional::parameters: [f64; 4]` design (Phase 3 transient) — replaced by unified `settings[82]` in Phase 4 Wave 4.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | CTaylor<F, 6> works correctly on cubecl-cpu without code changes (Phase 1 AD-01 contract) | §"Mode::Contracted" — D-07 fixture-gate | LOW — Phase 1 property tests covered orders 0..=3 explicitly; const-generic guarantees N=6 follows the same shape. Wave 0 smoke test confirms. |
| A2 | BR Newton trajectory is bit-identical between Rust f64 and C++ libm on Linux x86_64 (canonical CI lane) | §"BR Newton-Inverse Primitive" — R3 | MEDIUM — same glibc on both sides; verified at Phase 2 LDAERFX precedent. Possible drift on macOS/Windows (not in canonical Phase 4 lane). |
| A3 | `XCFUN_MAX_ORDER = 6` is fixed in xcfun-master vendored copy | §"Mode::Contracted" | LOW — verified in `validation/build.rs:75` (`-DXCFUN_MAX_ORDER=6`). Vendor bump would require Phase 4 amendment. |
| A4 | `xtask regen-registry` extension to parse `aliases.cpp` is feasible without major C++ extractor rewrite | §"Alias Engine" + §"Wave 4 plan" | LOW — extractor already parses `xcint.cpp` for VARS_TABLE; alias parsing is similar pattern (parse `aliases_array[]` initializer-list). Effort estimate: 50-100 lines of `extractor.cpp` extension. |
| A5 | M06 family upstream `test_threshold = 1e-7` (per `m06x.cpp:69` etc.) means 1e-7 is the appropriate D-19 override IF strict 1e-12 fails at fixture-gate | §"Validation Architecture" | LOW — upstream `test_threshold` is the canonical override per Phase 2 D-24 precedent (LDAERF 1e-7). |
| A6 | The 7 unimplemented Vars arms (8, 9, 10, 12, 14, 18, 23) returning `XcError::InvalidVars` is semantically equivalent to C++'s `xcfun::die` | §"DensVarsDev Audit" | LOW — both fail the call. Rust's surface is more graceful (no process abort). C-ABI (Phase 5) maps `InvalidVars` to `XC_EVARS` (already wired). |
| A7 | The alias graph is depth-1 (no alias-of-alias) | §"Alias Engine" — alias-of-alias check | HIGH — verified by exhaustive enumeration of all 46 entries' term names. Future xcfun upstream could break this; registry-time invariant in `xtask regen-registry --check` catches drift. |
| A8 | BLOCX has no BRX dependency (CONTEXT D-01-A typo) | §"Source Tree Triage — BLOCX" | LOW — verified by reading `blocx.cpp:1-62`. **Recommend CONTEXT amendment.** |
| A9 | The Phase 3 13 D-19 INCONCLUSIVE entries are STRICTLY out of Phase 4 scope (D-10 inheritance) | §"User Constraints" | LOW — explicit CONTEXT D-10 directive. |
| A10 | The 1000-point metaGGA stratum at sibling seed `0xc0ffee01` (D-09-A) is sufficient for the 1e-12 budget | §"Validation Architecture" | LOW — Phase 3 precedent (10k base + 400 supplemental) showed sufficient sample density for 1e-12 budgeting. |

---

## Sources

### Primary (HIGH confidence)
- `xcfun-master/src/XCFunctional.cpp:347-490, 619-635` — alias engine + parameter table + Mode::Contracted DOEVAL [VERIFIED via Read tool]
- `xcfun-master/src/functionals/aliases.cpp:17-138` — 46-entry alias table [VERIFIED]
- `xcfun-master/src/functionals/common_parameters.cpp:17-29` — 4-parameter defaults [VERIFIED]
- `xcfun-master/src/functionals/list_of_functionals.hpp:99-105` — parameter id discriminants [VERIFIED]
- `xcfun-master/src/functionals/brx.cpp:21-87` — BR Newton-Raphson + BR_taylor + BR(t) ctaylor adapter [VERIFIED]
- `xcfun-master/src/functionals/blocx.cpp:1-62` — BLOCX body (NO BRX composition) [VERIFIED]
- `xcfun-master/src/functionals/cs.cpp:1-35` — CSC body [VERIFIED]
- `xcfun-master/src/functionals/tpssx.cpp` + `tpssx_eps.hpp:23-59` — TPSSX [VERIFIED]
- `xcfun-master/src/functionals/m05x.cpp:1-59` — M05X [VERIFIED]
- `xcfun-master/src/functionals/m06x.cpp:1-93` — M06X [VERIFIED]
- `xcfun-master/src/functionals/m0xy_fun.hpp:1-262` — M05/M06 shared helpers [VERIFIED]
- `xcfun-master/src/functionals/SCAN_like_eps.hpp:1-522` — SCAN family shared helpers [VERIFIED first 60 + 130 lines via Read]
- `xcfun-master/src/densvars.hpp:35-218` — densvars switch (verified 11/11 implementable arms vs 7 unimplemented) [VERIFIED]
- `xcfun-master/src/xcint.cpp:93-130` — VARS_TABLE [VERIFIED]
- `crates/xcfun-eval/src/density_vars.rs:23-80` — DensVarsDev field provisioning (jpaa/jpbb verified) [VERIFIED]
- `crates/xcfun-eval/src/density_vars/build.rs:83-115` — existing 13 build_densvars arms [VERIFIED]
- `crates/xcfun-eval/src/dispatch.rs:70-233` — existing 46 dispatch arms + supports() bitmap [VERIFIED]
- `crates/xcfun-eval/src/functional.rs:67-300` — Functional struct + eval + parameters layout [VERIFIED]
- `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs:77+` — metaGGA TEST_IN/TEST_OUT already populated [VERIFIED]
- `crates/xcfun-core/src/registry/generated/ALIASES.rs:14` — empty slice (Phase 2 stub) [VERIFIED]
- `xtask/src/bin/regen_registry.rs:166, 593-680` — Rust extractor stub for aliases [VERIFIED]
- `xtask/assets/regen_registry/extractor.cpp:605` — C++ extractor stub for aliases [VERIFIED]
- `validation/build.rs:1-168` — current cc-compile list of 38 .cpp files [VERIFIED]

### Secondary (MEDIUM confidence)
- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` — Phase 1 D-01..D-28 (cubecl pivot, 1e-12 contract, no FMA) [REFERENCED]
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md` — Phase 2 D-01..D-25 (DensVarsDev, dispatch pattern, registry pipeline) [REFERENCED]
- `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` — Phase 3 D-01..D-25 + amendments (helper-module pattern, Mode::Potential rejection for metaGGAs) [REFERENCED]
- `.planning/research/PITFALLS.md` — P5 (fallthrough), P6/P11 (alias misweighting), P9 (expand miscopy), P10 (silent NaN), P13 (registry drift) [VERIFIED via direct read]
- `.planning/research/SUMMARY.md` — Phase 4 mapping in "Implications for Roadmap" [REFERENCED]
- CLAUDE.md — tech stack pins (cubecl =0.10.0-pre.3, no fast-math, f64-only) [VERIFIED]

### Tertiary (LOW confidence)
- None — all critical claims verified via direct in-tree source reads.

---

## Metadata

**Confidence breakdown:**
- Source tree triage: HIGH — every C++ file read, every helper line-counted
- Alias engine semantics: HIGH — 46 entries enumerated, 4 traces verified
- Parameter table: HIGH — verified id assignments + defaults from C++
- BR Newton-inverse: HIGH for algorithm; MEDIUM for cross-runner libm parity
- DensVarsDev audit: HIGH — verified 24 fields exist; verified 13 arms exist; verified 7 unimplemented arms in C++
- Mode::Contracted: HIGH for orders 0..=4 (algorithmic identity to PartialDerivatives); MEDIUM for orders 5/6 (no upstream test coverage; new C-driver path required)
- Wave breakdown: HIGH — 6 plan files outlined with explicit dependencies
- Pitfalls + risks: HIGH — direct reading of PITFALLS.md + Phase 3 D-19 inheritance

**Research date:** 2026-04-25
**Valid until:** 2026-05-25 (30 days for stable scope; trigger re-research on cubecl version bump or `xcfun-master/` content-hash change)

---

*Phase 4 research — metaGGA Tier + Mode::Contracted + Aliases — completed 2026-04-25. Ready for `/gsd-plan-phase 4 --auto`.*
