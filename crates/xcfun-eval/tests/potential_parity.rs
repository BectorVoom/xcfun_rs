//! Mode::Potential — strict 1e-12 parity vs C++ xcfun reference (Phase 3
//! plan 03-05 Task 3).
//!
//! Loads the offline-generated fixture
//! `tests/data/potential_reference_100.json` (100 records: 5 GGA functionals
//! × 20 Gaussian-atom density points; produced by
//! `xtask/src/bin/gen_potential_fixtures.rs` driving the cc-compiled C++
//! reference under MODE=XC_POTENTIAL + VARS=XC_A_B_2ND_TAYLOR), runs every
//! record through `Functional::eval` and asserts strict 1e-12 relative
//! error against the C++ ground truth.
//!
//! Per D-14: strict 1e-12 for GGA Mode::Potential, no blanket relaxation.
//! Per D-18: failures escalate via PLANNING INCONCLUSIVE — they are NOT
//! silently widened.

#![cfg(feature = "testing")]

use approx::assert_relative_eq;
use serde::Deserialize;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;

#[derive(Deserialize, Debug)]
struct PotentialRecord {
    functional_name: String,
    vars: String,
    input: Vec<f64>,
    expected_output: Vec<f64>,
}

/// Resolve `XC_PBEX` (etc.) → `FunctionalId` for the 5 fixture-grid
/// functionals.
fn fid_from_name(name: &str) -> Option<FunctionalId> {
    match name {
        "XC_PBEX" => Some(FunctionalId::XC_PBEX),
        "XC_BECKEX" => Some(FunctionalId::XC_BECKEX),
        "XC_LYPC" => Some(FunctionalId::XC_LYPC),
        "XC_PW91X" => Some(FunctionalId::XC_PW91X),
        "XC_B97X" => Some(FunctionalId::XC_B97X),
        _ => None,
    }
}

#[test]
fn potential_parity_100() {
    let json_str = include_str!("data/potential_reference_100.json");
    let records: Vec<PotentialRecord> =
        serde_json::from_str(json_str).expect("potential_reference_100.json must parse");
    assert!(
        records.len() >= 100,
        "expected ≥ 100 records, got {}",
        records.len()
    );

    let mut tested = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for rec in &records {
        let id = fid_from_name(&rec.functional_name)
            .unwrap_or_else(|| panic!("unknown functional {} in fixture", rec.functional_name));
        // Vars::A_B_2ND_TAYLOR is the only vars used by the fixture grid
        // (5 GGA functionals).
        assert_eq!(
            rec.vars, "A_B_2ND_TAYLOR",
            "fixture vars must be A_B_2ND_TAYLOR"
        );
        let f = Functional {
            // Plan 06-06 D-17: weights is now Vec<(FunctionalId, f64)>; no leak.
            weights: vec![(id, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let mut out = vec![0.0_f64; rec.expected_output.len()];
        f.eval(&rec.input, &mut out)
            .expect("Mode::Potential eval should succeed");

        for (i, (&got, &want)) in out.iter().zip(rec.expected_output.iter()).enumerate() {
            // Strict 1e-12 per D-14. Use approx style: rel_err =
            // |got - want| / max(|want|, 1.0); assert_relative_eq! uses
            // |got - want| / max(|want|, |got|) which is slightly stricter
            // — both are appropriate here.
            let abs_err = (got - want).abs();
            let denom = want.abs().max(got.abs()).max(1.0);
            let rel_err = abs_err / denom;
            if rel_err > 1e-12 {
                failed.push(format!(
                    "{}[i={}]: got {} want {} (abs_err {:.3e}, rel_err {:.3e})",
                    rec.functional_name, i, got, want, abs_err, rel_err
                ));
            }
        }
        tested += 1;
    }

    println!(
        "potential_parity_100: {} records tested, {} failures",
        tested,
        failed.len()
    );

    if !failed.is_empty() {
        // D-18: do NOT widen the threshold. Surface the failure for
        // PLANNING INCONCLUSIVE escalation.
        // Per-functional failure count for diagnosis.
        let mut by_fn: std::collections::BTreeMap<String, (usize, f64)> =
            std::collections::BTreeMap::new();
        for line in &failed {
            // Extract functional name (prefix before "[").
            if let Some(idx) = line.find('[') {
                let fname = line[..idx].to_string();
                let entry = by_fn.entry(fname).or_insert((0, 0.0));
                entry.0 += 1;
                // Extract rel_err.
                if let Some(re_pos) = line.rfind("rel_err ") {
                    if let Ok(re) = line[re_pos + 8..]
                        .trim_end_matches(')')
                        .trim()
                        .parse::<f64>()
                    {
                        if re > entry.1 {
                            entry.1 = re;
                        }
                    } else {
                        // Try parsing scientific notation up to ')'
                        let s = &line[re_pos + 8..];
                        let upto = s.find(')').unwrap_or(s.len());
                        if let Ok(re) = s[..upto].trim().parse::<f64>()
                            && re > entry.1
                        {
                            entry.1 = re;
                        }
                    }
                }
            }
        }
        let preview: Vec<&String> = failed.iter().take(15).collect();
        let summary: String = by_fn
            .iter()
            .map(|(k, (n, max_re))| format!("  {}: {} fails (max rel_err {:.3e})", k, n, max_re))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "potential_parity_100 FAILED ({} > 1e-12 records):\n=== Per-functional ===\n{}\n=== First {} samples ===\n{}",
            failed.len(),
            summary,
            preview.len(),
            preview
                .iter()
                .map(|s| format!("  {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // Sanity: assert one explicit comparison via approx for at least one
    // record so the assert_relative_eq macro is exercised in the test
    // binary.
    if let Some(rec) = records.first() {
        let id = fid_from_name(&rec.functional_name).unwrap();
        let f = Functional {
            // Plan 06-06 D-17: weights is now Vec<(FunctionalId, f64)>; no leak.
            weights: vec![(id, 1.0)],
            vars: Vars::A_B_2ND_TAYLOR,
            mode: Mode::Potential,
            order: 0,
            settings: DEFAULT_SETTINGS,
            settings_gen: 0,
        };
        let mut out = vec![0.0_f64; rec.expected_output.len()];
        f.eval(&rec.input, &mut out).unwrap();
        for (i, (&g, &w)) in out.iter().zip(rec.expected_output.iter()).enumerate() {
            assert_relative_eq!(
                g,
                w,
                max_relative = 1e-12,
                epsilon = 1e-20,
                // Help diagnose if this ever fails:
                // (record 0 is PBEX at the first selected grid point).
            );
            let _ = i;
        }
    }
}
