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
    // Phase 3 plan 03-05 — `--mode {partial_derivatives,potential}` flag.
    // Default `partial_derivatives` keeps Phase-2 CLI behaviour intact.
    let mode_str = parse_arg(&args, "--mode").unwrap_or("partial_derivatives");
    let mode = match mode_str {
        "partial_derivatives" => validation::driver::HarnessMode::PartialDerivatives,
        "potential" => validation::driver::HarnessMode::Potential,
        // Plan 04-05 D-06-C — Mode::Contracted at orders 5/6 vs C++ DOEVAL.
        "contracted" => validation::driver::HarnessMode::Contracted,
        other => anyhow::bail!(
            "--mode must be 'partial_derivatives', 'potential', or 'contracted'; got {}",
            other
        ),
    };

    // Phase 3 plan 03-06 — `--grid {default, supplemental}` flag.
    // `default` retains the Phase-2 10k-point seeded grid (seed 0x1234abcd).
    // `supplemental` extends with the 400-point GGA-stratified supplement
    // (seed 0xdeadbeef per PATTERNS.md J2).
    let grid_name = parse_arg(&args, "--grid").unwrap_or("default");

    if backend != "cpu" {
        anyhow::bail!(
            "Phase 2 only supports --backend cpu; got {} (D-23)",
            backend
        );
    }
    // Phase 3 plan 03-06 — Mode::PartialDerivatives orders extended to 0..=4
    // per MODE-01 D-16. Mode::Potential ignores the --order flag (its
    // `output_length` is fixed at 2 or 3 per D-15).
    if mode == validation::driver::HarnessMode::PartialDerivatives && order > 4 {
        anyhow::bail!("Phase 3 supports order ≤ 4 (MODE-01 D-16); got {}", order);
    }
    // Plan 04-05 D-06 — Mode::Contracted supports orders 0..=6 (XCFUN_MAX_ORDER).
    if mode == validation::driver::HarnessMode::Contracted && order > 6 {
        anyhow::bail!(
            "Mode::Contracted caps at order 6 (XCFUN_MAX_ORDER, Plan 04-05 D-06); got {}",
            order
        );
    }

    let regex = regex::Regex::new(filter).context("invalid --filter regex")?;
    tracing::info!(
        "Tier-2 harness: backend={} mode={} order={} filter={} grid={}",
        backend,
        mode_str,
        order,
        filter,
        grid_name
    );

    let grid = match grid_name {
        "default" => validation::fixtures::generate_grid(),
        "supplemental" => {
            let mut g = validation::fixtures::generate_grid();
            g.extend(validation::fixtures::gga_stratified_supplement());
            g
        }
        other => anyhow::bail!(
            "--grid must be 'default' or 'supplemental'; got {}",
            other
        ),
    };
    tracing::info!("Generated grid: {} points", grid.len());

    let report = validation::driver::run_with_mode(&grid, order, &regex, mode)?;
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
