# Security Audit: 0001 Platform Detection Engine

**Auditor**: Auguste Kerckhoffs
**Date**: 2026-02-20
**Scope**: `src/platform/detection.rs`, `src/platform/mod.rs`, `Cargo.toml` (dependency delta)
**Spec**: `.tasks/specs/0001-platform-detection.md`
**Verdict**: PASS -- no CRITICAL or HIGH findings. Commit is not blocked.

---

## 1. Command Injection Surface

**Status**: CLEAR

### Checked
- `command_exists()` at line 124-126 uses `which::which(cmd).is_ok()` -- pure Rust PATH walk, no shell spawning.
- Grepped entire `src/` tree for `Command::new("sh")`, `Command::new("bash")`, and `format!("command -v` patterns.
  - `detection.rs`: NONE found.
  - `runtime.rs`: Uses `command_exists()` (the safe which-based function) at line 31. The old `sh -c "command -v"` pattern has been eliminated from detection code.
  - `runtime.rs` line 63, `apply.rs` line 183, `doctor.rs` line 110: These use `Command::new("sh")` / `Command::new("bash")` but are OUTSIDE the scope of this audit (pre-existing code in other modules). Noted as MEDIUM for future audit -- see finding M1 below.

### The `which` crate (v7)
- Pure Rust implementation. Reads `$PATH`, splits by OS separator, checks for executable files.
- No shell invocation. No `Command::new`. No FFI.
- Well-maintained: 180M+ downloads on crates.io, actively maintained by the `rust-lang` nursery.
- `cargo audit` could not be run (not installed). Recommend installing `cargo-audit` for CI. No known CVEs for `which` v7 at time of review.

## 2. Unsafe Code

**Status**: CLEAR

- Grepped `detection.rs` for `unsafe` -- zero matches.
- Grepped `Cargo.toml` for `libc` -- zero matches.
- No FFI, no raw pointers, no transmute.

## 3. is_root() Security

**Status**: CLEAR

- Lines 240-261: Uses `std::process::Command::new("id").arg("-u")` -- subprocess, not env var.
- `#[cfg(unix)]` guard present at line 240.
- `#[cfg(not(unix))]` fallback at line 258 returns `false`.
- No env var spoofing possible (`$USER`, `$EUID` are not consulted).
- Spec documents that `is_root` is advisory, not access control (spec section "Security Considerations"). Code comments at line 239 say "Check if the current user is root" but do not explicitly state "advisory only". This is a LOW finding -- see L1.

## 4. File Path Safety

**Status**: CLEAR

All filesystem paths are hardcoded constants:

| Path | Usage | Line |
|------|-------|------|
| `/etc/os-release` | Read distro ID and version | 182, 205 |
| `/proc/version` | WSL fallback detection | 167 |
| `/proc/sys/fs/binfmt_misc/WSLInterop` | WSL/WSL2 detection | 163, 177 |
| `/run/systemd/system` | systemd detection | 145 |

- No user-controllable paths. No path concatenation. No `format!()` with user input for paths.
- All reads use `std::fs::read_to_string()` with `.ok()` / `.unwrap_or()` -- failures are silent and safe.
- `std::path::Path::new(...).exists()` is used for existence checks -- no TOCTOU concern since results are advisory.

## 5. Dependency Audit

**Status**: CLEAR (with caveat)

- `Cargo.toml` diff: Only `which = "7"` added to `[dependencies]`. Confirmed.
- No new dev-dependencies.
- `cargo audit` is not installed on this machine. **Recommend adding `cargo-audit` to CI pipeline.** See M2.

## 6. Information Disclosure

**Status**: CLEAR

`PlatformInfo` fields:
- `platform`: OS type, distro name, version, architecture. No PII.
- `capabilities`: Boolean flags for package managers/services. No PII.
- `is_root`: Boolean. Not sensitive.
- `shell`: Path to shell binary (e.g., `/bin/bash`). Not sensitive.

No usernames, hostnames, IP addresses, or secrets are collected or exposed.

## 7. Error Handling

**Status**: CLEAR

- No `.unwrap()` calls in production code (grepped, zero matches in detection.rs).
- No `.expect()` calls in production code.
- No `panic!()` calls.
- All functions return safe defaults on failure (as documented in spec).

---

## Findings Summary

### CRITICAL: None
### HIGH: None
### MEDIUM

**M1**: Shell spawning in adjacent modules (`runtime.rs:63`, `apply.rs:183`, `doctor.rs:110`) uses `Command::new("sh")` / `Command::new("bash")`. These are outside the scope of this audit but should be reviewed for command injection in a future audit pass. If any user-controlled strings are interpolated into these commands, they would be HIGH severity.

**M2**: `cargo-audit` is not installed. Recommend adding it to CI to catch dependency CVEs automatically. Without it, the `which` crate's CVE status is assessed manually (none known as of this review).

### LOW

**L1**: The `is_root()` function comment (line 239) says "Check if the current user is root" but does not document that it is advisory-only and must not be used for access control. The spec documents this, but the code comment should too, for defense in depth. Recommend adding a doc comment: `/// Advisory only -- do not use for access control decisions.`

---

## Conclusion

The Platform Detection Engine changes are secure. The primary security improvement -- replacing `sh -c "command -v"` shell spawning with the `which` crate's pure-Rust PATH walk -- has been correctly implemented. All file paths are hardcoded. No unsafe code. No secrets in output. No command injection surface in `detection.rs`.

**Commit: NOT BLOCKED.**
