// Phase 7 Plan 07-01 stub — workspace + dep wiring only.
// Full #[pymodule] skeleton lands in Plan 07-02.
//
// No public surface yet. The pyo3 + numpy crates are pulled in by Cargo.toml so
// that downstream plans can import them without re-touching the manifest.
#![allow(dead_code)]

// Smoke: confirm we can name pyo3 and xcfun_rs at the type level. These
// imports prevent "unused dependency" warnings in stub builds.
#[allow(unused_imports)]
use pyo3 as _pyo3;
#[allow(unused_imports)]
use xcfun_rs as _xcfun_rs;
