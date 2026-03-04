# Nightingale Selection Report — Task 0039

**Date:** 2026-03-04
**Agent:** Florence Nightingale (Requirements Curator)
**Next agent:** Ada Lovelace (Specification)

---

## 1. Task Selected and Why

**Task:** 0039 — Docker-on-WSL2 container falsely detected as WSL
**Priority:** P2 (bugfix)
**Module:** `src/platform/detection.rs`

Selected because:

- It is the only P2 in the backlog. Task 0040 (exit code inconsistency) is blocked on a design decision. Task 0041 (mcp test error message) is P3 cosmetic.
- The root cause is precisely identified: `is_wsl()` and `is_wsl2()` use two filesystem probes (`/proc/sys/fs/binfmt_misc/WSLInterop` and `/proc/version`) that are inherited from the WSL2 host kernel by Docker containers. Both return false positives inside containers.
- The fix is self-contained: one file, two functions, no dependency changes.
- Impact is real and user-facing: `great apply` inside a Docker container on a WSL2 host may attempt to copy fonts to `C:\Users\...`, invoke `cmd.exe`, or generate wrong install paths — all of which fail.
- The existing `OsProbe` / `MockProbe` test infrastructure is already in place. New test scenarios slot in without structural changes.

---

## 2. Requirements Summary

The fix must add a container-detection guard that runs before any WSL conclusion is drawn. The guard checks four indicators in priority order: `/.dockerenv` (most reliable, Docker always creates it), `DOCKER_CONTAINER` env var (secondary, set by some base images), `container` env var (OCI runtimes including Podman), and `/proc/1/cgroup` content (v1 cgroup fallback). If any indicator is found, the environment is classified as a container, not WSL.

### Acceptance Criteria

- [ ] AC1: `is_wsl_with_probe()` returns `false` when `/.dockerenv` exists, even when `/proc/sys/fs/binfmt_misc/WSLInterop` is also present and `/proc/version` contains "microsoft".
- [ ] AC2: `is_wsl_with_probe()` returns `false` when the `container` env var or `DOCKER_CONTAINER` env var is set, regardless of all other WSL indicators.
- [ ] AC3: `is_wsl_with_probe()` continues to return `true` on a genuine WSL2 environment — WSLInterop present, `/proc/version` contains "microsoft", and no container indicators are present.
- [ ] AC4: `is_wsl2_with_probe()` applies the same container-exclusion guard and returns `false` when any container indicator is detected (even if WSLInterop exists).
- [ ] AC5: `great status` run inside an `ubuntu:24.04` Docker container on a WSL2 host prints `Platform: Linux Ubuntu 24.04 (x86_64)`, not `WSL Ubuntu 24.04 (x86_64)`.

---

## 3. Key Files

### Primary change target

**`/home/isaac/src/sh.great/src/platform/detection.rs`**

Lines of interest:

- Line 94: `if is_wsl()` — the call site in `detect_platform()`. No change needed here; the fix goes inside `is_wsl()`.
- Lines 169–181: `fn is_wsl()` — add container guard at the top, before any WSL probes. If `is_container()` returns `true`, return `false` immediately.
- Lines 187–189: `fn is_wsl2()` — same container guard required.
- Lines 303–316: `fn is_wsl_with_probe()` — same guard, using `probe.path_exists("/.dockerenv")`, `probe.env_var("DOCKER_CONTAINER")`, `probe.env_var("container")`, and `probe.read_file("/proc/1/cgroup")`. Guard runs before all existing checks.
- Lines 319–321: `fn is_wsl2_with_probe()` — same guard.
- Lines 405–537: existing mock-based tests — add four new test functions covering AC1, AC2, AC4 (AC3 is already covered by `test_wsl_detected_from_interop_file` with no container indicators set; a clean positive-WSL test with explicit empty container state is still worth adding for clarity).

### New helper to add

A private `is_container()` function (and a `is_container_with_probe()` variant for tests) that checks the four indicators. Suggested signature:

```rust
fn is_container() -> bool {
    std::path::Path::new("/.dockerenv").exists()
        || std::env::var("DOCKER_CONTAINER").is_ok()
        || std::env::var("container").is_ok()
        || std::fs::read_to_string("/proc/1/cgroup")
            .map(|c| c.contains("docker"))
            .unwrap_or(false)
}
```

The probe variant passes all four checks through `probe.path_exists` / `probe.env_var` / `probe.read_file`.

### No other files require changes

`src/cli/apply.rs`, `src/cli/doctor.rs`, `src/cli/status.rs` all consume `Platform::Wsl` / `PlatformCapabilities.is_wsl2` downstream. Fixing detection fixes them without touching those files.

---

## 4. Confirmation Ready for Lovelace

This task is ready for specification. The scope is tight:

- One Rust source file.
- Two production functions to patch (`is_wsl`, `is_wsl2`).
- One new private helper (`is_container`).
- Two testable probe variants to update (`is_wsl_with_probe`, `is_wsl2_with_probe`).
- One new probe helper (`is_container_with_probe`).
- Four new unit tests (MockProbe-based, no I/O).
- No new dependencies. No public API changes.

Lovelace should write a specification that covers the exact insertion point for the container guard in each function, the order of the four container indicator checks, and the mapping from each acceptance criterion to a named test function.

The `OsProbe` trait already has all methods needed (`path_exists`, `env_var`, `read_file`). No trait extension is required.
