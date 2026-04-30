---
plan_id: 05-01-rust-facade
phase: 05
wave: 2
depends_on:
  - 05-00-topology-foundation
files_modified:
  - crates/xcfun-rs/Cargo.toml
  - crates/xcfun-rs/src/lib.rs
  - crates/xcfun-rs/src/functional.rs
  - crates/xcfun-rs/src/free_fns.rs
  - crates/xcfun-rs/assets/splash.txt
  - crates/xcfun-rs/assets/authors.txt
  - crates/xcfun-rs/tests/send_sync.rs
  - crates/xcfun-rs/tests/zero_alloc.rs
  - crates/xcfun-rs/tests/free_fns.rs
requirements:
  - RS-01
  - RS-02
  - RS-03
  - RS-04
  - RS-05
  - RS-06
  - RS-07
  - RS-09
  - RS-10
autonomous: true
---

## objective
<objective>
Phase 5 Wave 2 — fill the `xcfun-rs` facade crate skeleton (created by Plan
05-00) with the **complete public Rust API surface** specified by RS-01..07,
RS-09, RS-10:

1. **`pub struct Functional(xcfun_eval::Functional)` newtype** with the 8
   methods `new`, `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`,
   `user_eval_setup`, `eval`, plus `input_length` / `output_length` accessors.
   All delegate to the inner `xcfun_eval::Functional`. (D-02, RS-01..07)
2. **11 free functions** at the crate root: `version`, `splash`, `authors`,
   `self_test`, `is_compatible_library`, `which_vars`, `which_mode`,
   `enumerate_parameters`, `enumerate_aliases`, `describe_short`,
   `describe_long` — line-for-line ports of XCFunctional.cpp:48-72,
   131-277, 302-348. (D-03, RS-09)
3. **Send + Sync compile-time gate** (`assert_impl_all!`). (D-17, RS-10)
4. **Zero-allocation invariant test** for the hot eval path (counting
   `#[global_allocator]`). (D-13, RS-07)
5. **Free-function behaviour tests** covering enumerate / describe /
   which_vars / which_mode parity with the C++ reference shape.

Purpose: this is the public Rust crate that all other consumers
(Phase 5 `xcfun-capi` C ABI, Phase 7 `xcfun-py` Python bindings,
external Rust users on crates.io) link against.

Output: a fully-populated `xcfun-rs` crate that `cargo test -p xcfun-rs`
exercises end-to-end including the Send+Sync compile-time gate, the
zero-alloc fixture, and at least 10 unit tests covering free-function
parity with the C++ XCFunctional.cpp reference.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-PATTERNS.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-00-SUMMARY.md

# C++ source-of-truth
@xcfun-master/api/xcfun.h
@xcfun-master/src/XCFunctional.cpp

# Existing Rust patterns to mirror
@crates/xcfun-eval/src/functional.rs
@crates/xcfun-core/src/lib.rs
@crates/xcfun-core/src/error.rs
@crates/xcfun-core/src/registry/generated/parameters.rs
@crates/xcfun-eval/tests/regularize_invariant.rs

<interfaces>
<!-- Existing types and exports the executor depends on. Cite verbatim — no exploration needed. -->

From crates/xcfun-core/src/lib.rs:20-28:
```rust
pub use constants::*;
pub use enums::{Mode, ParameterId, Vars};
pub use error::XcError;
pub use functional_id::FunctionalId;
pub use registry::{
    ALIASES, Alias, FUNCTIONAL_DESCRIPTORS, FunctionalDescriptor, PARAMETERS,
    ParameterEntry, VARS_TABLE, VarsRow,
};
pub use traits::Dependency;
```

From crates/xcfun-core/src/lib.rs (taylorlen):
```rust
pub const fn taylorlen(n_vars: usize, order: usize) -> usize { ... }
```

From crates/xcfun-eval/src/functional.rs:85-474 (Functional public surface — all methods to delegate to):
```rust
pub struct Functional {
    pub weights:  &'static [(FunctionalId, f64)],
    pub vars:     Vars,
    pub mode:     Mode,
    pub order:    u32,
    pub settings: [f64; 82],
}

impl Functional {
    pub const fn new() -> Self;
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError>;
    pub fn get(&self, name: &str) -> Result<f64, XcError>;
    pub const fn input_length(vars: Vars) -> usize;
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError>;
    pub fn dependencies(&self) -> Dependency;
    pub fn output_length(vars: Vars, mode: Mode, order: u32) -> Result<usize, XcError>;
    pub fn eval_setup(&self, vars: Vars, mode: Mode, order: u32) -> Result<(), XcError>;
}
```

NOTE: `xcfun_eval::Functional::eval_setup` is **read-only validation** (does
NOT mutate self.vars/self.mode/self.order). Phase 5's `xcfun-rs::Functional`
wrapper owns the field-write side-effect AFTER successful inner validation.

From xcfun-master/src/XCFunctional.cpp:48-72 (version/splash/authors source spec):
```cpp
const char * xcfun_version() { static auto retval = xcfun::version_as_string(); return retval.c_str(); }
const char * xcfun_splash()  { return "XCFun DFT library Copyright 2009-2020 Ulf Ekstrom and contributors.\n..."; }
const char * xcfun_authors() { return "XCFun was written by Ulf Ekstrom, with contributions from\nAndre S. P. Gomes\n..."; }
```

From xcfun-master/src/XCFunctional.cpp:131-277 (which_vars C++ switch table — 31 cases mapping bitwise_vars to xcfun_vars):
```cpp
int bitwise_vars = (func_type << 6) | (dens_type << 4) | (laplacian << 3) | (kinetic << 2) | (current << 1) | explicit_derivatives;
switch (bitwise_vars) {
  case 0:   vars = XC_A;
  case 16:  vars = XC_N;
  case 32:  vars = XC_A_B;
  case 48:  vars = XC_N_S;
  case 64:  vars = XC_A_GAA;
  case 65:  vars = XC_A_AX_AY_AZ;
  case 80:  vars = XC_N_GNN;
  case 81:  vars = XC_N_NX_NY_NZ;
  case 96:  vars = XC_A_B_GAA_GAB_GBB;
  case 97:  vars = XC_A_B_AX_AY_AZ_BX_BY_BZ;
  case 112: vars = XC_N_S_GNN_GNS_GSS;
  case 113: vars = XC_N_S_NX_NY_NZ_SX_SY_SZ;
  case 132: vars = XC_A_GAA_TAUA;
  case 133: vars = XC_A_AX_AY_AZ_TAUA;
  case 136: vars = XC_A_GAA_LAPA;
  case 148: vars = XC_N_GNN_TAUN;
  case 149: vars = XC_N_NX_NY_NZ_TAUN;
  case 152: vars = XC_N_GNN_LAPN;
  case 164: vars = XC_A_B_GAA_GAB_GBB_TAUA_TAUB;
  case 165: vars = XC_A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB;
  case 168: vars = XC_A_B_GAA_GAB_GBB_LAPA_LAPB;
  case 172: vars = XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB;
  case 174: vars = XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB;
  case 180: vars = XC_N_S_GNN_GNS_GSS_TAUN_TAUS;
  case 181: vars = XC_N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS;
  case 184: vars = XC_N_S_GNN_GNS_GSS_LAPN_LAPS;
  case 188: vars = XC_N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS;
  case 192: vars = XC_A_2ND_TAYLOR;
  case 208: vars = XC_N_2ND_TAYLOR;
  case 224: vars = XC_A_B_2ND_TAYLOR;
  case 240: vars = XC_N_S_2ND_TAYLOR;
  default:  xcfun::die("xc_user_eval_setup: Invalid vars", bitwise_vars);
}
// Range checks: func_type<=3, dens_type<=3, laplacian<=1, kinetic<=1, current<=1, explicit_derivatives<=1.
```

From xcfun-master/src/XCFunctional.cpp:281-300 (which_mode C++):
```cpp
xcfun_mode xcfun_which_mode(unsigned int mode_type) {
  if (mode_type > 3) xcfun::die(...);
  switch (mode_type) {
    case 1: return XC_PARTIAL_DERIVATIVES;
    case 2: return XC_POTENTIAL;
    case 3: return XC_CONTRACTED;
    default: xcfun::die("xc_user_eval_setup: Invalid mode", ...);
  }
}
```

From xcfun-master/src/XCFunctional.cpp:302-348 (enumerate / describe C++):
```cpp
const char * xcfun_enumerate_parameters(int param) {
  if (param >= 0 && param < XC_NR_FUNCTIONALS) return xcint_funs[param].name;     // 0..77 -> functional name
  else if (param < XC_NR_PARAMETERS_AND_FUNCTIONALS) return xcint_params[param].name;  // 78..81 -> parameter name
  else return 0;
}
const char * xcfun_enumerate_aliases(int n) {
  if (n >= 0 && n < XC_MAX_ALIASES) return xcint_aliases[n].name;
  else return 0;
}
const char * xcfun_describe_short(const char * name) {
  if ((k = xcint_lookup_functional(name)) >= 0) return xcint_funs[k].short_description;
  else if ((k = xcint_lookup_parameter(name)) >= 0) return xcint_params[k].description;
  else if ((k = xcint_lookup_alias(name)) >= 0) return xcint_aliases[k].description;
  else return 0;
}
// describe_long is identical except `long_description` for the functional case.
```

From crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (struct shape):
```rust
pub struct FunctionalDescriptor {
    pub id: FunctionalId,
    pub name: &'static str,
    pub short_description: &'static str,
    pub long_description: &'static str,
    pub depends: Dependency,
    /* test_* fields */
}
```
</interfaces>
</context>

## must_haves
<must_haves>
truths:
  - "`xcfun_rs::Functional::new()` returns an empty wrapper; `Functional::set(\"slaterx\", 1.0)` succeeds; `Functional::get(\"slaterx\")` returns 1.0; `is_gga()` and `is_metagga()` reflect the active functional set's `Dependency` aggregation. (RS-01..04, D-02)"
  - "`Functional::eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)` after `set(\"slaterx\", 1.0)` returns `Ok(())` and mutates the wrapped inner `Functional`'s vars/mode/order fields. (RS-05, D-02)"
  - "`Functional::user_eval_setup(0, 0, 2, 1, 0, 0, 0, 0)` (LDA / Alpha+Beta / Mode::PartialDerivatives) is equivalent to `eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)`. (RS-06)"
  - "`Functional::eval(&input, &mut output)` after a successful `eval_setup` returns `Ok(())` and produces non-zero output for an active functional. (RS-07)"
  - "`Functional` is `Send + Sync` (compile-time `assert_impl_all!` gate). (RS-10, D-17)"
  - "Free function `version()` returns the compile-time crate version `env!(\"CARGO_PKG_VERSION\")`. (RS-09)"
  - "Free function `which_vars(0, 2, 0, 0, 0, 0)` returns `Some(Vars::A_B)`; `which_vars(1, 2, 0, 0, 0, 0)` returns `Some(Vars::A_B_GAA_GAB_GBB)`; out-of-range inputs (e.g. `func_type=4`) return `None` (NOT `xcfun::die` — the Rust facade returns `None` instead of aborting; the C ABI in Plan 05-02 maps `None → die_with`). (RS-09)"
  - "Free function `which_mode(1)` returns `Some(Mode::PartialDerivatives)`; `which_mode(2)` returns `Some(Mode::Potential)`; `which_mode(3)` returns `Some(Mode::Contracted)`; `which_mode(0)` and `which_mode(4)` return `None`. (RS-09)"
  - "Free function `enumerate_parameters(0)` returns `Some(\"XC_SLATERX\")` (or whichever name `FUNCTIONAL_DESCRIPTORS[0].name` carries); `enumerate_parameters(78)` returns `Some(\"RANGESEP_MU\")` (`PARAMETERS[0].name`); `enumerate_parameters(82)` returns `None`; `enumerate_parameters(-1)` returns `None`. (RS-09)"
  - "Free function `enumerate_aliases(0)` returns `Some(\"null\")` (`ALIASES[0].name`); `enumerate_aliases(46)` returns `None` (out of range); `enumerate_aliases(-1)` returns `None`. (RS-09)"
  - "Free function `describe_short(\"SLATERX\")` returns `Some(\"Slater LDA exchange\")`; `describe_short(\"BLYP\")` returns `Some(\"<BLYP alias description>\")` (case-insensitive across functionals → parameters → aliases). (RS-09)"
  - "`describe_long(\"SLATERX\")` returns the SLATERX `long_description` string. (RS-09)"
  - "`Functional::eval` after a successful warm-up performs ZERO heap allocations on 100 subsequent calls (counting global allocator delta == 0). (RS-07, D-13)"
artifacts:
  - path: "crates/xcfun-rs/src/lib.rs"
    provides: "Crate root with re-exports + module declarations + crate doc"
    contains: "pub use xcfun_core"
  - path: "crates/xcfun-rs/src/functional.rs"
    provides: "Functional newtype wrapping xcfun_eval::Functional"
    contains: "pub struct Functional"
  - path: "crates/xcfun-rs/src/free_fns.rs"
    provides: "11 module-level free functions per RS-09"
    contains: "pub fn version|pub fn splash|pub fn authors|pub fn which_vars|pub fn which_mode|pub fn enumerate_parameters|pub fn enumerate_aliases|pub fn describe_short|pub fn describe_long|pub fn self_test|pub fn is_compatible_library"
  - path: "crates/xcfun-rs/tests/send_sync.rs"
    provides: "Compile-time Send+Sync gate"
    contains: "assert_impl_all!(Functional: Send, Sync)"
  - path: "crates/xcfun-rs/tests/zero_alloc.rs"
    provides: "RS-07 hot-path zero-alloc fixture"
    contains: "ALLOC_COUNT"
  - path: "crates/xcfun-rs/tests/free_fns.rs"
    provides: "Behaviour tests for 11 free functions"
    contains: "fn version_returns"
key_links:
  - from: "crates/xcfun-rs/src/functional.rs"
    to: "crates/xcfun-eval/src/functional.rs::Functional"
    via: "newtype wrapper field access via self.0"
    pattern: "self\\.0\\.(set|get|eval|eval_setup|input_length|output_length|dependencies)"
  - from: "crates/xcfun-rs/src/free_fns.rs::enumerate_parameters"
    to: "FUNCTIONAL_DESCRIPTORS + PARAMETERS"
    via: "indexed slice access then concatenated lookup"
    pattern: "FUNCTIONAL_DESCRIPTORS\\["
  - from: "crates/xcfun-rs/src/free_fns.rs::describe_short"
    to: "FunctionalId::from_name + ParameterId::from_name + ALIASES iter"
    via: "3-table cascade with eq_ignore_ascii_case on aliases"
    pattern: "eq_ignore_ascii_case|from_name"
</must_haves>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `Functional::set(name: &str, value: f64)` | User-supplied UTF-8 name; case-insensitive, must not panic on arbitrary input. |
| `Functional::eval(&[f64], &mut [f64])` | User-supplied buffers; lengths checked by inner `xcfun_eval::Functional::eval` before any kernel launch. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-05-01-01 | Denial of service | `Functional::set` recursion through alias terms | accept | Recursion depth bounded at 1 by static `ALIASES` table content (no alias references another alias — verified by Phase 4 test `aliases_all_terms_resolve_to_known_names`). |
| T-05-01-02 | Tampering | `xcfun-rs` is a library — `#![forbid(unsafe_code)]` enforced at crate root | mitigate | Add `#![forbid(unsafe_code)]` to `crates/xcfun-rs/src/lib.rs`. CI `cargo check -p xcfun-rs` fails if the attribute is dropped. |
| T-05-01-03 | Information disclosure | Free functions returning `&'static str` | accept | Splash and authors strings come from committed assets/*.txt; no caller-controlled formatting. |
| T-05-01-04 | Elevation of privilege | `Functional` field privacy | mitigate | `pub struct Functional(xcfun_eval::Functional)` keeps the inner field PRIVATE — callers cannot mutate `weights` / `settings` directly bypassing `set` validation. |
</threat_model>

## tasks
<tasks>

<task type="auto" tdd="true">
  <name>Task 1.1: xcfun-rs::Functional newtype + assets + Send+Sync gate</name>
  <files>crates/xcfun-rs/src/lib.rs, crates/xcfun-rs/src/functional.rs, crates/xcfun-rs/assets/splash.txt, crates/xcfun-rs/assets/authors.txt, crates/xcfun-rs/tests/send_sync.rs</files>
  <read_first>
    - crates/xcfun-rs/Cargo.toml (created by Plan 05-00 — must already declare xcfun-eval with `features = ["testing"]`)
    - crates/xcfun-rs/src/lib.rs (current content from Plan 05-00 — placeholder doc + `#![forbid(unsafe_code)]`; this task replaces it)
    - crates/xcfun-eval/src/functional.rs:85-474 (signatures of every method to delegate to — copy them verbatim including doc-comments stripped)
    - crates/xcfun-core/src/lib.rs:1-28 (re-export pattern for the crate root)
    - xcfun-master/src/XCFunctional.cpp:53-72 (verbatim splash + authors strings)
    - xcfun-master/src/XCFunctional.cpp:420-426 (is_gga / is_metagga semantics: `(depends & XC_GRADIENT)` and `(depends & (XC_LAPLACIAN | XC_KINETIC))`)
    - crates/xcfun-core/src/error.rs (existing assert_impl_all! pattern)
  </read_first>
  <behavior>
    - `Functional::new()` constructs an empty wrapper; `is_gga() == false`, `is_metagga() == false` initially.
    - After `set("slaterx", 1.0).unwrap()`, `get("slaterx").unwrap() == 1.0` and `is_gga() == false` (LDA), `is_metagga() == false`.
    - After `set("pbex", 1.0).unwrap()`, `is_gga() == true` (PBEX has `Dependency::GRADIENT`), `is_metagga() == false`.
    - After `set("tpssx", 1.0).unwrap()`, `is_metagga() == true`.
    - `set("not_a_known_name", 1.0)` returns `Err(XcError::UnknownName)`.
    - `eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)` after `set("slaterx", 1.0)` returns `Ok(())` and mutates the wrapper's inner `vars`/`mode`/`order` so subsequent `input_length()` / `output_length()` reflect the new state.
    - `input_length()` returns `2` after `eval_setup(Vars::A_B, _, _)`.
    - `output_length()` returns `1` after `eval_setup(Vars::A_B, Mode::PartialDerivatives, 0)`.
    - `eval(&[0.5, 0.5], &mut out)` writes a non-zero value to `out[0]`.
    - `assert_impl_all!(Functional: Send, Sync)` compiles.
  </behavior>
  <action>
    1. **Create `crates/xcfun-rs/assets/splash.txt`** with the C++ string from
       `XCFunctional.cpp:54-61` (do NOT include the trailing semicolon or the
       outer `return "..."` C++ syntax). Verbatim contents:
       ```text
       XCFun DFT library Copyright 2009-2020 Ulf Ekstrom and contributors.
       See http://dftlibs.org/xcfun/ for more information.

       This is free software; see the source code for copying conditions.
       There is ABSOLUTELY NO WARRANTY; not even for MERCHANTABILITY or
       FITNESS FOR A PARTICULAR PURPOSE. For details see the documentation.
       Scientific users of this library should cite
       U. Ekstrom, L. Visscher, R. Bast, A. J. Thorvaldsen and K. Ruud;
       J.Chem.Theor.Comp. 2010, DOI: 10.1021/ct100117s
       ```
       The file must end with a final `\n` (the C++ string ends `... DOI:.../n");`).

    2. **Create `crates/xcfun-rs/assets/authors.txt`** verbatim from
       `XCFunctional.cpp:65-71`:
       ```text
       XCFun was written by Ulf Ekstrom, with contributions from
       Andre S. P. Gomes
       Radovan Bast
       Andrea Debnarova
       Paola Gori-Giorgi
       Alexei Yakovlev
       Michael Seth
       ```
       File ends with `\n`.

    3. **Replace `crates/xcfun-rs/src/lib.rs`** with:
       ```rust
       //! xcfun-rs — native Rust public API for xcfun_rs (Phase 5).
       //!
       //! Stable Rust facade over `xcfun-eval::Functional`. Decouples the
       //! public surface from cubecl internals (Phase 5 D-02).
       //!
       //! # Public surface (RS-01..10)
       //! - `Functional` newtype + 8 methods.
       //! - 11 free functions: see [`free_fns`].
       //! - Re-exports of public types: [`Mode`], [`Vars`], [`XcError`],
       //!   [`ParameterId`], [`FunctionalId`], [`Dependency`].

       #![forbid(unsafe_code)]

       mod functional;
       mod free_fns;

       pub use functional::Functional;
       pub use free_fns::{
           authors, describe_long, describe_short, enumerate_aliases,
           enumerate_parameters, is_compatible_library, self_test, splash,
           version, which_mode, which_vars,
       };
       pub use xcfun_core::{Dependency, FunctionalId, Mode, ParameterId, Vars, XcError};
       ```

    4. **Create `crates/xcfun-rs/src/functional.rs`** with:
       ```rust
       //! Native Rust facade `Functional` (Phase 5 D-02).
       //!
       //! Newtype around `xcfun_eval::Functional`. Methods delegate. The
       //! field is private so callers cannot bypass `set` validation by
       //! mutating `weights` / `settings` directly.

       use xcfun_core::{Dependency, Mode, Vars, XcError};

       /// The exchange-correlation functional handle.
       ///
       /// RS-01..10 surface. Construct via [`Self::new`], then configure
       /// active functionals + parameters via [`Self::set`], then invoke
       /// [`Self::eval_setup`] before [`Self::eval`].
       #[derive(Debug)]
       pub struct Functional(xcfun_eval::Functional);

       impl Functional {
           /// RS-01 — fresh handle: no active functionals, parameters at
           /// their defaults (XCFunctional.cpp:350-355).
           pub const fn new() -> Self { Self(xcfun_eval::Functional::new()) }

           /// RS-02 — case-insensitive name set. Three-case dispatch
           /// (functional / parameter / alias) per XCFunctional.cpp:369-405.
           pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
               self.0.set(name, value)
           }

           /// RS-03 — read functional weight or parameter value.
           /// Aliases NOT supported (mirror XCFunctional.cpp:407-419).
           pub fn get(&self, name: &str) -> Result<f64, XcError> {
               self.0.get(name)
           }

           /// RS-04 — `(depends & XC_GRADIENT)` per XCFunctional.cpp:420.
           pub fn is_gga(&self) -> bool {
               self.0.dependencies().contains(Dependency::GRADIENT)
           }

           /// RS-04 — `(depends & (XC_LAPLACIAN | XC_KINETIC))` per
           /// XCFunctional.cpp:422-424.
           pub fn is_metagga(&self) -> bool {
               let d = self.0.dependencies();
               d.contains(Dependency::LAPLACIAN) || d.contains(Dependency::KINETIC)
           }

           /// RS-05 — validate `(vars, mode, order)` against the active
           /// functional set's dependencies; on success mutate the inner
           /// `vars`/`mode`/`order` so subsequent `input_length()` and
           /// `output_length()` reflect the new state.
           ///
           /// xcfun_eval::Functional::eval_setup is read-only; the field
           /// write happens here at the facade boundary so xcfun-eval's
           /// hot path stays untouched.
           pub fn eval_setup(
               &mut self,
               vars: Vars,
               mode: Mode,
               order: u32,
           ) -> Result<(), XcError> {
               // XCFunctional.cpp:438-441 — validate first; mutate only on success.
               self.0.eval_setup(vars, mode, order)?;
               self.0.vars  = vars;
               self.0.mode  = mode;
               self.0.order = order;
               Ok(())
           }

           /// RS-06 — host-program-friendly setup. Compose `which_vars` +
           /// `which_mode` then call `eval_setup`. Port of
           /// XCFunctional.cpp:472-485.
           ///
           /// Out-of-range numeric inputs (any of `func_type > 3`,
           /// `dens_type > 3`, `laplacian/kinetic/current/explicit_derivatives > 1`,
           /// or `mode_type ∈ {0, 4..}`) return `Err(XcError::InvalidEncoding)`
           /// — diverges from C++ which calls `xcfun::die`. The C ABI in
           /// Plan 05-02 maps this back to abort.
           pub fn user_eval_setup(
               &mut self,
               order: i32,
               func_type:            u32,
               dens_type:            u32,
               mode_type:            u32,
               laplacian:            u32,
               kinetic:              u32,
               current:              u32,
               explicit_derivatives: u32,
           ) -> Result<(), XcError> {
               let vars = crate::which_vars(
                   func_type, dens_type, laplacian, kinetic, current, explicit_derivatives,
               ).ok_or(XcError::InvalidEncoding)?;
               let mode = crate::which_mode(mode_type).ok_or(XcError::InvalidEncoding)?;
               if order < 0 {
                   return Err(XcError::InvalidOrder {
                       order: 0, mode, n_vars: Self::input_length_of(vars),
                   });
               }
               self.eval_setup(vars, mode, order as u32)
           }

           /// MODE-04 / RS-09 — number of `f64` inputs to `eval`.
           pub fn input_length(&self) -> usize {
               xcfun_eval::Functional::input_length(self.0.vars)
           }

           /// MODE-05 / RS-09 — number of `f64` outputs `eval` writes.
           pub fn output_length(&self) -> Result<usize, XcError> {
               xcfun_eval::Functional::output_length(
                   self.0.vars, self.0.mode, self.0.order,
               )
           }

           /// RS-07 — evaluate. Zero heap allocation on the success path;
           /// see `tests/zero_alloc.rs` for the verifying fixture.
           pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
               self.0.eval(input, output)
           }

           // -- internal helper used by `user_eval_setup` for the
           //    `InvalidOrder.n_vars` field -----------------------------
           #[inline]
           fn input_length_of(vars: Vars) -> usize {
               xcfun_eval::Functional::input_length(vars)
           }
       }

       impl Default for Functional {
           fn default() -> Self { Self::new() }
       }
       ```

    5. **Create `crates/xcfun-rs/tests/send_sync.rs`** with:
       ```rust
       //! RS-10 — `Functional` MUST be `Send + Sync`. Compile-time gate.
       use static_assertions::assert_impl_all;
       use xcfun_rs::Functional;
       assert_impl_all!(Functional: Send, Sync);
       ```

    Add **inline `#[cfg(test)] mod tests { ... }`** at the bottom of
    `functional.rs` covering all <behavior> bullets above. Each test
    constructs a `Functional`, exercises one bullet, asserts the result.
    Use `xcfun_core::Vars`, `xcfun_core::Mode`, `xcfun_core::XcError`
    (already re-exported).
  </action>
  <verify>
    <automated>cargo check -p xcfun-rs 2>&1 | tee /tmp/check_05_01a.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>cargo test -p xcfun-rs --lib functional::tests 2>&1 | tee /tmp/test_05_01a.log; grep -E "test result: ok\." /tmp/test_05_01a.log</automated>
    <automated>cargo test -p xcfun-rs --test send_sync 2>&1 | grep -E "(test result: ok|0 passed)"</automated>
    <automated>grep -nF "pub struct Functional(xcfun_eval::Functional)" crates/xcfun-rs/src/functional.rs</automated>
    <automated>grep -nF "pub use functional::Functional" crates/xcfun-rs/src/lib.rs</automated>
    <automated>test -f crates/xcfun-rs/assets/splash.txt && grep -F "XCFun DFT library Copyright" crates/xcfun-rs/assets/splash.txt</automated>
    <automated>test -f crates/xcfun-rs/assets/authors.txt && grep -F "XCFun was written by Ulf Ekstrom" crates/xcfun-rs/assets/authors.txt</automated>
  </verify>
  <done>
    - `crates/xcfun-rs/src/functional.rs` exists with `pub struct Functional(xcfun_eval::Functional)` (private inner field), 9 public methods + `Default` impl.
    - `crates/xcfun-rs/src/lib.rs` re-exports `Functional` and `xcfun_core::{Mode, Vars, XcError, ParameterId, FunctionalId, Dependency}`.
    - `crates/xcfun-rs/tests/send_sync.rs` compiles and runs.
    - `crates/xcfun-rs/assets/{splash,authors}.txt` exist with verbatim C++ content.
    - All inline functional::tests pass (≥10 tests covering the <behavior> bullets).
    - `cargo check -p xcfun-rs` exits 0; `cargo test -p xcfun-rs` (lib + send_sync test) exits 0.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 1.2: free functions module — version/splash/authors/which_vars/which_mode/enumerate_*/describe_*/self_test/is_compatible_library</name>
  <files>crates/xcfun-rs/src/free_fns.rs, crates/xcfun-rs/tests/free_fns.rs</files>
  <read_first>
    - crates/xcfun-rs/src/lib.rs (Task 1.1 output — must already declare `mod free_fns;` and the `pub use free_fns::*` line)
    - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (FunctionalDescriptor struct; lookup `name`, `short_description`, `long_description`)
    - crates/xcfun-core/src/registry/generated/parameters.rs (ParameterEntry struct; lookup `name`, `description`)
    - crates/xcfun-core/src/registry/generated/ALIASES.rs (Alias struct; lookup `name`, `description`)
    - crates/xcfun-core/src/functional_id.rs:93-115 (`FunctionalId::from_name` case-insensitive lookup pattern)
    - crates/xcfun-core/src/enums.rs:39-90 (`ParameterId::from_name` case-insensitive lookup pattern)
    - xcfun-master/src/XCFunctional.cpp:131-348 (which_vars + which_mode + enumerate_* + describe_* C++ source — verbatim port targets)
  </read_first>
  <behavior>
    - `version()` returns `env!("CARGO_PKG_VERSION")` (must be a non-empty `&'static str` whose first char is `'0'`..`'9'`).
    - `splash()` returns the contents of `assets/splash.txt` via `include_str!` — first line begins with `"XCFun DFT library Copyright 2009-2020 Ulf Ekstrom"`.
    - `authors()` returns the contents of `assets/authors.txt` — first line begins with `"XCFun was written by Ulf Ekstrom"`.
    - `is_compatible_library()` returns `true` (single-process build per CONTEXT D-03 implication; matches XCFunctional.cpp:126-129 same-major-version check trivially when major comes from the same crate).
    - `self_test()` returns `0` for now (Phase 5 may ship a small subset later — Claude's Discretion per CONTEXT). Choose: iterate the 5 LDAs that have `test_in.is_some()` in `FUNCTIONAL_DESCRIPTORS`, run a Mode::PartialDerivatives order-0 eval, count failures relative to `test_threshold`. Return total failure count.
    - `which_vars(0, 0, 0, 0, 0, 0) == Some(Vars::A)`; `which_vars(0, 1, 0, 0, 0, 0) == Some(Vars::N)`; `which_vars(0, 2, 0, 0, 0, 0) == Some(Vars::A_B)`; `which_vars(0, 3, 0, 0, 0, 0) == Some(Vars::N_S)`; `which_vars(1, 2, 0, 0, 0, 0) == Some(Vars::A_B_GAA_GAB_GBB)`; `which_vars(1, 2, 0, 0, 0, 1) == Some(Vars::A_B_AX_AY_AZ_BX_BY_BZ)`; `which_vars(2, 2, 0, 1, 0, 0) == Some(Vars::A_B_GAA_GAB_GBB_TAUA_TAUB)`; `which_vars(3, 2, 0, 0, 0, 0) == Some(Vars::A_B_2ND_TAYLOR)`.
    - `which_vars(4, 0, 0, 0, 0, 0) == None` (func_type out of range).
    - `which_vars(0, 4, 0, 0, 0, 0) == None` (dens_type out of range).
    - `which_vars(0, 0, 2, 0, 0, 0) == None` (laplacian out of range).
    - `which_vars(0, 1, 1, 0, 0, 0) == None` (no `case 24` in C++ table — unmapped bitwise_vars).
    - `which_mode(1) == Some(Mode::PartialDerivatives)`; `which_mode(2) == Some(Mode::Potential)`; `which_mode(3) == Some(Mode::Contracted)`.
    - `which_mode(0) == None`; `which_mode(4) == None`.
    - `enumerate_parameters(0)` returns `Some(FUNCTIONAL_DESCRIPTORS[0].name)` — i.e. `Some("XC_SLATERX")`.
    - `enumerate_parameters(77)` returns `Some(FUNCTIONAL_DESCRIPTORS[77].name)`.
    - `enumerate_parameters(78)` returns `Some(PARAMETERS[0].name)` — i.e. `Some("RANGESEP_MU")`.
    - `enumerate_parameters(81)` returns `Some(PARAMETERS[3].name)` — i.e. `Some("CAM_BETA")`.
    - `enumerate_parameters(82) == None`; `enumerate_parameters(-1) == None`.
    - `enumerate_aliases(0)` returns `Some("null")`; `enumerate_aliases(46) == None`; `enumerate_aliases(-1) == None`.
    - `describe_short("SLATERX") == Some("Slater LDA exchange")` (from FUNCTIONAL_DESCRIPTORS).
    - `describe_short("slaterx") == Some("Slater LDA exchange")` (case-insensitive).
    - `describe_short("RANGESEP_MU")` returns `Some("Range separation inverse length [1/a0]")` (from PARAMETERS).
    - `describe_short("BLYP")` returns `Some(<ALIASES BLYP description>)`.
    - `describe_short("not_a_known_thing") == None`.
    - `describe_long("SLATERX")` returns the `long_description` for SLATERX (multi-line string).
  </behavior>
  <action>
    1. **Create `crates/xcfun-rs/src/free_fns.rs`** with EXACTLY this skeleton (fill in the `which_vars` switch table from the C++ source verbatim — every one of the 31 cases listed in `<interfaces>` above):
       ```rust
       //! 11 free functions per RS-09 + Phase 5 D-03.
       //!
       //! Source-of-truth file references in each fn doc-comment.

       use xcfun_core::{
           ALIASES, FUNCTIONAL_DESCRIPTORS, FunctionalId, Mode, PARAMETERS,
           ParameterId, Vars,
       };

       /// RS-09 — version string. Mirrors xcfun_version (XCFunctional.cpp:48-51).
       pub fn version() -> &'static str {
           env!("CARGO_PKG_VERSION")
       }

       /// RS-09 — splash. Mirrors xcfun_splash (XCFunctional.cpp:53-62).
       pub fn splash() -> &'static str {
           include_str!("../assets/splash.txt")
       }

       /// RS-09 — authors. Mirrors xcfun_authors (XCFunctional.cpp:64-72).
       pub fn authors() -> &'static str {
           include_str!("../assets/authors.txt")
       }

       /// RS-09 — single-process header/library compatibility.
       /// Always `true` because the header is generated from THIS crate
       /// (Phase 5 Plan 05-03). Mirrors xcfun_is_compatible_library
       /// (XCFunctional.cpp:126-129).
       pub fn is_compatible_library() -> bool {
           true
       }

       /// RS-09 — run Tier-1 self-tests over functionals carrying upstream
       /// `test_in` data. Returns failure count. Mirrors xcfun_test
       /// (XCFunctional.cpp:74-124) but limited to the populated subset
       /// (Claude's Discretion per CONTEXT — keep smoke-test fast).
       pub fn self_test() -> i32 {
           let mut nfail: i32 = 0;
           for fd in FUNCTIONAL_DESCRIPTORS.iter() {
               if let (Some(test_in), Some(test_out), Some(test_vars), Some(test_mode),
                       Some(test_order), Some(test_threshold))
                   = (fd.test_in, fd.test_out, fd.test_vars, fd.test_mode,
                      fd.test_order, fd.test_threshold)
               {
                   let mut fun = crate::Functional::new();
                   if fun.set(fd.name, 1.0).is_err() {
                       nfail += 1; continue;
                   }
                   if fun.eval_setup(test_vars, test_mode, test_order).is_err() {
                       nfail += 1; continue;
                   }
                   let outlen = match fun.output_length() { Ok(n) => n, Err(_) => { nfail += 1; continue; } };
                   if outlen != test_out.len() { nfail += 1; continue; }
                   let mut out = vec![0.0_f64; outlen];
                   if fun.eval(test_in, &mut out).is_err() { nfail += 1; continue; }
                   for (computed, reference) in out.iter().zip(test_out.iter()) {
                       if (computed - reference).abs() > reference.abs() * test_threshold {
                           nfail += 1;
                       }
                   }
               }
           }
           nfail
       }

       /// RS-09 — bitwise dispatch port of XCFunctional.cpp:131-277.
       /// Out-of-range inputs return `None` (instead of C++ `xcfun::die`).
       pub fn which_vars(
           func_type:            u32,
           dens_type:            u32,
           laplacian:            u32,
           kinetic:              u32,
           current:              u32,
           explicit_derivatives: u32,
       ) -> Option<Vars> {
           if func_type > 3 || dens_type > 3
               || laplacian > 1 || kinetic > 1
               || current > 1 || explicit_derivatives > 1
           {
               return None;
           }
           let bw = (func_type << 6)
                  | (dens_type << 4)
                  | (laplacian << 3)
                  | (kinetic   << 2)
                  | (current   << 1)
                  | explicit_derivatives;
           Some(match bw {
               0   => Vars::A,
               16  => Vars::N,
               32  => Vars::A_B,
               48  => Vars::N_S,
               64  => Vars::A_GAA,
               65  => Vars::A_AX_AY_AZ,
               80  => Vars::N_GNN,
               81  => Vars::N_NX_NY_NZ,
               96  => Vars::A_B_GAA_GAB_GBB,
               97  => Vars::A_B_AX_AY_AZ_BX_BY_BZ,
               112 => Vars::N_S_GNN_GNS_GSS,
               113 => Vars::N_S_NX_NY_NZ_SX_SY_SZ,
               132 => Vars::A_GAA_TAUA,
               133 => Vars::A_AX_AY_AZ_TAUA,
               136 => Vars::A_GAA_LAPA,
               148 => Vars::N_GNN_TAUN,
               149 => Vars::N_NX_NY_NZ_TAUN,
               152 => Vars::N_GNN_LAPN,
               164 => Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
               165 => Vars::A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB,
               168 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB,
               172 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB,
               174 => Vars::A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB,
               180 => Vars::N_S_GNN_GNS_GSS_TAUN_TAUS,
               181 => Vars::N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS,
               184 => Vars::N_S_GNN_GNS_GSS_LAPN_LAPS,
               188 => Vars::N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS,
               192 => Vars::A_2ND_TAYLOR,
               208 => Vars::N_2ND_TAYLOR,
               224 => Vars::A_B_2ND_TAYLOR,
               240 => Vars::N_S_2ND_TAYLOR,
               _   => return None,
           })
       }

       /// RS-09 — port of XCFunctional.cpp:281-300.
       pub fn which_mode(mode_type: u32) -> Option<Mode> {
           match mode_type {
               1 => Some(Mode::PartialDerivatives),
               2 => Some(Mode::Potential),
               3 => Some(Mode::Contracted),
               _ => None,
           }
       }

       /// RS-09 — port of XCFunctional.cpp:302-311.
       /// Indices 0..78 → functional names; 78..82 → parameter names.
       pub fn enumerate_parameters(param: i32) -> Option<&'static str> {
           if param < 0 { return None; }
           let i = param as usize;
           if i < FUNCTIONAL_DESCRIPTORS.len() {
               Some(FUNCTIONAL_DESCRIPTORS[i].name)
           } else if i < FUNCTIONAL_DESCRIPTORS.len() + PARAMETERS.len() {
               Some(PARAMETERS[i - FUNCTIONAL_DESCRIPTORS.len()].name)
           } else {
               None
           }
       }

       /// RS-09 — port of XCFunctional.cpp:313-320.
       pub fn enumerate_aliases(n: i32) -> Option<&'static str> {
           if n < 0 { return None; }
           ALIASES.get(n as usize).map(|a| a.name)
       }

       /// RS-09 — case-insensitive 3-table cascade. Port of
       /// XCFunctional.cpp:322-334.
       pub fn describe_short(name: &str) -> Option<&'static str> {
           if let Some(id) = FunctionalId::from_name(name) {
               return Some(FUNCTIONAL_DESCRIPTORS[id as usize].short_description);
           }
           if let Some(pid) = ParameterId::from_name(name) {
               // ParameterId discriminants are 78..81; map back to PARAMETERS index.
               let off = (pid as usize) - FUNCTIONAL_DESCRIPTORS.len();
               return Some(PARAMETERS[off].description);
           }
           if let Some(alias) = ALIASES.iter().find(|a| a.name.eq_ignore_ascii_case(name)) {
               return Some(alias.description);
           }
           None
       }

       /// RS-09 — port of XCFunctional.cpp:336-348. Identical 3-table cascade
       /// to `describe_short` except returns `long_description` for the
       /// functional case.
       pub fn describe_long(name: &str) -> Option<&'static str> {
           if let Some(id) = FunctionalId::from_name(name) {
               return Some(FUNCTIONAL_DESCRIPTORS[id as usize].long_description);
           }
           if let Some(pid) = ParameterId::from_name(name) {
               let off = (pid as usize) - FUNCTIONAL_DESCRIPTORS.len();
               return Some(PARAMETERS[off].description);
           }
           if let Some(alias) = ALIASES.iter().find(|a| a.name.eq_ignore_ascii_case(name)) {
               return Some(alias.description);
           }
           None
       }
       ```

    2. **Create `crates/xcfun-rs/tests/free_fns.rs`** with one test per `<behavior>` bullet — minimum 23 tests:
       ```rust
       //! Free-function behaviour parity with XCFunctional.cpp:48-348.
       use xcfun_rs::*;

       #[test] fn version_returns_crate_version() {
           let v = version();
           assert!(!v.is_empty());
           assert!(v.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false), "got {v}");
       }
       #[test] fn splash_starts_with_xcfun_dft_library() {
           assert!(splash().starts_with("XCFun DFT library Copyright 2009-2020 Ulf Ekstrom"));
       }
       #[test] fn authors_starts_with_written_by() {
           assert!(authors().starts_with("XCFun was written by Ulf Ekstrom"));
       }
       #[test] fn compat_returns_true() { assert!(is_compatible_library()); }
       // ... 19 more tests covering every <behavior> bullet ...
       ```
       At least one test per <behavior> bullet for which_vars, which_mode,
       enumerate_parameters, enumerate_aliases, describe_short, describe_long.
  </action>
  <verify>
    <automated>cargo test -p xcfun-rs --test free_fns 2>&1 | tee /tmp/test_05_01b.log; grep -E "test result: ok\." /tmp/test_05_01b.log</automated>
    <automated>cargo test -p xcfun-rs --test free_fns -- which_vars 2>&1 | grep -E "test result: ok"</automated>
    <automated>cargo test -p xcfun-rs --test free_fns -- enumerate 2>&1 | grep -E "test result: ok"</automated>
    <automated>cargo test -p xcfun-rs --test free_fns -- describe 2>&1 | grep -E "test result: ok"</automated>
    <automated>grep -cE "^pub fn (version|splash|authors|self_test|is_compatible_library|which_vars|which_mode|enumerate_parameters|enumerate_aliases|describe_short|describe_long)\b" crates/xcfun-rs/src/free_fns.rs | grep -E "^11$"</automated>
    <automated>grep -cE "^\s*(0|16|32|48|64|65|80|81|96|97|112|113|132|133|136|148|149|152|164|165|168|172|174|180|181|184|188|192|208|224|240)\s*=>" crates/xcfun-rs/src/free_fns.rs | grep -E "^31$"</automated>
  </verify>
  <done>
    - `crates/xcfun-rs/src/free_fns.rs` defines all 11 free functions with bodies as specified.
    - The `which_vars` match has all 31 cases from XCFunctional.cpp:131-277 verbatim (verified by counting `=> Vars::` arms).
    - `crates/xcfun-rs/tests/free_fns.rs` runs ≥ 23 tests, all pass.
    - `cargo test -p xcfun-rs --test free_fns` exits 0.
  </done>
</task>

<task type="auto">
  <name>Task 1.3: zero-allocation hot-path test (RS-07, D-13)</name>
  <files>crates/xcfun-rs/tests/zero_alloc.rs</files>
  <read_first>
    - crates/xcfun-rs/src/functional.rs (Task 1.1 output — `Functional::eval` signature)
    - crates/xcfun-eval/src/functional.rs:234-339 (existing eval body — observe the per-launch `cpu_client().create_from_slice(...)` call which allocates inside cubecl-cpu — this is the known risk per 05-PATTERNS.md A.3 line 324)
    - crates/xcfun-eval/tests/regularize_invariant.rs (analog: integration test that exercises eval, asserts post-condition)
  </read_first>
  <action>
    Create `crates/xcfun-rs/tests/zero_alloc.rs` with EXACTLY this content:
    ```rust
    //! RS-07 + Phase 5 D-13 — verify `Functional::eval` performs zero
    //! heap allocation on the success path AT THE FACADE BOUNDARY.
    //!
    //! NOTE: this test counts allocations made through the global allocator
    //! during repeated `eval` calls AFTER an initial warm-up run. The
    //! warm-up exists to amortize one-time lazy initialisations
    //! (`OnceLock<CpuClient>` in cubecl-cpu, etc.). The test then asserts
    //! that 100 subsequent evals contribute ZERO additional global-allocator
    //! allocations.
    //!
    //! If cubecl-cpu's per-launch `create_from_slice` triggers allocations
    //! (a known risk per 05-PATTERNS.md §A.3), this test will fail and
    //! the failure must be triaged: either (a) revise xcfun-eval's eval
    //! to use a pre-allocated reusable handle, or (b) document the
    //! cubecl-cpu allocation as a Phase 6 concern and relax this test
    //! to count allocations made by the FACADE wrapper only (i.e. wrap
    //! the `Functional::eval` call only, not the inner cubecl launch).
    //!
    //! The test as-written checks the strictest interpretation: zero
    //! global-allocator allocations during the steady-state eval loop.
    //! If reality is option (b), execute Plan 05-01 with this test
    //! present and observe the failure mode; the gap-closure path is
    //! documented in 05-CONTEXT.md D-13.

    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingAllocator;
    static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
            unsafe { System.alloc(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) };
        }
    }

    #[global_allocator]
    static ALLOC: CountingAllocator = CountingAllocator;

    #[test]
    fn eval_is_zero_alloc_on_hot_path() {
        use xcfun_rs::{Functional, Mode, Vars};

        let mut f = Functional::new();
        f.set("slaterx", 1.0).unwrap();
        f.eval_setup(Vars::A_B, Mode::PartialDerivatives, 0).unwrap();

        let outlen = f.output_length().unwrap();
        let inlen  = f.input_length();
        let mut input  = vec![0.5_f64; inlen];
        let mut output = vec![0.0_f64; outlen];

        // Warm up — trigger any lazy statics (cubecl-cpu OnceLock<CpuClient>).
        f.eval(&input, &mut output).unwrap();

        let baseline = ALLOC_COUNT.load(Ordering::SeqCst);
        for k in 0..100_usize {
            input[0] = 0.5 + (k as f64) * 0.001;
            input[1] = 0.5 - (k as f64) * 0.001;
            f.eval(&input, &mut output).unwrap();
        }
        let delta = ALLOC_COUNT.load(Ordering::SeqCst) - baseline;
        assert_eq!(
            delta, 0,
            "expected zero allocations on hot path, observed {delta}",
        );
    }
    ```

    **Important risk note:** per 05-PATTERNS.md §A.3, cubecl-cpu's
    `client.create_from_slice` may allocate per-launch. If `cargo test
    --test zero_alloc` fails with `delta > 0`:

    1. Capture the failure (delta value, allocation count).
    2. STOP and post a comment on the executor's task summary in
       `05-01-SUMMARY.md` flagging the failure.
    3. Either: (a) revise `xcfun_eval::Functional::eval` to use a
       pre-allocated `&'a CpuClient` handle and rerun, or (b) relax this
       test (per the doc-comment) to use a scope-local allocation counter
       that wraps only the facade boundary (not inner cubecl).

    The plan ASSUMES (a) succeeds; (b) is the documented fall-back to
    avoid blocking Phase 5 sign-off on a Phase 6 concern.
  </action>
  <verify>
    <automated>cargo test -p xcfun-rs --test zero_alloc 2>&1 | tee /tmp/test_05_01c.log; grep -E "test result: ok|test eval_is_zero_alloc_on_hot_path \.\.\. ok" /tmp/test_05_01c.log</automated>
    <automated>grep -nF "ALLOC_COUNT" crates/xcfun-rs/tests/zero_alloc.rs</automated>
    <automated>grep -nF "#[global_allocator]" crates/xcfun-rs/tests/zero_alloc.rs</automated>
  </verify>
  <done>
    - `crates/xcfun-rs/tests/zero_alloc.rs` exists with the counting allocator.
    - `cargo test -p xcfun-rs --test zero_alloc` exits 0 (delta == 0). If
      it fails per the inline risk note, the executor follows the
      documented (b) fall-back AND records the path chosen in 05-01-SUMMARY.md.
  </done>
</task>

</tasks>

<verification>
Run after all tasks complete:

```bash
# All xcfun-rs tests
cargo test -p xcfun-rs

# Public surface compiles
cargo check -p xcfun-rs

# Send + Sync gate
cargo test -p xcfun-rs --test send_sync

# Free-fn parity
cargo test -p xcfun-rs --test free_fns

# Zero alloc
cargo test -p xcfun-rs --test zero_alloc

# Surface check — 11 free fns + 1 Functional struct + 6 re-exports
grep -cE "^pub fn |^pub use |^pub struct " crates/xcfun-rs/src/lib.rs crates/xcfun-rs/src/free_fns.rs crates/xcfun-rs/src/functional.rs

# No anyhow leak
! grep -rE "use anyhow|anyhow::" crates/xcfun-rs/src/
```
</verification>

<success_criteria>
- `Functional` newtype + 9 methods + Default impl exist with private inner field.
- 11 free functions exist at crate root, behaviour parity with C++ verified by ≥ 23 unit tests.
- `Send + Sync` compile-time gate passes.
- Zero-allocation hot-path test passes (or documented fall-back applied per Task 1.3 risk note).
- `cargo test -p xcfun-rs` exits 0 (lib + 3 integration tests).
- No `anyhow` import in `crates/xcfun-rs/src/`.
- All requirements RS-01..07, RS-09, RS-10 wired through to a verifiable test.
</success_criteria>

<output>
After completion, create `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-01-SUMMARY.md` documenting:
- File listing (Cargo.toml, lib.rs, functional.rs, free_fns.rs, 3 tests, 2 assets).
- Test counts: number of inline functional::tests, free_fns tests, the zero_alloc test outcome.
- If zero_alloc fall-back (b) was used, note WHY (cubecl-cpu allocation
  observed) and link to a Phase 6 follow-up.
- Confirmation that `Functional` is `Send + Sync`.
- Confirmation that re-exports cover `Mode, Vars, XcError, ParameterId, FunctionalId, Dependency`.
</output>
