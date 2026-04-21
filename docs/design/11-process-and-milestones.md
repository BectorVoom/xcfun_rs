# 11 — Process and milestones (工程)

Nine ordered milestones, each with explicit entry criteria, exit criteria, and a single non-negotiable demo that proves the milestone shipped.

A milestone is considered landed when all exit criteria are met on `main`; until then, work continues on a dedicated phase branch under `.planning/phases/NN-<milestone-slug>/`.

---

## M0 — Workspace scaffolding

**Entry criteria**: empty repository, CLAUDE.md present.

**Work**:
- Create crate skeletons for every crate in [01-source-tree.md](01-source-tree.md).
- Set up workspace `Cargo.toml` and per-crate manifests with the dependency lists in [10-build-and-dependencies.md](10-build-and-dependencies.md).
- Install CI jobs (`fmt`, `clippy`, `test`, `deny`, `no-anyhow`, `headers-match`).
- Add `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `.cargo/config.toml`.

**Exit criteria**:
- `cargo build --workspace` succeeds (trivial contents).
- `cargo test --workspace` passes (trivial tests).
- CI green on all above jobs.
- `cargo xtask check-no-anyhow` passes.
- `docs/design/*` committed and linked from top-level `README.md`.

**Demo**: open the repo, run `cargo build`. Done.

---

## M1 — `xcfun-ad`: the Taylor algebra

**Entry criteria**: M0.

**Work**:
- Implement `CTaylor<T, const N: usize>` with `Add`, `Sub`, `Neg`, `Mul`, `Mul<T>` (scalar).
- Implement the `Num` trait for `f64` and for `CTaylor<f64, N>`.
- Port every `*_expand` function from `xcfun-master/src/taylor/tmath.hpp`.
- Implement `reciprocal`, `sqrt`, `exp`, `log`, `pow`, `powi`, `erf`, `asinh`, `atan` via series composition.
- Property tests: ring axioms; round-trips (exp/log, sqrt², reciprocal); derivative-of-product rule.
- Benchmark: `CTaylor<f64, N> * CTaylor<f64, N>` for `N ∈ {2, 3, 4, 5, 6}`.

**Exit criteria**:
- All unit tests pass.
- Property tests run 10 000 iterations per property without failure.
- `cargo bench -p xcfun-ad` produces a baseline.
- Benchmark results within 30 % of a manually-written comparison multiply (informational, not a hard gate).

**Demo**: compute `d/dx (exp(x) · log(x))` at `x = 2.0` via `CTaylor<f64, 1>`; assert the `VAR0` coefficient equals `exp(x) * (log(x) + 1/x)` to within 10 × ULP.

---

## M2 — `xcfun-core` foundations: registry, vars, setup

**Entry criteria**: M1.

**Work**:
- Define `Vars`, `Mode`, `Dependency`, `XcError`.
- Port `VARS_TABLE` from `xcfun-master/src/xcint.cpp` (31 rows) via codegen (`xtask regen-registry`).
- Implement `DensVars<T>` for every `Vars` arm (31 match arms).
- Implement `Functional::new`, `set`, `get`, `is_gga`, `is_metagga`, `eval_setup`, `user_eval_setup`, `input_length`, `output_length`.
- Implement the free text APIs (`version`, `splash`, `authors`, `describe_*`, `enumerate_*`, `which_*`, `self_test`).
- Implement the registry loader: empty `FUNCTIONAL_DESCRIPTORS` (78 stubs returning 0.0), empty `ALIASES` (50 stubs).

**Exit criteria**:
- `Functional::eval_setup` returns the correct `XcError` for every error branch in `XCFunctional.cpp::xcfun_eval_setup`.
- `input_length` and `output_length` match the reference for all 31 vars × 7 orders × 3 modes (where defined).
- `xcfun_which_vars` and `xcfun_which_mode` match the reference on every encoded bit pattern.

**Demo**: `Functional::new().set("lda", 1.0).unwrap(); fun.eval_setup(Vars::AB, Mode::PartialDerivatives, 1).unwrap();` prints `input_length=2, output_length=3`; zero-body functionals produce zeros (expected at this stage).

---

## M3 — Functional ports: LDA tier

**Entry criteria**: M2.

**Work**:
- Port `slaterx`, `vwn3c`, `vwn5c`, `pw92c`, `pz81c`, `ldaerfx`, `ldaerfc`, `ldaerfc_jt`, `tfk`, `tw`, `vonw` (11 LDA functionals).
- Port helper files: `slater.hpp`, `vwn.hpp`, `pw92eps.hpp`, `pz81c.hpp`.
- Register each port in `FUNCTIONAL_DESCRIPTORS` with the full `test_in`/`test_out` from the C++ source.
- Dispatcher: implement `Mode::PartialDerivatives` order 0..=4 using `CTaylor<f64, {0, 2, 3, 4}>`.

**Exit criteria**:
- Tier 1 self-tests (see [09-testing-strategy.md](09-testing-strategy.md)) pass for every LDA functional.
- Tier 2 harness on LDA functionals at order 0..=2 passes with max rel-err ≤ 1e-12.

**Demo**: `cargo xtask validate --filter 'slaterx|vwn5c|pw92c' --order 2 --backend cpu` reports zero failures.

---

## M4 — Functional ports: GGA tier

**Entry criteria**: M3.

**Work**:
- Port 45 GGA functionals from `xcfun-master/src/functionals/` (exchange + correlation).
- Port helpers: `pbex.hpp/pbec_eps.hpp`, `p86c.hpp`, `b97*.hpp`, `pw9xx.hpp`, etc.
- Register in `FUNCTIONAL_DESCRIPTORS`.
- Implement `Mode::Potential` for GGA (the `CTaylor<f64, 2>` divergence trick from §5 of [04-control-flow.md](04-control-flow.md)).

**Exit criteria**:
- Tier 1 + Tier 2 at order 0..=2 pass for every GGA functional.
- `Functional::is_gga` returns true for each; `eval_setup` enforces `_2ND_TAYLOR` vars when mode is `Potential`.

**Demo**: `cargo xtask validate --filter 'pbex|pbec|beckex|lypc' --order 2 --backend cpu` reports zero failures.

---

## M5 — Functional ports: metaGGA and Taylor modes

**Entry criteria**: M4.

**Work**:
- Port 15 metaGGA functionals (TPSS, SCAN, rSCAN, r2SCAN, r4SCAN, rppSCAN, revTPSS, M05, M06 family, blocx, btk).
- Port helpers: `tpssx_eps.hpp`, `tpssc_eps.hpp`, `revtpssx_eps.hpp`, `SCAN_like_eps.hpp`, `m0xy_fun.hpp`.
- Implement `Mode::Contracted` for orders 0..=6.
- Aliases: port `aliases.cpp` (50+ aliases) with composition weights, including `b3lyp`, `pbe0`, `camb3lyp`, `scan`, `m06*`, `b97*`.

**Exit criteria**:
- Tier 1 + Tier 2 at orders 0..=3 pass for every functional including metaGGAs.
- Aliases resolve correctly: `Functional::new().set("b3lyp", 1.0)` produces the same weight set as the manual composition.

**Demo**: `cargo xtask validate --backend cpu --order 3` reports zero failures across all 78 functionals.

---

## M6 — `xcfun-rs` and `xcfun-capi` APIs

**Entry criteria**: M5.

**Work**:
- Finalise `xcfun-rs::Functional` with the full public API from [03-api-surface.md](03-api-surface.md).
- Implement `xcfun-capi`: every C symbol, `catch_unwind` shim, `xcfun_t` wrapper.
- Generate `xcfun.h` via `cbindgen`; wire the `headers_match` test.
- Compile the `tests/c_abi.c` harness; verify it links against the staticlib.

**Exit criteria**:
- `cargo test -p xcfun-capi --test c_abi` passes (C program produces same output as a Rust program for 10 random fixtures).
- `headers_match` test passes.
- `xcfun-rs::tests::api_coverage` invokes every public function.

**Demo**: compile a small C driver (the xcfun README example) against the generated `xcfun.h` and the Rust cdylib; output matches the output of the same driver linked against `libxcfun_cpp.so`.

---

## M7 — cubecl kernels: CPU backend

**Entry criteria**: M6.

**Work**:
- Port every scalar functional body to a `#[cube]` counterpart (78 ports).
- Port `DensVars::build` to `DensVarsDev::build_<variant>` (31 `#[cube]` ports).
- Port `CTaylor` arithmetic to a device-resident `CTaylorDev<F, N>` with identical bit-flag indexing.
- Implement `eval_batch_kernel<F: Float>` with compile-time vars/mode/order specialisation.
- Implement `xcfun-gpu::Batch` for the `CpuRuntime` backend.
- Update `xcfun-rs::Functional::eval_vec` to dispatch to `Batch<CpuRuntime>` when `nr_points ≥ 64`.

**Exit criteria**:
- Tier 3 parity: `eval_vec` on a 10k-point grid via `Batch<CpuRuntime>` matches the scalar `eval` loop within 1e-13 rel-err.
- `Functional::eval_vec` on 100k GGA points is within 10 % of the C++ wall-clock on the same machine.

**Demo**: `cargo run --example b3lyp_grid --release` processes 100 000 grid points on CPU and prints wall-clock time.

---

## M8 — GPU backends: CUDA, Wgpu

**Entry criteria**: M7.

**Work**:
- Enable `cubecl-cuda` (feature `cuda`) and `cubecl-wgpu` (feature `wgpu`) paths in `xcfun-gpu::Batch`.
- Implement `Backend::Cuda`, `Backend::Wgpu`, `auto_backend()`.
- Wire the device buffer pool with growth policy.
- Handle range-separated `erf`-using functionals by forcing `Backend::Cpu` for those on the `Wgpu` backend.

**Exit criteria**:
- Tier 3 parity on `Cuda`: 10k grid points rel-err ≤ 1e-13 vs. CPU.
- Tier 3 on `Wgpu`: tolerance 1e-9, excluding range-separated functionals.
- 1M-point GGA kernel on A100 achieves > 1e9 evaluations/second (informational).

**Demo**: `cargo run --example m06_cuda --features cuda --release` processes 1 000 000 grid points on GPU, compares with CPU, reports max rel-err < 1e-13.

---

## M9 — Python bindings and release

**Entry criteria**: M8.

**Work**:
- Implement `xcfun-py` (PyFunctional, free functions, numpy interop).
- Configure `pyproject.toml` and `maturin` for abi3 py3.10+.
- CI: `python` job with `pytest`.
- Release prep: semver tags, CHANGELOG, documentation polish.
- Publish wheels via `maturin publish` (dry-run in CI, actual push on tag).

**Exit criteria**:
- `pip install xcfun_rs` yields a wheel that passes `pytest`.
- Release artefacts: crates on crates.io, wheels on PyPI, `xcfun.h` checked into the release branch.
- All 78 functionals + 50 aliases + all 3 modes + all 4 orders (0..=4 for partial derivatives, 0..=6 for contracted) pass the validation harness.

**Demo**: `pip install xcfun_rs; python -c "import xcfun_rs; ..."` reproduces the output of a reference Python script written against the C++ bindings.

---

## Parallelisability

Milestones are mostly sequential, but within M3, M4, M5, the functional ports parallelise trivially (one PR per functional). CI runs per-PR with the narrow-scope validation filter (`cargo xtask validate --filter '<functional>'`).

## Rollback plan

If a functional port cannot meet 1e-12 parity despite algorithmic identity:
1. First, check for accidental reassociation (`a * b + c` vs `a * (b + c)`) — by far the common cause.
2. If that's not the cause, inspect the C++ intermediate values at the first divergent coefficient (see [07-accuracy-strategy.md §7](07-accuracy-strategy.md)).
3. If the divergence is in a libm call, relax tolerance for that specific functional in the harness by one order of magnitude and document the relaxation in CHANGELOG with a comment pointing to the libm source (`musl` vs `glibc`).

## Out-of-band timebox

Each milestone gets an 8-week timebox. If a milestone overruns, work is paused for a retrospective before continuing. The timebox is not a deadline — the exit criteria are the deadline.
