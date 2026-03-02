# 0033 — Marketing Site: Wire mcpBridgeOutput into a Bridge Section Component

**Priority:** P2
**Type:** feature (content/site)
**Module:** site/
**Status:** backlog
**Complexity:** S
**Created:** 2026-02-28

## Context

Task 0032 (iteration 029) exported `mcpBridgeOutput` from
`site/src/data/commands.ts` but deliberately left it unwired, noting:

> "unwired — future task"
> "filing it here so Lovelace can spec the component connection as a follow-up
> (0033) if desired"

`mcpBridgeOutput` is a terminal demo string showing `great mcp-bridge --preset
agent` startup output — 3 discovered backends, preset name, tool list, and
the "Listening on stdio" confirmation line. It mirrors the style of
`loopInstallOutput`, which is rendered by the `Loop` section as a
`TerminalWindow` alongside explanatory copy.

The "Built-in AI Bridge" feature card (one of 5 in the Features grid) and the
comparison row for "Built-in multi-AI bridge (no Node.js)" both exist, but
there is no dedicated deep-dive section for mcp-bridge. Every other major
feature (Config, HowItWorks, Loop) has its own `AnimatedSection` with a
terminal window demo. The bridge is the newest, most differentiated feature
(pure-Rust, no Node.js, 5 backends, 4 presets) — a shallow feature card is
underselling it.

Current section order in `App.tsx`:
`Hero → Features → Config → HowItWorks → Loop → Templates → Comparison → OpenSource`

The bridge section should be inserted after `Loop` and before `Templates`,
keeping the narrative flow: setup → loop → bridge → templates → compare.

## What mcpBridgeOutput Shows

```
$ great mcp-bridge --preset agent

  great.sh MCP Bridge — Starting (preset: agent)

  Discovering backends...
  [check] Gemini CLI    gemini (GEMINI_API_KEY set)
  [check] Codex CLI     codex  (OPENAI_API_KEY set)
  [check] Claude CLI    claude (logged in)

  Preset: agent (6 tools)
  Tools: prompt, run, wait, list_tasks, get_result, kill_task

  Listening on stdio (JSON-RPC 2.0)
```

## Requirements

1. Create `site/src/components/sections/Bridge.tsx` — a new `AnimatedSection`
   with `id="bridge"`. Layout: two-column (lg), left = copy + feature bullets,
   right = `TerminalWindow` rendering `mcpBridgeOutput`. Follow the exact
   structural pattern of `Loop.tsx` (heading, sub-copy, two-column grid with
   terminal + prose panel). Heading text: "great mcp-bridge — five backends,
   zero Node.js". No new shared components; use `AnimatedSection`, `Container`,
   `TerminalWindow` (all existing).

2. Wire the new `Bridge` section into `site/src/App.tsx` between `<Loop />` and
   `<Templates />`.

3. Add a nav link for the bridge section to `site/src/components/layout/Nav.tsx`
   — label `"Bridge"`, href `"#bridge"`. Insert between the `"Loop"` and
   `"Templates"` entries so the nav reflects page order.

4. The left-column copy must cover four bullets matching what ships in the
   binary: (a) 5 backends — Gemini, Codex, Claude, Grok, Ollama; (b) 4 presets
   — minimal / agent / research / full; (c) zero Node.js — pure Rust, single
   binary; (d) auto-registered in `.mcp.json` by `great apply`. Do not invent
   capabilities; only describe what is in `src/mcp/bridge/`.

5. No Rust code changes. No new npm/pnpm packages. No modifications to
   `commands.ts` or `features.ts` — `mcpBridgeOutput` is already correct.

## Acceptance Criteria

- [ ] `site/src/components/sections/Bridge.tsx` exists, imports `mcpBridgeOutput`
      from `@/data/commands`, and renders it inside a `<TerminalWindow>` with
      title `"great mcp-bridge --preset agent"`
- [ ] `site/src/App.tsx` renders `<Bridge />` between `<Loop />` and
      `<Templates />`, and imports it from `@/components/sections/Bridge`
- [ ] `site/src/components/layout/Nav.tsx` includes `{ label: 'Bridge', href: '#bridge' }`
      between the Loop and Templates entries in `navLinks`
- [ ] `pnpm --filter great-sh build` (or `pnpm build:site`) exits 0 with zero
      TypeScript errors after all three file changes
- [ ] The four feature bullets (5 backends, 4 presets, no Node.js, auto-registered
      by `great apply`) are present in the rendered copy and are factually correct
      per `src/mcp/bridge/backends.rs` (BACKEND_SPECS) and `src/mcp/bridge/tools.rs`
      (Preset enum)

## Dependencies

- `mcpBridgeOutput` already exported from `site/src/data/commands.ts` (task 0032)
- `TerminalWindow`, `AnimatedSection`, `Container` components all exist
- `motion/react` already imported in `Loop.tsx` — same import available for
  entrance animations

## Notes

- The Bridge section is S-complexity: 3 files, no new shared components, no
  Rust changes, no new dependencies. Lovelace spec should be short (~300 lines).
- Keep Rust tests at 327; this task touches no Rust files. Turing / Kerckhoffs /
  Wirth can be skipped (same rationale as iterations 028-029).
- Dijkstra WARN from iteration 029 about "unused export" (`mcpBridgeOutput`)
  is resolved by this task — reference that finding in the Lovelace spec.
- Do not add a sixth nav link if the total nav links would exceed 7 on desktop;
  current count is 6, adding "Bridge" brings it to 7, which is acceptable.
- Mobile nav collapse at `md:` breakpoint already handles overflow gracefully.
