---
phase: 1
slug: core-types-ad-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-17
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test -p xcfun-ad --lib && cargo test -p xcfun-core --lib` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xcfun-ad --lib && cargo test -p xcfun-core --lib`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | CORE-01 | — | N/A | unit | `cargo test -p xcfun-core density_vars` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | CORE-02..08 | — | N/A | unit | `cargo test -p xcfun-core enums` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | AD-01..02 | — | N/A | unit | `cargo test -p xcfun-ad ctaylor` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | AD-03..05 | — | N/A | unit | `cargo test -p xcfun-ad transcendentals` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 1 | AD-06..07 | — | N/A | unit | `cargo test -p xcfun-ad compose` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | AD-08..09 | — | N/A | integration | `cargo test -p xcfun-ad --test integration` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/xcfun-core/src/lib.rs` — crate skeleton with module declarations
- [ ] `crates/xcfun-ad/src/lib.rs` — crate skeleton with module declarations
- [ ] Workspace Cargo.toml with both crates as members

*Existing infrastructure: cargo test is built-in, no additional framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Numerical stability near zero | AD-09 | Edge case values need visual inspection of output ranges | Evaluate exp, log, pow at values near 1e-300 and 1e300, verify no NaN/Inf |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
