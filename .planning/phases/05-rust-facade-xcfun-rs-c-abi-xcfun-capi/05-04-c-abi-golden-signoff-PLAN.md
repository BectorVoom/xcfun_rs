---
plan_id: 05-04-c-abi-golden-signoff
phase: 05
wave: 5
depends_on:
  - 05-03-cbindgen-headers-match
files_modified:
  - crates/xcfun-capi/Cargo.toml
  - crates/xcfun-capi/tests/c_abi.c
  - crates/xcfun-capi/tests/c_abi.rs
  - crates/xcfun-capi/tests/fixtures/expected.json
  - .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
requirements:
  - CAPI-07
autonomous: true
---

## objective
<objective>
Phase 5 Wave 5 — close the loop on CAPI-07 by exercising
`crates/xcfun-capi/tests/c_abi.c` against `libxcfun_capi.a` + the
generated `xcfun-capi/include/xcfun.h`, then sign off the phase:

1. **`crates/xcfun-capi/tests/c_abi.c`** — hand-written C source per
   D-14 implementing **10 fixture functions** covering LDA, GGA,
   metaGGA (TPSSX, M06, SCANX), alias (additive + range-separated),
   Mode::Contracted, and Mode::Potential. The 10-row D-14 fixture
   table is preserved verbatim:

   | # | Functional | Vars | Mode | Order |
   |---|-----------|------|------|-------|
   | 1 | LDA | XC_A_B | PartialDerivatives | 0 |
   | 2 | PBE | XC_A_B_AX_AY_AZ_BX_BY_BZ | PartialDerivatives | 1 |
   | 3 | BECKEX | XC_N_NX_NY_NZ | PartialDerivatives | 2 |
   | 4 | B3LYP (alias → 5 terms) | XC_A_B_AX_AY_AZ_BX_BY_BZ | PartialDerivatives | 1 |
   | 5 | PBE0 (alias) | XC_N_NX_NY_NZ | PartialDerivatives | 1 |
   | 6 | M06 (alias = m06c + m06x; metaGGA) | XC_A_B_GAA_GAB_GBB_TAUA_TAUB | PartialDerivatives | 0 |
   | 7 | M06X | XC_A_B_GAA_GAB_GBB_TAUA_TAUB | Contracted | 3 |
   | 8 | SCANX | XC_A_B_GAA_GAB_GBB_TAUA_TAUB | PartialDerivatives | 0 |
   | 9 | CAMB3LYP (alias, range-separated) | XC_A_B_AX_AY_AZ_BX_BY_BZ | PartialDerivatives | 0 |
   | 10 | LB94 → LDA (substitute) | XC_A_B | Potential | 0 |

   **Row 8 (SCANX) authorized fallback (CONTEXT D-14 specifics):** if
   SCANX fails Tier-1 self-tests at execution time, substitute
   `TPSSX` (also metaGGA, no SCAN substrate dependency). This
   substitution MUST trigger an **Escalation Gate** event — recorded
   in 05-04-SUMMARY.md and surfaced to the user before the phase
   sign-off proceeds. The fallback is NOT pre-authorized for any
   other row.

   **Row 10 (LB94) — non-evaluability caveat:** Per D-16 verification,
   `xcfun-master/src/functionals/lb94.cpp:15` opens with `#if 0 //
   Does not work and should maybe not be here in the first place`.
   The entire LB94 body (lines 15-97) is dead code in the upstream
   C++ tree — there is no working LB94 functional body to call. For
   the C-ABI golden test, **the runtime call uses LDA on Mode::Potential
   instead** (LB94 cannot pass a parity check against a non-existent
   reference). The LB94 *descriptor* is still added to the registry
   per D-16 (Plan 05-00 Task 0.4 — see "LB94 descriptor add-back"
   below); the C-ABI test simply does not invoke it. This is a
   documented CONTEXT-decision-drift caveat surfaced in the
   05-VERIFICATION matrix.

   **Total fixtures executed: 10** (rows 1-9 verbatim per D-14; row
   10 evaluates LDA in Mode::Potential as the documented LB94 substitute).

2. **`crates/xcfun-capi/tests/c_abi.rs`** — Rust integration test that
   uses `cc` at test-runtime to compile `tests/c_abi.c`, link it
   against `libxcfun_capi.a`, run the resulting binary, and assert
   exit code == 0. Mirrors the validation crate's cc::Build pattern.

3. **`crates/xcfun-capi/tests/fixtures/expected.json`** — pre-computed
   reference values for the 10 fixtures, computed ONCE during plan
   execution by calling `xcfun_rs::Functional::eval` with the
   per-fixture density input. Loaded by the executor and embedded
   into `c_abi.c` as `static const double expected_<n>[]` blocks.

4. **`crates/xcfun-capi/Cargo.toml`** — add `cc = { workspace = true }`
   to `[dev-dependencies]` so `tests/c_abi.rs` can drive the C compiler.

5. **Phase 5 sign-off artifacts**:
   - `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md` — verdict matrix per requirement.
   - `.planning/REQUIREMENTS.md` — mark RS-01..07 / RS-09 / RS-10 / CAPI-01..07 as `[x] Complete`.
   - `.planning/ROADMAP.md` — mark Phase 5 as complete; tick the 5 success criteria.
   - `.planning/STATE.md` — advance to Phase 6 head-of-line.

Output: a Rust-compiled C binary that exercises the C ABI on 10
fixtures, all returning relative-error ≤ 1e-12 against pre-computed
Rust references, plus a complete sign-off package.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-PATTERNS.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-00-SUMMARY.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-01-SUMMARY.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-02-SUMMARY.md
@.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-03-SUMMARY.md

# Source files this plan reads / drives
@crates/xcfun-capi/include/xcfun.h
@crates/xcfun-capi/src/lib.rs
@crates/xcfun-rs/src/functional.rs

# cc::Build pattern reference
@validation/build.rs

# Upstream lb94 disclaimer (verifies row-10 caveat rationale)
@xcfun-master/src/functionals/lb94.cpp

<interfaces>
<!-- 10 fixtures used in tests/c_abi.c — D-14 verbatim with the documented LB94 row-10 caveat -->

| # | Functional | Vars              | Mode               | Order | Density inputs (length matches Vars::input_len) |
|---|-----------|-------------------|--------------------|-------|--------------------------------------------------|
| 1 | LDA       | XC_A_B (2)        | PartialDerivatives | 0 | [0.5, 0.5] |
| 2 | PBE       | XC_A_B_AX_AY_AZ_BX_BY_BZ (8) | PartialDerivatives | 1 | [0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0] |
| 3 | BECKEX    | XC_N_NX_NY_NZ (4) | PartialDerivatives | 2 | [1.0, 0.2, 0.0, 0.0] |
| 4 | B3LYP     | XC_A_B_AX_AY_AZ_BX_BY_BZ (8) | PartialDerivatives | 1 | [0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0] |
| 5 | PBE0      | XC_N_NX_NY_NZ (4) | PartialDerivatives | 1 | [1.0, 0.2, 0.0, 0.0] |
| 6 | M06       | XC_A_B_GAA_GAB_GBB_TAUA_TAUB (7) | PartialDerivatives | 0 | [0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05] |
| 7 | M06X      | XC_A_B_GAA_GAB_GBB_TAUA_TAUB (7) | Contracted | 3 | (7 inputs × 8 = 56 doubles, see expected.json) |
| 8 | SCANX     | XC_A_B_GAA_GAB_GBB_TAUA_TAUB (7) | PartialDerivatives | 0 | [0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05] |
| 9 | CAMB3LYP  | XC_A_B_AX_AY_AZ_BX_BY_BZ (8) | PartialDerivatives | 0 | [0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0] |
| 10 | LB94 → LDA (substitute, see caveat) | XC_A_B (2) | Potential | 0 | [0.5, 0.5] |

The Mode::Potential output for fixture 10 has length 3 (xcfun_output_length
returns 3 for spin-resolved Vars per Functional::output_length).

The Mode::Contracted fixture (7) at order 3 produces `1 << 3 = 8` outputs.

Each fixture's expected[] array length:
- fixture 1: outlen = taylorlen(2, 0) = 1
- fixture 2: outlen = taylorlen(8, 1) = 9
- fixture 3: outlen = taylorlen(4, 2) = 15
- fixture 4: outlen = taylorlen(8, 1) = 9
- fixture 5: outlen = taylorlen(4, 1) = 5
- fixture 6: outlen = taylorlen(7, 0) = 1
- fixture 7: outlen = 8 (Contracted, 1 << 3)
- fixture 8: outlen = taylorlen(7, 0) = 1
- fixture 9: outlen = taylorlen(8, 0) = 1
- fixture 10: outlen = 3 (Potential, spin-resolved)

Total expected doubles: 1+9+15+9+5+1+8+1+1+3 = 53 reference values.
</interfaces>
</context>

## must_haves
<must_haves>
truths:
  - "`cargo test -p xcfun-capi --test c_abi` exits 0. The test builds tests/c_abi.c via cc (test-runtime), links libxcfun_capi.a, runs the resulting binary, asserts exit code 0. (CAPI-07)"
  - "Each of 10 fixtures' relative error ≤ 1e-12 against the pre-computed Rust reference (per element). (CAPI-07, D-14)"
  - "`tests/c_abi.c` includes `xcfun.h` (the GENERATED header, not the upstream one) and links the staticlib. (CAPI-07, D-14)"
  - "`tests/c_abi.c` uses `xcfun_new` / `xcfun_delete` per fixture; calls `xcfun_set` / `xcfun_eval_setup` / `xcfun_eval` (or `xcfun_eval_vec` for fixture 7's Contracted mode); compares each output to the embedded `expected_<n>[]` block."
  - "`c_abi.rs` does NOT pass `-ffast-math` / `-Cfast-math` to the cc invocation; passes `-fno-fast-math -ffp-contract=off` per CLAUDE.md ACC-05/06."
  - "Row 8 SCANX→TPSSX fallback (if it triggers) is recorded as an Escalation Gate event in 05-04-SUMMARY.md and presented to the user. (D-14 specifics)"
  - "Row 10 evaluates LDA on Mode::Potential as a documented LB94 substitute. The LB94 descriptor is present in xcfun-core (added by Plan 05-00 Task 0.4 per D-16) but is NOT invoked at runtime because the upstream lb94.cpp body is `#if 0`'d."
  - "Phase 5 sign-off artifacts written: 05-VERIFICATION.md (verdict matrix with the LB94 caveat block), REQUIREMENTS.md updates (16 reqs marked Complete), ROADMAP.md tick, STATE.md advance to Phase 6."
artifacts:
  - path: "crates/xcfun-capi/tests/c_abi.c"
    provides: "Hand-written C source — 10 fixture functions + main"
    contains: "xcfun_new"
  - path: "crates/xcfun-capi/tests/c_abi.rs"
    provides: "Rust test compiling+linking+running the C binary"
    contains: "cc::Build"
  - path: "crates/xcfun-capi/tests/fixtures/expected.json"
    provides: "Pre-computed Rust reference values (committed; serves as the embedded constant in c_abi.c)"
    contains: "fixtures"
  - path: ".planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md"
    provides: "Phase 5 verdict matrix (16 requirements)"
    contains: "RS-07"
  - path: ".planning/REQUIREMENTS.md"
    provides: "RS-01..07/09/10 + CAPI-01..07 marked [x] Complete with Phase 5 link"
    contains: "[x] **RS-01**"
  - path: ".planning/ROADMAP.md"
    provides: "Phase 5 marked [x] Complete"
    contains: "[x] **Phase 5"
  - path: ".planning/STATE.md"
    provides: "Phase 5 sign-off recorded; current focus advances to Phase 6"
    contains: "Phase 5 (Rust Facade"
key_links:
  - from: "crates/xcfun-capi/tests/c_abi.rs"
    to: "crates/xcfun-capi/tests/c_abi.c + target/release/libxcfun_capi.a + crates/xcfun-capi/include/xcfun.h"
    via: "cc::Build chain at test-runtime"
    pattern: "cc::Build"
  - from: "tests/c_abi.c"
    to: "xcfun_new + xcfun_set + xcfun_eval_setup + xcfun_eval"
    via: "C function calls through generated xcfun.h"
    pattern: "xcfun_new|xcfun_set|xcfun_eval"
</must_haves>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| **C source ↔ generated header ↔ Rust staticlib** | The whole point of CAPI-07: the C consumer should compile against the generated header, link against the staticlib, and produce numbers identical (within 1e-12) to a Rust direct call. Any divergence at this boundary is a phase regression. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-05-04-01 | Tampering — Float reassociation | cc invocation in tests/c_abi.rs | mitigate | Pass `-fno-fast-math -ffp-contract=off` to the cc invocation per CLAUDE.md ACC-05/06. CI gate prevents `-ffast-math` from creeping in. |
| T-05-04-02 | Tampering — Wrong header used | The generated `xcfun-capi/include/xcfun.h` vs. accidentally vendored upstream | mitigate | `tests/c_abi.rs` passes `--include-dir crates/xcfun-capi/include` (NOT `xcfun-master/api`) to the cc invocation. Test asserts the include path is correct. |
| T-05-04-03 | DoS — Unlinkable binary | libxcfun_capi.a not found at link time | mitigate | `tests/c_abi.rs` resolves the staticlib path via `env!("CARGO_TARGET_DIR")` + the active profile; runs `cargo build -p xcfun-capi --release` first if needed. Failure surfaces as test failure with a clear diagnostic. |
| T-05-04-04 | Disclosure — Test binary on disk | Compiled binary persists in `target/` | accept | Standard cargo behaviour; not a risk surface. |
</threat_model>

## tasks
<tasks>

<task type="auto">
  <name>Task 4.1: Generate expected.json + write tests/c_abi.c with embedded references (10 fixtures)</name>
  <files>crates/xcfun-capi/Cargo.toml, crates/xcfun-capi/tests/fixtures/expected.json, crates/xcfun-capi/tests/c_abi.c</files>
  <read_first>
    - crates/xcfun-capi/include/xcfun.h (Plan 05-03 output — the header c_abi.c includes)
    - crates/xcfun-rs/src/functional.rs (the Rust caller that pre-computes expected values)
    - crates/xcfun-eval/src/functional.rs:212-340 (Functional::eval signature)
    - crates/xcfun-core/src/registry/generated/FUNCTIONAL_DESCRIPTORS.rs (verify LB94 descriptor present after Plan 05-00 Task 0.4)
    - xcfun-master/src/functionals/lb94.cpp (verify line 15 `#if 0` to confirm the runtime substitution rationale)
    - xcfun-master/src/functionals/aliases.cpp:49 (verify `m06` alias = m06c + m06x)
  </read_first>
  <action>
    1. **Add `cc` dev-dep to `crates/xcfun-capi/Cargo.toml`:**
       ```toml
       [dev-dependencies]
       cc = { workspace = true }
       serde      = { workspace = true, features = ["derive"] }
       serde_json = { workspace = true }
       ```

    2. **Create a one-shot Rust generator example** at
       `crates/xcfun-capi/examples/gen_expected.rs` — invokes
       `xcfun_rs::Functional::eval` for each D-14 fixture, dumps the
       expected output array as a JSON file. Run via:
       ```bash
       cargo run -p xcfun-capi --example gen_expected
       ```

       ```rust
       //! Plan 05-04 helper — generate tests/fixtures/expected.json
       //! by calling xcfun_rs::Functional::eval for each D-14 fixture.
       //! Committed at tests/fixtures/expected.json.

       use std::fs;
       use std::path::Path;
       use xcfun_rs::{Functional, Mode, Vars};

       #[derive(serde::Serialize)]
       struct Fixture {
           id: u32, functional: String, vars: i32, mode: i32, order: i32,
           density: Vec<f64>, expected: Vec<f64>,
       }

       fn run(id: u32, name: &str, vars: Vars, mode: Mode, order: u32, density: &[f64])
           -> Fixture
       {
           let mut f = Functional::new();
           f.set(name, 1.0).unwrap();
           f.eval_setup(vars, mode, order).unwrap();
           let outlen = f.output_length().unwrap();
           let mut out = vec![0.0_f64; outlen];
           f.eval(density, &mut out).unwrap();
           Fixture {
               id, functional: name.into(), vars: vars as i32, mode: mode as i32,
               order: order as i32, density: density.to_vec(), expected: out,
           }
       }

       fn main() -> std::io::Result<()> {
           let mut fxs: Vec<Fixture> = Vec::new();
           // Fixture 1 — LDA / Partial / 0
           fxs.push(run(1, "lda", Vars::A_B, Mode::PartialDerivatives, 0, &[0.5, 0.5]));
           // Fixture 2 — PBE / Partial / 1
           fxs.push(run(2, "pbe", Vars::A_B_AX_AY_AZ_BX_BY_BZ, Mode::PartialDerivatives, 1,
                        &[0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0]));
           // Fixture 3 — BECKEX / Partial / 2
           fxs.push(run(3, "beckex", Vars::N_NX_NY_NZ, Mode::PartialDerivatives, 2,
                        &[1.0, 0.2, 0.0, 0.0]));
           // Fixture 4 — alias B3LYP
           fxs.push(run(4, "b3lyp", Vars::A_B_AX_AY_AZ_BX_BY_BZ, Mode::PartialDerivatives, 1,
                        &[0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0]));
           // Fixture 5 — alias PBE0
           fxs.push(run(5, "pbe0", Vars::N_NX_NY_NZ, Mode::PartialDerivatives, 1,
                        &[1.0, 0.2, 0.0, 0.0]));
           // Fixture 6 — alias M06 (= m06c + m06x; metaGGA)
           fxs.push(run(6, "m06", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
                        Mode::PartialDerivatives, 0,
                        &[0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05]));
           // Fixture 7 — M06X / Contracted / 3 (7 vars × 8 = 56 doubles).
           // Density layout: var-major flattening (same as xcfun-eval contracted launcher).
           let mut d7 = Vec::with_capacity(7 * 8);
           for var in &[0.5_f64, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05] {
               for _ in 0..8 { d7.push(*var); }
           }
           fxs.push(run(7, "m06x", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, Mode::Contracted, 3, &d7));
           // Fixture 8 — SCANX (metaGGA). Authorized fallback to TPSSX if Tier-1 self-tests
           // fail at execution time — the fallback MUST trigger an Escalation Gate event
           // (recorded in 05-04-SUMMARY.md and surfaced to the user).
           fxs.push(run(8, "scanx", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
                        Mode::PartialDerivatives, 0,
                        &[0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05]));
           // Fixture 9 — alias CAMB3LYP (range-separated)
           fxs.push(run(9, "camb3lyp", Vars::A_B_AX_AY_AZ_BX_BY_BZ, Mode::PartialDerivatives, 0,
                        &[0.5, 0.5, 0.1, 0.0, 0.0, 0.1, 0.0, 0.0]));
           // Fixture 10 — LB94 / Mode::Potential. Per D-16 + xcfun-master/src/functionals/
           // lb94.cpp:15 (`#if 0`), LB94 has no working upstream body. The LB94
           // descriptor is registered in xcfun-core (added by Plan 05-00 Task 0.4) but
           // its eval path returns XcError::Runtime. This fixture therefore evaluates
           // LDA on Mode::Potential as the documented substitute, preserving the
           // Mode::Potential coverage goal of D-14 row 10. The substitution is
           // recorded as a CONTEXT-decision-drift caveat in 05-VERIFICATION.md.
           fxs.push(run(10, "lda", Vars::A_B, Mode::Potential, 0, &[0.5, 0.5]));

           let path = Path::new(env!("CARGO_MANIFEST_DIR"))
               .join("tests/fixtures/expected.json");
           fs::create_dir_all(path.parent().unwrap())?;
           fs::write(&path, serde_json::to_string_pretty(&fxs).unwrap())?;
           eprintln!("wrote {} ({} fixtures)", path.display(), fxs.len());
           Ok(())
       }
       ```

       Add the example to Cargo.toml:
       ```toml
       [[example]]
       name = "gen_expected"
       path = "examples/gen_expected.rs"
       required-features = []
       ```

       **Row 8 SCANX execution-time fallback protocol (per CONTEXT D-14):**
       If `cargo run -p xcfun-capi --example gen_expected` fails on
       fixture 8 with a SCANX-specific runtime/Tier-1-self-test failure
       (e.g. `Functional::eval` returns `XcError::Runtime` from a SCAN
       substrate panic at the chosen density point):
       1. STOP execution.
       2. Record the failure mode in 05-04-SUMMARY.md under an
          "Escalation Gate" heading: include the exact stderr / panic
          message, the SCANX dependency mask, and the density input.
       3. Surface the event to the user; await acknowledgement before
          proceeding (interactive mode) or proceed automatically with
          the substitution recorded (yolo mode — but the SUMMARY entry
          is non-negotiable).
       4. After acknowledgement, edit `gen_expected.rs` fixture 8 to
          substitute `"tpssx"` in place of `"scanx"`; re-run.
       5. The 05-VERIFICATION matrix records the SCANX→TPSSX
          substitution alongside the LB94 row-10 caveat as
          documented CONTEXT-decision drift.

       **Row 10 LB94 protocol (no fallback at execution time — the
       substitute is pre-decided here):** call `Functional::set("lda",
       1.0)` directly. This is NOT an execution-time fallback (LB94
       was never going to evaluate); it is a planned substitution
       documented in this plan's `<objective>` block and propagated
       to 05-VERIFICATION.md.

    3. **Run the example to write `tests/fixtures/expected.json`**:
       ```bash
       cargo run -p xcfun-capi --example gen_expected
       ```
       Commit `crates/xcfun-capi/tests/fixtures/expected.json`. The
       file MUST contain 10 entries with `id` fields 1..10 in order.

    4. **Create `crates/xcfun-capi/tests/c_abi.c`** by reading
       `expected.json` and embedding each fixture as
       `static const double expected_<n>[]` and `static const double
       density_<n>[]` arrays (the executor pastes the literal array
       contents from expected.json into the C source). The C file is
       SELF-CONTAINED — it does NOT read expected.json at runtime.

       Skeleton:
       ```c
       /* Phase 5 D-14 — drop-in C-side golden test.
        * Compiled by crates/xcfun-capi/tests/c_abi.rs against
        * libxcfun_capi.a + crates/xcfun-capi/include/xcfun.h.
        *
        * 10 reference-driven fixtures spanning the public surface:
        *    1: LDA / XC_A_B / Partial / 0
        *    2: PBE / XC_A_B_AX_AY_AZ_BX_BY_BZ / Partial / 1
        *    3: BECKEX / XC_N_NX_NY_NZ / Partial / 2
        *    4: B3LYP alias / XC_A_B_AX_AY_AZ_BX_BY_BZ / Partial / 1
        *    5: PBE0 alias / XC_N_NX_NY_NZ / Partial / 1
        *    6: M06 alias (metaGGA = m06c + m06x) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0
        *    7: M06X (metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Contracted / 3
        *    8: SCANX (metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0
        *       — authorized SCANX→TPSSX fallback per CONTEXT D-14 if Tier-1 self-tests fail
        *    9: CAMB3LYP alias (range-separated) / XC_A_B_AX_AY_AZ_BX_BY_BZ / Partial / 0
        *   10: LB94 (Mode::Potential) — runtime substitute LDA per D-16
        *       (xcfun-master/src/functionals/lb94.cpp:15 is `#if 0`'d)
        */
       #include <math.h>
       #include <stdio.h>
       #include <stdlib.h>
       #include "xcfun.h"

       static int compare(const char* tag, const double* got, const double* want, int n) {
           int err = 0;
           for (int i = 0; i < n; i++) {
               double w = want[i];
               double rel = (fabs(w) > 1e-300)
                            ? fabs((got[i] - w) / w)
                            : fabs(got[i] - w);
               if (rel > 1e-12) {
                   fprintf(stderr,
                           "FAIL %s: out[%d] = %.16e expected %.16e rel %.2e\n",
                           tag, i, got[i], w, rel);
                   err++;
               }
           }
           return err;
           }

       static int run_fixture_1_lda(void) {
           xcfun_t* fun = xcfun_new();
           if (xcfun_set(fun, "lda", 1.0) != 0) { xcfun_delete(fun); return 1; }
           if (xcfun_eval_setup(fun, XC_A_B, XC_PARTIAL_DERIVATIVES, 0) != 0) {
               xcfun_delete(fun); return 2;
           }
           static const double density_1[2]  = { 0.5, 0.5 };
           /* Paste from expected.json fixture id=1 */
           static const double expected_1[1] = { /* TODO from expected.json */ };
           double out[1];
           xcfun_eval(fun, density_1, out);
           int e = compare("lda", out, expected_1, 1);
           xcfun_delete(fun);
           return e == 0 ? 0 : 100 + e;
       }
       /* ... 9 more fixture functions: run_fixture_2_pbe ... run_fixture_10_lb94_substitute ... */

       int main(void) {
           int rc;
           if ((rc = run_fixture_1_lda())                != 0) return rc;
           if ((rc = run_fixture_2_pbe())                != 0) return rc;
           if ((rc = run_fixture_3_beckex())             != 0) return rc;
           if ((rc = run_fixture_4_b3lyp())              != 0) return rc;
           if ((rc = run_fixture_5_pbe0())               != 0) return rc;
           if ((rc = run_fixture_6_m06())                != 0) return rc;
           if ((rc = run_fixture_7_m06x_contracted())    != 0) return rc;
           if ((rc = run_fixture_8_scanx())              != 0) return rc;  /* or tpssx fallback */
           if ((rc = run_fixture_9_camb3lyp())           != 0) return rc;
           if ((rc = run_fixture_10_lb94_substitute())   != 0) return rc;
           printf("ALL FIXTURES PASS\n");
           return 0;
       }
       ```

       Each `run_fixture_<n>_<name>` function carries inline `density_<n>`
       and `expected_<n>` arrays sourced from expected.json. Fixture 6
       passes "m06" to `xcfun_set` (the alias resolves via the engine);
       fixture 8 passes "scanx" (or "tpssx" if the SCANX→TPSSX fallback
       triggered during Task 4.1 step 3); fixture 10 passes "lda".

       For fixture 7 (Mode::Contracted), the C side calls `xcfun_eval`
       directly per CONTEXT D-14 — Mode::Contracted's internal layout per
       Phase 4 D-06-A is `inlen × (1 << order)` flat doubles for input,
       `(1 << order)` doubles for output. Call signature:
       `xcfun_eval(fun, density_7_56, out_8)`.
  </action>
  <verify>
    <automated>cargo run -p xcfun-capi --example gen_expected 2>&1 | grep -F "wrote" 2>&1 | tee /tmp/gen_05_04a.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>test -f crates/xcfun-capi/tests/fixtures/expected.json</automated>
    <automated>python3 -c 'import json; d=json.load(open("crates/xcfun-capi/tests/fixtures/expected.json")); assert len(d) == 10, len(d); ids=sorted(f["id"] for f in d); assert ids == list(range(1,11)), ids; print("expected.json OK", ids)'</automated>
    <automated>test -f crates/xcfun-capi/tests/c_abi.c && grep -F '#include "xcfun.h"' crates/xcfun-capi/tests/c_abi.c</automated>
    <automated>grep -cE '^static int run_fixture_' crates/xcfun-capi/tests/c_abi.c | grep -E "^10$"</automated>
    <automated>grep -cE 'static const double expected_[0-9]+\[' crates/xcfun-capi/tests/c_abi.c | grep -E "^10$"</automated>
    <automated>grep -cE 'static const double density_[0-9]+\[' crates/xcfun-capi/tests/c_abi.c | grep -E "^10$"</automated>
  </verify>
  <done>
    - `crates/xcfun-capi/tests/fixtures/expected.json` exists with 10 fixture entries (ids 1..10).
    - `crates/xcfun-capi/tests/c_abi.c` exists with 10 `run_fixture_*` functions, each carrying inline `density_<n>` + `expected_<n>` arrays from expected.json.
    - The C source includes `xcfun.h` (the generated header, found via cc include-dir).
    - cc dev-dep added to xcfun-capi/Cargo.toml.
    - If the SCANX→TPSSX fallback triggered, an Escalation Gate entry exists in 05-04-SUMMARY.md (created by Task 4.3).
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 4.2: c_abi.rs — cc compile + link + run integration test (10 fixtures)</name>
  <files>crates/xcfun-capi/tests/c_abi.rs</files>
  <read_first>
    - validation/build.rs:1-100 (cc::Build invocation pattern)
    - crates/xcfun-capi/tests/c_abi.c (Task 4.1 output — the C source to compile)
    - crates/xcfun-capi/Cargo.toml (cc dev-dep added in Task 4.1)
    - crates/xcfun-capi/include/xcfun.h (the include the C source consumes)
  </read_first>
  <behavior>
    - `cargo test -p xcfun-capi --test c_abi -- c_abi_drop_in_test` exits 0.
    - The test:
      1. Locates `target/<profile>/libxcfun_capi.a` (running `cargo build
         -p xcfun-capi` first if missing).
      2. Compiles `tests/c_abi.c` to an object via `cc::Build` with
         `-fno-fast-math -ffp-contract=off` and `-I crates/xcfun-capi/include`.
      3. Links the object against `libxcfun_capi.a` to produce a test
         binary in `target/c_abi_test/c_abi_runner` (or under
         `OUT_DIR` for cargo cleanliness).
      4. Runs the binary; captures stdout + stderr + exit code.
      5. Asserts exit code == 0 AND stdout contains `"ALL FIXTURES PASS"`.
    - On failure, the test stderr forwards the C binary's stderr (showing
      the FAIL line from `compare()`).
  </behavior>
  <action>
    Create `crates/xcfun-capi/tests/c_abi.rs` with:
    ```rust
    //! Phase 5 D-14 + CAPI-07 — compile tests/c_abi.c against
    //! libxcfun_capi.a + crates/xcfun-capi/include/xcfun.h, run the
    //! binary, assert exit code 0.
    //!
    //! 10 D-14 fixtures executed; row 8 SCANX→TPSSX fallback is
    //! authorized by CONTEXT and recorded as Escalation Gate in
    //! 05-04-SUMMARY.md if it triggers; row 10 LB94→LDA(Potential)
    //! substitution is documented per D-16 verification.

    use std::path::PathBuf;
    use std::process::Command;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()  // crates/
            .parent().unwrap()  // workspace root
            .to_path_buf()
    }

    fn out_dir() -> PathBuf {
        let target = std::env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root().join("target"));
        let dir = target.join("c_abi_test");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn staticlib_path() -> PathBuf {
        let target = std::env::var("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_root().join("target"));
        let candidates = [
            target.join("release/libxcfun_capi.a"),
            target.join("debug/libxcfun_capi.a"),
        ];
        for c in &candidates {
            if c.exists() { return c.clone(); }
        }
        let status = Command::new("cargo")
            .args(["build", "-p", "xcfun-capi", "--release"])
            .current_dir(workspace_root())
            .status()
            .expect("cargo build failed to spawn");
        assert!(status.success(), "cargo build -p xcfun-capi --release failed");
        candidates[0].clone()
    }

    fn cc_command() -> String {
        std::env::var("CC").unwrap_or_else(|_| "cc".to_string())
    }

    #[test]
    fn c_abi_drop_in_test() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let c_source = manifest.join("tests/c_abi.c");
        let include  = manifest.join("include");
        let staticlib = staticlib_path();

        let outdir = out_dir();
        let obj    = outdir.join("c_abi.o");
        let bin    = outdir.join("c_abi_runner");

        // Compile the C source. Per CLAUDE.md ACC-05/06 do NOT enable
        // -ffast-math or any reassociation flags.
        let compile = Command::new(cc_command())
            .args([
                "-c",
                "-O2",
                "-fno-fast-math",
                "-ffp-contract=off",
                "-Wall",
                "-Werror",
            ])
            .arg("-I").arg(&include)
            .arg("-o").arg(&obj)
            .arg(&c_source)
            .status()
            .expect("cc compile failed to spawn");
        assert!(compile.success(), "cc compile of {} failed", c_source.display());

        // Link against the staticlib + libm. On Linux we need pthread + dl
        // because cubecl-cpu pulls in those.
        let link = Command::new(cc_command())
            .arg("-o").arg(&bin)
            .arg(&obj)
            .arg(&staticlib)
            .args(["-lm", "-lpthread", "-ldl"])
            .status()
            .expect("cc link failed to spawn");
        assert!(link.success(), "cc link of {} failed", bin.display());

        // Run it.
        let output = Command::new(&bin)
            .output()
            .expect("c_abi binary failed to spawn");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("stdout:\n{stdout}\nstderr:\n{stderr}");
        assert!(
            output.status.success(),
            "c_abi binary exited with {:?} -- stderr: {stderr}",
            output.status
        );
        assert!(stdout.contains("ALL FIXTURES PASS"), "stdout: {stdout}");
    }
    ```

    Notes on link flags: cubecl-cpu (a transitive dep) pulls in MLIR JIT
    which uses `pthread` and `dl`. The exact link flags may need tuning
    on macOS / Windows; for Linux CI (the canonical target), `-lm -lpthread
    -ldl` plus the staticlib should suffice. If the executor encounters
    "undefined reference to ..." errors, document the missing flags in
    05-04-SUMMARY.md.

    **First-run risk:** if the C binary fails because of a numerical
    mismatch (1e-12 violation on any of the 10 fixtures), STOP and
    investigate. The expected outputs were generated by the SAME Rust
    code path that the C ABI links against — any mismatch means the C
    ABI shim layer is dropping precision. Do NOT widen the tolerance;
    fix the shim.
  </action>
  <verify>
    <automated>cargo build -p xcfun-capi --release 2>&1 | tee /tmp/build_05_04b.log; test ${PIPESTATUS[0]} -eq 0</automated>
    <automated>cargo test -p xcfun-capi --test c_abi 2>&1 | tee /tmp/test_05_04b.log; grep -F "test c_abi_drop_in_test ... ok" /tmp/test_05_04b.log</automated>
    <automated>grep -F "ALL FIXTURES PASS" /tmp/test_05_04b.log</automated>
    <automated>! grep -E "FAIL " /tmp/test_05_04b.log</automated>
  </verify>
  <done>
    - `crates/xcfun-capi/tests/c_abi.rs` exists with the cc compile + link + run logic.
    - `cargo test -p xcfun-capi --test c_abi` exits 0.
    - The C binary prints `ALL FIXTURES PASS` to stdout.
    - All 10 fixtures pass with relative error ≤ 1e-12.
    - No `-ffast-math` / fast-math flag passed to cc.
  </done>
</task>

<task type="auto">
  <name>Task 4.3: Phase 5 sign-off — verification matrix + REQUIREMENTS / ROADMAP / STATE updates</name>
  <files>.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md, .planning/REQUIREMENTS.md, .planning/ROADMAP.md, .planning/STATE.md</files>
  <read_first>
    - .planning/REQUIREMENTS.md (current Phase 5 requirement statuses — all `[ ] Pending`; need flip to `[x] Complete` with Phase 5 plan citation)
    - .planning/ROADMAP.md (Phase 5 entry; 5 success criteria)
    - .planning/STATE.md (current Phase 4 sign-off entry; need to add Phase 5 entry + advance "Current focus")
    - .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-00-SUMMARY.md, 05-01-SUMMARY.md, 05-02-SUMMARY.md, 05-03-SUMMARY.md (cite per-plan summaries in the verification matrix)
  </read_first>
  <action>
    1. **Create `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md`** with this matrix:
       ```markdown
       # Phase 5 — Rust Facade + C ABI — Verification

       **Sign-off date:** {current-date}
       **Status:** signed_off
       **Plans:** 5 (05-00 .. 05-04)

       ## Requirement Verdict Matrix

       | Req     | Status      | Plan(s)            | Evidence |
       |---------|-------------|---------------------|----------|
       | RS-01   | [x] Complete | 05-01              | `Functional::new()` newtype + Default impl; `cargo test -p xcfun-rs --lib functional::tests` |
       | RS-02   | [x] Complete | 05-01              | `Functional::set` 3-case dispatch verified by inline tests + free_fns test for case-insensitive |
       | RS-03   | [x] Complete | 05-01              | `Functional::get` 2-case dispatch tested |
       | RS-04   | [x] Complete | 05-01              | `is_gga` / `is_metagga` tested for SLATERX (false), PBEX (true GGA), TPSSX (true metaGGA) |
       | RS-05   | [x] Complete | 05-00, 05-01       | `eval_setup` mutates inner state; combined `InvalidVarsAndMode` variant added; tested for all 4 error branches |
       | RS-06   | [x] Complete | 05-01              | `user_eval_setup` composes `which_vars` + `which_mode` + `eval_setup` |
       | RS-07   | [x] Complete | 05-01              | `eval` delegates to xcfun-eval; zero-alloc fixture passes (or documented fallback) |
       | RS-08   | -- (Phase 6) | --                 | Out of scope for Phase 5 (eval_vec GPU dispatch) |
       | RS-09   | [x] Complete | 05-00, 05-01       | 11 free fns; ≥23 unit tests in tests/free_fns.rs; `enumerate_parameters` includes the 4 parameters; LB94 is NOT enumerated as a parameter (it is a functional, see 05-00 Task 0.4) |
       | RS-10   | [x] Complete | 05-00, 05-01       | `assert_impl_all!(Functional: Send, Sync)` compile-time gate; `enumerate_aliases` includes the 46 aliases; LB94 is NOT enumerated as an alias (it is a standalone functional) |
       | CAPI-01 | [x] Complete | 05-02              | 23 `#[no_mangle]` exports in `crates/xcfun-capi/src/lib.rs`; `nm` confirms 23 T xcfun_* symbols |
       | CAPI-02 | [x] Complete | 05-03              | `headers_match` test exits 0; cbindgen-generated `xcfun.h` diff-matches upstream modulo whitespace + comments |
       | CAPI-03 | [x] Complete | 05-02              | `xcfun_new` returns `Box::into_raw`; `xcfun_delete(NULL)` is silent no-op (api_smoke test `xcfun_new_and_delete_null_safe`) |
       | CAPI-04 | [x] Complete | 05-02              | Every C entry wrapped in `c_entry!` (catch_unwind + abort); 23 `c_entry!` invocations |
       | CAPI-05 | [x] Complete | 05-00, 05-02       | `XcError::as_c_code` returns 0/1/2/4/6/-1; 11 unit tests in error::tests |
       | CAPI-06 | [x] Complete | 05-02              | `cargo build -p xcfun-capi --release` produces `libxcfun_capi.{so,a}`; cdylib + staticlib + rlib triple |
       | CAPI-07 | [x] Complete | 05-04              | `tests/c_abi.c` driven by `tests/c_abi.rs`; ALL FIXTURES PASS at 1e-12 across 10 fixtures (LDA, GGA, metaGGA M06+M06X+SCANX+TPSSX-if-fallback, alias additive + range-separated, Mode::Contracted, Mode::Potential) |

       ## D-Decisions Coverage Audit

       | Decision | Plan(s)        | Status |
       |----------|----------------|--------|
       | D-01 (rename xcfun-ffi→xcfun-capi) | 05-00 | [x] |
       | D-02 (xcfun-rs newtype wrapper) | 05-01 | [x] |
       | D-03 (lookup tables stay in xcfun-core; free fns in xcfun-rs) | 05-01 | [x] |
       | D-04 (delete xcfun-functionals) | 05-00 | [x] |
       | D-05 (c_entry! macro: catch_unwind + abort) | 05-02 | [x] |
       | D-06 (void-returning C fns abort on Err via die_with) | 05-02 | [x] |
       | D-07 (NULL pointer guards in c_entry!) | 05-02 | [x] |
       | D-08 (C ABI trusts caller buffers; Rust validates internally) | 05-02 | [x] |
       | D-08-A (XcError::as_c_code mapping; combined InvalidVarsAndMode variant) | 05-00 | [x] |
       | D-09 (cbindgen via xtask + checked-in xcfun.h + sha256 drift gate) | 05-03 | [x] |
       | D-10 (headers_match.rs lives at xcfun-capi/tests/) | 05-03 | [x] |
       | D-11 (cbindgen.toml documentation = false) | 05-03 | [x] |
       | D-12 (cbindgen function.prefix = XCFun_API + prelude inline) | 05-03 | [x] |
       | D-13 (zero-alloc verification via counting global allocator) | 05-01 | [x] (or documented fallback) |
       | D-14 (10-fixture tests/c_abi.c) | 05-04 | [x] **10 fixtures**; row 8 SCANX with authorized fallback to TPSSX (Escalation Gate if triggered); row 10 LB94→LDA(Potential) runtime substitute documented per D-16 verification |
       | D-15 ([lib] crate-type triple) | 05-02 | [x] |
       | D-15-A (xcfun-rs crate-type rlib) | 05-00 | [x] |
       | D-16 (LB94 inclusion verification) | 05-00 (Task 0.4) | [x] LB94 descriptor added to xcfun-core (FunctionalId::XC_LB94 = 78; FUNCTIONAL_DESCRIPTORS stub entry; eval path returns XcError::Runtime per upstream `#if 0`'d body); D-14 row 10 runtime substitution kept separate from descriptor presence |
       | D-17 (Functional Send + Sync) | 05-01 | [x] |

       ## Phase 5 CONTEXT-decision-drift caveats

       ### Caveat 1 — D-14 row 10 LB94 runtime substitution

       **Original D-14 row 10:** "LB94 (Phase-3 D-19 deferred surface) /
       LDA Vars / Potential / 0".

       **Verification finding (Plan 05-04 Task 4.1):**
       `xcfun-master/src/functionals/lb94.cpp:15` opens with `#if 0 //
       Does not work and should maybe not be here in the first place`.
       The entire LB94 body (lines 15-97) is dead code in the upstream
       C++ tree. There is no working LB94 functional body in any
       reference implementation that the C-ABI golden can call.

       **Resolution:** Per D-16 (Plan 05-00 Task 0.4), the LB94
       *descriptor* is added to xcfun-core (FunctionalId::XC_LB94,
       FUNCTIONAL_DESCRIPTORS stub entry); a downstream consumer can
       call `xcfun_set("lb94", 1.0)` and the call succeeds. However,
       `xcfun_eval_setup` followed by `xcfun_eval` on LB94 returns
       `XcError::Runtime` (mapped to C return code -1) because the
       upstream evaluable body does not exist. The C-ABI golden test
       (D-14 row 10) therefore evaluates **LDA / Mode::Potential / 0**
       at the same density point as the LB94 row would have used,
       preserving the explicit Mode::Potential coverage goal of D-14
       without inventing a body that does not exist upstream.

       **Sign-off impact:** none — D-16 LB94-descriptor-presence
       requirement is satisfied (descriptor exists; reachable via
       `xcfun_set` and `enumerate_*` would surface it if they were the
       intended enumerators, which they are not — LB94 is a functional,
       not a parameter or alias). D-14 Mode::Potential coverage is
       satisfied by the LDA-Potential evaluation. CAPI-07 success
       criterion ("matching output to the Rust reference driver on 10
       fixtures") is satisfied with all 10 rows passing at 1e-12.

       ### Caveat 2 — Row 8 SCANX→TPSSX fallback (if triggered)

       **CONTEXT D-14 specifics:** "SCAN family is excluded_by_upstream_spec
       for full sweeps; specific point + order chosen to be in-domain —
       verify it passes Tier-1 self-tests; otherwise substitute TPSSX".

       **Resolution at execution time:**
       - If SCANX passed Tier-1 self-tests at the chosen density point:
         row 8 evaluates SCANX as specified. No caveat.
       - If SCANX failed: an Escalation Gate event was raised in
         05-04-SUMMARY.md before substitution. Row 8 then evaluates
         TPSSX (also metaGGA, no SCAN substrate dependency). The
         escalation event documents the SCANX failure mode and the user
         acknowledged the substitution.

       The fallback is authorized **only for row 8** by CONTEXT D-14
       and triggers only on Tier-1 self-test failure. No other row may
       use this fallback.

       ## Plan Summaries

       - 05-00-SUMMARY.md — workspace topology + XcError::as_c_code + InvalidVarsAndMode variant + LB94 descriptor add-back (D-16)
       - 05-01-SUMMARY.md — xcfun-rs facade (Functional + 11 free fns + Send+Sync + zero-alloc)
       - 05-02-SUMMARY.md — xcfun-capi 23 FFI exports + c_entry! macro + triple crate-type
       - 05-03-SUMMARY.md — cbindgen flow + headers_match drift gate
       - 05-04-SUMMARY.md — c_abi.c golden (10 fixtures) + Phase 5 sign-off (THIS plan)
       ```

    2. **Update `.planning/REQUIREMENTS.md`** — mark the 16 Phase 5 items
       as Complete with Phase 5 plan citations. Edit the lines for
       RS-01..07, RS-09, RS-10, CAPI-01..07. Example pattern:
       ```markdown
       - [x] **RS-01**: `Functional::new` returns ... (Plan 05-01; verified by xcfun-rs/tests + lib::functional::tests)
       ```
       Also update the **Traceability** table at the bottom (lines ~263-280)
       to flip "Pending" → "Complete" for those 16 IDs.

    3. **Update `.planning/ROADMAP.md`** Phase 5 entry to mark it `[x]
       **Phase 5: Rust Facade (xcfun-rs) + C ABI (xcfun-capi)** -
       Complete (YYYY-MM-DD) — thin facade re-exports + full C ABI ...`,
       and tick the 5 success criteria. Update the "Plans" line under
       Phase 5 to list the 5 PLAN.md files (05-00..05-04). Update the
       progress table at the bottom: `5. Rust Facade + C ABI | 5/5 |
       Complete | YYYY-MM-DD`.

    4. **Update `.planning/STATE.md`** — append a new entry summarizing
       Phase 5 sign-off:
       - Add a "Phase 5 sign-off summary (YYYY-MM-DD)" block analogous
         to the Phase 4 block (lines 39-51 of current STATE.md).
       - Update "Current focus" → "Phase 5 (Rust Facade + C ABI) —
         **COMPLETE (YYYY-MM-DD)**".
       - Update "Phase" line to point at Phase 5 = signed_off, next is
         Phase 6.
       - Update progress: phases completed 4/8 → 5/8; total plans
         32/32 → 37/37 (32 prior + 5 new Phase 5 plans).
       - Add the 16 D-decisions (D-01..D-17 + D-08-A) to "Decisions
         added in Phase 5" section.
       - Note the two D-14 caveats (row 10 LB94→LDA(Potential) substitute;
         row 8 SCANX→TPSSX fallback if triggered) and the D-16
         compliance via FunctionalId::XC_LB94 descriptor.

    5. Verify NO regression by running ALL Phase 1-5 tests at the end.
  </action>
  <verify>
    <automated>test -f .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md && grep -F "[x] Complete" .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md</automated>
    <automated>grep -cE "^\| (RS-0[12345679]|RS-10|CAPI-0[1234567]) +\| \[x\] Complete" .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-VERIFICATION.md | grep -E "^16$"</automated>
    <automated>grep -cE "^\- \[x\] \*\*(RS-0[12345679]|RS-10|CAPI-0[1234567])\*\*" .planning/REQUIREMENTS.md | grep -E "^16$"</automated>
    <automated>grep -F "[x] **Phase 5: Rust Facade" .planning/ROADMAP.md</automated>
    <automated>grep -F "Phase 5 sign-off" .planning/STATE.md</automated>
    <automated>cargo test --workspace 2>&1 | tee /tmp/all_tests_05_04c.log; grep -E "test result: FAILED" /tmp/all_tests_05_04c.log && exit 1 || true</automated>
  </verify>
  <done>
    - `05-VERIFICATION.md` exists with the 16-row matrix (all `[x]
      Complete`) plus the two CONTEXT-decision-drift caveat blocks
      (LB94 row 10 + SCANX row 8 fallback).
    - `REQUIREMENTS.md` flips 16 items to `[x] Complete` with
      Phase 5 plan references; Traceability table updated.
    - `ROADMAP.md` marks Phase 5 complete with the 5 PLAN files listed.
    - `STATE.md` advances current focus to Phase 6, records sign-off.
    - `cargo test --workspace` exits 0 (no regression).
  </done>
</task>

</tasks>

<verification>
Run after all tasks complete:

```bash
# C ABI golden test
cargo test -p xcfun-capi --test c_abi   # ALL FIXTURES PASS (10 fixtures)

# All Phase 5 tests
cargo test -p xcfun-rs
cargo test -p xcfun-capi
cargo test -p xcfun-core --lib error::tests
cargo test -p xcfun-eval --features testing --lib functional::tests

# Cross-phase regression
cargo test --workspace

# Sign-off artifacts
grep -F "Phase 5 sign-off" .planning/STATE.md
grep -F "[x] **Phase 5" .planning/ROADMAP.md
grep -cE "^- \[x\] \*\*(RS|CAPI)" .planning/REQUIREMENTS.md   # >= 16

# 10 fixtures committed
test -f crates/xcfun-capi/tests/fixtures/expected.json
python3 -c 'import json; assert len(json.load(open("crates/xcfun-capi/tests/fixtures/expected.json"))) == 10'

# Generated header committed
test -f crates/xcfun-capi/include/xcfun.h
test -f crates/xcfun-capi/include/xcfun.h.sha256

# LB94 descriptor present (D-16)
cargo run -p xcfun-rs --example list_functionals 2>/dev/null | grep -i "lb94" || \
  grep -F "XC_LB94" crates/xcfun-core/src/functional_id.rs
```
</verification>

<success_criteria>
- `cargo test -p xcfun-capi --test c_abi` exits 0; ALL 10 fixtures pass at 1e-12 relative error.
- `cargo test --workspace` exits 0 — no Phase 1-5 regression.
- 16 Phase 5 requirements (RS-01..07, RS-09, RS-10, CAPI-01..07) marked `[x] Complete` in REQUIREMENTS.md.
- Phase 5 marked complete in ROADMAP.md.
- STATE.md advances; Phase 5 sign-off recorded with the two D-14 caveat blocks (row 10 LB94→LDA(Potential); row 8 SCANX→TPSSX if triggered) and D-16 compliance via Plan 05-00 Task 0.4.
- 05-VERIFICATION.md verdict matrix is the canonical record of Phase 5 completion.
</success_criteria>

<output>
After completion, create `.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-04-SUMMARY.md` documenting:
- The 10 fixtures + their `expected.json` shasum.
- The cc invocation flags used (Linux: `-fno-fast-math -ffp-contract=off -lm -lpthread -ldl`).
- Confirmation of the D-14 row 10 LB94→LDA(Mode::Potential) runtime substitution rationale (xcfun-master/src/functionals/lb94.cpp:15 `#if 0`'d).
- If the SCANX→TPSSX fallback triggered: a dedicated "Escalation Gate" subsection with the SCANX failure mode + user acknowledgement record.
- Final phase-level metrics: total Phase 5 plans (5), total tasks executed (~14, including the new 05-00 Task 0.4 LB94 descriptor add-back), total Phase 5 unit + integration tests added (≥80).
- Any cross-phase regression caught and resolved.
- Pointer to `05-VERIFICATION.md` as the canonical sign-off ledger.
</output>
</content>
</invoke>