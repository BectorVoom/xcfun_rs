# 10 — Build and dependencies

Workspace `Cargo.toml`, per-crate dependency tables, feature flags, MSRV, toolchain, and build-time codegen. All version constraints inherit the decisions in the top-level CLAUDE.md.

---

## 1. Rust toolchain

| Setting | Value |
|---------|-------|
| Rust edition | 2024 |
| MSRV | 1.85 |
| Channel | stable (no nightly features) |
| Components | `rustfmt`, `clippy` |
| Targets | `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`; `x86_64-unknown-linux-gnu` with `cuda` feature for nightly GPU runs |

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.85"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

`RUSTFLAGS` in CI: **empty**. No fast-math, no `-C target-cpu=native`, no reassociation flags. This is a hard accuracy requirement (see [07-accuracy-strategy.md §2](07-accuracy-strategy.md)).

---

## 2. Workspace manifest

`Cargo.toml` at the repository root:

```toml
[workspace]
resolver = "3"
members  = [
    "crates/xcfun-ad",
    "crates/xcfun-core",
    "crates/xcfun-kernels",
    "crates/xcfun-gpu",
    "crates/xcfun-rs",
    "crates/xcfun-capi",
    "crates/xcfun-py",
    "validation",
    "xtask",
]

[workspace.package]
edition    = "2024"
rust-version = "1.85"
license    = "MPL-2.0"
repository = "https://github.com/<owner>/xcfun_rs"
authors    = ["xcfun_rs contributors"]

[workspace.dependencies]
thiserror   = "2.0.18"
bitflags    = "2.10.0"
tracing     = { version = "0.1.44", default-features = false }
cubecl      = { version = "=0.10.0-pre.3" }
cubecl-cpu  = { version = "=0.10.0-pre.3" }
cubecl-cuda = { version = "=0.10.0-pre.3" }
cubecl-wgpu = { version = "=0.10.0-pre.3" }
pyo3        = { version = "0.28.3", features = ["extension-module", "abi3-py310"] }
numpy       = "0.28.0"

# Build / dev tools
cbindgen    = "0.29.2"
cc          = "1.1"
criterion   = { version = "0.8.2", default-features = false, features = ["html_reports"] }
approx      = "0.5"
proptest    = "1.6"
rstest      = "0.23"
anyhow      = "1.0"
serde_json  = "1.0"
rand_xoshiro = "0.7"

[profile.release]
opt-level   = 3
lto         = "thin"
codegen-units = 1
debug       = 0
incremental = false
panic       = "abort"

[profile.bench]
opt-level   = 3
lto         = "thin"
codegen-units = 1
debug       = 1

[profile.test]
opt-level   = 1          # keep tests reasonably fast while preserving debug info
debug       = true
```

`lto = "thin"` lets the compiler inline functional kernels across crate boundaries without the slowdown of `"fat"`. `codegen-units = 1` preserves consistent floating-point sequencing (a practical precaution, though not strictly guaranteed).

---

## 3. Per-crate dependencies

### 3.1 `crates/xcfun-ad/Cargo.toml`

```toml
[package]
name        = "xcfun-ad"
version     = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default = ["std"]
std     = []
testing = []

[dependencies]
# none on the happy path

[dev-dependencies]
approx   = { workspace = true }
proptest = { workspace = true }
rstest   = { workspace = true }
```

Zero runtime dependencies. `no_std`-capable when `default-features = false` is passed.

### 3.2 `crates/xcfun-core/Cargo.toml`

```toml
[package]
name = "xcfun-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default = ["std"]
std     = []
testing = []

[dependencies]
xcfun-ad   = { path = "../xcfun-ad", default-features = false }
thiserror  = { workspace = true }
bitflags   = { workspace = true }

[build-dependencies]
# optional: if we generate the registry at build time; default generation is via xtask

[dev-dependencies]
approx    = { workspace = true }
proptest  = { workspace = true }
rstest    = { workspace = true }
rand_xoshiro = { workspace = true }
```

### 3.3 `crates/xcfun-kernels/Cargo.toml`

```toml
[package]
name = "xcfun-kernels"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default      = []
cpu-testing  = ["cubecl-cpu"]

[dependencies]
xcfun-core = { path = "../xcfun-core" }
cubecl     = { workspace = true }
cubecl-cpu = { workspace = true, optional = true }

[dev-dependencies]
approx = { workspace = true }
```

### 3.4 `crates/xcfun-gpu/Cargo.toml`

```toml
[package]
name = "xcfun-gpu"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default = ["cpu"]
cpu     = ["cubecl-cpu"]
cuda    = ["cubecl-cuda"]
wgpu    = ["cubecl-wgpu"]
metrics = []

[dependencies]
xcfun-core    = { path = "../xcfun-core" }
xcfun-kernels = { path = "../xcfun-kernels" }
cubecl        = { workspace = true }
cubecl-cpu    = { workspace = true, optional = true }
cubecl-cuda   = { workspace = true, optional = true }
cubecl-wgpu   = { workspace = true, optional = true }
tracing       = { workspace = true }

[dev-dependencies]
approx    = { workspace = true }
```

### 3.5 `crates/xcfun-rs/Cargo.toml`

```toml
[package]
name = "xcfun-rs"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default = []
cuda    = ["xcfun-gpu/cuda"]
wgpu    = ["xcfun-gpu/wgpu"]

[dependencies]
xcfun-core = { path = "../xcfun-core" }
xcfun-gpu  = { path = "../xcfun-gpu" }

[dev-dependencies]
approx = { workspace = true }
```

### 3.6 `crates/xcfun-capi/Cargo.toml`

```toml
[package]
name = "xcfun-capi"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "staticlib"]

[features]
default     = []
panic-hook  = []

[dependencies]
xcfun-rs = { path = "../xcfun-rs" }

[build-dependencies]
cbindgen = { workspace = true }

[dev-dependencies]
cc = { workspace = true }
```

### 3.7 `crates/xcfun-py/Cargo.toml`

```toml
[package]
name = "xcfun-py"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib"]
name       = "xcfun_rs"

[dependencies]
xcfun-rs = { path = "../xcfun-rs" }
pyo3     = { workspace = true }
numpy    = { workspace = true }
```

`pyproject.toml` in the same crate directory drives `maturin`:

```toml
[build-system]
requires      = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name            = "xcfun_rs"
version         = "0.1.0"
description     = "Rust reimplementation of xcfun"
requires-python = ">=3.10"

[tool.maturin]
features = ["pyo3/extension-module"]
```

### 3.8 `validation/Cargo.toml`

```toml
[package]
name = "xcfun-validate"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[[bin]]
name = "xcfun-validate"
path = "src/main.rs"

[dependencies]
xcfun-rs  = { path = "../crates/xcfun-rs" }
anyhow    = { workspace = true }
approx    = { workspace = true }
serde_json = { workspace = true }
rand_xoshiro = { workspace = true }
tracing-subscriber = "0.3"

[build-dependencies]
cc = { workspace = true, features = ["parallel"] }
```

`validation/build.rs` compiles the C++ reference:

```rust
fn main() {
    cc::Build::new()
        .cpp(true)
        .include("../xcfun-master/api")
        .include("../xcfun-master/src")
        .file("../xcfun-master/src/XCFunctional.cpp")
        .file("../xcfun-master/src/xcint.cpp")
        // ... all *.cpp files in xcfun-master/src/ and functionals/
        .flag_if_supported("-std=c++17")
        .compile("xcfun_cpp");
}
```

The resulting `libxcfun_cpp.a` is statically linked into the `xcfun-validate` binary.

### 3.9 `xtask/Cargo.toml`

```toml
[package]
name = "xtask"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[[bin]]
name = "xtask"
path = "src/main.rs"

[dependencies]
anyhow      = { workspace = true }
serde_json  = { workspace = true }
```

Minimal; xtask uses stdlib for file I/O and shell-outs to `cargo`, `cbindgen`, etc.

---

## 4. Summary table — every library used

| Crate | Version | Purpose | Consumers |
|-------|---------|---------|-----------|
| `thiserror` | 2.0.18 | Library error types | `xcfun-core` (re-exported everywhere) |
| `bitflags` | 2.10.0 | `Dependency` bit flags | `xcfun-core` |
| `tracing` | 0.1.44 | Structured logging on batch boundaries | `xcfun-gpu`, `xcfun-capi` (optional) |
| `cubecl` | =0.10.0-pre.3 | Kernel DSL + runtime abstraction | `xcfun-kernels`, `xcfun-gpu` |
| `cubecl-cpu` | =0.10.0-pre.3 | CPU backend for cubecl | `xcfun-gpu` (default), `xcfun-kernels` (feature `cpu-testing`) |
| `cubecl-cuda` | =0.10.0-pre.3 | CUDA backend | `xcfun-gpu` (feature `cuda`) |
| `cubecl-wgpu` | =0.10.0-pre.3 | WebGPU backend | `xcfun-gpu` (feature `wgpu`) |
| `pyo3` | 0.28.3 | Python bindings (abi3 py3.10+) | `xcfun-py` |
| `numpy` (rust-numpy) | 0.28.0 | Zero-copy numpy interop | `xcfun-py` |
| `cbindgen` | 0.29.2 | Generate `xcfun.h` from Rust | `xcfun-capi` (build) |
| `cc` | 1.1 | Compile C / C++ | `validation` (build); `xcfun-capi` (dev-dep for c_abi test) |
| `anyhow` | 1.0 | Application error handling | `validation`, `xtask`, `examples/*`, `benches/*` — **not** library crates |
| `criterion` | 0.8.2 | Statistical benchmarking | All `crates/*/benches/` |
| `approx` | 0.5 | Float comparison assertions | Every crate's `[dev-dependencies]` |
| `proptest` | 1.6 | Property-based testing | `xcfun-ad` (dev), `xcfun-core` (dev) |
| `rstest` | 0.23 | Parameterised tests | `xcfun-core` (dev) |
| `rand_xoshiro` | 0.7 | Deterministic RNG for fixtures | `validation`, `xcfun-core` (dev) |
| `serde_json` | 1.0 | Report serialisation | `validation`, `xtask` |
| `tracing-subscriber` | 0.3 | Subscriber for diagnostic logging | `validation` only |

---

## 5. Build tools (not Cargo deps)

| Tool | Version | Usage |
|------|---------|-------|
| `maturin` | >=1.0, <2.0 | Build and publish Python wheels; declared in `pyproject.toml` |
| `cargo-nextest` | latest | Test runner; installed via `cargo install cargo-nextest --locked` in CI |
| `cargo-deny` | latest | License + advisory audit; installed in CI |
| `cargo-criterion` | latest | Benchmark driver; wraps `criterion` |
| `cargo-udeps` | latest | Unused-dependency detection; weekly CI |
| `cbindgen` (binary) | 0.29.2 | Optional; build scripts embed the library form |
| `pre-commit` | 3+ | Runs `cargo fmt`, `cargo clippy`, and a `check-no-anyhow` hook |

---

## 6. Feature-flag matrix

Compile-time features documented per crate in [01-source-tree.md §6](01-source-tree.md). Summary:

| Feature | Default? | Crate | Effect |
|---------|----------|-------|--------|
| `std` | yes | `xcfun-ad`, `xcfun-core` | Enables heap-backed `SmallString` fallback; disable for `no_std` builds |
| `testing` | no | `xcfun-ad`, `xcfun-core` | Exposes `for_tests::*` seams |
| `cpu` | yes | `xcfun-gpu` | `Backend::Cpu` via `cubecl-cpu` |
| `cuda` | no | `xcfun-gpu`, `xcfun-rs` | `Backend::Cuda`; requires CUDA toolkit |
| `wgpu` | no | `xcfun-gpu`, `xcfun-rs` | `Backend::Wgpu`; requires Vulkan / Metal runtime |
| `cpu-testing` | no | `xcfun-kernels` | Launch kernels on CpuRuntime in tests |
| `metrics` | no | `xcfun-gpu` | `BatchMetrics` counters |
| `panic-hook` | no | `xcfun-capi` | Write panics to `${XCFUN_PANIC_LOG}` |

Feature flags never change numerical output.

---

## 7. CI jobs

| Job | Runs | Triggers |
|-----|------|---------|
| `fmt` | `cargo fmt --check` | every push |
| `clippy` | `cargo clippy --workspace --all-features -- -D warnings` | every push |
| `test` | `cargo nextest run --workspace` | every push |
| `validate-low-order` | `cargo xtask validate --order 0..2 --backend cpu` | every PR |
| `validate-high-order` | `cargo xtask validate --order 3..4 --backend cpu` | PR into `main` |
| `validate-cuda` | `cargo xtask validate --backend cuda` | nightly, self-hosted |
| `validate-wgpu` | `cargo xtask validate --backend wgpu` | nightly, self-hosted |
| `deny` | `cargo deny check licenses advisories bans` | every push |
| `no-anyhow` | `cargo xtask check-no-anyhow` | every push |
| `headers-match` | `cargo test -p xcfun-capi --test headers_match` | every push |
| `bench` | `cargo criterion --baseline pr` | nightly; comments on the PR |
| `python` | `maturin develop && pytest` | every push |

---

## 8. Supply chain

- All dependencies are pinned in the workspace `Cargo.toml` or `=x.y.z` for pre-releases (`cubecl`).
- `cargo-deny` enforces a license allowlist: `MPL-2.0` (project), `MIT`, `Apache-2.0`, `BSD-3-Clause`, `ISC`, `Unicode-DFS-2016`. `GPL` rejected.
- `cargo audit` runs on every PR; vulnerabilities block merge.
- Upgrades are gated by a fresh run of the validation harness — accuracy regressions block the bump.

---

## 9. Reproducible builds

- `Cargo.lock` is checked in at the workspace root.
- Docker image (`docker/Dockerfile`) pins the Rust toolchain and the CUDA version for CI.
- `cargo build --locked` is the release build command; anything that fails `--locked` fails CI.

---

## 10. Summary

The dependency set is small and fully justified. Library crates depend only on `thiserror`, `bitflags`, `tracing`, `cubecl`, and `xcfun-ad`/`xcfun-core` internally. Applications add `anyhow`, `cc`, `criterion`, `proptest`, `approx`, `rstest`, `serde_json`, `rand_xoshiro`, `tracing-subscriber`. No crate in the dependency closure is under maintenance risk: every one is 100M+ downloads or pinned to the upstream maintainer's release (cubecl).
