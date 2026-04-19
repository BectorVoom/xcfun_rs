# 12 — Design decisions and rejected alternatives

> **Revision history**
>
> - **2026-04-19 PM — Phase 1 cubecl pivot.** D1, D2, D4 are marked
>   **SUPERSEDED** (see per-decision banners below +
>   `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`).
>   The original D1/D2/D4 text is retained below each SUPERSEDED banner so
>   anyone tracing the evolution of the design can see why the substrate
>   shifted from "hand-Rust scalar port" to "cubecl-native `#[cube]` port
>   validated on `CpuRuntime`". Every other decision (D3, D5..D20) is
>   unchanged.

Every architectural choice recorded with rationale and the alternatives we passed on. This document is the project's ADR (architecture decision record).

---

## D1 — Port xcfun's bit-flag Taylor polynomial AD verbatim

> **SUPERSEDED 2026-04-19 PM by Phase 1 cubecl pivot.** The in-house port
> is now cubecl-native: `CTaylor<F: Float, const N: u32>` is a pure
> `#[cube]` type backed by cubecl `Array<F>` storage. The original
> rejection of existing Rust AD crates (`autodiff`/`hyperdual`/`ad`/
> `num-dual`) still stands — no crate replicates xcfun's bit-flag
> multilinear polynomial structure — but the implementation substrate
> shifted from scalar Rust to cubecl. See
> `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md`
> for the 28 locked decisions.

**Decision**: implement `CTaylor<T, N>` with the exact multilinear polynomial representation indexed by `N`-bit variable flags (0 = constant, `VARi = 1 << i`), matching `xcfun-master/src/taylor/ctaylor.hpp`.

**Rationale**: the 1e-12 parity target requires algorithmic identity in the AD core (see [07-accuracy-strategy.md §3](07-accuracy-strategy.md)). The xcfun AD is an unusual design (bit-flag indexed multilinear polynomials) that no third-party Rust crate replicates; only a faithful port preserves the rounding pattern.

**Alternatives rejected**:

| Alternative | Rejected because |
|-------------|------------------|
| `autodiff` / Enzyme | Requires nightly Rust; source-to-source transformation doesn't match xcfun's runtime polynomial approach; numerical equivalence would be hard to guarantee. |
| `hyperdual` | Only supports up to 2nd derivatives. xcfun needs order up to 6. |
| `ad` crate | Tape-based reverse-mode AD. Rounding pattern differs from xcfun. |
| Custom dense coefficient array indexed by multi-index (i1, …, ik) | Works, but forces a different coefficient ordering and different multiply routine; rounding pattern diverges from xcfun; would require per-functional retuning. |
| Sparse `BTreeMap<usize, T>` | Orders of magnitude slower; unnecessary because the dense representation is small (`<= 64` f64) and fits in cache. |

---

## D2 — Custom `Num` trait; reject `num-traits::Float`

> **SUPERSEDED 2026-04-19 PM by Phase 1 cubecl pivot.** The custom `Num`
> trait is **RETIRED**. Cubecl's `Float` trait (from
> `cubecl_core::prelude::Float`) replaces it. All `#[cube] fn` operating
> on `CTaylor` are generic over `F: Float` (not `F: Num`). See
> CONTEXT.md D-09, D-10.

**Decision**: define `xcfun-ad::Num` with `Add`, `Sub`, `Mul`, `Div`, `Neg`, `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan`, and two constants (`ZERO`, `ONE`).

**Rationale**: `num-traits::Float` assumes IEEE-754 scalar semantics and includes `is_nan`, `classify`, `floor`, `ceil` — all of which are meaningless on `CTaylor`. Extending `Float` to polynomials would require unsound default implementations. A narrow, purpose-built trait expresses exactly what the functionals need.

**Alternatives rejected**: `num-traits::Float`; `simba::scalar::Field`; hand-rolled methods on `CTaylor` only. The last was tempting but forces every generic functional body to carry `T: Copy + ...` bounds which are tedious to maintain.

---

## D3 — cubecl as the sole kernel DSL

**Decision**: every per-point functional body is a `#[cube]` function, compiled to `CpuRuntime`, `CudaRuntime`, or `WgpuRuntime` as needed. No duplicate scalar/GPU implementations.

**Rationale**: see [06-cubecl-strategy.md](06-cubecl-strategy.md). The cost of maintaining two parallel implementations of 78 functionals is prohibitive, and drift between them would create hard-to-debug accuracy failures. One source ⇒ one algorithm ⇒ easier parity verification.

**Alternatives rejected**:

| Alternative | Rejected because |
|-------------|------------------|
| Hand-written CUDA C++ via `rust-cuda` | Double maintenance burden; WGSL/Metal portability lost |
| `wgpu` raw with WGSL shaders | Requires writing WGSL separately from Rust; no Rust-native kernel language |
| `cudarc` | NVIDIA-only; eliminates macOS/Metal and Vulkan |
| OpenCL via `ocl` | Declining ecosystem; cubecl covers the same backends with better Rust integration |
| Pure CPU + separate GPU layer | Drift risk; two sources of truth for 78 functionals |

**Note on pre-release pin**: `cubecl` is pinned at `=0.10.0-pre.3`. This is a conscious risk: the API can change. Mitigation: isolate cubecl imports inside `xcfun-kernels` and `xcfun-gpu`; a major version bump triggers the full validation harness.

---

## D4 — f64 everywhere; no f32 path

> **SUPERSEDED 2026-04-19 PM by Phase 1 cubecl pivot.** `f32` is no
> longer "intentionally unimplemented at the AD layer". `xcfun-ad`
> exposes a generic API over cubecl's `Float` trait, so `f32`/`f16`
> *can* be instantiated; the ban on `f32` on the numerical path is
> **ENFORCED UPSTREAM** by `xcfun-rs::Functional` refusing to
> instantiate with any `F != f64`. See CONTEXT.md D-10, D-11.

**Decision**: `Num` is implemented only for `f64` and `CTaylor<f64, N>` (and the cubecl `F: Float` device analogues). No `f32` variant exists on the numerical path.

**Rationale**: 1e-12 relative tolerance is not achievable in f32 (whose unit roundoff is ≈ 6e-8). A mixed f32/f64 API would invite silent accuracy loss.

**Alternatives rejected**: generic-over-scalar `Num` implemented for `f32` (would compile but accuracy-fail); explicit `f64` only (chosen).

---

## D5 — `cubecl-wgpu` backend is best-effort, not primary

**Decision**: `wgpu` feature is available but validated at looser tolerance (1e-9) and skips range-separated functionals that use `erf`.

**Rationale**: Wgpu's `erf` lowering varies across devices by up to 1.5e-7 (documented in cubecl manual, `erf.md`). Wgpu devices without `SHADER_F64` are unsupported for any numerical path.

**Alternatives rejected**: make Wgpu a first-class path at 1e-12 tolerance — infeasible without writing our own high-accuracy `erf` in WGSL, which is out of scope; disable Wgpu entirely — removes a useful portability target.

---

## D6 — Single `#[cube(launch_unchecked)]` entry point with comptime specialisation

**Decision**: one top-level kernel function, specialised at compile time on `(vars, mode, order)` via `#[comptime]` arguments. The 78 functional dispatches remain a `match` over `FunctionalId`.

**Rationale**: keeps the binary size manageable (dozens of kernel variants rather than tens of thousands), while preserving the critical `#[comptime]` specialisation of the outer three parameters (which change how the input array is interpreted and how the output is laid out).

**Alternatives rejected**:

| Alternative | Rejected because |
|-------------|------------------|
| One kernel per `(functional, vars, mode, order)` | 78 × 31 × 3 × 7 = ~50 000 kernel variants; intractable binary size and PTX compile time |
| No specialisation, fully dynamic | Branch-heavy inner loops on GPU; unacceptable warp divergence |
| Specialisation on `(mode, order)` only | Chosen (via a compile-time `match`); vars are specialised because input layout differs enough to justify it |

---

## D7 — Parallel arrays for active functionals, not a `Vec<Active>`

**Decision**: `Functional` stores `active_ids: [FunctionalId; NR_FUNCTIONALS]` and `nr_active_functionals: u8`; weights are in `settings[0..NR_FUNCTIONALS]`.

**Rationale**: inline storage avoids a heap allocation; bounded because there are only 78 functionals. The cost (78 × size_of::<FunctionalId>() = 78 bytes) is trivial.

**Alternatives rejected**: `Vec<ActiveFunctional>` — heap allocation on every `set`; `SmallVec<[_; 8]>` — adds a dependency and inline/heap branching; `[Option<_>; 78]` — wastes layout.

---

## D8 — `bitflags` for `Dependency`, not a hand-rolled u8

**Decision**: `bitflags::bitflags!` generates `Dependency` with `#[repr(transparent)]` over `u8`.

**Rationale**: standard ecosystem type, ergonomic ops, single-byte representation identical to the C header's bit values.

**Alternatives rejected**: hand-rolled `#[repr(u8)] pub struct Dependency(u8)` — more code, less ergonomic; `enumflags2` — similar feature set, smaller download count, less familiar API.

---

## D9 — Codegen from C++ sources via `xtask`, not a `build.rs`

**Decision**: the registry (functionals, aliases, vars, test vectors) is generated by `cargo xtask regen-registry` and the generated files are checked into git.

**Rationale**: a `build.rs` that parses C++ would force every consumer to have a C++ toolchain installed. Check-in makes `cargo build` fast and hermetic. The `xtask` step is run by maintainers when the reference version is bumped.

**Alternatives rejected**:

| Alternative | Rejected because |
|-------------|------------------|
| `build.rs` with C++ parser | Forces C++ toolchain; slow builds; fragile |
| Manual transcription | 78 functionals × ~150 bytes of test data each = ~12 KB of error-prone copy-paste |
| Parse at runtime | Unneeded; registry is static |

---

## D10 — `thiserror` 2 for library errors; no `anyhow` in libraries

**Decision**: see [08-error-model.md](08-error-model.md). Every library crate uses `thiserror`; only applications use `anyhow`.

**Rationale**: libraries need structured, pattern-matchable errors; applications need ergonomic propagation. Mixing them ties the library's public error type to `anyhow` transitively, which bloats downstream errors.

**Alternatives rejected**: `anyhow` in libraries — the obvious antipattern; `snafu` — similar feature set, much smaller community; `eyre` — applications; same constraints as `anyhow`.

---

## D11 — Static `FUNCTIONAL_DESCRIPTORS` array, not a lazy global

**Decision**: `pub static FUNCTIONAL_DESCRIPTORS: [FunctionalDescriptor; 78]` is constructed entirely at compile time in `.rodata`.

**Rationale**: no runtime initialisation, no `lazy_static!`, no `OnceCell`. The reference C++ code has a runtime "retarded_helper" recursion that sets the `.name` field from `.symbol`; in Rust we can bake the name in at compile time via the `#[functional(id = XC_FOO)]` attribute macro.

**Alternatives rejected**: `LazyLock<HashMap<&str, FunctionalDescriptor>>` — adds a hash, slower lookup (we iterate < 78 entries linearly; hashing doesn't help); `phf` static hash map — extra dependency; benefit marginal on 78 entries.

---

## D12 — `#[repr(transparent)]` handle for the C ABI

**Decision**: `xcfun-capi::xcfun_t = #[repr(transparent)] struct(xcfun_rs::Functional)`; `xcfun_t *` on the C side is identical to `*mut Functional` in Rust.

**Rationale**: zero-cost transmute at FFI boundary; no pointer-level translation; preserves the opacity requirement from the C header.

**Alternatives rejected**: `struct xcfun_t { inner: *mut Functional }` — unnecessary indirection; `Box<Functional>` directly — equivalent but less explicit.

---

## D13 — `catch_unwind` at every C entry point

**Decision**: every `#[no_mangle] extern "C"` function in `xcfun-capi` is wrapped in a `c_entry!` macro that calls `std::panic::catch_unwind` and converts a panic to a stderr message + `abort()`.

**Rationale**: unwinding Rust panics across the FFI boundary is undefined behaviour; `abort` on panic matches the reference `xcfun::die` behaviour and preserves the C ABI guarantee.

**Alternatives rejected**: `panic = "abort"` (set globally in release, but not `test`; we still need the shim for `test` builds and for library consumers who link at `panic = "unwind"`); ignore panics — UB.

---

## D14 — No distributed / multi-node support

**Decision**: `xcfun_rs` is a single-process, single-host library. Distribution is the caller's concern.

**Rationale**: DFT grids are local per molecular fragment in every caller we know of. Adding MPI/NCCL support would expand scope without addressing a clear need.

**Alternatives rejected**: built-in MPI — new dependency, new failure modes, no caller asking for it.

---

## D15 — No stream-overlapped async GPU path in the first release

**Decision**: `Batch::launch` is synchronous from the user's perspective. Internally we use `client.sync()` between upload/launch/download.

**Rationale**: for the workload we target (1M-point evaluation), PCIe transfer is ≈ 5 ms out of ≈ 30 ms total. Overlap buys < 20 % throughput at the cost of a substantially more complex API with async buffer lifetimes.

**Alternatives rejected**: Rust `async` API around the batch — premature optimisation; breaks the "no heap in hot path" rule because async futures require allocation.

---

## D16 — Python bindings first-class, but not driving API design

**Decision**: `xcfun-py` is a thin wrapper around `xcfun-rs`. Every Python method calls directly into a Rust method; no Python-side logic.

**Rationale**: keeps the two APIs in sync by construction; avoids duplicating parameter validation.

**Alternatives rejected**: pure-Python on top of `xcfun-capi` — slower start-up; ctypes fragility; no numpy zero-copy by default.

---

## D17 — Reject `std::thread::scope` / `rayon` inside library

**Decision**: the library never spawns threads. The `CpuRuntime` backend of cubecl may parallelise under the hood, but `Functional::eval` and `Functional::eval_vec` (at small `nr_points`) are single-threaded.

**Rationale**: callers who want parallelism run their grid loop on their own (as is standard in DFT codes). Introducing a thread pool inside the library complicates determinism (thread ordering affects summation order in functional composition across active functionals, which is already a determinism-sensitive point).

**Alternatives rejected**: `rayon` on `eval_vec` — we cede this responsibility to `cubecl::CpuRuntime`; built-in thread pool — premature.

---

## D18 — `xcfun-master` vendored, not a git submodule

**Decision**: the reference C++ source lives under `xcfun-master/` as a direct directory (copied from the upstream release tag).

**Rationale**: git submodules complicate `cargo build --locked` because the submodule hash is not part of the lockfile. Vendoring gives bit-for-bit reproducibility and simplifies CI.

**Alternatives rejected**: git submodule — tooling headaches; `cargo vendor` — doesn't apply to C++ sources; build-time download — violates reproducibility.

---

## D19 — Order max 4 for `PartialDerivatives`, 6 for `Contracted`

**Decision**: mirror the reference exactly. `eval_setup` rejects `order > 4` in `PartialDerivatives` mode; `Contracted` accepts 0..=6.

**Rationale**: preserves parity with the reference, which enforces the same limit via `if (mode == XC_PARTIAL_DERIVATIVES && order > 4) return XC_EORDER;` (see `XCFunctional.cpp` line 435).

**Alternatives rejected**: accept order 5–6 in `PartialDerivatives` — would diverge from the reference; the layout for order 5–6 in that mode is not defined by xcfun.

---

## D20 — `cargo-xtask` instead of `Makefile` or shell scripts

**Decision**: all maintainer automation lives in `xtask/src/main.rs` as Rust code.

**Rationale**: portable across Windows/macOS/Linux; type-checked; shares crate infrastructure (`cargo metadata`, etc.); no new DSL to learn.

**Alternatives rejected**: `just` — adds a dependency users must install; `make` — not native on Windows; shell — brittle across platforms.

---

## Summary

These twenty decisions are binding. Revisiting any one requires a retrospective and an updated 12-design-decisions.md entry (not a silent change). The rejected-alternative column exists specifically to preempt "why didn't you just use X?" discussions — the answer is recorded.
