//! `check-boundaries` — QG-02 gate.
//!
//! Runs `cargo metadata --format-version 1 --no-deps`, walks the `packages[]`
//! array, and for each crate in the allowlist asserts that its normal
//! dependencies (kind is null) are a subset of the allowed set.
//!
//! Allowlist (Phase 2 scope):
//!   - xcfun-core: {thiserror, bitflags}
//!   - xcfun-ad:   {cubecl, cubecl-cpu, bytemuck}
//!   - xcfun-eval: {xcfun-core, xcfun-ad, cubecl, cubecl-cpu, thiserror}
//!     (forward-compatible: Plan 02-03 adds the crate; this gate is OK when
//!      it's not yet present)
//! `validation` / `xtask` — unrestricted (app-boundary).
//!
//! Exit codes:
//!   0 — PASS
//!   1 — cargo metadata invocation / parse error
//!   2 — FAIL: a crate pulls in a dep outside its allowlist

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin check-boundaries")?;
    Ok(PathBuf::from(manifest)
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf())
}

fn allowlist() -> HashMap<&'static str, &'static [&'static str]> {
    let mut m = HashMap::new();
    m.insert("xcfun-core", &["thiserror", "bitflags"][..]);
    m.insert("xcfun-ad", &["cubecl", "cubecl-cpu", "bytemuck"][..]);
    m.insert(
        "xcfun-eval",
        &["xcfun-core", "xcfun-ad", "cubecl", "cubecl-cpu", "thiserror"][..],
    );
    m
}

fn main() -> Result<()> {
    let root = project_root()?;
    let output = Command::new("cargo")
        .current_dir(&root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
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
    let allow = allowlist();
    let mut violations = Vec::new();
    let mut checked = Vec::new();

    let empty_vec: Vec<Value> = Vec::new();
    let packages = metadata["packages"].as_array().unwrap_or(&empty_vec);
    for pkg in packages {
        let name = pkg["name"].as_str().unwrap_or("");
        let Some(allowed) = allow.get(name) else {
            continue;
        };
        checked.push(name.to_string());
        let deps = pkg["dependencies"].as_array().unwrap_or(&empty_vec);
        for dep in deps {
            let dep_name = dep["name"].as_str().unwrap_or("");
            // `kind` is null for normal deps, "dev" for dev-deps, "build" for
            // build-deps. Only normal deps are gated.
            let is_normal = dep["kind"].is_null();
            if !is_normal {
                continue;
            }
            if !allowed.contains(&dep_name) {
                violations.push(format!(
                    "{}: normal dep `{}` not in allowlist {:?}",
                    name, dep_name, allowed
                ));
            }
        }
    }

    if violations.is_empty() {
        println!(
            "check-boundaries: PASS ({} gated crate(s) checked: {:?})",
            checked.len(),
            checked
        );
        Ok(())
    } else {
        eprintln!("\ncheck-boundaries: FAIL");
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!(
            "\nQG-02: per-crate dependency allowlist violated. Add the dep to\n\
             the allowlist in xtask/src/bin/check_boundaries.rs if it is\n\
             intentional, or drop the dep to keep the library boundary clean."
        );
        std::process::exit(2);
    }
}
