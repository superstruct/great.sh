# Turing Test Report: Spec 0001 — Platform Detection Engine

**Tester**: Alan Turing (Adversarial Tester)
**Date**: 2026-02-20
**Spec**: `.tasks/specs/0001-platform-detection.md`
**Implementation**: `src/platform/detection.rs` (unstaged working-tree changes)
**Verdict**: CONDITIONAL PASS — all spec-required checks pass, two pre-existing clippy
failures exist in unrelated modules

---

## Summary

- Tests: 139 total (115 unit + 24 integration), all pass
- `cargo clippy` (production only): PASS — zero warnings
- `cargo clippy --tests`: FAIL — 2 errors (both pre-existing, not in platform/detection.rs)
- `.unwrap()` / `.expect()` in platform production code: ZERO (PASS)
- `which::which` used for command_exists: CONFIRMED
- All 24 spec-mandated tests present: CONFIRMED
- `cargo run -- status` and `status --verbose`: work correctly

---

## Detailed Check Results

---

### CHECK 1: cargo test — all tests pass

```
RESULT: PASS
EVIDENCE:
  Unit tests:    115 passed, 0 failed
  Integration:    24 passed, 0 failed
  Total:         139 tests, 0 failures
  Relevant platform tests (24 in detection.rs):
    test_detect_architecture .......... ok
    test_detect_platform_not_unknown .. ok
    test_command_exists_positive ...... ok
    test_command_exists_negative ...... ok
    test_command_exists_empty_string .. ok
    test_platform_display ............. ok
    test_platform_display_detailed .... ok
    test_detect_platform_info ......... ok
    test_detect_capabilities .......... ok
    test_platform_serialize_roundtrip . ok
    test_wsl_detected_from_env_var .... ok
    test_wsl_detected_from_interop_file ok
    test_wsl_detected_from_proc_version ok
    test_not_wsl_when_all_checks_fail . ok
    test_wsl2_true_when_interop_exists  ok
    test_wsl2_false_when_interop_missing ok
    test_distro_ubuntu ................ ok
    test_distro_unknown_on_missing_file ok
    test_distro_quoted_id ............. ok
    test_distro_arch .................. ok
    test_is_root_true ................. ok
    test_is_root_false ................ ok
    test_detect_shell_unix ............ ok
    test_detect_shell_unset ........... ok
```

---

### CHECK 2: cargo clippy -- -D warnings (production binary only)

```
RESULT: PASS
EVIDENCE:
  $ cargo clippy -- -D warnings
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
  Zero errors, zero warnings.
```

**Note — pre-existing clippy failures in `--tests` mode** (not introduced by spec 0001):

```
SEVERITY: MEDIUM (pre-existing, unrelated to spec 0001)

Failure 1: tests/cli_smoke.rs:7
  error: use of deprecated associated function `assert_cmd::Command::cargo_bin`
  Introduced in commit 7721706 (pre-dates spec 0001 work)

Failure 2: src/cli/loop_cmd.rs:464
  error: this expression always evaluates to false
    |  assert!(!OBSERVER_TEMPLATE.is_empty());
  Introduced in commit 7f648cf (pre-dates spec 0001 work)
  (Clippy::const_is_empty lint — template is actually non-empty at 785 bytes;
   the assertion is a tautology that always passes, but clippy flags it)

Reproduction:
  $ cargo clippy --tests -- -D warnings

These two failures are in unrelated modules (loop_cmd.rs and cli_smoke.rs).
Neither was introduced by the spec 0001 platform detection changes.
Da Vinci MUST fix these before merging, but they are not regressions from this spec.
```

---

### CHECK 3: No .unwrap() or .expect() in production platform code

```
RESULT: PASS
EVIDENCE:
  Scanned src/platform/detection.rs production section (lines 1-350, before #[cfg(test)]).
  Zero bare .unwrap() or .expect() calls.
  All fallible operations use .unwrap_or(false), .unwrap_or_else(|_| "unknown".into()),
  .ok()?, or match expressions.

  Note: .expect() calls DO exist in other production modules (not governed by spec 0001):
    src/cli/output.rs:36     - .expect("valid spinner template")   [indicatif API, const str]
    src/cli/vault.rs:216     - .expect("env provider always exists") [infallible by design]
    src/cli/apply.rs:451     - .expect("bw has install spec")
    src/cli/apply.rs:663     - .expect("valid regex")             [compile-time const pattern]
    src/config/schema.rs:179 - .expect("valid regex")             [compile-time const pattern]
    src/mcp/mod.rs:80        - .expect("valid regex")             [compile-time const pattern]
  These are outside spec 0001 scope.
```

---

### CHECK 4: command_exists uses which crate, no sh -c or command -v

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:125
    pub fn command_exists(cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }

  $ grep -n "which::which" src/platform/detection.rs
  125:    which::which(cmd).is_ok()

  $ grep -n "sh -c\|command -v\|Command::new.*sh\b" src/platform/detection.rs
  (no output — none present)

  Note: A prior commit (e285e5f) had reverted to sh -c / command -v. Da Vinci's
  spec 0001 implementation correctly restored the which crate per spec requirement.
```

---

### CHECK 5: is_wsl() has THREE checks

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:158-170

  fn is_wsl() -> bool {
      if std::env::var("WSL_DISTRO_NAME").is_ok() {       // Check 1: env var
          return true;
      }
      if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {  // Check 2: file
          return true;
      }
      std::fs::read_to_string("/proc/version")            // Check 3: /proc/version
          .map(|v| v.to_lowercase().contains("microsoft"))
          .unwrap_or(false)
  }

  All three checks present. Mock tests verify each independently (tests 11-14).
```

---

### CHECK 6: is_wsl2() exists and checks ONLY /proc/sys/fs/binfmt_misc/WSLInterop

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:176-178

  fn is_wsl2() -> bool {
      std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
  }

  Single check, correct file path. Mock tests verify true/false cases (tests 15-16).
```

---

### CHECK 7: is_root() uses id -u with #[cfg(unix)] / #[cfg(not(unix))] split

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:240-261

  #[cfg(unix)]
  fn is_root() -> bool {
      std::process::Command::new("id")
          .arg("-u")
          .output()
          .ok()
          .and_then(|o| { ... String::from_utf8(o.stdout).ok() })
          .map(|uid| uid.trim() == "0")
          .unwrap_or(false)
  }

  #[cfg(not(unix))]
  fn is_root() -> bool {
      false
  }

  Correct #[cfg] split. No libc/nix dependency. No unsafe.
```

---

### CHECK 8: detect_shell() has #[cfg] blocks for unix/windows/other

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:264-277

  fn detect_shell() -> String {
      #[cfg(unix)]
      { std::env::var("SHELL").unwrap_or_else(|_| "unknown".into()) }
      #[cfg(windows)]
      { std::env::var("COMSPEC").unwrap_or_else(|_| "unknown".into()) }
      #[cfg(not(any(unix, windows)))]
      { "unknown".into() }
  }

  All three cfg variants present.
```

---

### CHECK 9: PlatformCapabilities has is_wsl2 (not is_wsl) and has_pacman

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:45-55

  pub struct PlatformCapabilities {
      pub has_homebrew: bool,
      pub has_apt: bool,
      pub has_dnf: bool,
      pub has_pacman: bool,   // present
      pub has_snap: bool,
      pub has_systemd: bool,
      pub is_wsl2: bool,      // renamed from is_wsl
      pub has_docker: bool,
  }
```

---

### CHECK 10: Platform derives Serialize, Deserialize but NOT Hash

```
RESULT: PASS
EVIDENCE:
  src/platform/detection.rs:22-23

  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub enum Platform {

  Serialize: present. Deserialize: present. Hash: absent (intentional per spec).
  Roundtrip test (test_platform_serialize_roundtrip) passes.
```

---

### CHECK 11: Mock tests exist — OsProbe trait, _with_probe variants, 14 mock-based tests

```
RESULT: PASS
EVIDENCE:
  OsProbe trait: src/platform/detection.rs:284-289 (#[cfg(test)] only)
  _with_probe functions present:
    is_wsl_with_probe (line 292)
    is_wsl2_with_probe (line 308)
    detect_linux_distro_with_probe (line 313)
    is_root_with_probe (line 336)
    detect_shell_with_probe (line 344)

  Mock-based tests (14 of them, tests 11-24):
    test_wsl_detected_from_env_var ........ line 481
    test_wsl_detected_from_interop_file ... line 490
    test_wsl_detected_from_proc_version ... line 498
    test_not_wsl_when_all_checks_fail ..... line 508
    test_wsl2_true_when_interop_exists .... line 514
    test_wsl2_false_when_interop_missing .. line 523
    test_distro_ubuntu .................... line 529
    test_distro_unknown_on_missing_file ... line 539
    test_distro_quoted_id ................. line 548
    test_distro_arch ...................... line 557
    test_is_root_true ..................... line 566
    test_is_root_false .................... line 572
    test_detect_shell_unix ................ line 580
    test_detect_shell_unset ............... line 587

  Count: 14 mock-based tests. Spec requires 14. EXACT match.
```

---

### CHECK 12: cargo run -- status works without crash

```
RESULT: PASS
EVIDENCE:
  $ cargo run -- status
  great status
  ℹ Platform: WSL Ubuntu 24.04 (X86_64)
  ⚠ No great.toml found. Run `great init` to create one.

  $ cargo run -- status --verbose
  great status
  ℹ Platform: WSL Ubuntu 24.04 (X86_64)
  ℹ Capabilities: homebrew, apt, snap, systemd, docker
  ℹ Shell: /bin/bash
  ℹ Root: false
  ⚠ No great.toml found. Run `great init` to create one.

  No panic, no crash, sensible output.
```

---

### CHECK 13: command_exists("") returns false

```
RESULT: PASS
EVIDENCE:
  Test test_command_exists_empty_string at src/platform/detection.rs:422-425:
    #[test]
    fn test_command_exists_empty_string() {
        assert!(!command_exists(""));
    }
  Test passes. The which crate returns Err for empty string input, so is_ok() returns false.
```

---

## Security Checks (Adversarial)

### No shell injection via command_exists

```
RESULT: PASS
The which crate performs a pure-Rust PATH walk. No shell is spawned.
Passing a string like "ls; rm -rf /" to command_exists() does a PATH lookup
for a file literally named "ls; rm -rf /", which will not be found.
No injection vector exists.
```

### No secrets in detection output

```
RESULT: PASS
PlatformInfo contains: platform (OS type/distro/version/arch), capabilities (bool flags),
is_root (bool), shell (path like /bin/bash). No usernames, no hostnames, no IP addresses,
no environment variable values, no filesystem paths beyond the shell binary.
```

### No unsafe blocks

```
RESULT: PASS
$ grep -n "unsafe" src/platform/detection.rs
(no output)
```

### File permissions: detection.rs not world-writable

```
RESULT: PASS
$ ls -la src/platform/detection.rs
-rw-r--r-- 1 isaac isaac 17853 Feb 20 22:15 src/platform/detection.rs
Mode 644 — no write access for group or others.
```

---

## Edge Case Probes

### ID= with empty value

```
RESULT: PASS (by code inspection)
In detect_linux_distro_with_probe, if ID= has empty value after strip_prefix and trim,
the match arm hits `other => LinuxDistro::Other(other.to_string())` which returns
LinuxDistro::Other(""). Spec says: 'ID= with empty value -> LinuxDistro::Other("")'.
CONFIRMED correct.
```

### Concurrent calls to detect_platform_info()

```
RESULT: PASS (by design)
All detection functions are stateless and read-only. No shared mutable state.
No lazy statics, no mutexes, no global caches. Thread-safe by construction.
```

### Network unavailable

```
RESULT: PASS (by design)
Zero network calls in any detection function. All local filesystem/env/subprocess reads.
```

---

## Issues Requiring Da Vinci Action

### ISSUE 1 — Pre-existing clippy failure: deprecated assert_cmd API

```
SEVERITY: MEDIUM
FILE: tests/cli_smoke.rs:7
ERROR: use of deprecated associated function `assert_cmd::Command::cargo_bin`
INTRODUCED: commit 7721706 (predates spec 0001)
FIX: Replace Command::cargo_bin("great") with cargo::cargo_bin_cmd!("great")
     or suppress with #[allow(deprecated)] if migration is blocked.
BLOCKS: cargo clippy --tests -- -D warnings
```

### ISSUE 2 — Pre-existing clippy failure: const_is_empty in test

```
SEVERITY: LOW
FILE: src/cli/loop_cmd.rs:464
ERROR: clippy::const_is_empty — assert!(!OBSERVER_TEMPLATE.is_empty()) is tautological
INTRODUCED: commit 7f648cf (predates spec 0001)
NOTE: The assertion always passes (template is 785 bytes). It is dead defensive code.
FIX: Remove the redundant assertion, or add #[allow(clippy::const_is_empty)].
BLOCKS: cargo clippy --tests -- -D warnings
```

---

## Final Verdict

**Spec 0001 implementation: PASS**

All 13 checks against spec 0001 requirements pass. The platform detection engine is
correctly implemented:
- `which` crate used for command_exists (pure Rust, no shell spawning)
- Three-tier WSL detection in is_wsl()
- Separate is_wsl2() with single file check
- is_root() correctly split with #[cfg(unix)] / #[cfg(not(unix))]
- detect_shell() with all three cfg variants
- PlatformCapabilities has is_wsl2 and has_pacman
- Platform derives Serialize+Deserialize, not Hash
- 24 unit tests (10 machine-dependent + 14 mock-based) all pass
- OsProbe trait correctly scoped to #[cfg(test)]
- Zero unsafe code, zero shell injection vectors, zero secrets in output

**Two pre-existing clippy issues in unrelated modules must be fixed before the
`cargo clippy --tests -- -D warnings` gate can pass project-wide. These are not
regressions from spec 0001 and should be tracked separately.**

---

*Reported by Alan Turing — "We can only see a short distance ahead, but we can see
plenty there that needs to be done."*
