# Spec Review R2: 0001 Platform Detection Engine

**Reviewer**: Socrates (Adversarial Spec Reviewer)
**Spec**: `.tasks/specs/0001-platform-detection.md` (Revision 2)
**Round**: 2 of 3
**Verdict**: **APPROVE**

---

## Resolution of Original Issues

### B1: `command_exists` implementation unspecified -- Resolved

The spec now specifies the `which` crate (pure-Rust PATH walk), adds `which = "7"` to `Cargo.toml`, documents edge cases (empty cmd, unset PATH, symlinks, Alpine, Windows), and provides the exact implementation (`which::which(cmd).is_ok()`). The current codebase at `/home/isaac/src/sh.great/src/platform/detection.rs:123-143` uses `sh -c "command -v ..."` which the spec replaces with a cleaner, no-shell-spawning approach.

**Status: Resolved.**

### B2: `is_root()` dependency resolved -- Resolved

The spec specifies `id -u` subprocess with `#[cfg(unix)]` / `#[cfg(not(unix))]` split. No `libc`, no `unsafe`, no new crate. The current code at `/home/isaac/src/sh.great/src/platform/detection.rs:248-250` uses `$USER == "root"` which fails in Docker containers without login sessions. The `id -u` approach is strictly better. Edge cases documented (binary not found, non-UTF8, Windows).

**Status: Resolved.**

### B3: WSL detection reconciled with backlog -- Resolved

The spec now implements a three-tier `is_wsl()` check: (1) `WSL_DISTRO_NAME` env var, (2) `/proc/sys/fs/binfmt_misc/WSLInterop` file existence, (3) `/proc/version` contains "microsoft". This matches the backlog requirement. A separate `is_wsl2()` function checks specifically for the `WSLInterop` file. The capability field is renamed from `is_wsl` to `is_wsl2`. WSL1 vs WSL2 distinction is clear: `is_wsl()` catches both, `is_wsl2()` catches only WSL2.

**Status: Resolved.**

### N1: Serialize/Deserialize decision made -- Resolved

`Platform`, `PlatformCapabilities`, and `PlatformInfo` all derive `Serialize, Deserialize`. The spec explicitly documents why `Hash` is NOT derived (fragile with `LinuxDistro::Other(String)`) and recommends `platform.to_string()` as map key. This is a sound decision.

**Status: Resolved.**

### N2: `detect_shell()` fallback specified -- Resolved

The spec documents that `$SHELL` returns the login shell (intentional for config file targeting), adds `$COMSPEC` on Windows, falls back to `"unknown"` when unset. The documented limitation about login shell vs active shell is honest and sufficient. The `#[cfg(not(any(unix, windows)))]` catch-all is a nice touch.

**Status: Resolved.**

### N3: Mock strategy for tests defined -- Resolved

The spec introduces an `OsProbe` trait (`#[cfg(test)]` only) with `read_file`, `env_var`, `path_exists`, and `command_output` methods. Internal functions get `_with_probe` variants. 14 mock-based tests are specified covering WSL detection, distro parsing, root detection, and shell detection. 10 machine-dependent tests cover real-environment sanity checks. This is a solid two-tier testing strategy.

**Status: Resolved.**

### N4: `has_pacman` added -- Resolved

`has_pacman: bool` is now in `PlatformCapabilities`, detected via `command_exists("pacman")`.

**Status: Resolved.**

---

## New Issues Identified in Revision 2

### Observation 1: Current code already exists and diverges from spec

The existing code at `/home/isaac/src/sh.great/src/platform/detection.rs` already implements the basic structure (enriched `Platform` enum with fields, `PlatformCapabilities`, `PlatformInfo`, `command_exists`, etc.). The spec reads as if it is designing from scratch, but the implementer will actually be MODIFYING existing code. The build order at spec lines 311-323 should note this is a modification, not a greenfield build. However, this does not block implementation -- the implementer can read the existing file and apply changes incrementally.

**Severity: Observation only. No action required.**

### Observation 2: `gpu_available` from the backlog is deferred but not acknowledged

The backlog at line 20 lists `gpu_available: bool` in `PlatformCapabilities`. The spec omits it and says "No hardware info in Phase 1" (design decision 4). This is a reasonable deferral, but the spec does not explicitly note the backlog deviation. The backlog should be updated when implementation begins to avoid confusion.

**Severity: Observation only. Backlog owner should note the deferral.**

### Observation 3: `has_dnf` from backlog not originally listed but spec adds it

The backlog's `PlatformCapabilities` lists `has_apt`, `has_snap`, `has_systemd`, `is_wsl2`, `gpu_available` -- but NOT `has_dnf` or `has_docker`. The spec adds both. This is a sensible addition (Fedora uses `dnf`, Docker detection is useful), but is technically scope expansion beyond the backlog. No issue -- just noting for the record.

**Severity: Observation only.**

### Observation 4: Existing `.unwrap_or(false)` in `command_exists`

The current code at `/home/isaac/src/sh.great/src/platform/detection.rs:131` and `:142` uses `.unwrap_or(false)` which is technically `.unwrap_or()` not `.unwrap()` -- this is safe and does not violate the "no `.unwrap()`" convention. The spec's `which::which(cmd).is_ok()` is equivalent and cleaner.

**Severity: Observation only.**

---

## Summary

| # | Issue | Original Severity | Status |
|---|-------|-------------------|--------|
| B1 | `command_exists` implementation | Blocking | Resolved |
| B2 | `is_root()` dependency | Blocking | Resolved |
| B3 | WSL detection vs backlog | Blocking | Resolved |
| N1 | Serialize/Deserialize | Non-blocking | Resolved |
| N2 | `detect_shell()` fallback | Non-blocking | Resolved |
| N3 | Mock strategy for tests | Non-blocking | Resolved |
| N4 | Missing `has_pacman` | Non-blocking | Resolved |

**New blocking issues: 0**
**New non-blocking issues: 0**
**Observations: 4 (informational)**

---

## Verdict: APPROVE

This spec is ready for implementation.

All 7 original issues (3 blocking, 4 non-blocking) have been addressed with specific, implementable solutions. The `which` crate choice for `command_exists` is sound. The `id -u` approach for root detection avoids `unsafe` and new crate dependencies. The three-tier WSL detection with explicit WSL1/WSL2 distinction matches the backlog requirements. The `OsProbe` trait provides a clean testing seam. The serde derives are in place for downstream tasks.

The spec is thorough: 24 tests specified, edge cases tabulated, error handling contract documented (no function returns `Result`, no panics), security considerations addressed, and build order is clear.

One note for the implementer: the existing code at `/home/isaac/src/sh.great/src/platform/detection.rs` already has much of the structure in place. The task is incremental modification, not greenfield. Read the existing file carefully before applying the spec's build order.

*"The unexamined spec is not worth implementing. This one has been examined -- twice -- and stands."*
