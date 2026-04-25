# Phase 4: metaGGA Tier + `Mode::Contracted` + Aliases - Context

**Gathered:** 2026-04-25 (discuss-phase `--auto`, Claude auto-selected recommended defaults across 11 gray areas; no user prompts)
**Status:** Ready for planning
**Supersedes:** None (first context for Phase 4). Inherits locked decisions from Phase 1 (`01-CONTEXT.md` D-01..D-28), Phase 2 (`02-CONTEXT.md` D-01..D-25), and Phase 3 (`03-CONTEXT.md` D-01..D-25 + amendments D-01-A/B/C, D-10-A).

<domain>
## Phase Boundary

Phase 4 closes out the **functional-tier work** for v1: ports the **metaGGA tier (28 functionals)**, picks up the **4 metaGGA-class deferrals from Phase 3 (BRX/BRC/BRXC + CSC, per Phase 3 D-01-A)**, ships **`Mode::Contracted` for orders 0..=6** (the only remaining evaluation mode), and lands the **alias engine (46 aliases) + 4 tunable parameters (XC_EXX, XC_RANGESEP_MU, XC_CAM_ALPHA, XC_CAM_BETA)**. After Phase 4, every functional, every alias, every parameter, every Vars arm, and every Mode is operational at strict 1e-12 parity on `cubecl-cpu`. Phase 5 then adds the user-facing facade + C ABI on top of an algorithmically complete engine.

A user can run:

```
cargo xtask validate --backend cpu --order 3 --filter '.*'
```

and see **zero failures at 1e-12 relative error** (subject to the inherited Phase 3 D-19 forwards) against the C++ reference on the 10 000-point seeded grid, for every `(functional ∈ all 78 ids minus LB94, vars, mode ∈ {PartialDerivatives, Potential, Contracted}, order ∈ 0..=3 for PartialDerivatives, 0..=6 for Contracted, density point)` tuple that is legal per `eval_setup`.

**In scope:**

- **xcfun-ad substrate (Wave 0):**
  - **`ctaylor_br_inverse` primitive** (D-02): a `#[cube] fn` port of `xcfun-master/src/functionals/brx.cpp:25-72` (`BR_z` + scalar `BR` Newton-Raphson root finder + linear-method `BR_taylor` polynomial inverter). The host-side scalar Newton iteration runs at f64 precision on the `CNST` slot only; the Taylor coefficients are populated by repeatedly evaluating `f = BR_z(t)` and back-solving `t[i] = -f[i] * t[1]` (Brent–Kung linear coefficient sweep). Required by GGA-03 (BR family, Phase 3 carryover) and BLOCX (MGGA-05 — uses `BRX` underneath). Strict 1e-12 fixture-gate.
  - **No other xcfun-ad additions** — every metaGGA body composes from the existing primitives (`ctaylor_*` ops + composed `math.rs` functions + `expm1` + `sqrtx_asinh_sqrtx` from Phase 3). If a fixture-gate failure surfaces a missing primitive at Wave-1 time, escalate via PLANNING INCONCLUSIVE per Phase 1 D-03.
- **DensVarsDev new Vars arms (Wave 0, D-03):** add 11 new comptime arms in `build_densvars` covering metaGGA Vars enum discriminants 8..=18 + 23..=26:
  - `A_GAA_LAPA` (id=8, inlen=3) — alpha-only metaGGA with laplacian.
  - `A_GAA_TAUA` (id=9, inlen=3) — alpha-only metaGGA with kinetic energy density.
  - `N_GNN_LAPN` (id=10, inlen=3) — total-density metaGGA with laplacian.
  - `N_GNN_TAUN` (id=11, inlen=3) — total-density metaGGA with kinetic energy density.
  - `A_B_GAA_GAB_GBB_LAPA_LAPB` (id=12, inlen=7).
  - `A_B_GAA_GAB_GBB_TAUA_TAUB` (id=13, inlen=7) — primary metaGGA Vars used by TPSS / SCAN / M0x families.
  - `N_S_GNN_GNS_GSS_LAPN_LAPS` (id=14, inlen=7).
  - `N_S_GNN_GNS_GSS_TAUN_TAUS` (id=15, inlen=7) — derived-basis metaGGA.
  - `A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB` (id=16, inlen=9) — TPSSC + SCAN-family + LYP-modified-form Vars.
  - `A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB` (id=17, inlen=11) — current-density metaGGA Vars; required by BR family (KINETIC|LAPLACIAN|JP). **This is the discriminant Phase 3 D-01-A flagged as load-bearing for the BR carryover.**
  - `N_S_GNN_GNS_GSS_LAPN_LAPS_TAUN_TAUS` (id=18, inlen=9) — derived-basis metaGGA with both laplacian and kinetic.
  - `A_AX_AY_AZ_TAUA` (id=23, inlen=5) — gradient-component metaGGA, alpha-only.
  - `A_B_AX_AY_AZ_BX_BY_BZ_TAUA_TAUB` (id=24, inlen=10) — gradient-component metaGGA, spin-resolved.
  - `N_NX_NY_NZ_TAUN` (id=25, inlen=5).
  - `N_S_NX_NY_NZ_SX_SY_SZ_TAUN_TAUS` (id=26, inlen=10).
  Pattern: identical to Phase 3 Wave-1 / Plan 03-01 — comptime if-chain extends, explicit helper-function chains (D-11 from Phase 3), no C-fallthrough. Source of truth: `xcfun-master/src/densvars.hpp:35-218` + `crates/xcfun-core/src/enums.rs:73-108`.
- **metaGGA shared helpers (Wave 0):** port the Phase-4 helper modules under `crates/xcfun-eval/src/functionals/mgga/shared/`:
  - `shared/scan_like.rs` — port of `SCAN_like_eps.hpp` (shared by SCANC, RSCANC, RPPSCANC, R2SCANC, R4SCANC; shared by exchange via `SCAN_like_eps`-derived enhancement).
  - `shared/tpss_like.rs` — TPSS exchange + correlation helpers (z, alpha, q-tilde construction).
  - `shared/m0x_like.rs` — Minnesota M0x kinetic-energy enhancement skeleton (shared by M05, M05-2X, M06, M06-2X, M06-L, M06-HF, plus correlation pieces).
  - `shared/br_like.rs` — BR Newton-inverse driver (host-side scalar `BR(z)` + comptime `ctaylor_br_inverse<F, N>` wrapper) + the `densvars` field-stitching used by `brx::energy`.
  - `shared/blocx.rs` — BLOCX setup that reuses BRX with a different prefactor (BLOCX = "B-LOC eXchange" — composes BRX).
  - `shared/cs.rs` — CSC (Colle–Salvetti correlation) helpers (composes Becke–Roussel-style algebra).
  Pattern mirrors Phase 3 D-08 (`gga/shared/{pw91_like, pbex, pbec_eps, b97_poly, optx, constants}.rs`).
- **32 metaGGA-tier `#[cube] fn <name>_kernel<F, const N>` bodies** in `crates/xcfun-eval/src/functionals/mgga/*.rs`:
  - **TPSS family (5, MGGA-01):** `XC_TPSSX`, `XC_TPSSC`, `XC_REVTPSSX`, `XC_REVTPSSC`, `XC_TPSSLOCC`.
  - **SCAN family (10, MGGA-02):** `XC_SCANX`, `XC_SCANC`, `XC_RSCANX`, `XC_RSCANC`, `XC_RPPSCANX`, `XC_RPPSCANC`, `XC_R2SCANX`, `XC_R2SCANC`, `XC_R4SCANX`, `XC_R4SCANC`.
  - **M05 family (4, MGGA-03):** `XC_M05X`, `XC_M05C`, `XC_M05X2X`, `XC_M05X2C`.
  - **M06 family (8, MGGA-04):** `XC_M06X`, `XC_M06C`, `XC_M06LX`, `XC_M06LC`, `XC_M06HFX`, `XC_M06HFC`, `XC_M06X2X`, `XC_M06X2C`.
  - **BLOCX (1, MGGA-05):** `XC_BLOCX`.
- **4 Phase-3 carryovers (Wave 2 alongside metaGGA, per Phase 3 D-01-A):**
  - **BR family (3, GGA-03):** `XC_BRX`, `XC_BRC`, `XC_BRXC` — depend on the `id=17` Vars arm + `ctaylor_br_inverse` primitive.
  - **CSC (1, GGA-10 minus LB94):** `XC_CSC` — depends on `id=17` Vars arm (and `cs.rs` shared helper). Note: CSC's `Dependency::KINETIC|LAPLACIAN|JP` is the same as BR family — same Vars arm covers both.
- **`Mode::Contracted` (D-06, MODE-03):** line-for-line port of `XCFunctional.cpp:619-635` `DOEVAL` macro. Comptime monomorphization over orders 0..=6: a single host-side dispatch maps `fun->order` to one of seven `#[cube] fn contracted_kernel<F, const ORDER>` invocations (CTaylor<F, 0> through CTaylor<F, 6>). Inputs are pre-computed Taylor coefficients (`(1 << order)` doubles per Vars-element, packed); outputs are the `(1 << order)` flat coefficient array `out.get(i)` for `i ∈ 0..(1 << order)`. Order cap = 6 (matches `XCFUN_MAX_ORDER`). Vars compatibility: any Vars whose `_2ND_TAYLOR` exists is compatible at order 2; orders > 2 require pre-seeded higher-order coefficients (caller's responsibility per `XCFunctional.cpp:619-635`).
- **Alias engine (Wave 4, D-04):**
  - **Alias table** populated at `crates/xcfun-core/src/registry/generated/aliases.rs` via `xtask regen-registry` (Phase 2 CORE-08 already established this pipeline; Phase 2 ships an empty slice, Phase 4 fills 46 entries).
  - **Multiplicative weight semantics** (D-04): port `XCFunctional.cpp:391-405` recursive `xcfun_set` resolution — when `set(name, value)` matches an alias, recurse into each `(term_name, weight)` pair with `set(term_name, value * weight)`. Functional weights ACCUMULATE (`settings[item] += value`) per L373; parameter weights OVERWRITE (`settings[item] = value`) per L387. Faithfully port the upstream **FIXME at L390** about EXX in aliases — preserve current C++ behaviour bit-for-bit (the FIXME-flagged code currently weights parameters by `value` even though its own comment expresses doubt; algorithmic-identity rule requires preservation).
  - **Negative-weight `camcompx` canary** explicitly tested per Success Criteria 3.
  - **MAX_ALIAS_TERMS = 10**, **XC_MAX_ALIASES = 60** (xcint.hpp:26-27); 46 actual entries fit comfortably.
- **Parameter table (Wave 4, D-05):**
  - **4 parameters** with C++ defaults from `xcfun-master/src/functionals/common_parameters.cpp:17-27`:
    - `XC_RANGESEP_MU` default `0.4`
    - `XC_EXX` default `0.0`
    - `XC_CAM_ALPHA` default `0.19`
    - `XC_CAM_BETA` default `0.46`
  - Storage shares the `settings[]` table with functionals (C++ pattern: `XC_NR_PARAMETERS_AND_FUNCTIONALS` = 78 + 4 = 82). Parameter ids = 78..=81. The Rust `FunctionalId` enum stays at COUNT=78; parameter ids live in a sibling `ParameterId` enum (4 variants) with `#[repr(u32)]` discriminants 78..=81.
  - Read path: `Functional::get(name)` resolves either functional or parameter; alias lookup is set-only (matches L405 — `xcfun_get` does NOT resolve aliases).
- **`dispatch_kernel` extension (D-08):** add 32 + 4 = 36 new comptime arms (28 metaGGA + 4 carryover). `supports(id)` bitmap bumps from 51 to 87. Pattern unchanged from Phase 3 D-04. Order-cap dispatch for `Mode::Contracted` adds 7 new comptime arms (orders 0..=6) at the host-side level; per-functional kernels remain `#[cube] fn <name>_kernel<F: Float, const N: u32>`. The Mode::Contracted host-side wrapper packs inputs and unpacks outputs around the existing kernel — no kernel-body changes for already-shipped functionals (LDA + GGA).
- **Tier-2 harness extension (D-09):** extend `validation/build.rs` to cc-compile the metaGGA `.cpp` + `.hpp` files incrementally per family wave. Shrink `validation/c_stubs.cpp` from ~32 (post-Phase-3) to ~3 (LB94 + any non-FUNCTIONAL-macro stubs). Grid generator gets metaGGA strata: random `tau ∈ [0, kF^2 * n^(2/3)]`-scaled, random `lap` near zero, random `JP_aa`, `JP_bb` on the `id=17` Vars points. Seed remains `0x1234abcd` for the canonical 10k grid; metaGGA-specific stratum gets a sibling seed (e.g., `0xc0ffee01`) to keep grid reproducibility additive across phases.
- **Phase 3 D-19 forward inheritance (D-10):** the 13 D-19 INCONCLUSIVE entries from Phase 3 (5 Wave-3 PW86X/APBEX/APBEC/P86C/PW91C + 3 Wave-4 B97C/B97_1C/B97_2C + 5 from full-matrix SPBEC/PBEINTC/PW91K/P86CORRC/BECKESRX) **stay forwarded to Phase 6** unchanged. Phase 4 does NOT attempt resolution. Rationale: those drifts are libm/port-order issues that Phase 6's libm-hybrid + asm-spot-check work is designed to address; spending Phase-4 budget on them dilutes the metaGGA + alias delivery.
- **`Mode::Potential` for metaGGAs is REJECTED** (inherited from Phase 3 D-13): `eval_setup` returns `XcError::InvalidMode` when `fun->depends & (LAPLACIAN | KINETIC) != 0` and `fun->mode == Potential`. Phase 4 adds NO `Mode::Potential` support for metaGGAs — Phase 6 + 7 are the appropriate venues if a future need arises. **`Mode::Potential` for metaGGA-Vars 2ND_TAYLOR variants** (e.g., a hypothetical metaGGA potential via 2nd-order Taylor) is not in the C++ reference and is therefore out of scope per algorithmic-identity rule.

**Out of scope (downstream):**

- **LB94 — deferred to Phase 5** (D-13). Confirmed not alias-feasible: LB94 uses the legacy `setup_lb94` pattern in `xcfun-master/src/functionals/lb94.cpp:1-60`, NOT the FUNCTIONAL macro, and is NOT a multiplicative composition of existing functionals — it's a GGA-modified LDA *potential* with no well-defined energy (per its own `.cpp` comment). REQUIREMENTS GGA-10 wording stays as Phase 3 left it; Phase 5 owns the FunctionalId-extension OR special-case dispatch decision.
- **`Mode::Potential` for metaGGAs** — never in scope (algorithmic-identity rule; C++ rejects it).
- **`Mode::PartialDerivatives` orders 5..=6** — not in C++ reference (XCFunctional.cpp falls through `case 3:` to `case 2:` per Phase 3 D-15 commentary; orders 5/6 are Contracted-only per `XCFUN_MAX_ORDER`).
- **GPU backends** (CUDA / Wgpu) — Phase 4 remains cubecl-cpu only. Phase 6 owns runtime-feature gating + tier-3 parity.
- **C ABI (`xcfun-capi`, Phase 5); Python (`xcfun-py`, Phase 7); facade (`xcfun-rs`, Phase 5).**
- **Full `Functional` API surface (RS-01..10)** — Phase 4 keeps the minimal dispatcher pattern from Phases 2/3, extending only with parameter setters, alias resolution, and Mode::Contracted dispatch. The `Functional::set(name, value)` entry point is wired in Phase 4 because aliases require it; the rest of RS-01..10 lands in Phase 5.
- **Criterion benchmarks for metaGGA** (PERF-01) — Phase 6.
- **Phase 6 libm-hybrid resolution for the 13 Phase-3 D-19 forwards** — inherited unchanged.

</domain>

<decisions>
## Implementation Decisions

### Scope, ordering & wave strategy

- **D-01:** **Phase 4 ports 32 functional bodies** (28 metaGGA + 4 carryovers BRX/BRC/BRXC + CSC). Plus alias engine (46 entries), parameter table (4 entries), Mode::Contracted (orders 0..=6). LB94 is NOT included — Phase 3 D-19 forwarded it to Phase 5 and Phase 4 reaffirms (see D-13 below). **Authorisation:** Auto-mode default; aligns with REQUIREMENTS MGGA-01..05 + Phase 3 D-01-A explicit carryovers.
- **D-01-A:** **Wave layout (planner finalises exact internal order):**
  - **Wave 0 (substrate):** xcfun-ad `ctaylor_br_inverse` primitive (one commit) + 11 new DensVarsDev Vars arms (one commit per arm or grouped) + metaGGA shared helper modules (one commit per module). Atomic; fixture-gate per primitive at strict 1e-12.
  - **Wave 1 (TPSS + BR + CSC, 9 functionals):** TPSS family (5) + BR family (3) + CSC (1). TPSS shares the `id=13` (TAUA_TAUB) Vars arm; BR + CSC share `id=17` (JPAA_JPBB).
  - **Wave 2 (SCAN family, 10 functionals):** SCAN, RSCAN, RPPSCAN, R2SCAN, R4SCAN — both X and C variants. Heavy on the `shared/scan_like.rs` module.
  - **Wave 3 (M05 + M06 + BLOCX, 13 functionals):** Minnesota families + BLOCX. Heavy on `shared/m0x_like.rs` + `shared/blocx.rs` (which composes BRX, so depends on Wave 1 BR ship).
  - **Wave 4 (alias engine + parameters):** populate `aliases.rs` registry (46 entries via `xtask regen-registry`), wire `Functional::set` recursion (multiplicative weights), wire 4 parameters with defaults from `common_parameters.cpp`, add `Functional::get(parameter_name)` path. Tier-1 alias-canary tests (b3lyp, m06, camcompx negative-weight, additive accumulation b3lyp+slaterx).
  - **Wave 5 (Mode::Contracted):** comptime `contracted_kernel<F, const ORDER>` orders 0..=6, host-side input pack / output unpack, eval_setup acceptance for Contracted, tier-2 parity at orders 0..=6 on a Contracted-mode subset of the 10k grid.
  - **Wave 6 (full-matrix tier-2 + Phase-4 sign-off):** Run `xtask validate --order 3` across all 78 - 1 (LB94) = 77 functional ids (Mode::PartialDerivatives) + Mode::Contracted spot checks at orders 5/6. Update REQUIREMENTS.md, ROADMAP.md Phase 4 success criteria, STATE.md. Forward any new D-19 INCONCLUSIVE entries to Phase 6 (NOT Phase 5 — Phase 5 is API-surface, not numerical).
  Planner may split / fuse waves based on pattern-mapper output. **Authorisation:** Auto-mode default; mirrors Phase 3's wave structure scaled up for ~3× the functional count.
- **D-01-B:** **Per-family parallelism inside each wave** mirrors Phase 3: per-functional ports are embarrassingly parallel within a family wave; helper modules block their family. SCAN family is the largest single wave (10 functionals); planner may sub-wave SCAN_X (5) parallel with SCAN_C (5). **Authorisation:** Auto-mode default.

### xcfun-ad substrate extension (BR Newton-inverse only)

- **D-02:** **Add `ctaylor_br_inverse` primitive to `xcfun-ad`.** Port target: `xcfun-master/src/functionals/brx.cpp:25-72` (the C++ helper lives in `brx.cpp`, NOT in `taylor/tmath.hpp` — verify before porting; the `BR_taylor` template at `brx.cpp:54-72` is the linear-method polynomial inverter we need). Implementation outline:
  1. Host-side scalar Newton-Raphson `BR(z) -> x` solving `BR_z(x) = z` where `BR_z(x) = (x-2)/x * exp(2/3 * x)`. 20-iter cap, 1e-15 relative convergence (matches C++).
  2. `#[cube] fn ctaylor_br_inverse<F: Float, const N: u32>(z: &CTaylor<F, N>, out: &mut CTaylor<F, N>, #[comptime] n: u32)` linear-method (Brent–Kung) coefficient sweep:
     - `out[CNST] = BR(z[CNST])` (scalar Newton on host slot).
     - `out[VAR0] = 1`; evaluate `f = BR_z(out)`; `out[VAR0] = 1 / f[VAR0]`.
     - For `i in 2..=N_terms`: evaluate `f = BR_z(out)`, set `out[i] = -f[i] * out[VAR0]`.
  3. Numerical fixture: 30 z-points covering the `BR(z)` dynamic range (`z < -1e4`, `-1e4 ≤ z < -2`, `-2 ≤ z < 1`, `z ≥ 1` — the four C++ initial-guess branches), order N ∈ {2, 3, 4} fixtures, strict 1e-12 vs C++ `BR_taylor`.
  4. Host-side scalar Newton runs in a `for_tests` cubecl single-launch kernel (per Phase 1 pattern) — no fast-math, no FMA, no reassociation.
  This is the only new xcfun-ad primitive in Phase 4. Every other metaGGA body composes from Phase 1/3 primitives. **Authorisation:** Auto-mode default — mandatory for GGA-03 (Phase 3 carryover) + MGGA-05 (BLOCX composes BRX); failing to add it blocks 4 functionals.
- **D-02-A:** **No other xcfun-ad additions.** All metaGGA bodies decompose into existing primitives. If a fixture-gate failure during Wave 1/2/3 surfaces a missing primitive (unlikely — the C++ metaGGA bodies use the same `exp`/`log`/`pow`/`sqrt`/`cbrt`/`atan`/`asinh`/`erf` palette as GGA), escalate via PLANNING INCONCLUSIVE per Phase 1 D-03; do NOT widen scope inside Phase 4 silently. **Authorisation:** Auto-mode default.

### DensVarsDev Vars arms (11 new arms for ids 8..=18 + 23..=26)

- **D-03:** **Add 11 new comptime arms in `build_densvars`** covering Vars discriminants 8..=18 + 23..=26 (the metaGGA + gradient-component metaGGA cluster). Authoritative discriminant source: `crates/xcfun-core/src/enums.rs:73-108`. Arms read raw fields (`a`, `b`, `gaa`, `gab`, `gbb`, `lapa`, `lapb`, `taua`, `taub`, `jpaa`, `jpbb`) and chain into the existing helper-function pattern that derives `n`, `s`, `gnn`, `gns`, `gss`, `lap`, `tau`, etc. **Pattern:** identical to Phase 3 D-10 + Plan 03-01 — explicit chains, no C-fallthrough, regularize-only-CNST invariant preserved (Phase 3 D-11 inheritance). **Authorisation:** Auto-mode default; verified against `xcfun-master/src/densvars.hpp:35-218` + `crates/xcfun-core/src/enums.rs:73-108`.
- **D-03-A:** **The `id=17` (`A_B_GAA_GAB_GBB_LAPA_LAPB_TAUA_TAUB_JPAA_JPBB`, inlen=11) arm is shared by BR family + CSC + (potentially) BLOCX**. Implement once; chain TAUA_TAUB / LAPA_LAPB / JPAA_JPBB derivations explicitly. The `JP` (current density) field mapping requires care — verify `densvars.hpp:200-210` for the precise field name (`d.jpaa`, `d.jpbb` or similar) before populating `DensVarsDev`. **Authorisation:** Auto-mode default; the inlen=11 Vars arm is the load-bearing structural change Phase 3 D-01-A flagged.
- **D-03-B:** **`DensVarsDev` struct may need new public fields** for `jpaa`, `jpbb`, `lap`, `tau` if they aren't already populated by Phase 2 (Plan 02-03 provisioned `tau`, `taua`, `taub`, `lapa`, `lapb` per Phase 3 code-context "Phase 3 → Phase 4 (metaGGA)"). Verify the field exists; if not, add (additive change to xcfun-eval, no other crate touched). **Authorisation:** Auto-mode default — Phase 3 D-08 noted Plan 02-03 provisioned tau/taua/taub/lapa/lapb. JP fields likely need adding.

### Alias engine (multiplicative weight semantics)

- **D-04:** **Line-for-line port of `XCFunctional.cpp:369-405`** (`xcfun_set` recursive resolution). Implementation:
  - `Functional::set(&mut self, name: &str, value: f64) -> Result<(), XcError>`:
    1. Lookup name as **functional id** → if found: `self.settings[id] += value`; if id not yet in `active_functionals`, append + bitwise-OR `depends`; return Ok.
    2. Lookup name as **parameter id** → if found: `self.settings[id] = value` (overwrites, NOT additive); return Ok.
    3. Lookup name as **alias id** → if found: iterate `terms[]` (max 10 terms per `MAX_ALIAS_TERMS`), recurse `self.set(term.name, value * term.weight)`. Recursion bounded by alias-graph depth (aliases-of-aliases not in upstream — every term resolves to a functional or a parameter); guard with a depth counter (max 3) or termination via "no alias resolves to another alias" invariant verified at registry generation time.
    4. Otherwise: return `Err(XcError::UnknownName)`.
  - **`Functional::get(&self, name: &str) -> Result<f64, XcError>`**: lookup functional or parameter ONLY (NOT alias — matches `xcfun_get` at L405 which returns -1 for aliases). Return `self.settings[id]`.
  - **Negative-weight handling:** `value * weight` carries sign through. The `camcompx` alias has `{"beckecamx", -1.0}` (or similar — verify in `aliases.cpp`); calling `set("camcompx", 0.37)` propagates `-0.37` into `beckecamx`'s setting via the additive accumulation rule.
  - **Hybrid functionals (b3lyp, pbe0, b97, ...) propagate `EXX` via the parameter overwrite rule (case 2)** — the FIXME at C++ L390 noting "Do not weight parameters with value for aliases, but what about EXX?" is preserved bit-for-bit. Algorithmic-identity rule: when a hybrid alias `b3lyp` with weight `1.0` resolves to `{"exx", 0.20}`, the recursive `set("exx", 1.0 * 0.20)` overwrites `settings[exx_id] = 0.20`. If a downstream caller then sets `set("b3lyp", 0.5)`, the recursion overwrites `settings[exx_id] = 0.5 * 0.20 = 0.10` (NOT `0.20 + 0.10 = 0.30`). This matches C++ behaviour exactly.
  **Authorisation:** Auto-mode default — algorithmic-identity rule + Success Criteria 3 & 4 require this exact semantics.
- **D-04-A:** **Alias table populated by `xtask regen-registry`** — extractor parses `aliases.cpp:17-139` (46 entries, each with name, description, terms[]). Output: `crates/xcfun-core/src/registry/generated/aliases.rs` (Phase 2 CORE-08 already provisioned the file path; Phase 4 fills the slice). Drift gate (`xtask regen-registry --check`) enforces parity with the `aliases.cpp` source. **Authorisation:** Auto-mode default — extends Phase 2 D-21 pipeline.
- **D-04-B:** **Case-insensitive name lookup** (matches C++ `strcasecmp` at xcint.cpp:40): use `eq_ignore_ascii_case` for the name match. Alias names in upstream are mixed-case (`b3lyp`, `M06`, `B97-1`, `pbe0`); preserve as-stored, compare case-insensitively. **Authorisation:** Auto-mode default — RS-02 success criterion explicitly says "case-insensitive name lookup".

### Parameter table

- **D-05:** **4 parameters with C++ defaults** stored at `crates/xcfun-core/src/registry/generated/parameters.rs`:
  - `XC_RANGESEP_MU` (id=78) default `0.4` — "Range separation inverse length [1/a0]"
  - `XC_EXX` (id=79) default `0.0` — exact exchange admixture
  - `XC_CAM_ALPHA` (id=80) default `0.19` — Coulomb-attenuating method α
  - `XC_CAM_BETA` (id=81) default `0.46` — Coulomb-attenuating method β
  Storage shares the `settings[]: [f64; 82]` array with functionals (matches C++ `XC_NR_PARAMETERS_AND_FUNCTIONALS = 82`). At `Functional::new`: `settings[0..78] = 0.0` (functional weights), `settings[78..82] = [0.4, 0.0, 0.19, 0.46]` (parameter defaults). **Authorisation:** Auto-mode default — mirrors `xcfun-master/src/XCFunctional.cpp:347-358` initialization + `common_parameters.cpp:17-27` defaults.
- **D-05-A:** **`ParameterId` enum in `xcfun-core`** (4 variants, `#[repr(u32)]`, discriminants 78..=81). Co-located in `enums.rs` next to `FunctionalId`. Lookup helper `parameter_id_from_name(&str) -> Option<ParameterId>` mirrors `functional_id_from_name`. **Authorisation:** Auto-mode default — clean Rust idiom while preserving C-ABI numbering.

### Mode::Contracted

- **D-06:** **Line-for-line port of `XCFunctional.cpp:619-635` `DOEVAL` macro** as 7 comptime kernels:
  - `#[cube] fn contracted_kernel<F: Float, const ORDER: u32>(...)` — one body, comptime-parametric on order. Inside: invoke the per-functional kernel at `N=ORDER` (CTaylor<F, ORDER> with `1 << ORDER` storage); pack inputs from a flat `[(1 << ORDER)] * inlen` host array; unpack outputs `out.get(i)` for `i ∈ 0..(1 << ORDER)`. Contracted Mode does NOT call the divergence loop; it calls the same `<name>_kernel<F, const N>` body per-active-functional, accumulates with weight, and reads `(1 << ORDER)` flat coefficients out (per `DOEVAL` line `output[i] = out.get(i)`).
  - Host-side dispatch: `Functional::eval` matches on `(mode, order)`; for `(Contracted, 0)` invoke `contracted_kernel<F, 0>`, for `(Contracted, 1)` invoke `contracted_kernel<F, 1>`, … through `(Contracted, 6)`. 7 comptime arms.
  - **Order cap = 6** (matches `XCFUN_MAX_ORDER`); `eval_setup` returns `XcError::InvalidOrder` for `order > 6` in Contracted mode. (Phase 3 D-15 already accepted PartialDerivatives 0..=4 with C++ runtime cap at 3; Phase 4 leaves PartialDerivatives unchanged at 0..=4.)
  - **Storage check:** CTaylor<F, 6> backing `Array<F>` is length `64`; `XC_MAX_INVARS = 20` from xcint.hpp:28; total per-point input is `20 * 64 = 1280` doubles = 10 KB on the stack inside one kernel invocation. Within Phase 1 / cubecl-cpu memory budget (verified at fixture-gate); no host-side heap allocation on the per-point hot path. **Authorisation:** Auto-mode default.
- **D-06-A:** **Vars compatibility for Contracted:** every Vars whose pre-computed Taylor coefficients can be packed at order N is compatible. `eval_setup` validates `input_len * (1 << order)` against the user's input array; no Vars-specific rejection beyond Phase 2's existing matrix. **Authorisation:** Auto-mode default — matches `XCFunctional.cpp:619-635` (no Vars rejection inside the DOEVAL macro).
- **D-06-B:** **`output_length` for Mode::Contracted = `1 << order`** (returns 1, 2, 4, 8, 16, 32, 64 for orders 0..=6). Encodes MODE-05 extension. **Authorisation:** Auto-mode default — direct spec transcription from `XCFunctional.cpp:482` (output count for Contracted).
- **D-06-C:** **Tier-2 Contracted-mode parity** at strict 1e-12 on a 1000-point Contracted-mode subset of the 10k grid (the C++ `eval` runs both modes from the same input data; we cross-check). Order-by-order: orders 0..=4 match scalar PartialDerivatives at `output[i] = result.get(i)` for the same density point (Contracted is a re-packaging of the same Taylor coefficients, NOT an algorithmic difference); orders 5..=6 cross-check requires extending the C++ harness to Contracted mode (`validation/build.rs` already has `xcfunctional.cpp` linked; no new source files needed). **Authorisation:** Auto-mode default.

### CTaylor<F, 6> capacity (no new primitive)

- **D-07:** **CTaylor<F, 6> is already declared valid at the type level** (Phase 1 `CTaylor<F, const N: u32>` with `N ∈ 0..=7` per AD-01). Phase 4 exercises it for the first time at order 6. Fixture-gate: a single `cargo test -p xcfun-ad --features cpu test_ctaylor_n6` smoke test in Wave 0 confirms allocation + arithmetic at N=6 before any kernel exercises it. If fixture-gate fails (unlikely — Phase 1 AD-06 ran property tests at orders 0..=3, and the type construction is generic), escalate via PLANNING INCONCLUSIVE per Phase 1 D-03. **Authorisation:** Auto-mode default — leverages existing AD-01 contract.

### Dispatch + supports() bitmap

- **D-08:** **`dispatch_kernel` gets 36 new comptime arms** (32 Phase-4 ports + 4 Phase-3 carryovers). `supports(id)` bitmap bumped from 51 to 87 ids. `Mode::Contracted` adds 7 host-side dispatch arms (ORDER 0..=6) wrapping the same per-functional kernels. **Authorisation:** Auto-mode default — direct extension of Phase 3 D-04.
- **D-08-A:** **The 91-id total** (78 functionals - 1 LB94 + 4 parameters = 81; `supports(id)` covers the 87 functional ids that are populated as kernels) reaches the bitmap-bit limit of `u128`. If bitmap exceeds 128 bits (extremely unlikely — 78 functional ids fit comfortably), planner switches to `[u64; 2]` or sparse representation. **Authorisation:** Auto-mode default.

### Validation harness (incremental per wave)

- **D-09:** **`validation/build.rs` extends per wave** — after each metaGGA family ships, append the corresponding C++ `.cpp` files to the `cc::Build::file(...)` list and shrink `c_stubs.cpp` accordingly. By Wave 6, `c_stubs.cpp` is reduced to ~3 stubs (LB94 + any non-FUNCTIONAL-macro extras). **Authorisation:** Auto-mode default — extends Phase 3 D-22.
- **D-09-A:** **Grid generator gains metaGGA strata** — extend the 10k-point grid (Phase 2 Plan 02-06 + Phase 3 Plan 03-01 design) with random `tau ∈ [0, kF^2 * n^(2/3)]` (positive-definite kinetic energy density), `lap` near zero (Laplacian sign random), `JP_aa`, `JP_bb` random in a physically reasonable range (paramagnetic current density components). Existing strata untouched. Sibling seed `0xc0ffee01` for reproducibility on the metaGGA stratum (the canonical seed `0x1234abcd` covers 0..=10k; metaGGA stratum extends to 10k..=11k). **Authorisation:** Auto-mode default — Phase 3 D-23 design intent for incremental extension.

### Phase 3 D-19 forwards: inheritance

- **D-10:** **The 13 D-19 INCONCLUSIVE entries from Phase 3 stay forwarded to Phase 6 unchanged.** Phase 4 does NOT attempt resolution. Rationale: those drifts (1e-6 to 4.88e-11 across PW86X/APBEX/APBEC/P86C/PW91C/B97C/B97_1C/B97_2C/SPBEC/PBEINTC/PW91K/P86CORRC/BECKESRX) are libm + port-order issues; Phase 6 has the libm-hybrid + asm-spot-check tooling that addresses them. Phase-4 budget goes to metaGGA + alias + Contracted delivery. **Authorisation:** Auto-mode default — explicit Phase 3 D-18 forward-action.

### Tier-2 parity threshold (metaGGA)

- **D-11:** **Strict 1e-12 default for metaGGA functionals.** The Phase 2 D-19 + Phase 3 D-18 framework applies: any per-functional override (e.g., 1e-7 for an `erf`-bearing metaGGA, analogous to LDAERF D-24) requires upstream documentation OR escalation via PLANNING INCONCLUSIVE. **Watch list (high-risk per literature):**
  - **M06 family**: known for numerical sensitivity; potential 1e-9..1e-10 drift if the polynomial enhancement evaluation order differs from C++. Apply algorithmic-identity rule strictly.
  - **R4SCAN**: high-degree polynomial; potential FMA / mul_add violation if a port slips. The `xtask check-no-mul-add` gate (Phase 2 D-24) catches this.
  - **BLOCX (composes BRX):** drift inherited from `ctaylor_br_inverse` fixture-gate; if Wave 0's fixture-gate hits the strict 1e-12 mark, BLOCX is safe.
  Any metaGGA that drifts above 1e-12 at fixture-gate time gets a D-19 INCONCLUSIVE entry forwarded to Phase 6 (mirroring Phase 3 protocol). **Authorisation:** Auto-mode default.
- **D-12:** **No blanket Mode::Contracted relaxation.** Mode::Contracted is a re-packaging of PartialDerivatives Taylor coefficients (per `XCFunctional.cpp:619-635`); if the underlying kernel passes 1e-12 in PartialDerivatives, Contracted at the same order trivially passes 1e-12. Phase 4 verifies this by cross-mode comparison on a 1000-point subset. **Authorisation:** Auto-mode default.

### LB94 disposition (final)

- **D-13:** **LB94 is NOT alias-feasible.** Examined `xcfun-master/src/functionals/lb94.cpp:1-60` and confirmed:
  1. LB94 uses the legacy `setup_lb94` pattern, NOT the FUNCTIONAL macro — it is a setup function that mutates a functional struct, not a multiplicative composition of existing terms.
  2. LB94 has no well-defined energy (per its own comment: "the LB94 energy is not well defined") — aliases compose energies multiplicatively, so this is structurally incompatible.
  3. LB94 is NOT in the 78-entry FunctionalId enum — adding it requires either (a) extending FunctionalId enum to COUNT=79 with manual discriminant assignment + a hand-authored FunctionalDescriptor, or (b) a special-case dispatch branch.
  All three options belong in **Phase 5 (facade + C ABI)** where the user-facing `Functional` API surface lands and where new variants are least disruptive. **Action item:** REQUIREMENTS.md GGA-10 wording stays as Phase 3 left it; Phase 5 explicitly owns LB94. Phase 4 does NOT re-attempt the alias-feasibility check. **Authorisation:** Auto-mode default — examined `lb94.cpp` and confirmed the Phase 3 D-19 expectation.

### Error model (no new variants)

- **D-14:** **No new `XcError` variants.** Phase 4 reaches new rejection paths in existing match arms:
  - `XcError::UnknownName` — case in `Functional::set` for names that match neither functional, parameter, nor alias.
  - `XcError::InvalidOrder` — case in `eval_setup` for `Mode::Contracted` with `order > 6`.
  - `XcError::InvalidMode` — already wired (Phase 3 D-13 metaGGA rejection); no change.
  - `XcError::InputLengthMismatch` — already wired (Phase 2 D-25); reachable in Contracted with mismatched `(1 << order) * inlen` user input.
  `XcError` stays at 9 variants, `Copy + Clone + Debug + non_exhaustive + thiserror::Error`. **Authorisation:** Auto-mode default — direct reuse of Phase 2 / Phase 3 error model.

### Claude's Discretion

- **Per-functional file layout:** one file per metaGGA functional (`tpssx.rs`, `tpssc.rs`, `scanx.rs`, `m06x.rs`, …) under `crates/xcfun-eval/src/functionals/mgga/<family>/<fn>.rs` OR flat `mgga/<fn>.rs` mirroring Phase 3's GGA flat layout. Planner picks based on compile-unit cohesion.
- **Helper-module granularity:** whether `shared/scan_like.rs` fuses with `shared/tpss_like.rs` (TPSS uses similar α / z constructions to early SCAN drafts) — planner decides.
- **Wave 1 internal ordering:** TPSS-first (5 functionals) vs BR-first (3 functionals — unblocks BLOCX in Wave 3). Planner picks.
- **Alias registry layout:** flat `&'static [Alias]` slice vs per-letter sub-table for binary search. Planner picks (case-insensitive comparison via `eq_ignore_ascii_case` works either way; binary search needs lowercased keys).
- **Parameter storage layout:** single `[f64; 82]` `settings` array (matches C++) vs split `functionals: [f64; 78] + parameters: [f64; 4]`. Planner picks; C-ABI compatibility (Phase 5) prefers the single array.
- **Mode::Contracted host-side input packing:** in-place pre-seed vs allocated scratch buffer. Planner picks; the per-point hot-path zero-allocation rule (Phase 1 D-01) requires in-place.
- **Alias depth-guard mechanism:** explicit recursion depth counter (max 3) vs registry-time invariant ("no alias resolves to another alias"). Planner picks; the registry-time invariant is stronger but adds an `xtask regen-registry --check` rule.
- **`DensVarsDev` JP-field naming:** verify `xcfun-master/src/densvars.hpp:200-210` for the precise field name (`d.jpaa`, `d.jpbb`); planner adopts the upstream name verbatim.

### Folded Todos

None surfaced — STATE.md `Active TODOs` lists only Phase 3 follow-ups in 03-HUMAN-UAT.md (order-3 capstone re-run, BECKESRX D-18 forensics, full 36-GGA Mode::Potential sweep), which are explicitly out of Phase 4 scope (HUMAN-UAT items belong in `/gsd-verify-work 3`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### C++ reference (algorithmic-identity source of truth)

- `xcfun-master/src/XCFunctional.cpp:347-358` — `XCFunctional::XCFunctional` constructor: settings[] initialization with parameter defaults.
- `xcfun-master/src/XCFunctional.cpp:369-405` — `xcfun_set` recursive resolution for functionals + parameters + aliases. **Line-for-line port target for D-04.**
- `xcfun-master/src/XCFunctional.cpp:407-419` — `xcfun_get` lookup (functional + parameter only, NOT alias). **Port target for D-04.**
- `xcfun-master/src/XCFunctional.cpp:619-635` — `DOEVAL` macro expansion for `Mode::Contracted`. **Port target for D-06.**
- `xcfun-master/src/XCFunctional.cpp:482-490` — `output_length` for Mode::Contracted = `1 << order`. **Port target for D-06-B.**
- `xcfun-master/src/functionals/aliases.cpp:17-139` — 46 alias entries with weights. **Port source for D-04-A.**
- `xcfun-master/src/functionals/common_parameters.cpp:17-27` — 4 parameter defaults. **Port source for D-05.**
- `xcfun-master/src/functionals/list_of_functionals.hpp:100-103` — parameter ids extension `XC_RANGESEP_MU = XC_NR_FUNCTIONALS, XC_EXX, XC_CAM_ALPHA, XC_CAM_BETA`. **Port source for D-05-A.**
- `xcfun-master/src/xcint.hpp:26-28` — `XC_MAX_ALIASES = 60`, `MAX_ALIAS_TERMS = 10`, `XC_MAX_INVARS = 20`. **Capacity constants for D-04 + D-06.**

### metaGGA functional sources (28 + BR + CSC = 32 ports)

- **TPSS family (5, MGGA-01):**
  - `xcfun-master/src/functionals/tpssx.cpp`, `tpssc.cpp`, `revtpssx.cpp`, `revtpssc.cpp`, `tpsslocc.cpp`.
- **SCAN family (10, MGGA-02):**
  - `xcfun-master/src/functionals/SCANx.cpp`, `SCANc.cpp`, `rSCANx.cpp`, `rSCANc.cpp`, `rppSCANx.cpp`, `rppSCANc.cpp`, `r2SCANx.cpp`, `r2SCANc.cpp`, `r4SCANx.cpp`, `r4SCANc.cpp`.
  - `xcfun-master/src/functionals/SCAN_like_eps.hpp` — shared correlation kernel (port target for `shared/scan_like.rs`).
- **M05 family (4, MGGA-03):** `m05x.cpp`, `m05c.cpp`, `m05x2x.cpp`, `m05x2c.cpp`.
- **M06 family (8, MGGA-04):** `m06x.cpp`, `m06c.cpp`, `m06lx.cpp`, `m06lc.cpp`, `m06hfx.cpp`, `m06hfc.cpp`, `m06x2x.cpp`, `m06x2c.cpp`.
- **BLOCX (1, MGGA-05):** `blocx.cpp` — composes BRX; depends on Wave-1 BR ship.
- **BR family carryover (3, GGA-03 → Phase 4 per Phase 3 D-01-A):** `brx.cpp`, `brc.cpp`, `brxc.cpp`. **`brx.cpp:25-72` is the port target for `ctaylor_br_inverse` D-02.**
- **CSC carryover (1, GGA-10 → Phase 4 per Phase 3 D-01-A):** `cs.cpp`.
- **LB94 (REFERENCE ONLY; D-13 defers to Phase 5):** `lb94.cpp`.
- **Densvars structural reference:** `xcfun-master/src/densvars.hpp:35-218` (case branches for ids 8..=18 + 23..=26) — D-03 explicit-chain port target.

### Mode::Contracted runtime

- `xcfun-master/src/XCFunctional.cpp:619-635` — DOEVAL macro (D-06).
- `xcfun-master/src/functional.hpp` — `FUNCTIONAL` macro defines `fp0`..`fp6` per-order energy entry points; D-06's per-order kernel selection mirrors this.
- `xcfun-master/src/xcint.cpp:29` — `xcint_params[XC_NR_PARAMETERS_AND_FUNCTIONALS]` shared array (D-05 storage layout).

### cubecl substrate (Phase 1 + Phase 3 outputs — unchanged except for D-02)

- `crates/xcfun-ad/src/{ctaylor.rs, ctaylor_rec/*, math.rs, tfuns.rs, expand/*}` — Phase 1 + Phase 3 deliverables. **D-02 adds one new primitive (`ctaylor_br_inverse`).** D-02-A commits to no further additions.
- `crates/xcfun-ad/src/index.rs` — CNST, VAR0..VAR7 indexing; no changes.
- `cubecl_core::prelude::{Float, Array, CUBE}` + `cubecl_cpu::CpuRuntime` — substrate unchanged from Phase 2/3.

### Phase 2 + Phase 3 substrate (consumed unchanged except for additive extensions)

- `crates/xcfun-eval/src/density_vars.rs` — `DensVarsDev<F>` 22-field struct. **D-03-B may need to add JP fields if not provisioned.**
- `crates/xcfun-eval/src/density_vars/build.rs` — existing arms (XC_A_B, XC_A_B_GAA_GAB_GBB + Phase 3's 7 GGA + 4 2ND_TAYLOR arms = 13 total). **D-03 adds 11 new arms (ids 8..=18, 23..=26).**
- `crates/xcfun-eval/src/density_vars/regularize.rs` — unchanged; preserved by D-11 explicit chains (Phase 3 D-11 inheritance).
- `crates/xcfun-eval/src/dispatch.rs` — `dispatch_kernel` + `supports(id)`. **D-08 extends by 36 functional arms + 7 Mode::Contracted host-side arms.**
- `crates/xcfun-eval/src/functional.rs` — `Functional` struct + `eval` method. **D-04 adds `set`/`get` recursion paths; D-06 adds Mode::Contracted dispatch path; D-06-B adds `output_length` Contracted arms.**
- `crates/xcfun-eval/src/functionals/{lda,gga}/*` — 11 LDA + ~36 GGA kernels unchanged; provide pattern templates for the 32 Phase-4 ports.
- `crates/xcfun-eval/src/functionals/lda/vwn_eps.rs`, `pw92eps.rs` — helper-module pattern template.
- `crates/xcfun-eval/src/functionals/gga/shared/*` — Phase 3 helper modules; pattern template for `mgga/shared/`.

### Registry + validation (consumed, extended)

- `xtask regen-registry` — extraction driver. **D-04-A extends to populate the 46 aliases at `crates/xcfun-core/src/registry/generated/aliases.rs`. D-05 extends to populate the 4 parameters at `crates/xcfun-core/src/registry/generated/parameters.rs`** (or equivalent path; pattern must match Phase 2 CORE-08 output convention).
- `validation/build.rs` — cc-compile C++ ref. **D-09 extends `cc::Build::file(...)` list per wave.**
- `validation/c_stubs.cpp` — auto-generated. **Shrinks from ~32 (post-Phase-3) to ~3 by Wave 6** (only LB94 + non-FUNCTIONAL-macro stubs remain).
- `validation/src/fixtures.rs` — 10k-point grid generator. **D-09-A extends with metaGGA stratum at sibling seed `0xc0ffee01`.**
- `validation/report.html` + `validation/report.jsonl` — tier-2 verdict artifacts. **Wave 6 commits an updated snapshot.**

### Phase 1 + Phase 2 + Phase 3 locked decisions (inherited verbatim)

- `.planning/phases/01-taylor-algebra-ad-primitives-xcfun-ad/01-CONTEXT.md` — 28 locked decisions. Of particular relevance for Phase 4: **D-01 (1e-12 strict on cubecl-cpu), D-02 (no mul_add), D-03 (PLANNING INCONCLUSIVE escalation rule for fixture-gate failures), D-09 (Num retired → Float), D-23 (per-functional `#[cube]` bodies land in Phases 2–4)**.
- `.planning/phases/02-core-foundations-lda-tier-parity-harness/02-CONTEXT.md` — 25 locked decisions. Of particular relevance: **D-02 (DensVarsDev as CubeType), D-03 (single generic kernel signature), D-19 (strict 1e-12 no blanket relaxation), D-21 (xtask regen-registry pipeline), D-24 (LDAERF 1e-7 upstream-sourced override pattern), D-25 (XcError 9 variants, no new ones in Phase 3/4)**.
- `.planning/phases/03-gga-tier-mode-potential/03-CONTEXT.md` — 25 locked decisions + 4 amendments. Of particular relevance: **D-01 (40-not-45 GGA count rule, see D-01-A amendment for the 36-not-40 reduction), D-04 (dispatcher comptime arms pattern), D-08 (gga/shared/ helper module pattern), D-10 (DensVarsDev arm pattern), D-11 (explicit helper-function chains, no fallthrough), D-13 (Mode::Potential metaGGA rejection), D-18 (BECKESRX/BECKECAMX strict 1e-12), D-19 (LB94 deferral), D-20 (ACC-04 re-run completed in Plan 03-06)**.
- `.planning/phases/03-gga-tier-mode-potential/03-06-SUMMARY.md` — Phase 3 capstone documenting the 13 D-19 INCONCLUSIVE forwards to Phase 6 (carried unchanged by D-10 above).

### Design brief (updates required per Phase 4)

- `docs/design/02-data-structures.md` §5 (`DensVars`) — needs update for the 11 new Vars arms (D-03) + D-03-A inlen=11 JP arm.
- `docs/design/03-api-surface.md` — `Functional::set` recursion semantics (D-04) + parameter table (D-05). **Note: full RS-01..10 surface lands in Phase 5; Phase 4 only ships the `set`/`get`/`eval` cross-section needed for aliases + Mode::Contracted.**
- `docs/design/04-control-flow.md` — Mode::Contracted dispatch (D-06).
- `docs/design/05-module-responsibilities.md` §3 (xcfun-eval) — metaGGA layout + shared helpers (D-01-A wave 0).
- `docs/design/06-cubecl-strategy.md` §3 — per-functional inner kernels metaGGA extension; ctaylor_br_inverse primitive (D-02).
- `docs/design/07-accuracy-strategy.md` §4 (tier architecture) — metaGGA tier added; §6 tolerance budget gains a metaGGA row.
- `docs/design/08-error-model.md` — no new variants (D-14); annotation update only.
- `docs/design/09-testing-strategy.md` — tier-1 metaGGA self-tests + tier-2 metaGGA parity + Mode::Contracted cross-mode comparison.
- `docs/design/11-process-and-milestones.md` §M5 — Phase 4 entry/exit criteria.
- `docs/design/12-design-decisions.md` — add Phase-4 decisions section.

### Research (pitfalls + phase mapping)

- `.planning/research/SUMMARY.md` "Implications for Roadmap" → Phase 4 (alias multiplicative semantics + Mode::Contracted at orders 5-6).
- `.planning/research/PITFALLS.md` **P5 (C-fallthrough — D-03 explicit chains), P9 (`*_expand` miscopy — D-02 fixture-gate for ctaylor_br_inverse), P10 (silent NaN — `BR(z)` Newton convergence preconditions in D-02), P13 (registry drift — `xtask regen-registry` rerun per D-04-A + D-05)**. **P11 (alias misweighting)** is newly relevant for D-04 — preserve the FIXME-flagged C++ behaviour bit-for-bit.
- `.planning/research/STACK.md` — `cubecl =0.10.0-pre.3`, `rand_xoshiro =0.8.0` (unchanged).

### Project-level

- `.planning/PROJECT.md` — Core Value (1e-12 parity); "Active" lists "All 4 tunable parameters" + "All 31 Vars combinations" + "All 3 evaluation modes" — Phase 4 closes 3 of these.
- `.planning/REQUIREMENTS.md` — MGGA-01..05, MODE-03, ALIAS-01..06. **REQUIREMENTS GGA-03 + GGA-10 (CSC portion) update:** Phase 4 ships the deferred BR family + CSC; flip those rows from "Deferred per D-01-A" to "Complete (Phase 4)" at sign-off.
- `.planning/ROADMAP.md` — Phase 4 Goal + 5 Success Criteria. Goal sentence's "78 functionals" needs an "(minus LB94, deferred to Phase 5)" annotation at sign-off.
- `.planning/STATE.md` — Accumulated Context + Session Continuity sections.
- `CLAUDE.md` — tech-stack pins (`cubecl =0.10.0-pre.3`); f64-only numerical path; anyhow allowed only in validation/xtask/benches.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets (from Phase 1 + 2 + 3)

- **`crates/xcfun-ad/src/math.rs` + `expand/*`** — 11 composed ops (reciprocal, sqrt, exp, log, pow, erf, asinh, atan, powi_0..=10, expm1 from Phase 3, sqrtx_asinh_sqrtx from Phase 3). All reusable for metaGGA bodies. **D-02 adds `ctaylor_br_inverse` only.**
- **`crates/xcfun-ad/src/ctaylor.rs`** — ctaylor_add, ctaylor_sub, ctaylor_scalar_mul, ctaylor_zero. Reused extensively in metaGGA bodies.
- **`crates/xcfun-ad/src/ctaylor_rec/mul.rs`** — ctaylor_mul. The hot-loop primitive in every metaGGA body.
- **`crates/xcfun-eval/src/density_vars.rs`** — `DensVarsDev<F>` 22-field struct. metaGGAs read `d.tau`, `d.taua`, `d.taub`, `d.lapa`, `d.lapb` (provisioned in Plan 02-03 per Phase 3 code-context). **D-03-B may add `d.jpaa`, `d.jpbb` if missing.**
- **`crates/xcfun-eval/src/functionals/{lda,gga}/*`** — pattern templates (per-functional file, kernel signature `#[cube] fn <name>_kernel<F: Float, const N: u32>(...)`, helper modules). Adopt verbatim.
- **`crates/xcfun-eval/src/functionals/gga/shared/{pw91_like, pbex, pbec_eps, b97_poly, optx, constants}.rs`** — Phase 3 helper-module template; mirror for `mgga/shared/{scan_like, tpss_like, m0x_like, br_like, blocx, cs}.rs` (D-01-A Wave 0).
- **`crates/xcfun-eval/src/dispatch.rs`** — 51-arm comptime if-chain (post-Phase-3); D-08 extends by 36 + 7 = 43 new arms.
- **`crates/xcfun-eval/src/functional.rs`** — `Functional::eval` host-side dispatch. D-04 adds set/get recursion; D-06 adds Mode::Contracted host-side dispatch; D-06-B adds output_length Contracted arms.
- **`validation/src/` + `validation/build.rs`** — tier-2 harness; D-09 extends incrementally per wave.
- **`xtask/src/regen_registry.rs`** — extractor driver; D-04-A + D-05 extend extraction targets.
- **`xcfun-master/src/functionals/`** — C++ reference; cc-compiled per `validation/build.rs`. 32 metaGGA + carryover `.cpp` files are the port source.

### Established Patterns

- **Kernel signature: `#[cube] fn <name>_kernel<F: Float>(d: &DensVarsDev<F>, out: &mut Array<F>, #[comptime] n: u32)`** — universal across Phase 2 LDAs + Phase 3 GGAs; every metaGGA body adopts verbatim.
- **Kernel body structure:** 1:1 port of C++ `energy_fn(densvars<num> & d)` → Rust `#[cube] fn ... { let t1 = ...; ctaylor_mul(&a, &b, &mut t1, n); ... out[...] = ...; }`. Operation order preserved; mul_add banned (ACC-06); no FMA.
- **Helper-module pattern (Phase 2 → Phase 3):** complex algebra extracted into `<tier>/shared/<helper>.rs`, re-used by multiple functional bodies. Mirror at `mgga/shared/`.
- **Algorithmic-identity port rule (Phase 1 D-01 → Phase 2 D-03 → Phase 3 D-08):** verbatim port, preserve recursion structure, no reassociation, no SIMD intrinsics. Let-bindings mirror C++ intermediate names.
- **Fixture-gate escalation (Phase 1 D-03 + Phase 2 D-19):** strict 1e-12; escalate via PLANNING INCONCLUSIVE rather than widen. D-11 inherits.
- **Atomic-commits per wave task (Phase 2 D-09, Phase 3 inheritance):** one commit per substrate extension, one commit per family wave's functional ports grouped by family.
- **`#[forbid(unsafe_code)]`** — xcfun-core + xcfun-eval crate-root attributes; Phase 4 preserves.
- **Phase 3 dispatch pattern** for `Mode::Contracted`: comptime `const ORDER: u32` parameter on the host-side wrapper kernel; per-functional kernels remain `<name>_kernel<F, const N>` — the wrapper invokes them at the order matching `ORDER`.

### Integration Points

- **Phase 3 → Phase 4:** xcfun-ad receives 1 new primitive (D-02). xcfun-eval receives 11 new Vars arms (D-03), 32 new metaGGA + carryover kernels (D-01), new mgga/shared/ helper modules (D-01-A Wave 0), dispatcher 36-arm extension (D-08), Mode::Contracted host-side + per-order kernels (D-06), parameter table (D-05), alias engine (D-04). xcfun-core receives `ParameterId` enum (D-05-A) + alias registry slice (D-04-A).
- **Phase 4 → Phase 5 (facade + C ABI):** xcfun-rs::Functional re-exports the now-complete `Functional::set/get/eval/eval_setup/output_length` surface from xcfun-eval. LB94 + the full RS-01..10 surface (RS-01 new, RS-09 free functions) land here.
- **Phase 4 → Phase 6 (GPU + tier-3):** every new `#[cube] fn` metaGGA body compiles unchanged for CudaRuntime + WgpuRuntime (subject to f64). Phase 6 adds tier-3 parity at 1e-13 (CPU vs CUDA) and 1e-9 (CPU vs Wgpu with `erf` fallback). The 13 Phase-3 D-19 forwards + any new Phase-4 D-19 forwards all converge here for libm-hybrid resolution.
- **Phase 4 → Phase 7 (Python):** `Functional::set`/`get`/`eval`/`eval_vec` (the latter is Phase 6) become the surface PyO3 wraps.

### No pre-phase code to remove

Phase 4 is purely additive. No analogues of Phase 1 Wave 0 revert or Phase 2 surgical cleanup — the LDA + GGA tiers stay verbatim.

### Known Hazards (from research + Phase 2/3 forensics)

- **`ctaylor_br_inverse` Newton convergence:** the C++ `BR(z)` scalar Newton runs 20 iterations with 1e-15 relative convergence. If Rust f64 differs from C++ f64 in the iteration trajectory (different libm `exp` / `log`), the converged x can differ by 1-3 ULP. Strict 1e-12 fixture-gate may force a per-functional 1e-11 override OR an algorithmic-identity matching of the Newton trajectory. **Mitigation:** match libm calls inside Newton verbatim (`f64::exp` / `f64::log` come from the same glibc as the C++ side on the validation runner; if they differ, the Phase-2 LDAERFX `erf_precise` libm port pattern applies analogously to `BR_z`'s `exp`).
- **Alias FIXME parity:** the C++ FIXME at L390 ("Do not weight parameters with value for aliases, but what about EXX?") is a known wart. Tests must verify the existing weird-but-deterministic behaviour, not "fix" it. If a future xcfun upstream patch corrects the FIXME, Phase 4 inherits the patch via vendor-bump and re-runs the alias-canary tests.
- **TPSS / SCAN / M06 numerical sensitivity:** these families have polynomial enhancements with high-degree terms; FMA emission would silently break parity. The `xtask check-no-mul-add` gate (Phase 2 D-24) covers `crates/xcfun-eval/src/functionals/**/*.rs`; verify it still triggers on the new `mgga/` subtree (no scope-list amendment needed if the glob is `**/*.rs`).
- **Mode::Contracted at orders 5/6:** never exercised in C++ for our 10k grid (PartialDerivatives capped at order 3). The Wave-5 cross-mode comparison at orders 0..=4 is the structural smoke test; orders 5/6 require a dedicated Contracted-mode harness extension on the C++ side. **Mitigation:** extend `validation/src/c_driver.rs` to call `xcfun_eval` with `XC_CONTRACTED` mode at order 5/6 on a 1000-point subset; cross-check.
- **DensVarsDev JP-field absence:** verify Plan 02-03's field provisioning at densvars.rs source. If `d.jpaa` / `d.jpbb` aren't there, D-03-B adds them (additive change to xcfun-eval, no other crate touched).
- **Alias depth/cycles:** the registry could theoretically ship an alias-of-alias entry. The recommended depth-guard counter (max 3 in D-04) catches infinite recursion; the registry-time invariant (D-04 Claude's Discretion) catches cycles at `xtask regen-registry --check` time, which is stronger.

</code_context>

<specifics>
## Specific Ideas

- **Kernel-name prefix:** `xcfun_eval_mgga_<fn>_kernel` (e.g., `xcfun_eval_mgga_tpssx_kernel`). Phase 3 used `xcfun_eval_<fn>_kernel` (without the `gga` segment per D-08); planner picks consistency-vs-clarity tradeoff. Same applies to `mgga`.
- **File layout (mirror GGA flat pattern):** `crates/xcfun-eval/src/functionals/mgga/<fn>.rs` — flat, one file per functional, mirroring Phase 3's `gga/<fn>.rs`. OR `mgga/<family>/<fn>.rs` if the family-specific helper imports get heavy. Planner picks.
- **TPSS α / z constants:** `tpssx.cpp` / `tpssc.cpp` use `const double TPSS_alpha_c = ...` constants. Precompute in `shared/constants.rs` (or `mgga/shared/tpss_constants.rs`); avoid kernel-entry recomputation.
- **SCAN family enhancement closure:** SCAN_like_eps.hpp builds a single `eps_x` enhancement function called by all 5 correlation variants. The Rust port should have ONE `#[cube] fn scan_eps<F>(d: &DensVars<F>, ...)` and let SCANC/RSCANC/RPPSCANC/R2SCANC/R4SCANC compose it with their per-variant prefactor.
- **M06 family parameter tables:** like Phase 3's B97 family, M0x family has multiple parameterisations (M06, M06-2X, M06-L, M06-HF). Consolidate into `shared/m0x_params.rs` with `const M06_X_PARAMS: [f64; N] = [...];` per functional; kernel body reads via `#[comptime]` index.
- **BR scalar Newton initial guess:** preserve C++ branch structure verbatim (`brx.cpp:30-40`):
  - `z < -1e4`: `x0 = -2/z`
  - `-1e4 ≤ z < -2`: `x0 = (sqrt(9*z² + 6*z + 49) + 3*z + 1)/4`
  - `-2 ≤ z < 1`: `x0 = 2 * (z * exp(-4/3) + 1)`
  - `z ≥ 1`: `x0 = (3/2) * log(z) + 3.75 / (1.5 + log(z))`
  Branch boundaries (z = -1e4, -2, 1) are sharp; any rounding-noise on the branch-condition leg flips the guess. Use `<` exactly (not `<=`) to match C++.
- **Alias additive vs overwrite invariant test:** add Wave-4 unit test verifying:
  - `set("b3lyp", 1.0); set("slaterx", 0.5); get("slaterx") == 0.80 + 0.5 == 1.30` (additive on functionals)
  - `set("b3lyp", 1.0); get("exx") == 0.20` (parameter overwrite via alias)
  - `set("b3lyp", 0.5); get("exx") == 0.10` (parameter overwrite, NOT additive)
  - `set("camcompx", 0.37); get("beckecamx") == -0.37` (negative-weight propagation; verify exact weight in `aliases.cpp`)
- **Mode::Contracted output indexing:** `output[i] = out.get(i)` for `i ∈ 0..(1 << order)`. The bit-flag indexing of CTaylor (CNST=0, VAR0=1, VAR0|VAR1=3, etc.) means `out.get(i)` for i=0..(1<<order)-1 reads the full coefficient array. Preserve verbatim — don't reshape into a "natural-derivative-order" matrix.
- **Order cap enforcement:** `eval_setup` for Mode::Contracted with order > 6 returns `XcError::InvalidOrder`. Phase 3 left PartialDerivatives at orders 0..=4 (with C++ runtime support only through 3); Phase 4 leaves that unchanged and adds Contracted orders 0..=6.
- **`ParameterId` enum naming:** keep the C-prefix dropped (Rust idiom): `ParameterId::RangesepMu`, `ParameterId::Exx`, `ParameterId::CamAlpha`, `ParameterId::CamBeta` with `#[repr(u32)]` discriminants 78..=81. Conversion helpers `ParameterId::name(&self) -> &'static str` return `"XC_RANGESEP_MU"` etc. for the C-ABI lookup.
- **Settings array layout:** `settings: [f64; 82]` matches C++ `XC_NR_PARAMETERS_AND_FUNCTIONALS = 82`. At `Functional::new`: zero-fill 0..78, parameter defaults at 78..82. C-ABI compatibility (Phase 5) requires exactly this layout.

</specifics>

<deferred>
## Deferred Ideas

- **LB94** — D-13. Phase 5 owns; Phase 4 confirmed not alias-feasible.
- **`Mode::Potential` for metaGGAs** — out of scope (algorithmic-identity rule; C++ rejects; not in any milestone).
- **`Mode::PartialDerivatives` orders 5..=6** — out of scope (not in C++ reference; orders 5/6 are Contracted-only).
- **GPU backends (CUDA/Wgpu) for metaGGA** — Phase 6.
- **Tier-3 parity (CPU vs CUDA at 1e-13, CPU vs Wgpu at 1e-9)** — Phase 6.
- **Criterion benchmarks for metaGGA + Mode::Contracted (PERF-01)** — Phase 6.
- **Phase 6 libm-hybrid resolution for the 13 Phase-3 D-19 forwards** — D-10 inherits unchanged.
- **Alias-feasibility re-check for LB94** — D-13 confirmed not feasible. Phase 5 owns; do NOT re-check in Phase 4.
- **Full RS-01..10 facade surface** — Phase 5; Phase 4 only ships the cross-section needed for aliases + Mode::Contracted.
- **C ABI cbindgen run, `xcfun.h` headers-match test, c_abi.c golden test (CAPI-01..07)** — Phase 5.
- **Python bindings (PY-01..06)** — Phase 7.
- **Stream-overlapped async GPU path (PERF-03)** — v2 backlog per PROJECT.md "Out of Scope".
- **Phase-3 HUMAN-UAT items (order-3 capstone re-run, BECKESRX D-18 forensics, full 36-GGA Mode::Potential sweep)** — owned by `/gsd-verify-work 3`, NOT Phase 4. Listed in `.planning/phases/03-gga-tier-mode-potential/03-HUMAN-UAT.md`.
- **`ctaylor_div` primitive** — Phase 1 deferred; not surfaced in Phase 4. If a metaGGA body's port reveals a need for true `a/b` (vs `a * (1/b)`), escalate.

### Reviewed Todos (not folded)

None reviewed at this session — STATE.md `Active TODOs` section lists Phase-3 follow-ups that belong in `/gsd-verify-work 3`, not Phase 4 scope.

</deferred>

---

*Phase: 04-metagga-tier-mode-contracted-aliases*
*Context gathered: 2026-04-25 (discuss `--auto`, 11 gray areas auto-resolved with recommended defaults logged inline as D-01..D-14)*
