# Spec 0001: Platform Detection Engine

**Author**: Ada Lovelace (Spec Writer)
**Status**: Approved
**Module**: `src/platform/`

## Overview

Enrich the minimal `Platform` enum into a full platform detection engine that all downstream commands depend on. The current detection returns only 5 bare enum variants. The new system provides architecture, OS version, Linux distro, and capability flags.

## Design Decisions

1. **Keep `Platform` as simple enum, add `PlatformInfo` wrapper** — backward compatible with existing `Display` impl and `status.rs` usage.
2. **WSL stays first-class** — the current repo correctly treats WSL as its own variant (not a Linux sub-case). This is important because WSL has different capabilities (no systemd in WSL1, different PATH, etc.).
3. **Capabilities are derived, not stored** — `PlatformCapabilities` is computed from runtime checks (`command_exists`, file probes), not hardcoded per-platform.
4. **No hardware info in Phase 1** — CPU/GPU/memory detection is deferred. Only capabilities relevant to package installation and tool management.

## Data Types

### `Architecture` enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}
```
Detect via `std::env::consts::ARCH` — "x86_64" → X86_64, "aarch64" → Aarch64, else Unknown.

### `LinuxDistro` enum
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    Other(String),
}
```
Detect by parsing `/etc/os-release` for `ID=` field.

### `Platform` enum (enriched)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    MacOS {
        version: Option<String>,   // e.g. "15.3.1"
        arch: Architecture,
    },
    Linux {
        distro: LinuxDistro,
        version: Option<String>,   // e.g. "24.04"
        arch: Architecture,
    },
    Wsl {
        distro: LinuxDistro,
        version: Option<String>,
        arch: Architecture,
    },
    Windows {
        arch: Architecture,
    },
    Unknown,
}
```

### `PlatformCapabilities` struct
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub has_homebrew: bool,
    pub has_apt: bool,
    pub has_dnf: bool,
    pub has_snap: bool,
    pub has_systemd: bool,
    pub is_wsl: bool,
    pub has_docker: bool,
}
```
Computed via `command_exists()` and file checks (`/run/systemd/system` for systemd).

### `PlatformInfo` struct (top-level result)
```rust
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub capabilities: PlatformCapabilities,
    pub is_root: bool,
    pub shell: String,
}
```

## Functions

### Public API
- `detect_platform() -> Platform` — existing signature preserved, now returns enriched enum
- `detect_platform_info() -> PlatformInfo` — full detection with capabilities
- `command_exists(cmd: &str) -> bool` — check if command is in PATH
- `detect_architecture() -> Architecture` — from `std::env::consts::ARCH`

### Internal (in detection.rs)
- `is_wsl() -> bool` — dual check: `WSL_DISTRO_NAME` env var OR `/proc/version` contains "microsoft"
- `detect_linux_distro() -> LinuxDistro` — parse `/etc/os-release` ID= field
- `detect_linux_version() -> Option<String>` — parse `/etc/os-release` VERSION_ID= field
- `detect_macos_version() -> Option<String>` — run `sw_vers -productVersion`
- `detect_capabilities(platform: &Platform) -> PlatformCapabilities` — runtime checks
- `is_root() -> bool` — check `geteuid() == 0` on Unix
- `detect_shell() -> String` — from `$SHELL` env var

## File Layout

```
src/platform/
├── mod.rs          — re-exports, Display impl
└── detection.rs    — all detection logic, types
```

Keep it in two files (no splitting yet). detection.rs grows but stays cohesive.

## Display Impl Changes

The `Display` impl in `mod.rs` must be updated for the new enum shape:
```rust
Platform::MacOS { .. } => write!(f, "macos"),
Platform::Linux { .. } => write!(f, "linux"),
// etc.
```

Add a `Platform::display_detailed()` method that shows "macOS 15.3.1 (aarch64)" etc.

## Impact on Existing Code

- `cli/status.rs` calls `platform::detect_platform()` — continues to work (same name, richer return)
- `config/schema.rs` — no change
- `error.rs` — no change
- Tests in `tests/cli_smoke.rs` — no change (they test CLI args, not platform output)

## Test Plan

Unit tests in `detection.rs`:
1. `test_detect_architecture` — verify it returns a non-Unknown value on the test machine
2. `test_detect_platform` — verify it returns Linux or Wsl (not Unknown) on this machine
3. `test_command_exists_positive` — `command_exists("ls")` returns true
4. `test_command_exists_negative` — `command_exists("nonexistent_cmd_xyz")` returns false
5. `test_is_wsl_on_linux` — verify WSL detection logic (may be true or false depending on machine)
6. `test_detect_capabilities` — verify capabilities struct has reasonable values
7. `test_platform_display` — verify Display impl produces expected strings

Integration test (in tests/):
- `great status --verbose` shows platform info (update existing test)
