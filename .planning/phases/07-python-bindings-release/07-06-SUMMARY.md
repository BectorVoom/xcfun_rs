---
phase: 07-python-bindings-release
plan: 06
subsystem: python-bindings
tags: [docs, pypi, readme, release-prep]
dependency_graph:
  requires:
    - "Plan 07-05 (pyproject.toml [project] readme = \"README.md\" already wired)"
    - "Locked decisions D-03 (CPU-only PyPI default), D-04 (Backend hidden + XCFUN_FORCE_BACKEND env-var), D-13 (v0.1.0 framing — cubecl pre-release dep + deferred UAT items)"
  provides:
    - "User-facing PyPI README rendered on the project page after publish"
    - "Reference document the GitHub Release notes will surface in Plan 07-09"
    - "Single source for the documented `pip install xcfun_rs` + `maturin build --features hip|cuda|wgpu` recipes"
  affects:
    - "crates/xcfun-py wheel content (README.md is bundled by maturin into source distribution + dist-info)"
tech_stack:
  added: []
  patterns:
    - "Documentation-only plan; no Cargo/Python dependency churn"
    - "Placeholder `<owner>` retained — operator fills pre-publish (Plan 07-09 release workflow's sed step)"
key_files:
  created:
    - "crates/xcfun-py/README.md (142 lines; ≥ 80-line acceptance threshold met)"
  modified: []
decisions:
  - "Honored D-03: documented PyPI wheel as CPU-only (broadest install compatibility), GPU acceleration only via source rebuild"
  - "Honored D-04: documented XCFUN_FORCE_BACKEND env-var as escape hatch; explicitly noted Python class does NOT expose backend= kwarg"
  - "Honored D-13: cited cubecl =0.10.0-pre.3 pre-release dependency as v0.1.0 framing rationale; listed 2 deferred HUMAN-UAT items (ROCm + Wgpu hardware sweeps)"
  - "Used the EXACT plan-supplied README content verbatim; no editorial divergence"
metrics:
  duration: "~5 minutes"
  completed_date: "2026-05-08"
  tasks_completed: 1
  tasks_total: 1
  files_created: 1
  files_modified: 0
---

# Phase 07 Plan 06: xcfun-py PyPI README Summary

User-facing `crates/xcfun-py/README.md` (142 lines) authored verbatim from the plan
spec; documents `pip install xcfun_rs` (CPU-only D-03 default), `maturin build
--features hip|cuda|wgpu` for GPU rebuilds, `XCFUN_FORCE_BACKEND` env-var escape
hatch (D-04), and the v0.1.0 caveat framing (D-13: cubecl pre-release + 2 deferred
HUMAN-UAT items). Linked from `pyproject.toml [project] readme = "README.md"`,
so it ships in the wheel and renders on PyPI post-publish.

## What Was Built

**`crates/xcfun-py/README.md`** — primary Python-user-facing doc, structured as:

1. **Header / value statement** — "Python bindings for xcfun_rs", 1e-12 parity
   claim, 78 functionals + 50+ aliases + 31 Vars × 3 modes feature surface,
   zero-copy NumPy `f64` interop callout.
2. **Install** — `pip install xcfun_rs` + CPU-only-by-default explanation +
   `abi3-py310` ABI note + v0.1.0 wheel matrix (`manylinux_2_28_x86_64`,
   `macosx_11_0_arm64`, `win_amd64`); aarch64 / macOS x86_64 / Windows aarch64
   deferred to v0.2.
3. **GPU rebuild (advanced)** — three `maturin build --release --features
   hip|cuda|wgpu` recipes verbatim from §6 of `07-RESEARCH.md`, with a note
   that CUDA/ROCm/Wgpu runtime libraries must be installed separately on the
   host.
4. **Backend selection (env-var escape hatch)** — D-04 escape hatch documented:
   `XCFUN_FORCE_BACKEND=cpu|hip|cuda|wgpu`; explicit note that the Python class
   does NOT expose a `backend=` kwarg.
5. **Quickstart** — 5-line minimal usage example: `import xcfun_rs as xc` →
   `xc.Functional("pbe", vars=..., mode=..., order=2)` → `f.eval_vec(densities)`
   → result; plus free-function utilities `xc.version()`, `xc.describe_short(...)`.
6. **Numerical contract** — restates the 1e-12 strict (CPU/CUDA/ROCm) /
   1e-13 mpmath-only-spec / 1e-9 Wgpu envelope.
7. **Performance** — GIL detached via `py.detach(...)`; `nr_points < 64` per-point
   fallback vs. `nr_points ≥ 64` batched dispatch (Pitfall 2 from §10 of RESEARCH).
8. **v0.1.0 caveats** — D-13 framing: cubecl pre-release dep + 2 deferred UAT
   items (ROCm tier-3 + Wgpu tier-3 hardware sweeps); CHANGELOG cross-link.
9. **License** — MPL-2.0 inheritance from C++ xcfun.
10. **Links** — repo / issue tracker / CHANGELOG / C++ xcfun upstream.

The placeholder `<owner>` (used in repo-URL anchors and Homepage links) is
intentionally left unfilled per plan instructions — Plan 07-09's release workflow
substitutes the actual GitHub owner via `sed` pre-publish.

## Tasks Completed

| Task | Name                                            | Status | Commit  | Files                          |
|------|-------------------------------------------------|--------|---------|--------------------------------|
| 6.1  | Author crates/xcfun-py/README.md                | Done   | 1a03f0a | crates/xcfun-py/README.md      |

## Verification Evidence

All automated and manual acceptance checks passed:

| Check                                                                       | Result            |
|-----------------------------------------------------------------------------|-------------------|
| `test -f crates/xcfun-py/README.md`                                         | PASS              |
| `grep -F 'pip install xcfun_rs' crates/xcfun-py/README.md`                  | 1 hit             |
| `grep -F 'CPU-only' crates/xcfun-py/README.md` (D-03 lock)                  | 2 hits (≥ 1)      |
| `grep -F 'maturin build --release --features hip' …`                        | 1 hit             |
| `grep -F 'maturin build --release --features cuda' …`                       | 1 hit             |
| `grep -F 'maturin build --release --features wgpu' …`                       | 1 hit             |
| `grep -F 'XCFUN_FORCE_BACKEND' …` (D-04 escape hatch)                       | 4 hits (≥ 1)      |
| `grep -F 'abi3-py310' …`                                                    | 1 hit             |
| `grep -F 'cubecl =0.10.0-pre.3' …` (D-13 framing)                           | 1 hit             |
| `grep -F 'manylinux_2_28' …` (wheel matrix lock)                            | 1 hit             |
| `wc -l crates/xcfun-py/README.md` (≥ 80 acceptance threshold)               | 142 lines (PASS)  |
| `grep -F 'readme          = "README.md"' crates/xcfun-py/pyproject.toml`    | 1 hit (link OK)   |

## Deviations from Plan

None — plan executed exactly as written. The README content was reproduced
verbatim from the plan's `<action>` block; placeholder `<owner>` retained per
explicit plan instruction ("Do NOT fill in a guess; placeholder is correct for
the v0.1.0 PR").

## Authentication Gates

None encountered (documentation-only plan; no network or API calls).

## Known Stubs

None. The placeholder `<owner>` is an intentional, plan-mandated value-substitution
point owned by Plan 07-09's release workflow — not a stub blocking this plan's
goal.

## Threat Flags

None — pure documentation change, no new trust boundary or attack surface.
The threat-register entry T-7-06-01 (README claims drift) is mitigated by
grep-verifiable acceptance criteria, all of which passed.

## Self-Check: PASSED

- File exists: `crates/xcfun-py/README.md` — FOUND
- Commit exists: `1a03f0a` — FOUND on branch `worktree-agent-a6b61d50902fa56fc`
- All 12 acceptance-criteria grep checks: PASS
- Line count 142 ≥ 80: PASS
- pyproject.toml `readme` link verified at line 9: PASS

## Next Steps (Wave 7+ context — not this plan's responsibility)

- Plan 07-07 will run `maturin build` and verify the README content is embedded
  in the resulting wheel's `*.dist-info/METADATA`.
- Plan 07-09 will surface this README into the GitHub Release notes for v0.1.0
  and run the `<owner>` → actual-owner sed substitution as part of the release
  workflow.
