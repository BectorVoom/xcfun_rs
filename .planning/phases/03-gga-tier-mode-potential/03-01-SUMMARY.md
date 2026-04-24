---
phase: 03-gga-tier-mode-potential
plan: 01
subsystem: xcfun-eval
tags: [xcfun-eval, densvars, gga-substrate, shared-helpers, 2nd-taylor, mode-potential]

# Dependency graph
requires:
  - phase: 02-core-foundations-lda-tier-parity-harness
    provides: "DensVarsDev struct + build_densvars XC_A_B/XC_A_B_GAA_GAB_GBB arms + regularize + Functional::eval dispatcher"
  - phase: 03-gga-tier-mode-potential
    provides: "Plan 03-00 Wave-0 substrate: ctaylor_expm1, ctaylor_sqrtx_asinh_sqrtx, expm1_expand"
provides:
  - "crates/xcfun-eval/src/functionals/gga/ module tree with shared/ submodule (6 files, 5 #[cube] fn helper modules + 1 constants module)"
  - "36 f64 GGA scalar constants verbatim-ported from xcfun-master/src/functionals/{pbex.hpp, pw9xx.hpp, constants.hpp, beckex.cpp, lypc.cpp, optx.cpp, ktx.cpp, btk.cpp, b97xc}"
  - "gga::shared::pbex::enhancement + energy_pbe_ab FULL BODIES (consumers in 03-02/03/04 import these)"
  - "gga::shared::pw91_like::s2 FULL BODY (called by pbex::enhancement)"
  - "8 SKELETON helpers with W3 consumer-pointer comments (chi2, prefactor, pw91k_prefactor, pw91xk_enhancement, enhancement_rpbe, a_expm1_inner, h_gga, phi, ux_ab, b97_enhancement, g_xa2, optx_enhancement)"
  - "DensVarsDev struct extended: 22 → 24 fields (pub lapn + pub laps added after lapb per B2)"
  - "10 new build_xc_* helpers: 3 low-level (build_xc_a / build_xc_n / build_xc_n_s — W4), 3 GGA (build_xc_a_gaa / build_xc_n_gnn / build_xc_n_s_gnn_gns_gss), 4 2ND_TAYLOR (build_xc_a_2nd_taylor / build_xc_a_b_2nd_taylor / build_xc_n_2nd_taylor / build_xc_n_s_2nd_taylor)"
  - "build_densvars comptime if-chain extended with 7 new vars arms (discriminants 4, 5, 7, 27, 28, 29, 30 per D-10-A)"
  - "Functional::output_length + Functional::dependencies + Functional::eval_setup new methods (D-13, D-15)"
  - "Pitfall G8 + Wave-0 gap 5 invariant CI-enforced via regularize_2nd_taylor.rs (2 tests)"
  - "10 new unit tests for output_length + eval_setup rejection paths (metaGGA, LAPLACIAN, GGA-non-2ND_TAYLOR)"
affects: [03-02, 03-03, 03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "gga/shared/ submodule tree: one #[cube] fn module per C++ .hpp helper cluster (D-08)"
    - "pub const <NAME>_F64: f64 constants with F::cast_from(<NAME>_F64) at call sites (PATTERNS §D6, §S2)"
    - "SKELETON body pattern: `// SKELETON — full body lands in 03-XX Task Y` inline marker + zero-output body (W3 resolution — executor can distinguish skeleton from full body at a glance)"
    - "Kappa-parameterised exchange helper (pbex::enhancement takes `r: F` so PBEX/REVPBE/PBESOL share one kernel)"
    - "2ND_TAYLOR Vars arm pattern: slot-copy 10 (or 20) CTaylor blocks + derive gaa/gbb/gab/gnn/gss/gns + lapa/lapb/lapn/laps + explicit chain to lower-variant builder"
    - "Host-side Mode::Potential gate pattern: eval_setup walks self.weights → aggregate Dependency → reject metaGGA-class + non-2ND_TAYLOR GGA upfront (pre-launch)"

key-files:
  created:
    - "crates/xcfun-eval/src/functionals/gga/mod.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/mod.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/constants.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/pbex.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/pbec_eps.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs"
    - "crates/xcfun-eval/src/functionals/gga/shared/optx.rs"
    - "crates/xcfun-eval/tests/regularize_2nd_taylor.rs"
    - ".planning/phases/03-gga-tier-mode-potential/03-01-SUMMARY.md"
  modified:
    - "crates/xcfun-eval/src/functionals/mod.rs — pub mod gga; added"
    - "crates/xcfun-eval/src/density_vars.rs — DensVarsDev 22 → 24 fields (pub lapn + pub laps)"
    - "crates/xcfun-eval/src/density_vars/build.rs — defensive zero-init + 7 new comptime arms + 10 new helper fns"
    - "crates/xcfun-eval/src/density_vars/regularize.rs — TINY_DENSITY_F64 pub(crate) → pub"
    - "crates/xcfun-eval/src/functional.rs — 3 new methods (output_length, dependencies, eval_setup) + 10 new tests + launch_eval_point handle-array widened 22 → 24"

key-decisions:
  - "S2_PREFACTOR_F64 corrected to 0.016455307846020558 (pw9xx.hpp:44-46) — plan spec carried 0.161620... which was off by 10x (Rule-1 bug fix; executor used mathematically correct value matching the C++ reference)"
  - "MU_PBE_F64 split into MU_PBE_F64 (0.066725·π²/3 = 0.21951645... — default branch pbex.hpp:34) and MU_PBE_RPBEX_F64 (literal 0.2195149727645171 — used by enhancement_RPBE pbex.hpp:44 regardless of XCFUN_REF_PBEX_MU flag)"
  - "pbex::energy_pbe_ab takes `rho_43: &Array<F>` as caller-supplied (from DensVarsDev::a_43/b_43) rather than recomputing `pow(rho, 4/3)` inside, matching how slaterx.rs consumes d.a_43/d.b_43 (Phase 2 convention)"
  - "regularize::TINY_DENSITY_F64 promoted pub so tests consume single source of truth (avoiding hardcoded floor values that would silently de-sync if TINY_DENSITY changes)"
  - "DensVarsDev field insertion point for lapn/laps: between lapb and zeta (preserving grouping: 'Laplacian terms' block). This shifts zeta/r_s/n_m13/a_43/b_43/jpaa/jpbb indices by 2 in the launch-handle array (updated in functional.rs)"
  - "Skeleton bodies zero their output via ctaylor_zero rather than panic!/assert! so that downstream callers that chain through (e.g. pbex::enhancement calling pw91_like::s2 — a FULL body; but if an accidentally-skeleton helper were chained, it would return 0 instead of garbage, keeping CI observable)"

patterns-established:
  - "Kappa-parameterised exchange: pbex::enhancement<F: Float>(r: F, ...) where R is runtime so PBEX (0.804) / REVPBE (1.245) / PBESOL share one monomorphised kernel per F"
  - "Constant scalar addition to CTaylor: `tmp[0] = tmp[0] + F::new(1.0)` — touches ONLY the CNST slot per bit-flag indexing, since there is no `ctaylor_add_scalar` primitive"
  - "2ND_TAYLOR slot layout enforcement: input slots are indexed as `input[k*size + i]` with k ∈ {0..=9} (single-spin) or k ∈ {0..=19} (double-spin), where size = 1<<n. α lives at k=0, β at k=10 for the A_B/N_S double-spin variants."
  - "Explicit helper-function chain at end of variant arm: replaces C-style `case`-fallthrough at densvars.hpp per D-11 + P5 (applied uniformly across all 10 new arms)"

requirements-completed: [GGA-01, GGA-02, GGA-04, GGA-05, GGA-06, GGA-07, GGA-08, GGA-09, GGA-10, MODE-02, MODE-05]

# Metrics
duration: ~30 min
completed: 2026-04-25
---

# Phase 3 Plan 01: Wave-1 GGA Substrate Summary

**GGA shared-helper scaffolding (6 files, 13 `#[cube] fn`s, 36 f64 constants), 10 new `build_densvars` arms for vars 0/1/3/4/5/7/27/28/29/30 with `lapn`/`laps` field additions, and `Functional::eval_setup` Mode::Potential host-side rejection paths all shipped and GREEN.**

## Performance

- **Duration:** ~30 min (wall-clock)
- **Completed:** 2026-04-25
- **Tasks:** 3 (all `type=auto`)
- **Files created:** 10 (9 Rust modules/tests + 1 summary)
- **Files modified:** 5 Rust modules (mod.rs, density_vars.rs, build.rs, regularize.rs, functional.rs)
- **xcfun-eval compile time:** 3.23s clean, 1.23s incremental (well under 45s G10 budget)
- **Test wall-clock:**
  - `cargo test --test regularize_2nd_taylor`: 0.07s (2 tests)
  - `cargo test --lib`: 0.00s (18 tests, all Functional::* unit tests)
  - `cargo test --test self_tests` (Phase-2 regression): 11.76s (all LDA functionals still GREEN)

## Accomplishments

1. **gga/ module tree scaffold + 6 shared helper files** — `pub mod gga;` exposed from `functionals/mod.rs`; `shared/` submodule hosts `constants.rs` + 5 `#[cube] fn` modules (pbex, pbec_eps, pw91_like, b97_poly, optx). 13 helper signatures across the 5 modules, 36 f64 constants in `constants.rs`.
2. **FULL-body helpers shipped:** `pbex::enhancement` (kappa-parameterised PBE/REVPBE/PBESOL), `pbex::energy_pbe_ab` (energy = prefactor · enhancement), and `pw91_like::s2` (the one `pbex::enhancement` depends on). All three compose via the Phase-1 ctaylor primitives without touching `mul_add`/`fma`.
3. **Skeleton helpers with W3 markers:** 10 SKELETONs across pbec_eps (3), pw91_like (4), b97_poly (2), optx (2), pbex::enhancement_rpbe (1). Each carries an inline `// SKELETON — full body lands in 03-XX Task Y` comment pointing to the consumer plan.
4. **DensVarsDev struct extended:** `pub lapn: Array<F>` + `pub laps: Array<F>` inserted after `lapb` per B2 plan-time-audited resolution. 22 → 24 fields. Launch handle array widened to 24; defensive zero-init extended.
5. **10 new `build_xc_*` helpers + 7 new `build_densvars` comptime arms:**
   - Low-level (W4): `build_xc_a` (vars=0), `build_xc_n` (vars=1), `build_xc_n_s` (vars=3).
   - GGA: `build_xc_a_gaa` (vars=4), `build_xc_n_gnn` (vars=5), `build_xc_n_s_gnn_gns_gss` (vars=7).
   - 2ND_TAYLOR (D-10-A discriminants): `build_xc_a_2nd_taylor` (vars=27), `build_xc_a_b_2nd_taylor` (vars=28), `build_xc_n_2nd_taylor` (vars=29), `build_xc_n_s_2nd_taylor` (vars=30). All populate `lapa`/`lapb`/`lapn`/`laps`/`gaa`/`gbb`/`gab`/`gnn`/`gss`/`gns` from 2nd-order spatial Taylor input slots per `XCFunctional.cpp:675-760`. Every arm uses an EXPLICIT helper-function chain (D-11 + P5) — no C-style fallthrough.
6. **Pitfall G8 invariant CI-enforced:** `regularize_2nd_taylor.rs` ships 2 tests at N=3 (size=8). Slot 0 below `TINY_DENSITY_F64` is clamped; slots 1..7 (which carry 2nd-Taylor-seeded derivative coefficients in production) are preserved bit-identical.
7. **Mode::Potential host-side gates live + tested:** `Functional::output_length` returns 2 (A / A_2ND_TAYLOR) or 3 (all other variants including A_B_2ND_TAYLOR / N_S_2ND_TAYLOR) per D-15. `Functional::eval_setup` rejects metaGGA-class deps (KINETIC / LAPLACIAN) with `XcError::InvalidMode`, GGA+non-2ND_TAYLOR combos with `XcError::InvalidVars`. No new `XcError` variants (D-25). 10 new tests cover every rejection path + positive cases (LDA-accepts, GGA-with-2ND_TAYLOR-accepts).
8. **Phase-2 LDA regression: ZERO.** `cargo test --test self_tests` still GREEN (tier1_self_tests_pass in 11.76s). `cargo test --test regularize_invariant` all 3 tests GREEN.

## Task Commits

1. **Task 1: Scaffold gga/ module tree + shared helper signatures** — `835d04a` (feat)
2. **Task 2: Extend build_densvars with 7 new Vars arms + DensVarsDev struct extension** — `fe57cc5` (feat)
3. **Task 3: regularize_2nd_taylor test + output_length/eval_setup for Mode::Potential** — `45e9aeb` (feat)

## Files Created/Modified

### Created (9 Rust + 1 summary)

- `crates/xcfun-eval/src/functionals/gga/mod.rs` — 27 LOC. Exposes `pub mod shared;` + commented placeholders for 9 family modules (pbe/becke/lyp/optx/pw91/p86/apbe/b97/kt) landing in 03-02/03/04.
- `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` — 14 LOC. Re-exports 6 submodules.
- `crates/xcfun-eval/src/functionals/gga/shared/constants.rs` — 110 LOC. 36 `pub const <NAME>_F64: f64` scalar constants extracted verbatim from xcfun-master C++. **36 >= 20 required per plan acceptance.**
- `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` — 128 LOC. enhancement (FULL), enhancement_rpbe (SKELETON), energy_pbe_ab (FULL).
- `crates/xcfun-eval/src/functionals/gga/shared/pbec_eps.rs` — 75 LOC. a_expm1_inner, h_gga, phi (all SKELETON → 03-02).
- `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` — 135 LOC. s2 (FULL), chi2/prefactor/pw91k_prefactor/pw91xk_enhancement (SKELETON → 03-02/03-03).
- `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` — 60 LOC. ux_ab / b97_enhancement (SKELETON → 03-04).
- `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` — 52 LOC. g_xa2 / optx_enhancement (SKELETON → 03-03).
- `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` — 78 LOC. 2 tests, N=3, `TINY_DENSITY_F64` single-source-of-truth.

### Modified

- `crates/xcfun-eval/src/functionals/mod.rs` — `pub mod gga;` added (1-line change).
- `crates/xcfun-eval/src/density_vars.rs` — 2 new struct fields (`lapn`, `laps`), 3 doc-comment updates. 22 → 24 fields.
- `crates/xcfun-eval/src/density_vars/build.rs` — 2 new `ctaylor_zero` calls in defensive zero-init; 10 new `#[cube] fn` arms; 7 new comptime-if arms in dispatch. +621 LOC total.
- `crates/xcfun-eval/src/density_vars/regularize.rs` — `pub(crate)` → `pub` on `TINY_DENSITY_F64`.
- `crates/xcfun-eval/src/functional.rs` — 3 new methods (`output_length`, `dependencies`, `eval_setup`); 10 new tests; launch_eval_point handle-array extended 22→24 with `lapn_h`/`laps_h` inserted in struct order (after `lapb_h`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `S2_PREFACTOR_F64` value corrected (10x error in plan spec)**
- **Found during:** Task 1 Step D (`constants.rs` authoring)
- **Issue:** Plan 03-01-PLAN.md §Action Step D and PATTERNS §D3 specified `S2_PREFACTOR_F64 = 0.161_620_459_673_998_68_f64`. The formula `pw9xx.hpp:44-46` is `(6^(2/3) / (12·π^(2/3)))²` which evaluates in f64 to **0.016455307846020558** — the plan value was off by exactly 10x. Keeping the plan's value would break PBEX/REVPBEX/PBESOL parity at 1e-12 by factor ~10 on the enhancement-factor path, and every downstream GGA consumer calling `pw91_like::s2` would inherit the error.
- **Fix:** Used the mathematically correct value `0.016_455_307_846_020_56_f64`. Verified via `python3 -c "(6**(2/3)/(12*π**(2/3)))**2"` = 0.01645530784602056. Documented in `constants.rs` §"Double-check audit" module header.
- **Files modified:** `crates/xcfun-eval/src/functionals/gga/shared/constants.rs`.
- **Verification:** `pw91_like::s2` body now uses the correct prefactor; `pbex::enhancement` calls it correctly.
- **Committed in:** `835d04a` (Task 1 commit).

**2. [Rule 1 - Bug] `MU_PBE_F64` default-branch value corrected**
- **Found during:** Task 1 Step D (`constants.rs` authoring — cross-referencing `pbex.hpp`)
- **Issue:** Plan specified `MU_PBE_F64: 0.2195149727645171`, but that value is the Daresbury branch (`XCFUN_REF_PBEX_MU` defined at `config.hpp:39`, which is currently commented out — so the DEFAULT branch is active). The default branch at `pbex.hpp:34` is `0.066725 · π² / 3` = **0.21951645122089580** (computed in f64). The `enhancement_RPBE` at `pbex.hpp:44` explicitly uses the literal `0.2195149727645171` regardless of the flag. Using the plan's value for the default branch would drift PBEX by ~7e-7 rel-err, breaking 1e-12 parity.
- **Fix:** Stored TWO constants: `MU_PBE_F64 = 0.21951645122089580` (default, used by `enhancement`) and `MU_PBE_RPBEX_F64 = 0.2195149727645171` (literal, used by `enhancement_rpbe` when the skeleton lands in 03-02). Documented in `constants.rs` §"Double-check audit".
- **Files modified:** `crates/xcfun-eval/src/functionals/gga/shared/constants.rs`.
- **Verification:** `pbex::enhancement` now casts from the correct default-branch value.
- **Committed in:** `835d04a` (Task 1 commit).

**3. [Rule 3 - Blocking] `DensVarsDevLaunch::new` arity propagated 22 → 24**
- **Found during:** Task 2 (post-edit build after adding `lapn` + `laps` to DensVarsDev)
- **Issue:** `#[derive(CubeLaunch)]` auto-generates `DensVarsDevLaunch::new(...)` with one positional arg per struct field. Adding 2 fields broke every call site in `functional.rs`. Compile error: "this function takes 24 arguments but 22 arguments were supplied" at line 454.
- **Fix:** Extended `launch_eval_point`'s densvar-handle array parameter from `[Handle; 22]` to `[Handle; 24]`; added `lapn_h`/`laps_h` in `run_launch` scratch-handle creation and inserted them in struct order (after `lapb_h`, before `zeta_h`) in every call-site array literal (used `replace_all` for the pattern `lapa_h.clone(), lapb_h.clone(), zeta_h.clone()` → `lapa_h.clone(), lapb_h.clone(), lapn_h.clone(), laps_h.clone(), zeta_h.clone()`, hitting all 27 call sites uniformly); added 2 more `ArrayArg::from_raw_parts(densvar_handles[22..=23])` lines in `DensVarsDevLaunch::new(...)`.
- **Files modified:** `crates/xcfun-eval/src/functional.rs`.
- **Verification:** `cargo build -p xcfun-eval --features testing` GREEN; Phase-2 LDA `tier1_self_tests_pass` still GREEN (11.76s).
- **Committed in:** `fe57cc5` (Task 2 commit).

**4. [Rule 3 - Blocking] `regularize::TINY_DENSITY_F64` promoted `pub(crate)` → `pub`**
- **Found during:** Task 3 (test authoring — `regularize_2nd_taylor.rs` needs to import the floor constant)
- **Issue:** The test lives in `crates/xcfun-eval/tests/` (integration test), so `pub(crate)` is not visible. Plan explicitly flagged this: "if `regularize::FLOOR_F64` is not a `pub const`, add it during this task. The test MUST NOT hardcode the floor value — it MUST pull from the single source of truth."
- **Fix:** Changed visibility to `pub`. No API surface risk — `TINY_DENSITY_F64` is a scalar f64 constant; promotion is safe.
- **Files modified:** `crates/xcfun-eval/src/density_vars/regularize.rs`.
- **Verification:** Test imports and compiles; tests pass.
- **Committed in:** `45e9aeb` (Task 3 commit).

---

**Total deviations:** 4 auto-fixed (2 Rule-1 bugs in plan specification, 2 Rule-3 blocking).

**Impact on plan:** Both Rule-1 bugs were numerical-constant values that would have silently broken 1e-12 parity in every PBEX/REVPBEX/PBESOL kernel landing in Wave 2. Caught at plan-time via cross-checking against the C++ source. The two Rule-3 blocking fixes were mechanical propagations (struct arity + visibility) that the plan author anticipated but didn't ship. No scope creep — the numerical contract and deliverable set ship exactly as the plan stipulated.

## Known Stubs

The helpers listed below are SKELETONS this plan — they return zero-filled output and are consumed by later plans. Each carries an inline `// SKELETON — full body lands in 03-XX Task Y` marker for W3 auditability. None are reachable from a Wave-2/3/4 kernel launch until their consumer plan lands (and the Wave-1 gga/ directory has NO kernel-dispatch integration yet — the gga/ tree is not called from `dispatch.rs` in this plan).

| Helper | File | Consumer plan | Wave |
|---|---|---|---|
| `pbex::enhancement_rpbe` | `shared/pbex.rs:110` | 03-02 Task 1 Step A (RPBEX) | 2 |
| `pbec_eps::a_expm1_inner` | `shared/pbec_eps.rs:27` | 03-02 Task 1 Step A (PBEC) | 2 |
| `pbec_eps::h_gga` | `shared/pbec_eps.rs:45` | 03-02 Task 1 Step A (PBEC) | 2 |
| `pbec_eps::phi` | `shared/pbec_eps.rs:63` | 03-02 Task 1 Step A (PBEC) | 2 |
| `pw91_like::chi2` | `shared/pw91_like.rs:30` | 03-02 Task 1 Step A (BECKE) | 2 |
| `pw91_like::prefactor` | `shared/pw91_like.rs:82` | 03-02 Task 1 Step A (PBEX) | 2 |
| `pw91_like::pw91k_prefactor` | `shared/pw91_like.rs:93` | 03-03 Task 2 Step A (PW91K) | 3 |
| `pw91_like::pw91xk_enhancement` | `shared/pw91_like.rs:116` | 03-03 Task 2 Step B (PW91X/PW91K) | 3 |
| `b97_poly::ux_ab` | `shared/b97_poly.rs:30` | 03-04 Task 1 Step A (B97 first consumer) | 4 |
| `b97_poly::b97_enhancement` | `shared/b97_poly.rs:45` | 03-04 Task 1 Step B (B97) | 4 |
| `optx::g_xa2` | `shared/optx.rs:18` | 03-03 Task 1 Step A (OPTX) | 3 |
| `optx::optx_enhancement` | `shared/optx.rs:34` | 03-03 Task 1 Step A (OPTX) | 3 |

Plan's W3 gate acceptance criterion: `rg -c "SKELETON — full body lands in" crates/xcfun-eval/src/functionals/gga/shared/` ≥ 8. Measured: **12 markers**. PASS.

## Issues Encountered

None beyond the 4 deviations above. Task 1/2/3 each executed in a single build-test-commit iteration (no debug cycles). The kappa-parameterised `pbex::enhancement` body composed correctly on first build by using explicit scalar-add-to-CNST-slot (`tmp[0] = tmp[0] + F::new(1.0)`) — there is no `ctaylor_add_scalar` primitive in the Phase-1/2 API surface, but the bit-flag-indexed CTaylor layout makes the direct slot-0 mutation safe and efficient.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

### Wave 2 (plan 03-02 — PBE family + APBE + RPBEX) unblocked

- `gga::shared::pbex::enhancement` + `energy_pbe_ab` + `pw91_like::s2` are FULL; PBEX / REVPBEX / PBESOLX / PBEINTX / PBELOCX can call them directly (Wave 2 wires them into per-functional kernel bodies under `gga/pbe/`).
- `pbex::enhancement_rpbe`, `pbec_eps::{a_expm1_inner, h_gga, phi}` SKELETONS land their full bodies in Wave 2 per W3 pointer.
- Wave 2 kernels use the 6 new `build_xc_*` GGA arms (vars ∈ {4, 5, 6, 7}) — all present and wired through `build_densvars`.

### Forward risks for 03-02 / 03-03 / 03-04

1. **`ctaylor_pow` with exponent -8/3 at N=4 (order 4) not fixture-gated.** `pw91_like::s2` calls `ctaylor_pow(rho, -8/3)` which at N=4 (size=16) runs through `pow_expand` with 16 coefficients. Plan 03-00 fixtured `ctaylor_expm1` and `ctaylor_sqrtx_asinh_sqrtx` through NVAR=3 (size=8). If tier-2 Wave-2 PBEX parity fails > 1e-12 at order 4 on the regularize-stratum grid (small rho), the fix is to extend `golden_expand.rs` fixtures for `pow_expand` at NVAR=4 — not a blocker for Wave-2's scalable order-2 slice.

2. **`ctaylor_add_scalar` not primitive.** The pattern used in `pbex::enhancement` (`tmp[0] = tmp[0] + F::new(1.0)`) is correct for the bit-flag CTaylor indexing but verbose. If Wave-2 / Wave-3 bodies make heavy use of `1 + expression` sub-expressions, it would be worth adding a `pub fn ctaylor_add_scalar<F: Float>(a: &Array<F>, s: F, out: &mut Array<F>, #[comptime] n: u32)` primitive in `xcfun-ad/src/ctaylor.rs` — 5 LOC, composable with `ctaylor_zero + scalar_mul(1) + slot-0 mutation`. Flagged as a potential Wave-2 micro-refactor, NOT a 03-01 blocker.

3. **`build_xc_a_b_2nd_taylor` chain semantics.** The body populates `out.a` + `out.b` from slots 0 + 10*size directly (inline, not via `build_xc_a_b::<F>(input, out, n)`), because `build_xc_a_b` reads α from slot 0 and β from slot `size` — but in `_2ND_TAYLOR` the β channel starts at slot `10*size`. Plan 03-01 ships the inline pattern (clean and unambiguous). Plan 03-05 Wave-5 Mode::Potential launch validates this end-to-end through a PBE-level fixture — if there's a semantic mismatch with the C++ reference's 20-slot layout, the fix lands there.

4. **Skeleton zero-output fallback semantics.** Skeletons call `ctaylor_zero(out, n)`, so if any Wave-2+ kernel *accidentally* routes through a not-yet-landed skeleton (e.g., forgetting to check that an enhancer full-body has shipped in its consumer plan), the silent-zero result is OBSERVABLE in tier-2 parity (it'll be zero on all grids, not just regularize-stratum). Defensive enough; no guard needed.

## TDD Gate Compliance

Plan 03-01's 3 tasks all carry `tdd="true"`. Gate sequence honoured per Phase-2 fixture-gate pattern:

- **Task 1 (scaffold + constants):** Compile is the gate — `cargo build -p xcfun-eval --features testing` transitions from red (no `gga/` module) to green (all signatures compile). No separate RED commit because the failing-compile state is observable via `cargo check`.
- **Task 2 (DensVarsDev extension + 10 new arms):** Compile gate again. Struct change caused `functional.rs` to red-state with arity error (E0061); Task 2's commit turns it green.
- **Task 3 (regularize_2nd_taylor test + output_length/eval_setup):** Tests ARE new artifacts. The test `regularize_preserves_2nd_taylor_coefficients` exercises the existing Phase-2 `regularize` body (no code change to regularize.rs itself beyond visibility); it's a GREEN-from-first-run test because the invariant already held. The `output_length_potential_*` + `eval_setup_rejects_*` tests required new `Functional` methods landing in the SAME commit — they RED-state-transition via the `unknown method` compile error before the impl block; GREEN post-impl. All 10 new tests pass on first run.

The commit log satisfies the type=tdd gate sequence: `test` artifacts + `feat` artifacts co-committed per task (not per gate) because Phase-2 established the compile-gate pattern rather than the strict RED/GREEN/REFACTOR separation.

## Self-Check: PASSED

Verified:

- `crates/xcfun-eval/src/functionals/gga/mod.rs` FOUND
- `crates/xcfun-eval/src/functionals/gga/shared/{mod, constants, pbex, pbec_eps, pw91_like, b97_poly, optx}.rs` all 7 files FOUND
- `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` FOUND
- All 3 task commits present in `git log --oneline`: `835d04a`, `fe57cc5`, `45e9aeb`
- `cargo build -p xcfun-eval --features testing` exits 0 in 3.23s clean / 1.23s incremental
- `cargo test -p xcfun-eval --features testing --test regularize_2nd_taylor`: 2/2 GREEN
- `cargo test -p xcfun-eval --features testing --lib`: 18/18 GREEN (10 new + 8 Phase-2)
- `cargo test -p xcfun-eval --features testing --test self_tests`: 1/1 GREEN (Phase-2 LDA regression)
- `cargo run -p xtask --bin check-no-mul-add`: PASS (23 files scanned)
- `cargo run -p xtask --bin check-no-fma`: PASS
- `cargo run -p xtask --bin check-no-anyhow`: PASS (7 library crates)
- `rg -c "comptime!\(vars == (4|5|7|27|28|29|30)\)" crates/xcfun-eval/src/density_vars/build.rs` = 7 ✓
- `rg -c "^pub fn build_xc_.*_2nd_taylor" crates/xcfun-eval/src/density_vars/build.rs` = 4 ✓
- `rg -c "^pub fn build_xc_(a|n|n_s)\b" crates/xcfun-eval/src/density_vars/build.rs` = 3 ✓ (W4)
- `rg "pub lapn:|pub laps:" crates/xcfun-eval/src/density_vars.rs` — both present ✓ (B2)
- `rg -c "^pub const .*_F64" crates/xcfun-eval/src/functionals/gga/shared/constants.rs` = 36 ≥ 20 ✓
- `rg -c "^#\[cube\]" crates/xcfun-eval/src/functionals/gga/shared/` = 15 ≥ 11 ✓
- `rg -c "^    // SKELETON — full body lands in" crates/xcfun-eval/src/functionals/gga/shared/` = 12 inline body markers (≥ 8 required) ✓ (W3); total including doc-comment mentions = 24
- `rg "mul_add|\.fma\(" crates/xcfun-eval/src/functionals/gga/` — zero matches in real code (2 mentions are in doc-comments `no mul_add per ACC-06`)

---
*Phase: 03-gga-tier-mode-potential*
*Plan: 01 (Wave 1 substrate)*
*Completed: 2026-04-25*
