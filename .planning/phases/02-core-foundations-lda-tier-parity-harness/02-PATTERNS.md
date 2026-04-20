# Phase 2: Core Foundations + LDA Tier + Parity Harness — Pattern Map

**Mapped:** 2026-04-20
**Files analyzed:** 49 new/modified files (Wave-0 surgical rewrite + Wave-1A registry codegen + Wave-1B/1C `xcfun-eval` cubecl launcher + 11 LDA bodies + Wave-2 validation harness)
**Analogs found:** 49 / 49 (every file maps to either a Phase 1 cubecl primitive in `crates/xcfun-ad/src/`, an existing `crates/xcfun-core/src/` file being rewritten, an existing `xtask/src/bin/` gate, or a `xcfun-master/` C++ port target)

**Important note on analogs:** Phase 2 is the **first cubecl launcher in the workspace**. There is no existing `#[derive(CubeType, CubeLaunch)]` Rust code in `crates/` (Phase 1 used kernel-local `Array<F>::new` rather than struct-of-Arrays types). Analogs fall into four categories:

| Analog kind | Role in the port |
|-------------|------------------|
| **Phase 1 cubecl primitive (`crates/xcfun-ad/src/**`)** | Composition target. Every LDA `#[cube] fn` body composes `ctaylor_*` + `*_expand` from Phase 1, never re-implements them. Operation order is locked at the C++ source level (Phase 1 D-08). |
| **C++ reference (`xcfun-master/src/**`)** | Algorithmic-identity source-of-truth for all 11 LDA functional bodies + `densvars.hpp` regularize/build chain + `XCFunctional.cpp` PartialDerivatives output dispatcher + `xcint.cpp` VARS_TABLE rows + `xcfun.h` C ABI / FFI shim target. |
| **Pre-pivot xcfun-core (`crates/xcfun-core/src/**`)** | About to be surgically rewritten in Wave-0 per D-05. `enums.rs` (rename + add Unset), `error.rs` (CORE-04 9 variants + Copy + #[non_exhaustive] + UnknownName drop payload per D-25), `traits.rs` (keep Dependency, drop Functional/TestData), `functional_id.rs` (re-order to match `list_of_functionals.hpp`), `constants.rs` (audit), `test_data.rs` (delete), `density_vars.rs` (delete 825 lines), `lib.rs` (drop `pub use xcfun_ad::Num`). |
| **xtask precedent (`xtask/src/bin/check_no_fma.rs` + `regen_ad_fixtures.rs`)** | Pattern source for all 5 new xtask binaries (regen-registry, check-no-mul-add, check-no-anyhow, check-boundaries, check-cubecl-pin) and for the validate wrapper. The `cargo rustc --emit=asm + scan` and `cc-compile + driver-exec + parse-stdout + sha256-stamp` idioms transfer directly. |

---

## File Classification

### Wave-0 surgical rewrite (xcfun-core)

| New / Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------------|------|-----------|----------------|---------------|
| `Cargo.toml` (workspace) | config | static data | self (current `members`/`exclude` list) | role-match — add cubecl-cpu workspace dep + re-include xcfun-core, xcfun-eval, validation members |
| `.cargo/config.toml` | config | static data | self (Phase 1 W13 form) | exact — verify present, no edits expected |
| `crates/xcfun-core/src/lib.rs` | crate-root / re-exports | config | self (lines 1-28) | role-match — drop `pub use xcfun_ad::Num`, drop density_vars mod, retain taylorlen + tests |
| `crates/xcfun-core/src/enums.rs` | model (Vars + Mode types) | static data | self (lines 1-322) | role-match — rename EvalMode→Mode (+Unset=0 +#[repr(u32)]), rename VarType→Vars (+#[allow(non_camel_case_types)]); variants kept verbatim |
| `crates/xcfun-core/src/error.rs` | model (XcError) | static data | self (lines 1-89) | role-match — extend to 9 variants, add Copy/Send/Sync + #[non_exhaustive]; drop String payload from UnknownName per D-25 |
| `crates/xcfun-core/src/traits.rs` | model (Dependency bitflags) | static data | self (lines 1-76) | role-match — keep Dependency bitflags, REMOVE `pub trait Functional` and `pub struct TestData` (functional surface moves to xcfun-eval per D-04) |
| `crates/xcfun-core/src/functional_id.rs` | model (FunctionalId enum, 78 entries) | static data | self (lines 1-300+) + `xcfun-master/src/functionals/list_of_functionals.hpp:17-95` | role-match — non-trivial REORDER to match xcint historical ordering (CORE-07 requirement); keep COUNT=78 |
| `crates/xcfun-core/src/constants.rs` | config (physical constants) | static data | self (lines 1-67) | exact — audit only; no expected edits |
| `crates/xcfun-core/src/test_data.rs` | (DELETE) | — | self (2 lines) | role-match — delete; superseded by FUNCTIONAL_DESCRIPTORS.test_in/test_out |
| `crates/xcfun-core/src/density_vars.rs` | (DELETE) | — | self (825 lines, host `<T:Num>` struct) | role-match — fully obsolete under D-01 (DensVarsDev moves to xcfun-eval as `#[cube]` type) |

### Wave-1A xtask regen-registry + QG gates

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `xtask/src/bin/regen_registry.rs` | utility (codegen orchestrator) | file-I/O + cc-compile + JSONL parse | `xtask/src/bin/regen_ad_fixtures.rs` | exact — same compile-driver-parse-write-stamp shape; swap bincode → .rs file emission + .sha256 stamp |
| `xtask/assets/regen_registry/extractor.cpp` | build (C++ extractor, ~150 LoC) | stdin/stdout (JSONL emission) | `xtask/assets/regen_ad_fixtures/driver.cpp` (referenced by Phase 1) | role-match — same standalone-cpp-on-vendored-headers shape; emits JSONL not semicolon-CSV |
| `xtask/src/bin/check_no_mul_add.rs` | utility (regex-grep CI gate) | file-I/O | `xtask/src/bin/check_no_fma.rs` | exact — clone the project_root + `cargo rustc --emit=asm + scan` shape, swap `ctaylor_mul`-symbol filter for `xcfun-eval/src/functionals/**/*.rs` source filter, swap FMA mnemonic list for `\.mul_add\s*\(` regex |
| `xtask/src/bin/check_no_anyhow.rs` | utility (Cargo.toml grep CI gate) | file-I/O | `check_no_fma.rs` (project_root + walk pattern) | role-match — different scan target (parse `crates/*/Cargo.toml` for `[dependencies]` containing anyhow); same exit-2-on-fail idiom |
| `xtask/src/bin/check_boundaries.rs` | utility (cargo metadata gate) | request-response (subprocess) | `check_no_fma.rs` (subprocess Command pattern) | role-match — `cargo metadata --format-version 1` then JSON-parse to assert library/app boundary rules |
| `xtask/src/bin/check_cubecl_pin.rs` | utility (cargo metadata gate) | request-response (subprocess) | `check_no_fma.rs` (subprocess Command pattern) | role-match — same `cargo metadata` parse, asserts `cubecl == 0.10.0-pre.3` and `cubecl-cpu == 0.10.0-pre.3` |
| `xtask/src/bin/validate.rs` | utility (CLI wrapper) | request-response (subprocess) | `xtask/src/main.rs` (subcommand dispatch) + `regen_ad_fixtures.rs` (subprocess pattern) | role-match — thin wrapper that delegates argv to `cargo run -p validation -- <args>` |

### Wave-1A registry codegen output (xcfun-core/src/registry/generated/)

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs` | model (78-entry static array) | static data | `xcfun-master/src/functionals/{slaterx,vwn3,…}.cpp` FUNCTIONAL macro payloads + `xcfun-master/src/functionals/list_of_functionals.hpp` (78-id ordering) | exact — extracted from C++ macros via the C++ extractor, 11 LDA populated + 67 stubs |
| `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` | model (31-row static array) | static data | `xcfun-master/src/xcint.cpp:93-135` | exact — verbatim row port; field shape `{symbol, len, provides}` |
| `crates/xcfun-core/src/registry/generated/ALIASES.rs` | model (empty static slice in Phase 2) | static data | `xcfun-master/src/functionals/aliases.cpp:17-…` (empty contribution this phase per D-12) | role-match — declared as `pub static ALIASES: &[Alias] = &[];` |
| `crates/xcfun-core/src/registry/generated/*.sha256` | config (drift stamp) | static data | `xtask/src/bin/regen_ad_fixtures.rs` lines 165-176 (`header_sha256`) | exact — SHA-256 hash committed alongside each .rs file |

### Wave-1B + Wave-1C xcfun-eval crate

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `crates/xcfun-eval/Cargo.toml` | config | static data | `crates/xcfun-ad/Cargo.toml` | role-match — same workspace inheritance pattern; add xcfun-core + xcfun-ad path deps + cubecl + cubecl-cpu pinned `=0.10.0-pre.3` + thiserror |
| `crates/xcfun-eval/src/lib.rs` | crate-root | config | `crates/xcfun-ad/src/lib.rs` | exact — same `#![forbid(unsafe_code)]` + `pub mod` wiring + `#[cfg(feature = "testing")] pub mod for_tests` |
| `crates/xcfun-eval/src/for_tests.rs` (or `for_tests/mod.rs` + `cpu_client.rs`) | utility (singleton `OnceLock<CpuClient>`) | singleton init | `crates/xcfun-ad/src/for_tests/cpu_client.rs` | exact — verbatim copy; only the `XcfunEval` crate-prefix changes in any future kernel-name comments |
| `crates/xcfun-eval/src/density_vars.rs` | model (`#[derive(CubeType, CubeLaunch)] DensVarsDev<F>`) | transform (struct of `Array<F>`) | NO IN-REPO ANALOG (Phase 1 used only kernel-local `Array<F>`); cubecl-book `language-support/struct.md` cited in RESEARCH §"D-02" | none — Pattern A from RESEARCH.md §"D-02 cubecl Nesting Decision"; first `#[derive(CubeType, CubeLaunch)]` site in the workspace |
| `crates/xcfun-eval/src/density_vars/build.rs` | transform (`build_densvars` dispatcher + 5 variant arms) | transform (per-variant chain) | `xcfun-master/src/densvars.hpp:35-218` (`densvars<T>::densvars(parent, d)` switch-fallthrough constructor) | exact — port with helper-function chains replacing C-style fallthrough (CORE-05, Pitfall P5) |
| `crates/xcfun-eval/src/density_vars/regularize.rs` | transform (`regularize` `#[cube] fn`) | transform | `xcfun-master/src/densvars.hpp:22-25` (`regularize(ctaylor<T,N>& x)`) + Phase 1 `crates/xcfun-ad/src/ctaylor.rs::ctaylor_zero` (cubecl idiom reference) | exact — mutates only `c[CNST]` (D-22, CORE-06) |
| `crates/xcfun-eval/src/functional.rs` | model (`Functional` struct) | request-response | `crates/xcfun-core/src/traits.rs:34-53` (pre-pivot `pub trait Functional`, being deleted) + RESEARCH §"Registry Shape + Circular-Dep Resolution" | role-match — minimal slice (D-21): `weights`, `vars`, `mode`, `order`; `eval(input, output) -> Result<(), XcError>` |
| `crates/xcfun-eval/src/dispatch.rs` | service (functional id → kernel match) | dispatch / control-flow | `xcfun-master/src/XCFunctional.cpp:493-617` (PartialDerivatives dispatcher) + RESEARCH §"Registry Shape + Circular-Dep Resolution" | role-match — match-on-FunctionalId resolves the registry circular-dep (xcfun-eval depends on xcfun-core, not vice versa) |
| `crates/xcfun-eval/src/functionals/lda/slaterx.rs` | transform (`#[cube] fn slaterx_kernel`) | transform | `xcfun-master/src/functionals/slaterx.cpp:18-37` + `slater.hpp:19-21` | exact — 1-line C++ → ~5-line cubecl with explicit ctaylor_add + ctaylor_scalar_mul |
| `crates/xcfun-eval/src/functionals/lda/vwn3c.rs` | transform | transform | `xcfun-master/src/functionals/vwn3.cpp:18-30` + `vwn.hpp` (vwn3_eps helper) | exact — `d.n * vwn::vwn3_eps(d)` ports to ctaylor_mul over a vwn3_eps helper |
| `crates/xcfun-eval/src/functionals/lda/vwn5c.rs` | transform | transform | `xcfun-master/src/functionals/vwn5c.cpp` + `vwn.hpp` (vwn5_eps helper) | exact — same pattern as vwn3c |
| `crates/xcfun-eval/src/functionals/lda/pw92c.rs` | transform | transform | `xcfun-master/src/functionals/pw92c.cpp` + `pw92eps.hpp:36-58` (XCFUN_REF_PW92C-undefined accurate constants per RESEARCH §"PW92C Legacy Constants") | exact — ship accurate constants only (no legacy-feature flag); escalate per D-19 if rel-err > 1e-12 |
| `crates/xcfun-eval/src/functionals/lda/pz81c.rs` | transform | transform | `xcfun-master/src/functionals/pz81c.cpp` + `pz81c.hpp` | exact |
| `crates/xcfun-eval/src/functionals/lda/ldaerfx.rs` | transform | transform + erf composition | `xcfun-master/src/functionals/ldaerfx.cpp:24-73` (esrx_ldaerfspin + lda_erfx) | exact — ports the 4-branch range-separated kernel; tier-2 overrides to 1e-7 per D-24 |
| `crates/xcfun-eval/src/functionals/lda/ldaerfc.rs` | transform | transform + erf composition | `xcfun-master/src/functionals/ldaerfc.cpp` (full body) | exact — same 1e-7 override per D-24 |
| `crates/xcfun-eval/src/functionals/lda/ldaerfc_jt.rs` | transform | transform + erf composition | `xcfun-master/src/functionals/ldaerfc_jt.cpp` | exact — no upstream test_in (line 64 ends at ENERGY_FUNCTION); tier-1 self-test SKIPS this functional; tier-2 covers it at 1e-7 per D-24 |
| `crates/xcfun-eval/src/functionals/lda/tfk.rs` | transform | transform | `xcfun-master/src/functionals/tfk.cpp:20-40` | exact — `CF * pow(d.n, 5/3)` → ctaylor_pow |
| `crates/xcfun-eval/src/functionals/lda/tw.rs` | transform | transform (kinetic-GGA) | `xcfun-master/src/functionals/tw.cpp:20-30` | exact — REQUIRES `XC_A_B_GAA_GAB_GBB` builder (Wave-1C; Pitfall PHASE2-D) |
| `crates/xcfun-eval/src/functionals/lda/vwk.rs` | transform | transform (kinetic-GGA) | `xcfun-master/src/functionals/vonw.cpp:17-30` (file is `vonw.cpp`, FUNCTIONAL is `XC_VWK`) | exact — `gaa/(8*a) + gbb/(8*b)`; same `XC_A_B_GAA_GAB_GBB` requirement |
| `crates/xcfun-eval/tests/self_tests.rs` | test (tier-1 parametric loop) | request-response | `crates/xcfun-ad/tests/cubecl_spike.rs` (test harness shape) + RESEARCH.md §"Phase Requirements → Test Map" tier-1 row | role-match — loops over `FUNCTIONAL_DESCRIPTORS.iter().filter(|d| d.test_in.is_some())` per ACC-04 |
| `crates/xcfun-eval/tests/densvars_field_parity.rs` | test (CORE-05 22-field check per variant) | request-response | (no in-repo analog; first 22-field densvars test) | none — design from RESEARCH.md §"build_densvars Pattern" + densvars.hpp:35-218 |

### Wave-2 validation harness

| New File | Role | Data Flow | Closest Analog | Match Quality |
|----------|------|-----------|----------------|---------------|
| `validation/Cargo.toml` | config | static data | `xtask/Cargo.toml` (only allowed `anyhow`-bearing crate today) | role-match — workspace inheritance + cc build-dep + anyhow + approx + serde_json + rand_xoshiro + tracing-subscriber |
| `validation/build.rs` | build (cc-compile xcfun-master subset) | file-I/O + subprocess | `xtask/src/bin/regen_ad_fixtures.rs::compile_driver` (lines 56-81) | role-match — `cc::Build` instead of raw `Command::new($CXX)`, but same flag set: `-std=c++17 -fno-fast-math -ffp-contract=off -DXCFUN_MAX_ORDER=6` |
| `validation/c_stubs.cpp` | build (auto-generated 67 stub `FUNCTIONAL` instantiations) | static data | `xcfun-master/src/functionals/slaterx.cpp` (FUNCTIONAL macro shape) + RESEARCH §"Validation Harness Bring-Up" | role-match — auto-emitted by extractor for Phase 2; xcint.cpp recursion needs every fundat_db<XC_*> specialization or link fails |
| `validation/src/main.rs` | controller (CLI entry) | request-response | `xtask/src/main.rs` (argv-dispatch shape) | role-match — extends to clap-style `--backend cpu --order N --filter regex` |
| `validation/src/ffi.rs` | service (extern "C" CppXcfun wrapper) | request-response | NO IN-REPO ANALOG (first FFI shim in workspace); `xcfun-master/api/xcfun.h:1-130` (extern "C" declaration shape) | role-match — extern "C" decls + RAII Drop wrapper |
| `validation/src/fixtures.rs` | utility (10k-point grid generator) | batch (4-stratum 70/30) | RESEARCH §"Grid Generator Spec" full skeleton + Phase 1 `crates/xcfun-ad/tests/proptest_algebra.rs` (rand_xoshiro seed pattern, commit `3514217`) | role-match — first stratified-grid generator; pattern is Pattern A from RESEARCH §"Grid Generator Spec" |
| `validation/src/driver.rs` | service (tuple loop, rel-err compute, accumulate) | batch + request-response | `xcfun-master/src/XCFunctional.cpp:493-617` (output element layout that the driver must mirror) | role-match — outer loop over (functional, vars, mode, order, point); inner loop over output elements |
| `validation/src/report.rs` | utility (HTML matrix + JSONL writer) | file-I/O | `xtask/src/bin/regen_ad_fixtures.rs::main` (manifest-write pattern, lines 285-300) + RESEARCH §"report.html schema" + §"report.jsonl schema" | role-match — serde_json::to_string for JSONL, hand-written HTML for matrix table per RESEARCH §"report.html schema" |

---

## Pattern Assignments

### Wave-0 surgical rewrite (xcfun-core)

#### `crates/xcfun-core/src/lib.rs` (crate-root, surgical rewrite)

**Analog:** self (lines 1-28 currently)

**Current state to remove** (lines 12, 20, 27):

```rust
pub mod density_vars;        // line 12 — DELETE (file is being deleted)
pub use density_vars::DensityVars;   // line 20 — DELETE
pub use xcfun_ad::Num;       // line 27 — DELETE (Num retired in Phase 1 D-09)
```

**Final shape after Wave-0c (drop broken `Num` re-export) + Wave-0d (rename):**

```rust
//! xcfun-core: Core types and registry tables for xcfun_rs.
#![forbid(unsafe_code)]

pub mod constants;
pub mod enums;
pub mod error;
pub mod functional_id;
pub mod registry;            // NEW (Wave-1A) — exposes FUNCTIONAL_DESCRIPTORS, VARS_TABLE, ALIASES
pub mod traits;

pub use constants::*;
pub use enums::{Mode, Vars};                  // renamed from {EvalMode, VarType}
pub use error::XcError;
pub use functional_id::FunctionalId;
pub use traits::Dependency;                   // Functional + TestData removed (D-04)
pub use registry::{FUNCTIONAL_DESCRIPTORS, VARS_TABLE, ALIASES, FunctionalDescriptor, VarsRow, Alias};

pub const fn taylorlen(n_vars: usize, order: usize) -> usize { /* unchanged */ }
```

**Adaptation notes:**
- Wave-0a/c-only: edit only the `mod` + `pub use` lines; preserve `taylorlen` body + tests verbatim.
- Wave-1A: add `pub mod registry;` + the registry re-exports.
- Add `#![forbid(unsafe_code)]` (Phase 1 idiom from `crates/xcfun-ad/src/lib.rs:10`).

**Cross-file dependencies:** depends on `enums.rs` (Wave-0d), `error.rs` (Wave-0e), `registry/` module (Wave-1A).

---

#### `crates/xcfun-core/src/enums.rs` (Wave-0d rename + Unset variant)

**Analog:** self (lines 1-322 currently)

**Current `EvalMode` (lines 5-14):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalMode {
    PartialDerivatives,
    Potential,
    Contracted,
}
```

**Target shape (D-07 + CORE-02 + xcfun.h:35-41):**
```rust
/// Evaluation mode for functional derivatives. Discriminants match `xcfun.h::xcfun_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Mode {
    Unset = 0,                     // matches XC_MODE_UNSET (xcfun.h:36)
    PartialDerivatives = 1,        // matches XC_PARTIAL_DERIVATIVES
    Potential = 2,                 // matches XC_POTENTIAL
    Contracted = 3,                // matches XC_CONTRACTED
}
```

**Current `VarType` (lines 21-72):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VarType {
    A = 0,
    N = 1,
    A_B = 2,
    /* ... 28 more variants ... */
}
```

**Target shape (D-08 + D-06 + CORE-01):**
```rust
/// Input variable specification. Discriminants match `xcfun.h::xcfun_vars` exactly.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Vars {
    A = 0,
    N = 1,
    A_B = 2,
    /* ... 28 more variants, names UNCHANGED ... */
}
```

**Adaptation notes:**
- Variant names + discriminants are kept verbatim (CORE-01 contract). Only the type names change.
- `#[allow(non_camel_case_types)]` is required because variants like `A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` are SCREAMING_SNAKE_CASE (D-06).
- `input_len`, `provides`, `is_spin_polarized` methods migrate verbatim (only the `VarType::` → `Vars::` and `EvalMode::` → `Mode::` qualifier renames).
- The existing `var_type_cpp_ordering` test (lines 268-274) becomes `vars_cpp_ordering` and stays unchanged structurally.
- `Mode` discriminants change from compiler-assigned to explicit `= 0..=3`. The existing `eval_mode_has_3_variants` test (lines 260-265) extends to test all 4 incl. `Mode::Unset`.

**Cross-file dependencies:** consumed by `error.rs` (Wave-0e), `traits.rs` (`Dependency` ref unchanged), and every Phase 2 downstream consumer.

---

#### `crates/xcfun-core/src/error.rs` (Wave-0e CORE-04 compliance + D-25 UnknownName drop payload)

**Analog:** self (lines 1-89 currently — 7 variants, no `Copy`, no `#[non_exhaustive]`)

**Current shape (lines 7-39):**
```rust
#[derive(Debug, thiserror::Error)]
pub enum XcError {
    #[error("invalid derivative order {order} for mode {mode:?} with {n_vars} input variables")]
    InvalidOrder { order: u32, mode: EvalMode, n_vars: usize },

    #[error("variable type {vars:?} does not provide required dependencies {required:?}")]
    InsufficientVars { vars: VarType, required: Dependency },

    #[error("mode {mode:?} is not supported for functionals with dependencies {depends:?}")]
    UnsupportedMode { mode: EvalMode, depends: Dependency },

    #[error("functional not configured: call eval_setup() before eval()")]
    NotConfigured,

    #[error("unknown functional or parameter name: {0:?}")]
    UnknownName(String),                                                 // ← NOT Copy

    #[error("input length {got} does not match expected {expected}")]
    InputLengthMismatch { expected: usize, got: usize },

    #[error("output length {got} does not match expected {expected}")]
    OutputLengthMismatch { expected: usize, got: usize },
}
```

**Target shape (CORE-04 9-variant + Copy + Send + Sync + #[non_exhaustive]; D-25 drops UnknownName payload):**
```rust
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[non_exhaustive]
pub enum XcError {
    #[error("invalid derivative order {order} for mode {mode:?} with {n_vars} input variables")]
    InvalidOrder { order: u32, mode: Mode, n_vars: usize },

    #[error("variable type {vars:?} does not provide required dependencies {required:?}")]
    InvalidVars { vars: Vars, required: Dependency },                    // renamed from InsufficientVars

    #[error("mode {mode:?} is not supported for functionals with dependencies {depends:?}")]
    InvalidMode { mode: Mode, depends: Dependency },                     // renamed from UnsupportedMode

    #[error("unknown functional name")]
    UnknownName,                                                          // D-25: payload dropped (was String)

    #[error("input length {got} does not match expected {expected}")]
    InputLengthMismatch { expected: usize, got: usize },

    #[error("output length {got} does not match expected {expected}")]
    OutputLengthMismatch { expected: usize, got: usize },

    #[error("functional not configured: call eval_setup() before eval()")]
    NotConfigured,

    #[error("invalid input encoding")]
    InvalidEncoding,                                                      // NEW (CORE-04)

    #[error("runtime error during kernel launch")]
    Runtime,                                                              // NEW (CORE-04)
}
```

**Adaptation notes:**
- All field types must be `Copy` — `Mode`, `Vars`, `Dependency` are already `Copy` (verified in current file).
- `UnknownName` drops the `String` payload entirely per D-25 (the variant becomes a unit variant).
- Add `static_assertions::assert_impl_all!(XcError: Copy, Send, Sync)` test (RESEARCH.md §"Phase Requirements → Test Map" CORE-04 row); add `static_assertions` to `[dev-dependencies]`.
- `#[non_exhaustive]` is mandatory per CORE-04. This forces downstream `match` sites to add `_ => …`, which is a feature not a bug for forward compat.
- `ffi_code` method (lines 41-50) is dropped from this phase; `as_c_code` is Phase 5 (CAPI-05).

**Cross-file dependencies:** consumed by `xcfun-eval::Functional::eval` return type, `xcfun-eval::dispatch::dispatch_kernel` return type, and Wave-2 `validation/` driver error handling.

---

#### `crates/xcfun-core/src/traits.rs` (Wave-0e — keep Dependency, drop Functional/TestData)

**Analog:** self (lines 1-76 currently)

**Keep verbatim** (lines 7-17):
```rust
bitflags::bitflags! {
    /// Dependency flags indicating which input quantities a functional requires.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Dependency: u32 {
        const DENSITY   = 0b0000_0001;
        const GRADIENT  = 0b0000_0010;
        const LAPLACIAN = 0b0000_0100;
        const KINETIC   = 0b0000_1000;
        const JP        = 0b0001_0000;
    }
}
```

**Delete** (lines 19-53):
- `pub struct TestData { … }` — superseded by `FunctionalDescriptor::{test_in, test_out, test_threshold}` post-codegen (CORE-07).
- `pub trait Functional { … }` — moves to `xcfun-eval::Functional` per D-04. The trait references `xcfun_ad::Num` (line 37) which is dead.
- `use crate::density_vars::DensityVars;` (line 3) — `density_vars.rs` is being deleted in Wave-0b.

**Adaptation notes:**
- C-header cross-check (RESEARCH §"Phase Requirements" CORE-03 row + xcint.hpp:46-50): `XC_DENSITY=1, XC_GRADIENT=2, XC_LAPLACIAN=4, XC_KINETIC=8, XC_JP=16` — **matches existing layout**. No bit-value edits.
- Existing `dependency_bits` + `dependency_bitwise_operations` tests (lines 56-74) stay verbatim.

**Cross-file dependencies:** consumed by `enums.rs::Vars::provides`, `error.rs::XcError::{InvalidVars, InvalidMode}`, registry codegen output (`FunctionalDescriptor::depends`), every functional's `Dependency` declaration.

---

#### `crates/xcfun-core/src/functional_id.rs` (Wave-0e non-trivial REORDER to match xcint historical insertion)

**Analog:** self (lines 1-300+, currently family-grouped) + `xcfun-master/src/functionals/list_of_functionals.hpp:17-95` (xcint historical-insertion ordering)

**Current ordering** (lines 11-102, family-grouped):
```rust
pub enum FunctionalId {
    SlaterX = 0,
    Vwn3C,           // = 1
    Vwn5C,           // = 2
    Pz81C,           // = 3
    Pw92C,           // = 4
    Pw86X,           // = 5  ← C++ has SlaterX=0, Pw86X=1, Vwn3C=2 …
    /* ... */
}
```

**Target ordering (CORE-07 + CORE-10 require `FunctionalId as u32 == xcfun_functional_id`):**
```rust
// 1:1 port of xcfun-master/src/functionals/list_of_functionals.hpp:17-95
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FunctionalId {
    XC_SLATERX = 0,         // line 18
    XC_PW86X = 1,           // line 19
    XC_VWN3C = 2,           // line 20
    XC_VWN5C = 3,           // line 21
    XC_PBEC = 4,            // line 22
    XC_PBEX = 5,
    /* … 78 entries, screaming-snake-case to match the C++ identifiers per D-06 spirit … */
    XC_VWK = 58,            // line 77
    /* … remaining entries … */
}
```

**Adaptation notes:**
- This is the **single largest Wave-0e churn** — current `from_name` lookup table (lines 109+) uses lowercase/family naming and must be rewritten in lockstep.
- Keep `pub const COUNT: usize = 78`.
- Rename to `XC_*` SCREAMING_SNAKE_CASE per D-06 spirit (matches xcfun.h C identifier exactly minus prefix removal).
- Add a unit test asserting `FunctionalId::XC_SLATERX as u32 == 0`, `XC_PW86X as u32 == 1`, `XC_VWK as u32 == 58`, etc., against `list_of_functionals.hpp` line numbers.
- Phase 5 (CAPI-01) will reuse this exact ordering for the C ABI; getting it right now avoids re-ordering churn later.

**Cross-file dependencies:** referenced by `FUNCTIONAL_DESCRIPTORS` array indexing (the `as u32` value indexes into the per-id fp-table downstream), `xcfun-eval::dispatch::dispatch_kernel` match arms, Wave-2 `validation/src/driver.rs` per-id loop.

---

#### `crates/xcfun-core/src/constants.rs` (Wave-0e audit; expected no edits)

**Analog:** self (lines 1-67 currently)

**Verify against C++** (no expected edits; cross-references):
- `C_SLATER = 0.9305257363491002` (line 10) — matches `pow(81 / (32 * PI), 1/3)`. Verified by existing test (lines 41-47).
- `CF = 2.8711842930059836` (line 13) — matches xcfun's runtime-computed Thomas-Fermi constant.
- `TINY_DENSITY = 1e-14` (line 16) — matches `xcfun-master/src/config.hpp:22` `XCFUN_TINY_DENSITY`.
- `RS_PREFACTOR = 0.6203504908994001` (line 34) — matches `pow(3/(4*PI), 1/3)`.

**Cross-file dependencies:** consumed by `xcfun-eval::density_vars::build` (RS_PREFACTOR for `r_s` derivation), `xcfun-eval::density_vars::regularize` (TINY_DENSITY threshold), `xcfun-eval::functionals::lda::*` (C_SLATER for slaterx, CF for tfk).

---

### Wave-0a workspace + .cargo/config.toml verification

#### `Cargo.toml` (workspace, Wave-0a edit)

**Analog:** self (lines 1-47)

**Current state** (lines 6-14):
```toml
members = ["crates/xcfun-ad", "xtask"]
exclude = [
    "crates/xcfun-core",
    "crates/xcfun-eval",
    /* ... */
]
```

**Wave-0a target (just add cubecl-cpu workspace dep, no member changes — defer member re-include to Wave-0f per RESEARCH §"Wave-0 Commit Order"):**
```toml
[workspace.dependencies]
# (existing entries unchanged)
cubecl = "=0.10.0-pre.3"
cubecl-cpu = "=0.10.0-pre.3"     # Phase 2 NEW — pre-stage for xcfun-eval (Wave-0g/Wave-1)
```

**Wave-0f target (after xcfun-core src cleanup is coherent):**
```toml
members = ["crates/xcfun-ad", "crates/xcfun-core", "xtask"]
exclude = [
    "crates/xcfun-eval",        # added back in Wave-1B-1
    "crates/xcfun-ffi",
    "crates/xcfun-functionals",
    "crates/xcfun-gpu",
    "crates/xcfun-python",
]
```

**Wave-1B-1 + Wave-2-1 target (final Phase 2 state):**
```toml
members = ["crates/xcfun-ad", "crates/xcfun-core", "crates/xcfun-eval", "xtask", "validation"]
exclude = ["crates/xcfun-ffi", "crates/xcfun-functionals", "crates/xcfun-gpu", "crates/xcfun-python"]
```

**Adaptation notes:**
- The 3-stage member-list mutation honors D-09 atomicity: each commit leaves the workspace in a buildable state.
- `cubecl-cpu` workspace dep can be added in Wave-0a (no consumer yet); resolves before xcfun-eval needs it in Wave-1B-1.

#### `.cargo/config.toml` (Wave-0a verify; no edits expected)

**Verify present** (Phase 1 W13 form):
```toml
[build]
rustflags = ["-Cllvm-args=-fp-contract=off"]

[target.'cfg(all())']
rustflags = ["-Cllvm-args=-fp-contract=off"]
```

**Adaptation notes:** ACC-05 inheritance from Phase 1. No edits expected. Wave-0a task: confirm both blocks are present.

---

### Wave-1A xtask binaries

#### `xtask/src/bin/regen_registry.rs` (NEW)

**Analog:** `xtask/src/bin/regen_ad_fixtures.rs` (whole file, lines 1-316)

**Project-root + manifest pattern** (regen_ad_fixtures.rs lines 45-54):
```rust
fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via `cargo run -p xtask --bin regen-registry`")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory — unexpected layout")?
        .to_path_buf();
    Ok(root)
}
```

**Compile-driver pattern** (regen_ad_fixtures.rs lines 56-81 — copy directly, swap `xcfun-master/external/upstream/taylor` for `xcfun-master/src` includes):
```rust
fn compile_extractor(xcfun_root: &Path, src: &Path, exe: &Path) -> Result<()> {
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".into());
    let status = Command::new(&compiler)
        .args([
            "-std=c++17",
            "-O2",
            "-fno-fast-math",
            "-ffp-contract=off",
            "-I",
        ])
        .arg(xcfun_root.join("api"))
        .arg("-I").arg(xcfun_root.join("src"))
        .arg("-I").arg(xcfun_root.join("src/functionals"))
        .arg(src)
        .arg("-o")
        .arg(exe)
        .status()
        .with_context(|| format!("failed to spawn C++ compiler ({})", compiler))?;
    if !status.success() {
        bail!("compiling extractor.cpp failed (compiler {}, exit {:?})", compiler, status.code());
    }
    Ok(())
}
```

**SHA-256 stamp pattern** (regen_ad_fixtures.rs lines 165-176 — copy verbatim):
```rust
fn header_sha256(xcfun_root: &Path, files: &[&str]) -> Result<String> {
    let mut hasher = Sha256::new();
    for fname in files {
        let path = xcfun_root.join(fname);
        let contents = fs::read(&path).with_context(|| format!("read xcfun-master file {:?}", path))?;
        hasher.update(&contents);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
```

**`--check` mode shape** (NEW — RESEARCH §"CORE-10 Extractor Recommendation" workflow):
1. Regenerate `*.rs` to a temp dir.
2. Compute SHA-256 of each generated file.
3. Diff against committed `crates/xcfun-core/src/registry/generated/*.rs.sha256`.
4. Non-zero diff → exit 2 with message pointing at `cargo run -p xtask --bin regen-registry`.

**Adaptation notes:**
- Replace bincode partition (regen_ad_fixtures.rs lines 232-282) with .rs file emission: parse extractor's JSONL stdout, format Rust source for `FUNCTIONAL_DESCRIPTORS.rs`, `VARS_TABLE.rs`, `ALIASES.rs`.
- Add `--check` flag (clap or manual argv parse — keep simple per `xtask/src/main.rs` precedent).
- xtask Cargo.toml: add `[[bin]] name = "regen-registry" path = "src/bin/regen_registry.rs"`.

**Cross-file dependencies:** writes to `crates/xcfun-core/src/registry/generated/*.rs` (consumed by xcfun-core lib.rs); reads from `xtask/assets/regen_registry/extractor.cpp`.

#### `xtask/assets/regen_registry/extractor.cpp` (NEW, ~150 LoC)

**Analog:** RESEARCH §"CORE-10 Extractor Recommendation" skeleton + `xcfun-master/src/functionals/slaterx.cpp:18-37` (FUNCTIONAL macro shape) + `xcfun-master/src/functional.hpp:20-28` (macro definition)

**Macro-payload shape to extract** (slaterx.cpp:18-37):
```cpp
FUNCTIONAL(XC_SLATERX) = {
    "Slater LDA exchange",            // short_desc
    "LDA Exchange functional\n…",     // long_desc
    XC_DENSITY,                        // depends bitmask
    ENERGY_FUNCTION(slaterx) XC_A_B,   // 7 fp{0..6} pointers + test_vars
    XC_PARTIAL_DERIVATIVES,            // test_mode
    2,                                 // test_order
    1e-11,                             // test_threshold
    {0.39E+02, 0.38E+02},              // test_in
    {-0.241948147838E+03, … }};        // test_out (length depends on test_outlen formula)
```

**JSONL output shape per RESEARCH §"CORE-10 Extractor Recommendation":**
```jsonl
{"id":"XC_SLATERX","short_desc":"Slater LDA exchange","long_desc":"…","depends":1,"test_vars":"XC_A_B","test_mode":"XC_PARTIAL_DERIVATIVES","test_order":2,"test_threshold":1e-11,"test_in":[39.0,38.0],"test_out":[-241.948,-4.207,-4.171,-0.036,0.0,-0.037]}
```

**Regex** (RESEARCH §"CORE-10 Extractor Recommendation"):
```regex
^FUNCTIONAL\(([A-Z_0-9]+)\)\s*=\s*\{(.*?)\};
```

**Adaptation notes:**
- Walk `xcfun-master/src/functionals/*.cpp`, regex-match FUNCTIONAL macros, emit JSONL on stdout.
- For 67 non-LDA functionals: emit a stub record with `test_in: null, test_out: null`.
- For VARS_TABLE: parse `xcfun-master/src/xcint.cpp:93-135` (the `xcint_vars[]` array). Different parse target — separate function.
- For ALIASES (Phase 2): emit empty array `[]` (no LDA-only aliases).
- Strip `// …` line comments before regex match to handle stray comments inside macro args.

#### `xtask/src/bin/check_no_mul_add.rs` (NEW)

**Analog:** `xtask/src/bin/check_no_fma.rs` (whole file, lines 1-218)

**Project-root pattern** (check_no_fma.rs lines 75-83 — copy verbatim).

**Source-file scan pattern** (NEW — different from check_no_fma.rs's asm-scan):
```rust
// Scan crates/xcfun-eval/src/functionals/**/*.rs (per D-13: target adjusted from xcfun-core to xcfun-eval per D-04)
// for `\.mul_add\s*\(` regex matches. Strip `//` line comments before match.
//
// Pattern from check_no_fma.rs::scan_asm_file (lines 178-217) but adapted:
//   - input is .rs source not .s asm
//   - violation marker is `\.mul_add\s*\(` not FORBIDDEN_MNEMONICS
//   - exit-2 message points at ACC-06 + design 07 §3 (algorithmic-identity rule)

fn main() -> Result<()> {
    let root = project_root()?;
    let target_glob = root.join("crates/xcfun-eval/src/functionals");
    // walkdir over .rs files; for each, grep for /\.mul_add\s*\(/ outside `//` comments.
    // Print violations in `path:line: <line>` form; exit 2 if any found.
    Ok(())
}
```

**Forbidden pattern** (RESEARCH §"Specific Ideas" + D-13):
```regex
\.mul_add\s*\(
```
Strip `//` line comments before match. Glob target: `crates/xcfun-eval/src/functionals/**/*.rs`.

**Adaptation notes:**
- xtask Cargo.toml: add `[[bin]] name = "check-no-mul-add" path = "src/bin/check_no_mul_add.rs"`.
- Add `walkdir = "2"` to xtask deps (or use `std::fs::read_dir` recursion).
- ACC-06 wording in REQUIREMENTS.md says "xcfun-core/src/functionals/" — adjust comment in code to note D-04 redirection: target is `xcfun-eval/src/functionals/` because functional bodies live there.

#### `xtask/src/bin/check_no_anyhow.rs` (NEW)

**Analog:** `xtask/src/bin/check_no_fma.rs` (project_root + exit-2 idiom; new scan target)

**Pattern:** Walk `crates/*/Cargo.toml`. For each, parse the TOML (use `toml = "0.8"` — add to xtask deps). Assert `[dependencies]` table does NOT contain `anyhow` (`[dev-dependencies]` is allowed — that's where Phase 1 `xcfun-ad/Cargo.toml:18-26` already uses anyhow-via-workspace).

**Whitelist:** `validation/Cargo.toml`, `xtask/Cargo.toml`, `crates/*/benches/*.rs`, `crates/*/examples/*.rs` — all permit anyhow per D-14 + CLAUDE.md.

**Exit shape** (clone from check_no_fma.rs lines 154-168):
```rust
if !violations.is_empty() {
    eprintln!("\ncheck-no-anyhow: FAIL — anyhow found in library [dependencies]:");
    for v in &violations { eprintln!("  {}", v); }
    eprintln!("\nQG-01: anyhow is permitted only at app boundaries (validation/, xtask/, benches/, examples/).");
    std::process::exit(2);
}
```

#### `xtask/src/bin/check_boundaries.rs` (NEW basic version)

**Analog:** `xtask/src/bin/check_no_fma.rs::main` (subprocess Command pattern, lines 92-114)

**Pattern:** invoke `cargo metadata --format-version 1 --no-deps`, parse JSON via `serde_json::Value`, walk packages array. Assert:
- `xcfun-core` deps: only `thiserror`, `bitflags`.
- `xcfun-ad` deps: only `cubecl`, `cubecl-cpu` (optional).
- `xcfun-eval` deps: only `xcfun-core`, `xcfun-ad`, `cubecl`, `cubecl-cpu`, `thiserror`.
- `validation` deps: any (anyhow allowed).
- `xtask` deps: any (anyhow allowed).

**Subprocess pattern** (check_no_fma.rs lines 92-107 — copy and adapt):
```rust
let output = Command::new("cargo")
    .current_dir(&root)
    .args(["metadata", "--format-version", "1", "--no-deps"])
    .output()
    .context("spawning cargo metadata")?;
if !output.status.success() {
    bail!("cargo metadata failed with exit {:?}", output.status.code());
}
let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
// … walk metadata.packages, enforce per-crate dep rules …
```

#### `xtask/src/bin/check_cubecl_pin.rs` (NEW)

**Analog:** `xtask/src/bin/check_no_fma.rs` (subprocess Command + exit-2 pattern)

**Pattern:** invoke `cargo metadata --format-version 1`, parse JSON, find `cubecl` and `cubecl-cpu` packages, assert `version == "0.10.0-pre.3"` exact match (no semver tolerance — per CLAUDE.md "all four cubecl crates must move in lockstep").

**Adaptation notes:** QG-06 inheritance per D-10. Trivial check; ~50 LoC.

#### `xtask/src/bin/validate.rs` (NEW thin wrapper)

**Analog:** `xtask/src/main.rs` (argv-dispatch shape, lines 1-22)

**Pattern:** parse argv, forward to `cargo run -p validation --release -- <args>`. RESEARCH §"Phase Requirements → Test Map" SC #5 row gives the canonical CLI: `--backend cpu --order 2 --filter lda`.

```rust
fn main() -> anyhow::Result<()> {
    let root = project_root()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(["run", "-p", "validation", "--release", "--"])
        .args(&args)
        .status()
        .context("spawning cargo run -p validation")?;
    std::process::exit(status.code().unwrap_or(1));
}
```

---

### Wave-1A registry codegen output

#### `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs` (NEW)

**Analog:** `xcfun-master/src/functionals/slaterx.cpp:18-37` (FUNCTIONAL macro shape) + RESEARCH §"Registry Shape + Circular-Dep Resolution" (descriptor struct shape)

**Target shape** (RESEARCH §"Registry Shape + Circular-Dep Resolution" verbatim):
```rust
// AUTO-GENERATED by xtask regen-registry — DO NOT EDIT MANUALLY
// Source: xcfun-master/src/functionals/*.cpp via xtask/assets/regen_registry/extractor.cpp
//
// To regenerate: cargo run -p xtask --bin regen-registry
// To check for drift: cargo run -p xtask --bin regen-registry -- --check

use crate::{Dependency, FunctionalId, Mode, Vars};

pub struct FunctionalDescriptor {
    pub id: FunctionalId,
    pub name: &'static str,
    pub short_description: &'static str,
    pub long_description: &'static str,
    pub depends: Dependency,
    pub test_vars: Option<Vars>,
    pub test_mode: Option<Mode>,
    pub test_order: Option<u32>,
    pub test_threshold: Option<f64>,
    pub test_in: Option<&'static [f64]>,
    pub test_out: Option<&'static [f64]>,
    pub test_outlen: u32,
}

impl FunctionalDescriptor {
    pub const fn stub(id: FunctionalId, name: &'static str, depends: Dependency) -> Self {
        Self { id, name, short_description: "(stub — not implemented in Phase 2)",
               long_description: "", depends,
               test_vars: None, test_mode: None, test_order: None,
               test_threshold: None, test_in: None, test_out: None, test_outlen: 0 }
    }
}

// Static slice arrays for test_in/test_out (rodata)
static SLATERX_TEST_IN: [f64; 2] = [0.39E+02, 0.38E+02];
static SLATERX_TEST_OUT: [f64; 6] = [-0.241948147838E+03, -0.420747936684E+01, -0.417120618800E+01,
                                     -0.359613621097E-01, 0.0, -0.365895279649E-01];
// … similar for the other 10 LDA functionals …

pub static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; 78] = [
    FunctionalDescriptor {
        id: FunctionalId::XC_SLATERX,
        name: "XC_SLATERX",
        short_description: "Slater LDA exchange",
        long_description: "LDA Exchange functional\nP.A.M. Dirac…",
        depends: Dependency::DENSITY,
        test_vars: Some(Vars::A_B),
        test_mode: Some(Mode::PartialDerivatives),
        test_order: Some(2),
        test_threshold: Some(1e-11),                      // from slaterx.cpp:30 verbatim
        test_in: Some(&SLATERX_TEST_IN),
        test_out: Some(&SLATERX_TEST_OUT),
        test_outlen: 6,
    },
    // … 10 more LDA-populated entries …
    // 67 stub entries:
    FunctionalDescriptor::stub(FunctionalId::XC_PBEX, "XC_PBEX", Dependency::DENSITY.union(Dependency::GRADIENT)),
    // …
];
```

**Adaptation notes:**
- Per D-12: 11 LDA entries fully populated, 67 stubs.
- LDA-08 (LDAERFC_JT): `test_in/test_out: None` — upstream provides no test data (line 64 of `ldaerfc_jt.cpp` ends at `ENERGY_FUNCTION(ldaerfc_jt)` without test arrays). Tier-1 self-test SKIPS this functional via the `desc.test_in.is_some()` filter.
- LDA-09 part 1 (TFK): test_threshold = 1e-5 per `tfk.cpp:33`.
- LDA-06/07 (LDAERFX/LDAERFC): test_threshold = 1e-7 per `ldaerfx.cpp:66` and `ldaerfc.cpp:124` (D-24 override aligned with upstream).
- LDA-09 part 2 (TW), LDA-10 (VWK): no upstream test data — `test_in/test_out: None`.
- Hash committed alongside as `FUNCTIONAL_DESCRIPTORS.rs.sha256` (regen-registry --check gate).

#### `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` (NEW)

**Analog:** `xcfun-master/src/xcint.cpp:93-135` (verbatim port)

**C++ source excerpt** (xcint.cpp:93-101):
```cpp
vars_data xcint_vars[XC_NR_VARS] = {
    {"XC_A", 1, XC_DENSITY},
    {"XC_N", 1, XC_DENSITY},
    {"XC_A_B", 2, XC_DENSITY},
    {"XC_N_S", 2, XC_DENSITY},
    {"XC_A_GAA", 2, XC_DENSITY | XC_GRADIENT},
    {"XC_N_GNN", 2, XC_DENSITY | XC_GRADIENT},
    {"XC_A_B_GAA_GAB_GBB", 5, XC_DENSITY | XC_GRADIENT},
    /* … 24 more rows … */
};
```

**Target shape:**
```rust
// AUTO-GENERATED by xtask regen-registry — DO NOT EDIT MANUALLY
use crate::Dependency;

#[repr(C)]
pub struct VarsRow {
    pub symbol: &'static str,
    pub len: u8,
    pub provides: Dependency,
}

pub static VARS_TABLE: [VarsRow; 31] = [
    VarsRow { symbol: "XC_A",   len: 1, provides: Dependency::DENSITY },
    VarsRow { symbol: "XC_N",   len: 1, provides: Dependency::DENSITY },
    VarsRow { symbol: "XC_A_B", len: 2, provides: Dependency::DENSITY },
    /* … 28 more rows, verbatim port of xcint.cpp:93-135 … */
];
```

**Adaptation notes:**
- `#[repr(C)]` per RESEARCH §"Specific Ideas" line for future CAPI stability.
- 31 rows total, all populated (CORE-09 complete in Phase 2).
- A unit test cross-checks `VARS_TABLE[i].len as usize == Vars::from_index(i).input_len()` for all 31 rows.

#### `crates/xcfun-core/src/registry/generated/ALIASES.rs` (NEW empty in Phase 2)

**Analog:** RESEARCH §"Open Questions for Planner" question 7

**Target shape:**
```rust
// AUTO-GENERATED by xtask regen-registry — DO NOT EDIT MANUALLY
//
// Phase 2: empty slice (no LDA-only aliases exist; CORE-08 deferred to Phase 4).
// Phase 4 re-runs regen-registry; xtask will emit the 46-entry alias table here.

pub struct Alias {
    pub name: &'static str,
    pub description: &'static str,
    pub components: &'static [(&'static str, f64)],
}

pub static ALIASES: &[Alias] = &[];
```

---

### Wave-1B + Wave-1C xcfun-eval crate

#### `crates/xcfun-eval/Cargo.toml` (NEW)

**Analog:** `crates/xcfun-ad/Cargo.toml` (whole file, lines 1-37)

**Target shape:**
```toml
[package]
name = "xcfun-eval"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
description = "cubecl launcher + functional bodies for xcfun_rs"
license = "MPL-2.0"

[features]
default = ["cpu"]
cpu = ["dep:cubecl-cpu"]   # mirror xcfun-ad pattern
testing = []                # gates pub mod for_tests

[dependencies]
xcfun-core = { path = "../xcfun-core" }
xcfun-ad = { path = "../xcfun-ad" }
cubecl = { workspace = true }
cubecl-cpu = { workspace = true, optional = true }
thiserror = { workspace = true }

[dev-dependencies]
xcfun-ad = { path = "../xcfun-ad", features = ["testing"] }
approx = { workspace = true }
rstest = { workspace = true }
```

**Adaptation notes:** verbatim copy of xcfun-ad/Cargo.toml shape, swap deps. Note dual `xcfun-ad` declaration (deps + dev-deps with `testing` feature) is the canonical Rust pattern for using `for_tests::cpu_client()` in tests.

#### `crates/xcfun-eval/src/lib.rs` (NEW)

**Analog:** `crates/xcfun-ad/src/lib.rs` (whole file, lines 1-22)

**Target shape:**
```rust
//! `xcfun-eval` — cubecl launcher + functional bodies for xcfun_rs.
//!
//! This crate hosts the `#[cube] fn` bodies for the 11 LDA functionals
//! (Phase 2), the `DensVarsDev<F>` `#[derive(CubeType, CubeLaunch)]` type,
//! the `build_densvars` dispatcher, and the minimal `Functional` struct +
//! `eval` entry point used by tier-1 self-tests + tier-2 parity harness.
//! See `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md`
//! for the 25 locked design decisions driving this crate.
#![forbid(unsafe_code)]

pub mod density_vars;
pub mod dispatch;
pub mod functional;
pub mod functionals;

#[cfg(feature = "testing")]
pub mod for_tests;

pub use functional::Functional;
```

#### `crates/xcfun-eval/src/for_tests.rs` (NEW — verbatim copy)

**Analog:** `crates/xcfun-ad/src/for_tests/cpu_client.rs` (whole file, lines 1-32)

**Verbatim copy:**
```rust
use cubecl::prelude::*;
use cubecl_cpu::{CpuDevice, CpuRuntime};
use std::sync::OnceLock;

pub type CpuClient = ComputeClient<CpuRuntime>;

static CPU_CLIENT: OnceLock<CpuClient> = OnceLock::new();

pub fn cpu_client() -> &'static CpuClient {
    CPU_CLIENT.get_or_init(|| {
        let device = CpuDevice;
        CpuRuntime::client(&device)
    })
}
```

**Adaptation notes:** byte-identical to Phase 1 `cpu_client.rs`. xcfun-eval and xcfun-ad each have their own `OnceLock<CpuClient>` (no cross-crate sharing — that's fine, both clients point at the same CPU device).

#### `crates/xcfun-eval/src/density_vars.rs` (NEW — first `#[derive(CubeType, CubeLaunch)]` site)

**Analog:** RESEARCH §"D-02 cubecl Nesting Decision" Pattern A (verbatim) + `xcfun-master/src/densvars.hpp:223-244` (field set)

**C++ field set** (densvars.hpp:223-244 — the 22 fields):
```cpp
T a{0}, b{0}, gaa{0}, gab{0}, gbb{0};
T n{0}, s{0};                  // n = a+b, s = a-b
T gnn{0}, gns{0}, gss{0};      // gradient-square combos
T tau{0}, taua{0}, taub{0};    // kinetic
T lapa{0}, lapb{0};            // Laplacian
T zeta{0}, r_s{0};             // s/n, (3/4pi)^(1/3) * n^(-1/3)
T n_m13{0};                    // pow(n, -1/3)
T a_43{0}, b_43{0};            // pow(a, 4/3), pow(b, 4/3)
T jpaa{0}, jpbb{0};            // current density
```

**Target Rust shape (Pattern A from RESEARCH §"D-02" verbatim):**
```rust
//! Device-side densvars container — `#[derive(CubeType, CubeLaunch)]` struct
//! holding 22 named `Array<F>` fields. 1:1 port of
//! `xcfun-master/src/densvars.hpp:223-244` field set per CORE-05 + D-02.

use cubecl::prelude::*;

#[derive(CubeType, CubeLaunch)]
pub struct DensVarsDev<F: Float> {
    pub a: Array<F>,
    pub b: Array<F>,
    pub gaa: Array<F>,
    pub gab: Array<F>,
    pub gbb: Array<F>,
    pub n: Array<F>,
    pub s: Array<F>,
    pub gnn: Array<F>,
    pub gns: Array<F>,
    pub gss: Array<F>,
    pub tau: Array<F>,
    pub taua: Array<F>,
    pub taub: Array<F>,
    pub lapa: Array<F>,
    pub lapb: Array<F>,
    pub zeta: Array<F>,
    pub r_s: Array<F>,
    pub n_m13: Array<F>,
    pub a_43: Array<F>,
    pub b_43: Array<F>,
    pub jpaa: Array<F>,
    pub jpbb: Array<F>,
}
```

**Adaptation notes:**
- Field NAMES are byte-identical to C++ `densvars<T>` for code-review parity (Pitfall P3 prevention).
- Field ORDER matches the C++ struct declaration order (densvars.hpp:223-244).
- Each field is a length-`(1 << N)` `Array<F>` allocated by the launcher (`DensVarsDevLaunch::new(…)` per cubecl 0.10-pre.3 idiom).
- 22 fields — matches RESEARCH §"D-02" Pattern A field count exactly. 7 raw inputs (a, b, gaa, gab, gbb, n, s) + 15 derived. Note: design 02 §5 mentioned "29 fields" — RESEARCH §"Phase Requirements" CORE-05 row clarifies "22 actual + 7 future-use raw input slots reserved". Phase 2 ships 22 named fields; Phase 3+ may add more.
- Phase 2 N range = 0..=2 per D-23 (orders 0..=2 for PartialDerivatives). cubecl monomorphizes at launch site.

**Cross-file dependencies:** consumed by `density_vars/build.rs`, `density_vars/regularize.rs`, every `functionals/lda/*.rs` kernel.

#### `crates/xcfun-eval/src/density_vars/build.rs` (NEW — `build_densvars` dispatcher + 5 variant arms)

**Analog:** RESEARCH §"build_densvars Pattern" (verbatim skeleton) + `xcfun-master/src/densvars.hpp:35-218` (port target with C-fallthrough flattened)

**C++ source for `XC_A_B` arm** (densvars.hpp:65-72):
```cpp
case XC_A_B:
    a = d[0];
    regularize(a);
    b = d[1];
    regularize(b);
    n = a + b;
    s = a - b;
    break;
```

**Rust port** (RESEARCH §"build_densvars Pattern" verbatim):
```rust
#[cube]
fn build_xc_a_b<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    use xcfun_ad::ctaylor::{ctaylor_from_scalar, ctaylor_add, ctaylor_sub};
    use crate::density_vars::regularize::regularize;

    // a = d[0]; regularize(a);
    ctaylor_from_scalar::<F>(input[0], &mut out.a, n);
    regularize::<F>(&mut out.a, n);
    // b = d[1]; regularize(b);
    ctaylor_from_scalar::<F>(input[1], &mut out.b, n);
    regularize::<F>(&mut out.b, n);
    // n = a + b; s = a - b;
    ctaylor_add::<F>(&out.a, &out.b, &mut out.n, n);
    ctaylor_sub::<F>(&out.a, &out.b, &mut out.s, n);
}
```

**C++ source for `XC_A_B_GAA_GAB_GBB` arm with fallthrough** (densvars.hpp:58-72):
```cpp
case XC_A_B_GAA_GAB_GBB:
    gaa = d[2]; gab = d[3]; gbb = d[4];
    gnn = gaa + 2 * gab + gbb;
    gss = gaa - 2 * gab + gbb;
    gns = gaa - gbb;
case XC_A_B:                  // <-- C-style fallthrough
    a = d[0];
    regularize(a);
    /* ... */
    break;
```

**Rust port — explicit chain instead of fallthrough** (CORE-05 + Pitfall P5):
```rust
#[cube]
fn build_xc_a_b_gaa_gab_gbb<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] n: u32,
) {
    use xcfun_ad::ctaylor::{ctaylor_from_scalar, ctaylor_add, ctaylor_sub, ctaylor_scalar_mul};

    // gaa = d[2]; gab = d[3]; gbb = d[4];
    ctaylor_from_scalar::<F>(input[2], &mut out.gaa, n);
    ctaylor_from_scalar::<F>(input[3], &mut out.gab, n);
    ctaylor_from_scalar::<F>(input[4], &mut out.gbb, n);
    // gnn = gaa + 2*gab + gbb (left-to-right; ACC-06 forbids mul_add)
    let mut t1 = Array::<F>::new(comptime!((1u32 << n) as usize));
    let mut t2 = Array::<F>::new(comptime!((1u32 << n) as usize));
    ctaylor_scalar_mul::<F>(&out.gab, F::new(2.0), &mut t1, n);
    ctaylor_add::<F>(&out.gaa, &t1, &mut t2, n);
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gnn, n);
    // gss = gaa - 2*gab + gbb
    ctaylor_sub::<F>(&out.gaa, &t1, &mut t2, n);
    ctaylor_add::<F>(&t2, &out.gbb, &mut out.gss, n);
    // gns = gaa - gbb
    ctaylor_sub::<F>(&out.gaa, &out.gbb, &mut out.gns, n);
    // EXPLICIT chain to XC_A_B builder (replaces C fallthrough)
    build_xc_a_b::<F>(input, out, n);
}
```

**Top-level dispatcher** (RESEARCH §"build_densvars Pattern"):
```rust
#[cube]
pub fn build_densvars<F: Float>(
    input: &Array<F>,
    out: &mut DensVarsDev<F>,
    #[comptime] vars: u32,         // Vars discriminant as comptime
    #[comptime] n: u32,
) {
    // First: zero ALL 22 fields (defensive — cubecl Array::new doesn't zero)
    use xcfun_ad::ctaylor::ctaylor_zero;
    ctaylor_zero::<F>(&mut out.a, n);
    /* … 21 more ctaylor_zero calls … */

    // Variant dispatch via comptime if-chain (Phase 2 = 5 arms)
    if comptime!(vars == 2) {                        // XC_A_B
        build_xc_a_b::<F>(input, out, n);
    } else if comptime!(vars == 6) {                 // XC_A_B_GAA_GAB_GBB
        build_xc_a_b_gaa_gab_gbb::<F>(input, out, n);
    }
    // 0 (XC_A), 1 (XC_N), 3 (XC_N_S) wired similarly.
    // 26 unsupported arms = host-side launcher returns XcError::InvalidVars BEFORE launch.

    // Derived fields (densvars.hpp:213-217) — same 4 lines for every variant
    use xcfun_ad::math::{ctaylor_pow, ctaylor_reciprocal};
    use xcfun_ad::ctaylor::{ctaylor_scalar_mul};
    // zeta = s / n  →  zeta = s * (1/n)
    let mut inv_n = Array::<F>::new(comptime!((1u32 << n) as usize));
    ctaylor_reciprocal::<F>(&out.n, &mut inv_n, n);
    use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
    ctaylor_mul::<F>(&out.s, &inv_n, &mut out.zeta, n);
    // n_m13 = pow(n, -1/3)
    ctaylor_pow::<F>(&out.n, F::new(-1.0/3.0), &mut out.n_m13, n);
    // a_43 = pow(a, 4/3); b_43 = pow(b, 4/3)
    ctaylor_pow::<F>(&out.a, F::new(4.0/3.0), &mut out.a_43, n);
    ctaylor_pow::<F>(&out.b, F::new(4.0/3.0), &mut out.b_43, n);
    // r_s = (3/(4*PI))^(1/3) * n_m13 — composed with constant prefactor
    let prefactor = F::new(0.6203504908994001);  // RS_PREFACTOR from xcfun-core::constants
    ctaylor_scalar_mul::<F>(&out.n_m13, prefactor, &mut out.r_s, n);
}
```

**Per-variant chain table** (RESEARCH §"build_densvars Pattern"):

| Vars | Phase 2 functionals | Helper fn | Chains to |
|------|---------------------|-----------|-----------|
| `XC_A_B` (=2) | SLATERX, VWN3C, VWN5C, PW92C, PZ81C, LDAERFX, LDAERFC, TFK | `build_xc_a_b` | (none) |
| `XC_A_B_GAA_GAB_GBB` (=6) | TW, VWK | `build_xc_a_b_gaa_gab_gbb` | `build_xc_a_b` |
| `XC_A` (=0), `XC_N` (=1), `XC_N_S` (=3) | (none in Phase 2 LDA tests) | seeded for Phase 3+ | (none) |

**Adaptation notes:**
- Pitfall PHASE2-D: TW + VWK use `XC_A_B_GAA_GAB_GBB`, NOT `XC_A_B`. Wave-1B handles 8 pure-density LDAs, Wave-1C extends with this builder arm.
- Defensive zero-init: cubecl `Array::new` does NOT guarantee zero-init (RESEARCH §"build_densvars Pattern"). Every field is explicitly zeroed via `ctaylor_zero` before the variant arm.
- LDAERFC_JT (LDA-08) uses LDAERFC formula but inherits same `XC_A_B` builder.
- The 26 non-Phase-2 variant arms are NOT included as comptime arms — host-side launcher returns `XcError::InvalidVars` for unsupported `vars` values BEFORE launch (cleaner than comptime panic).

**Cross-file dependencies:** depends on Phase 1 `xcfun-ad::ctaylor::{ctaylor_from_scalar, ctaylor_add, ctaylor_sub, ctaylor_scalar_mul, ctaylor_zero}` + `xcfun-ad::ctaylor_rec::mul::ctaylor_mul` + `xcfun-ad::math::{ctaylor_pow, ctaylor_reciprocal}` + `crate::density_vars::regularize`. Consumed by every `functionals/lda/*.rs` kernel + `dispatch::dispatch_kernel`.

#### `crates/xcfun-eval/src/density_vars/regularize.rs` (NEW)

**Analog:** `xcfun-master/src/densvars.hpp:22-25` + Phase 1 `crates/xcfun-ad/src/ctaylor.rs:30-38` (`ctaylor_zero` cubecl idiom)

**C++ source** (densvars.hpp:22-25):
```cpp
template <typename T, int N> void regularize(ctaylor<T, N> & x) {
    if (x < xcfun::XCFUN_TINY_DENSITY)
        x.set(0, xcfun::XCFUN_TINY_DENSITY);   // sets only c[0] (CNST coefficient)
}
```

**Phase 1 cubecl idiom** (ctaylor.rs:30-38, `ctaylor_zero`):
```rust
#[cube]
pub fn ctaylor_zero<F: Float>(out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!(1_u32 << n);
    let zero = F::new(0.0);
    #[unroll]
    for i in 0..size {
        out[i as usize] = zero;
    }
}
```

**Target Rust shape** (D-22 + CORE-06):
```rust
//! `regularize` — clamp `x[CNST]` to `>= XCFUN_TINY_DENSITY`. Higher-order
//! coefficients (`x[VAR0..]`) preserved per D-22.
//!
//! Port of `xcfun-master/src/densvars.hpp:22-25`:
//!
//! ```cpp
//! template <typename T, int N> void regularize(ctaylor<T, N> & x) {
//!     if (x < xcfun::XCFUN_TINY_DENSITY)
//!         x.set(0, xcfun::XCFUN_TINY_DENSITY);
//! }
//! ```
//!
//! Verified by unit test (CORE-06) — `regularize_preserves_derivatives`.

use cubecl::prelude::*;

const TINY_DENSITY: f64 = 1e-14;     // matches xcfun::XCFUN_TINY_DENSITY (config.hpp:22)

#[cube]
pub fn regularize<F: Float>(x: &mut Array<F>, #[comptime] _n: u32) {
    // C++ `if (x < XCFUN_TINY_DENSITY)` reads x.c[0]; cubecl Float comparison
    // is a primitive so this lowers cleanly.
    let tiny = F::new(TINY_DENSITY as f32);   // F::new takes f32 per cubecl 0.10-pre.3
    if x[0] < tiny {
        x[0] = tiny;
    }
    // Higher coefficients x[1..] left unchanged — matches C++ set(0, …) which
    // mutates only c[0]. CORE-06 + D-22 contract.
}
```

**Adaptation notes:**
- `F::new(TINY_DENSITY as f32)` — cubecl's `F::new` takes `f32` per Phase 1 `ctaylor.rs:33`. The f32 literal converts back to f64 inside the kernel (OK at the magnitudes involved — 1e-14 is exactly representable in both formats; loss is bounded at 1e-22 absolute).
- Unit test in `tests/densvars_field_parity.rs` or co-located: seed CTaylor<f64, 2> with `[1e-15, 0.5, 0.7, 0.0]`, run regularize, assert `[1e-14, 0.5, 0.7, 0.0]`.

**Cross-file dependencies:** consumed by `density_vars/build.rs::build_xc_a_b` (and 4 other variant arms).

#### `crates/xcfun-eval/src/functional.rs` (NEW)

**Analog:** RESEARCH §"Registry Shape + Circular-Dep Resolution" (Functional minimal slice) + `crates/xcfun-core/src/traits.rs:34-53` (pre-pivot trait surface, being deleted) + Phase 1 `for_tests/raw_eval_scalar.rs` (1-thread launch shape)

**Target shape (D-21 minimal slice):**
```rust
//! `Functional` struct + `eval` entry point. Phase 2 minimal slice per D-21:
//! carries weights/vars/mode/order, dispatches via cubecl-cpu launches over the
//! registry. Phase 5 (RS-01..10) re-exports through `xcfun-rs::Functional` with
//! the full public API surface.

use xcfun_core::{FunctionalId, Mode, Vars, XcError};

pub struct Functional {
    pub weights: &'static [(FunctionalId, f64)],
    pub vars: Vars,
    pub mode: Mode,
    pub order: u32,
}

impl Functional {
    /// Compute the partial derivatives of the weighted-sum functional at `input`
    /// and write into `output`. Output length = `taylorlen(VARS_TABLE[vars].len, order)`.
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // 1. Validate input/output lengths against VARS_TABLE + taylorlen.
        // 2. Per-(FunctionalId, weight) pair: launch dispatch_kernel as a
        //    1-thread cubecl-cpu kernel (Phase 1 raw_eval_scalar pattern).
        // 3. Sum weighted outputs into `output`.
        // 4. Return Ok(()) or XcError::{InvalidVars, InvalidMode, InvalidOrder, …}.
        //
        // Phase 2 supports only Mode::PartialDerivatives at orders 0..=2 (D-23).
        // Mode::Potential / Mode::Contracted return XcError::InvalidMode.
        // Order > 2 returns XcError::InvalidOrder.
        todo!("Wave-1B-5 implementation per RESEARCH §Mode::PartialDerivatives Output Layout")
    }
}
```

**Adaptation notes:**
- `weights: &'static [(FunctionalId, f64)]` is the simplest representation; downstream Phase 5 may swap for `Vec<(FunctionalId, f64)>` if dynamic functionals are needed.
- The `eval` body's launch pattern mirrors Phase 1 `raw_eval_scalar` (`for_tests/raw_eval_scalar.rs:40-56`): create input/output handles, call `dispatch_kernel::launch_unchecked`, read output back.
- Output element layout per RESEARCH §"Mode::PartialDerivatives Output Layout":
  - Order 0: `[energy]`
  - Order 1, inlen=2: `[energy, ∂/∂a, ∂/∂b]`
  - Order 2, inlen=2: `[energy, ∂/∂a, ∂/∂b, ∂²/∂a², ∂²/∂a∂b, ∂²/∂b²]`
- For multi-functional weighted sums (e.g., `lda` alias = `slaterx + vwn5c`), the `eval` calls dispatch_kernel once per `(FunctionalId, weight)` pair and accumulates.

**Cross-file dependencies:** depends on `xcfun-core::{FunctionalId, Mode, Vars, XcError, VARS_TABLE, taylorlen}`, `crate::dispatch::dispatch_kernel`, `crate::for_tests::cpu_client` (for cubecl-cpu launch).

#### `crates/xcfun-eval/src/dispatch.rs` (NEW)

**Analog:** RESEARCH §"Registry Shape + Circular-Dep Resolution" (verbatim skeleton)

**Target shape:**
```rust
//! Functional dispatcher — match-on-FunctionalId resolves the registry
//! circular-dep (xcfun-core has no fp-table; xcfun-eval owns dispatch).
//!
//! See RESEARCH §"Registry Shape + Circular-Dep Resolution" for rationale.

use cubecl::prelude::*;
use xcfun_core::{FunctionalId, XcError};

use crate::density_vars::DensVarsDev;
use crate::functionals::lda;

#[cube]
pub fn dispatch_kernel<F: Float, const N: u32>(
    #[comptime] id: u32,           // FunctionalId as u32 (comptime)
    d: &DensVarsDev<F>,
    out: &mut Array<F>,            // CTaylor<F, N> as length-(1<<N) Array
) {
    if comptime!(id == 0) {           // XC_SLATERX
        lda::slaterx::slaterx_kernel::<F, N>(d, out);
    } else if comptime!(id == 2) {    // XC_VWN3C
        lda::vwn3c::vwn3c_kernel::<F, N>(d, out);
    } else if comptime!(id == 3) {    // XC_VWN5C
        lda::vwn5c::vwn5c_kernel::<F, N>(d, out);
    } else if comptime!(id == 28) {   // XC_PW92C
        lda::pw92c::pw92c_kernel::<F, N>(d, out);
    } else if comptime!(id == 54) {   // XC_PZ81C
        lda::pz81c::pz81c_kernel::<F, N>(d, out);
    } else if comptime!(id == 13) {   // XC_LDAERFX
        lda::ldaerfx::ldaerfx_kernel::<F, N>(d, out);
    } else if comptime!(id == 14) {   // XC_LDAERFC
        lda::ldaerfc::ldaerfc_kernel::<F, N>(d, out);
    } else if comptime!(id == 15) {   // XC_LDAERFC_JT
        lda::ldaerfc_jt::ldaerfc_jt_kernel::<F, N>(d, out);
    } else if comptime!(id == 24) {   // XC_TFK
        lda::tfk::tfk_kernel::<F, N>(d, out);
    } else if comptime!(id == 25) {   // XC_TW
        lda::tw::tw_kernel::<F, N>(d, out);
    } else if comptime!(id == 58) {   // XC_VWK
        lda::vwk::vwk_kernel::<F, N>(d, out);
    }
    // Stubs (other 67 IDs): host-side launcher returns
    // XcError::NotConfigured BEFORE launch — they should never reach here.
}

/// Host-side guard: for stubs, refuse to launch. Called by Functional::eval
/// before invoking dispatch_kernel.
pub fn supports(id: FunctionalId) -> bool {
    matches!(id as u32, 0 | 2 | 3 | 13 | 14 | 15 | 24 | 25 | 28 | 54 | 58)
}
```

**Adaptation notes:**
- Discriminants must match the post-Wave-0e re-ordered FunctionalId enum (xcint historical insertion order from `list_of_functionals.hpp`). The id values shown above are placeholders — Wave-1B-6 cross-checks against the actual generated enum.
- Phase 2 LDA-only: 11 supported IDs. The 67 stubs return `XcError::NotConfigured` from `Functional::eval` BEFORE launch (host-side `supports()` check).
- `comptime!(id == K)` chain matches Phase 1 `ctaylor_powi` dispatcher pattern (`crates/xcfun-ad/src/math.rs:502-540`).

**Cross-file dependencies:** depends on every `crate::functionals::lda::*::<name>_kernel`. Consumed by `crate::functional::Functional::eval`.

#### `crates/xcfun-eval/src/functionals/lda/slaterx.rs` (NEW — LDA-01)

**Analog:** `xcfun-master/src/functionals/slaterx.cpp:18-37` + `xcfun-master/src/functionals/slater.hpp:19-21`

**C++ source** (slater.hpp:19-21):
```cpp
template <typename num> static num slaterx(const densvars<num> & d) {
    return (-xcfun_constants::c_slater) * (d.a_43 + d.b_43);
}
```

**Target Rust shape** (D-03 + D-17 kernel signature, exact algorithmic-identity port):
```rust
//! Slater LDA exchange functional.
//!
//! # Source
//! - `xcfun-master/src/functionals/slaterx.cpp` (FUNCTIONAL macro + test data)
//! - `xcfun-master/src/functionals/slater.hpp:19-21` (formula)
//!
//! # Formula
//! $$ E_x = -c_{\text{slater}} \cdot (a^{4/3} + b^{4/3}) $$
//! where $c_{\text{slater}} = (81/(32\pi))^{1/3} \approx 0.93052574$
//!
//! # Preconditions
//! - `d.a_43` and `d.b_43` populated by `build_densvars` (XC_A_B variant).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};

use crate::density_vars::DensVarsDev;

const C_SLATER: f64 = 0.9305257363491002;   // (81/(32*PI))^(1/3); from xcfun-core::constants

#[cube]
pub fn slaterx_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
) {
    // C++: return (-c_slater) * (d.a_43 + d.b_43);
    //
    // Operation order (left-to-right; ACC-06 forbids mul_add):
    //   1. tmp = a_43 + b_43          (ctaylor_add)
    //   2. out = (-c_slater) * tmp     (ctaylor_scalar_mul)
    let mut tmp = Array::<F>::new(comptime!((1u32 << N) as usize));
    ctaylor_add::<F>(&d.a_43, &d.b_43, &mut tmp, N);
    let neg_c_slater = F::new(-C_SLATER as f32);
    ctaylor_scalar_mul::<F>(&tmp, neg_c_slater, out, N);
}
```

**Adaptation notes:**
- Field accesses `d.a_43`, `d.b_43` use the per-field `Array<F>` from `DensVarsDev` (Pattern A from RESEARCH §"D-02").
- Operation order matches C++ exactly: `(a_43 + b_43)` first, then negate-scalar-multiply. ACC-06 forbids `mul_add`; the explicit `add → scalar_mul` chain has no fused multiply-add risk.
- Doc-comment header has 3 items (upstream source, formula in LaTeX, preconditions) per RESEARCH §"Specific Ideas" pattern matching Phase 1 `expand/*` modules.
- `F::new(-C_SLATER as f32)` — cubecl 0.10-pre.3's `F::new` takes f32. The f32 representation of -0.9305257363491002 is -0.9305257_f32, with relative loss ~1e-8. For a 1e-12 contract we need to verify this lowering is acceptable; if not, the alternative is `F::cast_from(-C_SLATER)` (cubecl pattern from Phase 1 `math.rs:476`).

**Cross-file dependencies:** depends on `xcfun-ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul}` (Phase 1 primitives) + `crate::density_vars::DensVarsDev`. Consumed by `crate::dispatch::dispatch_kernel` (XC_SLATERX arm).

#### `crates/xcfun-eval/src/functionals/lda/vwn3c.rs` (NEW — LDA-02)

**Analog:** `xcfun-master/src/functionals/vwn3.cpp:18-30` + `xcfun-master/src/functionals/vwn.hpp` (vwn3_eps helper)

**C++ source** (vwn3.cpp:18-20):
```cpp
template <typename num> static num vwn3c(const densvars<num> & d) {
    return d.n * vwn::vwn3_eps(d);
}
```

**Target Rust shape:**
```rust
//! VWN3 LDA correlation functional.

use cubecl::prelude::*;
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use crate::density_vars::DensVarsDev;

mod vwn_eps;   // Helper module with vwn3_eps and vwn5_eps `#[cube] fn`s
                // (1:1 port of vwn.hpp helpers)

#[cube]
pub fn vwn3c_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
) {
    // C++: return d.n * vwn::vwn3_eps(d);
    let mut eps = Array::<F>::new(comptime!((1u32 << N) as usize));
    vwn_eps::vwn3_eps::<F, N>(d, &mut eps);
    ctaylor_mul::<F>(&d.n, &eps, out, N);
}
```

**Adaptation notes:**
- `vwn.hpp` helper functions (vwn3_eps, vwn5_eps) get their own `#[cube] fn` ports in a `vwn_eps.rs` (or `mod vwn_eps;`) module. Both LDA-02 and LDA-03 share these helpers.
- Operation order: helper computes eps first, then `ctaylor_mul(d.n, eps, out)`.
- vwn3_eps and vwn5_eps differ only in spin-interpolation constants — the helpers can be parameterized.

#### `crates/xcfun-eval/src/functionals/lda/ldaerfx.rs` (NEW — LDA-06)

**Analog:** `xcfun-master/src/functionals/ldaerfx.cpp:24-73` (full body)

**C++ source** (ldaerfx.cpp:24-48 — `esrx_ldaerfspin`):
```cpp
template <typename num> static num esrx_ldaerfspin(const num & na, parameter mu) {
    const parameter ckf = 3.093667726280136;
    const num & rhoa = 2 * na;                       // spin-scaling
    num akf = ckf * pow(rhoa, 1.0 / 3.0);
    num a = mu / (2 * akf);
    num a2 = a * a;
    num a3 = a2 * a;
    if (a < 1e-9)
        return -3.0 / 8.0 * rhoa * pow(24.0 * rhoa / M_PI, 1.0 / 3.0);
    else if (a < 100)
        return -(rhoa * pow(24.0 * rhoa / M_PI, 1.0 / 3.0)) *
               (3.0 / 8.0 - a * (sqrt(M_PI) * erf(0.5 / a) +
                                 (2 * a - 4 * a3) * exp(-0.25 / a2) - 3.0 * a + 4 * a3));
    else if (a < 1e9)
        return -(rhoa * pow(24.0 * rhoa / M_PI, 1.0 / 3.0)) / (96.0 * a2);
    else
        return 0;
}

template <typename num> static num lda_erfx(const densvars<num> & d) {
    double mu = d.get_param(XC_RANGESEP_MU);
    return 0.5 * (esrx_ldaerfspin(d.a, mu) + esrx_ldaerfspin(d.b, mu));
}
```

**Target Rust shape:**
```rust
//! Short-range spin-dependent LDA exchange functional (range-separated).
//!
//! # Source
//! - `xcfun-master/src/functionals/ldaerfx.cpp` (full body)
//!
//! # Tier-2 tolerance override per D-24
//! Upstream `test_threshold` is 1e-7 (ldaerfx.cpp:66). cubecl 0.10-pre.3
//! `Float::erf` polyfill (~1.3e-8 ULP) propagates to ~2e-8 final-output
//! rel-error vs C++ libm `erf`. We match upstream's threshold to remain
//! within published xcfun spec; tier-2 reports this as a documented
//! divergence, NOT silent widening (RESEARCH §"D-19 LDAERF Tolerance Analysis").

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_sub, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_pow, ctaylor_exp, ctaylor_erf, ctaylor_reciprocal, ctaylor_powi_2, ctaylor_powi_3};
use crate::density_vars::DensVarsDev;

const CKF: f64 = 3.093667726280136;
const SQRT_PI: f64 = 1.7724538509055159;
const PI: f64 = std::f64::consts::PI;
const RANGESEP_MU: f64 = 0.4;     // XC_RANGESEP_MU default; future: read from XCFunctional settings

#[cube]
fn esrx_ldaerfspin<F: Float, const N: u32>(
    na: &Array<F>,
    out: &mut Array<F>,
) {
    // 1:1 port of esrx_ldaerfspin (ldaerfx.cpp:24-48).
    // (Branch dispatch via comptime is OK because the input scalar branches
    //  are NOT comptime — branches handled via host-side per-grid-point dispatch.)
    //
    // Phase 2 simplification: the host-side launcher decides which branch
    // applies based on input[0] and selects the appropriate `#[cube] fn`.
    // Inside the kernel, only the chosen branch's formula runs (no in-kernel
    // if-else over runtime values, which cubecl 0.10-pre.3 doesn't optimize well).
    todo!("LDA-06 implementation per ldaerfx.cpp:24-48 — branched into 4 sub-kernels: \
          esrx_branch_a (a<1e-9), esrx_branch_b (1e-9..=100), esrx_branch_c (100..=1e9), \
          esrx_branch_d (>=1e9). Each is a pure #[cube] fn with no runtime branches.")
}

#[cube]
pub fn ldaerfx_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
) {
    // C++: return 0.5 * (esrx_ldaerfspin(d.a, mu) + esrx_ldaerfspin(d.b, mu));
    let mut esrx_a = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut esrx_b = Array::<F>::new(comptime!((1u32 << N) as usize));
    esrx_ldaerfspin::<F, N>(&d.a, &mut esrx_a);
    esrx_ldaerfspin::<F, N>(&d.b, &mut esrx_b);
    let mut sum = Array::<F>::new(comptime!((1u32 << N) as usize));
    ctaylor_add::<F>(&esrx_a, &esrx_b, &mut sum, N);
    ctaylor_scalar_mul::<F>(&sum, F::new(0.5), out, N);
}
```

**Adaptation notes:**
- The 4-branch dispatch on `a < 1e-9` etc. (ldaerfx.cpp:34-47) is a runtime conditional. cubecl-cpu can lower runtime conditionals on `F` values, but the cleanest port is to dispatch host-side: read `input[0]`, decide which sub-kernel to launch. Wave-1B-11 plan revisits this.
- Range-separation parameter `XC_RANGESEP_MU` is held in `XCFunctional::settings[]` in C++. Phase 2 hard-codes the default 0.4; Phase 5 (RS-01..10) wires the runtime parameter API.
- `pow(24*rhoa/PI, 1/3)` uses `ctaylor_pow` (Phase 1 `math.rs:185-196`) with `a = 1.0/3.0`.
- `erf(0.5/a)` uses `ctaylor_erf` (Phase 1 `math.rs:215-222`) on the inverted ctaylor `1/a`.
- `exp(-0.25/a2)` uses `ctaylor_exp` (Phase 1 `math.rs:135-142`) on negated reciprocal of `a^2`.
- Tier-2 tolerance override 1e-7 per D-24 + RESEARCH §"D-19 LDAERF Tolerance Analysis"; documented in `report.html` as a per-functional divergence note.

#### `crates/xcfun-eval/src/functionals/lda/tw.rs` (NEW — LDA-09 part 2, kinetic-GGA)

**Analog:** `xcfun-master/src/functionals/tw.cpp:20-30` (full body)

**C++ source** (tw.cpp:20-22):
```cpp
template <typename num> static num tw(const densvars<num> & d) {
    return 1. / 8. * pow(d.gaa + d.gbb, 2.0) / d.n;
}
```

**Target Rust shape:**
```rust
//! Tw kinetic energy functional (kinetic-GGA).
//!
//! # Source
//! - `xcfun-master/src/functionals/tw.cpp:20-22`
//!
//! # Formula
//! $$ T_W = \frac{1}{8} \cdot (\text{gaa} + \text{gbb})^2 / n $$
//!
//! # Preconditions
//! - `d.gaa`, `d.gbb` populated by `build_densvars` XC_A_B_GAA_GAB_GBB arm
//!   (Wave-1C). Pure-LDA `XC_A_B` arm leaves these zero — TW would return 0.
//! - `d.n > 0` (regularize ensures `>= 1e-14`).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::{ctaylor_powi_2, ctaylor_reciprocal};
use crate::density_vars::DensVarsDev;

#[cube]
pub fn tw_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
) {
    // C++ operation order:
    //   1. sum = gaa + gbb
    //   2. sum2 = sum^2          (use ctaylor_powi_2 — fused x*x)
    //   3. inv_n = 1/n           (ctaylor_reciprocal)
    //   4. tmp = sum2 * inv_n    (ctaylor_mul)
    //   5. out = (1/8) * tmp     (ctaylor_scalar_mul)
    let mut sum = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut sum2 = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut inv_n = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut tmp = Array::<F>::new(comptime!((1u32 << N) as usize));

    ctaylor_add::<F>(&d.gaa, &d.gbb, &mut sum, N);
    ctaylor_powi_2::<F>(&sum, &mut sum2, N);
    ctaylor_reciprocal::<F>(&d.n, &mut inv_n, N);
    ctaylor_mul::<F>(&sum2, &inv_n, &mut tmp, N);
    ctaylor_scalar_mul::<F>(&tmp, F::new(0.125), out, N);
}
```

**Adaptation notes:**
- Pitfall PHASE2-D: this functional REQUIRES the `XC_A_B_GAA_GAB_GBB` builder arm (Wave-1C). Wave-1B's `XC_A_B` arm leaves `gaa, gbb` zero — TW would return 0 silently.
- `pow(x, 2.0)` ports to `ctaylor_powi_2` (Phase 1 `math.rs:333-344`) which is a fused `x * x` (no series expansion needed for integer exponent 2).

#### `crates/xcfun-eval/src/functionals/lda/vwk.rs` (NEW — LDA-10, kinetic-GGA)

**Analog:** `xcfun-master/src/functionals/vonw.cpp:17-30` (file is `vonw.cpp`, FUNCTIONAL is `XC_VWK`)

**C++ source** (vonw.cpp:17-23):
```cpp
template <typename num> static num vW_alpha(const num & na, const num & gaa) {
    return gaa / (8 * na);
}

template <typename num> static num vW(const densvars<num> & d) {
    return vW_alpha(d.a, d.gaa) + vW_alpha(d.b, d.gbb);
}
```

**Target Rust shape:**
```rust
//! von Weizsäcker kinetic energy functional (XC_VWK).
//!
//! # Source
//! - `xcfun-master/src/functionals/vonw.cpp:17-29` (note: file is `vonw.cpp`,
//!   FUNCTIONAL macro is `XC_VWK`)
//!
//! # Formula
//! $$ T_W = \frac{\text{gaa}}{8 \cdot a} + \frac{\text{gbb}}{8 \cdot b} $$
//!
//! # Preconditions
//! - `d.gaa`, `d.gbb` populated by build_densvars XC_A_B_GAA_GAB_GBB arm.
//! - `d.a`, `d.b > 0` (regularize ensures `>= 1e-14`).

use cubecl::prelude::*;
use xcfun_ad::ctaylor::{ctaylor_add, ctaylor_scalar_mul};
use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;
use xcfun_ad::math::ctaylor_reciprocal;
use crate::density_vars::DensVarsDev;

#[cube]
fn vw_alpha<F: Float, const N: u32>(
    na: &Array<F>,
    gaa: &Array<F>,
    out: &mut Array<F>,
) {
    // C++: return gaa / (8 * na);  →  out = (1/8) * gaa * (1/na)
    let mut inv_na = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut tmp = Array::<F>::new(comptime!((1u32 << N) as usize));
    ctaylor_reciprocal::<F>(na, &mut inv_na, N);
    ctaylor_mul::<F>(gaa, &inv_na, &mut tmp, N);
    ctaylor_scalar_mul::<F>(&tmp, F::new(0.125), out, N);
}

#[cube]
pub fn vwk_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,
) {
    // C++: return vW_alpha(d.a, d.gaa) + vW_alpha(d.b, d.gbb);
    let mut va = Array::<F>::new(comptime!((1u32 << N) as usize));
    let mut vb = Array::<F>::new(comptime!((1u32 << N) as usize));
    vw_alpha::<F, N>(&d.a, &d.gaa, &mut va);
    vw_alpha::<F, N>(&d.b, &d.gbb, &mut vb);
    ctaylor_add::<F>(&va, &vb, out, N);
}
```

#### `crates/xcfun-eval/tests/self_tests.rs` (NEW — tier-1 parametric loop)

**Analog:** `crates/xcfun-ad/tests/cubecl_spike.rs` (test harness shape, lines 18-94) + RESEARCH §"Phase Requirements → Test Map" tier-1 row

**Pattern:**
```rust
//! Tier-1 self-tests — validate each LDA functional against its upstream
//! `test_in`/`test_out` data at `desc.test_threshold`. Source: D-16 + ACC-04.

#![cfg(feature = "testing")]

use approx::assert_relative_eq;
use xcfun_core::{FUNCTIONAL_DESCRIPTORS, Mode};
use xcfun_eval::Functional;

#[test]
fn tier1_self_tests_pass() {
    for desc in FUNCTIONAL_DESCRIPTORS.iter() {
        // Skip stubs and entries without test data (e.g., LDAERFC_JT, TW, VWK).
        let Some(test_in) = desc.test_in else { continue };
        let Some(test_out) = desc.test_out else { continue };
        let test_threshold = desc.test_threshold.expect("test data → threshold");
        let test_vars = desc.test_vars.expect("test data → vars");
        let test_order = desc.test_order.expect("test data → order");

        let fun = Functional {
            weights: &[(desc.id, 1.0)],
            vars: test_vars,
            mode: Mode::PartialDerivatives,
            order: test_order,
        };

        let mut output = vec![0.0_f64; test_out.len()];
        fun.eval(test_in, &mut output).expect("eval ok");

        for (i, (got, want)) in output.iter().zip(test_out.iter()).enumerate() {
            assert_relative_eq!(
                *got, *want,
                max_relative = test_threshold,
                epsilon = test_threshold,
            );
        }
    }
}
```

**Adaptation notes:**
- Loops over `FUNCTIONAL_DESCRIPTORS` — picks up new entries automatically as Phase 3/4 add functionals.
- Uses `desc.test_threshold` (per-functional, sourced from upstream — 1e-11 for SLATERX, 1e-7 for LDAERFX/LDAERFC, 1e-5 for TFK).
- Skips stubs + LDAERFC_JT (no upstream test data per ldaerfc_jt.cpp:64) + TW + VWK (no test data either).
- Single test per the loop — runs in <5s per ACC-04.

#### `crates/xcfun-eval/tests/densvars_field_parity.rs` (NEW — CORE-05 22-field check)

**Analog:** RESEARCH §"build_densvars Pattern" + `xcfun-master/src/densvars.hpp:35-218` (per-variant arm targets)

**Pattern (sketch):**
```rust
#![cfg(feature = "testing")]

use approx::assert_relative_eq;
use xcfun_eval::density_vars::{DensVarsDev, build_densvars};

#[test]
fn xc_a_b_arm_populates_22_fields() {
    // Build with input = [a=1.0, b=0.5], variant = XC_A_B (=2), N=2.
    // Verify: a, b, n=a+b=1.5, s=a-b=0.5, zeta=s/n=0.333, n_m13=pow(1.5, -1/3),
    //         a_43=pow(1.0,4/3)=1.0, b_43=pow(0.5,4/3),
    //         r_s=RS_PREFACTOR * n_m13.
    // Other 14 fields (gaa..jpbb) should be zero (defensively initialised).
    todo!("Wave-1B-2 — exercise build_densvars(input, &mut dv, vars=2, n=2)")
}

#[test]
fn xc_a_b_gaa_gab_gbb_arm_populates_gradient_fields() {
    // Wave-1C: input = [a, b, gaa, gab, gbb], variant = XC_A_B_GAA_GAB_GBB (=6).
    // Verify gaa, gab, gbb populated; gnn = gaa + 2*gab + gbb;
    //         gss = gaa - 2*gab + gbb; gns = gaa - gbb;
    //         then chain to XC_A_B sets a, b, n, s.
    todo!()
}
```

---

### Wave-2 validation harness

#### `validation/Cargo.toml` (NEW)

**Analog:** `xtask/Cargo.toml` (lines 1-36)

**Target shape:**
```toml
[package]
name = "validation"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false
description = "Tier-2 parity harness — Rust cubecl-cpu vs C++ xcfun"

[[bin]]
name = "validation"
path = "src/main.rs"

[dependencies]
xcfun-eval = { path = "../crates/xcfun-eval", features = ["testing"] }
xcfun-core = { path = "../crates/xcfun-core" }
anyhow = { workspace = true }
approx = { workspace = true }
serde_json = { workspace = true }
rand_xoshiro = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { version = "=0.3.23", features = ["fmt"] }

[build-dependencies]
cc = { workspace = true }
```

#### `validation/build.rs` (NEW)

**Analog:** RESEARCH §"Validation Harness Bring-Up" `cc::Build setup` (verbatim) + `xtask/src/bin/regen_ad_fixtures.rs::compile_driver` (Phase 1 cc-flag pattern)

**Target shape (RESEARCH verbatim):**
```rust
fn main() -> std::io::Result<()> {
    let xcfun_root = "../xcfun-master";
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .flag("-fno-fast-math")
        .flag("-ffp-contract=off")           // ACC-05 — match Rust-side flag
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
    // Stubs for the other 67 (auto-generated by xtask regen-registry).
    build.file("c_stubs.cpp");

    build.compile("xcfun_cpp_lda");
    Ok(())
}
```

**Adaptation notes:**
- Same flags as Phase 1 `regen_ad_fixtures.rs:60-65` (`-fno-fast-math -ffp-contract=off`) — these are ACC-05 inheritance.
- `cc 1.2.60` parallel feature already enabled via workspace deps — speeds up cold compile of 14 .cpp files.
- `c_stubs.cpp` is auto-generated by `xtask regen-registry` from the 67 non-LDA functional IDs (RESEARCH §"Validation Harness Bring-Up").

#### `validation/c_stubs.cpp` (NEW — auto-generated by extractor)

**Analog:** RESEARCH §"Validation Harness Bring-Up" `Workaround for the GGA/MGGA exclusion`

**Pattern:**
```cpp
// validation/c_stubs.cpp — AUTO-GENERATED. DO NOT EDIT.
// Stubs for Phase 2 cc-compile — every non-LDA functional ID needs a
// fundat_db specialization or xcint.cpp won't link.
#include "functional.hpp"

template <typename num> static num stub_unimpl(const densvars<num> &) { return num(0); }

FUNCTIONAL(XC_PW86X) = {"stub", "stub", XC_DENSITY|XC_GRADIENT, ENERGY_FUNCTION(stub_unimpl)};
FUNCTIONAL(XC_PBEX)  = {"stub", "stub", XC_DENSITY|XC_GRADIENT, ENERGY_FUNCTION(stub_unimpl)};
// … 65 more stubs for GGA/MGGA …
```

#### `validation/src/main.rs` (NEW CLI controller)

**Analog:** `xtask/src/main.rs` (subcommand-dispatch shape)

**Pattern:**
```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Parse: --backend cpu --order N --filter regex
    // (Use either clap or manual argv parse — keep simple per xtask precedent.)

    let backend = parse_arg(&args, "--backend").unwrap_or("cpu".into());
    let order: u32 = parse_arg(&args, "--order").unwrap_or("2".into()).parse()?;
    let filter = parse_arg(&args, "--filter").unwrap_or(".*".into());

    let grid = validation::fixtures::generate_grid();
    let report = validation::driver::run(&grid, &backend, order, &filter)?;
    validation::report::write_html(&report, "report.html")?;
    validation::report::write_jsonl(&report, "report.jsonl")?;

    if report.has_failures() {
        std::process::exit(2);
    }
    Ok(())
}
```

#### `validation/src/ffi.rs` (NEW)

**Analog:** RESEARCH §"Validation Harness Bring-Up" `FFI shim shape` (verbatim) + `xcfun-master/api/xcfun.h:1-130` (extern "C" declarations)

**Pattern (verbatim from RESEARCH):**
```rust
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

**Adaptation notes:**
- This is the only `unsafe extern "C"` block in the workspace today. The crate root must drop `#![forbid(unsafe_code)]` (validation crate is the one place where unsafe FFI is permitted, parallel to anyhow allowance).
- `unsafe extern "C"` with edition 2024 syntax (per CLAUDE.md MSRV 1.85, Rust 2024 edition).

#### `validation/src/fixtures.rs` (NEW)

**Analog:** RESEARCH §"Grid Generator Spec" (verbatim skeleton) + `crates/xcfun-ad/tests/proptest_algebra.rs` (Phase 1 rand_xoshiro seed pattern, commit `3514217`)

**Pattern (verbatim from RESEARCH, lines 938-1010):**
```rust
use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};
use rand_core::RngCore;

pub const GRID_SEED: u64 = 0x1234abcd;
pub const TOTAL_POINTS: usize = 10_000;

#[derive(Clone, Copy)]
pub struct GridPoint {
    pub n: f64,
    pub s: f64,
    pub gnn: f64, pub gns: f64, pub gss: f64,
    pub gaa: f64, pub gab: f64, pub gbb: f64,
}

impl GridPoint {
    pub fn ab_from_ns(&self) -> (f64, f64) {
        ((self.n + self.s) * 0.5, (self.n - self.s) * 0.5)
    }
}

pub fn generate_grid() -> Vec<GridPoint> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(GRID_SEED);
    let mut out = Vec::with_capacity(TOTAL_POINTS);
    out.extend(generate_bulk(&mut rng, 7000));
    out.extend(generate_regularize_stress(&mut rng, 1000));
    out.extend(generate_polarised(&mut rng, 1000));
    out.extend(generate_gradient_stress(&mut rng, 1000));
    out
}

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

**Adaptation notes:**
- Seed `0x1234abcd` is fixed; reproducibility verified across platforms per Phase 1's experience.
- 4 strata × seed gives byte-identical grid on every harness run — no committed fixtures (D-15).
- `rand_xoshiro 0.8.0` matches CLAUDE.md pin.

#### `validation/src/driver.rs` (NEW)

**Analog:** `xcfun-master/src/XCFunctional.cpp:493-617` (PartialDerivatives output element layout that Rust must mirror) + RESEARCH §"Mode::PartialDerivatives Output Layout"

**Pattern:**
```rust
//! Tier-2 parity driver — for each (functional, vars, mode, order, point):
//! evaluate Rust + C++, compute rel-error per output element, accumulate.

use anyhow::Result;
use xcfun_core::{FUNCTIONAL_DESCRIPTORS, Mode, taylorlen};
use xcfun_eval::Functional;
use crate::ffi::CppXcfun;
use crate::fixtures::GridPoint;

pub struct ReportRecord {
    pub functional: &'static str,
    pub vars: &'static str,
    pub mode: Mode,
    pub order: u32,
    pub point_idx: usize,
    pub element_idx: usize,
    pub input: Vec<f64>,
    pub rust: f64,
    pub cpp: f64,
    pub abs_err: f64,
    pub rel_err: f64,
    pub pass: bool,
}

pub fn run(grid: &[GridPoint], backend: &str, order: u32, filter: &str) -> Result<Report> {
    let regex = regex::Regex::new(filter)?;
    let mut report = Report::new();

    for desc in FUNCTIONAL_DESCRIPTORS.iter() {
        if !regex.is_match(desc.name.to_lowercase().as_str()) { continue }
        let Some(test_vars) = desc.test_vars else { continue };
        // Phase 2: order ∈ {0, 1, 2} only per D-23.
        for ord in 0..=order.min(2) {
            // Set up Rust Functional + CppXcfun
            let rust_fun = Functional { weights: &[(desc.id, 1.0)], vars: test_vars, mode: Mode::PartialDerivatives, order: ord };
            let mut cpp_fun = CppXcfun::new();
            cpp_fun.set(&desc.name.to_lowercase(), 1.0);
            cpp_fun.eval_setup(test_vars as u32, Mode::PartialDerivatives as u32, ord as i32);

            let inlen = xcfun_core::VARS_TABLE[test_vars as usize].len as usize;
            let outlen = taylorlen(inlen, ord as usize);

            for (point_idx, gp) in grid.iter().enumerate() {
                let input = build_input(*gp, test_vars, inlen);
                let mut rust_out = vec![0.0_f64; outlen];
                let mut cpp_out  = vec![0.0_f64; outlen];

                rust_fun.eval(&input, &mut rust_out)?;
                cpp_fun.eval(&input, &mut cpp_out);

                for elem_idx in 0..outlen {
                    let rec = compute_record(desc, test_vars, ord, point_idx, elem_idx, &input, rust_out[elem_idx], cpp_out[elem_idx]);
                    report.push(rec);
                }
            }
        }
    }
    Ok(report)
}

fn compute_record(/* … */) -> ReportRecord {
    let abs_err = (rust - cpp).abs();
    let rel_err = abs_err / cpp.abs().max(1.0);
    let threshold = if desc.name.starts_with("XC_LDAERF") { 1e-7 } else { 1e-12 };  // D-24
    ReportRecord { /* … */, pass: rel_err <= threshold }
}
```

**Adaptation notes:**
- Per-functional threshold logic matches D-24: `XC_LDAERFX`, `XC_LDAERFC`, `XC_LDAERFC_JT` use 1e-7; all other LDAs use 1e-12.
- Output element ordering mirrors `XCFunctional.cpp:493-612` (verified in RESEARCH §"Mode::PartialDerivatives Output Layout").
- `build_input(gp, vars, inlen)` converts a `GridPoint` to the right input shape per `Vars` discriminant. For `XC_A_B`: `(a, b) = ab_from_ns(gp)` → `[a, b]`. For `XC_A_B_GAA_GAB_GBB`: `[a, b, gp.gaa, gp.gab, gp.gbb]`.

#### `validation/src/report.rs` (NEW)

**Analog:** RESEARCH §"report.html schema" + §"report.jsonl schema" + `xtask/src/bin/regen_ad_fixtures.rs::main` (manifest-write pattern, lines 285-300)

**Pattern (HTML matrix + JSONL writers):**
```rust
use std::fs;
use std::io::Write;
use anyhow::Result;
use crate::driver::{Report, ReportRecord};

pub fn write_jsonl(report: &Report, path: &str) -> Result<()> {
    let mut f = fs::File::create(path)?;
    for rec in &report.records {
        writeln!(f, "{}", serde_json::to_string(rec)?)?;
    }
    Ok(())
}

pub fn write_html(report: &Report, path: &str) -> Result<()> {
    let mut html = String::new();
    html.push_str("<html><head><title>XCFun Tier-2 Parity Report</title>\n");
    html.push_str("<style>td.green{background:#cfc}td.yellow{background:#ffc}td.red{background:#fcc}</style>\n");
    html.push_str("</head><body>\n");
    html.push_str("<h1>XCFun Tier-2 Parity Report</h1>\n");
    html.push_str("<table><thead><tr><th>Functional</th><th>order=0</th><th>order=1</th><th>order=2</th><th>Tolerance</th></tr></thead><tbody>\n");
    for (functional, by_order) in report.matrix() {
        html.push_str(&format!("<tr><td>{}</td>", functional));
        for ord in 0..=2 {
            let cell = by_order.get(&ord).map(|r| {
                let max_rel = r.max_rel_err;
                let cls = if max_rel < 1e-13 { "green" } else if max_rel < r.threshold { "yellow" } else { "red" };
                format!("<td class=\"{}\">{:.2e}</td>", cls, max_rel)
            }).unwrap_or_else(|| "<td>—</td>".into());
            html.push_str(&cell);
        }
        html.push_str(&format!("<td>{:.0e}</td></tr>\n", by_order.values().next().map(|r| r.threshold).unwrap_or(1e-12)));
    }
    html.push_str("</tbody></table></body></html>\n");
    fs::write(path, html)?;
    Ok(())
}
```

**Adaptation notes:**
- `report.html` matrix shape per RESEARCH §"report.html schema" — Functional × order with max-rel-error per cell, color-coded green/yellow/red.
- `report.jsonl` per-record shape per RESEARCH §"report.jsonl schema" — one record per `(functional, vars, mode, order, point_idx, element_idx)` tuple.
- Per-functional threshold (1e-7 for LDAERFs, 1e-12 for others) is reflected in the "Tolerance" column.

---

## Shared Patterns

### Pattern 1: `#[cube] fn` kernel signature (D-17)

**Source:** `crates/xcfun-ad/src/ctaylor.rs:30-38` (`ctaylor_zero`) + Phase 1 D-17 + Phase 2 D-17

**Apply to:** Every `crates/xcfun-eval/src/functionals/lda/*.rs` kernel

**Excerpt:**
```rust
#[cube]
pub fn <name>_kernel<F: Float, const N: u32>(
    d: &DensVarsDev<F>,
    out: &mut Array<F>,            // CTaylor<F, N> as length-(1<<N) Array
) {
    // Allocate kernel-local scratch as Array::<F>::new(comptime!((1u32 << N) as usize))
    // (Phase 1 idiom from math.rs:89-95)
    let mut tmp = Array::<F>::new(comptime!((1u32 << N) as usize));
    // … operations preserving C++ source order …
}
```

### Pattern 2: Composed-fn pipeline (Phase 1 D-14)

**Source:** `crates/xcfun-ad/src/math.rs:82-96` (`ctaylor_reciprocal`)

**Apply to:** Every `*_kernel` that uses `pow`, `exp`, `log`, `sqrt`, `erf`, `asinh`, `atan` on a CTaylor (i.e., LDA-02..LDA-08, LDA-09 part 2, LDA-10).

**Excerpt:**
```rust
// 3-step pipeline (math.rs:82-96):
//   1. *_expand — fills length-(n+1) scalar Taylor scratch
//   2. ctaylor_compose — composes scratch with x as inner polynomial
//   3. (implicit) out holds Taylor coefficients of op(x)
//
// Helper math fns:
//   ctaylor_reciprocal, ctaylor_sqrt, ctaylor_exp, ctaylor_log, ctaylor_pow,
//   ctaylor_powi, ctaylor_erf, ctaylor_asinh, ctaylor_atan.
```

### Pattern 3: ACC-06 mul_add prohibition

**Source:** D-13 + RESEARCH §"Specific Ideas" + Phase 1 `xtask/src/bin/check_no_fma.rs`

**Apply to:** Every `crates/xcfun-eval/src/functionals/**/*.rs` source file

**Excerpt:**
```rust
// FORBIDDEN: a.mul_add(b, c) — emits FMA, violates 1e-12 contract
// REQUIRED: a * b + c written as explicit two-step:
//   ctaylor_mul(a, b, &mut tmp, n);
//   ctaylor_add(&tmp, &c, &mut out, n);
```

CI gate: `cargo run -p xtask --bin check-no-mul-add` greps for `\.mul_add\s*\(` and exits 2 on any match.

### Pattern 4: 1-thread cubecl-cpu launch for scalar eval (Phase 1 D-15)

**Source:** `crates/xcfun-ad/src/for_tests/raw_eval_scalar.rs:40-56`

**Apply to:** `crates/xcfun-eval/src/functional.rs::Functional::eval` body (every per-point evaluation)

**Excerpt:**
```rust
let client = cpu_client();
let in_handle = client.create_from_slice(f64::as_bytes(input));
let out_handle = client.empty(out_len * core::mem::size_of::<f64>());
let read_handle = out_handle.clone();
unsafe {
    dispatch_kernel::launch_unchecked::<f64, CpuRuntime>(
        client,
        CubeCount::Static(1, 1, 1),
        CubeDim::new_3d(1, 1, 1),
        /* args including ArrayArg::from_raw_parts(in_handle, in_len), out_handle, comptime args */
    );
}
let bytes = client.read_one_unchecked(read_handle);
let out = f64::from_bytes(&bytes);
```

### Pattern 5: SHA-256 drift-stamp gate (QG-07)

**Source:** `xtask/src/bin/regen_ad_fixtures.rs:165-176` + RESEARCH §"CORE-10 Extractor Recommendation"

**Apply to:** `xtask/src/bin/regen_registry.rs --check` mode

**Excerpt:**
```rust
fn header_sha256(xcfun_root: &Path, files: &[&str]) -> Result<String> {
    let mut hasher = Sha256::new();
    for fname in files {
        let path = xcfun_root.join(fname);
        let contents = fs::read(&path).with_context(|| format!("read xcfun-master file {:?}", path))?;
        hasher.update(&contents);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
```

In `--check` mode: regenerate `*.rs` to a temp dir, hash each, diff against committed `*.sha256`.

### Pattern 6: Doc-comment header (3 items) per port

**Source:** Phase 1 idiom in every `expand/*.rs` — see e.g. `crates/xcfun-ad/src/math.rs:64-78` (header for `ctaylor_reciprocal`)

**Apply to:** Every `crates/xcfun-eval/src/functionals/lda/*.rs` and `density_vars/build.rs` arm

**Required items:**
1. Upstream source (`xcfun-master/src/functionals/<file>.cpp:LL-LL`).
2. Mathematical formula in LaTeX (`$$ E_{xc} = … $$`).
3. Preconditions (e.g., `d.a > 0` regularize-enforced, `d.gaa, d.gbb` populated by which builder arm).

### Pattern 7: `xcfun.h` C ABI extern "C" declaration (validation FFI shim)

**Source:** `xcfun-master/api/xcfun.h:1-130` (functions declared `XCFun_API`)

**Apply to:** `validation/src/ffi.rs`

**Excerpt:**
```c
XCFun_API int xcfun_eval_setup(xcfun_t fun, xcfun_vars vars, xcfun_mode mode, int order);
XCFun_API int xcfun_input_length(xcfun_t fun);
XCFun_API int xcfun_output_length(xcfun_t fun);
XCFun_API void xcfun_eval(xcfun_t fun, const double density[], double result[]);
```

Rust port:
```rust
unsafe extern "C" {
    pub fn xcfun_eval_setup(fun: *mut std::ffi::c_void, vars: u32, mode: u32, order: i32) -> i32;
    pub fn xcfun_input_length(fun: *const std::ffi::c_void) -> i32;
    pub fn xcfun_output_length(fun: *const std::ffi::c_void) -> i32;
    pub fn xcfun_eval(fun: *const std::ffi::c_void, density: *const f64, result: *mut f64);
}
```

---

## No Analog Found

Files with no close analog in the codebase (planner should use RESEARCH.md patterns or external references):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/xcfun-eval/src/density_vars.rs` | model (`#[derive(CubeType, CubeLaunch)] struct`) | transform | First `#[derive(CubeType, CubeLaunch)]` site in workspace. Phase 1 used only kernel-local `Array<F>::new`, not nested struct types. Source pattern: cubecl-book `language-support/struct.md` cited in RESEARCH §"D-02 cubecl Nesting Decision" (Pattern A skeleton). |
| `crates/xcfun-eval/tests/densvars_field_parity.rs` | test (per-variant 22-field check) | request-response | First test that exercises `DensVarsDev` against C++ `densvars<T>` reference output. Wave-1B-2 task spec from RESEARCH §"build_densvars Pattern". |
| `xtask/assets/regen_registry/extractor.cpp` | build (~150 LoC C++) | stdin/stdout | First C++ extractor in workspace (Phase 1 had a fixture `driver.cpp` but that was a numerical driver, not a parser). Pattern: regex over `FUNCTIONAL` macros + JSONL output per RESEARCH §"CORE-10 Extractor Recommendation". |
| `validation/src/ffi.rs` | service (extern "C" wrapper) | request-response | First FFI shim in workspace — `xcfun-capi` is Phase 5. Reference: `xcfun-master/api/xcfun.h:1-130` decl shape + RESEARCH §"FFI shim shape" verbatim skeleton. |
| `validation/src/fixtures.rs` | utility (stratified grid generator) | batch | First stratified-grid generator. Phase 1 proptest used flat `next_f64`. Pattern: 4-stratum 70/30 per RESEARCH §"Grid Generator Spec" verbatim skeleton. |
| `validation/src/report.rs` | utility (HTML matrix writer) | file-I/O | First HTML emitter in workspace. Pattern: hand-written table per RESEARCH §"report.html schema". |
| `crates/xcfun-eval/src/functional.rs` | model (Functional struct + eval) | request-response | The pre-pivot `xcfun-core::traits::Functional` trait surface is being deleted in Wave-0e; the new struct-based Functional in xcfun-eval is a fresh design. Closest precedent: RESEARCH §"Registry Shape + Circular-Dep Resolution" + Phase 1 `for_tests::raw_eval_scalar` (1-thread launch shape). |

---

## Metadata

**Analog search scope:**
- `crates/xcfun-ad/src/**` (Phase 1 cubecl primitives)
- `crates/xcfun-core/src/**` (pre-pivot scaffolding being rewritten)
- `xtask/src/bin/**` (existing xtask binaries: regen_ad_fixtures, check_no_fma)
- `xtask/src/main.rs`, `xtask/src/fixtures.rs`
- `xcfun-master/src/**` (C++ reference: densvars.hpp, XCFunctional.cpp, xcint.cpp, functional.hpp, every src/functionals/*.cpp for the 11 LDAs)
- `xcfun-master/api/xcfun.h` (C ABI declarations)
- `Cargo.toml`, `.cargo/config.toml` (workspace config)
- `crates/xcfun-ad/tests/cubecl_spike.rs` (test harness reference)

**Files scanned:** ~45 source files (cubecl-ad: 12, xcfun-core: 8, xtask: 4, xcfun-master: 18, workspace cfg: 3)

**Pattern extraction date:** 2026-04-20
