# > great.sh

**One MCP server, five AI backends.** Alpha — open to testing and feedback.

great.sh is a single Rust binary that puts every AI CLI on your machine behind
one MCP server — Gemini, Codex, Claude, Grok, and local Ollama — and can
provision the rest of your AI dev environment while it's at it. Open source,
Apache-2.0, built by [Superstruct](https://superstruct.nz).

## Quick start — the MCP bridge in 30 seconds

```sh
curl -sS https://great.sh/install.sh | sh          # install the great binary
claude mcp add --scope user great-bridge -- great mcp-bridge
claude
```

That's it. Claude Code now has bridge tools: ask it to *"use great-bridge to
ask gemini for a second opinion on this diff"* or *"run this refactor question
past the local ollama model"*. The bridge auto-detects whichever backend CLIs
are installed (`gemini`, `codex`, `claude`, `grok`, `ollama`) — no Node.js, no
extra config.

Drop `--scope user` to register it for the current project only.

## The MCP bridge

`great mcp-bridge` is a stdio JSON-RPC 2.0 MCP server that multiplexes AI CLI
backends behind one tool surface:

```
                                        ┌──> gemini CLI   (GEMINI_API_KEY)
 Claude Code ──­ MCP (stdio) ──> great ──┼──> codex CLI    (OPENAI_API_KEY)
 (or any MCP client)          mcp-bridge├──> claude CLI   (login)
                                        ├──> grok CLI     (XAI_API_KEY)
                                        └──> ollama       (local, no key)
```

- **Sync and async** — `prompt` for a single round-trip; `run`/`wait`/
  `get_result`/`kill_task` for long-running background tasks with timeouts
  and auto-cleanup
- **Higher-level tools** — `research` (query + file context) and
  `analyze_code` (review / explain / optimize / security / test), plus
  `clink` for spawning an isolated subagent with a custom system prompt
- **Presets** — `minimal` (1 tool), `agent` (7, default), `research` (9),
  `full` (10): expose only what the client needs
- **Session resume** — continue Claude/Codex conversations across calls
- **Guardrails** — optional `allowed-dirs` allowlist for file-reading tools,
  per-task timeouts, opt-out of auto-approval flags

Configure it in `great.toml` (all optional):

```toml
[mcp-bridge]
backends = ["gemini", "ollama"]   # default: auto-detect all installed
default-backend = "gemini"
preset = "agent"                  # minimal | agent | research | full
timeout-secs = 300
allowed-dirs = ["~/src"]          # restrict file-reading tools
```

Full reference — backends, config schema, adding a backend, security notes:
[docs/mcp-bridge.md](docs/mcp-bridge.md).

## Environment provisioning

The same binary takes a machine from blank to a fully configured AI dev
environment:

- **One command setup** — `great init` then `great apply`: tools, MCP servers,
  agents, and secrets from a single `great.toml`
- **MCP server management** — install, configure, and credential-inject MCP
  servers from the registry
- **Secret management** — `great vault` sources API keys from env, 1Password,
  Bitwarden, or the macOS Keychain and injects them where tools expect them

| Command | Description |
|---------|-------------|
| `great init` | Initialize a new great.sh environment |
| `great apply` | Apply configuration to the current environment |
| `great status` | Show environment status |
| `great mcp-bridge` | Run the multi-backend MCP bridge server |
| `great mcp` | Manage MCP servers |
| `great vault` | Manage credentials via local secret providers |
| `great doctor` | Diagnose environment issues |
| `great update` | Update great.sh to the latest version |
| `great diff` | Show configuration diff |
| `great template` | Manage configuration templates |
| `great sync` | Export/import config snapshots (local storage) |
| `great loop` | Install and manage the great.sh Loop plugin |

## The great.sh Loop

An evidence-gated AI agent methodology shipped as a Claude Code plugin: four
functional roles instead of a fixed pipeline, recalibrated for models that
plan and verify their own work.

- **builder** — implements the spec, runs quality gates, answers findings with evidence
- **verifier** — adversarial: tries to prove the change broken or insecure (correctness, regression, security, performance)
- **reviewer** — read-only quality review (structure, simplification, UX, output design, docs)
- **scout** (optional) — read-only recon for large or unfamiliar change surfaces

```
Plan (lead) → Build & Verify (builder + verifier + reviewer) → Finish (lead)
```

Your session is the team lead: it writes the spec, self-reviews it for gaps, spawns the team, and commits. A phase ends when its exit criteria are met — quality gates green plus no CONFIRMED blocking findings — never after a fixed number of rounds. Verifier findings must cite reproductions; the builder responds with rerun evidence, not re-argument.

Roles inherit your session model by default. Pin a tier per role in `~/.claude/teams/loop/config.json` when the work demands it — e.g. Opus for security-audit-heavy verification, since Fable-class cyber safety classifiers can refuse security-probing work mid-audit.

```sh
great loop install            # registers the plugin with Claude Code
great loop install --project  # adds .tasks/ working state to the current repo
claude                        # then: /great:backlog → /great:loop
```

## Templates

Pre-configured environment templates from [architecton.ai](https://architecton.ai):

- **AI Full Stack (TypeScript)** — Claude Code + Codex + Gemini, Playwright MCP server, gh CLI
- **AI Full Stack (Python)** — Python with uv, PostgreSQL MCP server, full AI agent setup
- **AI Data Science** — CUDA, Jupyter, Gemini for data analysis, database MCP servers
- **AI DevOps** — Terraform, AWS CLI, Docker, Kubernetes MCP servers
- **AI Minimal** — Just Claude Code with Filesystem and Memory MCP servers

## Development

```sh
# CLI
cargo build
cargo test
cargo clippy

# Marketing site
pnpm dev
pnpm build:site
```

## Status and roadmap

Alpha (v0.1.0). The MCP bridge, provisioning (`init`/`apply`/`doctor`), vault
providers, and the Loop plugin work today. We welcome bug reports and feedback.

Being candid about scope: `great sync` currently exports/imports configuration
snapshots to local storage. A hosted cloud-sync service is **not planned** —
use the local secret providers (env, 1Password, Bitwarden, Keychain) to share
credentials across machines.

- [GitHub Issues](https://github.com/superstruct/great.sh/issues)
- [Discussions](https://github.com/superstruct/great.sh/discussions)

## License

Apache-2.0 — see [LICENSE](LICENSE)

Built by [Superstruct](https://superstruct.nz). Copyright 2025 Superstruct Ltd
