# 0007: Package Manager Abstraction Layer

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/platform/package_manager.rs` (new file)
**Status:** pending

## Context

The CLI needs to install tools declared in `great.toml` across different operating systems. Each OS family uses different package managers (Homebrew on macOS, apt on Ubuntu/Debian/WSL2, dnf on Fedora/RHEL), and cross-platform tool-specific managers (cargo for Rust tools, npm for Node.js tools) must also be supported. The `PlatformCapabilities` struct in `src/platform/detection.rs` already detects which package managers are available (`has_homebrew`, `has_apt`, `has_dnf`, `has_snap`), but there is no abstraction for actually invoking them.

The legacy repo at `/home/isaac/src/great-sh/` contains prior art for package installation logic that can be consulted but not copied directly -- the new codebase uses `anyhow::Result` and must follow the no-unwrap convention.

This module is a prerequisite for `great apply` (task 0009), which orchestrates the full provisioning pipeline.

## Requirements

1. **Define the `PackageManager` trait**: Create a trait with methods: `fn install(&self, package: &str, version: Option<&str>) -> Result<()>`, `fn is_installed(&self, package: &str) -> bool`, `fn installed_version(&self, package: &str) -> Option<String>`, `fn update(&self, package: &str) -> Result<()>`, and `fn name(&self) -> &str`. All methods that invoke external commands must use `std::process::Command` with proper error propagation (no unwrap, no panic). The trait should be object-safe (`dyn PackageManager`).

2. **Implement `Homebrew` struct**: Implement the trait for Homebrew. The `install` method runs `brew install <package>`. The `is_installed` method runs `brew list --formula <package>` and checks exit code. The `installed_version` method runs `brew info --json=v2 <package>` and parses the version from JSON output (or falls back to `brew list --versions <package>`). Must handle the path difference between Intel (`/usr/local/bin/brew`) and Apple Silicon (`/opt/homebrew/bin/brew`) -- use `command_exists("brew")` to find it on PATH regardless.

3. **Implement `Apt` struct**: Implement the trait for apt (Ubuntu, Debian, WSL2). The `install` method runs `sudo apt-get install -y <package>`. The `is_installed` method runs `dpkg -s <package>` and checks exit code. The `installed_version` method parses the `Version:` field from `dpkg -s` output. Must handle the case where `sudo` requires a password in interactive mode and fail gracefully in non-interactive mode with a clear error message.

4. **Implement `Cargo` struct**: Implement the trait for `cargo install`. The `is_installed` method uses `command_exists()` from the platform module. The `installed_version` method runs `<package> --version` and parses the output. The `install` method runs `cargo install <package>` with an optional `--version` flag.

5. **Implement `Npm` struct**: Implement the trait for npm global installs. The `install` method runs `npm install -g <package>`. The `is_installed` method uses `command_exists()`. The `installed_version` method runs `npm list -g <package> --json` and parses the version. Must handle the case where npm is not installed (return a clear error, not a panic).

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/platform/package_manager.rs`.
- [ ] The `PackageManager` trait compiles and is object-safe (can be used as `Box<dyn PackageManager>`).
- [ ] Unit tests verify: `Homebrew::is_installed("nonexistent_xyz")` returns false (on a machine without that package), `Cargo::is_installed("cargo")` returns true (since cargo is present), and all implementations return `Err` (not panic) when the underlying package manager is not available.
- [ ] No `.unwrap()` or `.expect()` calls exist in `src/platform/package_manager.rs`.
- [ ] The module is re-exported from `src/platform/mod.rs` and accessible as `crate::platform::package_manager::*`.

## Dependencies

- Task 0001 (platform detection) -- already landed; `command_exists()`, `PlatformCapabilities`, and platform detection are available for determining which package managers exist on the system.

## Notes

- **Idempotency is critical**: The `install` method should check `is_installed` first and skip if the package is already present at the correct version. This makes `great apply` safe to re-run.
- The trait is intentionally simple (one package at a time) to keep error handling clear. Batch installation can be optimized later by running `brew install pkg1 pkg2 ...` in a single invocation.
- The `version` parameter in `install` is `Option<&str>` because some package managers (apt) do not support version pinning easily, while others (cargo, npm) do. Implementations should install the latest if `None` is passed.
- Consider adding a `detect_package_manager(platform: &Platform) -> Vec<Box<dyn PackageManager>>` factory function that returns the appropriate implementations for the current platform. This is useful for `great apply` to iterate over available managers.
- The `sudo` requirement for `apt` is a UX concern. In non-interactive mode (`--non-interactive` global flag from task 0003), the command should fail with a message like "apt requires sudo -- run with interactive mode or use sudo". This integrates with the `Context` struct from the CLI infrastructure.
