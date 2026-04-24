# Phase 03 — GGA Tier + `Mode::Potential` — Pattern Map

**Mapped:** 2026-04-24
**Files analyzed:** ~60 new/modified (10 Wave-0 substrate + 36–40 GGA bodies + 7 DensVars arms + 2 potential kernels + 4 test/tool extensions)
**Analogs found:** 58 / 60 (exact or role+data-flow match in Phase-1/Phase-2 codebase)

> **Flag for planner (A9 from RESEARCH):** `_2ND_TAYLOR` discriminants in `crates/xcfun-core/src/enums.rs:73-76` are **27, 28, 29, 30** (verified by `grep -n "_2ND_TAYLOR" enums.rs`). CONTEXT.md D-10 states 26..29 — **off by one**. Planner MUST use 27..30 when writing the comptime `if`-chain arms. VALIDATION.md §Manual-Only Verifications already flags this.
>
> **Flag for planner (A8 / RESEARCH §BR, §CSC):** BRX/BRC/BRXC (10/11/12) **and CSC (66)** declare `XC_KINETIC | XC_LAPLACIAN | XC_JP` — metaGGA-class deps requiring `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` (Vars=17, inlen=11) **NOT in D-10**. If planner accepts the deferral amendment, Phase 3 ships 36 functionals (not 40). Patterns below cover the 36-functional case; if BR/CSC stay in, their row notes `NEW-STYLE pattern (no LDA analog)`.

---

## File Classification

### Area A — xcfun-ad substrate (Wave 0)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-ad/src/expand/expm1.rs` | expand primitive | transform (scalar → `Array<F>` series) | `crates/xcfun-ad/src/expand/exp.rs` | **exact** |
| `crates/xcfun-ad/src/math.rs` (+`ctaylor_expm1`, +`ctaylor_sqrtx_asinh_sqrtx`) | composed op | transform (`Array<F>` → `Array<F>`) | same file, existing `ctaylor_exp` / `ctaylor_log` | **exact** |
| `crates/xcfun-ad/src/expand/mod.rs` (+`pub mod expm1`) | module decl | n/a | same file, existing `pub mod exp/log/...` | **exact** |
| `crates/xcfun-ad/tests/golden_expand.rs` (+`expm1` arm) | test | fixture-compare | same file, existing `exp_expand` arm | **exact** |
| `crates/xcfun-ad/tests/golden_composed.rs` (+`sqrtx_asinh_sqrtx` arm) | test | fixture-compare | same file, existing `ctaylor_exp` arm | **exact** |

### Area B — xtask (Wave 0)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `xtask/assets/regen_ad_fixtures/driver.cpp` (+`emit_expm1_expand` + `emit_sqrtx_asinh_sqrtx`) | C++ fixture emitter | batch | same file, existing `emit_exp_expand` | **exact** |
| `xtask/src/bin/regen_ad_fixtures.rs` (no struct changes; re-runs driver) | Rust driver | build-time | same file, unchanged call path | **exact** |

### Area C — xcfun-eval DensVarsDev arms (Wave 0, D-10 corrected to 27..30)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-eval/src/density_vars/build.rs` (+7 `build_xc_*` fns + 7 comptime arms) | densvars builder arm | transform (input slice → DensVarsDev) | same file, existing `build_xc_a_b_gaa_gab_gbb` | **exact** |
| `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` (NEW) | test | unit | `crates/xcfun-eval/tests/regularize_invariant.rs` | **exact** |

### Area D — xcfun-eval GGA shared helpers (Wave 0, D-08) — 6 new files

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` | shared helper | transform | `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs` | **role match** |
| `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` | shared helper | transform | `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs` | **role match** |
| `crates/xcfun-eval/src/functionals/gga/shared/pbec_eps.rs` | shared helper | transform | `crates/xcfun-eval/src/functionals/lda/pw92eps.rs` | **exact** |
| `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` | shared helper | transform | `crates/xcfun-eval/src/functionals/lda/pw92eps.rs` | **role match** |
| `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` | shared helper | transform | `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs` | **role match** |
| `crates/xcfun-eval/src/functionals/gga/shared/constants.rs` | const module | n/a (constants) | **NEW-STYLE** (no LDA analog — LDA consts are per-file) | **none** |
| `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` | mod decl | n/a | `crates/xcfun-eval/src/functionals/lda/mod.rs` | **exact** |
| `crates/xcfun-eval/src/functionals/gga/mod.rs` | mod decl | n/a | `crates/xcfun-eval/src/functionals/lda/mod.rs` | **exact** |
| `crates/xcfun-eval/src/functionals/mod.rs` (+`pub mod gga;`) | mod decl | n/a | same file, existing `pub mod lda;` | **exact** |

### Area E — xcfun-eval GGA kernel bodies (Waves 1–3)

One file per FunctionalId. Grouped by family — one representative shown; the rest follow the same pattern.

| Family | Representative File (NEW) | Rest of Family | LDA Analog |
|--------|---------------------------|----------------|-----------|
| PBE (12) | `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` | `pbec.rs`, `revpbex.rs`, `rpbex.rs`, `pbesolx.rs`, `pbeintx.rs`, `pbeintc.rs`, `spbec.rs`, `pbelocc.rs`, `zvpbesolc.rs`, `zvpbeintc.rs`, `vwn_pbec.rs` | `lda/slaterx.rs` (pure algebra) + `lda/pw92c.rs` (correlation w/ pw92eps) |
| Becke (4) | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` | `beckecorrx.rs`, `beckesrx.rs`, `beckecamx.rs` | `lda/slaterx.rs` + `lda/ldaerfx.rs` (erf-bearing) |
| BR (3, **DEFER recommended**) | `crates/xcfun-eval/src/functionals/gga/br/brx.rs` | `brc.rs`, `brxc.rs` | **NEW-STYLE** — no LDA analog (Newton-inverse Taylor needed) |
| LYP (1) | `crates/xcfun-eval/src/functionals/gga/lyp.rs` | — | `lda/pw92c.rs` |
| OPTX (2) | `crates/xcfun-eval/src/functionals/gga/optx/optx.rs` | `optxcorr.rs` | `lda/slaterx.rs` |
| PW86/PW91 (4) | `crates/xcfun-eval/src/functionals/gga/pw91/pw91x.rs` | `pw86x.rs`, `pw91c.rs`, `pw91k.rs` | `lda/slaterx.rs` + `lda/pw92c.rs` + `lda/tw.rs` |
| P86 (2) | `crates/xcfun-eval/src/functionals/gga/p86/p86c.rs` | `p86corrc.rs` | `lda/pz81c.rs` |
| APBE (2) | `crates/xcfun-eval/src/functionals/gga/apbe/apbex.rs` | `apbec.rs` | `lda/slaterx.rs` + `lda/pw92c.rs` |
| B97 (6) | `crates/xcfun-eval/src/functionals/gga/b97/b97x.rs` | `b97c.rs`, `b97_1x.rs`, `b97_1c.rs`, `b97_2x.rs`, `b97_2c.rs` | `lda/slaterx.rs` + `lda/pw92c.rs` |
| KT/BTK (2; CSC **DEFER**) | `crates/xcfun-eval/src/functionals/gga/kt/ktx.rs` | `btk.rs` | `lda/slaterx.rs` + `lda/tfk.rs` |

**Role:** kernel body. **Data flow:** request-response (reads `&DensVarsDev<F>` → writes `&mut Array<F>`).

### Area F — xcfun-eval dispatch + Functional (Waves 1–4)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-eval/src/dispatch.rs` (+36–40 comptime arms, bitmap bump) | dispatcher | request-response | same file, existing 11 arms | **exact** |
| `crates/xcfun-eval/src/functional.rs` (order 3/4 arms + Mode::Potential branch + 40-ID `run_launch` arms) | facade | request-response | same file, existing `launch_and_accumulate` + `run_launch` | **exact** |

### Area G — xcfun-eval Mode::Potential (Wave 4, D-13/D-14)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-eval/src/functionals/mode_potential.rs` (NEW) | kernel body | transform | `crates/xcfun-eval/src/functionals/lda/tw.rs` (shape) + `crates/xcfun-eval/src/functional.rs::run_launch` (host seeding) | **role match** |
| `crates/xcfun-eval/tests/potential_parity.rs` (NEW, implicit) | test | integration | `crates/xcfun-eval/tests/self_tests.rs` | **role match** |

### Area H — xcfun-eval tests (Wave 5)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `crates/xcfun-eval/tests/self_tests.rs` (NO EDITS — auto-enumerates via `FUNCTIONAL_DESCRIPTORS.iter()`) | test | integration | — | **passes automatically once registry populates** |

### Area I — xtask registry regen (per-wave)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `xtask/src/bin/regen_registry.rs` (NO CODE CHANGES, re-invoked per wave) | extractor driver | batch | same file, Phase-2 usage | **exact** |
| `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs` (AUTOGEN — grows by 40) | generated registry | batch | same file, Phase-2 11 LDAs | **exact** |
| `validation/c_stubs.cpp` (AUTOGEN — shrinks by 40) | C++ stubs | build-time | same file, Phase-2 67 stubs | **exact** |

### Area J — validation harness (per-wave)

| New/Modified File | Role | Data Flow | Closest Analog | Match |
|-------------------|------|-----------|----------------|-------|
| `validation/build.rs` (+40 `.file(...)` entries per wave) | build script | build-time | same file, existing 11 LDA file entries | **exact** |
| `validation/src/fixtures.rs` (+`gga_stratified_supplement(seed=0xdeadbeef)`) | fixture generator | batch | same file, existing `generate_grid` + `generate_gradient_stress` | **exact** |
| `validation/src/driver.rs` (+`--mode potential` routing + `--grid supplemental`) | driver | batch | same file, existing Mode::PartialDerivatives loop | **role match** |
| `validation/src/main.rs` (+`--mode` CLI flag + `--grid` flag) | CLI | request-response | same file, existing `--backend/--order/--filter` | **exact** |

---

## Pattern Assignments

### A1. `crates/xcfun-ad/src/expand/expm1.rs` (NEW)

**Analog:** `crates/xcfun-ad/src/expand/exp.rs` (full file — 48 lines).

**Module-doc + imports + `#[cube] fn` signature pattern** (`exp.rs:1-34`):
```rust
//! `expm1_expand` — Taylor series of `exp(x0 + x) - 1` in `x`, around `x = 0`.
//!
//! Port of `xcfun-master/external/upstream/taylor/tmath.hpp:??-??`.
//! # C++ source (tmath.hpp)
//!   template <class T, int Ndeg> static void expm1_expand(T * t, const T & x0) {
//!     T ifac = 1;
//!     t[0] = exp(x0) - 1;              // or expm1(x0) for the near-zero branch
//!     for (int i = 1; i <= Ndeg; i++) {
//!       ifac *= i;
//!       t[i] = exp(x0) / ifac;         // NOTE: derivative is exp, not expm1
//!     }
//!   }
//! # Precondition: none (expm1 is analytic on ℝ).
use cubecl::prelude::*;

#[cube]
pub fn expm1_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // t[0] = expm1(x0)  — stable near x0=0 via expm1; `x0.exp() - F::new(1.0)`
    // loses precision for |x0| < ~1e-8. cubecl 0.10-pre.3: use `(x0.exp() -
    // F::new(1.0))` as the portable path; IF `Float::expm1` lands in a later
    // pre-release, switch. (Fixture-gate will catch the divergence.)
```

**Body pattern (copy cumulative-ifac idiom from `exp.rs:35-48`):**
```rust
    let mut ifac = F::new(1.0);
    t[0] = x0.exp() - F::new(1.0);     // ← KEY DEVIATION from exp_expand
    let exp_x0 = x0.exp();              // cache for the derivative branch
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        ifac *= i_f;
        t[k] = exp_x0 / ifac;           // i-th derivative of expm1 is exp
    }
}
```

**Deviation note (Pitfall P9 + CONTEXT D-05):** `t[0]` uses `x0.exp() - 1` unless cubecl grows `Float::expm1`; callers needing `x0 → 0` precision must stable-bracket at the host side. Document this explicitly in the module header. Fixture-gate at `x0 ∈ {1e-15, 1e-10, 1e-6, 1e-3, 0.1, 1.0, 10.0}` to detect cancellation.

---

### A2. `crates/xcfun-ad/src/math.rs` (EXTEND — 2 new public entry points)

**Analog:** same file, existing `ctaylor_exp` (`math.rs:136-142`).

**`ctaylor_expm1` entry-point pattern** (copy `ctaylor_exp` verbatim, change the expand call):
```rust
#[cube]
pub fn ctaylor_expm1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);
    expm1_expand::<F>(&mut scratch, x[0], n);
    ctaylor_compose::<F>(out, x, &scratch, n);
}
```

**`ctaylor_sqrtx_asinh_sqrtx` entry-point pattern** (NEW-STYLE composed op — no perfect analog; closest is the `ctaylor_powi_positive` private helper at `math.rs:349-365`). Compose `sqrt` → `asinh` → `mul`:
```rust
/// `out = sqrt(x) * asinh(sqrt(x))`. Used by PW91X/PW91K enhancement formula.
/// # Precondition: `x[0] >= 0`.
#[cube]
pub fn ctaylor_sqrtx_asinh_sqrtx<F: Float>(
    x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    let mut sx = Array::<F>::new(size);        // sx = sqrt(x)
    let mut asx = Array::<F>::new(size);       // asx = asinh(sx)
    ctaylor_sqrt::<F>(x, &mut sx, n);
    ctaylor_asinh::<F>(&sx, &mut asx, n);
    ctaylor_mul::<F>(&sx, &asx, out, n);
    // Near-zero stable bracket (D-06): if x[0] ~ 0, the product is order-x^(1/2)
    // and both sub-results stay numerically bounded. Fixture-gate verifies.
}
```

**Import addition:** `use crate::expand::expm1::expm1_expand;` alongside the existing `use crate::expand::...` imports at `math.rs:55-62`.

---

### A3. `crates/xcfun-ad/tests/golden_expand.rs` (EXTEND)

**Analog:** same file, existing `kernel_exp` arm at `golden_expand.rs:44-47`.

**Kernel adapter pattern (copy verbatim, swap op):**
```rust
#[cube(launch_unchecked)]
fn kernel_expm1<F: Float>(scalars: &Array<F>, t: &mut Array<F>, #[comptime] n: u32) {
    expm1::expm1_expand::<F>(t, scalars[0], n);
}
```

**Dispatch match arm** (extend `golden_expand.rs:123-133`):
```rust
match rec.op.as_str() {
    "inv_expand"   => launch_unary!(kernel_inv),
    "exp_expand"   => launch_unary!(kernel_exp),
    "expm1_expand" => launch_unary!(kernel_expm1),   // ← NEW
    // ... rest unchanged
}
```

**Tolerance:** inherit default `REL_TOL = 1e-12` — `expm1` has no polyfill drift. Add to `relaxed_tolerance_for()` if Wave-0 fixture-gate exposes cancellation drift at `x0 → 0`.

---

### A4. `crates/xcfun-ad/tests/golden_composed.rs` (EXTEND)

**Analog:** same file, existing `kernel_exp` adapter at `golden_composed.rs:48-50`.

**Kernel adapter pattern:**
```rust
#[cube(launch_unchecked)]
fn kernel_expm1<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    math::ctaylor_expm1::<F>(x, out, n);
}
#[cube(launch_unchecked)]
fn kernel_sqrtx_asinh_sqrtx<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    math::ctaylor_sqrtx_asinh_sqrtx::<F>(x, out, n);
}
```

**Dispatch extension** (add two arms to the `match rec.op.as_str()` at `golden_composed.rs:141-165`):
```rust
"ctaylor_expm1"              => launch_unary!(kernel_expm1),
"ctaylor_sqrtx_asinh_sqrtx"  => launch_unary!(kernel_sqrtx_asinh_sqrtx),
```

---

### B1. `xtask/assets/regen_ad_fixtures/driver.cpp` (EXTEND)

**Analog:** same file, `emit_exp_expand` at `driver.cpp:58-65`.

**Emitter template (copy verbatim, swap C++ fn name):**
```cpp
template <int N>
static void emit_expm1_expand(double x0) {
    double t[N + 1];
    expm1_expand<double, N>(t, x0);          // declared in tmath.hpp
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("expm1_expand", N, inputs, coeffs);
}

template <int N>
static void emit_sqrtx_asinh_sqrtx(double x0) {
    // Compose the ctaylor result at N variables: wrap x0 into a 1-var ctaylor,
    // call sqrt/asinh/mul per ctaylor_math.hpp.
    ctaylor<double, N> x(x0); x.c[1] = 1.0;  // seed VAR0 = 1 for derivative
    ctaylor<double, N> sx = sqrt(x);
    ctaylor<double, N> asx = asinh(sx);
    ctaylor<double, N> y = sx * asx;
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(y.c, y.c + (1 << N));
    emit_record("ctaylor_sqrtx_asinh_sqrtx", N, inputs, coeffs);
}
```

**Call-site additions in `main()`** (follow the existing "500 records at N=0..=4" pattern):
```cpp
std::mt19937_64 rng(0x1234abcd);
std::uniform_real_distribution<double> ud(1e-10, 10.0);  // strict >0 for sqrt
for (int i = 0; i < 500; i++) {
    double x0 = ud(rng);
    emit_expm1_expand<0>(x0); emit_expm1_expand<1>(x0); /*...=4*/
    emit_sqrtx_asinh_sqrtx<0>(x0); /*...=4*/
}
```

**Deviation note:** RESEARCH §Example 2 calls for `500 × (N+1 orders) = 2500` per op. Planner decides final grid spread (include near-zero stable-bracket points `x0 ∈ [1e-15, 1e-3]`).

---

### C1. `crates/xcfun-eval/src/density_vars/build.rs` (EXTEND — 7 new arms)

**Analog:** same file, existing `build_xc_a_b_gaa_gab_gbb` at `build.rs:191-231` (explicit-chain pattern per CORE-05 + P5).

**Comptime if-chain extension** (add to the `build_densvars` dispatcher at `build.rs:77-85`):
```rust
if comptime!(vars == 2) {            // XC_A_B (Phase 2, existing)
    build_xc_a_b::<F>(input, out, n);
} else if comptime!(vars == 6) {     // XC_A_B_GAA_GAB_GBB (Phase 2)
    build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
}
// --- Phase 3 additions (D-10 — corrected discriminants per A9 / enums.rs:73-76) ---
else if comptime!(vars == 4)  { build_xc_a_gaa::<F>(input, out, n); }
else if comptime!(vars == 5)  { build_xc_n_gnn::<F>(input, out, n); }
else if comptime!(vars == 7)  { build_xc_n_s_gnn_gns_gss::<F>(input, out, n); }
else if comptime!(vars == 27) { build_xc_a_2nd_taylor::<F>(input, out, n); }
else if comptime!(vars == 28) { build_xc_a_b_2nd_taylor::<F>(input, out, n); }
else if comptime!(vars == 29) { build_xc_n_2nd_taylor::<F>(input, out, n); }
else if comptime!(vars == 30) { build_xc_n_s_2nd_taylor::<F>(input, out, n); }
```

**Per-arm helper pattern** — copy `build_xc_a_b_gaa_gab_gbb` (`build.rs:191-231`):
1. Pre-seeded CTaylor input layout: `input[i*size..(i+1)*size]` per input slot (D-12 layout).
2. Copy coefficients via `#[unroll] for i in 0..size { out.<field>[i] = input[j*size + i]; }`.
3. Derived fields via `ctaylor_add/sub/scalar_mul` (no `mul_add` — ACC-06).
4. Explicit chain to the lower-variant builder where the C++ `densvars.hpp` switch falls through (e.g., `build_xc_n_s_gnn_gns_gss` chains to `build_xc_n_s`; `build_xc_a_b_2nd_taylor` chains to `build_xc_a_b` after populating the 18-coefficient Taylor block).

**Example skeleton for `build_xc_a_b_2nd_taylor`** (inlen=20, per-spin 2nd Taylor):
```rust
#[cube]
pub fn build_xc_a_b_2nd_taylor<F: Float>(
    input: &Array<F>, out: &mut DensVarsDev<F>, #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // Input layout (D-12 + XCFunctional.cpp:437-447):
    //   slots 0..10  = a's 2nd-order Taylor: {a, ∂a/∂x, ∂a/∂y, ∂a/∂z,
    //                   ∂²a/∂x², ∂²a/∂x∂y, ∂²a/∂x∂z, ∂²a/∂y², ∂²a/∂y∂z, ∂²a/∂z²}
    //   slots 10..20 = b's 2nd-order Taylor (same layout)
    // For derivative-field population (d.lapa = 0.5·(a_xx + a_yy + a_zz), etc.)
    // see XCFunctional.cpp:437-447. Populate a, b, gaa, gab, gbb, lapa, lapb
    // from the 20 slots, THEN explicit-chain to build_xc_a_b.
    /* ... unrolled copy loops + ctaylor_add chains to derive gaa/gab/gbb/lapa/lapb ... */
    build_xc_a_b::<F>(input, out, n);   // explicit chain (CORE-05 + P5)
}
```

**Deviation note:** The 2ND_TAYLOR arms differ from the base `build_xc_a_b` in that they populate additional fields (lapa, lapb, gaa, gab, gbb) from the pre-seeded 2nd-order Taylor coefficients. CONTEXT specifics line 308 explicitly states `regularize` touches ONLY `a[CNST]` (+ `b[CNST]` for spin-resolved), preserving the Taylor-seeded derivatives.

---

### C2. `crates/xcfun-eval/tests/regularize_2nd_taylor.rs` (NEW)

**Analog:** `crates/xcfun-eval/tests/regularize_invariant.rs:1-60` — identical test scaffold.

**Pattern (copy the scaffold verbatim, extend with 2ND_TAYLOR-specific assertions):**
```rust
#![cfg(feature = "testing")]
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_eval::density_vars::regularize::regularize;
use xcfun_eval::for_tests::cpu_client;

#[cube(launch_unchecked)]
fn regularize_kernel<F: Float>(x: &mut Array<F>, #[comptime] n: u32) {
    regularize::<F>(x, n);
}

/// Verify regularize clamps ONLY c[CNST] even when higher-order slots carry
/// 2ND_TAYLOR-derived derivative coefficients. (D-11 + CORE-06 invariant.)
#[test]
fn regularize_preserves_2nd_taylor_coefficients() {
    // Input: CTaylor<F,3> with 8 coefficients. c[CNST]=1e-15 (below clamp),
    // and c[VAR0]=0.3, c[VAR1]=0.5, c[VAR2]=0.7, c[VAR0|VAR1]=1.1 (Laplacian
    // contribution), … — the 2nd-order Taylor slots MUST be preserved.
    let input = [1e-15_f64, 0.3, 0.5, 1.1, 0.7, 0.0, 0.0, 0.0];
    let output = run_regularize(&input);
    assert!(output[0] > 9.99e-15 && output[0] < 1.01e-14, "CNST clamped");
    assert_eq!(output[1], 0.3);
    assert_eq!(output[2], 0.5);
    assert_eq!(output[3], 1.1);
    // ... etc for remaining slots.
}
```

---

### D1. `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` (NEW)

**Analog:** `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs:1-80` (module header + const table + `#[cube] fn` helpers).

**Module-doc pattern** (copy from `vwn_eps.rs`, swap references):
```rust
//! Shared PBE-family enhancement helpers. 1:1 port of
//! `xcfun-master/src/functionals/pbex.hpp:??-??`.
//!
//! # Source
//! - `xcfun-master/src/functionals/pbex.hpp:19-70`
//!
//! # enhancement(R, rho, grad) formula (pbex.hpp:??)
//!   mu     = 0.066725 * π² / 3
//!   t1     = 1 + (mu / R) · S²(rho, grad)
//!   enh    = 1 + R - R / t1
//! # Preconditions: rho > 0 (post-regularize), grad >= 0.
use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul, ctaylor_sub, ctaylor_zero};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_reciprocal, ctaylor_pow};
use crate::density_vars::DensVarsDev;

// PBE constants (pbex.hpp:20-23). f64 precision per SP-2 (F::new takes f32).
const MU_PBE_F64:         f64 = 0.2195149727645171_f64;   // 0.066725·π²/3
const R_PBE_F64:          f64 = 0.804_f64;                 // kappa
const R_REVPBE_F64:       f64 = 1.245_f64;
// ...

/// enhancement(R, rho, grad) — port of pbex.hpp. R is a comptime constant
/// cast to F inside the kernel for 1e-12 parity.
#[cube]
pub fn enhancement<F: Float>(
    r: F,
    rho: &Array<F>,     // ρ (post-regularize)
    grad2: &Array<F>,   // |∇ρ|²
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // S2 = S²(rho, grad) — reuse pw91_like::S2 helper
    let mut s2 = Array::<F>::new(size);
    crate::functionals::gga::shared::pw91_like::s2::<F>(rho, grad2, &mut s2, n);
    // mu_over_R = MU_PBE / R — compile-time scalar, but R is runtime-F → scalar_mul
    let mu_over_r_scalar = F::cast_from(MU_PBE_F64) / r;
    let mut t0 = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&s2, mu_over_r_scalar, &mut t0, n);
    // ... 1 + t0 via add_const; reciprocal; subtract; etc.
}
```

**Deviation note:** Unlike `vwn_eps` which has one `vwn_f_para/ferro/inter` per spin-channel, `pbex::enhancement` takes the kappa `R` as a scalar `F` argument so REVPBEX (R=1.245) and PBEX (R=0.804) can share the kernel. Scale-factor choice: `scalar_mul` with a precomputed `MU/R` avoids a `ctaylor_reciprocal` on a scalar.

---

### D2. `crates/xcfun-eval/src/functionals/gga/shared/pbec_eps.rs` (NEW)

**Analog:** `crates/xcfun-eval/src/functionals/lda/pw92eps.rs:1-351` (the closest 1:1 structural match — both port a `hpp` helper cluster to a single `.rs` module with multiple internal `#[cube] fn`s).

**Pattern:** copy the 3-helper layout from `pw92eps.rs` (`eopt`, `omega_zeta`, `pow4`, then public `pw92_eps`). For PBEC:
```rust
/// A_expm1_inner — port of pbec_eps.hpp:??.
/// Uses D-05 `ctaylor_expm1` critically: A = β_gamma / expm1(-eps / (γ·u3))
#[cube]
fn a_expm1_inner<F: Float>(/* args */, out: &mut Array<F>, #[comptime] n: u32) {
    // 1:1 port of pbec_eps.hpp lines X-Y; each C++ intermediate becomes a
    // `let mut <name> = Array::<F>::new(size);` followed by the op.
    // Use `ctaylor_expm1` from Wave 0 D-05.
}

/// H(d2, eps, u3) — port of pbec_eps.hpp:??.
#[cube]
fn h<F: Float>(/* args */, out: &mut Array<F>, #[comptime] n: u32) { /* ... */ }

/// phi(zeta) — port of pbec_eps.hpp:??.  Uses ctaylor_pow with 4/3 exponent.
#[cube]
fn phi<F: Float>(zeta: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // Same pattern as pw92eps::omega_zeta at pw92eps.rs:199-234
}
```

**Critical port rule (Known Hazard §PBEC β/γ):** preserve operation order **around the `expm1`** — `β_gamma / expm1(-eps / (γ·u3))` must be implemented as `ctaylor_expm1` on the inner argument, then `ctaylor_reciprocal`, then `scalar_mul` by `β_gamma`. Do NOT algebraically simplify to `β_gamma / (exp(...) - 1)` — that loses the x→0 stable-bracket.

---

### D3. `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` (NEW)

**Analog:** `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs` (mixed helper cluster with constants + `#[cube] fn` surface).

**Skeleton:**
```rust
//! Shared PW91/PW91K helpers — port of `pw9xx.hpp:17-??`.
//! Ports: chi2, S2, prefactor (for PW91X), pw91k_prefactor (for PW91K), pw91xk_enhancement.
use cubecl::prelude::*;
/* imports identical to vwn_eps.rs:34-39 */

// Constants (RESEARCH §Example 2, A7):
const S2_PREFACTOR_F64: f64 = /* 1/(4·(3π²)^(2/3)) = 0.16162045967399868 */;

#[cube] pub fn chi2<F: Float>(rho: &Array<F>, grad2: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) { /* ... */ }
#[cube] pub fn s2<F: Float>(rho: &Array<F>,   grad2: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) { /* ... */ }
#[cube] pub fn prefactor<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) { /* ... */ }
#[cube] pub fn pw91k_prefactor<F: Float>(rho: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) { /* ... */ }
#[cube] pub fn pw91xk_enhancement<F: Float>(/* params */, out: &mut Array<F>, #[comptime] n: u32) { /* uses D-06 sqrtx_asinh_sqrtx */ }
```

---

### D4. `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` (NEW)

**Analog:** `crates/xcfun-eval/src/functionals/lda/pw92eps.rs` (multi-`#[cube] fn` helper module) + `crates/xcfun-eval/src/functionals/gga/shared/constants.rs` (coefficient tables per D4b).

**Pattern:**
```rust
//! B97-family polynomial enhancement helpers. Ports b97x.hpp/b97c.hpp/b97xc.hpp.
//! Coefficient tables live in `shared/constants.rs` (per-functional `[f64; 3]`).
#[cube]
pub fn ux_ab<F: Float>(gamma: F, s2: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    // u_x(γ, s²) = γ·s² / (1 + γ·s²)
    /* ctaylor_scalar_mul + ctaylor_add + ctaylor_reciprocal + ctaylor_mul chain */
}

#[cube]
pub fn enhancement<F: Float>(
    gamma: F,                      // γ scalar (B97_GAMMA_X etc.)
    c0: F, c1: F, c2: F,            // per-functional coef row (from constants.rs)
    s2: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // enh = c0 + c1·u + c2·u²
    /* use ctaylor_powi_2 + ctaylor_scalar_mul + ctaylor_add chain */
}
```

---

### D5. `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` (NEW)

**Analog:** `lda/pw92eps.rs` — same "multi-fn helper module" pattern. Small body (~80 LOC).

---

### D6. `crates/xcfun-eval/src/functionals/gga/shared/constants.rs` (NEW — **NEW-STYLE pattern**)

**No perfect analog** — Phase 2 inlines constants per-file. Proposed pattern (following CLAUDE.md SP-2 / Phase 2 `NEG_C_SLATER_F64` convention):
```rust
//! Per-family scalar constants for GGA functionals. One `const` per magic
//! number extracted from `xcfun-master/src/functionals/*.hpp`, stored at
//! f64 precision for 1e-12 parity (SP-2 + slaterx.rs:26 precedent).
//!
//! NO `F::new(f32)` literals — callers cast via `F::cast_from(CONST)` at
//! kernel-time. This mirrors the pattern in pw92eps.rs:45-64 for legacy-
//! constant tables.

// PBE family (pbex.hpp:20-27)
pub const MU_PBE_F64:             f64 = 0.2195149727645171_f64;
pub const R_PBE_F64:              f64 = 0.804_f64;
pub const R_REVPBE_F64:           f64 = 1.245_f64;
pub const R_RPBE_F64:             f64 = 0.804_f64;
pub const KAPPA_PBESOL_F64:       f64 = 0.046_f64;

// B97 family (b97xc.hpp / b97-1xc.hpp / b97-2xc.hpp — verbatim from RESEARCH §B97)
pub const B97_GAMMA_X_F64:        f64 = 0.004_f64;
pub const B97_GAMMA_C_PAR_F64:    f64 = 0.2_f64;
pub const B97_GAMMA_C_ANTIPAR_F64:f64 = 0.006_f64;
pub const B97_X_COEF:    [f64; 3] = [0.8094,   0.5073,   0.7481];
pub const B97_1X_COEF:   [f64; 3] = [0.789518, 0.573805, 0.660975];
pub const B97_2X_COEF:   [f64; 3] = [0.827642, 0.047840, 1.76125];
// ... (full set from RESEARCH lines 825-833 verbatim)

// KT family (ktx.cpp)
pub const KTX_A_F64:   f64 = 0.006_f64;

// OPTX family (optx.cpp)
pub const OPTX_GAMMA_F64: f64 = 0.006_f64;
pub const OPTX_COEF:      [f64; 2] = [1.05151_f64, 1.43169_f64];

// BTK (btk.cpp:18) — kinetic-GGA
pub const BTK_FUDGE_F64: f64 = 1e-24_f64;  // Avoid division-by-zero at |∇ρ|²=0.
```

**Rationale:** consolidating constants avoids the Phase-2 pattern where every `slaterx.rs:26`-style `NEG_C_SLATER_F64` lives in its own file. For 40 functionals with ~200 constants total, the per-file scatter would hurt maintainability. Module-level visibility (`pub const`) keeps the re-export cost zero.

---

### E (representative). `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` (NEW)

**Analog:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs` (full 45-line file) — the simplest "one formula, no helper" kernel pattern. Every GGA kernel with pure algebra uses this template.

**Full pattern** (copy slaterx.rs verbatim, substitute the formula + shared-helper call):
```rust
//! PBE exchange functional. **GGA-01 (PBEX).**
//!
//! # Source
//! - `xcfun-master/src/functionals/pbex.cpp:20-50` (FUNCTIONAL macro + test data)
//! - `xcfun-master/src/functionals/pbex.hpp:??-??` (energy_pbe_ab)
//!
//! # Formula
//!   E_x = energy_pbe_ab(R=0.804, a, gaa) + energy_pbe_ab(R=0.804, b, gbb)
//! where
//!   energy_pbe_ab(R, rho, grad) = -c_slater · rho^(4/3) · enhancement(R, rho, grad)
//!
//! # Preconditions (per XC_A_B_GAA_GAB_GBB Vars arm)
//! - d.a, d.b, d.gaa, d.gbb, d.a_43, d.b_43 populated
//! - a > 0, b > 0 (post-regularize)

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

use crate::density_vars::DensVarsDev;
use crate::functionals::gga::shared::pbex as pbex_shared;
use crate::functionals::gga::shared::constants::{NEG_C_SLATER_F64, R_PBE_F64};

#[cube]
pub fn pbex_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // enhancement_a = enhancement(R_PBE, d.a, d.gaa)
    let mut enh_a = Array::<F>::new(size);
    pbex_shared::enhancement::<F>(F::cast_from(R_PBE_F64), &d.a, &d.gaa, &mut enh_a, n);

    // lsda_a = -c_slater · d.a_43
    let mut lsda_a = Array::<F>::new(size);
    ctaylor_scalar_mul::<F>(&d.a_43, F::cast_from(NEG_C_SLATER_F64), &mut lsda_a, n);

    // e_a = lsda_a · enhancement_a
    let mut e_a = Array::<F>::new(size);
    ctaylor_mul::<F>(&lsda_a, &enh_a, &mut e_a, n);

    // (mirror for b-spin; then out = e_a + e_b)
    /* ... */
    ctaylor_add::<F>(&e_a, &e_b, out, n);
}
```

**Every other GGA body file follows this template**, differing only by:
1. Which shared helper(s) it imports (`pbex_shared`, `pw91_like`, `pbec_eps`, `b97_poly`, `optx`).
2. Which DensVarsDev fields it reads (`d.a`, `d.gaa`, `d.zeta`, `d.r_s`, etc.).
3. The post-enhancement algebra (usually `ctaylor_add` of two spin contributions, sometimes a correlation product).
4. Which constants it pulls from `shared/constants.rs`.

**Specific families with notable deviations:**
- **Becke (beckex.rs / beckecorrx.rs):** use `ctaylor_sqrtx_asinh_sqrtx` (D-06) instead of `enhancement`. Analog is still slaterx.rs for the skeleton.
- **BECKESRX / BECKECAMX:** add `ctaylor_erf` + `ctaylor_expm1` + parameter read `d.parameters[XC_RANGESEP_MU]` — RESEARCH §Open Question 1 flags this as a **planner decision point** (parameter-reading API design). Closest analog: `lda/ldaerfx.rs`.
- **LYPC:** pure algebra with `ctaylor_exp` + `ctaylor_pow` + `ctaylor_reciprocal`. Slightly longer (~60 lines) than slaterx — analog: pw92c.rs structurally.
- **PW91C:** RESEARCH flags as **THE LONGEST BODY** (`pw91c.cpp:39-87`, ~120 lines). Analog: pw92eps.rs for the multi-intermediate chain pattern.
- **P86C / P86CORRC:** reuse `pz81eps` from `lda/pz81c.rs` — **planner must verify `pz81eps` is re-exported** (RESEARCH §P86 bullet).

---

### F1. `crates/xcfun-eval/src/dispatch.rs` (EXTEND)

**Analog:** same file, existing 11-arm if-chain at `dispatch.rs:40-73`.

**Pattern (2 representative new arms verbatim — rest follow same shape):**
```rust
} else if comptime!(id == 1) {
    // XC_PW86X (GGA-06)
    crate::functionals::gga::pw91::pw86x::pw86x_kernel::<F>(d, out, n);
} else if comptime!(id == 5) {
    // XC_PBEX (GGA-01)
    crate::functionals::gga::pbe::pbex::pbex_kernel::<F>(d, out, n);
}
// ... 34–38 more arms, one per GGA FunctionalId.
```

**`supports()` bitmap bump** (extend the `matches!` macro at `dispatch.rs:84-89`):
```rust
pub fn supports(id: FunctionalId) -> bool {
    matches!(
        id as u32,
        // Phase 2 LDAs (unchanged):
        0 | 2 | 3 | 13 | 14 | 15 | 24 | 25 | 28 | 55 | 59
        // Phase 3 GGAs (D-04 — 36 or 40 new IDs):
        | 1 | 5 | 6 | 7 | 8 | 9 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 26
        | 27 | 56 | 57 | 58 | 60 | 61 | 62 | 63 | 64 | 65 | 67 | 68 | 69 | 71
        | 72 | 73 | 74 | 76 | 77
        // + 10, 11, 12, 66 if BR/CSC NOT deferred — RESEARCH §BR / §CSC
    )
}
```

---

### F2. `crates/xcfun-eval/src/functional.rs` (EXTEND — order 3/4 + Mode::Potential)

**Analog:** same file. The existing `launch_and_accumulate` at `functional.rs:171-282` already handles orders 0/1/2 for inlen=2.

**Order 3/4 arm pattern** (extend `match order { 0 => ..., 1 => ..., 2 => ..., }` at `functional.rs:180-281` with new arms). The seeding loop is derivable from `XCFunctional.cpp:562-588`:
```rust
3 => {
    // inlen = 2 or 5: triple-nested (i,j,k) with i ≤ j ≤ k.
    // Output slots: (inlen + C(inlen,2) + C(inlen,3))-th new entry is
    //   ∂³/∂x_i ∂x_j ∂x_k, read from out[VAR0|VAR1|VAR2] (bit-flag = 7).
    // See RESEARCH §"PartialDerivatives Order 3-4 Output Layout" + XCFunctional.cpp:562-588.
    /* i, j, k triple loop; pack flat[i*sz + 1] = 1; flat[j*sz + 2] = 1;
       flat[k*sz + 4] = 1; run launch at N=3 (sz=8); read out[7]. */
}
4 => { /* quadruple loop, N=4, sz=16, read out[VAR0|VAR1|VAR2|VAR3 = 15] */ }
```

**Mode::Potential branch** (add to `eval` at `functional.rs:89-99` — replaces the current `Mode::Potential => InvalidMode` rejection):
```rust
Mode::Potential => {
    // D-13: check metaGGA rejection + Vars compatibility
    let deps = self.weights.iter().map(|(id, _)| descriptor_depends(*id)).reduce(|a,b| a|b)
        .unwrap_or(Dependency::DENSITY);
    if deps.contains(Dependency::LAPLACIAN | Dependency::KINETIC) {
        return Err(XcError::InvalidMode { mode: self.mode, depends: deps });
    }
    // D-13 vars gate for GGAs: require 2ND_TAYLOR variants
    if deps.contains(Dependency::GRADIENT)
        && !matches!(self.vars, Vars::A_2ND_TAYLOR | Vars::A_B_2ND_TAYLOR
                              | Vars::N_2ND_TAYLOR | Vars::N_S_2ND_TAYLOR) {
        return Err(XcError::InvalidVars { vars: self.vars });
    }
    // Dispatch to potential_lda_kernel (N=1) or potential_gga_kernel (N=2).
    launch_potential(self, input, output)?;
    return Ok(());
}
```

**`run_launch` extension:** the existing 33-arm `match (id_u32, n)` at `functional.rs:341-427` must grow by ~35-IDs × 5-N = 175 new arms. **RESEARCH §Open Question 4 flags this as a compile-time risk** — planner may split `run_launch` into per-family submatches (PBE vs Becke vs …) to cap per-match size.

---

### G1. `crates/xcfun-eval/src/functionals/mode_potential.rs` (NEW — NEW-STYLE)

**No exact analog** — this is the first file ported directly from `XCFunctional.cpp` (not from `*.cpp` + `*.hpp`). Closest shape analogs:
1. **Kernel body shape:** `crates/xcfun-eval/src/functionals/lda/tw.rs` — simple multi-step pipeline.
2. **Host-side seeding / launch orchestration:** `crates/xcfun-eval/src/functional.rs::launch_and_accumulate` — the derivative-seeding convention.

**Proposed skeleton:**
```rust
//! Mode::Potential kernels. Line-for-line port of
//! `xcfun-master/src/XCFunctional.cpp:637-790` (LDA + GGA divergence paths).
//!
//! # LDA path (XCFunctional.cpp:642-670)
//!   Seed VAR0=1 on the density slot; launch kernel at N=1; read out[CNST]
//!   (energy) + out[VAR0] (∂E/∂ρ).
//!
//! # GGA path (XCFunctional.cpp:672-790)
//!   For each spatial direction d ∈ {x, y, z}:
//!     seed VAR0=1 on density, VAR1=1 on ∇_dρ; launch N=2;
//!     accumulate output[1+spin] -= out[VAR1]  (the ∇·(dE/dg) term)
//!   Plus the direct potential term from out[VAR0] of one launch.
//!
//! # Output layout (D-15)
//!   nspin=1 → [energy, pot_alpha]
//!   nspin=2 → [energy, pot_alpha, pot_beta]

use cubecl::prelude::*;
use crate::density_vars::DensVarsDev;
use crate::dispatch::dispatch_kernel;

/// LDA-path potential kernel. Runs at N=1.
#[cube]
pub fn potential_lda_kernel<F: Float>(
    d: &DensVarsDev<F>,
    #[comptime] id: u32,
    out: &mut Array<F>,
    #[comptime] _n: u32,   // always 1 for LDA path
) {
    let mut eval_out = Array::<F>::new(2);   // N=1 → size = 2
    dispatch_kernel::<F>(id, d, &mut eval_out, 1);
    out[0] = eval_out[0];                     // energy
    out[1] = eval_out[1];                     // ∂E/∂ρ = pot
}

/// GGA-path potential kernel. Runs at N=2 (one of three directional passes).
/// Line-for-line port of XCFunctional.cpp:678-760.
#[cube]
pub fn potential_gga_kernel<F: Float>(
    d: &DensVarsDev<F>,
    #[comptime] id: u32,
    out: &mut Array<F>,   // size = 4 (N=2)
    #[comptime] _n: u32,   // always 2 for GGA path
) {
    dispatch_kernel::<F>(id, d, out, 2);
    // Caller (host-side launch_potential) accumulates:
    //   final_output[0] = out[CNST]      (energy, from any one pass)
    //   final_output[1] += -out[VAR1]     (∇·(dE/dg) contribution, 3× per dir)
    //   final_output[1] += out[VAR0]/3    (direct-density term / 3 passes)
}
```

**Host-side orchestration** (in `functional.rs::launch_potential`, new fn):
```rust
// Mirror XCFunctional.cpp:637-790:
//   For LDA: single N=1 launch with VAR0 seeded on density.
//   For GGA: 3 N=2 launches (one per spatial direction x/y/z).
//
// Sign convention (XCFunctional.cpp:666-669):
//   output[1+spin] = out[VAR0]  (direct term)   +  accumulated -out[VAR1·direction]
```

**CRITICAL (CONTEXT Specifics line 305):** `output[0] = energy`; `output[j+1] = pot_α (or _β)` — the `+1` offset reserves slot 0 for energy. Do NOT simplify to `output[j] = ...`.

---

### I. `xtask/src/bin/regen_registry.rs` (NO CODE CHANGES — re-invoked per wave)

**Analog:** Phase-2 invocation. Pattern is unchanged: `cargo run -p xtask --bin regen-registry` after adding new GGA `.cpp` files to `validation/build.rs`. Generates updated `FUNCTIONAL_DESCRIPTORS.rs` + shrunken `c_stubs.cpp`.

**Planner check (RESEARCH §Open Question 3):** if `#ifdef XCFUN_REF_PBEX_MU` conditional test data gets mis-extracted (Phase-2 precedent: extractor ignores #ifdef), hand-patch until xtask is fixed.

---

### J1. `validation/build.rs` (EXTEND per-wave)

**Analog:** same file, existing 11-file LDA loop at `validation/build.rs:87-103`.

**Per-wave extension pattern (copy the `for f in &[...]` block):**
```rust
// Wave 1 additions — PBE + Becke + LYP (17 files):
for f in &[
    "pbex", "pbec", "revpbex", "rpbex", "pbesolx", "pbeintx",
    "pbeintc", "spbec", "pbelocc", "zvpbesolc", "zvpbeint",
    "vwn_pbec",    // PBE (12)
    "beckex", "beckecorrx", "beckesrx", "beckecamx",  // Becke (4)
    "lypc",    // LYP (1)
] {
    build.file(format!("{}/src/functionals/{}.cpp", xcfun_root, f));
}
// Wave 2, Wave 3 extensions follow the same pattern.
```

**`cargo:rerun-if-changed`** — no change, the `{xcfun_root}/src` line at `build.rs:108` already covers new files.

---

### J2. `validation/src/fixtures.rs` (EXTEND — supplemental 400-pt grid)

**Analog:** same file, existing `generate_gradient_stress` (line 80+ of fixtures.rs; see `generate_grid` dispatcher at `fixtures.rs:72-80`).

**Pattern (copy `generate_gradient_stress` — it's the closest existing stratum generator):**
```rust
/// 400-point supplemental GGA grid — extends `generate_grid()` with GGA-specific
/// stress points missed by the Phase-2 base grid. Uses its own seed to stay
/// deterministic + independent of the existing grid (avoids reindexing tier-2
/// record IDs).
///
/// Stratification (from RESEARCH §"GGA Stratified Supplement"):
///   - 100 points:  uniform random `s = |∇ρ| / (2(3π²)^(1/3) ρ^(4/3)) ∈ [0, 5]`
///                  (enhancement-factor sweep)
///   - 100 points:  small-density + large-gradient (tests regularize + PBE/LYP
///                  stability at density floor)
///   - 100 points:  high polarisation (ζ ∈ [0.9, 0.999]) + GGA gradient
///   - 100 points:  r_s ∈ [1e-2, 1e6] sweep at fixed gradient
pub fn gga_stratified_supplement() -> Vec<GridPoint> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(0xdeadbeef);
    let mut out = Vec::with_capacity(400);
    out.extend(enhancement_sweep(&mut rng, 100));
    out.extend(low_density_high_gradient(&mut rng, 100));
    out.extend(high_polarisation(&mut rng, 100));
    out.extend(rs_sweep(&mut rng, 100));
    out
}
```

---

### J3. `validation/src/main.rs` (EXTEND — `--mode potential` + `--grid supplemental`)

**Analog:** same file, existing `parse_arg` helper at `main.rs:13-21` + CLI flag reads at `main.rs:26-31`.

**Pattern:** extend the flag-parsing block, route per-flag to either `Mode::PartialDerivatives` (default) or `Mode::Potential`; grid choice routes to either `generate_grid` or a concat of `generate_grid + gga_stratified_supplement`.

```rust
let mode = parse_arg(&args, "--mode").unwrap_or("partial_derivatives");
let grid_name = parse_arg(&args, "--grid").unwrap_or("default");
let grid = match grid_name {
    "default"       => validation::fixtures::generate_grid(),
    "supplemental"  => {
        let mut g = validation::fixtures::generate_grid();
        g.extend(validation::fixtures::gga_stratified_supplement());
        g
    }
    _ => anyhow::bail!("--grid must be 'default' or 'supplemental'"),
};
```

---

## Shared Patterns

### S1. `#[cube] fn <name>_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32)`
**Source:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs:29-44`.
**Apply to:** every GGA kernel file in Area E (36–40 files).
Universal signature. Body layout: zero-or-more `let mut <tmp> = Array::<F>::new(comptime!((1_u32 << n) as usize));` allocations, then linear sequence of `ctaylor_*` calls with operation order preserved from C++.

### S2. f64-precision constants via `F::cast_from(<NAME>_F64)`
**Source:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs:22-26` + `pw92eps.rs:34-64`.
**Apply to:** every file in Area D (shared helpers) and Area E (kernel bodies).
```rust
const <NAME>_F64: f64 = <exact-decimal>_f64;
// ...
F::cast_from(<NAME>_F64)   // NOT F::new(<val>_f32)
```
Rationale: `F::new` takes `f32` — widening to f64 introduces ~1e-7 rel-error that breaks 1e-12 parity.

### S3. No `mul_add` / `fma` (ACC-06 + `check-no-mul-add` CI gate)
**Source:** CLAUDE.md constraints + `crates/xcfun-eval/src/density_vars/build.rs:215-227` (ex: `t1 = 2*gab; t2 = gaa + t1; gnn = t2 + gbb`).
**Apply to:** every kernel body. Three-operand operations must decompose into two `ctaylor_add/mul/scalar_mul` calls. `xtask check-no-mul-add` must be extended to scan `crates/xcfun-eval/src/functionals/gga/**/*.rs`.

### S4. Explicit helper-function chains (no C-fallthrough)
**Source:** `crates/xcfun-eval/src/density_vars/build.rs:229-231` (CORE-05 + Pitfall P5).
**Apply to:** every `_2ND_TAYLOR` variant arm + `XC_N_S_GNN_GNS_GSS`.
```rust
// Explicit chain (replaces C-style fallthrough at densvars.hpp:65-72):
build_xc_a_b::<F>(input, out, n);
```

### S5. Module-doc header convention
**Source:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs:1-13` + `lda/tw.rs:1-14`.
**Apply to:** every new kernel file. Structure:
```
//! <FullName> <Class> <tier>. **<REQ-ID>.**
//! # Source: <xcfun-master/...cpp:line-range>
//! # Formula: <LaTeX or plaintext>
//! # Preconditions: <densvars fields populated; clamp invariants>
```

### S6. Fixture-gate before family wave-port
**Source:** Phase 1 D-03 + Phase 2 D-19 (escalation policy).
**Apply to:** every family wave entry. Workflow: (a) regen fixtures → (b) `cargo test -p xcfun-ad golden_expand::test_<new_op>` must PASS → (c) then begin kernel ports. Drift > 1e-12 triggers `PLANNING INCONCLUSIVE`.

### S7. Pre-seeded CTaylor input layout (D-12)
**Source:** `crates/xcfun-eval/src/density_vars/build.rs:108-155` (build_xc_a_b docstring lines 107-127).
**Apply to:** every new DensVarsDev arm. Input is a flat `(inlen × (1 << n))` block; `input[slot*size + coef_idx]` — host packs derivative-seed markers before launch.

### S8. Dispatch comptime if-chain (D-04)
**Source:** `crates/xcfun-eval/src/dispatch.rs:40-73`.
**Apply to:** every new dispatch arm in `dispatch.rs`. Pattern:
```rust
} else if comptime!(id == <N>) {
    // XC_<NAME>  (GGA-<family>)
    crate::functionals::gga::<family>::<fn>::<fn>_kernel::<F>(d, out, n);
}
```

### S9. Parameter-carrying API for BECKESRX/BECKECAMX (**PLANNER DECISION — RESEARCH Open Q1**)
**No existing analog.** Planner must choose option (a) [add `parameters: [f64; 4]` to `Functional`, forward through launch] vs (b) [kernel args]. RESEARCH recommends (a); closest existing pattern is `Functional { weights }` at `functional.rs:65-74`. Document in Wave 0 spec.

---

## No Analog Found (NEW-STYLE)

| File | Reason |
|------|--------|
| `crates/xcfun-eval/src/functionals/gga/shared/constants.rs` | Phase 2 inlines constants per-file; consolidation is new for Phase 3. Pattern proposed in D6 above. |
| `crates/xcfun-eval/src/functionals/mode_potential.rs` | First file ported from `XCFunctional.cpp` rather than `functionals/*.cpp`. Closest analogs: `lda/tw.rs` (kernel shape) + `functional.rs::launch_and_accumulate` (seeding orchestration). Pattern proposed in G1. |
| `crates/xcfun-eval/src/functionals/gga/br/brx.rs` etc. (if NOT deferred) | Newton-inverse BR_taylor algorithm requires a brand-new xcfun-ad module (`taylor.hpp` dense polynomial, not bit-flag multilinear). **RESEARCH strongly recommends deferring to Phase 4.** |
| `Functional::parameters: [f64; 4]` field | RESEARCH Open Q1. No Phase-2 analog; planner-owned design. |

---

## Metadata

**Analog search scope:**
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-ad/**/*.rs`
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-eval/**/*.rs`
- `/home/chemtech/workspace/xcfun_rs/crates/xcfun-core/src/enums.rs`
- `/home/chemtech/workspace/xcfun_rs/validation/**/*.rs`
- `/home/chemtech/workspace/xcfun_rs/xtask/src/**/*.rs`
- `/home/chemtech/workspace/xcfun_rs/xtask/assets/regen_ad_fixtures/driver.cpp`

**Files read (all read-only):** 22 existing files across crates/ and validation/ — primary analogs quoted with line ranges above.

**Key patterns identified:**
1. **Universal kernel signature** `#[cube] fn <name>_kernel<F: Float>(d, out, n)` — 100% GGA reuse from LDA.
2. **f64-precision constant convention** `F::cast_from(<NAME>_F64)` — mandatory for 1e-12 parity.
3. **Explicit helper-function chains** instead of C-fallthrough — every 2ND_TAYLOR arm + the 4 new derived-var arms.
4. **Shared-helper module pattern** — Phase 2's `lda/vwn_eps.rs` + `lda/pw92eps.rs` directly transplantable to `gga/shared/*.rs`.
5. **Dispatch + `supports()` bitmap** — 40 new arms mechanically mirror the 11 LDA arms.
6. **Fixture-gate before body ports** — D-05/D-06 primitives must pass golden_expand / golden_composed at 1e-12 before Wave-1 begins.

**Pattern extraction date:** 2026-04-24.
**Planner can now consume this document to write PLAN.md files for Wave 0–5.**
