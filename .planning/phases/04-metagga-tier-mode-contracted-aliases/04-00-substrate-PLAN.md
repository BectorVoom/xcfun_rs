---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "00"
type: execute
wave: 1
depends_on: []
requirements:
  - MGGA-01
  - MGGA-02
  - MGGA-03
  - MGGA-04
  - MGGA-05
files_modified:
  - crates/xcfun-ad/src/expand/br_inverse.rs
  - crates/xcfun-ad/src/expand/mod.rs
  - crates/xcfun-ad/src/math.rs
  - crates/xcfun-ad/src/lib.rs
  - crates/xcfun-ad/tests/golden_br_inverse.rs
  - crates/xcfun-ad/tests/test_ctaylor_n6.rs
  - xtask/src/bin/regen_ad_fixtures.rs
  - crates/xcfun-eval/src/density_vars/build.rs
  - crates/xcfun-eval/tests/regularize_mgga_invariant.rs
  - crates/xcfun-eval/src/functionals/mod.rs
  - crates/xcfun-eval/src/functionals/mgga/mod.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/mod.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/constants.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/cs.rs
  - validation/src/fixtures.rs
autonomous: true
created: "2026-04-25"
goal: "Wave 0 substrate — ctaylor_br_inverse primitive, CTaylor<F,6> smoke test, DensVarsDev metaGGA arms (id=13+id=17), mgga module tree + 7 shared helpers, metaGGA grid stratum"

must_haves:
  truths:
    - "ctaylor_br_inverse at strict 1e-12 vs C++ BR_taylor on 30 z-points × N in {2,3,4}"
    - "CTaylor<F, 6> allocates and multiplies without panic on cubecl-cpu"
    - "build_densvars accepts Vars id=13 (TAUA_TAUB, inlen=7) and id=17 (full JP, inlen=11)"
    - "All 7 mgga/shared helpers compile (tpss_like, scan_like, m0x_like, br_like, blocx, cs, constants)"
    - "metaGGA grid stratum (seed 0xc0ffee01, 1000 points) generates without panic"
  artifacts:
    - path: crates/xcfun-ad/src/expand/br_inverse.rs
      provides: br_scalar (host Newton) + br_inverse_expand #[cube] fn
      exports: [br_scalar, br_inverse_expand, ctaylor_br_inverse]
    - path: crates/xcfun-eval/src/density_vars/build.rs
      provides: build_xc_a_b_gaa_gab_gbb_taua_taub (id=13) + build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb (id=17)
      contains: "comptime!(vars == 13)"
    - path: crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs
      provides: get_SCAN_Fx + r2SCAN_C (IDELEC comptime dispatch)
      min_lines: 200
    - path: validation/src/fixtures.rs
      provides: metaGGA stratum at seed 0xc0ffee01
      contains: "0xc0ffee01"
  key_links:
    - from: crates/xcfun-ad/src/math.rs
      to: crates/xcfun-ad/src/expand/br_inverse.rs
      via: ctaylor_br_inverse call to br_inverse_expand
      pattern: "ctaylor_br_inverse"
    - from: crates/xcfun-eval/src/density_vars/build.rs
      to: build_xc_a_b_gaa_gab_gbb_taua_taub
      via: comptime if-chain in build_densvars
      pattern: "comptime!(vars == 13)"
---

<objective>
Atomic Wave 0 substrate delivery: the single new xcfun-ad primitive `ctaylor_br_inverse` (port of `brx.cpp:25-72`), a CTaylor<F,6> smoke test (D-07), the two mandatory DensVarsDev arms for ids 13 and 17 (D-03/D-03-A), the metaGGA module tree (`mgga/`) with 7 shared helpers (D-01-A Wave 0), and the metaGGA grid stratum extension (D-09-A). No functional kernels land in this plan — only the substrate that Waves 1-3 depend on.

Purpose: Waves 1, 2, 3 cannot make correct kernel calls without the Vars arms. The BR family cannot be tested without `ctaylor_br_inverse`. The SCAN family cannot be tested without `scan_like.rs`. All must be GREEN at strict 1e-12 before any kernel ships.

Output: 20 files created/modified; fixture-gate for BR inverse at strict 1e-12; cargo test GREEN.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md

<interfaces>
<!-- Key patterns extracted from existing Phase 1/2/3 substrate. -->

From crates/xcfun-ad/src/expand/sqrt.rs (expand-helper analog):
```rust
use cubecl::prelude::*;

#[cube]
pub fn sqrt_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // recurrence writes into t[0..n]
}
```

From crates/xcfun-ad/src/math.rs (ctaylor_* wrapper pattern):
```rust
#[cube]
pub fn ctaylor_sqrt<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);
    sqrt_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}
```

From crates/xcfun-eval/src/density_vars/build.rs (arm pattern — line ~86):
```rust
} else if comptime!(vars == 6) {
    build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
}
```

From crates/xcfun-eval/src/functionals/gga/shared/mod.rs (helper index pattern):
```rust
pub mod constants;
pub mod pbex;
pub mod pbec_eps;
pub mod pw91_like;
pub mod b97_poly;
pub mod optx;
```

From crates/xcfun-eval/src/functionals/gga/mod.rs (mod root pattern):
```rust
pub mod shared;
pub mod pbe;
pub mod becke;
// ... family modules by wave
```

From validation/src/fixtures.rs (grid generator pattern — existing strata):
```rust
const CANONICAL_SEED: u64 = 0x1234abcd;
// generate_grid produces 10k points across 4 strata
```
</interfaces>
</context>

<tasks>

<task id="0.1" type="auto" tdd="true">
  <name>Task 1: ctaylor_br_inverse primitive (D-02)</name>
  <files>
    crates/xcfun-ad/src/expand/br_inverse.rs,
    crates/xcfun-ad/src/expand/mod.rs,
    crates/xcfun-ad/src/math.rs,
    crates/xcfun-ad/src/lib.rs,
    xtask/src/bin/regen_ad_fixtures.rs,
    crates/xcfun-ad/tests/golden_br_inverse.rs,
    crates/xcfun-ad/tests/test_ctaylor_n6.rs
  </files>
  <read_first>
    - `crates/xcfun-ad/src/expand/sqrt.rs` — expand-helper shape (recurrence writer into `&mut Array<F>`)
    - `crates/xcfun-ad/src/math.rs` — ctaylor_* wrapper pattern (ctaylor_sqrt, ctaylor_reciprocal)
    - `crates/xcfun-ad/src/expand/mod.rs` — to see existing `pub mod` list
    - `crates/xcfun-ad/src/lib.rs` — to see existing re-exports
    - `crates/xcfun-ad/tests/golden_mul.rs` — golden fixture test pattern
    - `xtask/src/bin/regen_ad_fixtures.rs` — existing fixture generator pattern
    - `xcfun-master/src/functionals/brx.cpp` lines 21-87 — AUTHORITATIVE C++ port target:
      - `BR_z(x)` at line 21-23: `(x - 2.0) / x * exp(2.0 * x / 3.0)`
      - Four Newton initial-guess branches at lines 29-40
      - 20-iter Newton loop at lines 41-47 with `1e-15 * (1 + |x|)` rel-tolerance
      - `BR` ctaylor wrapper at lines 78-87 (calls `BR_taylor<T, Nvar>`)
      - `BR_taylor<T, Ndeg>` at lines 53-71 — the Brent-Kung linear-method polynomial sweep
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-02, D-02-A, D-07
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-PATTERNS.md` — A.1 (br_inverse.rs pattern), A.2 (golden_br_inverse.rs pattern)
  </read_first>
  <behavior>
    - Test 1: `br_scalar(-2.0)` converges to a finite f64 in 20 iterations
    - Test 2: For any converged `x = br_scalar(z)`, `br_z(x) - z` < 1e-13 relative
    - Test 3: `ctaylor_br_inverse` at N=2 on 3 z-values matches C++ `BR_taylor` at strict 1e-12
    - Test 4: CTaylor<f64, 6> multiply-by-self result at N=6 is finite and non-NaN
  </behavior>
  <action>
    Per D-02 and PATTERNS A.1, implement three components:

    **1. `crates/xcfun-ad/src/expand/br_inverse.rs`:**

    Host-side scalar functions (plain `fn`, not `#[cube]`):
    ```rust
    // Port of brx.cpp:21-23
    #[inline] fn br_z(x: f64) -> f64 { (x - 2.0) / x * ((2.0 / 3.0) * x).exp() }

    // Port of brx.cpp:29-48 (Newton-Raphson root finder for BR(z)->x)
    pub fn br_scalar(z: f64) -> f64 {
        // Four initial-guess branches per brx.cpp:29-40:
        let mut x = if z < -1.0e4 { -2.0 / z }
            else if z < -2.0 { ((9.0*z*z + 6.0*z + 49.0_f64).sqrt() + 3.0*z + 1.0) / 4.0 }
            else if z < 1.0  { 2.0 * (z * (-4.0_f64/3.0_f64).exp() + 1.0) }
            else              { 1.5 * z.ln() + 3.75 / (1.5 + z.ln()) };
        // 20-iter Newton per brx.cpp:41-47 with convergence: |dx| < 1e-15 * (1 + |x|)
        for _ in 0..20 {
            let f = br_z(x) - z;
            // f'(x) = d/dx [(x-2)/x * exp(2x/3)] = [(-2/x^2) * exp(2x/3) + (x-2)/x * (2/3)*exp(2x/3)]
            //       = exp(2x/3) * [(-2/x^2) + (2/3)*(x-2)/x]
            let exp_val = ((2.0 / 3.0) * x).exp();
            let fp = exp_val * ((-2.0 / (x * x)) + (2.0 / 3.0) * (x - 2.0) / x);
            let dx = -f / fp;
            x += dx;
            if dx.abs() < 1.0e-15 * (1.0 + x.abs()) { return x; }
        }
        x  // return best estimate (matches C++ brx.cpp:47)
    }
    ```

    `#[cube] fn br_inverse_expand` — linear-method (Brent-Kung) polynomial sweep per `brx.cpp:53-71` (`BR_taylor<T, Ndeg>`). Signature:
    ```rust
    #[cube]
    pub fn br_inverse_expand<F: Float>(t: &mut Array<F>, z0: F, #[comptime] n: u32) {
        // t[0] must be pre-seeded with br_scalar(z0) by the caller (host-side Newton).
        // Step: t[1] = 1; evaluate f = BR_z(t) via ctaylor ops; t[1] = 1 / f[1].
        // Then for i in 2..=max(n,3): evaluate f = BR_z(t); t[i] = -f[i] * t[1].
        // BR_z(t) = (t - 2) / t * exp((2/3)*t) using ctaylor_sub, ctaylor_reciprocal,
        //           ctaylor_scalar_mul, ctaylor_mul, ctaylor_exp.
    }
    ```
    The `#[cube]` body must replicate `brx.cpp:58-70` verbatim: the comptime-recurrence sets coefficients one-by-one. IMPORTANT: This is NOT a `ctaylor_compose` pattern — it is a direct linear-method that writes `t[i]` from `f[i]` computed from the partial-t state. The `n` argument governs depth; per `brx.cpp:80` (`(Nvar >= 3) ? Nvar : 3`), use `comptime!(if n < 3 { 3 } else { n })` as the working depth.

    **2. `crates/xcfun-ad/src/math.rs` (extend existing):**

    Add at the end:
    ```rust
    #[cube]
    pub fn ctaylor_br_inverse<F: Float>(z: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
        // Step 1: seed out[CNST] = br_scalar(z[CNST]) — host slot.
        // The host-side caller seeds out[0] before launching.
        // Step 2: call br_inverse_expand to populate out[1..n] via linear method.
        br_inverse_expand::<F>(out, z[CNST], n);
    }
    ```
    NOTE: The host-side caller (in the fixture test kernel) seeds `out[0] = br_scalar(z[0])` as a plain f64 before launching the `#[cube]` kernel. Inside the cube, `out[0]` is already set; `br_inverse_expand` starts from slot 1.

    **3. Update `crates/xcfun-ad/src/expand/mod.rs`:** add `pub mod br_inverse;`

    **4. Update `crates/xcfun-ad/src/lib.rs`:** re-export `pub use expand::br_inverse::{br_scalar, br_inverse_expand};` and `pub use math::ctaylor_br_inverse;`

    **5. `xtask/src/bin/regen_ad_fixtures.rs` (extend):** Add a `gen_br_inverse_fixtures()` function that generates 30 z-points (covering all four C++ initial-guess branches: 7 points in `z < -1e4`, 8 points in `-1e4 <= z < -2`, 8 points in `-2 <= z < 1`, 7 points in `z >= 1`) at orders N in {2, 3, 4}. For each (z, N), run C++ `BR_taylor` via the cc-compiled validation harness to get reference coefficients. Serialize to `crates/xcfun-ad/tests/fixtures/br_inverse.bincode` using the same FixtureRecord schema as `golden_mul.rs`.

    **6. `crates/xcfun-ad/tests/golden_br_inverse.rs`:** Follow `golden_mul.rs` pattern exactly. Load fixtures, for each record seed `out[0] = br_scalar(record.inputs[0])` (host-side), launch `ctaylor_br_inverse` via `cpu_client()`, assert `max_relative = 1e-12`. Feature-gate: `#![cfg(feature = "testing")]`.

    **7. `crates/xcfun-ad/tests/test_ctaylor_n6.rs` (D-07 capacity smoke test):** Create a minimal `#[cube(launch_unchecked)] fn ctaylor_n6_smoke<F: Float>(a: &Array<F>, b: &Array<F>, out: &mut Array<F>, #[comptime] n: u32)` that multiplies a by b at N=6 (Array<F> length 64) and write the result. Seed `a` and `b` with trivial f64 values (all-ones), launch, assert no panic and `out[0]` is finite. Strict 1e-12 comparison not needed — smoke test only. Per D-07.

    CRITICAL per CLAUDE.md: NO `mul_add` anywhere. No fast-math. `RUSTFLAGS` empty. `thiserror` only in library crates.
  </action>
  <acceptance_criteria>
    1. `cargo test -p xcfun-ad --features cpu test_ctaylor_n6` passes without panic.
    2. `cargo test -p xcfun-ad --features cpu golden_br_inverse` passes — 90 records (30 z-points × 3 orders) all at `max_relative = 1e-12` vs C++ `BR_taylor`. Command exits 0.
    3. `grep -n "br_inverse" crates/xcfun-ad/src/expand/mod.rs` finds `pub mod br_inverse;`
    4. `grep -n "ctaylor_br_inverse" crates/xcfun-ad/src/lib.rs` finds the re-export.
    5. `grep -rn "mul_add" crates/xcfun-ad/src/expand/br_inverse.rs` returns empty (no mul_add).
    6. `grep -rn "mul_add" crates/xcfun-ad/src/math.rs` returns empty on any new lines added.
    7. `cargo build -p xcfun-ad --features cpu` exits 0 with no warnings (other than pre-existing).
  </acceptance_criteria>
  <done>ctaylor_br_inverse fixture-gate GREEN at strict 1e-12. CTaylor N=6 smoke passes. Both compile warning-free.</done>
</task>

<task id="0.2" type="auto">
  <name>Task 2: DensVarsDev arms for id=13 and id=17 + mgga module tree + 7 shared helpers (D-03, D-01-A)</name>
  <files>
    crates/xcfun-eval/src/density_vars/build.rs,
    crates/xcfun-eval/tests/regularize_mgga_invariant.rs,
    crates/xcfun-eval/src/functionals/mod.rs,
    crates/xcfun-eval/src/functionals/mgga/mod.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/mod.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/constants.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs,
    crates/xcfun-eval/src/functionals/mgga/shared/cs.rs,
    validation/src/fixtures.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/src/density_vars/build.rs` — READ FULLY to see all existing arms (XC_A_B id=1, id=2, id=5, id=6, and Phase 3 arms). Must add id=13 and id=17 arms in the `build_densvars` comptime if-chain. Existing build_xc_a_b_gaa_gab_gbb at lines ~222-262 is the explicit-chain pattern.
    - `crates/xcfun-eval/src/density_vars.rs` — READ to confirm which fields exist on `DensVarsDev<F>`. RESEARCH confirms `jpaa`, `jpbb`, `lapa`, `lapb`, `tau`, `taua`, `taub` already exist.
    - `xcfun-master/src/densvars.hpp` lines 54-72 (id=13 TAUA_TAUB) and lines 187-208 (id=17 full JP) — AUTHORITATIVE field mapping.
    - `crates/xcfun-eval/src/functionals/gga/mod.rs` — module root pattern.
    - `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` — helper index pattern.
    - `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` — multi-formula `#[cube] fn` module pattern.
    - `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` — multi-formula module pattern (for scan_like.rs).
    - `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` — polynomial helper pattern (for m0x_like.rs).
    - `crates/xcfun-eval/tests/regularize_invariant.rs` — analog for the invariant test.
    - `xcfun-master/src/functionals/SCAN_like_eps.hpp` — FULL READ (522 lines) for scan_like.rs. Must port all exported functions: `get_SCAN_Fx`, `r2SCAN_C`, `scan_ec0`, `scan_ec1`, `lda_0`, `gcor2`, `get_lsda1`, `ufunc`.
    - `xcfun-master/src/functionals/tpssx_eps.hpp` — for tpss_like.rs `F_x`, `fx_unif`
    - `xcfun-master/src/functionals/tpssc_eps.hpp` — for tpss_like.rs `tpssc_eps`
    - `xcfun-master/src/functionals/revtpssx_eps.hpp` — for tpss_like.rs `revtpssx_eps`
    - `xcfun-master/src/functionals/revtpssc_eps.hpp` — for tpss_like.rs `revtpssc_eps`
    - `xcfun-master/src/functionals/m0xy_fun.hpp` — FULL READ (262 lines) for m0x_like.rs
    - `validation/src/fixtures.rs` — to see existing stratum pattern and add metaGGA stratum.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-03, D-03-A, D-03-B, D-09-A
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-PATTERNS.md` — B.1 (build.rs), B.2 (invariant test), C.1-C.7 (shared helpers)
  </read_first>
  <action>
    **1. `crates/xcfun-eval/src/density_vars/build.rs` — two new mandatory arms:**

    Add after the existing last arm in the `build_densvars` comptime if-chain:

    ```rust
    } else if comptime!(vars == 13) {
        // XC_A_B_GAA_GAB_GBB_TAUA_TAUB (densvars.hpp:54-72). id=13, inlen=7.
        build_xc_a_b_gaa_gab_gbb_taua_taub::<F>(input, out, n);
    } else if comptime!(vars == 17) {
        // XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB (densvars.hpp:187-208). id=17, inlen=11.
        build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb::<F>(input, out, n);
    }
    ```

    Add two new `#[cube] pub fn` below the existing builder functions:

    `build_xc_a_b_gaa_gab_gbb_taua_taub` (id=13, inlen=7):
    - Copy `gaa` from `input[2*size..3*size]`, `gab` from `input[3*size..4*size]`, `gbb` from `input[4*size..5*size]`, `taua` from `input[5*size..6*size]`, `taub` from `input[6*size..7*size]` into `out.*` via `#[unroll] for i in 0..size` slot-copy.
    - Derive: `gnn = gaa + 2*gab + gbb`, `gss = gaa - 2*gab + gbb`, `gns = (gnn - gss)/2` via `ctaylor_add`/`ctaylor_sub`/`ctaylor_scalar_mul`.
    - Derive: `tau = taua + taub` via `ctaylor_add`.
    - EXPLICIT chain: call `build_xc_a_b::<F>(input, out, n)` at the end (replacing C fallthrough per Phase 3 D-11).
    - No `mul_add`, no FMA per CLAUDE.md ACC-06.

    `build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb` (id=17, inlen=11):
    - Per PATTERNS B.1: implement as FREESTANDING (do NOT chain through id=16/id=13).
    - Copy slots: `a` from `input[0..size]`, `b` from `input[1*size..2*size]`, `gaa` from `input[2*size..3*size]`, `gab` from `input[3*size..4*size]`, `gbb` from `input[4*size..5*size]`, `lapa` from `input[5*size..6*size]`, `lapb` from `input[6*size..7*size]`, `taua` from `input[7*size..8*size]`, `taub` from `input[8*size..9*size]`, `jpaa` from `input[9*size..10*size]`, `jpbb` from `input[10*size..11*size]`.
    - Derive same gnn/gss/gns chains as id=13, plus tau = taua + taub.
    - Derive all other chained fields (n, s, n_m13, etc.) via explicit call to `build_xc_a_b::<F>(input, out, n)` at the end.
    - Verify field names against `xcfun-master/src/densvars.hpp:187-208` (`d.jpaa`, `d.jpbb`).
    - No `mul_add`.

    **2. `crates/xcfun-eval/tests/regularize_mgga_invariant.rs`:**
    Follow `regularize_invariant.rs` pattern. Launch `build_densvars` for id=13 and id=17 with a known input array, assert that tau/taua/taub/lapa/lapb/jpaa/jpbb slots are bit-exact copies of the input and that gnn/gns/gss are correctly derived. Feature-gate `#![cfg(feature = "testing")]`.

    **3. Module tree creation:**

    `crates/xcfun-eval/src/functionals/mod.rs` — add `pub mod mgga;` (alongside existing `pub mod gga; pub mod lda;`).

    `crates/xcfun-eval/src/functionals/mgga/mod.rs` — follow gga/mod.rs pattern. Declare `pub mod shared;` plus stub declarations for family modules (commented out — they land in plans 04-01/02/03):
    ```rust
    pub mod shared;
    // Wave 1 (04-01): TPSS family + BR + CSC — modules added in plan 04-01.
    // Wave 2 (04-02): SCAN family — modules added in plan 04-02.
    // Wave 3 (04-03): M0x + BLOCX — modules added in plan 04-03.
    ```

    `crates/xcfun-eval/src/functionals/mgga/shared/mod.rs`:
    ```rust
    pub mod constants;
    pub mod tpss_like;
    pub mod scan_like;
    pub mod m0x_like;
    pub mod br_like;
    pub mod blocx;
    pub mod cs;
    ```

    **4. `mgga/shared/constants.rs`:** collect all scalar constants needed by TPSS, SCAN, M0x, BLOCX, CSC families. Use `pub const NAME: f64 = VALUE;` format matching the C++ constants. Include at minimum:
    - TPSS: `TPSS_KAPPA: f64 = 0.804`, `TPSS_MU: f64 = 0.21951` (from tpssx_eps.hpp)
    - SCAN: `SCAN_KAPPA: f64 = 0.804`, `SCAN_MUK: f64 = 10.0/81.0` (from SCAN_like_eps.hpp)
    - M0x: constants from m0xy_fun.hpp (M05/M06 coefficient arrays stored as `&[f64]` or as const arrays)
    - CSC: `CSC_A: f64`, `CSC_B: f64`, `CSC_C: f64`, `CSC_D: f64`, `CSC_GAMMA: f64` (from cs.cpp)
    Read each source header to extract exact constant values; do not guess.

    **5. `mgga/shared/tpss_like.rs`:** Port `tpssx_eps.hpp` (60 LOC), `tpssc_eps.hpp` (62 LOC), `revtpssx_eps.hpp` (65 LOC), `revtpssc_eps.hpp` (111 LOC) as `#[cube] pub fn` functions. Fused in one module per CONTEXT D-01-A. Follow `pbex.rs` multi-formula pattern. Key exports: `tpss_fx_unif`, `tpss_F_x`, `tpss_eps`, `revtpss_fx`, `revtpss_eps`. No mul_add.

    **6. `mgga/shared/scan_like.rs`:** Port `SCAN_like_eps.hpp` (522 LOC). This is the single largest helper module. Key design decisions per PATTERNS C.4:
    - `get_SCAN_Fx` takes a `#[comptime] idelec: u32` parameter (values 0=SCAN, 1=rSCAN, 2=r++SCAN, 3=r2SCAN, 4=r4SCAN) with `if comptime!(idelec == 0) { ... }` branches. This avoids runtime branching inside the kernel — each SCAN-family functional calls `get_SCAN_Fx` with a different comptime `idelec`.
    - `r2SCAN_C` similarly takes comptime `idelec`.
    - `gcor2` outputs two values: use tuple return `(F, F)` or two `&mut Array<F>` — prefer the `#[cube]` idiomatic approach (two separate output parameters since `#[cube]` doesn't return structs).
    - `get_lsda1` uses out-parameter style → two separate output `&mut Array<F>` parameters.
    - No mul_add anywhere. Strict algebraic identity with C++ source order.
    - Add module-level doc citing `xcfun-master/src/functionals/SCAN_like_eps.hpp`.

    **7. `mgga/shared/m0x_like.rs`:** Port `m0xy_fun.hpp` (262 LOC). Exports: `m0x_fw`, `m0x_chi2`, `m0x_Dsigma`, `m0x_zet`, `m0x_gamma`, `m0x_h`, `m0x_g`, `m06_c_anti`, `m06_c_para`, `m05_c_anti`, `m05_c_para`, `ueg_c_para`, `ueg_c_anti`. Reuses `pw92eps::pw92eps` (already in `gga/shared/pw91_like.rs`) and `pbex::energy_pbe_ab` (already in `gga/shared/pbex.rs`). Follow b97_poly.rs pattern for polynomial Horner evaluation of M05/M06 coefficient arrays.

    **8. `mgga/shared/br_like.rs`:** Port the `polarized(na, gaa, lapa, taua, jpaa)` helper from `brx.cpp:89-101`. This calls `ctaylor_br_inverse` from `xcfun-ad` (Plan 04-00 Task 1). Also include the ctaylor-level `BR(t)` wrapper (`brx.cpp:78-87`) that evaluates `BR_z` on a CTaylor argument using existing `ctaylor_exp`, `ctaylor_mul`, `ctaylor_sub`, `ctaylor_reciprocal`. Import: `use xcfun_ad::ctaylor_br_inverse;`.

    **9. `mgga/shared/blocx.rs`:** Single `#[cube] pub fn blocx_energy` body inline (no BRX dependency confirmed by RESEARCH). Port `xcfun-master/src/functionals/blocx.cpp:18-46`. Uses `pow`, `sqrt`, `log`, `exp` on `d_n`, `p`, `tau_w` ratios via existing ctaylor primitives.

    **10. `mgga/shared/cs.rs`:** Single `#[cube] pub fn csc_energy` body for CSC correlation (10 active LOC from `xcfun-master/src/functionals/cs.cpp:17-27`). Reads `d.a`, `d.b`, `d.n`, `d.taua`, `d.taub`, `d.gnn`, `d.jpaa`, `d.jpbb`, `d.n_m13`. Computes `gamma = 2*(1 - (a²+b²)/n²)` and `curv = a*taua + b*taub - gnn/8 - (jpaa+jpbb)` then the final energy expression via ctaylor ops.

    **11. `validation/src/fixtures.rs` (D-09-A):** Extend `generate_grid` to add a metaGGA stratum at sibling seed `0xc0ffee01`, 1000 points. Each point includes random `tau_a ∈ [0, kF^2 * n_a^(2/3)]` and `tau_b` similarly (positive-definite kinetic energy density), `lap_a` and `lap_b` near zero (Laplacian), `jpaa` and `jpbb` in a physically reasonable range (random in [-0.1, 0.1] a.u.). Append these 1000 points to the grid under a new `GridStratum::MetaGGA` variant (or extend the existing stratum enum). The existing 10k canonical grid at seed `0x1234abcd` is NOT modified. Use `rand_xoshiro::Xoshiro256PlusPlus` with seed `0xc0ffee01_u64`.
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0 (module tree compiles).
    2. `grep -n "comptime!(vars == 13)" crates/xcfun-eval/src/density_vars/build.rs` returns a match.
    3. `grep -n "comptime!(vars == 17)" crates/xcfun-eval/src/density_vars/build.rs` returns a match.
    4. `grep -n "0xc0ffee01" validation/src/fixtures.rs` returns a match.
    5. `cargo test -p xcfun-eval --test regularize_mgga_invariant --features testing` passes.
    6. `grep -n "pub mod mgga" crates/xcfun-eval/src/functionals/mod.rs` returns a match.
    7. `grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/` returns empty (no mul_add in any helper).
    8. `grep -rn "get_SCAN_Fx" crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs` returns a match (function exported).
    9. `grep -rn "idelec" crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs` returns matches (comptime IDELEC dispatch present).
    10. `grep -rn "ctaylor_br_inverse" crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs` returns a match (uses the Wave-0 Task-1 primitive).
    11. `cargo build -p validation --release` exits 0 (fixtures.rs change compiles).
  </acceptance_criteria>
  <done>DensVarsDev arms id=13 and id=17 compiling and passing invariant test. All 7 mgga/shared helpers compile. metaGGA grid stratum compiles. cargo build -p xcfun-eval --release exits 0.</done>
</task>

</tasks>

<verification>
```bash
# Full Wave 0 verification suite:
cargo build -p xcfun-ad --features cpu 2>&1 | tail -5
cargo test -p xcfun-ad --features cpu test_ctaylor_n6 2>&1 | tail -10
cargo test -p xcfun-ad --features cpu golden_br_inverse 2>&1 | tail -10
cargo build -p xcfun-eval --release 2>&1 | tail -5
cargo test -p xcfun-eval --test regularize_mgga_invariant --features testing 2>&1 | tail -10
cargo build -p validation --release 2>&1 | tail -5

# Structural checks:
grep -n "comptime!(vars == 13)" crates/xcfun-eval/src/density_vars/build.rs
grep -n "comptime!(vars == 17)" crates/xcfun-eval/src/density_vars/build.rs
grep -n "0xc0ffee01" validation/src/fixtures.rs
grep -rn "mul_add" crates/xcfun-eval/src/functionals/mgga/
grep -rn "get_SCAN_Fx" crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs
```
</verification>

<success_criteria>
- `cargo test -p xcfun-ad --features cpu golden_br_inverse` exits 0 with 90/90 records at max_relative=1e-12.
- `cargo test -p xcfun-ad --features cpu test_ctaylor_n6` exits 0 (no panic).
- `cargo build -p xcfun-eval --release` exits 0.
- `cargo test -p xcfun-eval --test regularize_mgga_invariant --features testing` exits 0.
- No `mul_add` in any new file under `crates/xcfun-eval/src/functionals/mgga/`.
- `grep -n "comptime!(vars == 13)"` and `grep -n "comptime!(vars == 17)"` both find matches in build.rs.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| xcfun-ad internals | No untrusted data — scaffold only; host-side Newton takes f64 from test fixtures |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-00-01 | Tampering | br_scalar Newton convergence | accept | Newton non-convergence prints to stderr and returns best estimate — matches C++ behaviour. No panic, no abort. No untrusted input path in this plan. |
| T-04-00-02 | Denial of Service | CTaylor<F,6> stack allocation (10 KB per kernel) | accept | 10 KB well within cubecl-cpu kernel stack. Verified in RESEARCH. No heap alloc on the hot path. |
| T-04-00-03 | Information Disclosure | None | accept | No FFI surface touched in this plan; pure internal implementation. |

No new attack surface introduced. Pure-internal implementation plan. Threats inherited from Phase 1 FFI gate.
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-00-SUMMARY.md`
</output>
