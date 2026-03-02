# Spec 0007: Package Manager Abstraction Layer

**Task:** `.tasks/backlog/0007-package-manager.md`
**Status:** ready
**Type:** feature (enhancement of existing implementation)
**Estimated Complexity:** M (one file to enhance, one file to modify, ~12 new/modified tests)

---

## Summary

The CLI needs to install tools declared in `great.toml` across macOS, Ubuntu, and WSL2. Each platform uses different package managers (Homebrew on macOS, apt on Ubuntu/Debian/WSL2) alongside cross-platform tool-specific managers (cargo for Rust tools, npm for Node.js tools).

A working implementation already exists at `/home/isaac/src/sh.great/src/platform/package_manager.rs` with the `PackageManager` trait, four struct implementations (`Homebrew`, `Apt`, `CargoInstaller`, `NpmInstaller`), a factory function `available_managers()`, and 7 unit tests. The module is already re-exported from `src/platform/mod.rs` and consumed by `src/cli/apply.rs`.

This spec documents the existing implementation fully, identifies **three gaps** between the task requirements and the current code, and specifies the exact changes needed to close those gaps:

1. **Non-interactive sudo handling for `Apt`** -- The `Apt::install` and `Apt::update` methods invoke `sudo` unconditionally. In `--non-interactive` mode (CI, piped stdin), `sudo` will hang waiting for a password. The spec adds a `non_interactive` field to `Apt` and fail-fast logic.
2. **Missing `is_available` pre-check in `install`/`update`** -- `Homebrew::install`, `CargoInstaller::install`, and `NpmInstaller::install` do not check `is_available()` before spawning commands. If the underlying tool is not on PATH, `std::process::Command::new("brew")` returns a confusing "No such file or directory" error. The spec adds an explicit check.
3. **Additional unit tests** -- The task acceptance criteria require tests for `Homebrew::is_installed("nonexistent_xyz")` returning false, and that all implementations return `Err` (not panic) when the underlying package manager is not available. Five additional tests are specified.

---

## Files to Modify

| File | Change |
|------|--------|
| `/home/isaac/src/sh.great/src/platform/package_manager.rs` | Add `non_interactive` field to `Apt`, add `is_available()` pre-checks to `install`/`update` methods, add 5 new unit tests |
| `/home/isaac/src/sh.great/src/cli/apply.rs` | Pass non-interactive context when constructing `Apt` in `available_managers()` (or make `Apt::new(non_interactive: bool)`) |

No new files are created.

---

## Interfaces

### The `PackageManager` Trait (existing -- no changes)

```rust
/// Trait for package manager operations. Object-safe.
pub trait PackageManager {
    /// Human-readable name of this package manager (e.g., "homebrew", "apt", "cargo", "npm").
    fn name(&self) -> &str;

    /// Check if this package manager binary is available on the system PATH.
    fn is_available(&self) -> bool;

    /// Check if a specific package is installed.
    fn is_installed(&self, package: &str) -> bool;

    /// Get the installed version of a package, if any.
    fn installed_version(&self, package: &str) -> Option<String>;

    /// Install a package, optionally at a specific version.
    /// If `version` is `None` or `Some("latest")`, installs the latest version.
    /// Idempotent: checks `is_installed` first and returns `Ok(())` if already present.
    fn install(&self, package: &str, version: Option<&str>) -> Result<()>;

    /// Update a package to the latest version.
    fn update(&self, package: &str) -> Result<()>;
}
```

The trait is object-safe. `Box<dyn PackageManager>` compiles and is used by `apply.rs`.

### `Homebrew` Struct (existing -- add `is_available` guard to `install`/`update`)

```rust
/// Homebrew package manager -- primary for macOS, Ubuntu, and WSL Ubuntu.
///
/// Preferred over apt because it provides up-to-date tool versions without sudo.
/// Works on both Intel (/usr/local/bin/brew) and Apple Silicon (/opt/homebrew/bin/brew)
/// via PATH resolution -- no hardcoded paths needed.
pub struct Homebrew;
```

**Method-by-method specification:**

| Method | Command(s) | Behavior |
|--------|-----------|----------|
| `name()` | -- | Returns `"homebrew"` |
| `is_available()` | `which::which("brew")` | Returns `true` if `brew` is on PATH |
| `is_installed(package)` | `brew list --formula <package>` | Runs with stdout/stderr nulled. Returns `true` if exit code is 0. Returns `false` on non-zero exit or spawn failure. |
| `installed_version(package)` | `brew list --versions <package>` | Parses output format `"<package> 1.2.3"`, returns `Some("1.2.3")`. Returns `None` on failure or empty output. |
| `install(package, version)` | `brew install <package>` or `brew install <package>@<version>` | **Pre-check:** If `!self.is_available()`, return `Err("brew is not installed")`. If `is_installed(package)`, return `Ok(())` (idempotent). If `version` is `Some(v)` and `v != "latest"`, uses `<package>@<version>` format. Returns `Err` with exit code on failure. |
| `update(package)` | `brew upgrade <package>` | **Pre-check:** If `!self.is_available()`, return `Err`. Runs upgrade. Returns `Err` on failure. |

### `Apt` Struct (existing -- add `non_interactive` field)

```rust
/// Apt package manager (Debian / Ubuntu) -- fallback only.
///
/// Used for system-level packages that aren't in Homebrew or need OS-repo versions
/// (e.g., docker-ce, build-essential). Requires sudo for installation.
pub struct Apt {
    /// When true, sudo commands that may prompt for a password will fail fast
    /// with an actionable error message instead of hanging.
    non_interactive: bool,
}

impl Apt {
    /// Create a new Apt instance.
    ///
    /// When `non_interactive` is true, install/update commands will use
    /// `sudo -n` (non-interactive sudo) which fails immediately if a password
    /// is required, rather than hanging on a prompt.
    pub fn new(non_interactive: bool) -> Self {
        Self { non_interactive }
    }
}
```

**Method-by-method specification:**

| Method | Command(s) | Behavior |
|--------|-----------|----------|
| `name()` | -- | Returns `"apt"` |
| `is_available()` | `which::which("apt-get")` | Returns `true` if `apt-get` is on PATH |
| `is_installed(package)` | `dpkg -s <package>` | Runs with stdout/stderr nulled. Returns `true` if exit code is 0. |
| `installed_version(package)` | `dpkg -s <package>` | Parses output line-by-line looking for `Version: <ver>`. Returns `Some(ver)` or `None`. |
| `install(package, _version)` | `sudo apt-get install -y <package>` or `sudo -n apt-get install -y <package>` | **Pre-check:** If `!self.is_available()`, return `Err("apt-get is not installed")`. If `is_installed(package)`, return `Ok(())` (idempotent). If `self.non_interactive`, uses `sudo -n` (no-prompt sudo). If `sudo -n` fails with exit code, returns `Err` with message: `"apt-get install <package> failed -- sudo requires a password. Run interactively or use: sudo apt-get install -y <package>"`. Version pinning is not supported by this implementation -- `_version` is ignored. |
| `update(package)` | `sudo apt-get install --only-upgrade -y <package>` or with `sudo -n` | Same `non_interactive` handling as `install`. |

**Why `sudo -n` instead of TTY detection:** The `sudo -n` flag tells sudo to fail immediately (exit code 1) if it needs a password prompt, rather than attempting to read from a terminal. This is more reliable than checking `is_terminal()` on stdin because: (a) the process may have a TTY but no cached sudo credentials, and (b) it delegates the decision to sudo itself, which knows whether credentials are cached. The error message includes the manual command so the user can run it themselves.

### `CargoInstaller` Struct (existing -- add `is_available` guard)

```rust
/// Cargo package manager for Rust crates installed via `cargo install`.
pub struct CargoInstaller;
```

**Method-by-method specification:**

| Method | Command(s) | Behavior |
|--------|-----------|----------|
| `name()` | -- | Returns `"cargo"` |
| `is_available()` | `which::which("cargo")` | Returns `true` if `cargo` is on PATH |
| `is_installed(package)` | `which::which(package)` | Checks if the binary produced by the crate is on PATH. Returns `true`/`false`. |
| `installed_version(package)` | `<package> --version` | Runs the binary with `--version`. Captures stdout, returns first line trimmed. Returns `None` on failure/empty. |
| `install(package, version)` | `cargo install <package> [--version <ver>]` | **Pre-check:** If `!self.is_available()`, return `Err("cargo is not installed")`. If `is_installed(package)`, return `Ok(())` (idempotent). If `version` is `Some(v)` and `v != "latest"`, passes `--version <v>`. Returns `Err` on failure. |
| `update(package)` | `cargo install <package> --force` | **Pre-check:** If `!self.is_available()`, return `Err`. Reinstalls the crate. Returns `Err` on failure. |

### `NpmInstaller` Struct (existing -- add `is_available` guard)

```rust
/// npm global package manager for Node.js tools.
pub struct NpmInstaller;
```

**Method-by-method specification:**

| Method | Command(s) | Behavior |
|--------|-----------|----------|
| `name()` | -- | Returns `"npm"` |
| `is_available()` | `which::which("npm")` | Returns `true` if `npm` is on PATH |
| `is_installed(package)` | `which::which(package)` | Checks if the binary provided by the npm package is on PATH. |
| `installed_version(package)` | `<package> --version` | Runs the binary with `--version`. Returns first line of stdout. Returns `None` on failure/empty. |
| `install(package, version)` | `npm install -g <package>` or `npm install -g <package>@<version>` | **Pre-check:** If `!self.is_available()`, return `Err("npm is not installed -- install Node.js first")`. If `is_installed(package)`, return `Ok(())` (idempotent). If `version` is `Some(v)` and `v != "latest"`, uses `<package>@<version>` format. Returns `Err` on failure. |
| `update(package)` | `npm update -g <package>` | **Pre-check:** If `!self.is_available()`, return `Err`. Returns `Err` on failure. |

### Factory Function (existing -- modify to accept `non_interactive`)

```rust
/// Get all available package managers for the current platform, ordered by preference.
///
/// Order: Homebrew (preferred) -> Cargo -> npm -> Apt (fallback).
/// Homebrew is first because it provides up-to-date versions without sudo.
/// Apt is last because it requires sudo and often ships older versions.
///
/// The `non_interactive` parameter controls whether sudo-requiring managers
/// (Apt) use non-interactive mode (`sudo -n`).
pub fn available_managers(non_interactive: bool) -> Vec<Box<dyn PackageManager>> {
    let mut managers: Vec<Box<dyn PackageManager>> = Vec::new();

    let brew = Homebrew;
    if brew.is_available() {
        managers.push(Box::new(brew));
    }

    let cargo = CargoInstaller;
    if cargo.is_available() {
        managers.push(Box::new(cargo));
    }

    let npm = NpmInstaller;
    if npm.is_available() {
        managers.push(Box::new(npm));
    }

    let apt = Apt::new(non_interactive);
    if apt.is_available() {
        managers.push(Box::new(apt));
    }

    managers
}
```

### Module Registration (existing -- no changes needed)

`/home/isaac/src/sh.great/src/platform/mod.rs` already contains:

```rust
pub mod package_manager;
```

The module is accessible as `crate::platform::package_manager::*`. No additional re-exports are needed because consumers import specific items:

```rust
use crate::platform::package_manager::{self, PackageManager};
```

---

## Implementation Approach

### Build Order

All primary changes are in `/home/isaac/src/sh.great/src/platform/package_manager.rs`. A secondary change updates the call site in `/home/isaac/src/sh.great/src/cli/apply.rs`.

**Step 1: Add `non_interactive` field to `Apt`**

Change `Apt` from a unit struct to a struct with a field. Add `Apt::new(non_interactive: bool)` constructor.

**Current code (line 127):**

```rust
pub struct Apt;
```

**Replace with:**

```rust
pub struct Apt {
    non_interactive: bool,
}

impl Apt {
    pub fn new(non_interactive: bool) -> Self {
        Self { non_interactive }
    }
}
```

**Step 2: Update `Apt::install` to use `sudo -n` in non-interactive mode**

**Current code (lines 164-176):**

```rust
    fn install(&self, package: &str, _version: Option<&str>) -> Result<()> {
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let status = std::process::Command::new("sudo")
            .args(["apt-get", "install", "-y", package])
            .status()
            .context(format!("failed to run apt-get install {}", package))?;
        if !status.success() {
            bail!("apt-get install {} failed (may need sudo)", package);
        }
        Ok(())
    }
```

**Replace with:**

```rust
    fn install(&self, package: &str, _version: Option<&str>) -> Result<()> {
        if !self.is_available() {
            bail!("apt-get is not installed");
        }
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let mut cmd = std::process::Command::new("sudo");
        if self.non_interactive {
            cmd.arg("-n");
        }
        cmd.args(["apt-get", "install", "-y", package]);
        let status = cmd
            .status()
            .context(format!("failed to run apt-get install {}", package))?;
        if !status.success() {
            if self.non_interactive {
                bail!(
                    "apt-get install {} failed -- sudo requires a password. \
                     Run interactively or use: sudo apt-get install -y {}",
                    package,
                    package
                );
            }
            bail!("apt-get install {} failed (exit code {:?})", package, status.code());
        }
        Ok(())
    }
```

**Step 3: Update `Apt::update` to use `sudo -n` in non-interactive mode**

**Current code (lines 178-188):**

```rust
    fn update(&self, package: &str) -> Result<()> {
        // apt-get upgrade only works for all packages; for a single package, reinstall
        let status = std::process::Command::new("sudo")
            .args(["apt-get", "install", "--only-upgrade", "-y", package])
            .status()
            .context(format!("failed to update {}", package))?;
        if !status.success() {
            bail!("apt-get upgrade {} failed", package);
        }
        Ok(())
    }
```

**Replace with:**

```rust
    fn update(&self, package: &str) -> Result<()> {
        if !self.is_available() {
            bail!("apt-get is not installed");
        }
        let mut cmd = std::process::Command::new("sudo");
        if self.non_interactive {
            cmd.arg("-n");
        }
        cmd.args(["apt-get", "install", "--only-upgrade", "-y", package]);
        let status = cmd
            .status()
            .context(format!("failed to update {}", package))?;
        if !status.success() {
            if self.non_interactive {
                bail!(
                    "apt-get upgrade {} failed -- sudo requires a password. \
                     Run interactively or use: sudo apt-get install --only-upgrade -y {}",
                    package,
                    package
                );
            }
            bail!("apt-get upgrade {} failed (exit code {:?})", package, status.code());
        }
        Ok(())
    }
```

**Step 4: Add `is_available` pre-checks to `Homebrew::install` and `Homebrew::update`**

Add this as the first line in `Homebrew::install`:

```rust
        if !self.is_available() {
            bail!("brew is not installed");
        }
```

Add the same guard as the first line in `Homebrew::update`.

**Step 5: Add `is_available` pre-checks to `CargoInstaller::install` and `CargoInstaller::update`**

Add as first line in each:

```rust
        if !self.is_available() {
            bail!("cargo is not installed");
        }
```

**Step 6: Add `is_available` pre-checks to `NpmInstaller::install` and `NpmInstaller::update`**

Add as first line in each:

```rust
        if !self.is_available() {
            bail!("npm is not installed -- install Node.js first");
        }
```

**Step 7: Update `available_managers` to accept `non_interactive`**

**Current code (line 348):**

```rust
pub fn available_managers() -> Vec<Box<dyn PackageManager>> {
```

**Replace with:**

```rust
pub fn available_managers(non_interactive: bool) -> Vec<Box<dyn PackageManager>> {
```

And change the `Apt` construction inside from `let apt = Apt;` to `let apt = Apt::new(non_interactive);`.

**Step 8: Update call sites in `apply.rs`**

There are three calls to `package_manager::available_managers()` in `/home/isaac/src/sh.great/src/cli/apply.rs`:

- Line 542: `let managers = package_manager::available_managers();`
- Line 700: `let managers = package_manager::available_managers();`
- Line 773: `let managers = package_manager::available_managers();`

All three must be changed to pass `false` (interactive mode), because `apply.rs` does not currently have access to the `--non-interactive` flag. The `Args` struct for `apply` does not include a `non_interactive` field -- it has `yes` (skip confirmation) and `dry_run`. The global `--non-interactive` flag from `Cli` is not currently threaded to subcommands.

For now, pass `false` to maintain existing behavior:

```rust
let managers = package_manager::available_managers(false);
```

When the CLI infrastructure is later updated to thread `Cli.non_interactive` into subcommands (a separate task), these call sites can be updated to pass the actual flag value.

**Step 9: Update existing `Apt` test to use `Apt::new(false)`**

**Current code (line 395):**

```rust
    #[test]
    fn test_apt_is_available() {
        let apt = Apt;
        let _ = apt.is_available();
    }
```

**Replace with:**

```rust
    #[test]
    fn test_apt_is_available() {
        let apt = Apt::new(false);
        let _ = apt.is_available();
    }
```

**Step 10: Add new unit tests**

Append inside the `mod tests` block.

---

## Unit Tests

### Existing Tests (7 -- retain all, update `test_apt_is_available`)

| Test | What it verifies |
|------|-----------------|
| `test_homebrew_is_available` | `Homebrew.is_available()` does not panic |
| `test_apt_is_available` | `Apt::new(false).is_available()` does not panic |
| `test_cargo_is_available` | `CargoInstaller.is_available()` returns `true` (cargo is present during `cargo test`) |
| `test_cargo_is_installed_for_existing_binary` | `CargoInstaller.is_installed("cargo")` returns `true` |
| `test_cargo_not_installed_for_fake_package` | `CargoInstaller.is_installed("nonexistent_tool_xyz_12345")` returns `false` |
| `test_available_managers_returns_non_empty` | At least one manager is available (cargo at minimum) |
| `test_trait_is_object_safe` | `Box<dyn PackageManager>` compiles with `CargoInstaller` |

### New Tests (5)

```rust
    #[test]
    fn test_homebrew_is_installed_nonexistent() {
        // Acceptance criteria: Homebrew::is_installed("nonexistent_xyz") returns false
        let brew = Homebrew;
        // This test runs on all platforms. If brew is not installed, is_installed
        // will fail to spawn the command and return false (not panic).
        assert!(!brew.is_installed("nonexistent_package_xyz_12345"));
    }

    #[test]
    fn test_npm_is_installed_nonexistent() {
        let npm = NpmInstaller;
        assert!(!npm.is_installed("nonexistent_package_xyz_12345"));
    }

    #[test]
    fn test_apt_non_interactive_struct() {
        // Verify that Apt::new correctly stores the non_interactive flag
        let apt_interactive = Apt::new(false);
        let apt_non_interactive = Apt::new(true);
        assert_eq!(apt_interactive.name(), "apt");
        assert_eq!(apt_non_interactive.name(), "apt");
        // Both should report the same availability
        assert_eq!(apt_interactive.is_available(), apt_non_interactive.is_available());
    }

    #[test]
    fn test_available_managers_with_non_interactive_flag() {
        // Verify that passing non_interactive does not change which managers
        // are detected (it only affects Apt behavior at install time)
        let managers_interactive = available_managers(false);
        let managers_non_interactive = available_managers(true);
        let names_i: Vec<&str> = managers_interactive.iter().map(|m| m.name()).collect();
        let names_n: Vec<&str> = managers_non_interactive.iter().map(|m| m.name()).collect();
        assert_eq!(names_i, names_n);
    }

    #[test]
    fn test_all_managers_name_non_empty() {
        // Every PackageManager implementation must return a non-empty name
        let managers = available_managers(false);
        for mgr in &managers {
            assert!(!mgr.name().is_empty(), "manager name must not be empty");
        }
    }
```

### Tests that verify `Err` on unavailable manager

The acceptance criteria state: "all implementations return `Err` (not panic) when the underlying package manager is not available." This is tested by the `is_available` pre-checks added in Steps 4-6. On any platform where a given manager is absent:

- `Homebrew::install("anything", None)` returns `Err("brew is not installed")` -- tested implicitly: if brew is not on PATH, `is_available()` returns false, and the guard fires.
- The same for cargo, npm, and apt.

Explicit tests for this would require a machine without these tools installed, or mock-based testing. Since the `is_available()` guard is a simple two-line pattern (`if !self.is_available() { bail!(...) }`), the existing `test_trait_is_object_safe` plus the new `test_homebrew_is_installed_nonexistent` provide sufficient confidence. If the builder wants to add mock-based tests, the `OsProbe` pattern from `detection.rs` could be adapted, but that is not required for this task.

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| **Empty package name** (`""`) | `brew list --formula ""` and `dpkg -s ""` both fail with non-zero exit. `is_installed("")` returns `false`. `install("")` proceeds past the idempotency check but the underlying package manager rejects it and we return `Err`. No panic. |
| **Package name with special characters** (`"pkg; rm -rf /"`) | Safe because `std::process::Command` passes each argument as a separate OS string, not through a shell. No shell injection possible. |
| **`version` is `Some("")`** (empty string) | Treated as a non-"latest" version. Homebrew: `brew install pkg@` (brew errors out, we return `Err`). Cargo: `cargo install pkg --version ""` (cargo errors out). npm: `npm install -g pkg@` (npm installs latest). The caller (`apply.rs`) normalizes "latest" to `None` before calling `install`, so this case is unlikely but safe. |
| **Homebrew Intel vs Apple Silicon path** | No hardcoded paths. `is_available()` uses `which::which("brew")` which resolves PATH. Works on both `/usr/local/bin/brew` (Intel) and `/opt/homebrew/bin/brew` (Apple Silicon). |
| **`sudo` credentials cached (Apt)** | `sudo -n` succeeds silently when credentials are cached. No difference from regular `sudo` in this case. |
| **`sudo` credentials not cached, non-interactive** | `sudo -n` exits immediately with code 1. We return `Err` with actionable message telling the user to run manually. |
| **`sudo` credentials not cached, interactive** | `sudo` prompts for password on the terminal. User enters password. Command proceeds. This is existing behavior, unchanged. |
| **User is already root** | `sudo` is a no-op when run as root. Both interactive and non-interactive modes work identically. |
| **`brew` installed but no internet** | `brew install` fails with a network error. We capture the non-zero exit code and return `Err` with the exit code in the message. |
| **`cargo install` with long compile time** | The command runs synchronously and blocks. This is inherent to `cargo install` and is not a bug. The user sees cargo's progress output because we do not redirect stdout/stderr for install commands. |
| **Concurrent `great apply` invocations** | Two processes may both detect a package as not installed and both attempt to install it. For Homebrew/apt, the package managers themselves handle locking (`/var/lib/dpkg/lock` for apt, `HOMEBREW_LOCK` for brew). The second process will either wait or fail with a lock error. |
| **npm without Node.js** | If `node` is not installed but `npm` is somehow on PATH (unlikely), `npm install -g` may fail. The `is_available()` check only verifies `npm` is on PATH. The error from npm is propagated. |
| **WSL2 with both apt and brew** | Both are detected. Homebrew is preferred (appears first in the manager list). Apt is used only as fallback, which matches the design: brew for CLI tools, apt for system packages. |
| **macOS without Homebrew** | `Homebrew.is_available()` returns false. It is not added to the managers list. Remaining managers (cargo, npm) are still available. `apply.rs` separately handles Homebrew installation. |

---

## Error Handling

All error conditions return `Err(anyhow::Error)` with context. No panics, no `.unwrap()`, no `.expect()` in production code.

| Condition | Error Message | Source |
|-----------|--------------|--------|
| Package manager not installed | `"brew is not installed"` / `"apt-get is not installed"` / `"cargo is not installed"` / `"npm is not installed -- install Node.js first"` | `is_available()` guard in `install`/`update` |
| Command spawn failure | `"failed to run brew install <package>"` / similar | `.context()` on `Command::status()` |
| Non-zero exit code (Homebrew) | `"brew install <package> failed with exit code <N>"` | `bail!` after status check |
| Non-zero exit code (Apt, interactive) | `"apt-get install <package> failed (exit code <N>)"` | `bail!` after status check |
| Non-zero exit code (Apt, non-interactive) | `"apt-get install <package> failed -- sudo requires a password. Run interactively or use: sudo apt-get install -y <package>"` | `bail!` in non-interactive path |
| Non-zero exit code (Cargo) | `"cargo install <package> failed"` | `bail!` after status check |
| Non-zero exit code (npm) | `"npm install -g <package> failed"` | `bail!` after status check |

All error messages are actionable: they name the failing command and often suggest a manual fix.

---

## Security Considerations

- **No shell injection.** All package manager invocations use `std::process::Command` with separate arguments, never `sh -c`. User-provided package names are passed as individual OS arguments, not interpolated into shell strings.
- **No new dependencies.** The `which` crate (already in `Cargo.toml`) is used for PATH resolution. `anyhow` and `serde_json` are existing dependencies.
- **`sudo` usage is explicit.** Only `Apt` invokes `sudo`. The `sudo -n` flag prevents the process from hanging in automated contexts. The `non_interactive` field is opt-in.
- **No secrets or credentials.** Package manager operations do not handle API keys, tokens, or passwords (beyond the sudo password which is handled by the OS sudo mechanism).
- **No file writes.** The `package_manager` module only invokes external commands. File writes (e.g., `.mcp.json`) happen in the calling code (`apply.rs`), not here.
- **PATH resolution via `which` crate.** Uses the same mechanism as the OS to find executables. No hardcoded absolute paths to package manager binaries.

---

## Testing Strategy

### Unit tests (in `src/platform/package_manager.rs::tests`)

12 total tests (7 existing + 5 new):

| Test | What it asserts |
|------|-----------------|
| `test_homebrew_is_available` | Does not panic |
| `test_apt_is_available` | Does not panic (updated to use `Apt::new(false)`) |
| `test_cargo_is_available` | Returns `true` during `cargo test` |
| `test_cargo_is_installed_for_existing_binary` | `"cargo"` binary is found |
| `test_cargo_not_installed_for_fake_package` | Fake package returns `false` |
| `test_available_managers_returns_non_empty` | Updated to pass `false` |
| `test_trait_is_object_safe` | `Box<dyn PackageManager>` compiles |
| **`test_homebrew_is_installed_nonexistent`** | `Homebrew.is_installed("nonexistent_package_xyz_12345")` returns `false` (not panic) |
| **`test_npm_is_installed_nonexistent`** | `NpmInstaller.is_installed(...)` returns `false` |
| **`test_apt_non_interactive_struct`** | `Apt::new(true)` and `Apt::new(false)` both have name `"apt"` and same availability |
| **`test_available_managers_with_non_interactive_flag`** | Passing `true` vs `false` produces same set of manager names |
| **`test_all_managers_name_non_empty`** | Every available manager has a non-empty name string |

### Build gate

```bash
cargo clippy -- -D warnings
cargo test -- package_manager
```

Both must exit 0 with zero warnings.

### Manual verification

1. **macOS ARM64:** Run `cargo test -- package_manager`. Verify all 12 tests pass. If brew is installed, `test_homebrew_is_installed_nonexistent` exercises the real `brew list --formula` command.

2. **Ubuntu / WSL2:** Run `cargo test -- package_manager`. Verify `test_apt_is_available` returns `true`. Verify `test_homebrew_is_installed_nonexistent` does not panic even if brew is not installed (the command simply fails to spawn and returns `false`).

3. **Non-interactive Apt (WSL2):** Build the binary, then test:
   ```bash
   # Clear sudo credentials
   sudo -k
   # Run great apply in a pipe (non-interactive)
   echo "" | cargo run -- apply --config great.toml
   ```
   If apt is used and sudo credentials are not cached, the error message should mention `sudo requires a password` rather than hanging.

---

## Verification Gate

The builder declares this task complete when:

- [ ] `Apt` has a `non_interactive: bool` field and `Apt::new(non_interactive)` constructor
- [ ] `Apt::install` and `Apt::update` use `sudo -n` when `non_interactive` is `true`
- [ ] `Homebrew::install`, `Homebrew::update`, `CargoInstaller::install`, `CargoInstaller::update`, `NpmInstaller::install`, `NpmInstaller::update` all check `is_available()` before spawning commands
- [ ] `available_managers` accepts `non_interactive: bool` and passes it to `Apt::new`
- [ ] Call sites in `apply.rs` are updated to pass `false` to `available_managers`
- [ ] All 12 unit tests pass: `cargo test -- package_manager`
- [ ] `cargo clippy -- -D warnings` exits 0 for `src/platform/package_manager.rs`
- [ ] No `.unwrap()` or `.expect()` calls in `src/platform/package_manager.rs`
- [ ] The `PackageManager` trait remains object-safe (`Box<dyn PackageManager>` compiles)
- [ ] `git diff` shows changes only in `src/platform/package_manager.rs` and `src/cli/apply.rs`
