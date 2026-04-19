//! Regenerate `crates/xcfun-ad/tests/fixtures/*.bincode` from a C++ driver
//! linking `xcfun-master/external/upstream/taylor/`.
//!
//! Phase 1 Plan 05 populates this. Phase 1 Plan 01 (this file) ships the
//! skeleton so the workspace compiles.

fn main() -> anyhow::Result<()> {
    println!("regen-ad-fixtures: skeleton only (Plan 01-01 scaffolding).");
    println!("Wire-up in Plan 01-05 compiles xtask/assets/regen_ad_fixtures/driver.cpp");
    println!("via the cc crate and emits `crates/xcfun-ad/tests/fixtures/*.bincode`.");
    Ok(())
}
