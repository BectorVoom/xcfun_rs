// Fixture driver for xcfun-ad. Compiled by xtask/src/bin/regen_ad_fixtures.rs
// (via a direct c++ compiler invocation — see the binary for the build story)
// and emits deterministic records to stdout in a line-oriented text format.
//
// Record format — one per line, semicolon-separated fields:
//   <op>;<n_var>;<input_count>;<i0>,<i1>,...,<i{k-1}>;<coeff_count>;<c0>,<c1>,...,<c{m-1}>
//
// f64 values are written via printf("%.17g", v) which preserves round-trip
// identity on IEEE-754 double (std::stod / Rust's str::parse::<f64> accept
// "%.17g" output verbatim).
//
// Scope — Phase 1 Plan 05:
//   Expand records: 168 = 3 inputs × 7 orders × 8 fns
//                   (inv, exp, log, pow, sqrt, cbrt, gauss, erf)
//   Mul records:    250 = 50 seeds × 5 N values (n_var ∈ {0, 1, 2, 3, 4})
//   Total:          418
// Phase 1 Plan 06 extends to 1668 by adding add/sub/neg/mul_assign/div.

#include <cstdio>
#include <cstdlib>
#include <cstddef>
#include <random>
#include <vector>
#include "ctaylor.hpp"
#include "ctaylor_math.hpp"
#include "tmath.hpp"

static void emit_record(const char * op,
                        int n_var,
                        const std::vector<double> & inputs,
                        const std::vector<double> & coeffs) {
    printf("%s;%d;%zu;", op, n_var, inputs.size());
    for (size_t i = 0; i < inputs.size(); i++) {
        printf("%.17g", inputs[i]);
        if (i + 1 < inputs.size()) printf(",");
    }
    printf(";%zu;", coeffs.size());
    for (size_t i = 0; i < coeffs.size(); i++) {
        printf("%.17g", coeffs[i]);
        if (i + 1 < coeffs.size()) printf(",");
    }
    printf("\n");
}

// ---------------------------------------------------------------------------
//  *_expand record emitters — templated on expansion order N.
// ---------------------------------------------------------------------------

template <int N>
static void emit_inv_expand(double a) {
    double t[N + 1];
    inv_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("inv_expand", N, inputs, coeffs);
}

template <int N>
static void emit_exp_expand(double x0) {
    double t[N + 1];
    exp_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("exp_expand", N, inputs, coeffs);
}

template <int N>
static void emit_log_expand(double x0) {
    double t[N + 1];
    log_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("log_expand", N, inputs, coeffs);
}

template <int N>
static void emit_pow_expand(double x0, double a) {
    double t[N + 1];
    pow_expand<double, N>(t, x0, a);
    std::vector<double> inputs = {x0, a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("pow_expand", N, inputs, coeffs);
}

template <int N>
static void emit_sqrt_expand(double x0) {
    double t[N + 1];
    sqrt_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("sqrt_expand", N, inputs, coeffs);
}

template <int N>
static void emit_cbrt_expand(double x0) {
    double t[N + 1];
    cbrt_expand<double, N>(t, x0);
    std::vector<double> inputs = {x0};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("cbrt_expand", N, inputs, coeffs);
}

template <int N>
static void emit_gauss_expand(double a) {
    double t[N + 1];
    gauss_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("gauss_expand", N, inputs, coeffs);
}

template <int N>
static void emit_erf_expand(double a) {
    double t[N + 1];
    erf_expand<double, N>(t, a);
    std::vector<double> inputs = {a};
    std::vector<double> coeffs(t, t + N + 1);
    emit_record("erf_expand", N, inputs, coeffs);
}

// ---------------------------------------------------------------------------
//  ctaylor_mul record emitters — one per n_var so template instantiation
//  picks the right per-N base case (ctaylor.hpp:86-152 + :41-65 recursion).
// ---------------------------------------------------------------------------

template <int NVAR>
static void emit_mul_record(const double * a, const double * b) {
    const int SIZE = 1 << NVAR;
    ctaylor<double, NVAR> ta, tb;
    for (int i = 0; i < SIZE; i++) ta.c[i] = a[i];
    for (int i = 0; i < SIZE; i++) tb.c[i] = b[i];
    ctaylor<double, NVAR> tc = ta * tb;

    std::vector<double> inputs(2 * SIZE);
    for (int i = 0; i < SIZE; i++) inputs[i] = a[i];
    for (int i = 0; i < SIZE; i++) inputs[SIZE + i] = b[i];
    std::vector<double> coeffs(tc.c, tc.c + SIZE);
    emit_record("mul", NVAR, inputs, coeffs);
}

int main() {
    // -------------------------------------------------------------------
    //  *_expand records: 3 inputs × 7 orders × 8 fns = 168
    // -------------------------------------------------------------------

    const double inv_inputs[]   = {0.1, 1.0, 10.0};
    const double exp_inputs[]   = {-1.0, 0.0, 2.0};
    const double log_inputs[]   = {0.1, 1.0, 10.0};
    const double sqrt_inputs[]  = {0.1, 1.0, 10.0};
    const double cbrt_inputs[]  = {0.1, 1.0, 10.0};
    const double gauss_inputs[] = {-1.0, 0.0, 1.0};
    const double erf_inputs[]   = {-1.0, 0.0, 1.0};
    // pow: 3 (x0, a) pairs. Preserve x0 > 0 precondition.
    struct PowInput { double x0, a; };
    const PowInput pow_inputs[] = {{1.0, 0.5}, {2.0, 1.5}, {10.0, -1.0}};

    #define EMIT_EXPAND(FN_NAME, INPUTS_ARR)              \
        for (auto & x : INPUTS_ARR) {                      \
            emit_ ## FN_NAME ## _expand<0>(x);             \
            emit_ ## FN_NAME ## _expand<1>(x);             \
            emit_ ## FN_NAME ## _expand<2>(x);             \
            emit_ ## FN_NAME ## _expand<3>(x);             \
            emit_ ## FN_NAME ## _expand<4>(x);             \
            emit_ ## FN_NAME ## _expand<5>(x);             \
            emit_ ## FN_NAME ## _expand<6>(x);             \
        }

    EMIT_EXPAND(inv,   inv_inputs);
    EMIT_EXPAND(exp,   exp_inputs);
    EMIT_EXPAND(log,   log_inputs);
    EMIT_EXPAND(sqrt,  sqrt_inputs);
    EMIT_EXPAND(cbrt,  cbrt_inputs);
    EMIT_EXPAND(gauss, gauss_inputs);
    EMIT_EXPAND(erf,   erf_inputs);

    #undef EMIT_EXPAND

    for (auto & p : pow_inputs) {
        emit_pow_expand<0>(p.x0, p.a);
        emit_pow_expand<1>(p.x0, p.a);
        emit_pow_expand<2>(p.x0, p.a);
        emit_pow_expand<3>(p.x0, p.a);
        emit_pow_expand<4>(p.x0, p.a);
        emit_pow_expand<5>(p.x0, p.a);
        emit_pow_expand<6>(p.x0, p.a);
    }

    // -------------------------------------------------------------------
    //  ctaylor_mul records: 50 seeds × 5 N values (n_var ∈ {0..=4}) = 250.
    //  Deterministic mt19937_64 with a fixed seed so re-running the driver
    //  produces byte-identical output.
    // -------------------------------------------------------------------

    std::mt19937_64 rng(0x1234abcdULL);
    std::uniform_real_distribution<double> dist(-10.0, 10.0);

    for (int n_var = 0; n_var <= 4; n_var++) {
        const int SIZE = 1 << n_var;
        for (int seed_i = 0; seed_i < 50; seed_i++) {
            std::vector<double> a(SIZE), b(SIZE);
            for (int i = 0; i < SIZE; i++) a[i] = dist(rng);
            for (int i = 0; i < SIZE; i++) b[i] = dist(rng);

            switch (n_var) {
                case 0: emit_mul_record<0>(a.data(), b.data()); break;
                case 1: emit_mul_record<1>(a.data(), b.data()); break;
                case 2: emit_mul_record<2>(a.data(), b.data()); break;
                case 3: emit_mul_record<3>(a.data(), b.data()); break;
                case 4: emit_mul_record<4>(a.data(), b.data()); break;
            }
        }
    }

    // NOTE: Plan 01-06 adds add/sub/neg/mul_assign/div records (5 more ops ×
    // 50 seeds × 5 N = 1250), bringing total to 1668.

    return 0;
}
