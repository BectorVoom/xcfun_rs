//! Rust binary that emits a parity ground-truth JSON fixture for the
//! Python pytest cross-language gate (PY-03 + the Phase 5 D-08-A bit-pattern).
//!
//! Run: `cargo run -p xcfun-py --release --example gen_py_fixtures`
//! Output: `crates/xcfun-py/tests/fixtures/eval_parity.json`
//!
//! Source: 07-RESEARCH Example F + xcfun-capi/examples/gen_expected.rs.
//!
//! Note: `anyhow` is allowed here (this is an example binary, NOT a library
//! crate). The xtask `check-no-anyhow` allowlist excludes example bins.

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use xcfun_rs::{Functional, Mode, Vars};

#[derive(Serialize)]
struct Fixture {
    functional: String,
    vars: u32,
    mode: u32,
    order: u32,
    density: Vec<f64>,
    expected: Vec<f64>,
}

fn run_one(name: &str, vars: Vars, mode: Mode, order: u32, density: Vec<f64>) -> Fixture {
    let mut f = Functional::new();
    f.set(name, 1.0)
        .unwrap_or_else(|e| panic!("set({name}, 1.0) failed: {e:?}"));
    f.eval_setup(vars, mode, order)
        .unwrap_or_else(|e| panic!("eval_setup({name}, {vars:?}, {mode:?}, {order}) failed: {e:?}"));
    let inlen = f.input_buffer_length();
    assert_eq!(
        density.len(),
        inlen,
        "fixture {name}: density.len()={} but input_buffer_length()={inlen}",
        density.len()
    );
    let outlen = f
        .output_length()
        .unwrap_or_else(|e| panic!("output_length({name}) failed: {e:?}"));
    let mut expected = vec![0.0_f64; outlen];
    f.eval(&density, &mut expected)
        .unwrap_or_else(|e| panic!("eval({name}) failed: {e:?}"));
    Fixture {
        functional: name.to_string(),
        vars: vars as u32,
        mode: mode as u32,
        order,
        density,
        expected,
    }
}

fn main() {
    let mut fixtures: Vec<Fixture> = Vec::new();

    // LDA tier (Phase 2)
    fixtures.push(run_one(
        "slaterx", Vars::A_B, Mode::PartialDerivatives, 0,
        vec![0.3, 0.2],
    ));
    fixtures.push(run_one(
        "slaterx", Vars::A_B, Mode::PartialDerivatives, 1,
        vec![0.3, 0.2],
    ));
    fixtures.push(run_one(
        "vwn5c", Vars::A_B, Mode::PartialDerivatives, 0,
        vec![0.4, 0.3],
    ));
    fixtures.push(run_one(
        "tfk", Vars::A_B, Mode::PartialDerivatives, 0,
        vec![0.5, 0.5],
    ));

    // GGA tier (Phase 3) — A_B_GAA_GAB_GBB = 5 inputs (rho_a, rho_b, gaa, gab, gbb)
    fixtures.push(run_one(
        "pbex", Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1,
        vec![0.30, 0.20, 0.10, 0.05, 0.10],
    ));
    fixtures.push(run_one(
        "pbec", Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1,
        vec![0.30, 0.20, 0.10, 0.05, 0.10],
    ));
    fixtures.push(run_one(
        "lypc", Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 2,
        vec![0.30, 0.20, 0.10, 0.05, 0.10],
    ));
    fixtures.push(run_one(
        "blyp", Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 2,
        vec![0.30, 0.20, 0.10, 0.05, 0.10],
    ));

    // metaGGA tier exemplar (Phase 4) — A_B_GAA_GAB_GBB_TAUA_TAUB = 7 inputs
    fixtures.push(run_one(
        "tpssx", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, Mode::PartialDerivatives, 1,
        vec![0.30, 0.20, 0.10, 0.05, 0.10, 0.20, 0.15],
    ));
    fixtures.push(run_one(
        "m06x", Vars::A_B_GAA_GAB_GBB_TAUA_TAUB, Mode::PartialDerivatives, 1,
        vec![0.30, 0.20, 0.10, 0.05, 0.10, 0.20, 0.15],
    ));

    let json = serde_json::to_string_pretty(&fixtures)
        .expect("serialize fixtures");
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("eval_parity.json");
    fs::create_dir_all(out_path.parent().unwrap())
        .expect("create fixtures dir");
    fs::write(&out_path, json).unwrap_or_else(|e| {
        panic!("write {}: {e}", out_path.display())
    });
    eprintln!(
        "wrote {} fixtures to {}",
        fixtures.len(),
        out_path.display()
    );
}
