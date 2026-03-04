# Socrates Review -- Spec 0039: Docker-on-WSL2 Container Falsely Detected as WSL

**Date:** 2026-03-04
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `.tasks/ready/0039-spec.md`
**Backlog:** `.tasks/backlog/0039-docker-on-wsl2-falsely-detected-as-wsl.md`
**Round:** 1

---

## VERDICT: APPROVED

---

## Analysis

### 1. Trait Pattern Compliance (`OsProbe` / `MockProbe`)

The spec correctly uses the existing `OsProbe` trait (line 295 of `detection.rs`) and `MockProbe` struct (line 368). No new trait methods are required -- `path_exists`, `env_var`, and `read_file` are already defined. The new `is_container_with_probe()` function follows the established `_with_probe` pattern used by `is_wsl_with_probe()`, `is_wsl2_with_probe()`, `is_root_with_probe()`, and `detect_shell_with_probe()`. Each has its own `#[cfg(test)]` attribute rather than being nested inside `mod tests`, which the spec correctly replicates.

The production `is_container()` function (section 3.1) uses `std::path::Path::exists()`, `std::env::var().is_ok()`, and `std::fs::read_to_string()` -- all matching the patterns already used by `is_wsl()` and `is_wsl2()`. Correct.

### 2. Container Guard Placement

Verified: the guard is inserted as the **first statement** in all four functions:
- `is_wsl()` (section 3.2) -- guard before `WSL_DISTRO_NAME` check
- `is_wsl2()` (section 3.3) -- guard before `WSLInterop` check
- `is_wsl_with_probe()` (section 3.5) -- guard before `WSL_DISTRO_NAME` check
- `is_wsl2_with_probe()` (section 3.6) -- guard before `WSLInterop` check

This ordering is correct. The container check MUST precede all WSL probes to avoid the false positive. Confirmed.

### 3. Container Indicator Priority Order

The four indicators are ordered by reliability:
1. `/.dockerenv` -- Docker sentinel file (most reliable)
2. `DOCKER_CONTAINER` env var -- set by some compose/base images
3. `container` env var (lowercase) -- OCI standard (Podman, systemd-nspawn, LXC)
4. `/proc/1/cgroup` contains "docker" -- cgroup v1 fallback

This ordering is sound. Short-circuit evaluation means the most reliable indicator is checked first. The cgroup fallback is correctly last because it fails on cgroup v2 systems (which show only `0::/`).

### 4. Edge Case Coverage

The spec's edge case table (section 6) is thorough and covers:
- Empty `/.dockerenv` file (path_exists is boolean, content irrelevant)
- Empty `container` env var (`env_var().is_ok()` / `.is_some()` returns true)
- Unreadable `/proc/1/cgroup` (graceful fallback via `unwrap_or(false)`)
- Cgroup v2 (no "docker" string; covered by `/.dockerenv` and env vars)
- LXC / systemd-nspawn (covered by `container` env var)
- Podman rootless (covered by `container` env var)
- Genuine WSL2 with Docker installed but not inside a container (host PID 1 cgroup does not contain "docker", no `/.dockerenv`)

### 5. Production Functions Coverage

The spec modifies BOTH production and test variants:
- Production: `is_wsl()` (line 169), `is_wsl2()` (line 187), new `is_container()` (inserted after line 189)
- Test: `is_wsl_with_probe()` (line 303), `is_wsl2_with_probe()` (line 319), new `is_container_with_probe()` (inserted after line 321)

The call chain is complete:
- `detect_platform()` (line 94) calls `is_wsl()` -- guarded
- `detect_capabilities()` (line 157) calls `is_wsl2()` -- guarded

No other callers of `is_wsl()` or `is_wsl2()` exist in the codebase (verified via grep).

### 6. Test Sufficiency (12 new tests)

The 12 proposed tests cover:

| Test | What it validates | AC |
|------|-------------------|-----|
| 25 | `is_container_with_probe` true from `/.dockerenv` | -- |
| 26 | `is_container_with_probe` true from `DOCKER_CONTAINER` env | -- |
| 27 | `is_container_with_probe` true from `container` env | -- |
| 28 | `is_container_with_probe` true from cgroup | -- |
| 29 | `is_container_with_probe` false when no indicators | -- |
| 30 | `is_wsl_with_probe` false when dockerenv + WSL indicators | AC1 |
| 31 | `is_wsl_with_probe` false when container env + all 3 WSL tiers | AC2 |
| 32 | `is_wsl_with_probe` false when DOCKER_CONTAINER env + WSL indicators | AC2 |
| 33 | `is_wsl_with_probe` true on genuine WSL2 (no container) | AC3 |
| 34 | `is_wsl2_with_probe` false when dockerenv + WSLInterop | AC4 |
| 35 | `is_wsl2_with_probe` true on genuine WSL2 (no container) | -- |
| 36 | `is_wsl_with_probe` false when cgroup-only container + WSL indicators | -- |

All 5 acceptance criteria are mapped to at least one test. The negative case (genuine WSL2, no container) is explicitly covered by tests 33 and 35.

### 7. Existing Test Regression Risk

Verified all 14 existing mock-based tests (lines 491-601). None of them insert:
- `/.dockerenv` into `paths`
- `DOCKER_CONTAINER` or `container` into `env_vars`
- `/proc/1/cgroup` into `files`

Therefore the container guard will evaluate `false` in all existing mock tests, and their behavior is unchanged. Confirmed.

For the 10 machine-dependent tests (lines 409-485): if CI runs inside a Docker container, `is_container()` returns `true`, causing `is_wsl()` and `is_wsl2()` to return `false`. The `test_detect_capabilities` test (line 468) checks `if !matches!(platform, Platform::Wsl { .. })` then asserts `!caps.is_wsl2` -- this passes because both platform and capabilities are consistent. No regression.

---

## Concerns

```
{
  "gap": "Kubernetes pods using containerd (not Docker) may lack all four indicators: no /.dockerenv, no DOCKER_CONTAINER env, container env var may or may not be set, and /proc/1/cgroup contains 'kubepods' not 'docker'.",
  "question": "Should the cgroup check also match 'kubepods' or 'containerd' in addition to 'docker'?",
  "severity": "ADVISORY",
  "recommendation": "The backlog explicitly scopes out exotic container runtimes, and Kubernetes-on-WSL2 is a very niche scenario. Document this as a known limitation in a code comment near the cgroup check. No code change required for this task."
}
```

```
{
  "gap": "The spec says is_container_with_probe() should be placed 'inside the #[cfg(test)] block' (section 3.4), but the _with_probe functions are not inside a single block -- they are standalone functions each with their own #[cfg(test)] attribute.",
  "question": "Could the phrasing 'inside the #[cfg(test)] block' confuse the builder into placing the function inside mod tests rather than among the other _with_probe functions?",
  "severity": "ADVISORY",
  "recommendation": "The code sample in section 3.4 shows the correct #[cfg(test)] annotation on the function, and the insertion point ('after is_wsl2_with_probe, after line 321') is unambiguous. A competent builder will follow the code sample, not the prose. No change needed."
}
```

```
{
  "gap": "The production is_container() function is private (fn, not pub fn) and only called from is_wsl() and is_wsl2(). If a future module needs container detection (e.g., doctor.rs adding container-specific checks), it would need to be promoted.",
  "question": "Should is_container() be pub(crate) from the start to avoid a future change?",
  "severity": "ADVISORY",
  "recommendation": "The spec correctly keeps scope minimal. The backlog says promoting to public API is out of scope. Private is correct for now; promoting is a one-line change if needed later."
}
```

---

## Line Number Verification

All line references in the spec were verified against the actual file `/home/isaac/src/sh.great/src/platform/detection.rs` (604 lines):

| Spec claim | Actual | Match |
|------------|--------|-------|
| `OsProbe` trait at line 295 | Line 294-300 | Yes (295 is the trait keyword line) |
| `is_wsl()` at lines 169-181 | Lines 169-181 | Yes |
| `is_wsl2()` at lines 187-189 | Lines 187-189 | Yes |
| `is_wsl_with_probe()` at lines 303-316 | Lines 303-316 | Yes |
| `is_wsl2_with_probe()` at lines 319-321 | Lines 319-321 | Yes |
| `MockProbe` at line 368 | Lines 368-384 | Yes |
| 10 machine-dependent tests | Lines 409-485 (10 `#[test]`) | Yes |
| 14 mock-based tests | Lines 491-601 (14 `#[test]`) | Yes |
| 24 existing tests total | 24 `#[test]` attributes counted | Yes |

---

## Summary

Clean, well-structured bugfix spec that correctly follows the established `OsProbe`/`MockProbe` testing pattern, guards both production and test code paths, places the container check before all WSL probes, and maps every acceptance criterion to a named test. All three concerns are ADVISORY. Implementable without further clarification.
