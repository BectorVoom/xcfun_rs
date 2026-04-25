//! Phase 3 plan 03-05 (B5 path (a)) — generate
//! `crates/xcfun-eval/tests/data/potential_reference_100.json` by driving
//! the cc-compiled C++ xcfun reference (vendored under `xcfun-master/`)
//! over a deterministic Gaussian-atom density grid.
//!
//! # Why path (a)
//!
//! The plan locks B5 to "path (a) chosen": a NEW xtask binary that drives
//! the C++ reference offline. The `validation/` crate already cc-compiles
//! the entire xcfun-master tree and exposes `CppXcfun` (a safe RAII
//! wrapper over the C ABI). To avoid duplicating that build, this xtask
//! depends on `validation` (gated behind the `gen-potential-fixtures`
//! feature so the standard xtask build remains lean). Functionally this
//! is identical to path (a) — the JSON IS produced by an offline run of
//! the cc-compiled C++ reference; only the wrapper lives in Rust.
//!
//! # Input grid (deterministic, seed-gated)
//!
//! - **Density profile:** Gaussian-atom radial sampling.
//!   `n(r) = n0 · exp(-α·r²)`, with:
//!     n0 ∈ {0.1, 1.0, 10.0}   (3 scale magnitudes)
//!     α  ∈ {0.5, 1.0}         (2 width parameters)
//!     r  ∈ {0.1, 0.5, 1.0, 2.0, 3.0}   (5 radial points)
//!   = 30 candidates → down-sample to 20 via xoshiro seed `0xf00dbabe`.
//! - **Functional selection:** 5 representative GGAs across the 36-functional
//!   spectrum: PBEX (5), BECKEX (6), LYPC (16), PW91X (26), B97X (60).
//! - **Total records:** 5 functionals × 20 points = 100.
//!
//! # JSON record schema
//!
//! ```json
//! {
//!   "functional_name": "XC_PBEX",
//!   "vars":  "A_B_2ND_TAYLOR",
//!   "input": [20 f64],          // α/β 2nd-Taylor expansion
//!   "expected_output": [3 f64]  // [energy, pot_α, pot_β]
//! }
//! ```
//!
//! # Output path
//!
//! `crates/xcfun-eval/tests/data/potential_reference_100.json` (committed
//! to git; consumed by `crates/xcfun-eval/tests/potential_parity.rs`).

use anyhow::{Context, Result};
use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::SeedableRng;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "gen-potential-fixtures")]
use validation::ffi::CppXcfun;

const SEED: u64 = 0xf00dbabe_u64;
const TOTAL_FIXTURE_COUNT: usize = 100;
const RECORDS_PER_FUNCTIONAL: usize = 20;

/// `xcfun_vars::XC_A_B_2ND_TAYLOR` discriminant — matches both
/// `api/xcfun.h:118` (counting XC_VARS_UNSET=-1 ⇒ XC_A=0 ⇒ ... ⇒
/// XC_A_B_2ND_TAYLOR=28) and `xcfun-core::Vars::A_B_2ND_TAYLOR` per
/// D-10-A. See enums.rs:74.
const VARS_A_B_2ND_TAYLOR: u32 = 28;
/// `xcfun_mode::XC_POTENTIAL` discriminant per `api/xcfun.h:38`.
const MODE_POTENTIAL: u32 = 2;

#[derive(Serialize)]
struct PotentialRecord {
    functional_name: &'static str,
    vars: &'static str,
    /// 20-element flat input slot vector matching the
    /// `Vars::A_B_2ND_TAYLOR` layout (α: 0..9, β: 10..19).
    input: Vec<f64>,
    /// `[energy, pot_α, pot_β]` from the cc-compiled C++ reference.
    expected_output: Vec<f64>,
}

/// 5 representative GGAs covering 4 of the 8 GGA families.
const FIXTURE_FUNCTIONALS: &[&str] = &[
    "XC_PBEX", "XC_BECKEX", "XC_LYPC", "XC_PW91X", "XC_B97X",
];

/// `(n0, α, r)` Gaussian-atom grid candidates: 30 points before
/// down-sampling.
fn gaussian_grid_candidates() -> Vec<(f64, f64, f64)> {
    let n0_set = [0.1_f64, 1.0, 10.0];
    let alpha_set = [0.5_f64, 1.0];
    let r_set = [0.1_f64, 0.5, 1.0, 2.0, 3.0];
    let mut out = Vec::with_capacity(n0_set.len() * alpha_set.len() * r_set.len());
    for &n0 in &n0_set {
        for &alpha in &alpha_set {
            for &r in &r_set {
                out.push((n0, alpha, r));
            }
        }
    }
    out
}

/// Deterministic 20-point selection from the 30 candidates using the
/// xoshiro seed (skip every third record then take 20).
fn select_20_points(candidates: &[(f64, f64, f64)]) -> Vec<(f64, f64, f64)> {
    let _rng = Xoshiro256PlusPlus::seed_from_u64(SEED);
    // Deterministic reproducible selection: stride = 3 keeps every
    // {0, 1, 2} pattern → take 20 from 30. This is mechanically
    // deterministic without needing the RNG to be advanced; the seed
    // exists for documentation + future-proofing if the candidate grid
    // grows beyond 30.
    candidates
        .iter()
        .enumerate()
        .filter_map(|(i, &p)| if i % 3 != 2 { Some(p) } else { None })
        .take(RECORDS_PER_FUNCTIONAL)
        .collect()
}

/// Build the 20-slot `Vars::A_B_2ND_TAYLOR` input from a Gaussian-atom
/// density at radius `r`:
///   n(r) = n0 · exp(-α·r²)
///   α-channel = β-channel = n/2 (closed-shell)
///
/// We use a 1D radial probe (only x-direction is non-zero):
///   density:    input[0]  = a = n/2
///   gradient:   input[1]  = a_x = (∂a/∂x)
///                input[2..3] = 0
///   Hessian:    input[4]  = a_xx = (∂²a/∂x²)
///                input[5..9] = analytical zero (1D probe)
///   β mirrors α at offset +10.
///
/// For Gaussian density `a(r) = (n0/2) · exp(-α·r²)`:
///   a_x  = -2·α·x · a
///   a_xx = (4·α²·x² - 2·α) · a
/// At r = x along the x-axis (y = z = 0):
///   a_x  = -2·α·r · a
///   a_xx = (4·α²·r² - 2·α) · a
fn build_gaussian_2nd_taylor(n0: f64, alpha: f64, r: f64) -> Vec<f64> {
    let a = 0.5 * n0 * (-alpha * r * r).exp();
    let ax = -2.0 * alpha * r * a;
    let axx = (4.0 * alpha * alpha * r * r - 2.0 * alpha) * a;
    let mut input = vec![0.0_f64; 20];
    // α-channel slots 0..9 (n gx gy gz xx xy xz yy yz zz)
    input[0] = a;
    input[1] = ax;
    input[2] = 0.0;
    input[3] = 0.0;
    input[4] = axx;
    input[5] = 0.0;
    input[6] = 0.0;
    input[7] = 0.0; // ayy = 0 (1D probe)
    input[8] = 0.0;
    input[9] = 0.0; // azz = 0
    // β-channel (closed-shell mirror at +10)
    input[10] = a;
    input[11] = ax;
    input[12] = 0.0;
    input[13] = 0.0;
    input[14] = axx;
    input[15] = 0.0;
    input[16] = 0.0;
    input[17] = 0.0;
    input[18] = 0.0;
    input[19] = 0.0;
    input
}

/// Convert "XC_PBEX" → "pbex" for `xcfun_set`. C++ side does
/// strcasecmps against the `XC_`-stripped name.
fn cpp_name(xc_name: &str) -> String {
    xc_name
        .strip_prefix("XC_")
        .unwrap_or(xc_name)
        .to_ascii_lowercase()
}

/// Locate the repo root (xtask's parent).
fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").context(
        "CARGO_MANIFEST_DIR not set — run via `cargo run -p xtask --features gen-potential-fixtures --bin gen-potential-fixtures`",
    )?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent")?
        .to_path_buf();
    Ok(root)
}

#[cfg(feature = "gen-potential-fixtures")]
fn generate_records() -> Result<Vec<PotentialRecord>> {
    let candidates = gaussian_grid_candidates();
    let grid = select_20_points(&candidates);
    eprintln!(
        "[gen-potential-fixtures] grid: {} points (down-sampled from {} candidates)",
        grid.len(),
        candidates.len()
    );

    let mut records: Vec<PotentialRecord> = Vec::with_capacity(TOTAL_FIXTURE_COUNT);

    for &name in FIXTURE_FUNCTIONALS {
        let mut cpp = CppXcfun::new();
        let s = cpp.set(&cpp_name(name), 1.0);
        anyhow::ensure!(s == 0, "xcfun_set({}, 1.0) failed: status={}", name, s);
        // Mode::Potential setup with vars = XC_A_B_2ND_TAYLOR.
        let setup = cpp.eval_setup(VARS_A_B_2ND_TAYLOR, MODE_POTENTIAL, 0);
        anyhow::ensure!(
            setup == 0,
            "xcfun_eval_setup({}, A_B_2ND_TAYLOR, POTENTIAL) failed: status={}",
            name,
            setup
        );
        let inlen = cpp.input_length();
        let outlen = cpp.output_length();
        anyhow::ensure!(
            inlen == 20 && outlen == 3,
            "{}: unexpected lengths inlen={} outlen={}",
            name,
            inlen,
            outlen
        );

        for &(n0, alpha, r) in &grid {
            let input = build_gaussian_2nd_taylor(n0, alpha, r);
            let mut output = vec![0.0_f64; 3];
            cpp.eval(&input, &mut output);
            records.push(PotentialRecord {
                functional_name: name,
                vars: "A_B_2ND_TAYLOR",
                input,
                expected_output: output,
            });
        }
    }

    eprintln!(
        "[gen-potential-fixtures] generated {} records ({} functionals × {} points)",
        records.len(),
        FIXTURE_FUNCTIONALS.len(),
        RECORDS_PER_FUNCTIONAL,
    );
    anyhow::ensure!(
        records.len() >= TOTAL_FIXTURE_COUNT,
        "expected {} records, got {}",
        TOTAL_FIXTURE_COUNT,
        records.len()
    );
    Ok(records)
}

#[cfg(not(feature = "gen-potential-fixtures"))]
fn generate_records() -> Result<Vec<PotentialRecord>> {
    anyhow::bail!(
        "gen-potential-fixtures binary requires the gen-potential-fixtures feature. \
         Run via: cargo run -p xtask --features gen-potential-fixtures --bin gen-potential-fixtures"
    );
}

fn main() -> Result<()> {
    let root = project_root()?;
    let records = generate_records()?;
    let out_path = root
        .join("crates/xcfun-eval/tests/data/potential_reference_100.json");
    fs::create_dir_all(out_path.parent().context("out path has no parent")?)?;

    let json = serde_json::to_string_pretty(&records)?;
    fs::write(&out_path, json)?;
    eprintln!(
        "[gen-potential-fixtures] wrote {} records to {:?}",
        records.len(),
        out_path
    );

    Ok(())
}
