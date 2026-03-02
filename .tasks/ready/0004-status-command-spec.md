# 0004: Complete `great status` Command -- Technical Specification

**Task:** 0004-status-command
**Status:** ready
**Module:** `src/cli/status.rs` (216 lines currently)
**Dependencies:** Tasks 0001, 0002, 0003 (all landed)
**Forward dependency:** Task 0005 will extract `get_tool_version()` to shared util; do NOT move it in this task.

---

## 1. Summary

The `great status` command is substantially implemented but has five gaps: (1) `--json` output emits only platform/arch/shell as a hand-formatted string, (2) `--verbose` only enriches the platform section, (3) `path.to_str().unwrap_or_default()` on line 70 is unsafe for non-UTF-8 paths, (4) no exit code semantics for CI usage, and (5) integration test coverage is minimal. This spec addresses all five.

---

## 2. Files to Modify

| File | Action | Lines affected |
|------|--------|----------------|
| `src/cli/status.rs` | Modify | All -- restructure `run()`, replace `run_json()`, add structs |
| `tests/cli_smoke.rs` | Modify | Append new status tests at end of file |

No new files. No Cargo.toml changes (`serde_json` is already a dependency).

---

## 3. New Struct Definitions

Add the following serialization structs at the top of `src/cli/status.rs`, after the existing `use` statements (after line 6). These drive both the `--json` output and the internal issue-tracking for exit codes.

```rust
use serde::Serialize;
use std::collections::HashMap;

/// Top-level JSON output for `great status --json`.
#[derive(Serialize)]
struct StatusReport {
    /// Platform identifier string, e.g. "macos", "linux", "wsl".
    platform: String,
    /// CPU architecture, e.g. "X86_64", "Aarch64".
    arch: String,
    /// User's shell, e.g. "/bin/zsh".
    shell: String,
    /// Whether the current user is root.
    is_root: bool,
    /// Path to the discovered great.toml, or null if none found.
    config_path: Option<String>,
    /// Tool statuses. Absent when no config loaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolStatus>>,
    /// MCP server statuses. Absent when no config loaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp: Option<Vec<McpStatus>>,
    /// Agent declarations. Absent when no config loaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    agents: Option<Vec<AgentStatus>>,
    /// Secret statuses. Absent when no config loaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Vec<SecretStatus>>,
    /// Whether any critical issues were found.
    has_issues: bool,
    /// List of critical issue descriptions.
    issues: Vec<String>,
}

/// Status of a single declared tool.
#[derive(Serialize)]
struct ToolStatus {
    name: String,
    declared_version: String,
    installed: bool,
    /// Detected version string from `--version`, if installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_version: Option<String>,
}

/// Status of a single MCP server declaration.
#[derive(Serialize)]
struct McpStatus {
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    command_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
}

/// Status of a single agent declaration.
#[derive(Serialize)]
struct AgentStatus {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

/// Status of a single required secret.
#[derive(Serialize)]
struct SecretStatus {
    name: String,
    is_set: bool,
}
```

---

## 4. Modified Function Signatures

### 4.1. `pub fn run(args: Args) -> Result<()>`

**Current signature (line 26):** `pub fn run(args: Args) -> Result<()>`
**New signature:** Same -- `pub fn run(args: Args) -> Result<()>`

The return type does NOT change. For exit code semantics, the function will call `std::process::exit(1)` at the very end when critical issues are found and `--json` is NOT active. This matches the pattern used throughout the crate (anyhow errors in `main` already exit 1, but `status` deliberately does not bail on warnings -- it prints the full report, then exits non-zero).

**Rationale for `process::exit(1)` instead of `bail!()`:** The status command must print its full report (all sections) even when issues exist. A `bail!()` would abort mid-report and print an error message to stderr, which is wrong for a diagnostic command. The `process::exit(1)` at the end of `run()` is the correct pattern: finish printing everything, then signal failure to the caller's exit code.

### 4.2. `fn run_json(info: &platform::PlatformInfo) -> Result<()>`

**Current signature (line 164):** `fn run_json(info: &platform::PlatformInfo) -> Result<()>`
**New signature:** `fn run_json(info: &platform::PlatformInfo, config_path: Option<&str>, config: Option<&crate::config::GreatConfig>) -> Result<()>`

The `--json` path needs access to the loaded config. Config discovery and loading will be hoisted out of the human-readable path so both branches can share it.

### 4.3. `fn get_tool_version(tool: &str) -> Option<String>` -- NO CHANGE

Stays in place per task boundary constraint. Task 0005 will extract it.

### 4.4. `fn print_tool_status(...)` -- MINOR CHANGE

**Current signature (line 203-208):**
```rust
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
)
```

**New signature:**
```rust
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
    verbose: bool,
)
```

When `verbose` is true AND installed, display the full `actual_version` string. When `verbose` is false, display only a short version (first word/token).

---

## 5. Implementation Approach and Build Order

Build in this exact sequence. Each step produces a compilable, testable state.

### Step 1: Add serialization structs (lines 1-7 area)

Insert the `use serde::Serialize;` import and all six `#[derive(Serialize)]` structs from Section 3 above, between the existing `use` block and the `Args` struct.

Verification: `cargo build` succeeds, `cargo clippy` clean.

### Step 2: Fix the `unwrap_or_default()` path issue (line 70)

**Current code (lines 67-76):**
```rust
let config = match config::discover_config() {
    Ok(path) => {
        output::info(&format!("Config: {}", path.display()));
        match config::load(Some(path.to_str().unwrap_or_default())) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                output::error(&format!("Failed to parse config: {}", e));
                None
            }
        }
    }
    Err(_) => {
        output::warning("No great.toml found. Run `great init` to create one.");
        None
    }
};
```

**New code:**
```rust
let (config_path_str, config) = match config::discover_config() {
    Ok(path) => {
        output::info(&format!("Config: {}", path.display()));
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!(
                "config path contains non-UTF-8 characters: {}",
                path.display()
            ))?;
        let path_owned = path_str.to_string();
        match config::load(Some(&path_owned)) {
            Ok(cfg) => (Some(path_owned), Some(cfg)),
            Err(e) => {
                output::error(&format!("Failed to parse config: {}", e));
                (Some(path_owned), None)
            }
        }
    }
    Err(_) => {
        output::warning("No great.toml found. Run `great init` to create one.");
        (None, None)
    }
};
```

This replaces `unwrap_or_default()` with `?`-propagated error. The `config_path_str` is captured for use in both the human-readable and JSON paths.

**Edge case -- non-UTF-8 paths:** This can only occur on Linux/WSL with intentionally crafted directory names. The error message includes the lossy display of the path so the user can fix it. On macOS (HFS+ normalizes to UTF-8) and Windows (paths are UTF-16 which always converts to UTF-8), this cannot trigger.

Verification: `cargo build`, `cargo clippy`, existing tests still pass.

### Step 3: Hoist config discovery above the `--json` branch

Currently, `run_json()` is called on line 29-31 before config is loaded. Restructure `run()` so that config discovery happens first, then either branch receives the result.

**New `run()` structure:**
```rust
pub fn run(args: Args) -> Result<()> {
    let info = platform::detect_platform_info();

    // -- Discover and load config (shared by both output modes) --------
    let (config_path_str, config) = match config::discover_config() {
        Ok(path) => {
            let path_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!(
                    "config path contains non-UTF-8 characters: {}",
                    path.display()
                ))?;
            let path_owned = path_str.to_string();
            match config::load(Some(&path_owned)) {
                Ok(cfg) => (Some(path_owned), Some(cfg)),
                Err(e) => {
                    if !args.json {
                        output::error(&format!("Failed to parse config: {}", e));
                    }
                    (Some(path_owned), None)
                }
            }
        }
        Err(_) => (None, None),
    };

    // -- JSON mode: serialize and exit (always exit 0) -----------------
    if args.json {
        return run_json(&info, config_path_str.as_deref(), config.as_ref());
    }

    // -- Human-readable mode continues below ---------------------------
    output::header("great status");
    println!();

    // ... (rest of human-readable output, refactored as described below)
}
```

### Step 4: Implement expanded `run_json()`

**Replace the entire `run_json()` function (lines 163-172):**

```rust
/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
fn run_json(
    info: &platform::PlatformInfo,
    config_path: Option<&str>,
    config: Option<&crate::config::GreatConfig>,
) -> Result<()> {
    let mut issues = Vec::new();

    let tools = config.and_then(|cfg| {
        cfg.tools.as_ref().map(|t| {
            let mut result = Vec::new();
            for (name, version) in &t.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = platform::command_exists(name);
                let actual_version = if installed {
                    get_tool_version(name)
                } else {
                    issues.push(format!("tool '{}' is not installed", name));
                    None
                };
                result.push(ToolStatus {
                    name: name.clone(),
                    declared_version: version.clone(),
                    installed,
                    actual_version,
                });
            }
            if let Some(cli_tools) = &t.cli {
                for (name, version) in cli_tools {
                    let installed = platform::command_exists(name);
                    let actual_version = if installed {
                        get_tool_version(name)
                    } else {
                        issues.push(format!("tool '{}' is not installed", name));
                        None
                    };
                    result.push(ToolStatus {
                        name: name.clone(),
                        declared_version: version.clone(),
                        installed,
                        actual_version,
                    });
                }
            }
            result
        })
    });

    let mcp = config.and_then(|cfg| {
        cfg.mcp.as_ref().map(|mcps| {
            mcps.iter()
                .map(|(name, m)| McpStatus {
                    name: name.clone(),
                    command: m.command.clone(),
                    args: m.args.clone(),
                    command_available: platform::command_exists(&m.command),
                    transport: m.transport.clone(),
                })
                .collect()
        })
    });

    let agents = config.and_then(|cfg| {
        cfg.agents.as_ref().map(|a| {
            a.iter()
                .map(|(name, agent)| AgentStatus {
                    name: name.clone(),
                    provider: agent.provider.clone(),
                    model: agent.model.clone(),
                })
                .collect()
        })
    });

    let secrets = config.and_then(|cfg| {
        cfg.secrets.as_ref().and_then(|s| {
            s.required.as_ref().map(|required| {
                required
                    .iter()
                    .map(|key| {
                        let is_set = std::env::var(key).is_ok();
                        if !is_set {
                            issues.push(format!("required secret '{}' is missing", key));
                        }
                        SecretStatus {
                            name: key.clone(),
                            is_set,
                        }
                    })
                    .collect()
            })
        })
    });

    let report = StatusReport {
        platform: info.platform.to_string(),
        arch: format!("{:?}", info.platform.arch()),
        shell: info.shell.clone(),
        is_root: info.is_root,
        config_path: config_path.map(|s| s.to_string()),
        has_issues: !issues.is_empty(),
        issues,
        tools,
        mcp,
        agents,
        secrets,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
```

**Key decisions:**
- `--json` ALWAYS exits 0. Issues are encoded in `has_issues` and `issues` fields.
- Pretty-printed JSON for human readability; callers can pipe through `jq` if they want compact.
- Uses `serde_json::to_string_pretty()` which is already available (serde_json 1.0 in Cargo.toml).

### Step 5: Expand verbose mode in the human-readable path

Modify the tools section (lines 86-116 area) to pass `args.verbose` through to `print_tool_status()`:

```rust
print_tool_status(name, version, installed, actual_version.as_deref(), args.verbose);
```

Modify `print_tool_status()` to handle verbose:

```rust
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
    verbose: bool,
) {
    if installed {
        let ver_info = match (actual_version, verbose) {
            (Some(full), true) => full,
            (Some(full), false) => full.split_whitespace().last().unwrap_or(full),
            (None, _) => "installed",
        };
        output::success(&format!("  {} {} ({})", name, declared_version, ver_info));
    } else {
        output::error(&format!("  {} {} -- not installed", name, declared_version));
    }
}
```

Modify the MCP servers section (lines 130-141 area) for verbose output:

```rust
// -- MCP Servers section --------------------------------------------
if let Some(mcps) = &cfg.mcp {
    println!();
    output::header("MCP Servers");
    for (name, mcp) in mcps {
        let cmd_available = command_exists(&mcp.command);
        if cmd_available {
            if args.verbose {
                let args_str = mcp
                    .args
                    .as_ref()
                    .map(|a| a.join(" "))
                    .unwrap_or_default();
                let transport = mcp.transport.as_deref().unwrap_or("stdio");
                output::success(&format!(
                    "  {} ({} {} [{}])",
                    name, mcp.command, args_str, transport
                ));
            } else {
                output::success(&format!("  {} ({})", name, mcp.command));
            }
        } else {
            output::error(&format!("  {} ({} -- not found)", name, mcp.command));
        }
    }
}
```

### Step 6: Add exit code semantics

At the end of the human-readable path (just before the final `Ok(())`), track issues and exit non-zero when critical issues are found.

**Insert issue tracking throughout the human-readable path:**

Declare a mutable `issues` counter at the top of the human-readable section:

```rust
let mut has_critical_issues = false;
```

In the tools loop, when a tool is not installed:
```rust
if !installed {
    has_critical_issues = true;
}
```

In the secrets loop, when a required secret is missing:
```rust
if std::env::var(key).is_err() {
    has_critical_issues = true;
    // ... existing output::error call
}
```

At the very end of `run()`, after all output is printed:
```rust
println!();

if has_critical_issues {
    std::process::exit(1);
}

Ok(())
```

**Exit code contract:**
| Condition | Exit code |
|-----------|-----------|
| No config found | 0 (not an error -- informational) |
| Config found, all OK | 0 |
| Config found, declared tools missing | 1 |
| Config found, required secrets missing | 1 |
| Config parse error | 0 (error printed, continues with platform-only info) |
| `--json` mode, any condition | Always 0 (issues in payload) |

**Rationale:** Missing tools and missing required secrets are _actionable_ issues that a CI pipeline should catch. A missing config file is not an error -- `great status` is useful for checking the platform alone. Config parse errors are treated as non-fatal because the command still reports useful platform information.

### Step 7: Add integration tests

Append the following tests to `/home/isaac/src/sh.great/tests/cli_smoke.rs`, in a new `// Status -- expanded` section.

```rust
// -----------------------------------------------------------------------
// Status -- expanded (task 0004)
// -----------------------------------------------------------------------

#[test]
fn status_with_valid_config_exits_ok() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("Config:"));
}

#[test]
fn status_verbose_accepted() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", "--verbose"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform:"));
}

#[test]
fn status_verbose_short_flag_accepted() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .args(["status", "-v"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Platform:"));
}

#[test]
fn status_json_valid_json() {
    let dir = TempDir::new().unwrap();
    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Must parse as valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // Must contain required top-level keys
    assert!(parsed.get("platform").is_some());
    assert!(parsed.get("arch").is_some());
    assert!(parsed.get("shell").is_some());
    assert!(parsed.get("has_issues").is_some());
    assert!(parsed.get("issues").is_some());
}

#[test]
fn status_json_no_config_still_valid() {
    let dir = TempDir::new().unwrap();
    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // config_path should be null
    assert!(parsed.get("config_path").unwrap().is_null());
    // tools/mcp/agents/secrets should be absent or null
    assert!(
        parsed.get("tools").is_none() || parsed.get("tools").unwrap().is_null()
    );
}

#[test]
fn status_json_with_config_includes_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz = "latest"
"#,
    )
    .unwrap();

    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    // --json always exits 0
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    // tools array should exist and contain our tool
    let tools = parsed.get("tools").unwrap().as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "nonexistent-tool-xyz"));
    assert!(tools.iter().any(|t| t["installed"] == false));
    // has_issues should be true (tool not installed)
    assert_eq!(parsed["has_issues"], true);
}

#[test]
fn status_json_with_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_TEST_SECRET_PRESENT", "GREAT_TEST_SECRET_MISSING"]
"#,
    )
    .unwrap();

    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .env("GREAT_TEST_SECRET_PRESENT", "value")
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    let secrets = parsed.get("secrets").unwrap().as_array().unwrap();
    let present = secrets
        .iter()
        .find(|s| s["name"] == "GREAT_TEST_SECRET_PRESENT")
        .unwrap();
    assert_eq!(present["is_set"], true);
    let missing = secrets
        .iter()
        .find(|s| s["name"] == "GREAT_TEST_SECRET_MISSING")
        .unwrap();
    assert_eq!(missing["is_set"], false);
}

#[test]
fn status_exit_code_nonzero_missing_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn status_exit_code_nonzero_missing_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing"));
}

#[test]
fn status_json_always_exits_zero_even_with_issues() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    // --json must exit 0 even when there are issues
    great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success();
}

#[test]
fn status_no_config_exits_zero() {
    let dir = TempDir::new().unwrap();
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success();
}

#[test]
fn status_verbose_with_config_shows_capabilities() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .args(["status", "--verbose"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Shell:"));
}
```

**Note on `serde_json` in dev-dependencies:** The test `status_json_valid_json` and several others call `serde_json::from_str()`. The `serde_json` crate is already in `[dependencies]` in Cargo.toml (version 1.0), so it is automatically available in integration tests. No `[dev-dependencies]` change is needed.

---

## 6. Complete Rewritten `src/cli/status.rs`

For absolute clarity, this is the full intended file content. The builder should use this as the reference implementation.

```rust
use anyhow::Result;
use clap::Args as ClapArgs;
use serde::Serialize;

use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists};

// ---------------------------------------------------------------------------
// JSON serialization structs
// ---------------------------------------------------------------------------

/// Top-level JSON output for `great status --json`.
#[derive(Serialize)]
struct StatusReport {
    platform: String,
    arch: String,
    shell: String,
    is_root: bool,
    config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp: Option<Vec<McpStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agents: Option<Vec<AgentStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Vec<SecretStatus>>,
    has_issues: bool,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct ToolStatus {
    name: String,
    declared_version: String,
    installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_version: Option<String>,
}

#[derive(Serialize)]
struct McpStatus {
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    command_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
}

#[derive(Serialize)]
struct AgentStatus {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Serialize)]
struct SecretStatus {
    name: String,
    is_set: bool,
}

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

/// Arguments for the `great status` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Show detailed status information
    #[arg(long, short)]
    pub verbose: bool,

    /// Output status as JSON
    #[arg(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the `great status` command.
pub fn run(args: Args) -> Result<()> {
    let info = platform::detect_platform_info();

    // -- Discover and load config (shared by both output modes) ---------
    let (config_path_str, config) = match config::discover_config() {
        Ok(path) => {
            let path_str = path
                .to_str()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "config path contains non-UTF-8 characters: {}",
                        path.display()
                    )
                })?;
            let path_owned = path_str.to_string();
            match config::load(Some(&path_owned)) {
                Ok(cfg) => (Some(path_owned), Some(cfg)),
                Err(e) => {
                    if !args.json {
                        output::error(&format!("Failed to parse config: {}", e));
                    }
                    (Some(path_owned), None)
                }
            }
        }
        Err(_) => (None, None),
    };

    // -- JSON mode: serialize and exit (always exit 0) ------------------
    if args.json {
        return run_json(&info, config_path_str.as_deref(), config.as_ref());
    }

    // -- Human-readable mode --------------------------------------------
    output::header("great status");
    println!();

    let mut has_critical_issues = false;

    // Platform section
    output::info(&format!("Platform: {}", info.platform.display_detailed()));
    if args.verbose {
        let caps = &info.capabilities;
        let mut cap_list = Vec::new();
        if caps.has_homebrew {
            cap_list.push("homebrew");
        }
        if caps.has_apt {
            cap_list.push("apt");
        }
        if caps.has_dnf {
            cap_list.push("dnf");
        }
        if caps.has_snap {
            cap_list.push("snap");
        }
        if caps.has_systemd {
            cap_list.push("systemd");
        }
        if caps.has_docker {
            cap_list.push("docker");
        }
        if !cap_list.is_empty() {
            output::info(&format!("Capabilities: {}", cap_list.join(", ")));
        }
        output::info(&format!("Shell: {}", info.shell));
        output::info(&format!("Root: {}", info.is_root));
    }

    // Config section
    if config_path_str.is_some() {
        output::info(&format!(
            "Config: {}",
            config_path_str.as_deref().unwrap_or("unknown")
        ));
    } else {
        output::warning("No great.toml found. Run `great init` to create one.");
    }

    if let Some(cfg) = &config {
        // Tools section
        if let Some(tools) = &cfg.tools {
            println!();
            output::header("Tools");

            for (name, version) in &tools.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = command_exists(name);
                let actual_version = if installed {
                    get_tool_version(name)
                } else {
                    has_critical_issues = true;
                    None
                };
                print_tool_status(
                    name,
                    version,
                    installed,
                    actual_version.as_deref(),
                    args.verbose,
                );
            }

            if let Some(cli_tools) = &tools.cli {
                for (name, version) in cli_tools {
                    let installed = command_exists(name);
                    let actual_version = if installed {
                        get_tool_version(name)
                    } else {
                        has_critical_issues = true;
                        None
                    };
                    print_tool_status(
                        name,
                        version,
                        installed,
                        actual_version.as_deref(),
                        args.verbose,
                    );
                }
            }
        }

        // Agents section
        if let Some(agents) = &cfg.agents {
            println!();
            output::header("Agents");
            for (name, agent) in agents {
                let provider = agent.provider.as_deref().unwrap_or("unknown");
                let model = agent.model.as_deref().unwrap_or("default");
                output::info(&format!("  {} ({}/{})", name, provider, model));
            }
        }

        // MCP Servers section
        if let Some(mcps) = &cfg.mcp {
            println!();
            output::header("MCP Servers");
            for (name, mcp) in mcps {
                let cmd_available = command_exists(&mcp.command);
                if cmd_available {
                    if args.verbose {
                        let args_str = mcp
                            .args
                            .as_ref()
                            .map(|a| a.join(" "))
                            .unwrap_or_default();
                        let transport = mcp.transport.as_deref().unwrap_or("stdio");
                        output::success(&format!(
                            "  {} ({} {} [{}])",
                            name, mcp.command, args_str, transport
                        ));
                    } else {
                        output::success(&format!("  {} ({})", name, mcp.command));
                    }
                } else {
                    output::error(&format!(
                        "  {} ({} -- not found)",
                        name, mcp.command
                    ));
                }
            }
        }

        // Secrets section
        if let Some(secrets) = &cfg.secrets {
            if let Some(required) = &secrets.required {
                println!();
                output::header("Secrets");
                for key in required {
                    if std::env::var(key).is_ok() {
                        output::success(&format!("  {} -- set", key));
                    } else {
                        has_critical_issues = true;
                        output::error(&format!("  {} -- missing", key));
                    }
                }
            }
        }
    }

    println!();

    if has_critical_issues {
        std::process::exit(1);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
fn run_json(
    info: &platform::PlatformInfo,
    config_path: Option<&str>,
    config: Option<&config::GreatConfig>,
) -> Result<()> {
    let mut issues = Vec::new();

    let tools = config.and_then(|cfg| {
        cfg.tools.as_ref().map(|t| {
            let mut result = Vec::new();
            for (name, version) in &t.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = command_exists(name);
                let actual_version = if installed {
                    get_tool_version(name)
                } else {
                    issues.push(format!("tool '{}' is not installed", name));
                    None
                };
                result.push(ToolStatus {
                    name: name.clone(),
                    declared_version: version.clone(),
                    installed,
                    actual_version,
                });
            }
            if let Some(cli_tools) = &t.cli {
                for (name, version) in cli_tools {
                    let installed = command_exists(name);
                    let actual_version = if installed {
                        get_tool_version(name)
                    } else {
                        issues.push(format!("tool '{}' is not installed", name));
                        None
                    };
                    result.push(ToolStatus {
                        name: name.clone(),
                        declared_version: version.clone(),
                        installed,
                        actual_version,
                    });
                }
            }
            result
        })
    });

    let mcp = config.and_then(|cfg| {
        cfg.mcp.as_ref().map(|mcps| {
            mcps.iter()
                .map(|(name, m)| McpStatus {
                    name: name.clone(),
                    command: m.command.clone(),
                    args: m.args.clone(),
                    command_available: command_exists(&m.command),
                    transport: m.transport.clone(),
                })
                .collect()
        })
    });

    let agents = config.and_then(|cfg| {
        cfg.agents.as_ref().map(|a| {
            a.iter()
                .map(|(name, agent)| AgentStatus {
                    name: name.clone(),
                    provider: agent.provider.clone(),
                    model: agent.model.clone(),
                })
                .collect()
        })
    });

    let secrets = config.and_then(|cfg| {
        cfg.secrets.as_ref().and_then(|s| {
            s.required.as_ref().map(|required| {
                required
                    .iter()
                    .map(|key| {
                        let is_set = std::env::var(key).is_ok();
                        if !is_set {
                            issues.push(format!(
                                "required secret '{}' is missing",
                                key
                            ));
                        }
                        SecretStatus {
                            name: key.clone(),
                            is_set,
                        }
                    })
                    .collect()
            })
        })
    });

    let report = StatusReport {
        platform: info.platform.to_string(),
        arch: format!("{:?}", info.platform.arch()),
        shell: info.shell.clone(),
        is_root: info.is_root,
        config_path: config_path.map(|s| s.to_string()),
        has_issues: !issues.is_empty(),
        issues,
        tools,
        mcp,
        agents,
        secrets,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Print a single tool's status line with color coding.
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
    verbose: bool,
) {
    if installed {
        let ver_info = match (actual_version, verbose) {
            (Some(full), true) => full,
            (Some(full), false) => full.split_whitespace().last().unwrap_or(full),
            (None, _) => "installed",
        };
        output::success(&format!(
            "  {} {} ({})",
            name, declared_version, ver_info
        ));
    } else {
        output::error(&format!(
            "  {} {} -- not installed",
            name, declared_version
        ));
    }
}
```

---

## 7. Edge Cases

### 7.1. Empty inputs
- **No config file:** Both modes handle gracefully. Human-readable prints warning and exits 0. JSON outputs `config_path: null`, all optional sections absent, `has_issues: false`.
- **Empty `[tools]` section (no runtimes, no cli):** No tool entries emitted. Not an issue.
- **Empty `[secrets.required]` array:** No secret entries. `has_issues` remains false.
- **Empty `[agents]` map:** Agents section skipped in human-readable; empty array in JSON.
- **Empty `[mcp]` map:** MCP section skipped in human-readable; empty array in JSON.

### 7.2. Platform differences
- **Non-UTF-8 config path (Linux/WSL):** Now returns `anyhow::Error` with descriptive message including the lossy-displayed path. Cannot occur on macOS (HFS+ enforces UTF-8) or Windows (UTF-16 round-trips to UTF-8).
- **Tool version detection on different platforms:** `get_tool_version()` runs `--version` which works identically across macOS, Linux, and WSL2. Windows is not a supported target for the CLI.

### 7.3. Network failures
- Not applicable. `great status` makes no network calls. All checks are local (PATH lookup, env var check, file read).

### 7.4. Concurrent access
- `great status` is read-only. It never modifies `great.toml` or any state. Safe to run concurrently with `great apply` or any other command. The only theoretical race is if `great.toml` is being written while `status` reads it -- this is a standard TOCTOU race that cannot be avoided without file locking, and the worst case is a parse error (handled gracefully).

### 7.5. Config parse error
- When `config::load()` fails (invalid TOML, validation error), the human-readable path prints the error and continues with platform-only info. Exit code is 0 (parse errors are not "critical issues" in the exit code contract -- the user is already told about the error). JSON path omits config-dependent sections.

### 7.6. Tool that exists but `--version` fails
- `get_tool_version()` returns `None`. Tool is still shown as "installed" in both modes. JSON: `installed: true, actual_version: null`. Human-readable: shows "installed" instead of version string.

### 7.7. Very large config (hundreds of tools/agents)
- No pagination or truncation needed. Status output is a diagnostic dump. Performance is bounded by the number of `--version` subprocess spawns; each takes <100ms. 100 tools would take ~10 seconds, which is acceptable for a diagnostic command.

---

## 8. Error Handling

| Error condition | Handler | User message |
|----------------|---------|-------------|
| Non-UTF-8 config path | `?` propagation to `main` | "config path contains non-UTF-8 characters: {lossy_path}" |
| Config file read error | `config::load` returns Err | "Failed to parse config: {error}" (stderr) |
| Config TOML parse error | `config::load` returns Err | "Failed to parse config: {error}" (stderr) |
| `serde_json` serialization failure | `?` propagation | "Error: {serde_json error}" (anyhow in main) |
| Tool `--version` spawn failure | Returns `None` silently | Tool shown as "installed" without version |
| `std::env::current_dir()` failure | Propagated from `discover_config` | "Error: {io error}" |

All error messages include enough context for the user to take corrective action. No bare "something went wrong" messages.

---

## 9. Security Considerations

- **Secret values are never printed.** The secrets section only reports whether a secret `is_set` (boolean). The actual value of `std::env::var(key)` is checked with `.is_ok()` and discarded. The JSON output contains only the key name and `is_set` flag.
- **No privilege escalation.** The command runs entirely as the current user. No `sudo`, no file writes, no network calls.
- **Tool version probing.** `get_tool_version()` executes arbitrary binaries found on `$PATH`. This is the same trust model as running those tools directly. The command name comes from the user's own `great.toml`, not from external input.
- **JSON output to stdout.** Only JSON goes to stdout; all human-readable output goes to stderr. This prevents accidentally piping diagnostic messages into downstream tools.

---

## 10. Testing Strategy

### 10.1. Unit tests (not added in this task)

The serialization structs are private to `status.rs` and tested indirectly through integration tests. Adding `#[cfg(test)]` unit tests for struct serialization would duplicate what `serde_json`'s own tests already verify.

### 10.2. Integration tests (11 new tests in `tests/cli_smoke.rs`)

| Test name | Asserts |
|-----------|---------|
| `status_with_valid_config_exits_ok` | Exit 0, stderr contains "Config:" |
| `status_verbose_accepted` | Exit 0, `--verbose` flag accepted |
| `status_verbose_short_flag_accepted` | Exit 0, `-v` alias works |
| `status_json_valid_json` | Exit 0, stdout parses as JSON, contains `platform`, `arch`, `shell`, `has_issues`, `issues` |
| `status_json_no_config_still_valid` | Exit 0, `config_path` is null, `tools` absent |
| `status_json_with_config_includes_tools` | Exit 0, `tools` array present, contains expected tool entry |
| `status_json_with_secrets` | Exit 0, `secrets` array with correct `is_set` values |
| `status_exit_code_nonzero_missing_tools` | Exit 1, stderr contains "not installed" |
| `status_exit_code_nonzero_missing_secrets` | Exit 1, stderr contains "missing" |
| `status_json_always_exits_zero_even_with_issues` | Exit 0 despite missing tools and secrets |
| `status_no_config_exits_zero` | Exit 0 (no config is not a failure) |
| `status_verbose_with_config_shows_capabilities` | Exit 0, verbose shows "Shell:" line |

### 10.3. Existing tests (3, already passing)

The three existing status tests (`status_shows_platform`, `status_warns_no_config`, `status_json_outputs_json`) continue to pass without modification. The `status_json_outputs_json` test checks for "platform" in stdout, which the new JSON output still contains.

### 10.4. Running tests

```bash
cargo test --test cli_smoke status
```

This runs all tests in `cli_smoke.rs` whose name contains "status".

---

## 11. Acceptance Criteria

- [ ] `cargo build` succeeds without warnings
- [ ] `cargo clippy` produces zero warnings for `src/cli/status.rs`
- [ ] No `.unwrap()` calls remain in production paths of `src/cli/status.rs` (the one in `get_tool_version` at `unwrap_or("")` is on an `Option` from `.lines().next()` which cannot panic on a non-empty string -- this is acceptable)
- [ ] `great status --json` emits valid JSON containing: `platform`, `arch`, `shell`, `is_root`, `config_path`, `has_issues`, `issues`, and (when config present) `tools`, `mcp`, `agents`, `secrets`
- [ ] `great status --json` always exits 0, even when issues exist
- [ ] `great status` exits 1 when declared tools are missing
- [ ] `great status` exits 1 when required secrets are missing
- [ ] `great status` exits 0 when no `great.toml` is found
- [ ] `great status --verbose` shows full version strings for tools and full command/args/transport for MCP servers
- [ ] All 3 existing status integration tests continue to pass
- [ ] All 11 new integration tests pass
- [ ] `great status` human-readable output goes to stderr; only `--json` output goes to stdout
