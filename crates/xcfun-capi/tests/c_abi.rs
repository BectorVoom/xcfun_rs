//! Phase 5 D-14 + CAPI-07 — compile tests/c_abi.c against
//! libxcfun_capi.a + crates/xcfun-capi/include/xcfun.h, run the
//! resulting binary, assert exit code 0 and stdout contains
//! "ALL FIXTURES PASS".
//!
//! 10 D-14 fixtures executed; row 8 SCANX→TPSSX fallback is authorized
//! by CONTEXT and recorded as Escalation Gate in 05-04-SUMMARY.md if
//! it triggers (not triggered during plan execution); row 10
//! LB94→LDA(Potential) substitution is documented per D-16 verification.
//!
//! Per CLAUDE.md ACC-05/06 the cc invocation passes
//! `-fno-fast-math -ffp-contract=off`; NEVER `-ffast-math` /
//! `-funsafe-math-optimizations` / any reassociation flag.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // workspace root
        .to_path_buf()
}

fn out_dir() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target"));
    let dir = target.join("c_abi_test");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn staticlib_path() -> PathBuf {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target"));
    let candidates = [
        target.join("release/libxcfun_capi.a"),
        target.join("debug/libxcfun_capi.a"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    // Build the staticlib if it isn't on disk yet.
    let status = Command::new("cargo")
        .args(["build", "-p", "xcfun-capi", "--release"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build failed to spawn");
    assert!(
        status.success(),
        "cargo build -p xcfun-capi --release failed"
    );
    candidates[0].clone()
}

fn cc_command() -> String {
    std::env::var("CC").unwrap_or_else(|_| "cc".to_string())
}

#[test]
fn c_abi_drop_in_test() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let c_source = manifest.join("tests/c_abi.c");
    let include = manifest.join("include");
    let staticlib = staticlib_path();

    let outdir = out_dir();
    let obj = outdir.join("c_abi.o");
    let bin = outdir.join("c_abi_runner");

    // Compile the C source. Per CLAUDE.md ACC-05/06 do NOT enable
    // -ffast-math or any reassociation flags.
    let compile = Command::new(cc_command())
        .args([
            "-c",
            "-O2",
            "-fno-fast-math",
            "-ffp-contract=off",
            "-Wall",
            "-Werror",
        ])
        .arg("-I")
        .arg(&include)
        .arg("-o")
        .arg(&obj)
        .arg(&c_source)
        .status()
        .expect("cc compile failed to spawn");
    assert!(
        compile.success(),
        "cc compile of {} failed",
        c_source.display()
    );

    // Link against the staticlib + libm + threading + C++ runtime.
    // cubecl-cpu (a transitive dep) pulls in MLIR/LLVM JIT (via the
    // tracel-llvm crate); the embedded LLVM/MLIR object code references
    // C++ std-lib symbols (operator new/delete, std::generic_category,
    // std::__cxx11::basic_string::_M_create, etc.) and pthread/dl
    // primitives. Linker flags resolve them in order:
    //   - `-lstdc++` — C++ runtime (resolves new/delete + std::*)
    //   - `-lm`      — libm (resolves the math.h fns the C source uses)
    //   - `-lpthread`/`-ldl` — POSIX threading + dynamic loader
    // The exact flag set may need tuning on macOS (libc++ via -lc++) /
    // Windows (no -ldl); for Linux CI (the canonical target), this set
    // resolves all undefined references.
    let link = Command::new(cc_command())
        .arg("-o")
        .arg(&bin)
        .arg(&obj)
        .arg(&staticlib)
        .args(["-lstdc++", "-lm", "-lpthread", "-ldl"])
        .status()
        .expect("cc link failed to spawn");
    assert!(link.success(), "cc link of {} failed", bin.display());

    // Run the binary and capture output.
    let output = Command::new(&bin)
        .output()
        .expect("c_abi binary failed to spawn");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("--- c_abi stdout ---\n{stdout}\n--- c_abi stderr ---\n{stderr}");
    assert!(
        output.status.success(),
        "c_abi binary exited with {:?} -- stderr: {stderr}",
        output.status
    );
    assert!(
        stdout.contains("ALL FIXTURES PASS"),
        "stdout missing 'ALL FIXTURES PASS': {stdout}"
    );
}
