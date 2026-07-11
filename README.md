# > great.sh

**The managed AI dev environment.** Alpha — open to testing and feedback.

One command. A full AI agent team. Fully configured. Open source.

## Quick Start

```sh
curl -sS https://great.sh/install.sh | sh
great init
claude
```

## What it does

- **One command setup** — from a blank machine to a fully configured AI dev environment
- **AI agent orchestration** — the great.sh Loop's evidence-gated agent team installed into Claude Code with `great loop install`
- **MCP server management** — install, configure, and credential-inject MCP servers from the registry
- **Cloud-synced credentials** — zero-knowledge encrypted vault syncs API keys across machines

## CLI Commands

| Command | Description |
|---------|-------------|
| `great init` | Initialize a new great.sh environment |
| `great apply` | Apply configuration to the current environment |
| `great status` | Show environment status |
| `great sync` | Sync configuration with the cloud |
| `great vault` | Manage encrypted credentials |
| `great mcp` | Manage MCP servers |
| `great doctor` | Diagnose environment issues |
| `great update` | Update great.sh to the latest version |
| `great diff` | Show configuration diff |
| `great template` | Manage configuration templates |
| `great loop` | Install and manage the great.sh Loop agent team |

## MCP Bridge

The `great mcp-bridge` command runs an MCP server that routes to multiple AI backends (Gemini, Codex, Claude, Grok, Ollama).

Add it to Claude Code for the current project:

```sh
claude mcp add great-bridge -- great mcp-bridge
```

Or add it globally (all projects):

```sh
claude mcp add --scope user great-bridge -- great mcp-bridge
```

## The great.sh Loop

An evidence-gated AI agent methodology shipped as a Claude Code plugin: four functional roles instead of a fixed pipeline, recalibrated for models that plan and verify their own work.

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

## Status

Alpha (v0.1.0). Core features work. We welcome bug reports and feedback.

- [GitHub Issues](https://github.com/superstruct/great.sh/issues)
- [Discussions](https://github.com/superstruct/great.sh/discussions)

## License

Apache-2.0 — see [LICENSE](LICENSE)

Copyright 2025 Superstruct Ltd
