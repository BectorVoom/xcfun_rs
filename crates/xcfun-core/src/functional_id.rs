//! FunctionalId enum -- stub for Task 1, fully implemented in Task 2.

/// Unique identifier for each exchange-correlation functional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FunctionalId {
    SlaterX = 0,
}
