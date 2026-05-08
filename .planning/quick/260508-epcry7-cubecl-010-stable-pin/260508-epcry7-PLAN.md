---
quick_id: 260508-epcry7
slug: cubecl-010-stable-pin
description: Update cubecl from =0.10.0-pre.3 to =0.10.0 stable across all workspace crates
date: 2026-05-08
mode: quick-full
must_haves:
  truths:
    - All 5 cubecl-* pins in Cargo.toml read "=0.10.0" (not "=0.10.0-pre.3")
    - check-cubecl-pin xtask REQUIRED_VERSION constant is "0.10.0"
    - CLAUDE.md references updated to reflect stable 0.10.0
    - cargo metadata resolves without error on the updated lockfile
  artifacts:
    - Cargo.toml (workspace root)
    - xtask/src/bin/check_cubecl_pin.rs
    - CLAUDE.md
  key_links:
    - Cargo.toml:47-54
    - xtask/src/bin/check_cubecl_pin.rs:16 (REQUIRED_VERSION)
---

# Quick Task 260508-epcry7: Update cubecl to =0.10.0 stable

## Context

All 5 cubecl-* crates published stable 0.10.0 on 2026-05-07 (not yanked, confirmed via crates.io API).
Current workspace pin: `=0.10.0-pre.3`. This is the "cubecl bump sub-phase" CLAUDE.md describes.

The xtask `check-cubecl-pin` binary enforces exact version matching — it must be updated in lockstep.

## Tasks

### Task 1: Bump workspace Cargo.toml pins

**Files:** `Cargo.toml`

**Action:** Replace all 5 occurrences of `"=0.10.0-pre.3"` with `"=0.10.0"` in the
`[workspace.dependencies]` block (lines ~50-54). Also update the comment on line ~47 that
references the pre-release pin.

**Verify:** `grep -n "0.10.0" Cargo.toml` shows only `"=0.10.0"`, no `-pre.3` suffix.

**Done:** All 5 cubecl-* lines read `= "=0.10.0"`.

---

### Task 2: Update xtask check-cubecl-pin

**Files:** `xtask/src/bin/check_cubecl_pin.rs`

**Action:**
- Change `const REQUIRED_VERSION: &str = "0.10.0-pre.3";` → `"0.10.0"`
- Update the file-level doc comment (line 4) and the QG-06 footer message to remove pre-release language
- Remove/update the comment "Pre-release crates do not respect semver" since 0.10.0 is now stable

**Verify:** `cargo run -p xtask --bin check-cubecl-pin` exits 0 (PASS).

**Done:** `REQUIRED_VERSION = "0.10.0"` and xtask passes.

---

### Task 3: Update CLAUDE.md

**Files:** `CLAUDE.md`

**Action:** Replace all occurrences of `0.10.0-pre.3` with `0.10.0` throughout CLAUDE.md.
Also update contextual notes that refer to "pre-release" status of 0.10.0 (e.g. "max_stable_version = 0.9.0"
note should reflect that 0.10.0 is now stable; the TL;DR table CONFIRM/FLAG note; Risk Assessment row for
"cubecl 0.10 never ships stable").

**Verify:** `grep "0.10.0-pre.3" CLAUDE.md` returns empty.

**Done:** CLAUDE.md reflects stable 0.10.0 everywhere.

---

### Task 4: Verify cargo resolves and compiles

**Files:** (none modified)

**Action:** Run `cargo metadata --format-version 1 2>&1 | tail -1` to confirm Cargo resolves.
Then run `cargo check -p xcfun-kernels -p xcfun-ad -p xcfun-eval 2>&1 | tail -5` for a fast
compile check against the default CPU feature set.

**Verify:** Both commands exit 0.

**Done:** No compile errors after pin bump.
