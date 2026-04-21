//! `validation` binary — tier-2 parity harness CLI.
//!
//! Usage:
//!   cargo run -p validation --release -- [--backend cpu] [--order N] [--filter REGEX]
//!
//! Exit codes:
//!   0 — all records within their per-functional threshold
//!   2 — at least one record exceeded its threshold (ACC-03 merge block)
//!   1 — internal error (bad CLI flag, build/FFI failure, etc.)

use anyhow::{Context, Result};

fn parse_arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().enumerate().find_map(|(i, a)| {
        if a == name {
            args.get(i + 1).map(|s| s.as_str())
        } else {
            None
        }
    })
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let backend = parse_arg(&args, "--backend").unwrap_or("cpu");
    let order: u32 = parse_arg(&args, "--order")
        .unwrap_or("2")
        .parse()
        .context("--order must be u32")?;
    let filter = parse_arg(&args, "--filter").unwrap_or(".*");

    if backend != "cpu" {
        anyhow::bail!(
            "Phase 2 only supports --backend cpu; got {} (D-23)",
            backend
        );
    }
    if order > 2 {
        anyhow::bail!("Phase 2 only supports order ≤ 2 (D-23); got {}", order);
    }

    let regex = regex::Regex::new(filter).context("invalid --filter regex")?;
    tracing::info!(
        "Tier-2 harness: backend={} order={} filter={}",
        backend,
        order,
        filter
    );

    let grid = validation::fixtures::generate_grid();
    tracing::info!("Generated grid: {} points", grid.len());

    let report = validation::driver::run(&grid, order, &regex)?;
    validation::report::write_html(&report, "validation/report.html")?;
    validation::report::write_jsonl(&report, "validation/report.jsonl")?;

    let n_failed = report.failed_count();
    if n_failed > 0 {
        tracing::error!(
            "Tier-2 FAIL: {} failing records (see validation/report.html)",
            n_failed
        );
        std::process::exit(2);
    }
    tracing::info!(
        "Tier-2 PASS: all {} records within tolerance",
        report.total_records()
    );
    Ok(())
}
