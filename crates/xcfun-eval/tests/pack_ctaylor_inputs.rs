//! W9 — pack_ctaylor_inputs_order3 / order4 unit tests (Plan 03-06 Task 1).
//!
//! Verifies VAR0/VAR1/VAR2/VAR3 bit-flag seeding for the CTaylor<F, 3> and
//! CTaylor<F, 4> input layouts used by orders 3 and 4 of `Mode::PartialDerivatives`.
//!
//! Layout per `xcfun-master/src/XCFunctional.cpp:562-612`:
//!   - Each slot l ∈ 0..inlen occupies `1 << N` consecutive f64s.
//!   - `coefficient[CNST=0]` = `input[l]`.
//!   - `coefficient[VAR0=1]` = 1.0 iff `l == i`.
//!   - `coefficient[VAR1=2]` = 1.0 iff `l == j`.
//!   - `coefficient[VAR2=4]` = 1.0 iff `l == k`.
//!   - `coefficient[VAR3=8]` = 1.0 iff `l == m` (order 4 only).
//!   - All cross-terms (VAR0|VAR1=3, VAR0|VAR2=5, etc.) start at 0.0.
//!
//! Inlen=2 is exhaustive for order 3 (C(4,3)=4 ordered triples i≤j≤k).

#![cfg(feature = "testing")]

use xcfun_eval::functional::{pack_ctaylor_inputs_order3, pack_ctaylor_inputs_order4};

/// W9 — verify VAR0=1/VAR1=2/VAR2=4 bit-flag seeding for order 3.
/// Uses inlen=2 so the test is exhaustive (C(4,3) = 4 index triples).
#[test]
fn pack_ctaylor_inputs_order3_places_vars() {
    let input = [0.7_f64, 0.3_f64];
    let inlen = 2;
    let size_n3 = 8;

    // Case A: (i=0, j=0, k=0) — all seeds on slot 0.
    let flat = pack_ctaylor_inputs_order3(&input, inlen, 0, 0, 0);
    assert_eq!(flat.len(), inlen * size_n3);
    assert_eq!(flat[0 * size_n3 + 0], 0.7); // CNST on slot 0
    assert_eq!(flat[1 * size_n3 + 0], 0.3); // CNST on slot 1
    // Slot 0: VAR0, VAR1, VAR2 all = 1.0.
    assert_eq!(flat[0 * size_n3 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[0 * size_n3 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[0 * size_n3 + 4 /* VAR2 */], 1.0);
    // All cross terms on slot 0 remain 0.0.
    assert_eq!(flat[0 * size_n3 + 3], 0.0);
    assert_eq!(flat[0 * size_n3 + 5], 0.0);
    assert_eq!(flat[0 * size_n3 + 6], 0.0);
    assert_eq!(flat[0 * size_n3 + 7], 0.0);
    // Slot 1: no seeds.
    for off in 1..size_n3 {
        assert_eq!(flat[1 * size_n3 + off], 0.0);
    }

    // Case B: (i=0, j=0, k=1) — VAR0 + VAR1 on slot 0, VAR2 on slot 1.
    let flat = pack_ctaylor_inputs_order3(&input, inlen, 0, 0, 1);
    assert_eq!(flat[0 * size_n3 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[0 * size_n3 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[0 * size_n3 + 4 /* VAR2 */], 0.0); // no VAR2 on slot 0
    assert_eq!(flat[1 * size_n3 + 4 /* VAR2 */], 1.0);
    assert_eq!(flat[1 * size_n3 + 1 /* VAR0 */], 0.0);
    assert_eq!(flat[1 * size_n3 + 2 /* VAR1 */], 0.0);

    // Case C: (i=0, j=1, k=1) — VAR0 on slot 0, VAR1 + VAR2 on slot 1.
    let flat = pack_ctaylor_inputs_order3(&input, inlen, 0, 1, 1);
    assert_eq!(flat[0 * size_n3 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[1 * size_n3 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[1 * size_n3 + 4 /* VAR2 */], 1.0);

    // Case D: (i=1, j=1, k=1) — all on slot 1.
    let flat = pack_ctaylor_inputs_order3(&input, inlen, 1, 1, 1);
    assert_eq!(flat[1 * size_n3 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[1 * size_n3 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[1 * size_n3 + 4 /* VAR2 */], 1.0);
    assert_eq!(flat[0 * size_n3 + 1], 0.0); // slot 0 untouched
}

/// W9 — verify VAR3=8 bit-flag for order 4.
#[test]
fn pack_ctaylor_inputs_order4_places_var3() {
    let input = [0.7_f64, 0.3_f64, 0.5_f64];
    let inlen = 3;
    let size_n4 = 16;

    let flat = pack_ctaylor_inputs_order4(&input, inlen, 0, 1, 2, 2);
    assert_eq!(flat.len(), inlen * size_n4);
    assert_eq!(flat[0 * size_n4 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[1 * size_n4 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[2 * size_n4 + 4 /* VAR2 */], 1.0);
    assert_eq!(flat[2 * size_n4 + 8 /* VAR3 */], 1.0);
    // Read-out index 15 (VAR0|VAR1|VAR2|VAR3) starts at 0.0.
    assert_eq!(flat[0 * size_n4 + 15], 0.0);
    assert_eq!(flat[1 * size_n4 + 15], 0.0);
    assert_eq!(flat[2 * size_n4 + 15], 0.0);

    // Verify CNST entries placed correctly.
    assert_eq!(flat[0 * size_n4 + 0], 0.7);
    assert_eq!(flat[1 * size_n4 + 0], 0.3);
    assert_eq!(flat[2 * size_n4 + 0], 0.5);
}

/// Verify all-on-same-slot: (i=j=k=m=0) — VAR0|VAR1|VAR2|VAR3 all on slot 0.
#[test]
fn pack_ctaylor_inputs_order4_all_same_slot() {
    let input = [1.0_f64, 2.0_f64];
    let inlen = 2;
    let size_n4 = 16;

    let flat = pack_ctaylor_inputs_order4(&input, inlen, 0, 0, 0, 0);
    assert_eq!(flat.len(), inlen * size_n4);
    assert_eq!(flat[0 * size_n4 + 0], 1.0); // CNST
    assert_eq!(flat[0 * size_n4 + 1 /* VAR0 */], 1.0);
    assert_eq!(flat[0 * size_n4 + 2 /* VAR1 */], 1.0);
    assert_eq!(flat[0 * size_n4 + 4 /* VAR2 */], 1.0);
    assert_eq!(flat[0 * size_n4 + 8 /* VAR3 */], 1.0);
    // Cross-terms at index 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15 all zero.
    for idx in [3_usize, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15] {
        assert_eq!(flat[0 * size_n4 + idx], 0.0, "cross-term at idx {} must be 0", idx);
    }
    // Slot 1 untouched.
    assert_eq!(flat[1 * size_n4 + 0], 2.0); // CNST
    for off in 1..size_n4 {
        assert_eq!(flat[1 * size_n4 + off], 0.0);
    }
}
