---
phase: 6
slug: gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Generated from `06-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `cargo nextest` (workspace already configured); validation harness binary `cargo run -p validation` |
| **Config file** | Workspace `Cargo.toml` (members include `validation`); per-crate `Cargo.toml` for feature gating |
| **Quick run command** | `cargo nextest run --workspace --tests -j 4` |
| **Full suite command** | `cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 --filter '.*' && cargo nextest run --workspace --tests` |
| **Phase tier-3 gate** | `cargo run -p validation --release --features hip -- --backend rocm --order 3 --jobs 18 --filter '.*'` |
| **Estimated runtime** | ~12 min full sweep (CPU tier-3) + ~5 min ROCm tier-3 + ~3 min Wgpu tier-3 |

---

## Sampling Rate

- **After every task commit:** Run `cargo nextest run -p <crate-touched> --tests` plus `cargo nextest run -p xcfun-core --tests` if `XcError` was modified.
- **After every plan wave:** Run `cargo nextest run --workspace --tests && cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga'` (~10 min — order-2 LDA+GGA quick sweep).
- **Before `/gsd-verify-work`:** Full suite must be green AND tier-3 gate (CPU 1e-12 strict + ROCm 1e-13 strict + Wgpu 1e-9 with `--exclude-erf`) GREEN.
- **Max feedback latency:** ~30 s for `cargo nextest run -p <crate>`; ~10 min for the per-wave order-2 sweep.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-00-01 | 06-00 | 1 | KER-03 (AD N≥4) | — | N/A | unit | `cargo nextest run -p xcfun-ad --test golden_compose_n4 --test golden_multo_n4 --test golden_compose_n5 --test golden_multo_n5 --test golden_compose_n6 --test golden_multo_n6` | ❌ W0 | ⬜ pending |
| 06-00-02 | 06-00 | 1 | KER-03 (D-11 erf) | — | N/A | unit | `cargo nextest run -p xcfun-ad --test erf_taylor_chain` | ❌ W0 | ⬜ pending |
| 06-00-03 | 06-00 | 1 | KER-06 (D-10 tau) | — | guard preserves f64 monotonic | unit | `cargo nextest run -p xcfun-eval --test tpss_tau_clamp` (pre-06-01 path) | ❌ W0 | ⬜ pending |
| 06-00-04 | 06-00 | 1 | ACC-04 (D-04 mpmath) | T-06-MPMATH | mpmath sidecar produces deterministic JSONL | integration | `cargo xtask regen-mpmath-fixtures --check` | ❌ W0 | ⬜ pending |
| 06-01-01 | 06-01 | 2 | KER-01..06 (D-08 split) | — | N/A | compile + unit | `cargo build --workspace && cargo nextest run --workspace --tests` (post `git mv`) | ❌ W0 | ⬜ pending |
| 06-02-01 | 06-02 | 3 | GPU-01 | — | N/A | unit | `cargo nextest run -p xcfun-gpu --test batch_api_shape` | ❌ W0 | ⬜ pending |
| 06-02-02 | 06-02 | 3 | GPU-02 (D-07) | — | priority chain monotone | unit | `cargo nextest run -p xcfun-gpu --test auto_backend_priority` | ❌ W0 | ⬜ pending |
| 06-02-03 | 06-02 | 3 | GPU-04 (D-15) | T-06-OOM | growth never exceeds 2× peak demand | unit | `cargo nextest run -p xcfun-gpu --test buffer_pool_growth` | ❌ W0 | ⬜ pending |
| 06-02-04 | 06-02 | 3 | GPU-06 (D-13/13-A) | T-06-WGPU-F32 | typed error blocks silent f32 downgrade | compile + unit | `cargo nextest run -p xcfun-core --test xcerror_copy_invariant && cargo nextest run -p xcfun-gpu --features wgpu --test wgpu_no_f64` | ❌ W0 | ⬜ pending |
| 06-02-05 | 06-02 | 3 | KER-04 / KER-06 | — | tier-3 CPU 10k-grid 1e-13 | integration | `cargo run -p validation --release -- --backend cpu --tier 3 --order 3 --filter '.*'` | ❌ W0 | ⬜ pending |
| 06-03-01 | 06-03 | 4 | GPU-07 (D-05) | T-06-ROCM-DRIFT | ROCm tier-3 at 1e-13 vs CPU | integration | `cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '.*'` | ❌ W0 (precondition: ROCm runtime installed) | ⬜ pending |
| 06-04-01 | 06-04 | 4 | GPU-03 + GPU-08 | — | feature compile + Wgpu tier-3 1e-9 | compile gate + integration | `cargo build -p xcfun-gpu --features hip --features cuda --features wgpu && cargo run -p validation --release --features wgpu -- --backend wgpu --tier 3 --order 3 --filter '.*' --exclude-erf` | ❌ W0 | ⬜ pending |
| 06-05-01 | 06-05 | 5 | RS-08 (D-14) | — | threshold + env override correct | unit | `cargo nextest run -p xcfun-rs --test eval_vec_threshold` | ❌ W0 | ⬜ pending |
| 06-05-02 | 06-05 | 5 | GPU-05 (D-07) | — | ERF auto-fallback to Cpu on Wgpu | integration | `cargo nextest run -p xcfun-gpu --features wgpu --test erf_fallback` | ❌ W0 | ⬜ pending |
| 06-06-01 | 06-06 | 6 | RS-07 (D-12) | T-06-ALLOC | strict 0 allocs/eval | unit | `cargo nextest run -p xcfun-rs --test zero_alloc_strict` | ❌ W0 | ⬜ pending |
| 06-06-02 | 06-06 | 6 | RS-10 (D-17) | T-06-LEAK | no Box::leak on set | unit | `cargo nextest run -p xcfun-rs --test no_leak_on_set` | ❌ W0 | ⬜ pending |
| 06-06-03 | 06-06 | 6 | KER-04 (D-18) | — | b3lyp/camb3lyp/bp86 dispatch correct | integration | `cargo nextest run -p xcfun-rs --test lda_gga_alias_dispatch` | ❌ W0 | ⬜ pending |
| 06-N1-01 | 06-N1 | 7 | ACC-01..04 | — | inherited Phase-3 forwards tighten to 1e-13 | integration | `cargo run -p validation --release -- --backend cpu --filter 'pbeintc\|beckesrx\|p86c\|pw91c\|spbec\|apbec\|b97.*c\|pw91k' --tier 3 --order 3` | ❌ W0 | ⬜ pending |
| 06-N2-01 | 06-N2 | 7 | ACC-04 (D-03) | — | 20 `excluded_by_upstream_spec` mpmath validation | integration | `cargo run -p validation --release -- --reference mpmath --filter 'br.\|csc\|blocx\|scan.*\|tw\|vwk\|pbelocc\|zvpbe(sol\|int)c' --tier 3 --order 3` | ❌ W0 | ⬜ pending |
| 06-N3-01 | 06-N3 | 7 | ACC-04 | — | small-magnitude AD residuals tighten post libm-hybrid | integration | `cargo run -p validation --release -- --backend cpu --filter 'm05.*\|m06.*\|b97.*\|lypc\|pw92c\|pbec\|optx' --tier 3 --order 3` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Threat refs: T-06-ROCM-DRIFT (ROCm/CPU intrinsic drift), T-06-WGPU-F32 (silent Wgpu f32 downgrade), T-06-OOM (unbounded buffer pool growth), T-06-ALLOC (zero-alloc contract regression), T-06-LEAK (Box::leak on hot path), T-06-MPMATH (Python sidecar reproducibility). Threat model formalised in each PLAN.md `<threat_model>` block.*

---

## Wave 0 Requirements

- [ ] `crates/xcfun-ad/tests/golden_compose_n4.rs`, `golden_compose_n5.rs`, `golden_compose_n6.rs` — covers AD N≥4 compose (Plan 06-00 Task 1)
- [ ] `crates/xcfun-ad/tests/golden_multo_n4.rs`, `golden_multo_n5.rs`, `golden_multo_n6.rs` — covers AD N≥4 multo (Plan 06-00 Task 1)
- [ ] `crates/xcfun-ad/tests/erf_taylor_chain.rs` — covers D-11 erf_precise_taylor (Plan 06-00 Task 2)
- [ ] `crates/xcfun-eval/tests/tpss_tau_clamp.rs` (or post-06-01: `crates/xcfun-kernels/tests/`) — covers D-10 (Plan 06-00 Task 3)
- [ ] `crates/xcfun-gpu/tests/batch_api_shape.rs`, `batch_kernel_smoke.rs`, `auto_backend_priority.rs`, `buffer_pool_growth.rs`, `erf_fallback.rs`, `wgpu_no_f64.rs` — covers GPU-01..06 (Plans 06-02 / 06-03 / 06-04 / 06-05)
- [ ] `crates/xcfun-rs/tests/eval_vec_threshold.rs`, `zero_alloc_strict.rs`, `no_leak_on_set.rs`, `lda_gga_alias_dispatch.rs` — covers RS-08 + D-12/D-17/D-18 (Plans 06-05 / 06-06)
- [ ] `xtask/src/bin/regen_mpmath_fixtures.rs` + `xtask/mpmath_eval/__main__.py` + JSONL fixtures — Plan 06-00 Task 4
- [ ] Validation driver extensions: `--reference {cpp, mpmath}` flag + `--backend rocm` flag + `--exclude-erf` flag — Plan 06-02 driver extension
- [ ] xtask gate scope updates: `check-cubecl-pin` (5 crates: cubecl, cubecl-cpu, cubecl-hip, cubecl-cuda, cubecl-wgpu); `check-no-mul-add` (add `xcfun-kernels/src/functionals/**/*.rs`); `check-no-anyhow` (add `xcfun-kernels` + `xcfun-gpu` to enforced set)
- [ ] `crates/xcfun-gpu/README.md` — RDNA-2 `HSA_OVERRIDE_GFX_VERSION=10.3.0` note + Apple Silicon caveat + env vars (`XCFUN_FORCE_BACKEND`, `XCFUN_MIN_BATCH_SIZE`)
- [ ] `crates/xcfun-kernels/Cargo.toml` (Wave 0 of Plan 06-01) — workspace member registration
- [ ] `crates/xcfun-gpu/Cargo.toml` updates — feature flags `default = ["cpu"]`, `hip`, `cuda`, `wgpu`; promote from `workspace.exclude` to `workspace.members`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ROCm tier-3 GREEN on RDNA-2 hardware | GPU-07 (D-05) | Requires AMD GPU + `/opt/rocm` runtime install | `export HSA_OVERRIDE_GFX_VERSION=10.3.0 && cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3 --filter '.*'` — must report 0 failing functionals at 1e-13 |
| CUDA tier-3 best-effort (cloud-CI) | GPU-07 (D-06) | No NVIDIA hardware in dev env | Cloud-CI workflow `cuda-tier-3.yml` (out of scope for Phase 6 sign-off; document expected workflow) |
| Apple Silicon Metal-via-Wgpu refusal-on-no-f64 | GPU-08 (D-06) | Requires Apple Silicon hardware | On Apple Silicon: `cargo run -p validation --release --features wgpu -- --backend wgpu --tier 3 --order 3 --filter '.*' --exclude-erf` should report `XcError::WgpuNoF64` at Batch::open and fall back to CPU per `auto_backend()` |
| mpmath fixture reproducibility | ACC-04 (D-03/D-04) | Python sidecar; `mpmath==1.4.x` on `python3 ≥ 3.12`; document floor in xtask README | Re-run `cargo xtask regen-mpmath-fixtures` on a clean checkout — `--check` should report no diff |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify command or Wave 0 dependency declared
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (verified by per-plan task ordering)
- [ ] Wave 0 covers all MISSING test-file references in the per-task map
- [ ] No watch-mode flags in any command (workspace nextest is one-shot)
- [ ] Feedback latency < 30 s for crate-local nextest; < 10 min for per-wave order-2 sweep; < 30 min for full tier-3 phase gate
- [ ] `nyquist_compliant: true` set in frontmatter once Wave 0 is built and the per-task map's "File Exists" column is fully ✅
- [ ] Manual-only verifications have explicit sign-off notes captured in `06-VERIFICATION.md` at phase exit

**Approval:** pending
