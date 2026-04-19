//! Test-only helpers. Gated behind `feature = "testing"` (D-22).
//! Consumed by integration tests in `tests/` and by property tests.

pub mod cpu_client;
pub mod raw_eval_scalar;

pub use cpu_client::{CpuClient, cpu_client};
pub use raw_eval_scalar::raw_eval_scalar;
