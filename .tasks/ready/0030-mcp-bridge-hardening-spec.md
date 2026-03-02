# 0030: MCP Bridge Hardening -- Specification

**Author:** Ada Lovelace (Spec Writer)
**Task:** `.tasks/backlog/0030-mcp-bridge-hardening.md`
**Date:** 2026-02-28
**Complexity:** M (five focused, independent changes; no new crate dependencies)

---

## Summary

Five follow-up hardening items deferred from the 0029 MCP Bridge implementation.
Each is self-contained and can be built and tested independently. No new crate
dependencies are required -- all changes use `std` library facilities and existing
project infrastructure.

---

## Item A -- Path Traversal Prevention in `research` and `analyze_code`

### Current Code

**`src/mcp/bridge/server.rs` lines 27-33** -- `GreatBridge` struct:

```rust
#[derive(Clone)]
pub struct GreatBridge {
    backends: Arc<Vec<BackendConfig>>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    tool_router: ToolRouter<Self>,
}
```

**`src/mcp/bridge/server.rs` lines 37-50** -- `GreatBridge::new()`:

```rust
pub fn new(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
) -> Self {
    Self {
        backends: Arc::new(backends),
        default_backend,
        registry,
        preset,
        tool_router: Self::tool_router(),
    }
}
```

**`src/mcp/bridge/server.rs` lines 167-188** -- `research` file read (no validation):

```rust
if let Some(files) = &params.0.files {
    for path in files {
        match std::fs::read(path) {
            Ok(bytes) => {
                // ... reads file unconditionally
```

**`src/mcp/bridge/server.rs` lines 220-232** -- `analyze_code` file read (no validation):

```rust
let code = if std::path::Path::new(&params.0.code_or_path).exists() {
    match std::fs::read_to_string(&params.0.code_or_path) {
        Ok(content) => content,
        // ...
```

**`src/mcp/bridge/server.rs` lines 441-448** -- `start_bridge()` signature:

```rust
pub async fn start_bridge(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
) -> anyhow::Result<()> {
```

### Changes

#### A1. Add `allowed_dirs` field to `GreatBridge`

```rust
use std::path::PathBuf;

#[derive(Clone)]
pub struct GreatBridge {
    backends: Arc<Vec<BackendConfig>>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    allowed_dirs: Option<Vec<PathBuf>>,  // NEW
    tool_router: ToolRouter<Self>,
}
```

Update `GreatBridge::new()` to accept and store the new field:

```rust
pub fn new(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    allowed_dirs: Option<Vec<PathBuf>>,  // NEW
) -> Self {
    Self {
        backends: Arc::new(backends),
        default_backend,
        registry,
        preset,
        allowed_dirs,  // NEW
        tool_router: Self::tool_router(),
    }
}
```

#### A2. Add a private path validation helper

Add this method to the `impl GreatBridge` block (private helpers section, after
`run_sync`):

```rust
/// Validate that a file path is allowed by the configured allowed_dirs.
///
/// When `allowed_dirs` is `None`, all paths are allowed (single-user
/// threat model). When `Some`, each requested path is canonicalized
/// and checked against the allowed directory prefixes.
fn validate_path(&self, raw_path: &str) -> Result<(), String> {
    let allowed = match &self.allowed_dirs {
        Some(dirs) => dirs,
        None => return Ok(()),
    };

    let canonical = std::fs::canonicalize(raw_path)
        .map_err(|e| format!("cannot resolve path '{}': {}", raw_path, e))?;

    for dir in allowed {
        if canonical.starts_with(dir) {
            return Ok(());
        }
    }

    Err(format!(
        "path not in allowed directories: '{}' (canonical: {}). \
         Allowed: {}",
        raw_path,
        canonical.display(),
        allowed
            .iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
    ))
}
```

Note on `canonicalize`: this resolves symlinks and relative paths to absolute
paths. A symlink pointing outside the allowed directories will be correctly
rejected because `canonicalize` follows the link before checking the prefix.
This is the desired behavior -- it prevents symlink-based traversal.

#### A3. Guard file reads in `research`

Replace the file-reading loop body in the `research` method (lines 168-188).
Insert a validation check before the `std::fs::read(path)` call:

```rust
if let Some(files) = &params.0.files {
    for path in files {
        if let Err(e) = self.validate_path(path) {
            return Ok(CallToolResult::error(vec![Content::text(e)]));
        }
        match std::fs::read(path) {
            // ... unchanged
```

#### A4. Guard file reads in `analyze_code`

Insert a validation check before the `std::fs::read_to_string` call in the
`analyze_code` method (lines 220-232):

```rust
let code = if std::path::Path::new(&params.0.code_or_path).exists() {
    if let Err(e) = self.validate_path(&params.0.code_or_path) {
        return Ok(CallToolResult::error(vec![Content::text(e)]));
    }
    match std::fs::read_to_string(&params.0.code_or_path) {
        // ... unchanged
```

#### A5. Update `start_bridge()` to accept `allowed_dirs`

```rust
pub async fn start_bridge(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    allowed_dirs: Option<Vec<PathBuf>>,  // NEW
) -> anyhow::Result<()> {
    let bridge = GreatBridge::new(
        backends, default_backend, registry.clone(), preset, allowed_dirs,
    );
    // ... rest unchanged
```

#### A6. Canonicalize allowed_dirs at startup

In `start_bridge()`, canonicalize the allowed dirs immediately so that
relative paths passed via CLI are resolved once:

```rust
// Canonicalize allowed_dirs at startup so relative paths work
let allowed_dirs = allowed_dirs.map(|dirs| {
    dirs.into_iter()
        .filter_map(|d| {
            std::fs::canonicalize(&d)
                .map_err(|e| {
                    tracing::warn!(
                        "allowed_dirs: cannot resolve '{}': {} (skipping)",
                        d.display(),
                        e
                    );
                    e
                })
                .ok()
        })
        .collect::<Vec<_>>()
});
```

### Config and CLI Changes for Item A

**`src/config/schema.rs`** -- Add field to `McpBridgeConfig` (after line 163):

```rust
/// Optional directory allowlist for file-reading tools (research, analyze_code).
/// When set, only files under these directories can be read.
/// Paths are canonicalized at startup; relative paths are resolved from cwd.
#[serde(skip_serializing_if = "Option::is_none")]
pub allowed_dirs: Option<Vec<String>>,
```

**`src/cli/mcp_bridge.rs`** -- Add `--allowed-dirs` arg to `Args` struct:

```rust
/// Restrict file-reading tools (research, analyze_code) to paths under
/// these directories. Comma-separated. Omit to allow all paths.
#[arg(long, value_delimiter = ',')]
pub allowed_dirs: Option<Vec<String>>,
```

In the `run()` function, merge CLI arg with config (CLI wins), then convert
to `Vec<PathBuf>`:

```rust
let allowed_dirs_raw: Option<Vec<String>> = args
    .allowed_dirs
    .or_else(|| bridge_config.as_ref().and_then(|c| c.allowed_dirs.clone()));

let allowed_dirs = allowed_dirs_raw.map(|dirs| {
    dirs.into_iter()
        .map(|s| std::path::PathBuf::from(s))
        .collect::<Vec<_>>()
});
```

Pass `allowed_dirs` to `start_bridge()`:

```rust
rt.block_on(start_bridge(
    backends, default_backend, registry, preset, allowed_dirs,
))
```

### Acceptance Criteria for Item A

- `great mcp-bridge --allowed-dirs /home/user/projects` rejects a `research`
  call targeting `/etc/shadow` with error text containing "path not in allowed
  directories".
- A path inside `/home/user/projects` succeeds normally.
- Symlinks that resolve outside allowed dirs are rejected.
- When `--allowed-dirs` is omitted and `allowed_dirs` is absent from config,
  all paths are allowed (backward-compatible).
- Non-existent allowed dirs are skipped with a tracing warning at startup.

---

## Item B -- `auto_approve` Config Opt-out and Doctor Warning

### Current Code

**`src/mcp/bridge/backends.rs` lines 110-158** -- `build_command_args()`:

```rust
pub fn build_command_args(
    backend: &BackendConfig,
    prompt: &str,
    model_override: Option<&str>,
    system_prompt: Option<&str>,
) -> (String, Vec<String>) {
    // ...
    } else {
        // Standard pattern: [binary] [auto_approve_flag] [-p prompt]
        if let Some(flag) = backend.auto_approve_flag {
            args.push(flag.to_string());       // <-- always pushed
        }
```

The auto-approve flag is always included when present on the `BackendConfig`.
There is no way to suppress it.

**`src/cli/doctor.rs` lines 614-670** -- `check_mcp_bridge()`:

No mention of auto-approve flags or security warnings.

### Changes

#### B1. Add `auto_approve` boolean parameter to `build_command_args`

Change the signature:

```rust
pub fn build_command_args(
    backend: &BackendConfig,
    prompt: &str,
    model_override: Option<&str>,
    system_prompt: Option<&str>,
    auto_approve: bool,               // NEW
) -> (String, Vec<String>) {
```

Change the auto-approve insertion (line 129):

```rust
    } else {
        // Standard pattern: [binary] [auto_approve_flag] [-p prompt]
        if auto_approve {
            if let Some(flag) = backend.auto_approve_flag {
                args.push(flag.to_string());
            }
        }
```

#### B2. Update all call sites of `build_command_args`

There are 4 call sites, all in `src/mcp/bridge/server.rs`:

- Line 60-65 (`prompt` tool handler)
- Line 192-197 (`research` tool handler)
- Line 236-240 (`analyze_code` tool handler)

And 1 in `src/mcp/bridge/registry.rs`:

- Line 94 (`spawn_task`)

Each call site must pass the `auto_approve` boolean. This means `GreatBridge`
and `TaskRegistry` need access to the auto_approve setting.

**Add `auto_approve` field to `GreatBridge`:**

```rust
pub struct GreatBridge {
    backends: Arc<Vec<BackendConfig>>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    allowed_dirs: Option<Vec<PathBuf>>,
    auto_approve: bool,  // NEW
    tool_router: ToolRouter<Self>,
}
```

Update `new()` accordingly (add `auto_approve: bool` param, store it).

In each server.rs call to `build_command_args`, pass `self.auto_approve`:

```rust
let (binary, args) = super::backends::build_command_args(
    backend,
    &params.0.prompt,
    params.0.model.as_deref(),
    None,
    self.auto_approve,  // NEW
);
```

**For `registry.rs`:** `spawn_task` calls `build_command_args` at line 94. Add
an `auto_approve: bool` field to `TaskRegistry` and pass it through:

```rust
pub struct TaskRegistry {
    tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
    pub default_timeout: Duration,
    pub auto_approve: bool,  // NEW
}

impl TaskRegistry {
    pub fn new(timeout_secs: u64, auto_approve: bool) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            default_timeout: Duration::from_secs(timeout_secs),
            auto_approve,
        }
    }
```

In `spawn_task` (line 94):

```rust
let (binary, args) = build_command_args(
    backend, prompt, model_override, system_prompt, self.auto_approve,
);
```

#### B3. Add `auto_approve` to `McpBridgeConfig`

**`src/config/schema.rs`** -- Add to `McpBridgeConfig`:

```rust
/// Whether to pass auto-approval flags (e.g., --dangerously-skip-permissions)
/// to backends. Default: true. Set to false to require interactive approval.
#[serde(skip_serializing_if = "Option::is_none")]
pub auto_approve: Option<bool>,
```

#### B4. Wire through `mcp_bridge.rs`

In `run()`, resolve the auto_approve setting (default true):

```rust
let auto_approve = bridge_config
    .as_ref()
    .and_then(|c| c.auto_approve)
    .unwrap_or(true);
```

Pass to `TaskRegistry::new()`:

```rust
let registry = TaskRegistry::new(timeout_secs, auto_approve);
```

Pass to `start_bridge()` (add parameter):

```rust
pub async fn start_bridge(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    allowed_dirs: Option<Vec<PathBuf>>,
    auto_approve: bool,  // NEW
) -> anyhow::Result<()> {
    let bridge = GreatBridge::new(
        backends, default_backend, registry.clone(), preset,
        allowed_dirs, auto_approve,
    );
```

#### B5. Add auto-approve warning to `check_mcp_bridge()` in doctor.rs

After the existing backend availability loop and before the `.mcp.json` check
(before line 657 in doctor.rs), add:

```rust
// Warn about auto-approve flags
let auto_approve_enabled = loaded_config
    .as_ref()
    .and_then(|c| c.mcp_bridge.as_ref())
    .and_then(|b| b.auto_approve)
    .unwrap_or(true);

if command_exists("claude") {
    if auto_approve_enabled {
        warn(
            result,
            "Claude backend uses --dangerously-skip-permissions (auto-approve enabled). \
             Set auto-approve = false in [mcp-bridge] to disable.",
        );
    } else {
        pass(
            result,
            "Claude backend: auto-approve disabled (--dangerously-skip-permissions suppressed)",
        );
    }
}
```

This requires `check_mcp_bridge` to have access to the config. Change
its signature to:

```rust
fn check_mcp_bridge(
    result: &mut DiagnosticResult,
    bridge_config: Option<&McpBridgeConfig>,
)
```

**Important:** The backlog acceptance criteria require the auto-approve warning
to appear even when no `[mcp-bridge]` section exists in `great.toml`. The
current code (lines 92-99) gates `check_mcp_bridge` behind
`mcp_bridge.is_some()`. Change the call site in `run()` to run unconditionally
when any backend is on PATH, passing the bridge config as `Option`:

```rust
// 7b. MCP Bridge backend checks (always check -- auto-approve warning
// should appear even without [mcp-bridge] config when Claude is on PATH)
let bridge_cfg = loaded_config
    .as_ref()
    .and_then(|c| c.mcp_bridge.as_ref());
check_mcp_bridge(&mut result, bridge_cfg);
```

This means the MCP Bridge section appears in doctor output whenever any backend
CLI is found on PATH, regardless of whether `[mcp-bridge]` is configured. The
`.mcp.json` registration check at the end of `check_mcp_bridge` should be
skipped when `bridge_cfg` is `None` (no config means the user hasn't opted in
to the bridge yet):

```rust
// Check .mcp.json registration -- only if bridge is configured
if bridge_config.is_some() {
    let mcp_path = crate::mcp::project_mcp_path();
    let mcp_json = crate::mcp::McpJsonConfig::load(&mcp_path).unwrap_or_default();
    if mcp_json.has_server("great-bridge") {
        pass(result, "great-bridge: registered in .mcp.json");
    } else {
        warn(
            result,
            "great-bridge: not in .mcp.json -- run `great apply` to register",
        );
    }
}
```

#### B6. Update existing tests in `backends.rs`

All 5 existing tests in `backends.rs` that call `build_command_args` must add
the `auto_approve` parameter. Pass `true` to preserve current behavior:

```rust
// e.g., test_build_command_args_claude:
let (bin, args) = build_command_args(&backend, "hello", None, None, true);
```

Add one new test for `auto_approve = false`:

```rust
#[test]
fn test_build_command_args_claude_no_auto_approve() {
    let backend = BackendConfig {
        name: "claude",
        binary: "/usr/bin/claude".to_string(),
        model: None,
        auto_approve_flag: Some("--dangerously-skip-permissions"),
        api_key_env: None,
    };
    let (_, args) = build_command_args(&backend, "hello", None, None, false);
    assert!(!args.contains(&"--dangerously-skip-permissions".to_string()));
    assert!(args.contains(&"-p".to_string()));
    assert!(args.contains(&"hello".to_string()));
}
```

### Acceptance Criteria for Item B

- Default behavior unchanged: `auto_approve` defaults to `true`, all backends
  include their auto-approve flags as before.
- `auto_approve = false` in `[mcp-bridge]` suppresses `--dangerously-skip-permissions`
  from Claude, `-y` from Gemini/Grok, and `--full-auto` from Codex.
- `great doctor` with Claude CLI on PATH and NO `[mcp-bridge]` config shows a
  warning line containing `--dangerously-skip-permissions` (auto-approve defaults
  to true when config is absent).
- Adding `auto_approve = false` to `[mcp-bridge]` in `great.toml` changes that
  doctor line from `warn` to `pass`.

---

## Item C -- Refactor `check_mcp_bridge()` to Use `discover_backends()`

### Current Code

**`src/cli/doctor.rs` lines 619-625** -- hardcoded backend list:

```rust
let backends = [
    ("gemini", "Gemini CLI", "GEMINI_API_KEY"),
    ("codex", "Codex CLI", "OPENAI_API_KEY"),
    ("claude", "Claude CLI", ""),
    ("grok", "Grok CLI", "XAI_API_KEY"),
    ("ollama", "Ollama", ""),
];
```

**`src/mcp/bridge/backends.rs` lines 27-68** -- `BACKEND_SPECS` (the source
of truth with `name`, `api_key_env`, `default_binary`).

**`src/mcp/bridge/backends.rs` line 13** -- dead_code annotation:

```rust
#[allow(dead_code)] // Planned for doctor integration (refactor check_mcp_bridge to use BackendConfig).
pub api_key_env: Option<&'static str>,
```

### Changes

#### C1. Add `display_name` field to `BackendSpec` and `BackendConfig`

The doctor currently uses display names like "Gemini CLI", "Claude CLI". These
are not in `BackendSpec` or `BackendConfig`. Add them.

**`src/mcp/bridge/backends.rs`** -- Add to `BackendSpec`:

```rust
struct BackendSpec {
    name: &'static str,
    display_name: &'static str,  // NEW
    default_binary: &'static str,
    env_override: &'static str,
    auto_approve_flag: Option<&'static str>,
    api_key_env: Option<&'static str>,
    default_model: Option<&'static str>,
}
```

Add to `BackendConfig`:

```rust
pub struct BackendConfig {
    pub name: &'static str,
    pub display_name: &'static str,  // NEW
    pub binary: String,
    pub model: Option<String>,
    pub auto_approve_flag: Option<&'static str>,
    pub api_key_env: Option<&'static str>,
}
```

Update the `BACKEND_SPECS` entries with display names:

```rust
const BACKEND_SPECS: &[BackendSpec] = &[
    BackendSpec {
        name: "gemini",
        display_name: "Gemini CLI",
        // ...
    },
    BackendSpec {
        name: "codex",
        display_name: "Codex CLI",
        // ...
    },
    BackendSpec {
        name: "claude",
        display_name: "Claude CLI",
        // ...
    },
    BackendSpec {
        name: "grok",
        display_name: "Grok CLI",
        // ...
    },
    BackendSpec {
        name: "ollama",
        display_name: "Ollama",
        // ...
    },
];
```

Update `discover_backends()` to propagate the field:

```rust
Some(BackendConfig {
    name: spec.name,
    display_name: spec.display_name,  // NEW
    binary,
    model,
    auto_approve_flag: spec.auto_approve_flag,
    api_key_env: spec.api_key_env,
})
```

#### C2. Add `all_backend_specs()` public function

The doctor needs to iterate ALL backends (not just discovered ones on PATH) to
report which are found vs. missing. `discover_backends` filters out missing ones.
Add a new function that returns the static spec info for all backends:

```rust
/// Return static metadata for all known backends, regardless of whether
/// they are installed. Used by `great doctor` to report availability.
pub fn all_backend_specs() -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    BACKEND_SPECS
        .iter()
        .map(|s| (s.name, s.display_name, s.api_key_env))
        .collect()
}
```

#### C3. Refactor `check_mcp_bridge()` in doctor.rs

Replace the hardcoded `backends` slice with a call to `all_backend_specs()`.
Add the import at the top of doctor.rs:

```rust
use crate::mcp::bridge::backends::all_backend_specs;
```

Replace lines 619-648 with:

```rust
let mut any_found = false;
for (binary, display_name, api_key_env) in all_backend_specs() {
    if command_exists(binary) {
        any_found = true;
        match api_key_env {
            None => {
                pass(result, &format!("{}: installed", display_name));
            }
            Some(env_var) => {
                if std::env::var(env_var).is_ok() {
                    pass(
                        result,
                        &format!("{}: installed, {} set", display_name, env_var),
                    );
                } else {
                    warn(
                        result,
                        &format!("{}: installed, {} not set", display_name, env_var),
                    );
                }
            }
        }
    } else {
        warn(result, &format!("{}: not found (optional)", display_name));
    }
}
```

#### C4. Remove `#[allow(dead_code)]` from `api_key_env`

**`src/mcp/bridge/backends.rs` line 13** -- Remove the annotation:

```rust
// Before:
#[allow(dead_code)] // Planned for doctor integration (refactor check_mcp_bridge to use BackendConfig).
pub api_key_env: Option<&'static str>,

// After:
pub api_key_env: Option<&'static str>,
```

The field is now used by `all_backend_specs()`, so the dead_code annotation is
no longer needed.

#### C5. Update test assertions in `backends.rs`

The `BackendConfig` structs in test code must include the new `display_name` field:

```rust
// e.g., test_build_command_args_ollama:
let backend = BackendConfig {
    name: "ollama",
    display_name: "Ollama",  // NEW
    binary: "/usr/bin/ollama".to_string(),
    model: Some("llama3.2".to_string()),
    auto_approve_flag: None,
    api_key_env: None,
};
```

### Acceptance Criteria for Item C

- `check_mcp_bridge()` contains zero hardcoded backend name strings.
- Adding a sixth backend to `BACKEND_SPECS` automatically appears in
  `great doctor` output without any change to `doctor.rs`.
- The `#[allow(dead_code)]` annotation on `api_key_env` is removed.
- Existing doctor output format is unchanged (same display names, same
  pass/warn/fail logic).

---

## Item D -- Wire Global `--verbose` / `--quiet` Flags into `mcp-bridge`

### Current Code

**`src/cli/mod.rs` lines 25-36** -- `Cli` struct has `verbose` and `quiet` globals.

**`src/main.rs` lines 39** -- `McpBridge(args)` call does not forward globals:

```rust
Command::McpBridge(args) => cli::mcp_bridge::run(args),
```

**`src/cli/mcp_bridge.rs` lines 11-30** -- `Args` struct and log level resolution:

```rust
pub struct Args {
    #[arg(long)]
    pub preset: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub backends: Option<Vec<String>>,
    #[arg(long)]
    pub timeout: Option<u64>,
    #[arg(long)]
    pub log_level: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    let log_level = args.log_level.unwrap_or_else(|| "warn".to_string());
    // ... uses log_level directly
```

### Changes

#### D1. Add `verbose` and `quiet` skip fields to `Args`

```rust
pub struct Args {
    #[arg(long)]
    pub preset: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub backends: Option<Vec<String>>,
    #[arg(long)]
    pub timeout: Option<u64>,
    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long, value_delimiter = ',')]
    pub allowed_dirs: Option<Vec<String>>,  // From Item A

    /// Set by main.rs from the global --verbose flag.
    #[arg(skip)]
    pub verbose: bool,
    /// Set by main.rs from the global --quiet flag.
    #[arg(skip)]
    pub quiet: bool,
}
```

#### D2. Forward globals in `main.rs`

Change the `McpBridge` match arm (line 39):

```rust
Command::McpBridge(mut args) => {
    args.verbose = cli.verbose;
    args.quiet = cli.quiet;
    cli::mcp_bridge::run(args)
}
```

#### D3. Resolve log level with precedence: `--log-level` > `--verbose`/`--quiet` > default

Replace the log level resolution in `run()` (lines 34-48):

```rust
// Resolve log level: explicit --log-level wins over global flags.
// Global flags: --verbose -> debug, --quiet -> error, default -> warn.
let log_level = if let Some(explicit) = args.log_level {
    explicit
} else if args.verbose {
    "debug".to_string()
} else if args.quiet {
    "error".to_string()
} else {
    "warn".to_string()
};

let filter = match log_level.as_str() {
    "off" => "off",
    "error" => "error",
    "warn" => "warn",
    "info" => "info",
    "debug" => "debug",
    "trace" => "trace",
    other => {
        eprintln!(
            "warning: unknown log level '{}', defaulting to 'warn'",
            other
        );
        "warn"
    }
};
```

### Acceptance Criteria for Item D

- `great --verbose mcp-bridge` starts with tracing at `debug` level.
- `great --quiet mcp-bridge` starts with tracing at `error` level.
- `great --verbose mcp-bridge --log-level info` starts at `info` (explicit wins).
- `great mcp-bridge` without flags starts at `warn` (default, unchanged).

---

## Item E -- Measure and Mitigate Binary Size Growth

### Current Measurement

The release binary at `target/release/great` is **14,269,080 bytes (13.6 MiB)**
as measured on the current main branch (post-0029 merge, commit `74939c2`).

The baseline before 0029 was **10,871,632 bytes (10.4 MiB)** per the Wirth
performance report.

**Actual increase: 3,397,448 bytes (+31.3%).**

This exceeds the 12.5 MB threshold defined in the task.

### Investigation

The builder must investigate mitigations. Prioritized options:

1. **Enable LTO in release profile.** Add to `Cargo.toml`:

   ```toml
   [profile.release]
   lto = true
   ```

   Expected savings: 10-30% of binary size. LTO enables cross-crate dead code
   elimination and monomorphization deduplication. Cost: slower release builds
   (acceptable for CI only).

2. **Strip debug symbols.** Add to `Cargo.toml`:

   ```toml
   [profile.release]
   strip = true
   ```

   Expected savings: varies; Rust release builds may retain some debug info.
   Combined with LTO this is often significant.

3. **Set `codegen-units = 1` in release profile** for better optimization:

   ```toml
   [profile.release]
   codegen-units = 1
   ```

4. If the above mitigations are insufficient to bring the binary below 12.5 MB,
   consider replacing `tracing-subscriber` with direct `eprintln!` macros
   (saves ~0.9-1.2 MB per Wirth's estimate). This is a larger change and should
   only be pursued if LTO + strip are insufficient.

### Procedure

1. Record the current release binary size (already done: 14,269,080 bytes).
2. Apply LTO + strip + codegen-units=1 to `[profile.release]`.
3. Build release binary: `cargo build --release`.
4. Record post-mitigation size.
5. If still above 12.5 MB, apply option 4 (tracing-subscriber removal).
6. Record final size in the iteration observer report.

### Acceptance Criteria for Item E

- A release binary size measurement is recorded in the observer report.
- At least one mitigation is applied (LTO + strip at minimum).
- Post-mitigation binary size is recorded.
- If size still exceeds 12.5 MB after all reasonable mitigations, this is
  documented with justification (the bridge replaces an entire Node.js runtime).

---

## Implementation Order

Build and test in this order:

1. **Item C** (doctor refactor) -- Lowest risk, pure refactoring. Add
   `display_name` to `BackendSpec`/`BackendConfig`, add `all_backend_specs()`,
   refactor `check_mcp_bridge()`, remove dead_code annotation. Test by running
   `great doctor` and inspecting output.

2. **Item D** (global flags) -- Small, isolated change. Add skip fields,
   forward in main.rs, update log-level resolution. Test by running
   `great --verbose mcp-bridge` and checking stderr output level.

3. **Item B** (auto_approve) -- Requires changes to `build_command_args`
   signature (ripples through server.rs, registry.rs, tests). Add config field,
   update doctor. Builds on Item C's doctor refactor.

4. **Item A** (path traversal) -- Requires adding field to GreatBridge,
   new helper function, guards in two tool handlers, CLI arg, config field.
   Builds on Item B's start_bridge signature changes.

5. **Item E** (binary size) -- Independent measurement + Cargo.toml profile
   changes. Do last because it requires a full release build. LTO changes
   slow the build, so apply after all code changes are complete.

---

## Files to Modify

| File | Items | Nature of Change |
|------|-------|-----------------|
| `src/mcp/bridge/backends.rs` | B, C | Add `display_name` field, `all_backend_specs()` fn, `auto_approve` param to `build_command_args`, remove dead_code, update tests |
| `src/mcp/bridge/server.rs` | A, B | Add `allowed_dirs` + `auto_approve` fields to `GreatBridge`, `validate_path()` helper, guard file reads, update `start_bridge()` signature |
| `src/mcp/bridge/registry.rs` | B | Add `auto_approve` field to `TaskRegistry`, pass through `spawn_task` |
| `src/cli/mcp_bridge.rs` | A, B, D | Add `--allowed-dirs` arg, `verbose`/`quiet` skip fields, resolve `auto_approve` and `allowed_dirs` from config, update log-level resolution |
| `src/cli/doctor.rs` | B, C | Refactor `check_mcp_bridge()` to use `all_backend_specs()`, add auto-approve warning, change function signature |
| `src/config/schema.rs` | A, B | Add `auto_approve` and `allowed_dirs` to `McpBridgeConfig` |
| `src/main.rs` | D | Forward `verbose`/`quiet` to `McpBridge` args |
| `Cargo.toml` | E | Add `[profile.release]` with LTO + strip + codegen-units |

No new files are created.

---

## Edge Cases

### Path Traversal (Item A)

- **Symlinks:** `canonicalize` resolves symlinks. A symlink inside allowed_dirs
  pointing to `/etc/shadow` is correctly rejected because the canonical path
  is `/etc/shadow`.
- **Relative paths in files param:** The MCP client may pass `../../etc/shadow`.
  `canonicalize` resolves this to an absolute path before prefix checking.
- **Non-existent paths:** `canonicalize` returns `Err` for non-existent paths.
  The error message from `validate_path` will be "cannot resolve path".
  The subsequent `std::fs::read` would also fail, but the validation error
  is more informative.
- **Empty `allowed_dirs` list:** `--allowed-dirs ""` results in
  `Some(vec![])` -- an empty allowlist that rejects ALL paths. This is
  technically correct (the user requested an empty allowlist) but surprising.
  The builder should add a tracing warning when the resolved allowlist is empty.
- **Windows paths:** The bridge runs on Unix only (the `#[cfg(unix)]` blocks
  in registry.rs). On WSL2, paths are Unix-style. No Windows path handling needed.

### Auto-Approve (Item B)

- **Ollama:** Has `auto_approve_flag: None`, so the `auto_approve` parameter
  has no effect. No flag to suppress.
- **Config absent:** When no `[mcp-bridge]` section exists, `auto_approve`
  defaults to `true` (preserving current behavior).
- **Doctor without config:** When `check_mcp_bridge` receives `None` for
  bridge_config, `auto_approve` defaults to `true` for the warning logic.
  The `.mcp.json` registration check is skipped (user hasn't opted in).

### Doctor Refactor (Item C)

- **New backends added to BACKEND_SPECS:** Automatically appear in doctor
  output. No manual sync needed.
- **Backend removed from BACKEND_SPECS:** Automatically disappears from
  doctor output.
- **Binary on PATH but not in BACKEND_SPECS:** Not possible -- the doctor
  only checks backends known to the bridge.

### Global Flags (Item D)

- **Both `--verbose` and `--quiet` passed:** `--verbose` is checked first,
  so `debug` wins. This matches the typical CLI convention (last explicit
  flag should win, but since both are booleans with no ordering, verbose
  takes precedence as the more informative option).
- **`--verbose` with `--log-level off`:** `--log-level` wins, output is off.
  This is correct -- explicit always beats derived.

### Binary Size (Item E)

- **LTO slows CI builds:** The release profile is only used for `cargo build
  --release`. Development builds (the default) are unaffected. CI already
  caches deps via `actions/cache`, so incremental LTO cost is bounded.
- **Strip removes backtraces:** `strip = true` removes symbols needed for
  panic backtraces. For a CLI tool this is acceptable -- users report errors
  via exit codes and stderr messages, not stack traces.

---

## Error Handling

All error paths produce actionable messages:

| Scenario | Message |
|----------|---------|
| Path outside allowed dirs | `"path not in allowed directories: '/etc/shadow' (canonical: /etc/shadow). Allowed: /home/user/projects"` |
| Path cannot be resolved | `"cannot resolve path '/nonexistent': No such file or directory (os error 2)"` |
| Empty allowed_dirs resolved | tracing::warn at startup: `"allowed_dirs: resolved to empty list; all file reads will be rejected"` |
| Non-existent allowed dir | tracing::warn at startup: `"allowed_dirs: cannot resolve '/bad/path': No such file or directory (skipping)"` |
| Unknown log level | `"warning: unknown log level 'trace2', defaulting to 'warn'"` (printed to stderr) |

---

## Security Considerations

- **Item A** is the primary security change. The `validate_path` function uses
  `std::fs::canonicalize` which resolves ALL symlinks. This prevents symlink-based
  escapes. The check is: canonical path must `starts_with` one of the canonical
  allowed directories. Both sides are canonicalized.
- **Item B** is a security UX improvement. Users are warned about the implications
  of `--dangerously-skip-permissions` and given a config knob to disable it.
- **TOCTOU risk in Item A:** There is a theoretical time-of-check/time-of-use
  race between `validate_path` (which calls `canonicalize`) and the subsequent
  `std::fs::read`. An attacker who can atomically swap a path between check and
  read could bypass the allowlist. This requires local filesystem access with
  the same UID, which is outside the threat model (the bridge is a same-user,
  same-machine tool). Documenting this is sufficient; no mitigation needed.

---

## Testing Strategy

### Unit Tests

**`src/mcp/bridge/backends.rs`:**

- Existing tests updated to include `display_name` field and `auto_approve`
  parameter.
- New test: `test_build_command_args_claude_no_auto_approve` -- verifies
  `--dangerously-skip-permissions` is absent when `auto_approve = false`.
- New test: `test_all_backend_specs_returns_all` -- verifies count matches
  `BACKEND_SPECS.len()`.

**`src/mcp/bridge/server.rs`** (or a new test module):

- `test_validate_path_allowed` -- path inside allowed dir passes.
- `test_validate_path_rejected` -- path outside allowed dir is rejected with
  correct error message.
- `test_validate_path_no_allowlist` -- `allowed_dirs: None` allows any path.
- `test_validate_path_nonexistent` -- non-existent path returns error from
  `canonicalize`.

These tests require `tempfile` (already in dev-deps) to create temporary
directories and files.

### Integration Tests

- `great doctor` with no config but Claude on PATH: verify MCP Bridge section
  appears with the auto-approve warning. The `.mcp.json` registration check
  should NOT appear (no config = not opted in).
- `great --verbose mcp-bridge --help`: verify the help text shows
  `--allowed-dirs` and the help exits cleanly with verbose flag.
- Binary size: `cargo build --release && ls -la target/release/great` --
  record in observer report.

### Manual Verification

- Start bridge with `--allowed-dirs /tmp`, send a `research` tool call
  with a file path outside `/tmp`, verify error response.
- Start bridge with `auto_approve = false` in config, observe that
  backend commands are invoked without `--dangerously-skip-permissions`.
- Run `great --verbose mcp-bridge` and verify `DEBUG` level tracing
  appears on stderr.
