# 0032: Marketing Site -- Surface MCP Bridge as a Feature

**Task:** `.tasks/backlog/0032-site-mcp-bridge-feature.md`
**Author:** Ada Lovelace (Spec Writer)
**Date:** 2026-02-28
**Status:** ready
**Complexity:** S (data-file additions + one icon-map update; no new components)

---

## Summary

The `great mcp-bridge` subcommand shipped in iteration 026 -- a pure-Rust MCP
bridge server (stdio JSON-RPC 2.0) that connects Claude Code to Gemini, Codex,
Claude, Grok, and Ollama with zero Node.js dependency. It ships 9 tools across
5 backends with 4 presets (minimal, agent, research, full), path traversal
prevention, and configurable auto-approve.

The marketing site currently has zero mention of this feature. This spec adds:

1. A fifth feature card in `features.ts` titled "Built-in AI Bridge"
2. A new comparison row in `comparison.ts` for the bridge capability
3. A terminal demo string (`mcpBridgeOutput`) in `commands.ts`
4. A `[mcp_bridge]` stanza appended to `sampleToml` in `commands.ts`
5. Updated copy for the existing "MCP Server Management" card to scope it to
   external/registry-sourced MCP servers
6. A new icon mapping (`Unplug`) in `Features.tsx` for the `bridge` icon key

No new components or pages are created. No Rust changes.

---

## Files to Modify

| File | Nature of Change |
|------|-----------------|
| `site/src/data/features.ts` | Add 5th feature entry; update 3rd entry description |
| `site/src/data/comparison.ts` | Add 1 new `ComparisonRow` |
| `site/src/data/commands.ts` | Add `mcpBridgeOutput` export; append `[mcp_bridge]` to `sampleToml` |
| `site/src/components/sections/Features.tsx` | Add `Unplug` to lucide import and `iconMap` |

No new files are created.

---

## Change 1: `site/src/data/features.ts`

### 1A. Update existing "MCP Server Management" card (index 2)

Replace the current description (line 23-24):

```typescript
// BEFORE:
{
  title: 'MCP Server Management',
  description:
    'Install, configure, and credential-inject MCP servers from the official registry. Health checks, cross-client config sync, curated bundles.',
  icon: 'server',
},

// AFTER:
{
  title: 'MCP Server Management',
  description:
    'Install, configure, and credential-inject external MCP servers from the official registry. Health checks, cross-client config sync, and curated bundles for third-party tools.',
  icon: 'server',
},
```

Key changes: Added "external" before "MCP servers" and "third-party tools" at the
end. This scopes the card to registry-sourced servers and avoids overlap with the
new bridge card.

### 1B. Add fifth feature entry after the "Cloud-Synced Credentials" card

Insert a new entry at array index 4 (after `shield`):

```typescript
{
  title: 'Built-in AI Bridge',
  description:
    'Pure-Rust MCP bridge server connecting Claude Code to Gemini, Codex, Claude, Grok, and Ollama. Ships inside the great binary \u2014 zero Node.js required. 9 tools, 5 backends, 4 presets.',
  icon: 'bridge',
},
```

### Full `features.ts` after changes

```typescript
export interface Feature {
  title: string
  description: string
  icon: string
}

export const features: Feature[] = [
  {
    title: 'One Command Setup',
    description:
      'From a blank machine to a fully configured AI dev environment. Install tools, runtimes, and shell config in a single command.',
    icon: 'terminal',
  },
  {
    title: 'AI Agent Orchestration',
    description:
      'The great.sh Loop: 16 specialized AI roles installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, performance checks, code reviews, UX inspections, visual reviews, docs, and deploys \u2014 orchestrated as a team. Run great loop install to set it up.',
    icon: 'brain',
  },
  {
    title: 'MCP Server Management',
    description:
      'Install, configure, and credential-inject external MCP servers from the official registry. Health checks, cross-client config sync, and curated bundles for third-party tools.',
    icon: 'server',
  },
  {
    title: 'Cloud-Synced Credentials',
    description:
      'Zero-knowledge encrypted vault syncs API keys and config across machines. BYO credentials \u2014 we never see your keys.',
    icon: 'shield',
  },
  {
    title: 'Built-in AI Bridge',
    description:
      'Pure-Rust MCP bridge server connecting Claude Code to Gemini, Codex, Claude, Grok, and Ollama. Ships inside the great binary \u2014 zero Node.js required. 9 tools, 5 backends, 4 presets.',
    icon: 'bridge',
  },
]
```

---

## Change 2: `site/src/data/comparison.ts`

### Add one new `ComparisonRow`

Insert after the existing "MCP server management" row (index 4, line 48-56) and
before the "Credential management" row. This placement groups the two MCP-related
rows together:

```typescript
{
  feature: 'Built-in multi-AI bridge (no Node.js)',
  great: true,
  chezmoi: false,
  mise: false,
  nix: false,
  mcpm: false,
  manual: false,
},
```

Rationale for column values:
- `great: true` -- `great mcp-bridge` provides this.
- `chezmoi: false` -- dotfiles manager, no MCP bridge capability.
- `mise: false` -- runtime version manager, no MCP bridge.
- `nix: false` -- package manager, no MCP bridge.
- `mcpm: false` -- MCP Package Manager lists and installs external MCP servers
  but has no built-in multi-AI bridge server. It also requires Node.js.
- `manual: false` -- no known manual single-binary multi-backend MCP bridge exists.

### Full `comparison.ts` after changes

```typescript
export interface ComparisonRow {
  feature: string
  great: boolean | string
  chezmoi: boolean | string
  mise: boolean | string
  nix: boolean | string
  mcpm: boolean | string
  manual: boolean | string
}

export const comparisonData: ComparisonRow[] = [
  {
    feature: 'One-command full setup',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Declarative config file',
    great: 'great.toml',
    chezmoi: 'chezmoi.toml',
    mise: 'mise.toml',
    nix: 'flake.nix',
    mcpm: false,
    manual: false,
  },
  {
    feature: 'AI CLI tool installation',
    great: true,
    chezmoi: false,
    mise: false,
    nix: 'Partial',
    mcpm: false,
    manual: true,
  },
  {
    feature: 'AI agent orchestration loop',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'MCP server management',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: 'List only',
    manual: true,
  },
  {
    feature: 'Built-in multi-AI bridge (no Node.js)',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Credential management',
    great: true,
    chezmoi: 'Partial',
    mise: false,
    nix: false,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Cross-machine sync',
    great: true,
    chezmoi: 'Git-based',
    mise: false,
    nix: 'Git-based',
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Runtime version management',
    great: 'Via mise',
    chezmoi: false,
    mise: true,
    nix: true,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Dotfiles management',
    great: true,
    chezmoi: true,
    mise: false,
    nix: true,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Learning curve',
    great: 'Minutes',
    chezmoi: 'Hours',
    mise: 'Hours',
    nix: 'Weeks',
    mcpm: 'Minutes',
    manual: 'Days',
  },
]
```

---

## Change 3: `site/src/data/commands.ts`

### 3A. Add `mcpBridgeOutput` export

Add after the existing `loopInstallOutput` export (after line 92):

```typescript
export const mcpBridgeOutput = `$ great mcp-bridge --preset agent

  great.sh MCP Bridge -- Starting (preset: agent)

  Discovering backends...
  [check] Gemini CLI    gemini (GEMINI_API_KEY set)
  [check] Codex CLI     codex  (OPENAI_API_KEY set)
  [check] Claude CLI    claude (logged in)

  Preset: agent (6 tools)
  Tools: prompt, run, wait, list_tasks, get_result, kill_task

  Listening on stdio (JSON-RPC 2.0)
  Server: great-mcp-bridge v0.3.0`
```

This output mirrors the style of `loopInstallOutput`:
- Starts with `$ great mcp-bridge --preset agent` (the invocation)
- Uses `[check]` markers consistent with existing terminal demos
- Shows 3 of 5 backends (the most common ones a user would have)
- Displays the preset and tool count
- Ends with the server identification line

Note: This is a representative terminal output for marketing purposes. The
actual bridge communicates via JSON-RPC on stdout and logs to stderr via
tracing. The demo shows a user-friendly summary of what happens at startup.

### 3B. Append `[mcp_bridge]` stanza to `sampleToml`

Add at the end of the `sampleToml` template literal, before the closing backtick
(after line 77, before the closing backtick on the same line):

```
\n
[mcp_bridge]
preset = "agent"
default_backend = "gemini"
```

### Full `sampleToml` after changes

The complete value including the new stanza:

```typescript
export const sampleToml = `# great.toml \u2014 AI Dev Environment Specification

[project]
name = "my-saas-app"
template = "ai-fullstack-ts"

[tools]
node = "22"
python = "3.12"

[tools.cli]
packages = [
  "gh", "docker", "ripgrep", "fzf",
  "starship", "zoxide", "bat", "eza",
  "lazygit", "atuin", "zellij",
]

[agents.claude]
role = "orchestrator"

[agents.codex]
role = "mcp-server"
transport = "stdio"

[agents.gemini]
role = "mcp-server"
transport = "stdio"

[mcp.filesystem]
source = "registry:modelcontextprotocol/server-filesystem"
transport = "stdio"

[mcp.memory]
source = "registry:modelcontextprotocol/server-memory"
transport = "stdio"

[mcp_bridge]
preset = "agent"
default_backend = "gemini"

[secrets]
provider = "great-vault"
required = [
  "ANTHROPIC_API_KEY",
  "OPENAI_API_KEY",
  "GITHUB_TOKEN",
]`
```

Note: The `[mcp_bridge]` stanza is placed between `[mcp.memory]` and `[secrets]`.
This groups all MCP-related config together before the secrets section. The
`[secrets]` section remains last as it is the natural end of the config file.

---

## Change 4: `site/src/components/sections/Features.tsx`

### Add `Unplug` icon to the import and `iconMap`

The `bridge` icon key needs a mapping. The lucide-react library includes `Unplug`
which visually represents a connection/bridge between systems (two connectors
linking together). This is distinct from the existing `Server` icon used for
"MCP Server Management."

**Line 4** -- update the lucide-react import:

```typescript
// BEFORE:
import { Terminal, BrainCircuit, Server, ShieldCheck } from 'lucide-react'

// AFTER:
import { Terminal, BrainCircuit, Server, ShieldCheck, Unplug } from 'lucide-react'
```

**Lines 7-12** -- update the `iconMap`:

```typescript
// BEFORE:
const iconMap = {
  terminal: Terminal,
  brain: BrainCircuit,
  server: Server,
  shield: ShieldCheck,
}

// AFTER:
const iconMap = {
  terminal: Terminal,
  brain: BrainCircuit,
  server: Server,
  shield: ShieldCheck,
  bridge: Unplug,
}
```

### Grid layout consideration

The Features component uses `grid grid-cols-1 md:grid-cols-2 gap-6` (line 26).
With 5 cards in a 2-column grid, the fifth card will render alone in the last
row on `md+` screens. This is acceptable and actually desirable -- it gives
the "Built-in AI Bridge" card visual prominence as the newest feature. The
card will stretch to its natural width within the first column of the grid.

No grid layout changes are needed. The `md:grid-cols-2` class handles odd
counts gracefully -- the last item occupies the first column of the last row.

If a full-width treatment is desired for the fifth card (spanning both columns),
the builder could optionally add a conditional class:

```typescript
className={`bg-bg-secondary border border-border rounded-xl p-8 hover:border-accent/30 transition-colors ${
  i === features.length - 1 && features.length % 2 !== 0 ? 'md:col-span-2' : ''
}`}
```

This is optional and NOT part of the required changes. The default single-column
rendering for the odd card is the simpler, safer choice.

---

## Edge Cases

### Empty state (no features)
Not applicable -- the features array is always non-empty (hardcoded data).

### TypeScript type safety
The `Feature` interface has `icon: string`, so `'bridge'` is a valid value. The
`iconMap` lookup in `Features.tsx` (line 28) uses `as keyof typeof iconMap`, which
means an unmapped icon key would result in `Icon` being `undefined` and a runtime
error. This is why the `iconMap` update is mandatory in the same PR.

### Comparison table overflow
Adding one row to the comparison table (11 rows total) does not introduce
horizontal or vertical overflow issues. The table already has `overflow-x-auto`
on its container (line 32 of `Comparison.tsx`).

### `mcpBridgeOutput` string not wired to a component
Per the backlog task, `mcpBridgeOutput` is exported but not rendered in any
component in this task. It is filed as data for a future task (0033) to add a
bridge-specific terminal demo section. The unused export will not cause a
TypeScript error -- unused exports are valid TS.

### `sampleToml` length increase
Adding 4 lines to `sampleToml` increases the Config section's code block height
slightly. The `CodeBlock` component renders inside a scrollable container, so
this will not break the layout. On small screens the code block is already
scrollable.

### Platform differences (macOS ARM64/x86_64, Ubuntu, WSL2)
Not applicable -- this is a React site with no platform-specific rendering.
The site builds identically on all platforms via `pnpm build:site`.

---

## Error Handling

No runtime error handling is needed for this task. All changes are to static
data files. The only failure mode is a TypeScript compilation error, which is
caught by the acceptance criteria build check.

| Scenario | Mitigation |
|----------|-----------|
| `bridge` icon key not in `iconMap` | TypeScript will not error (string index), but runtime will crash. The spec mandates updating `iconMap` in the same PR. |
| Typo in `ComparisonRow` field name | TypeScript will error because `ComparisonRow` interface requires all 7 fields. The build check catches this. |
| `mcpBridgeOutput` template literal syntax error | TypeScript build will fail. The spec provides the exact string to copy. |

---

## Security Considerations

None. This task modifies only static site data files (TypeScript) that are
compiled into a client-side bundle served from S3/CloudFront. No user input is
processed. No API calls are made. No secrets are involved.

---

## Testing Strategy

### Build verification (required)

```bash
pnpm --filter great-sh build
```

Must exit 0 with no TypeScript errors. This is the primary acceptance gate.

### Visual verification (manual)

```bash
pnpm --filter great-sh dev
```

1. Navigate to `/#features` -- verify 5 feature cards render. The fifth card
   ("Built-in AI Bridge") should display the Unplug icon in the accent green
   color. The "MCP Server Management" card should say "external MCP servers."

2. Navigate to `/#compare` -- verify the "Built-in multi-AI bridge (no Node.js)"
   row appears between "MCP server management" and "Credential management." The
   `great.sh` column should show a green check. All other columns should show
   an X.

3. Navigate to `/#config` -- verify the `great.toml` code block now includes the
   `[mcp_bridge]` stanza with `preset = "agent"` and `default_backend = "gemini"`.

### Automated checks (existing CI)

The existing `pnpm build:site` step in CI (which runs typecheck + Vite build)
will catch any TypeScript errors introduced by these changes. No new test files
are needed.

---

## Acceptance Criteria (from backlog, verified against spec)

- [x] `site/src/data/features.ts` contains exactly 5 `Feature` entries; the new
      entry has `icon: "bridge"` and title "Built-in AI Bridge"
- [x] `site/src/data/comparison.ts` contains a row with
      `feature: "Built-in multi-AI bridge (no Node.js)"` where `great: true`
      and all other columns are `false`
- [x] `site/src/data/commands.ts` exports `mcpBridgeOutput` (a template-literal
      string showing `$ great mcp-bridge --preset agent` startup output with
      3 detected backends listed: Gemini, Codex, Claude)
- [x] `pnpm --filter great-sh build` exits 0 with no TypeScript errors
- [x] The existing "MCP Server Management" card description is updated to scope
      it to "external MCP servers from the registry" (added "external" and
      "third-party tools")
- [x] The `iconMap` in `Features.tsx` maps `bridge` to `Unplug` from lucide-react
- [x] `sampleToml` in `commands.ts` includes a `[mcp_bridge]` stanza

---

## Implementation Order

All changes are independent and can be made in any order. However, for a clean
commit, the recommended order is:

1. **`site/src/data/features.ts`** -- Update MCP Server Management description,
   add Built-in AI Bridge entry. (2 edits in one file)
2. **`site/src/data/comparison.ts`** -- Add new row after "MCP server management."
   (1 insertion)
3. **`site/src/data/commands.ts`** -- Add `mcpBridgeOutput` export, update
   `sampleToml`. (2 edits in one file)
4. **`site/src/components/sections/Features.tsx`** -- Add `Unplug` import and
   `iconMap` entry. (2 line changes)
5. **Build check** -- `pnpm --filter great-sh build` to verify TypeScript
   compilation succeeds.
