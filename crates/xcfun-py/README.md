# xcfun_rs (Python bindings)

Python bindings for [`xcfun_rs`](https://github.com/<owner>/xcfun_rs) — a
Rust-from-scratch reimplementation of the [xcfun](https://dftlibs.org/xcfun/)
XC functional library for density functional theory (DFT). Memory-safe,
GPU-capable, with bit-level parity (≤ 1.0 × 10⁻¹² relative error) against C++ xcfun.

- 78 functionals across LDA / GGA / metaGGA tiers
- 50+ aliases + 4 parameters (EXX, RANGESEP_MU, CAM_ALPHA, CAM_BETA)
- 31 `Vars` × 3 evaluation modes (`PartialDerivatives` orders 0..=4, `Potential`,
  `Contracted` orders 0..=4)
- Zero-copy NumPy `f64` interop on `eval_vec`

## Install

```bash
pip install xcfun_rs
```

The PyPI wheel is **CPU-only** by default (broadest install compatibility — no CUDA,
ROCm, or Vulkan runtime libraries required at install time). One wheel per platform
covers Python 3.10 / 3.11 / 3.12 / 3.13 via the `abi3-py310` ABI.

Wheel matrix (v0.1.0): `manylinux_2_28_x86_64`, `macosx_11_0_arm64`, `win_amd64`.
Linux aarch64 / macOS x86_64 / Windows aarch64 are deferred to v0.2.

### GPU rebuild (advanced)

The PyPI wheel ships only the CPU runtime. For GPU acceleration, rebuild from source
using [maturin](https://www.maturin.rs/) with the appropriate feature flag:

```bash
git clone https://github.com/<owner>/xcfun_rs
cd xcfun_rs/crates/xcfun-py
pip install 'maturin>=1.12,<2.0'

# AMD/ROCm primary GPU backend
maturin build --release --features hip --out dist/

# NVIDIA CUDA (opt-in best-effort)
maturin build --release --features cuda --out dist/

# Vulkan / Metal / WebGPU portable fallback (relaxed 1e-9 tolerance;
# ERF-using functionals auto-fall-back to CPU)
maturin build --release --features wgpu --out dist/

pip install dist/xcfun_rs-*.whl --force-reinstall
```

The CUDA / ROCm / Wgpu runtime libraries must be installed separately on the host;
see the upstream [cubecl](https://github.com/tracel-ai/cubecl) docs.

### Backend selection (env-var escape hatch)

Backend selection is driven by `xcfun_rs::auto_backend()` on the Rust side; the
Python class does NOT expose a `backend=` kwarg (per design — keeps the Python
surface small). For benchmarking / debugging you can force a specific backend
via the environment variable:

```bash
XCFUN_FORCE_BACKEND=cpu  python -c "import xcfun_rs; ..."
XCFUN_FORCE_BACKEND=hip  python -c "import xcfun_rs; ..."
XCFUN_FORCE_BACKEND=cuda python -c "import xcfun_rs; ..."
XCFUN_FORCE_BACKEND=wgpu python -c "import xcfun_rs; ..."
```

The CPU-only PyPI wheel only ever resolves to `Backend::Cpu`; the env-var is a
no-op there.

## Quickstart

```python
import numpy as np
import xcfun_rs as xc

# Construct + configure in one step (eager constructor).
f = xc.Functional(
    "pbe",
    vars=xc.Vars.A_B_GAA_GAB_GBB,
    mode=xc.Mode.PartialDerivatives,
    order=2,
)

# Strict zero-copy NumPy contract — float64 + C-contiguous required.
densities = np.ascontiguousarray(np.random.rand(1024, 5).astype(np.float64))
out = f.eval_vec(densities)   # shape (1024, output_length); freshly allocated f64

print(out[0])                  # XC energy + partial derivatives at point 0
```

Free-function utilities:

```python
print(xc.version())             # "0.1.0"
print(xc.describe_short("pbe")) # "Perdew-Burke-Ernzerhof exchange-correlation"
```

## Numerical contract

- **Default tolerance:** `≤ 1.0 × 10⁻¹²` relative error vs C++ `xcfun` for all
  `(functional, vars, mode, order, density point)` tuples in the validation grid.
- **CPU + CUDA + ROCm:** strict 1e-12 (1e-13 on the 26 mpmath-only-spec functionals
  where C++ documents floating-point cancellation; mpmath@200 ground truth substitutes).
- **Wgpu:** relaxed 1e-9; functionals with `Dependency::ERF` automatically fall back
  to CPU at 1e-12.
- **No `f32`** anywhere on the numerical path.

## Performance

`eval_vec` releases the Python GIL via `py.detach(...)` during the Rust hot path,
so multiple Python threads can pipeline evaluations without contention.

Internal dispatch:

- `nr_points < 64` → per-point fallback (cubecl-cpu substrate cost ~287 allocs/eval).
- `nr_points ≥ 64` → batched `Batch<CpuRuntime>` dispatch (zero-allocation hot path).

Test BOTH paths in your benchmark; small batches are CPU-bound on the substrate.

## v0.1.0 caveats

This is a `0.x` release. The API may evolve before `1.0`:

- `cubecl =0.10.0-pre.3` is itself a pre-release; a stable 1.0 SLA on top of an
  unstable dependency would be contradictory.
- 2 HUMAN-UAT items deferred to v0.2:
  - **ROCm tier-3 1e-13 hardware sweep** — no AMD/ROCm GPU on the cloud-CI runner.
  - **Wgpu tier-3 1e-9 hardware sweep** — no SHADER_F64-capable adapter.

See [CHANGELOG.md](https://github.com/<owner>/xcfun_rs/blob/master/CHANGELOG.md)
for the full v0.1.0 release notes.

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/) — inherited from C++ xcfun.

## Links

- [Repository](https://github.com/<owner>/xcfun_rs)
- [Issue tracker](https://github.com/<owner>/xcfun_rs/issues)
- [CHANGELOG](https://github.com/<owner>/xcfun_rs/blob/master/CHANGELOG.md)
- [C++ xcfun reference (algorithmic source of truth)](https://github.com/dftlibs/xcfun)
