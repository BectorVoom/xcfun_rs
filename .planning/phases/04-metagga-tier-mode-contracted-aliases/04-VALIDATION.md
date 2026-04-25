---
phase: 4
slug: metagga-tier-mode-contracted-aliases
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-25
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Detailed per-requirement test map lives in **04-RESEARCH.md §Validation Architecture**; this file is the operational sampling contract.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + `cargo nextest` (Phase 2 ACC-01..03 tier-2 harness) |
| **Config file** | `Cargo.toml` workspace + `validation/build.rs` (cc-compiled C++ ref) |
| **Quick run command** | `cargo test -p xcfun-eval --test self_tests --features testing` |
| **Mid-tier command** | `cargo xtask validate --backend cpu --order 2 --filter '{family}'` |
| **Full suite command** | `cargo xtask validate --backend cpu --order 3 --filter '.*'` |
| **xcfun-ad fixture-gate** | `cargo test -p xcfun-ad --features cpu` (golden_mul/expand/composed + br_inverse fixtures) |
| **Estimated runtime** | quick ~5s · mid ~60s/family · full ~10-20min depending on order cap |

---

## Sampling Rate

- **After every task commit:** `cargo build -p xcfun-eval --release` + `cargo test -p xcfun-eval --test self_tests --features testing` (5s feedback)
- **After every functional port:** Tier-1 self-test for the specific FunctionalId (sub-second per id)
- **After every wave:** `cargo xtask validate --backend cpu --order 2 --filter '<family>'` (mid-tier parity gate, 60s)
- **After Wave 6 capstone:** Full-matrix `cargo xtask validate --backend cpu --order 3 --filter '.*'` ; commit `validation/report.html` + `validation/report.jsonl`
- **Before `/gsd-verify-work`:** Full suite must be GREEN (subject to inherited Phase-3 D-19 forwards = 13 entries, plus any new D-19 forwards from Phase 4 with explicit sign-off)
- **Max feedback latency:** 5s for the quick-test gate after each task commit

---

## Per-Task Verification Map

> Filled by the planner per task. Each task's `<acceptance_criteria>` MUST include at least one grep-checkable or test-command verification per Phase 2 D-09 + Phase 3 inheritance. The 04-RESEARCH.md §Validation Architecture block enumerates concrete acceptance criteria per requirement ID (MGGA-01..05, MODE-03, ALIAS-01..06, GGA-03 carryover, GGA-10 CSC carryover); the planner copies them into per-task acceptance_criteria fields.

**Tier-1 self-tests** (per-functional, sub-second):
- 32 metaGGA + carryover IDs each get a tier-1 entry via `xtask regen-registry` extraction of FUNCTIONAL macro `test_in`/`test_out`/`test_threshold` from C++ source.
- Strict 1e-12 by default; per-id override allowed only with upstream-documented `test_threshold` (D-11 + Phase 3 D-18 inheritance).

**Tier-2 cross-mode parity** (10k-point grid + metaGGA stratum):
- Existing 10k-point grid (Phase 2 Plan 02-06 + Phase 3 D-23) reused unchanged at canonical seed `0x1234abcd`.
- New metaGGA stratum at sibling seed `0xc0ffee01` (D-09-A): 1000 records with random tau ∈ [0, kF² × n^(2/3)], lap near zero, JP_aa/JP_bb in physically reasonable range.
- Per-record gate: `|rust − cpp| / max(|cpp|, 1.0) < 1e-12` (subject to per-functional override).

**Tier-2 alias canary** (Wave 4 unit tests):
- Negative-weight propagation: `set("camcompx", 0.37)` → settings[beckecamx] == −0.37 × 0.37 (verify exact weight in `aliases.cpp`).
- Additive accumulation: `set("b3lyp", 1.0); set("slaterx", 0.5); get("slaterx") == 0.80 + 0.5 == 1.30`.
- Parameter overwrite (NOT additive): `set("b3lyp", 0.5); get("exx") == 0.10` (NOT 0.30).
- Case-insensitive lookup: `set("B3LYP", 1.0)` and `set("b3lyp", 1.0)` produce identical settings[].
- All 46 aliases × value=1.0 round-trip verified against manual composition.

**Tier-2 Mode::Contracted cross-mode** (Wave 5):
- Orders 0..=4 cross-checked against PartialDerivatives Taylor coefficients (algorithmic-identity → trivial pass if PartialDerivatives is GREEN at the same order).
- Orders 5..=6 require a new C++ harness path in `validation/src/c_driver.rs` calling `xcfun_eval` with `XC_CONTRACTED` mode at order 5/6 on a 100-point subset × 4 representative functionals (e.g., SLATERX, PBEX, TPSSX, M06X — one per tier).
- Per-record gate: `1 << order` doubles match the C++ reference at strict 1e-12.

**Tier-3 (GPU)** — OUT OF SCOPE for Phase 4 (Phase 6 owns CUDA + Wgpu tier-3).

---

## Wave 0 Requirements

- [ ] `crates/xcfun-ad/src/expand/br_inverse.rs` — `ctaylor_br_inverse` primitive (D-02)
- [ ] `xtask/src/regen_ad_fixtures.rs` extension — golden BR-inverse fixtures (30 z-points × orders {2,3,4})
- [ ] `crates/xcfun-ad/tests/golden_br_inverse.rs` — fixture-gate at strict 1e-12
- [ ] `crates/xcfun-ad/tests/test_ctaylor_n6.rs` — smoke test for CTaylor<F, 6> at orders 5/6 (D-07 capacity verification)
- [ ] `crates/xcfun-eval/src/density_vars/build.rs` — 2 new mandatory metaGGA arms (id=13 TAUA_TAUB, id=17 full JP); 6 optional arms per planner discretion (research finding §DensVarsDev Audit)
- [ ] `crates/xcfun-eval/src/functionals/mgga/mod.rs` + `mgga/shared/{tpss_like,scan_like,m0x_like,br_like,blocx,cs}.rs` — 6 helper modules (D-01-A Wave 0)
- [ ] `crates/xcfun-eval/tests/regularize_mgga_invariant.rs` — regularize-only-CNST invariant for new metaGGA Vars arms (Phase 3 D-11 inheritance)
- [ ] `validation/src/fixtures.rs` — extend grid generator with metaGGA stratum at seed `0xc0ffee01` (D-09-A)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Mode::Contracted parity at orders 5/6 vs C++ | MODE-03 | C++ reference tests don't exist for orders 5/6 in the vendored xcfun-master copy; new harness path required (research §Mode::Contracted) | After Wave 5 lands, run `cargo xtask validate --backend cpu --mode contracted --order 6 --filter 'slaterx,pbex,tpssx,m06x'`; commit `validation/report-contracted-orders-5-6.html` snapshot |
| Alias FIXME parity at L390 (EXX through alias) | ALIAS-06 | The C++ behaviour is "weird-but-deterministic" per the upstream FIXME comment; algorithmic-identity rule requires bit-for-bit preservation, not "fixing" | Inspect `Functional::set("b3lyp", 0.5); get("exx")` returns 0.10; document in test comments that this matches C++ FIXME-flagged behaviour |
| BR Newton-inverse libm-drift across runners | GGA-03 carryover (BR family) | `f64::exp` / `f64::log` from glibc may differ ≤ 1 ULP across CI runners; the BR Newton iteration trajectory amplifies this | Run BR fixture-gate on Linux x86_64 and macOS x86_64 (CI matrix); if drift > 1e-12, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03 |
| BLOCX kernel non-dependency on BR primitive | MGGA-05 | Research §Source Tree Triage confirmed BLOCX has zero `BR(...)` calls (TPSS-shaped enhancement) — supersedes CONTEXT D-01-A's claimed dependency | Confirm during Wave 3 that `crates/xcfun-eval/src/functionals/mgga/blocx.rs` compiles + passes self-test independently of Wave 1 BR ship |

---

## Validation Sign-Off

- [ ] All tasks have `<acceptance_criteria>` with grep-checkable or test-command verifications (Phase 2 D-09 + Phase 3 inheritance)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (BR primitive + metaGGA shared helpers + Vars arms + fixture gates)
- [ ] No `cargo watch` or watch-mode flags (CI-deterministic only)
- [ ] Feedback latency < 5s for quick-test gate
- [ ] Tier-2 metaGGA stratum reproducible at seed `0xc0ffee01`
- [ ] Mode::Contracted orders 5/6 cross-mode harness extension committed in Wave 5
- [ ] All 13 Phase-3 D-19 forwards remain forwarded to Phase 6 (D-10 inheritance — no Phase-4 attempt at resolution)
- [ ] `nyquist_compliant: true` set in frontmatter at Wave 6 sign-off

**Approval:** pending — flips to approved YYYY-MM-DD at Wave 6 sign-off via `/gsd:execute-phase 4`.
