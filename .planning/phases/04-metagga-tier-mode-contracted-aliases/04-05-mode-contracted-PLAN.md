---
phase: 04-metagga-tier-mode-contracted-aliases
plan_number: "05"
type: execute
wave: 5
depends_on:
  - "04-04"
requirements:
  - MODE-03
files_modified:
  - crates/xcfun-eval/src/functionals/contracted.rs
  - crates/xcfun-eval/src/functional.rs
  - crates/xcfun-eval/src/functionals/mod.rs
  - crates/xcfun-eval/tests/contracted_cross_mode.rs
  - validation/src/c_driver.rs
autonomous: true
created: "2026-04-25"
goal: "Wave 5 — Mode::Contracted host-side DOEVAL dispatch (orders 0..=6), output_length extension, tier-2 cross-mode parity at strict 1e-12 for orders 0..=4; orders 5/6 via new C++ harness path"

must_haves:
  truths:
    - "Functional::eval with mode=Contracted at order=0 produces 1 output matching C++ DOEVAL expansion"
    - "Functional::eval with mode=Contracted at order=6 produces 64 outputs matching C++ DOEVAL"
    - "output_length returns 1<<order for Mode::Contracted at orders 0..=6"
    - "eval_setup returns XcError::InvalidOrder for order > 6 in Contracted mode"
    - "contracted_cross_mode parity test: Contracted at orders 0..=4 matches PartialDerivatives Taylor coefficients at strict 1e-12 on 1000 points"
    - "Contracted at orders 5/6 cross-checked against C++ xcfun_eval(XC_CONTRACTED) on 100-point subset at strict 1e-12"
  artifacts:
    - path: crates/xcfun-eval/src/functionals/contracted.rs
      provides: launch_contracted host-side dispatcher for orders 0..=6
      exports: [launch_contracted]
    - path: crates/xcfun-eval/tests/contracted_cross_mode.rs
      provides: Contracted vs PartialDerivatives cross-mode parity test
      contains: "cross_mode"
    - path: validation/src/c_driver.rs
      provides: C++ harness extension for Contracted mode at orders 5/6
      contains: "XC_CONTRACTED"
  key_links:
    - from: crates/xcfun-eval/src/functional.rs
      to: crates/xcfun-eval/src/functionals/contracted.rs
      via: eval mode=Contracted match arm calling launch_contracted
      pattern: "launch_contracted"
    - from: validation/src/c_driver.rs
      to: xcfun-master C++ library
      via: xcfun_set_mode(XC_CONTRACTED) + xcfun_eval at order 5/6
      pattern: "XC_CONTRACTED"
---

<objective>
Implement `Mode::Contracted` as a host-side DOEVAL dispatch layer: for each order N in 0..=6, pack inputs as `inlen × (1 << N)` flat doubles, invoke the same per-functional `#[cube] fn <name>_kernel<F, const N>` body, unpack `(1 << N)` flat output coefficients. This is a re-packaging of existing kernels — zero per-functional kernel changes needed for already-shipped LDA/GGA/metaGGA functionals.

RESEARCH KEY FINDING: Mode::Contracted at orders 0..=4 is algorithmically equivalent to PartialDerivatives (same kernel, different I/O packing). The cross-mode parity test at orders 0..=4 is a structural smoke test that MUST pass at strict 1e-12 for any functional. Orders 5/6 require a new C++ harness path (the vendored xcfun-master has no test files for Contracted at orders 5/6 per RESEARCH).

Note: This plan depends on Plan 04-04 (alias + parameter engine must be wired before Contracted mode validation runs against the full functional set).

Output: 2 new files, 2 files modified, C++ harness extended for orders 5/6.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md

<interfaces>
<!-- From Phase 3 Mode::Potential analog. -->

From crates/xcfun-eval/src/functionals/potential.rs (Mode::Potential analog — launch_potential pattern):
```rust
pub fn launch_potential(
    client: &ComputeClient<CpuServer, CpuChannel>,
    functional: &Functional,
    input: &[f64],
    output: &mut [f64],
) -> Result<(), XcError> {
    // mode-specific host-side pack/unpack over per-functional kernel
}
```

From xcfun-master/src/XCFunctional.cpp:619-635 (verbatim port target):
```cpp
} else if (fun->mode == XC_CONTRACTED) {
#define DOEVAL(N, E)                                              \
  if (fun->order == N) {                                          \
    typedef ctaylor<ireal_t, N> ttype;                            \
    int inlen = xcint_vars[fun->vars].len;                        \
    ttype in[XC_MAX_INVARS], out = 0;                             \
    int k = 0;                                                    \
    for (int i = 0; i < inlen; i++)                               \
      for (int j = 0; j < (1 << fun->order); j++)                 \
        in[i].set(j, input[k++]);                                 \
    densvars<ttype> d(fun, in);                                   \
    for (int i = 0; i < fun->nr_active_functionals; i++)          \
      out += fun->settings[fun->active_functionals[i]->id] *      \
             fun->active_functionals[i]->fp##N(d);                \
    for (int i = 0; i < (1 << fun->order); i++)                   \
      output[i] = out.get(i);                                     \
  } else                                                          \
  FOR_EACH(XCFUN_MAX_ORDER, DOEVAL, )                             \
  xcfun::die("bug! Order too high in XC_CONTRACTED", fun->order);
}
```

From crates/xcfun-eval/src/functional.rs (eval mode-dispatch pattern):
```rust
pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
    let client = cpu_client();
    match self.mode {
        Mode::PartialDerivatives => { ... }
        Mode::Potential => launch_potential(&client, self, input, output),
        Mode::Contracted => { /* PLACEHOLDER — Plan 04-05 fills this */ }
        _ => Err(XcError::InvalidMode),
    }
}
```
</interfaces>
</context>

<tasks>

<task id="5.1" type="auto">
  <name>Task 1: contracted.rs host-side DOEVAL dispatch + functional.rs Mode::Contracted wiring</name>
  <files>
    crates/xcfun-eval/src/functionals/contracted.rs,
    crates/xcfun-eval/src/functional.rs,
    crates/xcfun-eval/src/functionals/mod.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/src/functionals/potential.rs` — READ FULLY. `launch_potential` is the structural analog for `launch_contracted`. Note how it packs input, iterates active functionals, reads output. `contracted.rs` follows the same host-side shape.
    - `crates/xcfun-eval/src/functional.rs` — READ FULLY. Find the `eval` method's mode-dispatch match arm. Also find `eval_setup` for order validation. Also find `output_length` to add the Contracted arm.
    - `xcfun-master/src/XCFunctional.cpp` lines 482-490 — output_length for Contracted (C++ calls `xcfun::die`; Rust returns `Ok(1 << order)` per D-06-B).
    - `xcfun-master/src/XCFunctional.cpp` lines 619-635 — DOEVAL macro (verbatim port target per D-06).
    - `xcfun-master/src/xcint.hpp` line 28 — `XC_MAX_INVARS = 20` (input array bound).
    - `crates/xcfun-eval/src/dispatch.rs` — `dispatch_kernel` signature (called per active functional in launch_contracted).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-06, D-06-A, D-06-B, D-06-C, D-07 (CTaylor<F,6> already valid), D-12 (no Contracted relaxation)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md` — §"Mode::Contracted Implementation" (input layout, output layout, stack budget, order 5/6 harness requirement)
  </read_first>
  <action>
    **1. `crates/xcfun-eval/src/functionals/contracted.rs` (NEW):**

    Port the DOEVAL macro as 7 comptime arms in a `launch_contracted` function:

    ```rust
    //! Mode::Contracted host-side dispatcher — port of XCFunctional.cpp:619-635 DOEVAL macro.
    //!
    //! Input layout: `inlen × (1 << order)` flat f64 doubles, inner-coefficient-major:
    //!   input[0..1<<N]         → in[0].coeff[0..(1<<N)]  (first Vars element, all N Taylor coeffs)
    //!   input[1<<N..2*(1<<N)]  → in[1].coeff[0..(1<<N)]  (second Vars element)
    //!   ...
    //! Output layout: `(1 << order)` flat f64 doubles.
    //!   output[i] = out.get(i) for i in 0..(1 << order)
    //!
    //! Per-functional kernel re-used from PartialDerivatives — no kernel changes needed.

    use cubecl::prelude::*;
    use cubecl_cpu::CpuRuntime;
    use xcfun_core::{Mode, Vars, XcError, VARS_TABLE};
    use crate::functional::Functional;
    use crate::dispatch::dispatch_kernel;
    use crate::density_vars::build::build_densvars;
    use crate::for_tests::cpu_client;

    /// Launch Mode::Contracted evaluation for one point.
    ///
    /// Dispatches to one of 7 comptime order arms (0..=6).
    /// Each arm: reads inlen × (1<<order) input doubles → builds CTaylor<f64, order> array
    /// via build_densvars → accumulates weighted kernel outputs → writes (1<<order) output doubles.
    pub fn launch_contracted(
        functional: &Functional,
        input: &[f64],
        output: &mut [f64],
    ) -> Result<(), XcError> {
        let order = functional.order;
        let vars = functional.vars;
        let inlen = VARS_TABLE[vars as usize].len as usize;
        let coeff_count = 1_usize << order;

        // Validate input/output lengths per D-06-A
        if input.len() < inlen * coeff_count {
            return Err(XcError::InputLengthMismatch);
        }
        if output.len() < coeff_count {
            return Err(XcError::InputLengthMismatch);  // or OutputLengthMismatch if that variant exists
        }

        match order {
            0 => launch_contracted_n::<0>(functional, input, output, inlen),
            1 => launch_contracted_n::<1>(functional, input, output, inlen),
            2 => launch_contracted_n::<2>(functional, input, output, inlen),
            3 => launch_contracted_n::<3>(functional, input, output, inlen),
            4 => launch_contracted_n::<4>(functional, input, output, inlen),
            5 => launch_contracted_n::<5>(functional, input, output, inlen),
            6 => launch_contracted_n::<6>(functional, input, output, inlen),
            _ => Err(XcError::InvalidOrder),
        }
    }

    fn launch_contracted_n<const ORDER: u32>(
        functional: &Functional,
        input: &[f64],
        output: &mut [f64],
        inlen: usize,
    ) -> Result<(), XcError> {
        // Port of DOEVAL(N, E) body:
        // 1. Allocate in[XC_MAX_INVARS] of CTaylor<f64, ORDER> — each has (1<<ORDER) coefficients.
        //    Read: for each Vars-element i, for each coefficient j in 0..(1<<ORDER),
        //          in[i].coeff[j] = input[i * (1<<ORDER) + j].
        // 2. Build DensVarsDev via build_densvars(in, vars, ORDER) — same as PartialDerivatives.
        // 3. For each active functional: out += settings[id] * kernel(d, ORDER).
        // 4. Write: output[j] = out.coeff[j] for j in 0..(1<<ORDER).
        //
        // Cubecl execution: the kernel reads a pre-seeded CTaylor input array and calls
        // build_densvars + dispatch_kernel. The host-side packs the CTaylor seeds into a
        // flat [f64; inlen * (1<<ORDER)] array before launch.
        //
        // Implementation note: this is structurally identical to the PartialDerivatives
        // launch loop but with N=ORDER fixed for all inlen Vars-elements in one shot,
        // instead of the multi-launch (i,j) pair iteration used for PartialDerivatives.
        // See RESEARCH §"Mode::Contracted Implementation: Input Layout" for the exact flat
        // read pattern.
        let client = cpu_client();
        // ... cubecl kernel launch at N=ORDER per the established eval_point_kernel adapter ...
        // Output: for j in 0..(1<<ORDER): output[j] = result.coeff[j].
        Ok(())
    }
    ```

    IMPORTANT implementation detail for the cubecl kernel: The existing `eval_point_kernel` adapter in `functional.rs` monomorphizes over `(id, vars, n)` where `n` is the CTaylor order. For Mode::Contracted, `n = ORDER` for all inlen Vars-elements — a single kernel launch covers all elements at once (unlike PartialDerivatives which iterates `(i,j)` pairs). The contracted mode kernel adapter must pre-seed CTaylor coefficients from the flat input array, invoke `build_densvars`, invoke `dispatch_kernel` for each active functional with its weight, and accumulate into `out`. This is the `DOEVAL` macro logic moved into the cubecl kernel body.

    Read `potential.rs` carefully for how it sequences the cubecl launch. `contracted.rs` follows the same pattern but with a different kernel monomorphization dimension.

    **2. `crates/xcfun-eval/src/functional.rs` — three changes:**

    a) `eval()` mode dispatch: wire `Mode::Contracted` arm:
    ```rust
    Mode::Contracted => contracted::launch_contracted(self, input, output),
    ```

    b) `eval_setup()`: add Contracted order validation. Existing `eval_setup` validates order for PartialDerivatives (0..=4). Add: for `Mode::Contracted`, accept `order` in `0..=6`; reject `order > 6` with `XcError::InvalidOrder` per D-06.

    c) `output_length()`: add Contracted arm:
    ```rust
    Mode::Contracted => Ok(1_usize << self.order),  // D-06-B
    ```
    (Previously may have returned `Err(XcError::InvalidMode)` or panicked.)

    **3. `crates/xcfun-eval/src/functionals/mod.rs`:** add `pub mod contracted;`
  </action>
  <acceptance_criteria>
    1. `cargo build -p xcfun-eval --release` exits 0.
    2. `grep -n "Mode::Contracted" crates/xcfun-eval/src/functional.rs` returns at least 3 matches (eval dispatch, eval_setup, output_length).
    3. `grep -n "launch_contracted" crates/xcfun-eval/src/functional.rs` returns a match (wired).
    4. `grep -n "1_usize << self.order" crates/xcfun-eval/src/functional.rs` returns a match in output_length.
    5. `grep -n "XcError::InvalidOrder" crates/xcfun-eval/src/functional.rs` returns a match in eval_setup for Contracted order > 6 case.
    6. Smoke test: manually launch Contracted mode at order=2 for SLATERX on a trivial input; output length is 4 (1 << 2 = 4). Can be verified by a quick `cargo test -p xcfun-eval test_contracted_output_len --features testing`.
  </acceptance_criteria>
  <done>contracted.rs compiles. Mode::Contracted wired in eval/eval_setup/output_length. Build GREEN.</done>
</task>

<task id="5.2" type="auto">
  <name>Task 2: Contracted cross-mode parity test (orders 0..=4) + C++ harness extension (orders 5/6)</name>
  <files>
    crates/xcfun-eval/tests/contracted_cross_mode.rs,
    validation/src/c_driver.rs
  </files>
  <read_first>
    - `crates/xcfun-eval/tests/potential_parity.rs` — READ FULLY (Phase 3 Mode::Potential parity test analog for contracted_cross_mode.rs).
    - `crates/xcfun-eval/src/functionals/contracted.rs` — the dispatcher implemented in Task 1.
    - `validation/src/c_driver.rs` — READ FULLY. Understand how it calls the C++ xcfun library (xcfun_new, xcfun_set, xcfun_set_mode, xcfun_set_order, xcfun_eval, xcfun_delete). Must extend to call `xcfun_set_mode(fun, XC_CONTRACTED)` + `xcfun_set_order(fun, 5)` or `6`.
    - `xcfun-master/src/XCFunctional.cpp` lines 619-635 — confirms that Mode::Contracted with `xcfun_eval` at order 5/6 is a valid C++ call path (no die() in the DOEVAL expansion for orders 0..=6).
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-CONTEXT.md` — D-06-C (tier-2 Contracted parity, 1000-point subset at strict 1e-12), D-12 (no relaxation)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-RESEARCH.md` — §"C++ tests reaching order 5/6" (no upstream tests exist; RESEARCH §"Mode::Contracted Implementation" §"Vars compatibility" for the implication)
    - `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-VALIDATION.md` — Tier-2 Mode::Contracted cross-mode test spec
  </read_first>
  <action>
    **1. `crates/xcfun-eval/tests/contracted_cross_mode.rs` (NEW):**

    Cross-mode parity test at orders 0..=4: for each order N, and for a subset of 100 density grid points (from the canonical 10k grid at seed 0x1234abcd), run both `Mode::Contracted` and `Mode::PartialDerivatives` and verify that the first `(1 << N)` outputs match bit-for-bit (algorithmic identity: Contracted is a re-packaging of PartialDerivatives Taylor coefficients).

    At orders 0/1/2: the existing PartialDerivatives output already provides the coefficients. At order N for PartialDerivatives, output[k] corresponds to the multi-index Taylor coefficient for a specific variable combination. For Contracted at order N, output[i] = out.coeff[i] = the CTaylor bit-flag indexed coefficient. Verify these match numerically at strict 1e-12 for representative functionals (SLATERX, PBEX, TPSSX, M06X — one per tier).

    ```rust
    #![cfg(feature = "testing")]
    //! Cross-mode parity: Mode::Contracted vs Mode::PartialDerivatives at orders 0..=4.
    //!
    //! At orders 0..=4, Contracted mode is a re-packaging of PartialDerivatives:
    //! the underlying kernel is identical; the DOEVAL input pack / output unpack
    //! is the only difference. Strict 1e-12 for all representative functionals.

    #[test]
    fn contracted_vs_partial_order_0_slaterx() { ... }
    // ... tests for orders 1..=4 and functionals PBEX, TPSSX, M06X
    ```

    Feature-gate: `#![cfg(feature = "testing")]`. Follow `potential_parity.rs` structure exactly.

    **2. `validation/src/c_driver.rs` extension for orders 5/6:**

    Extend the C++ driver to call `xcfun_eval` in `XC_CONTRACTED` mode at orders 5 and 6 on a 100-point subset × 4 representative functionals. The driver already calls `xcfun_set_mode` and `xcfun_set_order`; add a new function or extend the existing `run_validation` to support `mode = XC_CONTRACTED` with `order in {5, 6}`.

    For orders 5/6: `inlen × (1 << 5)` = `inlen × 32` or `inlen × 64` doubles per point. For `XC_A_B` (inlen=2): 64 or 128 doubles per point at order 5/6. Generate synthetic pre-seeded CTaylor inputs (e.g., set CNST=density point value, VAR0=1, all others=0 for order=5; caller is responsible for meaning of higher-order coefficients).

    Diff Rust output vs C++ output at strict 1e-12. Write results to `validation/report-contracted-orders-5-6.jsonl`.

    CRITICAL: If any order-5/6 record fails strict 1e-12, create a D-19 INCONCLUSIVE entry. Do NOT widen tolerance. Per D-12, Contracted is a structural re-packaging — a failure here would indicate a DOEVAL pack/unpack bug, not a numerical precision issue.
  </action>
  <acceptance_criteria>
    1. `cargo test -p xcfun-eval --test contracted_cross_mode --features testing` passes — cross-mode parity at orders 0..=4 for SLATERX/PBEX/TPSSX/M06X at strict 1e-12 (4 functionals × 5 orders × 100 points = 2000 records).
    2. `grep -n "max_relative.*1e-12" crates/xcfun-eval/tests/contracted_cross_mode.rs` returns matches.
    3. `grep -n "XC_CONTRACTED" validation/src/c_driver.rs` returns a match (C++ harness extended).
    4. `cargo xtask validate --backend cpu --mode contracted --order 5 --filter 'slaterx,pbex,tpssx,m06x'` exits 0 OR produces a D-19 INCONCLUSIVE report at strict 1e-12.
    5. `grep -n "output_length" crates/xcfun-eval/src/functional.rs | grep "1_usize << self.order"` confirms D-06-B implementation.
  </acceptance_criteria>
  <done>contracted_cross_mode parity test GREEN. C++ harness extended for orders 5/6. output_length correct. eval_setup rejects order > 6.</done>
</task>

</tasks>

<verification>
```bash
# Build
cargo build -p xcfun-eval --release 2>&1 | tail -5

# Cross-mode parity test
cargo test -p xcfun-eval --test contracted_cross_mode --features testing 2>&1 | tail -20

# Structural checks
grep -n "Mode::Contracted" crates/xcfun-eval/src/functional.rs | head -10
grep -n "1_usize << self.order" crates/xcfun-eval/src/functional.rs
grep -n "XC_CONTRACTED" validation/src/c_driver.rs | head -5

# Validation for Contracted mode at order 5/6
cargo xtask validate --backend cpu --mode contracted --order 5 --filter 'slaterx,pbex,tpssx,m06x' 2>&1 | tail -10
```
</verification>

<success_criteria>
- `launch_contracted` dispatches 7 comptime order arms (0..=6).
- `output_length` returns `1 << order` for Contracted per D-06-B.
- `eval_setup` rejects order > 6 in Contracted mode.
- Cross-mode parity test: 2000 records at strict 1e-12 (or D-19 INCONCLUSIVE entries if any fail).
- C++ harness extended for orders 5/6 and committed to `validation/src/c_driver.rs`.
</success_criteria>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Functional::eval (Mode::Contracted) | Caller-supplied input/output slices with caller-computed length `inlen × (1 << order)`. Length validation in launch_contracted guards against undersized slices. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-04-05-01 | Denial of Service | Contracted mode at order=6 allocates 20 × 64 × 8 = 10 KB stack per kernel | accept | 10 KB per kernel invocation is within cubecl-cpu stack budget. Verified in RESEARCH §"Mode::Contracted Implementation: CTaylor<F,6> capacity check." |
| T-04-05-02 | Tampering | Undersized caller input slice → out-of-bounds read in DOEVAL flat read loop | mitigate | `launch_contracted` validates `input.len() >= inlen * (1 << order)` before dispatch. Returns `XcError::InputLengthMismatch` on violation. Phase 5 C ABI adds catch_unwind over all eval paths. |
| T-04-05-03 | Tampering | Undersized caller output slice → out-of-bounds write in DOEVAL flat write loop | mitigate | `launch_contracted` validates `output.len() >= 1 << order` before dispatch. Returns `XcError::InputLengthMismatch` on violation. |
| T-04-05-04 | Integer overflow | `1_usize << order` where order = 6: 64, within usize range | accept | Maximum shift is 6 (constant, not user-controlled beyond eval_setup validation). No overflow possible. |
</threat_model>

<output>
After completion, create `.planning/phases/04-metagga-tier-mode-contracted-aliases/04-05-SUMMARY.md`
</output>
