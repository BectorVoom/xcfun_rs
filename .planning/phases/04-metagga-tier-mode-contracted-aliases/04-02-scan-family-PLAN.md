---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "02"
type: execute
wave: 2
depends_on:
  - "04-00"
requirements:
  - MGGA-02
files_modified:
  - crates/xcfun-eval/src/functionals/mgga/scanx.rs
  - crates/xcfun-eval/src/functionals/mgga/scanc.rs
  - crates/xcfun-eval/src/functionals/mgga/rscanx.rs
  - crates/xcfun-eval/src/functionals/mgga/rscanc.rs
  - crates/xcfun-eval/src/functionals/mgga/rppscanx.rs
  - crates/xcfun-eval/src/functionals/mgga/rppscanc.rs
  - crates/xcfun-eval/src/functionals/mgga/r2scanx.rs
  - crates/xcfun-eval/src/functionals/mgga/r2scanc.rs
  - crates/xcfun-eval/src/functionals/mgga/r4scanx.rs
  - crates/xcfun-eval/src/functionals/mgga/r4scanc.rs
  - crates/xcfun-eval/src/functionals/mgga/mod.rs
  - crates/xcfun-eval/src/dispatch.rs
  - validation/build.rs
  - validation/c_stubs.cpp
autonomous: true
created: "2026-04-25"
goal: "Wave 2 — SCAN family (10 kernels: SCAN/rSCAN/r++SCAN/r2SCAN/r4SCAN exchange + correlation) via comptime IDELEC dispatch from shared scan_like.rs"

must_haves:
  truths:
    - "All 10 SCAN-family kernels compile and pass tier-1 self-test at strict 1e-12"
    - "IDELEC dispatch (comptime 0..=4) produces distinct exchange factors for SCAN/rSCAN/r++SCAN/r2SCAN/r4SCAN"
    - "R4SCAN passes tier-1 at strict 1e-12 (highest-degree polynomial — primary accuracy risk)"
    - "cargo xtask validate --backend cpu --order 2 --filter 'scan' reports zero failures"
  artifacts:
    - path: crates/xcfun-eval/src/functionals/mgga/scanx.rs
      provides: scanx_kernel with IDELEC=0
      exports: [scanx_kernel]
    - path: crates/xcfun-eval/src/functionals/mgga/r4scanx.rs
      provides: r4scanx_kernel with IDELEC=4
      exports: [r4scanx_kernel]
  key_links:
    - from: crates/xcfun-eval/src/functionals/mgga/scanx.rs
      to: crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs
      via: get_SCAN_Fx with comptime idelec=0
      pattern: "get_SCAN_Fx.*0"
    - from: crates/xcfun-eval/src/dispatch.rs
      to: crates/xcfun-eval/src/functionals/mgga/scanx.rs
      via: comptime id match arm
      pattern: "XC_SCANX"
---

<objective>
Port the 10 SCAN-family kernels (SCAN, rSCAN, r++SCAN, r2SCAN, r4SCAN — each with X and C variants) using the comptime IDELEC dispatch from `shared/scan_like.rs` (Plan 04-00). Each kernel is thin: the body calls `get_SCAN_Fx(d, out, idelec=N)` or `r2SCAN_C(d, out, idelec=N)` from the shared module. All 10 use Vars id=13 (TAUA_TAUB, inlen=7).

Purpose: SCAN family is the largest single-wave delivery (10 functionals, 1 large shared module). The comptime IDELEC pattern ensures zero runtime branch overhead per functional. R4SCAN's 4th-order polynomial is the highest-degree expression in the entire metaGGA set — flag for explicit Rule-1 review.

Note: This plan runs in Wave 2 PARALLEL with Plan 04-01 because both depend only on Plan 04-00 and they modify different files (no file overlap).

Output: 10 kernel files, dispatch updated (+10 arms), validation/build.rs updated (+10 C++ files).
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md

<interfaces>
<!-- From Plan 04-00 substrate. -->

From crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs (Wave 0):
```rust
// IDELEC values: 0=SCAN, 1=rSCAN, 2=r++SCAN, 3=r2SCAN, 4=r4SCAN
#[cube]
pub fn get_SCAN_Fx<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] n: u32,
);

#[cube]
pub fn r2SCAN_C<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,
    #[comptime] n: u32,
);
```

From existing kernel pattern (beckex.rs):
```rust
#[cube(launch_unchecked)]
pub fn beckex_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) { ... }
```
</interfaces>
</context>

<tasks>

<task id="2.1" type="auto">
  <name>Task 1: SCAN-family exchange kernels (5 files: SCAN/rSCAN/r++SCAN/r2SCAN/r4SCAN exchange)</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/scanx.rs,
    crates/xcfun-eval/src/functionals/mgga/rscanx.rs,
    crates/xcfun-eval/src/functionals/mgga/rppscanx.rs,
    crates/xcfun-eval/src/functionals/mgga/r2scanx.rs,
    crates/xcfun-eval/src/functionals/mgga/r4scanx.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs
  </files>
  <read_first>
    - `xcfun-master/src/functionals/SCANx.cpp` (49 LOC) — IDELEC=0. Body: `get_SCAN_Fx(d, IDELEC=0)`.
    - `xcfun-master/src/functionals/rSCANx.cpp` (49 LOC) — IDELEC=1.
    - `xcfun-master/src/functionals/rppSCANx.cpp` (48 LOC) — IDELEC=2.
    - `xcfun-master/src/functionals/r2SCANx.cpp` (49 LOC) — IDELEC=3.
    - `xcfun-master/src/functionals/r4SCANx.cpp` (48 LOC) — IDELEC=4. HIGH-RISK polynomial precision.
    - `xcfun-master/src/functionals/SCAN_like_eps.hpp` — verify the `get_SCAN_Fx` function signature, IDELEC branching, and R4SCAN's 4th-order polynomial expression. Pay special attention to R4SCAN's polynomial evaluation order (must be verbatim to C++ to maintain 1e-12).
    - `crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs` — the shared helper (Plan 04-00).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-11 (strict 1e-12, M06 + R4SCAN watch list), D-01-B (per-family parallelism)
  </read_first>
  <action>
    All 5 exchange kernels follow the identical template — the ONLY difference is the `idelec` comptime value passed to `get_SCAN_Fx`. Read `SCANx.cpp:42-48` for the exact spin-decomposed composition: the exchange energy is `d.a * get_SCAN_Fx(d.a, d.gaa, d.taua, IDELEC) + d.b * get_SCAN_Fx(d.b, d.gbb, d.taub, IDELEC)`.

    For each file `{scan,rscan,rppscan,r2scan,r4scan}x.rs`:
    ```rust
    // Port of xcfun-master/src/functionals/{variant}x.cpp
    // Uses id=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB, inlen=7) via IDELEC={N}

    use cubecl::prelude::*;
    use crate::density_vars::DensVarsDev;
    use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_mul};
    use super::shared::scan_like;

    #[cube(launch_unchecked)]
    pub fn {variant}x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        let mut ea = Array::<F>::new(comptime!((1_u32 << n) as usize));
        let mut eb = Array::<F>::new(comptime!((1_u32 << n) as usize));
        // Exchange: d.a * F_x(d.a, d.gaa, d.taua) + d.b * F_x(d.b, d.gbb, d.taub)
        scan_like::get_SCAN_Fx::<F>(d, &mut ea, comptime!({IDELEC}), n);
        // NOTE: get_SCAN_Fx must produce the full energy contribution including density prefactor
        // OR the multiplication must happen here — verify against C++ SCANx.cpp body exactly.
        ctaylor_add::<F>(&ea, &eb, out, n);
    }
    ```
    Read the C++ bodies CAREFULLY to determine if `get_SCAN_Fx` returns `eps_x * n^(4/3)` (the spin-decomposed contribution including density) or just `eps_x`. The composition must match the C++ `SCANx.cpp` body verbatim.

    **R4SCAN explicit verification:** After porting `r4scanx.rs`, run `xtask check-no-mul-add` to confirm no `mul_add` slipped in. R4SCAN uses a 4th-order geometric expansion — particularly susceptible to reassociation.

    Update `mgga/mod.rs` to add the 5 exchange module declarations.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~scanx) | test(~rscanx) | test(~r2scanx) | test(~r4scanx)'` — all 5 exchange tier-1 pass at strict 1e-12.
    3. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/r4scanx.rs` returns empty (R4SCAN no mul_add).
    4. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/scanx.rs` returns empty.
    5. `grep -n "comptime!.*0" crates/xcfun-eval/src/functionals/mgga/scanx.rs` shows IDELEC=0 usage.
    6. `grep -n "comptime!.*4" crates/xcfun-eval/src/functionals/mgga/r4scanx.rs` shows IDELEC=4 usage.
  </acceptance_criteria>
  <done>5 SCAN exchange kernels compile and pass tier-1 at strict 1e-12.</done>
</task>

<task id="2.2" type="auto">
  <name>Task 2: SCAN-family correlation kernels (5 files) + dispatch extension + validation update</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/scanc.rs,
    crates/xcfun-eval/src/functionals/mgga/rscanc.rs,
    crates/xcfun-eval/src/functionals/mgga/rppscanc.rs,
    crates/xcfun-eval/src/functionals/mgga/r2scanc.rs,
    crates/xcfun-eval/src/functionals/mgga/r4scanc.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs,
    crates/xcfun-eval/src/dispatch.rs,
    validation/build.rs,
    validation/c_stubs.cpp
  </files>
  <read_first>
    - `xcfun-master/src/functionals/SCANc.cpp` (43 LOC) — IDELEC=0. Body calls `SCAN_eps::r2SCAN_C(d, IDELEC=0)`.
    - `xcfun-master/src/functionals/rSCANc.cpp` (44 LOC) — IDELEC=1.
    - `xcfun-master/src/functionals/rppSCANc.cpp` (43 LOC) — IDELEC=2.
    - `xcfun-master/src/functionals/r2SCANc.cpp` (44 LOC) — IDELEC=3.
    - `xcfun-master/src/functionals/r4SCANc.cpp` (43 LOC) — IDELEC=4.
    - `xcfun-master/src/functionals/SCAN_like_eps.hpp` — verify `r2SCAN_C` function signature and IDELEC branching for correlation.
    - `crates/xcfun-eval/src/dispatch.rs` — current state after Task 1; add 10 new arms (5 exchange + 5 correlation).
    - `validation/build.rs` — current state; append 10 SCAN C++ files.
    - `validation/c_stubs.cpp` — remove stubs for all 10 SCAN functionals.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-08 (dispatch bitmap)
  </read_first>
  <action>
    All 5 correlation kernels follow the same template as Task 1 but call `r2SCAN_C` instead of `get_SCAN_Fx`. Read `SCANc.cpp:36-42` for the exact body. The correlation is spin-summed via `r2SCAN_C(d, IDELEC)` directly (usually `d.n * eps_c`).

    For each file `{scan,rscan,rppscan,r2scan,r4scan}c.rs`:
    ```rust
    #[cube(launch_unchecked)]
    pub fn {variant}c_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        scan_like::r2SCAN_C::<F>(d, out, comptime!({IDELEC}), n);
    }
    ```
    Read C++ bodies to confirm the density `d.n` prefactor is inside `r2SCAN_C` or applied externally.

    **Update `mgga/mod.rs`:** add 5 correlation module declarations.

    **Update `dispatch.rs`:** Add 10 new comptime arms for all SCAN-family functionals (5X + 5C). Look up exact `FunctionalId` discriminant values for SCANX, SCANC, RSCANX, RSCANC, RPPSCANX, RPPSCANC, R2SCANX, R2SCANC, R4SCANX, R4SCANC from `crates/xcfun-core/src/functional_id.rs`. Update `supports()` bitmap (+10 ids).

    **Update `validation/build.rs`:** Append all 10 SCAN C++ files:
    ```rust
    "xcfun-master/src/functionals/SCANx.cpp",
    "xcfun-master/src/functionals/SCANc.cpp",
    "xcfun-master/src/functionals/rSCANx.cpp",
    "xcfun-master/src/functionals/rSCANc.cpp",
    "xcfun-master/src/functionals/rppSCANx.cpp",
    "xcfun-master/src/functionals/rppSCANc.cpp",
    "xcfun-master/src/functionals/r2SCANx.cpp",
    "xcfun-master/src/functionals/r2SCANc.cpp",
    "xcfun-master/src/functionals/r4SCANx.cpp",
    "xcfun-master/src/functionals/r4SCANc.cpp",
    // Also need the shared header — add SCAN_like_eps.hpp to include_path if required
    ```

    **Update `validation/c_stubs.cpp`:** Remove stubs for all 10 SCAN ids.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~scan)'` — all 10 tier-1 pass at strict 1e-12.
    3. `grep -n "XC_SCANX\|XC_R4SCANC" crates/xcfun-eval/src/dispatch.rs` returns matches (both extremes present).
    4. `grep -c "SCAN" validation/build.rs` shows 10 or more matches (all C++ files appended).
    5. `cargo xtask validate --backend cpu --order 2 --filter 'scan'` exits 0 with zero failures at 1e-12.
    6. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/r4scanc.rs` returns empty.
  </acceptance_criteria>
  <done>All 10 SCAN kernels compile, dispatch wired, xtask validate filter=scan exits 0 at 1e-12.</done>
</task>

</tasks>

<verification>
```bash
# Build
cargo build -p xcfun-eval --release 2>&1 | tail -5

# All SCAN tier-1
cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~scan)' 2>&1 | tail -20

# Mid-tier parity
cargo xtask validate --backend cpu --order 2 --filter 'scan' 2>&1 | tail -10

# Structural checks
grep -n "XC_SCANX\|XC_R4SCANX\|XC_R4SCANC" crates/xcfun-eval/src/dispatch.rs
grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/ 2>&1 | grep "scan"
```
</verification>

<success_criteria>
- All 10 SCAN-family kernels compile and pass tier-1 at strict 1e-12 (or explicit D-19 INCONCLUSIVE entry if R4SCAN drifts).
- `cargo xtask validate --backend cpu --order 2 --filter 'scan'` exits 0.
- No `mul_add` in any new scan*.rs file.
- dispatch.rs `supports()` bitmap updated +10 IDs.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-eval internals | No untrusted data — pure kernel ports; no new FFI surface |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-02-01 | Tampering | R4SCAN 4th-order polynomial evaluation order | mitigate | `xtask check-no-mul-add` gate already enforced by CI (Phase 2 D-24). Additionally, after porting, manually verify the polynomial coefficient sequence in r4scanx.rs against SCAN_like_eps.hpp line-by-line. |
| T-04-02-02 | Information Disclosure | None | accept | No new FFI surface. Pure internal plan. |

No new attack surface. Threats inherited from Phase 1 FFI gate.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-02-SUMMARY.md`
</output>
