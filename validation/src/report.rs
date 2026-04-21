//! Report writers — HTML matrix + JSONL records.
//!
//! Per RESEARCH §"report.html schema" + §"report.jsonl schema" + CONTEXT D-15.
//! - `report.html`: Functional × order matrix with color-coded max-rel-err
//!   per cell, plus a Tolerance column that annotates `1e-7 (D-24 override)`
//!   for the LDAERF family (transparent per CONTEXT D-24).
//! - `report.jsonl`: one JSON object per line, one per ReportRecord (failing
//!   records + sampled passing records per (functional, order) for
//!   transparency; never all 10k × outlen points — that would be ~40 MB).

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::driver::Report;

/// Write `report.jsonl` — one serde_json record per line (ACC-03).
pub fn write_jsonl(report: &Report, path: &str) -> Result<()> {
    let mut f = fs::File::create(path)?;
    for rec in &report.records {
        writeln!(f, "{}", serde_json::to_string(rec)?)?;
    }
    Ok(())
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

    // Collect functionals in a stable order that matches the plan's table.
    let stable_order = [
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

    html.push_str("<table>\n<thead><tr>\n");
    html.push_str("<th>Functional</th>\n");
    html.push_str("<th>order=0</th><th>order=1</th><th>order=2</th>\n");
    html.push_str("<th>Tolerance</th>\n");
    html.push_str("</tr></thead>\n<tbody>\n");

    for name in &stable_order {
        let cells: [Option<_>; 3] = [
            report.matrix.get(&((*name).to_string(), 0)),
            report.matrix.get(&((*name).to_string(), 1)),
            report.matrix.get(&((*name).to_string(), 2)),
        ];
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
