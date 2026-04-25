---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "04"
type: execute
wave: 4
depends_on:
  - "04-03"
requirements:
  - ALIAS-01
  - ALIAS-02
  - ALIAS-03
  - ALIAS-04
  - ALIAS-05
  - ALIAS-06
files_modified:
  - crates/xcfun-core/src/enums.rs
  - crates/xcfun-core/src/registry/generated/ALIASES.rs
  - crates/xcfun-core/src/registry/generated/parameters.rs
  - crates/xcfun-core/src/lib.rs
  - crates/xcfun-eval/src/functional.rs
  - xtask/src/bin/regen_registry.rs
autonomous: true
created: "2026-04-25"
goal: "Wave 4 — Parameter table (4 entries), ParameterId enum, 46-alias registry populated, Functional::set/get recursion engine wired; alias canary tests GREEN"

must_haves:
  truths:
    - "Functional::set('b3lyp', 1.0) produces settings[SLATERX]=0.80, settings[BECKECORRX]=0.72, settings[LYPC]=0.81, settings[VWN5C]=0.19, settings[EXX]=0.20"
    - "Functional::set('camcompx', 0.37) produces settings[BECKECAMX]=-0.37 (negative-weight canary)"
    - "Functional::set('b3lyp', 1.0); set('slaterx', 0.5); get('slaterx') == 1.30 (additive accumulation)"
    - "Functional::set('b3lyp', 0.5); get('exx') == 0.10 (parameter overwrite, not additive)"
    - "XC_EXX default 0.0, XC_RANGESEP_MU default 0.4, XC_CAM_ALPHA default 0.19, XC_CAM_BETA default 0.46 are readable via get()"
    - "All 46 aliases resolve × value=1.0 without XcError::UnknownName"
    - "Case-insensitive lookup: set('B3LYP', 1.0) and set('b3lyp', 1.0) produce identical settings"
  artifacts:
    - path: crates/xcfun-core/src/enums.rs
      provides: ParameterId enum (4 variants, #[repr(u32)], discriminants 78..=81)
      contains: "ParameterId"
    - path: crates/xcfun-core/src/registry/generated/ALIASES.rs
      provides: ALIASES static slice (46 entries)
      contains: "camcompx"
    - path: crates/xcfun-core/src/registry/generated/parameters.rs
      provides: PARAMETERS static slice (4 entries with defaults)
      contains: "XC_RANGESEP_MU"
    - path: crates/xcfun-eval/src/functional.rs
      provides: Functional::set + Functional::get with alias recursion
      contains: "fn set"
  key_links:
    - from: crates/xcfun-eval/src/functional.rs
      to: crates/xcfun-core/src/registry/generated/ALIASES.rs
      via: Functional::set alias lookup
      pattern: "xcfun_core::ALIASES"
    - from: crates/xcfun-eval/src/functional.rs
      to: crates/xcfun-core/src/enums.rs
      via: ParameterId enum discriminant lookup
      pattern: "ParameterId"
---

<objective>
Implement the alias engine and parameter table: populate the 46-entry alias registry and 4-entry parameter registry via `xtask regen-registry` extension, add `ParameterId` enum to xcfun-core, implement `Functional::set` (3-case recursion matching `XCFunctional.cpp:369-405`) and `Functional::get` (functional + parameter only), expand `settings[]` from `[f64; 78]` to `[f64; 82]` with parameter defaults.

This plan implements the exact C++ semantics including the FIXME at L390 (EXX weighted through aliases by `value * weight` — parameter overwrite semantics). Algorithmic-identity rule forbids "fixing" the FIXME.

Output: 6 files modified. All 46 alias canary tests GREEN. parameters at correct defaults.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md

<interfaces>
<!-- From Phase 2 substrate. -->

From crates/xcfun-core/src/functional_id.rs (existing analog):
```rust
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum FunctionalId {
    XC_SLATERX = 0,
    // ... 77 more variants
    COUNT = 78,
}

pub fn functional_id_from_name(name: &str) -> Option<FunctionalId> { ... }
```

From crates/xcfun-core/src/registry/generated/ALIASES.rs (current state — empty stub):
```rust
pub static ALIASES: &[Alias] = &[];
```

From xcfun-master/src/XCFunctional.cpp:369-405 (verbatim port target):
```cpp
int xcfun_set(XCFunctional * fun, const char * name, double value) {
  xcint_assure_setup();
  int item;
  if ((item = xcint_lookup_functional(name)) >= 0) {
    fun->settings[item] += value;                          // case 1: functional ADDITIVE
    // ... activate functional ...
    return 0;
  } else if ((item = xcint_lookup_parameter(name)) >= 0) {
    fun->settings[item] = value;                           // case 2: parameter OVERWRITE
    return 0;
  } else if ((item = xcint_lookup_alias(name)) >= 0) {
    for (int i = 0; i < MAX_ALIAS_TERMS; i++) {
      if (!xcint_aliases[item].terms[i].name) break;
      if (xcfun_set(fun, xcint_aliases[item].terms[i].name,
                    value * xcint_aliases[item].terms[i].weight) != 0) {  // case 3: recursive
        xcfun::die(...);
      }
    }
    return 0;
  }
  return -1;  // XcError::UnknownName
}
```

From crates/xcfun-core/src/registry/generated/VARS_TABLE.rs (static slice pattern):
```rust
pub static VARS_TABLE: &[VarsEntry] = &[
    VarsEntry { id: 0, name: "XC_A_B", len: 2, provides: ... },
    // ...
];
```

From crates/xcfun-eval/src/functional.rs (Functional struct — needs settings[] expansion):
```rust
pub struct Functional {
    pub vars: Vars,
    pub mode: Mode,
    pub order: u32,
    pub parameters: [f64; 4],  // D-05: REPLACE with settings: [f64; 82]
    // ...
}
```
</interfaces>
</context>

<tasks>

<task id="4.1" type="auto" tdd="true">
  <name>Task 1: ParameterId enum + parameter registry + alias registry + xtask extension</name>
  <files>
    crates/xcfun-core/src/enums.rs,
    crates/xcfun-core/src/registry/generated/ALIASES.rs,
    crates/xcfun-core/src/registry/generated/parameters.rs,
    crates/xcfun-core/src/lib.rs,
    xtask/src/bin/regen_registry.rs
  </files>
  <read_first>
    - `crates/xcfun-core/src/enums.rs` — READ FULLY. Must add `ParameterId` enum alongside existing `FunctionalId`, `Mode`, `Vars`, `Dependency` enums.
    - `crates/xcfun-core/src/functional_id.rs` — READ FULLY. `ParameterId` must follow the same `#[repr(u32)]` + `functional_id_from_name` pattern.
    - `crates/xcfun-core/src/lib.rs` — READ to see existing re-exports; add `ParameterId` re-export.
    - `crates/xcfun-core/src/registry/generated/ALIASES.rs` — current state (empty slice).
    - `crates/xcfun-core/src/registry/generated/VARS_TABLE.rs` — pattern for the new `parameters.rs` static slice.
    - `xcfun-master/src/functionals/list_of_functionals.hpp` lines 99-105 — AUTHORITATIVE: `XC_RANGESEP_MU = XC_NR_FUNCTIONALS, XC_EXX, XC_CAM_ALPHA, XC_CAM_BETA, XC_NR_PARAMETERS_AND_FUNCTIONALS`. Discriminants 78..=81.
    - `xcfun-master/src/functionals/common_parameters.cpp` lines 17-29 — AUTHORITATIVE parameter defaults: `XC_RANGESEP_MU=0.4, XC_EXX=0.0, XC_CAM_ALPHA=0.19, XC_CAM_BETA=0.46`.
    - `xcfun-master/src/functionals/aliases.cpp` lines 17-139 — FULL READ. ALL 46 alias entries with terms and weights. The RESEARCH §"All 46 aliases enumerated" table lists all 46; use it as a reference but read the C++ source to verify exact weight values.
    - `xtask/src/bin/regen_registry.rs` — READ FULLY to understand the existing extractor pipeline; extend to parse `aliases.cpp` and `common_parameters.cpp`.
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-04, D-04-A, D-04-B, D-05, D-05-A
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md` — §"All 46 aliases enumerated" (complete table) + §"Parameter Table"
  </read_first>
  <behavior>
    - Test 1: `ParameterId::XC_RANGESEP_MU as u32 == 78`
    - Test 2: `ParameterId::XC_EXX as u32 == 79`
    - Test 3: `ParameterId::XC_CAM_ALPHA as u32 == 80`
    - Test 4: `ParameterId::XC_CAM_BETA as u32 == 81`
    - Test 5: `ALIASES.len() == 46`
    - Test 6: `ALIASES.iter().any(|a| a.name.eq_ignore_ascii_case("camcompx"))` is true
    - Test 7: `PARAMETERS` static slice has 4 entries with defaults [0.4, 0.0, 0.19, 0.46]
  </behavior>
  <action>
    **1. `crates/xcfun-core/src/enums.rs` — add `ParameterId` enum:**
    ```rust
    #[allow(non_camel_case_types)]
    #[repr(u32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ParameterId {
        XC_RANGESEP_MU = 78,
        XC_EXX        = 79,
        XC_CAM_ALPHA  = 80,
        XC_CAM_BETA   = 81,
    }

    impl ParameterId {
        pub fn from_name(name: &str) -> Option<Self> {
            // case-insensitive lookup per D-04-B
            if name.eq_ignore_ascii_case("rangesep_mu") || name.eq_ignore_ascii_case("XC_RANGESEP_MU") {
                Some(Self::XC_RANGESEP_MU)
            } else if name.eq_ignore_ascii_case("exx") || name.eq_ignore_ascii_case("XC_EXX") {
                Some(Self::XC_EXX)
            } else if name.eq_ignore_ascii_case("cam_alpha") || name.eq_ignore_ascii_case("XC_CAM_ALPHA") {
                Some(Self::XC_CAM_ALPHA)
            } else if name.eq_ignore_ascii_case("cam_beta") || name.eq_ignore_ascii_case("XC_CAM_BETA") {
                Some(Self::XC_CAM_BETA)
            } else {
                None
            }
        }
        pub const fn default_value(self) -> f64 {
            match self {
                Self::XC_RANGESEP_MU => 0.4,  // from common_parameters.cpp:17
                Self::XC_EXX        => 0.0,   // from common_parameters.cpp:20
                Self::XC_CAM_ALPHA  => 0.19,  // from common_parameters.cpp:23
                Self::XC_CAM_BETA   => 0.46,  // from common_parameters.cpp:26
            }
        }
    }
    ```
    Add `thiserror`-compatible display impl if needed (same pattern as `XcError`). Do NOT add `anyhow` here (library crate).

    **2. `crates/xcfun-core/src/registry/generated/parameters.rs` (NEW):**
    ```rust
    /// Parameter table — port of xcfun-master/src/functionals/common_parameters.cpp:17-27
    /// + list_of_functionals.hpp:100-104.
    use crate::ParameterId;

    pub struct ParameterEntry {
        pub id: ParameterId,
        pub name: &'static str,
        pub description: &'static str,
        pub default: f64,
    }

    pub static PARAMETERS: &[ParameterEntry] = &[
        ParameterEntry { id: ParameterId::XC_RANGESEP_MU, name: "rangesep_mu", description: "Range separation inverse length [1/a0]", default: 0.4 },
        ParameterEntry { id: ParameterId::XC_EXX,        name: "exx",         description: "Exact exchange admixture",               default: 0.0 },
        ParameterEntry { id: ParameterId::XC_CAM_ALPHA,  name: "cam_alpha",   description: "Coulomb-attenuating method alpha",       default: 0.19 },
        ParameterEntry { id: ParameterId::XC_CAM_BETA,   name: "cam_beta",    description: "Coulomb-attenuating method beta",        default: 0.46 },
    ];
    ```

    **3. `crates/xcfun-core/src/registry/generated/ALIASES.rs` — populate 46 entries:**

    The `Alias` struct should already exist from Phase 2 (CORE-08 pipeline); verify its shape. Each alias entry has `name: &'static str`, `description: &'static str`, and `terms: &'static [AliasTerm]` where `AliasTerm { name: &'static str, weight: f64 }`.

    Port ALL 46 entries from `xcfun-master/src/functionals/aliases.cpp:17-139`. Read the C++ file verbatim. For the `camcompx` canary (alias #6 in the RESEARCH table; lines 119-125 in aliases.cpp), the terms must include `{"beckecamx", -1.0}` (negative weight).

    The RESEARCH table in §"All 46 aliases enumerated" lists all 46 entries — use it as a cross-reference but always verify against the C++ source.

    This is DATA entry work — must be exact. Every weight value must match `aliases.cpp` to 16 decimal digits. Use `f64` literals with full precision from the C++ source (e.g., `kt2` has `slaterx: 1.07173`, `vwn5c: 0.576727`).

    Add the `xtask regen-registry --check` drift gate assertion (compare against `aliases.cpp` hash).

    **4. `xtask/src/bin/regen_registry.rs` — extend:**
    Add a `gen_aliases()` function that parses `xcfun-master/src/functionals/aliases.cpp:17-139` and generates the 46-entry `ALIASES.rs` slice. Add a `gen_parameters()` function that parses `common_parameters.cpp:17-29` and generates `parameters.rs`. Both must be invoked when `xtask regen-registry` is run. The `--check` flag diffs the generated output against the committed file (existing pattern for `FUNCTIONAL_DESCRIPTORS.rs`).

    **5. `crates/xcfun-core/src/lib.rs`:** re-export `ParameterId`, `PARAMETERS`, and `ALIASES` (or ensure they're accessible via the public API).
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-core` exits 0.
    2. `cargo test -p xcfun-core` — unit tests for ParameterId discriminants pass: `assert_eq!(ParameterId::XC_RANGESEP_MU as u32, 78)` etc.
    3. `grep -c "AliasTerm" crates/xcfun-core/src/registry/generated/ALIASES.rs | wc -c` — file has 46+ AliasTerm entries.
    4. `grep -n "camcompx" crates/xcfun-core/src/registry/generated/ALIASES.rs` finds the entry.
    5. `grep -n "beckecamx.*-1" crates/xcfun-core/src/registry/generated/ALIASES.rs` finds the negative-weight term.
    6. `grep -n "XC_RANGESEP_MU" crates/xcfun-core/src/registry/generated/parameters.rs` finds the entry with `default: 0.4`.
    7. `cargo xtask regen-registry --check` exits 0 (generated == committed, no drift).
  </acceptance_criteria>
  <done>ParameterId enum correct. ALIASES has 46 entries. PARAMETERS has 4 entries. xtask regen-registry --check GREEN.</done>
</task>

<task id="4.2" type="auto" tdd="true">
  <name>Task 2: Functional::set/get recursion engine + settings[82] expansion</name>
  <files>
    crates/xcfun-eval/src/functional.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/src/functional.rs` — READ FULLY. Must understand the current `Functional` struct (has `parameters: [f64; 4]`), the `eval` method, `eval_setup`, `output_length`, and the `Functional::new()` constructor.
    - `xcfun-master/src/XCFunctional.cpp` lines 347-358 (constructor: parameter initialization), 369-405 (xcfun_set), 407-419 (xcfun_get) — AUTHORITATIVE line-for-line port target.
    - `xcfun-master/src/xcint.hpp` lines 26-28 — `MAX_ALIAS_TERMS = 10`, `XC_MAX_ALIASES = 60`.
    - `crates/xcfun-core/src/enums.rs` — `ParameterId` enum (from Task 1 of this plan).
    - `crates/xcfun-core/src/registry/generated/ALIASES.rs` — `ALIASES` static slice (from Task 1).
    - `crates/xcfun-core/src/registry/generated/parameters.rs` — `PARAMETERS` static slice (from Task 1).
    - `crates/xcfun-core/src/functional_id.rs` — `functional_id_from_name` (case-insensitive per D-04-B).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-04, D-04-A, D-04-B, D-05, D-14 (no new XcError variants)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md` — §"Alias Engine Semantics" (verbatim trace for b3lyp, camcompx, additive accumulation)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md` — Tier-2 alias canary tests (b3lyp, camcompx, additive, case-insensitive)
  </read_first>
  <behavior>
    - Test 1: `Functional::new()` produces `settings[78..82] == [0.4, 0.0, 0.19, 0.46]` (parameter defaults)
    - Test 2: `set("b3lyp", 1.0)` → `settings[SLATERX_id] == 0.80`, `settings[EXX_id=79] == 0.20`
    - Test 3: `set("b3lyp", 1.0); set("slaterx", 0.5); get("slaterx") == 1.30`
    - Test 4: `set("b3lyp", 0.5); get("exx") == 0.10` (NOT 0.30 — overwrite semantics)
    - Test 5: `set("camcompx", 0.37); get("beckecamx") == -0.37` (negative-weight canary)
    - Test 6: `set("B3LYP", 1.0)` and `set("b3lyp", 1.0)` produce identical settings arrays
    - Test 7: `set("unknown_functional_xyz", 1.0)` returns `Err(XcError::UnknownName)`
    - Test 8: `get("b3lyp", ...)` returns `Err(XcError::UnknownName)` (aliases are NOT readable via get)
    - Test 9: `get("exx")` returns `Ok(0.0)` (default parameter value)
  </behavior>
  <action>
    **Expand `Functional` struct settings array:**

    Replace `parameters: [f64; 4]` with `settings: [f64; 82]`. Update `Functional::new()` to initialize:
    ```rust
    let mut settings = [0.0f64; 82];
    // Parameter defaults at indices 78..=81 per D-05 + common_parameters.cpp:17-27
    settings[78] = 0.4;   // XC_RANGESEP_MU
    settings[79] = 0.0;   // XC_EXX
    settings[80] = 0.19;  // XC_CAM_ALPHA
    settings[81] = 0.46;  // XC_CAM_BETA
    ```
    Update any existing code that reads `self.parameters[i]` to use `self.settings[78 + i]` or the `ParameterId as u32` index.

    **Implement `Functional::set` — port of `XCFunctional.cpp:369-405`:**
    ```rust
    pub fn set(&mut self, name: &str, value: f64) -> Result<(), XcError> {
        // Case 1: Functional name match → ADDITIVE accumulation (C++ L373)
        if let Some(id) = xcfun_core::FunctionalId::from_name(name) {
            self.settings[id as usize] += value;   // additive per L373
            // Activate: add to active list if not already present; OR depends.
            if !self.active_functionals.contains(&id) {
                self.active_functionals.push(id);
                // self.depends |= FUNCTIONAL_DESCRIPTORS[id as usize].depends;
            }
            return Ok(());
        }
        // Case 2: Parameter name match → OVERWRITE (C++ L379-L382)
        if let Some(pid) = xcfun_core::ParameterId::from_name(name) {
            self.settings[pid as usize] = value;   // overwrite per L381
            return Ok(());
        }
        // Case 3: Alias name match → RECURSIVE (C++ L383-L398)
        if let Some(alias) = xcfun_core::ALIASES.iter()
            .find(|a| a.name.eq_ignore_ascii_case(name))
        {
            for term in alias.terms {
                // FIXME note: parameter weights ARE multiplied by value (C++ L390 FIXME preserved)
                self.set(term.name, value * term.weight)?;
            }
            return Ok(());
        }
        // No match → UnknownName
        Err(XcError::UnknownName)
    }
    ```
    The depth guard: aliases-of-aliases do NOT exist in the xcfun source (verified by reading all 46 entries). Maximum recursion depth = 1 alias → functional/parameter terms. No explicit depth counter needed; the algorithm terminates because no alias refers to another alias. Add a comment documenting this invariant.

    **Implement `Functional::get` — port of `XCFunctional.cpp:407-419`:**
    ```rust
    pub fn get(&self, name: &str) -> Result<f64, XcError> {
        // Case 1: Functional name match
        if let Some(id) = xcfun_core::FunctionalId::from_name(name) {
            return Ok(self.settings[id as usize]);
        }
        // Case 2: Parameter name match
        if let Some(pid) = xcfun_core::ParameterId::from_name(name) {
            return Ok(self.settings[pid as usize]);
        }
        // Aliases NOT readable via get (matches C++ xcfun_get at L419 which returns -1 for aliases)
        Err(XcError::UnknownName)
    }
    ```

    **Update `active_functionals` management:** The current `Functional` struct may store active functionals differently. Align with the C++ pattern: `active_functionals` is a Vec/array of FunctionalId (or indices) that track which functionals have non-zero weight. When `set("slaterx", 0.5)` is called for the second time (already active from b3lyp), the accumulation happens but no duplicate added.

    **Alias canary unit tests** — add in `functional.rs` or a companion `tests/alias_canary.rs`:
    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_b3lyp_trace() {
            let mut f = Functional::new();
            f.set("b3lyp", 1.0).unwrap();
            // Verify exact weight set from RESEARCH §"Concrete trace: set('b3lyp', 1.0)"
            assert!((f.get("slaterx").unwrap() - 0.80).abs() < 1e-15);
            assert!((f.get("beckecorrx").unwrap() - 0.72).abs() < 1e-15);
            assert!((f.get("exx").unwrap() - 0.20).abs() < 1e-15);
        }
        #[test]
        fn test_b3lyp_additive_accumulation() {
            let mut f = Functional::new();
            f.set("b3lyp", 1.0).unwrap();
            f.set("slaterx", 0.5).unwrap();
            let w = f.get("slaterx").unwrap();
            assert!((w - 1.30).abs() < 1e-15, "expected 1.30, got {}", w);
        }
        #[test]
        fn test_exx_parameter_overwrite() {
            let mut f = Functional::new();
            f.set("b3lyp", 0.5).unwrap();
            let exx = f.get("exx").unwrap();
            assert!((exx - 0.10).abs() < 1e-15, "expected 0.10 (overwrite), got {}", exx);
        }
        #[test]
        fn test_camcompx_negative_weight() {
            let mut f = Functional::new();
            f.set("camcompx", 0.37).unwrap();
            let w = f.get("beckecamx").unwrap();
            assert!((w - (-0.37)).abs() < 1e-15, "expected -0.37, got {}", w);
        }
        #[test]
        fn test_case_insensitive() {
            let mut f1 = Functional::new();
            let mut f2 = Functional::new();
            f1.set("b3lyp", 1.0).unwrap();
            f2.set("B3LYP", 1.0).unwrap();
            for i in 0..82 { assert_eq!(f1.settings[i], f2.settings[i]); }
        }
        #[test]
        fn test_parameter_defaults() {
            let f = Functional::new();
            assert_eq!(f.get("rangesep_mu").unwrap(), 0.4);
            assert_eq!(f.get("exx").unwrap(), 0.0);
            assert_eq!(f.get("cam_alpha").unwrap(), 0.19);
            assert_eq!(f.get("cam_beta").unwrap(), 0.46);
        }
        #[test]
        fn test_all_46_aliases_resolve() {
            let mut f = Functional::new();
            for alias in xcfun_core::ALIASES {
                assert!(f.set(alias.name, 1.0).is_ok(),
                    "alias '{}' returned UnknownName", alias.name);
            }
        }
    }
    ```

    IMPORTANT: `thiserror` only in library crates; no `anyhow`. `XcError::UnknownName` variant already exists (Phase 2 D-25 — no new variants per D-14). No `mul_add` anywhere in this file.
  </action>
  <acceptance_criteria>
    1. `cargo test -p xcfun-eval test_b3lyp_trace` passes with all 5 weight assertions correct.
    2. `cargo test -p xcfun-eval test_b3lyp_additive_accumulation` passes: `get("slaterx") == 1.30`.
    3. `cargo test -p xcfun-eval test_exx_parameter_overwrite` passes: `get("exx") == 0.10`.
    4. `cargo test -p xcfun-eval test_camcompx_negative_weight` passes: `get("beckecamx") == -0.37`.
    5. `cargo test -p xcfun-eval test_case_insensitive` passes (identical settings for B3LYP vs b3lyp).
    6. `cargo test -p xcfun-eval test_parameter_defaults` passes: all 4 defaults correct.
    7. `cargo test -p xcfun-eval test_all_46_aliases_resolve` passes: all 46 aliases return Ok.
    8. `cargo build -p xcfun-eval --release` exits 0.
    9. `grep -n "anyhow" crates/xcfun-eval/src/functional.rs` returns empty (no anyhow in library).
    10. `grep -n "settings\[78\]" crates/xcfun-eval/src/functional.rs` confirms settings array initialised with parameter defaults.
  </acceptance_criteria>
  <done>All 9 alias canary tests GREEN. Functional::set/get correctly implements 3-case recursion. settings[82] with parameter defaults. No anyhow. No mul_add.</done>
</task>

</tasks>

<verification>
```bash
# Unit tests
cargo test -p xcfun-eval 2>&1 | grep -E "test_b3lyp|test_camcompx|test_exx|test_param|test_case|test_all_46|FAILED|ok" | head -20
cargo test -p xcfun-core 2>&1 | grep -E "parameter|alias|FAILED|ok" | head -10

# Structural checks
grep -n "camcompx" crates/xcfun-core/src/registry/generated/ALIASES.rs
grep -n "beckecamx.*-1" crates/xcfun-core/src/registry/generated/ALIASES.rs
grep -n "XC_RANGESEP_MU\|0\.4" crates/xcfun-core/src/registry/generated/parameters.rs | head -5
grep -n "settings\[82\]\|settings: \[f64; 82\]" crates/xcfun-eval/src/functional.rs

# Drift gate
cargo xtask regen-registry --check 2>&1 | tail -5

# No anyhow in library
grep -rn "anyhow" crates/xcfun-eval/src/functional.rs
```
</verification>

<success_criteria>
- All 9 canary unit tests GREEN.
- ALIASES has exactly 46 entries.
- ParameterId discriminants 78..=81 verified by unit test.
- `xtask regen-registry --check` exits 0.
- `cargo build -p xcfun-eval --release` exits 0.
- No `anyhow` in `xcfun-eval` library code.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Functional::set public API | User-supplied name string → alias/functional/parameter lookup. Name is not sanitized against injection — but it is only used for equality lookup against a static known set. No arbitrary code execution path. |
| Alias resolution recursion | User calls set(alias_name, value); recursion resolves into functional/parameter set calls. Max depth = 1 (invariant: no alias refers to another alias in the xcfun source). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-04-01 | Tampering | Alias name string lookup allowing unexpected name collisions | mitigate | Lookup is sequential: functional → parameter → alias. A name can only match ONE category per the C++ priority order. Case-insensitive `eq_ignore_ascii_case` prevents case-variation bypass. The static alias table is checked-in + drift-gated. |
| T-04-04-02 | Elevation of Privilege | Alias recursion → arbitrary state mutation via crafted name | mitigate | Aliases resolve ONLY to names present in the static `ALIASES` table (46 known-good entries, drift-gated). No user-controlled alias term names. Max recursion depth = 1 (structural invariant). |
| T-04-04-03 | Tampering | Parameter overwrite semantics (case ② in Functional::set) allows resetting XC_EXX to 0 | accept | This IS the documented C++ FIXME behaviour. The caller controls parameter values. No untrusted external input in this library's phase-4 scope; Phase 5 C ABI adds the next trust boundary. |
| T-04-04-04 | Denial of Service | Integer overflow in `settings[id as usize]` if id >= 82 | mitigate | FunctionalId max discriminant = 77; ParameterId max = 81. Static bounds. `settings: [f64; 82]` covers 0..=81. Index out of bounds would panic — but all lookup paths bound-check via enum discriminant. The panic would be caught by Phase 5 `catch_unwind` at the C FFI boundary per Phase 3 D-13. |
| T-04-04-05 | Information Disclosure | get() on alias name returns UnknownName (not a secret) | accept | Correct behaviour; matches C++ xcfun_get returning -1 for aliases. Not a security issue. |
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-04-SUMMARY.md`
</output>
