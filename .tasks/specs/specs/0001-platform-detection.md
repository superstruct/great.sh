# Spec 0001: Platform Detection Engine (Revision 2)

**Author**: Ada Lovelace (Spec Writer)
**Status**: Revised (addressing Socrates review)
**Module**: `src/platform/`
**Revision history**: R1 rejected by Socrates (7 issues). R2 addresses all 7.

## Overview

Enrich the minimal `Platform` enum into a full platform detection engine that all downstream commands depend on. The current detection returns only 5 bare enum variants. The new system provides architecture, OS version, Linux distro, and capability flags.

## Design Decisions

1. **Keep `Platform` as simple enum, add `PlatformInfo` wrapper** -- backward compatible with existing `Display` impl and `status.rs` usage.
2. **WSL stays first-class** -- the current repo correctly treats WSL as its own variant (not a Linux sub-case). WSL2 is specifically distinguished from WSL1 via `/proc/sys/fs/binfmt_misc/WSLInterop` file existence.
3. **Capabilities are derived, not stored** -- `PlatformCapabilities` is computed from runtime checks (`command_exists`, file probes), not hardcoded per-platform.
4. **No hardware info in Phase 1** -- CPU/GPU/memory detection is deferred. Only capabilities relevant to package installation and tool management.
5. **`which` crate for command lookups** -- pure-Rust, cross-platform PATH resolution. No shell spawning for command existence checks. (Addresses B1.)
6. **Root detection via `id -u`** -- avoids adding `libc` or `nix` crate. Shells out to `id -u` on Unix, returns `false` on Windows. (Addresses B2.)
7. **WSL detection uses `WSL_DISTRO_NAME` OR `/proc/sys/fs/binfmt_misc/WSLInterop`** -- per backlog requirements. WSLInterop file presence specifically indicates WSL2. The `/proc/version` check is retained as a tertiary fallback. (Addresses B3.)
8. **`Platform` derives `Serialize, Deserialize`** -- needed by downstream tasks 0002 (config) and 0003 (CLI `--json` output). `Hash` is NOT derived because `LinuxDistro::Other(String)` makes it fragile as a map key; use `Platform::to_string()` as key instead. (Addresses N1.)
9. **`detect_shell()` returns login shell with documented fallbacks** -- `$SHELL` on Unix (login shell, which is sufficient for config file generation), `$COMSPEC` on Windows, `"unknown"` when neither is set. (Addresses N2.)
10. **Filesystem-dependent tests use a trait-based abstraction** -- `OsProbe` trait allows injection of mock filesystem/env state in tests. Live tests run against real machine state and are gated by platform. (Addresses N3.)
11. **`has_pacman` added to `PlatformCapabilities`** -- Arch Linux is a supported distro, so its package manager must be detectable. (Addresses N4.)

## Dependency Changes to `Cargo.toml`

Add to `[dependencies]`:
```toml
which = "7"
```

No other new dependencies. The `which` crate is pure Rust with no transitive C dependencies.

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
Detection: `std::env::consts::ARCH` -- `"x86_64"` maps to `X86_64`, `"aarch64"` maps to `Aarch64`, all others map to `Unknown`.

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
Detection: parse `/etc/os-release` for `ID=` field. Strip quotes and whitespace. Map `"ubuntu"` to `Ubuntu`, `"debian"` to `Debian`, `"fedora"` to `Fedora`, `"arch"` to `Arch`, anything else to `Other(id.to_string())`. If `/etc/os-release` cannot be read, return `Other("unknown".into())`.

### `Platform` enum
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

Note: `Serialize` and `Deserialize` are derived (change from R1). `Hash` is intentionally NOT derived -- `LinuxDistro::Other(String)` implements `Hash` via `String::hash`, but using `Platform` as a HashMap key is fragile because `Other("ubuntu")` and `Ubuntu` would hash differently. Downstream code that needs platform-keyed maps should use `platform.to_string()` (the `Display` output: `"macos"`, `"linux"`, `"wsl"`, `"windows"`, `"unknown"`).

### `PlatformCapabilities` struct
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub has_homebrew: bool,
    pub has_apt: bool,
    pub has_dnf: bool,
    pub has_pacman: bool,
    pub has_snap: bool,
    pub has_systemd: bool,
    pub is_wsl2: bool,
    pub has_docker: bool,
}
```

Changes from R1:
- Added `has_pacman: bool` (N4).
- Renamed `is_wsl` to `is_wsl2` (B3). This is `true` only when running inside WSL2 specifically, not WSL1. Detection: `/proc/sys/fs/binfmt_misc/WSLInterop` file exists.
- Added `Serialize, Deserialize` derives (for consistency with `Platform`).

### `PlatformInfo` struct (top-level result)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub capabilities: PlatformCapabilities,
    pub is_root: bool,
    pub shell: String,
}
```

Added `Serialize, Deserialize` derives for downstream JSON output.

## Functions

### Public API

#### `detect_platform() -> Platform`
Existing signature preserved. Returns enriched enum. Detection order:

1. `cfg!(target_os = "macos")`: return `Platform::MacOS { version: detect_macos_version(), arch: detect_architecture() }`.
2. `cfg!(target_os = "linux")`: detect arch, distro, version. Then check `is_wsl()` -- if true, return `Platform::Wsl { .. }`, else return `Platform::Linux { .. }`.
3. `cfg!(target_os = "windows")`: return `Platform::Windows { arch: detect_architecture() }`.
4. Otherwise: `Platform::Unknown`.

#### `detect_platform_info() -> PlatformInfo`
Full detection with capabilities, root status, and shell. Calls `detect_platform()`, then `detect_capabilities()`, `is_root()`, `detect_shell()`.

#### `command_exists(cmd: &str) -> bool`
Check if a command is available on `$PATH` using the `which` crate.

**Implementation**:
```rust
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}
```

The `which` crate performs a pure-Rust PATH walk: reads `$PATH` (or `%PATH%` on Windows), splits by OS separator, checks each directory for a file named `cmd` with executable permission (Unix) or matching `%PATHEXT%` extensions (Windows). No shell spawning. No dependency on external `which` or `where` binaries.

**Edge cases**:
- Empty `cmd`: returns `false` (which crate returns `Err`).
- `$PATH` unset: returns `false`.
- Symlinks: resolved by `which` crate, works correctly.
- Alpine Linux / minimal containers: works (no external binary needed).
- Windows: searches `%PATH%` using `%PATHEXT%` extensions (`.exe`, `.cmd`, `.bat`, etc.).

#### `detect_architecture() -> Architecture`
From `std::env::consts::ARCH`. Pure, no I/O.

### Internal Functions (in detection.rs)

#### `is_wsl() -> bool`
Returns `true` if running inside any version of WSL (1 or 2).

**Detection strategy** (three checks, any one sufficient):
1. `std::env::var("WSL_DISTRO_NAME").is_ok()` -- fast path, no I/O.
2. `Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()` -- present in WSL2 and some WSL1 builds.
3. `std::fs::read_to_string("/proc/version")` contains `"microsoft"` (case-insensitive) -- fallback for edge cases where env var is stripped.

Returns `true` if ANY check passes. Returns `false` if all three fail or if not on Linux (`cfg!(target_os = "linux")` guard).

#### `is_wsl2() -> bool`
Returns `true` only if running inside WSL2 (not WSL1).

**Detection**: `/proc/sys/fs/binfmt_misc/WSLInterop` file exists. This file is present in WSL2 but absent in WSL1. Called by `detect_capabilities()` to populate `PlatformCapabilities::is_wsl2`.

#### `detect_linux_distro() -> LinuxDistro`
Parse `/etc/os-release` for the `ID=` line. Strip quotes and whitespace.

**Mapping**: `"ubuntu"` -> `Ubuntu`, `"debian"` -> `Debian`, `"fedora"` -> `Fedora`, `"arch"` -> `Arch`, anything else -> `Other(id.to_string())`.

**Failure**: if `/etc/os-release` does not exist or cannot be read, return `LinuxDistro::Other("unknown".into())`. No panics.

#### `detect_linux_version() -> Option<String>`
Parse `/etc/os-release` for `VERSION_ID=` line. Strip quotes. Return `None` if not found, empty, or file unreadable.

#### `detect_macos_version() -> Option<String>`
Run `sw_vers -productVersion`. Return `None` if command fails, output is empty, or not on macOS.

#### `detect_capabilities(platform: &Platform) -> PlatformCapabilities`
Runtime checks for available tools and services.

```rust
pub fn detect_capabilities(platform: &Platform) -> PlatformCapabilities {
    PlatformCapabilities {
        has_homebrew: command_exists("brew"),
        has_apt: command_exists("apt"),
        has_dnf: command_exists("dnf"),
        has_pacman: command_exists("pacman"),
        has_snap: command_exists("snap"),
        has_systemd: Path::new("/run/systemd/system").exists(),
        is_wsl2: is_wsl2(),
        has_docker: command_exists("docker"),
    }
}
```

Note: `is_wsl2` is computed independently of the `Platform::Wsl` variant. This allows detecting WSL2 even if the platform variant is `Linux` (defensive). In practice, if `is_wsl2()` returns `true`, the platform variant should already be `Wsl`.

#### `is_root() -> bool`
Check if current user has root/admin privileges.

**Implementation**:
```rust
#[cfg(unix)]
fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|uid| uid.trim() == "0")
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_root() -> bool {
    false
}
```

**Rationale**: Using `id -u` avoids adding the `nix` or `libc` crate as a dependency for a single function call. The `id` command is mandated by POSIX and available on all Unix systems including Alpine Linux, macOS, and WSL. It correctly detects root even when `$USER` is not set (e.g., in Docker containers running as root without a login session, or after `su` without `-l`).

**Edge cases**:
- `id` binary not found (extremely unlikely on any Unix): returns `false`.
- Non-UTF8 output: returns `false`.
- Windows: always `false` (admin detection is out of scope for Phase 1).

#### `detect_shell() -> String`
Return the user's shell.

**Implementation**:
```rust
fn detect_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "unknown".into())
    }
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "unknown".into())
    }
    #[cfg(not(any(unix, windows)))]
    {
        "unknown".into()
    }
}
```

**Documented limitation**: On Unix, `$SHELL` returns the user's login shell (from `/etc/passwd`), NOT the currently running shell. A user whose login shell is `bash` but who launched `fish` will see `"bash"`. This is intentional and sufficient for the CLI's purposes: `$SHELL` answers "what shell should config files target?" which is the login shell. If a future task needs the active shell, it should inspect `/proc/self/exe` -> parent pid -> `/proc/<ppid>/exe` on Linux or `$0` inspection.

**Edge cases**:
- `$SHELL` unset (rare on Unix, common in some Docker images): returns `"unknown"`.
- Windows: uses `$COMSPEC` (typically `C:\Windows\system32\cmd.exe`). PowerShell does not set `$COMSPEC` differently, so this returns the system default. Returns `"unknown"` if `$COMSPEC` is unset.
- CI environments: `$SHELL` may be `/bin/bash` regardless of actual runner shell. This is acceptable.

## File Layout

```
src/platform/
    mod.rs          -- re-exports, Display impl, display_detailed(), arch() method
    detection.rs    -- all detection logic, types, OsProbe trait, tests
    package_manager.rs  -- (existing, unchanged)
    runtime.rs          -- (existing, unchanged)
```

Keep detection in two files (no splitting yet). `detection.rs` grows but stays cohesive.

## Display Impl (in mod.rs)

The `Display` impl must be updated for the new enum shape (already done in current code):
```rust
Platform::MacOS { .. } => write!(f, "macos"),
Platform::Linux { .. } => write!(f, "linux"),
Platform::Wsl { .. } => write!(f, "wsl"),
Platform::Windows { .. } => write!(f, "windows"),
Platform::Unknown => write!(f, "unknown"),
```

The `display_detailed()` method returns human-readable strings:
- `"macOS 15.3.1 (Aarch64)"`
- `"Linux Ubuntu 24.04 (X86_64)"`
- `"WSL Ubuntu 24.04 (X86_64)"`
- `"Windows (X86_64)"`
- `"Unknown"`

The `arch()` method extracts the `Architecture` from any variant. Returns `Architecture::Unknown` for `Platform::Unknown`.

## Impact on Existing Code

- `cli/status.rs` calls `platform::detect_platform()` -- continues to work (same name, richer return).
- `config/schema.rs` -- no change.
- `error.rs` -- no change.
- Tests in `tests/cli_smoke.rs` -- no change (they test CLI args, not platform output).
- `mod.rs` re-exports: add `command_exists`, `detect_architecture`, `detect_platform_info`, `Architecture`, `LinuxDistro`, `PlatformCapabilities`, `PlatformInfo` to public API.

## Build Order

1. Add `which = "7"` to `Cargo.toml`.
2. Update `detection.rs` data types: add `Serialize`/`Deserialize` to `Platform` and `PlatformCapabilities`, add `has_pacman` field, rename `is_wsl` to `is_wsl2`.
3. Update `command_exists()` to use `which::which()`.
4. Update `is_wsl()` to add `/proc/sys/fs/binfmt_misc/WSLInterop` check (between env var and `/proc/version` checks).
5. Add `is_wsl2()` function.
6. Update `is_root()` to use `id -u` with `#[cfg(unix)]` / `#[cfg(not(unix))]` split.
7. Update `detect_shell()` with `#[cfg]` blocks and `$COMSPEC` fallback.
8. Update `detect_capabilities()` to add `has_pacman` and use `is_wsl2()`.
9. Add `OsProbe` trait and mock-based tests.
10. Run `cargo clippy` and `cargo test`.

## Testing Strategy

### Approach: Trait-based abstraction for mockable tests (N3)

Define an `OsProbe` trait that abstracts filesystem and environment access:

```rust
#[cfg(test)]
trait OsProbe {
    fn read_file(&self, path: &str) -> Option<String>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn path_exists(&self, path: &str) -> bool;
    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String>;
}
```

The trait is `#[cfg(test)]` only -- production code calls the real filesystem/env directly. Test helpers create mock `OsProbe` implementations.

Internal detection functions have `_with_probe` variants used by tests:
```rust
fn is_wsl_with_probe(probe: &dyn OsProbe) -> bool { ... }
fn detect_linux_distro_with_probe(probe: &dyn OsProbe) -> LinuxDistro { ... }
```

The public functions (`is_wsl()`, `detect_linux_distro()`) call the `_with_probe` variants with a real probe (direct fs/env calls).

### Unit tests (in detection.rs `#[cfg(test)] mod tests`)

**Machine-dependent tests** (run on any platform, assert reasonable values):
1. `test_detect_architecture` -- returns non-`Unknown` on any real machine.
2. `test_detect_platform_not_unknown` -- returns a valid variant, not `Unknown`.
3. `test_command_exists_positive` -- `command_exists("ls")` on Unix, `command_exists("cmd")` on Windows.
4. `test_command_exists_negative` -- `command_exists("nonexistent_command_xyz_12345")` returns `false`.
5. `test_command_exists_empty_string` -- `command_exists("")` returns `false`.
6. `test_platform_display` -- `format!("{}", platform)` is one of the 5 known strings.
7. `test_platform_display_detailed` -- `display_detailed()` is non-empty and contains architecture.
8. `test_detect_platform_info` -- shell is non-empty, capabilities struct populated without panic.
9. `test_detect_capabilities` -- `is_wsl2` matches whether platform is `Wsl` variant (on this machine).
10. `test_platform_serialize_roundtrip` -- `serde_json::to_string` then `from_str` round-trips a `Platform` value.

**Mock-based tests** (run everywhere, test logic not environment):
11. `test_wsl_detected_from_env_var` -- mock `WSL_DISTRO_NAME=Ubuntu`, no files. `is_wsl_with_probe` returns `true`.
12. `test_wsl_detected_from_interop_file` -- mock no env var, `/proc/sys/fs/binfmt_misc/WSLInterop` exists. `is_wsl_with_probe` returns `true`.
13. `test_wsl_detected_from_proc_version` -- mock no env var, no interop file, `/proc/version` contains "microsoft". `is_wsl_with_probe` returns `true`.
14. `test_not_wsl_when_all_checks_fail` -- mock no env var, no files. `is_wsl_with_probe` returns `false`.
15. `test_wsl2_true_when_interop_exists` -- mock interop file exists. `is_wsl2_with_probe` returns `true`.
16. `test_wsl2_false_when_interop_missing` -- mock interop file does not exist. Returns `false` (WSL1 or non-WSL).
17. `test_distro_ubuntu` -- mock `/etc/os-release` with `ID=ubuntu`. Returns `LinuxDistro::Ubuntu`.
18. `test_distro_unknown_on_missing_file` -- mock file read failure. Returns `LinuxDistro::Other("unknown")`.
19. `test_distro_quoted_id` -- mock `ID="debian"` (with quotes). Returns `LinuxDistro::Debian`.
20. `test_distro_arch` -- mock `ID=arch`. Returns `LinuxDistro::Arch`.
21. `test_is_root_true` -- mock `id -u` returns "0". `is_root` returns `true`.
22. `test_is_root_false` -- mock `id -u` returns "1000". `is_root` returns `false`.
23. `test_detect_shell_unix` -- mock `$SHELL=/bin/zsh`. Returns `"/bin/zsh"`.
24. `test_detect_shell_unset` -- mock no `$SHELL`. Returns `"unknown"`.

### Integration test (in `tests/`)

25. `great status --verbose` shows platform info with architecture, distro (if Linux/WSL), and capabilities. Update existing smoke test if needed.

## Edge Cases

| Scenario | Expected behavior |
|---|---|
| `/etc/os-release` missing (Docker scratch) | `LinuxDistro::Other("unknown")`, no panic |
| `/etc/os-release` has `ID=` with empty value | `LinuxDistro::Other("")` |
| `$PATH` empty or unset | `command_exists()` returns `false` for all commands |
| `sw_vers` not found (not macOS) | `detect_macos_version()` returns `None` |
| WSL1 (no WSLInterop file) | `is_wsl()` true (env var or /proc/version), `is_wsl2()` false |
| WSL2 (WSLInterop file present) | `is_wsl()` true, `is_wsl2()` true |
| Docker container as root, no `$USER` | `is_root()` returns `true` (uses `id -u`, not `$USER`) |
| Non-UTF8 system locale | `detect_shell()` returns `"unknown"` if `$SHELL` parse fails |
| Windows without `$COMSPEC` | `detect_shell()` returns `"unknown"` |
| `std::env::consts::ARCH` returns unexpected value | `Architecture::Unknown` |
| Concurrent calls to `detect_platform_info()` | Safe -- all functions are stateless, read-only |
| Network unavailable | No detection function uses the network. All local. |

## Security Considerations

- **No `unsafe` code**: `is_root()` uses `id -u` subprocess, not `libc::geteuid()`. No unsafe blocks.
- **No shell injection in `command_exists()`**: The `which` crate does a PATH walk, never invokes a shell. Command names are used as file lookups, not shell arguments.
- **No secrets in detection output**: Platform info does not include usernames, hostnames, or IP addresses.
- **Root detection is advisory**: `is_root` is used for warnings ("you're running as root"), not for security gates. Do not use it as an access control mechanism.

## Error Handling

All detection functions follow the same contract: **never fail, never panic, return a safe default**.

| Function | Failure mode | Return value |
|---|---|---|
| `detect_platform()` | OS not recognized | `Platform::Unknown` |
| `detect_architecture()` | Arch string unknown | `Architecture::Unknown` |
| `detect_linux_distro()` | File unreadable or missing | `LinuxDistro::Other("unknown")` |
| `detect_linux_version()` | File unreadable or field missing | `None` |
| `detect_macos_version()` | Command fails | `None` |
| `command_exists(cmd)` | PATH unset, probe fails | `false` |
| `is_wsl()` | All checks fail | `false` |
| `is_wsl2()` | File check fails | `false` |
| `is_root()` | `id` command fails | `false` |
| `detect_shell()` | Env var unset | `"unknown"` |

No function in `src/platform/` returns `Result`. No function uses `.unwrap()` or `.expect()`. No function calls `panic!()`.

## Socrates Review Resolution Checklist

| Issue | Resolution |
|---|---|
| **B1**: `command_exists` implementation unspecified | Use `which` crate (pure Rust PATH walk). Add `which = "7"` to Cargo.toml. Cross-platform, no shell spawning, works on Alpine. |
| **B2**: `is_root()` needs `libc` | Use `id -u` subprocess on Unix via `#[cfg(unix)]`. Always `false` on non-Unix. No `unsafe`, no new crate. |
| **B3**: WSL detection contradicts backlog | Added `/proc/sys/fs/binfmt_misc/WSLInterop` check per backlog. Three-tier detection for `is_wsl()`. Separate `is_wsl2()` function. Renamed capability to `is_wsl2`. |
| **N1**: Missing `Serialize`/`Deserialize` on `Platform` | Added `Serialize, Deserialize` derives to `Platform`, `PlatformCapabilities`, `PlatformInfo`. `Hash` intentionally omitted with documented rationale. |
| **N2**: `detect_shell()` unreliable | Documented that `$SHELL` is login shell (intentional). Added `$COMSPEC` fallback on Windows. Fallback to `"unknown"` when unset. |
| **N3**: No mock strategy for tests | Added `OsProbe` trait (`#[cfg(test)]` only) with `_with_probe` function variants. 14 mock-based tests specified. |
| **N4**: Missing `has_pacman` | Added `has_pacman: bool` to `PlatformCapabilities`. Detected via `command_exists("pacman")`. |
