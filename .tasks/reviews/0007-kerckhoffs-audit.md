# Security Audit: 0007 Package Manager Abstraction Layer

**Auditor:** Auguste Kerckhoffs
**Date:** 2026-02-24
**Files reviewed:**
- `/home/isaac/src/sh.great/src/platform/package_manager.rs` (lines 1-444, production code)
- `/home/isaac/src/sh.great/src/cli/apply.rs` (call sites at lines 542, 700, 773)
- `/home/isaac/src/sh.great/src/cli/doctor.rs` (call site at line 108)
- `/home/isaac/src/sh.great/Cargo.toml` (dependency check)
- `/home/isaac/src/sh.great/src/platform/detection.rs` (command_exists verification)

**Verdict: PASS -- No CRITICAL or HIGH findings.**

---

## Checklist Results

### 1. Command Injection -- PASS

All 14 `Command::new()` calls in `package_manager.rs` use either:
- Hardcoded command names: `"brew"`, `"dpkg"`, `"sudo"`, `"cargo"`, `"npm"`
- The `package` parameter passed via `.arg()` / `.args()` (separate OS strings)

No shell invocation (`sh -c`, `bash -c`) anywhere in the file. `std::process::Command` passes arguments as individual OS strings without shell expansion. A malicious package name like `"pkg; rm -rf /"` in `great.toml` is passed as a single argument to the package manager binary, which will simply fail with "package not found."

`command_exists()` at `detection.rs:135` uses `which::which()` (crate v7) -- pure PATH lookup, no shell.

### 2. Privilege Escalation via sudo -- PASS

- `sudo` is only invoked by `Apt::install` (line 192) and `Apt::update` (line 222)
- When `non_interactive` is true, `sudo -n` is used (lines 193-195, 223-225), which fails immediately (exit 1) if credentials are not cached -- no hanging on password prompt
- Error messages for non-interactive failure are actionable and tell the user to run the command manually (lines 202-207, 231-236)
- `non_interactive` field is properly gated via `Apt::new(bool)` constructor (line 145)
- Other managers (Homebrew, Cargo, npm) never invoke `sudo`

### 3. Credential Leakage -- PASS

- Error messages contain only: package names, exit codes, and command names
- No API keys, tokens, secrets, or environment variable values appear in any error path
- The word "password" appears only in doc comments and in the user-facing sudo guidance message ("sudo requires a password")
- No `Debug` derive on any struct that could leak fields via `{:?}` formatting

### 4. .unwrap() / .expect() in Production Code -- PASS

Zero instances of `.unwrap()` or `.expect()` in production code (lines 1-444).

Four safe `.unwrap_or()` calls:
- Lines 59, 166: `.unwrap_or(false)` on `Result<ExitStatus>` -- returns false if spawn fails
- Lines 279, 360: `.unwrap_or("")` on `Option<&str>` from iterator -- returns empty string

### 5. Supply Chain -- PASS

- **No new dependencies added.** `Cargo.toml` is unchanged from the previous commit.
- All crates used (`anyhow`, `which`, `std::process`) were already in the dependency tree.

### 6. is_available() Guards -- PASS

All 8 `install()`/`update()` methods now have `is_available()` pre-checks that bail with clear error messages before attempting to spawn any command:
- `Homebrew::install` (line 77), `Homebrew::update` (line 109)
- `Apt::install` (line 186), `Apt::update` (line 219)
- `CargoInstaller::install` (line 291), `CargoInstaller::update` (line 314)
- `NpmInstaller::install` (line 372), `NpmInstaller::update` (line 393)

---

## LOW Findings (P3 -- non-blocking)

### L1: installed_version executes user-specified binary name

`CargoInstaller::installed_version` (line 271) and `NpmInstaller::installed_version` (line 352) call `Command::new(package).arg("--version")` where `package` comes from `great.toml`. This executes whatever binary the package name resolves to on PATH. This is by design (the user controls their own config file) and the `--version` flag is benign, but worth noting that a `great.toml` in a cloned repo could cause arbitrary binary execution via `great apply` or `great status`.

**Risk:** LOW. The user must explicitly run `great apply` and the binary must already be on PATH. The `great.toml` is a trust boundary the user has opted into.

### L2: Apt version pinning silently ignored

`Apt::install` (line 185) takes `_version: Option<&str>` but ignores it entirely. If a user specifies a version for an apt package in `great.toml`, it will silently install the latest available version. Not a security issue, but a correctness note.

---

## Build Verification (post-fix)

After Da Vinci completed all call-site and test updates:
- `cargo check` -- PASS (clean compile)
- `cargo clippy -- -D warnings` -- PASS (zero warnings)
- `cargo test -- package_manager` -- PASS (12/12 tests)
- All 4 call sites in `apply.rs` (lines 542, 700, 773) and `doctor.rs` (line 108) correctly pass `false`
- All test code uses `Apt::new(false)` and `available_managers(false)`

---

## Summary

| Severity | Count | Details |
|----------|-------|---------|
| CRITICAL | 0 | -- |
| HIGH     | 0 | -- |
| MEDIUM   | 0 | -- |
| LOW      | 2 | L1: binary execution by name, L2: apt version ignored |

**Verdict: PASS.** No findings that block commit. The code is secure and the build is green.
