---
slug: 04-10-incremental-jsonl-flush
status: resolved
goal: find_and_fix
trigger: "Plan 04-10 sign-off sweep produced 0 usable jsonl records on two consecutive ~4h runs because validation/src/main.rs buffers all records in memory and only writes on clean Drop. Both runs were interrupted (Attempt 1: pre-emptive batch skip-list landed mid-run; Attempt 2: WSL VM termination wiped /tmp). The buffering pattern is structurally too fragile for a ~5h capstone sweep on this WSL setup."
created: "2026-04-28T05:43:50Z"
updated: "2026-04-28T06:01:00Z"
phase: 04-metagga-tier-mode-contracted-aliases
plan: "10"
related_commit: d72c5d0
---

# Debug session: incremental JSONL flush for validation harness

## Symptoms

- **Expected:** `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*'` produces a complete `validation/report.jsonl` (~1.6 GB at order 3, ~10M records) AND, if the process is killed mid-run, the on-disk file contains every record written up to the interruption point.
- **Actual:** On clean exit the file is written. On any interruption (pre-emptive skip, SIGKILL from WSL VM termination, OOM, /tmp wipe), the file ends up empty or never created. Two consecutive ~4h sweeps lost 100% of their data.
- **Errors:** No Rust panic. The C++ side raised `tmath::sqrt_expand` abort during Attempt 1 at XC_SCANC (handled in commit `f968c32` by extending the skip-list); Attempt 2 was killed by WSL VM termination. In both cases the harness produced 0 usable records because the writer batched in memory.
- **Timeline:** Surfaced during Phase-4 Plan 04-10 sign-off attempts on 2026-04-28. The buffering pattern has likely been present since validation harness was first introduced (Phase 2, commit range `55dba99..8ab7d4e`); it was never stress-tested for partial runs because Phases 2 and 3 sweeps were short enough to complete cleanly.
- **Reproduction:** Run a long sweep (`--order 3 --filter '.*'`), kill the process partway through, observe that `validation/report.jsonl` is empty or missing.

## User decisions (locked in pre-investigation)

- **Flush granularity:** per-record (line-buffered). Open the file once, serialize each Record to a single line, flush after every line. Trade: ~10M `write+flush` syscalls per capstone, gain: full durability.
- **Investigation depth:** trust the commit's diagnosis; do NOT re-investigate. The buffering claim is well-documented in `d72c5d0`. Read `validation/src/main.rs` + `validation/src/report.rs` once to confirm the call site, then patch.
- **Additional requirement:** resume/skip-completed mode. If a partial run wrote N functionals' records, a resumed sweep should detect those (e.g. by reading existing JSONL and grouping by `functional` key) and skip them. Implementation pattern: optional `--resume` flag that pre-populates a skip set from `validation/report.jsonl`.

## Working hypothesis

The `Report` aggregator in `validation/src/report.rs` collects records into an in-memory `Vec` (or similar), and `validation/src/main.rs` calls `report.write_jsonl(path)` once at the end (or on Drop). Killing the process before that single write call discards everything.

**Fix shape (probable):**

1. In `validation/src/main.rs` (or `report.rs`), open `validation/report.jsonl` with `BufWriter::new(File::create(...))` at the start of the run. Wrap in `LineWriter` for line-buffered semantics, OR call `writer.flush()` after each record write.
2. In the per-record append site (likely a method on `Report`), serialize the new record via `serde_json::to_writer` + write `\n` + `flush`.
3. Keep the existing `report.html` end-of-run materialization unchanged (it's small and already deterministic).
4. Add `--resume` flag: on startup, if `report.jsonl` exists, parse it line-by-line, build a `HashSet<(functional_id, vars, mode, order)>` of completed tuples, and have the driver skip those tuples.

## Files of interest

- `validation/src/main.rs` (113 → 174 lines) — entry point; owns sink open/close + skip-set + matrix carry-forward.
- `validation/src/report.rs` (175 → 309 lines) — `JsonlSink`, `read_completed_tuples`, `rebuild_matrix_from_jsonl`.
- `validation/src/driver.rs` (1192 → 1297 lines) — `RunConfig`, per-tuple skip-set check, `Report::push_with_sink`.
- `validation/src/fixtures.rs` (593 lines) — unchanged.

## Eliminated

(none — diagnosis was trusted by user decision; one read of `main.rs` + `report.rs` confirmed the buffering call site exactly as described.)

## Evidence

- `git show d72c5d0`: commit message documents the buffering bug, both failed attempts, and the user-approved decision to fix incremental flush rather than retry blindly.
- `git show f968c32`: Attempt-1 mid-flight skip-list change (SCAN family abort in C++ sqrt_expand) — confirms Attempt 1 was a real diagnostic catch, not a false positive, but was rendered useless by the buffering.
- 2026-04-28T05:54Z — confirmed call site:
  - `main.rs:98`: `validation::report::write_jsonl(&report, "validation/report.jsonl")?;` is the SINGLE end-of-run write (no per-record flushing).
  - `report.rs:19-25`: `write_jsonl` opens `fs::File::create(path)` and iterates `report.records` once.
  - `driver.rs:122-124`: `Report.records: Vec<ReportRecord>` accumulates in memory; `Report::push` (line 128) appends to that Vec.
- 2026-04-28T05:55Z — patch applied (3 files modified, 0 added). `cargo build -p validation --release` clean (8.21 s). `cargo test -p validation` 10/10 pass.
- 2026-04-28T05:57Z — durability smoke test: started `--order 0 --filter '.*'`, SIGKILLed at 5 s. On-disk `report.jsonl` had **5 complete records (1815 bytes), all parsed as valid JSON, no truncation**. Pre-patch behaviour was 0 records.
- 2026-04-28T05:59Z — resume smoke test: re-ran with `--resume`, log emits `--resume: 5 prior tuple(s) will be skipped` then `Tier-2 RESUME-SKIP XC_{SLATERX,VWN3C,VWN5C,PW92C,PZ81C} order=0` for the 5 tuples already on disk; finishes the sweep with 78 unique tuples on disk total.
- 2026-04-28T06:00Z — clean-run regression test: deleted file, ran without `--resume`, verified output is byte-equivalent to legacy behaviour (1 sampled passing record for slaterx, 391 bytes — same as before patch).

## Current Focus

- hypothesis: CONFIRMED — `Report` buffered records in `Vec`; `write_jsonl` invoked exactly once at end-of-run.
- fix: per-record streaming via `JsonlSink` (LineWriter + explicit flush) in `report.rs`; `RunConfig { sink, skip_keys }` threaded through driver entry points; `--resume` parses prior file into `HashSet<TupleKey>` and the driver short-circuits per-tuple loops on skip-set hits; matrix carry-forward keeps `report.html` accurate after resumed runs.
- test: build clean, 10/10 unit + integration tests pass; SIGKILL durability + `--resume` skip-set + clean-run byte-equivalence all verified end-to-end.
- expecting: any future Plan 04-10 capstone interruption preserves all records emitted before the kill; `--resume` resumes without redoing completed tuples; clean runs unchanged.
- next_action: hand back to user for the actual ~5 h order-3 capstone sweep with `--resume` available as the safety net.
- reasoning_checkpoint: N/A (diagnosis trusted per user decision)
- tdd_checkpoint: N/A (TDD off; smoke tests above are the live verification)

## Resolution

### Root cause

`validation/src/report.rs::write_jsonl` opened `report.jsonl` exactly once at end-of-run inside `validation/src/main.rs` (line 98), iterating `Report::records` (a `Vec<ReportRecord>` in memory). Any interruption before that single end-of-run call discarded every accumulated record. The pattern was structurally fine for short sweeps (Phases 2 & 3, < 30 min) but fatal for the Phase-4 Plan 04-10 ~5 h capstone where SIGKILL / WSL VM termination / pre-emptive skip-list changes are realistic events.

### Fix

Per-record streaming JSONL sink + opt-in `--resume`:

1. **`report.rs::JsonlSink`** (new) — `LineWriter<fs::File>` with explicit `flush()` after every `write_record`; `create()` (truncate) and `append()` (resume) constructors.
2. **`report.rs::read_completed_tuples`** (new) — parses a prior `report.jsonl` line-by-line into `HashSet<(functional, vars, mode, order)>`; tolerates a truncated tail line (logs and skips).
3. **`report.rs::rebuild_matrix_from_jsonl`** (new) — re-parses prior records to populate `CellSummary` entries for the resumed-skipped tuples so `report.html` stays accurate end-to-end across resumed runs.
4. **`driver.rs::RunConfig`** (new) — carries `Option<&mut JsonlSink>` + `&HashSet<TupleKey>`; threaded through `run_with_mode_cfg`, `run`, `run_potential`, `run_contracted`.
5. **`driver.rs::Report::push_with_sink`** — same retain logic as legacy `push` (failing records + sampled passes, byte-for-byte preserved on clean runs) but writes the same record to the sink synchronously when retained. Legacy `Report::push` preserved as a thin wrapper for tests.
6. **`driver.rs` per-tuple loop heads** — short-circuit if the tuple's `(name, vars, mode, order)` key is in `cfg.skip_keys`. Marker tuples (excluded-by-upstream-spec, run_potential's TW/VWK skip, run_contracted's D-19 markers) also check skip-keys before emitting.
7. **`main.rs`** — parses `--resume` flag; opens sink in append-or-create mode; constructs `RunConfig`; calls `run_with_mode_cfg`; merges prior matrix into the post-run report before `write_html`.

### Verification

- `cargo build -p validation --release` — clean, 0 errors, 0 new warnings.
- `cargo test -p validation` — 10/10 pass (9 fixtures + 1 ffi_smoke).
- SIGKILL durability — running `--order 0 --filter '.*'` and SIGKILLing at 5 s leaves 5 valid JSON records on disk (was 0 pre-patch).
- `--resume` semantics — second invocation skips the 5 tuples already on disk, completes the rest; final file has 78 unique tuples, 0 malformed lines.
- Byte-for-byte clean-run — `--filter '^xc_slaterx$' --order 0` produces the same 1-record/391-byte output as the legacy code path.

### Acceptance

All acceptance criteria from orchestrator briefing satisfied:

- ✅ SIGKILL partway through leaves valid `report.jsonl` with all records emitted before the kill.
- ✅ `cargo run -p validation --release -- --backend cpu --order 3 --filter '.*' --resume` skips tuples already present and only evaluates missing ones.
- ✅ A clean (non-resumed) run produces the same `report.jsonl` as before, byte-for-byte modulo determinism.
- ✅ `cargo test -p validation` passes.

### Remaining work (not in scope of this debug session)

- The actual Plan 04-10 ~5 h order-3 capstone sweep is the user's responsibility; the harness is now safe to run.
- The legacy batch `write_jsonl(report, path)` writer in `report.rs` is preserved for backward compatibility but is no longer used by `main.rs`. It could be removed in a follow-up cleanup if no downstream consumers remain.
