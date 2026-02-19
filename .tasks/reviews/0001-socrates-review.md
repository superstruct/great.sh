# Spec Review: 0001 Platform Detection Engine

**Reviewer**: Socrates (Adversarial Spec Reviewer)
**Spec**: `.tasks/specs/0001-platform-detection.md`
**Verdict**: **REJECT** -- 7 issues found, 3 of which will cause bugs or require rework if not addressed before implementation.

---

## BLOCKING Issues (must fix before implementation)

### B1. `command_exists` will fail on Windows -- spec says nothing about the implementation

The spec declares `command_exists(cmd: &str) -> bool` as a public API but never specifies HOW it checks the PATH. The legacy repo at `/home/isaac/src/great-sh/src/platform/detection.rs:276` uses `which`, and the legacy utils at `/home/isaac/src/great-sh/src/core/utils.rs:28` has a cross-platform version using `which` on Unix and `where` on Windows.

The spec's silence here is a bug waiting to happen. Question: Does the implementer use `which` (not available on Windows), `where` (not available on Unix), shell out to both with `#[cfg]`, or do a pure-Rust PATH walk? The `which` binary is also not guaranteed on minimal Linux containers (Alpine ships `busybox which` with different exit codes).

**Required resolution**: Specify the implementation strategy. Recommendation from the backlog item 0001 itself: "consider `std::process::Command::new(name).arg("--version")` as a cross-platform alternative" -- but that is also wrong because not all commands accept `--version`. The cleanest approach is a pure-Rust PATH lookup (`std::env::var("PATH")` + iterate + check file exists + executable bit), or use the `which` crate. Pick one and state it.

### B2. `is_root()` via `geteuid()` requires `libc` -- which is not in Cargo.toml

The spec says `is_root() -> bool` checks `geteuid() == 0` on Unix. The current `Cargo.toml` at `/home/isaac/src/sh.great/Cargo.toml` does not include the `libc` crate. The legacy code at `/home/isaac/src/great-sh/src/core/utils.rs:153` uses `unsafe { libc::geteuid() == 0 }`.

Questions:
- Does the spec intend to add `libc` as a dependency? If so, state it.
- Is `unsafe` acceptable here? The project CLAUDE.md says no `.unwrap()` but says nothing about `unsafe`. Should the spec take a position?
- Alternative: `std::process::Command::new("id").arg("-u")` avoids `unsafe` and `libc` but spawns a process. Another alternative: Rust 1.80+ has `std::os::unix::fs::MetadataExt` to check uid, or `nix` crate for safe wrappers. Pick one.

### B3. WSL detection spec contradicts the backlog requirements

The spec at line 104 says WSL detection uses: "`WSL_DISTRO_NAME` env var OR `/proc/version` contains 'microsoft'".

The backlog at `/home/isaac/src/sh.great/.tasks/backlog/0001-platform-detection.md` line 18 says: "WSL2 detection must use a dual check: file existence at `/proc/sys/fs/binfmt_misc/WSLInterop` AND the `WSL_DISTRO_NAME` environment variable (either one triggers WSL2)."

These are different checks:
- The spec uses `/proc/version` contains "microsoft" -- this is the CURRENT code's approach.
- The backlog requires `/proc/sys/fs/binfmt_misc/WSLInterop` file existence -- this is NOT in the spec.
- The backlog explicitly says the new check "improves on the legacy check which only used `/proc/version`."

Why did the spec drop the `/proc/sys/fs/binfmt_misc/WSLInterop` check that the backlog explicitly requested? Is the `/proc/version` approach sufficient? If so, document why the backlog's requirement was overridden. If not, add the WSLInterop check.

Additionally: The spec says `is_wsl` is `bool` but the backlog distinguishes WSL1 vs WSL2 (`is_wsl2: bool` in capabilities). The spec's `PlatformCapabilities` has `is_wsl: bool` (line 79) -- is this WSL-any or WSL2-only? WSL1 vs WSL2 matters because WSL1 lacks `/proc/sys/fs/binfmt_misc/WSLInterop`, lacks real Linux kernel, and has no systemd. Downstream tasks for tool installation will need to know the difference.

---

## NON-BLOCKING Issues (should fix, but implementer can proceed with judgment)

### N1. `Platform` enum loses `Hash` and `Serialize`/`Deserialize` derives

The spec's `Platform` enum at line 46 derives only `Debug, Clone, PartialEq, Eq`. But it contains `LinuxDistro` which has an `Other(String)` variant, and `Architecture` which derives `Hash, Serialize, Deserialize`.

Questions:
- `status.rs` at `/home/isaac/src/sh.great/src/cli/status.rs:21` emits JSON with `platform` as a string via `Display`. If any downstream task (0002 config, 0003 CLI with `--json`) needs to serialize `Platform` or `PlatformInfo` to JSON, the lack of `Serialize` on `Platform` will require rework. Should `Platform` derive `Serialize, Deserialize`?
- `LinuxDistro` derives `Hash` but `Platform` does not. If anyone wants to use `Platform` as a HashMap key (e.g., platform-specific config overrides in task 0002), they cannot. Intentional?

### N2. `detect_shell()` from `$SHELL` is unreliable

The spec says `detect_shell() -> String` reads from `$SHELL` env var. But:
- `$SHELL` is the user's login shell, NOT the currently running shell. A user with `/bin/bash` as login shell running `fish` will report "bash".
- On Windows, `$SHELL` does not exist. What does the function return? The spec does not say.
- In CI environments (GitHub Actions), `$SHELL` may be set to `/bin/bash` even inside a Docker container with a different shell.

Is "login shell" good enough for the CLI's purposes? If so, document that explicitly. What is the fallback when `$SHELL` is unset -- empty string? "unknown"? The return type is `String`, not `Option<String>`, so there is no way to signal "could not detect."

### N3. No `#[cfg(test)]` mock strategy for filesystem-dependent detection

The backlog at line 43 says: "Consider adding `#[cfg(test)]` mock helpers for filesystem-dependent detection so tests run in CI without real `/etc/os-release`."

The spec's test plan at line 143-152 lists 7 unit tests, but every single test depends on the REAL machine state:
- `test_detect_platform` -- "verify it returns Linux or Wsl (not Unknown) on this machine"
- `test_is_wsl_on_linux` -- "may be true or false depending on machine"
- `test_detect_capabilities` -- "verify capabilities struct has reasonable values"

How do you test WSL detection on a non-WSL machine? How do you test macOS detection from a Linux CI runner? How do you test that a bad `/etc/os-release` falls back to `LinuxDistro::Other` instead of panicking?

The spec should either (a) specify a trait-based abstraction for filesystem/env access that tests can mock, or (b) explicitly state that unit tests are machine-dependent and cross-platform correctness is verified only by manual testing or platform-specific CI.

### N4. Missing `has_pacman` in `PlatformCapabilities` -- Arch is in the distro list but not the package manager list

`LinuxDistro` includes `Arch` (line 38), but `PlatformCapabilities` has `has_apt`, `has_dnf`, `has_snap` -- no `has_pacman`. If `great apply` needs to install packages on Arch Linux, there is no capability flag for it. Is Arch supported for install or only for detection?

---

## Observations (no action required)

- The `Architecture` enum correctly derives `Copy`. Good -- it is a fieldless enum and `Copy` avoids unnecessary cloning.
- The `Display` impl change from `Platform::MacOS => ...` to `Platform::MacOS { .. } => ...` is straightforward and backward compatible with the existing `status.rs` consumer.
- The spec wisely keeps `detect_platform()` as the existing public API and adds `detect_platform_info()` as the new rich API. This is good for incremental adoption.

---

## Summary

| # | Issue | Severity | Action |
|---|-------|----------|--------|
| B1 | `command_exists` implementation unspecified; `which` unavailable on Windows/Alpine | Blocking | Specify implementation strategy |
| B2 | `is_root()` needs `libc` not in Cargo.toml; `unsafe` policy unaddressed | Blocking | Add dependency or specify alternative |
| B3 | WSL detection contradicts backlog; WSL1 vs WSL2 not distinguished | Blocking | Reconcile with backlog or document override |
| N1 | Missing `Serialize`/`Deserialize` on `Platform`; `Hash` inconsistency | Non-blocking | Decide now to avoid rework in task 0002/0003 |
| N2 | `$SHELL` is login shell not current shell; undefined on Windows | Non-blocking | Document behavior and fallback |
| N3 | No mock strategy for filesystem-dependent tests | Non-blocking | Specify test approach for CI |
| N4 | Arch in distro list but no `has_pacman` capability | Non-blocking | Add or explain omission |

*I know that I know nothing -- and this spec, in three specific places, also knows less than it thinks it does.*
