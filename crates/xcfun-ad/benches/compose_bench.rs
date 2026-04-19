//! Criterion baseline for composed CTaylor functions (`exp`, `log`, `pow`) at
//! `N = 4` × batch sizes `{1, 64, 1024}`. CONTEXT.md D-19.
//!
//! No regression gate in v1 (D-19; PERF-01 deferred).
//!
//! # Design choice
//!
//! Each composed function (`ctaylor_exp`, `ctaylor_log`, `ctaylor_pow`)
//! internally allocates an `Array<F>` scratch buffer inside the `#[cube]`
//! body (stack-local on cubecl-cpu), calls the corresponding `*_expand`
//! scalar series filler, then routes through `ctaylor_compose`. At `N = 4`
//! the composition unrolls over 16 coefficients per input.
//!
//! To measure kernel-launch amortization vs. per-element cost, the bench
//! launches one kernel per batch element. This is the conservative option:
//! cubecl 0.10-pre.3 does not admit zero-copy sub-slicing of `Array<F>`
//! across batch elements, and exploring the scratch-per-unit layout
//! was out of scope for Plan 01-07 Task 2. A follow-up (Phase 6 GPU
//! kernels) will revisit the true batched topology.
//!
//! Bench outputs:
//!   - `compose_n4_exp/{1,64,1024}`
//!   - `compose_n4_log/{1,64,1024}`
//!   - `compose_n4_pow/{1,64,1024}`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;
use std::hint::black_box;
use xcfun_ad::for_tests::cpu_client;
use xcfun_ad::math::{ctaylor_exp, ctaylor_log, ctaylor_pow};

// Kernel adapters — one per composed fn, comptime n threaded through.

#[cube(launch_unchecked)]
fn kernel_exp<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_exp::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_log<F: Float>(x: &Array<F>, out: &mut Array<F>, #[comptime] n: u32) {
    ctaylor_log::<F>(x, out, n);
}

#[cube(launch_unchecked)]
fn kernel_pow<F: Float>(
    x: &Array<F>,
    aa: &Array<F>,
    out: &mut Array<F>,
    #[comptime] n: u32,
) {
    ctaylor_pow::<F>(x, aa[0], out, n);
}

const N: u32 = 4;
const SIZE: usize = 1_usize << 4; // = 16

fn bench_compose(c: &mut Criterion) {
    let mut group = c.benchmark_group("ctaylor_composed_n4");
    let client = cpu_client();

    // Per-batch CTaylor input: a domain-safe point with CNST > 0 (all three
    // ops require x[CNST] > 0).
    let mut x_in = vec![0.0_f64; SIZE];
    x_in[0] = 1.3; // CNST
    x_in[1] = 0.25; // VAR0 slope

    for (op_name, which) in &[("exp", 0u8), ("log", 1u8), ("pow", 2u8)] {
        for &batch in &[1_usize, 64, 1024] {
            let id = BenchmarkId::new(*op_name, batch);
            group.bench_with_input(id, &batch, |bencher, &bs| {
                bencher.iter(|| {
                    // Kernel is launched `bs` times per iter. This measures the
                    // launch-amortized cost at the chosen batch size.
                    for _ in 0..bs {
                        let x_h = client.create_from_slice(f64::as_bytes(&x_in));
                        let out_h = client.empty(SIZE * core::mem::size_of::<f64>());
                        let read_h = out_h.clone();

                        match *which {
                            0 => unsafe {
                                kernel_exp::launch_unchecked::<f64, CpuRuntime>(
                                    client,
                                    CubeCount::Static(1, 1, 1),
                                    CubeDim::new_1d(1),
                                    ArrayArg::from_raw_parts(x_h, SIZE),
                                    ArrayArg::from_raw_parts(out_h, SIZE),
                                    N,
                                );
                            },
                            1 => unsafe {
                                kernel_log::launch_unchecked::<f64, CpuRuntime>(
                                    client,
                                    CubeCount::Static(1, 1, 1),
                                    CubeDim::new_1d(1),
                                    ArrayArg::from_raw_parts(x_h, SIZE),
                                    ArrayArg::from_raw_parts(out_h, SIZE),
                                    N,
                                );
                            },
                            _ => {
                                let aa = [1.5_f64];
                                let aa_h = client.create_from_slice(f64::as_bytes(&aa));
                                unsafe {
                                    kernel_pow::launch_unchecked::<f64, CpuRuntime>(
                                        client,
                                        CubeCount::Static(1, 1, 1),
                                        CubeDim::new_1d(1),
                                        ArrayArg::from_raw_parts(x_h, SIZE),
                                        ArrayArg::from_raw_parts(aa_h, 1),
                                        ArrayArg::from_raw_parts(out_h, SIZE),
                                        N,
                                    );
                                }
                            }
                        };

                        let bytes = client.read_one_unchecked(read_h);
                        black_box(bytes);
                    }
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_compose);
criterion_main!(benches);
