//! D-17 — Phase 5 `sync_weights_from_settings` used `Box::leak` per
//! `xcfun_rs::Functional::set` call.  Plan 06-06 refactors
//! `xcfun_eval::Functional::weights` from `&'static [(FunctionalId, f64)]`
//! to `Vec<(FunctionalId, f64)>` and drops the leak.
//!
//! This test asserts that 100 successive `set` calls do NOT produce 100
//! never-freed heap allocations.  The CountingAllocator tracks both `alloc`
//! and `dealloc`; if the leak is gone, the difference (alloc − dealloc) does
//! not grow linearly with the number of `set` calls.
//!
//! # Bound
//!
//! `Vec::clear()` does not deallocate (capacity is preserved).  `Vec::push`
//! allocates only when growing past current capacity.  The active-functional
//! set for `XC_SLATERX` has length 1; capacity stabilises after the first
//! `set`.  So we expect `(alloc − dealloc)` after 100 `set` calls to differ
//! from the snapshot by ≤ a small constant — never the ~100 unfreed
//! allocations the Phase 5 `Box::leak` would have produced.
//!
//! # Why the slack of ±5
//!
//! The CountingAllocator measures every allocation in the entire process,
//! including ones from `xcfun_eval::Functional::set`'s alias-resolution
//! recursion (small `format!`-free path), the `OnceLock` for `cpu_client`
//! (one-time, but warm-up may not catch every static init), and any other
//! incidental Rust-runtime allocations (panic handler init, etc.).  We are
//! NOT trying to assert "zero allocations" — only "no LEAK", i.e., no growth
//! in the alloc-minus-dealloc count proportional to the number of `set` calls.
//!
//! The Phase 5 `Box::leak` regression would yield ~100 net allocations
//! (one leak per set call); the slack ±5 is comfortable distance from that.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator;

#[test]
fn set_does_not_leak() {
    let mut f = xcfun_rs::Functional::new();

    // Warm-up: the FIRST set() call may grow the inner Vec from capacity 0
    // to capacity ≥ 1, plus the first `OnceLock` initialisation of
    // FUNCTIONAL_DESCRIPTORS / ALIASES static tables.  Snapshot AFTER
    // warm-up so we measure only the steady-state per-`set` cost.
    f.set("slaterx", 0.01).unwrap();
    f.set("slaterx", 0.02).unwrap();

    let snap_alloc = ALLOC_COUNT.load(Ordering::SeqCst);
    let snap_dealloc = DEALLOC_COUNT.load(Ordering::SeqCst);

    for i in 0..100_usize {
        // Vary the value to defeat any potential dedup elision.
        f.set("slaterx", (i as f64 + 3.0) * 0.01).unwrap();
    }

    let alloc_delta = ALLOC_COUNT.load(Ordering::SeqCst) - snap_alloc;
    let dealloc_delta = DEALLOC_COUNT.load(Ordering::SeqCst) - snap_dealloc;
    let net_leaked = (alloc_delta as i64) - (dealloc_delta as i64);

    // Phase 5 regression bar: a `Box::leak` per call would produce
    // ~100 net allocations.  Post-D-17 the inner `Vec` re-uses capacity;
    // `net_leaked` should be a small constant (typically 0).
    assert!(
        net_leaked.abs() <= 5,
        "Box::leak regression suspected: {} net allocations across 100 set() calls \
         (alloc={}, dealloc={}). D-17 expects ~0; Phase 5 baseline was ~100.",
        net_leaked,
        alloc_delta,
        dealloc_delta,
    );
}
