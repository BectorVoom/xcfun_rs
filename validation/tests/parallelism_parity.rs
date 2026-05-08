//! Parallel parity integration test for the validation harness.
//!
//! Quick task 260430-4x7 — proves `--jobs 1` and `--jobs N>1` produce
//! identical records and an identical Report.matrix on a small filter,
//! AND that the parallel JSONL on disk is content-equivalent to the
//! serial JSONL after sort. This is the on-disk counterpart to the
//! in-memory parity test in `driver::parallel_run_tests`.

#![cfg(not(miri))]

use std::collections::HashSet;
use std::num::NonZeroUsize;

use validation::driver::{HarnessMode, ReportRecord, RunConfig, run_with_mode_cfg};
use validation::fixtures::generate_grid;
use validation::report::{JsonlSink, TupleKey};

fn run_to_jsonl(jobs: usize, filter: &str, path: &str) -> validation::driver::Report {
    let grid = generate_grid();
    // 128-point slice keeps the test fast (~3 s end-to-end) while still
    // exercising the parallel path across multiple jobs.
    let small_grid: Vec<_> = grid.into_iter().take(128).collect();
    let regex = regex::Regex::new(filter).unwrap();
    let mut sink = JsonlSink::create(path).unwrap();
    let empty: HashSet<TupleKey> = HashSet::new();
    let mut cfg = RunConfig {
        sink: Some(&mut sink),
        skip_keys: &empty,
        jobs: NonZeroUsize::new(jobs).unwrap(),
    };
    let report = run_with_mode_cfg(
        &small_grid,
        0, // max_order
        &regex,
        HarnessMode::PartialDerivatives,
        &mut cfg,
    )
    .unwrap();
    drop(cfg);
    drop(sink);
    report
}

fn parse_jsonl(path: &str) -> Vec<ReportRecord> {
    use std::io::{BufRead, BufReader};
    let f = std::fs::File::open(path).unwrap();
    BufReader::new(f)
        .lines()
        .filter_map(|l| {
            let l = l.ok()?;
            if l.is_empty() {
                return None;
            }
            serde_json::from_str::<ReportRecord>(&l).ok()
        })
        .collect()
}

fn key(r: &ReportRecord) -> (String, String, u32, u32, usize, usize) {
    (
        r.functional.clone(),
        r.vars.clone(),
        r.mode,
        r.order,
        r.point_idx,
        r.element_idx,
    )
}

/// `--jobs 1` and `--jobs 4` write JSONL streams that, after sort by
/// `(functional, vars, mode, order, point_idx, element_idx)`, contain
/// byte-identical record content. Proves the on-disk JSONL is
/// content-equivalent under parallel emission — even though the
/// on-disk LINE order may differ.
#[test]
fn parallel_matches_serial_via_jsonl() {
    let dir = tempfile::tempdir().expect("tempdir");
    let serial_path = dir.path().join("serial.jsonl");
    let parallel_path = dir.path().join("parallel.jsonl");

    let _ = run_to_jsonl(1, "xc_slaterx", serial_path.to_str().unwrap());
    let _ = run_to_jsonl(4, "xc_slaterx", parallel_path.to_str().unwrap());

    let mut s = parse_jsonl(serial_path.to_str().unwrap());
    let mut p = parse_jsonl(parallel_path.to_str().unwrap());
    assert_eq!(
        s.len(),
        p.len(),
        "JSONL record count mismatch (serial={} parallel={})",
        s.len(),
        p.len()
    );
    s.sort_by_key(|r| key(r));
    p.sort_by_key(|r| key(r));
    for (a, b) in s.iter().zip(p.iter()) {
        assert_eq!(
            serde_json::to_string(a).unwrap(),
            serde_json::to_string(b).unwrap(),
            "JSONL record content mismatch at key {:?}",
            key(a)
        );
    }
}

/// In-memory `Report.matrix` parity (mirrors the unit test in
/// `driver::parallel_run_tests` but goes through `run_with_mode_cfg`,
/// the same entry point `main.rs` calls).
#[test]
fn parallel_matches_serial_via_matrix() {
    let dir = tempfile::tempdir().expect("tempdir");
    let serial_path = dir.path().join("serial.jsonl");
    let parallel_path = dir.path().join("parallel.jsonl");

    let serial = run_to_jsonl(1, "xc_slaterx", serial_path.to_str().unwrap());
    let parallel = run_to_jsonl(4, "xc_slaterx", parallel_path.to_str().unwrap());

    assert_eq!(serial.matrix.len(), parallel.matrix.len());
    for (k, sv) in &serial.matrix {
        let pv = parallel
            .matrix
            .get(k)
            .expect("missing cell in parallel matrix");
        assert_eq!(sv.records_total, pv.records_total);
        assert_eq!(sv.records_failed, pv.records_failed);
        assert_eq!(sv.rust_unavailable, pv.rust_unavailable);
        assert!((sv.max_rel_err - pv.max_rel_err).abs() <= f64::EPSILON);
        assert_eq!(sv.threshold, pv.threshold);
    }
}
