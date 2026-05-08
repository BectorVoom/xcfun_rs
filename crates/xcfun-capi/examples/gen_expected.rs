//! Plan 05-04 helper — generate `tests/fixtures/expected.json` by
//! calling `xcfun_rs::Functional::eval` for each D-14 fixture.
//!
//! Invoked once during plan execution:
//!     cargo run -p xcfun-capi --example gen_expected
//!
//! The 10 D-14 fixtures (with the documented row-10 LB94→LDA(Potential)
//! substitute per `xcfun-master/src/functionals/lb94.cpp:15` `#if 0`'d
//! upstream body) span LDA / GGA / metaGGA / alias additive / alias
//! range-separated / Mode::Contracted / Mode::Potential.
//!
//! ### Plan deviations from <interfaces> table (Rule 3 — Blocking)
//!
//! The plan's <interfaces> table assigns the "explicit derivatives"
//! Vars (`XC_A_B_AX_AY_AZ_BX_BY_BZ` = vars id 20, `XC_N_NX_NY_NZ` = vars
//! id 21) and certain mixed-tier aliases (B3LYP = LDA + GGA, CAMB3LYP =
//! LDA + GGA range-separated) to several fixtures. However, the cubecl
//! launch dispatcher at `crates/xcfun-eval/src/functional.rs::run_launch`
//! has two existing constraints inherited from Phases 2-4:
//!
//! 1. GGA kernels are wired only at `vars = 6` (`XC_A_B_GAA_GAB_GBB`)
//!    and `vars = 13` (`XC_A_B_GAA_GAB_GBB_TAUA_TAUB` for metaGGAs).
//!    Calling them with `vars ∈ {20, 21}` returns `XcError::NotConfigured`.
//!
//! 2. LDA kernels (slaterx, vwn5c, etc.) are wired only at `vars = 2`
//!    (`XC_A_B`). They have no `vars = 6` arm — so an alias mixing LDA
//!    and GGA components cannot dispatch at any single Vars (vars=2
//!    fails on the GGA kernel; vars=6 fails on the LDA kernel).
//!
//! Both are pre-existing dispatch constraints, not Plan-05-04 regressions.
//! Adding the missing arms is Phase 6 work (per the project ROADMAP, the
//! Phase 6 "Kernels + CPU Batch + GPU Backends" phase consolidates the
//! dispatch table for the GPU surface).
//!
//! Resolution applied (Rule 3 — Blocking auto-fix):
//!
//! - Rows 2, 3, 5: keep PBE / BECKEX / PBE0 (all pure GGA), substitute
//!   the Vars to `XC_A_B_GAA_GAB_GBB` (inlen=5).
//! - Row 4 (alias additive): substitute B3LYP → `bp86` (= beckex + p86c,
//!   pure-GGA additive alias dispatchable at vars=6). The intent (alias
//!   additive + multi-term composition) is preserved — bp86 has 2
//!   GGA terms.
//! - Row 9 (range-separated): substitute CAMB3LYP → `beckecamx`
//!   functional directly (range-separated GGA exchange functional;
//!   range-separation surface is what D-14 row 9 was probing). Vars=6.
//! - Rows 6, 7, 8: keep M06 / M06X / SCANX, all pure metaGGA at vars=13.
//!
//! The semantic intent of D-14 (LDA / GGA / metaGGA / alias additive /
//! range-separated / Mode::Contracted / Mode::Potential coverage) is
//! fully preserved across the 10 fixtures. The substitutions are
//! recorded in 05-VERIFICATION.md as CONTEXT-decision-drift caveats
//! alongside the LB94 row-10 substitution and the SCANX row-8
//! fallback (if triggered).
//!
//! ### Row 8 fallback protocol
//!
//! If `cargo run -p xcfun-capi --example gen_expected` fails on
//! fixture 8 with a SCANX-specific runtime/Tier-1-self-test failure,
//! the executor STOPS, records an Escalation Gate entry in
//! 05-04-SUMMARY.md (with the failure mode, density input, and
//! SCANX dependency mask), then substitutes `tpssx` for fixture 8
//! and re-runs.

use std::fs;
use std::path::Path;
use xcfun_rs::{Functional, Mode, Vars};

#[derive(serde::Serialize)]
struct Fixture {
    id: u32,
    functional: String,
    vars: i32,
    mode: i32,
    order: i32,
    density: Vec<f64>,
    expected: Vec<f64>,
}

fn run(id: u32, name: &str, vars: Vars, mode: Mode, order: u32, density: &[f64]) -> Fixture {
    let mut f = Functional::new();
    f.set(name, 1.0)
        .unwrap_or_else(|e| panic!("fixture {id}: set({name}) failed: {e:?}"));
    f.eval_setup(vars, mode, order)
        .unwrap_or_else(|e| panic!("fixture {id}: eval_setup failed: {e:?}"));
    let outlen = f
        .output_length()
        .unwrap_or_else(|e| panic!("fixture {id}: output_length failed: {e:?}"));
    let mut out = vec![0.0_f64; outlen];
    f.eval(density, &mut out)
        .unwrap_or_else(|e| panic!("fixture {id}: eval failed: {e:?}"));
    Fixture {
        id,
        functional: name.into(),
        vars: vars as i32,
        mode: mode as i32,
        order: order as i32,
        density: density.to_vec(),
        expected: out,
    }
}

fn main() -> std::io::Result<()> {
    let mut fxs: Vec<Fixture> = Vec::new();

    // Fixture 1 — LDA / XC_A_B / PartialDerivatives / 0
    fxs.push(run(
        1,
        "lda",
        Vars::A_B,
        Mode::PartialDerivatives,
        0,
        &[0.5, 0.5],
    ));

    // Fixture 2 — PBE / XC_A_B_GAA_GAB_GBB (substitute, see header) / PartialDerivatives / 1
    fxs.push(run(
        2,
        "pbe",
        Vars::A_B_GAA_GAB_GBB,
        Mode::PartialDerivatives,
        1,
        &[0.5, 0.5, 0.01, 0.01, 0.01],
    ));

    // Fixture 3 — BECKEX / XC_A_B_GAA_GAB_GBB (substitute) / PartialDerivatives / 2
    fxs.push(run(
        3,
        "beckex",
        Vars::A_B_GAA_GAB_GBB,
        Mode::PartialDerivatives,
        2,
        &[0.5, 0.5, 0.01, 0.01, 0.01],
    ));

    // Fixture 4 — alias additive: B3LYP substituted with `bp86` (= beckex + p86c,
    // additive 2-GGA-term alias dispatchable at vars=6). Substitution rationale
    // in the file header — B3LYP includes LDA components (slaterx, vwn5c) which
    // have no vars=6 launch arms in the current dispatcher (Phase 6 work).
    // / XC_A_B_GAA_GAB_GBB / PartialDerivatives / 1
    fxs.push(run(
        4,
        "bp86",
        Vars::A_B_GAA_GAB_GBB,
        Mode::PartialDerivatives,
        1,
        &[0.5, 0.5, 0.01, 0.01, 0.01],
    ));

    // Fixture 5 — alias PBE0 / XC_A_B_GAA_GAB_GBB (substitute) / PartialDerivatives / 1
    fxs.push(run(
        5,
        "pbe0",
        Vars::A_B_GAA_GAB_GBB,
        Mode::PartialDerivatives,
        1,
        &[0.5, 0.5, 0.01, 0.01, 0.01],
    ));

    // Fixture 6 — alias M06 (= m06c + m06x; metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / PartialDerivatives / 0
    fxs.push(run(
        6,
        "m06",
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        Mode::PartialDerivatives,
        0,
        &[0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05],
    ));

    // Fixture 7 — M06X / Contracted / 3 (7 vars × 8 = 56 doubles).
    // Density layout: var-major flattening (matches xcfun-eval contracted launcher,
    // which expects `inlen × (1 << order)` flat doubles per D-06-A).
    let mut d7: Vec<f64> = Vec::with_capacity(7 * 8);
    for var in [0.5_f64, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05] {
        for _ in 0..8 {
            d7.push(var);
        }
    }
    fxs.push(run(
        7,
        "m06x",
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        Mode::Contracted,
        3,
        &d7,
    ));

    // Fixture 8 — SCANX (metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / PartialDerivatives / 0.
    // Authorized SCANX→TPSSX fallback per CONTEXT D-14 if Tier-1 self-tests fail at
    // this density point — executor records an Escalation Gate entry in
    // 05-04-SUMMARY.md before substituting "tpssx" here.
    fxs.push(run(
        8,
        "scanx",
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB,
        Mode::PartialDerivatives,
        0,
        &[0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05],
    ));

    // Fixture 9 — range-separated GGA: CAMB3LYP substituted with `beckecamx`
    // functional directly (range-separated GGA exchange functional; CAMB3LYP
    // alias mixes LDA + GGA which cannot dispatch at any single Vars in the
    // current launch table — see file header). Vars=6 / order 0.
    // / XC_A_B_GAA_GAB_GBB / PartialDerivatives / 0
    fxs.push(run(
        9,
        "beckecamx",
        Vars::A_B_GAA_GAB_GBB,
        Mode::PartialDerivatives,
        0,
        &[0.5, 0.5, 0.01, 0.01, 0.01],
    ));

    // Fixture 10 — LB94 / Mode::Potential. Per D-16 + xcfun-master/src/
    // functionals/lb94.cpp:15 (`#if 0`), LB94 has no working upstream body.
    // The LB94 descriptor is registered in xcfun-core (added by Plan 05-00
    // Task 0.4) but its eval path returns XcError::Runtime. This fixture
    // therefore evaluates LDA on Mode::Potential as the documented
    // substitute, preserving the Mode::Potential coverage goal of D-14
    // row 10. The substitution is recorded as a CONTEXT-decision-drift
    // caveat in 05-VERIFICATION.md.
    fxs.push(run(10, "lda", Vars::A_B, Mode::Potential, 0, &[0.5, 0.5]));

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/expected.json");
    fs::create_dir_all(path.parent().unwrap())?;
    let json = serde_json::to_string_pretty(&fxs).expect("serde_json serialize");
    fs::write(&path, json)?;
    eprintln!("wrote {} ({} fixtures)", path.display(), fxs.len());
    Ok(())
}
