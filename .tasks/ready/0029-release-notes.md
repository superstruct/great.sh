# Release Notes: Task 0029 — Inbuilt MCP Bridge Server

**Date:** 2026-02-27
**Scope:** `src/cli/mcp_bridge.rs` (new), `src/mcp/bridge/` (new), `src/cli/mod.rs`, `src/main.rs`, `src/config/schema.rs`, `src/cli/apply.rs`, `src/cli/doctor.rs`, `Cargo.toml`

---

## What Changed

`great` gains a new top-level subcommand, `great mcp-bridge`, that starts an
MCP-compliant stdio server (JSON-RPC 2.0) directly inside the `great` binary.
The bridge exposes up to 9 tools across 5 AI CLI backends — `gemini`, `codex`,
`claude`, `grok`, and `ollama` — with no Node.js or npm dependency required.

---

## Why It Matters

great.sh previously relied on external npm packages (`gemini-mcp`,
`ai-cli-mcp`, `pal-mcp-server`) to bridge AI CLI tools into an MCP client such
as Claude Desktop or a custom agent host. Every user had to have Node.js
installed and run a separate npm install step before the bridge was usable.
This contradicted great.sh's promise of a self-contained, single-binary AI dev
environment. `great mcp-bridge` replaces all three Node.js bridges with a
single native implementation that is installed alongside the `great` binary and
requires nothing else.

---

## New subcommand: `great mcp-bridge`

```
great mcp-bridge [OPTIONS]

Options:
  --preset <NAME>        Tool preset: minimal | agent | research | full  [default: agent]
  --backends <LIST>      Comma-separated backends to enable (default: auto-detect installed)
  --timeout <SECS>       Per-task timeout in seconds  [default: 300]
  --log-level <LEVEL>    Stderr verbosity: off | error | warn | info | debug  [default: warn]
```

The server reads JSON-RPC 2.0 messages from stdin and writes responses to
stdout, one message per newline, conforming to MCP protocol version
`2024-11-05`. All log output goes to stderr and is never mixed with the
JSON-RPC stream.

---

## Tool presets

Presets are cumulative: each includes all tools of lower presets.

| Preset     | Tools included                                                       |
|------------|----------------------------------------------------------------------|
| `minimal`  | `prompt`                                                             |
| `agent`    | `prompt`, `run`, `wait`, `list_tasks`, `get_result`, `kill_task`     |
| `research` | above + `research`, `analyze_code`                                   |
| `full`     | all 9 tools, including `clink`                                       |

**Synchronous tools** (`prompt`, `research`, `analyze_code`) execute a backend
process and return when it completes. Output is truncated to 80,000 characters
if the backend produces more than approximately 20,000 tokens.

**Async tools** (`run`, `wait`, `list_tasks`, `get_result`, `kill_task`) use
an in-process task registry backed by `tokio::process`. `run` returns a
`task_id` immediately; `wait` blocks until one or more tasks reach a terminal
state. Completed task records are automatically purged after 30 minutes.

**`clink`** spawns an isolated subagent with a custom system prompt, using
backend-specific flag injection where supported (e.g., `--system-prompt` for
Claude CLI) and inline prompt prepending for backends that do not support a
separate flag.

---

## Backend discovery

Backends are discovered at startup via `which` (already used elsewhere in the
binary). Only backends whose binary is found on PATH — or overridden by a
`GREAT_*_CLI` env var — are enabled.

| Backend  | Default binary | Auto-approve flag                   | API key env       |
|----------|---------------|-------------------------------------|-------------------|
| `gemini` | `gemini`       | `-y`                               | `GEMINI_API_KEY`  |
| `codex`  | `codex`        | `--full-auto`                      | `OPENAI_API_KEY`  |
| `claude` | `claude`       | `--dangerously-skip-permissions`    | (uses login)      |
| `grok`   | `grok`         | `-y`                               | `XAI_API_KEY`     |
| `ollama` | `ollama`       | (none)                              | (none)            |

Binary path overrides: `GREAT_GEMINI_CLI`, `GREAT_CODEX_CLI`,
`GREAT_CLAUDE_CLI`, `GREAT_GROK_CLI`, `GREAT_OLLAMA_CLI`.

For Ollama, the default model is `llama3.2`, overridable via
`GREAT_OLLAMA_MODEL` or the `--model` tool input.

---

## `great.toml` configuration

A new optional `[mcp-bridge]` section configures bridge defaults:

```toml
[mcp-bridge]
backends = ["gemini", "claude"]  # restrict to a subset (default: all discovered)
default_backend = "gemini"       # backend used when a tool call omits the backend field
timeout_secs = 300               # per-task timeout
preset = "agent"                 # tool preset
```

CLI flags override `great.toml` values when both are present.

---

## `great apply` integration

When `great.toml` contains an `[mcp-bridge]` section, `great apply`
automatically writes a `great-bridge` entry to `.mcp.json`:

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

If `[mcp-bridge]` specifies a preset or backend list, those are appended as
args: `["mcp-bridge", "--preset", "agent", "--backends", "gemini,claude"]`.

Registration is idempotent: if the entry already exists with identical args,
`great apply` leaves it unchanged. If `great.toml` options have changed, the
existing entry is overwritten. Entries for other MCP servers in `.mcp.json`
are never touched.

---

## `great doctor` integration

`great doctor` now includes an "MCP Bridge" check section reporting:

- Each of the five backends: `success` (binary found + API key env set),
  `warning` (binary found but API key env absent), or `info` (binary not found
  — not required).
- Whether `.mcp.json` contains a `great-bridge` entry: `success` if present,
  `warning` with "run `great apply` to register the bridge" if absent.

---

## New Cargo dependencies

| Crate                  | Version | Purpose                                                        |
|------------------------|---------|----------------------------------------------------------------|
| `rmcp`                 | 0.16    | Official Rust MCP SDK (`#[tool]` macros, stdio transport, JSON-RPC 2.0 protocol handling) |
| `uuid`                 | 1       | UUID v4 task IDs for the async process registry               |
| `tracing`              | 0.1     | Structured async logging across `.await` boundaries           |
| `tracing-subscriber`   | 0.3     | Stderr-only log output with env-filter level control          |
| `schemars`             | 1.0     | JSON Schema generation from Rust types (used by `rmcp` macros) |

`process-wrap` is used for process group management (clean process tree
teardown on timeout or shutdown), eliminating the need for manual
SIGTERM/SIGKILL sequences.

All other dependencies (`tokio`, `serde_json`, `which`, `clap`) were already
present in `Cargo.toml`.

---

## Migration notes

No breaking changes. No changes to existing `great.toml` keys, subcommands,
or exit codes.

To opt in:

1. Add `[mcp-bridge]` to `great.toml` (minimum: an empty section `[mcp-bridge]`
   is sufficient to trigger auto-registration).
2. Run `great apply` to write the `.mcp.json` entry.
3. Restart your MCP client (e.g., reload Claude Desktop settings).

To verify the server starts correctly, pipe the MCP handshake manually:

```sh
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | great mcp-bridge
```

The response stream must contain `"protocolVersion"` in the first JSON object
and a `"tools"` array in the third.
