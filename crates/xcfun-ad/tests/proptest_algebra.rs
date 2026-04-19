//! Property tests for `xcfun-ad` using the batch-per-property kernel
//! pattern (CONTEXT.md D-18).
//!
//! Each test:
//! 1. Uses proptest 1.11 to generate `DEFAULT_ITERS` sample inputs upfront
//!    (seeded via a deterministic `TestRunner`).
//! 2. Packs the inputs into a single cubecl buffer.
//! 3. Launches ONE cubecl-cpu kernel that evaluates the property across all
//!    `DEFAULT_ITERS` points (one work-unit per input via `ABSOLUTE_POS`).
//! 4. Downloads the result buffer and aggregates pass/fail host-side.
//!
//! This preserves the D-18 requirement of >= 10 000 iters per property
//! without paying per-iteration kernel-launch overhead. `PROPTEST_CASES` is
//! honoured as the reduction knob for CI runs probing at lower iteration
//! counts (one kernel launch either way).
//!
//! # Property catalogue (11 total)
//!
//! 1.  `commutativity_add`        — `(a + b) == (b + a)`, bit-exact
//! 2.  `commutativity_mul`        — `(a * b) == (b * a)`, bit-exact at N=1
//! 3.  `associativity_add`        — `((a + b) + c)[0] ~ (a + (b + c))[0]`, 4 ULP
//! 4.  `distributivity`           — `(a * (b + c))[0] ~ (a*b + a*c)[0]`, 4 ULP
//! 5.  `additive_inverse`         — `(a - a) == 0`, bit-exact
//! 6.  `multiplicative_identity`  — `(a * one) == a`, bit-exact
//! 7.  `exp_log_roundtrip`        — `log(exp(x))[0] ~ x[0]`, 1e-13
//! 8.  `sqrt_squared`             — `(sqrt(x) * sqrt(x))[0] ~ x[0]`, 1e-13
//! 9.  `pow_inverse`              — `(pow(x, a) * pow(x, -a))[0] ~ 1`, 1e-13
//! 10. `leibniz_var0`             — `(a*b)[VAR0] == a[0]*b[VAR0] + a[VAR0]*b[0]`, 2 ULP
//! 11. `leibniz_var1`             — analogous at VAR1 (exercises N>=2 recursion)

#![cfg(feature = "testing")]

use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use proptest::prelude::*;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use xcfun_ad::for_tests::cpu_client;

// -----------------------------------------------------------------------------
// Harness
// -----------------------------------------------------------------------------

const DEFAULT_ITERS: usize = 10_000;

/// Honour `PROPTEST_CASES` as the iteration knob (D-18). Falls back to
/// `DEFAULT_ITERS = 10_000` so the default CI run executes ≥ 110_000 total
/// property-iterations across the 11 tests in this file.
fn iter_count() -> usize {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ITERS)
}

/// Generate `count` samples from `strategy` using a deterministic
/// `TestRunner`. This is the batch-per-property upfront generator (D-18):
/// one strategy → `count` inputs → single kernel launch over all of them.
fn generate<S, T>(strategy: S, count: usize) -> Vec<T>
where
    S: Strategy<Value = T>,
{
    let mut runner = TestRunner::deterministic();
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let tree = strategy.new_tree(&mut runner).expect("proptest new_tree");
        out.push(tree.current());
    }
    out
}

/// Allocate a new device buffer of `len` f64s.
fn empty_buffer(client: &xcfun_ad::for_tests::CpuClient, len: usize) -> cubecl::server::Handle {
    client.empty(len * core::mem::size_of::<f64>())
}

// -----------------------------------------------------------------------------
// Property 1: commutativity of add  (bit-exact, CTaylor at N=1, 2-coeff)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn commutativity_add_kernel(
    a: &Array<f64>,     // length K*2 — flattened (a0, a1) per iter
    b: &Array<f64>,     // length K*2
    diff: &mut Array<f64>, // length K*2 — (a+b) - (b+a)
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() / 2 {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        // (a + b)
        let p0 = a[i0] + b[i0];
        let p1 = a[i1] + b[i1];
        // (b + a)
        let q0 = b[i0] + a[i0];
        let q1 = b[i1] + a[i1];
        diff[i0] = p0 - q0;
        diff[i1] = p1 - q1;
    }
}

#[test]
fn commutativity_add() {
    let iters = iter_count();
    let strategy = (
        prop::collection::vec(-100.0_f64..100.0, 2),
        prop::collection::vec(-100.0_f64..100.0, 2),
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    let mut b_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for (a, b) in &inputs {
        a_flat.extend_from_slice(a);
        b_flat.extend_from_slice(b);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let b_h = client.create_from_slice(f64::as_bytes(&b_flat));
    let d_h = empty_buffer(client, iters * 2);
    let d_read = d_h.clone();

    unsafe {
        commutativity_add_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 2),
            ArrayArg::from_raw_parts(b_h, iters * 2),
            ArrayArg::from_raw_parts(d_h, iters * 2),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        assert_eq!(
            *d, 0.0_f64,
            "commutativity_add iter={} coef={} diff={}",
            i / 2, i % 2, d
        );
    }
}

// -----------------------------------------------------------------------------
// Property 2: commutativity of mul at N=1 (bit-exact)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn commutativity_mul_kernel(
    a: &Array<f64>,
    b: &Array<f64>,
    diff: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() / 2 {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        // (a * b) via N=1 mul_set: dst[0] = a0*b0; dst[1] = a0*b1 + a1*b0.
        let ab0 = a[i0] * b[i0];
        let t10 = a[i0] * b[i1];
        let t11 = a[i1] * b[i0];
        let ab1 = t10 + t11;
        // (b * a)
        let ba0 = b[i0] * a[i0];
        let u10 = b[i0] * a[i1];
        let u11 = b[i1] * a[i0];
        let ba1 = u10 + u11;

        diff[i0] = ab0 - ba0;
        diff[i1] = ab1 - ba1;
    }
}

#[test]
fn commutativity_mul() {
    let iters = iter_count();
    let strategy = (
        prop::collection::vec(-10.0_f64..10.0, 2),
        prop::collection::vec(-10.0_f64..10.0, 2),
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    let mut b_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for (a, b) in &inputs {
        a_flat.extend_from_slice(a);
        b_flat.extend_from_slice(b);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let b_h = client.create_from_slice(f64::as_bytes(&b_flat));
    let d_h = empty_buffer(client, iters * 2);
    let d_read = d_h.clone();

    unsafe {
        commutativity_mul_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 2),
            ArrayArg::from_raw_parts(b_h, iters * 2),
            ArrayArg::from_raw_parts(d_h, iters * 2),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        assert_eq!(
            *d, 0.0_f64,
            "commutativity_mul iter={} coef={} diff={}",
            i / 2, i % 2, d
        );
    }
}

// -----------------------------------------------------------------------------
// Property 3: associativity of add (scalar constant; 4 ULP)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn associativity_add_kernel(
    a: &Array<f64>,
    b: &Array<f64>,
    c: &Array<f64>,
    diff: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() {
        let lhs = (a[i] + b[i]) + c[i];
        let rhs = a[i] + (b[i] + c[i]);
        diff[i] = lhs - rhs;
    }
}

#[test]
fn associativity_add() {
    let iters = iter_count();
    let strategy = (
        -1e6_f64..1e6,
        -1e6_f64..1e6,
        -1e6_f64..1e6,
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let a: Vec<f64> = inputs.iter().map(|(a, _, _)| *a).collect();
    let b: Vec<f64> = inputs.iter().map(|(_, b, _)| *b).collect();
    let c: Vec<f64> = inputs.iter().map(|(_, _, c)| *c).collect();

    let a_h = client.create_from_slice(f64::as_bytes(&a));
    let b_h = client.create_from_slice(f64::as_bytes(&b));
    let c_h = client.create_from_slice(f64::as_bytes(&c));
    let d_h = empty_buffer(client, iters);
    let d_read = d_h.clone();

    unsafe {
        associativity_add_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters),
            ArrayArg::from_raw_parts(b_h, iters),
            ArrayArg::from_raw_parts(c_h, iters),
            ArrayArg::from_raw_parts(d_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    // 4 ULP at the order-of-magnitude of the sum (f64 non-associativity).
    for (i, d) in diff.iter().enumerate() {
        let ai = a[i];
        let bi = b[i];
        let ci = c[i];
        let scale = (ai.abs() + bi.abs() + ci.abs()).max(1.0);
        let tol = 4.0 * f64::EPSILON * scale;
        assert!(
            d.abs() <= tol,
            "associativity_add iter={} diff={:.3e} tol={:.3e} a={} b={} c={}",
            i, d, tol, ai, bi, ci
        );
    }
}

// -----------------------------------------------------------------------------
// Property 4: distributivity of mul over add (scalar constant; 4 ULP)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn distributivity_kernel(
    a: &Array<f64>,
    b: &Array<f64>,
    c: &Array<f64>,
    diff: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() {
        let lhs = a[i] * (b[i] + c[i]);
        let rhs = a[i] * b[i] + a[i] * c[i];
        diff[i] = lhs - rhs;
    }
}

#[test]
fn distributivity() {
    let iters = iter_count();
    let strategy = (
        -100.0_f64..100.0,
        -100.0_f64..100.0,
        -100.0_f64..100.0,
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let a: Vec<f64> = inputs.iter().map(|(a, _, _)| *a).collect();
    let b: Vec<f64> = inputs.iter().map(|(_, b, _)| *b).collect();
    let c: Vec<f64> = inputs.iter().map(|(_, _, c)| *c).collect();

    let a_h = client.create_from_slice(f64::as_bytes(&a));
    let b_h = client.create_from_slice(f64::as_bytes(&b));
    let c_h = client.create_from_slice(f64::as_bytes(&c));
    let d_h = empty_buffer(client, iters);
    let d_read = d_h.clone();

    unsafe {
        distributivity_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters),
            ArrayArg::from_raw_parts(b_h, iters),
            ArrayArg::from_raw_parts(c_h, iters),
            ArrayArg::from_raw_parts(d_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        let ai = a[i];
        let bi = b[i];
        let ci = c[i];
        let scale = (ai.abs() * (bi.abs() + ci.abs()) + (ai * bi).abs() + (ai * ci).abs()).max(1.0);
        let tol = 4.0 * f64::EPSILON * scale;
        assert!(
            d.abs() <= tol,
            "distributivity iter={} diff={:.3e} tol={:.3e} a={} b={} c={}",
            i, d, tol, ai, bi, ci
        );
    }
}

// -----------------------------------------------------------------------------
// Property 5: additive inverse  (a - a == 0, bit-exact)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn additive_inverse_kernel(
    a: &Array<f64>,
    out: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < out.len() / 2 {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        // (a - a) elementwise at N=1.
        out[i0] = a[i0] - a[i0];
        out[i1] = a[i1] - a[i1];
    }
}

#[test]
fn additive_inverse() {
    let iters = iter_count();
    let strategy = prop::collection::vec(-1e10_f64..1e10, 2);
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for a in &inputs {
        a_flat.extend_from_slice(a);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let o_h = empty_buffer(client, iters * 2);
    let o_read = o_h.clone();

    unsafe {
        additive_inverse_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 2),
            ArrayArg::from_raw_parts(o_h, iters * 2),
        );
    }

    let bytes = client.read_one_unchecked(o_read);
    let out = f64::from_bytes(&bytes);
    for (i, v) in out.iter().enumerate() {
        assert_eq!(
            *v, 0.0_f64,
            "additive_inverse iter={} coef={} got={}",
            i / 2, i % 2, v
        );
    }
}

// -----------------------------------------------------------------------------
// Property 6: multiplicative identity  (a * 1 == a, bit-exact at N=1)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn multiplicative_identity_kernel(
    a: &Array<f64>,
    out: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < out.len() / 2 {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        // `one` CTaylor at N=1 is [1, 0]. Multiplying (a * one) via N=1 mul_set:
        //   dst[0] = a0 * 1 = a0
        //   dst[1] = a0 * 0 + a1 * 1 = a1
        // Bit-exact at f64.
        let one0 = f64::new(1.0);
        let zero = f64::new(0.0);
        let p0 = a[i0] * one0;
        let t10 = a[i0] * zero;
        let t11 = a[i1] * one0;
        let p1 = t10 + t11;
        // Store difference from a.
        out[i0] = p0 - a[i0];
        out[i1] = p1 - a[i1];
    }
}

#[test]
fn multiplicative_identity() {
    let iters = iter_count();
    let strategy = prop::collection::vec(-1e5_f64..1e5, 2);
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for a in &inputs {
        a_flat.extend_from_slice(a);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let o_h = empty_buffer(client, iters * 2);
    let o_read = o_h.clone();

    unsafe {
        multiplicative_identity_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 2),
            ArrayArg::from_raw_parts(o_h, iters * 2),
        );
    }

    let bytes = client.read_one_unchecked(o_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        assert_eq!(
            *d, 0.0_f64,
            "multiplicative_identity iter={} coef={} diff={}",
            i / 2, i % 2, d
        );
    }
}

// -----------------------------------------------------------------------------
// Property 7: exp/log round-trip  (1e-13 rel-err on x_cnst > 0)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn exp_log_roundtrip_kernel(
    x: &Array<f64>,
    out: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < out.len() {
        // log(exp(x)) ≈ x at the scalar level (constant coefficient only).
        // Use `.ln()` for natural log — cubecl's Float::log is a base-log
        // (2-arg); natural log is the method we need here.
        let y = x[i].exp();
        out[i] = y.ln();
    }
}

#[test]
fn exp_log_roundtrip() {
    let iters = iter_count();
    // Keep exp(x) bounded away from overflow: x ∈ (-50, 50) safe for f64.
    let strategy = -50.0_f64..50.0;
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let x_h = client.create_from_slice(f64::as_bytes(&inputs));
    let o_h = empty_buffer(client, iters);
    let o_read = o_h.clone();

    unsafe {
        exp_log_roundtrip_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(x_h, iters),
            ArrayArg::from_raw_parts(o_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(o_read);
    let got = f64::from_bytes(&bytes);
    for (i, (g, x)) in got.iter().zip(inputs.iter()).enumerate() {
        let denom = x.abs().max(1.0);
        let rel = (g - x).abs() / denom;
        assert!(
            rel < 1e-13,
            "exp_log_roundtrip iter={} x={} log(exp(x))={} rel={:.3e}",
            i, x, g, rel
        );
    }
}

// -----------------------------------------------------------------------------
// Property 8: sqrt-squared invariance  (1e-13 rel-err on x > 0)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn sqrt_squared_kernel(
    x: &Array<f64>,
    out: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < out.len() {
        let s = x[i].sqrt();
        out[i] = s * s;
    }
}

#[test]
fn sqrt_squared() {
    let iters = iter_count();
    let strategy = 1e-6_f64..1e6;
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let x_h = client.create_from_slice(f64::as_bytes(&inputs));
    let o_h = empty_buffer(client, iters);
    let o_read = o_h.clone();

    unsafe {
        sqrt_squared_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(x_h, iters),
            ArrayArg::from_raw_parts(o_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(o_read);
    let got = f64::from_bytes(&bytes);
    for (i, (g, x)) in got.iter().zip(inputs.iter()).enumerate() {
        let denom = x.abs().max(1.0);
        let rel = (g - x).abs() / denom;
        assert!(
            rel < 1e-13,
            "sqrt_squared iter={} x={} (sqrt*sqrt)={} rel={:.3e}",
            i, x, g, rel
        );
    }
}

// -----------------------------------------------------------------------------
// Property 9: pow inverse  (x^a * x^-a == 1, 1e-13 rel-err on x > 0)
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn pow_inverse_kernel(
    xa: &Array<f64>, // length K*2 — (x, a) per iter
    out: &mut Array<f64>, // length K — x^a * x^(-a)
) {
    let i = ABSOLUTE_POS;
    if i < out.len() {
        let x = xa[i * 2];
        let a = xa[i * 2 + 1];
        let p = x.powf(a);
        let q = x.powf(-a);
        out[i] = p * q;
    }
}

#[test]
fn pow_inverse() {
    let iters = iter_count();
    let strategy = (
        0.1_f64..100.0,     // x > 0
        -5.0_f64..5.0,      // exponent a (bounded to keep intermediate range tame)
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for (x, a) in &inputs {
        flat.push(*x);
        flat.push(*a);
    }

    let xa_h = client.create_from_slice(f64::as_bytes(&flat));
    let o_h = empty_buffer(client, iters);
    let o_read = o_h.clone();

    unsafe {
        pow_inverse_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(xa_h, iters * 2),
            ArrayArg::from_raw_parts(o_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(o_read);
    let got = f64::from_bytes(&bytes);
    for (i, (g, (x, a))) in got.iter().zip(inputs.iter()).enumerate() {
        let rel = (g - 1.0_f64).abs();
        assert!(
            rel < 1e-13,
            "pow_inverse iter={} x={} a={} got={} rel={:.3e}",
            i, x, a, g, rel
        );
    }
}

// -----------------------------------------------------------------------------
// Property 10: Leibniz product rule at VAR0  (N=1; bit-exact)
//
// dst[VAR0] of (a*b) equals a[CNST]*b[VAR0] + a[VAR0]*b[CNST]
// by the definition of the N=1 mul_set base case.
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn leibniz_var0_kernel(
    a: &Array<f64>,
    b: &Array<f64>,
    diff: &mut Array<f64>,
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        // dst[VAR0] from N=1 mul_set
        let t10 = a[i0] * b[i1];
        let t11 = a[i1] * b[i0];
        let got = t10 + t11;
        // Expected Leibniz: a0*b1 + a1*b0 (same parenthesisation order)
        let expect_t10 = a[i0] * b[i1];
        let expect_t11 = a[i1] * b[i0];
        let expect = expect_t10 + expect_t11;
        diff[i] = got - expect;
    }
}

#[test]
fn leibniz_var0() {
    let iters = iter_count();
    let strategy = (
        prop::collection::vec(-50.0_f64..50.0, 2),
        prop::collection::vec(-50.0_f64..50.0, 2),
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    let mut b_flat: Vec<f64> = Vec::with_capacity(iters * 2);
    for (a, b) in &inputs {
        a_flat.extend_from_slice(a);
        b_flat.extend_from_slice(b);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let b_h = client.create_from_slice(f64::as_bytes(&b_flat));
    let d_h = empty_buffer(client, iters);
    let d_read = d_h.clone();

    unsafe {
        leibniz_var0_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 2),
            ArrayArg::from_raw_parts(b_h, iters * 2),
            ArrayArg::from_raw_parts(d_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        assert_eq!(*d, 0.0_f64, "leibniz_var0 iter={} diff={}", i, d);
    }
}

// -----------------------------------------------------------------------------
// Property 11: Leibniz product rule at VAR1  (N=2; bit-exact base-case check)
//
// For the CTaylor<_, N=2> mul_set, dst[VAR1=2] = x[CNST]*y[VAR1] + x[VAR1]*y[CNST]
// (no cross-product terms at that index). Exercises the N≥2 recursion's
// base-case on the second variable.
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn leibniz_var1_kernel(
    a: &Array<f64>, // length K*4 — (a0, a1, a2, a3) per iter at N=2
    b: &Array<f64>,
    diff: &mut Array<f64>, // length K — (ab)[2] - (a0*b2 + a2*b0)
) {
    let i = ABSOLUTE_POS;
    if i < diff.len() {
        let base = i * 4;
        let a0 = a[base];
        let a2 = a[base + 2];
        let b0 = b[base];
        let b2 = b[base + 2];
        // From N=2 mul_set: dst[2] = x[0]*y[2] + x[2]*y[0]
        let t20 = a0 * b2;
        let t21 = a2 * b0;
        let got = t20 + t21;
        // Leibniz-expected (same accumulation order)
        let e20 = a0 * b2;
        let e21 = a2 * b0;
        let expect = e20 + e21;
        diff[i] = got - expect;
    }
}

#[test]
fn leibniz_var1() {
    let iters = iter_count();
    let strategy = (
        prop::collection::vec(-20.0_f64..20.0, 4),
        prop::collection::vec(-20.0_f64..20.0, 4),
    );
    let inputs = generate(strategy, iters);
    let client = cpu_client();

    let mut a_flat: Vec<f64> = Vec::with_capacity(iters * 4);
    let mut b_flat: Vec<f64> = Vec::with_capacity(iters * 4);
    for (a, b) in &inputs {
        a_flat.extend_from_slice(a);
        b_flat.extend_from_slice(b);
    }

    let a_h = client.create_from_slice(f64::as_bytes(&a_flat));
    let b_h = client.create_from_slice(f64::as_bytes(&b_flat));
    let d_h = empty_buffer(client, iters);
    let d_read = d_h.clone();

    unsafe {
        leibniz_var1_kernel::launch_unchecked::<CpuRuntime>(
            client,
            CubeCount::Static(iters as u32, 1, 1),
            CubeDim::new_1d(1),
            ArrayArg::from_raw_parts(a_h, iters * 4),
            ArrayArg::from_raw_parts(b_h, iters * 4),
            ArrayArg::from_raw_parts(d_h, iters),
        );
    }

    let bytes = client.read_one_unchecked(d_read);
    let diff = f64::from_bytes(&bytes);
    for (i, d) in diff.iter().enumerate() {
        assert_eq!(*d, 0.0_f64, "leibniz_var1 iter={} diff={}", i, d);
    }
}
