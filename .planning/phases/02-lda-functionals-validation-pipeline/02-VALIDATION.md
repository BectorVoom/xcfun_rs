---
phase: 2
slug: lda-functionals-validation-pipeline
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-18
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `approx` 0.5.x |
| **Config file** | Cargo.toml workspace `[dev-dependencies]` |
| **Quick run command** | `cargo test -p xcfun-functionals --lib` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xcfun-functionals --lib`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | Status |
|---------|------|------|-------------|-----------|-------------------|--------|
| 02-01-01 | 01 | 1 | LDA-01 | unit | `cargo test -p xcfun-functionals slaterx` | ⬜ pending |
| 02-01-02 | 01 | 1 | LDA-02 | unit | `cargo test -p xcfun-functionals vwn3c` | ⬜ pending |
| 02-01-03 | 01 | 1 | LDA-03 | unit | `cargo test -p xcfun-functionals vwn5c` | ⬜ pending |
| 02-01-04 | 01 | 1 | LDA-04 | unit | `cargo test -p xcfun-functionals pz81c` | ⬜ pending |
| 02-01-05 | 01 | 1 | LDA-05 | unit | `cargo test -p xcfun-functionals pw92c` | ⬜ pending |
| 02-02-01 | 02 | 2 | EVAL-01..08 | integration | `cargo test -p xcfun-eval` | ⬜ pending |
| 02-02-02 | 02 | 2 | LDA-06 | integration | `cargo test -p xcfun-eval alias` | ⬜ pending |
| 02-03-01 | 03 | 2 | VAL-01..04 | integration | `cargo test --workspace -- accuracy` | ⬜ pending |
| 02-03-02 | 03 | 2 | EVAL-05 | integration | `cargo test -p xcfun-gpu batch` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `approx` dev-dependency added to xcfun-functionals and xcfun-eval Cargo.toml
- [ ] cubecl-cpu dev-dependency added to xcfun-gpu Cargo.toml for CpuRuntime testing

*Existing xcfun-core test infrastructure covers AD engine requirements.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
