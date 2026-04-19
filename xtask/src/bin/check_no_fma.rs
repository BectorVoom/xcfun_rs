//! ROADMAP Phase 1 SC #6: assert that cubecl-cpu's MLIR JIT does NOT emit
//! FMA (`vfmadd{132,213,231}pd/sd`, `fmadd`, `fma213`, `fma231`) in the
//! `ctaylor_mul*` symbols when `xcfun-ad` is compiled `--release` with
//! `-Cllvm-args=-fp-contract=off` in `.cargo/config.toml`.
//!
//! Operation (D-02, D-03):
//!   1. `cargo rustc -p xcfun-ad --release --lib --features cpu -- --emit=asm`
//!   2. Locate every `.s` file emitted under `target/release/deps/`.
//!   3. Split each `.s` into function blocks by symbol label.
//!   4. Demangle each label via `rustc_demangle::demangle`.
//!   5. If the demangled name contains `ctaylor_mul`, grep the body for
//!      forbidden FMA mnemonics (`vfmadd*`, `fmadd`, `fma213`, `fma231`).
//!   6. If any match → print the offending symbol + line + exit 2 (D-03
//!      escalation: reopen the hand-Rust alternative).
//!      Else → exit 0.
//!
//! Usage:
//!   cargo run -p xtask --bin check-no-fma
//!
//! CI integration: add as a required gate in the `ci.yml` workflow for any
//! PR touching `crates/xcfun-ad/src/ctaylor_rec/**` or `.cargo/config.toml`.
//!
//! Expected output on a clean Phase 1 build:
//!   check-no-fma: emitting asm for xcfun-ad --release ...
//!   check-no-fma: scanning N asm file(s)
//!   check-no-fma: PASS — no FMA mnemonics on ctaylor_mul symbols.
//!
//! Exit codes:
//!   0 — PASS (no FMA found)
//!   1 — build or IO error (bails via anyhow)
//!   2 — FAIL: forbidden mnemonic detected (D-03 escalation)

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const FORBIDDEN_MNEMONICS: &[&str] = &[
    // x86-64 fused multiply-add family (SSE/AVX scalar + packed double)
    "vfmadd132pd", "vfmadd213pd", "vfmadd231pd",
    "vfmadd132sd", "vfmadd213sd", "vfmadd231sd",
    "vfmsub132pd", "vfmsub213pd", "vfmsub231pd",
    "vfmsub132sd", "vfmsub213sd", "vfmsub231sd",
    "vfnmadd132pd", "vfnmadd213pd", "vfnmadd231pd",
    "vfnmadd132sd", "vfnmadd213sd", "vfnmadd231sd",
    "vfnmsub132pd", "vfnmsub213pd", "vfnmsub231pd",
    "vfnmsub132sd", "vfnmsub213sd", "vfnmsub231sd",
    // aarch64 + generic spellings
    "fmadd",
    "fmsub",
    "fnmadd",
    "fnmsub",
    // LLVM-intrinsic-style (belt-and-suspenders — should never appear after
    // lowering, but grep for them anyway)
    "fma213", "fma231",
];

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf();
    Ok(root)
}

fn main() -> Result<()> {
    let root = project_root()?;

    // Step 1: cargo rustc --emit=asm
    //
    // NOTE: we DO NOT clean target/ first; letting cargo decide what is
    // fresh keeps the local dev loop fast. CI runs with a clean workspace.
    println!("check-no-fma: emitting asm for xcfun-ad --release ...");
    let status = Command::new("cargo")
        .current_dir(&root)
        .args([
            "rustc",
            "-p", "xcfun-ad",
            "--release",
            "--lib",
            "--features", "cpu",
            "--",
            "--emit=asm",
        ])
        .status()
        .context("spawning cargo rustc")?;
    if !status.success() {
        bail!(
            "cargo rustc --emit=asm failed with exit {:?}",
            status.code()
        );
    }

    // Step 2: find all xcfun_ad-*.s files under target/release/deps/
    let deps_dir = root.join("target/release/deps");
    let mut asm_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&deps_dir)
        .with_context(|| format!("read_dir {}", deps_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let ext_ok = path.extension().and_then(|s| s.to_str()) == Some("s");
        let stem_ok = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("xcfun_ad"))
            .unwrap_or(false);
        if ext_ok && stem_ok {
            asm_files.push(path);
        }
    }
    if asm_files.is_empty() {
        bail!(
            "no xcfun_ad-*.s files found under {}. Did `cargo rustc --emit=asm` actually run? \
             If asm was already emitted by a previous build, try `cargo clean -p xcfun-ad` first.",
            deps_dir.display()
        );
    }
    println!(
        "check-no-fma: scanning {} asm file(s)",
        asm_files.len()
    );

    // Step 3..5: parse each .s file, find ctaylor_mul symbols, grep bodies
    let mut violations: Vec<String> = Vec::new();
    for path in &asm_files {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        scan_asm_file(
            &contents,
            path.display().to_string().as_str(),
            &mut violations,
        );
    }

    // Step 6: fail or pass
    if !violations.is_empty() {
        eprintln!(
            "\ncheck-no-fma: FAIL — FMA mnemonics found on ctaylor_mul path:"
        );
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!(
            "\nCONTEXT.md D-03 escalation: cubecl-cpu's MLIR lowering fused\n\
             arithmetic into FMA instructions despite\n\
             `-Cllvm-args=-fp-contract=off`. The 1e-12 parity contract\n\
             cannot be trusted. Planning must escalate via\n\
             `PLANNING INCONCLUSIVE` per Phase 1 CONTEXT.md D-03."
        );
        std::process::exit(2);
    }

    println!("check-no-fma: PASS — no FMA mnemonics on ctaylor_mul symbols.");
    Ok(())
}

/// Scan a single asm file. Splits by symbol labels (lines ending in `:`
/// that are not `.` directives or empty), demangles each, and — if the
/// demangled name contains `ctaylor_mul` — greps the symbol's body (up
/// to the next symbol label) for forbidden FMA mnemonics.
fn scan_asm_file(contents: &str, path: &str, violations: &mut Vec<String>) {
    let mut current_sym: Option<String> = None;
    let mut sym_is_ctaylor_mul = false;

    for (lineno, line) in contents.lines().enumerate() {
        let trimmed = line.trim_start();

        // Detect a new symbol label: e.g. `_ZN8xcfun_ad12ctaylor_rec3mul...:`
        if let Some(stripped) = trimmed.strip_suffix(':') {
            if !stripped.starts_with('.') && !stripped.is_empty() {
                let demangled = rustc_demangle::demangle(stripped).to_string();
                sym_is_ctaylor_mul = demangled.contains("ctaylor_mul");
                current_sym = Some(demangled);
                continue;
            }
        }

        if !sym_is_ctaylor_mul {
            continue;
        }

        // Inside a `ctaylor_mul*` symbol — grep for forbidden mnemonics.
        // Mnemonic tokens start after any leading whitespace and end at
        // the first whitespace/comma. We use a conservative substring
        // check: false positives (e.g. mnemonic text inside a comment)
        // still make the test fail, and fixing a false positive is
        // cheaper than missing a real FMA emission.
        for mnemonic in FORBIDDEN_MNEMONICS {
            if line.contains(mnemonic) {
                violations.push(format!(
                    "{}:{} [{}] {}",
                    path,
                    lineno + 1,
                    current_sym.as_deref().unwrap_or("<unknown>"),
                    line.trim(),
                ));
            }
        }
    }
}
