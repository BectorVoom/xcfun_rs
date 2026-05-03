//! `check-no-mul-add` — ACC-06 gate.
//!
//! Scans the per-functional `#[cube] fn` body trees for `.mul_add(` calls.
//! `mul_add` lowers to an FMA instruction, which fuses the multiply+add into
//! a single rounding step. For the 1e-12 algorithmic-identity contract vs.
//! C++ xcfun (which does NOT use FMA), any `.mul_add(...)` call would break
//! bit-level parity.
//!
//! Phase 6 Plan 06-01 (D-08) split `xcfun-kernels` out of `xcfun-eval`;
//! after the migration, the 78 functional bodies live under
//! `crates/xcfun-kernels/src/functionals/`. The legacy
//! `crates/xcfun-eval/src/functionals/` path is retained in the scan list
//! because it still hosts host-side launchers (`contracted.rs`) — those
//! must also stay FMA-free since they ship with the per-point cubecl-cpu
//! launch loop.
//!
//! Either directory missing is OK (vacuously clean); both present is the
//! steady state.
//!
//! Exit codes:
//!   0 — PASS (or every target directory absent)
//!   1 — I/O or setup error
//!   2 — FAIL: at least one `.mul_add(` call detected

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Directories scanned by `check-no-mul-add`. Order is purely cosmetic
/// (errors are aggregated across all of them).
///
/// Phase 6 Plan 06-01 (D-08) added `crates/xcfun-kernels/src/functionals/`
/// when the 78 `#[cube]` kernel bodies moved there from `xcfun-eval`.
const SCAN_DIRS: &[&str] = &[
    "crates/xcfun-eval/src/functionals",      // host-side launchers (contracted.rs); legacy bodies path
    "crates/xcfun-kernels/src/functionals",   // Phase 6 Plan 06-01: new home for the 78 #[cube] bodies
];

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin check-no-mul-add")?;
    Ok(PathBuf::from(manifest)
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf())
}

/// Scan a single `.rs` file for `.mul_add(` occurrences outside `//` line
/// comments. Returns `(line_number, line_text)` pairs for each hit.
fn scan_file(path: &Path) -> Result<Vec<(usize, String)>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut hits = Vec::new();
    for (i, line) in contents.lines().enumerate() {
        // Strip inline `//` line comment — anything after `//` is not code.
        let code = match line.find("//") {
            Some(pos) => &line[..pos],
            None => line,
        };
        // Hand-rolled scan for `.mul_add` followed by optional whitespace + `(`.
        // Avoids pulling in a regex crate.
        let needle = ".mul_add";
        if let Some(idx) = code.find(needle) {
            let after = &code[idx + needle.len()..];
            let trimmed = after.trim_start();
            if trimmed.starts_with('(') {
                hits.push((i + 1, line.to_string()));
            }
        }
    }
    Ok(hits)
}

fn main() -> Result<()> {
    let root = project_root()?;
    let mut violations = Vec::new();
    let mut files_scanned = 0usize;
    let mut dirs_present = 0usize;

    for rel in SCAN_DIRS {
        let target = root.join(rel);
        if !target.exists() {
            continue;
        }
        dirs_present += 1;
        for entry in WalkDir::new(&target).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            files_scanned += 1;
            for (line, text) in scan_file(entry.path())? {
                violations.push(format!(
                    "{}:{}: {}",
                    entry.path().display(),
                    line,
                    text.trim()
                ));
            }
        }
    }

    if violations.is_empty() {
        if dirs_present == 0 {
            println!(
                "check-no-mul-add: PASS (no scan target exists yet — gate vacuously clean)"
            );
        } else {
            println!(
                "check-no-mul-add: PASS ({} file(s) scanned across {} target directory(ies))",
                files_scanned, dirs_present
            );
        }
        Ok(())
    } else {
        eprintln!(
            "\ncheck-no-mul-add: FAIL — {} violation(s):",
            violations.len()
        );
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!(
            "\nACC-06: `.mul_add(...)` lowers to FMA, which fuses multiply+add\n\
             into one rounding step and breaks the 1e-12 algorithmic-identity\n\
             contract vs. C++ xcfun. Use explicit two-step: compute the product\n\
             into a temp, then add."
        );
        std::process::exit(2);
    }
}
