# Pitfalls Research

**Domain:** Numerical C++ → Rust port (DFT exchange-correlation library) with unified CPU/GPU kernels via `cubecl` and 1e-12 reference parity
**Researched:** 2026-04-18
**Confidence:** HIGH (derived from direct reading of `xcfun-master/` sources and design docs `06`, `07`, `12`; cubecl specifics MEDIUM pending empirical confirmation on Wgpu erf variance)

---

## Critical Pitfalls

### Pitfall 1: Silent floating-point reassociation by the Rust/LLVM backend

**What goes wrong:**
A chain like `gnn + 2.0 * gab + gbb` that xcfun writes left-to-right in C++ is folded by LLVM into a different evaluation order (e.g. `fma(2.0, gab, gnn + gbb)` or a Horner contraction) because Rust's `+` on `f64` is `frelaxed`-compatible under optimization even without `-Cfast-math`. The resulting rounding pattern drifts from C++ by 1–3 ULP. Individually tolerable; accumulated across a 200-op GGA body it breaks parity.

**Why it happens:**
- Rustaceans assume `f64` arithmetic is already sequenced; it is, at the AST level, but `rustc` with `-Copt-level=3` enables `-Cllvm-args=-enable-unsafe-fp-math=false` only by default, and certain intrinsic calls (`f64::mul_add`, autovectorized SIMD) insert FMAs LLVM would not otherwise emit.
- The C++ reference compiles without `-ffast-math`; the Rust port assumes the same hygiene transfers automatically. It does not: e.g. `a.mul_add(b, c)` and `a*b + c` compile to different instruction sequences with different rounding.
- Porting code via "clean up the expression" formatting (removing redundant parentheses, merging nested temporaries) silently re-associates.

**How to avoid:**
- **Build-flag hygiene:** CI asserts `RUSTFLAGS` contains none of `-Cfast-math`, `-Cllvm-args=-enable-unsafe-fp-math`, `-Cllvm-args=-fp-contract=fast`. Add `-Cllvm-args=-fp-contract=off` to the CI profile for functional crates to suppress auto-FMA contraction.
- **Algorithmic identity rule** (per `docs/design/07-accuracy-strategy.md` §3): every functional body is a textual port; identical parenthesisation, identical intermediate `let` bindings, never consolidate multi-line expressions.
- **Never call `f64::mul_add` unless the C++ reference does.** Grep-gate the codebase with a lint: `rg 'mul_add' xcfun-core/src/functionals/` must return zero lines.
- **Disassembly spot-check:** for at least one representative per functional class (LDA, GGA, metaGGA, range-separated, hybrid), diff `cargo asm` output against the C++ reference to confirm op count and order.
- **Per-expression `#[inline(never)]` on helpers when necessary** to prevent cross-function reassociation, but use sparingly — prefer structural discipline.

**Warning signs:**
- Tier-2 parity harness reports an element at 5e-13 to 5e-12 relative error (just outside the budget) across *many* `(vars, mode, order)` tuples — classic reassociation signature.
- The offending element changes when the surrounding code is edited in unrelated ways (because LLVM's fusion decisions depend on register pressure).
- `cargo bench` improves ~5 % after a refactor and parity regresses simultaneously.

**Phase to address:**
Phase 1 (AD engine) sets the discipline. Phase 2 (LDA functionals) is where the lint/CI gate must be enforced — LDA bodies are short enough that any drift immediately implicates the compiler, not the math.

---

### Pitfall 2: libm vs CUDA libdevice vs WGSL transcendental drift

**What goes wrong:**
`f64::exp(x)` dispatches to glibc libm on Linux CI, `msvcrt` on Windows, Apple libm on macOS, nvdevice's `__nv_exp` on CUDA, and a WGSL-lowered polynomial on Wgpu. These differ in the last 1–4 ULP for `exp`/`log`/`pow`, and by **millions of ULP** for `erf` on Wgpu (≈ 1.5e-7 absolute across devices per the cubecl manual). A parity comparison that passes on Linux x86_64 fails on the macOS M-series CI runner, or on Wgpu, without any Rust code change.

**Why it happens:**
- Every runtime selects its own transcendental implementation. Pinning `cubecl` does not pin the underlying libm on CPU or the `libdevice.bc` on CUDA.
- Range-separated functionals (`ldaerfx`, `ldaerfc`, `beckecamx`, `beckesrx`, `ldaerfc_jt`, `camb3lyp`, `camcompx`) apply `erf` to intermediates that may approach zero or large arguments, where WGSL's polynomial approximation is worst.
- `pow(x, y)` is typically implemented as `exp(y*log(x))` — it accumulates errors from two transcendentals. Using `powi(x, n)` where the C++ uses `pow(x, int_n)` saves one transcendental's drift.

**How to avoid:**
- **Backend-qualified tolerance table:** CPU-reference ≤ 1e-12 (primary contract); CUDA vs CPU ≤ 1e-13 (cross-backend); Wgpu vs CPU ≤ 1e-9 and `erf`-using functionals **force Backend::Cpu** by inspecting `depends`.
- **Use `powi` aggressively.** Any `pow(x, k)` with integer `k` in xcfun sources must port to `powi(x, k as i32)` in Rust — this both matches the C++ overload selection and avoids the `exp(k*log(x))` drift.
- **No Chebyshev/Remez re-implementations** of libm functions (per `docs/design/07-accuracy-strategy.md` §8). Adding a "more accurate" `erf` is a regression: it creates a new drift source against C++ libm.
- **Document the runner matrix** in `validation/README.md`: Linux x86_64 glibc is the canonical CPU; macOS and Windows CI use the same 1e-12 bar but have a known-issues list if Apple/MSVC libm diverges.
- **libm pinning:** for the reference CPU CI job, lock the glibc version via the container image (e.g. `ubuntu:22.04` exact tag); an OS upgrade is a validation-regenerating event.

**Warning signs:**
- Range-separated functional test passes on CUDA, fails at 1e-8 on Wgpu → expected, route to CPU.
- Same functional fails at 3e-12 on macOS but passes on Linux → Apple libm drift, escalate but don't panic.
- Failures cluster around specific input values (x near 0, x near 1) → transcendental argument reduction boundary.

**Phase to address:**
Phase 3 (GGA + range-separated functionals) and the GPU-backend phase. The dispatcher in `xcfun-gpu` gains an `erf`-aware fallback guard before any Wgpu code path lands.

---

### Pitfall 3: CTaylor coefficient ordering (bit-flag layout) corrupted on GPU

**What goes wrong:**
`ctaylor<T, N>` stores `2^N` coefficients indexed by variable bit-masks: index 0 = constant, index 1 = `∂/∂x₀`, index 2 = `∂/∂x₁`, index 3 = `∂²/∂x₀∂x₁`, index 4 = `∂/∂x₂`, and so on — see `ctaylor.hpp:22-30`. The **recursive multiply** in `ctaylor_rec::mul`/`multo` (`ctaylor.hpp:41-65`) splits on the highest bit and recurses. A natural-looking Rust rewrite using an index-based triple loop over `(i, j, k)` produces a different summation order for the "cross-multiplication" terms `x_low * y_high` vs `x_high * y_low`, breaking parity with C++ by 1–2 ULP per multiply — unsurvivable after dozens of multiplications in a metaGGA body.

**Why it happens:**
- The recursion structure **encodes the summation order**. C++ does `mul(dst, x_lo, y_lo); mul(dst+h, x_hi, y_lo); mul(dst+h, x_lo, y_hi)` — specifically `dst[h+k] += x[h+k]*y[0] + ... + x[0]*y[h+k] + ...` with a deterministic accumulation order driven by the recursive descent.
- A flat Rust loop that iterates indices in numeric order produces the sum in a different permutation. `a+b+c` vs `a+c+b` on `f64` differs by ≥ ½ ULP.
- On GPU, CubeCL may re-vectorize the inner loops via `Line<F>` or unroll attributes that change fusion patterns.

**How to avoid:**
- **Port the recursion verbatim** using const-generic recursion: a `CTaylorRec<T, N>` trait specialized for `N = 0, 1, 2` (explicit) and `N > 2` (split on high bit, recurse to `N - 1`). Mirror `ctaylor_rec::mul`, `mul_set`, `multo`, `multo_skipconst` body-for-body.
- **Unit-test the recursion against golden coefficient traces.** For a canonical input `CTaylor<f64, 3>(2.0) + x0 + x1 + x2`, record the 8-element coefficient vector after each of `+`, `*`, `mul_set`, `multo`, and compare byte-for-byte (`f64::to_bits`) with a C++ test driver. This is a higher bar than 1e-12; if bits match here, the rest of the stack inherits algorithmic identity.
- **In the `#[cube]` device form, do not add `#[unroll]` on recursive multiply loops.** Let `cubecl` choose; instrument with `validation/device-bit-equiv` tests that assert `f64::to_bits` parity of the 2^N array between CPU and CUDA for simple inputs (this is achievable because IEEE-754 is defined for `+`/`*` even if libm isn't).
- **Reject CTaylor representation refactors.** Do not substitute `[T; 1 << N]` with `Vec<T>` or a "more Rust-idiomatic" nested-struct-by-order design. Coefficient layout is the API between host and device.

**Warning signs:**
- Parity fails at order ≥ 2 but passes at order 0–1.
- Failure is localized to a specific coefficient index (e.g. index 3 = `∂²/∂x₀∂x₁`) across multiple functionals — implicates the multiply routine, not the functional body.
- Failure appears only after enabling cross-backend tier-3 test (CPU vs CUDA), not in the single-backend tier-2 — implicates GPU-side unroll.

**Phase to address:**
Phase 1 (AD engine). The bit-equivalence test against C++ intermediates is a non-negotiable Phase-1 exit criterion.

---

### Pitfall 4: `regularize()` applied to the wrong CTaylor coefficient

**What goes wrong:**
The C++ `regularize(ctaylor<T,N> & x)` function (`densvars.hpp:22-25`) modifies **only** `x.c[CNST]` (the constant coefficient), leaving all derivatives untouched: `if (x < XCFUN_TINY_DENSITY) x.set(0, XCFUN_TINY_DENSITY)`. A naive Rust port that does `if x.value() < 1e-14 { x = CTaylor::from(1e-14) }` zeros all higher-order derivatives, which means `∂f/∂ρ` evaluated at `ρ = 0` becomes zero on the Rust side but non-zero on the C++ side. Silent: energies at `ρ > 0` pass; potentials fail at grid edges.

**Why it happens:**
- The comparison operator `x < 1e-14` hides the fact that only the constant term is being replaced; reading it as "if the density is tiny, replace the density" misses the AD structure.
- Newcomers writing Rust API-first (`DensVars` constructor returns a clean struct) replace the whole CTaylor, not its constant coefficient.
- The `regularize` name is misleading — "regularize" sounds like "zero-clamp the density"; the actual semantic is "floor the value; preserve derivative information".

**How to avoid:**
- **Mirror the C++ method on `CTaylor`:** `impl<T: Num, const N: usize> CTaylor<T, N> { pub fn regularize(&mut self, floor: T) { if self.c[CNST] < floor { self.c[CNST] = floor; } } }`. Do **not** add a `from_scalar_regularized` constructor.
- **Phase-1 unit test:** `let mut x = CTaylor::<f64, 2>::new(1e-16); x.set(VAR0, 1.0); x.set(VAR1, 2.0); x.regularize(1e-14); assert_eq!(x.c[CNST], 1e-14); assert_eq!(x.c[VAR0], 1.0); assert_eq!(x.c[VAR1], 2.0);`.
- **Test the regularization boundary in tier-2:** the fixture grid must include points with `ρ ∈ [0, 1e-14]` and explicitly check derivative-coefficient parity, not just energy parity.
- **API discipline:** `DensVars::build` calls `self.n.regularize()`, `self.a.regularize()`, etc. — each an in-place method call, not a re-assignment.

**Warning signs:**
- Potentials (mode `Potential`) fail parity while energies pass.
- Failures concentrate at small-density grid points (`n < 1e-10`).
- `order ≥ 1` fails; `order = 0` passes.

**Phase to address:**
Phase 2 (`DensVars` port). The unit test above is a blocker for Phase-2 completion.

---

### Pitfall 5: `densvars` switch fallthrough lost in Rust `match`

**What goes wrong:**
The C++ `densvars` constructor (`densvars.hpp:40-212`) uses **intentional C-style fallthrough** in its `switch(vars)`:

```cpp
case XC_A_B_GAA_GAB_GBB_TAUA_TAUB:
  taua = d[5];
  taub = d[6];
  tau = taua + taub;
  // FALLS THROUGH
case XC_A_B_GAA_GAB_GBB:
  gaa = d[2];
  // ...
  // FALLS THROUGH
case XC_A_B:
  a = d[0];
  // ...
  break;
```

A Rust `match` does **not** fall through. A literal port produces incomplete state (e.g. `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` sets only `taua`, `taub`, `tau` and leaves `gaa`, `gab`, `gbb`, `a`, `b`, `n`, `s` at zero). Catastrophic silent error — every meta-GGA with tau returns garbage.

**Why it happens:**
- The fallthrough is idiomatic in xcfun's densvars but uncommon elsewhere; reviewers miss it.
- A direct transliteration of `case … : … case … : …` to `match … { X => … Y => … }` is syntactically clean and compiles without warning.
- The fallthrough is not marked `[[fallthrough]]` (the file predates C++17's attribute), so `clang-tidy` doesn't flag it.

**How to avoid:**
- **Flatten fallthrough arms** into a hierarchy of helper functions. Example:
  ```rust
  fn fill_ab(d: &[f64], dv: &mut DensVars) { dv.a = d[0]; dv.a.regularize(); dv.b = d[1]; dv.b.regularize(); dv.n = dv.a + dv.b; dv.s = dv.a - dv.b; }
  fn fill_ab_gaa_gab_gbb(d: &[f64], dv: &mut DensVars) { dv.gaa = d[2]; dv.gab = d[3]; dv.gbb = d[4]; dv.gnn = dv.gaa + 2.0*dv.gab + dv.gbb; dv.gss = dv.gaa - 2.0*dv.gab + dv.gbb; dv.gns = dv.gaa - dv.gbb; fill_ab(d, dv); }
  fn fill_ab_gaa_gab_gbb_taua_taub(d: &[f64], dv: &mut DensVars) { dv.taua = d[5]; dv.taub = d[6]; dv.tau = dv.taua + dv.taub; fill_ab_gaa_gab_gbb(d, dv); }
  ```
  Each arm calls into the next; match arms become one-liners.
- **Parity trace test per Vars variant:** for each of the 31 `Vars` values, populate a `DensVars` in Rust, call the C++ driver on the same input, compare **every field** (not just `n`, `s`) for bit equality on arithmetic-only variants and 1-ULP equality on pow/regularize-touched fields.
- **Codegen the variant-specific builders** from a declarative table in `xtask regen-registry` so that the fallthrough graph is visible as a list, not as implicit C-switch semantics.

**Warning signs:**
- Meta-GGA functionals (scan, r2scan, m06L, tpss, revtpss) fail parity on `tau`-containing variants but pass on GGA-only variants.
- `DensVars` struct inspection shows `gaa == 0.0` on meta-GGA inputs where it should be populated.
- Specific variant indices (the "superset" variants) fail; their "subset" variants pass.

**Phase to address:**
Phase 2 (`DensVars` port). Codegen the builders; enforce field-by-field parity test in Phase-2 CI.

---

### Pitfall 6: Alias weight propagation — multiplicative, not additive

**What goes wrong:**
`xcfun_set(fun, "camb3lyp", 1.0)` recurses into each alias term with `value * term.weight`: see `XCFunctional.cpp:389-402`. The **final weight** applied to a functional is the product of the user-supplied value and the alias weight. A Rust port that accumulates alias terms as `settings[id] = weight` (replacing) or `settings[id] += weight` (ignoring `value`) breaks every hybrid. Subtler still: `xcfun_set(fun, "camb3lyp", 0.5)` must halve every camb3lyp term weight — forgetting to multiply by `value` gives camb3lyp-at-1.0 instead of scaled.

Even subtler: aliases may overlap. Calling `xcfun_set(fun, "b3lyp", 1.0)` then `xcfun_set(fun, "slaterx", 0.2)` **adds** to the existing `slaterx` weight (`fun->settings[item] += value` — note the `+=` on line 373). This is intentional: it lets users overlay weights. A Rust port using a map-overwrite semantic breaks the additive contract.

**Why it happens:**
- Aliases look like macros ("lda = slaterx + vwn5c"), tempting a substitution model that ignores the composition's linearity.
- The `+=` on functional weights vs `=` on parameter weights (`settings[item] = value` at line 386 for parameters) is a two-line distinction that is easy to miss.
- Aliases like `camcompx` have **negative** weights (`{"beckecamx", -1.0}`), so a test with positive-only weights won't catch a sign error.

**How to avoid:**
- **Port the reference semantic byte-for-byte:** `fn set_functional(id, value) { settings[id] += value; activate(id); }`, `fn set_parameter(id, value) { settings[id] = value; }`, `fn set_alias(id, value) { for term in alias[id].terms { set(fun, term.name, value * term.weight); } }`. The recursion resolves alias-of-alias; the recursion depth is bounded because the alias table does not contain cycles (verify at codegen time with a topological sort).
- **Alias-resolution codegen test:** for each of the 50+ aliases, set the alias with `value = 1.0` and then with `value = 0.37`; compare resolved `settings` array against the C++ reference's post-set state.
- **Negative-weight canary:** `camcompx` has `{"beckecamx", -1.0}` — a specific test asserts `settings[beckecamx_id] == -0.37` after `xcfun_set(fun, "camcompx", 0.37)`.
- **Parameter vs functional lookup order:** lookup must try functional first, then parameter, then alias (mirror `XCFunctional.cpp:370-403`). A reordered lookup breaks aliases that share a name with a functional.

**Warning signs:**
- Hybrid functionals (`b3lyp`, `pbe0`, `camb3lyp`) fail parity; their component functionals pass individually.
- Failures scale with the user-supplied weight: 10 % weight gives 10 % error.
- `camcompx` fails with a sign flip (both beckex and beckecamx active but one has wrong sign).

**Phase to address:**
Phase 4 or 5 (alias table & `Functional::set` API). Alias-resolution codegen lives in `xtask regen-registry`; add the additive-weight test to the Phase-4 exit criteria.

---

### Pitfall 7: Wgpu backend quietly running in f32 despite `F: Float`

**What goes wrong:**
`cubecl` permits `#[cube] fn k<F: Float>(...)` to be instantiated with `F = f32` or `F = f64`. If a runtime consumer forgets to constrain the launch site to `f64`, or picks a Wgpu adapter without the `SHADER_F64` feature, the kernel silently instantiates with `f32`. The parity harness reports ~1e-7 relative error — three ULP in f32 — which swamps the 1e-12 budget by five orders of magnitude. The error message is often misleading ("transcendental drift") instead of the real cause ("wrong precision").

**Why it happens:**
- `cubecl`'s `Float` trait is generic; `f32` is a valid instantiation.
- `auto_backend` (design doc 06 §2) claims to "refuse" Wgpu without f64, but the refusal has to be programmed — if the gate is missing, the default instantiation path leaks through.
- Wgpu adapters (e.g. llvmpipe in CI, WebGPU on macOS non-Apple-Silicon) frequently lack `SHADER_F64`.

**How to avoid:**
- **Type-level pin:** the top-level kernel launcher takes `&ComputeClient<R, _>` and concrete scalar `f64`, not generic `F`. The generic-over-`F` lives only inside `#[cube]` bodies for code reuse.
- **Runtime gate:** on Wgpu, before instantiation, query the adapter for `Features::SHADER_F64`. If absent, return `Err(XcError::WgpuNoF64)`. Tested in CI with a software Wgpu adapter that lacks f64 → expect the error, not silent f32.
- **Compile-time assertion:** `static_assert::<{ std::mem::size_of::<Scalar>() == 8 }>` on the scalar type alias at the `xcfun-kernels` crate root.
- **Validation tolerance tripwire:** if a parity run reports > 1e-6 relative error on *any* functional, fail fast with a "suspected f32 instantiation" diagnostic — do not flood the report.

**Warning signs:**
- Every functional fails at ~1e-7 on Wgpu; the error is suspiciously uniform.
- `cudaGetDeviceProperties` / wgpu adapter debug shows no `SHADER_F64`.
- Kernel binary size is ~half the expected size (half the scalar width).

**Phase to address:**
Phase 6 or 7 (GPU backend). The runtime-gate test and the compile-time assertion are both Phase-6 exit criteria.

---

### Pitfall 8: `cubecl =0.10.0-pre.3` API drift between pre-releases

**What goes wrong:**
`cubecl` at `=0.10.0-pre.3` is a pre-release; `0.10.0-pre.4` or `0.10.0` may rename `#[cube(launch_unchecked)]` to `#[cube(launch)]`, change the `Array<F>` parameter semantics, or move `ABSOLUTE_POS` to a different module. A naive `cargo update` silently pulls the newer pre-release (because `=0.10.0-pre.3` is a soft-equals in practice only if `Cargo.lock` is checked in and respected), breaking compilation or changing kernel semantics without a git diff to review.

**Why it happens:**
- Pre-release semver has no stability guarantee.
- `cargo update` and `cargo update -p cubecl` can bypass the `=` pin if a dependency re-exports cubecl types transitively.
- CI re-running `cargo update` "to check for advisories" inadvertently picks up new pre-releases.

**How to avoid:**
- **Lock + verify:** commit `Cargo.lock`; CI asserts the exact lockfile line `cubecl = "0.10.0-pre.3"` survives `cargo metadata`.
- **Isolate cubecl imports:** only `xcfun-kernels` and `xcfun-gpu` import from `cubecl`. Other crates depend on these via the project's own abstraction types. A cubecl API change triggers at most two crates' rewrites.
- **Bump-triggers-validation policy:** the CI YAML has a rule: any PR that modifies `Cargo.lock`'s `cubecl` line triggers the full tier-2 + tier-3 harness on CUDA, Wgpu, and CPU. Non-bump PRs skip the GPU tiers.
- **Cargo-deny rule** that disallows transitive cubecl version mismatches (`multiple-versions = deny` scoped to `cubecl-*` crates).

**Warning signs:**
- CI turns red on a PR that didn't touch kernel code.
- `cargo build` passes locally on a developer's machine but fails on a freshly cloned CI container.
- Runtime panic from cubecl that mentions a missing feature flag or an unrecognized attribute.

**Phase to address:**
Foundations phase (cargo workspace setup). The CI gate and cargo-deny rule land in the first CI commit, before any kernel code exists.

---

### Pitfall 9: Series-expansion coefficient layout miscopied from `tmath.hpp`

**What goes wrong:**
`tmath.hpp` defines `inv_expand`, `exp_expand`, `log_expand`, `pow_expand`, `sqrt_expand`, `cbrt_expand`, `atan_expand`, `gauss_expand`, `erf_expand`, `asinh_expand`, `asin_expand`, `acos_expand` — each produces `N+1` Taylor coefficients for the given analytic function at expansion point `x₀`. The recursion pattern differs per function:

- `pow_expand`: `t[i] = t[i-1] * x0inv * (a - i + 1) / i`
- `log_expand`: `t[i] = (xn / i) * (2*(i & 1) - 1); xn *= x0inv;` (alternating sign)
- `sqrt_expand`: `t[i] = t[i-1] * ((3*x0inv)/(2*i) - x0inv)`
- `cbrt_expand`: `t[i] = t[i-1] * ((4*x0inv)/(3*i) - x0inv)`

A Rust port that uses the closed-form binomial coefficient (`t[i] = choose(a, i) * x0^(a-i)`) is **mathematically equivalent** but uses a different number of multiplications and a different accumulation order — 1–2 ULP drift per coefficient, compounded over composition with `ctaylor_rec::compose`. Result: `log`, `pow`, `sqrt` parity fails at ~1e-12, just barely.

**Why it happens:**
- The recursion forms are optimized for numerical stability, not readability. Rewriting "for clarity" silently changes rounding.
- `pow_expand` uses `x0inv = 1/x0` and multiplies, saving one division per iteration. A port that divides by `i * x0 / (a - i + 1)` is algebraically equivalent but does the division last, changing rounding.

**How to avoid:**
- **Port each `*_expand` function body-for-body** into `xcfun-ad::expand::*`, preserving loop bounds, intermediate variables, and operator precedence. Add a line-level diff comment linking to `tmath.hpp:L-L`.
- **Golden-coefficient test:** for each `*_expand`, generate a C++ test driver that dumps the 7-element coefficient array for inputs in `{0.1, 1.0, 10.0}` at `N ∈ {0, 1, 2, 3, 4, 5, 6}`, and diff bit-for-bit against the Rust version. Bit-for-bit is achievable here because the only non-deterministic op is `pow(x0, a)` / `exp(x0)` / `log(x0)` / `sqrt(x0)` at a single point — the rest is pure +/*//.
- **Reject "vectorized" rewrites.** `x0inv` is a loop-invariant that the compiler will hoist anyway; don't manually pre-compute and rearrange.

**Warning signs:**
- Functionals using `pow(ρ, 4/3)` (exchange) fail parity; functionals using `pow(ρ, integer)` pass.
- `log_expand` alternating-sign coefficient at odd-index positions drifts while even-index passes.
- Parity margin is exactly at 1e-12 (just barely failing), indicating a per-coefficient ULP-level drift rather than a structural bug.

**Phase to address:**
Phase 1 (AD engine) — the `*_expand` functions live in `xcfun-ad::expand`. Golden-coefficient test is Phase-1 exit criterion.

---

### Pitfall 10: `pow_expand` / `sqrt_expand` / `log_expand` called without regularize

**What goes wrong:**
`pow_expand(t, x0, a)` asserts `x0 > 0`; `log_expand` asserts the same; `sqrt_expand` asserts `x0 > 0` (`tmath.hpp:156, 144, 165`). In C++, if the regularize step is skipped for some new `Vars` variant, the assert fires with `x0 <= 0` and the process aborts. In a Rust port, `debug_assert!(x0 > 0.0)` compiles out in release — the result is a NaN silently propagating through the AD chain, producing a meaningless number that the parity harness dutifully compares against the C++ assert-abort. The harness reports "C++ crashed" and moves on.

**Why it happens:**
- Rust's `debug_assert!` is not a safety check.
- A new `Vars` variant added in the Rust port (even as a defensive "catch-all") might reach `pow(n, -1/3)` before `regularize(n)` has fired.
- The `densvars` constructor in `densvars.hpp:214-217` calls `pow(n, -1/3)` etc. **after** all the switch arms; if a switch arm fails to set `n` (see Pitfall 5), `n = 0` reaches `pow`.

**How to avoid:**
- **Use `assert!` (not `debug_assert!`) on preconditions** for `*_expand` functions, at least in the library crate's default build; or return `Result<..., XcError::NonAnalyticInput>`.
- **Enforce DensVars invariant by construction:** `DensVars::build` returns `Result<Self, XcError>` and checks that `a, b, n > 0` after regularization. This makes the "forgot to regularize" bug a compile-time / test-time failure, not a runtime NaN.
- **NaN canary in the parity harness:** if `out_rust[i].is_nan()`, emit a distinct failure category ("NonFiniteOutput") and treat as blocker, separate from tolerance failures.
- **Fuzz the densvars grid** with `ρ ∈ [1e-30, 1e-10]` explicitly — the regularize boundary.

**Warning signs:**
- Some outputs are `NaN`; parity harness reports NaN but C++ reports a finite value (or C++ aborts with "not real analytic").
- NaN cluster around specific `Vars` variants introduced most recently.
- Release build fails parity; debug build fires an assert (opposite direction is the usual one, but in this case the Rust-specific asymmetry makes this the tell).

**Phase to address:**
Phase 1 (AD engine, `*_expand` functions) and Phase 2 (`DensVars` port). Fuzz seed covering `ρ ∈ [1e-30, 1e-10]` is in Phase-2 exit criteria.

---

### Pitfall 11: `poly` Horner evaluation ported in ascending order

**What goes wrong:**
`specmath::poly` (`specmath.hpp:24-33`) uses **descending Horner**: start from the highest-degree coefficient, multiply by `x`, add the next coefficient, repeat. A Rust port written in ascending order (start from constant, accumulate powers of `x`) is algebraically equivalent for exact arithmetic but produces a different rounding pattern: specifically, descending Horner has optimal forward error for large `|x|`, while ascending accumulation has optimal forward error for small `|x|`. For polynomials used in VWN5 correlation, PW92, and M06 exchange, both regimes appear — xcfun's choice of descending-Horner is the rounding behavior that the reference test vectors encode.

**Why it happens:**
- Both orders are taught as "Horner's rule"; programmers pick whichever feels natural.
- Rust iterator idioms (`coeffs.iter().fold(0.0, |acc, c| acc * x + c)`) default to ascending (from the first element) — opposite of C++'s `coeffs[--ndeg]`-first.

**How to avoid:**
- **Port the Horner direction verbatim:** `pub fn poly<T: Num, const N: usize>(x: T, coeffs: &[f64; N]) -> T { let mut res: T = coeffs[N - 1].into(); for i in (0..N-1).rev() { res = res * x + coeffs[i]; } res }` — iterate descending.
- **Unit test against a known polynomial:** `poly(2.0, &[1.0, 2.0, 3.0])` (representing `1 + 2x + 3x²`) returns `1 + 4 + 12 = 17.0` — trivial for integer-valued inputs, but also verify at `x = 1e-6` and `x = 1e6` that both Rust and C++ match bit-for-bit.
- **Grep-gate:** `rg 'fold\(' xcfun-core/src/functionals/` should return zero lines in any polynomial-evaluation context (a `fold`-based Horner is the common bug form).

**Warning signs:**
- VWN5 correlation, PW92, M06 fail parity; simpler functionals (Slater, Becke88 exchange) pass.
- Failure magnitude scales with the polynomial degree used in the specific functional.
- Failure worse at extreme `r_s` (very small or very large `ρ`).

**Phase to address:**
Phase 2 (`specmath` port, shared helpers). `poly` is a one-page helper; unit test lands in Phase 2.

---

### Pitfall 12: PW92C constants — `XCFUN_REF_PW92C` toggle

**What goes wrong:**
`config.hpp:35-36` documents: "Use `#define XCFUN_REF_PW92C` to use inaccurate constants in PW92C. This matches the reference implementation." Similarly `XCFUN_REF_PBEX_MU`. These toggles exist because the xcfun maintainers ship two constant tables: the "correct" PW92 constants from the original paper and the "reference" (slightly inaccurate) ones that match an earlier xcfun release. **The C++ test vectors were generated with the reference constants** in at least some historical versions. If the Rust port uses the "correct" constants while the C++ vendored copy also uses the correct ones, parity passes — but a user comparing against an older xcfun binary in the wild sees mismatches.

**Why it happens:**
- The toggles are `#define` macros in `config.hpp`, easy to miss during a Rust port.
- "Fix the constants to be correct" is tempting; a reviewer may approve it without realizing the C++ source-of-truth (via `cc` link) may have the other flag.
- The commented-out `#define XCFUN_REF_PBEX_MU` is a trap: the default is "accurate", but upstream forks may have flipped it.

**How to avoid:**
- **Read `config.hpp` in Phase 0**; port both constant tables; select via Cargo feature `pw92c-legacy-constants` (default: off, matching the default C++ define state).
- **Pin the C++ build flags used for fixture generation** in `xtask regen-fixtures`: run `cmake` with the exact defines documented, commit the cmake config to the fixture-generation pipeline.
- **Document in `xcfun_version()` return string** which constants were compiled in.

**Warning signs:**
- PW92C functional fails parity at ~1e-6 to 1e-4 — way outside 1e-12, signaling a coefficient change rather than a rounding drift.
- PBE exchange fails at ~1e-6 with a similar magnitude.
- Failure is uniform across all inputs (not input-dependent).

**Phase to address:**
Phase 0 (scoping) and Phase 2 (LDA/GGA constants). Reading `config.hpp` is a Phase-0 task; the toggle design is a Phase-2 decision.

---

### Pitfall 13: Functional registry codegen drift between the xtask and the rustc build

**What goes wrong:**
Per decision D9, the functional registry is generated by `cargo xtask regen-registry` and checked into git. The xtask reads `xcfun-master/src/functionals/*.cpp` (test vectors, descriptors) and writes `xcfun-core/src/registry.rs`. If the xtask is not re-run after a C++ source edit, the registry silently reflects the old reference. If the xtask is re-run but `xcfun-master/` is the wrong vendored version, the registry silently reflects a different reference. CI can pass (because the registry and the C++ fixtures are self-consistent) while the vendored C++ is not what the project documents.

**Why it happens:**
- Check-in codegen is fast and hermetic at build time, but creates a manual dependency on "did you re-run xtask?".
- `xcfun-master/` is vendored (D18), not a submodule — there is no version pin beyond the directory contents.
- Developers assume CI regenerates the registry; CI does not (that would require a C++ toolchain).

**How to avoid:**
- **Content-hash gate:** `xcfun-core/src/registry.rs` starts with a `// Generated from xcfun-master/ SHA256: <hash>` comment. `xtask ci-check` computes the hash of `xcfun-master/src/functionals/` and asserts it matches the comment. Any mismatch fails CI.
- **Forcing function:** CI runs `cargo xtask regen-registry --check` which regenerates the registry to a temp file and diffs against the checked-in file. Non-zero diff fails CI.
- **Version-stamped fixtures:** `validation/fixtures/*.jsonl` records `xcfun_version` (already in the spec). Loader asserts the fixture version matches the vendored xcfun version from `xcfun-master/CMakeLists.txt`.
- **Contributor docs:** `CONTRIBUTING.md` step 1 is "run `cargo xtask regen-registry` before committing if you edited `xcfun-master/`".

**Warning signs:**
- Parity passes locally but fails in CI after a `xcfun-master/` update.
- Test vectors in `registry.rs` don't match the test vectors compiled into the vendored C++ library.
- A functional's `test_threshold` changed upstream but the Rust registry still has the old value.

**Phase to address:**
Foundations (xtask setup) and Phase 2 (registry validation). The content-hash gate is part of the first xtask commit.

---

### Pitfall 14: Panic-across-FFI from `xcfun-capi`

**What goes wrong:**
`xcfun-capi` exports `#[no_mangle] extern "C"` symbols that Rust can unwind out of if a panic fires (e.g. slice bounds check, integer overflow). Unwinding across an FFI boundary is undefined behaviour. A C caller (or the Python binding's `maturin`-linked FFI) sees arbitrary corruption, missing destructors, or a segfault. Design decision D13 mandates `catch_unwind` at every entry point; a missed entry point breaks the contract silently.

**Why it happens:**
- It's easy to add a new `#[no_mangle] extern "C" fn xcfun_newthing` without remembering to wrap the body in the `c_entry!` macro.
- `panic = "abort"` in the release profile is a partial mitigation but does not cover `cargo test` builds nor consumers linking at `panic = "unwind"`.
- `cbindgen`-generated headers don't record that the Rust function may panic.

**How to avoid:**
- **Force the macro at every entry point:** a clippy lint or a compile-time `#[must_use]` wrapper macro. `c_entry! { pub fn xcfun_eval(fun: *const xcfun_t, input: *const f64, output: *mut f64) { ... } }` is the only idiom allowed; raw `#[no_mangle] extern "C"` is lint-rejected in `xcfun-capi`.
- **Panic-abort profile for `xcfun-capi` cdylib:** `[profile.release] panic = "abort"` in the workspace, enforced by CI.
- **Fuzz the C ABI:** `cargo fuzz` driver sends malformed inputs (null pointers, misaligned, out-of-bounds sizes) and asserts none produce UB.

**Warning signs:**
- Segfaults in Python tests after a Rust-side refactor.
- `cargo test` (which defaults to `panic = "unwind"`) fails with UB sanitizer messages.
- cbindgen-generated `xcfun.h` drifts from the reference (a new unwrapped export shows up).

**Phase to address:**
Phase 5 or 6 (C ABI + Python bindings). Lint rule and macro discipline land with the first `xcfun-capi` commit.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Port CTaylor multiply as a flat triple-loop instead of recursive descent | Faster to write (~30 minutes vs 2 hours) | Parity drift at order ≥ 2 across all functionals; weeks of "why is my rounding off" debugging | Never — parity-breaking |
| Skip the `*_expand` golden-coefficient tests; rely on end-to-end parity | Saves writing 12 tests × 3 inputs × 7 orders | When a parity failure hits in a GGA functional, you cannot tell if the bug is in the functional body, the expand routine, or the ctaylor multiply | Never — debuggability-breaking |
| Hand-transcribe the 78-functional test-vector table instead of codegen | Avoids the xtask scaffolding | Drift between xcfun-master and the registry becomes invisible | Only for < 10 functionals in a prototype spike; re-codegen before Phase 2 exit |
| Use `debug_assert!` for numerical preconditions | Matches C++ `assert` in debug builds | Release builds silently produce NaN | Never in the library hot path; OK in test-only code |
| Use `num-traits::Float` bound instead of custom `Num` | Saves defining a trait | Pulls in `is_nan`, `classify` etc. that are meaningless for CTaylor; forces unsound defaults | Never — explicitly rejected in D2 |
| Skip Wgpu validation, ship CUDA-only | Removes a flaky CI lane | Users on AMD GPUs and macOS silently run at 1e-8 precision | OK for v0.1; Phase-7 must add the `erf`-aware CPU fallback and document the 1e-9 Wgpu bar |
| Call `f64::mul_add` for "performance" in functional bodies | +2–5 % on microbenchmarks | Breaks rounding parity with C++ (C++ does not emit FMA by default) | Never in functional bodies; OK in auxiliary kernels that don't enter the parity harness |
| Skip cross-backend tier-3 in PR CI; run only nightly | Faster PR cycle | GPU-introduced drift lands in main; rollback becomes harder | OK if a GPU smoke test (1 functional × 1 vars × 1 order) runs per PR; full tier-3 nightly |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Linking xcfun-master via `cc` for tier-2 parity | Building with different `-O` level than the reference's default → different libm calls generated inline | Mirror the reference `CMakeLists.txt` flags exactly in `build.rs`; freeze the compiler flags used for fixture generation |
| PyO3 0.28 with numpy 0.28 | Using PyO3 0.28 against numpy 0.27 (or vice versa) → compile error or UB | Pin both to `0.28.x` in `Cargo.toml`; enforced by `cargo-deny` |
| cubecl `Array<F>` on CUDA | Passing a host slice directly instead of via `client.create(...)` | Use cubecl's client-side allocator always; `CpuRuntime` allows zero-copy but other runtimes do not |
| cbindgen header generation | Running cbindgen against a stale build → generated `xcfun.h` misses newly added exports | `xtask regen-header` runs cbindgen with `--config cbindgen.toml`; CI diff-checks against `xcfun-master/api/xcfun.h` |
| C ABI `xcfun_eval` with mismatched `density_pitch` | Caller passes a pitch that doesn't match the Vars' `input_length` | Validate `density_pitch >= xcfun_input_length(fun)` at the ABI boundary; return `XC_EPITCH` (a new error code) |
| Python `numpy.ndarray` input | User passes an `f32` array to a function expecting `f64` | `rust-numpy`'s `PyReadonlyArray1<f64>` rejects at runtime; surface as a Python `TypeError`, not a panic |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Allocating CTaylor on the heap | `malloc` shows up in `perf` hot path; per-point eval takes > 1 µs | CTaylor is `[T; 1 << N]`; never `Vec<T>`. Test: `cargo clippy -- -D clippy::needless_collect -D clippy::box_default` | Any per-point path; scales with grid size |
| Launching a cubecl kernel for < 64 points | GPU warm-up cost (~100 µs) dominates; total time 10× the CPU equivalent | `eval_vec` dispatches to CpuRuntime below `XCFUN_MIN_BATCH_SIZE` (default 64); env-var tunable | Small-grid users (molecule relaxation at individual points) |
| Re-uploading `weights_buf` on every `eval_vec` | PCIe HtoD traffic dominates; ~5 % of kernel time wasted on constant | `Functional::set` bumps a generation counter; `Batch` re-uploads only when counter changes | Iterative workflows where the active set is fixed across millions of grid points |
| Invoking libm transcendentals in a tight inner loop without SIMD | CPU path at ~50 % of theoretical throughput | Let LLVM autovectorize `log`/`exp` via `-Ctarget-feature=+avx2` in the CI/release profile; CubeCL CPU runtime vectorizes naturally | Large grid CPU workloads |
| Dropping into a debug build for correctness tests | Debug is 20–50× slower; CI timeout | Tier-2/3 runs in `cargo test --release`; use `--profile test-release` with overflow-checks on | Development workflow (not prod); but CI timeout is a prod issue |
| `rayon`/`std::thread::scope` added at library level | Non-deterministic summation order → parity tests flake | Design decision D17 forbids library-internal threading; CpuRuntime handles parallelism | Any multi-threaded test run; manifests as intermittent failure |
| Excessive kernel specializations exploding compile time | `cargo build --release` takes > 10 min; binary > 100 MB | Single kernel with `#[comptime]` on `(vars, mode, order)` only; functional id is runtime (78-arm match); measured trade-off per D6 | Full-matrix specialization; catastrophic for PTX compile time |

## Accuracy Traps (domain-specific, goes in addition to above)

| Trap | Symptoms | Prevention | Trigger Scale |
|------|----------|------------|---------------|
| `pow(x, 1.0/3.0)` vs `cbrt(x)` | ~1 ULP difference; silently breaks at 1e-12 | Use `cbrt` where the C++ uses `cbrt`; use `pow` where the C++ uses `pow`. `cbrt_expand` and `pow_expand` are separate functions in `tmath.hpp`; use the corresponding Rust expand. | Any cube-root use; LDA r_s computation |
| `exp(-x*x)` vs `gauss_expand`-ported form | `exp(-x*x)` in floating point loses precision when `x*x` overflows intermediate range | `gauss_expand` (tmath.hpp) uses the stable form `exp(-a²)·exp(-2ax)·exp(-x²)`; port this, not `exp(-x*x)` | Range-separated functionals with large mu |
| Accumulating `out += w1 * f1(d) + w2 * f2(d)` | Compiler may reassociate across the + to `(out + w1*f1) + w2*f2` vs `out + (w1*f1 + w2*f2)` | Sum explicit: `out += w1 * f1(d); out += w2 * f2(d);` — each statement is its own FP op | Always (hybrid functionals) |
| Forgetting to apply `settings[id]` weight in kernel | All-active-functional kernel evaluates to 0 | `call_functional`-dispatched body returns an unweighted functional value; weighting happens in the enclosing loop | Always |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Panic-across-FFI (covered in Pitfall 14) | UB, segfaults exploitable by malicious inputs | `c_entry!` macro on every extern "C" function; panic = "abort" in cdylib release |
| Integer overflow in `density_pitch * p + i` for huge grids | Heap corruption via out-of-bounds write | All pitch arithmetic uses `usize`; validate `nr_points * density_pitch < isize::MAX` at the ABI boundary; `checked_mul`-propagate on untrusted input |
| C ABI accepting null pointers without check | Null-pointer deref panic → UB across FFI | Every `*const`/`*mut` arg validated at the top of each `c_entry!`; null → `XC_EARG` return code |
| Python bindings accepting arbitrarily shaped numpy arrays | Read-out-of-bounds → process crash | `rust-numpy`'s shape checks at the PyO3 boundary; never trust `.as_slice_unchecked()` |
| Deserializing fixture files without integrity check | Malicious fixture file can inject wrong reference values and silently break CI | Fixture file ships with SHA-256 hash committed to `Cargo.toml` metadata; CI verifies before loading |

## "Looks Done But Isn't" Checklist

- [ ] **CTaylor multiply:** Often missing order-0 and order-1 explicit specializations that C++ provides for efficiency — verify `ctaylor_rec<T, 0>` and `ctaylor_rec<T, 1>` are ported as explicit base cases, not as `if N == 0 { ... }` in the generic path.
- [ ] **Regularize:** Often missing the "derivative coefficients preserved" guarantee — verify via unit test that `regularize` on a `CTaylor` with nonzero derivative coefficients leaves those coefficients unchanged.
- [ ] **DensVars fallthrough arms:** Often missing the superset variants (e.g. `XC_A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB`) whose fallthrough chain is deepest — verify every `DensVars` field is populated for every variant by a field-by-field test.
- [ ] **Alias resolution:** Often missing the additive-weight semantic — verify `xcfun_set(fun, "b3lyp", 1.0); xcfun_set(fun, "slaterx", 0.5)` results in `slaterx` weight of 1.3 (0.8 from b3lyp + 0.5), not 0.5.
- [ ] **`pow` vs `powi` in functional bodies:** Often missing the distinction — verify that exchange functionals using `pow(ρ, 4.0/3.0)` use `pow_expand`, not a simpler `x*cbrt(x)*cbrt(x)` trick.
- [ ] **Negative-weight aliases:** Often untested — verify `camcompx` (with `-1.0` beckecamx weight) resolves correctly.
- [ ] **Order-0 partial derivatives output:** Often missing a test that order-0 and order-1 produce consistent `out[0]` energies — the C++ special-cases `ctaylor<T, 0>` at `XCFunctional.cpp:503`; Rust must mirror.
- [ ] **Contracted mode at `order = 6`:** Often tested only at order ≤ 4 — exercise the order-5 and order-6 paths explicitly.
- [ ] **Potential mode with GGA:** Often missing the "needs full laplacian" check — verify `xcfun_eval_setup` returns `XC_EVARS | XC_EMODE` on misuse (mirrors `XCFunctional.cpp:438-447`).
- [ ] **cbindgen output:** Often drifts silently — CI diff-checks generated `xcfun.h` against the reference.
- [ ] **Python wheel f64 dispatch:** Often defaults to f32 for "performance" — verify `xcfun_py.eval(arr)` where `arr.dtype == np.float32` either raises or up-casts explicitly.
- [ ] **cubecl version pin:** Often relaxed accidentally via `cargo update` — CI asserts `Cargo.lock` keeps `cubecl = "0.10.0-pre.3"` exactly.
- [ ] **Wgpu SHADER_F64 gate:** Often missing on untested adapters — CI runs a software Wgpu adapter without f64 and asserts `auto_backend` returns `Err(WgpuNoF64)`.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Reassociation drift (P1) | MEDIUM | 1. Add `-Cllvm-args=-fp-contract=off` to release profile; 2. Grep and remove `mul_add` calls; 3. Re-run tier-2; 4. If still drifting, `cargo asm` diff against C++ and re-introduce `let` intermediates. |
| CTaylor coefficient layout bug (P3) | HIGH | 1. Write the bit-equivalence test against C++; 2. Port `ctaylor_rec` recursion byte-for-byte (abandon any "cleanup" rewrites); 3. Re-run tier-1 (self-tests) then tier-2. Recovery is weeks if the layout has been used by downstream functional ports. |
| `regularize` bug (P4) | LOW | 1. Fix the `regularize` to touch only `c[CNST]`; 2. Add the unit test with preserved derivatives; 3. Re-run tier-2 at small-ρ boundary. Isolated to densvars. |
| DensVars fallthrough bug (P5) | LOW to MEDIUM | 1. Refactor to helper-function chain; 2. Add field-by-field parity test per Vars variant. Cost scales with how many functionals were developed on incorrect DensVars. |
| Alias weight bug (P6) | LOW | 1. Port the recursive `set` with `value * weight`; 2. Add the camb3lyp + camcompx parity tests. Isolated. |
| Wgpu silent f32 (P7) | MEDIUM | 1. Add the adapter feature check; 2. Surface as a hard error; 3. Document in Wgpu troubleshooting section. Users on broken adapters see an error instead of wrong results, which is the goal. |
| cubecl API drift (P8) | HIGH if a major API change landed silently | 1. `cargo update` the workspace; 2. Fix compile errors (usually in 1-2 places due to isolation); 3. Re-run full tier-2 + tier-3. If results change, re-validate against C++ reference. |
| Series-expand coefficient bug (P9) | MEDIUM | 1. Re-port the expand body from `tmath.hpp`; 2. Run golden-coefficient test; 3. Re-run tier-2. Isolated to a single expand function typically. |
| Horner direction (P11) | LOW | Flip the iteration direction; add the ascending-vs-descending differential test. |
| PW92C constants (P12) | LOW | Flip the Cargo feature or fix the constants; re-run tier-2 for PW92C-using functionals. |
| Registry drift (P13) | LOW if caught early | Re-run `xtask regen-registry`; update the content hash; commit. CI gate catches it in practice. |
| Panic-across-FFI (P14) | HIGH if shipped; caller crashes | Add `c_entry!` wrapper; add UBsan CI job; release patch version. Already-deployed consumers need a library upgrade. |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P1 Reassociation drift | Foundations + Phase 2 (first functionals) | CI `RUSTFLAGS`-audit job; `rg mul_add` lint; disassembly spot-check on slaterx + pw92c |
| P2 Transcendental drift | Phase 3 (GGA + range-separated) and GPU phase | Backend-qualified tolerance table; erf-aware CPU fallback; macOS/Windows/Linux runners |
| P3 CTaylor layout | Phase 1 (AD engine) | Bit-equivalence test against C++ intermediates for simple inputs; layout is a Phase-1 exit criterion |
| P4 regularize scope | Phase 2 (densvars) | Unit test: regularize preserves higher-order coefficients |
| P5 Densvars fallthrough | Phase 2 (densvars) | Per-variant field-by-field parity test (all 31 variants) |
| P6 Alias weights | Phase 4/5 (Functional API + alias table) | Alias-resolution test for all 50 aliases at `value ∈ {1.0, 0.37}`; negative-weight canary (camcompx) |
| P7 Wgpu silent f32 | GPU phase | Software Wgpu adapter without SHADER_F64 → expect `Err(WgpuNoF64)` |
| P8 cubecl API drift | Foundations | `Cargo.lock` CI assertion; cubecl isolation to `xcfun-kernels` + `xcfun-gpu` |
| P9 `*_expand` coefficient layout | Phase 1 (AD engine) | Golden-coefficient tests for each expand function at 3 inputs × 7 orders |
| P10 Non-analytic `*_expand` inputs | Phase 1 + Phase 2 | Fuzz densvars with `ρ ∈ [1e-30, 1e-10]`; NaN canary in parity harness |
| P11 Horner direction | Phase 2 (specmath) | Differential test descending vs ascending; inspection in code review |
| P12 PW92C constants | Phase 0 scoping + Phase 2 | Read `config.hpp` in Phase 0; `pw92c-legacy-constants` Cargo feature with validation against both |
| P13 Registry drift | Foundations (xtask setup) | Content-hash comment on registry.rs; `xtask regen-registry --check` in CI |
| P14 Panic-across-FFI | Phase 5/6 (C ABI + Python) | `c_entry!` macro discipline; UBsan CI job; `panic = "abort"` in cdylib release |

## Sources

- `xcfun-master/src/densvars.hpp` — regularize scope and densvars fallthrough (lines 22-25, 40-212)
- `xcfun-master/src/XCFunctional.cpp` — alias recursion and set semantics (lines 369-405); dispatcher eval order (lines 493-796)
- `xcfun-master/external/upstream/taylor/ctaylor.hpp` — bit-flag layout and multiplication recursion (lines 22-30, 41-65)
- `xcfun-master/external/upstream/taylor/tmath.hpp` — series-expansion coefficient formulas (lines 124-314)
- `xcfun-master/src/specmath.hpp` — descending Horner's rule (lines 24-33)
- `xcfun-master/src/config.hpp` — `XCFUN_TINY_DENSITY = 1e-14`, `XCFUN_REF_PW92C`, `XCFUN_REF_PBEX_MU` toggles (lines 22, 35-38)
- `docs/design/07-accuracy-strategy.md` — 1e-12 invariant, divergence sources table, failure-handling protocol
- `docs/design/06-cubecl-strategy.md` — Wgpu `erf` variance documentation, host/device purity rule, `SHADER_F64` gate
- `docs/design/12-design-decisions.md` — D1 (verbatim AD port), D4 (f64-only), D5 (Wgpu best-effort), D13 (catch_unwind FFI), D17 (no library threads)
- `CLAUDE.md` project stack pins — cubecl `=0.10.0-pre.3`, PyO3/numpy 0.28.x major alignment
- `xcfun-master/src/functionals/aliases.cpp` — 50+ alias definitions including negative-weight `camcompx` and `camb3lyp` chain

---
*Pitfalls research for: xcfun C++ → Rust port with cubecl CPU/GPU kernels and 1e-12 parity*
*Researched: 2026-04-18*
