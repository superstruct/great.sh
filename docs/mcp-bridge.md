# The great.sh MCP Bridge

`great mcp-bridge` runs a stdio JSON-RPC 2.0 MCP server that multiplexes
multiple AI CLI backends behind one tool surface. It's a single Rust binary —
no Node.js, no daemon, no configuration required to get started.

```
                                        ┌──> gemini CLI   (GEMINI_API_KEY)
 Claude Code ──­ MCP (stdio) ──> great ──┼──> codex CLI    (OPENAI_API_KEY)
 (or any MCP client)          mcp-bridge├──> claude CLI   (login)
                                        ├──> grok CLI     (XAI_API_KEY)
                                        └──> ollama       (local, no key)
```

## Quickstart

```sh
# 1. Install great.sh
curl -sS https://great.sh/install.sh | sh

# 2. Register the bridge with Claude Code (all projects)
claude mcp add --scope user great-bridge -- great mcp-bridge

# 3. Use it
claude
> Use great-bridge to ask gemini for a second opinion on this diff.
```

The bridge auto-detects every supported backend CLI on your `PATH` at startup.
If none are found it starts in degraded mode and logs a warning to stderr —
install at least one backend CLI to do anything useful.

Any MCP client works, not just Claude Code — the bridge speaks standard MCP
over stdio.

## Backends

| Backend | Binary | Auth | Path override env | Session resume |
|---------|--------|------|-------------------|----------------|
| Gemini CLI | `gemini` | `GEMINI_API_KEY` | `GREAT_GEMINI_CLI` | no |
| Codex CLI | `codex` | `OPENAI_API_KEY` | `GREAT_CODEX_CLI` | yes |
| Claude CLI | `claude` | interactive login | `GREAT_CLAUDE_CLI` | yes |
| Grok CLI | `grok` | `XAI_API_KEY` | `GREAT_GROK_CLI` | no |
| Ollama | `ollama` | none (local) | `GREAT_OLLAMA_CLI` | no |

Notes:

- Discovery checks the path-override env var first, then `PATH` — set
  `GREAT_<BACKEND>_CLI=/path/to/binary` to pin a specific build.
- Ollama's default model is `llama3.2`; override with `GREAT_OLLAMA_MODEL` or
  the per-call `model` parameter.
- `session_id` is honored by Claude (`-r <id>`) and Codex
  (`exec resume <id>`); other backends log a warning and ignore it.
- `great doctor` reports which backends are installed and which API keys are
  set.

## Tools

| Tool | What it does |
|------|--------------|
| `prompt` | Synchronous single round-trip to a backend |
| `run` | Spawn an async task; returns a task ID immediately |
| `wait` | Block until the given task IDs complete (with timeout) |
| `list_tasks` | List known tasks and their states |
| `get_result` | Fetch a task's result (`verbose` adds tool usage + token counts) |
| `kill_task` | Terminate a running task |
| `cleanup_tasks` | Remove terminal-state tasks (TTL-based, or `force` for all) |
| `research` | Query with file context — file paths are inlined into the prompt |
| `analyze_code` | Code or file path + analysis type: `review`, `explain`, `optimize`, `security`, `test` |
| `clink` | Spawn an isolated subagent with a custom system prompt |

Every tool that talks to a backend accepts an optional `backend` parameter
(`gemini`, `codex`, `claude`, `grok`, `ollama`) and an optional `model`
override. Omit `backend` to use the configured default, falling back to the
first discovered backend.

### Presets

Presets control which tools are exposed via `tools/list` — give a client only
what it needs:

| Preset | Tools |
|--------|-------|
| `minimal` | `prompt` (1) |
| `agent` *(default)* | + `run`, `wait`, `list_tasks`, `get_result`, `kill_task`, `cleanup_tasks` (7) |
| `research` | + `research`, `analyze_code` (9) |
| `full` | + `clink` (10) |

## Configuration

CLI flags win over `great.toml`; everything is optional:

```toml
[mcp-bridge]
# Restrict to a subset of backends (default: auto-detect all installed)
backends = ["gemini", "ollama"]

# Backend used when a tool call omits `backend`
# (default: first discovered backend)
default-backend = "gemini"

# Tool preset: minimal | agent | research | full (default: agent)
preset = "agent"

# Per-task timeout in seconds (default: 300)
timeout-secs = 300

# Pass backend auto-approval flags, e.g. claude's
# --dangerously-skip-permissions (default: true — see Security below)
auto-approve = true

# Restrict file-reading tools (research, analyze_code) to these directories
# (default: unrestricted)
allowed-dirs = ["~/src"]

# Keep completed/failed tasks this long before auto-cleanup (default: 1800)
cleanup-ttl-secs = 1800
```

Equivalent flags: `--backends`, `--preset`, `--timeout`, `--allowed-dirs`,
`--log-level` (stderr verbosity: `off`–`trace`; logs never mix with the stdio
protocol stream).

## Security notes

- **Credential sourcing** — the bridge never stores, reads, or logs API keys.
  Each backend CLI authenticates itself from its own environment
  (`GEMINI_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`) or login state (Claude);
  Ollama is local and needs no key. To manage those keys, use the existing
  vault providers — `great vault` sources secrets from env, 1Password,
  Bitwarden, or the macOS Keychain, and `great apply` validates that required
  secrets are present.
- **Auto-approval** — by default the bridge passes each backend's
  non-interactive flag (e.g. `claude --dangerously-skip-permissions`,
  `codex --full-auto`) so backends don't hang waiting for a TTY. That means a
  backend invoked through the bridge can act without per-action confirmation.
  Set `auto-approve = false` to suppress those flags — backends without a TTY
  will then typically error or auto-decline rather than prompt.
- **File access** — `research` and `analyze_code` read files named in tool
  calls. Set `allowed-dirs` to canonicalized allowlist directories to stop a
  client (or a prompt-injected model) from reading arbitrary paths.
- **Prompt handling** — prompts are passed as single argv values (with `--`
  or space-prefix guards so content can't be parsed as flags), or via stdin
  for very large prompts on backends that support it. Nothing is ever passed
  through a shell.

## Adding a backend

Backends are declared statically — adding one is four small changes in the
Rust source:

1. **`src/mcp/bridge/backends.rs`** — add a `BackendSpec` entry to
   `BACKEND_SPECS`: name, display name, default binary, path-override env var,
   auto-approve flag (if any), API-key env (if any), default model (if any),
   and the CLI flags that switch it to structured output. If the CLI's
   invocation shape differs from the standard `[flags] -p <prompt>` pattern,
   extend `build_command_args` (see the `ollama` and `codex` branches).
2. **`src/mcp/bridge/parsers.rs`** — add a `parse_<name>_output` function and
   a dispatch arm in `parse_output` if the backend emits structured output
   (JSON/JSONL). Backends without structured output fall through to
   `parse_raw_output` automatically.
3. **`src/config/schema.rs`** — add the name to the `known_backends` list in
   `validate()` so config validation accepts it.
4. **Tests** — add `build_command_args` cases in `backends.rs` and parser
   fixtures in `parsers.rs`, mirroring the existing per-backend tests.

`great doctor` picks the new backend up automatically via
`all_backend_specs()`.

## Troubleshooting

- **"No AI CLI backends found"** — nothing on `PATH`. Install one of the
  backend CLIs, or set `GREAT_<BACKEND>_CLI` to its absolute path.
- **Tool calls time out** — raise `timeout-secs` (or per-call
  `timeout_secs` on `run`); backend CLIs cold-starting large local models
  (Ollama) can exceed the 300 s default.
- **Backend errors immediately with `auto-approve = false`** — expected for
  CLIs that require a TTY to prompt; re-enable auto-approve or run that
  backend interactively outside the bridge.
- **Debugging** — `great mcp-bridge --log-level debug` logs discovery,
  spawned commands, and parse results to stderr without disturbing the MCP
  stream.
