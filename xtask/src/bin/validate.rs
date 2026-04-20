//! `validate` — CLI entry point that delegates to the `validation` crate.
//!
//! This is a thin wrapper: all argv after the binary name is passed through
//! to `cargo run -p validation --release -- <args>`. The `validation` crate
//! is landed in Plan 02-06; until then, running this wrapper fails with
//! cargo's standard "package `validation` not found" message, which is the
//! expected behaviour (the plan's success-criteria test runs end-to-end
//! only after Plan 02-06 lands).
//!
//! Exit codes:
//!   * Whatever `cargo run -p validation --release -- <args>` returned.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin validate")?;
    Ok(PathBuf::from(manifest)
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf())
}

fn main() -> Result<()> {
    let root = project_root()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(["run", "-p", "validation", "--release", "--"])
        .args(&args)
        .status()
        .context("spawning `cargo run -p validation`")?;
    std::process::exit(status.code().unwrap_or(1));
}
