//! Regenerate `crates/xcfun-core/src/registry/generated/{FUNCTIONAL_DESCRIPTORS,
//! VARS_TABLE, ALIASES}.rs` + matching `.sha256` stamp files from a cc-compiled
//! C++ extractor walking `xcfun-master/src/functionals/*.cpp` + `xcint.cpp`.
//!
//! Workflow:
//!   1. Compile `xtask/assets/regen_registry/extractor.cpp` to an executable
//!      under `target/regen_registry/` using the system C++ compiler
//!      (`$CXX` env var, else `c++`). Mirrors the pattern from
//!      `xtask/src/bin/regen_ad_fixtures.rs`.
//!   2. Run the extractor with `xcfun-master` root as argv[1]; capture JSONL
//!      on stdout.
//!   3. Parse each JSONL line: `{"type":"functional",...}`,
//!      `{"type":"vars_row",...}`, or `{"type":"aliases",...}`.
//!   4. Emit three Rust source files into
//!      `crates/xcfun-core/src/registry/generated/`:
//!        - `FUNCTIONAL_DESCRIPTORS.rs` — a `pub static FUNCTIONAL_DESCRIPTORS:
//!          [FunctionalDescriptor; 78]` literal. Entries not found by the
//!          extractor are emitted as `FunctionalDescriptor::stub(id, name,
//!          Dependency::DENSITY)`; the 8 LDAs with upstream `test_in`/
//!          `test_out` are emitted fully-populated.
//!        - `VARS_TABLE.rs` — a `pub static VARS_TABLE: [VarsRow; 31]` literal
//!          sourced from the xcint_vars table.
//!        - `ALIASES.rs` — a `pub static ALIASES: &[Alias] = &[]` (Phase 2
//!          empty; Phase 4 populates 46 aliases).
//!      Alongside each `.rs` file an `<NAME>.rs.sha256` stamp is written
//!      containing the hex SHA-256 of the Rust source (for `--check` drift
//!      detection).
//!   5. `--check` mode regenerates the Rust sources in memory, hashes them,
//!      and compares against the committed `.sha256` stamps. Exits 2 on
//!      drift (QG-07 gate).
//!
//! Invocation:
//!   - `cargo run -p xtask --bin regen-registry`               (write mode)
//!   - `cargo run -p xtask --bin regen-registry -- --check`    (CI drift gate)

use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------- project + compile helpers ----------

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin regen-registry")?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf();
    Ok(root)
}

fn compile_extractor(xcfun_root: &Path, src: &Path, exe: &Path) -> Result<()> {
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
        .arg(xcfun_root.join("api"))
        .arg("-I")
        .arg(xcfun_root.join("src"))
        .arg("-I")
        .arg(xcfun_root.join("src/functionals"))
        .arg(src)
        .arg("-o")
        .arg(exe)
        .status()
        .with_context(|| format!("failed to spawn C++ compiler ({})", compiler))?;
    if !status.success() {
        bail!(
            "compiling extractor.cpp failed (compiler {}, exit {:?})",
            compiler,
            status.code()
        );
    }
    Ok(())
}

fn run_extractor(exe: &Path, xcfun_root: &Path) -> Result<String> {
    let output = Command::new(exe)
        .arg(xcfun_root)
        .output()
        .with_context(|| format!("failed to spawn extractor at {}", exe.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "extractor exited non-zero ({:?}): {}",
            output.status.code(),
            stderr
        );
    }
    Ok(String::from_utf8(output.stdout).context("extractor stdout not UTF-8")?)
}

// ---------- parsed shapes ----------

#[derive(Debug, Clone)]
struct FunctionalRec {
    id: String,
    short_desc: String,
    long_desc: String,
    depends: u32,
    test_vars: Option<String>,
    test_mode: Option<String>,
    test_order: Option<u32>,
    test_threshold: Option<f64>,
    test_in: Option<Vec<f64>>,
    test_out: Option<Vec<f64>>,
}

#[derive(Debug, Clone)]
struct VarsRowRec {
    symbol: String,
    len: u8,
    provides: u32,
}

fn parse_jsonl(jsonl: &str) -> Result<(Vec<FunctionalRec>, Vec<VarsRowRec>)> {
    let mut functionals = Vec::new();
    let mut vars_rows = Vec::new();
    for (i, line) in jsonl.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line)
            .with_context(|| format!("parse JSONL line {}: {}", i + 1, line))?;
        let ty = v
            .get("type")
            .and_then(|t| t.as_str())
            .context("JSONL line missing `type` field")?;
        match ty {
            "functional" => {
                functionals.push(FunctionalRec {
                    id: v["id"].as_str().unwrap_or("").to_string(),
                    short_desc: v["short_desc"].as_str().unwrap_or("").to_string(),
                    long_desc: v["long_desc"].as_str().unwrap_or("").to_string(),
                    depends: v["depends"].as_u64().unwrap_or(0) as u32,
                    test_vars: v["test_vars"].as_str().map(|s| s.to_string()),
                    test_mode: v["test_mode"].as_str().map(|s| s.to_string()),
                    test_order: v["test_order"].as_u64().map(|n| n as u32),
                    test_threshold: v["test_threshold"].as_f64(),
                    test_in: v["test_in"].as_array().map(|a| {
                        a.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>()
                    }),
                    test_out: v["test_out"].as_array().map(|a| {
                        a.iter().filter_map(|x| x.as_f64()).collect::<Vec<_>>()
                    }),
                });
            }
            "vars_row" => {
                vars_rows.push(VarsRowRec {
                    symbol: v["symbol"].as_str().unwrap_or("").to_string(),
                    len: v["len"].as_u64().unwrap_or(0) as u8,
                    provides: v["provides"].as_u64().unwrap_or(0) as u32,
                });
            }
            "aliases" => { /* Phase 2: ignore — we ship empty. */ }
            other => bail!("unexpected JSONL type {:?}", other),
        }
    }
    Ok((functionals, vars_rows))
}

// ---------- FunctionalId order (MUST match crates/xcfun-core/src/functional_id.rs) ----------

/// Ordered list of (enum variant, XC_* identifier) pairs in FunctionalId
/// discriminant order 0..78. Keep in sync with
/// `crates/xcfun-core/src/functional_id.rs`.
const FUNCTIONAL_IDS: &[&str] = &[
    "XC_SLATERX",       // 0
    "XC_PW86X",         // 1
    "XC_VWN3C",         // 2
    "XC_VWN5C",         // 3
    "XC_PBEC",          // 4
    "XC_PBEX",          // 5
    "XC_BECKEX",        // 6
    "XC_BECKECORRX",    // 7
    "XC_BECKESRX",      // 8
    "XC_BECKECAMX",     // 9
    "XC_BRX",           // 10
    "XC_BRC",           // 11
    "XC_BRXC",          // 12
    "XC_LDAERFX",       // 13
    "XC_LDAERFC",       // 14
    "XC_LDAERFC_JT",    // 15
    "XC_LYPC",          // 16
    "XC_OPTX",          // 17
    "XC_OPTXCORR",      // 18
    "XC_REVPBEX",       // 19
    "XC_RPBEX",         // 20
    "XC_SPBEC",         // 21
    "XC_VWN_PBEC",      // 22
    "XC_KTX",           // 23
    "XC_TFK",           // 24
    "XC_TW",            // 25
    "XC_PW91X",         // 26
    "XC_PW91K",         // 27
    "XC_PW92C",         // 28
    "XC_M05X",          // 29
    "XC_M05X2X",        // 30
    "XC_M06X",          // 31
    "XC_M06X2X",        // 32
    "XC_M06LX",         // 33
    "XC_M06HFX",        // 34
    "XC_M05X2C",        // 35
    "XC_M05C",          // 36
    "XC_M06C",          // 37
    "XC_M06HFC",        // 38
    "XC_M06LC",         // 39
    "XC_M06X2C",        // 40
    "XC_TPSSC",         // 41
    "XC_TPSSX",         // 42
    "XC_REVTPSSC",      // 43
    "XC_REVTPSSX",      // 44
    "XC_SCANC",         // 45
    "XC_SCANX",         // 46
    "XC_RSCANC",        // 47
    "XC_RSCANX",        // 48
    "XC_RPPSCANC",      // 49
    "XC_RPPSCANX",      // 50
    "XC_R2SCANC",       // 51
    "XC_R2SCANX",       // 52
    "XC_R4SCANC",       // 53
    "XC_R4SCANX",       // 54
    "XC_PZ81C",         // 55
    "XC_P86C",          // 56
    "XC_P86CORRC",      // 57
    "XC_BTK",           // 58
    "XC_VWK",           // 59
    "XC_B97X",          // 60
    "XC_B97C",          // 61
    "XC_B97_1X",        // 62
    "XC_B97_1C",        // 63
    "XC_B97_2X",        // 64
    "XC_B97_2C",        // 65
    "XC_CSC",           // 66
    "XC_APBEC",         // 67
    "XC_APBEX",         // 68
    "XC_ZVPBESOLC",     // 69
    "XC_BLOCX",         // 70
    "XC_PBEINTC",       // 71
    "XC_PBEINTX",       // 72
    "XC_PBELOCC",       // 73
    "XC_PBESOLX",       // 74
    "XC_TPSSLOCC",      // 75
    "XC_ZVPBEINTC",     // 76
    "XC_PW91C",         // 77
];

// ---------- Rust source emission ----------

/// Render a `Dependency::FLAG | Dependency::FLAG` expression from a bitmask.
fn render_dependency(bits: u32) -> String {
    let mut parts = Vec::new();
    if bits & 1 != 0 { parts.push("Dependency::DENSITY"); }
    if bits & 2 != 0 { parts.push("Dependency::GRADIENT"); }
    if bits & 4 != 0 { parts.push("Dependency::LAPLACIAN"); }
    if bits & 8 != 0 { parts.push("Dependency::KINETIC"); }
    if bits & 16 != 0 { parts.push("Dependency::JP"); }
    if parts.is_empty() {
        "Dependency::empty()".to_string()
    } else if parts.len() == 1 {
        parts[0].to_string()
    } else {
        parts.join(".union(")
            .chars()
            .collect::<String>()
            + &")".repeat(parts.len() - 1)
    }
}

/// Translate `XC_A_B` -> `Vars::A_B`, `XC_A_B_GAA_GAB_GBB` -> `Vars::A_B_GAA_GAB_GBB`.
fn render_vars(xc_ident: &str) -> String {
    let trimmed = xc_ident.strip_prefix("XC_").unwrap_or(xc_ident);
    format!("Vars::{}", trimmed)
}

/// Translate `XC_PARTIAL_DERIVATIVES` / `XC_POTENTIAL` / `XC_CONTRACTED` ->
/// `Mode::PartialDerivatives` / `Mode::Potential` / `Mode::Contracted`.
fn render_mode(xc_ident: &str) -> String {
    match xc_ident {
        "XC_PARTIAL_DERIVATIVES" => "Mode::PartialDerivatives".to_string(),
        "XC_POTENTIAL" => "Mode::Potential".to_string(),
        "XC_CONTRACTED" => "Mode::Contracted".to_string(),
        other => format!("/* unknown mode {} */ Mode::Unset", other),
    }
}

fn rust_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

fn emit_functional_descriptors_rs(functionals: &[FunctionalRec]) -> String {
    // Build lookup map id -> FunctionalRec.
    let map: BTreeMap<&str, &FunctionalRec> =
        functionals.iter().map(|f| (f.id.as_str(), f)).collect();

    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.\n\
         // Source: xcfun-master/src/functionals/*.cpp + xcfun-master/src/xcint.cpp\n\
         // Regenerate with: cargo run -p xtask --bin regen-registry\n\
         // Drift check: cargo run -p xtask --bin regen-registry -- --check (exit 2 on drift)\n\n",
    );
    // NOTE: no `use` imports here — the three generated files are included
    // into a single module via `include!`; the parent (`registry/mod.rs`)
    // imports once and every `include!`-d file sees the same scope.

    // FunctionalDescriptor struct definition.
    out.push_str(
        "/// Registry row describing one of the 78 functionals.\n\
         ///\n\
         /// See PLAN 02-02 Wave-1A-2 for provenance. The `test_in`/`test_out`/\n\
         /// `test_threshold` slots are populated for the 8 LDA functionals that ship\n\
         /// upstream reference values; the other 70 entries are stubs with\n\
         /// `test_in = None` (downstream plans fill them from the C++ reference).\n\
         #[derive(Debug, Clone, Copy)]\n\
         pub struct FunctionalDescriptor {\n\
         \x20   pub id: FunctionalId,\n\
         \x20   pub name: &'static str,\n\
         \x20   pub short_description: &'static str,\n\
         \x20   pub long_description: &'static str,\n\
         \x20   pub depends: Dependency,\n\
         \x20   pub test_vars: Option<Vars>,\n\
         \x20   pub test_mode: Option<Mode>,\n\
         \x20   pub test_order: Option<u32>,\n\
         \x20   pub test_threshold: Option<f64>,\n\
         \x20   pub test_in: Option<&'static [f64]>,\n\
         \x20   pub test_out: Option<&'static [f64]>,\n\
         \x20   pub test_outlen: u32,\n\
         }\n\n",
    );
    out.push_str(
        "impl FunctionalDescriptor {\n\
         \x20   /// Construct a stub entry for a functional whose body / metadata\n\
         \x20   /// is not yet wired up in the extractor. Used for the 70 non-LDA\n\
         \x20   /// functionals in Phase 2; Phase 4+ replaces these with fully\n\
         \x20   /// populated rows.\n\
         \x20   pub const fn stub(id: FunctionalId, name: &'static str, depends: Dependency) -> Self {\n\
         \x20       Self {\n\
         \x20           id,\n\
         \x20           name,\n\
         \x20           short_description: \"\",\n\
         \x20           long_description: \"\",\n\
         \x20           depends,\n\
         \x20           test_vars: None,\n\
         \x20           test_mode: None,\n\
         \x20           test_order: None,\n\
         \x20           test_threshold: None,\n\
         \x20           test_in: None,\n\
         \x20           test_out: None,\n\
         \x20           test_outlen: 0,\n\
         \x20       }\n\
         \x20   }\n\
         }\n\n",
    );

    // Per-LDA `static <ID>_TEST_IN: [f64; N] = [...];` rodata arrays.
    for id in FUNCTIONAL_IDS {
        if let Some(rec) = map.get(id) {
            if let (Some(ti), Some(to)) = (rec.test_in.as_ref(), rec.test_out.as_ref()) {
                out.push_str(&format!("static {}_TEST_IN: [f64; {}] = [", id, ti.len()));
                for (i, v) in ti.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&format_f64(*v));
                }
                out.push_str("];\n");
                out.push_str(&format!("static {}_TEST_OUT: [f64; {}] = [", id, to.len()));
                for (i, v) in to.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&format_f64(*v));
                }
                out.push_str("];\n");
            }
        }
    }
    out.push('\n');

    // The array literal — 78 entries in FunctionalId order.
    out.push_str("/// 78-entry registry table indexed by `FunctionalId as usize`.\n");
    out.push_str(&format!(
        "pub static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; {}] = [\n",
        FUNCTIONAL_IDS.len()
    ));
    for id in FUNCTIONAL_IDS {
        if let Some(rec) = map.get(id) {
            if rec.test_in.is_some() && rec.test_out.is_some() {
                // Fully-populated entry.
                let ti = rec.test_in.as_ref().unwrap();
                let to = rec.test_out.as_ref().unwrap();
                out.push_str("    FunctionalDescriptor {\n");
                out.push_str(&format!("        id: FunctionalId::{},\n", id));
                out.push_str(&format!("        name: \"{}\",\n", id));
                out.push_str(&format!("        short_description: \"{}\",\n", rust_escape(&rec.short_desc)));
                out.push_str(&format!("        long_description: \"{}\",\n", rust_escape(&rec.long_desc)));
                out.push_str(&format!("        depends: {},\n", render_dependency(rec.depends)));
                out.push_str(&format!(
                    "        test_vars: Some({}),\n",
                    rec.test_vars.as_deref().map(render_vars).unwrap_or_else(|| "Vars::A_B".to_string())
                ));
                out.push_str(&format!(
                    "        test_mode: Some({}),\n",
                    rec.test_mode.as_deref().map(render_mode).unwrap_or_else(|| "Mode::PartialDerivatives".to_string())
                ));
                out.push_str(&format!("        test_order: Some({}),\n", rec.test_order.unwrap_or(0)));
                out.push_str(&format!(
                    "        test_threshold: Some({}),\n",
                    format_f64(rec.test_threshold.unwrap_or(0.0))
                ));
                out.push_str(&format!("        test_in: Some(&{}_TEST_IN),\n", id));
                out.push_str(&format!("        test_out: Some(&{}_TEST_OUT),\n", id));
                out.push_str(&format!("        test_outlen: {},\n", to.len()));
                // Suppress unused for ti since we reference via the static.
                let _ = ti;
                out.push_str("    },\n");
            } else {
                // Extractor saw the macro but no test data (VWN3C, LDAERFC_JT, TW, VWK).
                // Emit a stub that still carries the correct depends bitmask.
                out.push_str(&format!(
                    "    FunctionalDescriptor::stub(FunctionalId::{}, \"{}\", {}),\n",
                    id, id, render_dependency(rec.depends)
                ));
            }
        } else {
            // Not found by extractor — emit generic stub.
            out.push_str(&format!(
                "    FunctionalDescriptor::stub(FunctionalId::{}, \"{}\", Dependency::DENSITY),\n",
                id, id
            ));
        }
    }
    out.push_str("];\n");
    out
}

fn emit_vars_table_rs(rows: &[VarsRowRec]) -> String {
    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.\n\
         // Source: xcfun-master/src/xcint.cpp (xcint_vars table)\n\n",
    );
    // NOTE: no `use` imports — shared module scope via `include!` in mod.rs.
    out.push_str(
        "/// Registry row describing one of the 31 `Vars` variants. Ordering matches\n\
         /// `Vars as u32`.\n\
         #[derive(Debug, Clone, Copy)]\n\
         #[repr(C)]\n\
         pub struct VarsRow {\n\
         \x20   pub symbol: &'static str,\n\
         \x20   pub len: u8,\n\
         \x20   pub provides: Dependency,\n\
         }\n\n",
    );
    out.push_str(&format!(
        "pub static VARS_TABLE: [VarsRow; {}] = [\n",
        rows.len()
    ));
    for r in rows {
        out.push_str(&format!(
            "    VarsRow {{ symbol: \"{}\", len: {}, provides: {} }},\n",
            r.symbol,
            r.len,
            render_dependency(r.provides)
        ));
    }
    out.push_str("];\n");
    out
}

// ---------- c_stubs.cpp emission (Plan 02-06 Wave-2-1) ----------

/// Phase 2 LDA functional IDs (those WITH a real Rust kernel in xcfun-eval).
///
/// The 11 LDA .cpp files in `xcfun-master/src/functionals/` provide their own
/// `FUNCTIONAL(XC_*)` instantiation (and thus `fundat_db<XC_*>::d`
/// specialisation), so they do NOT need a stub in `c_stubs.cpp`. The remaining
/// 67 functional IDs do — without a stub for each, `xcint.cpp`'s template
/// recursion through `XC_NR_FUNCTIONALS` fails to link.
const PHASE2_LDA_IDS: &[&str] = &[
    "XC_SLATERX",
    "XC_VWN3C",
    "XC_VWN5C",
    "XC_PW92C",
    "XC_PZ81C",
    "XC_LDAERFX",
    "XC_LDAERFC",
    "XC_LDAERFC_JT",
    "XC_TFK",
    "XC_TW",
    "XC_VWK",
];

/// Render a depends bitmask as a C++ `XC_DENSITY|XC_GRADIENT|...` expression.
fn format_depends_cpp(bits: u32) -> String {
    let mut parts: Vec<&'static str> = Vec::new();
    if bits & 1 != 0 {
        parts.push("XC_DENSITY");
    }
    if bits & 2 != 0 {
        parts.push("XC_GRADIENT");
    }
    if bits & 4 != 0 {
        parts.push("XC_LAPLACIAN");
    }
    if bits & 8 != 0 {
        parts.push("XC_KINETIC");
    }
    if bits & 16 != 0 {
        parts.push("XC_JP");
    }
    if parts.is_empty() {
        "XC_DENSITY".to_string() // Safe default — bit-0 is always set for the reference build.
    } else {
        parts.join("|")
    }
}

/// Emit `validation/c_stubs.cpp` with stub `FUNCTIONAL(XC_*)` macros for every
/// non-LDA functional ID. Required for `xcint.cpp`'s template recursion to link
/// in the Plan 02-06 cc-build (Wave-2-2).
///
/// Returns (source_text, stub_count).
fn emit_c_stubs_cpp(functionals: &[FunctionalRec]) -> (String, usize) {
    // Build lookup: id -> depends bitmask (from extractor output).
    let depends_map: BTreeMap<&str, u32> = functionals
        .iter()
        .map(|f| (f.id.as_str(), f.depends))
        .collect();

    let mut out = String::new();
    out.push_str(
        "// validation/c_stubs.cpp — AUTO-GENERATED by xtask regen-registry. DO NOT EDIT.\n",
    );
    out.push_str(
        "// Stubs for Phase 2 cc-compile — every non-LDA functional ID needs a fundat_db\n",
    );
    out.push_str(
        "// specialisation or xcint.cpp template recursion fails to link.\n",
    );
    out.push_str("//\n");
    out.push_str(
        "// Phase 3+ extends this file by re-running regen-registry; LDA IDs already get\n",
    );
    out.push_str(
        "// the real ENERGY_FUNCTION via their respective .cpp files compiled by build.rs.\n\n",
    );
    out.push_str("#include \"functional.hpp\"\n\n");
    out.push_str(
        "template <typename num> static num stub_unimpl(const densvars<num> &) { return num(0); }\n\n",
    );

    let mut count = 0usize;
    for id in FUNCTIONAL_IDS {
        if PHASE2_LDA_IDS.contains(id) {
            continue; // LDA IDs get the real ENERGY_FUNCTION from their .cpp file.
        }
        // Depends bitmask: use the extractor-reported value when available; else
        // fall back to XC_DENSITY (safe default — bit 0 is always set).
        let bits = depends_map.get(id).copied().unwrap_or(1);
        let depends_str = format_depends_cpp(bits);
        out.push_str(&format!(
            "FUNCTIONAL({}) = {{\"stub\", \"stub\", {}, ENERGY_FUNCTION(stub_unimpl)}};\n",
            id, depends_str
        ));
        count += 1;
    }
    (out, count)
}

/// One parsed alias entry from `aliases.cpp`.
#[derive(Debug, Clone)]
struct AliasRec {
    name: String,
    description: String,
    /// (term_name, weight) pairs in declaration order.
    terms: Vec<(String, f64)>,
}

/// One parsed parameter entry from `common_parameters.cpp`.
#[derive(Debug, Clone)]
struct ParameterRec {
    /// XC_ identifier as written, e.g. "XC_RANGESEP_MU".
    xc_ident: String,
    description: String,
    default: f64,
}

/// Strip C++ `// ...` line comments and `/* ... */` block comments; helps
/// the regex-free parsers below avoid pattern leakage from documentation.
fn strip_cpp_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0usize;
    let mut in_str = false;
    let mut in_chr = false;
    let mut esc = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            out.push(b as char);
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_chr {
            out.push(b as char);
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'\'' {
                in_chr = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'\'' {
            in_chr = true;
            out.push('\'');
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            let nb = bytes[i + 1];
            if nb == b'/' {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if nb == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Parse a C-style `"..."` string literal starting at `pos` (the opening quote).
/// Returns (decoded_string, pos_after_closing_quote).
fn parse_c_string(src: &str, pos: usize) -> Result<(String, usize)> {
    let bytes = src.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'"' {
        bail!("expected opening '\"' at byte {}", pos);
    }
    let mut i = pos + 1;
    let mut out = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            let nb = bytes[i + 1];
            let ch = match nb {
                b'n' => '\n',
                b't' => '\t',
                b'r' => '\r',
                b'\\' => '\\',
                b'"' => '"',
                b'\'' => '\'',
                b'0' => '\0',
                other => other as char,
            };
            out.push(ch);
            i += 2;
        } else if b == b'"' {
            return Ok((out, i + 1));
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    bail!("unterminated string literal starting at byte {}", pos)
}

/// Parse an f64 literal token starting at `pos` (skipping leading whitespace
/// already done by the caller). Accepts `1.0`, `0.576727`, `-0.006`, `1.`,
/// `1e-3`, etc. Returns (value, pos_after_token).
fn parse_f64_token(src: &str, mut pos: usize) -> Result<(f64, usize)> {
    let bytes = src.as_bytes();
    let start = pos;
    if pos < bytes.len() && (bytes[pos] == b'-' || bytes[pos] == b'+') {
        pos += 1;
    }
    while pos < bytes.len()
        && (bytes[pos].is_ascii_digit()
            || bytes[pos] == b'.'
            || bytes[pos] == b'e'
            || bytes[pos] == b'E'
            || ((bytes[pos] == b'+' || bytes[pos] == b'-')
                && (bytes[pos - 1] == b'e' || bytes[pos - 1] == b'E')))
    {
        pos += 1;
    }
    let token = &src[start..pos];
    let trimmed = token.trim_end_matches(['f', 'F', 'l', 'L']);
    let value: f64 = trimmed
        .parse()
        .with_context(|| format!("failed to parse f64 literal '{}'", token))?;
    Ok((value, pos))
}

/// Skip ASCII whitespace starting at `pos`. Returns the new position.
fn skip_ws(src: &str, mut pos: usize) -> usize {
    let bytes = src.as_bytes();
    while pos < bytes.len() && (bytes[pos] as char).is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

/// Locate `aliases_array[...] = {` in the cleaned source and parse every
/// `{ "name", "description", { {"term", w}, ... } }` row up to the closing
/// `}` of the outer brace block.
fn parse_aliases_cpp(cleaned: &str) -> Result<Vec<AliasRec>> {
    let bytes = cleaned.as_bytes();
    // Locate `aliases_array` then advance past the first `{` (start of array).
    let arr_pos = cleaned
        .find("aliases_array")
        .context("aliases.cpp does not declare `aliases_array`")?;
    let mut i = arr_pos + "aliases_array".len();
    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    if i >= bytes.len() {
        bail!("aliases_array opening brace not found");
    }
    i += 1; // skip outer '{'

    let mut out = Vec::new();
    loop {
        i = skip_ws(cleaned, i);
        if i >= bytes.len() {
            bail!("unexpected EOF inside aliases_array");
        }
        if bytes[i] == b'}' {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        if bytes[i] != b'{' {
            bail!(
                "expected '{{' or '}}' at offset {}, got '{}'",
                i,
                bytes[i] as char
            );
        }
        i += 1; // enter row '{'
        i = skip_ws(cleaned, i);

        // name
        let (name, after) = parse_c_string(cleaned, i)?;
        i = after;
        i = skip_ws(cleaned, i);
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
            i = skip_ws(cleaned, i);
        } else {
            bail!("expected ',' after alias name '{}'", name);
        }

        // description
        let (description, after) = parse_c_string(cleaned, i)?;
        i = after;
        i = skip_ws(cleaned, i);
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
            i = skip_ws(cleaned, i);
        } else {
            bail!("expected ',' after alias description for '{}'", name);
        }

        // terms list — opens '{', contains '{...},...', closes '}'.
        if i >= bytes.len() || bytes[i] != b'{' {
            bail!("expected '{{' before terms list of alias '{}'", name);
        }
        i += 1;

        let mut terms = Vec::new();
        loop {
            i = skip_ws(cleaned, i);
            if i >= bytes.len() {
                bail!("unexpected EOF in terms list of alias '{}'", name);
            }
            if bytes[i] == b'}' {
                i += 1;
                break;
            }
            if bytes[i] == b',' {
                i += 1;
                continue;
            }
            if bytes[i] != b'{' {
                bail!(
                    "expected '{{' or '}}' inside terms list of '{}', got '{}'",
                    name,
                    bytes[i] as char
                );
            }
            i += 1; // enter term '{'
            i = skip_ws(cleaned, i);
            let (tname, after) = parse_c_string(cleaned, i)?;
            i = after;
            i = skip_ws(cleaned, i);
            if i >= bytes.len() || bytes[i] != b',' {
                bail!("expected ',' after term name '{}' in alias '{}'", tname, name);
            }
            i += 1;
            i = skip_ws(cleaned, i);
            let (weight, after) = parse_f64_token(cleaned, i)?;
            i = after;
            i = skip_ws(cleaned, i);
            if i >= bytes.len() || bytes[i] != b'}' {
                bail!(
                    "expected '}}' after weight {} in term '{}' of alias '{}'",
                    weight,
                    tname,
                    name
                );
            }
            i += 1;
            terms.push((tname, weight));
        }
        i = skip_ws(cleaned, i);
        // Optional trailing ',' before next row.
        if i < bytes.len() && bytes[i] == b',' {
            // consume in next loop iteration
        }
        // End of row brace
        // We already consumed the inner '}' of terms; now the outer '}' of the row.
        if i >= bytes.len() || bytes[i] != b'}' {
            bail!("expected '}}' at end of row for alias '{}'", name);
        }
        i += 1;

        out.push(AliasRec {
            name,
            description,
            terms,
        });
    }
    Ok(out)
}

/// Parse `common_parameters.cpp` for `PARAMETER(XC_FOO) = {"description", default};`
/// declarations. Order of returned entries matches source order, which equals
/// `list_of_functionals.hpp:99-104` discriminant order (78..=81).
fn parse_common_parameters_cpp(cleaned: &str) -> Result<Vec<ParameterRec>> {
    let bytes = cleaned.as_bytes();
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = cleaned[search_from..].find("PARAMETER(") {
        let mut i = search_from + rel + "PARAMETER(".len();
        // Identifier up to ')'.
        let id_start = i;
        while i < bytes.len() && bytes[i] != b')' {
            i += 1;
        }
        if i >= bytes.len() {
            bail!("unterminated PARAMETER() at offset {}", id_start);
        }
        let xc_ident = cleaned[id_start..i].trim().to_string();
        i += 1; // skip ')'
        i = skip_ws(cleaned, i);
        if i >= bytes.len() || bytes[i] != b'=' {
            bail!("expected '=' after PARAMETER({})", xc_ident);
        }
        i += 1;
        i = skip_ws(cleaned, i);
        if i >= bytes.len() || bytes[i] != b'{' {
            bail!(
                "expected '{{' to open PARAMETER({}) initializer",
                xc_ident
            );
        }
        i += 1;
        i = skip_ws(cleaned, i);
        let (description, after) = parse_c_string(cleaned, i)?;
        i = after;
        i = skip_ws(cleaned, i);
        if i >= bytes.len() || bytes[i] != b',' {
            bail!(
                "expected ',' after description in PARAMETER({})",
                xc_ident
            );
        }
        i += 1;
        i = skip_ws(cleaned, i);
        let (default, after) = parse_f64_token(cleaned, i)?;
        i = after;
        i = skip_ws(cleaned, i);
        if i >= bytes.len() || bytes[i] != b'}' {
            bail!(
                "expected '}}' to close PARAMETER({}) initializer",
                xc_ident
            );
        }
        i += 1;

        out.push(ParameterRec {
            xc_ident,
            description,
            default,
        });
        search_from = i;
    }
    if out.is_empty() {
        bail!("common_parameters.cpp parsed no PARAMETER() entries");
    }
    Ok(out)
}

/// Map an XC_ parameter identifier to its `ParameterId::XC_*` enum form
/// (matches the discriminant order in `enums.rs`).
fn parameter_id_variant(xc_ident: &str) -> Result<&'static str> {
    match xc_ident {
        "XC_RANGESEP_MU" => Ok("ParameterId::XC_RANGESEP_MU"),
        "XC_EXX" => Ok("ParameterId::XC_EXX"),
        "XC_CAM_ALPHA" => Ok("ParameterId::XC_CAM_ALPHA"),
        "XC_CAM_BETA" => Ok("ParameterId::XC_CAM_BETA"),
        other => bail!("unknown parameter identifier {}", other),
    }
}

fn emit_aliases_rs(aliases: &[AliasRec]) -> String {
    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.\n\
         // Source: xcfun-master/src/functionals/aliases.cpp:17-138 (46 alias entries).\n\
         // Regenerate with: cargo run -p xtask --bin regen-registry\n\
         // Drift check:    cargo run -p xtask --bin regen-registry -- --check (exit 2 on drift)\n\
         //\n\
         // Phase 4 D-04 — populated. Each `Alias` row corresponds to one entry in the\n\
         // C++ `aliases_array[]` table. `name` and `description` mirror the C++ string\n\
         // literals byte-for-byte; `components` mirrors the `terms[MAX_ALIAS_TERMS]`\n\
         // list with the trailing null-name sentinels truncated.\n\n",
    );
    out.push_str(
        "/// Registry row describing one alias functional (e.g. \"BLYP\" ->\n\
         /// 1.0*BECKEX + 1.0*LYPC). Resolved by `Functional::set` via case-insensitive\n\
         /// lookup and recursive `set(term_name, value * term_weight)` per\n\
         /// `XCFunctional.cpp:389-401`.\n\
         #[derive(Debug, Clone, Copy)]\n\
         pub struct Alias {\n\
         \x20   pub name: &'static str,\n\
         \x20   pub description: &'static str,\n\
         \x20   pub components: &'static [(&'static str, f64)],\n\
         }\n\n",
    );
    out.push_str("pub static ALIASES: &[Alias] = &[\n");
    for a in aliases {
        out.push_str("    Alias {\n");
        out.push_str(&format!("        name: \"{}\",\n", rust_escape(&a.name)));
        out.push_str(&format!(
            "        description: \"{}\",\n",
            rust_escape(&a.description)
        ));
        out.push_str("        components: &[");
        for (i, (tname, w)) in a.terms.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("(\"{}\", {})", rust_escape(tname), format_f64(*w)));
        }
        out.push_str("],\n");
        out.push_str("    },\n");
    }
    out.push_str("];\n");
    out
}

fn emit_parameters_rs(params: &[ParameterRec]) -> Result<String> {
    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.\n\
         // Source: xcfun-master/src/functionals/common_parameters.cpp:17-29\n\
         //       + xcfun-master/src/functionals/list_of_functionals.hpp:99-104\n\
         // Regenerate with: cargo run -p xtask --bin regen-registry\n\
         // Drift check:    cargo run -p xtask --bin regen-registry -- --check (exit 2 on drift)\n\n",
    );
    out.push_str(
        "/// Registry row describing one common parameter (range-separation /\n\
         /// CAM coefficients). Lookup name matches the C++ symbol with the\n\
         /// `XC_` prefix stripped (`xcint.cpp:86`).\n\
         ///\n\
         /// Phase 4 D-05 — populated with the 4 entries declared in\n\
         /// `common_parameters.cpp` in `list_of_functionals.hpp` discriminant order\n\
         /// (78..=81).\n\
         #[derive(Debug, Clone, Copy)]\n\
         pub struct ParameterEntry {\n\
         \x20   pub id: ParameterId,\n\
         \x20   /// Lookup name (XC_ prefix stripped, matches `pardat_db<P>::d.name`).\n\
         \x20   pub name: &'static str,\n\
         \x20   /// Documentation string from the `PARAMETER(...)` macro payload.\n\
         \x20   pub description: &'static str,\n\
         \x20   /// Default value seeded into `Functional::settings[id as usize]` by\n\
         \x20   /// `Functional::new()`.\n\
         \x20   pub default: f64,\n\
         }\n\n",
    );
    out.push_str(&format!(
        "pub static PARAMETERS: [ParameterEntry; {}] = [\n",
        params.len()
    ));
    for p in params {
        let variant = parameter_id_variant(&p.xc_ident)?;
        let bare = p
            .xc_ident
            .strip_prefix("XC_")
            .ok_or_else(|| anyhow::anyhow!("parameter ident missing XC_ prefix: {}", p.xc_ident))?;
        out.push_str("    ParameterEntry {\n");
        out.push_str(&format!("        id: {},\n", variant));
        out.push_str(&format!("        name: \"{}\",\n", rust_escape(bare)));
        out.push_str(&format!(
            "        description: \"{}\",\n",
            rust_escape(&p.description)
        ));
        out.push_str(&format!("        default: {},\n", format_f64(p.default)));
        out.push_str("    },\n");
    }
    out.push_str("];\n");
    Ok(out)
}

fn format_f64(v: f64) -> String {
    // Use `.17g` round-trip representation, falling back to decimal for
    // canonical readability. `{:e}` gives e.g. `1e-7` for small values.
    if v == 0.0 {
        return "0.0".to_string();
    }
    // Round-trip via {:e} then ensure it's parseable back.
    let s = format!("{:e}", v);
    // Ensure there's a decimal point so `1e0` / `2e0` stays f64.
    if !s.contains('.') && !s.contains('e') {
        return format!("{}.0", s);
    }
    s
}

// ---------- SHA-256 stamp helpers ----------

fn sha256_hex(contents: &str) -> String {
    format!("{:x}", Sha256::digest(contents.as_bytes()))
}

fn write_with_sha256_stamp(path: &Path, contents: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, contents)?;
    let stamp = format!("{}.sha256", path.display());
    fs::write(&stamp, sha256_hex(contents) + "\n")?;
    Ok(())
}

// ---------- main ----------

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check_only = args.iter().any(|a| a == "--check");

    let root = project_root()?;
    let xcfun_root = root.join("xcfun-master");
    let asset_src = root.join("xtask/assets/regen_registry/extractor.cpp");
    let target_dir = root.join("target/regen_registry");
    fs::create_dir_all(&target_dir)?;
    let exe = target_dir.join(if cfg!(windows) { "extractor.exe" } else { "extractor" });

    anyhow::ensure!(
        asset_src.exists(),
        "extractor.cpp not found at {}",
        asset_src.display()
    );

    eprintln!("[regen-registry] compiling extractor -> {}", exe.display());
    compile_extractor(&xcfun_root, &asset_src, &exe)?;

    eprintln!("[regen-registry] running extractor against {}", xcfun_root.display());
    let jsonl = run_extractor(&exe, &xcfun_root)?;
    let (functionals, vars_rows) = parse_jsonl(&jsonl)?;

    anyhow::ensure!(
        vars_rows.len() == 31,
        "expected 31 vars rows, extractor produced {}",
        vars_rows.len()
    );

    // Plan 04-04 D-04 — parse aliases.cpp for the 46-entry alias registry.
    let aliases_src_path = xcfun_root.join("src/functionals/aliases.cpp");
    let aliases_raw = fs::read_to_string(&aliases_src_path)
        .with_context(|| format!("read {}", aliases_src_path.display()))?;
    let aliases_clean = strip_cpp_comments(&aliases_raw);
    let aliases_records = parse_aliases_cpp(&aliases_clean)
        .with_context(|| format!("parsing {}", aliases_src_path.display()))?;
    anyhow::ensure!(
        aliases_records.len() == 46,
        "expected 46 aliases in aliases.cpp, parsed {}",
        aliases_records.len()
    );

    // Plan 04-04 D-05 — parse common_parameters.cpp for the 4-entry table.
    let params_src_path = xcfun_root.join("src/functionals/common_parameters.cpp");
    let params_raw = fs::read_to_string(&params_src_path)
        .with_context(|| format!("read {}", params_src_path.display()))?;
    let params_clean = strip_cpp_comments(&params_raw);
    let params_records = parse_common_parameters_cpp(&params_clean)
        .with_context(|| format!("parsing {}", params_src_path.display()))?;
    anyhow::ensure!(
        params_records.len() == 4,
        "expected 4 PARAMETER(...) entries, parsed {}",
        params_records.len()
    );

    let descriptors_src = emit_functional_descriptors_rs(&functionals);
    let vars_src = emit_vars_table_rs(&vars_rows);
    let aliases_src = emit_aliases_rs(&aliases_records);
    let parameters_src = emit_parameters_rs(&params_records)?;
    let (c_stubs_src, c_stubs_count) = emit_c_stubs_cpp(&functionals);

    let generated_dir = root.join("crates/xcfun-core/src/registry/generated");
    let validation_dir = root.join("validation");
    let c_stubs_path = validation_dir.join("c_stubs.cpp");

    if check_only {
        let mut drifted = false;
        for (name, src) in [
            ("FUNCTIONAL_DESCRIPTORS.rs", &descriptors_src),
            ("VARS_TABLE.rs", &vars_src),
            ("ALIASES.rs", &aliases_src),
            ("parameters.rs", &parameters_src),
        ] {
            let actual = sha256_hex(src);
            let stamp_path = generated_dir.join(format!("{}.sha256", name));
            if !stamp_path.exists() {
                eprintln!(
                    "DRIFT: {} missing committed stamp at {}",
                    name,
                    stamp_path.display()
                );
                drifted = true;
                continue;
            }
            let expected = fs::read_to_string(&stamp_path)?.trim().to_string();
            if actual != expected {
                eprintln!(
                    "DRIFT: {} actual={} committed-stamp={}",
                    name, actual, expected
                );
                drifted = true;
            }
        }
        // c_stubs.cpp drift check (Plan 02-06 Wave-2-1).
        {
            let actual = sha256_hex(&c_stubs_src);
            let stamp_path = validation_dir.join("c_stubs.cpp.sha256");
            if !stamp_path.exists() {
                eprintln!(
                    "DRIFT: validation/c_stubs.cpp missing committed stamp at {}",
                    stamp_path.display()
                );
                drifted = true;
            } else {
                let expected = fs::read_to_string(&stamp_path)?.trim().to_string();
                if actual != expected {
                    eprintln!(
                        "DRIFT: validation/c_stubs.cpp actual={} committed-stamp={}",
                        actual, expected
                    );
                    drifted = true;
                }
            }
        }
        if drifted {
            eprintln!();
            eprintln!(
                "Committed registry sources drifted from a fresh regeneration. Run:"
            );
            eprintln!("    cargo run -p xtask --bin regen-registry");
            eprintln!("and commit the resulting changes under");
            eprintln!("    crates/xcfun-core/src/registry/generated/");
            eprintln!("    validation/c_stubs.cpp (+ .sha256)");
            std::process::exit(2);
        }
        println!("regen-registry --check: OK (no drift)");
        return Ok(());
    }

    fs::create_dir_all(&generated_dir)?;
    write_with_sha256_stamp(&generated_dir.join("FUNCTIONAL_DESCRIPTORS.rs"), &descriptors_src)?;
    write_with_sha256_stamp(&generated_dir.join("VARS_TABLE.rs"), &vars_src)?;
    write_with_sha256_stamp(&generated_dir.join("ALIASES.rs"), &aliases_src)?;
    write_with_sha256_stamp(&generated_dir.join("parameters.rs"), &parameters_src)?;

    // Plan 02-06 Wave-2-1: also emit validation/c_stubs.cpp for the cc-build's
    // template recursion over XC_NR_FUNCTIONALS. 67 stubs (78 IDs - 11 LDAs).
    fs::create_dir_all(&validation_dir)?;
    write_with_sha256_stamp(&c_stubs_path, &c_stubs_src)?;

    eprintln!(
        "[regen-registry] wrote {} functional descriptors, {} vars rows, {} aliases, {} parameters",
        FUNCTIONAL_IDS.len(),
        vars_rows.len(),
        aliases_records.len(),
        params_records.len()
    );
    eprintln!(
        "[regen-registry] wrote validation/c_stubs.cpp ({} stubs)",
        c_stubs_count
    );
    Ok(())
}
