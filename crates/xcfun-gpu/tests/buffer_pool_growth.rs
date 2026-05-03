//! GPU-04 — buffer pool grows powers-of-two on overflow; never shrinks
//! (CONTEXT D-15).
//!
//! Plan 06-02a contract: initial capacity = 64. After `reserve(10)` the
//! capacity stays at 64. After `reserve(50)` it stays at 64. After
//! `reserve(200)` it grows to 256 (the next power of two ≥ 200).

#![cfg(feature = "cpu")]

use cubecl_cpu::CpuRuntime;
use xcfun_core::{FunctionalId, Mode, Vars};
use xcfun_eval::Functional;
use xcfun_eval::functional::DEFAULT_SETTINGS;
use xcfun_gpu::Batch;

static SLATERX_WEIGHTS: &[(FunctionalId, f64)] = &[(FunctionalId::XC_SLATERX, 1.0)];

fn slater_functional() -> Functional {
    Functional {
        weights: SLATERX_WEIGHTS,
        vars: Vars::A_B,
        mode: Mode::PartialDerivatives,
        order: 0,
        settings: DEFAULT_SETTINGS,
        settings_gen: 0,
    }
}

#[test]
fn initial_capacity_is_64() {
    let fun = slater_functional();
    let batch = Batch::<CpuRuntime>::open_cpu(&fun).expect("open_cpu");
    assert_eq!(batch.capacity(), 64);
}

#[test]
fn reserve_below_initial_capacity_is_a_noop() {
    let fun = slater_functional();
    let mut batch = Batch::<CpuRuntime>::open_cpu(&fun).expect("open_cpu");
    batch.reserve(10);
    assert_eq!(batch.capacity(), 64);
    batch.reserve(50);
    assert_eq!(batch.capacity(), 64);
    batch.reserve(64); // exactly at boundary — no growth
    assert_eq!(batch.capacity(), 64);
}

#[test]
fn reserve_above_initial_capacity_doubles_to_next_power_of_two() {
    let fun = slater_functional();
    let mut batch = Batch::<CpuRuntime>::open_cpu(&fun).expect("open_cpu");
    batch.reserve(200);
    // 64 → 128 → 256 ; 256 is the first power of two ≥ 200.
    assert_eq!(batch.capacity(), 256);
}

#[test]
fn capacity_never_shrinks() {
    let fun = slater_functional();
    let mut batch = Batch::<CpuRuntime>::open_cpu(&fun).expect("open_cpu");
    batch.reserve(200);
    assert_eq!(batch.capacity(), 256);
    // A subsequent smaller reserve must NOT shrink the buffer.
    batch.reserve(10);
    assert_eq!(batch.capacity(), 256);
    batch.reserve(50);
    assert_eq!(batch.capacity(), 256);
}

#[test]
fn reserve_grows_through_multiple_doublings() {
    let fun = slater_functional();
    let mut batch = Batch::<CpuRuntime>::open_cpu(&fun).expect("open_cpu");
    batch.reserve(1000);
    // 64 → 128 → 256 → 512 → 1024 ; 1024 is the first power of two ≥ 1000.
    assert_eq!(batch.capacity(), 1024);
}
