//! `validation` binary — tier-2 parity harness CLI.
//!
//! Usage:
//!   cargo run -p validation --release -- [--backend {cpu|rocm|cuda|wgpu|metal}]
//!                                        [--tier {2|3}]
//!                                        [--reference {cpp|mpmath}]
//!                                        [--exclude-erf]
//!                                        [--order N] [--filter REGEX]
//!                                        [--mode {partial_derivatives,potential,contracted}]
//!                                        [--grid {default,supplemental}]
//!                                        [--resume]
//!
//! Exit codes:
//!   0 — all records within their per-functional threshold
//!   2 — at least one record exceeded its threshold (ACC-03 merge block)
//!   1 — internal error (bad CLI flag, build/FFI failure, etc.)
//!
//! ## Phase 6 Plan 06-02b — `--tier`, `--reference`, `--exclude-erf`, extended `--backend`
//!
//! - `--tier 2` (default) preserves Phase 2-5 cc-vs-Rust behaviour at 1e-12.
//! - `--tier 3` dispatches to the cross-backend `Batch<R>` parity skeleton
//!   (`run_tier3` in driver.rs). Plan 06-02b implements the Cpu arm
//!   skeleton; the actual KER-06 strict-1e-13 sweep + 17-functional bar is
//!   owned by Plan 06-05 (revision-1 B-4).
//! - `--reference {cpp|mpmath}` selects ground truth. Default `cpp`. The
//!   `mpmath` branch is wired by Plan 06-N2 (mpmath-only fixtures for the
//!   20 `excluded_by_upstream_spec` functionals + the 5 ERF cases per
//!   ACC-04 amendment / D-03).
//! - `--exclude-erf` filters out functionals carrying `Dependency::ERF`
//!   (consumed by Plan 06-04 for the Wgpu tier-3 1e-9 sweep per GPU-08).
//! - `--backend` accepts `cpu | rocm | cuda | wgpu | metal`. Plans 06-03 /
//!   06-04 wire concrete arms; until then non-cpu values bail with a
//!   helpful error message identifying the required `--features` flag.
//!
//! ## --resume (Plan 04-10, 2026-04-28)
//!
//! When passed, the harness:
//!   1. Parses the existing `validation/report.jsonl` line-by-line into a
//!      `HashSet<(functional, vars, mode, order)>` of completed tuples.
//!   2. Opens `validation/report.jsonl` in **append** mode (does NOT
//!      truncate). New records flush per-line.
//!   3. Re-builds matrix entries from the prior file for the skipped tuples
//!      so `report.html` end-of-run remains accurate.
//!   4. Skips any `(functional, vars, mode, order)` tuple in the skip-set —
//!      the driver short-circuits at the start of every per-tuple loop.
//!
//! Without `--resume`, `report.jsonl` is truncated as it has always been.
//! Either way, every record is now flushed to disk synchronously so a
//! SIGKILL/OOM/WSL-VM-termination cannot lose data already written.

use anyhow::{Context, Result};
use std::collections::HashSet;

fn parse_arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().enumerate().find_map(|(i, a)| {
        if a == name {
            args.get(i + 1).map(|s| s.as_str())
        } else {
            None
        }
    })
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let backend = parse_arg(&args, "--backend").unwrap_or("cpu");
    // Phase 6 Plan 06-02b — `--tier {2|3}`. Default 2 preserves Phase 2-5
    // cc-vs-Rust behaviour. Tier 3 dispatches to `run_tier3` (cross-backend
    // Batch<R> parity skeleton; Cpu arm wired in 06-02b, GPU arms in 06-03/04).
    let tier: u32 = parse_arg(&args, "--tier")
        .unwrap_or("2")
        .parse()
        .context("--tier must be 2 or 3")?;
    if tier != 2 && tier != 3 {
        anyhow::bail!("--tier must be 2 or 3; got {}", tier);
    }
    // Phase 6 Plan 06-02b — `--reference {cpp|mpmath}`. Default `cpp` preserves
    // existing behaviour. Plan 06-N2 wires the `mpmath` branch (ACC-04 / D-03
    // mpmath-truth amendment for the 20 `excluded_by_upstream_spec` functionals
    // and the 5 ERF range-separated functionals where C++ is documentably
    // unstable in the cancellation regime).
    let reference = parse_arg(&args, "--reference").unwrap_or("cpp");
    let reference_e = validation::driver::Reference::from_str(reference)
        .with_context(|| format!("--reference must be 'cpp' or 'mpmath'; got {}", reference))?;
    // Phase 6 Plan 06-02b — `--exclude-erf` filter; consumed by Plan 06-04 for
    // the Wgpu tier-3 1e-9 sweep per GPU-08 (range-separated functionals
    // carrying Dependency::ERF route to CPU on Wgpu/Metal backends).
    let exclude_erf = has_flag(&args, "--exclude-erf");

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

    // Plan 04-10 — `--resume` flag. Off-by-default to preserve the legacy
    // truncating clean-run behaviour (and the byte-for-byte JSONL invariant
    // expected by the merge gate).
    let resume = has_flag(&args, "--resume");

    // Quick task 260430-4x7 — `--jobs auto|N`. Default `auto` uses
    // `std::thread::available_parallelism()`; `--jobs 1` reproduces the
    // legacy serial path; `--jobs N` (N > 1) spawns N workers inside
    // `std::thread::scope` per driver entry point. Bails on `--jobs 0`.
    let jobs_arg = parse_arg(&args, "--jobs").unwrap_or("auto");
    let jobs_count = match jobs_arg {
        "auto" => std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
        s => s
            .parse::<usize>()
            .with_context(|| format!("--jobs must be 'auto' or a positive integer; got {}", s))?,
    };
    if jobs_count == 0 {
        anyhow::bail!("--jobs must be >= 1; got 0");
    }
    let jobs = std::num::NonZeroUsize::new(jobs_count).unwrap();
    tracing::info!(
        "Parallelism: --jobs {} ({} effective)",
        jobs_arg,
        jobs.get()
    );

    // Phase 6 Plan 06-02b — Tier-3 dispatch. The actual KER-06 sign-off
    // command is owned by Plan 06-05 (revision-1 B-4); Plan 06-02b ships the
    // CLI wiring + the `run_tier3` driver skeleton (Cpu arm scoped for 06-05,
    // ROCm/CUDA/Wgpu/Metal arms bail with feature-flag hints).
    if tier == 3 {
        return validation::driver::run_tier3(backend, order, jobs.get(), filter, exclude_erf);
    }

    // Tier-2 path (default; Phase 2-5 behaviour). Only `--backend cpu` is
    // wired here today. Plans 06-03 / 06-04 may add tier-2 GPU dispatch if
    // ever needed; not in 06-02b scope.
    if backend != "cpu" {
        anyhow::bail!(
            "Tier-2 harness supports --backend cpu only; got {} (use --tier 3 for cross-backend dispatch)",
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
        "Tier-2 harness: backend={} mode={} order={} filter={} grid={} resume={}",
        backend,
        mode_str,
        order,
        filter,
        grid_name,
        resume,
    );

    let grid = match grid_name {
        "default" => validation::fixtures::generate_grid(),
        "supplemental" => {
            let mut g = validation::fixtures::generate_grid();
            g.extend(validation::fixtures::gga_stratified_supplement());
            g
        }
        other => anyhow::bail!("--grid must be 'default' or 'supplemental'; got {}", other),
    };
    tracing::info!("Generated grid: {} points", grid.len());

    // Plan 04-10 — durability + resume.
    //
    // 1. If --resume: parse existing report.jsonl into a skip-set; rebuild
    //    prior matrix entries; open the sink in append mode.
    // 2. Else: empty skip-set; open the sink in truncate mode (legacy
    //    semantics).
    let jsonl_path = "validation/report.jsonl";
    let skip_keys: HashSet<validation::report::TupleKey> = if resume {
        let s = validation::report::read_completed_tuples(jsonl_path)?;
        tracing::info!("--resume: {} prior tuple(s) will be skipped", s.len());
        s
    } else {
        HashSet::new()
    };
    let prior_matrix = if resume {
        validation::report::rebuild_matrix_from_jsonl(jsonl_path, &skip_keys)?
    } else {
        Default::default()
    };
    let mut sink = if resume {
        validation::report::JsonlSink::append(jsonl_path)?
    } else {
        validation::report::JsonlSink::create(jsonl_path)?
    };

    let mut cfg = validation::driver::RunConfig {
        sink: Some(&mut sink),
        skip_keys: &skip_keys,
        jobs,
    };

    // Quick task 260430-4x7 — FFI globals pre-warm.
    //
    // xcfun-master's `xcint_assure_setup` (xcint.cpp:138) uses a one-shot
    // `static bool is_setup` guard. The body is idempotent but writes
    // descriptor tables non-atomically, so on the first-ever call two
    // threads racing here could observe torn pointers. Construct one
    // CppXcfun on the main thread to drive the guard to true before any
    // worker spawn. After this, all C++ state is per-handle (xcfun_new
    // returns a heap-owned XCFunctional*) and is safe to use from
    // many threads concurrently — each worker constructs its own.
    {
        let _prewarm = validation::ffi::CppXcfun::new();
        // _prewarm is dropped at end-of-scope — xcfun_delete is called.
    }

    // Phase 6 Plan 06-N2 — when --reference mpmath is specified, dispatch
    // to the mpmath-truth driver (`run_tier2_mpmath`) instead of the C++-paired
    // driver. The mpmath path consumes JSONL fixtures from
    // `validation/fixtures/mpmath/<functional>.jsonl` (committed source-of-truth
    // produced by the offline ~6h `cargo run -p xtask --bin regen-mpmath-fixtures`
    // run; see 06-N2-SUMMARY.md for the exact command).
    let mut report = match reference_e {
        validation::driver::Reference::Mpmath => {
            tracing::info!(
                "Tier-2 (--reference mpmath): consuming mpmath fixtures \
                 at validation/fixtures/mpmath/<functional>.jsonl"
            );
            validation::driver::run_tier2_mpmath(&regex, &mut cfg)?
        }
        validation::driver::Reference::Cpp => {
            validation::driver::run_with_mode_cfg(&grid, order, &regex, mode, &mut cfg)?
        }
    };

    // Drop the sink to ensure all buffered bytes hit the OS before we
    // hand the report off to the HTML writer (per-line flush already
    // wrote each record; this is belt-and-braces for the underlying file).
    drop(cfg);
    drop(sink);

    // Carry forward prior-run cells for tuples we did not re-evaluate, so
    // report.html shows complete coverage (otherwise a --resume run's HTML
    // would only display the cells touched in this invocation).
    if !prior_matrix.is_empty() {
        report.extend_matrix_from_prior(prior_matrix);
    }

    validation::report::write_html(&report, "validation/report.html")?;

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
