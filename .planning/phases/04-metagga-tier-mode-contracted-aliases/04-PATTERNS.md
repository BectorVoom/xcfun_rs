# Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases — Pattern Map

**Mapped:** 2026-04-25
**Files analyzed:** 56 new + 13 modified (per CONTEXT D-01..D-14, RESEARCH §"Wave 0 Gaps" / §"Recommended Wave Breakdown")
**Analogs found:** 53 / 56 new (3 originate the pattern: Mode::Contracted host dispatch, alias engine, parameter table)

Phase 4 is overwhelmingly an **extension phase** — every new file extends a Phase-1/2/3 pattern that has been GREEN at strict 1e-12. Where an analog exists in `crates/xcfun-eval/src/functionals/{lda,gga}/...` or `crates/xcfun-ad/src/{expand,math,for_tests}.rs`, the planner is instructed to copy the analog's structure verbatim and substitute kernel/helper names per the per-file pattern assignments below.

---

## File Classification

### New files (56)

#### A. xcfun-ad substrate (Wave 0 — D-02)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-ad/src/expand/br_inverse.rs` | expand-helper (Newton scalar + linear-method polynomial sweep) | host-side scalar + transform | `crates/xcfun-ad/src/expand/sqrt.rs` (recurrence-style writer into `&mut Array<F>`) + `crates/xcfun-ad/src/math.rs` `ctaylor_sqrtx_asinh_sqrtx` (compose pattern) | role-match (Newton on CNST slot is novel; the linear-method polynomial sweep is exactly `*_expand` shape) |
| `crates/xcfun-ad/tests/golden_br_inverse.rs` | fixture-gate test | request-response (load fixture / launch / assert) | `crates/xcfun-ad/tests/golden_mul.rs` (load bincode, launch via `cpu_client`, assert at 1e-13) | exact |

#### B. DensVarsDev arms (Wave 0 — D-03)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| 2 new build functions in existing `crates/xcfun-eval/src/density_vars/build.rs` (id=13 `build_xc_a_b_gaa_gab_gbb_taua_taub`; id=17 `build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb`) | density-var-builder (slot copy + chain) | transform | `build_xc_a_b_gaa_gab_gbb` (lines 222-262 of `build.rs`) — explicit chain replacing C-fallthrough | exact |
| `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` | invariant test (slot-preservation + non-mutation of TAU/JP fields) | request-response | `crates/xcfun-eval/tests/regularize_invariant.rs` (Phase 2; launch_unchecked wrapper + four-element fixture) | exact |

#### C. metaGGA shared helpers (Wave 0 — D-01-A)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-eval/src/functionals/mgga/mod.rs` | module-root | declarative | `crates/xcfun-eval/src/functionals/gga/mod.rs` (Wave-by-Wave family declarations + comments) | exact |
| `crates/xcfun-eval/src/functionals/mgga/shared/mod.rs` | helper-index | declarative | `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` (6-line `pub mod` list) | exact |
| `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs` | shared-helper (tpssx_eps/tpssc_eps/revtpssx_eps/revtpssc_eps fused) | transform | `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` (multi-formula module with `#[cube] fn` per C++ helper) | exact |
| `crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs` | shared-helper (522-line `SCAN_like_eps.hpp` port, IDELEC dispatch) | transform | `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` (multi-formula `#[cube] fn` exports) | role-match (no existing helper of comparable size; IDELEC `#[comptime] u32` branch is novel) |
| `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs` | shared-helper (m0xy_fun.hpp, M05+M06 substrate) | transform | `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` (polynomial Horner pattern) + `pbex.rs` | role-match |
| `crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs` | shared-helper (BR Newton scalar driver + ctaylor adapter `BR(t)`) | transform | `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` (caller-driven helper) | role-match |
| `crates/xcfun-eval/src/functionals/mgga/shared/blocx.rs` | shared-helper (single inline body, no chain) | transform | `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` (single-body inline helper) | exact |
| `crates/xcfun-eval/src/functionals/mgga/shared/cs.rs` | shared-helper (CSC inline body) | transform | `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` | exact |

#### D. metaGGA functional kernels (Waves 1-3 — D-01-A; 32 files)

All 32 follow the same template (kernel signature + comptime `n` + read `DensVarsDev` fields):

| Wave / Files | Role | Closest Analog | Match Quality |
|--------------|------|----------------|---------------|
| **Wave 1 (9):** `mgga/{tpssx,tpssc,revtpssx,revtpssc,tpsslocc,brx,brc,brxc,csc}.rs` (CSC may live as `gga/cs.rs` per D-01-A note) | per-functional kernel | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` (helper-driven kernel with alpha/beta composition) for spin-decomposed; `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` for single-call kernel | exact |
| **Wave 2 (10):** `mgga/{scanx,scanc,rscanx,rscanc,rppscanx,rppscanc,r2scanx,r2scanc,r4scanx,r4scanc}.rs` | per-functional kernel | `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` + IDELEC `#[comptime] u32` parameter (novel) | role-match |
| **Wave 3 (13):** `mgga/{m05x,m05c,m05x2x,m05x2c,m06x,m06c,m06lx,m06lc,m06hfx,m06hfc,m06x2x,m06x2c,blocx}.rs` | per-functional kernel | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` (parameter-array-driven helper composition) | exact |

#### E. Registry tables (Wave 4 — D-04, D-05)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-core/src/parameter_id.rs` | enum + helpers | static | `crates/xcfun-core/src/functional_id.rs` (`#[repr(u32)]` enum, 78 variants, `from_name` lookup) | exact |
| `crates/xcfun-core/src/registry/generated/parameters.rs` | static slice (4 rows) | static | `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` (31-row static slice with metadata) | exact |
| Populate `crates/xcfun-core/src/registry/generated/ALIASES.rs` (replace empty stub with 46 rows) | static slice | static | `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` (31-row literal pattern) — currently the stub at `ALIASES.rs:14` is `pub static ALIASES: &[Alias] = &[]` | exact |

#### F. Mode::Contracted (Wave 5 — D-06)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-eval/src/functionals/contracted.rs` (new module) | mode-dispatcher (host-side input pack + cubecl launch + output unpack) | request-response | `crates/xcfun-eval/src/functional.rs` `launch_potential` (Phase 3 Mode::Potential host-side launcher composed over per-functional `dispatch_kernel`) — same shape but for orders 0..=6 | role-match (Mode::Contracted DOEVAL is a new dispatch shape; unpack `output[i] = out.get(i)` is novel) |
| `crates/xcfun-eval/tests/contracted_cross_mode.rs` | parity test (PartialDerivatives vs Contracted at orders 0..=4) | request-response | `crates/xcfun-eval/tests/potential_parity.rs` (Phase 3 cross-mode parity) | role-match |

### Modified files (13)

| File | Change | Pattern source |
|------|--------|----------------|
| `crates/xcfun-ad/src/expand/mod.rs` | add `pub mod br_inverse;` | already lists 9 expand modules (lines 34-46) |
| `crates/xcfun-ad/src/lib.rs` | re-export `ctaylor_br_inverse` | `pub use index::{CNST, VAR0..};` pattern at line 22 |
| `crates/xcfun-eval/src/density_vars/build.rs` | add 2 mandatory new arms (id=13, id=17); add 2 if-arms in `build_densvars` | `build_xc_a_b_gaa_gab_gbb` arm at lines 86-89 + `build_xc_a_b_gaa_gab_gbb` body at 222-262 |
| `crates/xcfun-eval/src/dispatch.rs` | add 36 new comptime arms (32 metaGGA + 4 carryovers); extend `supports()` bitmap from 46 → 82 | existing 46-arm chain at lines 70-208 + `supports()` matches at 220-233 |
| `crates/xcfun-eval/src/functional.rs` | (a) replace `Functional::parameters: [f64; 4]` with `settings: [f64; 82]` matching C++ layout (R1/R7 in research); (b) implement `Functional::set/get` with alias resolution recursion (D-04); (c) wire `Mode::Contracted` host-side dispatch in `eval` (D-06); (d) extend `output_length` Mode::Contracted arm (D-06-B) | `eval` mode-match at lines 113-203; `eval_setup` at 273-315; `output_length` at 231-255; `launch_potential` at 332-... (mode-dispatch pattern) |
| `crates/xcfun-core/src/enums.rs` | add `ParameterId` enum (4 variants, discriminants 78..=81) | `Vars` enum at lines 25-77 (`#[allow(non_camel_case_types)]` + `#[repr(u32)]`) |
| `crates/xcfun-eval/src/functionals/mod.rs` | add `pub mod mgga;` | already declares `pub mod gga; pub mod lda;` |
| `xtask/src/bin/regen_registry.rs` | extend extractor to populate aliases.rs (46) + parameters.rs (4) | existing functional/vars extraction at lines 36-80+ |
| `xtask/src/bin/regen_ad_fixtures.rs` | extend with BR-inverse fixture generation (30 z-points × N∈{2,3,4}) | follows existing `gen_*_fixtures` patterns |
| `validation/build.rs` | append 32 new `.cpp` source files per wave to `cc::Build::file` | existing `for f in &["pbex","pbec",...]` loop at lines 105-156 |
| `validation/c_stubs.cpp` | auto-shrink (`xtask regen-registry` removes stub rows for landed metaGGAs) | already at 32 stubs after Phase 3; Phase 4 shrinks to ~3 (BLOCX with no test_in stays as a stub-but-implemented row) |
| `validation/src/fixtures.rs` | add metaGGA stratum at sibling seed `0xc0ffee01`, 1000 points (D-09-A) | existing `generate_grid` + 4-stratum pattern at lines 71-110 |

---

## Pattern Assignments

### A.1 — `crates/xcfun-ad/src/expand/br_inverse.rs` (NEW; D-02)

**Closest analog:** `crates/xcfun-ad/src/expand/sqrt.rs` (recurrence-style writer into `&mut Array<F>`).

**Imports pattern** (sqrt.rs:36):

```rust
use cubecl::prelude::*;
```

**Module-doc + C++ source citation pattern** (sqrt.rs:1-35):

```rust
//! `br_inverse_expand` — Brent–Kung linear-method polynomial inversion of
//! `BR_z(x) = (x-2)/x · exp(2x/3)` for the BR family.
//!
//! Port of `xcfun-master/src/functionals/brx.cpp:50-71` (`BR_taylor`).
//!
//! # C++ source (brx.cpp:50-71)
//!
//! ```cpp
//! template <typename T, int Ndeg>
//! taylor<T, 1, Ndeg> BR_taylor(const T & z0) {
//!   static_assert(Ndeg >= 3, ...);
//!   ...
//! }
//! ```
//!
//! # Precondition
//!
//! `Ndeg >= 3` (C++ `static_assert`).
//!
//! # Cubecl 0.10-pre.3 deviation from D-11
//!
//! ... (boilerplate-copy from sqrt.rs:28-34)
```

**Core pattern — host-side scalar Newton (NEW shape; not a `#[cube]`)** — see RESEARCH §"BR Newton-Inverse Primitive" lines 644-678 for the verbatim port. Place in `crates/xcfun-ad/src/expand/br_inverse.rs` as a `pub fn br_scalar(z: f64) -> f64` ABOVE the `#[cube]` writer:

```rust
// Port of brx.cpp:21-23 + 25-27 + 29-48.
#[inline] fn br_z(x: f64) -> f64 { (x - 2.0) / x * (2.0 / 3.0 * x).exp() }
#[inline] fn nr_step(x: f64, z: f64) -> f64 {
    (x * (3.0 * x * ((-2.0 / 3.0 * x).exp() * z - 1.0) + 6.0))
        / (x * (2.0 * x - 4.0) + 6.0)
}
pub fn br_scalar(z: f64) -> f64 {
    let mut x0 = if z < -1e4 { -2.0 / z }
        else if z < -2.0 { ((9.0 * z * z + 6.0 * z + 49.0).sqrt() + 3.0 * z + 1.0) / 4.0 }
        else if z < 1.0 { 2.0 * (z * (-4.0_f64 / 3.0).exp() + 1.0) }
        else { 1.5 * z.ln() + 3.75 / (1.5 + z.ln()) };
    for _ in 0..20 {
        let xold = x0;
        x0 += nr_step(x0, z);
        if (xold - x0).abs() < 1e-15 * (1.0 + x0) { return x0; }
    }
    eprintln!("BR: Not converged for z = {:e}", z);
    x0
}
```

**Core pattern — `#[cube] fn` linear-method polynomial sweep**: writes into `&mut Array<F>` (matches `sqrt_expand` shape at sqrt.rs:42-62):

```rust
#[cube]
pub fn br_inverse_expand<F: Float>(t: &mut Array<F>, z0: F, #[comptime] n: u32) {
    // brx.cpp:55: t[0] = BR(z0)  — host-side scalar; threaded via launcher.
    // (NOTE: br_scalar lifts to host through the `for_tests::cpu_client`
    //  single-launch wrapper, NOT inside the #[cube] body.)

    // brx.cpp:56-59: t[1] = 1; f = BR_z(t); t[1] = 1 / f[1];
    // brx.cpp:62-67: for (int i = 2; i <= Ndeg; i++) { f = BR_z(t); t[i] = -f[i] * t[1]; }
    //
    // Implementation: explicit-let-binding per Phase-3 SP-2 + ACC-06 (no mul_add).
    // Each iteration evaluates f = BR_z(t) using ctaylor_mul/sub/exp/reciprocal
    // primitives.
    ...
}
```

**Caller pattern** in `crates/xcfun-ad/src/math.rs` (companion to sqrt → ctaylor_sqrt) — add `ctaylor_br_inverse` per the math.rs:50-98 `ctaylor_reciprocal` template:

```rust
// crates/xcfun-ad/src/math.rs (extend)
#[cube]
pub fn ctaylor_br_inverse<F: Float>(z: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let scratch_len = comptime!((n + 1) as usize);
    let mut scratch = Array::<F>::new(scratch_len);
    br_inverse_expand::<F>(&mut scratch, z[0], n);   // mirrors inv_expand call
    ctaylor_compose::<F>(out, z, &scratch, n);       // mirrors reciprocal compose
}
```

**Differences planner must specify:**
- Host-side scalar `br_scalar` runs on slot 0 BEFORE the `#[cube]` body — mirrors the Phase 1 D-04 design (`for_tests::cpu_client` pattern).
- Recurrence depth at `Ndeg = max(N, 3)` per `brx.cpp:80` (`(Nvar >= 3) ? Nvar : 3`) — explicit comptime branch on N.
- 20-iteration Newton cap and `1e-15 * (1 + x0)` rel-tolerance match C++ exactly.

**Cross-references:** CONTEXT D-02, D-02-A; RESEARCH §"BR Newton-Inverse Primitive" (lines 638-791).

---

### A.2 — `crates/xcfun-ad/tests/golden_br_inverse.rs` (NEW)

**Closest analog:** `crates/xcfun-ad/tests/golden_mul.rs` (lines 1-100).

**Module-doc + feature-gate** (golden_mul.rs:1-21):

```rust
//! Golden-fixture test for `ctaylor_br_inverse` on cubecl-cpu.
//!
//! Loads `tests/fixtures/br_inverse.bincode` (generated by
//! `cargo run -p xtask --bin regen-ad-fixtures`) and runs every record
//! through the cubecl `ctaylor_br_inverse` kernel.
//!
//! Strict 1e-12 vs C++ `BR_taylor` for n_var ∈ {2, 3, 4}.

#![cfg(feature = "testing")]
```

**Fixture-record schema** (copy verbatim from golden_mul.rs:30-36):

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FixtureRecord {
    op: String,
    n_var: u8,
    inputs: Vec<f64>,   // single z-input + scratch slots
    coeffs: Vec<f64>,
}
```

**Kernel adapter pattern** (golden_mul.rs:41-49):

```rust
#[cube(launch_unchecked)]
fn kernel_br_inverse<F: Float>(
    z: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_br_inverse::<F>(z, out, n);
}
```

**Differences planner must specify:**
- 30 z-points × N∈{2,3,4} = 90 records (per CONTEXT D-02).
- Strict 1e-12 (NOT 1e-13 — Newton convergence drift permitted; CONTEXT D-11 sets the floor).
- Fixture binary path: `crates/xcfun-ad/tests/fixtures/br_inverse.bincode`.

**Cross-references:** CONTEXT D-02; RESEARCH §"Tier Architecture" Tier-1 row.

---

### B.1 — Two new build functions in `crates/xcfun-eval/src/density_vars/build.rs` (D-03)

**Closest analog:** `build_xc_a_b_gaa_gab_gbb` body at `build.rs:222-262` — exact pattern for the explicit-chain id=17 → id=13 case.

**Pattern excerpt (build.rs:222-262, the chain pattern that id=13 and id=17 must replicate):**

```rust
#[cube]
pub fn build_xc_a_b_gaa_gab_gbb<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);

    // Copy pre-seeded coefficients of `gaa` from input[2*size..3*size] into out.gaa.
    #[unroll]
    for i in 0..size {
        out.gaa[i] = input[2 * size + i];
    }
    // ... gab, gbb similarly ...

    // gnn = gaa + 2*gab + gbb   (left-to-right, no mul_add per ACC-06)
    let mut t1 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    let mut t2 = Array::<F>::new(comptime!((1_u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&out.gab, F::cast_from(2.0_f64), &mut t1, n);
    ctaylor_add::<F>(&out.gaa, &t1, &mut t2, n);
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gnn, n);

    // ... gss, gns ...

    // EXPLICIT chain to XC_A_B (replaces C fallthrough at densvars.hpp:65-72).
    build_xc_a_b::<F>(input, out, n);
}
```

**Dispatch arm pattern (build.rs:86-89):**

```rust
} else if comptime!(vars == 6) {
    // XC_A_B_GAA_GAB_GBB (densvars.hpp:58-72). Pitfall PHASE2-D fix.
    build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
}
```

**Differences planner must specify for new arms:**

**id=13 `build_xc_a_b_gaa_gab_gbb_taua_taub` — port of `densvars.hpp:54-72`:**
- inlen=7; reads `input[5..6]` for `taua`/`taub`, then chains to `build_xc_a_b_gaa_gab_gbb` (NOT to id=17).
- After taua/taub copy: `tau = taua + taub` via `ctaylor_add::<F>(&out.taua, &out.taub, &mut out.tau, n)`.

**id=17 `build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb` — port of `densvars.hpp:187-208`:**
- inlen=11; per RESEARCH (lines 921: "id=17 reads d[9], d[10] then chains into id=16"). Per RESEARCH recommendation: implement as **free-standing** (do NOT chain through id=16/id=13 — the savings are minor and algorithmic-identity rule favours mirroring C++ source structure verbatim).
- Reads `input[9..10]` for `jpaa`/`jpbb`; reads `input[5..8]` for `lapa`/`lapb`/`taua`/`taub`; reads `input[2..4]` for `gaa`/`gab`/`gbb`; reads `input[0..1]` for `a`/`b`. All via the same `for i in 0..size { out.X[i] = input[k * size + i]; }` slot-copy pattern.

**New if-arms in `build_densvars` dispatcher (build.rs:83-115 chain):**

```rust
} else if comptime!(vars == 13) {
    build_xc_a_b_gaa_gab_gbb_taua_taub::<F>(input, out, n);
} else if comptime!(vars == 17) {
    build_xc_a_b_gaa_gab_gbb_lapa_lapb_taua_taub_jpaa_jpbb::<F>(input, out, n);
}
```

**Cross-references:** CONTEXT D-03, D-03-A, D-03-B; RESEARCH §"DensVarsDev Audit" (lines 796-921). Pitfall P5 (no fallthrough).

---

### B.2 — `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/tests/regularize_invariant.rs` (lines 1-50 — Phase 2 test).

**Pattern excerpt (regularize_invariant.rs:9-44):**

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

fn run_regularize(input: &[f64; 4]) -> [f64; 4] {
    let client = cpu_client();
    let x_h = client.create_from_slice(f64::as_bytes(input));
    let read_h = x_h.clone();

    unsafe {
        regularize_kernel::launch_unchecked::<f64, CpuRuntime>(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_3d(1, 1, 1),
            ArrayArg::from_raw_parts(x_h, 4),
            2_u32,
        );
    }
    // ... read + return
}
```

**Differences planner must specify:**
- Wrap `build_densvars` (NOT `regularize`) — exercise the id=13 + id=17 arms via `launch_unchecked`.
- Assert that `tau`, `taua`, `taub`, `lapa`, `lapb`, `jpaa`, `jpbb` slot copy is bit-exact and that the chain populates ALL gradient/density fields.

**Cross-references:** CONTEXT D-03; Pitfall P5.

---

### C.1 — `crates/xcfun-eval/src/functionals/mgga/mod.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/mod.rs` (lines 1-34 — module-root).

**Pattern excerpt (gga/mod.rs:1-34) — copy verbatim shape, substitute MGGA waves:**

```rust
//! MetaGGA (Meta-Generalised-Gradient-Approximation) functional bodies.
//!
//! Phase 4 ships **32 functional IDs** per D-01 (28 metaGGA + 4 carryovers
//! BRX/BRC/BRXC + CSC). LB94 (id=66) deferred to Phase 5 per D-13.
//!
//! # Layout
//!
//! - `shared::` — cross-family helpers (tpss_like, scan_like, m0x_like,
//!   br_like, blocx, cs). Populated Wave 0.
//! - Family modules land in plans 04-01 (TPSS+BR+CSC), 04-02 (SCAN), 04-03 (M0x+BLOCX).

pub mod shared;

// Wave 1 (04-01): TPSS family (5) + BR family (3) + CSC (1).
pub mod tpssx;
pub mod tpssc;
// ... etc.

// Wave 2 (04-02): SCAN family (10).
pub mod scanx;
// ... etc.

// Wave 3 (04-03): M0x family (12) + BLOCX (1).
pub mod m05x;
// ... etc.
```

---

### C.2 — `crates/xcfun-eval/src/functionals/mgga/shared/mod.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/mod.rs` (6 lines):

```rust
pub mod constants;
pub mod pbex;
pub mod pbec_eps;
pub mod pw91_like;
pub mod b97_poly;
pub mod optx;
```

**Phase 4 contents (per CONTEXT D-01-A Wave 0):**

```rust
pub mod constants;     // metaGGA scalar constants (TPSS, SCAN, M0x, BLOCX, CSC)
pub mod tpss_like;     // tpssx_eps + tpssc_eps + revtpssx_eps + revtpssc_eps fused
pub mod scan_like;     // SCAN_like_eps.hpp port (522 lines, IDELEC dispatch)
pub mod m0x_like;      // m0xy_fun.hpp port (262 lines, M05 + M06 substrate)
pub mod br_like;       // BR polarized() helper + ctaylor adapter BR(t)
pub mod blocx;         // BLOCX single-body inline helper
pub mod cs;            // CSC single-body inline helper
```

---

### C.3 — `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` (multi-formula module with `#[cube] fn` per C++ helper).

**Imports + module-doc pattern (pbex.rs:1-33):**

```rust
//! TPSS exchange + correlation enhancement helpers — port of
//! `xcfun-master/src/functionals/{tpssx_eps.hpp, tpssc_eps.hpp,
//! revtpssx_eps.hpp, revtpssc_eps.hpp}`.
//!
//! # Sources
//! - `xcfun-master/src/functionals/tpssx_eps.hpp:1-60`  — `F_x`, `fx_unif`
//! - `xcfun-master/src/functionals/tpssc_eps.hpp:1-62`  — `tpssc_eps`
//! - `xcfun-master/src/functionals/revtpssx_eps.hpp:1-65` — revTPSS exchange
//! - `xcfun-master/src/functionals/revtpssc_eps.hpp:1-111` — revTPSS correlation

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_reciprocal, ctaylor_sqrt};

use super::constants::{TPSS_KAPPA_F64, TPSS_C_F64, /* ... */};
```

**Per-helper `#[cube] fn` pattern (pbex.rs:57-...):**

```rust
#[cube]
pub fn F_x<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // ... port of tpssx_eps.hpp:F_x ...
}

#[cube]
pub fn fx_unif<F: Float>(
    rho: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // ... port of tpssx_eps.hpp:fx_unif ...
}
```

**Differences planner must specify:**
- 4 helper modules fused per CONTEXT D-01-A Wave 0; planner may split (see CONTEXT "Claude's Discretion" — Helper-module granularity).
- Each `#[cube] fn` must avoid `mul_add`/FMA per CLAUDE.md ACC-06.

---

### C.4 — `crates/xcfun-eval/src/functionals/mgga/shared/scan_like.rs` (NEW; 522-line port)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs` (multi-formula module).

**IDELEC dispatch pattern (NOVEL — no analog):**

```rust
/// SCAN-family enhancement factor `F_x`. Branches on `IDELEC` selecting the
/// regularisation variant: 0 = SCAN, 1 = rSCAN, 2 = r++SCAN, 3 = r2SCAN, 4 = r4SCAN.
/// Port of `xcfun-master/src/functionals/SCAN_like_eps.hpp:386-461`.
#[cube]
pub fn get_SCAN_Fx<F: Float>(
    rho: &Array<F>,
    grad2: &Array<F>,
    tau: &Array<F>,
    out: &mut Array<F>,
    #[comptime] idelec: u32,    // <-- comptime IDELEC selector
    #[comptime] n: u32,
) {
    if comptime!(idelec == 0) {
        // SCAN path
    } else if comptime!(idelec == 1) {
        // rSCAN path
    } else if comptime!(idelec == 2) {
        // r++SCAN path
    } else if comptime!(idelec == 3) {
        // r2SCAN path (full GE correction)
    } else if comptime!(idelec == 4) {
        // r4SCAN path (4th-order GE — HIGH-RISK polynomial precision; ACC-06 explicit-let-binding)
    }
}
```

**Out-parameter translation pattern** (RESEARCH §"SCAN family" — `gcor2(P[6], rs, sqrtrs, GG, GGRS)` writes both `GG` and `GGRS` via reference; in cubecl this is `&mut Array<F>` × 2):

```rust
#[cube]
pub fn gcor2<F: Float>(
    p: &Array<F>,             // P[6] coefficients passed as Array
    rs: &Array<F>,
    sqrtrs: &Array<F>,
    gg: &mut Array<F>,        // out 1
    ggrs: &mut Array<F>,      // out 2
    #[comptime] n: u32,
) {
    // ... port of SCAN_like_eps.hpp:500-519 ...
}
```

**Differences planner must specify:**
- 6+ exported `#[cube] fn`: `get_SCAN_Fx`, `r2SCAN_C`, `scan_ec0`, `scan_ec1`, `lda_0`, `gcor2`, `get_lsda1`, `ufunc`.
- R4SCAN's 4th-order GE polynomial: explicit-let-binding (Rule-1) to suppress compiler reordering — flagged HIGH-RISK in RESEARCH R2.

**Cross-references:** CONTEXT D-01-A Wave 2; RESEARCH §"SCAN family" (lines 191-211); Pitfall P9; Risk R2.

---

### C.5 — `crates/xcfun-eval/src/functionals/mgga/shared/m0x_like.rs` (NEW; 262-line port)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs` (polynomial Horner pattern) + `pbex.rs` (multi-helper module).

**Differences planner must specify:**
- Exports: `zet`, `gamma`, `h`, `fw`, `chi2`, `Dsigma`, `g`, `m06_c_anti`, `m06_c_para`, `m05_c_anti`, `m05_c_para`, `ueg_c_para`, `ueg_c_anti` (13 functions per RESEARCH §"M05 family").
- `fw(param_a[12], rho, tau)` — 12-coefficient polynomial; port descending Horner from `specmath.hpp:24-33` (Pitfall P11).
- Reuses `pw92eps::pw92eps` (already in `crates/xcfun-eval/src/functionals/lda/pw92eps.rs`) and `pw9xx::chi2` (`gga/shared/pw91_like.rs::chi2`) — `use` statements at top.

**Cross-references:** CONTEXT D-01-A Wave 3; RESEARCH §"M05 family" + §"M06 family"; Risk R1.

---

### C.6 — `crates/xcfun-eval/src/functionals/mgga/shared/br_like.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/pbex.rs` for caller-driven helpers + `crates/xcfun-ad/src/math.rs` `ctaylor_br_inverse` for the underlying primitive.

**Pattern: `polarized` helper port of `brx.cpp:89-101` (5-arg BR-family helper):**

```rust
#[cube]
pub fn polarized<F: Float>(
    na: &Array<F>,        // d.a
    gaa: &Array<F>,       // d.gaa
    lapa: &Array<F>,      // d.lapa
    taua: &Array<F>,      // d.taua
    jpaa: &Array<F>,      // d.jpaa
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    // ... compose ctaylor_br_inverse + arithmetic primitives ...
    // The `BR(t)` adapter (brx.cpp:78-87) lifts to ctaylor_br_inverse + composition.
}
```

**Cross-references:** CONTEXT D-01-A Wave 1; RESEARCH §"BR family carryover" (lines 254-263); Risk R3.

---

### C.7 / C.8 — `mgga/shared/blocx.rs` + `mgga/shared/cs.rs` (NEW; single-body inline helpers)

**Closest analog:** `crates/xcfun-eval/src/functionals/gga/shared/optx.rs` (single-body inline helper).

**Differences:** BLOCX is **independent of BRX** — RESEARCH §"BLOCX" verifies CONTEXT D-01-A's "BLOCX composes BRX" is FALSE. Port `blocx.cpp:18-46` directly using `pow`, `sqrt`, `log`, `exp` over a TPSS-shaped enhancement structure with `tauw / d_tau` ratio. No Newton, no `BR(...)` call. Risk R5 already neutralised.

CSC is the simplest carryover — single inline expression `-a * gamma * (...)` per RESEARCH §"CSC carryover" lines 268-271.

---

### D — Per-functional kernels (32 files; Waves 1–3)

**Closest analog by family-shape:**

| Functional family | Closest analog | Reason |
|-------------------|----------------|--------|
| TPSS family (5) | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` (lines 26-95) | Spin-decomposed exchange; `becke_alpha` helper called twice for α / β; final `ctaylor_add` |
| SCAN family (10) | `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` (lines 19-38) | Single-call kernel reading `d.a/b/gaa/gbb/...`; helper-driven body |
| M05 / M06 family (12) | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` | Helper-driven (m0xy_fun.hpp helpers) with parameter-array dispatch |
| BLOCX (1) | `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs` | Single inline body, no chain |
| BR family (3) | `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs` (helper-driven) | `polarized` helper called for spin α / β; `BRC`/`BRXC` add correlation terms |
| CSC (1) | `crates/xcfun-eval/src/functionals/lda/slaterx.rs` (lines 28-44) | Single-line body; trivially port |

**Universal kernel template (port of `pbex.rs:19-38`):**

```rust
//! XC_<NAME> — <description>. MGGA-XX.
//!
//! # Source
//! - `xcfun-master/src/functionals/<name>.cpp:<L0>-<L1>`
//!
//! # Formula (port of `<name>.cpp:<L0>-<L1>`):
//! ```cpp
//! <verbatim C++ body>
//! ```

use cubecl::prelude::*;
use xcfun_ad::ctaylor::ctaylor_add;
// ... per-helper imports ...

use crate::density_vars::DensVarsDev;
use crate::functionals::mgga::shared::<helper> as <name>_shared;

#[cube]
pub fn <name>_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // ... body composing shared helpers + DensVarsDev fields ...
}
```

**Spin-decomposed pattern (port of `beckex.rs:83-95`):**

```rust
#[cube]
pub fn <name>_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    let mut e_alpha = Array::<F>::new(size);
    <name>_alpha::<F>(&d.a_43, &d.a, &d.gaa, /* &d.taua / &d.lapa / &d.jpaa */, &mut e_alpha, n);
    let mut e_beta = Array::<F>::new(size);
    <name>_alpha::<F>(&d.b_43, &d.b, &d.gbb, /* &d.taub / &d.lapb / &d.jpbb */, &mut e_beta, n);
    ctaylor_add::<F>(&e_alpha, &e_beta, out, n);
}
```

**Differences planner must specify per family:**

- **TPSS family:** kernel reads `d.a/b/gaa/gbb/taua/taub` (Vars id=13). TPSSLOCC additionally consumes `pw92eps::pw92eps` (verify `pub use` at `crates/xcfun-eval/src/functionals/lda/pw92eps.rs` per RESEARCH R9).
- **SCAN family:** kernel passes `#[comptime] idelec: u32` to `scan_like::get_SCAN_Fx` / `scan_like::r2SCAN_C` (0..=4 per variant). All SCAN kernels read Vars id=13.
- **M05/M06 family:** kernel takes parameter array (12-coef or 6-coef) as cubecl `Array<F>` argument; passes to `m0x_like::fw`. Vars id=13.
- **BLOCX:** Vars id=13, single inline body, no helper.
- **BR family:** kernel reads Vars id=17 (full JP); each calls `br_like::polarized` for α / β (BRC adds correlation; BRXC sums).
- **CSC:** Vars id=17, single inline expression per RESEARCH §"CSC carryover".

**Cross-references:** CONTEXT D-01-A; RESEARCH §"Source Tree Triage"; Risks R1, R2, R3.

---

### E.1 — `crates/xcfun-core/src/parameter_id.rs` (NEW; D-05-A)

**Closest analog:** `crates/xcfun-core/src/functional_id.rs` (78 variants, `#[repr(u32)]`).

**Pattern excerpt (functional_id.rs:1-40):**

```rust
//! ParameterId enum — 4 tunable parameters appended after FunctionalId.
//!
//! Discriminant ordering matches `xcfun-master/src/functionals/list_of_functionals.hpp:99-105`
//! (XC_RANGESEP_MU = XC_NR_FUNCTIONALS = 78, ..., XC_NR_PARAMETERS_AND_FUNCTIONALS = 82).

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
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
    pub fn from_name(name: &str) -> Option<Self> {
        // Case-insensitive (D-04-B) — match C++ `strcasecmp`.
        // ...
    }
}
```

**Defaults** (from `xcfun-master/src/functionals/common_parameters.cpp:17-29`):

| ParameterId | Default |
|-------------|---------|
| XC_RANGESEP_MU | 0.4 |
| XC_EXX | 0.0 |
| XC_CAM_ALPHA | 0.19 |
| XC_CAM_BETA | 0.46 |

**Cross-references:** CONTEXT D-05, D-05-A; RESEARCH §"Parameter Table" (lines 562-634).

---

### E.2 — `crates/xcfun-core/src/registry/generated/parameters.rs` (NEW)

**Closest analog:** `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` (31-row static slice).

**Pattern:**

```rust
// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.
// Source: xcfun-master/src/functionals/common_parameters.cpp:17-29

use crate::ParameterId;

pub struct ParameterRow {
    pub id: ParameterId,
    pub name: &'static str,
    pub description: &'static str,
    pub default_value: f64,
}

pub static PARAMETERS: [ParameterRow; 4] = [
    ParameterRow { id: ParameterId::XC_RANGESEP_MU, name: "rangesep_mu",
        description: "Range separation inverse length [1/a0]", default_value: 0.4 },
    ParameterRow { id: ParameterId::XC_EXX, name: "exx",
        description: "Amount of exact (HF like) exchange (must be provided externally)",
        default_value: 0.0 },
    ParameterRow { id: ParameterId::XC_CAM_ALPHA, name: "cam_alpha",
        description: "Amount of exact (HF like) exchange within CAM-B3LYP functional",
        default_value: 0.19 },
    ParameterRow { id: ParameterId::XC_CAM_BETA, name: "cam_beta",
        description: "Amount of long-range (HF like) exchange within CAM-B3LYP functional",
        default_value: 0.46 },
];
```

---

### E.3 — Populate `crates/xcfun-core/src/registry/generated/ALIASES.rs` (CURRENTLY EMPTY)

**Closest analog:** `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` (31-row literal).

**Current state (ALIASES.rs:1-14, EMPTY STUB):**

```rust
// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.
// Source: xcfun-master/src/functionals/aliases.cpp (Phase 2: empty; Phase 4 populates 46 rows)

#[derive(Debug, Clone, Copy)]
pub struct Alias {
    pub name: &'static str,
    pub description: &'static str,
    pub components: &'static [(&'static str, f64)],
}

pub static ALIASES: &[Alias] = &[];
```

**Phase 4 target (46 rows per RESEARCH §"All 46 aliases enumerated"):**

```rust
pub static ALIASES: &[Alias] = &[
    Alias { name: "null", description: "No functional",
        components: &[("slaterx", 0.0)] },
    Alias { name: "lda", description: "Slater + VWN5",
        components: &[("slaterx", 1.0), ("vwn5c", 1.0)] },
    Alias { name: "blyp", description: "Becke + LYP",
        components: &[("beckex", 1.0), ("lypc", 1.0)] },
    // ... 43 more rows from RESEARCH §"All 46 aliases enumerated" lines 487-535 ...
];
```

**Differences planner must specify:**
- Generated by extending `xtask/assets/regen_registry/extractor.cpp` (or Rust extractor) to parse `xcfun-master/src/functionals/aliases.cpp:17-138`.
- Registry-time invariant (RESEARCH lines 549-552): no alias term name matches an alias name. Drift gate at `xtask regen-registry --check`.

**Cross-references:** CONTEXT D-04, D-04-A, D-04-B; RESEARCH §"Alias Engine Semantics" + §"All 46 aliases enumerated".

---

### F.1 — `Mode::Contracted` host-side dispatch in `Functional::eval` (modify `functional.rs`)

**Closest analog:** `crates/xcfun-eval/src/functional.rs` `eval` mode-match at lines 113-203 + `launch_potential` (Phase 3 D-13 dispatcher).

**Pattern excerpt — current Mode::Contracted reject path (functional.rs:118-124):**

```rust
Mode::Contracted => {
    return Err(XcError::InvalidMode {
        mode: self.mode,
        depends: xcfun_core::Dependency::DENSITY,
    });
}
```

**Phase 4 target — replace reject with comptime ORDER dispatch (per RESEARCH §"Verbatim port of `XCFunctional.cpp:619-635`"):**

```rust
Mode::Contracted => {
    // Per XCFunctional.cpp:619-635 DOEVAL macro: 7 comptime arms (orders 0..=6).
    self.eval_contracted(input, output)?;
}
```

```rust
fn eval_contracted(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
    match self.order {
        0 => self.launch_contracted::<0>(input, output),
        1 => self.launch_contracted::<1>(input, output),
        2 => self.launch_contracted::<2>(input, output),
        3 => self.launch_contracted::<3>(input, output),
        4 => self.launch_contracted::<4>(input, output),
        5 => self.launch_contracted::<5>(input, output),
        6 => self.launch_contracted::<6>(input, output),
        _ => Err(XcError::InvalidOrder { order: self.order, mode: self.mode, n_vars: 0 }),
    }
}

fn launch_contracted<const ORDER: u32>(&self, input: &[f64], output: &mut [f64])
    -> Result<(), XcError>
{
    let inlen = self.vars.input_len();
    // Verify lengths per DOEVAL: input[inlen × (1 << ORDER)], output[1 << ORDER].
    // Pack input into per-Vars-element CTaylor block.
    // For each (id, weight): launch eval_point_kernel<F, ORDER> with packed input,
    //   accumulate weight * out[i] into output[i].
}
```

**`output_length` extension (functional.rs:231-255 — existing `Mode::Contracted` reject must be replaced per D-06-B):**

```rust
Mode::Contracted => Ok(1usize << order),     // D-06-B
```

**Cross-references:** CONTEXT D-06, D-06-A, D-06-B, D-06-C; RESEARCH §"Mode::Contracted Implementation" (lines 308-385).

---

### F.2 — `crates/xcfun-eval/tests/contracted_cross_mode.rs` (NEW)

**Closest analog:** `crates/xcfun-eval/tests/potential_parity.rs` (Phase 3 cross-mode parity test).

**Differences:**
- Cross-checks orders 0..=4 between PartialDerivatives and Contracted on a 1000-point subset (D-06-C).
- Orders 5/6 require new C-driver path (`validation/src/c_driver.rs` extension; Wave 5 deliverable).

**Cross-references:** CONTEXT D-06-C; RESEARCH §"Tier-2 Mode::Contracted cross-mode" (lines 996-1006).

---

### F.3 — Alias engine in `Functional::set` / `Functional::get` (modify `functional.rs`)

**Closest analog:** No prior Rust analog (Phase 4 introduces the alias engine). The shape is dictated by `XCFunctional.cpp:369-405`.

**Verbatim port (RESEARCH §"Verbatim port of `XCFunctional.cpp:369-405`" + §"Code Examples — Alias resolution recursion"):**

```rust
impl Functional {
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        // Case ①: functional name (additive)
        if let Some(id) = FunctionalId::from_name_case_insensitive(name) {
            self.settings[id as usize] += value;
            if !self.is_active(id) {
                self.active_functionals.push(id);
                self.depends |= FUNCTIONAL_DESCRIPTORS[id as usize].depends;
            }
            return Ok(());
        }
        // Case ②: parameter name (overwrite)
        if let Some(pid) = ParameterId::from_name(name) {
            self.settings[pid as usize] = value;
            return Ok(());
        }
        // Case ③: alias name (recursive with multiplicative weight)
        if let Some(alias) = ALIASES.iter().find(|a| a.name.eq_ignore_ascii_case(name)) {
            for (term_name, weight) in alias.components {
                self.set(term_name, value * weight)?;
            }
            return Ok(());
        }
        Err(XcError::UnknownName)
    }
}
```

**Differences planner must specify:**
- `Functional::settings: [f64; 82]` migration (R7) — replace `parameters: [f64; 4]` with the C++-aligned `[f64; 82]` layout. BECKESRX/BECKECAMX kernels read `settings[78]` instead of `parameters[1]` after migration.
- Algorithmic-identity rule **forbids** "fixing" the C++ `EXX-FIXME` (RESEARCH §"Verbatim port" line 432); Risk R6.

**Cross-references:** CONTEXT D-04; RESEARCH §"Alias Engine Semantics"; Pitfall P11; Risks R6, R7.

---

## Shared Patterns (cross-cutting; apply to multiple plans)

### S.1 — `#[cube] fn kernel<F: Float>` signature (D-09)

**Source:** `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs:19-24`

**Apply to:** all 32 metaGGA kernel files + every `mgga/shared/<helper>.rs` `#[cube] fn`.

```rust
#[cube]
pub fn <name>_kernel<F: Float>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    // ...
}
```

### S.2 — Algorithmic-identity rule (CLAUDE.md ACC-06)

**Source:** `crates/xcfun-eval/src/density_vars/build.rs:246-251`

**Apply to:** every kernel body composing primitives.

```rust
// gnn = gaa + 2*gab + gbb   (left-to-right, no mul_add per ACC-06)
let mut t1 = Array::<F>::new(comptime!((1_u32 << n) as usize));
let mut t2 = Array::<F>::new(comptime!((1_u32 << n) as usize));
ctaylor_scalar_mul::<F>(&out.gab, F::cast_from(2.0_f64), &mut t1, n);
ctaylor_add::<F>(&out.gaa, &t1, &mut t2, n);
ctaylor_add::<F>(&t2, &out.gbb, &mut out.gnn, n);
```

**CI gate:** `xtask check-no-mul-add` + `xtask check-no-fma` cover `crates/xcfun-eval/src/functionals/**/*.rs` (the `**` glob already includes `mgga/`).

### S.3 — Scratch allocation inside kernel body

**Source:** `crates/xcfun-ad/src/math.rs:90-92` + `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs:29`

**Apply to:** every `#[cube] fn` allocating temporaries.

```rust
let mut tmp = Array::<F>::new(comptime!((1_u32 << n) as usize));
// or for expand-style scratch (length n+1):
let mut scratch = Array::<F>::new(comptime!((n + 1) as usize));
```

### S.4 — `F::cast_from(<NAME>_F64)` for full f64 precision (Phase 2 ACC-04 lesson)

**Source:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs:26-43`

**Apply to:** every scalar constant in metaGGA kernel/helper bodies.

```rust
const NEG_C_SLATER_F64: f64 = -0.9305257363491002_f64;
// in kernel:
let neg_c_slater = F::cast_from(NEG_C_SLATER_F64);
ctaylor_scalar_mul::<F>(&tmp, neg_c_slater, out, n);
```

`F::new(f32)` introduces ~1.3e-7 rel-error and breaks the strict 1e-12 tier-1 threshold.

### S.5 — Module-doc + C++ source citation header

**Source:** `crates/xcfun-eval/src/functionals/lda/slaterx.rs:1-13` + `crates/xcfun-ad/src/expand/sqrt.rs:1-35`

**Apply to:** every new `.rs` file (kernel, helper, primitive, test).

Required sections: title + LDA/GGA/MGGA-XX label + `# Source` (cite `xcfun-master/src/functionals/<name>.cpp:<L0>-<L1>`) + `# Formula` block with **verbatim C++ excerpt** + `# Preconditions` (regularize / domain).

### S.6 — Explicit dispatch comptime if-chain (D-08, RESEARCH-confirmed extension to 36 arms)

**Source:** `crates/xcfun-eval/src/dispatch.rs:69-209` + `supports()` matches at 220-233

**Apply to:** dispatch.rs extension.

```rust
} else if comptime!(id == 71) {
    // XC_TPSSX (Phase 4 Wave 1)
    crate::functionals::mgga::tpssx::tpssx_kernel::<F>(d, out, n);
} else if comptime!(id == 72) {
    // XC_TPSSC
    crate::functionals::mgga::tpssc::tpssc_kernel::<F>(d, out, n);
}
// ... 34 more arms ...
```

`supports()` matches block bumped from 46 → 82 ids per CONTEXT D-08. Add 36 ids: 32 metaGGA + 4 carryovers (BRX=10, BRC=11, BRXC=12, CSC=66).

### S.7 — `#[forbid(unsafe_code)]` crate-root attribute (CLAUDE.md security)

**Source:** `crates/xcfun-ad/src/lib.rs:10`

**Apply to:** crate roots `crates/xcfun-core/src/lib.rs` + `crates/xcfun-eval/src/lib.rs` (verify presence; do not add new `unsafe` to any new file).

---

## No Analog Found (3 patterns — Phase 4 originates)

| File / Pattern | Role | Reason | Source spec |
|----------------|------|--------|-------------|
| `Functional::set/get` alias resolution recursion | mode-dispatcher (recursive) | Phase 1/2/3 do not implement set/get with alias semantics | `xcfun-master/src/XCFunctional.cpp:369-405` (verbatim port; RESEARCH §"Alias Engine Semantics") |
| `Mode::Contracted` host-side dispatch (`launch_contracted<const ORDER: u32>`) | mode-dispatcher | `Mode::Contracted` rejected in Phase 2/3 (`functional.rs:118-124`) | `xcfun-master/src/XCFunctional.cpp:619-635` DOEVAL macro |
| `ParameterId` enum (4 variants 78..=81) | enum + helpers | `FunctionalId` has 78 variants; `ParameterId` is sibling but conceptually separate | `xcfun-master/src/functionals/list_of_functionals.hpp:99-105` |

For these three, planner should treat RESEARCH §"Mode::Contracted Implementation" / §"Alias Engine Semantics" / §"Parameter Table" as the **primary specification** (verbatim port from C++ source) rather than seeking a Rust analog.

---

## Metadata

**Analog search scope:**
- `crates/xcfun-eval/src/functionals/{lda,gga}/**/*.rs` — kernel + helper analogs
- `crates/xcfun-ad/src/{expand,math}.rs` — primitive analogs
- `crates/xcfun-eval/src/density_vars/build.rs` — densvars-arm analog
- `crates/xcfun-eval/src/dispatch.rs` — dispatch analog
- `crates/xcfun-eval/src/functional.rs` — host-side mode-dispatcher analog
- `crates/xcfun-core/src/{enums.rs,functional_id.rs,registry/generated/*.rs}` — registry analogs
- `crates/xcfun-{ad,eval}/tests/*.rs` — test analogs
- `xtask/src/bin/regen_registry.rs` — extractor pattern
- `validation/{build.rs,c_stubs.cpp,src/fixtures.rs}` — validation harness analogs

**Files scanned:** ~110 (`*.rs` files in `crates/xcfun-{core,eval,ad}/src/**` + `crates/xcfun-{eval,ad}/tests/**` + `validation/{src,build.rs,c_stubs.cpp}` + `xtask/src/bin/**`).

**Pattern extraction date:** 2026-04-25.

**Strongest analogs (5 most copyable):**
1. `crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs:19-38` (single-call kernel template)
2. `crates/xcfun-eval/src/functionals/gga/becke/beckex.rs:83-95` (spin-decomposed kernel template)
3. `crates/xcfun-eval/src/density_vars/build.rs:222-262` (densvars chain pattern)
4. `crates/xcfun-eval/src/dispatch.rs:69-209` (comptime if-chain extension)
5. `crates/xcfun-ad/src/expand/sqrt.rs:42-62` (recurrence-style writer into `&mut Array<F>`)
