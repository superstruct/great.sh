# 0029: Inbuilt MCP Bridge Server

**Priority:** P1
**Type:** feature
**Module:** `src/cli/mcp_bridge.rs` (new) + `src/mcp/bridge/` (new) + `src/cli/mod.rs` + `src/main.rs` + `src/config/schema.rs` + `src/cli/doctor.rs` + `src/cli/apply.rs`
**Status:** open
**Date:** 2026-02-27
**Estimated Complexity:** XL

---

## Context

great.sh currently relies on external Node.js/npm packages to bridge AI CLI tools
into the MCP ecosystem. Three representative packages exist in the wild:

- **gemini-mcp** (RLabs-Inc) — Node.js MCP server bridging to Gemini API via HTTP;
  37 tools across 18 groups; tool preset system for context efficiency.
- **ai-cli-mcp / claude-code-mcp** (mkXultra) — Node.js MCP server that spawns AI
  CLI subprocesses (Claude CLI, Codex CLI, Gemini CLI); agent-in-agent model with
  async task registry; tools: `run`, `wait`, `list_processes`, `get_result`,
  `kill_process`.
- **pal-mcp-server** (BeehiveInnovations) — Node.js "Provider Abstraction Layer";
  50+ models across 7 providers; `clink` tool spawns isolated subagents with custom
  system prompts; token management for MCP's 25K token limit.

All three are npm packages, so every great.sh user must have Node.js installed and
run an npm install step before the bridge is available. This contradicts the
great.sh promise of a self-contained, single-binary AI dev environment.

This task implements an equivalent MCP bridge directly inside the `great` Rust
binary. The new `great mcp-bridge` subcommand starts a stdio MCP server (JSON-RPC
2.0) exposing multi-provider AI CLI tools without any npm/Node.js dependency.

### Existing surface area that this task builds on

`src/mcp/mod.rs` already provides:
- `McpJsonConfig` and `McpServerEntry` structs (lines 11-29) — the `.mcp.json`
  wire format; used by `great apply` and `great mcp list/add/test`.
- `resolve_env()` (line 79) — `${VAR}` expansion in env maps; reuse for bridge
  backend config.
- `test_server()` (line 107) — spawns and probes MCP server processes; the bridge
  itself becomes a server that passes this test.
- `project_mcp_path()` (line 93) — `.mcp.json` file location.

`src/config/schema.rs` provides:
- `McpConfig` struct (lines 92-108) — `command`, `args`, `env`, `transport`,
  `url`, `enabled`; the bridge self-registers as an `McpConfig` entry.
- `GreatConfig` (lines 10-24) — top-level config; a new `[mcp-bridge]` section
  will be added here.

`src/cli/mod.rs` (lines 41-78) — `Command` enum; a new `MpcBridge` arm is added.

`src/main.rs` (lines 17-36) — dispatch match; one new arm.

`Cargo.toml` already includes `tokio = { version = "1.0", features = ["full"] }`
(line 16), `serde_json = "1.0"` (line 13), `anyhow = "1.0"` (line 11). No new
runtime dependencies are needed for the core bridge. The `which` crate (line 25)
is already present for backend discovery.

---

## Requirements

### R1 — New subcommand: `great mcp-bridge`

Add a `MpcBridge(mcp_bridge::Args)` arm to the `Command` enum in
`src/cli/mod.rs` and the dispatch match in `src/main.rs`. Create
`src/cli/mcp_bridge.rs` with the standard `Args` struct and `pub fn run(args:
Args) -> Result<()>`.

`Args` fields (all optional with defaults):
- `--preset <NAME>` — tool preset: `minimal`, `chat`, `agent`, `full` (default:
  `agent`). Controls which tool groups are exposed via `tools/list`.
- `--backends <LIST>` — comma-separated list of enabled backends: `gemini`,
  `codex`, `claude`, `grok`, `ollama` (default: auto-detect all installed).
- `--timeout <SECS>` — per-task timeout in seconds (default: 300).
- `--log-level <LEVEL>` — logging verbosity for stderr: `off`, `error`, `warn`,
  `info`, `debug` (default: `warn`).

The `run()` function:
1. Reads backend config from `great.toml` `[mcp-bridge]` section if present,
   merging with CLI flags (CLI flags win).
2. Detects available backends via `which` (see R3).
3. Instantiates the process registry (see R4).
4. Starts the MCP stdio server loop (see R2).

The subcommand must be documented in the clap `about` string as:
"Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) — no Node.js required."

### R2 — MCP stdio server loop (JSON-RPC 2.0 over stdin/stdout)

Implement the MCP protocol lifecycle in `src/mcp/bridge/server.rs`:

**Protocol messages to handle (in order):**
1. `initialize` request — respond with `InitializeResult` containing:
   - `protocolVersion`: `"2024-11-05"` (current stable MCP version).
   - `capabilities.tools`: `{}` (tools supported).
   - `serverInfo`: `{"name": "great-mcp-bridge", "version": env!("CARGO_PKG_VERSION")}`.
2. `notifications/initialized` — no-op acknowledgement; log to stderr.
3. `tools/list` — respond with the tool manifest filtered by active preset (see R5).
4. `tools/call` — dispatch to the appropriate tool handler (see R5, R6).
5. Unknown methods — respond with JSON-RPC error code `-32601` ("Method not found").

**Wire format requirements:**
- Read newline-delimited JSON from stdin; each line is one JSON-RPC message.
- Write newline-delimited JSON to stdout; each response on a single line.
- All log output goes to stderr (not captured by MCP clients).
- Use `tokio::io::AsyncBufReadExt` for async line-by-line stdin reading.
- Use `tokio::io::AsyncWriteExt` for stdout writes.
- Each response must be flushed immediately after write.

**Error handling:**
- Malformed JSON on stdin: write JSON-RPC parse error (`-32700`) and continue.
- Tool execution error: return JSON-RPC error in the `tools/call` result (do not
  crash the server).
- Shutdown: on stdin EOF, initiate graceful shutdown (kill all running tasks,
  await their processes).

All JSON-RPC types (Request, Response, Error, Notification) must be defined as
serde structs in `src/mcp/bridge/protocol.rs`.

### R3 — Backend discovery and configuration

Implement backend discovery in `src/mcp/bridge/backends.rs`:

**Struct `BackendConfig`:**
```rust
pub struct BackendConfig {
    pub name: &'static str,       // "gemini", "codex", "claude", "grok", "ollama"
    pub binary: String,           // resolved path from which/env override
    pub model: Option<String>,    // --model flag value if backend supports it
    pub auto_approve_flag: &'static str, // bypass interactive approval
    pub api_key_env: Option<&'static str>, // env var name for API key
}
```

**Per-backend defaults:**
| Backend | Default binary | Auto-approve flag | API key env |
|---------|---------------|-------------------|-------------|
| `gemini` | `gemini` | `-y` | `GEMINI_API_KEY` |
| `codex` | `codex` | `--full-auto` | `OPENAI_API_KEY` |
| `claude` | `claude` | `--dangerously-skip-permissions` | (uses login) |
| `grok` | `grok` | `-y` | `XAI_API_KEY` |
| `ollama` | `ollama` | (none) | (none) |

**Environment variable overrides** (checked before `which` lookup):
- `GREAT_GEMINI_CLI`, `GREAT_CODEX_CLI`, `GREAT_CLAUDE_CLI`, `GREAT_GROK_CLI`,
  `GREAT_OLLAMA_CLI` — override the binary path for the corresponding backend.

**Discovery function:**
```rust
pub fn discover_backends(enabled: &[&str]) -> Vec<BackendConfig>
```
Uses the `which` crate (already in `Cargo.toml` line 25) to resolve each binary.
Returns only backends whose binary exists on PATH (or env override). If `enabled`
is empty, all discovered backends are returned.

**`[mcp-bridge]` section in `great.toml`** (add to `GreatConfig` in
`src/config/schema.rs`):
```toml
[mcp-bridge]
backends = ["gemini", "claude"]   # optional: restrict to subset
default_backend = "gemini"         # optional: used when tool call omits backend
timeout_secs = 300                 # optional: per-task timeout
preset = "agent"                   # optional: tool preset
```

Add `MpcBridgeConfig` struct to `src/config/schema.rs` and a `mcp_bridge:
Option<MpcBridgeConfig>` field to `GreatConfig`.

### R4 — Async process registry

Implement the task registry in `src/mcp/bridge/registry.rs`:

**`TaskState` enum:**
```rust
pub enum TaskState {
    Running { pid: u32, started_at: Instant },
    Completed { exit_code: i32, stdout: String, stderr: String, duration: Duration },
    Failed { error: String, duration: Duration },
    TimedOut { duration: Duration },
    Killed,
}
```

**`TaskHandle` struct:**
```rust
pub struct TaskHandle {
    pub task_id: String,          // UUID v4
    pub backend: String,
    pub prompt_preview: String,   // first 80 chars of prompt (for list_tasks display)
    pub state: TaskState,
    pub child: Option<tokio::process::Child>,
}
```

**`TaskRegistry` struct:**
- `tasks: Arc<Mutex<HashMap<String, TaskHandle>>>` — shared across async tasks.
- `fn spawn_task(backend: &BackendConfig, prompt: &str, timeout: Duration) -> Result<String>` —
  spawns the CLI subprocess with `tokio::process::Command`, captures stdout/stderr
  via `Stdio::piped()`, inserts a `Running` handle, and returns the `task_id`.
  The actual process output is collected in a background `tokio::spawn` task that
  updates the registry to `Completed`/`Failed` on exit.
- `fn get_task(&self, task_id: &str) -> Option<TaskSnapshot>` — returns a
  `TaskSnapshot` (non-owning view: state + preview) for `get_result`.
- `fn kill_task(&self, task_id: &str) -> Result<()>` — kills the child process
  and sets state to `Killed`.
- `fn cleanup_completed(&self)` — removes tasks in terminal state older than
  30 minutes (called on each `list_tasks` invocation to prevent unbounded growth).
- `fn shutdown_all(&self)` — kills all `Running` tasks; called on stdin EOF.

Process spawning: build the command as `[binary, auto_approve_flag, "-p", prompt]`
(or equivalent per-backend argument structure). For `ollama`, the command is
`[ollama, run, model, prompt]` with no approval flag.

### R5 — Tool manifest and preset system

Define the tool groups and presets in `src/mcp/bridge/tools.rs`.

**Tool groups:**
- `chat` — `prompt` tool (synchronous, single round-trip to a backend)
- `agent` — `run`, `wait`, `list_tasks`, `get_result`, `kill_task` (async process
  registry tools)
- `research` — `research` tool (prompt + optional file paths for context)
- `analysis` — `analyze_code` tool (prompt + analysis type: review/explain/
  optimize/security/test)
- `subagent` — `clink` tool (spawn isolated CLI subagent with custom system prompt
  and model override)

**Presets** (cumulative — each preset includes all groups of lower presets):
| Preset | Tool groups included |
|--------|---------------------|
| `minimal` | `chat` |
| `agent` | `chat`, `agent` |
| `research` | `chat`, `agent`, `research`, `analysis` |
| `full` | all groups |

**Tool input schemas** (MCP JSON Schema format for each tool):

`prompt`:
```json
{
  "backend": {"type": "string", "description": "Backend name (gemini|codex|claude|grok|ollama). Omit to use default."},
  "prompt": {"type": "string", "description": "The prompt text."},
  "model": {"type": "string", "description": "Optional model override."}
}
```
Required: `["prompt"]`

`run`:
```json
{
  "backend": {"type": "string"},
  "prompt": {"type": "string", "description": "Prompt to send asynchronously."},
  "timeout_secs": {"type": "integer", "description": "Override per-task timeout."}
}
```
Required: `["prompt"]`. Returns `{"task_id": "..."}`.

`wait`:
```json
{
  "task_ids": {"type": "array", "items": {"type": "string"}, "description": "Task IDs to wait for."},
  "timeout_secs": {"type": "integer"}
}
```
Required: `["task_ids"]`. Blocks until all tasks reach terminal state. Returns
array of `{task_id, exit_code, stdout, stderr, duration_ms}`.

`list_tasks`: No required inputs. Returns array of
`{task_id, backend, status, prompt_preview, started_at}`.

`get_result`:
```json
{"task_id": {"type": "string"}}
```
Required: `["task_id"]`. Returns `{task_id, state, stdout, stderr, exit_code}`.

`kill_task`:
```json
{"task_id": {"type": "string"}}
```
Required: `["task_id"]`.

`research`:
```json
{
  "query": {"type": "string"},
  "backend": {"type": "string"},
  "files": {"type": "array", "items": {"type": "string"}, "description": "Absolute file paths for context."},
  "model": {"type": "string"}
}
```
Required: `["query"]`. For file context: reads each file and prepends its content
to the prompt with a `--- FILE: {path} ---` separator.

`analyze_code`:
```json
{
  "code_or_path": {"type": "string", "description": "Code snippet or absolute file path."},
  "analysis_type": {"type": "string", "enum": ["review", "explain", "optimize", "security", "test"]},
  "backend": {"type": "string"},
  "model": {"type": "string"}
}
```
Required: `["code_or_path", "analysis_type"]`. If `code_or_path` is an existing
file path, read its contents; otherwise treat as inline code.

`clink`:
```json
{
  "system_prompt": {"type": "string", "description": "Custom system prompt for the subagent."},
  "prompt": {"type": "string", "description": "Task prompt for the subagent."},
  "backend": {"type": "string"},
  "model": {"type": "string"},
  "session_id": {"type": "string", "description": "Resume an existing session (backend must support it)."}
}
```
Required: `["system_prompt", "prompt"]`. The system prompt is prepended to the
prompt using a backend-specific mechanism (e.g., `--system-prompt` flag for Claude,
embedded in the prompt text for backends that do not support a separate flag).

### R6 — Tool execution handlers

Implement `src/mcp/bridge/handlers.rs` with one async handler function per tool.

**`handle_prompt()`** — synchronous execution path:
1. Select backend (from input `backend` field, or `default_backend`, or first
   available).
2. Build command: `[binary, auto_approve_flag, "-p", prompt]`.
3. Spawn with `tokio::process::Command`, `stdout(Stdio::piped())`,
   `stderr(Stdio::piped())`.
4. `tokio::time::timeout(timeout, child.wait_with_output())`.
5. On success: return `{"content": [{"type": "text", "text": stdout_string}]}`.
6. On timeout: return MCP error `{"isError": true, "content": [{"type": "text",
   "text": "timeout after Ns"}]}`.
7. Non-zero exit: return `{"isError": true, ...}` with combined stdout+stderr.

**`handle_run()`** — delegates to `TaskRegistry::spawn_task()`. Returns
`{"content": [{"type": "text", "text": "{\"task_id\": \"...\"}"}]}`.

**`handle_wait()`** — polls `TaskRegistry::get_task()` for each ID in a loop with
100ms sleep intervals until all reach terminal state or timeout expires.

**`handle_list_tasks()`** — calls `TaskRegistry::cleanup_completed()` then lists
all remaining tasks as JSON array.

**`handle_get_result()`**, **`handle_kill_task()`** — thin wrappers around
`TaskRegistry` methods.

**`handle_research()`** — reads file contents (up to 100 KiB per file, truncating
with a notice if larger), builds a composite prompt, then calls `handle_prompt()`.

**`handle_analyze_code()`** — resolves `code_or_path` to code text (file read if
path exists), prepends an analysis-type-specific instruction prefix:
- `review`: "Review this code for correctness, design, and maintainability:\n\n"
- `explain`: "Explain what this code does, step by step:\n\n"
- `optimize`: "Suggest performance and readability improvements for this code:\n\n"
- `security`: "Audit this code for security vulnerabilities:\n\n"
- `test`: "Write comprehensive tests for this code:\n\n"

**`handle_clink()`** — spawns via `TaskRegistry::spawn_task()` with a modified
command that injects the system prompt. For backends supporting `--system-prompt`:
`[binary, auto_approve_flag, "--system-prompt", system_prompt, "-p", prompt]`.
For other backends: prepend `SYSTEM: {system_prompt}\n\nTASK: ` to the prompt and
use the standard command form.

### R7 — Integration with `great apply` and `great doctor`

**`great apply` integration** (`src/cli/apply.rs`):

When `great.toml` contains an `[mcp-bridge]` section (or when `great.toml`
declares agents using backends that great supports), `great apply` should write
a bridge entry into `.mcp.json`:

```json
{
  "mcpServers": {
    "great-bridge": {
      "command": "great",
      "args": ["mcp-bridge"]
    }
  }
}
```

If `[mcp-bridge]` declares a preset or specific backends, append those as args:
`["mcp-bridge", "--preset", "agent", "--backends", "gemini,claude"]`.

The entry is written using the existing `McpJsonConfig::add_server()` method and
`McpJsonConfig::save()`. Do not overwrite existing `.mcp.json` entries for servers
not managed by great (use `has_server("great-bridge")` to skip if already present).

**`great doctor` integration** (`src/cli/doctor.rs`):

Add a new check section "MCP Bridge" to `run()` after the existing checks. For
each backend in `[gemini, codex, claude, grok, ollama]`:
- Check if the binary is on PATH using `command_exists()` (already used in
  `doctor.rs` line 7).
- Report as `success` (binary found + API key env set), `warning` (binary found
  but API key env absent), or `info` (binary not found — not required).

Also check whether `.mcp.json` contains a `great-bridge` entry and report as
`success` ("bridge registered in .mcp.json") or `warning` ("run `great apply`
to register the bridge").

---

## Acceptance Criteria

- [ ] `great mcp-bridge` starts without error when at least one backend binary is
      available. The process accepts JSON-RPC `initialize` + `tools/list` on stdin
      and responds with valid MCP protocol messages on stdout. Verified by piping:
      `printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n' | great mcp-bridge`
      and asserting the output contains `"protocolVersion"` and `"tools"`.

- [ ] `great mcp-bridge --preset minimal` exposes only the `prompt` tool in
      `tools/list`. `great mcp-bridge --preset full` exposes all 8 tools
      (`prompt`, `run`, `wait`, `list_tasks`, `get_result`, `kill_task`,
      `research`, `analyze_code`, `clink`). Verified by parsing the `tools/list`
      response and asserting `tools.length`.

- [ ] `great apply` writes a `great-bridge` entry to `.mcp.json` when `great.toml`
      contains `[mcp-bridge]`. Running `great apply` twice does not duplicate the
      entry (idempotent). Verified by asserting `jq '.mcpServers["great-bridge"]'`
      is non-null after apply and that `jq '.mcpServers | keys | length'` does not
      increase on second apply.

- [ ] `great doctor` reports discovered backends: at least one backend present on
      the test machine is shown as `success` or `warning` in the "MCP Bridge"
      section. A backend not on PATH is shown as `info`, not as a failure.

- [ ] `cargo test` passes with zero failures and `cargo clippy -- -D warnings`
      produces zero new warnings introduced by this task.

---

## Files That Need to Change

| File | Change |
|------|--------|
| `src/cli/mcp_bridge.rs` | New: `Args` struct, `pub fn run(args: Args) -> Result<()>`; reads config, discovers backends, starts MCP server loop |
| `src/cli/mod.rs` | Add `pub mod mcp_bridge;` (line 7 area) and `MpcBridge(mcp_bridge::Args)` to `Command` enum |
| `src/main.rs` | Add `Command::MpcBridge(args) => cli::mcp_bridge::run(args)` to dispatch match |
| `src/mcp/bridge/mod.rs` | New: module root re-exporting `server`, `protocol`, `backends`, `registry`, `tools`, `handlers` |
| `src/mcp/bridge/protocol.rs` | New: JSON-RPC 2.0 serde structs: `Request`, `Response`, `ErrorObject`, `Notification`; MCP `InitializeResult`, `Tool`, `ToolCall`, `ToolResult` |
| `src/mcp/bridge/backends.rs` | New: `BackendConfig` struct, per-backend defaults table, `discover_backends()` |
| `src/mcp/bridge/registry.rs` | New: `TaskState`, `TaskHandle`, `TaskRegistry` with spawn/get/kill/list/shutdown |
| `src/mcp/bridge/tools.rs` | New: tool group definitions, preset filter, `ToolManifest::for_preset()` returning `Vec<Tool>` |
| `src/mcp/bridge/handlers.rs` | New: one async handler fn per tool; calls registry and backends |
| `src/mcp/bridge/server.rs` | New: async stdin/stdout loop; dispatches JSON-RPC to handlers |
| `src/mcp/mod.rs` | Add `pub mod bridge;` |
| `src/config/schema.rs` | Add `MpcBridgeConfig` struct; add `mcp_bridge: Option<MpcBridgeConfig>` to `GreatConfig` |
| `src/cli/apply.rs` | Add bridge registration block: if `mcp_bridge` config present, write `great-bridge` entry to `.mcp.json` |
| `src/cli/doctor.rs` | Add "MCP Bridge" check section after existing checks |

---

## Dependencies

### Cargo dependencies (new)

```toml
rmcp = { version = "0.16", features = ["server", "transport-io"] }
process-wrap = { version = "9.0", features = ["tokio1", "process-group", "kill-on-drop"] }
uuid = { version = "1.21", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**`rmcp`** (0.16.0, 4.2M downloads, ~3K GitHub stars) — the official
Anthropic-endorsed Rust MCP SDK, hosted under `modelcontextprotocol` GitHub org.
This **replaces the need for hand-rolled JSON-RPC 2.0 protocol structs** entirely.
The SDK handles: `initialize`/`initialized` lifecycle, `tools/list`, `tools/call`
dispatch, request/response serialization, error handling, and notification routing.
Stdio transport is built-in via `TokioChildProcess`. The companion `rmcp-macros`
crate provides `#[tool]` proc macros with automatic JSON Schema generation via
`schemars`. Consequence: **R2 (MCP stdio server loop) and `protocol.rs` are
dramatically simplified** — no raw JSON-RPC parsing, no manual message framing.

**`process-wrap`** (9.0.0, ~1.5M downloads) — successor to `command-group` by the
watchexec maintainer (Félix Saparelli). Composable wrappers for process management:
```rust
TokioCommandWrap::from(cmd)
    .wrap(ProcessGroup::leader())  // new process group (setpgid/killpg)
    .wrap(KillOnDrop)              // cleanup on Drop
    .spawn()?;
```
Solves the critical problem of killing entire process trees (not just the immediate
child) on timeout or server shutdown. Unix: `setpgid`/`killpg`. Windows: Job Objects.
**Replaces manual SIGTERM/SIGKILL logic in R4.**

**`uuid`** (1.21.0, 454M downloads) — quasi-official Rust infrastructure, maintained
by rust-lang-nursery. `Uuid::new_v4()` for task IDs. Only adds `getrandom` as a
transitive dependency.

**`tracing`** + **`tracing-subscriber`** (262M downloads) — tokio team's structured
logging. Async span propagation across `.await` boundaries is essential for debugging
concurrent subprocess tasks. Stderr-only config:
```rust
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .with_env_filter("info")
    .init();
```

### Cargo dependencies (already present)

- `tokio = { version = "1.0", features = ["full"] }` — `Cargo.toml` line 16.
  `tokio::process::Command` for subprocess spawning, `tokio::io` for async stdio,
  `tokio::sync::mpsc`/`watch`/`oneshot` for internal channels,
  `tokio::time::timeout` for per-task timeouts, `tokio::signal::ctrl_c` for
  graceful shutdown, `tokio_util::sync::CancellationToken` for shutdown propagation.
- `serde = { version = "1", features = ["derive"] }` + `serde_json = "1.0"` —
  already in `Cargo.toml`; used by `rmcp` and tool schemas.
- `which = "7"` — `Cargo.toml` line 25; used in `discover_backends()`.
- `clap = { version = "4", features = ["derive"] }` — already present; the new
  `Args` struct uses derive macros.

### Crates explicitly NOT needed

- **No separate JSON-RPC crate** — `rmcp` absorbs the entire JSON-RPC 2.0 layer
  (message framing, request/response types, error codes). The `protocol.rs` file
  from R2 becomes a thin re-export or is eliminated entirely.
- **No `duct`** — sync-only pipeline crate; wrong runtime model for tokio.
- **No `async-process`** — smol runtime, not tokio-compatible.
- **No `crossbeam-channel`** — sync-only; would block the tokio runtime.
- **No `nanoid`** — unmaintained for 5 years, pinned to `rand 0.8`.

### Task dependencies

- Task 0009 (done) — `great apply` infrastructure is in place; this task extends
  it with one new write block.
- Task 0010 Group C (done) — `great mcp add` uses `toml_edit` for format-preserving
  writes; this task does not need `toml_edit` (it writes `.mcp.json`, not
  `great.toml`).
- MCP specification — `rmcp` 0.16 tracks MCP spec version `2025-11-05`. Builder
  should verify the `protocolVersion` string at implementation time against the
  locally installed `rmcp` version's constants.

---

## Notes

### Why a flat subcommand (`mcp-bridge`) rather than `mcp bridge`

The existing `great mcp` subcommand group (in `src/cli/mcp.rs`) uses nested
subcommands for server management operations (list, add, test). The bridge is a
long-running daemon process, not a management command. A flat top-level subcommand
avoids clutter in the `great mcp --help` output and makes the entry point for
`.mcp.json` registration unambiguous: `{"command": "great", "args": ["mcp-bridge"]}`.

### Backend command argument conventions

Each AI CLI has its own flag conventions. The `auto_approve_flag` in `BackendConfig`
covers the most important difference (interactive approval bypass). The `-p` flag
for prompt input is common to Claude CLI and Codex CLI; Gemini CLI and others may
use positional arguments or `--prompt`. The builder should verify each CLI's
current interface before implementing `handle_prompt()`. Use `cargo doc` or `man`
for locally installed CLIs, not assumed knowledge.

### MCP token limit and response truncation

MCP clients enforce a ~25K token limit on tool responses. For backends that
produce long outputs, `handle_prompt()` should truncate stdout to 80,000 characters
(approximately 20K tokens at 4 chars/token) and append a truncation notice:
`\n\n[output truncated at 80,000 chars — use \`run\`/\`wait\` for full async output]`.

### Process registry memory bounds

`TaskRegistry::cleanup_completed()` removes terminal-state tasks older than 30
minutes. This is called on every `list_tasks` invocation. For long-running bridge
instances, this prevents unbounded HashMap growth. The 30-minute window is chosen
to allow `wait` calls that arrive after a long `run` completes.

### Self-registration idempotency in `great apply`

`McpJsonConfig::has_server("great-bridge")` (line 64 of `src/mcp/mod.rs`) is used
to skip writing if the entry already exists. This is sufficient for idempotency
because the bridge entry has a stable key and stable args determined by the config.
If the user changes `[mcp-bridge]` options in `great.toml` (e.g., changes preset),
`great apply` must overwrite the existing entry rather than skip it. The check
should therefore compare the existing entry's args against the desired args, and
overwrite if different.

### Ollama model selection

Ollama requires a model name as part of the command (`ollama run <model> <prompt>`).
`BackendConfig.model` defaults to `"llama3.2"` for Ollama when not specified in
`great.toml` or CLI `--model` flag. This default should be configurable via
`GREAT_OLLAMA_MODEL` env var.

### rmcp eliminates hand-rolled protocol code

With `rmcp`, **R2 (MCP stdio server loop) is dramatically simplified.** Instead of
manually parsing JSON-RPC messages from stdin, the bridge uses:

```rust
use rmcp::{ServiceExt, transport::stdio};

let service = GreatBridge::new(registry, backends);
let server = service.serve(stdio()).await?;
server.waiting().await?;
```

Tools are registered via `rmcp-macros` proc macros:
```rust
#[tool(description = "Send a prompt to an AI backend")]
async fn prompt(&self, backend: Option<String>, prompt: String) -> Result<String> { ... }
```

This auto-generates JSON Schema from the function signature (via `schemars`),
handles MCP framing, and dispatches `tools/call` to the correct handler. The files
`src/mcp/bridge/protocol.rs` and `src/mcp/bridge/server.rs` from the original R2
collapse into a single idiomatic rmcp server setup.

### Process group management via process-wrap

Instead of manual SIGTERM→sleep→SIGKILL logic, `process-wrap` handles process tree
cleanup:
```rust
let mut child = TokioCommandWrap::from(Command::new(&backend.binary))
    .wrap(ProcessGroup::leader())  // child gets its own process group
    .wrap(KillOnDrop)              // entire group killed when handle drops
    .spawn()?;
```

On timeout, simply dropping the child handle kills the entire process tree. This
replaces the manual cleanup code in R4's `kill_task()` and `shutdown_all()`.

### Graceful shutdown pattern

Use `tokio_util::sync::CancellationToken` (built into tokio, zero extra deps):
```rust
let token = CancellationToken::new();
tokio::select! {
    _ = server.waiting() => {},           // MCP client disconnected
    _ = tokio::signal::ctrl_c() => {},    // SIGINT
    _ = token.cancelled() => {},          // programmatic shutdown
}
registry.shutdown_all().await;
```

### No HTTP clients needed

All 5 backends are spawned as CLI subprocesses — no `reqwest` calls. This keeps the
binary size impact minimal and avoids TLS/certificate complexity. The existing
`reqwest` dependency (for `great sync`) is not touched.
