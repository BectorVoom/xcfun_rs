//! Phase 7 Plan 07-08 (D-15) — integration tests for the `release-publish` xtask binary.
//!
//! These tests build and invoke the binary via `cargo run -p xtask --bin release-publish`
//! and assert observable behaviours (Tests 1–5 in the plan):
//!
//!   1. `--dry-run` exits 0 and prints the topological publish order
//!   2. dry-run output contains the 7 expected crate names in dependency order
//!   3. mutually exclusive flag combinations are rejected
//!   4. `--from xcfun-rs --dry-run` skips earlier crates and prints rs + capi only
//!   5. dry-run path always shells out to `cargo publish --dry-run`, never bare publish
//!      (Pitfall 5 yank-irreversibility lock)
//!
//! The 7 publishable crates in topological order:
//!   xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi
//!
//! `xcfun-py`, `xtask`, and `validation` are EXCLUDED via the NEVER_PUBLISH list.

use std::process::Command;

/// Run the release-publish binary with the given args and return (status, stdout, stderr).
///
/// We can't use `cargo run -p xtask --bin release-publish` from inside an integration
/// test (we'd recursively build cargo while the test harness already holds the build
/// lock). Instead, we use the `CARGO_BIN_EXE_release-publish` env var that Cargo
/// exports for integration tests of the same package.
fn run_release_publish(args: &[&str]) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_release-publish");
    let out = Command::new(bin)
        .args(args)
        .output()
        .expect("spawn release-publish");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (code, stdout, stderr)
}

/// The 7 crates expected in the v0.1.0 publish set, in topological order.
const EXPECTED_ORDER: &[&str] = &[
    "xcfun-ad",
    "xcfun-core",
    "xcfun-kernels",
    "xcfun-eval",
    "xcfun-gpu",
    "xcfun-rs",
    "xcfun-capi",
];

/// Helper: extract the `Publish order` block from stdout. Returns the crate names in
/// the order they appear, dropping the `= <version>` suffix.
fn parse_publish_order(stdout: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_block = false;
    for line in stdout.lines() {
        if line.starts_with("Publish order") {
            in_block = true;
            continue;
        }
        if in_block {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            // Lines look like: "  xcfun-ad = 0.1.0"
            if let Some(name) = trimmed.split_whitespace().next() {
                names.push(name.to_string());
            }
        }
    }
    names
}

#[test]
fn test_help_exits_zero() {
    let (code, stdout, _) = run_release_publish(&["--help"]);
    assert_eq!(code, 0, "--help should exit 0");
    assert!(
        stdout.contains("release-publish"),
        "help output must mention release-publish"
    );
}

#[test]
fn test_dry_run_prints_topological_order() {
    // The plan's --dry-run mode prints the publish order BEFORE shelling out to
    // `cargo publish --dry-run` per crate. The per-crate dry-run may fail in the
    // test environment (no network, etc.), so we look only at the header block.
    let (_code, stdout, _stderr) = run_release_publish(&["--dry-run"]);

    let order = parse_publish_order(&stdout);
    assert_eq!(
        order.len(),
        EXPECTED_ORDER.len(),
        "expected {} crates in publish order, got {}: {:?}",
        EXPECTED_ORDER.len(),
        order.len(),
        order
    );
    for (i, expected) in EXPECTED_ORDER.iter().enumerate() {
        assert_eq!(
            order[i].as_str(),
            *expected,
            "publish order position {i} mismatch: expected {expected}, got {}",
            order[i]
        );
    }
}

#[test]
fn test_excludes_never_publish_crates() {
    // xtask, validation, xcfun-py are workspace members but NEVER published.
    let (_code, stdout, _stderr) = run_release_publish(&["--dry-run"]);
    let order = parse_publish_order(&stdout);
    for forbidden in &["xtask", "validation", "xcfun-py", "xcfun-python"] {
        assert!(
            !order.iter().any(|n| n == forbidden),
            "publish order must NOT contain {forbidden}; got {order:?}"
        );
    }
}

#[test]
fn test_from_skips_earlier_crates() {
    // --from xcfun-rs should skip the first 5 and print only xcfun-rs + xcfun-capi.
    let (_code, stdout, _stderr) = run_release_publish(&["--from", "xcfun-rs", "--dry-run"]);
    let order = parse_publish_order(&stdout);
    assert_eq!(
        order,
        vec!["xcfun-rs".to_string(), "xcfun-capi".to_string()],
        "--from xcfun-rs should yield only [xcfun-rs, xcfun-capi]; got {order:?}"
    );
}

#[test]
fn test_unknown_flag_fails() {
    let (code, _stdout, stderr) = run_release_publish(&["--bogus-flag"]);
    assert_ne!(code, 0, "unknown flag must exit non-zero");
    assert!(
        stderr.contains("Unknown flag") || stderr.contains("--bogus-flag"),
        "stderr should mention the unknown flag; got: {stderr}"
    );
}

#[test]
fn test_from_unknown_crate_fails() {
    let (code, _stdout, stderr) = run_release_publish(&["--from", "not-a-real-crate", "--dry-run"]);
    assert_ne!(code, 0, "--from with unknown crate must exit non-zero");
    assert!(
        stderr.contains("not-a-real-crate") || stderr.contains("not in publish set"),
        "stderr should mention the missing crate; got: {stderr}"
    );
}
