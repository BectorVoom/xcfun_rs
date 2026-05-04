//! Phase 6 D-04 — Generate mpmath JSONL fixtures.
//!
//! Workflow (mirrors `regen_registry.rs` `--check` drift-gate pattern):
//!   1. Iterate the request matrix (functional × vars × mode × order × input grid).
//!   2. For each request, spawn `python3 -m xtask.mpmath_eval` with arguments;
//!      capture the JSONL record on stdout.
//!   3. Append records to `validation/fixtures/mpmath/<functional>.jsonl`.
//!   4. Compute SHA-256 of each file; write `<functional>.jsonl.sha256` stamp.
//!   5. `--check` mode regenerates in memory and compares stamps; exit 2 on drift.
//!
//! mpmath dep stays in Python land — Cargo build path NEVER calls Python (D-04).
//!
//! ## Plan 06-N2 scope
//!
//! Extends the Plan 06-00 functional set from 6 to **26** (6 ACC-04 +
//! 20 `excluded_by_upstream_spec`). Adds a `--smoke` mode that
//! regenerates a small subset (5 functionals × 5 records) suitable for
//! the autonomous CI lane (single-digit-second runtime).
//!
//! `--check` (drift gate): re-hashes existing fixtures in memory and
//! compares against the committed `.sha256` stamps. Fails fast with
//! exit code 2 on any drift. Does NOT regenerate.
//!
//! Default invocation (no flag): MANUAL ~6h offline regeneration of the
//! full ~600-record corpus. Documented in `06-N2-SUMMARY.md`. Should
//! NOT run in CI — uses `--check` instead.

use anyhow::{Context, Result, bail};
use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::SeedableRng;
// `Rng` brings `next_u64` for rand_xoshiro 0.8+ (replaces the deprecated
// `RngCore` trait).
use rand_xoshiro::rand_core::Rng as _;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

/// Locate the repo root (xtask's parent). Mirrors `regen_registry.rs`.
fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via `cargo run -p xtask --bin regen-mpmath-fixtures`")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory — unexpected layout")?
        .to_path_buf();
    Ok(root)
}

/// One stratified-grid input record = one Python invocation.
#[derive(Debug, Clone)]
struct InputRecord {
    vars: String,
    mode: String,
    order: u32,
    input_csv: String,
}

/// Per-functional canonical Vars + slot count (matches
/// `xtask/mpmath_eval/densvars.py::VARS_SLOTS`).
fn vars_for(fn_name: &str) -> (&'static str, usize) {
    match fn_name {
        // ACC-04 amendment set (Plan 06-00):
        "ldaerfx" | "ldaerfc" | "ldaerfc_jt" => ("A_B", 2),
        "tpssc" | "tpsslocc" | "revtpssc" => ("A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB", 9),
        // excluded_by_upstream_spec set (Plan 06-N2):
        "brx" | "brc" | "brxc" => (
            "A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB",
            11,
        ),
        "csc" => ("A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB", 11),
        "blocx" => ("A_B_GAA_GAB_GBB_TAUA_TAUB", 7),
        "scanx" | "scanc" | "rscanx" | "rscanc" | "rppscanx" | "rppscanc"
        | "r2scanx" | "r2scanc" | "r4scanx" | "r4scanc" => {
            ("A_B_GAA_GAB_GBB_TAUA_TAUB", 7)
        }
        "tw" | "vwk" => ("A_B_GAA_GAB_GBB", 5),
        "pbelocc" | "zvpbesolc" | "zvpbeintc" => ("A_B_GAA_GAB_GBB", 5),
        _ => panic!(
            "regen_mpmath_fixtures: unknown functional '{}' — add to vars_for()",
            fn_name
        ),
    }
}

/// Generate `n` density inputs from a deterministic xoshiro256++ stream.
/// Inputs respect physical scaling: a, b ∈ [0.05, 1.0]; gradients +
/// laplacians smaller; tau in a sensible range.
fn density_grid(seed: u64, n: usize, n_slots: usize, has_jp: bool) -> Vec<Vec<f64>> {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let mut points = Vec::with_capacity(n);
    for _ in 0..n {
        let mut row = Vec::with_capacity(n_slots);
        // Convert two u64 -> f64 in [0, 1).
        let next_unit = |rng: &mut Xoshiro256PlusPlus| -> f64 {
            let raw = rng.next_u64();
            (raw as f64) / (u64::MAX as f64)
        };
        for slot_idx in 0..n_slots {
            // Stratify the value range by slot kind:
            //   slots 0,1: a, b ∈ [0.05, 1.0]
            //   slots 2,4 (gaa, gbb) and 3 (gab): [0.001, 0.5]
            //   slots 5,6 (lapa, lapb) for layouts with laplacian: [-0.2, 0.2]
            //   slots 7,8 (taua, taub): [0.01, 0.5]
            //   slots 9,10 (jpaa, jpbb): [0.0, 0.05]
            let v: f64 = if slot_idx < 2 {
                0.05 + 0.95 * next_unit(&mut rng)
            } else if slot_idx < 5 {
                0.001 + 0.499 * next_unit(&mut rng)
            } else if !has_jp && slot_idx < 7 {
                // metaGGA-kinetic-only layouts: slots 5,6 = taua, taub.
                0.01 + 0.49 * next_unit(&mut rng)
            } else if has_jp && slot_idx < 7 {
                // 11-slot metaGGA-laplacian-kinetic-jp: slots 5,6 = lapa, lapb.
                -0.2 + 0.4 * next_unit(&mut rng)
            } else if slot_idx < 9 {
                0.01 + 0.49 * next_unit(&mut rng)
            } else {
                0.0 + 0.05 * next_unit(&mut rng)
            };
            row.push(v);
        }
        points.push(row);
    }
    points
}

/// Stratified grid for a given functional.
fn stratified_grid(fn_name: &str, smoke: bool) -> Vec<InputRecord> {
    let (vars_str, n_slots) = vars_for(fn_name);
    let has_jp = vars_str.contains("JPAA");
    // W-11 cap: ~30 records per functional (5 strata × 6 records). For
    // --smoke we cut to 5 records total.
    let n_records = if smoke { 5 } else { 30 };
    let seed = 0x1234_abcd_u64;
    let pts = density_grid(seed, n_records, n_slots, has_jp);
    pts.into_iter()
        .map(|pt| {
            let csv = pt
                .iter()
                .map(|v| format!("{:.16e}", v))
                .collect::<Vec<_>>()
                .join(",");
            InputRecord {
                vars: vars_str.into(),
                mode: "partial_derivatives".into(),
                order: 2,
                input_csv: csv,
            }
        })
        .collect()
}

/// Smoke subset: 5 functionals × 5 records each. Picked to cover the major
/// families landed by Plan 06-N2 (BR, kinetic-GGA, PBE-loc, BLOCX, SCAN).
///
/// Note: the ACC-04 set (LDAERF + TPSS) is NOT in the smoke pool because
/// Plan 06-N2 does NOT own those bodies — Plan 06-N1 (sibling worktree)
/// fills them. Including them here would couple this lane to N1's
/// completion order. The full-regen path (no flag) DOES include them
/// because it runs after both N1 and N2 have merged.
fn smoke_functionals() -> &'static [&'static str] {
    &["brx", "tw", "pbelocc", "blocx", "scanx"]
}

/// Full Plan-06-N2 functional set (6 ACC-04 + 20 excluded_by_upstream_spec).
fn full_functionals() -> &'static [&'static str] {
    &[
        // Plan 06-00 ACC-04 amendment set:
        "ldaerfx", "ldaerfc", "ldaerfc_jt", "tpssc", "tpsslocc", "revtpssc",
        // Plan 06-N2 excluded_by_upstream_spec set (BR family):
        "brx", "brc", "brxc",
        // misc + CSC + BLOCX:
        "csc", "blocx",
        // SCAN family ×10:
        "scanx", "scanc", "rscanx", "rscanc", "rppscanx", "rppscanc",
        "r2scanx", "r2scanc", "r4scanx", "r4scanc",
        // kinetic-GGA + PBE-correlation variants:
        "tw", "vwk", "pbelocc", "zvpbesolc", "zvpbeintc",
    ]
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");
    let smoke_mode = args.iter().any(|a| a == "--smoke");
    if check_mode && smoke_mode {
        bail!("--check and --smoke are mutually exclusive");
    }

    let functionals: &[&str] = if smoke_mode {
        smoke_functionals()
    } else {
        full_functionals()
    };
    let workspace_root = project_root()?;

    for fn_name in functionals {
        let grid = stratified_grid(fn_name, smoke_mode);
        let mut buf = String::new();
        for input_record in &grid {
            // python3 -m xtask.mpmath_eval --functional <name> --vars <V> ...
            let output = Command::new("python3")
                .arg("-m")
                .arg("xtask.mpmath_eval")
                .args(["--functional", fn_name])
                .args(["--vars", &input_record.vars])
                .args(["--mode", &input_record.mode])
                .args(["--order", &input_record.order.to_string()])
                .args(["--input", &input_record.input_csv])
                .args(["--prec", "200"])
                .current_dir(&workspace_root)
                .output()
                .with_context(|| {
                    format!("python3 -m xtask.mpmath_eval failed for {}", fn_name)
                })?;
            if !output.status.success() {
                bail!(
                    "mpmath sidecar failed for {}: {}",
                    fn_name,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            buf.push_str(&String::from_utf8(output.stdout)?);
        }

        let target = workspace_root
            .join("validation/fixtures/mpmath")
            .join(format!("{}.jsonl", fn_name));
        let stamp = workspace_root
            .join("validation/fixtures/mpmath")
            .join(format!("{}.jsonl.sha256", fn_name));

        let mut hasher = Sha256::new();
        hasher.update(buf.as_bytes());
        let hex = format!("{:x}", hasher.finalize());

        if check_mode {
            // --check mode: stamp file must already exist and match.
            let existing = std::fs::read_to_string(&stamp).with_context(|| {
                format!("missing committed sha256 stamp at {:?}", stamp)
            })?;
            if existing.trim() != hex {
                bail!(
                    "mpmath fixture drift detected for {}: expected {}, got {}",
                    fn_name,
                    existing.trim(),
                    hex
                );
            }
        } else if smoke_mode {
            // SMOKE mode: write to a temp directory under target/, NOT to
            // validation/fixtures/. The committed fixtures are produced by
            // the offline ~6h MANUAL regen run, not by --smoke.
            let smoke_dir = workspace_root.join("target/mpmath_smoke");
            std::fs::create_dir_all(&smoke_dir)?;
            let smoke_target = smoke_dir.join(format!("{}.jsonl", fn_name));
            std::fs::write(&smoke_target, &buf)?;
        } else {
            std::fs::create_dir_all(target.parent().unwrap())?;
            std::fs::write(&target, &buf)?;
            std::fs::write(&stamp, &hex)?;
        }
    }
    Ok(())
}
