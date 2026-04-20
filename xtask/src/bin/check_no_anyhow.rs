//! `check-no-anyhow` — QG-01 gate.
//!
//! Walks `crates/*/Cargo.toml`, parses the `[dependencies]` table, and fails
//! if any library crate declares a `anyhow` dependency. `anyhow` is an
//! app-boundary dep (for `validation/`, `xtask/`, `benches/`, `examples/`);
//! library crates in the `xcfun-*` graph must use `thiserror` + `XcError`.
//!
//! `[dev-dependencies]` is ALLOWED (Phase 1 `xcfun-ad` uses anyhow in its
//! test harness). Only the normal `[dependencies]` table is checked.
//!
//! Exit codes:
//!   0 — PASS
//!   1 — I/O or parse error
//!   2 — FAIL: a library crate depends on anyhow

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use toml::Value;

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin check-no-anyhow")?;
    Ok(PathBuf::from(manifest)
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf())
}

/// Check one crate's Cargo.toml for an anyhow dep in `[dependencies]`.
fn check_cargo_toml(path: &Path) -> Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        toml::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    let mut violations = Vec::new();
    if let Some(deps) = value.get("dependencies").and_then(|v| v.as_table()) {
        if deps.contains_key("anyhow") {
            violations.push(format!(
                "{}: [dependencies] contains `anyhow` (library crates must use thiserror + XcError)",
                path.display()
            ));
        }
    }
    Ok(violations)
}

fn main() -> Result<()> {
    let root = project_root()?;
    let crates_dir = root.join("crates");
    let mut violations = Vec::new();
    let mut checked = 0usize;

    if crates_dir.is_dir() {
        for entry in std::fs::read_dir(&crates_dir)
            .with_context(|| format!("read_dir {}", crates_dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let cargo_toml = entry.path().join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }
            checked += 1;
            violations.extend(check_cargo_toml(&cargo_toml)?);
        }
    }

    if violations.is_empty() {
        println!(
            "check-no-anyhow: PASS ({} library crate(s) checked; no anyhow in normal deps)",
            checked
        );
        Ok(())
    } else {
        eprintln!("\ncheck-no-anyhow: FAIL");
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!(
            "\nQG-01: `anyhow` is permitted only at app boundaries\n\
             (validation/, xtask/, benches/, examples/). Library crates must\n\
             return structured errors via `thiserror` + `XcError`."
        );
        std::process::exit(2);
    }
}
