//! Regenerate `crates/xcfun-ad/tests/fixtures/{mul.bincode, expand.bincode,
//! fixtures.json}` from a C++ driver linking
//! `xcfun-master/external/upstream/taylor/`.
//!
//! Workflow:
//!   1. Compile `xtask/assets/regen_ad_fixtures/driver.cpp` to an executable
//!      using the system C++ compiler (`$CXX` env var, else `c++`). We
//!      invoke the compiler directly rather than via `cc::Build` because
//!      the cc crate is oriented toward linking static libs into Rust
//!      binaries, not producing standalone C++ executables.
//!   2. Run the driver, capture stdout, parse its semicolon-separated
//!      records into `FixtureRecord` values.
//!   3. Partition records by op family → `mul.bincode` (op == "mul") and
//!      `expand.bincode` (op ends with "_expand").
//!   4. Serialise each partition via `bincode` into `crates/xcfun-ad/tests/
//!      fixtures/`.
//!   5. Write `fixtures.json` manifest with:
//!        - sha256 of the three vendored taylor headers
//!          (`ctaylor.hpp`, `ctaylor_math.hpp`, `tmath.hpp`) —
//!          drift detector
//!        - per-op record counts
//!        - RFC 3339 generated_at timestamp
//!        - optional git rev-parse HEAD
//!
//! The resulting bincode files are COMMITTED to the repo (D-19); CI does
//! not regenerate them. Re-running this binary on the same xcfun-master
//! tree produces byte-identical output (deterministic mt19937_64 seed +
//! deterministic bincode serialisation).

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// Re-export the shared schema defined in `xtask/src/fixtures.rs`.
#[path = "../fixtures.rs"]
mod fixtures;
use fixtures::{FixtureRecord, FixturesManifest};

/// Locate the repo root (xtask's parent). Relies on `CARGO_MANIFEST_DIR`
/// being `<repo>/xtask`.
fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").context(
        "CARGO_MANIFEST_DIR not set — run via `cargo run -p xtask --bin regen-ad-fixtures`",
    )?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory — unexpected layout")?
        .to_path_buf();
    Ok(root)
}

fn compile_driver(xcfun_taylor: &Path, driver_src: &Path, driver_exe: &Path) -> Result<()> {
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".into());
    let status = Command::new(&compiler)
        .args([
            "-std=c++17",
            "-O2",
            // Preserve arithmetic parity — no FMA fusion / reassociation.
            "-fno-fast-math",
            "-ffp-contract=off",
            "-I",
        ])
        .arg(xcfun_taylor)
        .arg(driver_src)
        .arg("-o")
        .arg(driver_exe)
        .status()
        .with_context(|| format!("failed to spawn C++ compiler ({})", compiler))?;
    if !status.success() {
        bail!(
            "compiling driver.cpp failed (compiler {}, exit {:?})",
            compiler,
            status.code()
        );
    }
    Ok(())
}

fn run_driver_capture(driver_exe: &Path) -> Result<String> {
    let mut cmd = Command::new(driver_exe)
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn driver exe at {:?}", driver_exe))?;
    let mut stdout = cmd.stdout.take().context("driver has no stdout pipe")?;
    let mut buf = String::new();
    stdout.read_to_string(&mut buf)?;
    let status = cmd.wait()?;
    if !status.success() {
        bail!("driver exited non-zero ({:?})", status.code());
    }
    Ok(buf)
}

/// Parse `<op>;<n_var>;<inp_count>;<i0,...>;<coeff_count>;<c0,...>` into a
/// `FixtureRecord`. Returns an error on any shape mismatch so caller can
/// fail the regen run (never commit malformed fixtures).
fn parse_line(line: &str, lineno: usize) -> Result<FixtureRecord> {
    let parts: Vec<&str> = line.split(';').collect();
    if parts.len() != 6 {
        bail!(
            "line {}: malformed (got {} fields, want 6): {}",
            lineno,
            parts.len(),
            line
        );
    }
    let op = parts[0].to_string();
    let n_var: u8 = parts[1]
        .parse()
        .with_context(|| format!("line {}: bad n_var", lineno))?;
    let input_count: usize = parts[2]
        .parse()
        .with_context(|| format!("line {}: bad input_count", lineno))?;
    let inputs: Vec<f64> = if input_count == 0 {
        Vec::new()
    } else {
        parts[3]
            .split(',')
            .map(|s| {
                s.parse::<f64>()
                    .with_context(|| format!("line {}: bad input f64 {:?}", lineno, s))
            })
            .collect::<Result<Vec<_>>>()?
    };
    anyhow::ensure!(
        inputs.len() == input_count,
        "line {}: input count mismatch ({} vs {})",
        lineno,
        inputs.len(),
        input_count
    );
    let coeff_count: usize = parts[4]
        .parse()
        .with_context(|| format!("line {}: bad coeff_count", lineno))?;
    let coeffs: Vec<f64> = if coeff_count == 0 {
        Vec::new()
    } else {
        parts[5]
            .split(',')
            .map(|s| {
                s.parse::<f64>()
                    .with_context(|| format!("line {}: bad coeff f64 {:?}", lineno, s))
            })
            .collect::<Result<Vec<_>>>()?
    };
    anyhow::ensure!(
        coeffs.len() == coeff_count,
        "line {}: coeff count mismatch ({} vs {})",
        lineno,
        coeffs.len(),
        coeff_count
    );
    Ok(FixtureRecord {
        op,
        n_var,
        inputs,
        coeffs,
    })
}

fn header_sha256(xcfun_taylor: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    // Hash in a deterministic order. Do NOT include Zone.Identifier files.
    for fname in ["ctaylor.hpp", "ctaylor_math.hpp", "tmath.hpp"] {
        let path = xcfun_taylor.join(fname);
        let contents =
            fs::read(&path).with_context(|| format!("read xcfun-master header {:?}", path))?;
        hasher.update(&contents);
    }
    let sha = hasher.finalize();
    Ok(format!("{:x}", sha))
}

fn git_head_sha(root: &Path) -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
}

fn main() -> Result<()> {
    let root = project_root()?;
    let xcfun_taylor = root.join("xcfun-master/external/upstream/taylor");
    let driver_src = root.join("xtask/assets/regen_ad_fixtures/driver.cpp");
    let out_dir = root.join("target/xtask/fixtures");
    let fixtures_dir = root.join("crates/xcfun-ad/tests/fixtures");

    anyhow::ensure!(
        xcfun_taylor.join("ctaylor.hpp").exists(),
        "xcfun-master taylor headers not found at {:?} — expected vendored sources",
        xcfun_taylor
    );
    anyhow::ensure!(
        driver_src.exists(),
        "driver source not found at {:?}",
        driver_src
    );

    fs::create_dir_all(&out_dir)?;
    fs::create_dir_all(&fixtures_dir)?;

    let driver_exe = out_dir.join("regen_ad_driver");
    eprintln!("[regen-ad-fixtures] compiling driver → {:?}", driver_exe);
    compile_driver(&xcfun_taylor, &driver_src, &driver_exe)?;

    eprintln!("[regen-ad-fixtures] running driver");
    let stdout = run_driver_capture(&driver_exe)?;

    // Parse driver stdout → Vec<FixtureRecord>.
    let mut records: Vec<FixtureRecord> = Vec::new();
    for (i, line) in stdout.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        records.push(parse_line(line, i)?);
    }
    anyhow::ensure!(
        !records.is_empty(),
        "driver emitted zero records — something went wrong"
    );

    // Partition: mul records vs expand records vs composed records.
    //   - op == "mul"             → mul.bincode
    //   - op ends with "_expand"  → expand.bincode
    //   - op starts with "ctaylor_" → composed.bincode  (Plan 01-06)
    // Any other op silently skipped (future plans can extend further).
    let mut mul_records: Vec<FixtureRecord> = Vec::new();
    let mut expand_records: Vec<FixtureRecord> = Vec::new();
    let mut composed_records: Vec<FixtureRecord> = Vec::new();
    for rec in &records {
        if rec.op == "mul" {
            mul_records.push(rec.clone());
        } else if rec.op.ends_with("_expand") {
            expand_records.push(rec.clone());
        } else if rec.op.starts_with("ctaylor_") {
            composed_records.push(rec.clone());
        }
    }

    // Sort records deterministically before bincode serialisation so byte-level
    // output is stable against implementation order changes.
    mul_records.sort_by(|a, b| {
        a.n_var.cmp(&b.n_var).then_with(|| {
            // Break ties by first input value to remain deterministic.
            a.inputs
                .partial_cmp(&b.inputs)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    expand_records.sort_by(|a, b| {
        a.op.cmp(&b.op).then_with(|| {
            a.n_var.cmp(&b.n_var).then_with(|| {
                a.inputs
                    .partial_cmp(&b.inputs)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
    });
    composed_records.sort_by(|a, b| {
        a.op.cmp(&b.op).then_with(|| {
            a.n_var.cmp(&b.n_var).then_with(|| {
                a.inputs
                    .partial_cmp(&b.inputs)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
    });

    let mul_bytes = bincode::serialize(&mul_records).context("serialise mul records")?;
    let expand_bytes = bincode::serialize(&expand_records).context("serialise expand records")?;
    let composed_bytes =
        bincode::serialize(&composed_records).context("serialise composed records")?;
    fs::write(fixtures_dir.join("mul.bincode"), &mul_bytes)?;
    fs::write(fixtures_dir.join("expand.bincode"), &expand_bytes)?;
    fs::write(fixtures_dir.join("composed.bincode"), &composed_bytes)?;

    // Manifest: per-op counts, content hash, driver_commit, timestamp.
    let mut per_op_counts: BTreeMap<String, usize> = BTreeMap::new();
    for rec in &records {
        *per_op_counts.entry(rec.op.clone()).or_insert(0) += 1;
    }
    let xcfun_version_git_sha = header_sha256(&xcfun_taylor)?;
    let manifest = FixturesManifest {
        xcfun_version_git_sha: xcfun_version_git_sha.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        per_op_counts,
        total_records: records.len(),
        driver_commit: git_head_sha(&root),
    };
    fs::write(
        fixtures_dir.join("fixtures.json"),
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;

    eprintln!(
        "[regen-ad-fixtures] wrote {} mul records ({} bytes), {} expand records ({} bytes), {} composed records ({} bytes)",
        mul_records.len(),
        mul_bytes.len(),
        expand_records.len(),
        expand_bytes.len(),
        composed_records.len(),
        composed_bytes.len()
    );
    eprintln!(
        "[regen-ad-fixtures] manifest sha256[..16] = {}",
        &xcfun_version_git_sha[..16]
    );
    Ok(())
}
