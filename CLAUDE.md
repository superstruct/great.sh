# great.sh — The Managed AI Dev Environment

## What this is

Monorepo for great.sh: a Rust CLI (`great`) that configures AI dev environments, plus a React marketing site.

## Tech stack

- **CLI**: Rust 2021, clap 4 (derive), anyhow/thiserror, tokio, reqwest (rustls)
- **Site**: React 18 + TypeScript, Vite 7, Tailwind CSS 3, Motion
- **Infra**: AWS CDK (S3 + CloudFront + Route53 + ACM + OIDC), GitHub Actions

## Commands

### Rust CLI
- `cargo build` — build the CLI
- `cargo run -- <subcommand>` — run a subcommand
- `cargo test` — run integration tests
- `cargo clippy` — lint

### Marketing site
- `pnpm dev` — site dev server
- `pnpm build:site` — typecheck + production build
- `pnpm --filter great-sh preview` — preview production build

## Project structure

```
Cargo.toml                # Rust CLI package
src/                      # Rust source
├── main.rs               # Entry point, CLI dispatch
├── error.rs              # GreatError enum (thiserror)
├── cli/                  # Subcommand modules (init, apply, status, sync, vault, mcp, doctor, update, diff, template)
├── config/               # great.toml loading + schema
├── platform/             # OS/arch/WSL detection + PlatformCapabilities (package managers, WSL2)
├── mcp/                  # MCP server management
├── vault/                # Credential vault
└── sync/                 # Cloud sync
tests/                    # Integration tests (assert_cmd)
site/                     # Marketing site (React + Vite)
├── index.html            # Vite entry point
├── src/                  # React components, data, hooks, styles
├── package.json          # Site dependencies
└── vite.config.ts        # Vite config
infra/cdk/                # AWS CDK stack
.github/workflows/        # CD (site deploy) + CDK (infra deploy)
```

## Conventions

### Rust
- Use `anyhow::Result` for application errors, `thiserror` for library error enums
- No `.unwrap()` in production code — propagate errors with `?`
- Each CLI subcommand: `Args` struct (clap derive) + `pub fn run(args) -> Result<()>`
- Nested subcommands (sync, vault, mcp, template) use inner enums

### Site
- Dark terminal theme: bg `#0a0a0a`, accent green `#22c55e`, red brand `#dc2626`
- Fonts: Space Grotesk (headings), Inter (body), JetBrains Mono (code)
- Section data in `site/src/data/`, components are presentation-only
- Path alias: `@/` → `./site/src/`

## Deployment

- Tag push (`v*`) triggers the release workflow: builds 4 Rust targets, creates GitHub Release, deploys site
- Push to `release` branch deploys the site to S3 via GitHub Actions OIDC
- CDK stack manages infra (S3 + CloudFront + ACM + Route53 + IAM OIDC roles)
- `CLOUDFRONT_DISTRIBUTION_ID` GitHub repo variable used for cache invalidation

## great.sh Loop (`loop/`)

The `loop/` directory contains the great.sh Loop — a 15-role AI agent orchestration methodology installed globally by the great.sh CLI. These instructions are **stack-agnostic** and must work with any language/framework/toolchain.

### Structure
```
loop/
├── agents/          # Agent persona definitions (15 agents)
├── commands/        # Slash commands (/loop, /bugfix, /deploy, /discover)
├── teams-config.json
└── observer-template.md
```

### Key rules for loop files
- **No language-specific commands** — agents detect build system from config files
- **No repo-specific paths** — no hardcoded paths to any project
- **Only reference tools installed by great.sh** — `gh` CLI, standard Unix tools
- **Context7 MCP** for library docs, not language-specific package registries
