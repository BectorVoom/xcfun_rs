---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "03"
type: execute
wave: 3
depends_on:
  - "04-01"
  - "04-02"
requirements:
  - MGGA-03
  - MGGA-04
  - MGGA-05
files_modified:
  - crates/xcfun-eval/src/functionals/mgga/m05x.rs
  - crates/xcfun-eval/src/functionals/mgga/m05c.rs
  - crates/xcfun-eval/src/functionals/mgga/m05x2x.rs
  - crates/xcfun-eval/src/functionals/mgga/m05x2c.rs
  - crates/xcfun-eval/src/functionals/mgga/m06x.rs
  - crates/xcfun-eval/src/functionals/mgga/m06c.rs
  - crates/xcfun-eval/src/functionals/mgga/m06lx.rs
  - crates/xcfun-eval/src/functionals/mgga/m06lc.rs
  - crates/xcfun-eval/src/functionals/mgga/m06hfx.rs
  - crates/xcfun-eval/src/functionals/mgga/m06hfc.rs
  - crates/xcfun-eval/src/functionals/mgga/m06x2x.rs
  - crates/xcfun-eval/src/functionals/mgga/m06x2c.rs
  - crates/xcfun-eval/src/functionals/mgga/blocx.rs
  - crates/xcfun-eval/src/functionals/mgga/mod.rs
  - crates/xcfun-eval/src/dispatch.rs
  - validation/build.rs
  - validation/c_stubs.cpp
autonomous: true
created: "2026-04-25"
goal: "Wave 3 — M05 family (4) + M06 family (8) + BLOCX (1) = 13 kernels; all metaGGA functional ports complete; dispatch reaches 82 arms"

must_haves:
  truths:
    - "XC_M05X, XC_M05C, XC_M05X2X, XC_M05X2C pass tier-1 self-test at strict 1e-12"
    - "XC_M06X, XC_M06C, XC_M06LX, XC_M06LC, XC_M06HFX, XC_M06HFC, XC_M06X2X, XC_M06X2C pass tier-1 at strict 1e-12"
    - "XC_BLOCX passes tier-1 self-test at strict 1e-12"
    - "BLOCX compiles and tests independently with zero BRX dependency (confirmed per RESEARCH)"
    - "cargo xtask validate --backend cpu --order 2 --filter 'm05|m06|blocx' reports zero failures"
  artifacts:
    - path: crates/xcfun-eval/src/functionals/mgga/m05x.rs
      provides: m05x_kernel
      exports: [m05x_kernel]
    - path: crates/xcfun-eval/src/functionals/mgga/m06c.rs
      provides: m06c_kernel (largest M06 body, 109 LOC)
      exports: [m06c_kernel]
    - path: crates/xcfun-eval/src/functionals/mgga/blocx.rs
      provides: blocx_kernel (TPSS-shaped, no BRX dependency)
      exports: [blocx_kernel]
  key_links:
    - from: crates/xcfun-eval/src/functionals/mgga/m05x.rs
      to: crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs
      via: m0x_fw helper call with 12-coefficient param array
      pattern: "m0x_like::m0x_fw"
    - from: crates/xcfun-eval/src/functionals/mgga/blocx.rs
      to: crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs
      via: blocx_energy helper call
      pattern: "shared::blocx::blocx_energy"
---

<objective>
Port 13 functional bodies (M05 ×4, M06 ×8, BLOCX ×1) using the `m0x_like.rs` shared substrate (from Plan 04-00). This is the final wave of metaGGA functional ports. After this plan, all 32 Phase-4 functional bodies ship and dispatch.rs covers 82 supported functional IDs.

RESEARCH KEY FINDING: BLOCX is INDEPENDENT of BRX. `blocx.cpp:18-46` is a TPSS-shaped enhancement with no `BR(...)` call. BLOCX can be implemented and tested without waiting for BR ship status. The plan depends on Wave 1 only to sequence correctly after TPSS ships (BLOCX and TPSS both use Vars id=13), but BLOCX has no functional code dependency on BRX.

ACCURACY NOTE: M06 family is the highest numerical-sensitivity family per literature. The 12-coefficient polynomial `m0x_fw` must be ported with verbatim C++ evaluation order. Apply `xtask check-no-mul-add` after each M06 body.

Output: 13 kernel files, dispatch.rs updated (+13 arms, bitmap reaches 82), validation updated.
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

From crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs (Wave 0):
```rust
// fw evaluates the M05/M06 kinetic-energy enhancement factor polynomial
// param_a is a compile-time-known 12-element coefficient array
#[cube]
pub fn m0x_fw<F: Float>(
    rho: &Array<F>, tau: &Array<F>,
    param_a: &Array<F>,  // 12 coefficients as runtime cubecl array
    out: &mut Array<F>,
    #[comptime] n: u32,
);

#[cube] pub fn m0x_zet<F: Float>(rho_a: &Array<F>, rho_b: &Array<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m0x_chi2<F: Float>(rho: &Array<F>, grad2: &Array<F>, tau: &Array<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m0x_Dsigma<F: Float>(rho: &Array<F>, grad2: &Array<F>, tau: &Array<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m06_c_anti<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m06_c_para<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m05_c_anti<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn m05_c_para<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
```

From crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs (Wave 0):
```rust
#[cube]
pub fn blocx_energy<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
```

From crates/xcfun-eval/src/functionals/gga/shared/pbex.rs (already ported):
```rust
#[cube] pub fn energy_pbe_ab<F: Float>(...);
#[cube] pub fn R_pbe<F: Float>(...);
```
</interfaces>
</context>

<tasks>

<task id="3.1" type="auto">
  <name>Task 1: M05 family (4 kernels) and BLOCX (1 kernel)</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/m05x.rs,
    crates/xcfun-eval/src/functionals/mgga/m05c.rs,
    crates/xcfun-eval/src/functionals/mgga/m05x2x.rs,
    crates/xcfun-eval/src/functionals/mgga/m05x2c.rs,
    crates/xcfun-eval/src/functionals/mgga/blocx.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs
  </files>
  <read_first>
    - `xcfun-master/src/functionals/m05x.cpp` (59 LOC) — exchange body: spin-decomposed `fw(param_a[12], rho, tau) * pbex::energy_pbe_ab(rho, gaa)`. param_a values hardcoded in the file.
    - `xcfun-master/src/functionals/m05c.cpp` (62 LOC) — correlation: `m05_c_para(d) + m05_c_anti(d) + m05_c_anti(d_mixed)` pattern.
    - `xcfun-master/src/functionals/m05x2x.cpp` (59 LOC) — same as M05X but different `param_a[12]` constants.
    - `xcfun-master/src/functionals/m05x2c.cpp` (62 LOC) — same as M05C but different correlation parameter set.
    - `xcfun-master/src/functionals/m0xy_fun.hpp` lines 24-150 — `fw`, `chi2`, `Dsigma` implementations; verify exact polynomial Horner order for `fw` (12 coefficients).
    - `xcfun-master/src/functionals/blocx.cpp` (61 LOC) — FULL READ. Confirm NO `BR(...)` call. Uses `pow(d_n, ...)`, `pow(p, ...)`, `sqrt`, `log`, `exp`. TPSS-shaped enhancement factor (tauw/tau ratio). Vars = id=13.
    - `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs` — m0x_fw, m0x_chi2, m0x_Dsigma (Plan 04-00).
    - `crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs` — blocx_energy (Plan 04-00).
    - `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` — energy_pbe_ab, R_pbe (Phase 3, already ported).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-11 (M06 watch list applies to M05 too for polynomial precision)
  </read_first>
  <action>
    **M05X kernel (`m05x.rs`):**
    Read `m05x.cpp:40-55` for the exact body. The exchange is spin-decomposed:
    - alpha: `fw(M05X_param_a, d.a, d.taua) * pbex::energy_pbe_ab(d.a, d.gaa)`
    - beta:  `fw(M05X_param_a, d.b, d.taub) * pbex::energy_pbe_ab(d.b, d.gbb)`
    - out = alpha + beta
    The 12 `param_a` constants for M05X come from `m05x.cpp`. Pass them as a cubecl `Array<F>` seeded on the host side (or as 12 individual `F` arguments — check what `m0x_fw` in m0x_like.rs expects; the planner chose the API in Plan 04-00 Wave 0 Task 2).

    ```rust
    // m05x.rs — port of xcfun-master/src/functionals/m05x.cpp
    // Vars: id=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB)
    const M05X_PARAM_A: [f64; 12] = [/* values from m05x.cpp */];

    #[cube(launch_unchecked)]
    pub fn m05x_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        // ... spin-decomposed fw * pbe_exchange ...
    }
    ```

    **M05X2X kernel (`m05x2x.rs`):** Identical to M05X but with `M05X2X_PARAM_A` constants from `m05x2x.cpp`.

    **M05C kernel (`m05c.rs`):** `m05_c_para(d) + m05_c_anti(d)` via `m0x_like::m05_c_para` + `m0x_like::m05_c_anti`.

    **M05X2C kernel (`m05x2c.rs`):** Same pattern as M05C but with M05-2X correlation parameters from `m05x2c.cpp`.

    **BLOCX kernel (`blocx.rs`):**
    ```rust
    // blocx.rs — port of xcfun-master/src/functionals/blocx.cpp:18-46
    // BLOCX is a TPSS-shaped enhancement: NO BRX dependency (confirmed by RESEARCH).
    // Vars: id=13 (XC_A_B_GAA_GAB_GBB_TAUA_TAUB)
    #[cube(launch_unchecked)]
    pub fn blocx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        super::shared::blocx::blocx_energy::<F>(d, out, n);
    }
    ```

    **Update `mgga/mod.rs`:** add M05 family (4) + BLOCX (1) module declarations.

    No `mul_add` in any kernel. No fast-math. Strict C++ port order for all polynomial evaluations.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~m05) | test(~blocx)'` — all 5 tier-1 pass at strict 1e-12.
    3. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/m05x.rs` returns empty.
    4. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/blocx.rs` returns empty.
    5. `grep -rn "BR\|br_scalar\|br_inverse" crates/xcfun-eval/src/functionals/mgga/blocx.rs` returns empty (BLOCX has zero BRX dependency).
  </acceptance_criteria>
  <done>M05 family (4) + BLOCX (1) tier-1 GREEN. BLOCX confirmed BRX-independent.</done>
</task>

<task id="3.2" type="auto">
  <name>Task 2: M06 family (8 kernels) + dispatch + validation harness extension</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/m06x.rs,
    crates/xcfun-eval/src/functionals/mgga/m06c.rs,
    crates/xcfun-eval/src/functionals/mgga/m06lx.rs,
    crates/xcfun-eval/src/functionals/mgga/m06lc.rs,
    crates/xcfun-eval/src/functionals/mgga/m06hfx.rs,
    crates/xcfun-eval/src/functionals/mgga/m06hfc.rs,
    crates/xcfun-eval/src/functionals/mgga/m06x2x.rs,
    crates/xcfun-eval/src/functionals/mgga/m06x2c.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs,
    crates/xcfun-eval/src/dispatch.rs,
    validation/build.rs,
    validation/c_stubs.cpp
  </files>
  <read_first>
    - `xcfun-master/src/functionals/m06x.cpp` (93 LOC) — M06X exchange: largest exchange body in M06 family. Uses 12-coef + 6-coef parameter sets. Read verbatim.
    - `xcfun-master/src/functionals/m06c.cpp` (109 LOC) — largest M06 body overall. Uses `m06_c_para(d) + m06_c_anti(d)`.
    - `xcfun-master/src/functionals/m06lx.cpp` (74 LOC) — M06-L exchange (local; no exact exchange in alias).
    - `xcfun-master/src/functionals/m06lc.cpp` (69 LOC).
    - `xcfun-master/src/functionals/m06hfx.cpp` (74 LOC) — M06-HF exchange (100% HF-like).
    - `xcfun-master/src/functionals/m06hfc.cpp` (96 LOC).
    - `xcfun-master/src/functionals/m06x2x.cpp` (61 LOC) — M06-2X exchange (2× exact exchange in alias).
    - `xcfun-master/src/functionals/m06x2c.cpp` (78 LOC).
    - `xcfun-master/src/functionals/m0xy_fun.hpp` lines 150-262 — `m06_c_anti`, `m06_c_para` (M06 correlation helpers); same `fw` as M05.
    - `crates/xcfun-eval/src/dispatch.rs` — add 13 arms (5 M05/BLOCX from Task 1 + 8 M06 here).
    - `validation/build.rs` — append 13 C++ files (5 M05/BLOCX + 8 M06).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-11 M06 watch list, D-08 dispatch bitmap (87 total after this plan)
  </read_first>
  <action>
    **M06X (`m06x.rs`):** Read `m06x.cpp` verbatim. The M06X exchange uses two parameter arrays (a 12-coef array for `fw` and a 6-coef array for `h`). Port the full body respecting the C++ computation order. The `h` function comes from `m0xy_fun.hpp:m0x_h`. No mul_add.

    **M06C (`m06c.rs`):** Read `m06c.cpp` verbatim (largest M06 body). Uses `m06_c_para(d) + m06_c_anti(d)` from `m0x_like::m06_c_para` + `m0x_like::m06_c_anti`.

    **M06LX, M06LC, M06HFX, M06HFC, M06X2X, M06X2C:** Follow the same patterns as M06X/M06C but with different parameter constant arrays. Read each C++ file for the exact parameter values — they CANNOT be guessed.

    For all 8 M06 kernels: the exchange variants differ ONLY in the `param_a` constants; the correlation variants differ ONLY in correlation parameters. Consider whether the `m0x_like` helpers already parametrise over these (Plan 04-00 Task 2 should have designed this). If not, pass parameters as host-side seeded `Array<F>` per kernel.

    **ACCURACY RULE for M06:** Port `m06c.cpp` expression-by-expression in exact C++ source order. Do not reorder additions. Do not combine multiplications. The 12-coefficient polynomial is evaluated left-to-right in C++ (Horner scheme) — replicate this exactly.

    **Update `mgga/mod.rs`:** add 8 M06 module declarations.

    **Update `dispatch.rs`:** Add 13 comptime arms (5 from Task 1: M05x4 + BLOCX, plus 8 M06 here). Look up FunctionalId discriminants for all 13. Update `supports()` bitmap — after this task, bitmap covers 82 functional IDs (46 GGA/LDA + 36 metaGGA).

    **Update `validation/build.rs`:** Append 13 C++ files (5 M05/BLOCX + 8 M06):
    - m05x.cpp, m05c.cpp, m05x2x.cpp, m05x2c.cpp, blocx.cpp
    - m06x.cpp, m06c.cpp, m06lx.cpp, m06lc.cpp, m06hfx.cpp, m06hfc.cpp, m06x2x.cpp, m06x2c.cpp
    Also include `m0xy_fun.hpp` path in the cc include search path if needed.

    **Update `validation/c_stubs.cpp`:** Remove stubs for all 13 ids.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~m06)'` — all 8 tier-1 pass at strict 1e-12 (or explicit D-19 INCONCLUSIVE entry per D-11 watch list).
    3. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/m06c.rs` returns empty (largest M06 body).
    4. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/m06x.rs` returns empty.
    5. `grep -n "XC_M06X\|XC_M06X2C\|XC_BLOCX" crates/xcfun-eval/src/dispatch.rs` returns 3 match arms.
    6. `cargo xtask validate --backend cpu --order 2 --filter 'm06'` exits 0 with zero failures (or D-19 entries forwarded).
    7. `cargo test -p xcfun-eval --test self_tests --features testing` overall still exits 0 (no regressions on earlier families).
  </acceptance_criteria>
  <done>All 13 Wave-3 kernels compile. dispatch.rs has 82 supported IDs. M06 tier-1 GREEN (or D-19 forwarded). xtask validate order=2 filter=m06 exits 0.</done>
</task>

</tasks>

<verification>
```bash
# Build
cargo build -p xcfun-eval --release 2>&1 | tail -5

# All M0x + BLOCX tier-1
cargo nextest run -p xcfun-eval --test self_tests --features testing \
  -E 'test(~m05) | test(~m06) | test(~blocx)' 2>&1 | tail -30

# Mid-tier parity
cargo xtask validate --backend cpu --order 2 --filter 'm05|m06|blocx' 2>&1 | tail -10

# Dispatch bitmap check
grep -c "comptime!(id ==" crates/xcfun-eval/src/dispatch.rs

# No mul_add in M06 (highest risk)
grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/m06c.rs
grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/m06x.rs

# BLOCX BRX-independence confirmation
grep -rn "BR\|br_scalar" crates/xcfun-eval/src/functionals/mgga/blocx.rs
```
</verification>

<success_criteria>
- All 13 Wave-3 kernels compile and tier-1 pass at strict 1e-12 (or D-19 forwarded per D-11 watch list).
- `cargo xtask validate --backend cpu --order 2 --filter 'm05|m06|blocx'` exits 0.
- dispatch.rs `supports()` bitmap reaches 82 functional IDs.
- No `mul_add` in any M05/M06/BLOCX file.
- BLOCX confirmed BRX-independent by grep.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-eval internals | No untrusted data — pure kernel ports; no new FFI surface |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-03-01 | Tampering | M06C 12-coefficient polynomial evaluation order | mitigate | Port verbatim from m06c.cpp. Apply `xtask check-no-mul-add` gate after porting. If tier-1 fails at 1e-12, create D-19 INCONCLUSIVE entry and forward to Phase 6 per D-11 watch list — do NOT widen tolerance silently. |
| T-04-03-02 | Tampering | M06X parameter arrays — wrong constants | mitigate | Read exact constant values from C++ source files; never guess. Add assertion comment citing source file and line number for each parameter array. |
| T-04-03-03 | Information Disclosure | None | accept | No new FFI surface. Pure internal plan. |

No new attack surface. Threats inherited from Phase 1 FFI gate.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-03-SUMMARY.md`
</output>
