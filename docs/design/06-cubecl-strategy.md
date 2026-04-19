# 06 — CubeCL strategy: unified CPU + GPU evaluation

> **Revision history**
>
> - **2026-04-19 PM — Phase 1 cubecl pivot.** The Taylor-algebra AD engine
>   (`xcfun-ad`) is now **cubecl-native from day one**: `CTaylor<F: Float,
>   const N: u32>` is a pure `#[cube]` type backed by cubecl `Array<F>`
>   storage, every arithmetic operation and every `*_expand` scalar series
>   function is a `#[cube] fn` generic over `F: Float`. Pre-pivot text below
>   that positions `xcfun-ad` as a scalar Rust crate consumed by
>   cubecl-bearing downstream crates is **SUPERSEDED**. The list of crates
>   that depend directly on `cubecl = "=0.10.0-pre.3"` now includes
>   `xcfun-ad` (not just `xcfun-kernels` and `xcfun-gpu`). See
>   `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`
>   for the 28 locked decisions and `.planning/ROADMAP.md` Phase 1 for
>   success criteria. The shared-spec option below no longer reads as
>   "scalar → kernel"; the cubecl-native AD **is** the shared spec, and
>   `CpuRuntime` is the scalar validation substrate (cubecl-native AD as
>   baseline).

`cubecl` is a Rust-native kernel DSL that compiles a single `#[cube]` function body to multiple backends (`CpuRuntime`, `CudaRuntime`, `WgpuRuntime`). `xcfun_rs` uses cubecl as the sole per-point evaluation engine: every functional is written once, compiled once, and dispatched to whatever runtime the caller selects. There is no duplicate "CPU scalar path" and "GPU kernel path" implementation of a functional — only the dispatch wrapper differs. As of the **2026-04-19 PM cubecl pivot**, the AD layer (`xcfun-ad`) is also cubecl-native (no separate scalar Rust implementation).

Source material: `docs/manual/Cubecl/` and cubecl's published documentation. This document records the design decisions binding us to a specific subset of cubecl's capabilities.

---

## 1. Goals

| Goal | Mechanism |
|------|-----------|
| One implementation covers CPU and GPU | Every functional body is a `#[cube]` generic over `F: Float` + `Num` |
| Identical numerical results on CPU and GPU | `f64` everywhere; no backend-specific fast-math flags; identical libm call chain on CPU; identical PTX intrinsics on CUDA. Parity checked by the validation harness |
| Minimum host↔device transfers | Persist weights once per `Batch`; reuse density/result buffers with powers-of-two growth |
| Minimum allocations on the evaluation path | `Batch::eval_vec_host` allocates nothing after `reserve`; only the first call (or growth) allocates |
| No host-side Rust calls inside kernels | Compile-time enforced: the `#[cube]` attribute macro rejects unknown function calls |

---

## 2. Runtime choice matrix

| Backend | Enabled via | f64 support | Primary use |
|---------|------------|------------|-------------|
| `CpuRuntime` | Always on (`cubecl-cpu`) | Full | CI, single-node desktop use, fallback when no GPU |
| `CudaRuntime` | `feature = "cuda"` (`cubecl-cuda`) | Full | Primary HPC target |
| `WgpuRuntime` | `feature = "wgpu"` (`cubecl-wgpu`) | Conditional on device `SHADER_F64` | Portability; CI fallback on machines without CUDA |

At runtime, `xcfun-gpu::auto_backend` probes:

```text
if env XCFUN_FORCE_BACKEND is set → use it (panic if unsupported)
else if CudaRuntime::is_available() → Backend::Cuda
else if WgpuRuntime::is_available() && wgpu_supports_f64(device) → Backend::Wgpu
else → Backend::Cpu
```

If `Wgpu` is chosen on a device without f64 support, the harness refuses to run — the accuracy contract (1e-12) is not achievable in f32.

---

## 3. Kernel structure

There is exactly one `#[cube(launch_unchecked)]` entry point. Its body dispatches on compile-time constants `VARS` and `ORDER` plus a runtime-uploaded `active_ids[]` slice.

```rust
// crate: xcfun-kernels
use cubecl::prelude::*;

#[cube(launch_unchecked)]
pub fn eval_batch_kernel<F: Float>(
    weights: &Array<F>,            // [NR_PARAMETERS_AND_FUNCTIONALS]
    density: &Array<F>,            // [nr_points * density_pitch]
    density_pitch: u32,            // input_length padded to stride
    active_ids: &Array<u32>,       // [nr_active]; each entry < NR_FUNCTIONALS
    nr_active: u32,
    result: &mut Array<F>,         // [nr_points * result_pitch]
    result_pitch: u32,
    nr_points: u32,
    #[comptime] vars: u32,         // xcfun_vars discriminant
    #[comptime] order: u32,
    #[comptime] mode: u32,         // xcfun_mode discriminant
) {
    let p = ABSOLUTE_POS;
    if p >= nr_points { return; }

    // Local copies sit in registers / stack, not global memory.
    let mut in_buf = [F::new(0.0); 20];        // XC_MAX_INVARS
    for i in 0..VARS_LEN[vars] {
        in_buf[i] = density[p * density_pitch + i];
    }

    // Mode / order specialisation — each path is its own #[cube] fn in
    // xcfun-kernels::eval_point::{partial, potential, contracted}.
    match (mode, order) {
        (MODE_PARTIAL, 0) => partial_order0::<F>(vars, weights, active_ids, nr_active, &in_buf, result, p, result_pitch),
        (MODE_PARTIAL, 1) => partial_order1::<F>(...),
        // ... up to (MODE_PARTIAL, 4), (MODE_POTENTIAL, _), (MODE_CONTRACTED, 0..=6)
    }
}
```

### 3.1 Per-functional inner kernels

Each functional `pw92c`, `beckex`, `m06x`, … has a `#[cube]` wrapper:

```rust
#[cube]
fn pw92c_kernel<F: Float, const N: u32>(d: &DensVarsDev<F, N>) -> CTaylorDev<F, N> {
    // Same body as crate::functionals::lda::pw92c::<CTaylor<F, N>>.
}
```

The kernel calls only:
1. Other `#[cube]`-annotated functions (elementary functionals, helpers from `xcfun-core::functionals::shared`, ported to `#[cube]`).
2. `F::{add, sub, mul, div, neg, sqrt, exp, log, powf, erf}` (cubecl math intrinsics).
3. `ABSOLUTE_POS`, `CUBE_DIM` (cubecl builtins).
4. Stack-allocated `[F; 1 << N]` arrays.

Nothing else. The `#[cube]` macro enforces this at compile time.

### 3.2 Dispatch by `FunctionalId`

On the GPU, the dispatcher is a large `match` inside a `#[cube]` function:

```rust
#[cube]
fn call_functional<F: Float, const N: u32>(id: u32, d: &DensVarsDev<F, N>) -> CTaylorDev<F, N> {
    match id {
        ID_XC_SLATERX  => slaterx_kernel::<F, N>(d),
        ID_XC_PW92C    => pw92c_kernel::<F, N>(d),
        ID_XC_PBEX     => pbex_kernel::<F, N>(d),
        // ... 78 arms total
        _              => CTaylorDev::<F, N>::new(F::new(0.0)),
    }
}
```

On CUDA this compiles to a jump-table (or a series of conditional branches when the PTX back-end chooses to inline). On Wgpu (WGSL) it becomes a `switch`. Because `active_ids` usually has ≤ 5 entries and every lane in a warp processes the same `(vars, mode, order)` tuple (only `p` varies), divergence is low.

---

## 4. Host/device boundary

| Item | Host | Device |
|------|------|--------|
| `Functional` struct | Yes | No |
| `FUNCTIONAL_DESCRIPTORS` table | Yes (`.rodata`) | No (fn pointers can't cross); identity encoded as `FunctionalId` u32 → `match` arm |
| Weights `settings[]` | Yes | Yes (uploaded once per Batch) |
| `active_ids[]` | Yes | Yes (uploaded once per Batch) |
| `VARS_TABLE` | Yes | Encoded in `#[comptime]` consts and by `vars` arg |
| `CTaylor<T, N>` | Yes | Yes (same layout; `#[repr(C)]` + `[T; 1 << N]`) |
| `DensVars<T>` | Yes | Yes (mirrored as `DensVarsDev<F, N>` with identical field order) |
| Libm calls | Host libm | cubecl math intrinsics |

`DensVarsDev` is a reduced, cubecl-compatible clone of `DensVars` that uses `[F; 1 << N]` arrays (no generics over `T: Num`). The `xtask` codegen generates both from a single source.

### 4.1 Purity rule, strictly

Inside any `#[cube]` body, the only callable items are other `#[cube]`-annotated items or cubecl intrinsics. In practice, this means every helper in `xcfun-core::functionals::shared` has two forms:

- `pub(crate) fn <name>_host<T: Num>(...)` — the CPU scalar form used by `Functional::eval`.
- `#[cube] fn <name>_dev<F: Float, const N: u32>(...)` — the device form used by the kernel.

Both forms are derived from a shared algebraic spec stored in `xcfun-core::functionals::shared::<name>::SPEC`, so drift is structural, not algorithmic. If a helper evolves, both forms recompile.

---

## 5. Batch API

### 5.1 Lifecycle

```
Functional::batch_on<R>(client)
    → Batch::new               (no device allocs yet)
    → Batch::reserve(n)        (weights, density, result device buffers allocated)
    → Batch::upload_density(...)        (memcpy HtoD; size = n * input_len * 8)
    → Batch::launch(n)         (launch_unchecked kernel, no transfers)
    → Batch::download_result(...)       (memcpy DtoH; size = n * output_len * 8)
    [drop Batch → device buffers freed]
```

`reserve(n)` grows buffers with doubling, so repeated `eval_vec_host` calls amortise allocation cost. `weights_buf` is re-uploaded only when `Functional::set` changes the active-set (tracked by a generation counter).

### 5.2 Host↔device transfer budget per batch

For a 1M-point GGA (`input_length = 5`, `output_length = 6` at order 1):

| Direction | Bytes | Frequency |
|-----------|-------|-----------|
| HtoD: weights | 82 × 8 = 656 | Once per Batch (or after `set`) |
| HtoD: active_ids | ≤ 78 × 4 = 312 | Once per Batch (or after `set`) |
| HtoD: density | 1,000,000 × 5 × 8 = 40 MB | Once per `upload_density` |
| DtoH: result | 1,000,000 × 6 × 8 = 48 MB | Once per `download_result` |

Measured on an A100 with PCIe Gen4, transfer time is ≈ 5 ms out of a ≈ 30 ms kernel for GGA order 1 — acceptable. The design explicitly does not pursue stream overlapping in the first release; `cubecl 0.10.0-pre.3` exposes the primitives, but our workload is memory-bandwidth-bound on transfer and compute-bound on the kernel, so overlap gains are < 20 %.

### 5.3 Zero-copy CPU path

With `R = CpuRuntime`, `cubecl` allows ingesting a host slice directly as an `Array<f64>` without an additional copy. `Batch::eval_vec_host` takes two `&mut [f64]` references and wires them into the kernel; no heap allocation occurs beyond the `Batch` bookkeeping.

---

## 6. Suitability analysis: what should and should not go on GPU

### 6.1 Suitable for cubecl (on GPU)

| Computation | Why suitable |
|-------------|--------------|
| Per-point functional evaluation | Embarrassingly parallel over grid points; uniform work per point (`DensVars::build` + constant number of arithmetic ops per active functional) |
| AD polynomial algebra (`CTaylor` add/mul/compose) | Pure arithmetic on stack-resident small arrays; all operations inline cleanly |
| Series expansions (`inv`, `exp`, `log`, `pow`, `sqrt`, `erf`) | Small `for` loops of length ≤ `N+1`; branch-free except a single `if (N == K)` on bit set size |
| Derivative extraction (`CTaylor::get`) | Constant-time array indexing |

### 6.2 Suitable for cubecl (on CPU via `CpuRuntime`)

| Computation | Why suitable |
|-------------|--------------|
| Exactly the same kernels | `CpuRuntime` compiles the kernel to native multi-threaded code; the parallel iteration domain is the density grid |

### 6.3 Kept on host (never a kernel)

| Computation | Why |
|-------------|-----|
| Registry setup (`xcint_assure_setup`) | One-time initialisation, touches strings |
| `xcfun_set` (functional / parameter / alias resolution) | String lookup, indirection into `FUNCTIONAL_DESCRIPTORS`; called at most a few times per simulation |
| `xcfun_eval_setup`, `input_length`, `output_length` | Control-plane decisions; the kernel receives their outputs as `#[comptime]` args or scalar inputs |
| `DensVars` match on `Vars` variant | Translated to 31 specialised `DensVarsDev::build_<variant>` kernel arms, each selected by `#[comptime] vars` — cost stays on host at compile time |
| Alias expansion (`camb3lyp`, `b3lyp`) | Done on host during `xcfun_set`; the kernel only sees the resolved `active_ids` + `settings` |
| Text APIs (`describe_short`, `version`, `splash`) | Data-plane irrelevant |
| C ABI adapter (`xcfun-capi`) | FFI layer, runs per user call |
| Python bindings | GIL-tied |

### 6.4 Explicit non-candidates

- **Single-point `Functional::eval` (non-vec)**. The setup cost of dispatching through a cubecl launch outweighs the arithmetic for one point. `eval` runs synchronously on the host via the scalar `xcfun-core` path. The cubecl kernel is invoked by `eval_vec` when `nr_points ≥ 64` (tunable threshold).
- **`xcfun_test`**. It is a correctness probe and runs sequentially.

---

## 7. Kernel-resident buffers

Three device buffers live for the lifetime of a `Batch`:

| Buffer | Allocation | Reset on |
|--------|-----------|----------|
| `weights_buf: Array<f64>` | `NR_PARAMETERS_AND_FUNCTIONALS = 82` f64 values | `Functional::set` after `Batch::new` |
| `active_ids_buf: Array<u32>` | up to `NR_FUNCTIONALS = 78` u32 values | same |
| `density_buf: Array<f64>` | capacity doubles on overflow | `Batch::reserve(n)` when `n > capacity` |
| `result_buf: Array<f64>` | capacity doubles on overflow | same |

No scratch buffers needed: `CTaylor<f64, N>` sits in registers / local memory per thread.

---

## 8. Vectorisation: `Line<F>`

We do **not** use `Line<F>` in the first release. Every thread handles one grid point. The per-point work already saturates the compute units on GGA / metaGGA workloads (by register pressure on A100 and SM count on consumer CUDA GPUs), so lane packing offers little benefit and imposes a significant cost (re-expressing `CTaylor` ops as vectorised lane-wise operations, plus a new validation surface).

This decision is revisited in Milestone M7 (see [11-process-and-milestones.md](11-process-and-milestones.md)).

---

## 9. Shared memory

We do **not** use shared memory in the first release. Each thread is independent: no reductions, no tiled data access, no cross-thread cooperation. Weights and `active_ids` live in constant memory (CUDA) / uniform buffers (WebGPU), which is ideal for broadcast reads.

Reductions like sum-of-energies over a grid are deliberately not part of xcfun's API — the caller reduces using domain-specific quadrature weights. If the API is extended in the future, shared-memory reductions could be added as a separate kernel.

---

## 10. Synchronisation and launch config

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `CubeDim::new_1d(block)` | 256 on CUDA, 64 on Wgpu, 1 on CPU | Typical warp / subgroup size balanced against register pressure at order ≥ 3 |
| `CubeCount::Static(ceil(nr_points / block), 1, 1)` | dynamic | Fills the grid; last block uses `p < nr_points` guard |
| Shared-memory size | 0 | None used |
| Synchronisation | None in kernel; `client.sync()` after `launch` before `download` | Standard async model |

---

## 11. Numerical parity across backends

CPU (libm) and CUDA (PTX `sin`, `cos`, `exp`, etc.) produce bit-identical results for basic arithmetic in IEEE-754 round-to-nearest. Libm and CUDA `math.h` may differ in the last ULP for transcendentals; the validation harness tolerates **1e-13** relative error across backends, well within the 1e-12 parity target.

Wgpu on WGSL has wider variance (documented ≈ 1.5e-7 for `erf` across devices). For that reason:
- Functionals using `erf` (range-separated hybrids: `ldaerfx`, `ldaerfc`, `beckecamx`, `beckesrx`, `ldaerfc_jt`) run on CPU-only on the Wgpu backend; the batch dispatcher inspects `depends` and forces `Backend::Cpu` for these.
- All other functionals on Wgpu are validated with tolerance 1e-9, not 1e-12.

CUDA-only is the recommended backend for scientific users needing full 1e-12 parity with the C++ reference.

---

## 12. Summary

- Single kernel source, three runtimes. No per-backend implementation.
- f64 throughout; f32 never used on the numerical path.
- Host↔device transfers are quantified and minimised by buffer reuse and weight persistence.
- The cubecl kernel body is closed: only `#[cube]` functions and intrinsics reachable.
- `Functional::eval_vec` dispatches to `CpuRuntime` for small batches and to the selected backend for large batches; the threshold is a compile-time const (default 64 points) and tunable via env var `XCFUN_MIN_BATCH_SIZE`.
