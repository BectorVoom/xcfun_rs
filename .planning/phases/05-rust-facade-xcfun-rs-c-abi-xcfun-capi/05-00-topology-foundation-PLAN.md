---
plan_id: 05-00-topology-foundation
phase: 05
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/xcfun-ffi/Cargo.toml
  - crates/xcfun-ffi/src/lib.rs
  - crates/xcfun-capi/Cargo.toml
  - crates/xcfun-capi/src/lib.rs
  - crates/xcfun-functionals/Cargo.toml
  - crates/xcfun-functionals/src/lib.rs
  - crates/xcfun-core/src/error.rs
  - crates/xcfun-core/src/functional_id.rs
  - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs
  - crates/xcfun-core/tests/registry_tables.rs
  - xtask/src/bin/regen_registry.rs
  - crates/xcfun-eval/src/functional.rs
  - crates/xcfun-eval/src/functionals/mgga/mod.rs
requirements:
  - CAPI-05
  - RS-09
  - RS-10
autonomous: true
---

## objective
<objective>
Phase 5 Wave 1 — establish the workspace topology, shared error-code surface,
and registry coverage for LB94 (D-16) that all downstream Phase 5 plans depend on:

1. **Rename `crates/xcfun-ffi` → `crates/xcfun-capi`** per D-01. Update workspace
   `members`/`exclude`, package name, and stub `lib.rs` so subsequent plans can
   target the new crate path.
2. **Delete `crates/xcfun-functionals`** per D-04 (dead post-cubecl-pivot stub).
3. **Promote `xcfun-rs` and `xcfun-capi` to workspace members.** Create empty
   `xcfun-rs` skeleton (Cargo.toml + lib.rs) so Plan 05-01 fills it in.
4. **Add `XcError::InvalidVarsAndMode` variant + `as_c_code()` method** per
   D-08-A so the C ABI can map `eval_setup` error returns to the upstream
   `XC_E*` codes (1 / 2 / 4 / 6 / -1) verbatim.
5. **Modify `xcfun_eval::Functional::eval_setup`** so the combined
   `(InvalidVars + InvalidMode)` case at `XCFunctional.cpp:442` (`return
   XC_EVARS | XC_EMODE`) is reachable through the typed Rust error.
6. **Add the LB94 descriptor to xcfun-core per D-16.** Verification of the
   current codebase (`crates/xcfun-core/src/functional_id.rs` lines 12-91 and
   `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs`,
   78-entry table) confirms LB94 is **absent** from `FunctionalId` and from
   the descriptor registry. The Phase 4 comment at
   `crates/xcfun-eval/src/functionals/mgga/mod.rs:4` explicitly defers LB94 to
   Phase 5. D-16 is conditional ("if not, Phase 5 adds it") and the condition
   is met — Phase 5 must add the descriptor. Task 0.4 adds:
   - `XC_LB94 = 78` to `FunctionalId` (becomes the 79th id; `COUNT` bumps to 79).
   - `XC_LB94` entry to the xtask `FUNCTIONAL_IDS` list with an exclusion in
     `emit_c_stubs_cpp` so `validation/c_stubs.cpp` does NOT emit
     `FUNCTIONAL(XC_LB94)` (the upstream `list_of_functionals.hpp` lacks
     `XC_LB94` because `lb94.cpp` is `#if 0`'d, so the C++ stub would fail
     to compile).
   - A regenerated `FUNCTIONAL_DESCRIPTORS.rs` with a stub LB94 entry
     (`Dependency::DENSITY | GRADIENT`, matching the `setup_lb94` macro
     declaration in the upstream `lb94.cpp:50` where `f.describe(XC_LB94,
     XC_GGA, ...)` is wrapped in `#if 0`).
   - The eval path for LB94 returns `XcError::Runtime` per Plan 05-01 wiring
     (the upstream body is `#if 0`'d and not evaluable). This is the
     discretion area noted by the checker — `Runtime` is the cleanest
     signal that "the descriptor exists but the body is not implemented",
     mapping to C ABI return code -1 via `as_c_code`. NO new XcError
     variant is added (the 9-variant + 1 InvalidVarsAndMode set is locked
     by Phase 4 D-14).
   - Test counts updated: `FunctionalId::COUNT == 79` (from 78);
     `FUNCTIONAL_DESCRIPTORS.len() == 79` (from 78); the existing
     `parameter_id_discriminants_match_cpp` test stays unchanged (the C++
     `XC_RANGESEP_MU = XC_NR_FUNCTIONALS = 78` invariant is independent of
     the Rust-side LB94 add-back; in upstream C++ XC_LB94 doesn't exist).
   - The Phase 4 comment at `mgga/mod.rs:4` is updated to reflect that
     LB94 is now in `FunctionalId` (with id 78, NOT 66 — the original
     comment was factually incorrect, id 66 is `XC_CSC`).

Purpose: every Phase 5 deliverable below this plan needs (a) the renamed crate
to exist, (b) `XcError::as_c_code` to compile, (c) the `eval_setup`
combined error to be observable from the facade, and (d) the LB94
descriptor present so D-14 row 10 references a registry-resident
functional even though its eval path returns Runtime.

Output: workspace topology aligned with Phase 5 D-01/D-04, `XcError` enum
extended with one new variant + one new method, `eval_setup` returning the
combined variant when both vars-vs-depends and mode-vs-vars predicates trip,
and `FunctionalId::XC_LB94` + descriptor present in xcfun-core per D-16.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-PATTERNS.md

# Phase 5 source-of-truth files
@xcfun-master/api/xcfun.h
@xcfun-master/src/XCFunctional.cpp
@xcfun-master/src/functionals/lb94.cpp
@xcfun-master/src/functionals/list_of_functionals.hpp

# Existing Rust files this plan modifies
@Cargo.toml
@crates/xcfun-core/src/error.rs
@crates/xcfun-core/src/functional_id.rs
@crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs
@crates/xcfun-core/src/lib.rs
@crates/xcfun-core/tests/registry_tables.rs
@xtask/src/bin/regen_registry.rs
@crates/xcfun-eval/src/functional.rs
@crates/xcfun-eval/src/functionals/mgga/mod.rs

# Existing stubs being renamed/deleted
@crates/xcfun-ffi/Cargo.toml
@crates/xcfun-ffi/src/lib.rs

<interfaces>
<!-- Key types the executor needs -->

From crates/xcfun-core/src/error.rs (existing):
```rust
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[non_exhaustive]
pub enum XcError {
    InvalidOrder { order: u32, mode: Mode, n_vars: usize },
    InvalidVars  { vars: Vars, required: Dependency },
    InvalidMode  { mode: Mode, depends: Dependency },
    UnknownName,
    InputLengthMismatch  { expected: usize, got: usize },
    OutputLengthMismatch { expected: usize, got: usize },
    NotConfigured,
    InvalidEncoding,
    Runtime,
}
```

From crates/xcfun-core/src/functional_id.rs (existing — Task 0.4 extends):
```rust
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FunctionalId {
    XC_SLATERX = 0,
    // ... 76 entries ...
    XC_PW91C = 77,
    // Task 0.4 appends: XC_LB94 = 78
}
impl FunctionalId {
    pub const COUNT: usize = 78;       // Task 0.4 bumps to 79
    pub fn from_name(name: &str) -> Option<Self> { /* match arms */ }
}
```

From xcfun-master/src/functionals/lb94.cpp:15 (verifies upstream non-evaluability):
```cpp
#if 0 // Does not work and should maybe not be here in the first place
// ... entire LB94 body wrapped in #if 0 ...
#endif
```

From xcfun-master/src/functionals/lb94.cpp:48-52 (the would-be descriptor — used to extract `Dependency::DENSITY | GRADIENT`):
```cpp
void setup_lb94(functional &f) {
  f.describe(XC_LB94, XC_GGA,
             "LB94 Exchange-correlation functional",
             /* ... */);
  // ...
}
```

From xcfun-master/src/functionals/list_of_functionals.hpp:15-105 (verifies upstream lacks XC_LB94 in enum):
```cpp
enum xcfun_functional_id {
  XC_SLATERX,        // 0
  // ... 76 entries (no XC_LB94) ...
  XC_PW91C,          // 77
  XC_NR_FUNCTIONALS  // 78
};
enum xc_parameter {
  XC_RANGESEP_MU = XC_NR_FUNCTIONALS,  // = 78 in upstream
  // ...
};
```

From xcfun-master/src/XCFunctional.cpp:437-470 (`xcfun_eval_setup`):
```cpp
int xcfun_eval_setup(XCFunctional * fun, xcfun_vars vars, xcfun_mode mode, int order) {
  if ((fun->depends & xcint_vars[vars].provides) != fun->depends) {
    return xcfun::XC_EVARS;          // 2
  }
  if ((order < 0 || order > XCFUN_MAX_ORDER) ||
      (mode == XC_PARTIAL_DERIVATIVES && order > 4))
    return xcfun::XC_EORDER;         // 1
  if (mode == XC_POTENTIAL) {
    if ((fun->depends & XC_GRADIENT) &&
        !(vars == XC_A_2ND_TAYLOR || vars == XC_A_B_2ND_TAYLOR ||
          vars == XC_N_2ND_TAYLOR || vars == XC_N_S_2ND_TAYLOR)) {
      return xcfun::XC_EVARS | xcfun::XC_EMODE;   // 6  <-- combined case
    }
    if (fun->depends & (XC_LAPLACIAN | XC_KINETIC))
      return xcfun::XC_EMODE;        // 4
  }
  fun->mode = mode; fun->vars = vars; fun->order = order;
  return 0;
}
```

From xcfun-master/src/XCFunctional.hpp:40-46 (XC_E* constants):
```cpp
constexpr auto XC_EORDER = 1;
constexpr auto XC_EVARS  = 2;
constexpr auto XC_EMODE  = 4;
```
</interfaces>
</context>

## must_haves
<must_haves>
truths:
  - "Workspace `cargo metadata` lists `xcfun-rs` and `xcfun-capi` as members; lists no `xcfun-ffi` or `xcfun-functionals` member or excluded entry. (D-01, D-04)"
  - "`crates/xcfun-ffi/` and `crates/xcfun-functionals/` directories no longer exist on disk. (D-01, D-04)"
  - "`crates/xcfun-capi/Cargo.toml` declares `name = \"xcfun-capi\"`. (D-01)"
  - "`crates/xcfun-rs/Cargo.toml` exists with `name = \"xcfun-rs\"`; the directory builds (`cargo check -p xcfun-rs`) even with an empty lib.rs. (D-02)"
  - "`XcError` enum has a new variant `InvalidVarsAndMode { vars, mode, depends }` accessible from `xcfun_core::XcError`. (D-08-A)"
  - "`XcError::as_c_code(&self) -> i32` returns: `0` is N/A (success path is `Ok`), `InvalidOrder→1`, `InvalidVars→2`, `InvalidMode→4`, `InvalidVarsAndMode→6`, `UnknownName / InputLengthMismatch / OutputLengthMismatch / NotConfigured / InvalidEncoding / Runtime → -1`. (CAPI-05, D-08-A)"
  - "`Functional::eval_setup` returns `Err(XcError::InvalidVarsAndMode { ... })` when `mode == Mode::Potential`, the active functional set has `Dependency::GRADIENT`, and `vars` is NOT one of the four `_2ND_TAYLOR` arms — replicating `XCFunctional.cpp:441-444`. (D-08-A)"
  - "`FunctionalId::XC_LB94 = 78` exists; `FunctionalId::COUNT == 79`; `FunctionalId::from_name(\"lb94\")` returns `Some(FunctionalId::XC_LB94)`. (D-16)"
  - "`FUNCTIONAL_DESCRIPTORS` (in `crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs`) has 79 entries; row 78 is the LB94 stub with `name = \"XC_LB94\"`, `depends = Dependency::DENSITY | Dependency::GRADIENT`. (D-16)"
  - "`validation/c_stubs.cpp` does NOT contain `FUNCTIONAL(XC_LB94)` (upstream `list_of_functionals.hpp` lacks the symbol; emitting it would fail to compile). The xtask `emit_c_stubs_cpp` excludes LB94. (D-16)"
  - "The `parameter_id_discriminants_match_cpp` test (`crates/xcfun-core/tests/parameter_and_alias_registry.rs:23-36`) STILL passes with `XC_RANGESEP_MU as u32 == 78`, etc. (D-16 — parameter discriminants are independent of the Rust-side LB94 add-back; in upstream C++ XC_LB94 does not exist.)"
artifacts:
  - path: "Cargo.toml"
    provides: "Workspace member list updated; xcfun-rs + xcfun-capi added; xcfun-ffi + xcfun-functionals removed from both members and exclude lists"
    contains: "crates/xcfun-rs"
  - path: "crates/xcfun-capi/Cargo.toml"
    provides: "Renamed package manifest"
    contains: "name = \"xcfun-capi\""
  - path: "crates/xcfun-rs/Cargo.toml"
    provides: "New facade crate manifest (Plan 05-01 fills body)"
    contains: "name = \"xcfun-rs\""
  - path: "crates/xcfun-rs/src/lib.rs"
    provides: "Empty lib.rs so the crate compiles; Plan 05-01 fills it"
  - path: "crates/xcfun-core/src/error.rs"
    provides: "XcError::InvalidVarsAndMode variant + as_c_code() method"
    contains: "pub fn as_c_code"
  - path: "crates/xcfun-core/src/functional_id.rs"
    provides: "FunctionalId enum extended with XC_LB94 = 78 (D-16); COUNT bumped to 79; from_name handles 'lb94'"
    contains: "XC_LB94"
  - path: "crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs"
    provides: "Auto-regenerated 79-entry table including LB94 stub (D-16)"
    contains: "XC_LB94"
  - path: "crates/xcfun-core/tests/registry_tables.rs"
    provides: "descriptors_count_is_78 → descriptors_count_is_79"
    contains: "79"
  - path: "xtask/src/bin/regen_registry.rs"
    provides: "FUNCTIONAL_IDS list extended with XC_LB94; emit_c_stubs_cpp excludes LB94 (D-16)"
    contains: "XC_LB94"
  - path: "crates/xcfun-eval/src/functional.rs"
    provides: "eval_setup returns InvalidVarsAndMode when both predicates trip"
    contains: "InvalidVarsAndMode"
  - path: "crates/xcfun-eval/src/functionals/mgga/mod.rs"
    provides: "Updated comment removing the factually-incorrect 'LB94 (id=66)' note (D-16)"
    contains: "LB94"
key_links:
  - from: "crates/xcfun-eval/src/functional.rs (eval_setup)"
    to: "XcError::InvalidVarsAndMode"
    via: "Err return at the GGA-non-2ND_TAYLOR branch"
    pattern: "InvalidVarsAndMode \\{"
  - from: "Cargo.toml workspace members"
    to: "crates/xcfun-rs and crates/xcfun-capi"
    via: "string entries in members array"
    pattern: "crates/xcfun-rs|crates/xcfun-capi"
  - from: "crates/xcfun-core/src/functional_id.rs"
    to: "crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs"
    via: "FunctionalId::XC_LB94 = 78 indexes row 78 of the descriptor array"
    pattern: "XC_LB94"
  - from: "xtask/src/bin/regen_registry.rs (FUNCTIONAL_IDS)"
    to: "crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs"
    via: "regen-registry binary writes FUNCTIONAL_DESCRIPTORS from FUNCTIONAL_IDS"
    pattern: "XC_LB94"
</must_haves>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| n/a | This plan operates entirely inside the workspace; no FFI surface created yet. |

## STRIDE Threat Register

Phase 5 threat surface centres on the C ABI. This plan creates the *foundation*
(workspace + error-code mapping + LB94 descriptor presence) and does NOT itself
add unsafe code. Threats listed for completeness; mitigation is structural
(kept inside Rust-only crates with `#![forbid(unsafe_code)]`).

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-05-00-01 | Information disclosure | `XcError` Display impl | accept | thiserror messages contain only enum field values (no addresses, no secrets); Rust panic messages are caller-side only. |
| T-05-00-02 | Tampering | Workspace member list (Cargo.toml) | mitigate | Member entries committed to git; CI `cargo metadata` runs on every PR. |
| T-05-00-03 | Spoofing | Renamed crate `xcfun-capi` | mitigate | Package `name = "xcfun-capi"` is a fresh string; no path collision with the deleted `xcfun-ffi`. CI builds verify. |
| T-05-00-04 | Tampering | LB94 descriptor presence vs. upstream `#if 0` | mitigate | LB94 descriptor exists ONLY in xcfun-core (Rust side); xtask `emit_c_stubs_cpp` explicitly skips LB94 so `validation/c_stubs.cpp` does NOT redefine `XC_LB94` for the upstream C++ tree. The `regen-registry --check` drift gate enforces this. |
</threat_model>

## tasks
<tasks>

<task type="auto">
  <name>Task 0.1: Workspace topology — rename xcfun-ffi → xcfun-capi, delete xcfun-functionals, register xcfun-rs</name>
  <files>Cargo.toml, crates/xcfun-capi/Cargo.toml, crates/xcfun-capi/src/lib.rs, crates/xcfun-rs/Cargo.toml, crates/xcfun-rs/src/lib.rs</files>
  <read_first>
    - Cargo.toml (current workspace `members` and `exclude` lists)
    - crates/xcfun-ffi/Cargo.toml (rename source — preserve license + edition + version)
    - crates/xcfun-ffi/src/lib.rs (current body to keep as a temporary 1-line stub)
    - crates/xcfun-eval/Cargo.toml (analog for xcfun-rs Cargo.toml shape — license + edition + workspace inheritance)
  </read_first>
  <action>
    Execute the topology mutation in this exact order:

    1. **Rename via git** (preserves history):
       ```bash
       git mv crates/xcfun-ffi crates/xcfun-capi
       ```

    2. **Edit `crates/xcfun-capi/Cargo.toml`** — change ONLY the `name` field
       and the `description`:
       ```toml
       [package]
       name = "xcfun-capi"
       version.workspace = true
       edition.workspace = true
       description = "C ABI drop-in replacement for xcfun-master/api/xcfun.h"
       license = "MPL-2.0"

       [dependencies]
       xcfun-core = { path = "../xcfun-core" }
       # Plan 05-02 adds xcfun-rs dep + crate-type triple. This plan only renames.
       ```
       Leave `crates/xcfun-capi/src/lib.rs` as a 1-line `//! C ABI for xcfun_rs (Plan 05-02 fills body).` Plan 05-02 replaces it.

    3. **Delete `crates/xcfun-functionals/`** (D-04):
       ```bash
       git rm -r crates/xcfun-functionals
       ```

    4. **Create `crates/xcfun-rs/`** (D-02 facade target — Plan 05-01 fills body):
       ```bash
       mkdir -p crates/xcfun-rs/src
       ```
       Write `crates/xcfun-rs/Cargo.toml` with EXACTLY these contents:
       ```toml
       [package]
       name = "xcfun-rs"
       version.workspace = true
       edition.workspace = true
       rust-version.workspace = true
       description = "Native Rust public API for xcfun_rs (Functional struct + free fns)"
       license = "MPL-2.0"

       [features]
       default = []
       # RS-08 GPU dispatch lives in Phase 6; Phase 5 ships a Functional::eval_vec stub.

       [dependencies]
       xcfun-core = { path = "../xcfun-core" }
       xcfun-eval = { path = "../xcfun-eval", features = ["testing"] }
       thiserror  = { workspace = true }

       [dev-dependencies]
       static_assertions = "1.1"
       ```
       Note: the `xcfun-eval = { ..., features = ["testing"] }` is REQUIRED
       because `Functional::eval` and `Functional::launch_potential` are gated
       behind `#[cfg(feature = "testing")]` in xcfun-eval/src/functional.rs:299
       and :495. Without this feature flag the wrapper's `eval` method returns
       `XcError::Runtime` per functional.rs:521-523.

       Write `crates/xcfun-rs/src/lib.rs` with EXACTLY:
       ```rust
       //! xcfun-rs — native Rust public API for xcfun_rs (Phase 5).
       //! Plan 05-01 fills in `Functional`, free functions, and tests.
       #![forbid(unsafe_code)]
       ```

    5. **Edit workspace `Cargo.toml`** — replace the existing `members` and
       `exclude` arrays with EXACTLY:
       ```toml
       members = [
           "crates/xcfun-ad",
           "crates/xcfun-core",
           "crates/xcfun-eval",
           "crates/xcfun-rs",
           "crates/xcfun-capi",
           "xtask",
           "validation",
       ]
       exclude = [
           "crates/xcfun-gpu",
           "crates/xcfun-python",
       ]
       ```
       (xcfun-ffi removed because path no longer exists; xcfun-functionals
       removed because directory deleted; xcfun-rs and xcfun-capi promoted
       to members per D-01 + D-02.)
  </action>
  <verify>
    <automated>cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys;m=json.load(sys.stdin)["packages"];names=sorted(p["name"] for p in m);print(names);assert "xcfun-rs" in names and "xcfun-capi" in names and "xcfun-ffi" not in names and "xcfun-functionals" not in names, names'</automated>
    <automated>test ! -d crates/xcfun-ffi && test ! -d crates/xcfun-functionals && test -f crates/xcfun-capi/Cargo.toml && test -f crates/xcfun-rs/Cargo.toml && test -f crates/xcfun-rs/src/lib.rs</automated>
    <automated>cargo check -p xcfun-rs -p xcfun-capi 2>&1 | tee /tmp/check_05_00.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>grep -F 'name = "xcfun-capi"' crates/xcfun-capi/Cargo.toml && grep -F 'name = "xcfun-rs"' crates/xcfun-rs/Cargo.toml</automated>
  </verify>
  <done>
    - `crates/xcfun-ffi/` and `crates/xcfun-functionals/` deleted from disk.
    - `crates/xcfun-capi/Cargo.toml` exists with `name = "xcfun-capi"`.
    - `crates/xcfun-rs/Cargo.toml` exists with `name = "xcfun-rs"` and the deps
      block above.
    - `cargo metadata --no-deps` lists xcfun-rs + xcfun-capi as packages, lists
      no xcfun-ffi or xcfun-functionals package.
    - `cargo check -p xcfun-rs -p xcfun-capi` exits 0.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 0.2: XcError — add InvalidVarsAndMode variant + as_c_code() method (D-08-A, CAPI-05)</name>
  <files>crates/xcfun-core/src/error.rs</files>
  <read_first>
    - crates/xcfun-core/src/error.rs (current 9-variant enum + the existing tests module at the bottom — copy the assert_impl_all! pattern)
    - crates/xcfun-core/src/enums.rs:39-71 (existing `impl ParameterId { default_value }` — same enum-method shape)
    - xcfun-master/src/XCFunctional.hpp lines 40-46 (XC_EORDER=1, XC_EVARS=2, XC_EMODE=4 constants)
    - xcfun-master/src/XCFunctional.cpp:437-470 (eval_setup C++ body — source for the `XC_EVARS | XC_EMODE = 6` combined case)
  </read_first>
  <behavior>
    - Test `as_c_code_invalid_order` — `XcError::InvalidOrder { order: 5, mode: Mode::PartialDerivatives, n_vars: 2 }.as_c_code() == 1`
    - Test `as_c_code_invalid_vars` — `XcError::InvalidVars { vars: Vars::A, required: Dependency::GRADIENT }.as_c_code() == 2`
    - Test `as_c_code_invalid_mode` — `XcError::InvalidMode { mode: Mode::Potential, depends: Dependency::KINETIC }.as_c_code() == 4`
    - Test `as_c_code_invalid_vars_and_mode` — `XcError::InvalidVarsAndMode { vars: Vars::A_B, mode: Mode::Potential, depends: Dependency::GRADIENT }.as_c_code() == 6`
    - Test `as_c_code_unknown_name` — `XcError::UnknownName.as_c_code() == -1`
    - Test `as_c_code_not_configured` — `XcError::NotConfigured.as_c_code() == -1`
    - Test `as_c_code_runtime` — `XcError::Runtime.as_c_code() == -1`
    - Test `as_c_code_input_length_mismatch` — `XcError::InputLengthMismatch { expected: 2, got: 1 }.as_c_code() == -1`
    - Test `as_c_code_output_length_mismatch` — `XcError::OutputLengthMismatch { expected: 2, got: 1 }.as_c_code() == -1`
    - Test `as_c_code_invalid_encoding` — `XcError::InvalidEncoding.as_c_code() == -1`
    - Test `xc_error_still_copy` — `assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug)` compiles.
  </behavior>
  <action>
    Edit `crates/xcfun-core/src/error.rs` and apply the changes in this order:

    1. **Add the new variant** to the enum (between `InvalidMode` and `UnknownName`)
       so the variant order matches the C++ XC_E* numeric ordering:
       ```rust
       /// Combined `XC_EVARS | XC_EMODE` (= 6) returned by
       /// `xcfun-master/src/XCFunctional.cpp:441-443` when `Mode::Potential`
       /// is requested for a GGA-tier functional set whose `Vars` is not
       /// one of the `_2ND_TAYLOR` arms.
       #[error("vars {vars:?} and mode {mode:?} both invalid for dependencies {depends:?}")]
       InvalidVarsAndMode {
           vars: Vars,
           mode: Mode,
           depends: Dependency,
       },
       ```
       The variant must be `Copy + Clone + Debug + thiserror::Error` like all
       existing variants — Rust derives carry through automatically because
       all field types (`Vars`, `Mode`, `Dependency`) are `Copy`.

    2. **Add `as_c_code` method** as a separate `impl XcError` block placed
       directly above the `#[cfg(test)] mod tests {` block:
       ```rust
       impl XcError {
           /// C ABI error code per CAPI-05 + Phase 5 D-08-A. Mirrors the
           /// `XC_E*` constants in `xcfun-master/src/XCFunctional.hpp:40-46`:
           /// `XC_EORDER=1, XC_EVARS=2, XC_EMODE=4`. The combined
           /// `XC_EVARS | XC_EMODE` (= 6) is produced by `Functional::eval_setup`
           /// when `Mode::Potential` is requested for a GGA tier whose
           /// `Vars` lacks the `_2ND_TAYLOR` shape (XCFunctional.cpp:441-443).
           ///
           /// Variants without a direct upstream `XC_E*` mapping
           /// (UnknownName, NotConfigured, Runtime, InputLengthMismatch,
           /// OutputLengthMismatch, InvalidEncoding) all map to `-1`,
           /// mirroring the C++ pattern of returning `-1` from
           /// `xcfun_set` / `xcfun_get` for unknown names. LB94's
           /// `XcError::Runtime` (returned by `Functional::eval` because
           /// the upstream lb94.cpp is `#if 0`'d) maps to `-1`.
           pub fn as_c_code(&self) -> i32 {
               match self {
                   Self::InvalidOrder { .. } => 1,         // XC_EORDER
                   Self::InvalidVars  { .. } => 2,         // XC_EVARS
                   Self::InvalidMode  { .. } => 4,         // XC_EMODE
                   Self::InvalidVarsAndMode { .. } => 6,   // XC_EVARS | XC_EMODE
                   _ => -1,
               }
           }
       }
       ```

    3. **Extend the existing `#[cfg(test)] mod tests`** block. Add the 11 tests
       listed in `<behavior>` above. The existing `assert_impl_all!` line
       must remain — `XcError` keeps `Copy + Clone + Send + Sync + Debug`.
       Skeleton:
       ```rust
       #[cfg(test)]
       mod tests {
           use super::*;
           use static_assertions::assert_impl_all;
           assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug);

           #[test] fn as_c_code_invalid_order() {
               assert_eq!(
                   XcError::InvalidOrder { order: 5, mode: Mode::PartialDerivatives, n_vars: 2 }.as_c_code(),
                   1,
               );
           }
           // ... 10 more — one per behavior bullet ...

           // Existing tests (invalid_order_display, unknown_name_display_drops_payload,
           // not_configured_display, xc_error_is_copy) remain untouched.
       }
       ```
       Use `crate::traits::Dependency` (already imported via `use crate::traits::Dependency;`
       at the top of error.rs).
  </action>
  <verify>
    <automated>cargo test -p xcfun-core --lib error::tests -- --nocapture 2>&1 | tee /tmp/err_test_05_00.log; grep -E "test result: ok\. ([0-9]+) passed" /tmp/err_test_05_00.log | grep -Ev "0 passed"</automated>
    <automated>grep -nF "InvalidVarsAndMode" crates/xcfun-core/src/error.rs | head -5</automated>
    <automated>grep -nF "pub fn as_c_code" crates/xcfun-core/src/error.rs</automated>
    <automated>cargo test -p xcfun-core --lib error::tests::as_c_code_invalid_vars_and_mode 2>&1 | grep -F "test error::tests::as_c_code_invalid_vars_and_mode ... ok"</automated>
  </verify>
  <done>
    - `XcError` enum has 10 variants (was 9); the new variant is named
      `InvalidVarsAndMode { vars, mode, depends }`.
    - `XcError::as_c_code(&self) -> i32` method exists and matches the table
      in `<behavior>`.
    - 11 new unit tests pass (`cargo test -p xcfun-core --lib error::tests`).
    - `assert_impl_all!(XcError: Copy, Clone, Send, Sync, std::fmt::Debug)`
      still compiles (variant carries Copy fields only).
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 0.3: eval_setup — emit InvalidVarsAndMode for the combined-error case (D-08-A)</name>
  <files>crates/xcfun-eval/src/functional.rs</files>
  <read_first>
    - crates/xcfun-eval/src/functional.rs:423-474 (existing `eval_setup` body — replace ONLY the `Mode::Potential` branch's `InvalidVars` Err arm with the new combined variant)
    - xcfun-master/src/XCFunctional.cpp:437-470 (C++ source; observe the predicate at line 441 — `(fun->depends & XC_GRADIENT) && !(vars in {A_2ND_TAYLOR, A_B_2ND_TAYLOR, N_2ND_TAYLOR, N_S_2ND_TAYLOR})` returns `XC_EVARS | XC_EMODE`)
    - crates/xcfun-eval/src/functional.rs:1920-1969 (existing eval_setup tests — extend with one new case)
  </read_first>
  <behavior>
    - Test `eval_setup_returns_combined_error_for_gga_non_2nd_taylor_potential` — given a `Functional` whose dependencies include `Dependency::GRADIENT` (e.g. `set("pbex", 1.0)`), `eval_setup(Vars::A_B, Mode::Potential, 0)` returns `Err(XcError::InvalidVarsAndMode { vars: Vars::A_B, mode: Mode::Potential, depends: <bitflags including GRADIENT> })` (NOT `InvalidVars` as before).
    - Existing test `eval_setup_accepts_gga_with_2nd_taylor_potential` still passes (no regression on the happy GGA-2nd-taylor path).
    - Existing test `eval_setup_rejects_metagga_potential` still passes — metaGGA path returns `InvalidMode` (NOT the combined variant).
    - Existing test `eval_setup_rejects_laplacian_potential` still passes — `InvalidMode` for laplacian-bearing deps.
    - Existing test `eval_setup_rejects_unset_mode` still passes — `NotConfigured`.
  </behavior>
  <action>
    Edit `crates/xcfun-eval/src/functional.rs` `eval_setup` (line 423-474 in
    the current file). Replace ONLY the `Vars::A_2ND_TAYLOR | ... => {} _ =>
    return Err(XcError::InvalidVars { ... })` arm. Before (line 458-470):
    ```rust
    if deps.contains(Dependency::GRADIENT) {
        match vars {
            Vars::A_2ND_TAYLOR
            | Vars::A_B_2ND_TAYLOR
            | Vars::N_2ND_TAYLOR
            | Vars::N_S_2ND_TAYLOR => {}
            _ => {
                return Err(XcError::InvalidVars {
                    vars,
                    required: Dependency::GRADIENT,
                });
            }
        }
    }
    ```
    After (D-08-A):
    ```rust
    if deps.contains(Dependency::GRADIENT) {
        match vars {
            Vars::A_2ND_TAYLOR
            | Vars::A_B_2ND_TAYLOR
            | Vars::N_2ND_TAYLOR
            | Vars::N_S_2ND_TAYLOR => {}
            _ => {
                // D-08-A — XCFunctional.cpp:441-443 returns
                // XC_EVARS | XC_EMODE (= 6) for this combined case.
                return Err(XcError::InvalidVarsAndMode {
                    vars,
                    mode,
                    depends: deps,
                });
            }
        }
    }
    ```

    Then **update the existing test** at line 1952-1969 (named
    `eval_setup_rejects_gga_non_2nd_taylor_potential`) so its match pattern
    accepts the new variant. Replace its body's match arm:
    ```rust
    match err {
        XcError::InvalidVars { .. } => {}
        e => panic!("expected InvalidVars, got {e:?}"),
    }
    ```
    with:
    ```rust
    match err {
        XcError::InvalidVarsAndMode { vars: v, mode: m, depends: d } => {
            assert_eq!(v, Vars::A_B);
            assert_eq!(m, Mode::Potential);
            assert!(d.contains(Dependency::GRADIENT));
        }
        e => panic!("expected InvalidVarsAndMode, got {e:?}"),
    }
    ```
    AND **add a new test** in the same `mod tests` block named
    `eval_setup_emits_combined_error_when_gga_potential_with_lda_vars`:
    ```rust
    #[test]
    fn eval_setup_emits_combined_error_when_gga_potential_with_lda_vars() {
        // PBEX = pure GGA, no laplacian/kinetic. Vars::A_B is LDA-shaped (no _2ND_TAYLOR).
        let mut f = Functional::new();
        f.set("pbex", 1.0).unwrap();
        let err = f.eval_setup(Vars::A_B, Mode::Potential, 0).unwrap_err();
        match err {
            XcError::InvalidVarsAndMode { vars, mode, depends } => {
                assert_eq!(vars, Vars::A_B);
                assert_eq!(mode, Mode::Potential);
                assert!(depends.contains(Dependency::GRADIENT));
            }
            e => panic!("expected InvalidVarsAndMode, got {e:?}"),
        }
    }
    ```

    Do NOT touch any other branch of `eval_setup`. The `Mode::Unset →
    NotConfigured`, `Mode::Contracted → InvalidOrder` for order > 6, and
    `Mode::Potential` + (LAPLACIAN | KINETIC) → `InvalidMode` arms remain
    byte-identical.
  </action>
  <verify>
    <automated>cargo test -p xcfun-eval --features testing --lib functional::tests::eval_setup_emits_combined_error_when_gga_potential_with_lda_vars 2>&1 | grep -F "test functional::tests::eval_setup_emits_combined_error_when_gga_potential_with_lda_vars ... ok"</automated>
    <automated>cargo test -p xcfun-eval --features testing --lib functional::tests 2>&1 | tee /tmp/eval_test_05_00.log; grep -E "test result: ok" /tmp/eval_test_05_00.log</automated>
    <automated>grep -nF "InvalidVarsAndMode" crates/xcfun-eval/src/functional.rs</automated>
  </verify>
  <done>
    - `crates/xcfun-eval/src/functional.rs::eval_setup` returns
      `Err(XcError::InvalidVarsAndMode { ... })` for the GGA-non-_2ND_TAYLOR
      Mode::Potential predicate.
    - Existing tests in `functional::tests` (eval_setup_rejects_metagga_potential,
      eval_setup_rejects_laplacian_potential, eval_setup_rejects_unset_mode,
      eval_setup_accepts_gga_with_2nd_taylor_potential,
      eval_setup_accepts_lda_with_any_vars_potential, eval_rejects_unset_mode,
      eval_contracted_mode_accepted_at_order_0,
      eval_rejects_contracted_order_above_6) STILL pass.
    - One new test `eval_setup_emits_combined_error_when_gga_potential_with_lda_vars`
      passes.
    - `cargo test -p xcfun-eval --features testing --lib functional::tests`
      exits 0.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 0.4: LB94 descriptor add-back per D-16 — extend FunctionalId, regen FUNCTIONAL_DESCRIPTORS, exclude from c_stubs</name>
  <files>crates/xcfun-core/src/functional_id.rs, xtask/src/bin/regen_registry.rs, crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs, crates/xcfun-core/tests/registry_tables.rs, crates/xcfun-eval/src/functionals/mgga/mod.rs</files>
  <read_first>
    - crates/xcfun-core/src/functional_id.rs (78-entry enum; COUNT = 78; from_name match arms — append XC_LB94 = 78, bump COUNT to 79, add "LB94" arm)
    - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (78-row auto-generated table — Task 0.4 step 4 regenerates to 79 rows)
    - crates/xcfun-core/tests/registry_tables.rs (lines 12-14: `descriptors_count_is_78` test that hardcodes 78; bump to 79)
    - crates/xcfun-core/src/functional_id.rs:193-195 (`count_is_78` test; bump to 79)
    - crates/xcfun-core/tests/parameter_and_alias_registry.rs:23-36 (`parameter_id_discriminants_match_cpp` — verify this test STAYS unchanged; XC_RANGESEP_MU stays at 78 because the upstream C++ enum lacks XC_LB94)
    - xtask/src/bin/regen_registry.rs:178-256 (FUNCTIONAL_IDS list — append "XC_LB94" with comment; verify the list stays in lockstep with FunctionalId enum order)
    - xtask/src/bin/regen_registry.rs:541-591 (`emit_c_stubs_cpp` — add LB94 to a skip-set so `validation/c_stubs.cpp` does NOT emit `FUNCTIONAL(XC_LB94)`)
    - xcfun-master/src/functionals/lb94.cpp:48-58 (`setup_lb94` macro — extracts `f.describe(XC_LB94, XC_GGA, ...)` so the descriptor's `depends` field should be `Dependency::DENSITY | Dependency::GRADIENT`)
    - xcfun-master/src/functionals/list_of_functionals.hpp:15-105 (verify upstream C++ enum lacks XC_LB94 — this is why `validation/c_stubs.cpp` MUST NOT emit it)
    - crates/xcfun-eval/src/functionals/mgga/mod.rs:1-10 (header comment containing the factually-incorrect "LB94 (id=66) deferred to Phase 5 per D-13" note — fix it)
  </read_first>
  <behavior>
    - Test `count_is_79` (replaces `count_is_78`) — `assert_eq!(FunctionalId::COUNT, 79)`.
    - Test `lb94_discriminant_is_78` (NEW) — `assert_eq!(FunctionalId::XC_LB94 as u32, 78)`.
    - Test `from_name_resolves_lb94` (NEW) — `FunctionalId::from_name("lb94") == Some(FunctionalId::XC_LB94)`; `from_name("LB94") == Some(...)`; `from_name("XC_LB94") == Some(...)`.
    - Test `descriptors_count_is_79` (replaces `descriptors_count_is_78` in `crates/xcfun-core/tests/registry_tables.rs`) — `assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), 79)`; `assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), FunctionalId::COUNT)`.
    - Test `lb94_descriptor_present` (NEW in `crates/xcfun-core/tests/registry_tables.rs`) — `FUNCTIONAL_DESCRIPTORS[78].id == FunctionalId::XC_LB94`; `FUNCTIONAL_DESCRIPTORS[78].name == "XC_LB94"`; `FUNCTIONAL_DESCRIPTORS[78].depends.contains(Dependency::DENSITY)`; `FUNCTIONAL_DESCRIPTORS[78].depends.contains(Dependency::GRADIENT)`.
    - Test `parameter_id_discriminants_match_cpp` (in `crates/xcfun-core/tests/parameter_and_alias_registry.rs`) STILL passes unchanged — XC_RANGESEP_MU == 78, XC_EXX == 79, XC_CAM_ALPHA == 80, XC_CAM_BETA == 81. (The C++ XC_NR_FUNCTIONALS = 78 invariant is independent of the Rust-side LB94 add-back; in the upstream C++ enum, XC_LB94 does not exist.)
    - Test `c_stubs_cpp_excludes_lb94` (NEW — add to `xtask/src/bin/regen_registry.rs` test module if test infra exists, else verify via the `--check` drift gate that `validation/c_stubs.cpp` produced by the regen step does NOT contain `FUNCTIONAL(XC_LB94)`).
    - `cargo run -p xtask --bin regen-registry -- --check` exits 0 (no drift after the regen step).
  </behavior>
  <action>
    Apply the changes in this order. Do NOT skip the `regen-registry --check`
    step at the end — the auto-generated `FUNCTIONAL_DESCRIPTORS.rs` file MUST
    be byte-identical to what the regen binary would produce, or the CI drift
    gate fails.

    1. **Edit `crates/xcfun-core/src/functional_id.rs`**:
       - Append the new variant inside `pub enum FunctionalId { ... }` after
         `XC_PW91C = 77,`:
         ```rust
             XC_PW91C = 77,
             /// LB94 — Van Leeuwen-Baerends potential (1994).
             ///
             /// **D-16 (Phase 5 CONTEXT):** Descriptor present per D-16; the
             /// upstream `xcfun-master/src/functionals/lb94.cpp:15` is wrapped
             /// in `#if 0` ("Does not work and should maybe not be here in
             /// the first place"), so `Functional::eval` returns
             /// `XcError::Runtime` for this id. The descriptor exists so
             /// downstream tools can enumerate the name and so that
             /// `xcfun_set("lb94", 1.0)` succeeds at the FFI surface
             /// (failing only at eval time). LB94 is NOT counted in the
             /// upstream C++ enum (`list_of_functionals.hpp` does not list
             /// XC_LB94 — `XC_NR_FUNCTIONALS` stays at 78 in the C++ world);
             /// this discriminant is Rust-side only and does not collide
             /// with the C ABI numeric surface (the C ABI uses string
             /// names, not numeric ids).
             XC_LB94 = 78,
         ```
       - Bump `pub const COUNT: usize = 78;` → `pub const COUNT: usize = 79;`.
       - Bump module-level header comment from "78 exchange-correlation
         functional identifiers" to "79 exchange-correlation functional
         identifiers (78 from upstream + LB94 stub per Phase 5 D-16)".
       - Append a new match arm to `from_name`:
         ```rust
             "LB94" => Some(Self::XC_LB94),
         ```
         (placed after `"PW91C" => Some(Self::XC_PW91C),`)
       - Update the existing `count_is_78` test → `count_is_79`:
         ```rust
         #[test]
         fn count_is_79() {
             assert_eq!(FunctionalId::COUNT, 79);
         }
         ```
       - Add a NEW test:
         ```rust
         #[test]
         fn lb94_discriminant_is_78() {
             assert_eq!(FunctionalId::XC_LB94 as u32, 78);
         }
         ```
       - Update the existing `from_name_round_trip` test to add an LB94 line:
         ```rust
         assert_eq!(FunctionalId::from_name("lb94"), Some(FunctionalId::XC_LB94));
         assert_eq!(FunctionalId::from_name("XC_LB94"), Some(FunctionalId::XC_LB94));
         ```

    2. **Edit `xtask/src/bin/regen_registry.rs`**:
       - Append `"XC_LB94"` to the `FUNCTIONAL_IDS` array (line 178-256) after
         `"XC_PW91C",         // 77`:
         ```rust
             "XC_PW91C",         // 77
             "XC_LB94",          // 78 — D-16 (Phase 5): present in Rust
                                 //          registry but NOT in upstream C++
                                 //          `list_of_functionals.hpp`. Body
                                 //          is `#if 0`'d in lb94.cpp. The
                                 //          descriptor is a stub; eval
                                 //          returns Runtime.
         ];
         ```
       - In `emit_c_stubs_cpp` (around line 546), add a guard right after the
         existing `if PHASE2_LDA_IDS.contains(id) { continue; }` line:
         ```rust
         for id in FUNCTIONAL_IDS {
             if PHASE2_LDA_IDS.contains(id) {
                 continue; // LDA IDs get the real ENERGY_FUNCTION from their .cpp file.
             }
             // D-16: LB94 has no upstream C++ symbol (lb94.cpp is `#if 0`'d
             // and `XC_LB94` is absent from list_of_functionals.hpp).
             // Emitting `FUNCTIONAL(XC_LB94)` would fail to compile in the
             // validation crate. Skip.
             if *id == "XC_LB94" {
                 continue;
             }
             // ... existing emit logic ...
         }
         ```
       - In the descriptor-emission loop (around line 408 — the loop that
         iterates `FUNCTIONAL_IDS` to write `FUNCTIONAL_DESCRIPTORS.rs`),
         the LB94 entry will fall into the "Not found by extractor" branch
         (line 448-451) and emit:
         ```rust
         FunctionalDescriptor::stub(FunctionalId::XC_LB94, "XC_LB94", Dependency::DENSITY),
         ```
         which is the wrong dependency mask. Override LB94 specifically:
         add a special case ABOVE the generic "Not found" branch:
         ```rust
         } else if *id == "XC_LB94" {
             // D-16: LB94 has dependency XC_GGA (= DENSITY | GRADIENT) per
             // setup_lb94 in xcfun-master/src/functionals/lb94.cpp:48-50.
             // Even though the body is #if 0'd, the descriptor records the
             // intended dependency mask for downstream tools.
             out.push_str(
                 "    FunctionalDescriptor::stub(FunctionalId::XC_LB94, \"XC_LB94\", \
                  Dependency::DENSITY.union(Dependency::GRADIENT)),\n"
             );
         } else {
             // Not found by extractor — emit generic stub.
             out.push_str(&format!(
                 "    FunctionalDescriptor::stub(FunctionalId::{}, \"{}\", Dependency::DENSITY),\n",
                 id, id
             ));
         }
         ```

    3. **Run the regen binary to update `FUNCTIONAL_DESCRIPTORS.rs`**:
       ```bash
       cargo run -p xtask --bin regen-registry
       ```
       Verify the output:
       ```bash
       grep -F "XC_LB94" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs
       grep -c "FunctionalDescriptor" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs
       # Expect 79 descriptor entries.
       ```
       Verify `validation/c_stubs.cpp` does NOT contain `FUNCTIONAL(XC_LB94)`:
       ```bash
       ! grep -F "FUNCTIONAL(XC_LB94)" validation/c_stubs.cpp
       ```

    4. **Edit `crates/xcfun-core/tests/registry_tables.rs`**:
       - Rename `descriptors_count_is_78` → `descriptors_count_is_79`:
         ```rust
         #[test]
         fn descriptors_count_is_79() {
             assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), 79);
             assert_eq!(FUNCTIONAL_DESCRIPTORS.len(), FunctionalId::COUNT);
         }
         ```
       - Add a NEW test `lb94_descriptor_present` next to it:
         ```rust
         #[test]
         fn lb94_descriptor_present() {
             // D-16 — descriptor exists at row 78; depends mask matches
             // setup_lb94 macro (xcfun-master/src/functionals/lb94.cpp:48-50).
             let lb94 = &FUNCTIONAL_DESCRIPTORS[78];
             assert_eq!(lb94.id, FunctionalId::XC_LB94);
             assert_eq!(lb94.name, "XC_LB94");
             assert!(lb94.depends.contains(Dependency::DENSITY));
             assert!(lb94.depends.contains(Dependency::GRADIENT));
             // No test data for LB94 (body is #if 0'd upstream).
             assert!(lb94.test_in.is_none());
             assert!(lb94.test_out.is_none());
         }
         ```
         (Add the necessary `use xcfun_core::{FunctionalId, Dependency};` import
         if not already present at the top of the file.)

    5. **Edit `crates/xcfun-eval/src/functionals/mgga/mod.rs`** — fix the
       factually-incorrect Phase 4 comment at line 4:
       ```rust
       //! Phase 4 ships **32 functional IDs** per D-01 (28 metaGGA + 4 carryovers
       //! BRX/BRC/BRXC + CSC). Phase 5 D-16 added LB94 (id=78) as a registry
       //! stub; eval returns XcError::Runtime since lb94.cpp is `#if 0`'d
       //! upstream. (The earlier comment claiming "id=66" was incorrect —
       //! id 66 is XC_CSC.)
       ```

    6. **Run the drift gate** to confirm `regen-registry --check` exits clean:
       ```bash
       cargo run -p xtask --bin regen-registry -- --check
       ```
       Exit code MUST be 0. If it returns 2 (drift), the manual edits to
       `FUNCTIONAL_DESCRIPTORS.rs` are wrong; re-run `cargo run -p xtask
       --bin regen-registry` (without `--check`) and try again.

    7. **Verify `parameter_and_alias_registry.rs` still passes** — the
       `parameter_id_discriminants_match_cpp` test asserts
       `XC_RANGESEP_MU as u32 == 78`. This MUST stay unchanged: in the
       upstream C++ enum, XC_LB94 does not exist (lb94.cpp is `#if 0`'d),
       so `XC_NR_FUNCTIONALS = 78` and `XC_RANGESEP_MU = 78`. The Rust-side
       `FunctionalId::XC_LB94 = 78` is in a different enum type — no
       collision. Run:
       ```bash
       cargo test -p xcfun-core --test parameter_and_alias_registry parameter_id_discriminants_match_cpp
       ```
  </action>
  <verify>
    <automated>cargo test -p xcfun-core --lib functional_id::tests::lb94_discriminant_is_78 2>&1 | grep -F "test functional_id::tests::lb94_discriminant_is_78 ... ok"</automated>
    <automated>cargo test -p xcfun-core --lib functional_id::tests::count_is_79 2>&1 | grep -F "test functional_id::tests::count_is_79 ... ok"</automated>
    <automated>cargo test -p xcfun-core --test registry_tables 2>&1 | tee /tmp/reg_05_00d.log; grep -E "test result: ok" /tmp/reg_05_00d.log</automated>
    <automated>cargo test -p xcfun-core --test parameter_and_alias_registry parameter_id_discriminants_match_cpp 2>&1 | grep -F "test parameter_id_discriminants_match_cpp ... ok"</automated>
    <automated>cargo run -p xtask --bin regen-registry -- --check 2>&1 | tee /tmp/regen_05_00d.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>grep -F "XC_LB94" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs | head -3</automated>
    <automated>grep -c "FunctionalDescriptor::" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs | grep -E "^79$"</automated>
    <automated>! grep -F "FUNCTIONAL(XC_LB94)" validation/c_stubs.cpp 2>/dev/null; test $? -eq 0</automated>
    <automated>grep -F "XC_LB94" crates/xcfun-core/src/functional_id.rs | head -3</automated>
  </verify>
  <done>
    - `FunctionalId` has 79 variants; `FunctionalId::XC_LB94 as u32 == 78`; `FunctionalId::COUNT == 79`.
    - `FunctionalId::from_name("lb94")` and `from_name("XC_LB94")` return `Some(FunctionalId::XC_LB94)`.
    - `FUNCTIONAL_DESCRIPTORS` has 79 rows; row 78 is the LB94 stub with `name = "XC_LB94"` and `depends = Dependency::DENSITY | Dependency::GRADIENT`.
    - `validation/c_stubs.cpp` does NOT contain `FUNCTIONAL(XC_LB94)`.
    - `cargo run -p xtask --bin regen-registry -- --check` exits 0.
    - `parameter_id_discriminants_match_cpp` test still passes — XC_RANGESEP_MU stays at 78.
    - `crates/xcfun-eval/src/functionals/mgga/mod.rs` header comment updated to reflect LB94 (id=78) registry presence; the factually-incorrect "id=66" claim removed.
  </done>
</task>

</tasks>

<verification>
Run after all tasks complete:

```bash
# Topology
cargo metadata --no-deps --format-version 1 | python3 -c '
import json, sys
m = json.load(sys.stdin)["packages"]
names = sorted(p["name"] for p in m)
assert "xcfun-rs"   in names, names
assert "xcfun-capi" in names, names
assert "xcfun-ffi" not in names
assert "xcfun-functionals" not in names
print("topology OK:", names)
'
test ! -d crates/xcfun-ffi
test ! -d crates/xcfun-functionals

# XcError surface
grep -F "InvalidVarsAndMode" crates/xcfun-core/src/error.rs
grep -F "pub fn as_c_code" crates/xcfun-core/src/error.rs

# Combined-error wiring
grep -F "InvalidVarsAndMode" crates/xcfun-eval/src/functional.rs

# LB94 descriptor (D-16)
grep -F "XC_LB94 = 78" crates/xcfun-core/src/functional_id.rs
grep -F "XC_LB94" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs
grep -c "FunctionalDescriptor::" crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs   # 79
! grep -F "FUNCTIONAL(XC_LB94)" validation/c_stubs.cpp
cargo run -p xtask --bin regen-registry -- --check  # exit 0

# Tests
cargo test -p xcfun-core --lib error::tests
cargo test -p xcfun-core --lib functional_id::tests
cargo test -p xcfun-core --test registry_tables
cargo test -p xcfun-core --test parameter_and_alias_registry
cargo test -p xcfun-eval --features testing --lib functional::tests

# Workspace builds clean
cargo check --workspace
```
</verification>

<success_criteria>
- All four tasks' `<verify>` blocks exit 0.
- `cargo check --workspace` exits 0.
- `cargo test -p xcfun-core --lib` and `cargo test -p xcfun-eval --features testing --lib` exit 0.
- `cargo run -p xtask --bin regen-registry -- --check` exits 0 (no drift).
- Workspace topology aligned with D-01/D-04: xcfun-rs + xcfun-capi as members; no xcfun-ffi / xcfun-functionals on disk.
- `XcError::as_c_code` mapping matches the CAPI-05 table verbatim (1 / 2 / 4 / 6 / -1).
- `eval_setup` emits the combined `InvalidVarsAndMode` variant for the
  GGA-non-_2ND_TAYLOR Mode::Potential branch — observable by Plan 05-02
  when wrapping the FFI surface.
- LB94 descriptor present in xcfun-core per D-16: FunctionalId::XC_LB94 = 78
  exists; FUNCTIONAL_DESCRIPTORS has 79 entries; validation/c_stubs.cpp
  excludes LB94; the upstream-C++-tied `parameter_id_discriminants_match_cpp`
  test still passes unchanged.
</success_criteria>

<output>
After completion, create `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-00-SUMMARY.md` documenting:
- The exact rename + delete commands run (one git mv, one git rm -r).
- Net delta on `XcError` (1 new variant, 1 new method, 11 new unit tests).
- One-line confirmation that `eval_setup`'s combined-error branch is reachable.
- Net delta on `FunctionalId` (1 new variant XC_LB94 = 78; COUNT bumped to 79;
  3 new unit tests; from_name handles 'lb94').
- Net delta on FUNCTIONAL_DESCRIPTORS (1 new stub row at index 78, regenerated
  via xtask regen-registry).
- Confirmation that validation/c_stubs.cpp does NOT contain FUNCTIONAL(XC_LB94)
  (the xtask `emit_c_stubs_cpp` exclusion is in place).
- Confirmation that the C++-tied parameter discriminant test
  (`parameter_id_discriminants_match_cpp`) still passes — XC_RANGESEP_MU
  stays at 78 because XC_LB94 is absent from upstream C++.
</output>
</content>
</invoke>