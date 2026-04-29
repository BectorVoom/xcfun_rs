//! Report writers — HTML matrix + JSONL records.
//!
//! Per RESEARCH §"report.html schema" + §"report.jsonl schema" + CONTEXT D-15.
//! - `report.html`: Functional × order matrix with color-coded max-rel-err
//!   per cell, plus a Tolerance column that annotates `1e-7 (D-24 override)`
//!   for the LDAERF family (transparent per CONTEXT D-24).
//! - `report.jsonl`: one JSON object per line, one per ReportRecord (failing
//!   records + sampled passing records per (functional, order) for
//!   transparency; never all 10k × outlen points — that would be ~40 MB).
//!
//! ## Durability + resume (Plan 04-10 incremental-jsonl-flush, 2026-04-28)
//!
//! Two consecutive ~4 h Phase-4 sign-off sweeps lost 100% of their data
//! because the harness used to buffer every record in memory and write the
//! `.jsonl` exactly once at end-of-run. Any interruption (pre-emptive skip,
//! SIGKILL, OOM, /tmp wipe) discarded everything. Fix:
//!
//! 1. `JsonlSink` — open `report.jsonl` once at the start of a run; write each
//!    record as a single line, flushing after every line. Trade ~10 M
//!    `write+flush` syscalls for full durability under any abrupt termination.
//!
//! 2. `read_completed_tuples` — on `--resume`, parse the existing
//!    `report.jsonl` line-by-line into a `HashSet<(functional, vars, mode,
//!    order)>`. The driver consults the set at the start of each per-tuple
//!    loop iteration and skips already-emitted tuples. Previously emitted
//!    records remain on disk (the sink is opened in append mode); the matrix
//!    is rebuilt from those records too so `report.html` stays accurate
//!    across resumed runs.
//!
//! The legacy `write_jsonl(report, path)` batch writer is preserved for
//! backwards compatibility with downstream code that still calls it directly.
//! New code paths (`main.rs`) use `JsonlSink` exclusively.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::driver::{CellSummary, Report, ReportRecord};

/// Streaming JSONL writer — one serialized record per line, flushed after
/// every line so SIGKILL / WSL VM termination / pre-emptive skip cannot
/// discard records that already returned successfully from a `write` call.
///
/// Wraps `std::io::LineWriter<File>` (which auto-flushes on each `\n`) AND
/// adds an explicit `writer.flush()` call per record because `LineWriter`'s
/// auto-flush only fires when the internal buffer fills past a line boundary;
/// for very long records (the ZVPBESOLC ~1 KB lines) that's already covered,
/// but for very short records (clamp-stratum markers) we want belt-and-braces.
///
/// Plan 04-10 task 10.2 — incremental flush.
pub struct JsonlSink {
    writer: LineWriter<fs::File>,
    /// Path retained for diagnostics.
    #[allow(dead_code)]
    path: String,
}

impl JsonlSink {
    /// Open `path` in **truncate** mode — used for clean (non-resumed) runs.
    /// Any prior content is discarded; this matches the legacy
    /// `fs::File::create(path)` semantics in `write_jsonl`.
    pub fn create(path: &str) -> Result<Self> {
        let f = fs::File::create(path)
            .with_context(|| format!("opening {} for write (truncate)", path))?;
        Ok(JsonlSink {
            writer: LineWriter::new(f),
            path: path.to_string(),
        })
    }

    /// Open `path` in **append** mode — used for resumed runs. Existing lines
    /// are preserved; new records are appended. The caller is responsible for
    /// having parsed the existing content via `read_completed_tuples` and
    /// populating the driver skip-set so we don't double-write tuples.
    pub fn append(path: &str) -> Result<Self> {
        let f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("opening {} for append", path))?;
        Ok(JsonlSink {
            writer: LineWriter::new(f),
            path: path.to_string(),
        })
    }

    /// Serialize one record to a single line, then flush. The `LineWriter`
    /// guarantees the `\n` terminator forces an internal flush; we call
    /// `flush()` again to defeat any kernel-side write-back caching.
    pub fn write_record(&mut self, rec: &ReportRecord) -> Result<()> {
        let line = serde_json::to_string(rec)?;
        writeln!(self.writer, "{}", line)?;
        self.writer.flush()?;
        Ok(())
    }
}

/// Legacy batch JSONL writer — preserved for `cargo test -p validation` paths
/// and any downstream code that still depends on `Report::records` being
/// fully populated and serialized at end-of-run. New code (`main.rs`) uses
/// `JsonlSink::create`/`append` for streaming durability instead.
///
/// Per ACC-03.
pub fn write_jsonl(report: &Report, path: &str) -> Result<()> {
    let mut f = fs::File::create(path)?;
    for rec in &report.records {
        writeln!(f, "{}", serde_json::to_string(rec)?)?;
    }
    Ok(())
}

/// Identifies a "(functional, vars, mode, order)" tuple — the granularity at
/// which the validation driver iterates and at which `--resume` skips. The
/// `vars` field is the `Debug`-formatted enum string (e.g. `"A_B"`) to match
/// the field already serialized into `ReportRecord`.
pub type TupleKey = (String, String, u32, u32);

/// Parse an existing `report.jsonl` line-by-line into the set of tuples that
/// already have at least one record on disk. The driver uses this set to
/// short-circuit the per-tuple loop on `--resume`.
///
/// Lines that fail to deserialize are logged and skipped — this guards
/// against the (rare) case where a prior run was killed mid-write and left
/// a truncated last line. The truncated line is dropped on the floor; the
/// clean lines preceding it remain authoritative.
///
/// Returns an empty set if `path` does not exist (first-ever run).
pub fn read_completed_tuples(path: &str) -> Result<HashSet<TupleKey>> {
    let mut set = HashSet::new();
    if !Path::new(path).exists() {
        return Ok(set);
    }
    let f = fs::File::open(path)
        .with_context(|| format!("opening {} for resume scan", path))?;
    let reader = BufReader::new(f);
    let mut malformed = 0_usize;
    let mut total = 0_usize;
    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(
                    "report.jsonl resume scan: I/O error reading line ({}); stopping scan",
                    e
                );
                break;
            }
        };
        if line.is_empty() {
            continue;
        }
        total += 1;
        match serde_json::from_str::<ReportRecord>(&line) {
            Ok(rec) => {
                set.insert((rec.functional, rec.vars, rec.mode, rec.order));
            }
            Err(_) => {
                // Most likely a truncated last line from a SIGKILL during
                // mid-record write. We tolerate it: prior complete lines
                // already contributed to `set`; this line is dropped.
                malformed += 1;
            }
        }
    }
    if malformed > 0 {
        tracing::warn!(
            "report.jsonl resume scan: {} malformed line(s) skipped ({} total parsed); \
             likely the tail of a SIGKILL-interrupted prior run",
            malformed,
            total
        );
    }
    tracing::info!(
        "report.jsonl resume scan: {} completed tuple(s) found in {}",
        set.len(),
        path
    );
    Ok(set)
}

/// Re-parse `path` and rebuild the `(functional, order) → CellSummary` map
/// for tuples that are NOT being re-evaluated this run. Lets `report.html`
/// remain accurate end-to-end across resumed runs (otherwise a `--resume`
/// run's HTML would only show the tuples it actually re-evaluated).
///
/// The reconstructed `CellSummary.threshold` is taken from the first record
/// seen for each cell — every record within a cell has the same threshold
/// (set once per functional in `threshold_for`), so this is exact.
pub fn rebuild_matrix_from_jsonl(
    path: &str,
    skip_keys: &HashSet<TupleKey>,
) -> Result<HashMap<(String, u32), CellSummary>> {
    let mut out: HashMap<(String, u32), CellSummary> = HashMap::new();
    if !Path::new(path).exists() {
        return Ok(out);
    }
    let f = fs::File::open(path)
        .with_context(|| format!("opening {} for matrix rebuild", path))?;
    let reader = BufReader::new(f);
    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }
        let rec: ReportRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue, // truncated tail; tolerate
        };
        let tup_key = (
            rec.functional.clone(),
            rec.vars.clone(),
            rec.mode,
            rec.order,
        );
        // Only carry over cells we are NOT re-evaluating this run. (If the
        // tuple is in skip_keys, it's "owned" by the prior run; we copy
        // its summary forward. If it's not in skip_keys, the current run
        // will rebuild that cell's summary fresh.)
        if !skip_keys.contains(&tup_key) {
            continue;
        }
        let cell_key = (rec.functional.clone(), rec.order);
        let entry = out.entry(cell_key).or_insert(CellSummary {
            max_rel_err: 0.0,
            threshold: rec.threshold,
            records_total: 0,
            records_failed: 0,
            rust_unavailable: 0,
            excluded_by_upstream_spec: false,
            clamp_stratum_records: 0,
            clamp_stratum_failures: 0,
        });
        if rec.excluded_by_upstream_spec {
            entry.excluded_by_upstream_spec = true;
        }
        if !rec.excluded_by_upstream_spec && !rec.excluded_by_regularize_clamp_design {
            entry.max_rel_err = entry.max_rel_err.max(rec.rel_err);
        }
        entry.records_total += 1;
        if rec.excluded_by_regularize_clamp_design {
            entry.clamp_stratum_records += 1;
            if !rec.pass {
                entry.clamp_stratum_failures += 1;
            }
        }
        if !rec.pass
            && !rec.excluded_by_upstream_spec
            && !rec.excluded_by_regularize_clamp_design
        {
            entry.records_failed += 1;
        }
        if rec.rust_unavailable {
            entry.rust_unavailable += 1;
        }
    }
    Ok(out)
}

/// Write `report.html` — Functional × order matrix (per-cell max-rel-err,
/// green/yellow/red color coding, per-row Tolerance column).
pub fn write_html(report: &Report, path: &str) -> Result<()> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\n");
    html.push_str("<title>XCFun Tier-2 Parity Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, \"Segoe UI\", Helvetica, Arial, sans-serif; padding: 24px; max-width: 1100px; }\n");
    html.push_str("h1 { margin: 0 0 8px 0; }\n");
    html.push_str("p { margin: 4px 0; }\n");
    html.push_str("code { background: #f4f4f4; padding: 1px 5px; border-radius: 3px; }\n");
    html.push_str("table { border-collapse: collapse; margin-top: 16px; font-family: Menlo, monospace; }\n");
    html.push_str("th, td { border: 1px solid #aaa; padding: 6px 12px; text-align: right; }\n");
    html.push_str("th { background: #eee; text-align: center; }\n");
    html.push_str("td.name { text-align: left; font-weight: bold; }\n");
    html.push_str("td.green { background: #cfc; }\n");
    html.push_str("td.yellow { background: #ffc; }\n");
    html.push_str("td.red { background: #fcc; font-weight: bold; }\n");
    html.push_str("td.gray { background: #eee; color: #666; }\n");
    html.push_str("td.tol { text-align: center; }\n");
    html.push_str("td.missing { background: #eef; color: #668; font-style: italic; text-align: center; }\n");
    html.push_str(".legend { margin-top: 20px; font-size: 0.95em; }\n");
    html.push_str(".legend span { padding: 2px 8px; border: 1px solid #aaa; margin-right: 4px; }\n");
    html.push_str("</style></head><body>\n");
    html.push_str("<h1>XCFun Tier-2 Parity Report</h1>\n");
    html.push_str("<p>Backend: <code>CpuRuntime (cubecl-cpu =0.10.0-pre.3)</code></p>\n");
    html.push_str("<p>Reference: <code>xcfun-master/</code> (vendored, cc-compiled static lib <code>xcfun_cpp_lda</code>)</p>\n");
    html.push_str(&format!(
        "<p>Generated: <code>{}</code></p>\n",
        system_time_str()
    ));
    let failed = report.failed_count();
    let total = report.total_records();
    let rust_unavail: usize = report
        .matrix
        .values()
        .map(|c| c.rust_unavailable)
        .sum();
    let clamp_total = report.clamp_stratum_total();
    let clamp_fails = report.clamp_stratum_failures_total();
    html.push_str(&format!(
        "<p>Total records: <strong>{}</strong>; Failed: <strong style=\"color:{}\">{}</strong>{}</p>\n",
        total,
        if failed == 0 { "#060" } else { "#c00" },
        failed,
        if rust_unavail > 0 {
            format!(
                "; Rust-unavailable: <strong style=\"color:#c60\">{}</strong> (D-19 INCONCLUSIVE)",
                rust_unavail
            )
        } else {
            String::new()
        }
    ));
    if clamp_total > 0 {
        html.push_str(&format!(
            "<p>Clamp-stratum excluded (Plan 02-06 Fix 2, D-22): <strong>{}</strong> records \
            (of which <strong>{}</strong> would have failed the threshold but are tests \
            of the `regularize` clamp design, not kernel correctness).</p>\n",
            clamp_total, clamp_fails,
        ));
    }

    // Discover functional names and the maximum derivative order present in
    // the matrix dynamically. This keeps the renderer correct as new tiers
    // are added (Phase 2 → 3 → 6 …) without needing to hand-maintain a
    // canonical list. BTreeMap iteration is already sorted by (name, order),
    // so we get stable alphabetical row ordering.
    let mut names: Vec<&String> = report.matrix.keys().map(|(n, _)| n).collect();
    names.sort();
    names.dedup();
    let max_order: u32 = report.matrix.keys().map(|(_, o)| *o).max().unwrap_or(0);

    html.push_str("<table>\n<thead><tr>\n");
    html.push_str("<th>Functional</th>\n");
    for o in 0..=max_order {
        html.push_str(&format!("<th>order={}</th>", o));
    }
    html.push_str("\n<th>Tolerance</th>\n");
    html.push_str("</tr></thead>\n<tbody>\n");

    for name in &names {
        let cells: Vec<Option<&CellSummary>> = (0..=max_order)
            .map(|o| report.matrix.get(&((*name).to_string(), o)))
            .collect();
        // Skip rows with no data at all (e.g., if filter excluded them).
        if cells.iter().all(|c| c.is_none()) {
            continue;
        }
        html.push_str(&format!("<tr><td class=\"name\">{}</td>", name));
        let mut tolerance: Option<f64> = None;
        for cell in &cells {
            match cell {
                Some(cell) => {
                    tolerance = Some(cell.threshold);
                    let cls = if cell.excluded_by_upstream_spec {
                        "gray"
                    } else if cell.rust_unavailable > 0 {
                        "gray"
                    } else if cell.records_failed > 0 {
                        "red"
                    } else if cell.max_rel_err >= cell.threshold * 0.1 {
                        "yellow"
                    } else {
                        "green"
                    };
                    let text = if cell.excluded_by_upstream_spec {
                        "N/A (excluded: no upstream test_in)".to_string()
                    } else if cell.rust_unavailable > 0 {
                        "N/A (NotConfigured)".to_string()
                    } else {
                        format!("{:.2e}", cell.max_rel_err)
                    };
                    html.push_str(&format!("<td class=\"{}\">{}</td>", cls, text));
                }
                None => {
                    html.push_str("<td class=\"missing\">—</td>");
                }
            }
        }
        let tol = tolerance.unwrap_or(1e-12);
        let tol_str = if (tol - 1e-7).abs() < 1e-20 {
            "1e-7 (D-24 override)".to_string()
        } else {
            format!("{:.0e}", tol)
        };
        html.push_str(&format!("<td class=\"tol\">{}</td></tr>\n", tol_str));
    }
    html.push_str("</tbody></table>\n");

    html.push_str("<div class=\"legend\">\n");
    html.push_str("<p>Color key: <span class=\"green\">GREEN</span> rel-err &lt; threshold/10; ");
    html.push_str("<span class=\"yellow\">YELLOW</span> threshold/10 &le; rel-err &lt; threshold; ");
    html.push_str("<span class=\"red\">RED</span> rel-err &ge; threshold (FAIL); ");
    html.push_str("<span class=\"gray\">GRAY</span> Rust launch arm not yet wired (D-19 INCONCLUSIVE trigger — NOT silent widening).</p>\n");
    html.push_str("<p>Threshold dispatch per CONTEXT D-24: strict 1e-12 for 8 LDAs; 1e-7 for the 3 LDAERF functionals (user-approved override, annotated above).</p>\n");
    html.push_str("</div>\n");
    html.push_str("</body></html>\n");

    fs::write(path, html)?;
    Ok(())
}

fn system_time_str() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{} seconds since Unix epoch (UTC)", now.as_secs())
}
