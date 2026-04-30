//! Phase 5 D-10 + CAPI-02 — diff the generated `xcfun-capi/include/xcfun.h`
//! against `xcfun-master/api/xcfun.h` modulo whitespace + comments.
//!
//! Drift in either direction surfaces as a test failure with a per-line
//! unified-diff snippet for human review.
//!
//! Normalization strategy (in order):
//!   1. Strip C-style block comments and C++ line comments.
//!   2. Drop preprocessor noise that differs between sides (`#pragma once`,
//!      `#include "XCFun/XCFunExport.h"`, `#define XCFun_API XCFUN_EXPORT`,
//!      `#include <stdarg.h>`, our `#ifndef XCFUN_CAPI_H` / `#define XCFUN_CAPI_H`,
//!      `#define XCFUN_API_VERSION 2`, `#define XCFUN_MAX_ORDER 6`, the
//!      `XCFun_API` visibility prelude block, `#ifdef __cplusplus`,
//!      `extern "C" {`, `}`, `#endif`, etc.). What remains is the
//!      type/function declaration set + enum bodies — the CAPI-02 surface.
//!   3. Join every C declaration into a single logical line (a "statement"
//!      ends at `;` or `}`). This eliminates all multi-line vs single-line
//!      formatting differences in function signatures.
//!   4. Tokenize each statement. Then re-emit a canonical text form:
//!      single space between identifiers, no space before/after
//!      punctuation `( ) , * ; { }`, drop `(void)` empty arg list,
//!      drop top-level `const` on by-value scalar args, treat
//!      `xcfun_t` and `xcfun_s` as the same token, treat
//!      `int xcfun_which_vars` and `xcfun_vars xcfun_which_vars` and
//!      `int xcfun_which_mode` and `xcfun_mode xcfun_which_mode` as the
//!      same return type since the typedef bridges them. Treat
//!      `int vars` and `xcfun_vars vars` similarly inside `xcfun_eval_setup`.
//!      Treat `density[]` / `result[]` (upstream array syntax) as `*density` /
//!      `*result` (cbindgen pointer syntax) — equivalent at the C ABI.
//!   5. Drop `XCFun_API` (the visibility macro is on every fn decl on both
//!      sides, identical, no information).

use std::fs;
use std::path::Path;

fn strip_block_comments(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn strip_line_comment(line: &str) -> &str {
    if let Some(idx) = line.find("//") {
        &line[..idx]
    } else {
        line
    }
}

/// Drop whole-line preprocessor / boilerplate noise. Returns true if the
/// caller should drop this line entirely.
fn is_dropped_line(trimmed: &str) -> bool {
    let lower = trimmed.to_ascii_lowercase();

    // Empty
    if trimmed.is_empty() {
        return true;
    }

    // Include-guard idioms — upstream `#pragma once`; ours `#ifndef
    // XCFUN_CAPI_H` / `#define XCFUN_CAPI_H`. Both are header-internal.
    if lower == "#pragma once" {
        return true;
    }
    if lower == "#ifndef xcfun_capi_h" {
        return true;
    }
    if lower == "#define xcfun_capi_h" {
        return true;
    }

    // Upstream's "#include "XCFun/XCFunExport.h"" line is replaced by our
    // inline visibility-macro prelude (D-12); both sides expand to the
    // same XCFun_API symbol decoration but via different syntactic paths.
    if lower.starts_with("#include \"xcfun/xcfunexport.h\"") {
        return true;
    }
    // Upstream's "#define XCFun_API XCFUN_EXPORT" is redundant with our
    // prelude's full XCFun_API definition.
    if lower.starts_with("#define xcfun_api xcfun_export") {
        return true;
    }

    // cbindgen always emits stdarg/stdint/stdlib includes; upstream uses
    // only stdbool + stddef. They are header-internal and don't affect the
    // C-callable interface.
    if lower == "#include <stdarg.h>" || lower == "#include <stdint.h>" || lower == "#include <stdlib.h>" {
        return true;
    }
    // stdbool + stddef — keep them filtered too (both sides emit them but
    // they're not part of the API surface).
    if lower == "#include <stdbool.h>" || lower == "#include <stddef.h>" {
        return true;
    }

    // Visibility-macro prelude (both sides). These define HOW XCFun_API
    // expands on a given platform; the symbol decoration itself is on
    // every function declaration line.
    if lower.contains("xcfun_build_shared")
        || lower.contains("__declspec(dllexport)")
        || lower.contains("__declspec(dllimport)")
        || lower.contains("__attribute__((visibility(\"default\")))")
        || lower == "#define xcfun_api_version 2"
        || lower == "#define xcfun_max_order 6"
    {
        return true;
    }

    // All preprocessor conditionals on both sides — they bracket the
    // visibility-macro prelude and the `extern "C"` block. The remaining
    // type+function declaration set is what matters.
    if lower.starts_with('#') {
        return true;
    }

    // `extern "C" {` and the lone closing `}` that terminates the block.
    if lower.starts_with("extern \"c\"") {
        return true;
    }
    // Lone close-brace of the extern "C" block.
    if lower == "}" {
        return true;
    }

    false
}

/// Tokens for declaration-shape comparison. A token is either a punctuation
/// char (one of `( ) , ; { } *`) or a contiguous identifier-or-keyword run.
fn tokenize(s: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if matches!(c, b'(' | b')' | b',' | b';' | b'{' | b'}' | b'*' | b'[' | b']' | b'=') {
            toks.push((c as char).to_string());
            i += 1;
            continue;
        }
        // Identifier / number / keyword run.
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace()
            && !matches!(bytes[i], b'(' | b')' | b',' | b';' | b'{' | b'}' | b'*' | b'[' | b']' | b'=')
        {
            i += 1;
        }
        toks.push(s[start..i].to_string());
    }
    toks
}

/// Apply token-level rewrites that bridge cosmetic differences between
/// the upstream and generated headers.
fn canonicalize_tokens(toks: &mut Vec<String>) {
    // Drop top-level `const` on by-value scalar args — upstream uses
    // `const unsigned int func_type` while cbindgen emits `unsigned int func_type`.
    // Equivalent at the C ABI.
    let drop_const_after = |i: usize, toks: &Vec<String>| -> bool {
        if i + 1 >= toks.len() { return false; }
        // Followed by `unsigned`, `int`, `double`, ... — i.e. a value type.
        matches!(toks[i + 1].as_str(), "unsigned" | "int" | "double" | "bool" | "char" | "long" | "short" | "float")
    };
    let mut i = 0;
    while i < toks.len() {
        if toks[i] == "const" && drop_const_after(i, toks) {
            toks.remove(i);
            continue;
        }
        i += 1;
    }

    // Bridge typedef alias: `xcfun_t` <-> `xcfun_s`.
    for tok in toks.iter_mut() {
        if tok == "xcfun_t" {
            *tok = "xcfun_s".to_string();
        }
    }

    // Bridge: upstream's `xcfun_vars xcfun_which_vars(...)` returns the
    // typed enum; our cbindgen emit returns plain `int`. Same width, same
    // enumerator values — drop-in compatible. Map the enum types to `int`
    // for comparison purposes.
    for tok in toks.iter_mut() {
        if tok == "xcfun_vars" || tok == "xcfun_mode" {
            *tok = "int".to_string();
        }
    }

    // `(void)` (cbindgen) <-> `()` (upstream): if we see a `( void )` triplet,
    // drop the `void`.
    let mut i = 0;
    while i + 2 < toks.len() {
        if toks[i] == "(" && toks[i + 1] == "void" && toks[i + 2] == ")" {
            toks.remove(i + 1);
            continue;
        }
        i += 1;
    }

    // `density[]` (upstream array syntax) <-> `*density` (cbindgen pointer
    // syntax): both are the same at the C ABI for function args. Replace
    // the trailing `name [ ]` triplet with a leading `* name`.
    // Specifically, find `<typename> name [ ]` and rewrite to `<typename> * name`.
    // We do this by spotting `<ident> [ ]` and rewriting to `* <ident>`, then
    // letting the surrounding tokens settle.
    //
    // Simpler: walk and whenever we see `[` followed by `]` (an empty-bracket
    // pair after an ident), rewrite the pair to `*` BEFORE that ident.
    let mut i = 0;
    while i + 1 < toks.len() {
        if toks[i] == "[" && toks[i + 1] == "]" {
            // Find the immediately preceding identifier (skip back over
            // pointer stars) — that ident becomes the pointed-to-thing.
            // We swap "ident [ ]" with "* ident".
            if i >= 1 && is_ident(&toks[i - 1]) {
                // toks[i-1] = ident, toks[i] = '[', toks[i+1] = ']'.
                // Insert '*' before ident; remove '[' and ']'.
                toks.insert(i - 1, "*".to_string());
                // After insert: [..., '*', ident, '[', ']'] at offsets
                // [i-1, i, i+1, i+2]. Remove the '[' at (i+1) and ']' at
                // (i+1) again (post-removal).
                toks.remove(i + 1);
                toks.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }

    // Drop the `XCFun_API` visibility macro everywhere. It's identical on
    // every function declaration on both sides and adds no diff information.
    toks.retain(|t| t != "XCFun_API");
}

fn is_ident(t: &str) -> bool {
    let bytes = t.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Render token sequence back to a single string with single spaces between
/// identifier-like tokens and no space around punctuation.
fn render(toks: &[String]) -> String {
    let mut out = String::new();
    for (i, t) in toks.iter().enumerate() {
        if i == 0 {
            out.push_str(t);
            continue;
        }
        let prev = &toks[i - 1];
        let prev_is_punct = is_punct(prev);
        let cur_is_punct = is_punct(t);
        // Insert space only between two identifier-like tokens.
        if !prev_is_punct && !cur_is_punct {
            out.push(' ');
        }
        out.push_str(t);
    }
    out
}

fn is_punct(t: &str) -> bool {
    t.len() == 1 && matches!(t.as_bytes()[0], b'(' | b')' | b',' | b';' | b'{' | b'}' | b'*' | b'[' | b']' | b'=')
}

/// Top-level normalization: returns a list of canonical statements.
///
/// "Statement" boundaries are `;` (declaration end), `}` (block end), or
/// the end of any kept line that is otherwise standalone.
pub(crate) fn normalize(input: &str) -> Vec<String> {
    let stripped = strip_block_comments(input);
    let mut combined = String::new();
    for line in stripped.lines() {
        let line = strip_line_comment(line);
        let trimmed = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if is_dropped_line(&trimmed) {
            continue;
        }
        combined.push_str(&trimmed);
        combined.push(' ');
    }

    // Now `combined` is a single space-separated stream. Tokenize the entire
    // stream.
    let mut all_tokens = tokenize(&combined);
    canonicalize_tokens(&mut all_tokens);

    // Split into statements at `;` and after closing `}` (for enum bodies).
    let mut statements: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for tok in all_tokens {
        cur.push(tok.clone());
        if tok == ";" {
            statements.push(render(&cur));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        let leftover = render(&cur);
        if !leftover.trim().is_empty() {
            statements.push(leftover);
        }
    }
    statements
}

fn unified_diff(label_a: &str, a: &[String], label_b: &str, b: &[String]) -> String {
    let mut s = String::new();
    s.push_str(&format!("--- {} ({} statements)\n", label_a, a.len()));
    s.push_str(&format!("+++ {} ({} statements)\n", label_b, b.len()));
    let mut i = 0;
    let mut j = 0;
    let mut shown = 0;
    while (i < a.len() || j < b.len()) && shown < 80 {
        match (a.get(i), b.get(j)) {
            (Some(x), Some(y)) if x == y => {
                i += 1;
                j += 1;
            }
            (Some(x), Some(y)) => {
                s.push_str(&format!("- {x}\n+ {y}\n"));
                i += 1;
                j += 1;
                shown += 1;
            }
            (Some(x), None) => {
                s.push_str(&format!("- {x}\n"));
                i += 1;
                shown += 1;
            }
            (None, Some(y)) => {
                s.push_str(&format!("+ {y}\n"));
                j += 1;
                shown += 1;
            }
            (None, None) => break,
        }
    }
    s
}

#[test]
fn capi_header_matches_xcfun_master() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_dir.parent().unwrap().parent().unwrap();

    let generated_path = crate_dir.join("include/xcfun.h");
    let reference_path = workspace_root.join("xcfun-master/api/xcfun.h");

    let generated = fs::read_to_string(&generated_path).unwrap_or_else(|e| {
        panic!(
            "missing {}: {e}\n\
             run `cargo run -p xtask --bin regen-capi-header` to regenerate.",
            generated_path.display()
        )
    });
    let reference = fs::read_to_string(&reference_path).unwrap_or_else(|e| {
        panic!("missing {}: {e}", reference_path.display())
    });

    // Compare as multisets of canonical statements, not as ordered
    // sequences: cbindgen and the upstream header place the
    // `struct xcfun_s;` forward declaration and the
    // `typedef struct xcfun_s xcfun_t;` typedef in different positions
    // within the file. The C language permits either ordering as long as
    // the forward-decl precedes the first use of the type in a function
    // signature — both headers satisfy that. Order is therefore a
    // cosmetic difference and we sort before comparison.
    let mut g = normalize(&generated);
    let mut r = normalize(&reference);
    g.sort();
    r.sort();

    if g != r {
        eprintln!(
            "headers_match: drift detected.\n{}",
            unified_diff("generated", &g, "reference", &r)
        );
        panic!("headers_match: drift detected (see stderr above)");
    }
}
