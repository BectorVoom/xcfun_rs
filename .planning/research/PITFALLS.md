# Domain Pitfalls

**Domain:** Rust reimplementation of C++ exchange-correlation functional library (DFT)
**Researched:** 2026-04-17

---

## Critical Pitfalls

Mistakes that cause rewrites, incorrect scientific results, or months of debugging.

### Pitfall 1: Operator Evaluation Order Divergence in CTaylor Multiplication

**What goes wrong:** The C++ ctaylor `multo` (in-place multiply) reads and writes to the same array, with the recursive structure processing from high indices to low indices. The exact order of reads/writes determines which intermediate values are still "old" vs already overwritten. Reimplementing this in Rust with different loop ordering, iterator patterns, or even different recursion flattening produces subtly different floating-point rounding, causing errors that appear only at derivative order >= 3.

**Why it happens:** The C++ `ctaylor_rec<T, Nvar>::multo` recurses as: first multo the upper half, then mul (accumulate cross-terms), then multo the lower half. This is NOT a simple loop. The dependency between the three recursive calls means dst[high] is updated using the old dst[low], but then dst[low] is updated using old y. If you flatten this recursion into iterative loops (tempting in Rust), you must preserve exactly this processing order.

**Consequences:** Derivatives appear correct at order 1-2 but diverge at order 3+. The error is often 1e-10 to 1e-8 -- just barely above tolerance. Extremely hard to debug because the error is small and only manifests in high-order mixed derivatives.

**Prevention:**
1. Port the recursive template specializations as recursive Rust functions with the SAME structure, not as iterative loops.
2. Use const generics to specialize at compile time (matching C++ template recursion), not runtime dispatch.
3. Test `multo` specifically with known polynomials at Nvar=3,4,5,6 before building functionals on top.
4. Compare bit-for-bit against C++ for the intermediate coefficient arrays, not just final results.

**Detection:** Test all 78 functionals at derivative orders 3, 4, 5, 6 early. Order 1-2 passing does NOT indicate correctness.

**Phase:** AD engine (Phase 1). Must be verified before any functional implementation begins.

---

### Pitfall 2: Taylor Composition (`compose`) Hardcoded Faa di Bruno Coefficients

**What goes wrong:** The C++ `tmath.hpp::tfuns<T,N>::compose()` function has a hand-coded switch statement for orders 0 through 6 with explicit Faa di Bruno coefficients (the combinatorial factors for higher-order chain rule). These are NOT computed from a formula at runtime -- they are manually derived constants. A single wrong coefficient (e.g., writing `3 * f[3]` instead of `6 * f[3]` in the order-5 term) produces incorrect high-order derivatives.

**Why it happens:** The Faa di Bruno formula for the n-th derivative of a composite function f(g(x)) involves partition-counting combinatorics. The C++ code has these baked in as magic numbers. The Rust port must reproduce them exactly. There is no runtime formula to verify against -- the constants ARE the implementation.

**Consequences:** Wrong derivatives for ALL transcendental functions (exp, log, pow, sqrt, erf, atan, asinh, etc.) at the affected order. Since compose() is the single codepath for all of these, one bug here breaks everything.

**Prevention:**
1. Port the compose() switch cases verbatim, character by character. Do NOT attempt to "derive" the formula or generate coefficients programmatically (unless you then verify every coefficient against the C++ values).
2. Write a dedicated test that computes d^n/dx^n of exp(x), log(x), sqrt(x) at multiple points for each order 1-6 and compares against symbolic derivatives (which are trivially known for these functions).
3. Test compose() in isolation with identity-like inputs before integrating with ctaylor.

**Detection:** The exp() test is the most sensitive: d^n/dx^n exp(x) = exp(x) for all n. Any discrepancy at any order immediately reveals a compose() bug.

**Phase:** AD engine (Phase 1). This is the single most critical function to get right.

---

### Pitfall 3: Regularization Must Only Touch c[0] (Constant Coefficient)

**What goes wrong:** When density is below `TINY_DENSITY = 1e-14`, the C++ code clamps only `c[0]` (the value) to the threshold, leaving derivative coefficients `c[1..N]` untouched. A naive Rust implementation might clamp the entire variable (e.g., `x = x.max(threshold)` where x is a CTaylor), which would zero out all derivatives -- producing correct energy but completely wrong potentials and higher derivatives.

**Why it happens:** In C++, the `regularize` function template-specializes on `ctaylor<T,N>` vs plain `T`. The ctaylor version uses `x.set(0, TINY_DENSITY)` while the scalar version uses `x = TINY_DENSITY`. In Rust, if `CTaylor` implements `PartialOrd` by comparing only `c[0]`, a `max` operation would still replace the entire struct.

**Consequences:** All derivatives (potentials, second derivatives) are wrong near zero density. This affects every functional. DFT codes will get wrong Kohn-Sham potentials, leading to incorrect SCF convergence or wrong total energies.

**Prevention:**
1. Implement `regularize` as a method on `CTaylor` that explicitly sets only `c[0]`.
2. Do NOT implement `Ord` or comparison traits on `CTaylor` that participate in `min`/`max` -- these should be value-only operations that return the full polynomial.
3. Write a test: `regularize(CTaylor::variable(1e-20, 0))` must produce `c[0] = 1e-14, c[1] = 1.0` (derivative preserved).

**Detection:** Test any functional's first derivative at density = 1e-20. If derivative is 0.0, regularization is wrong.

**Phase:** Core types (Phase 1). Must be correct before densvars construction.

---

### Pitfall 4: C++ Switch Statement Fallthrough in densvars Constructor

**What goes wrong:** The C++ `densvars` constructor uses switch-case fallthrough extensively. For example, `XC_A_B_GAA_GAB_GBB_TAUA_TAUB` falls through to `XC_A_B_GAA_GAB_GBB` which falls through to `XC_A_B`. This means code for gradient setup runs for ALL cases that include gradients. In Rust, match arms don't fall through. Porting this as a simple `match` with break-equivalent semantics per arm will produce WRONG field initialization for every VarType except the simplest ones.

**Why it happens:** C switch fallthrough is a feature Rust intentionally lacks. Each C++ case block relies on the blocks below it to complete initialization. The Rust port must either: (a) explicitly duplicate the shared code in each arm, (b) use helper functions that compose, or (c) restructure as layered initialization.

**Consequences:** Fields like `n`, `s`, `gnn`, `gns`, `gss` are not initialized for variable types that include gradients or kinetic energy. Every functional evaluation with gradient variables produces garbage.

**Prevention:**
1. Map each C++ case + its fallthrough chain to the complete set of assignments that actually execute.
2. Implement as layered helpers: `init_spin(d) -> init_gradient(d) -> init_kinetic(d)` that compose.
3. Write an exhaustive test for every VarType that verifies ALL fields match C++ output for a known input.

**Detection:** Test `densvars` construction for every VarType variant. Check that derived quantities (r_s, zeta, n_m13, a_43, b_43) match C++ values.

**Phase:** Core types (Phase 1). Blocking for all functional work.

---

### Pitfall 5: GPU f64 Support is Backend-Dependent (WebGPU Has NO f64)

**What goes wrong:** CubeCL targets multiple backends: CUDA, ROCm, Metal, and WGPU (WebGPU/Vulkan). WebGPU does NOT support f64 (IEEE-754 binary64) in compute shaders -- it only supports f32. Metal has limited f64 support. CUDA and ROCm support f64 but consumer GPUs have 1/32 the f64 throughput of f32. Writing kernels that use `f64` will either fail to compile on WGPU/Metal or run at dramatically reduced speed on consumer NVIDIA GPUs.

**Why it happens:** The cubecl `Float` trait abstracts over floating-point types, but the available precisions depend on the backend. The xcfun project requires 1e-12 accuracy, which is physically impossible with f32 (which has ~7 decimal digits of precision, providing at best ~1e-7 relative error).

**Consequences:** GPU path either: (a) doesn't compile on WGPU backend, (b) silently produces wrong results if forced to f32, or (c) runs 32x slower than expected on consumer GPUs.

**Prevention:**
1. GPU kernels must use f64 exclusively -- f32 cannot meet the 1e-12 accuracy requirement.
2. Restrict supported backends to CUDA and ROCm (professional GPUs with good f64 throughput). Document that WGPU backend is not supported for this library.
3. Add a runtime check for f64 capability and fall back to CPU if unavailable.
4. For orders 0-1 only: consider a mixed-precision approach where accumulation uses f64 but intermediate steps could use f32 with Kahan compensation. (Not recommended -- complexity exceeds benefit.)

**Detection:** CI must test GPU path with f64 inputs and verify 1e-12 accuracy against CPU reference, not just "kernel runs without error."

**Phase:** GPU acceleration (Phase 4). Design the fallback strategy before writing any kernels. Confidence: HIGH (verified via WebGPU spec issue #2805 and WGPU issue #7017).

---

### Pitfall 6: Monomorphization Explosion from CTaylor<f64, N> x 78 Functionals

**What goes wrong:** Each functional's `energy<T>()` is generic over T. When T = `CTaylor<f64, N>`, the compiler generates separate code for each N (0 through 6 = 7 variants). With 78 functionals, this is 78 * 7 = 546 monomorphized function bodies, each containing the full functional formula with CTaylor arithmetic (which itself is 2^N coefficient operations per multiply). Compile time explodes (10+ minutes), binary size balloons (potentially 50+ MB), and instruction cache pressure degrades runtime performance.

**Why it happens:** Rust's monomorphization is the same as C++ template instantiation. The C++ xcfun has the same theoretical problem but mitigates it because the C++ compiler's optimizer is more aggressive at deduplication and the build system uses unity builds. Rust's incremental compilation helps but doesn't solve the fundamental issue.

**Consequences:** CI build times exceed 15 minutes. Debug builds are unusably slow. Binary size prevents embedding in Python wheels. Instruction cache misses cause 2-3x performance regression vs C++ despite identical algorithmic complexity.

**Prevention:**
1. Only instantiate the derivative orders actually requested by `eval_setup()`. Use enum dispatch over order (match on order, call the specific `CTaylor<f64, ORDER>` variant) rather than generating all 7 at compile time.
2. Split functionals across compilation units (the 7-crate workspace helps here -- xcfun-functionals can have internal modules per functional family).
3. Use `#[inline(never)]` on individual functional `energy()` implementations to prevent the compiler from inlining 546 function bodies into the evaluation loop.
4. Profile compile times early. If debug builds exceed 60 seconds, the monomorphization strategy needs revision.

**Detection:** Track `cargo build` time and `target/release/` binary size in CI. Alert on regressions.

**Phase:** Functional implementation (Phase 2-3). The explosion becomes apparent only after implementing 20+ functionals. Design the dispatch strategy in Phase 1.

---

## Moderate Pitfalls

### Pitfall 7: Floating-Point Constant Precision Mismatch

**What goes wrong:** C++ functional implementations contain numeric constants like `0.0621814` (VWN5 para[1]) which may have different precision in C++ vs Rust. C++ double literals are truncated to the precision written. Rust `f64` literals are also truncated. But if the C++ code computes a constant at compile time (e.g., `pow(3 * M_PI * M_PI, -1.0)` in VWN5 inter[1]), the C++ compiler's constant folding may produce a different bit pattern than Rust's equivalent.

**Prevention:**
1. Extract ALL numeric constants from C++ source as hex float literals (`0x1.FEDCBAp+5` format) to guarantee bit-exact representation.
2. For compile-time computed constants (like `pow(3*PI^2, -1)`), compute them once in C++ with `printf("%.20e", value)` and paste the result into Rust as a literal.
3. Never use `std::f64::consts::PI` if C++ uses `M_PI` -- they may differ in the last bit. Use the exact same bit pattern.

**Detection:** Compare `DensityVars` field values for identical inputs between C++ and Rust. A 1-2 ULP difference in constants propagates to ~1e-14 error in energy, which is close to the 1e-12 tolerance.

**Phase:** Functional implementation (Phase 2). Each functional port must verify constants.

---

### Pitfall 8: pow(x, non-integer) Near x=0 Divergent Derivatives

**What goes wrong:** Many functionals use `pow(density, 4.0/3.0)` or `pow(r_s, 0.5)`. The Taylor expansion of `pow(x, a)` around x=x0 has coefficients `t[i] = t[i-1] * (1/x0) * (a-i+1)/i`. When x0 is small (near the regularization threshold 1e-14), `1/x0` is ~1e14, and the higher-order Taylor coefficients grow explosively: t[6] can be O(1e84). While these huge coefficients are mathematically correct (they represent the rapid variation of the function near zero), they cause numerical overflow and catastrophic cancellation when multiplied with tiny derivative perturbations.

**Prevention:**
1. Regularize density BEFORE computing pow. This is already the C++ approach (regularize is called at the top of densvars constructor).
2. For `pow(a, 4.0/3.0)` and `pow(b, 4.0/3.0)`, the regularized density 1e-14 gives `pow(1e-14, 4/3) = 1e-18.67` -- small but finite. Verify these survive 6 orders of differentiation without overflow.
3. Add `debug_assert!` checks that no CTaylor coefficient exceeds 1e100 after transcendental operations.

**Detection:** Test every functional at `density = [1e-14, 1e-14]` (regularization boundary) with order 6. Check for NaN, Inf, or relative errors exceeding 1e-6.

**Phase:** AD engine (Phase 1) and functional implementation (Phase 2).

---

### Pitfall 9: cbindgen Cannot Generate Headers for Generic or Complex Rust Types

**What goes wrong:** cbindgen generates C headers from Rust source. It cannot handle: generic types (CTaylor<f64, N>), trait objects, enums with non-trivial payloads, or types behind feature gates. The FFI layer must expose only simple, `#[repr(C)]`-compatible types. Attempting to expose the internal `XcFunctional` struct directly will fail because it contains Vecs, Options, and enum-dispatched functionals.

**Prevention:**
1. The FFI crate must define opaque handle types: `pub struct XcFunHandle { _private: () }` exposed as `struct xcfun_t;` in C.
2. All FFI functions take `*mut XcFunHandle` and internally cast to `&mut XcFunctional` using `Box::from_raw` / `Box::into_raw`.
3. Use `#[unsafe(no_mangle)]` and `extern "C"` on every FFI function. Return only primitives (c_int, c_double, pointers).
4. Run cbindgen in CI to generate the header and diff it against the committed header -- catch accidental API changes.

**Detection:** If cbindgen fails to generate a header or generates `#include` for non-existent types, the FFI layer needs restructuring.

**Phase:** FFI (Phase 5). Design the opaque handle pattern before implementing any FFI functions.

---

### Pitfall 10: PyO3 GIL Contention for Batch Evaluation

**What goes wrong:** PyO3 bindings that hold the GIL during batch evaluation prevent Python threads from doing anything else. For a 100K-point batch taking 50ms, the entire Python process is locked. This is particularly problematic when the Python caller is a DFT code with its own parallelism (e.g., PySCF uses multiprocessing).

**Prevention:**
1. Use `py.detach()` (PyO3 0.23+) to release the GIL during all batch evaluations.
2. Accept numpy arrays via `PyReadonlyArray1<f64>` for zero-copy input access.
3. Return results as numpy arrays allocated on the Rust side via `PyArray1::from_vec(py, results)`.
4. Never hold references to Python objects while GIL is released.

**Detection:** Benchmark `xcfun.eval_batch()` from two Python threads simultaneously. If throughput is 1x instead of 2x, GIL is not released.

**Phase:** Python bindings (Phase 6).

---

### Pitfall 11: The `sqrtx_asinh_sqrtx` Pade Approximation Boundary

**What goes wrong:** The C++ code uses a [8,8] Pade approximation for `sqrt(x)*asinh(sqrt(x))` when |x| < 0.5, switching to the direct formula when |x| >= 0.5. This boundary at 0.5 is carefully chosen -- the Pade approximation diverges beyond it, and the direct formula loses accuracy below it. If the Rust implementation uses a different threshold, or if the Pade coefficients are not exactly reproduced, the function will have a discontinuity or accuracy loss at the boundary.

**Prevention:**
1. Use EXACTLY the same Pade coefficients (all 18 values in P[] and Q[]).
2. Use EXACTLY the same threshold (0.5).
3. Test at x = 0.499, 0.500, 0.501 to verify continuity of the function AND its derivatives up to order 6.
4. Note the C++ asserts `Nvar < ASINH_TABSIZE (9)` -- this limits this function to at most 8 variables (which is fine for xcfun's max of 6).

**Detection:** Plot `sqrtx_asinh_sqrtx` and its derivatives across the boundary. Any jump exceeding 1e-12 indicates a problem.

**Phase:** AD engine (Phase 1). This is a special function unique to xcfun.

---

### Pitfall 12: AoS-to-SoA Transpose Correctness for GPU Batch Path

**What goes wrong:** The C FFI and Rust API use AoS layout (all variables for one grid point are contiguous). The GPU path needs SoA (each variable across all points is contiguous). The transpose must handle variable-count-dependent layouts: A_B has 2 input variables, A_B_GAA_GAB_GBB has 5, and A_B_GAA_GAB_GBB_TAUA_TAUB has 7. A generic transpose that assumes a fixed stride will produce garbled data for any VarType other than the one it was tested with.

**Prevention:**
1. Parameterize the transpose by `n_vars` (from `VarType::input_len()`).
2. Test the AoS->SoA->AoS roundtrip for every VarType variant, not just A_B.
3. The SoA buffer layout must match what the GPU kernel expects. Define the layout in one place and share it between the transpose and the kernel.

**Detection:** The GPU/CPU consistency test (section 4 of test strategy) will catch this, but only if it tests all VarType variants.

**Phase:** GPU acceleration (Phase 4).

---

## Minor Pitfalls

### Pitfall 13: Rust Edition 2024 `unsafe` Block Requirements

**What goes wrong:** Rust Edition 2024 requires `unsafe` blocks inside `unsafe fn` bodies. The FFI crate has many `unsafe` operations (pointer dereference, slice creation from raw parts). Code that compiles on Edition 2021 may fail on 2024 due to stricter linting and the `unsafe_op_in_unsafe_fn` lint being deny-by-default.

**Prevention:** Use `#[unsafe(no_mangle)]` (not `#[no_mangle]`) and wrap every unsafe operation in an explicit `unsafe {}` block, even inside `extern "C"` functions.

**Phase:** FFI (Phase 5). The Cargo.toml already specifies edition 2024.

---

### Pitfall 14: cubecl Pre-Release API Instability

**What goes wrong:** The project depends on `cubecl = "=0.10.0-pre.3"` (exact pre-release pin). Pre-release CubeCL APIs change between minor versions. The kernel launch syntax, tensor argument types, and compilation pipeline may change before 0.10.0 stable.

**Prevention:**
1. Keep the exact version pin.
2. Isolate all cubecl usage behind the `xcfun-gpu` crate boundary. No cubecl types should leak into the public API.
3. When cubecl reaches stable, update in a single focused PR.
4. Write GPU integration tests that can be disabled when cubecl breaks.

**Detection:** CI build failure after dependency update. The feature-gated isolation ensures CPU-only builds always work.

**Phase:** GPU acceleration (Phase 4).

---

### Pitfall 15: Panic Across FFI Boundary is Undefined Behavior

**What goes wrong:** If a Rust function panics while being called from C, the behavior is undefined (the unwinding crosses a language boundary). The current FFI design document shows `expect()` in `xcfun_eval`, which will panic if the functional is not configured.

**Prevention:**
1. Wrap every FFI function body in `std::panic::catch_unwind()`.
2. Convert caught panics to error codes (e.g., `XC_EINTERNAL = 8`).
3. For functions that return void (like `xcfun_eval`), either: (a) make them return an error code, or (b) use `abort()` instead of panic (which the design doc already suggests).
4. Set `panic = "abort"` in the `[profile.release]` for the FFI cdylib crate to prevent unwinding entirely.

**Detection:** Call `xcfun_eval` from C without calling `xcfun_eval_setup` first. If the process crashes with a Rust panic message instead of a clean error code, the boundary is unprotected.

**Phase:** FFI (Phase 5).

---

### Pitfall 16: `1 - zeta^4` Cancellation Near Unpolarized Limit

**What goes wrong:** The VWN functional (both VWN3 and VWN5) computes `1 - zeta^4` where `zeta = (n_a - n_b) / (n_a + n_b)`. For nearly unpolarized systems, zeta is close to 0, so `zeta^4` is tiny (e.g., 1e-32), and `1 - zeta^4` loses many digits of precision to catastrophic cancellation. The C++ code even has a FIXME comment about this: `"FIXME: 1 - zeta^4 has a cancellation free form (ask maxima)"`.

**Prevention:**
1. For the initial port: replicate the C++ behavior exactly (including the cancellation). This matches the reference output.
2. For future improvement: factor as `(1 - zeta^2)(1 + zeta^2)` and further as `(1-zeta)(1+zeta)(1+zeta^2)`. Each factor is well-conditioned near zeta=0. But do NOT apply this fix unless the C++ reference values are updated to match.
3. This affects accuracy at the ~1e-14 level for nearly-unpolarized systems, which is within tolerance.

**Detection:** Test VWN5 with input `[39.0, 39.0]` (equal alpha/beta) and verify the result matches C++ to 1e-12.

**Phase:** LDA functionals (Phase 2). Low priority -- the C++ code has the same limitation.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| AD engine (Phase 1) | Pitfall 1 (multo order), Pitfall 2 (compose coefficients), Pitfall 3 (regularization), Pitfall 11 (sqrtx_asinh_sqrtx) | Port C++ recursion structure verbatim. Test each function at all orders before proceeding. |
| Core types (Phase 1) | Pitfall 4 (switch fallthrough), Pitfall 7 (constant precision) | Test densvars for every VarType. Extract constants as hex floats. |
| LDA functionals (Phase 2) | Pitfall 8 (pow near zero), Pitfall 16 (cancellation) | Test at regularization boundary. Accept C++ cancellation behavior. |
| GGA/mGGA functionals (Phase 3) | Pitfall 6 (monomorphization), Pitfall 7 (constants) | Monitor compile times. Profile binary size after 20+ functionals. |
| GPU acceleration (Phase 4) | Pitfall 5 (f64 backend), Pitfall 12 (AoS/SoA), Pitfall 14 (cubecl stability) | CUDA/ROCm only. Test all VarTypes. Pin cubecl version. |
| C FFI (Phase 5) | Pitfall 9 (cbindgen), Pitfall 13 (edition 2024 unsafe), Pitfall 15 (panic across FFI) | Opaque handles. catch_unwind or panic=abort. |
| Python bindings (Phase 6) | Pitfall 10 (GIL contention) | py.detach() for all batch operations. Zero-copy numpy arrays. |

---

## Sources

- C++ xcfun source: `/xcfun-master/external/upstream/taylor/ctaylor.hpp`, `ctaylor_math.hpp`, `tmath.hpp`
- C++ densvars: `/xcfun-master/src/densvars.hpp`
- C++ VWN functional: `/xcfun-master/src/functionals/vwn.hpp`
- Project design docs: `docs/design/07-error-handling.md`, `08-testing.md`, `10-design-decisions.md`
- [WebGPU f64 limitation (gpuweb issue #2805)](https://github.com/gpuweb/gpuweb/issues/2805) -- HIGH confidence
- [WGPU strict f32-only compliance (wgpu issue #7017)](https://github.com/gfx-rs/wgpu/issues/7017) -- HIGH confidence
- [Rust monomorphization bloat (rust-lang issue #77767)](https://github.com/rust-lang/rust/issues/77767) -- HIGH confidence
- [cbindgen opaque types (cbindgen issue #492)](https://github.com/mozilla/cbindgen/issues/492) -- MEDIUM confidence
- [PyO3 GIL parallelism docs](https://github.com/pyo3/pyo3/blob/main/guide/src/parallelism.md) -- HIGH confidence
- [CubeCL documentation](https://burn.dev/books/cubecl/print.html) -- MEDIUM confidence (pre-release API)
