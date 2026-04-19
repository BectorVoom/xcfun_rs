# 07 — Accuracy strategy: guaranteeing ≤ 1e-12 parity with C++ xcfun

> **Revision history**
>
> - **2026-04-19 PM — Phase 1 cubecl pivot.** Any pre-pivot language in
>   this document alluding to a hand-Rust scalar reimplementation of `ctaylor.hpp` /
>   `tmath.hpp` is replaced by **"cubecl-cpu lowering"**; the 1e-12 parity
>   contract applies to the cubecl-cpu `CpuRuntime`-executed `#[cube] fn`
>   output, not to a separate hand-Rust scalar implementation. The FMA
>   suppression mechanism is unchanged in intent; its enforcement now runs
>   the `check-no-fma` xtask binary (`cargo run -p xtask --bin
>   check-no-fma`) against cubecl-cpu's MLIR-lowered asm output. See
>   `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`
>   for the 28 locked decisions and Plan 01-07 Task 3 for the asm-gate
>   implementation.

This is the load-bearing correctness document. The 1e-12 relative-error target defines what "correct" means for `xcfun_rs`. Every design decision in the AD and densvars subsystems is driven by this constraint, because floating-point accuracy is not a property one can bolt on retroactively.

---

## 1. Statement of the invariant

For every `(functional_set, vars, mode, order)` tuple supported by the C++ library, and for every density input `d` in the library's test vectors or in a fuzz-generated grid satisfying the preconditions, and for every element `i` of the output array:

```
|out_rust[i] - out_cpp[i]| / max(|out_cpp[i]|, 1.0) ≤ 1.0e-12
```

The denominator uses `max(|out_cpp[i]|, 1.0)` to avoid division by zero where `out_cpp[i] = 0.0` in the exact arithmetic limit — otherwise the absolute error `|out_rust[i] - out_cpp[i]|` must be ≤ 1.0e-12.

This is not bit identity. Bit identity between libm implementations across platforms is not enforceable. The 1e-12 threshold is three orders of magnitude wider than the worst-case ULP error in a chain of double-precision arithmetic operations typical of a functional (~1e-15), which gives a comfortable margin for minor transcendental differences.

---

## 2. Sources of numerical divergence (and how we pin them)

| Source | Reference behaviour | Our strategy |
|--------|---------------------|--------------|
| Operation order in `CTaylor::mul` | xcfun's `ctaylor_rec::multo` uses a specific recursion structure: `P_N = P_{N-1} + x_N · R_{N-1}` | Port that recursion verbatim, with the same accumulation order, per const-generic `N`. |
| Series-expansion coefficients | xcfun's `tmath.hpp` expansions (`inv_expand`, `exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, `erf_expand`) compute `Nvar+1` coefficients with explicit recursion over index | Port each `*_expand` function byte-for-byte into `xcfun-ad::expand::*`, preserving loop bounds and accumulation order. |
| `densvars` derived fields (zeta, r_s, n_m13, a_43, b_43) | C++ constructor computes eagerly in a fixed order | Rust constructor computes in the same order using the same formulas. |
| `regularize()` threshold (`XCFUN_TINY_DENSITY`) | Only `c[CNST]` coefficient is modified | Rust mirrors: only `c.c[CNST]` altered; derivatives pass through. |
| libm vs CUDA math | `exp`, `log`, `erf`, `pow` may differ by 1–4 ULPs | Tolerance budget: the 1e-12 parity envelope is ≥ 1000× wider than worst-case transcendental divergence over a chain of 50 operations. |
| Reassociation from compiler flags (`-ffast-math`) | xcfun builds without fast-math | We also build without fast-math: `RUSTFLAGS` contains neither `-Ctarget-cpu=native` nor any `-Zreassociate*` flags in the CI profile. |
| Horner's rule polynomials | `specmath.hpp::poly` uses descending index | `xcfun-core::functionals::shared::specmath::poly` mirrors this exactly. |
| Functional-weight accumulation in the dispatcher | `out += fun->settings[id] * fp(d)` — strict left-to-right | Rust dispatcher accumulates in the same order (same iteration over `active_ids`, same scalar multiplications). |
| `f64` associativity | Always non-associative; matters for sums of many terms | Each functional's own algebraic formula is single-chain; the only place we accumulate across functionals is the active-set loop (short, deterministic). |

---

## 3. Algorithmic identity, not just algebraic equivalence

Two numerically-accurate AD implementations of the same functional can produce outputs that differ by ~1e-12 because the rounding pattern depends on the order of operations. To stay within 1e-12 we enforce **algorithmic identity** where it matters most:

- The bit-flag index scheme is identical (0 = constant, `VARi = 1 << i`).
- The recursion structure of `CTaylor::mul` is preserved (split on the highest bit, recurse on each half, cross-multiply the halves into the upper half of the result).
- Series composition uses the same scalar coefficient table ordering.
- Elementary functions are composed with the same intermediate expansion depth (always `N` for an `CTaylor<T, N>`, never a truncation).
- Each functional body is a textual port of the C++ source, preserving parenthesisation and the order of sub-expressions unless the Rust compiler can be proved not to re-associate.

Algebraic equivalence (re-writing `(a + b) + c` as `a + (b + c)`) is **not** assumed. Where the C++ reference has intermediate variables that the compiler might re-order, we introduce the same intermediates in Rust with `let` bindings.

On the cubecl-cpu runtime (where the cubecl-cpu lowering is the scalar validation substrate — see the Revision history banner above and CONTEXT.md D-01/D-08), any re-association risk is mitigated by the repository-wide `.cargo/config.toml` pinning `-Cllvm-args=-fp-contract=off` under both `[build]` and `[target.'cfg(all())']`, and validated actively by `cargo run -p xtask --bin check-no-fma` (Phase 1 Plan 07 Task 3), which emits asm for `xcfun-ad --release` and greps `ctaylor_mul*` symbols for FMA mnemonics. Any match causes the CI gate to exit with status 2 (D-03 escalation).

---

## 4. Test architecture: how the invariant is verified

Four independent test tiers, each catching a different failure mode.

### 4.1 Tier 1 — self-tests (per functional)

Every entry in `FUNCTIONAL_DESCRIPTORS` has a recorded `test_in[]` and `test_out[]`. The test runner evaluates the functional against its recorded input and compares element-by-element with tolerance `test_threshold` (typically 1e-11 in the reference). This is the cheapest check and runs on `cargo test`.

Source of truth: `xcfun-master/src/functionals/*.cpp` FUNCTIONAL macro.

### 4.2 Tier 2 — parity harness (per element)

The `validation` binary:
1. Builds `xcfun-master` as a static library via `cc`.
2. For each of the 78 functionals (plus each of the 50 aliases), iterates over a sampled subset of `(vars, mode, order)` tuples valid for that functional.
3. Generates 10 000 random density points per tuple using a fixed seed (deterministic reproducibility).
4. Evaluates both implementations; compares with `max |rust[i] - cpp[i]| / max(|cpp[i]|, 1) ≤ 1e-12`.
5. Emits `validation/report.html` and `validation/report.jsonl`.

Any element failing the tolerance blocks merge. The grid generator stresses the ρ → 0 regularization path, the zeta → ±1 fully-polarised limit, and the high-gradient `chi²` → ∞ limit.

### 4.3 Tier 3 — cross-backend parity

Holds `Backend::Cpu` as reference. For `Cuda` and (f64-capable) `Wgpu`, runs the same 10 000-point grid and asserts rel-error ≤ 1e-13 vs. the CPU output. This is looser than the C++-parity bound because the test is checking that the cubecl code path inside our repo is consistent; the C++-parity bound is checked separately on CPU.

### 4.4 Tier 4 — property tests

`proptest`-driven:

| Property | Description |
|----------|-------------|
| Ring axioms on `CTaylor` | `(a + b) + c == a + (b + c)` to within 2 × ULP |
| Derivative of product | coefficient at `VAR0` of `a * b` equals `a[CNST] * b[VAR0] + a[VAR0] * b[CNST]` |
| Inverse invariance | `x * x.reciprocal() ≈ 1` on non-zero `x` |
| Exp/log round-trip | `log(exp(x)) ≈ x` |
| Weight linearity | For `fun1` + `fun2`, output equals `w1 * fun1_out + w2 * fun2_out` computed separately, to within 10 × ULP |

---

## 5. Test fixtures derived from C++ xcfun

Under `validation/fixtures/`, we store gzip-compressed JSONL files with records of the form:

```json
{
  "functional": "b3lyp",
  "vars": "XC_A_B_GAA_GAB_GBB",
  "mode": "XC_PARTIAL_DERIVATIVES",
  "order": 2,
  "input": [0.39, 0.38, 0.1, 0.05, 0.1],
  "output": [-0.412..., ...],
  "xcfun_version": "2.1.1",
  "seed": 0x1234abcd
}
```

Generation: the `xtask regen-fixtures` subcommand rebuilds `xcfun-master`, runs the C++ library on the fixed density grid, and writes the records. This produces ~40 MB of reference data, committed to the repository. The Rust tests load the fixtures and verify per-element parity.

Regenerating fixtures requires a C++ toolchain; running the tests does not.

---

## 6. Tolerance budget breakdown (typical GGA functional, order 2)

| Operation chain | Count | Per-op ULP | Cumulative rel. err |
|-----------------|-------|-----------|--------------------|
| `DensVars::build` arithmetic | ~20 | 0.5 | ≈ 2e-15 |
| `CTaylor` additions / multiplications in the functional body | ~200 | 0.5 | ≈ 2e-14 |
| `CTaylor` compositions (`pow`, `log`, `exp`) | ~5 | 2 | ≈ 2e-14 |
| `erf` (range-separated only) | 0–2 | 3 | ≈ 1e-13 on CPU libm, larger on some GPU back-ends |
| Accumulation over active functionals | ~4 | 0.5 | ≈ 2e-15 |

Total estimated worst case on CPU: ~1e-13, safely under 1e-12. With the cross-backend tolerance at 1e-13, the combined CPU+GPU envelope is well within 1e-12.

The `erf` case on Wgpu exceeds the budget; accordingly the GPU dispatcher falls back to CPU for range-separated functionals when the selected backend is Wgpu (see [06-cubecl-strategy.md](06-cubecl-strategy.md) §11).

---

## 7. Failure-handling protocol

When the validation harness reports a failing element:

1. Examine the failing `(functional, vars, mode, order, input_index, output_index)` tuple.
2. Re-run with `RUST_LOG=xcfun=trace` to emit the intermediate CTaylor coefficients. The tracing subscriber lives in `validation/` only; production code has no tracing on the hot path.
3. Regenerate the same intermediate in C++ by compiling a minimal test driver against `xcfun-master`. Store the intermediate `ctaylor<double, N>::c[0..]` values.
4. Diff Rust's intermediates against C++'s. The first coefficient that diverges identifies the offending operation.
5. Fix by aligning the Rust op order to the C++ order (most common root cause: accidental re-parenthesisation).

This protocol is rehearsed in `docs/design/12-design-decisions.md` rationale and in the validation harness's `--diagnose` mode.

---

## 8. Non-tolerated behaviours

| Behaviour | Action |
|-----------|--------|
| Using `f32` anywhere on the numerical path | Compile error: `Num` is not implemented for `f32` in this crate |
| Fast-math compiler flags | CI asserts `RUSTFLAGS` contains no `-Cfast-math*` |
| Re-associating sums for performance | Rejected in review; `#[inline(always)]` is used sparingly to let the compiler inline cleanly without reassociation |
| Skipping the `regularize` step | Compile-time reachability: `DensVars::build` calls `regularize` unconditionally |
| Using a different elementary-function approximation (e.g. Chebyshev) | Rejected: adds a new source of ULP error without benefit |
| Adopting a third-party AD library | Rejected: structural identity with xcfun's AD algebra is required (see 12-design-decisions.md) |

---

## 9. Ongoing assurance

- Every pull request runs the full validation harness on CPU; a failure blocks merge.
- Nightly CI runs Tier 3 on a CUDA runner and a Wgpu runner.
- Release tags include a `validation-report.html` artefact.
- The registry codegen step fails if any reference test-vector element is missing.
