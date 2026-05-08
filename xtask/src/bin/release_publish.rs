//! Phase 7 Plan 07-08 (D-15) — topological cargo publish driver.
//!
//! Walks the workspace dependency DAG via `cargo metadata --format-version 1`,
//! computes the topological publish order over the publishable Rust library
//! crates, runs `cargo publish --dry-run` per crate (Pitfall 5 — yank
//! irreversibility lock), and (under `--execute`) shells out to bare
//! `cargo publish -p <crate>` with index-propagation polling between
//! successive crates (Pitfall 4 — wait for crates.io to make the new
//! version visible before depending on it from the next crate).
//!
//! Run modes:
//!   --dry-run    print order; per-crate `cargo publish --dry-run`; no side effects (DEFAULT)
//!   --execute    after dry-run, run `cargo publish` per crate with index polling
//!   --from <c>   resume from crate `<c>`; idempotent recovery for partial runs
//!   --skip <c>   debug — skip crate `<c>` entirely (may be passed multiple times)
//!   --help|-h    usage
//!
//! The expected order for v0.1.0 is:
//!   xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi
//!
//! `xcfun-py` is PyPI-only (NOT on crates.io). `xtask` and `validation` are
//! workspace-internal (`publish = false`). Both are filtered via NEVER_PUBLISH.
//!
//! Per D-15 the maturin publish for `xcfun-py` is the LAST step but is NOT
//! invoked by this binary — the driver prints the recommended command and
//! exits. The CI-driven PyPI publish lives in `.github/workflows/release.yml`
//! (Plan 07-09 / Open Question 2).

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

/// Workspace-internal crates that are NEVER published to crates.io.
/// `xtask` + `validation` are `publish = false`; `xcfun-py` ships PyPI wheels
/// only (built by maturin, not cargo publish). `xcfun-python` is the legacy
/// pre-D-01 directory name kept here for defence-in-depth.
const NEVER_PUBLISH: &[&str] = &["xtask", "validation", "xcfun-py", "xcfun-python"];

/// Polling interval between `cargo search` index-propagation checks.
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// Total cap on index-propagation polling per crate (Pitfall 4).
const POLL_TIMEOUT: Duration = Duration::from_secs(300);
/// Cooldown between successive `cargo publish` calls when --execute is set.
/// 30 s is conservative; the cargo-search poll loop tightens this dynamically.
const PUBLISH_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
struct CrateInfo {
    name: String,
    version: String,
    /// Other workspace publishable crates this crate depends on (kind ∈ {normal, build}).
    deps_in_workspace: Vec<String>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut dry_run = true;
    let mut execute = false;
    let mut explicit_dry_run = false;
    let mut from: Option<String> = None;
    let mut skip: Vec<String> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dry-run" => {
                dry_run = true;
                explicit_dry_run = true;
            }
            "--execute" => {
                if explicit_dry_run {
                    bail!("--execute and --dry-run are mutually exclusive");
                }
                dry_run = false;
                execute = true;
            }
            "--from" => {
                from = iter
                    .next()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow!("--from requires a crate name"))?
                    .into();
            }
            "--skip" => {
                let s = iter
                    .next()
                    .ok_or_else(|| anyhow!("--skip requires a crate name"))?;
                skip.push(s.to_string());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => bail!("Unknown flag: {arg}. Run --help for usage."),
        }
    }

    if execute && explicit_dry_run {
        bail!("--execute and --dry-run are mutually exclusive");
    }

    let crates = topological_publish_order()?;
    let crates: Vec<CrateInfo> = crates
        .into_iter()
        .filter(|c| !skip.contains(&c.name))
        .collect();

    // Apply --from filter (resume idempotency for partially-published runs).
    let crates: Vec<CrateInfo> = if let Some(start) = from.as_deref() {
        let pos = crates
            .iter()
            .position(|c| c.name == start)
            .with_context(|| format!("--from: crate {start} not in publish set"))?;
        crates.into_iter().skip(pos).collect()
    } else {
        crates
    };

    println!("Publish order ({} crates):", crates.len());
    for c in &crates {
        println!("  {} = {}", c.name, c.version);
    }
    println!();

    for c in &crates {
        // Idempotent skip — already on crates.io at this exact version.
        if is_published_at_version(&c.name, &c.version)? {
            println!("[skip] {}@{} already on crates.io", c.name, c.version);
            continue;
        }

        // Pitfall 5 — `cargo publish --dry-run` BEFORE `cargo publish`.
        // The dry-run validates the package but does NOT consume a version
        // slot, so we can re-run the driver after a fix without burning a
        // patch number.
        println!("[dry-run] cargo publish -p {} --dry-run", c.name);
        let dry = run(&["cargo", "publish", "-p", &c.name, "--dry-run"]);
        if let Err(e) = dry {
            // In dry-run-only mode (the default), a failure of `cargo publish
            // --dry-run` is informational — the user is checking publishability,
            // not actually publishing. Print and continue so they see ALL the
            // failing crates in one pass.
            if dry_run {
                eprintln!("[warn] dry-run failed for {}: {e:#}", c.name);
                continue;
            } else {
                return Err(e).with_context(|| format!("dry-run failed for {}", c.name));
            }
        }

        if dry_run {
            println!("[dry-run-only] would publish {}@{}", c.name, c.version);
            continue;
        }

        // --execute path
        println!("[execute] cargo publish -p {}", c.name);
        run(&["cargo", "publish", "-p", &c.name])
            .with_context(|| format!("publish failed for {}", c.name))?;

        // Pitfall 4 — poll cargo search until the new version is visible on
        // the index, capped at POLL_TIMEOUT. Without this, the next crate's
        // `cargo publish` may fail because its dependency on this just-
        // published crate isn't yet resolvable.
        println!(
            "[wait] polling crates.io index for {}@{} ...",
            c.name, c.version
        );
        wait_for_index(&c.name, &c.version)?;

        thread::sleep(PUBLISH_COOLDOWN);
    }

    println!();
    if execute {
        println!("All Rust crates published. Now run (locally or in CI release.yml):");
    } else {
        println!("Dry-run complete. To publish for real:");
        println!("    cargo run -p xtask --bin release-publish -- --execute");
        println!();
        println!("After Rust crates land, the maturin step (D-15 — local OR CI release.yml):");
    }
    println!("    cd crates/xcfun-py && maturin publish --skip-existing");
    Ok(())
}

fn print_help() {
    println!("xtask release-publish — topological cargo publish driver (D-15)");
    println!();
    println!("USAGE:");
    println!("    cargo run -p xtask --bin release-publish -- [FLAGS]");
    println!();
    println!("FLAGS:");
    println!(
        "    --dry-run         (default) print order + per-crate `cargo publish --dry-run`; no side effects"
    );
    println!(
        "    --execute         actually publish; runs --dry-run first per crate, then `cargo publish`"
    );
    println!(
        "    --from <crate>    resume from <crate>; idempotent recovery for partially-published runs"
    );
    println!("    --skip <crate>    skip <crate> entirely (debug only; repeatable)");
    println!("    --help, -h        this message");
}

/// Compute the topological publish order from `cargo metadata`.
///
/// Modern cargo (≥ 1.77) emits `workspace_members` as PackageId strings of the
/// form `path+file:///abs/path#0.1.0` rather than the legacy `name version source`
/// triple. We therefore filter packages by exact PackageId match rather than
/// parsing the workspace_members entry directly.
///
/// Kahn's algorithm with deterministic alphabetical tiebreak yields the
/// expected order for the v0.1.0 graph:
///   xcfun-ad → xcfun-core → xcfun-kernels → xcfun-eval → xcfun-gpu → xcfun-rs → xcfun-capi
fn topological_publish_order() -> Result<Vec<CrateInfo>> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .context("invoke cargo metadata")?;
    if !out.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let v: Value = serde_json::from_slice(&out.stdout).context("parse cargo metadata JSON")?;

    // Workspace member PackageIds (full strings, used for exact-match lookup).
    let member_ids: HashSet<String> = v["workspace_members"]
        .as_array()
        .ok_or_else(|| anyhow!("missing workspace_members"))?
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();

    let packages = v["packages"]
        .as_array()
        .ok_or_else(|| anyhow!("missing packages"))?;

    // First pass — collect names of all workspace members (so we can detect
    // intra-workspace dep edges by name in the second pass).
    let mut workspace_names: HashSet<String> = HashSet::new();
    for pkg in packages {
        let id = pkg["id"].as_str().unwrap_or("");
        if !member_ids.contains(id) {
            continue;
        }
        if let Some(name) = pkg["name"].as_str() {
            workspace_names.insert(name.to_string());
        }
    }

    // Second pass — build CrateInfo for each publishable crate.
    let mut crates: BTreeMap<String, CrateInfo> = BTreeMap::new();
    for pkg in packages {
        let id = pkg["id"].as_str().unwrap_or("");
        if !member_ids.contains(id) {
            continue;
        }
        let name = pkg["name"]
            .as_str()
            .ok_or_else(|| anyhow!("package missing name"))?
            .to_string();

        // Skip workspace-internal crates.
        if NEVER_PUBLISH.contains(&name.as_str()) {
            continue;
        }
        // Skip explicit `publish = false` (cargo metadata emits "publish": []).
        if pkg.get("publish") == Some(&Value::Array(vec![])) {
            continue;
        }

        let version = pkg["version"]
            .as_str()
            .ok_or_else(|| anyhow!("{name}: missing version"))?
            .to_string();

        // Collect intra-workspace dep edges, deduplicated. `dev` deps are
        // intentionally excluded (they don't affect publish ordering — they're
        // present only when running tests, and `cargo publish` does not require
        // them resolvable on the registry side). `normal` and `build` deps DO
        // affect ordering.
        let mut deps_set: HashSet<String> = HashSet::new();
        if let Some(dep_arr) = pkg["dependencies"].as_array() {
            for d in dep_arr {
                let dname = match d["name"].as_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                let kind = d["kind"].as_str(); // null=normal, "dev", "build"
                if kind == Some("dev") {
                    continue;
                }
                if !workspace_names.contains(&dname) {
                    continue;
                }
                if NEVER_PUBLISH.contains(&dname.as_str()) {
                    continue;
                }
                deps_set.insert(dname);
            }
        }
        let mut deps_in_workspace: Vec<String> = deps_set.into_iter().collect();
        deps_in_workspace.sort(); // deterministic

        crates.insert(
            name.clone(),
            CrateInfo {
                name,
                version,
                deps_in_workspace,
            },
        );
    }

    // Kahn's algorithm with alphabetical tiebreak for determinism.
    let mut in_degree: BTreeMap<String, usize> = crates
        .iter()
        .map(|(k, v)| (k.clone(), v.deps_in_workspace.len()))
        .collect();
    let mut order: Vec<CrateInfo> = Vec::with_capacity(crates.len());

    loop {
        // Pick the smallest-named crate whose in-degree is 0.
        let next: Option<String> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .next(); // BTreeMap iterates in key order ⇒ alphabetical tiebreak
        let Some(name) = next else { break };
        in_degree.remove(&name);
        let info = crates
            .get(&name)
            .ok_or_else(|| anyhow!("internal: crate {name} missing from map"))?
            .clone();
        order.push(info);

        // Decrement in-degree of every still-pending crate that depends on `name`.
        for c in crates.values() {
            if !c.deps_in_workspace.contains(&name) {
                continue;
            }
            if let Some(d) = in_degree.get_mut(&c.name) {
                *d = d.saturating_sub(1);
            }
        }
    }

    if !in_degree.is_empty() {
        bail!(
            "cyclic publish graph: {:?}",
            in_degree.keys().collect::<Vec<_>>()
        );
    }

    Ok(order)
}

/// Check whether `name` at `version` is already on crates.io.
///
/// Implemented via `cargo search <name> --limit 1`; the output line takes the
/// shape `name = "version"  # description`. A match means we've already
/// consumed that version slot — `cargo publish` would fail with 422, so we
/// idempotently skip.
///
/// Network failure ⇒ assume not-yet-published + warn; the user can pass
/// `--skip <crate>` to opt out.
fn is_published_at_version(name: &str, version: &str) -> Result<bool> {
    let out = Command::new("cargo")
        .args(["search", name, "--limit", "1"])
        .output()
        .context("cargo search")?;
    if !out.status.success() {
        eprintln!("[warn] cargo search failed for {name}; assuming not-yet-published");
        return Ok(false);
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let needle = format!("{name} = \"{version}\"");
    Ok(stdout.contains(&needle))
}

/// Pitfall 4 — poll the crates.io index until `name@version` is visible.
///
/// `cargo publish` returns success once the upload is accepted, but the
/// sparse index may take several seconds to surface the new version. The next
/// crate's `cargo publish --dry-run` will fail to resolve its dep on this
/// crate until the index has caught up. Cap at POLL_TIMEOUT (5 min) and bail
/// loudly so the operator can investigate (network issue, registry outage,
/// etc.) rather than silently corrupt the publish sequence.
fn wait_for_index(name: &str, version: &str) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < POLL_TIMEOUT {
        if is_published_at_version(name, version)? {
            println!("[ok] {name}@{version} visible on index");
            return Ok(());
        }
        thread::sleep(POLL_INTERVAL);
    }
    bail!("timed out waiting for {name}@{version} index propagation (>{POLL_TIMEOUT:?})")
}

/// Spawn `args[0]` with `args[1..]`, inherit stdio, return Err on non-zero exit.
fn run(args: &[&str]) -> Result<()> {
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()
        .with_context(|| format!("spawn {:?}", args))?;
    if !status.success() {
        bail!("command {:?} exited with {}", args, status);
    }
    Ok(())
}
