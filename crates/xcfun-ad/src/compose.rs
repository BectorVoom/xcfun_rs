//! Recursive multiplication and composition algorithms for CTaylor.
//!
//! These are direct ports of the C++ `ctaylor_rec` algorithms from xcfun.
//! The recursive structure operates on slices of length 2^N, splitting
//! into halves at each recursion level.

/// Accumulating recursive multiply: `dst += a * b`
///
/// Base case (n==1): `dst[0] += a[0] * b[0]`
/// Recursive case: splits into three sub-multiplications matching
/// the C++ `ctaylor_rec<T, Nvar>::mul`.
pub fn mul_recursive(dst: &mut [f64], a: &[f64], b: &[f64]) {
    let n = dst.len();
    debug_assert_eq!(a.len(), n);
    debug_assert_eq!(b.len(), n);

    if n == 1 {
        dst[0] += a[0] * b[0];
        return;
    }

    let half = n / 2;
    // dst_lo += a_lo * b_lo
    mul_recursive(&mut dst[..half], &a[..half], &b[..half]);
    // dst_hi += a_hi * b_lo
    mul_recursive(&mut dst[half..], &a[half..], &b[..half]);
    // dst_hi += a_lo * b_hi
    mul_recursive(&mut dst[half..], &a[..half], &b[half..]);
}

/// Non-accumulating recursive multiply: `dst = a * b`
///
/// Base case (n==1): `dst[0] = a[0] * b[0]`
/// Recursive case: first sub-call uses mul_set (non-accumulating),
/// second uses mul_set, third uses mul (accumulating) -- matching
/// the C++ `ctaylor_rec<T, Nvar>::mul_set`.
pub fn mul_set_recursive(dst: &mut [f64], a: &[f64], b: &[f64]) {
    let n = dst.len();
    debug_assert_eq!(a.len(), n);
    debug_assert_eq!(b.len(), n);

    if n == 1 {
        dst[0] = a[0] * b[0];
        return;
    }

    let half = n / 2;
    // dst_lo = a_lo * b_lo (set, not accumulate)
    mul_set_recursive(&mut dst[..half], &a[..half], &b[..half]);
    // dst_hi = a_hi * b_lo (set, not accumulate)
    mul_set_recursive(&mut dst[half..], &a[half..], &b[..half]);
    // dst_hi += a_lo * b_hi (accumulate)
    mul_recursive(&mut dst[half..], &a[..half], &b[half..]);
}

/// In-place multiply by `(y - y[0])`: `dst = dst * (y - y[0])`
///
/// CRITICAL ordering to match C++:
/// 1. Process high half recursively (reads low half before modification)
/// 2. Add cross terms: `dst_hi += dst_lo * y_hi`
/// 3. Process low half recursively
///
/// Base case (n==1): `dst[0] = 0.0` (multiplying by zero since y-y[0] has no constant)
///
/// For step 2, we need to read dst[..half] while writing dst[half..].
/// We copy dst[..half] to a temporary to satisfy the borrow checker.
pub fn multo_skipconst(dst: &mut [f64], y: &[f64]) {
    let n = dst.len();
    debug_assert_eq!(y.len(), n);

    if n == 1 {
        dst[0] = 0.0;
        return;
    }

    let half = n / 2;

    // Step 1: Process high half recursively
    multo_skipconst(&mut dst[half..], &y[..half]);

    // Step 2: Cross terms -- need dst[..half] (read) and dst[half..] (write)
    // Copy low half to temporary to satisfy borrow checker
    let lo_copy: Vec<f64> = dst[..half].to_vec();
    mul_recursive(&mut dst[half..], &lo_copy, &y[half..]);

    // Step 3: Process low half recursively
    multo_skipconst(&mut dst[..half], &y[..half]);
}

/// Horner composition: evaluate `sum_i coeffs[i] * (x - x[0])^i` into `result`.
///
/// This matches C++ `ctaylor_rec<T, Nvar>::compose`:
/// ```text
/// result[0] = coeffs[last];
/// result[1..] = 0;
/// for i = (len-2) downto 0:
///     multo_skipconst(result, x);
///     result[0] += coeffs[i];
/// ```
///
/// The `coeffs` slice has length `N + 1` where N is the number of variables.
/// `x` has length `2^N` (the full CTaylor coefficient array).
/// `result` has length `2^N`.
pub fn compose(result: &mut [f64], x: &[f64], coeffs: &[f64]) {
    let n = result.len();
    debug_assert_eq!(x.len(), n);
    debug_assert!(!coeffs.is_empty());

    let ncoeffs = coeffs.len();

    // result[0] = coeffs[last]
    result[0] = coeffs[ncoeffs - 1];
    // result[1..] = 0
    for r in result.iter_mut().skip(1) {
        *r = 0.0;
    }

    // Horner iteration: from second-to-last down to 0
    for i in (0..ncoeffs - 1).rev() {
        multo_skipconst(result, x);
        result[0] += coeffs[i];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_set_recursive_base_case() {
        // N=0: [a0] * [b0] = [a0*b0]
        let a = [3.0];
        let b = [5.0];
        let mut dst = [0.0];
        mul_set_recursive(&mut dst, &a, &b);
        assert_eq!(dst[0], 15.0);
    }

    #[test]
    fn test_mul_recursive_accumulates() {
        // mul_recursive should ADD to dst
        let a = [2.0];
        let b = [3.0];
        let mut dst = [10.0];
        mul_recursive(&mut dst, &a, &b);
        assert_eq!(dst[0], 16.0); // 10 + 2*3
    }

    #[test]
    fn test_mul_set_recursive_n1() {
        // N=1: 2 elements
        // (a0 + a1*x) * (b0 + b1*x) = a0*b0 + (a0*b1 + a1*b0)*x
        // (no x^2 term -- multilinear drops hi*hi)
        let a = [2.0, 3.0];
        let b = [5.0, 7.0];
        let mut dst = [0.0, 0.0];
        mul_set_recursive(&mut dst, &a, &b);
        assert_eq!(dst[0], 10.0); // 2*5
        assert_eq!(dst[1], 2.0 * 7.0 + 3.0 * 5.0); // 14 + 15 = 29
    }

    #[test]
    fn test_multo_skipconst_base_case() {
        let mut dst = [42.0];
        let y = [999.0];
        multo_skipconst(&mut dst, &y);
        assert_eq!(dst[0], 0.0);
    }

    #[test]
    fn test_multo_skipconst_zeroes_constant() {
        // For N=1: dst = dst * (y - y[0])
        // dst = [d0, d1], y = [y0, y1]
        // y - y[0] has no constant, so result constant = 0
        // result[1] = d0 * y[1]
        let mut dst = [3.0, 5.0];
        let y = [2.0, 7.0];
        multo_skipconst(&mut dst, &y);
        assert_eq!(dst[0], 0.0);
        assert_eq!(dst[1], 3.0 * 7.0); // 21
    }

    #[test]
    fn test_compose_constant() {
        // compose with coeffs [c0] should give result = c0
        let x = [1.0, 0.0];
        let mut result = [0.0, 0.0];
        let coeffs = [42.0];
        compose(&mut result, &x, &coeffs);
        assert_eq!(result[0], 42.0);
        assert_eq!(result[1], 0.0);
    }

    #[test]
    fn test_compose_linear() {
        // compose with coeffs [c0, c1] and variable x
        // Result should be c0 + c1*(x - x[0])
        // For x = variable(2.0, 0) with N=1: x.c = [2.0, 1.0]
        // x - x[0] = [0.0, 1.0]
        // c0 + c1*(x - x[0]) = [c0, c1]
        let x = [2.0, 1.0];
        let mut result = [0.0, 0.0];
        let coeffs = [3.0, 5.0];
        compose(&mut result, &x, &coeffs);
        assert_eq!(result[0], 3.0);
        assert_eq!(result[1], 5.0);
    }

    #[test]
    fn test_compose_quadratic_horner() {
        // compose with coeffs [c0, c1, c2] for N=2 variables
        // Horner: start with c2, then multo_skipconst and add c1, then multo_skipconst and add c0
        // For variable x with N=2, var=0: x.c = [x0, 1, 0, 0]
        // (x - x0) = [0, 1, 0, 0]
        // After first multo_skipconst on [c2, 0, 0, 0] * (x - x0): [0, c2, 0, 0]
        // Add c1: [c1, c2, 0, 0]
        // After second multo_skipconst on [c1, c2, 0, 0] * (x - x0): [0, c1, 0, c2]
        // Add c0: [c0, c1, 0, c2]
        let x0 = 2.0;
        let x = [x0, 1.0, 0.0, 0.0]; // variable(2.0, 0) for N=2
        let mut result = [0.0, 0.0, 0.0, 0.0];
        let coeffs = [10.0, 5.0, 3.0]; // 10 + 5*(x-x0) + 3*(x-x0)^2 terms
        compose(&mut result, &x, &coeffs);
        assert_eq!(result[0], 10.0);
        assert_eq!(result[1], 5.0);
        assert_eq!(result[2], 0.0);
        // result[3] = c2 * x[1] * x[2] cross term... but x[2]=0, so 0
        // Actually for a single-variable-like case, the quadratic cross is different
        // Let me trace: compose for N=2:
        // init: result = [3.0, 0, 0, 0]
        // i=1: multo_skipconst(result, x) then result[0] += 5.0
        //   multo_skipconst on [3.0, 0, 0, 0] with x=[2.0, 1.0, 0, 0]:
        //     high half dst[2..4] = [0, 0], y[0..2] = [2.0, 1.0]
        //       multo_skipconst([0,0], [2.0, 1.0]) -> [0, 0*1.0=0] -> [0, 0]
        //     cross: lo_copy = [3.0, 0], y_hi = [0, 0]
        //       mul_recursive(dst[2..4], [3.0, 0], [0, 0]) -> adds nothing
        //     low half dst[0..2] = [3.0, 0], y[0..2] = [2.0, 1.0]
        //       multo_skipconst([3.0, 0], [2.0, 1.0]) -> [0, 3.0*1.0] = [0, 3.0]
        //   result = [0, 3.0, 0, 0], then result[0] += 5.0 => [5.0, 3.0, 0, 0]
        // i=0: multo_skipconst(result, x) then result[0] += 10.0
        //   multo_skipconst on [5.0, 3.0, 0, 0] with x=[2.0, 1.0, 0, 0]:
        //     high half dst[2..4] = [0, 0], y[0..2] = [2.0, 1.0]
        //       multo_skipconst([0,0], [2.0, 1.0]) -> [0, 0]
        //     cross: lo_copy = [5.0, 3.0], y_hi = [0, 0]
        //       mul_recursive(dst[2..4], [5.0, 3.0], [0, 0]) -> adds nothing
        //     low half dst[0..2] = [5.0, 3.0], y[0..2] = [2.0, 1.0]
        //       multo_skipconst([5.0, 3.0], [2.0, 1.0]):
        //         n=2, half=1
        //         high: multo_skipconst([3.0], [2.0]) -> [0]
        //         cross: lo_copy=[5.0], y_hi=[1.0], mul_recursive([0], [5.0], [1.0]) -> [5.0]
        //         low: multo_skipconst([5.0], [2.0]) -> [0]
        //       => [0, 5.0]
        //   result = [0, 5.0, 0, 0], then result[0] += 10.0 => [10.0, 5.0, 0, 0]
        assert_eq!(result[3], 0.0);
    }

    #[test]
    fn test_compose_exp_like_coefficients() {
        // exp(x) around x0=0: coeffs = [1, 1, 0.5] (1 + x + x^2/2)
        // Applied to variable(0.0, 0) with N=1: x.c = [0.0, 1.0]
        // compose: result = c0 + c1*(x-x0) + c2*(x-x0)^2
        // Since x0=0, (x-x0) = x itself basically but in multilinear sense
        // N=1: compose with coeffs of length 2 (N+1=2)
        // init: result = [0.5, 0]  (wait, coeffs has 3 elements but N=1 so coeffs should have N+1=2)
        // Actually coeffs length is independent of N -- it's the number of Taylor terms
        // Let me use N=2 for a proper test
        let x = [0.0, 1.0, 0.0, 0.0]; // variable(0.0, 0) for N=2
        let mut result = [0.0; 4];
        let coeffs = [1.0, 1.0, 0.5]; // exp-like: 1 + x + x^2/2
        compose(&mut result, &x, &coeffs);
        // Constant term: 1.0 (the function value at x=0)
        assert_eq!(result[0], 1.0);
        // Linear term in var0: 1.0 (first derivative of exp at 0)
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn test_multilinear_mul_drops_quadratic() {
        // x * x should have c[VAR0|VAR0] = 0 since multilinear drops hi*hi
        // For N=2, VAR0=1, variable(1.0, 0): c = [1.0, 1.0, 0.0, 0.0]
        // x * x:
        //   mul_set on [1,1,0,0] * [1,1,0,0]
        //   lo = [1,1], hi_a = [0,0], hi_b = [0,0]
        //   dst_lo = mul_set([1,1], [1,1]) = [1, 1+1] = [1, 2]
        //   dst_hi = mul_set([0,0], [1,1]) = [0, 0]
        //   dst_hi += mul([1,1], [0,0]) = [0, 0]
        //   result = [1, 2, 0, 0]
        // But wait for N=1 directly:
        // x = [1.0, 1.0], x*x:
        //   mul_set_recursive: dst[0] = 1*1 = 1, dst[1] = 1*1+1*1 = 2
        //   NO x^2 term because the recursion doesn't create it!
        let a = [1.0, 1.0];
        let b = [1.0, 1.0];
        let mut dst = [0.0, 0.0];
        mul_set_recursive(&mut dst, &a, &b);
        assert_eq!(dst[0], 1.0);
        assert_eq!(dst[1], 2.0); // 2*x term, but no x^2 -- multilinear!
    }

    #[test]
    fn test_mul_n2_cross_terms() {
        // (1 + x) * (1 + y) for N=2
        // a = [1, 1, 0, 0] (1 + x), b = [1, 0, 1, 0] (1 + y)
        // Expected: 1 + x + y + xy = [1, 1, 1, 1]
        let a = [1.0, 1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 1.0, 0.0];
        let mut dst = [0.0; 4];
        mul_set_recursive(&mut dst, &a, &b);
        assert_eq!(dst[0], 1.0); // constant
        assert_eq!(dst[1], 1.0); // x
        assert_eq!(dst[2], 1.0); // y
        assert_eq!(dst[3], 1.0); // xy
    }
}
