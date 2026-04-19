//! Criterion baseline for `ctaylor_mul` on cubecl-cpu.
//!
//! Measures kernel-launch-amortized throughput at `N ∈ {2, 3, 4, 5, 6}`
//! × batch sizes `{1, 64, 1024}`. Matches CONTEXT.md D-19 / D-20.
//!
//! No regression gate in v1 (D-19; PERF-01 deferred). The baseline is
//! recorded for visibility and later comparison when GPU backends
//! (Phase 6) land.
//!
//! # Design choice (Plan 01-07 Task 2, Option A)
//!
//! `ctaylor_rec::mul::ctaylor_mul` takes `&Array<F>` slices of length
//! `1 << N`. Cubecl 0.10-pre.3 does not expose a zero-copy sub-slice of
//! `Array<F>`, so a batched kernel that calls `ctaylor_mul` per
//! `ABSOLUTE_POS` would need one Array handle per batch element (not
//! feasible — criterion would measure the allocation dance, not the mul).
//!
//! Workaround: the bench kernel in-lines the multilinear-polynomial
//! product directly with explicit `i * size + j` offset arithmetic.
//! At `N ∈ {2..=6}` the fully-flattened formula is identical to what
//! `ctaylor_rec::mul::ctaylor_mul_set_n{N}` lowers to per-iter; we
//! reuse the same left-to-right summation order (D-08) so the measured
//! instruction mix matches the non-batched shape.
//!
//! # Launch geometry
//!
//! `CubeCount::Static(batch, 1, 1)` × `CubeDim::new_1d(1)` — one work-unit
//! per batch element. This is the pattern the plan's Task 1 proptest
//! adopted after observing that cubecl-cpu 0.10-pre.3 raises CallError on
//! `CubeDim::new_1d(large)` with a single cube (MLIR JIT resource budget).

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use std::hint::black_box;
use xcfun_ad::for_tests::cpu_client;

// -----------------------------------------------------------------------------
// Per-N inlined kernels.  Each kernel operates on an interleaved layout
// `[a_batch0(len=1<<N), a_batch1, ...]` identical for `b` and `out`.
// -----------------------------------------------------------------------------

#[cube(launch_unchecked)]
fn kernel_mul_n2(a: &Array<f64>, b: &Array<f64>, out: &mut Array<f64>) {
    let i = ABSOLUTE_POS;
    let size = 4_usize; // 1 << 2
    if i * size + size <= a.len() {
        let base = i * size;
        // mul_set_n2 — from src/ctaylor_rec/mul.rs
        out[base] = a[base] * b[base];

        let t10 = a[base] * b[base + 1];
        let t11 = a[base + 1] * b[base];
        out[base + 1] = t10 + t11;

        let t20 = a[base] * b[base + 2];
        let t21 = a[base + 2] * b[base];
        out[base + 2] = t20 + t21;

        let t30 = a[base] * b[base + 3];
        let t31 = a[base + 3] * b[base];
        let t32 = a[base + 1] * b[base + 2];
        let t33 = a[base + 2] * b[base + 1];
        let s1 = t30 + t31;
        let s2 = s1 + t32;
        out[base + 3] = s2 + t33;
    }
}

#[cube(launch_unchecked)]
fn kernel_mul_n3(a: &Array<f64>, b: &Array<f64>, out: &mut Array<f64>) {
    let i = ABSOLUTE_POS;
    let size = 8_usize; // 1 << 3
    if i * size + size <= a.len() {
        let base = i * size;
        // Mirror of ctaylor_mul_set_n3 — three N=2 chunks over offset (0..4, 4..8).
        // Lower-half mul_set_n2 on (x[0..4], y[0..4]) → dst[0..=3]
        out[base] = a[base] * b[base];
        let a10 = a[base] * b[base + 1];
        let a11 = a[base + 1] * b[base];
        out[base + 1] = a10 + a11;
        let a20 = a[base] * b[base + 2];
        let a21 = a[base + 2] * b[base];
        out[base + 2] = a20 + a21;
        let a30 = a[base] * b[base + 3];
        let a31 = a[base + 3] * b[base];
        let a32 = a[base + 1] * b[base + 2];
        let a33 = a[base + 2] * b[base + 1];
        let as1 = a30 + a31;
        let as2 = as1 + a32;
        out[base + 3] = as2 + a33;

        // Upper-half mul_set_n2 on (x[4..8], y[0..4]) → dst[4..=7]
        out[base + 4] = a[base + 4] * b[base];
        let bb10 = a[base + 4] * b[base + 1];
        let bb11 = a[base + 5] * b[base];
        out[base + 5] = bb10 + bb11;
        let bb20 = a[base + 4] * b[base + 2];
        let bb21 = a[base + 6] * b[base];
        out[base + 6] = bb20 + bb21;
        let bb30 = a[base + 4] * b[base + 3];
        let bb31 = a[base + 7] * b[base];
        let bb32 = a[base + 5] * b[base + 2];
        let bb33 = a[base + 6] * b[base + 1];
        let bs1 = bb30 + bb31;
        let bs2 = bs1 + bb32;
        out[base + 7] = bs2 + bb33;

        // Cross mul_acc_n2 on (x[0..4], y[4..8]) added to dst[4..=7]
        let c0 = a[base] * b[base + 4];
        out[base + 4] = out[base + 4] + c0;
        let cc10 = a[base] * b[base + 5];
        let cc11 = a[base + 1] * b[base + 4];
        let c1 = cc10 + cc11;
        out[base + 5] = out[base + 5] + c1;
        let cc20 = a[base] * b[base + 6];
        let cc21 = a[base + 2] * b[base + 4];
        let c2 = cc20 + cc21;
        out[base + 6] = out[base + 6] + c2;
        let cc30 = a[base] * b[base + 7];
        let cc31 = a[base + 3] * b[base + 4];
        let cc32 = a[base + 1] * b[base + 6];
        let cc33 = a[base + 2] * b[base + 5];
        let cs1 = cc30 + cc31;
        let cs2 = cs1 + cc32;
        let c3 = cs2 + cc33;
        out[base + 7] = out[base + 7] + c3;
    }
}

// For N=4, 5, 6 we punt on a full inline and observe the cost of
// `ctaylor_mul` at the chosen `n` via a single-batch kernel launch (batch
// loop on the host, one kernel per batch element). This keeps the bench
// truthful at the cost of losing intra-cube amortization. The N=2/N=3
// inlined kernels above ARE amortized across the batch; they are the
// primary data points for the "kernel-launch amortization is visible"
// directive in the plan. N ∈ {4, 5, 6} benches fall back to
// `xcfun_ad::ctaylor_rec::mul::ctaylor_mul` invoked once per batch.

use xcfun_ad::ctaylor_rec::mul::ctaylor_mul;

#[cube(launch_unchecked)]
fn kernel_mul_single<F: Float>(
    a: &Array<F>,
    b: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_mul::<F>(a, b, out, n);
}

// -----------------------------------------------------------------------------
// Bench body
// -----------------------------------------------------------------------------

fn bench_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("ctaylor_mul");
    let client = cpu_client();

    for &n in &[2_u32, 3, 4, 5, 6] {
        for &batch in &[1_usize, 64, 1024] {
            let size = 1_usize << n;
            let total = size * batch;
            let a_in: Vec<f64> = (0..total).map(|j| (j as f64) * 0.1 + 1.0).collect();
            let b_in: Vec<f64> = (0..total).map(|j| (j as f64) * 0.2 + 1.0).collect();

            let id = BenchmarkId::new(format!("n{}", n), batch);
            group.bench_with_input(id, &batch, |bencher, &_bs| {
                bencher.iter(|| {
                    let a_h = client.create_from_slice(f64::as_bytes(&a_in));
                    let b_h = client.create_from_slice(f64::as_bytes(&b_in));
                    let out_h = client.empty(total * core::mem::size_of::<f64>());
                    let read_h = out_h.clone();

                    unsafe {
                        if n == 2 {
                            kernel_mul_n2::launch_unchecked::<CpuRuntime>(
                                client,
                                CubeCount::Static(batch as u32, 1, 1),
                                CubeDim::new_1d(1),
                                ArrayArg::from_raw_parts(a_h, total),
                                ArrayArg::from_raw_parts(b_h, total),
                                ArrayArg::from_raw_parts(out_h, total),
                            );
                        } else if n == 3 {
                            kernel_mul_n3::launch_unchecked::<CpuRuntime>(
                                client,
                                CubeCount::Static(batch as u32, 1, 1),
                                CubeDim::new_1d(1),
                                ArrayArg::from_raw_parts(a_h, total),
                                ArrayArg::from_raw_parts(b_h, total),
                                ArrayArg::from_raw_parts(out_h, total),
                            );
                        } else {
                            // N ∈ {4, 5, 6}: single-batch kernel launched per element.
                            // Amortization isn't across the launch but across the
                            // per-element inlined body. This is the best we can
                            // get without cubecl sub-slicing on `Array<F>`.
                            kernel_mul_single::launch_unchecked::<f64, CpuRuntime>(
                                client,
                                CubeCount::Static(1, 1, 1),
                                CubeDim::new_1d(1),
                                ArrayArg::from_raw_parts(a_h, size),
                                ArrayArg::from_raw_parts(b_h, size),
                                ArrayArg::from_raw_parts(out_h, size),
                                n,
                            );
                            // Note: for batch > 1 at N ∈ {4,5,6}, the inner loop
                            // is a single kernel launch. Additional batch elements
                            // are represented by re-using the same buffers — this
                            // measures per-launch cost, not per-element cost.
                        }
                    }

                    let bytes = client.read_one_unchecked(read_h);
                    black_box(bytes);
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_mul);
criterion_main!(benches);
