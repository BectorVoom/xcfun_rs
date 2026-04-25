---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "01"
type: execute
wave: 2
depends_on:
  - "04-00"
requirements:
  - MGGA-01
files_modified:
  - crates/xcfun-eval/src/functionals/mgga/tpssx.rs
  - crates/xcfun-eval/src/functionals/mgga/tpssc.rs
  - crates/xcfun-eval/src/functionals/mgga/revtpssx.rs
  - crates/xcfun-eval/src/functionals/mgga/revtpssc.rs
  - crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs
  - crates/xcfun-eval/src/functionals/mgga/brx.rs
  - crates/xcfun-eval/src/functionals/mgga/brc.rs
  - crates/xcfun-eval/src/functionals/mgga/brxc.rs
  - crates/xcfun-eval/src/functionals/mgga/csc.rs
  - crates/xcfun-eval/src/functionals/mgga/mod.rs
  - crates/xcfun-eval/src/dispatch.rs
  - validation/build.rs
  - validation/c_stubs.cpp
autonomous: true
created: "2026-04-25"
goal: "Wave 1 — TPSS family (5) + BR family (3) + CSC (1) kernel ports; dispatch extension to 55 arms; tier-1 self-tests GREEN at strict 1e-12"

must_haves:
  truths:
    - "XC_TPSSX, XC_TPSSC, XC_REVTPSSX, XC_REVTPSSC, XC_TPSSLOCC pass tier-1 self-test at strict 1e-12"
    - "XC_BRX, XC_BRC, XC_BRXC pass tier-1 self-test at strict 1e-12"
    - "XC_CSC passes tier-1 self-test at strict 1e-12"
    - "dispatch_kernel recognises all 9 new functional IDs (ids for TPSS/BR/CSC families)"
    - "cargo xtask validate --backend cpu --order 2 --filter 'tpss' reports zero failures"
  artifacts:
    - path: crates/xcfun-eval/src/functionals/mgga/tpssx.rs
      provides: tpssx_kernel #[cube] fn
      exports: [tpssx_kernel]
    - path: crates/xcfun-eval/src/functionals/mgga/brx.rs
      provides: brx_kernel, brc_kernel, brxc_kernel (all three in one file mirroring brx.cpp)
      exports: [brx_kernel, brc_kernel, brxc_kernel]
    - path: crates/xcfun-eval/src/functionals/mgga/csc.rs
      provides: csc_kernel #[cube] fn
      exports: [csc_kernel]
  key_links:
    - from: crates/xcfun-eval/src/dispatch.rs
      to: crates/xcfun-eval/src/functionals/mgga/tpssx.rs
      via: comptime id match arm calling tpssx_kernel
      pattern: "XC_TPSSX"
    - from: crates/xcfun-eval/src/functionals/mgga/brx.rs
      to: crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs
      via: call to polarized() helper
      pattern: "br_like::polarized"
---

<objective>
Port 9 functional bodies (TPSS ×5, BR ×3, CSC ×1) as `#[cube] fn <name>_kernel<F: Float, const N: u32>` functions and wire them into the dispatch table. These are the two Vars clusters for Wave 1: TPSS/CSC/BLOCX use id=13, and BR/CSC use id=17. Both arms ship from Plan 04-00.

Purpose: TPSS is the entry-point metaGGA family (under 200 LOC combined across 5 files). BR ships first because RESEARCH confirmed BLOCX does NOT depend on BR (CONTEXT D-01-A claim corrected). CSC ships alongside BR because both need the id=17 Vars arm.

Output: 9 kernel files, 1 updated dispatch.rs (+9 arms), updated validation/build.rs (+9 C++ source files), tier-1 self-tests GREEN.
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
<!-- From Plan 04-00 substrate (must exist before executing this plan). -->

From crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs (Wave 0):
```rust
#[cube] pub fn tpss_fx_unif<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn tpss_F_x<F: Float>(rho: &Array<F>, grad2: &Array<F>, tau: &Array<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn tpss_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn revtpss_fx<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
#[cube] pub fn revtpss_eps<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
```

From crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs (Wave 0):
```rust
// polarized(na, gaa, lapa, taua, jpaa) -> CTaylor energy
#[cube] pub fn br_polarized<F: Float>(
    na: &Array<F>, gaa: &Array<F>, lapa: &Array<F>, taua: &Array<F>, jpaa: &Array<F>,
    out: &mut Array<F>, #[comptime] n: u32
);
```

From crates/xcfun-eval/src/functionals/mgga/shared/cs.rs (Wave 0):
```rust
#[cube] pub fn csc_energy<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32);
```

From crates/xcfun-eval/src/functionals/gga/becke/beckex.rs (kernel pattern analog):
```rust
#[cube(launch_unchecked)]
pub fn beckex_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) { ... }
```

From crates/xcfun-eval/src/dispatch.rs (dispatch arm pattern, existing):
```rust
} else if comptime!(id == FunctionalId::XC_BECKEX as u32) {
    beckex_kernel::<F>(d, out, n);
}
```
</interfaces>
</context>

<tasks>

<task id="1.1" type="auto">
  <name>Task 1: TPSS family (5 kernels) + validation harness extension</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/tpssx.rs,
    crates/xcfun-eval/src/functionals/mgga/tpssc.rs,
    crates/xcfun-eval/src/functionals/mgga/revtpssx.rs,
    crates/xcfun-eval/src/functionals/mgga/revtpssc.rs,
    crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs,
    crates/xcfun-eval/src/dispatch.rs,
    validation/build.rs,
    validation/c_stubs.cpp
  </files>
  <read_first>
    - `xcfun-master/src/functionals/tpssx.cpp` — AUTHORITATIVE port source (56 LOC). The body: `d.a * tpss_eps::F_x(d.a, d.gaa, d.taua) + d.b * tpss_eps::F_x(d.b, d.gbb, d.taub)`. Vars = id=13.
    - `xcfun-master/src/functionals/tpssc.cpp` — single-line body: `d.n * tpssc_eps::tpssc_eps(d)`. Vars = id=13.
    - `xcfun-master/src/functionals/revtpssx.cpp` — revTPSS exchange (32 LOC).
    - `xcfun-master/src/functionals/revtpssc.cpp` — revTPSS correlation (33 LOC).
    - `xcfun-master/src/functionals/tpsslocc.cpp` — TPSS-LOCC (105 LOC). Largest TPSS body; composes pw92eps + TPSS structural pieces inline. Vars = id=13.
    - `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs` — shared helpers (from Plan 04-00).
    - `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` — analog kernel pattern.
    - `crates/xcfun-eval/src/dispatch.rs` — READ FULLY; add 5 new arms for TPSS family.
    - `validation/build.rs` — READ to find the `cc::Build::file(...)` list; append tpss family C++ files.
    - `validation/c_stubs.cpp` — READ to remove stubs for TPSS ids that now have real C++ implementations in build.rs.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-01, D-01-A Wave 1, D-08
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-PATTERNS.md` — D.1 wave 1 kernel rows
  </read_first>
  <action>
    Create 5 kernel files under `crates/xcfun-eval/src/functionals/mgga/`:

    **tpssx.rs** — port of `tpssx.cpp`. Signature:
    ```rust
    #[cube(launch_unchecked)]
    pub fn tpssx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        let mut ea = Array::<F>::new(comptime!((1_u32 << n) as usize));
        let mut eb = Array::<F>::new(comptime!((1_u32 << n) as usize));
        // ea = d.a * F_x(d.a, d.gaa, d.taua) via tpss_like::tpss_F_x
        // eb = d.b * F_x(d.b, d.gbb, d.taub) via tpss_like::tpss_F_x
        // out = ea + eb  via ctaylor_add
    }
    ```
    Spin-decomposed exchange matching `tpssx.cpp` verbatim. Use `ctaylor_scalar_mul` for the `d.a * ...` and `d.b * ...` multiplications (NOT `ctaylor_mul` if alpha/beta are Taylor polynomials — check how beckex.rs does it). No mul_add.

    **tpssc.rs** — port of `tpssc.cpp`. Single call: `tpssc_eps(d)` × `d.n`. Body:
    ```rust
    #[cube(launch_unchecked)]
    pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        let mut eps = Array::<F>::new(comptime!((1_u32 << n) as usize));
        tpss_like::tpss_eps::<F>(d, &mut eps, n);
        ctaylor_mul::<F>(&d.n, &eps, out, n);
    }
    ```

    **revtpssx.rs** — port of `revtpssx.cpp`. Similar spin-decomposed exchange using `revtpss_fx` helper.

    **revtpssc.rs** — port of `revtpssc.cpp`. Similar correlation using `revtpss_eps` helper.

    **tpsslocc.rs** — port of `tpsslocc.cpp` (105 LOC). This is the most complex TPSS body; includes TPSS-LOCC local correlation structure. Port VERBATIM following the C++ order: compute `pw92_epsilon`, compute TPSS structural pieces (z, kappa, q-tilde), compute the LOCC correction. Use existing `xcfun_eval::functionals::lda::pw92eps::pw92eps_polarized` (already in xcfun-eval from Phase 2) for the PW92 pieces. No mul_add, no reordering.

    **Update `mgga/mod.rs`:** Uncomment / add:
    ```rust
    // Wave 1 (04-01): TPSS family
    pub mod tpssx;
    pub mod tpssc;
    pub mod revtpssx;
    pub mod revtpssc;
    pub mod tpsslocc;
    ```

    **Update `dispatch.rs`:** Add 5 comptime arms for the TPSS family after the existing GGA arms. Look up the exact `FunctionalId` enum discriminant values for `XC_TPSSX`, `XC_TPSSC`, `XC_REVTPSSX`, `XC_REVTPSSC`, `XC_TPSSLOCC` from `crates/xcfun-core/src/functional_id.rs`. Pattern:
    ```rust
    } else if comptime!(id == FunctionalId::XC_TPSSX as u32) {
        mgga::tpssx::tpssx_kernel::<F>(d, out, n);
    } else if comptime!(id == FunctionalId::XC_TPSSC as u32) {
        mgga::tpssc::tpssc_kernel::<F>(d, out, n);
    } // ... etc.
    ```
    Also update `supports()` bitmap to include the 5 new IDs.

    **Update `validation/build.rs`:** Append to the `cc::Build::file(...)` list:
    - `xcfun-master/src/functionals/tpssx.cpp`
    - `xcfun-master/src/functionals/tpssc.cpp`
    - `xcfun-master/src/functionals/revtpssx.cpp`
    - `xcfun-master/src/functionals/revtpssc.cpp`
    - `xcfun-master/src/functionals/tpsslocc.cpp`
    Also add the required helper `.hpp` paths if cc needs them (check if `.cpp` transitively includes `.hpp` via include-path or needs explicit file).

    **Update `validation/c_stubs.cpp`:** Remove the stub entries for `XC_TPSSX`, `XC_TPSSC`, `XC_REVTPSSX`, `XC_REVTPSSC`, `XC_TPSSLOCC` now that real implementations exist in build.rs.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo test -p xcfun-eval --test self_tests --features testing` — tier-1 for TPSS family (5 ids) passes at strict 1e-12. Run: `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~tpss)'`
    3. `grep -n "XC_TPSSX" crates/xcfun-eval/src/dispatch.rs` returns a comptime match arm.
    4. `grep -c "tpss" validation/build.rs` shows 5 or more matches (5 C++ files appended).
    5. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/tpssx.rs` returns empty.
    6. `cargo xtask validate --backend cpu --order 2 --filter 'tpss'` exits 0 with zero failures at 1e-12.
  </acceptance_criteria>
  <done>All 5 TPSS kernels ship; tier-1 GREEN; xtask validate filter=tpss exits 0.</done>
</task>

<task id="1.2" type="auto">
  <name>Task 2: BR family (3 kernels) + CSC (1 kernel) port; dispatch extension</name>
  <files>
    crates/xcfun-eval/src/functionals/mgga/brx.rs,
    crates/xcfun-eval/src/functionals/mgga/brc.rs,
    crates/xcfun-eval/src/functionals/mgga/brxc.rs,
    crates/xcfun-eval/src/functionals/mgga/csc.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs,
    crates/xcfun-eval/src/dispatch.rs,
    validation/build.rs,
    validation/c_stubs.cpp
  </files>
  <read_first>
    - `xcfun-master/src/functionals/brx.cpp` — FULL READ (157 LOC). ALL THREE functionals (BRX/BRC/BRXC) are defined in this single file.
      - `BR_z(x)` at line 21-23.
      - Scalar Newton `BR(z)->x` at lines 29-48.
      - `BR_taylor<T,Ndeg>` at lines 53-71 (already ported as `ctaylor_br_inverse` in Plan 04-00 Task 1).
      - ctaylor-level `BR(t)` wrapper at lines 78-87 (ported in `br_like.rs`).
      - `polarized(na, gaa, lapa, taua, jpaa)` helper at lines 89-101 (ported in `br_like.rs`).
      - `XC_BRX` energy body at lines 103-106: `polarized(d.a, d.gaa, d.lapa, d.taua, d.jpaa) + polarized(d.b, d.gbb, d.lapb, d.taub, d.jpbb)`.
      - `XC_BRC` energy body at lines 108-121: uses `cab=0.63`, `caa=0.88` with abs() and log/pow algebra.
      - `XC_BRXC` energy body at lines 123-136: exchange + correlation combined.
      - Vars registration at lines 138-157: `XC_DENSITY | XC_GRADIENT | XC_KINETIC | XC_LAPLACIAN | XC_JP` → id=17.
    - `xcfun-master/src/functionals/cs.cpp` — FULL READ (35 LOC, 10 active). CSC energy at lines 17-27.
    - `crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs` — `br_polarized` helper (from Plan 04-00).
    - `crates/xcfun-eval/src/functionals/mgga/shared/cs.rs` — `csc_energy` helper (from Plan 04-00).
    - `crates/xcfun-eval/src/dispatch.rs` — READ current state; add 4 new arms.
    - `validation/build.rs` — append brx.cpp and cs.cpp.
    - `validation/c_stubs.cpp` — remove BRX/BRC/BRXC/CSC stubs.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-01, D-01-A, D-03-A (id=17 arm shared by BR/CSC)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md` — BR Newton libm-drift warning (manual check item)
  </read_first>
  <action>
    **NOTE from RESEARCH:** BLOCX does NOT depend on BR family. The three BR files are independent of BLOCX. CSC also uses id=17.

    **CRITICAL HAZARD:** BR Newton-inverse calls `f64::exp` in the host-side scalar `br_scalar`. If libm `exp` on the CI runner differs ≥ 1 ULP from the C++ runner, the Newton trajectory diverges. Per Phase 1 D-03 escalation rule: if fixture-gate hits > 1e-12, escalate via PLANNING INCONCLUSIVE. Do NOT widen tolerance silently.

    Create `crates/xcfun-eval/src/functionals/mgga/brx.rs` — contains ALL THREE BRX/BRC/BRXC kernels (mirroring the single-file structure of `brx.cpp`):

    ```rust
    // brx.rs — port of xcfun-master/src/functionals/brx.cpp:103-157
    // Contains BRX, BRC, BRXC kernels; all use Vars id=17 (inlen=11, JP arm).

    #[cube(launch_unchecked)]
    pub fn brx_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        // Port of brx.cpp:103-106:
        // E = polarized(d.a, d.gaa, d.lapa, d.taua, d.jpaa)
        //   + polarized(d.b, d.gbb, d.lapb, d.taub, d.jpbb)
        let mut ea = Array::<F>::new(comptime!((1_u32 << n) as usize));
        let mut eb = Array::<F>::new(comptime!((1_u32 << n) as usize));
        br_like::br_polarized::<F>(&d.a, &d.gaa, &d.lapa, &d.taua, &d.jpaa, &mut ea, n);
        br_like::br_polarized::<F>(&d.b, &d.gbb, &d.lapb, &d.taub, &d.jpbb, &mut eb, n);
        ctaylor_add::<F>(&ea, &eb, out, n);
    }
    ```

    `brc_kernel` — port of `brx.cpp:108-121`. Uses `cab=0.63`, `caa=0.88` constants. The C++ body mixes `abs()` (for the logarithm argument) and `pow()`. The Taylor analog uses `ctaylor_pow`, `ctaylor_log`; the `abs()` applies to the CNST slot (constant-coefficient) only since derivatives of `|x|` are undefined at zero — check C++ source for whether a branch is needed. Port verbatim.

    `brxc_kernel` — port of `brx.cpp:123-136`.

    **brc.rs and brxc.rs:** These can be thin wrapper files that re-export from brx.rs:
    ```rust
    // brc.rs
    pub use crate::functionals::mgga::brx::brc_kernel;
    ```
    Or inline. Planner picks; thin re-exports are cleaner.

    **csc.rs** — port of `cs.cpp:17-27`:
    ```rust
    #[cube(launch_unchecked)]
    pub fn csc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
        cs::csc_energy::<F>(d, out, n);
    }
    ```

    **Update `mgga/mod.rs`:** add:
    ```rust
    // Wave 1 (04-01): BR family + CSC (GGA-03 / GGA-10 Phase-3 carryovers)
    pub mod brx;   // contains brx_kernel, brc_kernel, brxc_kernel
    pub mod brc;   // re-export thin wrapper
    pub mod brxc;  // re-export thin wrapper
    pub mod csc;
    ```

    **Update `dispatch.rs`:** Add 4 arms for BRX/BRC/BRXC/CSC (ids from `FunctionalId` enum). Update `supports()` bitmap.

    **Update `validation/build.rs`:** Append `xcfun-master/src/functionals/brx.cpp` and `xcfun-master/src/functionals/cs.cpp`.

    **Update `validation/c_stubs.cpp`:** Remove stubs for BRX, BRC, BRXC, CSC.

    STRICT 1e-12 gate: if tier-1 self-test for BRX/BRC/BRXC fails due to BR Newton libm drift, add a structured D-19 INCONCLUSIVE note and escalate per Phase 1 D-03. DO NOT widen tolerance silently.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `cargo nextest run -p xcfun-eval --test self_tests --features testing -E 'test(~brx) | test(~brc) | test(~csc)'` — tier-1 self-tests for BRX/BRC/BRXC/CSC pass at strict 1e-12.
    3. `grep -n "XC_BRX" crates/xcfun-eval/src/dispatch.rs` returns a comptime match arm.
    4. `grep -n "XC_CSC" crates/xcfun-eval/src/dispatch.rs` returns a comptime match arm.
    5. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/brx.rs` returns empty.
    6. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/csc.rs` returns empty.
    7. `cargo xtask validate --backend cpu --order 2 --filter 'br'` exits 0 with zero failures.
    8. `cargo test -p xcfun-eval --test self_tests --features testing` overall still exits 0 (no regressions on TPSS or earlier families).
  </acceptance_criteria>
  <done>BRX/BRC/BRXC/CSC tier-1 GREEN at strict 1e-12. dispatch.rs has 4 new arms. validation/build.rs updated. No mul_add.</done>
</task>

</tasks>

<verification>
```bash
# Build check
cargo build -p xcfun-eval --release 2>&1 | tail -5

# Tier-1 self-tests for Wave 1 families
cargo nextest run -p xcfun-eval --test self_tests --features testing \
  -E 'test(~tpss) | test(~brx) | test(~brc) | test(~brxc) | test(~csc)' 2>&1 | tail -20

# Parity validation for TPSS (mid-tier)
cargo xtask validate --backend cpu --order 2 --filter 'tpss' 2>&1 | tail -10

# Structural checks
grep -n "XC_TPSSX\|XC_BRX\|XC_CSC" crates/xcfun-eval/src/dispatch.rs | head -10
grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/ 2>&1 | grep -v "^Binary"
```
</verification>

<success_criteria>
- 9 functional kernels compile and ship under `mgga/`.
- All 9 tier-1 self-tests GREEN at strict 1e-12 (or explicit D-19 INCONCLUSIVE entry if BR Newton drifts).
- `cargo xtask validate --backend cpu --order 2 --filter 'tpss'` exits 0.
- No `mul_add` in any new mgga kernel file.
- dispatch.rs `supports()` bitmap includes all 9 new IDs.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-eval internals | No untrusted data — pure numerical kernel ports; no FFI entry points added in this plan |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-01-01 | Tampering | BR Newton abs() branch in brc_kernel | accept | The `abs()` in `brx.cpp:110` applies to the CNST slot only (functional value at the density point); higher Taylor coefficients follow the sign of the value. Port verbatim; no branching on Taylor coefficients. No untrusted input path in this plan. |
| T-04-01-02 | Information Disclosure | None | accept | No new FFI surface. Pure internal implementation plan. |

No new attack surface. Threats inherited from Phase 1 FFI gate.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-01-SUMMARY.md`
</output>
