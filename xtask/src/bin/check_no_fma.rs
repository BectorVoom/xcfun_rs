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

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const FORBIDDEN_MNEMONICS: &[&str] = &[
    // x86-64 fused multiply-add family (SSE/AVX scalar + packed double)
    "vfmadd132pd",
    "vfmadd213pd",
    "vfmadd231pd",
    "vfmadd132sd",
    "vfmadd213sd",
    "vfmadd231sd",
    "vfmsub132pd",
    "vfmsub213pd",
    "vfmsub231pd",
    "vfmsub132sd",
    "vfmsub213sd",
    "vfmsub231sd",
    "vfnmadd132pd",
    "vfnmadd213pd",
    "vfnmadd231pd",
    "vfnmadd132sd",
    "vfnmadd213sd",
    "vfnmadd231sd",
    "vfnmsub132pd",
    "vfnmsub213pd",
    "vfnmsub231pd",
    "vfnmsub132sd",
    "vfnmsub213sd",
    "vfnmsub231sd",
    // aarch64 + generic spellings
    "fmadd",
    "fmsub",
    "fnmadd",
    "fnmsub",
    // LLVM-intrinsic-style (belt-and-suspenders — should never appear after
    // lowering, but grep for them anyway)
    "fma213",
    "fma231",
];

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf();
    Ok(root)
}

/// Crate + asm-filename-stem pairs scanned by this gate. `xcfun-eval` is
/// added in Plan 02-03 Wave-1B-1; if it's not in the workspace yet we skip
/// it gracefully with a stdout note (ACC-05 forward-compatible per D-10).
const SCAN_TARGETS: &[(&str, &str, &[&str])] = &[
    (
        "xcfun-ad",
        "xcfun_ad",
        // Legacy Phase-1 target — grep for `ctaylor_mul` anywhere in demangled symbols.
        &["ctaylor_mul"],
    ),
    (
        "xcfun-eval",
        "xcfun_eval",
        // Phase-2 extension: kernel functions are named `xcfun_eval_*_kernel`.
        &["xcfun_eval_"],
    ),
];

fn is_in_workspace(root: &PathBuf, crate_name: &str) -> Result<bool> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .context("spawning cargo metadata")?;
    if !output.status.success() {
        return Ok(false);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    // Cheap substring check — avoids pulling serde_json into check_no_fma.
    Ok(text.contains(&format!("\"name\":\"{}\"", crate_name)))
}

fn main() -> Result<()> {
    let root = project_root()?;
    let mut asm_files: Vec<(PathBuf, Vec<&'static str>)> = Vec::new();

    for (crate_name, asm_stem, needles) in SCAN_TARGETS {
        if !is_in_workspace(&root, crate_name)? {
            println!(
                "check-no-fma: {} not in workspace yet (Plan 02-03 adds it); \
                 skipping this target and continuing with remaining crate(s).",
                crate_name
            );
            continue;
        }

        // Step 1: cargo rustc --emit=asm for this crate. We retain the Phase-1
        // `--features cpu` for xcfun-ad; xcfun-eval is built with its default
        // feature set until the gate is extended further.
        println!(
            "check-no-fma: emitting asm for {} --release ...",
            crate_name
        );
        let mut cargo_args: Vec<&str> = vec!["rustc", "-p", crate_name, "--release", "--lib"];
        if *crate_name == "xcfun-ad" {
            cargo_args.extend_from_slice(&["--features", "cpu"]);
        }
        cargo_args.extend_from_slice(&["--", "--emit=asm"]);
        let status = Command::new("cargo")
            .current_dir(&root)
            .args(&cargo_args)
            .status()
            .context("spawning cargo rustc")?;
        if !status.success() {
            bail!(
                "cargo rustc --emit=asm ({}) failed with exit {:?}",
                crate_name,
                status.code()
            );
        }

        // Step 2: find all <stem>-*.s files under target/release/deps/
        let deps_dir = root.join("target/release/deps");
        let mut found = 0usize;
        for entry in
            fs::read_dir(&deps_dir).with_context(|| format!("read_dir {}", deps_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let ext_ok = path.extension().and_then(|s| s.to_str()) == Some("s");
            let stem_ok = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with(asm_stem))
                .unwrap_or(false);
            if ext_ok && stem_ok {
                asm_files.push((path, needles.to_vec()));
                found += 1;
            }
        }
        if found == 0 {
            bail!(
                "no {}-*.s files found under {}. Did `cargo rustc --emit=asm` actually run?",
                asm_stem,
                deps_dir.display()
            );
        }
    }

    if asm_files.is_empty() {
        println!(
            "check-no-fma: no targets resolved — neither xcfun-ad nor xcfun-eval \
             are in the workspace. This is unexpected in Phase 2+; check\n\
             root Cargo.toml workspace members."
        );
        return Ok(());
    }
    println!("check-no-fma: scanning {} asm file(s)", asm_files.len());

    // Step 3..5: parse each .s file, find symbols matching the per-target
    // needle list, grep bodies.
    let mut violations: Vec<String> = Vec::new();
    for (path, needles) in &asm_files {
        let contents =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        scan_asm_file(
            &contents,
            path.display().to_string().as_str(),
            needles,
            &mut violations,
        );
    }

    // Step 6: fail or pass
    if !violations.is_empty() {
        eprintln!("\ncheck-no-fma: FAIL — FMA mnemonics found on guarded symbols:");
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

    println!("check-no-fma: PASS — no FMA mnemonics on guarded symbols.");
    Ok(())
}

/// Scan a single asm file. Splits by symbol labels (lines ending in `:`
/// that are not `.` directives or empty), demangles each, and — if the
/// demangled name contains any of the supplied `needles` — greps the
/// symbol's body (up to the next symbol label) for forbidden FMA
/// mnemonics. `needles` is the per-target interest list: Phase-1 passes
/// `["ctaylor_mul"]`, Phase-2 extends with `["xcfun_eval_"]` for the
/// xcfun-eval kernel symbols (ACC-05).
fn scan_asm_file(contents: &str, path: &str, needles: &[&str], violations: &mut Vec<String>) {
    let mut current_sym: Option<String> = None;
    let mut sym_of_interest = false;

    for (lineno, line) in contents.lines().enumerate() {
        let trimmed = line.trim_start();

        // Detect a new symbol label: e.g. `_ZN8xcfun_ad12ctaylor_rec3mul...:`
        if let Some(stripped) = trimmed.strip_suffix(':')
            && !stripped.starts_with('.')
            && !stripped.is_empty()
        {
            let demangled = rustc_demangle::demangle(stripped).to_string();
            sym_of_interest = needles.iter().any(|n| demangled.contains(n));
            current_sym = Some(demangled);
            continue;
        }

        if !sym_of_interest {
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
