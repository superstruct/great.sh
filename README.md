# > great.sh

**The managed AI dev environment.** Alpha — open to testing and feedback.

One command. 15 AI agents. Fully configured. Open source.

## Quick Start

```sh
curl -sS https://great.sh/install.sh | sh
great init
claude
```

## What it does

- **One command setup** — from a blank machine to a fully configured AI dev environment
- **AI agent orchestration** — 15 specialized agents installed into Claude Code with `great loop install`
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

## The great.sh Loop

A 15-role AI agent orchestration methodology. Each role is embodied by a historical figure whose expertise maps to the task.

```
Nightingale → Lovelace → Socrates → Humboldt → Da Vinci →
  Wirth / Turing / Kerckhoffs / Nielsen (parallel) →
  Dijkstra (code review) → Rams (visual) →
  Knuth + Gutenberg (docs) →
Hopper → Deming
```

Requirements → spec → review → scout → build → test/security/perf/UX → code review → visual → docs → commit → observe. One iteration at a time.

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
