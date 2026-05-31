# xcfun-gpu

GPU batch lifecycle + `auto_backend` dispatch for `xcfun_rs`.

`xcfun-gpu` is the runtime-agnostic dispatch layer for `xcfun_rs::Functional::eval_vec`.
It owns the `Backend` enum, the generic `Batch<'fun, R: cubecl::Runtime>` struct
(generation-counter buffer pool per CONTEXT D-15), the `auto_backend()` priority
chain (CONTEXT D-07), and the `OnceLock<R::Client>` cache per runtime.

The cubecl runtime crates are pulled in behind opt-in feature flags:

| Feature | Cubecl crate     | Default? | Plan that wired it |
|---------|------------------|----------|--------------------|
| `cpu`   | `cubecl-cpu`     | yes      | 06-02a             |
| `hip`   | `cubecl-hip`     | no       | 06-03 (PRIMARY)    |
| `cuda`  | `cubecl-cuda`    | no       | 06-04 (planned)    |
| `wgpu`  | `cubecl-wgpu`    | no       | 06-04 (planned)    |
| `metal` | (alias for `wgpu`; no separate `cubecl-metal` crate exists per RESEARCH §"Pitfall 9") | no | 06-04 (planned) |

All cubecl crates are pinned at `=0.10.0-pre.3`. Pre-release crates cross-reference
internal types — drift between `cubecl-hip 0.10.0-pre.3` and `cubecl 0.10.0-pre.4`
produces opaque "type X1 is not the same as type X2" compiler errors. The
`xtask check-cubecl-pin` gate enforces lockstep across all five crates on every PR
(`cubecl`, `cubecl-cpu`, `cubecl-hip`, `cubecl-cuda`, `cubecl-wgpu`).

## Backend Priority (CONTEXT D-07)

`auto_backend()` selects the runtime in this order:

1. `XCFUN_FORCE_BACKEND` env var (`cpu` | `rocm` | `hip` | `cuda` | `metal` | `wgpu`)
   — overrides everything; an unrecognised value PANICS so a misconfigured CI job
   fails loudly rather than silently picking the wrong backend.
2. **ROCm** via `cubecl-hip` (PRIMARY per CONTEXT D-05) — feature `hip`.
3. **CUDA** via `cubecl-cuda` — feature `cuda` (community-maintained best-effort).
4. **Metal** via `cubecl-wgpu` Metal backend — feature `metal` (alias of `wgpu`);
   requires hardware f64 (Apple Silicon LACKS f64 — falls through to step 6).
5. **Wgpu** (Vulkan / DX12 / WebGPU) via `cubecl-wgpu` — feature `wgpu`; requires
   `wgpu::Features::SHADER_F64`.
6. **Cpu** via `cubecl-cpu` — always available (validation substrate).

Override the priority chain at process start:

```bash
export XCFUN_FORCE_BACKEND=rocm   # or cpu | cuda | metal | wgpu
```

## Environment Variables

| Variable                          | Required by                                        | Effect                                                                                                                                                                                                                                                                                                                                |
|-----------------------------------|----------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `HSA_OVERRIDE_GFX_VERSION=10.3.0` | RDNA-2 GPUs (RX 6000-series, gfx1031/1032/1033)    | Coerces RDNA-2 to RDNA-3 PTX. Without this, kernel launches fail with `code object load failed`. **MANDATORY** before any `Backend::Rocm` use on RX 6000-series. Set in the process environment **before** the first call to `auto_backend()` / `Batch::open_rocm()` — HIP's runtime loader consults the env var at client init time. |
| `HSA_OVERRIDE_GFX_VERSION=11.0.0` | RDNA-3.5 iGPUs not in the HIP target list (e.g. gfx1152 / Radeon 860M, Ryzen AI 300-series) | Coerces a too-new gfx target to gfx1100's code object. Without this the `HipRuntime` probe (`rocm_available()` / `Batch::open_rocm()`) bails because the installed HIP compiler ships no native code object for the device. Same timing rule as the 10.3.0 row: export **before** the first `auto_backend()` / `Batch::open_rocm()`. Verified 2026-06-01 on gfx1152 — tier-3 oracle 0 failing at 1e-13. |
| `XCFUN_FORCE_BACKEND=<name>`      | optional                                           | Forces `auto_backend()` selection. Recognised: `cpu`, `rocm`, `hip` (alias of `rocm`), `cuda`, `metal`, `wgpu`. Unrecognised values panic.                                                                                                                                                                                            |
| `XCFUN_MIN_BATCH_SIZE=<usize>`    | optional                                           | Overrides default `eval_vec` dispatch threshold (default 64 per CONTEXT D-14). Below the threshold `eval_vec` falls back to a scalar `Functional::eval` loop on the host.                                                                                                                                                             |

## Apple Silicon Caveat

Apple Silicon GPUs (M1/M2/M3 family) lack hardware f64 in their Metal compute
units. `cubecl-wgpu` on Apple Silicon will runtime-probe and refuse to instantiate
`Batch::<WgpuRuntime>` (returns `XcError::WgpuNoF64`). Fall-through:
**Apple Silicon = CPU-only** for `xcfun_rs`.

## Numerical Tolerance Envelope (CONTEXT D-02)

| Backend | Strict bar  | Per-functional override                                        |
|---------|-------------|----------------------------------------------------------------|
| Cpu (validation substrate) | 1e-13 | none                                              |
| Rocm (PRIMARY)             | 1e-13 | none                                              |
| Cuda (opt-in)              | 1e-13 (best-effort; cloud-CI) | none                              |
| Metal (opt-in via Wgpu)    | best-effort | range-separated functionals (`Dependency::ERF`) auto-fall-back to Cpu |
| Wgpu (portable)            | 1e-9        | range-separated functionals auto-fall-back to Cpu |

The strict `1e-13` bar is Phase 6 D-02. Range-separated functionals carrying
`Dependency::ERF` route to Cpu on Wgpu/Metal automatically per
`error_routing::must_fall_back_to_cpu` (CONTEXT GPU-05).

## ROCm Install (Linux)

cubecl-hip links against the ROCm 7.x HIP runtime. On Ubuntu / Debian:

```bash
# AMD's rocm.docs.amd.com Quick Start (verify against your distro's
# AMD-recommended install path; the package names below match Ubuntu 24.04).
sudo apt install rocm-hip-runtime libamd-comgr-dev

# RDNA-2 users (RX 6000-series only):
export HSA_OVERRIDE_GFX_VERSION=10.3.0

# RDNA-3.5 iGPUs the installed HIP compiler has no code object for
# (e.g. gfx1152 / Radeon 860M on Ryzen AI 300-series) — coerce to gfx1100:
export HSA_OVERRIDE_GFX_VERSION=11.0.0
```

If `rocminfo` lists your `gfx` target but `Batch::open_rocm()` still bails at
the probe, the HIP compiler likely lacks a native code object for that target —
pick the override row above matching your architecture generation.

System prerequisites compiled in via `cargo build --features hip`:
- ROCm runtime libraries on the loader path (`/opt/rocm/lib` typical).
- `libamd_comgr.so.X` reachable (`apt install libamd-comgr-dev`).
- `cubecl-hip-sys` (transitively pulled by `cubecl-hip 0.10.0-pre.3`) builds
  against the system ROCm headers; missing headers fail at **build** time, not
  at runtime probe time.

Verify:

```bash
rocminfo                                  # should list a gfx target
cargo build -p xcfun-gpu --features hip   # should compile (build-deps only)
cargo run -p validation --release --features hip -- --backend rocm --tier 3 --order 3
```

## Programmatic Usage

```rust
use xcfun_gpu::{auto_backend, Backend, Batch};

let backend = auto_backend();
match backend {
    Backend::Cpu => {
        // CPU substrate — always available.
    }
    Backend::Rocm => {
        // ROCm probe succeeded. Open a HIP batch:
        // let mut b = Batch::<cubecl_hip::HipRuntime>::open_rocm(&fun)?;
    }
    _ => unreachable!("Plans 06-04+ wire the remaining arms"),
}
```

For the full `eval_vec` dispatch (with min-batch threshold, ERF fallback, and
runtime probe gating), use `xcfun_rs::Functional::eval_vec` (Plan 06-05) which
delegates to `xcfun-gpu` internally.

## License

MPL-2.0 (matches the parent `xcfun_rs` workspace).
