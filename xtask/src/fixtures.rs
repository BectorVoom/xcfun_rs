//! Fixture record schema. Shared between the regen-ad-fixtures binary
//! (writer) and the downstream xcfun-ad test suite (reader).
//!
//! The xcfun-ad side duplicates this struct verbatim in each
//! `tests/golden_*.rs` file (flagged with "keep in sync with xtask/src/fixtures.rs")
//! rather than taking an `xtask` path-dep, per Plan 01-05 decision.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One fixture record. Schema shared between the C++ driver (writer) and the
/// xcfun-ad test suite (reader).
///
/// Layout semantics by `op`:
/// - `op = "mul"`: `inputs` = first `1 << n_var` entries are `a.c[..]`, next
///   `1 << n_var` entries are `b.c[..]`. `coeffs` = `(a * b).c[..]` of length
///   `1 << n_var`.
/// - `op = "inv_expand" | "exp_expand" | "log_expand" | "sqrt_expand" |`
///   `"cbrt_expand" | "gauss_expand" | "erf_expand"`: `inputs = [x0]` (length 1);
///   `coeffs = t[0..=n_var]` of length `n_var + 1`. Here `n_var` reuses the
///   `u8` field for the expansion order — slightly abusing the schema to keep
///   one struct.
/// - `op = "pow_expand"`: `inputs = [x0, a]` (length 2); `coeffs = t[0..=n_var]`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixtureRecord {
    pub op: String,
    pub n_var: u8,
    pub inputs: Vec<f64>,
    pub coeffs: Vec<f64>,
}

/// Top-level manifest written as `fixtures.json` alongside the bincode files.
///
/// Pins `xcfun_version_git_sha` (sha256 of the three xcfun taylor headers)
/// so that if the vendored C++ reference drifts, CI detects it when the
/// regenerated manifest hash stops matching the committed manifest hash.
#[derive(Serialize, Deserialize, Debug)]
pub struct FixturesManifest {
    /// SHA-256 of the concatenated contents of
    /// `xcfun-master/external/upstream/taylor/{ctaylor.hpp, ctaylor_math.hpp, tmath.hpp}`.
    pub xcfun_version_git_sha: String,
    /// RFC 3339 timestamp at regen time.
    pub generated_at: String,
    /// Per-op record counts (e.g. "mul" -> 250, "exp_expand" -> 21).
    pub per_op_counts: BTreeMap<String, usize>,
    /// Sum over all `per_op_counts`.
    pub total_records: usize,
    /// `git rev-parse HEAD` of the xcfun_rs repo at regen time (best-effort).
    pub driver_commit: Option<String>,
}
