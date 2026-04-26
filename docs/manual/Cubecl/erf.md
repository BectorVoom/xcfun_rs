# CubeCL `erf` Manual

## Overview

`erf` is the Gaussian error function:

`erf(x) = (2 / sqrt(pi)) * integral(0..x, exp(-t^2) dt)`

In CubeCL kernels, `erf` is exposed as a unary floating-point operation. It is commonly used in probability/statistics transforms and activation functions such as GELU.

## What `erf` does

In `#[cube]` code, `erf` computes the error function for each element of the input value (or line/vector lane), returning a value in `[-1, 1]`.

Key behavior:

- `erf(0) = 0`
- `erf` is odd: `erf(-x) = -erf(x)`
- `erf(x)` approaches `1` for large positive `x`, and `-1` for large negative `x`

## API / Signature

CubeCL exposes `erf` through the unary operation trait for floating-point cube primitives.

Practical call form in kernel code:

- `x.erf()`

Input and output type are the same (element-wise unary transform).

## Parameters and return value

- Input: floating-point cube value (`F`) or line type (`Line<F>`) in `#[cube]` context.
- Return: same type as input.
- Semantics: element-wise unary transformation.

## Supported types / constraints

From CubeCL `0.9.0` frontend registration, `erf` is available for:

- `f16`
- `bf16`
- `flex32`
- `tf32`
- `f32`
- `f64`

Constraints:

- Not available for integer or boolean element types.
- Intended for CubeCL kernel/frontend code (`#[cube]` functions).

## Build-verified examples

All source snippets below were verified to build successfully on **2026-04-09** in this environment with:

```bash
CARGO_HOME=/tmp/cargo-home cargo check --example erf_minimal
CARGO_HOME=/tmp/cargo-home cargo check --example erf_gelu_kernel
```

### Minimal example

```rust
use cubecl::prelude::*;

#[cube]
fn apply_erf<F: Float>(x: Line<F>) -> Line<F> {
    x.erf()
}

fn main() {}
```

### Realistic example: GELU kernel using `erf`

```rust
use cubecl::prelude::*;

#[cube]
fn gelu_scalar<F: Float>(x: Line<F>) -> Line<F> {
    let sqrt2 = F::new(comptime!(2.0f32.sqrt()));
    let tmp = x / Line::new(sqrt2);
    x * (tmp.erf() + 1.0) / 2.0
}

#[cube(launch_unchecked)]
fn gelu_array<F: Float>(input: &Array<Line<F>>, output: &mut Array<Line<F>>) {
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = gelu_scalar(input[ABSOLUTE_POS]);
    }
}

fn main() {}
```

## Practical usage notes

- `erf` is often used to implement GELU and Gaussian/CDF-like transforms directly in kernels.
- Keep your kernel generic with `F: Float` when you need broad float-type compatibility.
- Because CubeCL operates on `Line<F>`, `erf` composes naturally with vectorized arithmetic.

## Numerical or semantic considerations

- CubeCL backends may lower `erf` differently:
- WGSL and CPU compilation paths expand `erf` through a CubeCL polyfill expression.
- The frontend polyfill references an approximation with documented max error around `1.5e-7` (for the approximation path).
- C++-family codegen paths emit backend-specific `erf` handling; Metal includes an explicit generated approximation routine.
- Because lowering differs by backend and numeric type (`f16`/`bf16`/`f32`/`f64`), tiny numerical differences across runtimes are expected.

## Common pitfalls

- Calling `erf` on non-float element types.
- Assuming bit-identical results across CPU/WGSL/CUDA/HIP/Metal paths.
- Mixing precision types without explicitly checking acceptable error bounds.

## Troubleshooting

- **`no method named erf` in kernel code**: ensure the type is floating-point and the function is in CubeCL kernel context (`#[cube]`, typically with `F: Float`).
- **Unexpected numeric drift between backends**: compare with tolerance, not exact equality; use a stricter precision type if needed.
- **Compilation issues on a specific backend**: reduce the kernel to `f32` first, confirm build, then reintroduce mixed/low-precision types.

## Summary

`erf` in CubeCL is a first-class unary floating-point kernel operation (`x.erf()`) available on CubeCL float types. It is practical for ML/statistical transforms (especially GELU), but you should treat results as backend-dependent floating-point approximations and validate with tolerances appropriate to your workload.
