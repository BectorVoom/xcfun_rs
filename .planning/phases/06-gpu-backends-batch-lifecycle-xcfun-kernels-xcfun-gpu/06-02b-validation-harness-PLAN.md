---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 02b
type: execute
wave: 4
depends_on:
  - 06-02a
files_modified:
  - validation/Cargo.toml
  - validation/src/main.rs
  - validation/src/driver.rs
  - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
  - .planning/REQUIREMENTS.md
autonomous: true
requirements:
  - GPU-02
  - GPU-06
must_haves:
  truths:
    - "validation harness gains --tier {2|3} flag (default 2 = preserves Phase 2-5 behaviour) and --reference {cpp|mpmath} flag (default cpp; Plan 06-N2 fills the mpmath branch); both parsed via the existing parse_arg helper."
    - "validation harness gains --exclude-erf flag — boolean filter that skips functionals with Dependency::ERF (consumed by Plan 06-04 for the Wgpu tier-3 1e-9 sweep per GPU-08)."
    - "validation harness gains --backend {cpu,rocm,cuda,wgpu,metal} dispatch — the cpu arm of run_tier3 is implemented in this plan (driver only — actual sweep gating moves to Plan 06-05 per B-4); rocm/cuda/wgpu arms bail with helpful error messages directing users to enable the corresponding feature flag (Plans 06-03/06-04 wire concrete arms)."
    - "B-5 (revision-1) documentation alignment: CONTEXT.md D-06 amended with a `### Amended on revision-1` block stating Metal is reached via cubecl-wgpu (NOT cubecl-metal); the original D-06 text is preserved above the amendment for audit-trail."
    - "B-5 (revision-1) REQUIREMENTS.md GPU-02 wording updated: `Backend enum (Cpu, Rocm, Cuda, Metal, Wgpu)` (was: `Cpu, Cuda, Wgpu`); GPU-07 wording updated: `Tier-3 parity on ROCm` (was: `on CUDA`)."
    - "BackendTag (xcfun-core) and Backend (xcfun-gpu) From/Into bridge already in place from 06-02a; this plan does NOT duplicate it."
  artifacts:
    - path: "validation/src/main.rs"
      provides: "--tier / --reference / --exclude-erf / --backend CLI parsing"
      contains: "--tier\\|--reference\\|--exclude-erf"
    - path: "validation/src/driver.rs"
      provides: "run_tier3 driver skeleton — cpu arm implemented; rocm/cuda/wgpu arms bail with feature-flag hint"
      contains: "run_tier3\\|Backend::Cpu"
    - path: "validation/Cargo.toml"
      provides: "xcfun-gpu dep + hip/cuda/wgpu/metal feature forwards"
      contains: "xcfun-gpu"
    - path: ".planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md"
      provides: "D-06 amendment block (Metal-via-Wgpu correction; original text preserved)"
      contains: "### Amended on revision-1"
    - path: ".planning/REQUIREMENTS.md"
      provides: "GPU-02 / GPU-07 wording updates (Backend enum 5 variants; ROCm primary)"
      contains: "Cpu, Rocm, Cuda, Metal, Wgpu"
  key_links:
    - from: "validation/src/main.rs::parse_arg --tier"
      to: "validation/src/driver.rs::run_tier3"
      via: "tier 3 → run_tier3 dispatch"
      pattern: "run_tier3"
    - from: "validation/Cargo.toml"
      to: "crates/xcfun-gpu (cpu feature)"
      via: "[dependencies] xcfun-gpu = { ..., features = [\"cpu\"] }"
      pattern: "xcfun-gpu"
---

<objective>
Sibling plan to 06-02a (Wave 3). The original Plan 06-02 from the first planning iteration was split per W-1 (revision-1) into:
- **06-02a** — xcfun-gpu skeleton (Backend enum + Batch<R> + buffer pool + WgpuNoF64 + CudaNoF64 + BackendTag + Functional::settings_generation).
- **06-02b** (this plan) — validation harness CLI extension + B-5 documentation alignment task.

Per revision-1 B-4, the actual KER-06 tier-3 CPU 10k-grid 1e-13 sweep is OWNED by Plan 06-05 (it fits more naturally next to the RS-08 `eval_vec` GPU dispatch wiring). This plan ships the *driver skeleton* (`run_tier3` with the Cpu arm implemented) so that 06-05 can run it without introducing new validation/* code; 06-05 just calls the new CLI flags.

Three deliverables:

1. **Validation CLI extension** — `validation/src/main.rs` parses three new flags:
   - `--tier {2|3}` — tier-2 (cc vs Rust 1e-12; existing default) or tier-3 (Batch<R> vs scalar 1e-13).
   - `--reference {cpp|mpmath}` — ground truth source. Default `cpp`. Plan 06-N2 fills the mpmath branch.
   - `--exclude-erf` — boolean filter to skip ERF-bearing functionals (Plan 06-04 uses for Wgpu 1e-9 sweep per GPU-08).
   Plus the `--backend` flag (already exists for Phase 2-5 with `cpu` only) gains parser entries for `rocm | cuda | wgpu | metal`.

2. **`run_tier3` driver skeleton** — `validation/src/driver.rs` gains a `run_tier3(backend, order, jobs, filter, exclude_erf)` function. The `Backend::Cpu` arm is fully implemented (uses `Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host` from 06-02a; iterates the Phase 2 stratified xoshiro 10k grid; computes per-record rel-err vs scalar `Functional::eval`; reports failures at strict 1e-13). The `Backend::Rocm | Backend::Cuda | Backend::Wgpu | Backend::Metal` arms return `anyhow::bail!(\"--backend X requires --features Y (Plan 06-03/06-04)\")`. **Plan 06-05 owns the actual KER-06 sign-off command** (per revision-1 B-4); this plan ships the wiring without claiming KER-06 in `requirements:`.

3. **(B-5 revision-1) Documentation alignment task** — amend CONTEXT.md D-06 + REQUIREMENTS.md GPU-02 / GPU-07 to reflect the locked decisions:
   - CONTEXT.md D-06: replace the `cubecl-metal = =0.10.0-pre.3 (feature \`metal\`)` line with the corrected statement that Metal is reached via cubecl-wgpu's Metal adapter, with the `metal` feature being an alias of `wgpu` per RESEARCH §R-02 / Pitfall 9. Original text preserved under a `### Amended on revision-1` block for audit-trail.
   - REQUIREMENTS.md GPU-02: change `Backend enum (Cpu, Cuda, Wgpu)` to `Backend enum (Cpu, Rocm, Cuda, Metal, Wgpu)` matching CONTEXT.md D-05/D-07.
   - REQUIREMENTS.md GPU-07: change `Tier-3 parity on CUDA` to `Tier-3 parity on ROCm` (CUDA opt-in best-effort).

Purpose: Ship the validation-harness scaffolding consumed by 06-03 / 06-04 / 06-05 without bloating 06-02a's skeleton scope. Documentation alignment lands here because it requires the same workspace compile + test cycle as the validation harness (cargo run -p validation must still succeed after the doc edits — they're text-only but the executor verifies via tier-2 LDA+GGA quick sweep).

Output: validation CLI extended with --tier/--reference/--exclude-erf/--backend; `run_tier3` driver skeleton with Cpu arm implemented; CONTEXT.md D-06 amended; REQUIREMENTS.md GPU-02/GPU-07 updated.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02a-xcfun-gpu-skeleton-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/REQUIREMENTS.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@validation/src/main.rs
@validation/src/driver.rs
@validation/src/fixtures.rs
@validation/Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Validation harness --tier / --reference / --exclude-erf / --backend CLI extension + run_tier3 driver skeleton (cpu arm only)</name>
  <files>validation/src/main.rs, validation/src/driver.rs, validation/Cargo.toml</files>
  <read_first>
    - validation/src/main.rs (full file — current --backend, --order, --jobs, --filter parsing pattern)
    - validation/src/driver.rs (full file — current cc-FFI run + JSONL emit)
    - validation/src/fixtures.rs (10k-point xoshiro grid; reuse for tier-3 driver skeleton)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-03 / 06-04 / 06-N2" CLI extension
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Validation Architecture"
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md per-task map
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02a-xcfun-gpu-skeleton-PLAN.md (consumes Batch<CpuRuntime>::eval_vec_host)
  </read_first>
  <action>
**Step A — Add CLI flags to `validation/src/main.rs`:**

Locate the existing arg parser. Add three new flags + extend `--backend`:

```rust
let backend  = parse_arg(&args, "--backend").unwrap_or("cpu");
let tier: u32 = parse_arg(&args, "--tier").unwrap_or("2").parse().context("--tier must be 2 or 3")?;
let reference = parse_arg(&args, "--reference").unwrap_or("cpp");
let exclude_erf = args.iter().any(|a| a == "--exclude-erf");
let filter = parse_arg(&args, "--filter").unwrap_or(".*");
let order: u32 = parse_arg(&args, "--order").unwrap_or("2").parse().context("--order must be u32")?;
let jobs: usize = parse_arg(&args, "--jobs").map(|s| s.parse().unwrap()).unwrap_or_else(|| std::thread::available_parallelism().unwrap().get());

if tier == 3 {
    return run_tier3(&backend, order, jobs, filter, exclude_erf);
}
run_tier2(&backend, order, jobs, filter, reference, exclude_erf)
```

Update help text to list `--backend` accepted values: `cpu | rocm | cuda | wgpu | metal`.

**Step B — Implement `run_tier3` driver skeleton in `validation/src/driver.rs` (cpu arm only):**

```rust
/// Phase 6 — tier-3 cross-backend parity sweep skeleton.
/// CPU arm implemented in this plan (06-02b). ROCm/CUDA/Wgpu arms wired in 06-03/06-04.
/// KER-06 sign-off command + 17-functional bar OWNED by Plan 06-05 (revision-1 B-4).
pub fn run_tier3(backend: &str, order: u32, jobs: usize, filter: &str, exclude_erf: bool)
    -> anyhow::Result<()>
{
    use xcfun_gpu::{Backend, Batch};

    let backend_e = Backend::from_str(backend).expect("backend unrecognised");
    match backend_e {
        Backend::Cpu => {
            // Iterate the 10k stratified xoshiro grid (Phase 2 fixtures::stratified_xoshiro_grid_10k())
            // For each (functional, vars, mode, order) tuple in iter_tuples(filter, exclude_erf):
            //   1. Build Functional via xcfun_eval (NOT xcfun_rs facade; W-3 keeps Batch bound to xcfun_eval).
            //   2. Call Batch::<cubecl_cpu::CpuRuntime>::eval_vec_host(&fun, density_flat, density_pitch,
            //                                                          &mut batch_out, out_pitch, grid.len())?;
            //   3. Loop scalar fun.eval per-point; compute max_rel_err.
            //   4. If max_rel_err > 1e-13, accumulate as failure; emit JSONL record.
            // Return Ok(()) regardless — actual gating (0 failures across 17-clean set) lives in Plan 06-05.
            // ... see 06-02a interfaces for Batch<CpuRuntime>::eval_vec_host signature ...
            todo!("CPU arm body — uses Batch<cubecl_cpu::CpuRuntime>::eval_vec_host from 06-02a")
        }
        #[cfg(not(feature = "hip"))]
        Backend::Rocm => anyhow::bail!("--backend rocm requires --features hip (Plan 06-03)"),
        #[cfg(not(feature = "cuda"))]
        Backend::Cuda => anyhow::bail!("--backend cuda requires --features cuda (Plan 06-04)"),
        #[cfg(not(feature = "wgpu"))]
        Backend::Wgpu | Backend::Metal => anyhow::bail!("--backend {:?} requires --features wgpu (Plan 06-04)", backend_e),
        // Plans 06-03/06-04 fill the feature-enabled arms.
        _ => unreachable!("Plan 06-03/06-04 wires this arm"),
    }
}
```

The `todo!()` for the CPU arm is fine for THIS plan (the skeleton compiles); Plan 06-05 fills the body when implementing KER-06. Acceptance gate is `cargo build -p validation --release` succeeds + `--backend cpu --tier 3 --order 0 --filter '^slaterx$'` returns Ok(()) (or panics with `not yet implemented` from `todo!()` — acceptable).

**Step C — Update `validation/Cargo.toml`** to depend on xcfun-gpu and forward feature flags:

```toml
[dependencies]
xcfun-core    = { path = "../crates/xcfun-core" }
xcfun-eval    = { path = "../crates/xcfun-eval" }
xcfun-kernels = { path = "../crates/xcfun-kernels" }
xcfun-rs      = { path = "../crates/xcfun-rs" }
xcfun-gpu     = { path = "../crates/xcfun-gpu", features = ["cpu"] }   # 06-02b
cubecl-cpu    = { workspace = true }

[features]
default = []
hip  = ["xcfun-gpu/hip"]   # Plan 06-03
cuda = ["xcfun-gpu/cuda"]  # Plan 06-04
wgpu = ["xcfun-gpu/wgpu"]  # Plan 06-04
metal = ["xcfun-gpu/metal"]
```

**Step D — Verify driver compiles + Phase 2-5 tier-2 sweep still GREEN:**

```bash
cargo build -p validation --release
cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga' --jobs 4   # tier-2 default; preserves Phase 2-5 behaviour
cargo run -p validation --release -- --backend cpu --tier 3 --order 0 --filter '^slaterx$'  # tier-3 skeleton; CPU arm — may todo!() until 06-05
cargo run -p validation --release -- --backend rocm                                          # bails with "requires --features hip"
```

The first command MUST exit 0 (no regression from CLI extension). The third command MUST exit non-zero with the helpful error message.
  </action>
  <verify>
    <automated>cargo build -p validation --release && cargo run -p validation --release -- --backend cpu --order 2 --filter '^slaterx$' --jobs 2</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c '"--tier"\|--tier ' validation/src/main.rs` >= 1
    - `grep -c '"--reference"\|--reference ' validation/src/main.rs` >= 1
    - `grep -c '"--exclude-erf"\|--exclude-erf' validation/src/main.rs` >= 1
    - `grep -c "run_tier3" validation/src/driver.rs` >= 1
    - `grep -c "Backend::Cpu" validation/src/driver.rs` >= 1
    - `grep -c "xcfun-gpu" validation/Cargo.toml` >= 1
    - `cargo build -p validation --release` exits 0.
    - `cargo run -p validation --release -- --backend cpu --order 2 --filter '^slaterx$' --jobs 2` exits 0 (tier-2 default; no regression).
    - `cargo run -p validation --release -- --backend rocm` exits non-zero with error message containing "requires --features hip" or "Plan 06-03".
  </acceptance_criteria>
  <done>validation harness CLI extended with --tier/--reference/--exclude-erf flags + extended --backend; run_tier3 driver skeleton in place with Cpu arm scoped for Plan 06-05 KER-06 ownership; tier-2 default behaviour unchanged.</done>
</task>

<task type="auto">
  <name>Task 2: B-5 documentation alignment — amend CONTEXT.md D-06 + REQUIREMENTS.md GPU-02 / GPU-07</name>
  <files>.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md, .planning/REQUIREMENTS.md</files>
  <read_first>
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md (full file — find D-06 verbatim + D-05/D-07 for reference)
    - .planning/REQUIREMENTS.md (find GPU-02 + GPU-07 entries)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §R-02 + §"Pitfall 9" (Metal-via-Wgpu rationale)
  </read_first>
  <action>
**Step A — CONTEXT.md D-06 amendment:**

Locate the D-06 entry under `### GPU backend strategy`. Insert an `### Amended on revision-1` block IMMEDIATELY AFTER the original D-06 paragraph (preserving the original text above for audit-trail). The amendment block reads:

```markdown
### Amended on revision-1 (2026-04-30)

**D-06 correction:** `cubecl-metal` does NOT exist as a separate crate on crates.io (verified by RESEARCH §"Standard Stack" + Pitfall 9 + R-02). Metal is reached via `cubecl-wgpu`'s Metal adapter; the `metal` cargo feature in `xcfun-gpu` is an alias of `wgpu` (`metal = ["wgpu"]`). The original D-06 text above mentioning `cubecl-metal = "=0.10.0-pre.3"` is preserved for audit-trail but supersedes by this amendment. All Plan 06-02a / 06-04 implementation work uses the `metal = ["wgpu"]` alias model.
```

DO NOT modify the original D-06 text (it stays as-is for audit-trail; the amendment block clarifies the locked decision).

**Step B — REQUIREMENTS.md GPU-02 + GPU-07 wording updates:**

Locate the `### GPU Backend (Batch lifecycle)` section. Apply these edits:

```diff
- - [ ] **GPU-02**: `Backend` enum (`Cpu`, `Cuda`, `Wgpu`); `auto_backend()` selects CUDA if available, else Wgpu with f64, else CPU
+ - [ ] **GPU-02**: `Backend` enum (`Cpu`, `Rocm`, `Cuda`, `Metal`, `Wgpu`); `auto_backend()` priority chain per CONTEXT.md D-07 — `XCFUN_FORCE_BACKEND` > Rocm > Cuda > Metal-with-f64 > Wgpu-with-SHADER_F64 > Cpu
- - [ ] **GPU-07**: Tier-3 parity on CUDA — 10k-point grid within 1e-13 rel-err vs. CPU
+ - [ ] **GPU-07**: Tier-3 parity on ROCm (PRIMARY per CONTEXT.md D-05) — 10k-point grid within 1e-13 rel-err vs. CPU. CUDA + Metal are opt-in best-effort per D-06.
```

The wording matches CONTEXT.md D-05 (ROCm primary) + D-07 (priority order) + D-06 (CUDA + Metal opt-in).

**Step C — Verify no broken references and a minimal sanity build:**

```bash
# Verify the CONTEXT.md amendment block parses (markdown-only, no code execution):
grep -c "### Amended on revision-1" .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
# Should be >= 1.

# Verify REQUIREMENTS.md updated:
grep -c "Cpu, Rocm, Cuda, Metal, Wgpu" .planning/REQUIREMENTS.md
# Should be >= 1.
grep -c "Tier-3 parity on ROCm" .planning/REQUIREMENTS.md
# Should be >= 1.

# Sanity: workspace still builds (markdown edits never break compile, but defensive).
cargo build --workspace
```
  </action>
  <verify>
    <automated>grep -c "### Amended on revision-1" .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md && grep -c "Cpu, Rocm, Cuda, Metal, Wgpu" .planning/REQUIREMENTS.md && grep -c "Tier-3 parity on ROCm" .planning/REQUIREMENTS.md</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "### Amended on revision-1" .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` >= 1
    - `grep -c "metal.*alias of.*wgpu\|metal cargo feature.*alias" .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` >= 1
    - `grep -c "Cpu, Rocm, Cuda, Metal, Wgpu" .planning/REQUIREMENTS.md` >= 1
    - `grep -c "Tier-3 parity on ROCm" .planning/REQUIREMENTS.md` >= 1
    - The original D-06 paragraph in CONTEXT.md is preserved (NOT deleted) — `grep -c "cubecl-metal = " .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md` >= 1 (the original cubecl-metal mention remains).
  </acceptance_criteria>
  <done>CONTEXT.md D-06 amended with revision-1 correction block (audit-trail preserved); REQUIREMENTS.md GPU-02 + GPU-07 wording aligned with CONTEXT.md locked decisions D-05/D-06/D-07.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| validation harness ↔ xcfun-gpu (Cpu feature) | One-way; validation depends on xcfun-gpu with `cpu` feature only in this plan |
| Documentation edits ↔ existing locked decisions | CONTEXT.md edits preserve original text for audit; REQUIREMENTS.md edits match CONTEXT.md D-05/06/07 |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-DOC-DRIFT | medium | Stale doc references to `cubecl-metal` could confuse future executors | B-5 amendment makes the correction explicit and timestamped (revision-1 2026-04-30) |
| T-06-CLI-REGRESSION | medium | Phase 2-5 tier-2 sweep behaviour must be unchanged | Step D runs tier-2 default sweep; acceptance criteria assert no regression |
| T-06-DRIVER-STUB | low | run_tier3 cpu arm is a `todo!()` skeleton at 06-02b; could panic at runtime | Acceptable per B-4 (KER-06 owned by Plan 06-05); 06-02b ships only the API shape |
</threat_model>

<verification>
- All acceptance criteria GREEN per Tasks 1+2.
- Phase 2-5 tier-2 sweep at order 2 (cpu only) still GREEN — no regression from CLI extension.
- 06-CONTEXT.md D-06 amendment block exists; original text preserved.
- REQUIREMENTS.md GPU-02 / GPU-07 reflect CONTEXT.md locked decisions.
- Plans 06-03 / 06-04 / 06-05 / 06-N2 consume the new CLI flags (`--tier`, `--reference`, `--exclude-erf`, `--backend rocm/cuda/wgpu`) without further validation/* edits.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 4 advanced: validation harness now supports cross-backend tier-3 sweeps via `--backend` + `--tier 3` flags.
- Documentation alignment closes B-5 from revision-1 review (CONTEXT.md / REQUIREMENTS.md / D-05/06/07 consistency).
- Plan 06-05 unblocked for KER-06 sign-off command (per revision-1 B-4): can call `cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --filter '.*'` directly.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-02b-SUMMARY.md` documenting:
- validation CLI extended with --tier/--reference/--exclude-erf/--backend
- run_tier3 driver skeleton landed (Cpu arm scoped for Plan 06-05; rocm/cuda/wgpu arms bail with feature-flag hint)
- B-5 documentation alignment: CONTEXT.md D-06 amended; REQUIREMENTS.md GPU-02 / GPU-07 updated
- KER-06 explicitly NOT claimed in `requirements:` (per revision-1 B-4; ownership passes to 06-05)
</output>
</content>
</invoke>