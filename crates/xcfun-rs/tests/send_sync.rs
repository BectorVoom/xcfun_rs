//! RS-10 — `Functional` MUST be `Send + Sync`. Compile-time gate.
use static_assertions::assert_impl_all;
use xcfun_rs::Functional;

assert_impl_all!(Functional: Send, Sync);
