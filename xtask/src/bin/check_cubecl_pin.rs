//! `check-cubecl-pin` — QG-06 gate.
//!
//! Runs `cargo metadata --format-version 1`, finds the `cubecl` and
//! `cubecl-cpu` packages in the resolved dependency graph, and asserts that
//! both are at EXACTLY `0.10.0-pre.3`. The pin exists because cubecl is a
//! pre-release; breaking changes can land between `-pre.N` and `-pre.N+1`
//! without semver bumps (see CLAUDE.md risk note).
//!
//! Exit codes:
//!   0 — PASS
//!   1 — cargo metadata invocation / parse error
//!   2 — FAIL: cubecl or cubecl-cpu at a version other than 0.10.0-pre.3

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const REQUIRED_VERSION: &str = "0.10.0-pre.3";
const PINNED_CRATES: &[&str] = &["cubecl", "cubecl-cpu"];

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin check-cubecl-pin")?;
    Ok(PathBuf::from(manifest)
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf())
}

fn main() -> Result<()> {
    let root = project_root()?;
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version", "1"])
        .output()
        .context("failed to spawn `cargo metadata`")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "cargo metadata failed (exit {:?}): {}",
            output.status.code(),
            stderr
        );
    }
    let metadata: Value =
        serde_json::from_slice(&output.stdout).context("parse cargo metadata JSON")?;

    let empty_vec: Vec<Value> = Vec::new();
    let packages = metadata["packages"].as_array().unwrap_or(&empty_vec);
    let mut violations = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();

    for pkg in packages {
        let name = pkg["name"].as_str().unwrap_or("");
        if !PINNED_CRATES.contains(&name) {
            continue;
        }
        let version = pkg["version"].as_str().unwrap_or("");
        seen.push((name.to_string(), version.to_string()));
        if version != REQUIRED_VERSION {
            violations.push(format!(
                "{}: version {} (expected {})",
                name, version, REQUIRED_VERSION
            ));
        }
    }

    if violations.is_empty() {
        println!(
            "check-cubecl-pin: PASS ({} cubecl crate(s) pinned at {})",
            seen.len(),
            REQUIRED_VERSION
        );
        Ok(())
    } else {
        eprintln!("\ncheck-cubecl-pin: FAIL");
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!(
            "\nQG-06: cubecl + cubecl-cpu must move in lockstep at\n\
             `{}`. Pre-release crates do not respect semver;\n\
             tip crates must share the exact `=` pin.",
            REQUIRED_VERSION
        );
        std::process::exit(2);
    }
}
