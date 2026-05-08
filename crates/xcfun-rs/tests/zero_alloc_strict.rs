//! D-12 — Strict zero-alloc per-point form (Phase 6 Plan 06-06).
//!
//! This is the strict counterpart of `tests/zero_alloc.rs` (Phase 5 fall-back
//! form (b) — head/tail mean stability).  Where the fall-back asserts that
//! per-call allocation counts are STABLE across 100 evals (proving the facade
//! wrapper itself is zero-alloc on top of the cubecl-cpu substrate's per-call
//! cost), the STRICT form asserts that the delta from snapshot is exactly 0
//! after warm-up — i.e., the substrate ALSO contributes nothing per call.
//!
//! Achieving strict 0 requires the D-12 reusable handle in
//! `xcfun-rs::Functional` (`UnsafeCell<EvalHandle>` with `input_buf`,
//! `out_buf`, `dens_vars` sized at `eval_setup` time) AND a cubecl-cpu launch
//! path that does not call `client.create_from_slice` / `client.empty`
//! per eval.
//!
//! # Status — Plan 06-06 sign-off
//!
//! The D-17 `weights: Vec<...>` refactor (drops Phase 5 `Box::leak`) is
//! verified independently by `tests/no_leak_on_set.rs`.  The structural
//! D-12 plumbing (`UnsafeCell<EvalHandle>` + `unsafe impl Send/Sync`) lands
//! in this plan.  However the SUBSTRATE-level work — replacing
//! `xcfun-eval::run_launch`'s per-call `client.create_from_slice` /
//! `client.empty` calls with handle-reuse via `EvalHandle` — requires
//! cubecl 0.10-pre.3 surface that does NOT exist (no in-place
//! `client.write(handle, bytes)` API; see RESEARCH §"Pitfall 8" follow-up
//! in 06-RESEARCH.md, and the upstream cubecl tracker noted in the SUMMARY).
//!
//! Until cubecl exposes a buffer-reuse API (or until xcfun-rs owns its own
//! direct cubecl-cpu launcher that bypasses `run_launch`), this test stays
//! `#[ignore]`'d.  When the substrate is upgraded, drop the `#[ignore]` —
//! the assertion `delta == 0` is the canonical regression detector for the
//! "strict zero-alloc per-point" contract (RS-07).
//!
//! The fall-back stability test in `tests/zero_alloc.rs` continues to gate
//! the facade wrapper layer in CI (its assertion: per-call alloc counts do
//! not drift upward across 100 evals — wrapper itself contributes 0).

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

/// RS-07 strict — assert ZERO heap allocations per `eval` call after warm-up.
///
/// **Currently `#[ignore]`'d** pending the cubecl-cpu substrate upgrade.
/// See module-level docs for the rationale.  The fall-back stability test
/// in `tests/zero_alloc.rs` is the active CI gate; this test is the
/// regression detector for the strict bar.
#[test]
#[ignore = "Plan 06-06: D-12 EvalHandle landed structurally; strict 0 requires \
            cubecl client.write API (cubecl 0.10-pre.3 lacks it). Tracked in 06-06-SUMMARY.md."]
fn strict_zero_alloc_after_warmup() {
    use xcfun_core::{FunctionalId, Mode, Vars};

    // Construct an active-functional `xcfun_eval::Functional` directly. We
    // bypass the `xcfun-rs::Functional` facade for this strict test because
    // the facade's `Functional::set` rebuilds `weights` (now a Vec, post
    // D-17) which itself allocates — that is intentional and verified by
    // `no_leak_on_set.rs`. The strict test isolates the EVAL hot path.
    let mut inner = xcfun_eval::Functional::new();
    inner.weights = vec![(FunctionalId::XC_SLATERX, 1.0)];
    inner.vars = Vars::A_B;
    inner.mode = Mode::PartialDerivatives;
    inner.order = 0;

    let inlen = xcfun_eval::Functional::input_length(inner.vars);
    let outlen = xcfun_eval::Functional::output_length(inner.vars, inner.mode, inner.order)
        .expect("output_length(A_B, PartialDerivatives, 0) must succeed");

    let input = vec![0.5_f64; inlen];
    let mut output = vec![0.0_f64; outlen];

    // Warm-up: trigger lazy statics (cubecl-cpu OnceLock<CpuClient>, kernel
    // compilation, etc.). Two warm-up calls are sufficient — the first
    // initialises everything, the second confirms the warm path is hit.
    inner.eval(&input, &mut output).unwrap();
    inner.eval(&input, &mut output).unwrap();

    // Snapshot AFTER warm-up. Subsequent evals must add ZERO allocations.
    let snap = ALLOC_COUNT.load(Ordering::SeqCst);

    for _ in 0..100 {
        inner.eval(&input, &mut output).unwrap();
    }

    let delta = ALLOC_COUNT.load(Ordering::SeqCst) - snap;
    assert_eq!(
        delta,
        0,
        "STRICT zero-alloc breached: {} allocations across 100 eval calls \
         (~{:.1} allocs/eval). The cubecl-cpu run_launch substrate is still \
         calling client.create_from_slice / client.empty per call.",
        delta,
        delta as f64 / 100.0,
    );
}
