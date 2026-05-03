---
phase: 06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu
plan: 06
type: execute
wave: 8
depends_on:
  - 06-05
  - 06-01
files_modified:
  - crates/xcfun-rs/src/functional.rs
  - crates/xcfun-eval/src/functional.rs
  - crates/xcfun-eval/src/for_tests.rs
  - crates/xcfun-eval/src/lib.rs
  - crates/xcfun-kernels/src/dispatch.rs
  - crates/xcfun-rs/tests/zero_alloc_strict.rs
  - crates/xcfun-rs/tests/no_leak_on_set.rs
  - crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs
autonomous: true
requirements:
  - RS-07
  - RS-10
  - KER-04
must_haves:
  truths:
    - "Strict zero-alloc per-point form: Functional::eval after eval_setup performs strict 0 heap allocations per call (counted via CountingAllocator)."
    - "Pre-allocated reusable handle: UnsafeCell<EvalHandle> in xcfun-rs::Functional holds input_buf + out_buf + dens_vars sized at eval_setup time; reused across all eval calls (D-12 contract)."
    - "Functional remains Send + Sync (RS-10); `unsafe impl Sync` carries doc-comment 'racy if called concurrently on the same instance — clone or wrap in Mutex' (D-12)."
    - "Phase 5 Box::leak in sync_weights_from_settings is REMOVED (D-17); xcfun-eval::Functional::weights field changes from `&'static [(FunctionalId, f64)]` to `Vec<(FunctionalId, f64)>`; no leak per `set` call."
    - "for_tests::cpu_client() promoted from `for_tests` to production module (per D-12 + RESEARCH §"Existing pattern: OnceLock<R::Client>")."
    - "DensVars-driven dispatch (D-18) lifts Phase-5 D-14 dispatch-table constraint: b3lyp / camb3lyp / bp86 (mixed LDA+GGA aliases) eval in-process via `Functional::eval`; LDA kernels can launch into vars=A_B_GAA_GAB_GBB Vars subset when their Dependency::DENSITY ⊆ vars_dep_mask."
    - "static_assertions::assert_impl_all!(Functional: Send, Sync) compile gate continues to GREEN."
  artifacts:
    - path: "crates/xcfun-rs/src/functional.rs"
      provides: "UnsafeCell<EvalHandle> reusable handle + zero-alloc eval path"
      contains: "UnsafeCell"
    - path: "crates/xcfun-eval/src/functional.rs"
      provides: "weights: Vec<(FunctionalId, f64)>; no Box::leak"
      contains: "weights: Vec"
    - path: "crates/xcfun-eval/src/lib.rs"
      provides: "for_tests::cpu_client promoted to production substrate module"
      contains: "pub mod for_tests"
    - path: "crates/xcfun-kernels/src/dispatch.rs"
      provides: "DensVars-driven dispatch (D-18): kernel Dependency mask vs vars_dep_mask subset matching"
      contains: "Dependency::DENSITY"
    - path: "crates/xcfun-rs/tests/zero_alloc_strict.rs"
      provides: "CountingAllocator-based strict 0-alloc test (delta == 0 every call)"
      contains: "AtomicUsize\\|CountingAllocator"
    - path: "crates/xcfun-rs/tests/no_leak_on_set.rs"
      provides: "Allocator-counting test: no Box::leak-induced unfreed allocs across 100 set() calls"
      contains: "Box::leak\\|fetch_add\\|alloc_count"
    - path: "crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs"
      provides: "b3lyp / camb3lyp / bp86 in-process eval test (D-18 contract)"
      contains: "b3lyp\\|camb3lyp\\|bp86"
  key_links:
    - from: "crates/xcfun-rs/src/functional.rs::Functional"
      to: "std::cell::UnsafeCell<EvalHandle>"
      via: "Interior mutability for &self.eval()"
      pattern: "UnsafeCell"
    - from: "crates/xcfun-eval::Functional::weights (Vec)"
      to: "Phase 5 sync_weights_from_settings (Box::leak removed)"
      via: "Direct Vec<(FunctionalId, f64)> assignment"
      pattern: "weights\\s*=\\s*active"
    - from: "crates/xcfun-kernels/src/dispatch.rs::dispatch_kernel"
      to: "Phase 5 D-14 alias substitution constraint (b3lyp / camb3lyp / bp86)"
      via: "Dependency-mask subset dispatch (D-18)"
      pattern: "depends_on_density_only"
---

<objective>
Close out the three Phase 5 → Phase 6 substrate forwards (per `.planning/STATE.md`):

1. **Strict zero-alloc per-point form** (D-12; RS-07 hardening) — Phase 5 D-13 forwarded the cubecl-cpu `create_from_slice` per-launch cost (~287 allocs/eval) to Phase 6. Land the **pre-allocated reusable handle** in `xcfun-rs::Functional`: private `UnsafeCell<EvalHandle>` with `input_buf` + `out_buf` + `dens_vars` sized at `eval_setup` time and reused across all subsequent `eval` calls. Strict bar: 0 allocs/eval after `eval_setup`. `unsafe impl Send / Sync` for `Functional` preserves Phase 5 RS-10; doc-comment "racy if called concurrently on the same instance — clone or wrap in Mutex" per D-12 explicit. **for_tests::cpu_client() promoted to production module** (still under `for_tests` directory but no longer `#[cfg(feature = "testing")]`-gated; consumed by the eval path).

2. **Phase 5 weights `Box::leak` refactor** (D-17) — Replace `xcfun-eval::Functional::weights: &'static [(FunctionalId, f64)]` (currently leaked via `Box::leak` once per `set` call in `sync_weights_from_settings` per Plan 05-01 commit) with `weights: Vec<(FunctionalId, f64)>`. `Vec` is `Send + Sync` so RS-10 preserved. All read-sites that iterate `for (fid, w) in self.weights.iter()` keep working unchanged.

3. **LDA-vars=6 / DensVars-driven dispatch** (D-18) — Currently `dispatch_kernel` only has LDA at `vars=A_B (2)` and GGA at `vars=A_B_GAA_GAB_GBB (6)` (per `.planning/STATE.md` Phase 5 caveats D-14 rows 4 + 9). Mixed-LDA+GGA aliases (b3lyp = LDA-VWN5C + GGA-Becke + GGA-LYP; camb3lyp; bp86) currently route through C++ validation harness only. Add **DensVars-driven dispatch**: a kernel's `Dependency` mask determines which Vars subset arms it can launch into. LDA kernel → Vars subset where `Dependency::DENSITY ⊆ vars_dep_mask`. GGA kernel → Vars subset where `Dependency::DENSITY | Dependency::GRADIENT ⊆ vars_dep_mask`. The dispatcher computes the subset at `eval_setup` time. Resolves Plan 05-04 D-14 rows 4 + 9 forward.

Per RESEARCH Pitfall 8: the Phase 5 form-(b) "head/tail mean stability" test asserted average delta == 0 across many calls (allowing one-time setup allocations). Strict form (delta == 0 every call after warm-up) requires removing every per-eval `Vec::new` / `Box::new` from the eval path. Plan 06-06 must include a **counting-allocator full trace** sweep, not just a delta-check, to surface every remaining heap allocation in the cubecl-cpu launch path.

Purpose: Final Phase 6 polish. After this plan, Phase 6 sign-off prep (Plans 06-N1/N2/N3 cleanup) can run against a strict zero-alloc, leak-free, dispatch-complete `Functional`. RS-07 contract (zero heap allocation on the success path) verified.

Output: UnsafeCell<EvalHandle> pre-allocated buffers; weights as Vec; for_tests promoted; DensVars-driven dispatch wiring; 3 new tests (zero_alloc_strict, no_leak_on_set, lda_gga_alias_dispatch) all GREEN.
</objective>

<execution_context>
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/workflows/execute-plan.md
@/home/chemtech/workspace/xcfun_rs/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/home/chemtech/workspace/xcfun_rs/.planning/PROJECT.md
@/home/chemtech/workspace/xcfun_rs/.planning/ROADMAP.md
@/home/chemtech/workspace/xcfun_rs/.planning/STATE.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-VALIDATION.md
@/home/chemtech/workspace/xcfun_rs/.planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md
@/home/chemtech/workspace/xcfun_rs/CLAUDE.md
@crates/xcfun-rs/src/functional.rs
@crates/xcfun-rs/tests/zero_alloc.rs
@crates/xcfun-rs/tests/send_sync.rs
@crates/xcfun-eval/src/functional.rs
@crates/xcfun-eval/src/for_tests.rs
@crates/xcfun-kernels/src/dispatch.rs
@crates/xcfun-eval/tests/alias_canary.rs

<interfaces>
<!-- Existing types/functions to refactor. -->

From crates/xcfun-rs/src/functional.rs (current — Phase 5 baseline; Plan 05-01):
```rust
fn sync_weights_from_settings(&mut self) {
    const UPSTREAM_FUNCTIONAL_COUNT: usize = 78;
    let mut active: Vec<(FunctionalId, f64)> = Vec::new();
    for fd in FUNCTIONAL_DESCRIPTORS.iter() {
        let idx = fd.id as usize;
        if idx >= UPSTREAM_FUNCTIONAL_COUNT { continue; }
        let w = self.0.settings[idx];
        if w != 0.0 { active.push((fd.id, w)); }
    }
    // Box::leak the slice to obtain `&'static [(FunctionalId, f64)]`.
    // Phase 6 D-17: replace with `weights: Vec<...>` and drop the leak.
    let leaked: &'static [(FunctionalId, f64)] = Box::leak(active.into_boxed_slice());
    self.0.weights = leaked;
}
```

From crates/xcfun-rs/tests/zero_alloc.rs (Phase 5 fall-back form b — extend pattern):
```rust
struct CountingAllocator;
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator;
```

From RESEARCH Pattern 2 (D-12 reusable handle):
```rust
pub struct Functional {
    inner: xcfun_eval::Functional,
    handle: UnsafeCell<EvalHandle>,
}
struct EvalHandle {
    input_buf: Option<cubecl::Handle>,    // sized at eval_setup
    out_buf:   Option<cubecl::Handle>,
    gen:       u64,
}
unsafe impl Send for Functional {}
unsafe impl Sync for Functional {}  // racy if shared concurrently
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Drop Box::leak in sync_weights_from_settings (D-17) + UnsafeCell<EvalHandle> reusable handle (D-12) + promote for_tests::cpu_client to prod</name>
  <files>crates/xcfun-rs/src/functional.rs, crates/xcfun-eval/src/functional.rs, crates/xcfun-eval/src/for_tests.rs, crates/xcfun-eval/src/lib.rs, crates/xcfun-rs/tests/zero_alloc_strict.rs, crates/xcfun-rs/tests/no_leak_on_set.rs</files>
  <read_first>
    - crates/xcfun-rs/src/functional.rs (full file — find sync_weights_from_settings + eval method)
    - crates/xcfun-rs/tests/zero_alloc.rs (CountingAllocator pattern; current fall-back-(b) form)
    - crates/xcfun-rs/tests/send_sync.rs (assert_impl_all! gate to verify Send + Sync preserved)
    - crates/xcfun-eval/src/functional.rs (find weights field declaration; change type)
    - crates/xcfun-eval/src/for_tests.rs (cpu_client OnceLock<CpuClient> pattern — verify before promoting)
    - crates/xcfun-eval/src/lib.rs (verify pub mod for_tests; remove #[cfg(feature = "testing")] gate)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-06" (lines 86-93, 715-770)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-RESEARCH.md §"Pattern 2" + §"Pitfall 8"
    - .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md D-13 (zero-alloc fall-back form (b) → strict here)
  </read_first>
  <behavior>
    - Test 1 (RED first — `tests/zero_alloc_strict.rs`): set up XC_SLATERX functional. Warm up: call `eval(...)` once (allocations during cubecl init are expected). Then snapshot `ALLOC_COUNT`. Call `eval(...)` 100 more times. Assert `ALLOC_COUNT - snapshot == 0` (strict 0 allocs/eval, NOT mean stability).
    - Test 2 (RED first — `tests/no_leak_on_set.rs`): set up Functional. Snapshot `ALLOC_COUNT - DEALLOC_COUNT` (i.e., net leaked bytes). Call `set("slaterx", 1.0)` 100 times with different values. Assert `(ALLOC_COUNT - DEALLOC_COUNT) - snapshot == 0` (no Box::leak-induced unfreed allocs).
    - Test 3: Phase 5 `assert_impl_all!(Functional: Send, Sync)` continues to compile (Send + Sync preserved by D-12 + D-17).
    - All RED before refactor; GREEN after.
  </behavior>
  <action>
**Step A — Refactor `crates/xcfun-eval/src/functional.rs` `weights` field (D-17):**

Find the field declaration (likely `pub weights: &'static [(FunctionalId, f64)]`). Change to:
```rust
pub struct Functional {
    pub settings: [f64; 82],
    pub weights: Vec<(FunctionalId, f64)>,   // CHANGED from &'static [(FunctionalId, f64)] (D-17)
    pub vars: Vars,
    pub mode: Mode,
    pub order: i32,
    pub settings_gen: u64,        // From Plan 06-02
    // ... other fields ...
}
```

Update default initialiser (`Functional::new()`):
```rust
impl Functional {
    pub fn new() -> Self {
        Self {
            settings: [0.0; 82],
            weights: Vec::new(),    // was: &[]
            vars: Vars::Unset,
            mode: Mode::Unset,
            order: -1,
            settings_gen: 0,
            // ... etc ...
        }
    }
}
```

Verify all read-sites — search workspace for `weights.iter()` / `weights.len()` / `weights[k]` and confirm they work unchanged on `Vec<...>` (they all do; `Vec` Deref's to `&[]`).

**Step B — Refactor `crates/xcfun-rs::sync_weights_from_settings` to drop Box::leak:**

Per Phase 5 baseline (Plan 05-01 commit referenced in PATTERNS.md lines 720-735):
```rust
// OLD:
fn sync_weights_from_settings(&mut self) {
    let mut active: Vec<(FunctionalId, f64)> = Vec::new();
    for fd in FUNCTIONAL_DESCRIPTORS.iter() {
        let idx = fd.id as usize;
        if idx >= UPSTREAM_FUNCTIONAL_COUNT { continue; }
        let w = self.0.settings[idx];
        if w != 0.0 { active.push((fd.id, w)); }
    }
    let leaked: &'static [(FunctionalId, f64)] = Box::leak(active.into_boxed_slice());
    self.0.weights = leaked;
}

// NEW (D-17):
fn sync_weights_from_settings(&mut self) {
    self.0.weights.clear();   // re-use existing Vec capacity (no alloc unless growing)
    for fd in FUNCTIONAL_DESCRIPTORS.iter() {
        let idx = fd.id as usize;
        if idx >= UPSTREAM_FUNCTIONAL_COUNT { continue; }
        let w = self.0.settings[idx];
        if w != 0.0 { self.0.weights.push((fd.id, w)); }
    }
}
```

`Vec::clear` does not deallocate — capacity preserved. `Vec::push` reallocates only when growing past capacity. After the first `set()` call that pushes 82 elements, no further allocation.

**Step C — Promote `crates/xcfun-eval/src/for_tests.rs::cpu_client()` to production:**

Plan 06-02 already re-exports cpu_client from xcfun-gpu. This task removes the `#[cfg(feature = "testing")]` gate (if present) and makes `cpu_client` always-available in `xcfun-eval`. Per `crates/xcfun-eval/src/lib.rs`:

```rust
// OLD: pub mod for_tests;  // gated #[cfg(any(test, feature = "testing"))]
// NEW: pub mod for_tests;  // always available; cpu_client is the production CPU substrate
```

Move the `for_tests` module if needed; the file path can stay the same. Add a doc comment noting the promotion:
```rust
//! Production CPU substrate. Phase 6 Plan 06-06 D-12 promoted from `for_tests`-gated
//! to always-available because xcfun-rs::Functional's reusable handle (UnsafeCell<EvalHandle>)
//! depends on cpu_client() at eval time, not just at test time.
```

**Step D — Add UnsafeCell<EvalHandle> to xcfun-rs::Functional (D-12):**

```rust
// crates/xcfun-rs/src/functional.rs
use std::cell::UnsafeCell;
use cubecl::prelude::*;
use cubecl_cpu::CpuRuntime;

/// Per-launch reusable buffer set, sized at eval_setup time.
struct EvalHandle {
    /// cubecl input buffer; capacity = max input_length over all eval_setup calls.
    input_buf: Option<cubecl::Handle>,
    /// cubecl output buffer; same capacity policy.
    out_buf:   Option<cubecl::Handle>,
    /// Pre-allocated DensVarsDev<f64> buffer.
    dens_vars: Option<cubecl::Handle>,
    /// Cached generation counter; bumped on `set()` (Plan 06-02 D-15).
    cached_gen: u64,
}

impl EvalHandle {
    fn new() -> Self {
        Self { input_buf: None, out_buf: None, dens_vars: None, cached_gen: 0 }
    }
    fn ensure_sized(&mut self, client: &CpuClient, inlen: usize, outlen: usize) {
        // Allocate fresh handles only on first call OR when size grew.
        match &self.input_buf {
            Some(_) => { /* check size; reuse if adequate */ }
            None => { self.input_buf = Some(client.empty(inlen * 8)); }
        }
        // ... same for out_buf, dens_vars ...
    }
}

pub struct Functional {
    inner: xcfun_eval::Functional,
    /// Phase 6 D-12 reusable handle. Interior mutability via UnsafeCell because
    /// eval(&self, ...) must not require &mut self (RS-07 + RS-10 contract).
    /// Concurrent eval on the same Functional is RACY — clone or wrap in Mutex.
    handle: UnsafeCell<EvalHandle>,
}

// Phase 6 D-12: documented "racy if shared". Send + Sync preserved per RS-10.
unsafe impl Send for Functional {}
unsafe impl Sync for Functional {}
```

**Step E — Update `eval()` to reuse handles:**

```rust
impl Functional {
    pub fn eval(&self, input: &[f64], output: &mut [f64]) -> Result<(), XcError> {
        // SAFETY: documented "racy if shared concurrently" per D-12 / RS-10.
        let handle = unsafe { &mut *self.handle.get() };
        let client = xcfun_eval::for_tests::cpu_client();
        let inlen  = self.inner.input_length();
        let outlen = self.inner.output_length()?;
        handle.ensure_sized(client, inlen, outlen);

        // Re-upload weights only when stale (D-15 generation counter).
        if self.inner.settings_generation() != handle.cached_gen {
            // ... write to weights_buf via client.write ...
            handle.cached_gen = self.inner.settings_generation();
        }

        // Write density into pre-allocated input_buf.
        client.write(handle.input_buf.as_ref().unwrap(), bytemuck::cast_slice(input));

        // Launch the kernel via xcfun_eval::Functional::eval_via_kernel
        // (or equivalent reflection). The cubecl-cpu launch_unchecked path
        // re-uses handle.input_buf, handle.out_buf, handle.dens_vars — no
        // per-call create_from_slice (RESEARCH Pitfall 8 fix).
        self.inner.eval_via_handle(client, handle.input_buf.as_ref().unwrap(),
                                   handle.dens_vars.as_ref().unwrap(),
                                   handle.out_buf.as_ref().unwrap())?;

        // Read output back into the caller's slice.
        let out_bytes = client.read_one(handle.out_buf.as_ref().unwrap().clone());
        let out_f64: &[f64] = bytemuck::cast_slice(&out_bytes);
        output.copy_from_slice(&out_f64[..outlen]);

        Ok(())
    }
}
```

(Exact form depends on existing `xcfun-eval::Functional::eval` shape; the executor reads it first and applies the analogous "reusable handle" pattern. Add `eval_via_handle(...)` to xcfun-eval::Functional if needed; OR refactor existing `eval` to take optional pre-allocated handles.)

**Step F — Write RED tests:**

`crates/xcfun-rs/tests/zero_alloc_strict.rs`:
```rust
// Mirrors crates/xcfun-rs/tests/zero_alloc.rs allocator pattern; STRICT form.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use xcfun_rs::Functional;

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
fn strict_zero_alloc_after_warmup() {
    let mut f = Functional::new();
    f.set("slaterx", 1.0).unwrap();
    f.eval_setup(xcfun_core::Vars::A_B, xcfun_core::Mode::PartialDerivatives, 0).unwrap();
    let inlen  = f.input_length();
    let outlen = f.output_length().unwrap();
    let input = vec![0.5_f64; inlen];
    let mut output = vec![0.0_f64; outlen];

    // Warm up (cubecl init, OnceLock<CpuClient>, EvalHandle::ensure_sized first-time alloc).
    f.eval(&input, &mut output).unwrap();
    f.eval(&input, &mut output).unwrap();

    let snapshot = ALLOC_COUNT.load(Ordering::SeqCst);
    for _ in 0..100 {
        f.eval(&input, &mut output).unwrap();
    }
    let delta = ALLOC_COUNT.load(Ordering::SeqCst) - snapshot;
    assert_eq!(delta, 0, "STRICT zero-alloc breached: {} allocs in 100 eval calls", delta);
}
```

`crates/xcfun-rs/tests/no_leak_on_set.rs`:
```rust
//! D-17 — Phase 5 sync_weights_from_settings used Box::leak per `set` call.
//! Plan 06-06 refactors to Vec; this test asserts no leak across 100 `set` calls.
// ... CountingAllocator setup ...
#[test]
fn set_does_not_leak() {
    let mut f = xcfun_rs::Functional::new();
    let snap_alloc = ALLOC_COUNT.load(Ordering::SeqCst);
    let snap_dealloc = DEALLOC_COUNT.load(Ordering::SeqCst);

    for i in 0..100 {
        f.set("slaterx", (i as f64 + 1.0) * 0.01).unwrap();
    }

    let net_leaked = (ALLOC_COUNT.load(Ordering::SeqCst) - snap_alloc) as i64
                   - (DEALLOC_COUNT.load(Ordering::SeqCst) - snap_dealloc) as i64;
    // Vec re-uses capacity after first push; net_leaked should be 0 or 1 (initial Vec capacity bump).
    assert!(net_leaked.abs() <= 1, "Box::leak regression: {} net allocs across 100 set() calls", net_leaked);
}
```

**Step G — Verify zero-alloc gate GREEN:**

```bash
cargo nextest run -p xcfun-rs --test zero_alloc_strict
cargo nextest run -p xcfun-rs --test no_leak_on_set
cargo nextest run -p xcfun-rs --test send_sync   # assert_impl_all gate
```

All exit 0.

**Forbidden:**
- Do NOT use `#[global_allocator]` outside of test files (it must stay in tests/zero_alloc_strict.rs and tests/no_leak_on_set.rs to avoid leaking into production).
- Do NOT add new `Vec::new()` / `Box::new()` calls on the eval path. RESEARCH Pitfall 8 / Assumption A7: Plan 06-06 needs full counting-allocator trace, not just delta-check.
- Do NOT add `format!(...)` to the eval path.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-rs --test zero_alloc_strict --test no_leak_on_set --test send_sync</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "weights:\s*Vec<" crates/xcfun-eval/src/functional.rs` >= 1
    - `grep -c "Box::leak" crates/xcfun-rs/src/functional.rs` == 0 (Phase 5 leak gone per D-17)
    - `grep -c "Box::leak" crates/xcfun-eval/src/functional.rs` == 0
    - `grep -c "UnsafeCell" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "unsafe impl Sync\|unsafe impl Send" crates/xcfun-rs/src/functional.rs` >= 1
    - `grep -c "racy if" crates/xcfun-rs/src/functional.rs` >= 1 (D-12 doc-comment)
    - `grep -E '#\[cfg\(.*testing.*\)\]\s*pub mod for_tests' crates/xcfun-eval/src/lib.rs | wc -l` == 0 (no longer testing-gated)
    - `cargo nextest run -p xcfun-rs --test zero_alloc_strict` exits 0.
    - `cargo nextest run -p xcfun-rs --test no_leak_on_set` exits 0.
    - `cargo nextest run -p xcfun-rs --test send_sync` exits 0 (assert_impl_all preserved).
    - `cargo nextest run --workspace --tests` exits 0 (no regression).
  </acceptance_criteria>
  <done>D-17 Box::leak gone; D-12 UnsafeCell<EvalHandle> reusable handle landed; for_tests::cpu_client promoted to production substrate; 3 zero-alloc / leak / send_sync tests GREEN; no regression in other tests.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: DensVars-driven dispatch (D-18) for mixed-LDA+GGA aliases (b3lyp / camb3lyp / bp86)</name>
  <files>crates/xcfun-kernels/src/dispatch.rs, crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs</files>
  <read_first>
    - crates/xcfun-kernels/src/dispatch.rs (full file — current FunctionalId-keyed if-chain at lines 79+; understand current vars-arm structure)
    - crates/xcfun-eval/tests/alias_canary.rs (Phase 4 alias resolution pattern)
    - crates/xcfun-core/src/registry/descriptors.rs (find FUNCTIONAL_DESCRIPTORS — each has a `dependencies: Dependency` field)
    - crates/xcfun-core/src/types.rs (Vars enum + VARS_TABLE; each Vars has a `vars_dep_mask`)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-PATTERNS.md "Plan 06-06 lda_gga_alias_dispatch.rs" (lines 92-93, 772-779)
    - .planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-CONTEXT.md D-18
    - .planning/phases/05-rust-facade-xcfun-rs-c-abi-xcfun-capi/05-CONTEXT.md D-14 (rows 4 + 9 substituted to bp86 / beckecamx; the constraint Plan 06-06 closes)
  </read_first>
  <behavior>
    - Test 1 (RED first): set up b3lyp via `set("b3lyp", 1.0)`. Call `eval_setup(Vars::A_B_GAA_GAB_GBB, Mode::PartialDerivatives, 1)`. b3lyp is an alias resolving to LDA-VWN5C + GGA-Becke + GGA-LYP — dispatcher needs to launch LDA-VWN5C kernel into the GGA Vars arm (since `Dependency::DENSITY ⊆ A_B_GAA_GAB_GBB.vars_dep_mask`). Call `eval(...)` and assert it returns `Ok(())` (NOT `XcError::NotConfigured` or `XcError::InvalidVars`).
    - Test 2: same shape for `camb3lyp` (range-separated CAM functional with mixed LDA+GGA components).
    - Test 3: same shape for `bp86` (additive 2-GGA-term alias with VWN3C LDA component).
    - Test 4: assert numerical agreement with C++ baseline at strict 1e-13 for all three aliases (verifies the dispatch produces correct output, not just non-error). Use the validation tier-3 fixture grid.
    - All RED before D-18 wiring; GREEN after.
  </behavior>
  <action>
**Step A — Inspect current dispatcher structure:**

```bash
git grep -n "fn run_launch\|comptime!(vars" crates/xcfun-kernels/src/dispatch.rs
git grep -n "Vars::A_B\b" crates/xcfun-eval/src/functional.rs   # find current vars-arm dispatch
```

Per `.planning/STATE.md` Phase 5 caveats D-14: dispatcher constraints in `run_launch` (LDA kernels at vars=2 only; GGA kernels at vars=6 only). Plan 06-06 closes this.

**Step B — Wire DensVars-driven dispatch in `crates/xcfun-kernels/src/dispatch.rs`:**

The current dispatcher likely looks like:
```rust
pub fn run_launch(client: &CpuClient, fid: FunctionalId, vars: Vars, ...) {
    match (fid, vars) {
        (XC_SLATERX,  Vars::A_B)               => slaterx_kernel_launch(...),
        (XC_PBEX,     Vars::A_B_GAA_GAB_GBB)   => pbex_kernel_launch(...),
        // ... 78 × 31 arms — but only LDA × A_B and GGA × A_B_GAA_GAB_GBB exist ...
        _ => return Err(XcError::InvalidVars),
    }
}
```

D-18: a kernel's `Dependency` mask determines which Vars arms it can launch into:
- LDA kernel (`Dependency::DENSITY` only) → LAUNCHABLE in any Vars where `DENSITY ⊆ vars_dep_mask`. Currently dispatcher only has the `A_B` arm; D-18 adds `A_B_GAA_GAB_GBB` arm too (LDA kernel uses only the rho components, ignores gradient inputs).
- GGA kernel (`Dependency::DENSITY | Dependency::GRADIENT`) → LAUNCHABLE in any Vars where `DENSITY|GRADIENT ⊆ vars_dep_mask`. Currently only `A_B_GAA_GAB_GBB`; could extend to `A_B_GAA_GAB_GBB_LAPLA_LAPLB` etc.

Implementation (Discretion: choose the cleanest representation):

```rust
/// Phase 6 D-18 — DensVars-driven dispatch.
///
/// Determines whether a functional kernel can launch into a given Vars arm.
/// Returns true if `kernel_deps ⊆ vars_dep_mask`.
fn kernel_can_launch_in_vars(kernel_deps: Dependency, vars: Vars) -> bool {
    let vars_dep_mask = VARS_TABLE[vars as usize].dep_mask;
    (kernel_deps & vars_dep_mask) == kernel_deps   // kernel_deps ⊆ vars_dep_mask
}
```

Then, for the dispatcher's per-kernel arms, expand from per-(fid, vars) to per-(fid, vars-subset). For LDA kernels, the subset is "all Vars with `Dependency::DENSITY` in their mask" — at minimum `A_B` and `A_B_GAA_GAB_GBB`. Add the missing arms:

```rust
#[cube]
pub fn dispatch_kernel<F: Float>(
    #[comptime] id: u32, d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32,
) {
    if comptime!(id == FunctionalId::XC_SLATERX as u32) {
        crate::functionals::lda::slaterx::slaterx_kernel::<F>(d, out, n);
    }
    // ... etc; ALL 78 arms read DensVarsDev fields by name (e.g. d.n, d.s),
    //     so they ALREADY work for any Vars that supplies those fields. ...
}
```

The `dispatch_kernel` body itself is `Vars`-agnostic — it reads named fields from `DensVarsDev<F>`. The constraint is in `run_launch` (the host-side dispatcher that picks a `comptime` Vars value) which currently has hardcoded (vars, kernel) pairings. Plan 06-06 widens those pairings: every LDA functional gets a launch-arm at `Vars::A_B_GAA_GAB_GBB` in addition to `Vars::A_B`.

```rust
// Pseudo-code for run_launch update:
match vars {
    Vars::A_B => { /* existing LDA arms */ }
    Vars::A_B_GAA_GAB_GBB => {
        // PRE-D-18: only GGA kernels launched here.
        // POST-D-18: LDA kernels CAN launch here too — DensVarsDev<F> provides .n / .s
        //            fields at this Vars (DensVarsDev is a superset).
        match fid {
            // ALL kernels launchable when their Dependency ⊆ A_B_GAA_GAB_GBB.dep_mask.
            // LDA kernels:
            FunctionalId::XC_SLATERX  => slaterx_kernel_launch::<F, vars=A_B_GAA_GAB_GBB>(...),
            FunctionalId::XC_VWN3C    => vwn3c_kernel_launch::<F, vars=A_B_GAA_GAB_GBB>(...),
            FunctionalId::XC_VWN5C    => vwn5c_kernel_launch::<F, vars=A_B_GAA_GAB_GBB>(...),
            FunctionalId::XC_PW92C    => pw92c_kernel_launch::<F, vars=A_B_GAA_GAB_GBB>(...),
            // ... all 11 LDAs ...
            // GGA kernels (existing):
            FunctionalId::XC_PBEX     => pbex_kernel_launch::<F, vars=A_B_GAA_GAB_GBB>(...),
            // ... etc ...
        }
    }
    _ => Err(XcError::InvalidVars),
}
```

**Step C — Functional::eval_setup must compute the "minimum sufficient Vars" for the active functional set:**

When user calls `set("b3lyp", 1.0)`, the active set becomes `{VWN5C (LDA), BECKEX (GGA), LYPC (GGA)}`. The OR of dependencies is `Dependency::DENSITY | Dependency::GRADIENT`. So `eval_setup` must accept `Vars::A_B_GAA_GAB_GBB` (minimum sufficient) and the dispatcher must dispatch each active functional INTO that Vars arm.

In `crates/xcfun-eval/src/functional.rs::eval_setup`:
```rust
pub fn eval_setup(&mut self, vars: Vars, mode: Mode, order: u32) -> Result<(), XcError> {
    let mut required_deps = Dependency::empty();
    for (fid, _w) in self.weights.iter() {
        let descr = &FUNCTIONAL_DESCRIPTORS[*fid as usize];
        required_deps |= descr.dependencies;
    }
    let vars_dep_mask = VARS_TABLE[vars as usize].dep_mask;
    if (required_deps & vars_dep_mask) != required_deps {
        return Err(XcError::InvalidVars { vars, depends: required_deps });
    }
    // ... rest of eval_setup ...
}
```

Then in `eval()`, the dispatcher iterates active functionals and launches each one's kernel into the user-selected `vars` arm (per Step B's expanded run_launch).

**Step D — Write RED test `crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs`:**

```rust
//! D-18 — DensVars-driven dispatch resolves Phase-5 D-14 rows 4+9 forward.
//! Mixed LDA+GGA aliases (b3lyp / camb3lyp / bp86) eval in-process.

use xcfun_rs::Functional;

#[test]
fn b3lyp_dispatches_in_process() {
    let mut f = Functional::new();
    f.set("b3lyp", 1.0).unwrap();
    // b3lyp = LDA-VWN5C + GGA-Becke + GGA-LYP (+ LDA-Slater fraction).
    // eval_setup at A_B_GAA_GAB_GBB must succeed (DensVars-driven dispatch).
    f.eval_setup(xcfun_core::Vars::A_B_GAA_GAB_GBB, xcfun_core::Mode::PartialDerivatives, 1).unwrap();
    let inlen  = f.input_length();
    let outlen = f.output_length().unwrap();
    let input = vec![0.5, 0.5, 0.1, 0.1, 0.1];   // Vars::A_B_GAA_GAB_GBB → 5 inputs
    let mut output = vec![0.0; outlen];
    let r = f.eval(&input, &mut output);
    assert!(r.is_ok(), "b3lyp dispatch failed: {:?}", r);
    // ... assert output[0] is finite + reasonable magnitude ...
}

#[test]
fn camb3lyp_dispatches_in_process() {
    let mut f = Functional::new();
    f.set("camb3lyp", 1.0).unwrap();
    f.eval_setup(xcfun_core::Vars::A_B_GAA_GAB_GBB, xcfun_core::Mode::PartialDerivatives, 1).unwrap();
    // ... test body ...
}

#[test]
fn bp86_dispatches_in_process() {
    let mut f = Functional::new();
    f.set("bp86", 1.0).unwrap();
    f.eval_setup(xcfun_core::Vars::A_B_GAA_GAB_GBB, xcfun_core::Mode::PartialDerivatives, 1).unwrap();
    // ... test body ...
}

#[test]
fn b3lyp_numerical_parity_with_cpp_at_known_point() {
    // Compare against a hand-curated baseline computed via the C++ harness
    // (offline: `validation` binary at vars=A_B_GAA_GAB_GBB; or use the Phase 4
    //  alias_canary.rs known b3lyp baseline if it exists).
    let mut f = Functional::new();
    f.set("b3lyp", 1.0).unwrap();
    f.eval_setup(xcfun_core::Vars::A_B_GAA_GAB_GBB, xcfun_core::Mode::PartialDerivatives, 0).unwrap();
    let input = vec![0.5_f64, 0.5, 0.1, 0.1, 0.1];
    let mut output = vec![0.0; f.output_length().unwrap()];
    f.eval(&input, &mut output).unwrap();
    // Hand-curated C++ baseline for b3lyp at this point:
    let expected = -0.6234567_f64;   // PLACEHOLDER — populate via offline cc run
    approx::assert_relative_eq!(output[0], expected, max_relative = 1e-13);
}
```

Run `cargo nextest run -p xcfun-rs --test lda_gga_alias_dispatch` — must PASS after Step B + C wire dispatch.

**Discretion (CONTEXT.md):** Macro-generation of the cross-product LDA × Vars-subset arms is acceptable to keep ~78 × 2-3 arms = ~200 arm-cases manageable. A `macro_rules! lda_kernel_launch_arms { ... }` keeps the cross-product visible and avoids hand-typing.

**Step E — Verify no regression in tier-2 LDA + GGA quick sweep:**

```bash
cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga' --jobs 4
```

Should still report 0 failures (the 17 known-clean + tier-2 GREEN order 0/1 functionals from Phase 2/3/4). Plan 06-N1 closes the inherited D-19 forwards; Plan 06-06 must not introduce new ones.
  </action>
  <verify>
    <automated>cargo nextest run -p xcfun-rs --test lda_gga_alias_dispatch && cargo nextest run --workspace --tests</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c "kernel_can_launch_in_vars\|vars_dep_mask\|Dependency::DENSITY" crates/xcfun-kernels/src/dispatch.rs` >= 1
    - `crates/xcfun-rs/tests/lda_gga_alias_dispatch.rs` exists with `b3lyp_dispatches_in_process`, `camb3lyp_dispatches_in_process`, `bp86_dispatches_in_process` test functions.
    - `cargo nextest run -p xcfun-rs --test lda_gga_alias_dispatch` exits 0.
    - `cargo nextest run --workspace --tests` exits 0 (no regression).
    - Existing alias_canary.rs (Phase 4) still GREEN: `cargo nextest run -p xcfun-eval --test alias_canary` exits 0.
    - tier-2 LDA+GGA quick sweep still GREEN: `cargo run -p validation --release -- --backend cpu --order 2 --filter 'lda|gga' --jobs 4` exits 0.
  </acceptance_criteria>
  <done>D-18 DensVars-driven dispatch lands; b3lyp / camb3lyp / bp86 eval in-process at strict 1e-13; Phase 5 D-14 dispatch-table constraint resolved.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| `&self.eval()` ↔ shared concurrent `Functional` | Documented racy per D-12; user must clone or wrap in Mutex |
| `UnsafeCell<EvalHandle>` ↔ unsafe impl Send/Sync | Compile-time gate via assert_impl_all + doc-comment carries the contract |
| Vec<(FunctionalId, f64)> ↔ Box::leak history | D-17 removes leak; tests verify across 100 set() calls |

## STRIDE Threat Register

| Threat ID | Severity | Description | Mitigation in this plan |
|-----------|----------|-------------|-------------------------|
| T-06-ALLOC | low | Strict zero-alloc contract regression | tests/zero_alloc_strict.rs asserts delta == 0 across 100 eval calls (CountingAllocator) |
| T-06-LEAK | medium | Phase 5 Box::leak per `set` → unbounded leak across long-running services | D-17 refactor (Step B) + tests/no_leak_on_set.rs verifies ≤ 1 net alloc across 100 set() calls |
| T-06-CONCURRENT-EVAL | medium | Race on shared Functional::eval | D-12 doc-comment "racy if shared concurrently"; UnsafeCell carries the contract; user clones or wraps in Mutex |
| T-06-DISPATCH-MISMATCH | medium | DensVars-driven dispatch could route LDA kernel into Vars where required fields are absent | Step B `kernel_can_launch_in_vars` check enforces `kernel_deps ⊆ vars_dep_mask`; eval_setup returns InvalidVars for mismatches |
| T-06-RS10-BREAKAGE | high | Adding UnsafeCell could break Send + Sync compile gate | tests/send_sync.rs assert_impl_all gate runs in CI; explicit `unsafe impl Send/Sync` with doc-comment |
</threat_model>

<verification>
- All 2 tasks GREEN per their automated commands.
- Full workspace test suite exits 0.
- Phase 5 fall-back zero_alloc.rs may stay or be retired — both tests measure stability; strict form supersedes.
- send_sync.rs compile gate continues to GREEN (RS-10 preserved).
- No new `Box::leak` anywhere in xcfun-* library crates: `grep -rE 'Box::leak' crates/xcfun-ad crates/xcfun-core crates/xcfun-kernels crates/xcfun-eval crates/xcfun-rs crates/xcfun-capi crates/xcfun-gpu | wc -l` == 0.
- xtask gates GREEN.
- tier-2 LDA + GGA quick sweep GREEN at order 2 (no algebraic regression from dispatch widening).
</verification>

<success_criteria>
- ROADMAP Phase 6 success criterion 2 substrate-ready: tier-3 CPU 10k-grid 1e-13 (KER-06) lands cleanly with strict zero-alloc Functional + Vec weights + DensVars-driven dispatch.
- RS-07 strict form satisfied (D-12 reusable handle).
- RS-10 preserved (D-12 + D-17).
- Phase 5 D-13 forward (zero-alloc fall-back form b → strict here) closed.
- Phase 5 D-14 forward (LDA-vars=6 / DensVars-driven dispatch for mixed LDA+GGA aliases) closed.
- Plans 06-N1 / 06-N2 / 06-N3 unblocked: a clean, leak-free, dispatch-complete Functional surface for the D-19 cleanup work.
</success_criteria>

<output>
After completion, create `.planning/phases/06-gpu-backends-batch-lifecycle-xcfun-kernels-xcfun-gpu/06-06-SUMMARY.md` documenting:
- D-12: UnsafeCell<EvalHandle> reusable handle (input_buf + out_buf + dens_vars sized at eval_setup)
- D-17: weights field changed from `&'static [(FunctionalId, f64)]` to `Vec<...>`; Box::leak removed
- for_tests::cpu_client promoted to production substrate (no longer #[cfg(feature = "testing")])
- D-18: DensVars-driven dispatch (kernel_can_launch_in_vars subset matching); LDA × A_B_GAA_GAB_GBB launch arms added; b3lyp / camb3lyp / bp86 eval in-process
- 3 new tests GREEN: zero_alloc_strict (delta == 0), no_leak_on_set (≤ 1 net alloc), lda_gga_alias_dispatch (3 mixed-alias eval cases)
- send_sync.rs compile gate still GREEN
- tier-2 LDA + GGA quick sweep at order 2 still GREEN (no algebraic regression)
- Phase 5 D-13 + D-14 forwards closed in REQUIREMENTS.md / STATE.md
</output>
