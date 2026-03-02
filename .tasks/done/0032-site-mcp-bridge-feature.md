# 0032 — Marketing Site: Surface mcp-bridge as a Feature

**Priority:** P2
**Type:** feature (content/site)
**Module:** site/
**Status:** backlog
**Created:** 2026-02-28

## Context

`great mcp-bridge` shipped in iteration 026: a pure-Rust stdio MCP server that
bridges Claude Code to Gemini, Codex, Claude, Grok, and Ollama with no Node.js
dependency. It is the only MCP bridge of its kind that ships as a single static
binary. As of 2026-02-28 the feature has zero presence on the marketing site:

- `site/src/data/features.ts` — 4 features, none mention mcp-bridge
- `site/src/data/commands.ts` — `initWizardOutput` and `sampleToml` do not
  reference `great mcp-bridge`
- `site/src/data/comparison.ts` — no row for "Built-in MCP bridge / no Node.js"
  even though `mcpm` requires Node.js — a direct competitive gap

The "MCP Server Management" feature card in `features.ts` describes install and
health checks only; it does not mention the bridge capability.

## What mcp-bridge actually does (for copy accuracy)

- Exposes Gemini, Codex, Claude, Grok, Ollama as MCP tools to Claude Code
- Pure Rust; ships inside the `great` binary; zero npm/Node.js required
- Launched via `great mcp-bridge` (stdio JSON-RPC 2.0)
- 4 presets: `minimal`, `agent`, `research`, `full`
- 9 tools across 5 backends
- Config in `great.toml` under `[mcp_bridge]`

## Requirements

1. Add a fifth feature card to `features.ts` specifically for mcp-bridge,
   distinct from the existing "MCP Server Management" card.
2. Add one comparison row to `comparison.ts`:
   `"Built-in multi-AI bridge (no Node.js)"` — great: true, all others: false
   (mcpm currently shows "List only"; this row shows it has no bridge at all).
3. Add a terminal demo snippet to `commands.ts` showing `great mcp-bridge --preset agent`
   startup output (mirrors the style of `loopInstallOutput`).
4. Update the `sampleToml` in `commands.ts` to include a `[mcp_bridge]` stanza
   with `preset = "agent"` so users see it is config-file driven.
5. The existing "MCP Server Management" feature card description must be updated
   to clarify it covers external MCP servers (registry-sourced), while the new
   card covers the built-in bridge — avoid description overlap.

## Acceptance Criteria

- [ ] `site/src/data/features.ts` contains exactly 5 Feature entries; the new
      entry has `icon: "bridge"` (or any icon already in the `iconMap` in
      `Features.tsx`; add a mapping if needed) and title "Built-in AI Bridge"
- [ ] `site/src/data/comparison.ts` contains a row with
      `feature: "Built-in multi-AI bridge (no Node.js)"` where `great: true`
      and all other columns are `false`
- [ ] `site/src/data/commands.ts` exports `mcpBridgeOutput` (a template-literal
      string showing `$ great mcp-bridge --preset agent` startup output with at
      least 3 detected backends listed)
- [ ] `pnpm --filter great-sh build` exits 0 with no TypeScript errors after
      all changes (icon mapping must be wired if a new icon key is introduced)
- [ ] No existing feature card description is left unchanged where it now
      overlaps with the new bridge card — the MCP Server Management card must
      be scoped to "external MCP servers from the registry"

## Dependencies

- No Rust changes required; all changes are in `site/src/data/`
- If a new icon key is introduced in `features.ts`, the `iconMap` in
  `site/src/components/sections/Features.tsx` must be updated in the same PR

## Notes

- Do not add a new page or section component — the existing Features grid and
  Comparison table are sufficient; this task is data-only plus the icon map
- The `mcpBridgeOutput` string does not need to be wired into a component in
  this task; filing it here so Lovelace can spec the component connection as a
  follow-up (0033) if desired
- `mcp-bridge` command uses `great mcp-bridge`, not `great mcp bridge` —
  verify copy reflects the correct invocation (it is a top-level subcommand)
