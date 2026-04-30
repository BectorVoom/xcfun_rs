/* Phase 5 D-14 — drop-in C-side golden test (CAPI-07).
 *
 * Compiled by crates/xcfun-capi/tests/c_abi.rs against
 * libxcfun_capi.a + crates/xcfun-capi/include/xcfun.h (the cbindgen
 * output, NOT the upstream xcfun-master/api/xcfun.h).
 *
 * 10 reference-driven fixtures spanning the public surface:
 *    1: LDA / XC_A_B / Partial / 0
 *    2: PBE / XC_A_B_GAA_GAB_GBB / Partial / 1
 *    3: BECKEX / XC_A_B_GAA_GAB_GBB / Partial / 2
 *    4: bp86 alias (additive: beckex + p86c) / XC_A_B_GAA_GAB_GBB / Partial / 1
 *       — substituted from D-14 row 4 (B3LYP) per Plan 05-04 Rule-3 deviation:
 *       B3LYP mixes LDA+GGA components, which cannot dispatch at any single
 *       Vars in the current launch table (Phase 6 work). bp86 = beckex + p86c
 *       is a pure-GGA additive 2-term alias preserving the D-14 intent.
 *    5: PBE0 alias / XC_A_B_GAA_GAB_GBB / Partial / 1
 *    6: M06 alias (= m06c + m06x; metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0
 *    7: M06X / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Contracted / 3
 *    8: SCANX (metaGGA) / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0
 *       — authorized SCANX→TPSSX fallback per CONTEXT D-14 if Tier-1 self-tests fail
 *         (no fallback triggered during this plan execution; SCANX evaluates cleanly)
 *    9: beckecamx (range-separated GGA) / XC_A_B_GAA_GAB_GBB / Partial / 0
 *       — substituted from D-14 row 9 (CAMB3LYP) per Plan 05-04 Rule-3 deviation:
 *       CAMB3LYP alias mixes LDA+GGA components. beckecamx is the underlying
 *       range-separated GGA kernel, preserving the range-separation surface
 *       coverage intent of D-14 row 9.
 *   10: LB94 (Mode::Potential) — runtime substitute LDA per D-16
 *       (xcfun-master/src/functionals/lb94.cpp:15 is `#if 0`'d in upstream)
 *
 * Each fixture's `density_<n>[]` and `expected_<n>[]` arrays are literal
 * copies of the corresponding entries in tests/fixtures/expected.json
 * (generated once by `cargo run -p xcfun-capi --example gen_expected`).
 *
 * Per-element check: |actual - expected| / max(|expected|, 1) <= 1e-12.
 *
 * cc invocation in tests/c_abi.rs uses `-fno-fast-math -ffp-contract=off`
 * per CLAUDE.md ACC-05/06; NEVER pass `-ffast-math`.
 */

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include "xcfun.h"

/* Per-element relative-error comparison. Returns the count of failing
 * slots; emits a stderr diagnostic for each failure with the slot index,
 * actual, expected, and computed relative error. */
static int compare(const char *tag, const double *got, const double *want, int n) {
    int err = 0;
    for (int i = 0; i < n; i++) {
        double w = want[i];
        double rel = (fabs(w) > 1e-300)
                         ? fabs((got[i] - w) / w)
                         : fabs(got[i] - w);
        if (rel > 1e-12) {
            fprintf(stderr,
                    "FAIL %s: out[%d] = %.16e expected %.16e rel %.2e\n",
                    tag, i, got[i], w, rel);
            err++;
        }
    }
    return err;
}

/* ------- Fixture 1 — LDA / XC_A_B / Partial / 0 -------------------- */
static int run_fixture_1_lda(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "lda", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B, XC_PARTIAL_DERIVATIVES, 0) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_1[2]  = { 0.5, 0.5 };
    static const double expected_1[1] = { -0.8101513786888133 };
    double out[1] = { 0.0 };
    xcfun_eval(fun, density_1, out);
    int e = compare("fixture_1_lda", out, expected_1, 1);
    xcfun_delete(fun);
    return e == 0 ? 0 : 100 + e;
}

/* ------- Fixture 2 — PBE / XC_A_B_GAA_GAB_GBB / Partial / 1 -------- */
static int run_fixture_2_pbe(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "pbe", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 1) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_2[5] = { 0.5, 0.5, 0.01, 0.01, 0.01 };
    static const double expected_2[6] = {
        -0.8097592379422408,
        -1.0642004388460637,
        -1.0642004388460637,
        -0.0042530673896230735,
        0.008423868637500153,
        -0.0042530673896230735
    };
    double out[6] = { 0.0, 0.0, 0.0, 0.0, 0.0, 0.0 };
    xcfun_eval(fun, density_2, out);
    int e = compare("fixture_2_pbe", out, expected_2, 6);
    xcfun_delete(fun);
    return e == 0 ? 0 : 200 + e;
}

/* ------- Fixture 3 — BECKEX / XC_A_B_GAA_GAB_GBB / Partial / 2 ----- */
static int run_fixture_3_beckex(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "beckex", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 2) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_3[5] = { 0.5, 0.5, 0.01, 0.01, 0.01 };
    static const double expected_3[21] = {
        -0.7387700984459259,
        -0.984464127793898,
        -0.984464127793898,
        -0.010550065012560079,
        0.0,
        -0.010550065012560079,
        -0.6578005720887483,
        0.0,
        0.027959129240307794,
        0.0,
        0.0,
        -0.6578005720887483,
        0.0,
        0.0,
        0.027959129240307794,
        0.0032695773722327035,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0032695773722327035
    };
    double out[21] = { 0.0 };
    xcfun_eval(fun, density_3, out);
    int e = compare("fixture_3_beckex", out, expected_3, 21);
    xcfun_delete(fun);
    return e == 0 ? 0 : 300 + e;
}

/* ------- Fixture 4 — bp86 alias / XC_A_B_GAA_GAB_GBB / Partial / 1 - */
static int run_fixture_4_bp86(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "bp86", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 1) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_4[5] = { 0.5, 0.5, 0.01, 0.01, 0.01 };
    static const double expected_4[6] = {
        -0.80924471125416,
        -1.0634951805129709,
        -1.0634951805129709,
        -0.006548587014420394,
        0.008002955996279369,
        -0.006548587014420394
    };
    double out[6] = { 0.0 };
    xcfun_eval(fun, density_4, out);
    int e = compare("fixture_4_bp86", out, expected_4, 6);
    xcfun_delete(fun);
    return e == 0 ? 0 : 400 + e;
}

/* ------- Fixture 5 — pbe0 alias / XC_A_B_GAA_GAB_GBB / Partial / 1 - */
static int run_fixture_5_pbe0(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "pbe0", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 1) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_5[5] = { 0.5, 0.5, 0.01, 0.01, 0.01 };
    static const double expected_5[6] = {
        -0.6250772092639272,
        -0.8180706006310906,
        -0.8180706006310906,
        -0.0021368169625297856,
        0.008423868637500153,
        -0.0021368169625297856
    };
    double out[6] = { 0.0 };
    xcfun_eval(fun, density_5, out);
    int e = compare("fixture_5_pbe0", out, expected_5, 6);
    xcfun_delete(fun);
    return e == 0 ? 0 : 500 + e;
}

/* ------- Fixture 6 — m06 alias / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0 - */
static int run_fixture_6_m06(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "m06", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB_TAUA_TAUB, XC_PARTIAL_DERIVATIVES, 0) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_6[7] = { 0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05 };
    static const double expected_6[1] = { -0.4708767063991616 };
    double out[1] = { 0.0 };
    xcfun_eval(fun, density_6, out);
    int e = compare("fixture_6_m06", out, expected_6, 1);
    xcfun_delete(fun);
    return e == 0 ? 0 : 600 + e;
}

/* ------- Fixture 7 — M06X / Contracted / 3 (7 vars * 8 = 56 doubles) - */
static int run_fixture_7_m06x_contracted(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "m06x", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB_TAUA_TAUB, XC_CONTRACTED, 3) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    /* Var-major flattening per D-06-A: inlen * (1 << order) = 7 * 8 = 56. */
    static const double density_7[56] = {
        /* var 0 (a) */     0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
        /* var 1 (b) */     0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
        /* var 2 (gaa) */   0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01,
        /* var 3 (gab) */   0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005, 0.005,
        /* var 4 (gbb) */   0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01,
        /* var 5 (taua) */  0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05,
        /* var 6 (taub) */  0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05, 0.05
    };
    static const double expected_7[8] = {
        -0.42445826744704507,
        -0.8610345404789357,
        -0.8610345404789357,
        -1.7485513550297869,
        -0.8610345404789357,
        -1.7485513550297869,
        -1.7485513550297869,
        -3.0758969706600827
    };
    double out[8] = { 0.0 };
    xcfun_eval(fun, density_7, out);
    int e = compare("fixture_7_m06x_contracted", out, expected_7, 8);
    xcfun_delete(fun);
    return e == 0 ? 0 : 700 + e;
}

/* ------- Fixture 8 — scanx / XC_A_B_GAA_GAB_GBB_TAUA_TAUB / Partial / 0 - */
static int run_fixture_8_scanx(void) {
    xcfun_t *fun = xcfun_new();
    /* SCANX → TPSSX fallback authorized by CONTEXT D-14 if SCANX fails
     * Tier-1 self-tests at this density point. Not triggered during the
     * Plan 05-04 execution — SCANX evaluated cleanly. */
    if (xcfun_set(fun, "scanx", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB_TAUA_TAUB, XC_PARTIAL_DERIVATIVES, 0) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_8[7] = { 0.5, 0.5, 0.01, 0.005, 0.01, 0.05, 0.05 };
    static const double expected_8[1] = { -0.8642541134979742 };
    double out[1] = { 0.0 };
    xcfun_eval(fun, density_8, out);
    int e = compare("fixture_8_scanx", out, expected_8, 1);
    xcfun_delete(fun);
    return e == 0 ? 0 : 800 + e;
}

/* ------- Fixture 9 — beckecamx (range-sep GGA) / XC_A_B_GAA_GAB_GBB / Partial / 0 - */
static int run_fixture_9_beckecamx(void) {
    xcfun_t *fun = xcfun_new();
    if (xcfun_set(fun, "beckecamx", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B_GAA_GAB_GBB, XC_PARTIAL_DERIVATIVES, 0) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_9[5] = { 0.5, 0.5, 0.01, 0.01, 0.01 };
    static const double expected_9[1] = { -0.5058506266720899 };
    double out[1] = { 0.0 };
    xcfun_eval(fun, density_9, out);
    int e = compare("fixture_9_beckecamx", out, expected_9, 1);
    xcfun_delete(fun);
    return e == 0 ? 0 : 900 + e;
}

/* ------- Fixture 10 — LB94→LDA(Potential) substitute / XC_A_B / Potential / 0 - */
static int run_fixture_10_lb94_substitute(void) {
    xcfun_t *fun = xcfun_new();
    /* LB94 has no working upstream body (xcfun-master/src/functionals/
     * lb94.cpp:15 `#if 0`'d). Per D-16, the LB94 descriptor is registered
     * in xcfun-core but its eval path returns XcError::Runtime. This
     * fixture evaluates LDA on Mode::Potential as the documented substitute,
     * preserving the Mode::Potential coverage goal of D-14 row 10. */
    if (xcfun_set(fun, "lda", 1.0) != 0) { xcfun_delete(fun); return 1; }
    if (xcfun_eval_setup(fun, XC_A_B, XC_POTENTIAL, 0) != 0) {
        xcfun_delete(fun);
        return 2;
    }
    static const double density_10[2] = { 0.5, 0.5 };
    static const double expected_10[3] = {
        -0.8101513786888133,
        -1.0646834050186824,
        -1.0646834050186824
    };
    double out[3] = { 0.0, 0.0, 0.0 };
    xcfun_eval(fun, density_10, out);
    int e = compare("fixture_10_lb94_substitute", out, expected_10, 3);
    xcfun_delete(fun);
    return e == 0 ? 0 : 1000 + e;
}

int main(void) {
    int rc;
    if ((rc = run_fixture_1_lda())                != 0) return rc;
    if ((rc = run_fixture_2_pbe())                != 0) return rc;
    if ((rc = run_fixture_3_beckex())             != 0) return rc;
    if ((rc = run_fixture_4_bp86())               != 0) return rc;
    if ((rc = run_fixture_5_pbe0())               != 0) return rc;
    if ((rc = run_fixture_6_m06())                != 0) return rc;
    if ((rc = run_fixture_7_m06x_contracted())    != 0) return rc;
    if ((rc = run_fixture_8_scanx())              != 0) return rc;
    if ((rc = run_fixture_9_beckecamx())          != 0) return rc;
    if ((rc = run_fixture_10_lb94_substitute())   != 0) return rc;
    printf("ALL FIXTURES PASS\n");
    return 0;
}
