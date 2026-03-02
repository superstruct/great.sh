# 0005: Complete `great doctor` Command -- Technical Specification

**Task:** 0005-doctor-command
**Status:** ready
**Author:** Lovelace (Spec Writer), Iteration 008
**Date:** 2026-02-24

---

## Summary

Complete the `great doctor` command by addressing five gaps: (1) extract the duplicated `get_command_version()` utility to a shared module, (2) add MCP server reachability checks, (3) add exit code 1 when any check fails, (4) fix the `unwrap_or_default()` path-safety issue, and (5) expand integration tests. The `--fix` auto-fix logic is already fully implemented and must not be rewritten.

**Files to create:** 1 (`src/cli/util.rs`)
**Files to modify:** 4 (`src/cli/mod.rs`, `src/cli/doctor.rs`, `src/cli/status.rs`, `tests/cli_smoke.rs`)
**Estimated lines changed:** ~90 added, ~40 removed

---

## 1. Create Shared Utility Module

### New file: `src/cli/util.rs`

This file extracts the duplicated version-checking function into a single location. Both `doctor.rs` and `status.rs` currently contain identical implementations (`get_command_version` and `get_tool_version` respectively). After this change, both modules import from `crate::cli::util`.

```rust
/// Shared CLI utility functions.
///
/// Extracts helpers that are used by multiple subcommands to avoid duplication.

/// Try to get a command's version string.
///
/// Runs `<cmd> --version` and returns the first line of stdout, or `None`
/// if the command fails or produces no output.
pub fn get_command_version(cmd: &str) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            None
        } else {
            Some(first_line.to_string())
        }
    } else {
        None
    }
}
```

### Modify: `src/cli/mod.rs`

Add `pub mod util;` to the module declarations.

**Current content (line 1-15):**
```rust
pub mod apply;
pub mod bootstrap;
pub mod diff;
pub mod doctor;
pub mod init;
pub mod loop_cmd;
pub mod mcp;
pub mod output;
pub mod status;
pub mod statusline;
pub mod sync;
pub mod template;
pub mod tuning;
pub mod update;
pub mod vault;
```

**Change:** Insert `pub mod util;` after `pub mod update;` (alphabetical ordering places it between `update` and `vault`).

**New content:**
```rust
pub mod apply;
pub mod bootstrap;
pub mod diff;
pub mod doctor;
pub mod init;
pub mod loop_cmd;
pub mod mcp;
pub mod output;
pub mod status;
pub mod statusline;
pub mod sync;
pub mod template;
pub mod tuning;
pub mod update;
pub mod util;
pub mod vault;
```

---

## 2. Modify `src/cli/doctor.rs`

### 2a. Update imports (line 1-7)

**Current:**
```rust
use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::{bootstrap, output, tuning};
use crate::config;
use crate::platform::package_manager;
use crate::platform::{self, command_exists, Platform, PlatformInfo};
```

**New:**
```rust
use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::{bootstrap, output, tuning, util};
use crate::config;
use crate::platform::package_manager;
use crate::platform::{self, command_exists, Platform, PlatformInfo};
```

The only change is adding `util` to the `crate::cli` import group.

### 2b. Restructure `run()` to pass loaded config and add exit code (lines 62-234)

The `run()` function currently calls `check_config()` which loads the config internally and discards it. The new MCP check needs the loaded config. Restructure so that config loading happens in `run()` and the loaded config is passed to both `check_config()` and `check_mcp_servers()`.

**Current `run()` function (lines 62-234) -- replace entirely:**

**New `run()` function:**
```rust
/// Run the `great doctor` diagnostic command.
pub fn run(args: Args) -> Result<()> {
    if args.fix {
        output::info("Auto-fix mode enabled.");
        println!();
    }

    output::header("great doctor");
    println!();

    let mut result = DiagnosticResult::default();
    let info = platform::detect_platform_info();

    // 1. Platform check
    check_platform(&mut result);

    // 2. System prerequisites check
    check_system_prerequisites(&mut result, &info);

    // 3. Essential tools check
    check_essential_tools(&mut result);

    // 4. Docker check
    check_docker(&mut result, &info);

    // 5. AI agents check
    check_ai_agents(&mut result);

    // 6. Config check — load config here so it can be shared with MCP check
    let loaded_config = check_config(&mut result);

    // 7. MCP server checks (only if config was loaded successfully)
    if let Some(ref cfg) = loaded_config {
        check_mcp_servers(&mut result, cfg);
    }

    // 8. Shell check
    check_shell(&mut result);

    // 9. System tuning check (Linux/WSL only)
    check_system_tuning(&mut result, &info);

    // Attempt auto-fixes if --fix was passed
    if args.fix && !result.fixable.is_empty() {
        println!();
        output::header("Auto-fix");
        let managers = package_manager::available_managers();
        let mut fixed = 0;

        for issue in &result.fixable {
            output::info(&format!("Fixing: {}", issue.description));
            match &issue.action {
                FixAction::InstallTool { binary, brew_name } => {
                    let mut ok = false;
                    for mgr in &managers {
                        if mgr.install(brew_name, None).is_ok() && command_exists(binary) {
                            output::success(&format!(
                                "  {} — installed via {}",
                                binary,
                                mgr.name()
                            ));
                            ok = true;
                            fixed += 1;
                            break;
                        }
                    }
                    if !ok {
                        output::error(&format!("  {} — could not install", binary));
                    }
                }
                FixAction::InstallHomebrew => {
                    let status = std::process::Command::new("bash")
                        .args(["-c", "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            output::success("  Homebrew — installed");
                            fixed += 1;
                        }
                        _ => output::error("  Homebrew — install failed"),
                    }
                }
                FixAction::CreateClaudeDir => {
                    if let Some(home) = dirs::home_dir() {
                        let claude_dir = home.join(".claude");
                        match std::fs::create_dir_all(&claude_dir) {
                            Ok(()) => {
                                output::success("  ~/.claude/ — created");
                                fixed += 1;
                            }
                            Err(e) => output::error(&format!("  ~/.claude/ — failed: {}", e)),
                        }
                    }
                }
                FixAction::AddLocalBinToPath => {
                    if let Some(home) = dirs::home_dir() {
                        let shell = std::env::var("SHELL").unwrap_or_default();
                        let profile = if shell.contains("zsh") {
                            home.join(".zshrc")
                        } else {
                            home.join(".bashrc")
                        };
                        let line = "\n# Added by great doctor --fix\nexport PATH=\"$HOME/.local/bin:$PATH\"\n";
                        match std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&profile)
                        {
                            Ok(mut f) => {
                                use std::io::Write;
                                if f.write_all(line.as_bytes()).is_ok() {
                                    output::success(&format!(
                                        "  Added ~/.local/bin to PATH in {}",
                                        profile.display()
                                    ));
                                    fixed += 1;
                                }
                            }
                            Err(e) => output::error(&format!("  Failed: {}", e)),
                        }
                    }
                }
                FixAction::InstallSystemPrerequisite { name } => {
                    match name.as_str() {
                        "curl" => bootstrap::ensure_curl(false, &info.platform),
                        "git" => bootstrap::ensure_git(false, &info.platform),
                        "build-essential" => {
                            bootstrap::ensure_build_essential(false, &info.platform)
                        }
                        "unzip" => bootstrap::ensure_unzip(false, &info.platform),
                        _ => output::error(&format!("  Unknown prerequisite: {}", name)),
                    }
                    fixed += 1;
                }
                FixAction::InstallDocker => {
                    bootstrap::ensure_docker(false, &info);
                    fixed += 1;
                }
                FixAction::InstallClaudeCode => {
                    bootstrap::ensure_claude_code(false);
                    fixed += 1;
                }
                FixAction::FixInotifyWatches => {
                    tuning::apply_system_tuning(false, &info);
                    fixed += 1;
                }
            }
        }

        println!();
        output::info(&format!(
            "Fixed {} of {} issues.",
            fixed,
            result.fixable.len()
        ));
    }

    // Summary
    println!();
    output::header("Summary");
    output::info(&format!(
        "  {} passed, {} warnings, {} errors",
        result.checks_passed, result.checks_warned, result.checks_failed
    ));

    if result.checks_failed > 0 && !args.fix {
        println!();
        output::warning("Run `great doctor --fix` to attempt automatic fixes.");
    } else if result.checks_warned > 0 {
        println!();
        output::success("No critical issues found.");
    } else if result.checks_failed == 0 {
        println!();
        output::success("Environment is healthy!");
    }

    // NOTE: Intentional use of process::exit — the doctor command must print
    // its full report before exiting non-zero. Using bail!() would abort
    // mid-report, which is wrong for a diagnostic command.
    if result.checks_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
```

**Changes from current `run()`:**
1. Line storing `check_config` return value: `let loaded_config = check_config(&mut result);`
2. New section 7 inserted between config check and shell check: MCP server checks
3. Section numbers incremented: shell is now 8, system tuning is now 9
4. Added `process::exit(1)` block after summary (before `Ok(())`)
5. The auto-fix block is UNCHANGED -- copied verbatim from the existing implementation

### 2c. Change `check_config()` return type (line 456)

The function must now return the loaded config so `run()` can pass it to `check_mcp_servers()`.

**Current signature (line 456):**
```rust
fn check_config(result: &mut DiagnosticResult) {
```

**New signature:**
```rust
fn check_config(result: &mut DiagnosticResult) -> Option<config::GreatConfig> {
```

**Current body (lines 456-518) -- replace with:**
```rust
fn check_config(result: &mut DiagnosticResult) -> Option<config::GreatConfig> {
    output::header("Configuration");

    let loaded_config = match config::discover_config() {
        Ok(path) => {
            pass(result, &format!("great.toml: found at {}", path.display()));
            let path_str = match path.to_str() {
                Some(s) => s,
                None => {
                    fail(
                        result,
                        &format!(
                            "great.toml: path contains non-UTF-8 characters: {}",
                            path.display()
                        ),
                    );
                    println!();
                    return None;
                }
            };
            match config::load(Some(path_str)) {
                Ok(cfg) => {
                    pass(result, "great.toml: valid syntax");
                    let messages = cfg.validate();
                    for msg in &messages {
                        match msg {
                            config::schema::ConfigMessage::Warning(w) => {
                                warn(result, &format!("Config: {}", w));
                            }
                            config::schema::ConfigMessage::Error(e) => {
                                fail(result, &format!("Config: {}", e));
                            }
                        }
                    }
                    // Check secret references
                    let refs = cfg.find_secret_refs();
                    for secret_ref in &refs {
                        if std::env::var(secret_ref).is_ok() {
                            pass(result, &format!("Secret ${{{}}}: resolved", secret_ref));
                        } else {
                            fail(
                                result,
                                &format!("Secret ${{{}}}: not set in environment", secret_ref),
                            );
                        }
                    }
                    Some(cfg)
                }
                Err(e) => {
                    fail(result, &format!("great.toml: parse error — {}", e));
                    None
                }
            }
        }
        Err(_) => {
            warn(
                result,
                "great.toml: not found — run `great init` to create one",
            );
            None
        }
    };

    // Check Claude config directories
    let home = dirs::home_dir();
    if let Some(home) = &home {
        let claude_dir = home.join(".claude");
        if claude_dir.exists() {
            pass(result, "~/.claude/ directory: exists");
        } else {
            warn(result, "~/.claude/ directory: not found");
            result.fixable.push(FixableIssue {
                description: "Create ~/.claude/ directory".to_string(),
                action: FixAction::CreateClaudeDir,
            });
        }
    }

    println!();
    loaded_config
}
```

**Key changes:**
1. Returns `Option<config::GreatConfig>` instead of `()`
2. Replaces `path.to_str().unwrap_or_default()` with a `match` that fails gracefully on non-UTF-8 paths
3. The `Ok(cfg)` branch now returns `Some(cfg)` and the error branches return `None`
4. All existing validation logic (ConfigMessage checking, secret ref checking, Claude dir checking) is preserved verbatim

### 2d. Add new `check_mcp_servers()` function

Insert this function after `check_config()` (after line 518 in the current file, but after the rewritten `check_config` in the new version).

```rust
/// Check that MCP server commands declared in great.toml are available on PATH.
fn check_mcp_servers(result: &mut DiagnosticResult, cfg: &config::GreatConfig) {
    let mcps = match &cfg.mcp {
        Some(m) if !m.is_empty() => m,
        _ => return,
    };

    output::header("MCP Servers");

    for (name, mcp) in mcps {
        // Skip disabled servers
        if mcp.enabled == Some(false) {
            pass(result, &format!("{}: disabled (skipped)", name));
            continue;
        }

        if command_exists(&mcp.command) {
            let transport = mcp.transport.as_deref().unwrap_or("stdio");
            pass(
                result,
                &format!("{}: {} found [{}]", name, mcp.command, transport),
            );
        } else {
            fail(
                result,
                &format!(
                    "{}: command '{}' not found on PATH",
                    name, mcp.command
                ),
            );
        }
    }

    println!();
}
```

**Design notes:**
- Mirrors the MCP section pattern from `status.rs` (lines 228-262) but with doctor-style pass/fail reporting
- Uses `command_exists()` which is already imported
- Respects the `enabled` field (MCP servers with `enabled = false` are reported as skipped, not failed)
- Returns early with no header if there are no MCP servers declared (avoids empty section noise)
- No fixable issues are registered -- MCP server installation is complex and context-dependent

### 2e. Replace local `get_command_version()` with import (lines 362, 715-738)

**In `check_essential_tools()` at line 362:**

**Current:**
```rust
            let version = get_command_version(cmd);
```

**New:**
```rust
            let version = util::get_command_version(cmd);
```

**Delete the entire `get_command_version` function (lines 715-738):**

Remove:
```rust
/// Try to get a command's version string.
///
/// Runs `<cmd> --version` and returns the first line of stdout, or `None`
/// if the command fails or produces no output.
fn get_command_version(cmd: &str) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            None
        } else {
            Some(first_line.to_string())
        }
    } else {
        None
    }
}
```

---

## 3. Modify `src/cli/status.rs`

### 3a. Update imports (lines 5-7)

**Current:**
```rust
use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists};
```

**New:**
```rust
use crate::cli::{output, util};
use crate::config;
use crate::platform::{self, command_exists};
```

### 3b. Replace all calls to `get_tool_version()` with `util::get_command_version()`

There are 4 call sites in status.rs. All use `get_tool_version(name)`.

**Lines 182, 199, 320, 335 -- each occurrence:**

**Current:**
```rust
                    get_tool_version(name)
```

**New:**
```rust
                    util::get_command_version(name)
```

Use `replace_all` since the pattern `get_tool_version(name)` appears exactly 4 times and all should change.

### 3c. Delete the local `get_tool_version()` function (lines 432-455)

Remove:
```rust
/// Try to get a tool's version by running `<tool> --version`.
///
/// Returns the first non-empty line of stdout, trimmed. Returns `None` if the
/// command fails or produces no output.
fn get_tool_version(tool: &str) -> Option<String> {
    let output = std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            None
        } else {
            Some(first_line.to_string())
        }
    } else {
        None
    }
}
```

---

## 4. Integration Tests

### Modify: `tests/cli_smoke.rs`

All new tests go in the existing Doctor section. The file has two doctor test blocks: lines 92-117 and lines 296-318. Add the new tests after line 318.

**IMPORTANT:** After exit-code semantics are added, the existing `doctor_runs_diagnostics` test (line 95) may start failing if the test environment has any check failures (e.g., Homebrew not installed on a CI runner). The existing test asserts `.success()`. This is addressed in the build order below -- the builder must verify that the test runner environment does not cause spurious failures, or adjust the existing assertion to allow either exit code and check stderr content only.

### 4a. Test: doctor with valid config shows MCP section

```rust
#[test]
fn doctor_with_mcp_config_checks_servers() {
    let dir = TempDir::new().unwrap();
    // Write a great.toml with an MCP server whose command exists (ls is universal)
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.test-server]
command = "ls"
args = ["--help"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("MCP Servers"))
        .stderr(predicate::str::contains("test-server"));
}
```

### 4b. Test: doctor with MCP server missing command reports failure

```rust
#[test]
fn doctor_mcp_missing_command_fails() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.broken]
command = "nonexistent_command_xyz_99999"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("not found on PATH"));
}
```

### 4c. Test: doctor exit code 1 when failures present

```rust
#[test]
fn doctor_exits_nonzero_on_failure() {
    let dir = TempDir::new().unwrap();
    // Write a config with an MCP server that has a nonexistent command.
    // This guarantees at least one check_failed.
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.broken]
command = "nonexistent_command_xyz_99999"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Summary"));
}
```

### 4d. Test: doctor with valid config shows config section

```rust
#[test]
fn doctor_with_valid_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test-project"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("great.toml: found at"))
        .stderr(predicate::str::contains("great.toml: valid syntax"));
}
```

### 4e. Update existing `doctor_runs_diagnostics` test

The existing test at line 95 asserts `.success()`. After exit-code semantics are added, the doctor may return exit code 1 if the test environment is missing tools (e.g., Homebrew not installed). The test runs in a `TempDir` with no `great.toml`, so MCP checks will not trigger. However, other checks (Homebrew, Docker) may fail depending on the CI environment.

**Decision:** Keep the `.success()` assertion. On dev machines and CI runners where Rust is installed (required to run `cargo test`), the checks that produce failures are: Homebrew (may be absent on some Linux CI), Docker (may be absent). These are already `fail` in the current code but the function returns `Ok(())`. After this change, the test will fail on environments missing Homebrew.

**Mitigation strategy:** The builder should verify on the target CI environment. If Homebrew is reliably absent, the test should be changed to not assert a specific exit code, or should assert only stderr content. The simplest approach:

```rust
#[test]
fn doctor_runs_diagnostics() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("doctor")
        .assert()
        .stderr(predicate::str::contains("Platform"))
        .stderr(predicate::str::contains("Essential Tools"))
        .stderr(predicate::str::contains("Summary"));
}
```

Remove the `.success()` assertion so the test validates output content regardless of exit code. The exit code is tested explicitly in `doctor_exits_nonzero_on_failure`.

---

## 5. Build Order

Execute these changes in this exact sequence. Each step must compile and pass `cargo clippy` before proceeding to the next.

### Step 1: Create `src/cli/util.rs` and register it in `src/cli/mod.rs`

1. Create `/home/isaac/src/sh.great/src/cli/util.rs` with the `get_command_version` function
2. Add `pub mod util;` to `/home/isaac/src/sh.great/src/cli/mod.rs`
3. Verify: `cargo build` succeeds (the new module is not yet imported anywhere, which is fine -- clippy will warn about dead code but not error)

### Step 2: Update `src/cli/status.rs` to use shared utility

1. Change import line from `use crate::cli::output;` to `use crate::cli::{output, util};`
2. Replace all 4 occurrences of `get_tool_version(name)` with `util::get_command_version(name)`
3. Delete the `get_tool_version()` function (lines 432-455)
4. Verify: `cargo build && cargo clippy`

### Step 3: Update `src/cli/doctor.rs` -- imports and shared utility

1. Add `util` to the imports: `use crate::cli::{bootstrap, output, tuning, util};`
2. Change `get_command_version(cmd)` call in `check_essential_tools()` to `util::get_command_version(cmd)`
3. Delete the local `get_command_version()` function (lines 715-738)
4. Verify: `cargo build && cargo clippy`

### Step 4: Update `src/cli/doctor.rs` -- fix `check_config()` return type and path safety

1. Change `check_config` signature to return `Option<config::GreatConfig>`
2. Replace `path.to_str().unwrap_or_default()` with the `match` pattern
3. Return `Some(cfg)` on success, `None` on error
4. Update `run()` to capture the return: `let loaded_config = check_config(&mut result);`
5. Verify: `cargo build && cargo clippy`

### Step 5: Add `check_mcp_servers()` and wire into `run()`

1. Add the `check_mcp_servers()` function after `check_config()`
2. Add the call in `run()`: section 7, after config check, before shell check
3. Verify: `cargo build && cargo clippy`

### Step 6: Add exit code semantics to `run()`

1. Add `process::exit(1)` block after summary in `run()`
2. Verify: `cargo build && cargo clippy`

### Step 7: Update and add integration tests

1. Remove `.success()` from the existing `doctor_runs_diagnostics` test
2. Add the 4 new tests
3. Verify: `cargo test`

---

## 6. Edge Cases

### Non-UTF-8 config path
- **Before:** `path.to_str().unwrap_or_default()` silently passes an empty string to `config::load`, causing a confusing "file not found" error for `""`
- **After:** Reports a clear failure message including the offending path via `path.display()`, returns `None`

### No `great.toml` present
- `check_config` returns `None`, `check_mcp_servers` is skipped entirely (no "MCP Servers" header printed)
- Existing behavior preserved: warning message "run `great init` to create one"

### Config has no MCP section
- `check_mcp_servers` receives a `GreatConfig` with `mcp: None`, early-returns without printing any header
- No noise in output for projects that do not use MCP servers

### Config has MCP section but all servers disabled
- Each disabled server prints a pass line: "server-name: disabled (skipped)"
- No failures registered for disabled servers

### Config has MCP section with empty map
- The `!m.is_empty()` guard causes early return -- no header printed

### Exit code when `--fix` resolves all failures
- After auto-fix runs, the `result.checks_failed` counter still reflects the original scan, not post-fix state
- This means `great doctor --fix` will still exit 1 if there were failures, even if fixes succeeded
- This is intentional: the user should re-run `great doctor` after fixing to confirm a clean bill of health
- This matches the pattern used by `rustup check`, `brew doctor`, and `npm doctor`

### Concurrent access
- Not applicable. `great doctor` is a read-only diagnostic command (except `--fix`). No file locks needed.

### Platform differences
- **macOS ARM64/x86_64:** All checks work identically. Homebrew path differs (`/opt/homebrew` vs `/usr/local`) but `command_exists` uses `which` crate which checks `$PATH`
- **Ubuntu:** Docker check offers apt-based auto-fix. System tuning check runs (inotify)
- **WSL2:** Docker check suggests Docker Desktop for Windows. System tuning check runs. No Docker auto-fix offered (WSL uses Windows Docker Desktop)

---

## 7. Error Handling

| Scenario | Handling | Message |
|---|---|---|
| `path.to_str()` returns `None` | `fail()` + return `None` | "great.toml: path contains non-UTF-8 characters: {path}" |
| Config parse fails | `fail()` + return `None` | "great.toml: parse error -- {error}" |
| MCP command not on PATH | `fail()` | "{name}: command '{cmd}' not found on PATH" |
| No config found | `warn()` | "great.toml: not found -- run `great init` to create one" |
| MCP server disabled | `pass()` | "{name}: disabled (skipped)" |
| Any check fails | `process::exit(1)` after summary | Exit code 1 |

All output goes to stderr (via the `output::*` helpers which use `eprintln!`). This is the existing convention.

---

## 8. Security Considerations

- **No secrets in output:** MCP commands are printed but arguments and environment variables are not included in the doctor output. The status command shows args in verbose mode, but doctor does not need to -- it only checks command availability.
- **`--fix` safety:** The auto-fix logic is already implemented and conservative. This spec does not modify any fix actions. The MCP check does not register fixable issues.
- **PATH traversal:** `command_exists` uses the `which` crate, which resolves against `$PATH`. No user-controlled paths are passed to `std::process::Command` beyond what is declared in `great.toml`.

---

## 9. Testing Strategy

### Unit tests
None needed. The `get_command_version` function is already tested indirectly by integration tests (it is called for every essential tool check). The function is trivial (spawn process, read stdout) and not worth mocking.

### Integration tests (assert_cmd)

| Test | Purpose | Exit code | Key stderr assertion |
|---|---|---|---|
| `doctor_runs_diagnostics` (existing, modified) | Smoke test -- sections appear | any | "Platform", "Essential Tools", "Summary" |
| `doctor_fix_runs_without_crash` (existing, unchanged) | `--fix` flag accepted | success | "Auto-fix mode" |
| `doctor_checks_system_prerequisites` (existing, unchanged) | Section appears | success | "System Prerequisites" |
| `doctor_checks_docker` (existing, unchanged) | Section appears | success | "Docker" |
| `doctor_with_valid_config` (new) | Config found and parsed | any | "great.toml: found at", "great.toml: valid syntax" |
| `doctor_with_mcp_config_checks_servers` (new) | MCP section appears with valid command | any | "MCP Servers", "test-server" |
| `doctor_mcp_missing_command_fails` (new) | Missing MCP command reported | any | "not found on PATH" |
| `doctor_exits_nonzero_on_failure` (new) | Exit code 1 on failure | failure | "Summary" |

### Verification commands

```bash
# Build check
cargo build 2>&1

# Lint check (zero warnings required)
cargo clippy -- -D warnings 2>&1

# Run all tests
cargo test 2>&1

# Run only doctor tests
cargo test doctor 2>&1

# Manual smoke test
cargo run -- doctor
echo "Exit code: $?"

# Manual smoke test with config
cd /home/isaac/src/sh.great && cargo run -- doctor
echo "Exit code: $?"
```

---

## 10. Acceptance Criteria

- [ ] `cargo build` succeeds with zero errors
- [ ] `cargo clippy -- -D warnings` produces zero warnings for all modified files
- [ ] `get_command_version()` exists in exactly one location: `src/cli/util.rs`
- [ ] `src/cli/doctor.rs` imports and calls `util::get_command_version()` -- no local copy
- [ ] `src/cli/status.rs` imports and calls `util::get_command_version()` -- no local copy (`get_tool_version` deleted)
- [ ] `check_config()` does not use `unwrap_or_default()` -- uses `match` with explicit error path
- [ ] When `great.toml` declares MCP servers, `great doctor` prints "MCP Servers" header and reports each server's command availability
- [ ] Disabled MCP servers (`enabled = false`) are reported as "disabled (skipped)", not failed
- [ ] `great doctor` returns exit code 0 when `checks_failed == 0`
- [ ] `great doctor` returns exit code 1 when `checks_failed > 0`
- [ ] All 8 integration tests pass (4 existing + 4 new)
- [ ] The `--fix` auto-fix logic is unchanged (no functional modifications to fix actions)

---

## Appendix: File Change Summary

| File | Action | Lines added | Lines removed |
|---|---|---|---|
| `src/cli/util.rs` | CREATE | ~28 | 0 |
| `src/cli/mod.rs` | MODIFY (add 1 line) | 1 | 0 |
| `src/cli/doctor.rs` | MODIFY (imports, check_config return, add check_mcp_servers, exit code, delete local fn) | ~50 | ~30 |
| `src/cli/status.rs` | MODIFY (imports, replace calls, delete local fn) | ~2 | ~26 |
| `tests/cli_smoke.rs` | MODIFY (update 1 test, add 4 tests) | ~75 | ~1 |
| **Total** | | **~156** | **~57** |
