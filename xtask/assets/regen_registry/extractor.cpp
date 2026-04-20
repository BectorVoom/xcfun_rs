// xcfun_rs Phase 2 Plan 02-02 Wave-1A-2 registry extractor.
//
// Reads xcfun-master/src/functionals/*.cpp, parses each FUNCTIONAL(XC_*)
// macro, and emits JSONL to stdout:
//
//   {"type":"functional","id":"XC_SLATERX","short_desc":"...","long_desc":"...",
//    "depends":1,"test_vars":"XC_A_B","test_mode":"XC_PARTIAL_DERIVATIVES",
//    "test_order":2,"test_threshold":1e-11,
//    "test_in":[39.0,38.0],"test_out":[...]}
//
// Also reads xcfun-master/src/xcint.cpp and emits one vars_row per xcint_vars
// entry. Aliases are out of scope for Phase 2 (emits empty array).
//
// Design decisions:
// 1. Regex-based parser (not a full C++ parser). Targets the very regular
//    FUNCTIONAL(XC_*) = { ... }; shape. LDAERFC_JT / TW / VWK / VWN3C short
//    macros (no test_vars/test_threshold/test_in/test_out) are handled by
//    the "optional tail" branch that matches {ENERGY_FUNCTION(...)}; with no
//    trailing identifiers.
// 2. String literals are collapsed (adjacent "foo" "bar" -> "foobar") during
//    pre-processing so the regex only sees one quoted literal per slot.
// 3. `#ifdef HIGH_DENSITY` branches (pz81c.cpp) — we strip `#ifdef HIGH_DENSITY
//    ... #else ... #endif` and keep the #else branch (the low-density default
//    used when the xcfun CMake build does not -DHIGH_DENSITY).
// 4. JSON escaping only covers `"` and `\` (xcfun text is ASCII).
// 5. Line comments `// ...` are stripped before regex application; block
//    comments `/* ... */` are stripped too.

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <regex>
#include <sstream>
#include <string>
#include <vector>

namespace fs = std::filesystem;

// ---------- helpers ----------

static std::string read_file(const fs::path & p) {
    std::ifstream f(p);
    if (!f) {
        std::cerr << "extractor: cannot open " << p << "\n";
        std::exit(2);
    }
    std::stringstream ss;
    ss << f.rdbuf();
    return ss.str();
}

// Strip // line comments (not inside strings). xcfun sources have no raw
// strings; simple state machine is sufficient.
static std::string strip_line_comments(const std::string & src) {
    std::string out;
    out.reserve(src.size());
    bool in_string = false;
    bool escape = false;
    for (size_t i = 0; i < src.size(); ++i) {
        char c = src[i];
        if (in_string) {
            out.push_back(c);
            if (escape) { escape = false; continue; }
            if (c == '\\') { escape = true; continue; }
            if (c == '"') { in_string = false; continue; }
            continue;
        }
        if (c == '"') { in_string = true; out.push_back(c); continue; }
        if (c == '/' && i + 1 < src.size() && src[i + 1] == '/') {
            // skip to end of line
            while (i < src.size() && src[i] != '\n') ++i;
            out.push_back('\n');
            continue;
        }
        out.push_back(c);
    }
    return out;
}

// Strip /* ... */ block comments.
static std::string strip_block_comments(const std::string & src) {
    std::string out;
    out.reserve(src.size());
    bool in_string = false;
    bool escape = false;
    for (size_t i = 0; i < src.size(); ++i) {
        char c = src[i];
        if (in_string) {
            out.push_back(c);
            if (escape) { escape = false; continue; }
            if (c == '\\') { escape = true; continue; }
            if (c == '"') { in_string = false; continue; }
            continue;
        }
        if (c == '"') { in_string = true; out.push_back(c); continue; }
        if (c == '/' && i + 1 < src.size() && src[i + 1] == '*') {
            // skip to */
            i += 2;
            while (i + 1 < src.size() && !(src[i] == '*' && src[i + 1] == '/')) ++i;
            i += 1;  // step to '/' (loop ++i moves past)
            out.push_back(' ');
            continue;
        }
        out.push_back(c);
    }
    return out;
}

// Resolve `#ifdef <SYMBOL> ... #else ... #endif` preprocessor blocks by
// keeping the `#else` branch. This matches the default xcfun build which
// does not define HIGH_DENSITY or INEXACT_PI. We iterate until the source
// stabilises so nested #ifdef blocks (e.g., p86c.cpp has HIGH_DENSITY
// wrapping INEXACT_PI wrapping INEXACT_PI) all collapse.
static std::string resolve_high_density(const std::string & src) {
    std::string cur = src;
    // Innermost (no nested #ifdef inside) first.
    std::regex re(R"(#ifdef\s+[A-Z_][A-Z0-9_]*\s*((?:(?!#ifdef|#else|#endif)[\s\S])*?)#else\s*((?:(?!#ifdef|#else|#endif)[\s\S])*?)#endif)",
                  std::regex::ECMAScript);
    // Iterate to handle nested #ifdefs (after the innermost collapses,
    // the outer becomes a flat #ifdef ... #else ... #endif).
    for (int iter = 0; iter < 32; ++iter) {
        std::string next = std::regex_replace(cur, re, "$2");
        if (next == cur) return cur;
        cur = next;
    }
    return cur;
}

// Remove ALL #ifdef / #ifndef / #endif lines and ALL #include / #pragma once
// (the extractor does not need them — it operates on the post-pre-processed
// textual skeleton).
static std::string strip_preprocessor(const std::string & src) {
    std::string out;
    std::stringstream ss(src);
    std::string line;
    while (std::getline(ss, line)) {
        std::string trimmed = line;
        size_t start = trimmed.find_first_not_of(" \t");
        if (start != std::string::npos && trimmed[start] == '#') {
            // drop preprocessor line
            out.push_back('\n');
            continue;
        }
        out += line;
        out.push_back('\n');
    }
    return out;
}

// Collapse adjacent string literals: "foo" "bar" -> "foobar".
// Walks char-by-char, detects a closing " followed by whitespace then an
// opening ", and removes the gap. Handles escaped quotes.
static std::string collapse_adjacent_strings(const std::string & src) {
    std::string out;
    out.reserve(src.size());
    size_t i = 0;
    while (i < src.size()) {
        char c = src[i];
        if (c != '"') { out.push_back(c); ++i; continue; }
        // start of a string literal; scan to its closing "
        std::string buf = "\"";
        ++i;
        while (i < src.size()) {
            if (src[i] == '\\' && i + 1 < src.size()) {
                buf.push_back(src[i]);
                buf.push_back(src[i + 1]);
                i += 2;
                continue;
            }
            if (src[i] == '"') {
                buf.push_back('"');
                ++i;
                break;
            }
            buf.push_back(src[i]);
            ++i;
        }
        // buf is one string literal including quotes. Peek forward for ws + ".
        size_t j = i;
        while (j < src.size() && (src[j] == ' ' || src[j] == '\t' || src[j] == '\n' || src[j] == '\r')) ++j;
        if (j < src.size() && src[j] == '"') {
            // merge: drop closing quote of buf, skip opening quote of next
            buf.pop_back();
            i = j + 1;
            // append contents until we hit the next closing quote (handle
            // escapes); keep appending across further adjacent literals by
            // re-entering the loop, but for simplicity iterate inline.
            bool done = false;
            while (!done) {
                while (i < src.size()) {
                    if (src[i] == '\\' && i + 1 < src.size()) {
                        buf.push_back(src[i]);
                        buf.push_back(src[i + 1]);
                        i += 2;
                        continue;
                    }
                    if (src[i] == '"') {
                        ++i;
                        break;
                    }
                    buf.push_back(src[i]);
                    ++i;
                }
                // look for another adjacent "
                size_t k = i;
                while (k < src.size() && (src[k] == ' ' || src[k] == '\t' || src[k] == '\n' || src[k] == '\r')) ++k;
                if (k < src.size() && src[k] == '"') {
                    i = k + 1;
                } else {
                    buf.push_back('"');
                    done = true;
                }
            }
        }
        out += buf;
    }
    return out;
}

// JSON-escape (only `"` and `\` need handling for xcfun ASCII text, plus \n).
static std::string json_escape(const std::string & s) {
    std::string out;
    out.reserve(s.size() + 8);
    for (char c : s) {
        switch (c) {
            case '"':  out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n";  break;
            case '\r': out += "\\r";  break;
            case '\t': out += "\\t";  break;
            default:
                if (static_cast<unsigned char>(c) < 0x20) {
                    char buf[8];
                    std::snprintf(buf, sizeof(buf), "\\u%04x", c);
                    out += buf;
                } else {
                    out.push_back(c);
                }
        }
    }
    return out;
}

// Decode a C string literal (with leading+trailing " already included) to
// its textual content.
static std::string decode_c_string(const std::string & literal) {
    std::string out;
    out.reserve(literal.size());
    if (literal.size() < 2 || literal.front() != '"' || literal.back() != '"') {
        return "";
    }
    for (size_t i = 1; i + 1 < literal.size(); ++i) {
        char c = literal[i];
        if (c == '\\' && i + 2 < literal.size()) {
            char n = literal[i + 1];
            ++i;
            switch (n) {
                case 'n':  out.push_back('\n'); break;
                case 'r':  out.push_back('\r'); break;
                case 't':  out.push_back('\t'); break;
                case '"':  out.push_back('"');  break;
                case '\\': out.push_back('\\'); break;
                case '0':  out.push_back('\0'); break;
                default:   out.push_back(n);    break;
            }
            continue;
        }
        out.push_back(c);
    }
    return out;
}

// Translate a depends bitmask expression (e.g., "XC_DENSITY | XC_GRADIENT")
// to integer bitmask (matches xcint.hpp:46-50).
static int depends_bits(const std::string & expr) {
    int bits = 0;
    if (expr.find("XC_DENSITY") != std::string::npos) bits |= 1;
    if (expr.find("XC_GRADIENT") != std::string::npos) bits |= 2;
    if (expr.find("XC_LAPLACIAN") != std::string::npos) bits |= 4;
    if (expr.find("XC_KINETIC") != std::string::npos) bits |= 8;
    if (expr.find("XC_JP") != std::string::npos) bits |= 16;
    return bits;
}

// Parse an array literal "{v1, v2, v3, ...}" into a vector<double>.
static std::vector<double> parse_array(const std::string & body) {
    std::vector<double> out;
    // Strip outer braces + whitespace.
    std::string s = body;
    size_t a = s.find('{');
    size_t b = s.rfind('}');
    if (a == std::string::npos || b == std::string::npos || a >= b) return out;
    std::string inner = s.substr(a + 1, b - a - 1);
    // Split by comma (depth 0). xcfun arrays are flat.
    std::stringstream ss(inner);
    std::string tok;
    while (std::getline(ss, tok, ',')) {
        // trim whitespace
        size_t start = tok.find_first_not_of(" \t\n\r");
        size_t end   = tok.find_last_not_of(" \t\n\r");
        if (start == std::string::npos) continue;
        std::string t = tok.substr(start, end - start + 1);
        if (t.empty()) continue;
        try {
            size_t used = 0;
            double val = std::stod(t, &used);
            out.push_back(val);
        } catch (...) {
            std::cerr << "extractor: cannot parse double '" << t << "'\n";
            std::exit(2);
        }
    }
    return out;
}

static void emit_doubles(std::ostream & os, const std::vector<double> & vals) {
    os << "[";
    for (size_t i = 0; i < vals.size(); ++i) {
        if (i) os << ",";
        char buf[64];
        std::snprintf(buf, sizeof(buf), "%.17g", vals[i]);
        os << buf;
    }
    os << "]";
}

// ---------- FUNCTIONAL macro parsing ----------

struct Functional {
    std::string id;
    std::string short_desc;
    std::string long_desc;
    int depends = 0;
    // Optional tail — absent for VWN3C / LDAERFC_JT / TW / VWK shapes.
    bool has_tail = false;
    std::string test_vars;
    std::string test_mode;
    int test_order = 0;
    double test_threshold = 0.0;
    std::vector<double> test_in;
    std::vector<double> test_out;
};

static std::vector<Functional> parse_functionals_from_file(const std::string & filename,
                                                            const std::string & src) {
    std::vector<Functional> out;
    // Find every FUNCTIONAL(<ID>) = { ... }; block.
    // The payload may contain nested braces inside {test_in}, {test_out}.
    // We locate the opening `{` after `=` and walk the string tracking brace
    // depth until depth returns to 0.
    size_t pos = 0;
    std::regex head_re(R"(FUNCTIONAL\s*\(\s*(XC_[A-Z0-9_]+)\s*\)\s*=\s*\{)",
                       std::regex::ECMAScript);
    auto it_begin = std::sregex_iterator(src.begin(), src.end(), head_re);
    auto it_end   = std::sregex_iterator();
    for (auto it = it_begin; it != it_end; ++it) {
        std::string id = (*it)[1].str();
        (void)filename;  // Available for optional stderr tracing.
        size_t brace_pos = it->position(0) + it->length(0) - 1;  // index of '{'
        int depth = 1;
        size_t i = brace_pos + 1;
        while (i < src.size() && depth > 0) {
            char c = src[i];
            if (c == '"') {
                // skip string literal
                ++i;
                while (i < src.size()) {
                    if (src[i] == '\\' && i + 1 < src.size()) { i += 2; continue; }
                    if (src[i] == '"') { ++i; break; }
                    ++i;
                }
                continue;
            }
            if (c == '{') ++depth;
            else if (c == '}') --depth;
            ++i;
        }
        if (depth != 0) {
            std::cerr << "extractor: unbalanced braces in " << filename << " for " << id << "\n";
            continue;
        }
        size_t close_pos = i - 1;  // index of matching '}'
        std::string payload = src.substr(brace_pos + 1, close_pos - brace_pos - 1);

        Functional f;
        f.id = id;

        // Split payload by commas at depth 0 (outside strings/braces).
        std::vector<std::string> fields;
        {
            std::string cur;
            int d = 0;
            bool in_str = false;
            bool esc = false;
            for (size_t k = 0; k < payload.size(); ++k) {
                char c = payload[k];
                if (in_str) {
                    cur.push_back(c);
                    if (esc) { esc = false; continue; }
                    if (c == '\\') { esc = true; continue; }
                    if (c == '"') in_str = false;
                    continue;
                }
                if (c == '"') { in_str = true; cur.push_back(c); continue; }
                if (c == '{') { ++d; cur.push_back(c); continue; }
                if (c == '}') { --d; cur.push_back(c); continue; }
                if (c == ',' && d == 0) {
                    fields.push_back(cur);
                    cur.clear();
                    continue;
                }
                cur.push_back(c);
            }
            if (!cur.empty()) fields.push_back(cur);
        }

        // Trim helper.
        auto trim = [](const std::string & s) {
            size_t a = s.find_first_not_of(" \t\n\r");
            size_t b = s.find_last_not_of(" \t\n\r");
            if (a == std::string::npos) return std::string();
            return s.substr(a, b - a + 1);
        };

        // Fields after splitting by commas at depth 0:
        //   0: "short_desc"
        //   1: "long_desc"
        //   2: depends expression (e.g., XC_DENSITY | XC_GRADIENT)
        //   3: ENERGY_FUNCTION(fn) [possibly followed by test_vars ident]
        //   4: test_mode
        //   5: test_order
        //   6: test_threshold
        //   7: test_in
        //   8: test_out
        if (fields.size() < 4) continue;

        // field 0: short_desc (string literal)
        f.short_desc = decode_c_string(trim(fields[0]));
        // field 1: long_desc (string literal)
        f.long_desc = decode_c_string(trim(fields[1]));
        // field 2: depends bitmask expression
        f.depends = depends_bits(trim(fields[2]));

        // field 3: ENERGY_FUNCTION(fn) [+ test_vars]
        std::string f3 = trim(fields[3]);
        std::smatch m;
        std::regex vars_re(R"(ENERGY_FUNCTION\s*\([^)]*\)\s+(XC_[A-Z0-9_]+))",
                           std::regex::ECMAScript);
        if (std::regex_search(f3, m, vars_re)) {
            f.test_vars = m[1].str();
        }

        if (fields.size() >= 9 && !f.test_vars.empty()) {
            f.has_tail = true;
            f.test_mode = trim(fields[4]);
            try {
                f.test_order = std::stoi(trim(fields[5]));
            } catch (...) { f.test_order = 0; }
            try {
                f.test_threshold = std::stod(trim(fields[6]));
            } catch (...) { f.test_threshold = 0.0; }
            f.test_in = parse_array(trim(fields[7]));
            f.test_out = parse_array(trim(fields[8]));
        }

        out.push_back(std::move(f));
    }
    return out;
}

// ---------- xcint.cpp vars table parsing ----------

struct VarsRow {
    std::string symbol;
    int len;
    int provides;
};

static std::vector<VarsRow> parse_vars_table(const std::string & xcint_src) {
    std::vector<VarsRow> out;
    // Locate `xcint_vars[XC_NR_VARS] = {`
    std::regex head_re(R"(xcint_vars\s*\[\s*XC_NR_VARS\s*\]\s*=\s*\{)",
                       std::regex::ECMAScript);
    std::smatch mh;
    if (!std::regex_search(xcint_src, mh, head_re)) {
        std::cerr << "extractor: xcint_vars table not found\n";
        std::exit(2);
    }
    size_t start = mh.position(0) + mh.length(0);
    // Find matching `};` by brace-depth walk.
    int depth = 1;
    size_t i = start;
    while (i < xcint_src.size() && depth > 0) {
        char c = xcint_src[i];
        if (c == '{') ++depth;
        else if (c == '}') --depth;
        ++i;
    }
    std::string body = xcint_src.substr(start, i - 1 - start);
    // Each row: {"XC_<NAME>", <len>, <depends_expr>}
    std::regex row_re(R"(\{\s*\"(XC_[A-Z0-9_]+)\"\s*,\s*(\d+)\s*,\s*([^}]*)\})",
                      std::regex::ECMAScript);
    auto it_begin = std::sregex_iterator(body.begin(), body.end(), row_re);
    auto it_end = std::sregex_iterator();
    for (auto it = it_begin; it != it_end; ++it) {
        VarsRow r;
        r.symbol = (*it)[1].str();
        r.len = std::stoi((*it)[2].str());
        r.provides = depends_bits((*it)[3].str());
        out.push_back(std::move(r));
    }
    return out;
}

// ---------- main ----------

int main(int argc, char ** argv) {
    if (argc < 2) {
        std::cerr << "usage: extractor <xcfun-master-root>\n";
        return 2;
    }
    fs::path root = argv[1];
    fs::path functionals = root / "src" / "functionals";
    fs::path xcint_cpp = root / "src" / "xcint.cpp";

    if (!fs::is_directory(functionals)) {
        std::cerr << "extractor: not a directory: " << functionals << "\n";
        return 2;
    }
    if (!fs::is_regular_file(xcint_cpp)) {
        std::cerr << "extractor: not a file: " << xcint_cpp << "\n";
        return 2;
    }

    // Collect .cpp files (sorted for deterministic output).
    std::vector<fs::path> cpp_files;
    for (auto & e : fs::directory_iterator(functionals)) {
        if (!e.is_regular_file()) continue;
        auto p = e.path();
        if (p.extension() != ".cpp") continue;
        // Skip zone-identifier sidecars.
        std::string name = p.filename().string();
        if (name.find("Zone.Identifier") != std::string::npos) continue;
        cpp_files.push_back(p);
    }
    std::sort(cpp_files.begin(), cpp_files.end());

    // Parse each .cpp file.
    std::vector<Functional> all;
    for (auto & p : cpp_files) {
        std::string raw = read_file(p);
        std::string s = strip_line_comments(raw);
        s = strip_block_comments(s);
        s = resolve_high_density(s);
        s = strip_preprocessor(s);
        s = collapse_adjacent_strings(s);
        auto fns = parse_functionals_from_file(p.filename().string(), s);
        for (auto & f : fns) all.push_back(std::move(f));
    }

    // Emit JSONL.
    for (const auto & f : all) {
        std::cout << "{\"type\":\"functional\""
                  << ",\"id\":\"" << f.id << "\""
                  << ",\"short_desc\":\"" << json_escape(f.short_desc) << "\""
                  << ",\"long_desc\":\"" << json_escape(f.long_desc) << "\""
                  << ",\"depends\":" << f.depends;
        if (f.has_tail) {
            std::cout << ",\"test_vars\":\"" << f.test_vars << "\""
                      << ",\"test_mode\":\"" << f.test_mode << "\""
                      << ",\"test_order\":" << f.test_order;
            char buf[64];
            std::snprintf(buf, sizeof(buf), "%.17g", f.test_threshold);
            std::cout << ",\"test_threshold\":" << buf;
            std::cout << ",\"test_in\":";
            emit_doubles(std::cout, f.test_in);
            std::cout << ",\"test_out\":";
            emit_doubles(std::cout, f.test_out);
        } else {
            std::cout << ",\"test_vars\":null,\"test_mode\":null,\"test_order\":null"
                      << ",\"test_threshold\":null,\"test_in\":null,\"test_out\":null";
        }
        std::cout << "}\n";
    }

    // xcint.cpp vars table.
    {
        std::string raw = read_file(xcint_cpp);
        std::string s = strip_line_comments(raw);
        s = strip_block_comments(s);
        auto rows = parse_vars_table(s);
        for (const auto & r : rows) {
            std::cout << "{\"type\":\"vars_row\""
                      << ",\"symbol\":\"" << r.symbol << "\""
                      << ",\"len\":" << r.len
                      << ",\"provides\":" << r.provides
                      << "}\n";
        }
    }

    // Aliases (Phase 2: empty).
    std::cout << "{\"type\":\"aliases\",\"entries\":[]}\n";
    return 0;
}
