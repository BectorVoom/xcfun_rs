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

fn emit_aliases_rs() -> String {
    let mut out = String::new();
    out.push_str(
        "// AUTO-GENERATED by `cargo run -p xtask --bin regen-registry` — do not edit by hand.\n\
         // Source: xcfun-master/src/functionals/aliases.cpp (Phase 2: empty; Phase 4 populates 46 rows)\n\n",
    );
    out.push_str(
        "/// Registry row describing one alias functional (e.g., \"BLYP\" ->\n\
         /// 1.0*BECKEX + 1.0*LYPC). Phase 2 ships empty; Phase 4 extends the\n\
         /// extractor to parse `aliases.cpp` and populates 46 rows.\n\
         #[derive(Debug, Clone, Copy)]\n\
         pub struct Alias {\n\
         \x20   pub name: &'static str,\n\
         \x20   pub description: &'static str,\n\
         \x20   pub components: &'static [(&'static str, f64)],\n\
         }\n\n",
    );
    out.push_str("pub static ALIASES: &[Alias] = &[];\n");
    out
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

    let descriptors_src = emit_functional_descriptors_rs(&functionals);
    let vars_src = emit_vars_table_rs(&vars_rows);
    let aliases_src = emit_aliases_rs();

    let generated_dir = root.join("crates/xcfun-core/src/registry/generated");

    if check_only {
        let mut drifted = false;
        for (name, src) in [
            ("FUNCTIONAL_DESCRIPTORS.rs", &descriptors_src),
            ("VARS_TABLE.rs", &vars_src),
            ("ALIASES.rs", &aliases_src),
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
        if drifted {
            eprintln!();
            eprintln!(
                "Committed registry sources drifted from a fresh regeneration. Run:"
            );
            eprintln!("    cargo run -p xtask --bin regen-registry");
            eprintln!("and commit the resulting changes under");
            eprintln!("    crates/xcfun-core/src/registry/generated/");
            std::process::exit(2);
        }
        println!("regen-registry --check: OK (no drift)");
        return Ok(());
    }

    fs::create_dir_all(&generated_dir)?;
    write_with_sha256_stamp(&generated_dir.join("FUNCTIONAL_DESCRIPTORS.rs"), &descriptors_src)?;
    write_with_sha256_stamp(&generated_dir.join("VARS_TABLE.rs"), &vars_src)?;
    write_with_sha256_stamp(&generated_dir.join("ALIASES.rs"), &aliases_src)?;

    eprintln!(
        "[regen-registry] wrote {} functional descriptors, {} vars rows, 0 aliases",
        FUNCTIONAL_IDS.len(),
        vars_rows.len()
    );
    Ok(())
}
