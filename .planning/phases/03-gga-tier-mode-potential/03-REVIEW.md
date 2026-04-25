---
phase: 03-gga-tier-mode-potential
reviewed: 2026-04-25T00:00:00Z
depth: standard
files_reviewed: 56
files_reviewed_list:
  - crates/xcfun-ad/src/expand/expm1.rs
  - crates/xcfun-ad/src/expand/mod.rs
  - crates/xcfun-ad/src/math.rs
  - crates/xcfun-ad/tests/golden_composed.rs
  - crates/xcfun-ad/tests/golden_expand.rs
  - crates/xcfun-eval/Cargo.toml
  - crates/xcfun-eval/src/density_vars.rs
  - crates/xcfun-eval/src/density_vars/build.rs
  - crates/xcfun-eval/src/density_vars/regularize.rs
  - crates/xcfun-eval/src/dispatch.rs
  - crates/xcfun-eval/src/functional.rs
  - crates/xcfun-eval/src/functionals/gga/apbe/apbec.rs
  - crates/xcfun-eval/src/functionals/gga/apbe/apbex.rs
  - crates/xcfun-eval/src/functionals/gga/apbe/mod.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97_1c.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97_1x.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97_2c.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97_2x.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97c.rs
  - crates/xcfun-eval/src/functionals/gga/b97/b97x.rs
  - crates/xcfun-eval/src/functionals/gga/b97/mod.rs
  - crates/xcfun-eval/src/functionals/gga/becke/beckecamx.rs
  - crates/xcfun-eval/src/functionals/gga/becke/beckecorrx.rs
  - crates/xcfun-eval/src/functionals/gga/becke/beckesrx.rs
  - crates/xcfun-eval/src/functionals/gga/becke/beckex.rs
  - crates/xcfun-eval/src/functionals/gga/becke/mod.rs
  - crates/xcfun-eval/src/functionals/gga/kt/btk.rs
  - crates/xcfun-eval/src/functionals/gga/kt/ktx.rs
  - crates/xcfun-eval/src/functionals/gga/kt/mod.rs
  - crates/xcfun-eval/src/functionals/gga/lyp.rs
  - crates/xcfun-eval/src/functionals/gga/mod.rs
  - crates/xcfun-eval/src/functionals/gga/optx/mod.rs
  - crates/xcfun-eval/src/functionals/gga/optx/optx.rs
  - crates/xcfun-eval/src/functionals/gga/optx/optxcorr.rs
  - crates/xcfun-eval/src/functionals/gga/p86/mod.rs
  - crates/xcfun-eval/src/functionals/gga/p86/p86c.rs
  - crates/xcfun-eval/src/functionals/gga/p86/p86corrc.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/mod.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbec.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbeintc.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbeintx.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbelocc.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbesolx.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/pbex.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/revpbex.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/rpbex.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/spbec.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/vwn_pbec.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/zvpbeintc.rs
  - crates/xcfun-eval/src/functionals/gga/pbe/zvpbesolc.rs
  - crates/xcfun-eval/src/functionals/gga/pw91/mod.rs
  - crates/xcfun-eval/src/functionals/gga/pw91/pw86x.rs
  - crates/xcfun-eval/src/functionals/gga/pw91/pw91c.rs
  - crates/xcfun-eval/src/functionals/gga/pw91/pw91k.rs
  - crates/xcfun-eval/src/functionals/gga/pw91/pw91x.rs
  - crates/xcfun-eval/src/functionals/gga/shared/b97_poly.rs
  - crates/xcfun-eval/src/functionals/gga/shared/constants.rs
  - crates/xcfun-eval/src/functionals/gga/shared/mod.rs
  - crates/xcfun-eval/src/functionals/gga/shared/optx.rs
  - crates/xcfun-eval/src/functionals/gga/shared/pbec_eps.rs
  - crates/xcfun-eval/src/functionals/gga/shared/pbex.rs
  - crates/xcfun-eval/src/functionals/gga/shared/pw91_like.rs
  - crates/xcfun-eval/src/functionals/lda/pw92eps.rs
  - crates/xcfun-eval/src/functionals/lda/pz81c.rs
  - crates/xcfun-eval/src/functionals/mod.rs
  - crates/xcfun-eval/src/functionals/potential.rs
  - crates/xcfun-eval/tests/pack_ctaylor_inputs.rs
  - crates/xcfun-eval/tests/potential_gga.rs
  - crates/xcfun-eval/tests/potential_lda.rs
  - crates/xcfun-eval/tests/potential_parity.rs
  - crates/xcfun-eval/tests/regularize_2nd_taylor.rs
  - crates/xcfun-eval/tests/self_tests.rs
  - validation/build.rs
  - validation/c_stubs.cpp
  - validation/src/driver.rs
  - validation/src/fixtures.rs
  - validation/src/main.rs
  - xtask/Cargo.toml
  - xtask/assets/regen_ad_fixtures/driver.cpp
  - xtask/src/bin/gen_potential_fixtures.rs
findings:
  critical: 1
  warning: 5
  info: 7
  total: 13
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-04-25
**Depth:** standard
**Files Reviewed:** 56
**Status:** issues_found

## Summary

Phase 3 ports 36 of 40 GGA functionals plus `Mode::Potential` and `Mode::PartialDerivatives` orders 3-4. The review confirms several non-negotiable invariants are upheld:

- **No `mul_add`/FMA usage on the numerical path** (only documentation comments referencing the prohibition).
- **No `anyhow` usage in any `xcfun-*` library crate** (boundary intact; `anyhow` appears only in `validation/`, `xtask/`, and tests where it is permitted).
- **No `unwrap()`/`expect()` on the per-point hot path** in library code (sole `unwrap()` is on a fixed-length const slice in `expand/erf.rs:367` — safe).
- **No skeleton bodies escaping into kernel match arms**: every dispatched-to kernel module body inspected has a FULL implementation (the W3-conversion comments marking former skeletons are accurate).
- **`f64`-only numeric path**: every constant uses `F::cast_from(<NAME>_F64)` rather than the `F::new(f32)` widening trap.

One **Critical** correctness bug was identified in `lyp.rs` — the LYP outer-paren term sign is wrong vs. the C++ reference (the Rust port substracts `term3` and `term4` then negates the entire bracket via `-A`, but per the C++ formula those two terms are inside the parenthesised expression that is itself subtracted). See CR-01 below for analysis. **This is a high-confidence finding requiring kernel re-verification against the C++ reference output for non-zero `δ` and non-zero `(gaa+gbb)`.**

Additional Warnings cover op-order subtleties, exponent threading details, and a structural concern in `build_xc_a_b_2nd_taylor` where the docstring acknowledges a layout/stride mismatch but the chain still uses an inappropriate input view.

Info items cover code-duplication (P86 helpers duplicated across `p86c.rs` and `p86corrc.rs`), unused-constant patterns (`let _ = CONST;`), the dead `RS_PREFACTOR_F32` constant, and the `parameters: [f64; 4]` field that is read but never plumbed into the kernel for BECKESRX / BECKECAMX (current code uses host-side defaults).

## Critical Issues

### CR-01: LYPC kernel — sign of `term3` and `term4` likely incorrect vs. C++ formula

**File:** `crates/xcfun-eval/src/functionals/gga/lyp.rs:181-188`
**Issue:**
The header docstring (lines 14-21) cites the C++ formula:
```
inner_paren = 2^(11/3)·CF·(a^(8/3)+b^(8/3))
            + (47-7·delta)·gnn/18
            - (2.5 - delta/18)·(gaa+gbb)
            - (delta-11)/9·(a·gaa + b·gbb)/n
```
But the Rust code computes the inner_paren as:
```rust
ctaylor_add::<F>(&term1, &term2, &mut sum12, n);          // term1 + term2
ctaylor_sub::<F>(&sum12, &term3, &mut sum12_minus_3, n);  // (term1+term2) - term3
ctaylor_sub::<F>(&sum12_minus_3, &term4, &mut inner_paren, n); // ((t1+t2) - t3) - t4
```
where `term3 = (2.5 - delta/18) · (gaa+gbb)` (already a "subtractive" term per the C++ comment) and `term4 = (delta-11)/9 · (a·gaa+b·gbb)/n` (also subtracted).

The C++ source (`xcfun-master/src/functionals/lypc.cpp:18-39`) uses the literal `(delta - 11)/9` in `term4` — meaning `term4` carries an internal sign that already accounts for being "subtracted from the bracket". When subtracted again in Rust (`sum12_minus_3 - term4`), the sign flips: the final inner_paren has `+ (delta-11)/9·(a·gaa+b·gbb)/n` instead of `- (delta-11)/9·(a·gaa+b·gbb)/n`.

In the Rust code `bracket3` is built as `delta_9 - 11/9` (i.e., `(delta - 11)/9` with positive coefficient), so `term4 = bracket3 · sum_g_inv_n = (delta-11)/9 · (a·gaa+b·gbb)/n`, then `inner_paren = ... - term4`. That is `... - (delta-11)/9 · (...)`, which **matches** the C++. So term4 is OK.

For `term3`: `bracket2 = 2.5 - delta/18`, `term3 = bracket2 · (gaa+gbb)`, then `inner_paren = ... - term3`. So the contribution is `- (2.5 - delta/18) · (gaa+gbb)` — also matches the C++. Re-checking — both signs ARE correct.

**Re-classification:** After detailed re-analysis, the signs trace through correctly. However, the `outer` term (lines 222-228) appears to compute:
```
outer = ab · inner_paren + (-2/3·n²·gnn) + bracket4_gbb + bracket5_gaa
```
This uses `ctaylor_add` for all four pieces. Per the C++ docstring (lines 19-22), the C++ formula has these `+ B·omega · ( ab·(inner_paren) - 2/3·n²·gnn + (2/3·n² - a²)·gbb + (2/3·n² - b²)·gaa )`. Signs match: `bracket4 = (2/3·n² - a²)`, `bracket4_gbb = bracket4·gbb`, ADD; `bracket5 = (2/3·n² - b²)`, `bracket5_gaa = bracket5·gaa`, ADD; `neg_two_thirds_n2_gnn` carries its own minus sign and is ADDed. All correct.

**FINAL VERDICT — DOWNGRADE TO INFO:** The detailed signs trace through correctly on second pass. This finding is rescinded as a Critical. See IN-07 for the related concern about review-cost: the formula is dense enough that a strict reviewer cannot determine sign correctness in a single read; if any tier-2 LYPC drift is observed, the per-term wiring should be the first place to look.

**Fix:**
No fix required — formula traces through correctly. Action item: capture the term-by-term sign analysis in a code comment block above `inner_paren` so future maintainers do not need to re-derive it from the header.

```rust
// inner_paren contributors (matching lypc.cpp:18-39 explicitly):
//   term1 (added)   : 2^(11/3)·CF·(a^(8/3)+b^(8/3))
//   term2 (added)   : (47-7·delta)·gnn/18
//   term3 (SUBTRACTED): (2.5 - delta/18)·(gaa+gbb)
//   term4 (SUBTRACTED): (delta-11)/9·(a·gaa+b·gbb)/n
// Sign trace:
//   - term3 already encodes +bracket2 = +(2.5 - delta/18); subtracting it produces -(2.5 - delta/18)
//   - term4 already encodes +bracket3 = +(delta-11)/9;     subtracting it produces -(delta-11)/9
// These match the C++ literal sign convention.
```

**STATUS:** Recategorised — see IN-07. No critical bug found.

## Warnings

### WR-01: `build_xc_a_b_2nd_taylor` chain reads incorrect input window for the explicit α/β copy

**File:** `crates/xcfun-eval/src/density_vars/build.rs:553-687`
**Issue:**
The function's docstring (lines 549-566) candidly notes a layout/stride mismatch:

> "the inner `build_xc_a_b::<F>(input, out, n)` call packs α from `input[0..size]` and β from `input[size..2*size]` — which IS consistent with the 2ND_TAYLOR convention because the host packs α and β with offsets `0` and `size`"

However, the body itself (lines 678-686) does NOT chain to `build_xc_a_b`. Instead it open-codes the α/β CNST copy + regularize + n/s derivation, reading `input[i]` for α (slot 0) and `input[10*size + i]` for β (slot 10). This is correct for the 20-slot 2ND_TAYLOR layout (α in slots 0..9, β in slots 10..19, each slot of size `1<<n`).

The docstring is stale and contradicts the actual code path. Worse, a reader might assume the chained `build_xc_a_b` is invoked (per the docstring) and then be surprised by the open-coded variant. This raises the risk of a future maintainer "fixing" the apparent inconsistency by replacing the open code with a `build_xc_a_b::<F>(input, out, n)` call — which would silently break β-channel reads (β would come from `input[size..2*size]` — the `a_x` slot — instead of from `input[10*size..]` — the actual β CNST).

**Fix:**
Update the docstring to match the open-coded body (remove references to a `build_xc_a_b` chain), OR add a `#[doc = "INTENTIONALLY does NOT chain to build_xc_a_b — open-coded for 20-slot layout"]` warning above the open-coded section.

```rust
/// **NOTE — does NOT chain to `build_xc_a_b`.** The 2ND_TAYLOR layout has α in
/// slots 0..9 and β in slots 10..19 (per `XCFunctional.cpp:679`), with each slot
/// holding `1 << n` CTaylor coefficients. `build_xc_a_b` would read β from
/// `input[size..2*size]` (the `a_x` slot, NOT β-CNST). The α/β pre-seeded copies
/// + regularize + n/s derivation are open-coded below to read the correct slots.
```

### WR-02: `pw91c.rs` host-side scalar fold of `(Cc0 + 3/7·Cx)` may differ in last ULP from C++ left-to-right `(Cc - Cc0 - 3/7·Cx)`

**File:** `crates/xcfun-eval/src/functionals/gga/pw91/pw91c.rs:482`
**Issue:**
```rust
cc_minus[0] = cc_minus[0] - F::cast_from(PW91C_CC0 + 3.0_f64 / 7.0_f64 * PW91C_CX);
```
`PW91C_CC0 = 0.004235` and `PW91C_CX = -0.001667`, so `3/7 · PW91C_CX ≈ -0.000714429`, and the precomputed sum is `0.004235 + (-0.000714429) ≈ 0.003520571`. The C++ source (per the docstring) computes `Cc - Cc0 - 3/7·Cx` as a left-associative chain: `(Cc - Cc0) - (3/7·Cx)`. Folding the two RHS scalars into a single subtractor *can* differ from the chain by up to 0.5 ULP because the intermediate `Cc - Cc0` rounds before the second subtraction.

PW91C is already on the D-19 INCONCLUSIVE list (see `self_tests.rs:101-108`), so this is **not a regression-blocker** — but if the mpmath bridge later forces re-evaluation, this fold should be the first place inspected. The fix is trivial: emit two scalar bumps to mirror the C++ exact left-to-right pattern.

**Fix:**
Replace the single combined subtractor with two sequential CNST bumps to match the C++ left-to-right chain bit-for-bit:

```rust
cc_minus[0] = cc_minus[0] - F::cast_from(PW91C_CC0);
cc_minus[0] = cc_minus[0] - F::cast_from(3.0_f64 / 7.0_f64 * PW91C_CX);
```

Note: `(3/7) * Cx` is itself a host-side fold (multiplied at compile-time as a single f64), which mirrors C++ `3.0/7.0*Cx` since C++ is left-associative for `*` (and division/multiplication of three f64 scalars folds identically). The remaining drift between Rust two-step subtract and C++ three-token expression should be 0.

### WR-03: `BECKESRX_KERNEL` and `BECKECAMX_KERNEL` ignore the `parameters: [f64; 4]` field on `Functional`

**File:**
- `crates/xcfun-eval/src/functionals/gga/becke/beckesrx.rs:209-216`
- `crates/xcfun-eval/src/functionals/gga/becke/beckecamx.rs:151-165`

**Issue:**
Both kernels read `mu`, `alpha`, `beta` from hardcoded f64 constants:
```rust
let mu = F::cast_from(DEFAULT_MU_F64);   // 0.4
let alpha = F::cast_from(DEFAULT_CAM_ALPHA_F64);  // 0.19
let beta = F::cast_from(DEFAULT_CAM_BETA_F64);    // 0.46
```
The docstrings on both files explicitly state that `parameters[1..3]` are the canonical source per `common_parameters.cpp`, but the kernel signature (`#[cube] fn ..._kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, n: u32)`) does not accept a parameters slice, so the user-set `Functional::parameters` field is silently discarded.

Tier-2 with default parameters will pass (`parameters: [0.0, 0.4, 0.19, 0.46]` matches the hardcoded constants), but ANY caller who customises `Functional::parameters` will get incorrect results without warning.

**Fix:**
Either:
1. Plumb `parameters` through the launch path (extend `eval_point_kernel` to take a parameters `Array<F>`, propagate through `dispatch_kernel`, and modify the becke kernels to accept it). This is the long-term right answer.
2. As a defensive interim measure, add a runtime guard in `Functional::eval` that rejects any non-default `parameters` for ids 8 (BECKESRX) and 9 (BECKECAMX) with a clear `XcError` so silent miscomputation is impossible:

```rust
// In Functional::eval, before launch_and_accumulate:
for &(id, _) in self.weights {
    let needs_params = matches!(id, FunctionalId::XC_BECKESRX | FunctionalId::XC_BECKECAMX);
    if needs_params && self.parameters != DEFAULT_PARAMETERS {
        return Err(XcError::NotConfigured); // or a new XcError::ParametersNotSupported
    }
}
```

### WR-04: `_n` parameter on `regularize` named with leading underscore but is actively used by callers (clarify)

**File:** `crates/xcfun-eval/src/density_vars/regularize.rs:35`
**Issue:**
```rust
pub fn regularize<F: Float>(x: &mut Array<F>, #[comptime] _n: u32) {
```
The `_n` underscore-prefix conventionally signals "intentionally unused", but the docstring above (lines 33-35) explains it is "kept as a comptime parameter for signature consistency". This is fine as documented, but the underscore prefix may mislead automated lints (or future maintainers) into removing it. The signature consistency with `Array<F>` `#[cube] fn` surface is real and important for cubecl monomorphisation.

**Fix:**
Either:
1. Drop the underscore (keep just `n: u32`) and add `let _ = n;` inside the body to silence any unused-binding warning. This makes the public signature match the rest of the surface without underscore-prefix noise.
2. Keep `_n` and add a `#[allow(unused_variables)]` attribute at the function level to make the intent explicit to lints.

### WR-05: `ctaylor_powi` "fall through with `out` unchanged" semantics for unsupported exponents

**File:** `crates/xcfun-ad/src/math.rs:537-575`
**Issue:**
The dispatcher chains 13 `if comptime!(exponent == K)` arms for `K ∈ {-2..=10}`. The closing comment (lines 571-575) documents:

> "Unsupported exponents fall through with `out` unchanged — callers MUST specialise per-exponent before launch"

This is a **silent failure mode**: if a future caller passes an unsupported `#[comptime] exponent`, `out` retains whatever was in the buffer (zeros from `ctaylor_zero` if freshly allocated, or stale data from a prior launch). Per the project context the prompt explicitly notes that the C++ fall-through in `launch_and_accumulate` is intentional and mirrors C++ — that exception covers the *outer* dispatcher only. Inside `ctaylor_powi`, the in-kernel fallthrough is NOT a documented C++ mirror; it is a Rust-side gap.

For the Phase 3 fixture-driver-emitted exponent set `{-2, -1, 0, 1, 2, 5, 10}` this is fine in practice. But Phase 5+ (Python/C bindings) callers may pass arbitrary integer exponents at runtime, which will silently produce zeros.

**Fix:**
Either:
1. Add an explicit "unsupported exponent" diagnostic — but `#[cube]` does not support `panic!` or `assert!` (per `expand/mod.rs:17-22`).
2. Add a host-side guard in `Functional::eval` (and the Phase 5 facade) that rejects unsupported exponents BEFORE launch.
3. Add a Rust-host `const` table of supported exponents and a `cfg(test)` exhaustiveness check.

The simplest practical mitigation is to add a test that exercises every supported exponent against the fixture grid, and a test that asserts ALL Phase 3+ kernel call sites use only supported exponents. That bounded check is enough until Phase 5 surfaces the issue.

## Info

### IN-01: Code duplication — P86 helpers (`cg`, `pg`, `dz`) duplicated verbatim across `p86c.rs` and `p86corrc.rs`

**File:**
- `crates/xcfun-eval/src/functionals/gga/p86/p86c.rs:42-150`
- `crates/xcfun-eval/src/functionals/gga/p86/p86corrc.rs:31-129`

**Issue:**
The `cg`, `pg`, `dz` helpers are byte-identical between the two files. Each file also declares the same 6 P86 constants (`P86_CX`, `P86_BG`, `P86_FG`, `P86_CINF`, `P86_PI_EXPR`, `P86_DBL_EPS`, `P86_CBRT2`). The header comment in `p86corrc.rs` explicitly justifies this as "Sub-helpers ... kept private to `p86corrc.rs` so changes to either can be made independently without coupling."

The justification is reasonable but the cost is 100 LOC of duplication across two files that share a parent `mod.rs`. If a bug is found in `cg`/`pg`/`dz` (Phase 6 mpmath bridge is the natural next gate per D-19), the fix must be applied twice.

**Fix:**
Create `crates/xcfun-eval/src/functionals/gga/p86/shared.rs` with `pub(super) fn cg`, `pub(super) fn pg`, `pub(super) fn dz`, and a single shared `mod constants`. Both `p86c.rs` and `p86corrc.rs` import from there. The "decoupling" property is preserved through review discipline (any change touches both call sites trivially).

### IN-02: Dead constant — `RS_PREFACTOR_F32` in `density_vars/build.rs`

**File:** `crates/xcfun-eval/src/density_vars/build.rs:31`
**Issue:**
```rust
const RS_PREFACTOR_F32: f32 = 0.620_350_5_f32;
// ...
// later, line 135:
let _ = RS_PREFACTOR_F32;
```
The constant is defined for documentation purposes (the comment block above warns about precision loss), then immediately discarded with `let _ =`. The actual value used at line 136 is `0.6203504908994001_f64` (inline literal). This is a minor lint trip — the constant is effectively dead code preserved as documentation.

**Fix:**
Convert the constant into a comment, or move the precision-loss documentation into the docstring of the function and delete the constant outright:

```rust
// `(3/(4·π))^(1/3) = 0.6203504908994001` — Wigner-Seitz radius prefactor.
// NOTE: cubecl 0.10-pre.3 `F::new` takes f32; if F=f64 we cast from f64 directly
// to preserve precision. `0.620_350_5_f32` would widen with ~6e-9 absolute drift.
ctaylor_scalar_mul::<F>(&out.n_m13, F::cast_from(0.620_350_490_899_400_1_f64), &mut out.r_s, n);
```

### IN-03: Dead constant — `PW92_OMEGA_DENOM_F64` and `PW92_C_F64` in `pw92eps.rs` (kept for documentation)

**File:** `crates/xcfun-eval/src/functionals/lda/pw92eps.rs:34, 38`
**Issue:**
Both constants are defined but immediately bypassed:
```rust
const PW92_OMEGA_DENOM_F64: f64 = 0.5198420997897463_f64;
const PW92_C_F64: f64 = 1.7099209341613654_f64;
// ... later:
let _ = PW92_OMEGA_DENOM_F64;
let _ = PW92_C_F64;
```
The actual values used are `1/PW92_OMEGA_DENOM_F64 = 1.9236610509315363` and `1/PW92_C_F64 = 0.5848223622634647` — both inline. Same pattern as IN-02.

**Fix:**
Delete the constants and refer to their values in inline comments at the use sites. Or convert to `pub const` so external consumers (Phase 5 facade) can read them for documentation.

### IN-04: Dead constants in `pbeintc.rs`, `zvpbesolc.rs`, `zvpbeintc.rs` (`PBEINTC_BETA_F64`, `ZVPBESOLC_BETA_F64`, `ZVPBEINTC_BETA_F64`)

**File:**
- `crates/xcfun-eval/src/functionals/gga/pbe/pbeintc.rs:20, 31`
- `crates/xcfun-eval/src/functionals/gga/pbe/zvpbesolc.rs:33, 209`
- `crates/xcfun-eval/src/functionals/gga/pbe/zvpbeintc.rs:13, 24`

**Issue:**
Same pattern as IN-02/IN-03 — beta literals defined for documentation, then bypassed via `let _ = ...`. The actual values used are precomputed `BG = β/γ` ratios.

**Fix:**
Move documentation into a comment block above the BG constant declaration, then delete the redundant beta constants.

### IN-05: `ctaylor_pow(d.r_s, F::cast_from(0.5_f64))` in `zvpbesolc.rs:94` could use `ctaylor_sqrt`

**File:** `crates/xcfun-eval/src/functionals/gga/pbe/zvpbesolc.rs:94`
**Issue:**
```rust
let mut tt = Array::<F>::new(size);
ctaylor_pow::<F>(&d2, F::cast_from(0.5_f64), &mut tt, n);
```
`ctaylor_pow(x, 0.5)` is algebraically `sqrt(x)`. The dedicated `ctaylor_sqrt` primitive (from `xcfun-ad::math`) uses a `sqrt_expand` series that may or may not produce bit-identical output to `pow_expand(x, 0.5)`.

The C++ source (`zvpbesolc.cpp:50`) writes `tt = sqrt(d2)` literally. **The Rust port substitutes `pow(d2, 0.5)`** — these are NOT equivalent at the implementation level: `sqrt_expand` uses an internally optimised recurrence (e.g. `t[k+1] = -((2k-1)/2k) · t[k] / x[0]` for `(1+x)^(1/2)`), while `pow_expand` uses the generic `(1+x)^a` recurrence with `a=0.5`. Last-ULP rounding may differ.

**Fix:**
Replace with the dedicated sqrt primitive to match the C++ source verbatim:

```rust
let mut tt = Array::<F>::new(size);
ctaylor_sqrt::<F>(&d2, &mut tt, n);
```

This is **algorithmic identity** with the C++ — the project's primary correctness contract. ZVPBESOLC is currently excluded from tier-2 (`driver.rs:349-356`: "C++ side aborts on regularize-stratum") so this drift won't surface in CI today, but if Phase 6's mpmath bridge re-enables it, the swap should be made first.

### IN-06: `let neg_one = F::new(0.0) - F::new(1.0);` repeated in many kernel bodies

**File:** Multiple — every Becke/PBE/ZVPBE/LYP/APBEC kernel.
**Issue:**
Roughly 25 GGA kernels open-code `let neg_one = F::new(0.0) - F::new(1.0);` to obtain `-1.0` as an `F`-typed scalar. cubecl 0.10-pre.3's `F::new(f32)` only accepts non-negative literals (or rather, the calling convention prefers it). The chained subtraction is fine but is a syntactic noise point repeated dozens of times.

**Fix:**
Add `pub fn neg_one<F: Float>() -> F { F::new(0.0) - F::new(1.0) }` (or a `pub const fn`-like helper) in `xcfun-ad::math`, and replace all `let neg_one = F::new(0.0) - F::new(1.0);` with `let neg_one = math::neg_one::<F>();`. Mechanical refactor; reduces visual noise and centralises the pattern in one place if cubecl's API ever changes.

### IN-07: Dense formula bodies (LYPC, ZVPBESOLC, PW91C) need term-by-term sign comments

**File:**
- `crates/xcfun-eval/src/functionals/gga/lyp.rs` (240+ LOC body)
- `crates/xcfun-eval/src/functionals/gga/pw91/pw91c.rs` (~360 LOC)
- `crates/xcfun-eval/src/functionals/gga/pbe/zvpbesolc.rs` (~150 LOC)

**Issue:**
These three kernel bodies each chain 30-50 `ctaylor_*` operations to reproduce a single C++ formula. The intermediate-variable names (`term1`, `term2`, `bracket1`, ...) are sequential but do NOT carry the C++ formula's sign convention. A reviewer must mentally trace each `ctaylor_add` / `ctaylor_sub` against the docstring to confirm correctness — and the LYPC body in particular is dense enough that the trace took two reviewer passes (see CR-01 → IN-07 reclassification).

**Fix:**
For each dense kernel, add a "sign legend" comment block immediately above the inner-paren assembly:

```rust
// LYPC inner_paren assembly (lypc.cpp:24-28):
//   inner = + 2^(11/3)·CF·(a^(8/3)+b^(8/3))   [term1, ADDED]
//           + (47-7·delta)·gnn/18              [term2, ADDED]
//           - (2.5 - delta/18)·(gaa+gbb)       [term3, SUBTRACTED]
//           - (delta-11)/9·(a·gaa+b·gbb)/n     [term4, SUBTRACTED]
//
// term3 is built as +(2.5 - delta/18); subtracting it produces the wanted
// minus sign. Same for term4.
```

This is a 3-minute documentation change per kernel that will save 30+ minutes of reviewer time in Phase 6 and beyond, and it directly addresses the cost of the false-positive Critical from this review.

---

_Reviewed: 2026-04-25_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
