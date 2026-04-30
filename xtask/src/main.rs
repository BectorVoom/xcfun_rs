//! xtask entry point — dispatches to sub-binaries.
//!
//! Usage: `cargo xtask <subcommand>` is driven by `.cargo/config.toml` alias
//! (not yet added — see Phase 0). For Phase 1, call sub-binaries directly:
//!   `cargo run -p xtask --bin regen-ad-fixtures`

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("regen-ad-fixtures") => {
            // Delegation target; wired fully in Plan 01-05.
            println!("xtask: regen-ad-fixtures is implemented as its own binary.");
            println!("Run: cargo run -p xtask --bin regen-ad-fixtures");
            Ok(())
        }
        Some("regen-capi-header") => {
            println!("xtask: regen-capi-header is implemented as its own binary.");
            println!("Run: cargo run -p xtask --bin regen-capi-header");
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown xtask subcommand: {other}"),
        None => {
            println!("xtask subcommands: regen-ad-fixtures regen-capi-header");
            Ok(())
        }
    }
}
