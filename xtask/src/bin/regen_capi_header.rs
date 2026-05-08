//! Phase 5 D-09 — Regenerate `crates/xcfun-capi/include/xcfun.h` from
//! cbindgen + matching `.sha256` stamp file.
//!
//! Workflow:
//!   1. cbindgen::Builder::new().with_crate(/* crates/xcfun-capi */)
//!        .with_config(/* cbindgen.toml */).generate()?
//!        .write_to_file(/* include/xcfun.h */).
//!   2. Read the just-written file; sha256 it; write `xcfun.h.sha256`.
//!   3. `--check` mode: regenerate in memory, sha256 it, compare to
//!      committed stamp; exit 2 on drift.
//!
//! Invocation:
//!   - `cargo run -p xtask --bin regen-capi-header`
//!   - `cargo run -p xtask --bin regen-capi-header -- --check`

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

fn project_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").context(
        "CARGO_MANIFEST_DIR not set — run via cargo run -p xtask --bin regen-capi-header",
    )?;
    let xtask_dir = PathBuf::from(manifest);
    let root = xtask_dir
        .parent()
        .context("xtask has no parent directory")?
        .to_path_buf();
    Ok(root)
}

fn main() -> Result<()> {
    let check_mode = std::env::args().any(|a| a == "--check");
    let root = project_root()?;
    let crate_dir = root.join("crates/xcfun-capi");
    let cbg_toml = crate_dir.join("cbindgen.toml");
    let header_path = crate_dir.join("include/xcfun.h");
    let sha_path = crate_dir.join("include/xcfun.h.sha256");

    let cfg = cbindgen::Config::from_file(&cbg_toml)
        .map_err(|e| anyhow::anyhow!("failed to load cbindgen.toml: {e}"))?;
    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cfg)
        .generate()
        .map_err(|e| anyhow::anyhow!("cbindgen generate failed: {e}"))?;

    let mut buf = Vec::<u8>::new();
    bindings.write(&mut buf);
    let hash = format!("{:x}", Sha256::digest(&buf));

    if check_mode {
        let committed = fs::read_to_string(&sha_path)
            .with_context(|| format!("missing {}", sha_path.display()))?
            .trim()
            .to_string();
        if committed != hash {
            bail!(
                "header drift detected — committed sha {committed} != regenerated sha {hash}\n\
                 run `cargo run -p xtask --bin regen-capi-header` and commit the result"
            );
        }
        eprintln!("regen-capi-header: OK (sha {hash})");
    } else {
        fs::create_dir_all(crate_dir.join("include"))?;
        fs::write(&header_path, &buf)?;
        fs::write(&sha_path, format!("{hash}\n"))?;
        eprintln!(
            "regen-capi-header: wrote {} ({} bytes; sha256 {})",
            header_path.display(),
            buf.len(),
            hash,
        );
    }
    Ok(())
}
