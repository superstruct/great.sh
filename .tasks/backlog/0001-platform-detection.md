# 0001: Platform Detection Engine

**Priority:** P0 (foundation -- everything else depends on this)
**Type:** feature
**Module:** `src/platform/`
**Status:** pending

## Context

The current `src/platform/detection.rs` contains a minimal `Platform` enum with five flat variants (MacOS, Linux, Wsl, Windows, Unknown) and basic WSL detection via `/proc/version`. This must be enriched into a full detection engine that downstream modules (config, doctor, apply, init) can rely on for platform-aware behavior.

The legacy repo at `/home/isaac/src/great-sh/src/platform/detection.rs` contains extensive prior art including: rich `Platform` enum with per-variant fields, `Architecture` enum using `std::env::consts::ARCH`, `LinuxDistribution` detection from `/etc/os-release`, `command_exists()` via `which`, `detect_package_managers()`, admin/root detection, and GPU detection via `lspci`. This logic should be consulted and adapted -- not copied wholesale -- to fit the new monorepo's conventions (anyhow for errors, no unwrap, no panics).

## Requirements

1. **Enrich the `Platform` enum** to carry architecture (`Architecture` enum: ARM64, X86_64, Unknown), OS version string, and admin/root status as fields on each variant. The existing `Display` impl in `src/platform/mod.rs` must be updated to match.

2. **Detect all target platforms**: macOS ARM64 (Apple Silicon), macOS x86_64, Ubuntu bare metal, Ubuntu WSL2, other Linux distros (Debian, Fedora, Arch, RHEL at minimum). WSL2 detection must use a dual check: file existence at `/proc/sys/fs/binfmt_misc/WSLInterop` AND the `WSL_DISTRO_NAME` environment variable (either one triggers WSL2). This improves on the legacy check which only used `/proc/version`.

3. **Add a `PlatformCapabilities` struct** returned by a `detect_capabilities()` function with fields: `has_homebrew: bool`, `has_apt: bool`, `has_snap: bool`, `has_systemd: bool`, `is_wsl2: bool`, `gpu_available: bool`. Each capability is detected by probing the filesystem or PATH (e.g., `command_exists("brew")`, checking `/run/systemd/system` for systemd).

4. **Add utility functions**: `command_exists(name: &str) -> bool` (check PATH using `which` on Unix, `where` on Windows), `detect_architecture() -> Architecture` (using `std::env::consts::ARCH`), and `detect_linux_distro() -> LinuxDistribution` (parsing `/etc/os-release` for ID field).

5. **Safety invariant**: All detection functions must be safe -- no `.unwrap()`, no `panic!()`, no `expect()` in production paths. Return `Unknown` / `false` / default values on any IO or parse failure. Use `anyhow::Result` only where the caller needs to distinguish failure from unknown.

## Acceptance Criteria

- [ ] `cargo test` passes with unit tests covering: architecture detection returns a valid variant on the current machine, `command_exists("cargo")` returns true, `command_exists("nonexistent_binary_xyz")` returns false, WSL2 dual-check logic (mock or conditional), and `PlatformCapabilities` struct is populated without panics.
- [ ] `cargo clippy` produces zero warnings for the `platform` module.
- [ ] Running `great doctor` (or a test binary) on the current Linux machine correctly reports the architecture, distro, and capabilities.
- [ ] No `.unwrap()` or `.expect()` calls exist anywhere in `src/platform/`.

## Dependencies

- None (this is the foundation layer).

## Notes

- The legacy repo's `detect_current_platform()` at `/home/isaac/src/great-sh/src/platform/detection.rs:980` is the closest prior art for the enriched platform detection function.
- The legacy `command_exists()` at line 276 uses `which` -- this works on Linux/macOS but consider `std::process::Command::new(name).arg("--version")` as a cross-platform alternative if needed later.
- The existing `src/platform/mod.rs` re-exports `detect_platform` and `Platform` and has a `Display` impl -- this must stay in sync with enum changes.
- GPU detection can be basic for now (`lspci` grep for VGA/3D on Linux, `system_profiler SPDisplaysDataType` on macOS) -- it will be refined in a later iteration.
- Consider adding `#[cfg(test)]` mock helpers for filesystem-dependent detection so tests run in CI without real `/etc/os-release`.
