# 0008: Runtime Version Manager Integration (mise)

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/platform/runtime.rs` (new file)
**Status:** pending

## Context

The `great.toml` `[tools]` section declares runtime versions (e.g., `node = "22"`, `python = "3.12"`, `rust = "stable"`). These runtimes need to be installed and activated at the correct versions. The tool `mise` (formerly `rtx`) is the chosen runtime version manager -- it handles Node.js, Python, Go, Rust, and many other runtimes with a single interface.

The `great doctor` command (task 0005) already checks for mise and recommends its installation. This task adds the actual integration: detecting mise, installing it if absent, and using it to provision declared runtimes.

The `great.toml` schema (task 0002) stores runtimes in `ToolsConfig.runtimes` as a `HashMap<String, String>` where keys are tool names and values are version strings. The key `"cli"` is reserved for CLI tools and must be excluded from runtime processing.

## Requirements

1. **Detect mise installation**: Create a `MiseManager` struct with a `is_available() -> bool` method that checks if `mise` is on PATH using `command_exists("mise")`. Add a `version() -> Option<String>` method that runs `mise --version` and returns the version string. If mise is not available, all other operations should return clear errors indicating mise needs to be installed first.

2. **Install mise if not present**: Add an `ensure_installed(pkg_manager: &dyn PackageManager) -> Result<()>` method that installs mise using the provided package manager. On macOS, this is `brew install mise`. On Linux/WSL2, this is `curl https://mise.jdx.dev/install.sh | sh` (since mise is not in apt). The method should verify installation succeeded by checking `command_exists("mise")` after the install command completes.

3. **Install a declared runtime**: Add an `install_runtime(name: &str, version: &str) -> Result<()>` method that runs `mise install <name>@<version>` and `mise use --global <name>@<version>`. The method should handle common runtime names: `node`, `python`, `go`, `rust`, `java`, `ruby`. It should fail gracefully with a clear error if mise does not support the requested runtime.

4. **Check installed runtime versions**: Add a `installed_version(name: &str) -> Option<String>` method that runs `mise current <name>` and parses the output to extract the active version. Return `None` if the runtime is not installed or mise is not available. This is used by `great diff` and `great status` to compare declared vs actual versions.

5. **Batch provision from config**: Add a `provision_from_config(tools: &ToolsConfig) -> Result<Vec<ProvisionResult>>` method that iterates over all entries in `tools.runtimes` (excluding `"cli"`), checks each against the installed version, and installs or updates as needed. Return a `ProvisionResult` struct for each runtime with fields: `name: String`, `declared_version: String`, `action: ProvisionAction` (enum: Installed, Updated, AlreadyCorrect, Failed(String)). This gives `great apply` structured feedback to display.

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/platform/runtime.rs`.
- [ ] `MiseManager::is_available()` correctly returns `true` on a machine with mise installed, `false` otherwise.
- [ ] `MiseManager::installed_version("node")` returns `Some(version_string)` when Node.js is installed via mise, `None` otherwise.
- [ ] `provision_from_config()` skips the `"cli"` key and processes only runtime entries.
- [ ] No `.unwrap()` or `.expect()` calls exist in `src/platform/runtime.rs`.

## Dependencies

- Task 0001 (platform detection) -- already landed; `command_exists()` is available.
- Task 0002 (config schema) -- already landed; `ToolsConfig` with `runtimes: HashMap<String, String>` is available.
- Task 0007 (package manager) -- required for installing mise itself via the `PackageManager` trait.

## Notes

- **mise vs direct installation**: For runtimes like Node.js and Python, mise provides version isolation and easy switching. For Rust, `rustup` is the canonical tool and mise delegates to it. The runtime manager should not fight with existing `rustup` installations -- if `rust` is declared, consider checking for `rustup` first and using it directly, or deferring to `mise use rust@<version>` which wraps `rustup` internally.
- **Version string normalization**: The `great.toml` version strings may be partial (e.g., `"22"` for Node.js means "latest 22.x"). mise handles this natively (`mise install node@22` installs the latest 22.x), so version strings can be passed through without normalization.
- **Environment activation**: `mise use --global` writes to `~/.config/mise/config.toml`. For project-local activation, `mise use` writes to `.mise.toml` in the current directory. The `great apply` command should use global activation by default and document how to override per-project.
- The module should be re-exported from `src/platform/mod.rs` as `pub mod runtime;`.
