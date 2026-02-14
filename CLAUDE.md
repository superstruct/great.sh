# great.sh — Marketing Site

## What this is

Marketing/landing site for great.sh, the managed AI dev environment CLI tool (Rust). This repo is the **website only** — the CLI lives at `/home/isaac/src/great-sh` (legacy Rust project being migrated).

## Tech stack

- React 18 + TypeScript, Vite 7 + SWC, Tailwind CSS 3
- Motion (Framer Motion) for scroll animations
- pnpm workspace: root (site) + `infra/cdk` (AWS infrastructure)
- AWS CDK: S3 + CloudFront + Route53 + ACM + OIDC IAM roles
- GitHub Actions: `release` branch triggers deploy

## Commands

- `pnpm dev` — dev server
- `pnpm build` — typecheck + production build (`tsc -b && vite build`)
- `pnpm preview` — preview production build

## Project structure

```
src/
├── components/
│   ├── layout/       # Nav, Footer, Container
│   ├── sections/     # Hero, Features, Config, HowItWorks, Templates, Comparison, OpenSource
│   └── shared/       # AnimatedSection, CodeBlock, TerminalWindow
├── data/             # Static content (features, templates, comparison, commands)
├── hooks/            # usePlatform (OS detection)
├── lib/              # utils (cn helper)
└── styles/           # globals.css (fonts, tailwind, base styles)
infra/cdk/            # AWS CDK stack (S3, CloudFront, Route53, IAM OIDC)
.github/workflows/    # CD (site deploy) + CDK (infra deploy)
```

## Conventions

- Dark terminal theme: bg `#0a0a0a`, accent green `#22c55e`, red brand `#dc2626`
- Fonts: Space Grotesk (headings), Inter (body), JetBrains Mono (code)
- All code blocks use `CodeBlock` or `TerminalWindow` shared components
- Section data lives in `src/data/`, components are presentation-only
- Scroll-reveal animations via `AnimatedSection` wrapper
- Path alias: `@/` → `./src/`

## Deployment

- Push to `release` branch deploys to S3 via GitHub Actions OIDC
- CDK stack uses same AWS account as superstruct.nz (756605216505)
- `hostedZoneId` in CDK bin is a placeholder until DNS is configured

## Future

- React Router for `/docs`, `/configurator`, `/c/:id` routes
- Interactive TOML configurator (drag-and-drop tool/agent/MCP selection)
- Short-URL install+preconfigure flow (`great.sh/c/abc123`)
