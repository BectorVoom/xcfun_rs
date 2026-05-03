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
//! ## Plan 06-00 scope
//!
//! Only the SUBPROCESS WIRING is exercised here. The 6 ACC-04-amended
//! functionals (LDAERF family + TPSS-correlation family) live as
//! `NotImplementedError` stubs in `xtask/mpmath_eval/functionals/`; running
//! `cargo run -p xtask --bin regen-mpmath-fixtures` will fail at the first
//! Python invocation. That is expected: Plan 06-N2 fills the bodies.
//!
//! Plan 06-00's smoke check is `cargo build -p xtask --bin regen-mpmath-fixtures`,
//! which verifies the Rust driver compiles and the Python subprocess
//! `Command::new("python3").arg("-m").arg("xtask.mpmath_eval")` is wired
//! per CONTEXT D-04.

use anyhow::{Context, Result, bail};
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

/// Stratified grid stub. Plan 06-N2 will populate this with the
/// xoshiro256++-seed-0x1234abcd 50-record grid per Phase 2 D-18.
fn stratified_grid(_fn_name: &str) -> Vec<InputRecord> {
    // Plan 06-00 ships an empty grid so `cargo build` smoke-tests the
    // Rust driver without exercising the Python sidecar (whose per-functional
    // bodies are NotImplementedError stubs). Plan 06-N2 fills both ends.
    Vec::new()
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");

    // Phase 6 Plan 06-00 set: 6 ACC-04-amended functionals × ~50 records each.
    // Plan 06-N2 will extend this set with the 20 `excluded_by_upstream_spec` functionals.
    let functionals = [
        "ldaerfx",
        "ldaerfc",
        "ldaerfc_jt",
        "tpssc",
        "tpsslocc",
        "revtpssc",
    ];
    let workspace_root = project_root()?;

    for fn_name in &functionals {
        let mut buf = String::new();
        for input_record in stratified_grid(fn_name) {
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
            let existing = std::fs::read_to_string(&stamp)
                .with_context(|| format!("missing committed sha256 stamp at {:?}", stamp))?;
            if existing.trim() != hex {
                bail!(
                    "mpmath fixture drift detected for {}: expected {}, got {}",
                    fn_name,
                    existing.trim(),
                    hex
                );
            }
        } else {
            std::fs::create_dir_all(target.parent().unwrap())?;
            std::fs::write(&target, &buf)?;
            std::fs::write(&stamp, &hex)?;
        }
    }
    Ok(())
}
