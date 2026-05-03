---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: N1
type: execute
wave: 9
depends_on:
  - 06-00
  - 06-01
files_modified:
  - crates/xcfun-kernels/src/functionals/gga/pbe/pbeintc.rs
  - crates/xcfun-kernels/src/functionals/gga/becke/beckesrx.rs
  - crates/xcfun-kernels/src/functionals/gga/p86/p86c.rs
  - crates/xcfun-kernels/src/functionals/gga/p86/p86corrc.rs
  - crates/xcfun-kernels/src/functionals/gga/pw91/pw91c.rs
  - crates/xcfun-kernels/src/functionals/gga/pbe/spbec.rs
  - crates/xcfun-kernels/src/functionals/gga/apbe/apbec.rs
  - crates/xcfun-kernels/src/functionals/gga/b97/b97c.rs
  - crates/xcfun-kernels/src/functionals/gga/b97/b97_1c.rs
  - crates/xcfun-kernels/src/functionals/gga/b97/b97_2c.rs
  - crates/xcfun-kernels/src/functionals/gga/pw91/pw91k.rs
  # (I-4 revision-2) Specific shared-helper files this plan may edit during cluster
  # fixes. NOT a directory glob (which was ambiguous and could collide with other
  # Wave-9 plans). Subset selected from the post-Plan-04 tree under
  # crates/xcfun-{eval|kernels}/src/functionals/gga/shared/ — only helpers actually
  # imported by the 11 inherited Phase-3 forwards in this plan's scope:
  #   - pbec_eps.rs   — used by PBEINTC, SPBEC, P86C, P86CORRC (PBE-correlation chain)
  #   - b97_poly.rs   — used by B97C, B97_1C, B97_2C (B97-correlation polynomial)
  #   - pw91_like.rs  — used by PW91C, PW91K, SPBEC (PW91-style helpers)
  #   - pbex.rs       — used by BECKESRX, APBEC (PBE-exchange substrate, ε source)
  #   - constants.rs  — shared LSDA / Wigner-Seitz constants
  #   - mod.rs        — re-exports if a new shared helper is added
  # optx.rs is OUT OF SCOPE here (OPTX belongs to 06-N3 if any). Wave-9 disjointness
  # with 06-N3 is automatic since I-3 Option B makes 06-N3 pure-verification (zero
  # kernel-source edits).
  - crates/xcfun-kernels/src/functionals/gga/shared/pbec_eps.rs
  - crates/xcfun-kernels/src/functionals/gga/shared/b97_poly.rs
  - crates/xcfun-kernels/src/functionals/gga/shared/pw91_like.rs
  - crates/xcfun-kernels/src/functionals/gga/shared/pbex.rs
  - crates/xcfun-kernels/src/functionals/gga/shared/constants.rs
  - crates/xcfun-kernels/src/functionals/gga/shared/mod.rs
  - validation/fixtures/d19_n1/pbeintc_baseline.jsonl
  - validation/fixtures/d19_n1/beckesrx_baseline.jsonl
  - validation/fixtures/d19_n1/p86c_baseline.jsonl
  - validation/fixtures/d19_n1/p86corrc_baseline.jsonl
  - validation/fixtures/d19_n1/pw91c_baseline.jsonl
  - validation/fixtures/d19_n1/spbec_baseline.jsonl
  - validation/fixtures/d19_n1/apbec_baseline.jsonl
  - validation/fixtures/d19_n1/b97c_baseline.jsonl
  - validation/fixtures/d19_n1/b97_1c_baseline.jsonl
  - validation/fixtures/d19_n1/b97_2c_baseline.jsonl
  - validation/fixtures/d19_n1/pw91k_baseline.jsonl
  - crates/xcfun-kernels/tests/d19_pbeintc.rs
  - crates/xcfun-kernels/tests/d19_beckesrx.rs
  - crates/xcfun-kernels/tests/d19_p86c.rs
  - crates/xcfun-kernels/tests/d19_p86corrc.rs
  - crates/xcfun-kernels/tests/d19_pw91c.rs
  - crates/xcfun-kernels/tests/d19_spbec.rs
  - crates/xcfun-kernels/tests/d19_apbec.rs
  - crates/xcfun-kernels/tests/d19_b97c.rs
  - crates/xcfun-kernels/tests/d19_b97_1c.rs
  - crates/xcfun-kernels/tests/d19_b97_2c.rs
  - crates/xcfun-kernels/tests/d19_pw91k.rs
autonomous: true
requirements:
  - ACC-01
  - ACC-02
  - ACC-03
  - ACC-04
acc04_eligible:
  # (W-9 revision-1) Pre-enumerated list of which inherited Phase-3 forwards
  # are eligible for ACC-04 mpmath substitution. The decision is made HERE at
  # plan-time, NOT by the executor at run-time. If a forward is NOT in this
  # list, the only legitimate paths are (a) Path-B fix or (b) escalation via
  # PLANNING INCONCLUSIVE for user-approved tolerance override.
  #
  # Per CONTEXT.md D-03: ACC-04 mpmath substitution is reserved for points
  # where the C++ source EXPLICITLY documents bracket cancellation. Of the
  # 11 inherited Phase-3/4 forwards, NONE has an explicit `test_threshold`-
  # style cancellation comment in upstream C++ source comparable to
  # `xcfun-master/src/functionals/ldaerfx.cpp:66`. Therefore:
  - "0 — bisect or escalate"
  # If Path-B reveals a cancellation in any of these 11 functionals during
  # execution, the executor proposes adding to this list via PLANNING
  # INCONCLUSIVE — NOT silent ACC-04 amendment.
must_haves:
  truths:
    - "Substrate-first hypothesis (per RESEARCH §"D-19 Bisection Methodology" Plan 06-N1 Step 1): re-run tier-2 at order 3 AFTER Plan 06-00 substrate work + Plan 06-01 reorg + Plan 06-06 dispatch widening; document which inherited Phase-3/4 D-19 forwards self-resolved."
    - "For each persistent forward (PBEINTC 6.2e+1, BECKESRX 2.3e+2, P86C 9.2e-2, P86CORRC, PW91C 1.7e-3, SPBEC 5.3e-4, APBEC 5.7e-9, B97C/B97_1C/B97_2C 7.8e-11, PW91K 1.4e-11): Path-B side-by-side reads of `xcfun-master/src/functionals/<name>.cpp` vs `crates/xcfun-kernels/src/functionals/<tier>/<name>.rs`."
    - "Per-functional fix per identified root cause (port-order, missing mul-vs-multo distinction, fp-contract hidden risk, summax decomposition issue)."
    - "Order-3 tier-2 sweep at strict 1e-12 GREEN for all 11 inherited Phase-3 forwards (or ACC-04 amendment with mpmath truth + per-functional override authorised by user — D-19 INCONCLUSIVE → Closed)."
    - "(I-4 revision-2) `files_modified` enumerates the SPECIFIC `gga/shared/*.rs` helper files this plan may edit (pbec_eps.rs, b97_poly.rs, pw91_like.rs, pbex.rs, constants.rs, mod.rs) — NOT the `gga/shared/` directory glob. The subset is the closure of helpers transitively imported by the 11 inherited forwards. optx.rs is excluded (OPTX-specific; out of scope). Wave-9 disjointness with 06-N3 is automatic given I-3 Option B (06-N3 is pure-verification — zero kernel-source edits)."
  artifacts:
    - path: "crates/xcfun-kernels/src/functionals/gga/pbe/pbeintc.rs"
      provides: "Order-3 tier-2 strict 1e-12 GREEN (was: 6.2e+1)"
      contains: "pbeintc_kernel"
    - path: "crates/xcfun-kernels/src/functionals/gga/becke/beckesrx.rs"
      provides: "Order-3 tier-2 strict 1e-12 GREEN (was: 2.3e+2)"
      contains: "beckesrx_kernel"
    - path: "crates/xcfun-kernels/src/functionals/gga/p86/p86c.rs"
      provides: "Order-3 tier-2 strict 1e-12 GREEN (was: 9.2e-2)"
      contains: "p86c_kernel"
  key_links:
    - from: "crates/xcfun-kernels/src/functionals/gga/<each>/<name>.rs"
      to: "xcfun-master/src/functionals/<name>.cpp (line-for-line port reference)"
      via: "Path-B side-by-side bisection per Phase 4 Plan 04-10 methodology"
      pattern: "<name>_kernel"
---

<objective>
Close out the **11 inherited Phase-3 D-19 forwards still failing at order 3** per `.planning/STATE.md` Phase 4 sign-off summary [^d19p4]:
- PBEINTC (max_rel 6.2e+1)
- BECKESRX (2.3e+2)
- P86C / P86CORRC (9.2e-2)
- PW91C (1.7e-3)
- SPBEC (5.3e-4)
- APBEC (5.7e-9)
- B97C / B97_1C / B97_2C (7.8e-11)
- PW91K (1.4e-11)

Methodology per RESEARCH §"D-19 Bisection Methodology" Plan 06-N1:

**Step 1 — Substrate-first hypothesis.** Plan 06-00 substrate (libm-hybrid `erf_precise_taylor` + AD N≥4 specialisations + tau guard) may incidentally tighten unrelated functionals. Plan 06-N3 (parallel with this plan) verifies the small-magnitude forwards already auto-tightened. For Plan 06-N1, the same "re-run tier-2 first, then bisect what didn't tighten" pattern applies. Document which forwards self-resolved (Plan 04-10 already showed PW86X + APBEX self-resolved this way at order 3 — RESEARCH §"D-19 Bisection Methodology"; expect similar bonus tightenings here).

**Step 2 — Path-B side-by-side reads** (per Phase 4 Plan 04-10 finding). For each persistent forward:
1. Open `xcfun-master/src/functionals/<name>.cpp` AND `crates/xcfun-kernels/src/functionals/<tier>/<name>.rs` side by side.
2. Trace the first divergence, looking for:
   - **Accidental re-parenthesisation** (most common root cause per `docs/design/07-accuracy-strategy.md §7`).
   - **Missing `mul`-vs-`multo` distinction** (`mul` allocates new array; `multo` is in-place; algorithmic-identity requires the same form as C++).
   - **fp-contract / FMA risk** (xtask check-no-mul-add should already block, but verify; some functionals may have shared-helper FMA hiding).
   - **For correlation kernels**: check the eps_pbe vs eps_pbe_polarized chain — Phase 4 Plan 04-10 found summax decomposition is line-for-line correct but f64-rounding cancellation regions need stratum-specific handling.

**Step 3 — Root-cause pattern recognition.** Phase 3/4 history shows shared-helper port-order bugs propagate to multiple functionals (Plan 03-05's `build_xc_a_b_2nd_taylor` fix tightened LYPC + others). Expect that fixing one root cause tightens 3-5 functionals at once.

**Step 4 — Per-functional stratum exclusion as last resort.** Per D-02, the bar is strict 1e-13 across all 78 — but in extremis a CONTEXT.md amendment can authorise a per-functional D-24-style override (Phase 2 LDAERF 1e-7 precedent). User approval required.

**Note on MGGA-class TPSS forwards:** Phase 4 Plan 04-10 identified TPSSC / TPSSLOCC / REVTPSSC catastrophic divergence. Plan 06-00 D-10 `tau ≥ tau_w` guard already addresses this — those forwards are NOT in 06-N1's scope; they're closed by Plan 06-00.

Purpose: Close the inherited Phase-3 D-19 forwards using Path-B methodology established by Plan 04-10. Each fix is local; analog is the file being fixed. Per RESEARCH "Plan 06-N1 — research-driven workflows": this plan is research-heavy and not strictly TDD-amenable — each functional is bisected, the root cause identified, and a per-file fix applied + verified via tier-2 sweep at order 3.

Output: 11 functional bodies (or fewer if root-cause clustering kicks in) tightened to strict 1e-12 at order 3; tier-2 sweep at order 3 reports 0 failing functionals from the Phase-3 inherited set; ACC-04 amendments authorised only where C++ documents cancellation that mpmath confirms.

**This plan is parallel-safe with 06-N2 and 06-N3** (per CONTEXT.md D-01 discretion: "parallel is fine because they touch independent functional sets"). Wave-9 disjointness is mechanically enforced by the explicit `files_modified` enumeration above (I-4 revision-2) plus 06-N3's pure-verification status (I-3 Option B).
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
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-00-substrate-PLAN.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@validation/src/driver.rs
@xcfun-master/src/functionals/pbeintc.cpp
@xcfun-master/src/functionals/beckesrx.cpp
@xcfun-master/src/functionals/p86c.cpp
@xcfun-master/src/functionals/pw91c.cpp
@xcfun-master/src/functionals/spbec.cpp
@xcfun-master/src/functionals/apbec.cpp
@xcfun-master/src/functionals/b97c.cpp
@xcfun-master/src/functionals/b97_1c.cpp
@xcfun-master/src/functionals/b97_2c.cpp
@xcfun-master/src/functionals/pw91k.cpp
</context>

<tasks>

<task type="auto">
  <name>Task 1: Substrate self-resolution audit + Path-B bisection setup</name>
  <files>(none — research + audit; no code changes)</files>
  <read_first>
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-VERIFICATION.md (full file — D-19 forwards consolidation)
    - .planning/phases/03-gga-tier-mode-potential/03-VERIFICATION.md if present (Phase 3 D-19 originals)
    - validation/report.html (committed Phase 4 capstone — per-functional verdict matrix)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"D-19 Bisection Methodology" Plan 06-N1
  </read_first>
  <action>
**Step A — Run order-3 tier-2 sweep AFTER Plan 06-00 substrate (no fixes yet):**

```bash
# Re-run order-3 tier-2 sweep on the 11 inherited forwards.
cargo run -p validation --release -- --backend cpu --order 3 --jobs 18 \
    --filter '^(pbeintc|beckesrx|p86c|p86corrc|pw91c|spbec|apbec|b97c|b97_1c|b97_2c|pw91k)$' \
    > /tmp/06-N1-pre-fix-sweep.log 2>&1
```

**Step B — Read `validation/report.html`** (or parse `validation/report.jsonl`) to extract:
- Per-functional `max_rel_err` at order 3.
- Identify which functionals self-resolved (rel_err < 1e-12) due to Plan 06-00 substrate work (libm-hybrid erf wrapper potentially affects functionals with `Dependency::ERF` or summax-via-erf chains; AD N≥4 affects metaGGA but should not impact pure-GGA forwards).

**Step C — For each remaining forward** (those still > 1e-12 at order 3), classify by suspected root cause:
- **PBEINTC (6.2e+1):** PBE-int-correlation; `pbe_eps`/`pbe_eps_polarized` chain. Likely shares structure with PBEC (which also has order-3 D-19 forward at 1.8e-12 in Plan 06-N3 scope).
- **BECKESRX (2.3e+2):** Becke short-range exchange with erf bracket. Plan 06-00 D-11 libm-hybrid `erf_precise_taylor` SHOULD self-resolve this — verify.
- **P86C / P86CORRC (9.2e-2):** P86 correlation. Same `pbec_eps`/`pbec_eps_polarized` chain shape as PBEC; likely shared-helper port-order issue.
- **PW91C (1.7e-3):** PW91 correlation. Different ε function from P86 but same gradient-stress amplification structure.
- **SPBEC (5.3e-4):** Spin-PBE-correlation. Variant of PBEC.
- **APBEC (5.7e-9):** Asymptotic-corrected PBE correlation. Mid-magnitude residual; bisection harder.
- **B97{,_1,_2}C (7.8e-11):** Becke 1997 correlation family. Phase 3 finding: "near-zero polarised gradient_stress" — same shape as Phase 4 Plan 04-10 small-magnitude metaGGA forwards (handled by 06-N3).
- **PW91K (1.4e-11):** PW91 kinetic. Mid-magnitude.

**Step D — Document the audit in `06-N1-progress.md`** (a planning-time scratch file, not committed; Plan 06-N1-SUMMARY.md absorbs the findings):

```markdown
# 06-N1 Pre-fix audit (after Plan 06-00 substrate)

| Functional | Order-3 max_rel_err (pre-06-00) | Order-3 max_rel_err (post-06-00) | Self-resolved? | Suspected root cause |
|------------|--------------------------------|----------------------------------|----------------|----------------------|
| PBEINTC    | 6.17e+1                        | (run sweep)                      | likely no      | shared pbec_eps helper |
| BECKESRX   | 2.27e+2                        | (run sweep)                      | LIKELY YES     | erf_precise_taylor lift |
| P86C       | 9.16e-2                        | (run sweep)                      | partial        | pbec_eps shared port |
| ...        | ...                            | ...                              | ...            | ...                  |
```

**Step E — Identify shared-helper candidates:**

Per Phase 3 Plan 03-05 finding: `build_xc_a_b_2nd_taylor` was the shared-helper bug that tightened LYPC + others. Search for cross-functional shared helpers in `crates/xcfun-kernels/src/functionals/gga/shared/`:

```bash
ls crates/xcfun-kernels/src/functionals/gga/shared/
git grep -n "fn pbec_eps\|fn pbec_eps_polarized\|fn pw91c_helper" crates/xcfun-kernels/src/functionals/gga/
```

Functionals sharing `pbec_eps` / `pbec_eps_polarized`: PBEC (06-N3), PBEINTC (this plan), SPBEC (this plan), P86C (this plan)— a fix to the shared helper could tighten 4 functionals at once.

**(I-4 revision-2 reminder)** If the audit identifies shared-helper candidates OUTSIDE the pre-enumerated `files_modified` list (`pbec_eps.rs`, `b97_poly.rs`, `pw91_like.rs`, `pbex.rs`, `constants.rs`, `mod.rs`), STOP and escalate via PLANNING INCONCLUSIVE — adding new shared-helper paths breaks the Wave-9 disjointness audit-trail and requires planner re-validation.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-kernels --test d19_pbeintc --test d19_beckesrx --test d19_p86c --test d19_p86corrc --test d19_pw91c --test d19_spbec --test d19_apbec --test d19_b97c --test d19_b97_1c --test d19_b97_2c --test d19_pw91k</automated>
  </verify>
  <acceptance_criteria>
    - Sweep completes without crash; logs show per-functional max_rel_err for the 11 inherited forwards.
    - Audit table populated in 06-N1-SUMMARY.md (or scratch file) classifying which forwards self-resolved vs need Path-B bisection.
    - At least one shared-helper candidate identified (e.g., pbec_eps in `gga/shared/`) for cluster-fix consideration.
  </acceptance_criteria>
  <done>Substrate self-resolution audit complete; per-functional triage list documented; Task 2 work plan derived (which functionals need individual Path-B reads vs which can be fixed via shared-helper edit).</done>
</task>

<task type="auto">
  <name>Task 2: Path-B fix campaign — read C++ ↔ Rust side-by-side; per-functional or cluster fixes</name>
  <files>crates/xcfun-kernels/src/functionals/gga/pbe/pbeintc.rs, crates/xcfun-kernels/src/functionals/gga/becke/beckesrx.rs, crates/xcfun-kernels/src/functionals/gga/p86/p86c.rs, crates/xcfun-kernels/src/functionals/gga/p86/p86corrc.rs, crates/xcfun-kernels/src/functionals/gga/pw91/pw91c.rs, crates/xcfun-kernels/src/functionals/gga/pbe/spbec.rs, crates/xcfun-kernels/src/functionals/gga/apbe/apbec.rs, crates/xcfun-kernels/src/functionals/gga/b97/b97c.rs, crates/xcfun-kernels/src/functionals/gga/b97/b97_1c.rs, crates/xcfun-kernels/src/functionals/gga/b97/b97_2c.rs, crates/xcfun-kernels/src/functionals/gga/pw91/pw91k.rs, crates/xcfun-kernels/src/functionals/gga/shared/pbec_eps.rs, crates/xcfun-kernels/src/functionals/gga/shared/b97_poly.rs, crates/xcfun-kernels/src/functionals/gga/shared/pw91_like.rs, crates/xcfun-kernels/src/functionals/gga/shared/pbex.rs, crates/xcfun-kernels/src/functionals/gga/shared/constants.rs, crates/xcfun-kernels/src/functionals/gga/shared/mod.rs</files>
  <read_first>
    - For each persistent forward, BOTH:
      - `xcfun-master/src/functionals/<name>.cpp` (full file)
      - `crates/xcfun-kernels/src/functionals/gga/<tier>/<name>.rs` (full file)
    - The shared helper headers in `xcfun-master/src/functionals/{pbec_eps.hpp, p86c_eps.hpp, b97_C_eps.hpp, pw91c_eps.hpp}` and the Rust analogs in `crates/xcfun-kernels/src/functionals/gga/shared/{pbec_eps.rs, b97_poly.rs, pw91_like.rs, pbex.rs}`.
    - Phase 4 Plan 04-10 SUMMARY (Path-B methodology reference).
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pitfall 1" (port-order divergence) + Pitfall §6 (tau_w underflow — already handled by Plan 06-00 D-10 for TPSS).
  </read_first>
  <action>
**Step A — For each persistent forward, perform Path-B bisection:**

For each functional in the 11-forward set that did NOT self-resolve (per Task 1 audit):

1. **Open C++ source** at `xcfun-master/src/functionals/<name>.cpp` and identify the body (typically a `static double <name>_polarized<F: float-or-Taylor>(...)` or `<name>_unpolarized` template at the bottom + a `setup_<name>(functional &fun)` factory).

2. **Open Rust port** at `crates/xcfun-kernels/src/functionals/gga/<tier>/<name>.rs` and read the `<name>_kernel<F: Float>(d, out, n)` body.

3. **Trace divergence** by comparing line-by-line:
   - **Accidental re-parenthesisation:** C++ `a*b + c*d` may have implicit left-to-right associativity that Rust port reordered. Check the AST shape.
   - **`mul` vs `multo` mismatch:** C++ uses `auto x = a * b;` (allocates) vs `a *= b;` (in-place); Rust port may have flipped these.
   - **Shared-helper port-order:** if functional uses `pbec_eps` or similar, read the shared-helper Rust port AND C++ analog. Phase 3 Plan 03-05 fix to `build_xc_a_b_2nd_taylor` is the precedent.
   - **fp-contract / FMA hiding:** `xtask check-no-mul-add` should block, but verify by inspecting the binary — `cargo objdump -p validation --release | grep -i fmadd | head` should be empty for the function.

4. **Apply the minimal fix** that brings rel-err to strict 1e-12. Document the fix in a per-functional fix-note: `06-N1-fixes/<name>.md`.

5. **Re-run validation** for the just-fixed functional:
   ```bash
   cargo run -p validation --release -- --backend cpu --order 3 --jobs 4 --filter '^<name>$'
   ```
   Must report 0 failures at strict 1e-12.

**Step B — Cluster fixes via shared-helper edits:**

If Task 1 audit identified `pbec_eps` (or analogous) as a shared helper across PBEINTC + SPBEC + P86C, fix the helper ONCE and re-run all three. Per RESEARCH §"D-19 Bisection Methodology" Step 3: expect 3-5 functionals tightened per shared-helper fix. The shared helpers this task may edit are pre-enumerated in `files_modified` (I-4 revision-2): `pbec_eps.rs`, `b97_poly.rs`, `pw91_like.rs`, `pbex.rs`, `constants.rs`, `mod.rs`. If a different shared helper proves necessary, halt and escalate via PLANNING INCONCLUSIVE.

**Step C — ACC-04 amendment for residuals where C++ documents cancellation:**

For any forward where Path-B reveals **the C++ source explicitly notes bracket cancellation** (e.g. `xcfun-master/src/functionals/<name>.cpp` has a `test_threshold` like `ldaerfx.cpp:66` or a comment about precision loss):
- Substitute mpmath truth at 200-digit precision per D-03.
- Generate fixture records via `cargo xtask regen-mpmath-fixtures` (Plan 06-00 sidecar; populate the `functionals.py` entry for this functional in this task).
- Add the functional to `validation/src/driver.rs`'s `--reference mpmath` list.
- Re-run with `--reference mpmath` and assert 1e-13 vs mpmath truth.

**Step D — Per-functional override (last resort):**

Per CONTEXT.md D-02 + RESEARCH §"D-19 Bisection Methodology" Step 4: If neither (a) Path-B fix nor (b) ACC-04 mpmath amendment closes the gap, escalate via PLANNING INCONCLUSIVE for user-approved override (Phase 2 D-24 LDAERF 1e-7 precedent). DO NOT silently widen tolerance.

**Step D-bis — (B-6 revision-1) Per-functional fixture + unit test pattern:**

For EACH functional bisected by this plan, BEFORE applying the Path-B fix:
1. Create a per-functional fixture file `validation/fixtures/d19_n1/<name>_baseline.jsonl` with 5–10 records at the failing density strata. Expected values come from C++ baseline (rel-err > 1e-12 at order 3 — these are the records that motivate the fix) OR mpmath truth at prec=200 (when ACC-04 substitution applies, only for functionals listed in `acc04_eligible:` frontmatter — which is EMPTY per W-9 unless escalated).
2. Create a per-functional unit test `crates/xcfun-kernels/tests/d19_<name>.rs` matching the pattern Plan 06-00 already established for golden_multo_n4..6 / golden_compose_n4..6:
   ```rust
   #![cfg(feature = "testing")]
   use cubecl::prelude::*;
   use cubecl_cpu::CpuRuntime;
   use xcfun_eval::for_tests::cpu_client;
   use approx::assert_relative_eq;
   // ... load fixture jsonl; for each record:
   //     run xcfun_kernels::functionals::gga::<tier>::<name>::<name>_kernel via cubecl-cpu launcher;
   //     assert_relative_eq!(rust_out[i], rec.expected[i], max_relative = 1e-12) for each i.
   ```
3. RED-then-GREEN: the test MUST FAIL before the Path-B fix lands (RED — the fixture's expected values come from the failing comparison) and PASS after (GREEN — strict 1e-12).

This per-functional unit test approach (B-6 revision-1) mirrors Plan 06-00's golden_multo / golden_compose pattern and gives each Path-B fix its OWN fast feedback loop (single-digit-second test) instead of relying on the slow 11-functional `--filter` validation sweep.

The per-functional `<automated>` test for THIS task runs `cargo nextest run -p xcfun-kernels --test d19_<name>` — NOT the full validation sweep. The full sweep stays as a manual sign-off command in Task 1's audit.

**Step E — Verify full 11-functional set GREEN at order 3:**

```bash
cargo run -p validation --release -- --backend cpu --order 3 --jobs 4 \
    --filter '^(pbeintc|beckesrx|p86c|p86corrc|pw91c|spbec|apbec|b97c|b97_1c|b97_2c|pw91k)$'
```

Must report 0 failures at strict 1e-12 (or 0 + amendments documented per Step C/D).

**Forbidden:**
- Do NOT introduce `mul_add(...)` (xtask check-no-mul-add blocks).
- Do NOT add `regularize` calls outside `c[CNST]` (Phase 2 D-22 invariant).
- Do NOT silently widen tolerance — always escalate via PLANNING INCONCLUSIVE.
- Do NOT alter the algorithmic-identity contract for any functional except where ACC-04 mpmath amendment is authorised by D-03 + user approval.
- **(I-4 revision-2)** Do NOT edit `gga/shared/optx.rs` (out of scope; OPTX belongs to 06-N3 only — and 06-N3 is pure-verification per I-3 Option B). Editing optx.rs requires a planner-approved revision.
  </action>
  <verify>
    <automated>cargo run -p validation --release -- --backend cpu --order 3 --jobs 4 --filter '^(pbeintc|beckesrx|p86c|p86corrc|pw91c|spbec|apbec|b97c|b97_1c|b97_2c|pw91k)$'</automated>
  </verify>
  <acceptance_criteria>
    - **(B-6 revision-1)** Per-functional unit tests GREEN: `cargo nextest run -p xcfun-kernels --test d19_pbeintc --test d19_beckesrx --test d19_p86c --test d19_p86corrc --test d19_pw91c --test d19_spbec --test d19_apbec --test d19_b97c --test d19_b97_1c --test d19_b97_2c --test d19_pw91k` exits 0 in single-digit seconds.
    - Each `validation/fixtures/d19_n1/<name>_baseline.jsonl` exists with 5-10 records: `find validation/fixtures/d19_n1 -name '*_baseline.jsonl' -size +0c | wc -l` >= 11.
    - Order-3 tier-2 full sweep on 11 inherited forwards (sign-off command, run separately) reports 0 failures at strict 1e-12 (or documented ACC-04 amendments + per-functional overrides per Step C/D).
    - Each fix has a per-functional note in `06-N1-SUMMARY.md` listing root cause + diff summary + post-fix rel_err.
    - No new `mul_add` introduced: `cargo run -p xtask --bin check-no-mul-add` exits 0.
    - No new `Box::leak` or `format!` introduced in xcfun-* lib graph: `git grep -E 'Box::leak|format!\\(' crates/xcfun-kernels/src/functionals/gga | wc -l` <= existing baseline.
    - tier-2 LDA + GGA quick sweep at order 2 still GREEN (no regression).
    - Existing tier-1 self-tests for the affected functionals still GREEN: `cargo nextest run -p xcfun-kernels --features testing --test self_tests` exits 0.
    - Path-B fix-notes exist in 06-N1-SUMMARY.md for each functional that needed bisection (vs. functionals that auto-tightened from substrate work).
    - **(I-4 revision-2)** No edits to `gga/shared/optx.rs` from this plan: `git diff --stat HEAD~1 -- crates/xcfun-kernels/src/functionals/gga/shared/optx.rs` reports no changes.
    - **(I-4 revision-2)** Any `gga/shared/*.rs` edits are confined to the pre-enumerated subset (pbec_eps / b97_poly / pw91_like / pbex / constants / mod): `git diff --name-only HEAD~1 -- crates/xcfun-kernels/src/functionals/gga/shared/ | grep -vE '(pbec_eps|b97_poly|pw91_like|pbex|constants|mod)\\.rs$' | wc -l` == 0.
  </acceptance_criteria>
  <done>11 inherited Phase-3 D-19 forwards closed (Path-B fixes OR ACC-04 amendments OR escalations); order-3 tier-2 sweep on this set GREEN at strict 1e-12; per-functional root-cause notes documented; shared-helper edits confined to the I-4 revision-2 enumerated subset.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust port ↔ C++ source | Algorithmic-identity contract per CLAUDE.md Core Value; deviations only via D-03 ACC-04 amendment (mpmath truth) |
| Shared helper edits ↔ multi-functional impact | One helper change can cascade across PBEINTC + SPBEC + P86C; full sweep verifies no regression |
| Wave-9 plan boundaries (06-N1 ↔ 06-N2 ↔ 06-N3) | (I-4 revision-2) Mechanical disjointness: 06-N1 enumerates specific `gga/shared/*.rs` files; 06-N2 touches only `xtask/mpmath_eval/**` and `validation/**`; 06-N3 is pure-verification (I-3 Option B). Any drift triggers PLANNING INCONCLUSIVE. |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-FAST-MATH | high | Path-B fixes could accidentally introduce `mul_add` or reassociate ops | xtask check-no-mul-add gate runs in Step E; each fix verified at strict 1e-12 (FMA would manifest as ULP-level deltas) |
| T-06-CUBECL-DRIFT | high | (cross-cutting) | Plan 06-04 xtask gate covers; this plan inherits |
| T-06-WIDEN-TOLERANCE | medium | Implementer tempted to silently widen tolerance to GREEN-pass a stubborn forward | Forbidden per CONTEXT.md D-02; Step D requires PLANNING INCONCLUSIVE escalation for user approval |
| T-06-SHARED-HELPER-REGRESSION | medium | Cluster fix to pbec_eps could break PBEC (in Plan 06-N3 scope; but 06-N3 is pure-verification) | Step E full sweep verifies; if regression detected, revert and refine shared helper approach |
| T-06-MPMATH | low | (only if Step C ACC-04 amendments invoked) — mpmath fixture reproducibility | Plan 06-00 sidecar drift gate (`--check`) covers |
| T-06-WAVE9-DRIFT | medium | A shared-helper edit lands outside the I-4 revision-2 enumerated subset, breaking Wave-9 disjointness | Acceptance criterion `git diff --name-only` audit + Forbidden: editing optx.rs; trigger PLANNING INCONCLUSIVE if scope expansion is genuinely required |
</threat_model>

<verification>
- Tier-2 order-3 sweep on the 11 inherited forwards: 0 failures at strict 1e-12 (or documented amendments).
- xtask check-no-mul-add GREEN.
- xtask check-no-anyhow GREEN.
- Tier-1 self-tests for affected functionals GREEN (no algorithmic-identity regression in physical regime).
- Phase 4 inherited D-19 ledger (`04-VERIFICATION.md`) updated to reflect Plan 06-N1 closures.
- **(I-4 revision-2)** All `gga/shared/*.rs` edits confined to the pre-enumerated subset.
</verification>

<success_criteria>
- 11 inherited Phase-3 D-19 forwards closed (per ROADMAP Phase 6 success criterion 2 indirectly, and ACC-04 amendment per D-03).
- Order-3 tier-2 sweep at strict 1e-12 GREEN across all 78 functionals (combined with Plan 06-00 substrate + Plan 06-N3 small-magnitude sweep + Plan 06-N2 mpmath-only fixtures for the 20 excluded set).
- Path-B methodology applied + documented (Phase 4 Plan 04-10 precedent extended).
- I-4 revision-2 closed: `files_modified` enumerates specific `gga/shared/*.rs` paths (no directory glob); Wave-9 disjointness with 06-N3 trivially provable since 06-N3 is pure-verification per I-3 Option B.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-N1-SUMMARY.md` documenting:
- Pre-fix audit table (which forwards self-resolved from Plan 06-00 substrate work; which needed Path-B)
- Per-functional root-cause notes (e.g., "PBEINTC: shared `pbec_eps` helper had a re-association in line N → fix tightened PBEINTC + SPBEC + P86C simultaneously")
- ACC-04 amendments invoked (functional × stratum × mpmath truth substitution rationale)
- Per-functional overrides escalated (if any)
- Order-3 tier-2 sweep verdict for the 11 inherited forwards (post-fix)
- REQUIREMENTS.md updates: GGA-* / MGGA-* entries with closure status
- I-4 revision-2 audit: list of `gga/shared/*.rs` files actually edited (must be subset of pbec_eps / b97_poly / pw91_like / pbex / constants / mod)
</output>
</content>
</invoke>