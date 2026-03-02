# 0029: Inbuilt MCP Bridge Server -- Technical Specification

**Task:** 0029-inbuilt-mcp-bridge
**Author:** Lovelace (Spec Writer)
**Date:** 2026-02-27
**Status:** ready
**Estimated Complexity:** XL

---

## Summary

The `great` binary gains a new top-level subcommand `great mcp-bridge` that
starts an MCP-compliant stdio server (JSON-RPC 2.0) bridging multiple AI CLI
backends (gemini, codex, claude, grok, ollama) into a unified tool surface.
This eliminates the Node.js/npm dependency currently required for MCP bridge
servers.

The implementation uses the `rmcp` crate (v0.16, the official Rust MCP SDK)
for protocol handling, `process-wrap` (v9) for process group management, and
exposes tools via rmcp's `#[tool]` / `#[tool_router]` / `#[tool_handler]`
proc macros. The bridge self-registers in `.mcp.json` via `great apply` and
is health-checked by `great doctor`.

---

## Requirements Preserved from Task 0029

This spec preserves all seven requirements (R1--R7) from the backlog task
file at `.tasks/backlog/0029-inbuilt-mcp-bridge.md`. The sections below
add concrete implementation details, exact types, and verified API patterns.

---

## Phase 1: Foundation (No Cross-Dependencies)

### 1.1 New Cargo Dependencies

Add to `Cargo.toml` (currently 25 lines of dependencies, line 11--25):

```toml
# After line 25 (which = "7"):
rmcp = { version = "0.16", features = ["server", "transport-io"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
schemars = "1.0"
```

**Why each dependency:**

- `rmcp` 0.16.0 with `server` + `transport-io` features: provides
  `ServerHandler` trait, `#[tool]` / `#[tool_router]` / `#[tool_handler]`
  proc macros, `model::*` types (`CallToolResult`, `Content`, `ServerInfo`,
  `ServerCapabilities`, `Implementation`), and `transport::stdio()` function.
  The `server` feature pulls in `transport-async-rw` and `schemars`. The
  `transport-io` feature adds `tokio/io-std`.
- `uuid` 1.x with `v4` + `serde`: task ID generation via `Uuid::new_v4()`.
- `tracing` 0.1 + `tracing-subscriber` 0.3: structured async-aware logging to
  stderr. Essential for debugging concurrent subprocess tasks without
  polluting the MCP stdout channel.
- `schemars` 1.0: required by rmcp's `#[tool]` macro for JSON Schema
  generation from tool parameter structs. Must be explicitly declared because
  rmcp reexports it but Cargo requires the consumer to declare it for derive
  macros to resolve.

**NOT adding `process-wrap`:** After analysis, `process-wrap` 9.0 adds
significant API complexity (composable wrappers, `TokioCommandWrap`) for a
feature we can achieve with simpler means: `tokio::process::Command` with
`kill_on_drop(true)` plus manual `libc::killpg` on Unix for process group
cleanup. The builder should evaluate whether `process-wrap` is worth the
dependency; if process tree cleanup proves insufficient with the simple
approach, add it as a follow-up. This keeps the initial PR smaller.

**Alternative if process-wrap IS added:**
```toml
process-wrap = { version = "9.0", features = ["tokio1", "process-group", "kill-on-drop"] }
```

### 1.2 Config Schema: `McpBridgeConfig`

**File:** `src/config/schema.rs`
**Location:** After `PlatformOverride` struct (line 135), before `ConfigMessage` enum (line 138).

```rust
/// Configuration for the `[mcp-bridge]` section of `great.toml`.
///
/// Controls which AI CLI backends the bridge exposes, the default backend
/// for tool calls that omit a backend parameter, and per-task timeout.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct McpBridgeConfig {
    /// Restrict to a subset of backends (default: auto-detect all installed).
    /// Valid values: "gemini", "codex", "claude", "grok", "ollama".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<Vec<String>>,

    /// Backend to use when a tool call omits the `backend` parameter.
    /// Falls back to the first discovered backend if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,

    /// Per-task timeout in seconds (default: 300).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    /// Tool preset: "minimal", "agent", "research", "full" (default: "agent").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}
```

**Add field to `GreatConfig`** (line 23, after `platform`):

```rust
    /// MCP bridge server configuration.
    #[serde(rename = "mcp-bridge", skip_serializing_if = "Option::is_none")]
    pub mcp_bridge: Option<McpBridgeConfig>,
```

**Validation addition** in `GreatConfig::validate()` -- add after the MCP
transport checks (around line 208):

```rust
// Check: mcp-bridge preset must be known
if let Some(bridge) = &self.mcp_bridge {
    if let Some(preset) = &bridge.preset {
        let known_presets = ["minimal", "agent", "research", "full"];
        if !known_presets.contains(&preset.as_str()) {
            messages.push(ConfigMessage::Warning(format!(
                "mcp-bridge: unknown preset '{}' -- known presets: {}",
                preset,
                known_presets.join(", ")
            )));
        }
    }
    if let Some(backends) = &bridge.backends {
        let known_backends = ["gemini", "codex", "claude", "grok", "ollama"];
        for b in backends {
            if !known_backends.contains(&b.as_str()) {
                messages.push(ConfigMessage::Warning(format!(
                    "mcp-bridge: unknown backend '{}' -- known backends: {}",
                    b,
                    known_backends.join(", ")
                )));
            }
        }
    }
}
```

**TOML example:**

```toml
[mcp-bridge]
backends = ["gemini", "claude"]
default-backend = "gemini"
timeout-secs = 300
preset = "agent"
```

Note: `serde(rename_all = "kebab-case")` means the Rust field
`default_backend` serializes as `default-backend` in TOML, matching
great.toml conventions.

### 1.3 Backend Discovery: `src/mcp/bridge/backends.rs`

**New file.** This module has zero dependencies on other bridge modules.

```rust
use anyhow::Result;

/// Static configuration for a supported AI CLI backend.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Backend identifier: "gemini", "codex", "claude", "grok", "ollama".
    pub name: &'static str,
    /// Resolved absolute path to the CLI binary.
    pub binary: String,
    /// Model override (set via config or tool call parameter).
    pub model: Option<String>,
    /// CLI flag to bypass interactive approval prompts.
    pub auto_approve_flag: Option<&'static str>,
    /// Environment variable name for the API key (None = uses login/no key).
    pub api_key_env: Option<&'static str>,
}

/// Per-backend static defaults. The builder must verify each CLI's actual
/// flag conventions at implementation time -- these are best-effort defaults
/// based on current (Feb 2026) CLI versions.
struct BackendSpec {
    name: &'static str,
    default_binary: &'static str,
    env_override: &'static str,
    auto_approve_flag: Option<&'static str>,
    api_key_env: Option<&'static str>,
    default_model: Option<&'static str>,
}

const BACKEND_SPECS: &[BackendSpec] = &[
    BackendSpec {
        name: "gemini",
        default_binary: "gemini",
        env_override: "GREAT_GEMINI_CLI",
        auto_approve_flag: Some("-y"),
        api_key_env: Some("GEMINI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "codex",
        default_binary: "codex",
        env_override: "GREAT_CODEX_CLI",
        auto_approve_flag: Some("--full-auto"),
        api_key_env: Some("OPENAI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "claude",
        default_binary: "claude",
        env_override: "GREAT_CLAUDE_CLI",
        auto_approve_flag: Some("--dangerously-skip-permissions"),
        api_key_env: None, // uses login
        default_model: None,
    },
    BackendSpec {
        name: "grok",
        default_binary: "grok",
        env_override: "GREAT_GROK_CLI",
        auto_approve_flag: Some("-y"),
        api_key_env: Some("XAI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "ollama",
        default_binary: "ollama",
        env_override: "GREAT_OLLAMA_CLI",
        auto_approve_flag: None,
        api_key_env: None,
        default_model: Some("llama3.2"),
    },
];

/// Discover available backends by checking PATH (via `which` crate) and
/// environment variable overrides.
///
/// If `filter` is non-empty, only backends whose name appears in `filter`
/// are considered. Otherwise all backends with a discoverable binary are
/// returned.
pub fn discover_backends(filter: &[String]) -> Vec<BackendConfig> {
    BACKEND_SPECS
        .iter()
        .filter(|spec| {
            filter.is_empty() || filter.iter().any(|f| f == spec.name)
        })
        .filter_map(|spec| {
            // Check env override first, then which
            let binary = std::env::var(spec.env_override)
                .ok()
                .or_else(|| which::which(spec.default_binary).ok().map(|p| p.to_string_lossy().to_string()))?;

            // For ollama, check GREAT_OLLAMA_MODEL env for default model
            let model = if spec.name == "ollama" {
                std::env::var("GREAT_OLLAMA_MODEL")
                    .ok()
                    .or_else(|| spec.default_model.map(|s| s.to_string()))
            } else {
                None
            };

            Some(BackendConfig {
                name: spec.name,
                binary,
                model,
                auto_approve_flag: spec.auto_approve_flag,
                api_key_env: spec.api_key_env,
            })
        })
        .collect()
}

/// Build the command arguments for invoking a backend with a prompt.
///
/// Returns `(binary, args_vec)` ready for `tokio::process::Command`.
pub fn build_command_args(
    backend: &BackendConfig,
    prompt: &str,
    model_override: Option<&str>,
    system_prompt: Option<&str>,
) -> (String, Vec<String>) {
    let mut args = Vec::new();

    if backend.name == "ollama" {
        // ollama run <model> <prompt>
        args.push("run".to_string());
        let model = model_override
            .map(|s| s.to_string())
            .or_else(|| backend.model.clone())
            .unwrap_or_else(|| "llama3.2".to_string());
        args.push(model);
        args.push(prompt.to_string());
    } else {
        // Standard pattern: [binary] [auto_approve_flag] [-p prompt]
        if let Some(flag) = backend.auto_approve_flag {
            args.push(flag.to_string());
        }
        // System prompt injection (claude supports --system-prompt)
        if let Some(sp) = system_prompt {
            if backend.name == "claude" {
                args.push("--system-prompt".to_string());
                args.push(sp.to_string());
            }
            // For other backends, system prompt is prepended to the main prompt
            // by the caller before reaching this function.
        }
        if let Some(m) = model_override.or(backend.model.as_deref()) {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
        args.push("-p".to_string());
        // For backends without native system prompt support, prepend it
        if system_prompt.is_some() && backend.name != "claude" {
            let sp = system_prompt.unwrap();
            args.push(format!("SYSTEM: {}\n\nTASK: {}", sp, prompt));
        } else {
            args.push(prompt.to_string());
        }
    }

    (backend.binary.clone(), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_specs_complete() {
        assert_eq!(BACKEND_SPECS.len(), 5);
        let names: Vec<&str> = BACKEND_SPECS.iter().map(|s| s.name).collect();
        assert!(names.contains(&"gemini"));
        assert!(names.contains(&"codex"));
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"grok"));
        assert!(names.contains(&"ollama"));
    }

    #[test]
    fn test_build_command_args_ollama() {
        let backend = BackendConfig {
            name: "ollama",
            binary: "/usr/bin/ollama".to_string(),
            model: Some("llama3.2".to_string()),
            auto_approve_flag: None,
            api_key_env: None,
        };
        let (bin, args) = build_command_args(&backend, "hello world", None, None);
        assert_eq!(bin, "/usr/bin/ollama");
        assert_eq!(args, vec!["run", "llama3.2", "hello world"]);
    }

    #[test]
    fn test_build_command_args_claude() {
        let backend = BackendConfig {
            name: "claude",
            binary: "/usr/bin/claude".to_string(),
            model: None,
            auto_approve_flag: Some("--dangerously-skip-permissions"),
            api_key_env: None,
        };
        let (bin, args) = build_command_args(&backend, "hello", None, None);
        assert_eq!(bin, "/usr/bin/claude");
        assert_eq!(args, vec!["--dangerously-skip-permissions", "-p", "hello"]);
    }

    #[test]
    fn test_build_command_args_claude_with_system_prompt() {
        let backend = BackendConfig {
            name: "claude",
            binary: "/usr/bin/claude".to_string(),
            model: None,
            auto_approve_flag: Some("--dangerously-skip-permissions"),
            api_key_env: None,
        };
        let (_, args) = build_command_args(&backend, "hello", None, Some("You are helpful"));
        assert!(args.contains(&"--system-prompt".to_string()));
        assert!(args.contains(&"You are helpful".to_string()));
    }

    #[test]
    fn test_discover_backends_empty_filter_returns_all_available() {
        // This test may return 0 backends in CI where no AI CLIs are installed
        let backends = discover_backends(&[]);
        // Just verify it doesn't panic
        for b in &backends {
            assert!(!b.binary.is_empty());
        }
    }
}
```

### 1.4 Task Registry: `src/mcp/bridge/registry.rs`

**New file.** Depends only on `backends.rs` for `BackendConfig` and
`build_command_args`.

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::backends::{BackendConfig, build_command_args};

/// The lifecycle state of a background task.
#[derive(Debug, Clone)]
pub enum TaskState {
    Running {
        pid: u32,
        started_at: Instant,
    },
    Completed {
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration: Duration,
    },
    Failed {
        error: String,
        duration: Duration,
    },
    TimedOut {
        duration: Duration,
    },
    Killed,
}

/// A non-owning snapshot of a task for reporting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub backend: String,
    pub status: String,
    pub prompt_preview: String,
    #[serde(skip)]
    pub started_at: Option<Instant>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Internal task handle stored in the registry.
struct TaskHandle {
    task_id: String,
    backend: String,
    prompt_preview: String,
    state: TaskState,
    /// When the task reached a terminal state (for cleanup timing).
    completed_at: Option<Instant>,
}

/// Thread-safe registry of spawned backend processes.
///
/// All methods take `&self` (shared reference) because internal state is
/// behind `Arc<Mutex<...>>`.
#[derive(Clone)]
pub struct TaskRegistry {
    tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
    default_timeout: Duration,
}

impl TaskRegistry {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            default_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Spawn a backend CLI process asynchronously. Returns the task ID.
    ///
    /// The process is spawned with `kill_on_drop(true)` so that dropping the
    /// child handle kills the process. A background tokio task collects the
    /// output and updates the registry.
    pub async fn spawn_task(
        &self,
        backend: &BackendConfig,
        prompt: &str,
        timeout_override: Option<Duration>,
        model_override: Option<&str>,
        system_prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        let task_id = Uuid::new_v4().to_string();
        let timeout = timeout_override.unwrap_or(self.default_timeout);

        let (binary, args) = build_command_args(backend, prompt, model_override, system_prompt);

        let mut cmd = Command::new(&binary);
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Create new process group on Unix so we can kill the tree
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let child = cmd.spawn().map_err(|e| {
            anyhow::anyhow!("failed to spawn {} ({}): {}", backend.name, binary, e)
        })?;

        let pid = child.id().unwrap_or(0);

        let handle = TaskHandle {
            task_id: task_id.clone(),
            backend: backend.name.to_string(),
            prompt_preview: prompt.chars().take(80).collect(),
            state: TaskState::Running {
                pid,
                started_at: Instant::now(),
            },
            completed_at: None,
        };

        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(task_id.clone(), handle);
        }

        // Background task to collect output
        let tasks_ref = self.tasks.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            let result = tokio::time::timeout(timeout, child.wait_with_output()).await;
            let duration = start.elapsed();

            let new_state = match result {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let exit_code = output.status.code().unwrap_or(-1);
                    if output.status.success() {
                        TaskState::Completed { exit_code, stdout, stderr, duration }
                    } else {
                        TaskState::Completed { exit_code, stdout, stderr, duration }
                    }
                }
                Ok(Err(e)) => TaskState::Failed {
                    error: e.to_string(),
                    duration,
                },
                Err(_) => {
                    // Timeout -- the child is killed on drop
                    TaskState::TimedOut { duration }
                }
            };

            let mut tasks = tasks_ref.lock().await;
            if let Some(handle) = tasks.get_mut(&tid) {
                handle.state = new_state;
                handle.completed_at = Some(Instant::now());
            }
        });

        Ok(task_id)
    }

    /// Get a snapshot of a task's current state.
    pub async fn get_task(&self, task_id: &str) -> Option<TaskSnapshot> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).map(|h| self.snapshot(h))
    }

    /// List all tasks (triggers cleanup of old completed tasks).
    pub async fn list_tasks(&self) -> Vec<TaskSnapshot> {
        let mut tasks = self.tasks.lock().await;

        // Cleanup terminal-state tasks older than 30 minutes
        let cutoff = Instant::now() - Duration::from_secs(30 * 60);
        tasks.retain(|_, h| {
            h.completed_at.map_or(true, |t| t > cutoff)
        });

        tasks.values().map(|h| self.snapshot(h)).collect()
    }

    /// Kill a running task.
    pub async fn kill_task(&self, task_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().await;
        let handle = tasks.get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("task {} not found", task_id))?;

        match &handle.state {
            TaskState::Running { pid, .. } => {
                let pid = *pid;
                // Kill process group on Unix
                #[cfg(unix)]
                {
                    unsafe { libc::killpg(pid as libc::pid_t, libc::SIGTERM); }
                    // Give it 2 seconds, then SIGKILL
                    let pid_copy = pid;
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        unsafe { libc::killpg(pid_copy as libc::pid_t, libc::SIGKILL); }
                    });
                }
                handle.state = TaskState::Killed;
                handle.completed_at = Some(Instant::now());
                Ok(())
            }
            _ => anyhow::bail!("task {} is not running (state: {:?})", task_id, handle.state),
        }
    }

    /// Kill all running tasks. Called on server shutdown.
    pub async fn shutdown_all(&self) {
        let mut tasks = self.tasks.lock().await;
        for handle in tasks.values_mut() {
            if let TaskState::Running { pid, .. } = &handle.state {
                #[cfg(unix)]
                unsafe {
                    libc::killpg(*pid as libc::pid_t, libc::SIGKILL);
                }
                handle.state = TaskState::Killed;
                handle.completed_at = Some(Instant::now());
            }
        }
    }

    /// Wait for specific tasks to reach terminal state.
    pub async fn wait_for_tasks(
        &self,
        task_ids: &[String],
        timeout: Duration,
    ) -> Vec<TaskSnapshot> {
        let deadline = Instant::now() + timeout;
        loop {
            let tasks = self.tasks.lock().await;
            let all_done = task_ids.iter().all(|id| {
                tasks.get(id).map_or(true, |h| !matches!(h.state, TaskState::Running { .. }))
            });
            if all_done || Instant::now() >= deadline {
                return task_ids
                    .iter()
                    .filter_map(|id| tasks.get(id).map(|h| self.snapshot(h)))
                    .collect();
            }
            drop(tasks); // release lock before sleeping
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn snapshot(&self, handle: &TaskHandle) -> TaskSnapshot {
        let (status, exit_code, stdout, stderr, duration_ms) = match &handle.state {
            TaskState::Running { .. } => ("running".to_string(), None, None, None, None),
            TaskState::Completed { exit_code, stdout, stderr, duration, .. } => (
                "completed".to_string(),
                Some(*exit_code),
                Some(stdout.clone()),
                Some(stderr.clone()),
                Some(duration.as_millis() as u64),
            ),
            TaskState::Failed { error, duration } => (
                format!("failed: {}", error),
                None,
                None,
                Some(error.clone()),
                Some(duration.as_millis() as u64),
            ),
            TaskState::TimedOut { duration } => (
                "timed_out".to_string(),
                None,
                None,
                None,
                Some(duration.as_millis() as u64),
            ),
            TaskState::Killed => ("killed".to_string(), None, None, None, None),
        };

        TaskSnapshot {
            task_id: handle.task_id.clone(),
            backend: handle.backend.clone(),
            status,
            prompt_preview: handle.prompt_preview.clone(),
            started_at: match &handle.state {
                TaskState::Running { started_at, .. } => Some(*started_at),
                _ => None,
            },
            exit_code,
            stdout,
            stderr,
            duration_ms,
        }
    }
}
```

**Note on `libc` dependency:** The `libc` crate is a transitive dependency
of `tokio` (on Unix targets) and is always available. No explicit Cargo.toml
entry is needed. Use `#[cfg(unix)]` gates for the `setpgid`/`killpg` calls.
On Windows, `kill_on_drop(true)` is the only cleanup mechanism (Windows Job
Objects via process-wrap would be a future enhancement).

**Note on `unsafe`:** The `pre_exec` closure and `killpg` calls are
inherently unsafe (Unix process management). The builder should wrap these
in well-documented functions and add `// SAFETY:` comments.

---

## Phase 2: rmcp Server and Tool Definitions (Depends on Phase 1)

### 2.1 Tool Parameter Structs: `src/mcp/bridge/tools.rs`

**New file.** Defines the serde/schemars structs for tool parameters and
the preset system.

```rust
use schemars::JsonSchema;
use serde::Deserialize;

// ── Tool parameter structs ──────────────────────────────────────

/// Parameters for the `prompt` tool (synchronous single round-trip).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PromptParams {
    /// The prompt text to send.
    pub prompt: String,
    /// Backend name: gemini, codex, claude, grok, ollama. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Parameters for the `run` tool (async task spawn).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunParams {
    /// Prompt to send asynchronously.
    pub prompt: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Override per-task timeout in seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parameters for the `wait` tool (block until tasks complete).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitParams {
    /// Task IDs to wait for.
    pub task_ids: Vec<String>,
    /// Timeout in seconds for the wait operation.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parameters for the `get_result` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetResultParams {
    /// Task ID to retrieve results for.
    pub task_id: String,
}

/// Parameters for the `kill_task` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KillTaskParams {
    /// Task ID to kill.
    pub task_id: String,
}

/// Parameters for the `research` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResearchParams {
    /// Research query.
    pub query: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Absolute file paths to include as context.
    #[serde(default)]
    pub files: Option<Vec<String>>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Parameters for the `analyze_code` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeCodeParams {
    /// Code snippet or absolute file path.
    pub code_or_path: String,
    /// Type of analysis: review, explain, optimize, security, test.
    pub analysis_type: AnalysisType,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Analysis types for the `analyze_code` tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisType {
    Review,
    Explain,
    Optimize,
    Security,
    Test,
}

impl AnalysisType {
    /// Return the instruction prefix for this analysis type.
    pub fn instruction_prefix(&self) -> &'static str {
        match self {
            Self::Review => "Review this code for correctness, design, and maintainability:\n\n",
            Self::Explain => "Explain what this code does, step by step:\n\n",
            Self::Optimize => "Suggest performance and readability improvements for this code:\n\n",
            Self::Security => "Audit this code for security vulnerabilities:\n\n",
            Self::Test => "Write comprehensive tests for this code:\n\n",
        }
    }
}

/// Parameters for the `clink` tool (isolated subagent spawn).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClinkParams {
    /// Custom system prompt for the subagent.
    pub system_prompt: String,
    /// Task prompt for the subagent.
    pub prompt: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

// ── Preset system ───────────────────────────────────────────────

/// Tool presets control which tool groups are exposed via `tools/list`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    Minimal,    // chat only
    Agent,      // chat + agent
    Research,   // chat + agent + research + analysis
    Full,       // all groups including subagent
}

impl Preset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "minimal" => Some(Self::Minimal),
            "agent" => Some(Self::Agent),
            "research" => Some(Self::Research),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    /// Returns the tool names included in this preset.
    pub fn tool_names(&self) -> Vec<&'static str> {
        match self {
            Self::Minimal => vec!["prompt"],
            Self::Agent => vec!["prompt", "run", "wait", "list_tasks", "get_result", "kill_task"],
            Self::Research => vec![
                "prompt", "run", "wait", "list_tasks", "get_result", "kill_task",
                "research", "analyze_code",
            ],
            Self::Full => vec![
                "prompt", "run", "wait", "list_tasks", "get_result", "kill_task",
                "research", "analyze_code", "clink",
            ],
        }
    }
}
```

### 2.2 The rmcp Server Implementation: `src/mcp/bridge/server.rs`

**New file.** This is the core of the bridge. It implements `ServerHandler`
using rmcp's proc macros.

```rust
use std::sync::Arc;
use std::time::Duration;

use rmcp::{
    ServerHandler, ServiceExt,
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
    handler::server::wrapper::Parameters,
};

use super::backends::BackendConfig;
use super::registry::{TaskRegistry, TaskSnapshot};
use super::tools::*;

/// Maximum characters in a synchronous tool response before truncation.
/// Approximately 20K tokens at 4 chars/token.
const MAX_RESPONSE_CHARS: usize = 80_000;

/// Maximum bytes to read from a single file for the `research` tool.
const MAX_FILE_BYTES: usize = 100 * 1024; // 100 KiB

/// The MCP bridge server handler.
///
/// This struct is the rmcp `ServerHandler` implementation. It holds shared
/// state (backends, registry, config) and routes tool calls via the
/// `ToolRouter` generated by the `#[tool_router]` macro.
#[derive(Clone)]
pub struct GreatBridge {
    backends: Arc<Vec<BackendConfig>>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GreatBridge {
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

    // ── chat group ──────────────────────────────────────────

    #[tool(description = "Send a prompt to an AI backend and get a synchronous response")]
    async fn prompt(
        &self,
        #[tool(param)] params: Parameters<PromptParams>,
    ) -> Result<CallToolResult, McpError> {
        let backend = self.resolve_backend(params.0.backend.as_deref())?;
        let (binary, args) = super::backends::build_command_args(
            backend,
            &params.0.prompt,
            params.0.model.as_deref(),
            None,
        );

        match self.run_sync(&binary, &args).await {
            Ok(output) => {
                let text = truncate_output(&output);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ── agent group ─────────────────────────────────────────

    #[tool(description = "Spawn an asynchronous AI backend task. Returns a task_id for later retrieval.")]
    async fn run(
        &self,
        #[tool(param)] params: Parameters<RunParams>,
    ) -> Result<CallToolResult, McpError> {
        let backend = self.resolve_backend(params.0.backend.as_deref())?;
        let timeout = params.0.timeout_secs.map(Duration::from_secs);

        match self.registry.spawn_task(backend, &params.0.prompt, timeout, None, None).await {
            Ok(task_id) => {
                let json = serde_json::json!({"task_id": task_id});
                Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(description = "Wait for one or more async tasks to complete. Returns results when all finish or timeout.")]
    async fn wait(
        &self,
        #[tool(param)] params: Parameters<WaitParams>,
    ) -> Result<CallToolResult, McpError> {
        let timeout = Duration::from_secs(params.0.timeout_secs.unwrap_or(300));
        let snapshots = self.registry.wait_for_tasks(&params.0.task_ids, timeout).await;
        let json = serde_json::to_string(&snapshots)
            .unwrap_or_else(|_| "[]".to_string());
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "List all tracked tasks with their current status.")]
    async fn list_tasks(&self) -> Result<CallToolResult, McpError> {
        let snapshots = self.registry.list_tasks().await;
        let json = serde_json::to_string(&snapshots)
            .unwrap_or_else(|_| "[]".to_string());
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get the result of a completed async task.")]
    async fn get_result(
        &self,
        #[tool(param)] params: Parameters<GetResultParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.registry.get_task(&params.0.task_id).await {
            Some(snapshot) => {
                let json = serde_json::to_string(&snapshot)
                    .unwrap_or_else(|_| "{}".to_string());
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(
                format!("task {} not found", params.0.task_id),
            )])),
        }
    }

    #[tool(description = "Kill a running async task.")]
    async fn kill_task(
        &self,
        #[tool(param)] params: Parameters<KillTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.registry.kill_task(&params.0.task_id).await {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text("killed")])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    // ── research group ──────────────────────────────────────

    #[tool(description = "Research a topic, optionally with file context. Sends a composite prompt to an AI backend.")]
    async fn research(
        &self,
        #[tool(param)] params: Parameters<ResearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let backend = self.resolve_backend(params.0.backend.as_deref())?;

        // Build composite prompt with file context
        let mut composite_prompt = String::new();
        if let Some(files) = &params.0.files {
            for path in files {
                match std::fs::read(path) {
                    Ok(bytes) => {
                        let content = if bytes.len() > MAX_FILE_BYTES {
                            let truncated = String::from_utf8_lossy(&bytes[..MAX_FILE_BYTES]);
                            format!("{}\n[truncated at {} bytes]", truncated, MAX_FILE_BYTES)
                        } else {
                            String::from_utf8_lossy(&bytes).to_string()
                        };
                        composite_prompt.push_str(&format!("--- FILE: {} ---\n{}\n\n", path, content));
                    }
                    Err(e) => {
                        composite_prompt.push_str(&format!("--- FILE: {} --- (error: {})\n\n", path, e));
                    }
                }
            }
        }
        composite_prompt.push_str(&params.0.query);

        let (binary, args) = super::backends::build_command_args(
            backend,
            &composite_prompt,
            params.0.model.as_deref(),
            None,
        );

        match self.run_sync(&binary, &args).await {
            Ok(output) => {
                let text = truncate_output(&output);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ── analysis group ──────────────────────────────────────

    #[tool(description = "Analyze code: review, explain, optimize, audit for security, or generate tests.")]
    async fn analyze_code(
        &self,
        #[tool(param)] params: Parameters<AnalyzeCodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let backend = self.resolve_backend(params.0.backend.as_deref())?;

        // Resolve code_or_path
        let code = if std::path::Path::new(&params.0.code_or_path).exists() {
            match std::fs::read_to_string(&params.0.code_or_path) {
                Ok(content) => content,
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(
                    format!("failed to read file {}: {}", params.0.code_or_path, e),
                )])),
            }
        } else {
            params.0.code_or_path.clone()
        };

        let prompt = format!("{}{}", params.0.analysis_type.instruction_prefix(), code);

        let (binary, args) = super::backends::build_command_args(
            backend,
            &prompt,
            params.0.model.as_deref(),
            None,
        );

        match self.run_sync(&binary, &args).await {
            Ok(output) => {
                let text = truncate_output(&output);
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }

    // ── subagent group ──────────────────────────────────────

    #[tool(description = "Spawn an isolated AI subagent with a custom system prompt. Returns a task_id.")]
    async fn clink(
        &self,
        #[tool(param)] params: Parameters<ClinkParams>,
    ) -> Result<CallToolResult, McpError> {
        let backend = self.resolve_backend(params.0.backend.as_deref())?;

        match self.registry.spawn_task(
            backend,
            &params.0.prompt,
            None,
            params.0.model.as_deref(),
            Some(&params.0.system_prompt),
        ).await {
            Ok(task_id) => {
                let json = serde_json::json!({"task_id": task_id});
                Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }
}

// ── ServerHandler implementation ────────────────────────────────

#[tool_handler]
impl ServerHandler for GreatBridge {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "great-mcp-bridge".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("great.sh MCP Bridge".to_string()),
                ..Default::default()
            },
            instructions: Some(
                "MCP bridge to AI CLI backends (gemini, codex, claude, grok, ollama). \
                 Use 'prompt' for synchronous queries, 'run'/'wait' for async tasks."
                    .to_string(),
            ),
        }
    }
}

// ── Private helpers ─────────────────────────────────────────────

impl GreatBridge {
    /// Resolve which backend to use from an optional name.
    fn resolve_backend(&self, name: Option<&str>) -> Result<&BackendConfig, McpError> {
        match name {
            Some(n) => self.backends.iter().find(|b| b.name == n).ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "backend '{}' not available. Available: {}",
                        n,
                        self.backends.iter().map(|b| b.name).collect::<Vec<_>>().join(", ")
                    ),
                    None,
                )
            }),
            None => {
                // Try default_backend, then first available
                if let Some(ref default) = self.default_backend {
                    self.backends.iter().find(|b| b.name == default.as_str())
                } else {
                    self.backends.first()
                }
                .ok_or_else(|| {
                    McpError::invalid_params("no backends available".to_string(), None)
                })
            }
        }
    }

    /// Execute a backend command synchronously (with timeout).
    async fn run_sync(&self, binary: &str, args: &[String]) -> Result<String, String> {
        let mut cmd = tokio::process::Command::new(binary);
        cmd.args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let child = cmd.spawn().map_err(|e| format!("spawn failed: {}", e))?;

        let timeout = self.registry.default_timeout;
        match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!(
                        "exit code {}: {}\n{}",
                        output.status.code().unwrap_or(-1),
                        stdout,
                        stderr,
                    ))
                }
            }
            Ok(Err(e)) => Err(format!("process error: {}", e)),
            Err(_) => Err(format!("timeout after {}s", timeout.as_secs())),
        }
    }
}

/// Truncate output to MAX_RESPONSE_CHARS, appending a notice if truncated.
fn truncate_output(text: &str) -> String {
    if text.len() > MAX_RESPONSE_CHARS {
        let mut result = text[..MAX_RESPONSE_CHARS].to_string();
        result.push_str("\n\n[output truncated at 80,000 chars -- use `run`/`wait` for full async output]");
        result
    } else {
        text.to_string()
    }
}

/// Start the bridge server on stdio. This is the main entry point
/// called by `cli/mcp_bridge.rs`.
pub async fn start_bridge(
    backends: Vec<BackendConfig>,
    default_backend: Option<String>,
    registry: TaskRegistry,
    preset: Preset,
) -> anyhow::Result<()> {
    let bridge = GreatBridge::new(backends, default_backend, registry, preset);

    tracing::info!(
        "great-mcp-bridge starting (preset={:?}, backends={})",
        preset,
        bridge.backends.iter().map(|b| b.name).collect::<Vec<_>>().join(","),
    );

    let service = bridge.serve(stdio()).await.map_err(|e| {
        anyhow::anyhow!("failed to start MCP bridge server: {}", e)
    })?;

    service.waiting().await.map_err(|e| {
        anyhow::anyhow!("bridge server error: {}", e)
    })?;

    Ok(())
}
```

**CRITICAL rmcp API notes for the builder:**

1. The `#[tool_router]` attribute on the `impl GreatBridge` block generates a
   `fn tool_router() -> ToolRouter<Self>` associated function. Each method
   annotated with `#[tool(...)]` is registered in the router.

2. The `#[tool_handler]` attribute on the `impl ServerHandler for GreatBridge`
   block generates the `list_tools()` and `call_tool()` dispatch. It delegates
   to the `ToolRouter` stored in `self.tool_router`.

3. `Parameters<T>` extracts and deserializes the `arguments` JSON object from
   the MCP `tools/call` request into type `T`. The `#[tool(param)]` attribute
   tells the macro to use `Parameters` extraction.

4. The `ServerHandler::get_info()` return type is `ServerInfo`, which is a type
   alias for `InitializeResult`. The protocol version defaults to
   `ProtocolVersion::LATEST` (currently `"2025-03-26"`).

5. `CallToolResult::success(vec![Content::text(...)])` is the standard success
   return. `CallToolResult::error(...)` sets `is_error: Some(true)`.

6. `stdio()` returns `(tokio::io::Stdin, tokio::io::Stdout)` which implements
   `IntoTransport` for the server role.

7. `.serve(transport)` performs the MCP initialize handshake and returns a
   `RunningService`. `.waiting()` blocks until the transport closes (stdin
   EOF or client disconnect).

**Preset filtering consideration:** The `#[tool_router]` macro registers all
9 tools. To filter by preset, the builder has two options:

- **Option A (recommended):** Override `list_tools()` in the `ServerHandler`
  impl to filter the tool list by preset. Tool calls to disabled tools return
  a clear error. This is the simplest approach.

- **Option B:** Build the `ToolRouter` dynamically based on preset. This
  requires not using `#[tool_router]` and instead manually constructing the
  router. More complex but prevents clients from even seeing disabled tools.

The builder should implement Option A first, then upgrade to Option B if
needed. For Option A, add this to the `ServerHandler` impl:

```rust
fn list_tools(
    &self,
    _request: ListToolsRequestParams,
    _context: RequestContext<RoleServer>,
) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
    let allowed = self.preset.tool_names();
    let all_tools = self.tool_router.list_all();
    let filtered: Vec<Tool> = all_tools
        .into_iter()
        .filter(|t| allowed.contains(&t.name.as_ref()))
        .collect();
    std::future::ready(Ok(ListToolsResult {
        tools: filtered,
        next_cursor: None,
    }))
}
```

**Builder must verify:** The exact method signature for `list_tools()` in the
`ServerHandler` trait. Check `src/handler/server.rs` line ~304 in the rmcp
source for the current signature.

### 2.3 Module Wiring: `src/mcp/bridge/mod.rs`

**New file:**

```rust
pub mod backends;
pub mod registry;
pub mod server;
pub mod tools;
```

**Add to `src/mcp/mod.rs`** (line 1 area):

```rust
pub mod bridge;
```

---

## Phase 3: CLI Integration (Depends on Phase 2)

### 3.1 New Subcommand: `src/cli/mcp_bridge.rs`

**New file.** Standard great.sh CLI module pattern.

```rust
use anyhow::{Context, Result};
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::config;
use crate::mcp::bridge::backends::discover_backends;
use crate::mcp::bridge::registry::TaskRegistry;
use crate::mcp::bridge::server::start_bridge;
use crate::mcp::bridge::tools::Preset;

/// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required.
#[derive(ClapArgs)]
pub struct Args {
    /// Tool preset: minimal, chat, agent, full (default: agent).
    /// Controls which tool groups are exposed via tools/list.
    #[arg(long, default_value = "agent")]
    pub preset: String,

    /// Comma-separated list of enabled backends: gemini, codex, claude, grok, ollama.
    /// Default: auto-detect all installed.
    #[arg(long, value_delimiter = ',')]
    pub backends: Option<Vec<String>>,

    /// Per-task timeout in seconds (default: 300).
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Logging verbosity for stderr: off, error, warn, info, debug (default: warn).
    #[arg(long, default_value = "warn")]
    pub log_level: String,
}

pub fn run(args: Args) -> Result<()> {
    // Initialize tracing to stderr
    let filter = match args.log_level.as_str() {
        "off" => "off",
        "error" => "error",
        "warn" => "warn",
        "info" => "info",
        "debug" => "debug",
        other => {
            eprintln!("warning: unknown log level '{}', defaulting to 'warn'", other);
            "warn"
        }
    };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    // Load config if available (non-fatal if missing)
    let bridge_config = config::discover_config()
        .ok()
        .and_then(|path| config::load(path.to_str()).ok())
        .and_then(|cfg| cfg.mcp_bridge);

    // Merge config with CLI args (CLI args win)
    let backend_filter: Vec<String> = args.backends
        .or_else(|| bridge_config.as_ref().and_then(|c| c.backends.clone()))
        .unwrap_or_default();

    let default_backend = bridge_config.as_ref().and_then(|c| c.default_backend.clone());

    let timeout_secs = if args.timeout != 300 {
        args.timeout  // CLI explicitly set
    } else {
        bridge_config.as_ref().and_then(|c| c.timeout_secs).unwrap_or(300)
    };

    let preset_str = if args.preset != "agent" {
        args.preset.clone()  // CLI explicitly set
    } else {
        bridge_config.as_ref().and_then(|c| c.preset.clone()).unwrap_or_else(|| "agent".to_string())
    };

    let preset = Preset::from_str(&preset_str)
        .context(format!("invalid preset '{}' -- use: minimal, agent, research, full", preset_str))?;

    // Discover backends
    let backends = discover_backends(&backend_filter);
    if backends.is_empty() {
        anyhow::bail!(
            "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama"
        );
    }

    tracing::info!(
        "Discovered backends: {}",
        backends.iter().map(|b| b.name).collect::<Vec<_>>().join(", ")
    );

    // Create registry and start the server
    let registry = TaskRegistry::new(timeout_secs);

    // Build and run the tokio runtime
    let rt = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
    rt.block_on(start_bridge(backends, default_backend, registry, preset))
}
```

**Note:** We create a new tokio runtime here rather than using `#[tokio::main]`
because `main.rs` currently uses a synchronous `fn main()` pattern. The bridge
is the only subcommand that needs async, so we create the runtime on demand.
This avoids making `main()` async, which would affect all subcommands.

### 3.2 Register in Command Enum

**File:** `src/cli/mod.rs`

Add module declaration (after line 7, `pub mod mcp;`):

```rust
pub mod mcp_bridge;
```

Add variant to `Command` enum (after line 77, `Statusline` variant):

```rust
    /// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) -- no Node.js required
    #[command(name = "mcp-bridge")]
    McpBridge(mcp_bridge::Args),
```

The `#[command(name = "mcp-bridge")]` attribute ensures the subcommand is
invoked as `great mcp-bridge` (with hyphen), while the Rust variant uses
CamelCase `McpBridge`.

### 3.3 Dispatch in main.rs

**File:** `src/main.rs`

Add after line 38 (`Command::Statusline(args) => cli::statusline::run(args),`):

```rust
        Command::McpBridge(args) => cli::mcp_bridge::run(args),
```

---

## Phase 4: Integration with Existing Commands (Depends on Phase 3)

### 4.1 `great apply` Bridge Registration

**File:** `src/cli/apply.rs`

Add a new section after the MCP servers block (after line 723, the closing
`}` of the `println!()` after MCP servers). This goes before the bitwarden
section (line 727).

```rust
    // 5a. Register MCP bridge in .mcp.json
    if cfg.mcp_bridge.is_some() {
        output::header("MCP Bridge");

        let mcp_json_path = crate::mcp::project_mcp_path();
        let mut mcp_json = crate::mcp::McpJsonConfig::load(&mcp_json_path)
            .unwrap_or_default();

        // Build desired args
        let mut bridge_args = vec!["mcp-bridge".to_string()];
        if let Some(ref bridge_cfg) = cfg.mcp_bridge {
            if let Some(ref preset) = bridge_cfg.preset {
                bridge_args.push("--preset".to_string());
                bridge_args.push(preset.clone());
            }
            if let Some(ref backends) = bridge_cfg.backends {
                bridge_args.push("--backends".to_string());
                bridge_args.push(backends.join(","));
            }
        }

        let desired_entry = crate::mcp::McpServerEntry {
            command: "great".to_string(),
            args: Some(bridge_args.clone()),
            env: None,
        };

        // Check if entry already exists with matching args
        let needs_update = if let Some(existing) = mcp_json.mcp_servers.get("great-bridge") {
            existing.args.as_ref() != Some(&bridge_args) || existing.command != "great"
        } else {
            true
        };

        if needs_update {
            if args.dry_run {
                output::info("  great-bridge -- would register in .mcp.json");
            } else {
                mcp_json.mcp_servers.insert("great-bridge".to_string(), desired_entry);
                if let Err(e) = mcp_json.save(&mcp_json_path) {
                    output::error(&format!("  great-bridge -- failed to write .mcp.json: {}", e));
                } else {
                    output::success("  great-bridge -- registered in .mcp.json");
                }
            }
        } else {
            output::success("  great-bridge -- already registered in .mcp.json");
        }

        println!();
    }
```

**Idempotency:** The code compares existing entry args against desired args.
If the user changes `[mcp-bridge]` options in great.toml, `great apply` will
update the `.mcp.json` entry. If args match, it skips (success message).

### 4.2 `great doctor` Bridge Checks

**File:** `src/cli/doctor.rs`

Add a new check function and call it from `run()` after the MCP server
checks (after line 90, `check_mcp_servers(&mut result, cfg);`).

Add the call:

```rust
    // 7b. MCP Bridge backend checks
    check_mcp_bridge(&mut result);
```

Add the function (after `check_mcp_servers`, around line 603):

```rust
/// Check MCP bridge backend availability and .mcp.json registration.
fn check_mcp_bridge(result: &mut DiagnosticResult) {
    output::header("MCP Bridge");

    // Check each backend binary
    let backends = [
        ("gemini", "Gemini CLI", "GEMINI_API_KEY"),
        ("codex", "Codex CLI", "OPENAI_API_KEY"),
        ("claude", "Claude CLI", ""),
        ("grok", "Grok CLI", "XAI_API_KEY"),
        ("ollama", "Ollama", ""),
    ];

    let mut any_found = false;
    for (binary, name, api_key_env) in &backends {
        if command_exists(binary) {
            any_found = true;
            if api_key_env.is_empty() {
                // No API key needed (uses login or local)
                pass(result, &format!("{}: installed", name));
            } else if std::env::var(api_key_env).is_ok() {
                pass(result, &format!("{}: installed, {} set", name, api_key_env));
            } else {
                warn(
                    result,
                    &format!("{}: installed, {} not set", name, api_key_env),
                );
            }
        } else {
            // Not found is informational, not a failure
            output::info(&format!("  {}: not found (optional)", name));
        }
    }

    if !any_found {
        warn(
            result,
            "No MCP bridge backends found. Install at least one: gemini, codex, claude, grok, ollama",
        );
    }

    // Check .mcp.json registration
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

    println!();
}
```

---

## Phase 5: Tests

### 5.1 Unit Tests

Each new module includes inline `#[cfg(test)] mod tests` with:

- `backends.rs`: Test `BACKEND_SPECS` completeness, `build_command_args`
  for each backend variant, `discover_backends` with empty filter.
- `tools.rs`: Test `Preset::tool_names()` counts (minimal=1, agent=6,
  research=8, full=9), `AnalysisType::instruction_prefix()` for all variants.
- `registry.rs`: Test `TaskRegistry::new()` creates empty registry,
  `snapshot()` produces valid JSON.

### 5.2 Integration Tests

Add to `tests/cli_smoke.rs`:

```rust
// -----------------------------------------------------------------------
// MCP Bridge
// -----------------------------------------------------------------------

#[test]
fn mcp_bridge_help_shows_description() {
    great()
        .args(["mcp-bridge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP bridge server"));
}

#[test]
fn mcp_bridge_unknown_preset_fails() {
    great()
        .args(["mcp-bridge", "--preset", "invalid"])
        .assert()
        .failure();
}
```

### 5.3 Protocol Smoke Test (Manual / CI Script)

The acceptance criteria specify a pipe test. Create a shell script at
`tests/mcp_bridge_protocol.sh` (not run by `cargo test`, used manually
or in CI):

```bash
#!/usr/bin/env bash
set -euo pipefail

# This test requires at least one backend on PATH (e.g., "echo" as a stub)
RESULT=$(printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n' \
  | timeout 10 cargo run -- mcp-bridge --preset minimal --log-level off 2>/dev/null)

echo "$RESULT" | grep -q '"protocolVersion"' || { echo "FAIL: no protocolVersion"; exit 1; }
echo "$RESULT" | grep -q '"tools"' || { echo "FAIL: no tools"; exit 1; }
echo "PASS: MCP bridge protocol smoke test"
```

---

## File Change Summary

| File | Action | Phase |
|------|--------|-------|
| `Cargo.toml` | Modify: add rmcp, uuid, tracing, tracing-subscriber, schemars | 1 |
| `src/config/schema.rs` | Modify: add `McpBridgeConfig` struct and field on `GreatConfig`, add validation | 1 |
| `src/mcp/bridge/mod.rs` | Create: module root | 1 |
| `src/mcp/bridge/backends.rs` | Create: `BackendConfig`, `discover_backends()`, `build_command_args()` | 1 |
| `src/mcp/bridge/registry.rs` | Create: `TaskState`, `TaskRegistry`, process spawning | 1 |
| `src/mcp/bridge/tools.rs` | Create: parameter structs, `Preset` enum | 2 |
| `src/mcp/bridge/server.rs` | Create: `GreatBridge` rmcp server, `start_bridge()` | 2 |
| `src/mcp/mod.rs` | Modify: add `pub mod bridge;` | 2 |
| `src/cli/mcp_bridge.rs` | Create: `Args`, `pub fn run()` | 3 |
| `src/cli/mod.rs` | Modify: add `pub mod mcp_bridge;` and `McpBridge` variant | 3 |
| `src/main.rs` | Modify: add dispatch arm | 3 |
| `src/cli/apply.rs` | Modify: add bridge registration block | 4 |
| `src/cli/doctor.rs` | Modify: add `check_mcp_bridge()` | 4 |
| `tests/cli_smoke.rs` | Modify: add bridge smoke tests | 5 |
| `tests/mcp_bridge_protocol.sh` | Create: protocol pipe test script | 5 |

**Total new files:** 7
**Total modified files:** 8

---

## Edge Cases

### Empty/No Backends
- `great mcp-bridge` with no backends on PATH: exits with a clear error
  message listing which binaries to install. Does NOT start the server.
- `--backends nonexistent`: exits with error, the `discover_backends()` filter
  produces an empty vec.

### Stdin EOF / Broken Pipe
- When the MCP client disconnects (stdin EOF), rmcp's transport layer
  detects the closed stream and `service.waiting()` returns. The bridge
  then calls `registry.shutdown_all()` to kill lingering processes.
- Broken stdout pipe (client crashes): tokio write errors propagate,
  rmcp handles transport errors and shuts down.

### Concurrent Tool Calls
- MCP clients may issue multiple `tools/call` requests concurrently. The
  `TaskRegistry` uses `Arc<Mutex<HashMap>>` for thread safety. The `Mutex`
  is only held briefly (insert/lookup), not across await points, so
  contention is minimal.

### Large Tool Responses
- Synchronous tools (`prompt`, `research`, `analyze_code`) truncate output
  at 80,000 characters with a notice. Async tools (`run`/`get_result`) return
  full output since the client explicitly retrieves it.

### File Access in `research` / `analyze_code`
- Non-existent files: error message included in response, not a server crash.
- Files larger than 100 KiB: truncated with notice.
- Permission denied: reported as error in the file context section.
- Symbolic links: followed (standard `std::fs::read` behavior).

### Platform Differences
- **macOS ARM64/x86_64:** `which` crate uses PATH lookup, works on both
  architectures. `setpgid`/`killpg` available via libc.
- **Ubuntu (x86_64):** Same as macOS for process management. Some AI CLIs
  may not have Linux builds (e.g., grok CLI) -- `discover_backends` simply
  skips them.
- **WSL2:** Process management works like native Linux. The bridge binary
  runs inside WSL, spawning Linux CLI tools. Windows-side tools (e.g.,
  Windows-native Ollama) are accessible if in the WSL PATH.
- **Windows (native):** `setpgid`/`killpg` calls are behind `#[cfg(unix)]`
  gates. On Windows, `kill_on_drop(true)` is the fallback. Process group
  cleanup is less thorough -- this is documented and acceptable for the
  initial release.

### Config Merging
- CLI flags take precedence over `great.toml` `[mcp-bridge]` values.
- Missing `great.toml` or missing `[mcp-bridge]` section: all defaults used.
- Invalid preset in config: validation warning (does not block `great apply`).

### Idempotent Apply
- `great apply` checks if `great-bridge` entry exists in `.mcp.json` with
  matching args. If args differ (user changed config), the entry is
  overwritten. If identical, it's skipped with a success message.

---

## Error Handling

| Scenario | Behavior | Message |
|----------|----------|---------|
| No backends on PATH | `run()` returns `Err` | "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama" |
| Invalid preset | `run()` returns `Err` | "invalid preset 'X' -- use: minimal, agent, research, full" |
| Backend binary not found at runtime | Tool returns MCP error | "spawn failed: No such file or directory" |
| Backend process timeout | Tool returns MCP error content | "timeout after Ns" |
| Backend non-zero exit | Tool returns error content | "exit code N: stdout\nstderr" |
| Task ID not found | `get_result`/`kill_task` returns error | "task UUID not found" |
| Kill non-running task | `kill_task` returns error | "task UUID is not running (state: Completed)" |
| Malformed tool arguments | rmcp returns JSON-RPC `-32602` | "failed to deserialize parameters: ..." |
| Unknown tool name | rmcp returns appropriate error | Handled by rmcp framework |
| File read error | Error in tool response | "failed to read file /path: Permission denied" |

---

## Security Considerations

1. **Command injection:** The bridge spawns CLI subprocesses using
   `tokio::process::Command` with explicit argument vectors (no shell
   expansion). Prompts are passed as a single argument string, never
   interpolated into a shell command. This prevents injection.

2. **API key exposure:** API keys are resolved from environment variables at
   the backend level (the AI CLI reads them). The bridge never logs or
   transmits API keys. The `BackendConfig.api_key_env` field stores only the
   *name* of the env var, not the value.

3. **File access:** The `research` and `analyze_code` tools read files from
   the filesystem. These paths come from the MCP client (i.e., the AI
   assistant). Since the bridge runs with the user's permissions and the AI
   assistant already has filesystem access, this does not escalate privileges.
   The 100 KiB per-file limit prevents accidental memory exhaustion.

4. **`--dangerously-skip-permissions` on Claude:** This flag is used to bypass
   Claude Code's interactive approval. It is the user's explicit choice to
   enable this backend in the bridge configuration. The flag name itself
   communicates the risk.

5. **Denial of service:** The `TaskRegistry` cleanup (30-minute TTL on terminal
   tasks) prevents unbounded memory growth. The per-task timeout prevents
   runaway processes. The MAX_RESPONSE_CHARS truncation prevents large
   responses from consuming client token budgets.

---

## Risks and Builder Verification Items

1. **CLI flag conventions are unstable.** The `-p` flag, `--model`, and
   auto-approve flags may change across AI CLI versions. The builder MUST
   verify the current flag conventions for each CLI before implementing
   `build_command_args()`. Use `<binary> --help` output to confirm.

2. **rmcp `#[tool]` macro behavior with `Parameters<T>`.** The spec shows
   `#[tool(param)]` for parameter extraction. The builder should verify this
   is the correct attribute syntax by checking the `rmcp-macros` source or
   the rmcp examples at `github.com/anthropics/mcp-rust-sdk/tree/main/examples`.
   The Counter example in the README uses bare parameters (not `Parameters<T>`),
   so the builder may need to adjust.

3. **rmcp `list_tools` override.** The spec describes overriding `list_tools()`
   to filter by preset. The builder must verify the exact trait method
   signature in `rmcp::handler::server::ServerHandler`. If `#[tool_handler]`
   conflicts with a manual override, the builder should use the dynamic
   `ToolRouter` approach instead.

4. **`ProtocolVersion::LATEST` value.** As of rmcp 0.16.0, `LATEST` is
   `"2025-03-26"`. The task file says `"2024-11-05"`. Use whatever rmcp's
   `Default` produces -- do NOT hardcode a version string.

5. **`schemars` version alignment.** rmcp 0.16 uses `schemars` 1.0. The
   builder must ensure the `schemars` version in `Cargo.toml` matches or
   is compatible with rmcp's transitive dependency. A version mismatch will
   cause derive macro errors.

6. **`tracing_subscriber::fmt().init()` can only be called once.** If other
   parts of the `great` binary also initialize tracing in the future, this
   will panic. Currently no other code initializes tracing, so this is safe.
   The builder should use `.try_init()` instead of `.init()` as a defensive
   measure.

7. **`tokio::runtime::Runtime::new()` inside `main()`.** The current `main()`
   is synchronous. Creating a runtime per-invocation for `mcp-bridge` is fine,
   but the builder should verify that no other subcommand also creates a
   runtime (which could cause nested runtime errors). Currently `apply.rs`
   uses `reqwest::blocking`, not a tokio runtime, so this is safe.

8. **`unsafe` blocks for `pre_exec` and `killpg`.** These require `// SAFETY:`
   comments explaining the invariants. `pre_exec` runs between `fork()` and
   `exec()` -- only async-signal-safe functions may be called. `setpgid(0, 0)`
   is async-signal-safe (POSIX).

---

## Build Order

The recommended implementation sequence minimizes compile-test cycles:

1. **`Cargo.toml`** -- add dependencies, `cargo check` to verify resolution
2. **`src/config/schema.rs`** -- `McpBridgeConfig`, field on `GreatConfig`,
   validation, unit tests. Run `cargo test config::`.
3. **`src/mcp/bridge/mod.rs`** + **`backends.rs`** -- backend discovery.
   Run `cargo test mcp::bridge::backends`.
4. **`src/mcp/bridge/registry.rs`** -- task registry. Run
   `cargo test mcp::bridge::registry`.
5. **`src/mcp/bridge/tools.rs`** -- parameter structs and presets. Run
   `cargo test mcp::bridge::tools`.
6. **`src/mcp/bridge/server.rs`** -- rmcp server. This is the hardest file;
   compile errors here likely relate to rmcp macro usage.
   Run `cargo check` first.
7. **`src/cli/mcp_bridge.rs`** + **`src/cli/mod.rs`** + **`src/main.rs`** --
   wire up the subcommand. Run `cargo run -- mcp-bridge --help`.
8. **`src/cli/apply.rs`** -- bridge registration. Run `cargo test` and
   manually verify with a test `great.toml`.
9. **`src/cli/doctor.rs`** -- bridge checks. Run `great doctor`.
10. **`tests/cli_smoke.rs`** -- integration tests. Run `cargo test`.
11. **`cargo clippy -- -D warnings`** -- fix any new warnings.
12. **Protocol smoke test** -- manual pipe test per acceptance criteria.

---

## Acceptance Criteria Verification

- [AC1] `great mcp-bridge` starts without error when backend available:
  Verified by the protocol pipe test in Phase 5.3.
- [AC2] Preset filtering: `--preset minimal` exposes 1 tool, `--preset full`
  exposes 9 tools. Verified by parsing `tools/list` response.
- [AC3] `great apply` writes `great-bridge` to `.mcp.json`: Verified by
  jq assertion. Idempotency verified by running apply twice.
- [AC4] `great doctor` reports backends: Verified by running `great doctor`
  on a machine with at least one AI CLI installed.
- [AC5] `cargo test` passes, `cargo clippy -- -D warnings` clean: Verified
  in the standard CI pipeline.
