//! Plan 06-N3 D-19 fixture regenerator.
//!
//! Runs `Functional::eval` over hand-curated density-strata grids for each
//! of the 18 in-scope functionals (M05/M06×10 + B97-X×3 + LYPC + VWN_PBEC +
//! PW92C + PBEC + OPTX) and emits the result as
//! `validation/fixtures/d19_n3/<name>_baseline.jsonl` records.
//!
//! ## Invocation
//!
//! ```bash
//! cargo test -p xcfun-kernels --test d19_n3_regen_fixtures -- --include-ignored
//! ```
//!
//! ## Why a `#[ignore]`-d test (instead of an xtask binary)
//!
//! - `Functional::eval` is gated behind the `xcfun-eval/testing` feature
//!   (cubecl-cpu launcher lives in `for_tests::cpu_client`). xtask binaries
//!   cannot easily depend on `xcfun-eval` with a feature flag without
//!   leaking the testing dep into the production xtask graph.
//! - The pattern mirrors the established `regen-ad-fixtures` and
//!   `regen-mpmath-fixtures` xtask binaries but lives in the test binary
//!   that consumes the fixtures, so generation + verification share the
//!   same `D19Record` schema.
//! - Regeneration is rare (happens once when fixtures are first created
//!   AND once more if a future kernel-edit plan changes outputs at these
//!   density points). The `#[ignore]` attribute keeps it out of the
//!   default test suite.
//!
//! ## Snapshot semantics — see `tests/common/mod.rs`
//!
//! In the parallel-execution worktree where `xcfun-master/` is gitignored
//! and not present, the `expected` field is a regression snapshot of the
//! current `Functional::eval` output (Plan 06-00 substrate revision). When
//! the orchestrator merges 06-N3 back to `master` and re-runs tier-2 with
//! `xcfun-master/` restored, the expected values can be re-emitted from
//! the C++ baseline by re-running this regenerator with a `--reference cpp`
//! flag (currently a no-op stub — Plan 06-00b/c may extend).

#![cfg(feature = "testing")]
#![allow(dead_code)] // common/ helpers are used by other test files but
                     // each cargo test integration target re-includes the
                     // module independently, so unused warnings are normal.

mod common;

use std::fs;
use std::io::Write;

use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;

/// One curated density-strata grid point, before evaluation. The
/// regenerator runs `Functional::eval` on each to produce the
/// `expected` field of the persisted fixture record.
struct GridPoint {
    /// Free-text label used only for fixture commenting; stripped on
    /// disk because the JSONL `D19Record` schema doesn't carry it.
    _label: &'static str,
    /// Flat density input matching `Vars::input_len(vars)`.
    input: &'static [f64],
}

/// Curated GGA-vars=6 grid (5 records). Layout: `[a, b, gaa, gab, gbb]`.
///
/// Mix of strata that triggered Phase-4 D-19 residuals on the GGA-vars=6
/// dispatch arms (LYPC 1.3e-10 + VWN_PBEC 6.9e-9 + PBEC 6.6e-9 etc.):
///
/// 1. low-density polarised: a >> b, both > regularize bound (2e-14).
/// 2. low-density polarised swapped: b >> a (symmetry stress).
/// 3. gradient-stress: large `gaa+gbb` relative to `(a+b)^(4/3)`.
/// 4. balanced moderate: typical bulk density.
/// 5. mixed-spin asymmetric: a ≈ b but with off-diagonal `gab` ≠ 0.
const GGA_GRID: &[GridPoint] = &[
    GridPoint {
        _label: "low_density_polarised_aa",
        input: &[1.0e-3, 1.0e-7, 1.0e-7, 1.0e-9, 1.0e-13],
    },
    GridPoint {
        _label: "low_density_polarised_bb",
        input: &[1.0e-7, 1.0e-3, 1.0e-13, 1.0e-9, 1.0e-7],
    },
    GridPoint {
        _label: "gradient_stress",
        input: &[1.0e-2, 1.0e-2, 1.0e-1, 5.0e-2, 1.0e-1],
    },
    GridPoint {
        _label: "balanced_moderate",
        input: &[0.5, 0.5, 0.1, 0.05, 0.1],
    },
    GridPoint {
        _label: "mixed_spin_asymmetric",
        input: &[0.3, 0.4, 0.05, 0.02, 0.07],
    },
];

/// Curated metaGGA-vars=13 grid (5 records). Layout:
/// `[a, b, gaa, gab, gbb, taua, taub]`.
///
/// Adds tau (kinetic-energy density) component on top of GGA grid. Mix of
/// strata that triggered Phase-4 D-19 residuals on the metaGGA-vars=13
/// dispatch arms (M05/M06 family 1.5e-12..6.3e-11):
///
/// 1. low-density + low-tau (tau ≈ tau_w boundary, but above the
///    Plan 06-00 D-10 guard since these are M05/M06 not TPSS).
/// 2. polarised + balanced tau.
/// 3. gradient-stress + small tau.
/// 4. balanced moderate density + balanced tau.
/// 5. asymmetric + asymmetric tau.
///
/// `tau` choices respect the physical `tau ≥ tau_w = |∇ρ|² / (8ρ)` bound
/// (with margin) so that none of the points trip the Plan 06-00 D-10
/// TPSS guard if any TPSS path is reached transitively.
const MGGA_GRID: &[GridPoint] = &[
    GridPoint {
        _label: "low_density_low_tau",
        input: &[1.0e-3, 5.0e-4, 1.0e-7, 5.0e-8, 1.0e-7, 1.0e-3, 5.0e-4],
    },
    GridPoint {
        _label: "polarised_balanced_tau",
        input: &[1.0e-2, 1.0e-4, 1.0e-5, 1.0e-7, 1.0e-9, 5.0e-2, 5.0e-4],
    },
    GridPoint {
        _label: "gradient_stress_small_tau",
        input: &[1.0e-2, 1.0e-2, 5.0e-2, 2.0e-2, 5.0e-2, 5.0e-2, 5.0e-2],
    },
    GridPoint {
        _label: "balanced_moderate",
        input: &[0.5, 0.5, 0.1, 0.05, 0.1, 0.3, 0.3],
    },
    GridPoint {
        _label: "asymmetric_density_tau",
        input: &[0.3, 0.4, 0.05, 0.02, 0.07, 0.15, 0.25],
    },
];

/// (Lowercase name, FunctionalId, Vars discriminant) for each in-scope
/// functional in this plan's scope.
///
/// Source: `04-VERIFICATION.md` D-19 ledger (Phase 4 sign-off summary in
/// `.planning/STATE.md`) + `xcfun-master/src/functionals/<name>.cpp`
/// `Dependency` mask (read via `crates/xcfun-core/src/registry/generated`).
const PLAN_SCOPE: &[(&str, FunctionalId, Vars)] = &[
    // M05 / M06 family — metaGGA, vars=13.
    ("m05x", FunctionalId::XC_M05X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m05c", FunctionalId::XC_M05C, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m05x2c", FunctionalId::XC_M05X2C, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06x", FunctionalId::XC_M06X, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06c", FunctionalId::XC_M06C, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06lx", FunctionalId::XC_M06LX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06lc", FunctionalId::XC_M06LC, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06hfx", FunctionalId::XC_M06HFX, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06hfc", FunctionalId::XC_M06HFC, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    ("m06x2c", FunctionalId::XC_M06X2C, Vars::A_B_GAA_GAB_GBB_TAUA_TAUB),
    // B97 family X-side — GGA, vars=6.
    ("b97x", FunctionalId::XC_B97X, Vars::A_B_GAA_GAB_GBB),
    ("b97_1x", FunctionalId::XC_B97_1X, Vars::A_B_GAA_GAB_GBB),
    ("b97_2x", FunctionalId::XC_B97_2X, Vars::A_B_GAA_GAB_GBB),
    // Singletons — all GGA, vars=6.
    ("lypc", FunctionalId::XC_LYPC, Vars::A_B_GAA_GAB_GBB),
    ("vwn_pbec", FunctionalId::XC_VWN_PBEC, Vars::A_B_GAA_GAB_GBB),
    ("pw92c", FunctionalId::XC_PW92C, Vars::A_B_GAA_GAB_GBB),
    ("pbec", FunctionalId::XC_PBEC, Vars::A_B_GAA_GAB_GBB),
    ("optx", FunctionalId::XC_OPTX, Vars::A_B_GAA_GAB_GBB),
];

const ORDER: u32 = 3;

/// Number of output values for `Mode::PartialDerivatives` at order N over
/// inlen variables: `binom(N + inlen, N)`. Reproduces `xcfun_core::taylorlen`
/// inline so we don't have to re-export it.
fn taylorlen(inlen: usize, order: usize) -> usize {
    // C(n+order, order)
    let n = inlen + order;
    let mut acc: u128 = 1;
    let k = order.min(n - order);
    for i in 0..k {
        acc = acc * ((n - i) as u128) / ((i + 1) as u128);
    }
    acc as usize
}

/// Emit one fixture file. Returns the number of records written.
fn emit_fixture(
    name: &str,
    id: FunctionalId,
    vars: Vars,
) -> usize {
    let inlen = vars.input_len();
    let outlen = taylorlen(inlen, ORDER as usize);

    let grid: &[GridPoint] = match vars {
        Vars::A_B_GAA_GAB_GBB => GGA_GRID,
        Vars::A_B_GAA_GAB_GBB_TAUA_TAUB => MGGA_GRID,
        _ => panic!("unsupported vars for Plan 06-N3: {:?}", vars),
    };

    // Sanity-check input lengths.
    for (i, gp) in grid.iter().enumerate() {
        assert_eq!(
            gp.input.len(),
            inlen,
            "grid record {} has wrong inlen: {} vs {}",
            i,
            gp.input.len(),
            inlen
        );
    }

    let mut fun = Functional::new();
    fun.weights = vec![(id, 1.0)];
    fun.vars = vars;
    fun.mode = Mode::PartialDerivatives;
    fun.order = ORDER;

    // Compute settings_gen so xcfun-rs facade tests would also see a
    // populated settings array; we don't go through `set()` here because
    // we're writing weights directly (xcfun-eval-side test pattern).
    fun.settings[id as usize] = 1.0;

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("validation")
        .join("fixtures")
        .join("d19_n3")
        .join(format!("{}_baseline.jsonl", name));

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut f = fs::File::create(&path).unwrap();

    let mut written = 0_usize;
    for gp in grid {
        let mut out = vec![0.0_f64; outlen];
        fun.eval(gp.input, &mut out).unwrap_or_else(|e| {
            panic!(
                "regen Functional::eval({}, vars={:?}, order=3) failed at point {:?}: {:?}",
                name, vars, gp.input, e
            )
        });

        // Manually emit JSONL with stable key order: functional, vars, mode,
        // order, input, expected, rel_err_threshold. We don't use serde's
        // derive on a struct here because we want the on-disk keys ordered
        // deterministically for stable git diffs.
        let mut record = String::with_capacity(256);
        record.push('{');
        record.push_str(&format!("\"functional\":\"{}\",", name));
        record.push_str(&format!("\"vars\":{},", vars as u32));
        record.push_str(&format!("\"mode\":{},", Mode::PartialDerivatives as u32));
        record.push_str(&format!("\"order\":{},", ORDER));
        record.push_str("\"input\":[");
        for (i, x) in gp.input.iter().enumerate() {
            if i > 0 {
                record.push(',');
            }
            record.push_str(&format_f64(*x));
        }
        record.push_str("],\"expected\":[");
        for (i, x) in out.iter().enumerate() {
            if i > 0 {
                record.push(',');
            }
            record.push_str(&format_f64(*x));
        }
        record.push_str("],\"rel_err_threshold\":1e-13");
        record.push('}');
        record.push('\n');
        f.write_all(record.as_bytes()).unwrap();
        written += 1;
    }
    f.flush().unwrap();
    written
}

/// Format an `f64` as a JSON number with full f64 precision. Uses the
/// shortest representation that round-trips bit-exactly via Rust's
/// `f64::to_string` + `<f64 as FromStr>`. JSONL consumers (serde_json)
/// parse this losslessly.
fn format_f64(x: f64) -> String {
    if x.is_nan() {
        // JSON has no NaN; we'd need to widen the schema. None of the
        // curated grid points produce NaN at order 3, so panic clearly
        // if it does.
        panic!("regen produced NaN — grid point or kernel needs investigation");
    }
    if x.is_infinite() {
        panic!("regen produced infinity — grid point or kernel needs investigation");
    }
    // {:?} on f64 emits a debug-format that round-trips; alternative:
    // `format!("{:e}", x)` is also stable but harder to read in diffs.
    // Use {:e} for consistency with serde_json's float-emitter style.
    if x == 0.0 {
        return "0.0".to_string();
    }
    // Use Rust's Display which emits the shortest-round-trip form.
    format!("{:e}", x)
}

/// `cargo test -p xcfun-kernels --test d19_n3_regen_fixtures -- --include-ignored`
///
/// Re-emits all 18 Plan 06-N3 fixture files from the current substrate
/// revision's `Functional::eval` output. Commit the diff explicitly with
/// a citation to the kernel-edit plan that motivated the regeneration.
#[test]
#[ignore = "regenerator — invoke with --include-ignored to re-emit fixtures"]
fn regen_all_d19_n3_fixtures() {
    let mut total = 0_usize;
    for &(name, id, vars) in PLAN_SCOPE {
        let n = emit_fixture(name, id, vars);
        total += n;
        println!("regen {}: {} records ({} vars={:?})", name, n, name, vars);
    }
    assert_eq!(total, PLAN_SCOPE.len() * 5, "expected 5 records per functional");
}
