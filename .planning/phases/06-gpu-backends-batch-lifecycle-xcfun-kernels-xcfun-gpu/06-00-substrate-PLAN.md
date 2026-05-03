---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 00
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/xcfun-ad/src/ctaylor_rec/multo.rs
  - crates/xcfun-ad/src/ctaylor_rec/compose.rs
  - crates/xcfun-ad/src/expand/erf.rs
  - crates/xcfun-ad/src/math.rs
  - crates/xcfun-ad/tests/golden_multo_n4.rs
  - crates/xcfun-ad/tests/golden_multo_n5.rs
  - crates/xcfun-ad/tests/golden_multo_n6.rs
  - crates/xcfun-ad/tests/golden_compose_n4.rs
  - crates/xcfun-ad/tests/golden_compose_n5.rs
  - crates/xcfun-ad/tests/golden_compose_n6.rs
  - crates/xcfun-ad/tests/erf_taylor_chain.rs
  - crates/xcfun-eval/src/functionals/mgga/tpssc.rs
  - crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs
  - crates/xcfun-eval/src/functionals/mgga/revtpssc.rs
  - crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs
  - crates/xcfun-eval/tests/tpss_tau_clamp.rs
  - xtask/src/bin/regen_mpmath_fixtures.rs
  - xtask/mpmath_eval/__init__.py
  - xtask/mpmath_eval/__main__.py
  - xtask/mpmath_eval/evaluator.py
  - xtask/mpmath_eval/functionals/__init__.py
  - xtask/mpmath_eval/functionals/ldaerfx.py
  - xtask/mpmath_eval/functionals/ldaerfc.py
  - xtask/mpmath_eval/functionals/ldaerfc_jt.py
  - xtask/mpmath_eval/functionals/tpssc.py
  - xtask/mpmath_eval/functionals/tpsslocc.py
  - xtask/mpmath_eval/functionals/revtpssc.py
  - xtask/mpmath_eval/ad_chain.py
  - xtask/mpmath_eval/densvars.py
  - xtask/mpmath_eval/README.md
  - validation/fixtures/mpmath/.gitkeep
autonomous: true
requirements:
  - KER-03
must_haves:
  truths:
    - "ctaylor_multo_n{4,5,6} and ctaylor_compose_n{4,5,6} match mpmath truth at 1e-13 (per D-02 sign-off bar) — unblocks Mode::Contracted orders 5..=6 metaGGA per Phase-4 Plan 04-05 D-19."
    - "erf_precise_taylor<F,N> seeds t[0] via erf_precise (FreeBSD msun port from Phase 2 Plan 02-06 dca382a) and uses derivative-Taylor for t[i≥1] — resolves order-3 LDAERF AD-chain amplification per D-11."
    - "TPSSC / TPSSLOCC / REVTPSSC kernel bodies call ctaylor_max(d.tau, tau_w) at top-of-body and pass tau_clamped to all downstream consumers — resolves the unphysical-regime f64 cancellation per D-10 / Phase-4 Plan 04-10 Path-B finding."
    - "xtask regen-mpmath-fixtures spawns python3 -m xtask.mpmath_eval, captures JSONL on stdout, writes validation/fixtures/mpmath/<f>.jsonl + .sha256 stamp; --check exits 2 on drift (per D-04 / D-21 pattern)."
    - "mpmath sidecar is xtask-only — `cargo build` of any xcfun-* library crate does NOT require Python3 (per D-04)."
  artifacts:
    - path: "crates/xcfun-ad/src/ctaylor_rec/multo.rs"
      provides: "ctaylor_multo_n4, ctaylor_multo_n5, ctaylor_multo_n6 #[cube] fn"
      contains: "ctaylor_multo_n4"
    - path: "crates/xcfun-ad/src/ctaylor_rec/compose.rs"
      provides: "ctaylor_compose_n4, ctaylor_compose_n5, ctaylor_compose_n6 #[cube] fn"
      contains: "ctaylor_compose_n4"
    - path: "crates/xcfun-ad/src/expand/erf.rs"
      provides: "erf_precise_taylor<F: Float, const N: u32> #[cube] fn"
      contains: "erf_precise_taylor"
    - path: "crates/xcfun-eval/src/functionals/mgga/tpssc.rs"
      provides: "tpssc_kernel with tau_clamped guard"
      contains: "ctaylor_max"
    - path: "xtask/src/bin/regen_mpmath_fixtures.rs"
      provides: "Python sidecar driver + --check drift gate"
      contains: "python3"
    - path: "xtask/mpmath_eval/__main__.py"
      provides: "mp.prec = 200 evaluator entry point"
      contains: "mp.prec"
  key_links:
    - from: "crates/xcfun-ad/src/math.rs::ctaylor_erf"
      to: "crates/xcfun-ad/src/expand/erf.rs::erf_precise_taylor"
      via: "rewire ctaylor_erf chain to call erf_precise_taylor instead of cubecl Float::erf polyfill"
      pattern: "erf_precise_taylor"
    - from: "crates/xcfun-eval/src/functionals/mgga/tpssc.rs::tpssc_kernel"
      to: "crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs::ctaylor_max"
      via: "tau_clamped insertion at top of kernel body"
      pattern: "ctaylor_max\\("
    - from: "xtask/src/bin/regen_mpmath_fixtures.rs"
      to: "xtask/mpmath_eval/__main__.py"
      via: "std::process::Command::new(\"python3\").arg(\"-m\").arg(\"xtask.mpmath_eval\")"
      pattern: "python3"
---

<objective>
Land the full algebraic substrate for Phase 6 in the CURRENT `xcfun-eval` tree (per D-09: substrate FIRST, git-mv SECOND so Plan 06-01 has zero algebraic deltas). Three coordinated extensions:

1. **AD `N≥4` recursion specialisations** — extend `ctaylor_rec::multo` and `ctaylor_rec::compose` to `N ∈ {4, 5, 6}` per the C++ general-recursion form at `xcfun-master/external/upstream/taylor/ctaylor.hpp:55-65` (multo) and `:72-82` (compose). Unblocks `Mode::Contracted` orders 5..=6 metaGGA (Phase 4 Plan 04-05 D-19 forward).
2. **Libm-hybrid `erf_precise_taylor`** (D-11) — new `#[cube] fn erf_precise_taylor<F: Float, const N: u32>` in `crates/xcfun-ad/src/expand/erf.rs` that seeds `t[0]` via existing `erf_precise` (≤1 ULP scalar precision, already shipped Phase 2 Plan 02-06 commit `dca382a`) and uses derivative-Taylor for `t[i≥1]`. Replaces the `cubecl::Float::erf` polyfill chain in `ctaylor_erf` (`math.rs`). Resolves LDAERFX `6.7e-2`, LDAERFC `4.6e-6`, LDAERFC_JT `4.6e-5` order-3 AD-chain amplification.
3. **TPSS `tau ≥ tau_w` hard-clamp guard** (D-10) — insert `let tau_clamped = ctaylor_max(d.tau, tau_w)` at top of TPSSC / TPSSLOCC / REVTPSSC kernel bodies (use existing `ctaylor_max` at `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818`). Resolves TPSSC `1.09e+30`, TPSSLOCC `8.89e+27`, REVTPSSC `3.73e+15` unphysical-regime divergence (Plan 04-10 Path-B finding).
4. **mpmath sidecar in xtask** (D-04) — Python module under `xtask/mpmath_eval/` invoked via `subprocess::Command::new("python3").arg("-m").arg("xtask.mpmath_eval")` from `xtask/src/bin/regen_mpmath_fixtures.rs`. Emits JSONL fixtures under `validation/fixtures/mpmath/`. Drift gate via `--check` mirrors Phase 2 D-21 `regen-registry --check` pattern. **Python is NOT a runtime/library dep — `cargo build` of xcfun-* crates must NOT require Python3.**

Purpose: Provide the algebraic + ground-truth foundations the rest of Phase 6 depends on. Without N≥4 specialisations, Mode::Contracted orders 5..=6 stay zero-filled. Without `erf_precise_taylor`, the order-3 LDAERF residuals stay catastrophic. Without the tau guard, TPSS-correlation diverges by 1e+27 in the unphysical regime. Without mpmath fixtures, ACC-04 amendment per D-03 has no ground truth.

Output: 6 new ctaylor_rec specialisations + 6 golden-fixture tests + erf_precise_taylor wrapper + 1 erf_taylor_chain test + 3 kernel-body guard insertions + 1 tpss_tau_clamp test + xtask binary + Python sidecar (5 files) + 1 fixtures dir.
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
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@crates/xcfun-ad/src/ctaylor_rec/multo.rs
@crates/xcfun-ad/src/ctaylor_rec/compose.rs
@crates/xcfun-ad/src/expand/erf.rs
@crates/xcfun-ad/src/expand/expm1.rs
@crates/xcfun-ad/src/math.rs
@crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs
@crates/xcfun-eval/src/functionals/mgga/tpssc.rs
@crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs
@crates/xcfun-eval/src/functionals/mgga/revtpssc.rs
@xtask/src/bin/regen_registry.rs
@xtask/src/bin/regen_ad_fixtures.rs
@xcfun-master/external/upstream/taylor/ctaylor.hpp
@xcfun-master/src/functionals/ldaerfx.cpp

<interfaces>
<!-- Existing types/exports the executor needs. Use these directly — no codebase exploration needed. -->

From crates/xcfun-ad/src/ctaylor_rec/multo.rs (existing N=2 base case for template):
```rust
#[cube]
pub(crate) fn ctaylor_multo_n2<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    let d0 = dst[0]; let d1 = dst[1]; let d2 = dst[2]; let d3 = dst[3];
    // dst[3] = d0*y[3] + d3*y[0] + d1*y[2] + d2*y[1]
    let t30 = d0 * y[3]; let t31 = d3 * y[0];
    let t32 = d1 * y[2]; let t33 = d2 * y[1];
    let s1 = t30 + t31; let s2 = s1 + t32;
    dst[3] = s2 + t33;
    // ... dst[2], dst[1], dst[0] in C++-descending order ...
}
```

From crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs:818 (existing — REUSE AS-IS):
```rust
#[cube]
pub fn ctaylor_max<F: Float>(
    a: &Array<F>, b: &Array<F>, out: &mut Array<F>, #[comptime] n: u32,
) {
    let size = comptime!((1_u32 << n) as usize);
    if a[0] >= b[0] {
        #[unroll]
        for i in 0..size { out[i] = a[i]; }
    } else {
        #[unroll]
        for i in 0..size { out[i] = b[i]; }
    }
}
```

From crates/xcfun-ad/src/expand/erf.rs:174-280 (existing — `erf_precise` scalar; SEED for new erf_precise_taylor):
- `pub fn erf_precise<F: Float>(x: F) -> F` — FreeBSD msun-derived port at ≤ 1 ULP vs `libm::erf`. Already used in `erf_expand` for the constant slot.

From crates/xcfun-ad/src/expand/expm1.rs:51-90 (existing — STABLE-BRACKET TEMPLATE):
```rust
#[cube]
pub fn expm1_expand<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    let mut ifac = F::new(1.0);
    t[0] = x0.exp();
    #[unroll]
    for i in 1_u32..=n {
        let k = i as usize;
        let i_f = F::cast_from(i);
        ifac = ifac * i_f;
        t[k] = t[0] / ifac;
    }
    // ... stable-bracket for t[0] when |x0| <= 1e-3 ...
}
```

From C++ ctaylor.hpp:55-65 (general recursion to port):
```cpp
template<typename T, std::size_t Nvar>
struct ctaylor_rec {
    static void multo(T * dst, const T * y) {
        ctaylor_rec<T, Nvar - 1>::multo(dst + POW2(Nvar - 1), y);
        ctaylor_rec<T, Nvar - 1>::mul  (dst + POW2(Nvar - 1), dst, y + POW2(Nvar - 1));
        ctaylor_rec<T, Nvar - 1>::multo(dst, y);
    }
};
```
For N=4: `POW2(3) = 8`, dst length 16; 3 calls total (multo upper + cross-mul + multo lower).
For N=5: `POW2(4) = 16`, dst length 32.
For N=6: `POW2(5) = 32`, dst length 64.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: AD N≥4 multo + compose specialisations + golden fixtures</name>
  <files>crates/xcfun-ad/src/ctaylor_rec/multo.rs, crates/xcfun-ad/src/ctaylor_rec/compose.rs, crates/xcfun-ad/tests/golden_multo_n4.rs, crates/xcfun-ad/tests/golden_multo_n5.rs, crates/xcfun-ad/tests/golden_multo_n6.rs, crates/xcfun-ad/tests/golden_compose_n4.rs, crates/xcfun-ad/tests/golden_compose_n5.rs, crates/xcfun-ad/tests/golden_compose_n6.rs, xtask/src/bin/regen_ad_fixtures.rs</files>
  <read_first>
    - crates/xcfun-ad/src/ctaylor_rec/multo.rs (full file — see N=0/1/2/3 specialisations)
    - crates/xcfun-ad/src/ctaylor_rec/compose.rs (full file — see N=0/1/2/3 specialisations)
    - crates/xcfun-ad/tests/golden_composed.rs (analog test shape; cubecl-cpu launcher pattern)
    - crates/xcfun-ad/tests/golden_mul.rs (analog test shape)
    - xtask/src/bin/regen_ad_fixtures.rs (cc-compile + extractor pattern)
    - xcfun-master/external/upstream/taylor/ctaylor.hpp lines 55-82 (general recursion source)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md §"Plan 06-00" (lines 144-213)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pattern 1" (lines 277-310) and §"Risk R-03" (lines 967-972)
  </read_first>
  <behavior>
    - Test 1 (RED first): `tests/golden_multo_n4.rs` loads bincode fixtures (extended via xtask) of `(dst_in, y, dst_out)` triples for N=4 generated by the C++ extractor binary. For each fixture record, run `ctaylor_multo_n4` via `cubecl-cpu` `launch_unchecked` adapter and assert `assert_relative_eq!(rust_dst[i], cpp_dst[i], max_relative = 1e-12)` for all 16 coefficients.
    - Test 2: same shape for N=5 (32 coefficients).
    - Test 3: same shape for N=6 (64 coefficients).
    - Test 4: `tests/golden_compose_n4.rs` — same shape, calling `ctaylor_compose_n4(out, x, f)` (16 coefficients).
    - Test 5: same shape for N=5 compose (32 coefficients).
    - Test 6: same shape for N=6 compose (64 coefficients).
    - All 6 tests MUST FAIL before implementation lands (RED) and PASS after (GREEN).
  </behavior>
  <action>
**Step A — Extend xtask C++ fixture extractor (per D-21 pattern):**

Update `xtask/src/bin/regen_ad_fixtures.rs` to compile the extractor with extra cases:
- For each `op ∈ {"multo", "compose"}` and `N ∈ {4, 5, 6}`, emit ≥ 100 fixture records using `xoshiro256++` seed `0x1234abcd` (matches Phase 2 D-18 stratification).
- Each record: `{ op, n_var: u8, inputs: Vec<f64>, coeffs: Vec<f64> }` written to `crates/xcfun-ad/tests/fixtures/multo_n{4,5,6}.bincode` and `compose_n{4,5,6}.bincode`.
- Run `cargo run -p xtask --bin regen-ad-fixtures` to generate; commit fixture files.

**Step B — Write 6 RED test files:**

In `crates/xcfun-ad/tests/golden_multo_n4.rs`:
```rust
#![cfg(feature = "testing")]
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use serde::{Deserialize, Serialize};
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::ctaylor_rec;
use approx::assert_relative_eq;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FixtureRecord { op: String, n_var: u8, inputs: Vec<f64>, coeffs: Vec<f64> }

#[cube(launch_unchecked)]
fn kernel_multo_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) {
    ctaylor_rec::multo::ctaylor_multo_n4::<F>(dst, y);
}

#[test]
fn golden_multo_n4() {
    let bytes = include_bytes!("fixtures/multo_n4.bincode");
    let records: Vec<FixtureRecord> = bincode::deserialize(bytes).unwrap();
    let client = cpu_client();
    for rec in &records {
        // dst length = 1<<4 = 16; y length = 16; inputs are dst_in (16) + y (16) = 32 doubles
        // expected output is in rec.coeffs (16 doubles)
        // ... launch kernel via launch_unchecked, read_one, compare element-by-element ...
        for i in 0..16 {
            assert_relative_eq!(out_rust[i], rec.coeffs[i], max_relative = 1e-12);
        }
    }
}
```
Repeat for `golden_multo_n5.rs` (length 32), `golden_multo_n6.rs` (length 64), `golden_compose_n{4,5,6}.rs`.

Run `cargo nextest run -p xcfun-ad --test golden_multo_n4` — MUST FAIL (RED) because `ctaylor_multo_n4` does not yet exist.

**Step C — Implement N=4 multo (extend `crates/xcfun-ad/src/ctaylor_rec/multo.rs`):**

Following the C++ recursion `multo(dst, y) = { multo_n3(dst+8, y); mul_n3(dst+8, dst, y+8); multo_n3(dst, y); }` from `ctaylor.hpp:55-65`, but cubecl 0.10-pre.3 cannot sub-slice `Array<F>` (verified by N=3 pattern). The N=4 implementation:

1. Capture all 16 dst values into local `let d0, d1, ..., d15` BEFORE any writes (matches N=2/N=3 pattern).
2. Compute the new dst[i] for each `i` in C++ DESCENDING order (i = 15, 14, ..., 0) using the recursion-equivalent algebra. The output coefficient at bit-mask index `i` = sum over all `(j, k)` such that `j | k == i` and `j & k == 0` of `dst_in[j] * y[k]`.
3. Use `let s1 = ...; let s2 = s1 + ...; ...` accumulation in the SAME order C++ uses (left-associative, no FMA, no `mul_add`). The `xtask check-no-mul-add` gate (Phase 2 D-13) will block any FMA-style accumulation.

Recommendation per RESEARCH §Pattern 1: macro-generate the bodies for N=4/5/6 to keep ~1000 LOC for N=6 maintainable. A `macro_rules! ctaylor_multo_specialise` taking `N` as input and emitting the per-N body keeps the C++ recursion structure visible.

Add public-crate exports:
```rust
#[cube]
pub(crate) fn ctaylor_multo_n4<F: Float>(dst: &mut Array<F>, y: &Array<F>) { /* 16-coeff body */ }
#[cube]
pub(crate) fn ctaylor_multo_n5<F: Float>(dst: &mut Array<F>, y: &Array<F>) { /* 32-coeff body */ }
#[cube]
pub(crate) fn ctaylor_multo_n6<F: Float>(dst: &mut Array<F>, y: &Array<F>) { /* 64-coeff body */ }
```

Wire the outer `ctaylor_multo<F>(dst, y, n)` dispatch (existing comptime if-chain in same file) to add arms for `n == 4`, `n == 5`, `n == 6`.

**Step D — Implement N=4/5/6 compose (extend `crates/xcfun-ad/src/ctaylor_rec/compose.rs`):**

Following C++ `ctaylor.hpp:72-82` descending-i Horner form. Per RESEARCH Pitfall 7: outer loop MUST be descending (`for i in (0..N).rev()`); reversing breaks 1e-12 parity for n ≥ 2. Use existing `ctaylor_multo_skipconst_n{3,4,5}` from `multo.rs` (Step C also extends `multo_skipconst` for N=4/5).

Add:
```rust
#[cube]
pub(crate) fn ctaylor_compose_n4<F: Float>(out: &mut Array<F>, x: &Array<F>, f: &Array<F>) { /* 16-coeff body */ }
#[cube]
pub(crate) fn ctaylor_compose_n5<F: Float>(out: &mut Array<F>, x: &Array<F>, f: &Array<F>) { /* 32-coeff body */ }
#[cube]
pub(crate) fn ctaylor_compose_n6<F: Float>(out: &mut Array<F>, x: &Array<F>, f: &Array<F>) { /* 64-coeff body */ }
```

Wire the outer `ctaylor_compose<F>(out, x, f, n)` dispatch to add arms for `n == 4`, `n == 5`, `n == 6`.

**Step E — Re-run all 6 tests (GREEN):**

`cargo nextest run -p xcfun-ad --test golden_multo_n4 --test golden_multo_n5 --test golden_multo_n6 --test golden_compose_n4 --test golden_compose_n5 --test golden_compose_n6` MUST PASS at strict `max_relative = 1e-12`.

**Forbidden:**
- No `mul_add(...)` calls (`xtask check-no-mul-add` will block; `let s = a + b` instead).
- No reassociation that differs from C++ left-to-right accumulation order.
- No `Float::erf` polyfill use; do not introduce `fast_math` attributes.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-ad --test golden_multo_n4 --test golden_multo_n5 --test golden_multo_n6 --test golden_compose_n4 --test golden_compose_n5 --test golden_compose_n6</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "fn ctaylor_multo_n4" crates/xcfun-ad/src/ctaylor_rec/multo.rs` >= 1
    - `grep -c "fn ctaylor_multo_n5" crates/xcfun-ad/src/ctaylor_rec/multo.rs` >= 1
    - `grep -c "fn ctaylor_multo_n6" crates/xcfun-ad/src/ctaylor_rec/multo.rs` >= 1
    - `grep -c "fn ctaylor_compose_n4" crates/xcfun-ad/src/ctaylor_rec/compose.rs` >= 1
    - `grep -c "fn ctaylor_compose_n5" crates/xcfun-ad/src/ctaylor_rec/compose.rs` >= 1
    - `grep -c "fn ctaylor_compose_n6" crates/xcfun-ad/src/ctaylor_rec/compose.rs` >= 1
    - `grep -v '^//' crates/xcfun-ad/src/ctaylor_rec/multo.rs | grep -c 'mul_add'` == 0
    - `grep -v '^//' crates/xcfun-ad/src/ctaylor_rec/compose.rs | grep -c 'mul_add'` == 0
    - `cargo nextest run -p xcfun-ad --test golden_multo_n4 --test golden_multo_n5 --test golden_multo_n6 --test golden_compose_n4 --test golden_compose_n5 --test golden_compose_n6` exits 0.
    - Fixture files `crates/xcfun-ad/tests/fixtures/multo_n{4,5,6}.bincode` and `compose_n{4,5,6}.bincode` exist and are non-empty.
  </acceptance_criteria>
  <done>All 6 N≥4 specialisations exported and wired into the comptime dispatch; all 6 golden tests GREEN at strict 1e-12; xtask `regen-ad-fixtures` extended; no `mul_add` in either modified file.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Libm-hybrid erf_precise_taylor + AD-chain rewire (D-11)</name>
  <files>crates/xcfun-ad/src/expand/erf.rs, crates/xcfun-ad/src/math.rs, crates/xcfun-ad/tests/erf_taylor_chain.rs</files>
  <read_first>
    - crates/xcfun-ad/src/expand/erf.rs (full file — existing erf_precise at line 174 + erf_expand at line 321)
    - crates/xcfun-ad/src/expand/expm1.rs (full file — stable-bracket pattern from Phase 2 Plan 02-06 Fix 1)
    - crates/xcfun-ad/src/math.rs (search for `ctaylor_erf` — current implementation rewrites to call new wrapper)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md lines 215-240 ("erf.rs add erf_precise_taylor")
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"D-11 erf_precise_taylor" + Code Examples
    - xcfun-master/src/functionals/ldaerfx.cpp:66 (test_threshold rationale)
  </read_first>
  <behavior>
    - Test 1 (RED first): `tests/erf_taylor_chain.rs` evaluates erf_precise_taylor at orders n ∈ {1, 2, 3, 4} with x[0] ∈ {0.1, 0.5, 1.0, 2.0, 5.0} (a stratified grid covering the bracket-cancellation regime per ldaerfx.cpp:66 rationale). Compare against mpmath ground truth at prec=200 (read from a committed JSONL fixture under `crates/xcfun-ad/tests/fixtures/erf_taylor_chain.jsonl`).
    - Tolerance: `max_relative = 1e-13` (one order tighter than the project Core Value 1e-12 — establishes the libm-hybrid budget per RESEARCH §6 tolerance-budget breakdown).
    - Test asserts every coefficient `t[i]` of the resulting CTaylor matches mpmath truth at strict 1e-13.
    - Test MUST FAIL before erf_precise_taylor exists (RED) and PASS after (GREEN).
  </behavior>
  <action>
**Step A — Generate mpmath fixture (one-shot manual run during this task):**

Create `crates/xcfun-ad/tests/fixtures/erf_taylor_chain.jsonl` with ≥ 50 records:
```json
{"x0": 0.5, "n": 3, "expected": [0.520499877813047..., ...], "mpmath_prec": 200, "source": "mpmath"}
```
Use a one-off Python script (or invoke the new `xtask/mpmath_eval/` from Task 4 if landed first) running `mpmath.erf` then computing derivatives via `mpmath.diff`. Stratified inputs covering small (`|x0| < 0.1`) and large (`|x0| > 1.5`) brackets where C++ exhibits cancellation.

**Step B — Write RED test:**

```rust
// crates/xcfun-ad/tests/erf_taylor_chain.rs
#![cfg(feature = "testing")]
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use serde::{Deserialize, Serialize};
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::expand::erf as erf_mod;
use approx::assert_relative_eq;

#[derive(Serialize, Deserialize, Debug)]
struct ErfTaylorRecord { x0: f64, n: u32, expected: Vec<f64>, mpmath_prec: u32 }

#[cube(launch_unchecked)]
fn kernel_erf_taylor<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    erf_mod::erf_precise_taylor::<F>(t, x0, n);
}

#[test]
fn erf_taylor_chain() {
    let lines = std::fs::read_to_string("tests/fixtures/erf_taylor_chain.jsonl").unwrap();
    for line in lines.lines() {
        let rec: ErfTaylorRecord = serde_json::from_str(line).unwrap();
        // ... launch kernel, read_one, compare ...
        for i in 0..=rec.n as usize {
            assert_relative_eq!(out_rust[i], rec.expected[i], max_relative = 1e-13);
        }
    }
}
```

Run `cargo nextest run -p xcfun-ad --test erf_taylor_chain` — MUST FAIL (RED) because `erf_precise_taylor` does not yet exist.

**Step C — Implement `erf_precise_taylor` (extend `crates/xcfun-ad/src/expand/erf.rs`):**

Add the wrapper at the bottom of the file (after existing `erf_expand` at line 321). Pattern follows `expm1_expand` (`crates/xcfun-ad/src/expand/expm1.rs:51-90`):

```rust
/// Phase 6 D-11 — libm-hybrid erf wrapper for the AD chain.
///
/// Seeds `t[0]` via the FreeBSD msun-port `erf_precise(x0)` (≤ 1 ULP scalar
/// precision; landed Phase 2 commit `dca382a`). For `t[i ≥ 1]`, uses the
/// derivative chain: d/dx erf(x) = (2/√π) · exp(-x²), then higher derivatives
/// via Hermite-polynomial recurrence — algebraically identical to the existing
/// `erf_expand` body (lines 321 onward), but seeded at the more-precise
/// constant slot.
///
/// Resolves Phase-4 D-19 LDAERFX (6.7e-2), LDAERFC (4.6e-6), LDAERFC_JT (4.6e-5)
/// order-3 AD-chain amplification per RESEARCH.md §D-11 + ldaerfx.cpp:66.
#[cube]
pub fn erf_precise_taylor<F: Float>(t: &mut Array<F>, x0: F, #[comptime] n: u32) {
    // Step 1: seed t[0] with the libm-precision scalar erf.
    t[0] = erf_precise::<F>(x0);
    // Step 2: t[i ≥ 1] via the existing Hermite-polynomial body of erf_expand,
    //         lifted into a private helper that takes t[0] as input rather than
    //         computing it via cubecl::Float::erf polyfill.
    // ... (≈ 60 LOC mirroring erf_expand:321-380 with the t[0] seed parameter) ...
}
```

Make `erf_precise` (existing scalar at line 174) `pub(crate)` if currently private; export `erf_precise_taylor` as `pub`.

**Step D — Rewire `ctaylor_erf` in `crates/xcfun-ad/src/math.rs`:**

Find existing `ctaylor_erf` (composed op that lifts scalar erf into the AD chain via cubecl `Float::erf` polyfill). Change the seed step to call `erf_precise_taylor` instead. Concrete change:

Search and replace the existing seed-via-`Float::erf` line (likely `let e0 = x[0].erf();`) with a call to the new wrapper:
```rust
// OLD (polyfill seed):
//   let e0 = x[0].erf();
//   t[0] = e0;
//   ... derivative chain via Float::erf ...
// NEW (libm-hybrid seed via erf_precise_taylor):
let mut t_seed = Array::<F>::new(comptime!((1_u32 << n) as usize));
erf_precise_taylor::<F>(&mut t_seed, x[0], n);
// ... compose t_seed with the existing ctaylor_compose recursion ...
```

The exact diff depends on the current `ctaylor_erf` body; the executor reads `crates/xcfun-ad/src/math.rs` first and applies the equivalent rewire.

**Step E — Re-run test (GREEN):**

`cargo nextest run -p xcfun-ad --test erf_taylor_chain` MUST PASS at strict 1e-13.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-ad --test erf_taylor_chain</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "pub fn erf_precise_taylor" crates/xcfun-ad/src/expand/erf.rs` >= 1
    - `grep -c "erf_precise_taylor" crates/xcfun-ad/src/math.rs` >= 1
    - `crates/xcfun-ad/tests/erf_taylor_chain.rs` exists and references `erf_precise_taylor` AND `mpmath_prec`.
    - `crates/xcfun-ad/tests/fixtures/erf_taylor_chain.jsonl` exists and is non-empty.
    - `cargo nextest run -p xcfun-ad --test erf_taylor_chain` exits 0.
    - `grep -v '^//' crates/xcfun-ad/src/expand/erf.rs | grep -c 'mul_add'` == 0
  </acceptance_criteria>
  <done>erf_precise_taylor exported as `pub fn`; ctaylor_erf in math.rs rewired to call it (no `Float::erf` polyfill seed remains); erf_taylor_chain test GREEN at strict 1e-13.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: TPSS tau ≥ tau_w hard-clamp guard (D-10) + tau_clamp test</name>
  <files>crates/xcfun-eval/src/functionals/mgga/tpssc.rs, crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs, crates/xcfun-eval/src/functionals/mgga/revtpssc.rs, crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs, crates/xcfun-eval/tests/tpss_tau_clamp.rs</files>
  <read_first>
    - crates/xcfun-eval/src/functionals/mgga/tpssc.rs (current kernel body — see how `d.tau` is consumed)
    - crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs (same)
    - crates/xcfun-eval/src/functionals/mgga/revtpssc.rs (same)
    - crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs lines 814-838 (existing `ctaylor_max` to REUSE)
    - crates/xcfun-eval/tests/regularize_mgga_invariant.rs (analog test pattern)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md lines 242-273
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pattern 5" (lines 466-494)
    - .planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md §"D-19" (TPSS Path-B finding)
  </read_first>
  <behavior>
    - Test 1 (RED first): `tests/tpss_tau_clamp.rs` constructs density grid points where `tau << tau_w` (the unphysical von Weizsäcker bound violation regime per Phase 4 Plan 04-10 Path-B finding). Specifically: `rho = 1.0`, `|∇rho|² = 100.0` (so `tau_w = |∇rho|²/(8 rho) = 12.5`), `tau ∈ {1e-6, 1e-3, 0.1}` (all below tau_w).
    - Run `tpssc_kernel`, `tpsslocc_kernel`, `revtpssc_kernel` at order 1 via cubecl-cpu launcher. Assert output is finite (not NaN, not infinity) AND that the value matches the kernel evaluated with `tau = tau_w` exactly (because the guard collapses `d.tau` to `tau_w` in this regime).
    - In the physical regime (`tau >= tau_w`, e.g., `tau = 100.0`, `tau_w = 12.5`), assert output is BIT-EXACT to the pre-guard kernel evaluation (because `ctaylor_max(tau, tau_w) == tau` and propagates through the same arithmetic).
    - Test MUST FAIL before guard insertion (RED — pre-guard kernel produces 1e+27 magnitudes) and PASS after (GREEN — output bounded).
  </behavior>
  <action>
**Step A — Write RED test:**

```rust
// crates/xcfun-eval/tests/tpss_tau_clamp.rs
#![cfg(feature = "testing")]
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use xcfun_eval::for_tests::cpu_client;
use xcfun_eval::density_vars::{DensVarsDev, build::build_densvars};
use xcfun_eval::functionals::mgga::{tpssc, tpsslocc, revtpssc};

#[cube(launch_unchecked)]
fn kernel_tpssc<F: Float>(input: &Array<F>, d: &mut DensVarsDev<F>, out: &mut Array<F>,
                           #[comptime] vars: u32, #[comptime] n: u32) {
    build_densvars::<F>(input, d, vars, n);
    tpssc::tpssc_kernel::<F>(d, out, n);
}

#[test]
fn tpssc_unphysical_regime_bounded() {
    // tau << tau_w: rho=1.0, |∇rho|²=100.0 → tau_w=12.5; tau=1e-6
    // Pre-guard: order-1 derivative blows up to 1e+27 (Plan 04-10 finding).
    // Post-guard: bounded.
    let input = vec![1.0, 1.0, /* gradient terms */ 100.0, /* tau */ 1e-6, /* lapl */ 0.0];
    let client = cpu_client();
    // ... launch kernel at order 1 (n=1, vars = a_b_gnn_gns_gss_lapla_laplb_taua_taub) ...
    for &v in &out_rust {
        assert!(v.is_finite(), "TPSSC unphysical-regime guard breached: {} non-finite", v);
        assert!(v.abs() < 1e10, "TPSSC unphysical-regime guard breached: {} > 1e10", v);
    }
}
// Repeat for tpsslocc and revtpssc.

#[test]
fn tpssc_physical_regime_bit_exact_to_pre_guard_baseline() {
    // tau >= tau_w: rho=1.0, |∇rho|²=100.0 → tau_w=12.5; tau=100.0
    // ctaylor_max(tau, tau_w) == tau; outputs bit-exact to pre-guard.
    // ... assert outputs match a hand-curated baseline ...
}
```

Run `cargo nextest run -p xcfun-eval --test tpss_tau_clamp` — MUST FAIL (RED).

**Step B — Add `build_tau_w` helper in `tpss_like.rs`:**

If not already present (search first), add to `crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs`:
```rust
/// Phase 6 D-10 — von Weizsäcker tau_w = |∇ρ|² / (8 ρ).
/// Used inside the TPSS-correlation `tau ≥ tau_w` clamp guard.
#[cube]
pub fn build_tau_w<F: Float>(d: &DensVarsDev<F>, tau_w: &mut Array<F>, #[comptime] n: u32) {
    // tau_w = (gnn + 2*gns + gss) / (8 * (a + b))   for vars=A_B_GAA_GAB_GBB_LAPLA_LAPLB_TAUA_TAUB
    // ... ~10 LOC using ctaylor_div / ctaylor_mul existing primitives ...
}
```

**Step C — Insert guard in TPSSC / TPSSLOCC / REVTPSSC kernel bodies:**

In `crates/xcfun-eval/src/functionals/mgga/tpssc.rs`, modify `tpssc_kernel`:
```rust
#[cube]
pub fn tpssc_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32) {
    let size = comptime!((1_u32 << n) as usize);
    // Phase 6 D-10: hard-clamp tau to tau_w to guard the unphysical regime
    // where von Weizsäcker bound is violated by f64-rounding cancellation.
    // Plan 04-10 Path-B confirmed algorithmically faithful port; the divergence
    // (tauwtau3 ≈ 1e+27 amplifying ULP differences) is pure f64 cancellation
    // in tau << tau_w regime. ACC-04 mpmath amendment (D-03) covers the boundary.
    let mut tau_w = Array::<F>::new(size);
    tpss_like::build_tau_w::<F>(d, &mut tau_w, n);
    let mut tau_clamped = Array::<F>::new(size);
    tpss_like::ctaylor_max::<F>(&d.tau, &tau_w, &mut tau_clamped, n);
    // Existing kernel logic, but with `tau_clamped` substituted everywhere d.tau was used:
    let mut eps = Array::<F>::new(size);
    tpss_like::tpss_eps_full_with_tau::<F>(d, &tau_clamped, &mut eps, n);  // new helper signature
    ctaylor_mul::<F>(&d.n, &eps, out, n);
}
```

If `tpss_eps_full` reads `d.tau` directly (not via parameter), add a `tpss_eps_full_with_tau<F>(d, tau_clamped, eps, n)` variant in `tpss_like.rs` that takes the clamped tau as an explicit `&Array<F>` parameter. Same pattern for `tpss_locc_eps_full` and `revtpss_eps_full`.

Repeat the guard insertion in `tpsslocc.rs` (calls `tpss_like::tpss_locc_eps_full`) and `revtpssc.rs` (calls `tpss_like::revtpss_eps_full`). All three kernels gain the same 4-line preamble.

**Step D — Re-run test (GREEN):**

`cargo nextest run -p xcfun-eval --test tpss_tau_clamp` MUST PASS.

**Forbidden:**
- Do NOT use `regularize`-style mutation on `d.tau[0]` directly — `regularize` per Phase 2 D-22 mutates only `c[CNST]`. The guard wants the larger of two CTaylors, not a CNST-floor. Use `ctaylor_max`.
- Do NOT add the guard to other tau-using metaGGAs (M06L, SCAN, etc.) in this task — scope is TPSSC/TPSSLOCC/REVTPSSC per D-10.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-eval --test tpss_tau_clamp</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "ctaylor_max" crates/xcfun-eval/src/functionals/mgga/tpssc.rs` >= 1
    - `grep -c "ctaylor_max" crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs` >= 1
    - `grep -c "ctaylor_max" crates/xcfun-eval/src/functionals/mgga/revtpssc.rs` >= 1
    - `grep -c "build_tau_w" crates/xcfun-eval/src/functionals/mgga/shared/tpss_like.rs` >= 1
    - `crates/xcfun-eval/tests/tpss_tau_clamp.rs` exists with `tpssc_unphysical_regime_bounded` AND `tpssc_physical_regime_bit_exact_to_pre_guard_baseline` test functions.
    - `cargo nextest run -p xcfun-eval --test tpss_tau_clamp` exits 0.
    - Existing tier-1 self-tests for TPSSC/TPSSLOCC/REVTPSSC still pass: `cargo nextest run -p xcfun-eval --features testing --test self_tests` exits 0.
  </acceptance_criteria>
  <done>Guard landed in all 3 TPSS-correlation kernels; helper `build_tau_w` in tpss_like.rs; tau_clamp test GREEN; existing self_tests still GREEN (no regression in physical regime).</done>
</task>

<task type="auto">
  <name>Task 4: mpmath sidecar in xtask + regen-mpmath-fixtures binary (D-04)</name>
  <files>xtask/src/bin/regen_mpmath_fixtures.rs, xtask/mpmath_eval/__init__.py, xtask/mpmath_eval/__main__.py, xtask/mpmath_eval/evaluator.py, xtask/mpmath_eval/functionals.py, xtask/mpmath_eval/README.md, validation/fixtures/mpmath/.gitkeep, xtask/Cargo.toml</files>
  <read_first>
    - xtask/src/bin/regen_registry.rs (full file — `--check` drift gate pattern + extractor + SHA-256 stamp)
    - xtask/src/bin/regen_ad_fixtures.rs (cc-compile-then-run subprocess pattern)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md lines 275-330 ("regen_mpmath_fixtures" + "mpmath_eval/*.py")
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"mpmath Sidecar Architecture" (lines 812-907)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md D-04 (Python sidecar contract)
  </read_first>
  <action>
**Step A — Create Python sidecar package layout** under `xtask/mpmath_eval/`:

`xtask/mpmath_eval/__init__.py` (empty marker file).

`xtask/mpmath_eval/__main__.py`:
```python
"""Phase 6 D-04 — mpmath sidecar entry point.

Invoked from Rust via:
  python3 -m xtask.mpmath_eval --functional <name> --vars <Vars> --mode <Mode>
                                --order <u32> --input <comma-sep> --prec 200

Emits one JSONL record on stdout. Reproducibility: mp.prec=200 + mpmath>=1.4.
"""
import argparse, json, sys
import mpmath
from .evaluator import eval_record

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--functional', required=True)
    p.add_argument('--vars', required=True)
    p.add_argument('--mode', required=True)
    p.add_argument('--order', type=int, required=True)
    p.add_argument('--input', required=True)
    p.add_argument('--prec', type=int, default=200)
    args = p.parse_args()
    mpmath.mp.prec = args.prec
    inputs = [mpmath.mpf(x) for x in args.input.split(',')]
    record = eval_record(args.functional, args.vars, args.mode, args.order, inputs, args.prec)
    json.dump(record, sys.stdout)
    sys.stdout.write('\n')

if __name__ == '__main__':
    main()
```

`xtask/mpmath_eval/evaluator.py`:
```python
"""Generic Taylor-series AD chain at mp.prec=200 + functional dispatch."""
import mpmath
from .functionals import LOOKUP

def eval_record(functional, vars_str, mode, order, inputs, prec):
    fn = LOOKUP[functional.lower()]
    output = fn(inputs, vars=vars_str, mode=mode, order=order)
    return {
        'functional': functional,
        'vars': vars_str,
        'mode': mode,
        'order': order,
        'input': [float(x) for x in inputs],
        'output': [float(x) for x in output],
        'mpmath_prec': prec,
        'source': 'mpmath',
    }
```

**(B-3 revision-1) `xtask/mpmath_eval/functionals/` package directory** (NOT a single `functionals.py` file). Creates the package layout from day one so Plan 06-N2 only adds new module files into the existing package — eliminating the mid-execution restructure risk.

`xtask/mpmath_eval/functionals/__init__.py`:
```python
"""mpmath ports of ACC-04-amended functionals (Plan 06-00 LDAERF + TPSS family;
Plan 06-N2 extends to the 20 excluded_by_upstream_spec set: BR×3, SCAN×10, CSC,
BLOCX, TW, VWK, PBELOCC, ZVPBESOLC, ZVPBEINTC).

Each per-functional module exposes `eval_<name>(inputs, vars, mode, order)`
taking mp.mpf inputs and returning a list of mp.mpf outputs.
"""
from . import ldaerfx, ldaerfc, ldaerfc_jt, tpssc, tpsslocc, revtpssc
# Plan 06-N2 imports brx, brc, brxc, csc, blocx, scan, tw, vwk, pbelocc,
# zvpbesolc, zvpbeintc into this same package.

LOOKUP = {
    'ldaerfx': ldaerfx.eval_ldaerfx,
    'ldaerfc': ldaerfc.eval_ldaerfc,
    'ldaerfc_jt': ldaerfc_jt.eval_ldaerfc_jt,
    'tpssc': tpssc.eval_tpssc,
    'tpsslocc': tpsslocc.eval_tpsslocc,
    'revtpssc': revtpssc.eval_revtpssc,
    # Plan 06-N2 extends with the 20 excluded_by_upstream_spec entries.
}
```

Then create per-functional skeleton modules, each one ~10-line stub raising
`NotImplementedError("Plan 06-N2 populates this body")` so the package layout is
already in place AND `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP"`
imports cleanly. Example (`xtask/mpmath_eval/functionals/ldaerfx.py`):

```python
"""mpmath port of xcfun-master/src/functionals/ldaerfx.cpp at mp.prec=200."""
import mpmath as mp

def eval_ldaerfx(inputs, vars, mode, order):
    # Plan 06-N2 populates with mpmath verbatim port. For Plan 06-00 substrate
    # work, this stub is enough — fixture generation in Plan 06-00 Task 4 only
    # validates the SUBPROCESS WIRING (Rust driver → python3 -m xtask.mpmath_eval),
    # not the functional bodies. Plan 06-N2 fills bodies AND generates fixtures.
    raise NotImplementedError("Plan 06-N2 populates this body")
```

Repeat for `ldaerfc.py`, `ldaerfc_jt.py`, `tpssc.py`, `tpsslocc.py`, `revtpssc.py`.

**Also create supporting modules** that Plan 06-N2 will use:
- `xtask/mpmath_eval/ad_chain.py` — generic Taylor-series AD chain at mp.prec=200 (stub: `def taylor_coeffs(f, x0, order, prec=200): raise NotImplementedError`).
- `xtask/mpmath_eval/densvars.py` — DensVars equivalent at prec=200 (stub: `def build_densvars(inputs, vars): raise NotImplementedError`).

Plan 06-N2 fills these stubs without restructuring the package layout.

`xtask/mpmath_eval/README.md`:
```markdown
# xtask.mpmath_eval — mpmath sidecar (Phase 6 D-04)

Python module providing 200-digit-precision ground truth for ACC-04-amended
functionals (LDAERF family, TPSS-correlation family) and Plan 06-N2's
20 `excluded_by_upstream_spec` set.

## Reproducibility

- Python: `>= 3.10` (project local: 3.14.4 — verified 2026-04-30)
- mpmath: `>= 1.4, < 2.0` (project local: 1.4.1 — verified)
- mp.prec = 200 (set in `__main__.py`); deterministic for fixed input

## Invocation

```bash
python3 -m xtask.mpmath_eval --functional ldaerfx --vars XC_A_B \
    --mode PartialDerivatives --order 3 --input 0.5,0.5 --prec 200
```

Emits one JSONL record on stdout. Used exclusively by
`xtask/src/bin/regen_mpmath_fixtures.rs`.

## NOT a runtime/library dep

`cargo build` of any `xcfun-*` library crate (xcfun-ad, xcfun-core,
xcfun-kernels post-Plan-06-01, xcfun-eval, xcfun-rs, xcfun-capi, xcfun-gpu)
does NOT require Python3. This module lives in the `xtask/` build-tools tree
and is invoked via `subprocess::Command::new("python3")` ONLY at fixture
regeneration time.
```

**Step B — Create Rust driver `xtask/src/bin/regen_mpmath_fixtures.rs`:**

```rust
//! Phase 6 D-04 — Generate mpmath JSONL fixtures.
//!
//! Workflow (mirrors `regen_registry.rs` pattern):
//!   1. Iterate the request matrix (functional × vars × mode × order × input grid).
//!   2. For each request, spawn `python3 -m xtask.mpmath_eval` with arguments;
//!      capture the JSONL record on stdout.
//!   3. Append records to `validation/fixtures/mpmath/<functional>.jsonl`.
//!   4. Compute SHA-256 of each file; write `<functional>.jsonl.sha256` stamp.
//!   5. `--check` mode regenerates in memory and compares stamps; exit 2 on drift.
//!
//! mpmath dep stays in Python land — Cargo build path NEVER calls Python (D-04).

use anyhow::{Context, Result, bail};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::process::Command;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let check_mode = args.iter().any(|a| a == "--check");

    // Phase 6 Plan 06-00 set: 6 ACC-04-amended functionals × ~50 records each.
    // Plan 06-N2 will extend this set with the 20 `excluded_by_upstream_spec` functionals.
    let functionals = ["ldaerfx", "ldaerfc", "ldaerfc_jt", "tpssc", "tpsslocc", "revtpssc"];
    let workspace_root = std::env::current_dir()?;

    for fn_name in &functionals {
        let mut buf = String::new();
        for input_record in stratified_grid(fn_name) {  // 50 stratified records per functional
            let output = Command::new("python3")
                .arg("-m").arg("xtask.mpmath_eval")
                .args(["--functional", fn_name])
                .args(["--vars", &input_record.vars])
                .args(["--mode", &input_record.mode])
                .args(["--order", &input_record.order.to_string()])
                .args(["--input", &input_record.input_csv])
                .args(["--prec", "200"])
                .current_dir(&workspace_root)
                .output()
                .with_context(|| format!("python3 -m xtask.mpmath_eval failed for {}", fn_name))?;
            if !output.status.success() {
                bail!("mpmath sidecar failed for {}: {}", fn_name, String::from_utf8_lossy(&output.stderr));
            }
            buf.push_str(&String::from_utf8(output.stdout)?);
        }

        let target = workspace_root.join("validation/fixtures/mpmath").join(format!("{}.jsonl", fn_name));
        let stamp = workspace_root.join("validation/fixtures/mpmath").join(format!("{}.jsonl.sha256", fn_name));

        let mut hasher = Sha256::new();
        hasher.update(buf.as_bytes());
        let hex = format!("{:x}", hasher.finalize());

        if check_mode {
            let existing = std::fs::read_to_string(&stamp).context("missing committed sha256 stamp")?;
            if existing.trim() != hex {
                bail!("mpmath fixture drift detected for {}: expected {}, got {}", fn_name, existing.trim(), hex);
            }
        } else {
            std::fs::create_dir_all(target.parent().unwrap())?;
            std::fs::write(&target, &buf)?;
            std::fs::write(&stamp, &hex)?;
        }
    }
    Ok(())
}

// stratified_grid stub — populated with xoshiro256++ seed 0x1234abcd grid per Phase 2 D-18.
struct InputRecord { vars: String, mode: String, order: u32, input_csv: String }
fn stratified_grid(fn_name: &str) -> Vec<InputRecord> { vec![/* 50 records */] }
```

**Step C — Update `xtask/Cargo.toml`** to declare the new `[[bin]]`:
```toml
[[bin]]
name = "regen-mpmath-fixtures"
path = "src/bin/regen_mpmath_fixtures.rs"
```

If `sha2` is not already in xtask deps, add `sha2 = "0.10"` (already used by `regen_registry.rs`).

**Step D — Create `validation/fixtures/mpmath/.gitkeep`** so the dir exists before fixtures land.

**Step E — Smoke test:**

```bash
cargo build -p xtask --bin regen-mpmath-fixtures             # must compile
python3 -c "import xtask.mpmath_eval"                         # must NOT raise
python3 -c "from xtask.mpmath_eval.functionals import LOOKUP"  # B-3: package-style import OK
python3 -c "from xtask.mpmath_eval.functionals.ldaerfx import eval_ldaerfx"  # B-3: per-module import OK
```

Note: `cargo run -p xtask --bin regen-mpmath-fixtures` will fail because the per-functional bodies are `NotImplementedError` stubs. Populating them is OUT OF SCOPE for Plan 06-00 substrate work — Plan 06-N2 fills the bodies for the 20 `excluded_by_upstream_spec` set + the 6 ACC-04-amended set. The B-3 (revision-1) package restructure ensures Plan 06-N2 only adds NEW module files into the existing `functionals/` directory rather than performing a mid-execution `mv functionals.py functionals/` restructure (which the original Plan 06-N2 had to do).
  </action>
  <verify>
    <automated>cargo build -p xtask --bin regen-mpmath-fixtures && python3 -c "import xtask.mpmath_eval; from xtask.mpmath_eval.functionals import LOOKUP; from xtask.mpmath_eval.functionals.ldaerfx import eval_ldaerfx"</automated>
  </verify>
  <acceptance_criteria>
    - `xtask/src/bin/regen_mpmath_fixtures.rs` exists.
    - `xtask/mpmath_eval/__init__.py`, `__main__.py`, `evaluator.py`, `README.md` all exist.
    - **(B-3 revision-1)** `xtask/mpmath_eval/functionals/` is a Python package directory (NOT a single .py file): `test -d xtask/mpmath_eval/functionals && test -f xtask/mpmath_eval/functionals/__init__.py` succeeds.
    - Per-functional stub modules exist for the 6 ACC-04-amended set: `find xtask/mpmath_eval/functionals -name 'ldaerfx.py' -o -name 'ldaerfc.py' -o -name 'ldaerfc_jt.py' -o -name 'tpssc.py' -o -name 'tpsslocc.py' -o -name 'revtpssc.py' | wc -l` >= 6.
    - Supporting modules exist: `test -f xtask/mpmath_eval/ad_chain.py && test -f xtask/mpmath_eval/densvars.py`.
    - `python3 -c "from xtask.mpmath_eval.functionals import LOOKUP; assert len(LOOKUP) >= 6"` exits 0.
    - `validation/fixtures/mpmath/.gitkeep` exists.
    - `cargo build -p xtask --bin regen-mpmath-fixtures` succeeds.
    - `python3 -c "import xtask.mpmath_eval; from xtask.mpmath_eval.evaluator import eval_record"` exits 0.
    - `grep -c "subprocess\|Command::new(\"python3\")" xtask/src/bin/regen_mpmath_fixtures.rs` >= 1
    - `grep -c "mp.prec = 200\|mpmath.mp.prec = args.prec" xtask/mpmath_eval/__main__.py` >= 1
    - `grep -c "NOT a runtime\|NOT.*runtime/library" xtask/mpmath_eval/README.md` >= 1
    - No Python imports introduced into `crates/xcfun-*` library crates: `grep -rl 'use python\|pyo3' crates/xcfun-ad crates/xcfun-core crates/xcfun-eval crates/xcfun-rs crates/xcfun-capi 2>/dev/null` returns empty.
  </acceptance_criteria>
  <done>mpmath sidecar landed at xtask boundary; Python sidecar package importable; Rust driver compiles and shells out via `Command::new("python3")`; `validation/fixtures/mpmath/` directory exists; `cargo build -p xtask --bin regen-mpmath-fixtures` GREEN. Functional bodies left as `NotImplementedError` stubs for Plan 06-N2.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Rust ↔ cubecl-cpu MLIR JIT | Algorithmic-identity contract enforced at 1e-12; cubecl 0.10-pre.3 pre-release API churn (Pitfall 4) |
| xtask Rust driver ↔ Python sidecar | `Command::new("python3")` subprocess; argument list is hardcoded in driver, no user input pass-through |
| Library graph ↔ Python | None — `cargo build` of xcfun-* never invokes Python (D-04 contract) |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-FAST-MATH | high | `-Cfast-math` / `mul_add` reassociation breaks 1e-13 parity in N≥4 specialisations | `xtask check-no-mul-add` extends to new ctaylor_rec arms; CI asserts empty `RUSTFLAGS`; acceptance criteria grep counts `mul_add` == 0 |
| T-06-CUBECL-DRIFT | high | cubecl pre-release internal type drift between `0.10-pre.3` and `0.10-pre.4` would break N≥4 specialisations | Hard pin `=0.10.0-pre.3` (existing `xtask check-cubecl-pin`); no `cargo update` in this plan |
| T-06-MPMATH | low | Python sidecar reproducibility — non-deterministic mpmath output across versions | `mp.prec = 200` set in `__main__.py`; `mpmath>=1.4,<2.0` floor documented in `xtask/mpmath_eval/README.md`; SHA-256 drift gate via `--check` |
| T-06-N4-CANCELLATION | medium | f64-rounding cumulative-error in N=4..6 multo (≤16-term sum per coefficient) could exceed 1e-13 budget per RESEARCH §6 | Tier-1 fixture-driven test BEFORE any kernel uses N=4: golden tests at strict 1e-12 against C++; if N=6 budget breached, escalate via PLANNING INCONCLUSIVE rather than widen tolerance |
| T-06-TPSS-GUARD-DIVERGENCE | medium | tau ≥ tau_w guard makes Rust diverge from C++ in unphysical regime | mitigated by ACC-04 amendment (D-03) — mpmath truth at boundary; tau_clamp test asserts physical-regime BIT-EXACT to pre-guard baseline (no divergence in valid regime) |
| T-06-PYTHON-LEAK | low | If `xcfun-*` library crate accidentally imports Python (e.g. via `pyo3` for ad-hoc test) it violates D-04 | acceptance criteria greps `pyo3` / `python` across `crates/xcfun-*` returns empty |
</threat_model>

<verification>
- All 4 tasks GREEN per their automated commands.
- `cargo nextest run -p xcfun-ad --tests` passes (no regression in Phase 1 N=0..3 specialisations).
- `cargo nextest run -p xcfun-eval --features testing --test self_tests` passes (no regression in TPSS-correlation tier-1 self-tests).
- No new `mul_add` introduced in any modified Rust file: `grep -v '^//' crates/xcfun-ad/src/ctaylor_rec/multo.rs crates/xcfun-ad/src/ctaylor_rec/compose.rs crates/xcfun-ad/src/expand/erf.rs crates/xcfun-ad/src/math.rs crates/xcfun-eval/src/functionals/mgga/tpssc.rs crates/xcfun-eval/src/functionals/mgga/tpsslocc.rs crates/xcfun-eval/src/functionals/mgga/revtpssc.rs | grep -c 'mul_add'` == 0.
- No new Python deps in xcfun-* library crates: `grep -rl 'pyo3\|cpython' crates/xcfun-ad crates/xcfun-core crates/xcfun-eval crates/xcfun-rs crates/xcfun-capi` returns empty.
- xtask check-cubecl-pin still GREEN: `cargo run -p xtask --bin check-cubecl-pin` exits 0.
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 1 advanced: per-functional `#[cube]` body N-coverage extended (was N=0..3 unblocking; now N=0..6 unblocked). ROADMAP success criterion 2 prerequisite (1e-13 tier-3) substrate-ready.
- ACC-04 amendment (per D-03) infrastructure landed: mpmath sidecar architecture in place; fixture pipeline ready for Plan 06-N2 + Plan 06-00 boundary fixtures.
- D-19 from Phase 4 (Plan 04-05 N≥4 forward, Plan 04-08 ERF forward, Plan 04-10 TPSS Path-B forward) all have substrate fixes landed in current xcfun-eval tree (per D-09 — git-mv in Plan 06-01 will be substrate-clean).
- Plan 06-01 unblocked: pure structural reorg; no concurrent algebraic deltas to bisect.
- Phase 6 invariants preserved: no `mul_add`, no `RUSTFLAGS`, cubecl pin lockstep, library graph Python-free.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-00-SUMMARY.md` documenting:
- AD N≥4 specialisations landed (6 fns + 6 golden tests + xtask fixture extension)
- erf_precise_taylor + ctaylor_erf rewire
- TPSS tau_clamp guard (3 kernels + tpss_like helper + 1 test)
- mpmath sidecar architecture (xtask binary + 5 Python files + fixtures dir)
- Stub functions in `xtask/mpmath_eval/functionals.py` left as `NotImplementedError` for Plan 06-N2 to populate
- Pre-Plan-06-01 confirmation: tier-1 self-tests still GREEN; tier-2 LDA+GGA quick sweep still GREEN at order 2
</output>
