# 0011: Update Marketing Site — Promote great loop + Reflect Current Features

**Priority:** P0
**Type:** feature
**Module:** `site/`
**Status:** pending
**Estimated Complexity:** M (data changes + one new section component)

---

## Context

The marketing site at `site/` was written before `great loop` existed as a
shipped feature. The site must now position `great loop` as the primary
differentiator — a one-command install of a 13-role AI agent orchestration
methodology into Claude Code. Secondary goal: align all data and copy with the
actual CLI as it exists today (all 11 subcommands, `kerckhoffs` security agent,
`great doctor --fix`, correct tool lists from `apply.rs`, etc.).

### What the site shows now

| Area | Current state |
|---|---|
| Hero tagline | "The managed AI dev environment." / "One command. Fully configured. Cloud-synced. Open source." |
| Features (4 cards) | One Command Setup, AI Agent Orchestration (generic), MCP Server Management, Cloud-Synced Credentials |
| Features description | "touches all five layers" — no mention of loop |
| HowItWorks steps | Install, Initialize, Code, Sync — no loop step |
| Nav links | Features, Config, How it Works, Templates, Compare — no "Loop" link |
| Comparison table | 9 rows, no "AI agent loop / methodology" row |
| No section | Nothing about the 13 agents, their roles, the slash commands, or `great loop install` |
| `initWizardOutput` in commands.ts | Shows Gemini CLI in agent selector — Gemini CLI is confirmed present in apply.rs but the loop agent count (13) is not surfaced |
| `sampleToml` in commands.ts | Does not include `[loop]` or reference the loop at all |
| Footer | Links to architecton.ai — inconsistent with the brand rename away from "Architecton" |
| Features "AI Agent Orchestration" card | Generic copy; does not name the loop, its 13 roles, or `great loop install` |
| Comparison table | No row for "AI agent orchestration loop" which is unique to great.sh |
| Templates section | Does not mention loop — templates could note "loop-ready" |

### What the site must show after this task

1. **A dedicated "great loop" section** (new) that is the centrepiece of the
   page — placed between Features and Config (or between HowItWorks and
   Templates).
2. **Hero sub-tagline** updated to reference the loop.
3. **Features card** for AI Agent Orchestration updated with loop specifics.
4. **HowItWorks** gains a step 05: `great loop install --project`.
5. **Nav** gains a "Loop" anchor link.
6. **Comparison table** gains an "AI agent orchestration loop" row.
7. **Footer** removes or replaces the `architecton.ai` link (inconsistent with
   project branding — the brand is "great.sh", not "Architecton").
8. **commands.ts** `initWizardOutput` updated to include `great loop install`
   after the environment is ready.

---

## great loop — Facts from Source Code

Extracted from `src/cli/loop_cmd.rs` and `loop/commands/loop.md`:

**CLI surface**
- `great loop install` — writes 13 agent `.md` files to `~/.claude/agents/`,
  4 slash-command `.md` files to `~/.claude/commands/`, a `teams-config.json`
  to `~/.claude/teams/loop/`, and a `settings.json` with
  `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`.
- `great loop install --project` — additionally creates `.tasks/` kanban dirs
  (`backlog/`, `ready/`, `in-progress/`, `done/`, `reports/`) and appends
  `.tasks/` to `.gitignore`.
- `great loop status` — checks all of the above are installed.
- `great loop uninstall` — removes loop files from `~/.claude/`.

**The 13 agents** (from `AGENTS` array in `loop_cmd.rs`):
nightingale, lovelace, socrates, humboldt, davinci, vonbraun, turing,
kerckhoffs, rams, nielsen, knuth, gutenberg, hopper

**The 4 slash commands**: `/loop`, `/bugfix`, `/deploy`, `/discover`

**Loop execution flow** (from `loop/commands/loop.md`):
Phase 1 sequential: Nightingale → Lovelace → Socrates → Humboldt
Phase 2 parallel team: Da Vinci + Turing + Kerckhoffs + Nielsen
Phase 3 gate check
Phase 4 finish: Rams → Hopper + Knuth + Gutenberg
Phase 5 clean up + observer report

**Install summary line** (from `run_install` output):
"14 roles: 4 teammates + 9 subagents + 1 team lead"
Usage: `claude` → `/loop [task description]`

**MCP routing used by loop agents** (from global CLAUDE.md):
- Gemini: large-context codebase analysis (Humboldt, Lovelace)
- Codex: fast code generation (Da Vinci)
- Context7: crate/library documentation (Lovelace, Da Vinci)
- Playwright: visual and UX review (Rams, Nielsen)

---

## Acceptance Criteria

1. **New "great loop" section exists in the page** between HowItWorks and
   Templates, with `id="loop"` and reachable via a Nav link labelled "Loop".
   The section must name at least: the 13-agent count, the 4 slash commands,
   `great loop install` / `great loop install --project`, and the sequential +
   parallel execution model. A terminal block showing `great loop install`
   output is required.

2. **Features section "AI Agent Orchestration" card** updated to reference
   the great loop by name, cite 13 agents + 4 slash commands, and mention
   `great loop install` as the setup command. Icon remains `brain`.

3. **HowItWorks section** gains a fifth step: number `05`, title "Start the
   Loop", description referencing `/loop [task]` in Claude Code, command
   `great loop install --project`. The terminal mock in `initWizardOutput`
   (commands.ts) appends a final line: `Run \`great loop install\` to add the
   13-agent team.`

4. **Comparison table** (`site/src/data/comparison.ts`) gains a new row:
   `feature: 'AI agent orchestration loop'` with `great: true` and all
   competitors `false`.

5. **Footer** removes the `architecton.ai` link and its surrounding text
   ("Part of the architecton.ai ecosystem"). Replace with a plain copyright
   line or nothing — confirm with designer before implementation, but the link
   must not appear in production.

---

## Files That Need to Change

### New file (create)
- `site/src/components/sections/Loop.tsx` — new section component

### Data files
- `site/src/data/features.ts` — update "AI Agent Orchestration" card copy
- `site/src/data/commands.ts` — append line to `initWizardOutput`; optionally
  add a `loopInstallOutput` export for the Loop section terminal block
- `site/src/data/comparison.ts` — add "AI agent orchestration loop" row

### Existing components
- `site/src/App.tsx` — import and insert `<Loop />` between `<HowItWorks />`
  and `<Templates />`
- `site/src/components/layout/Nav.tsx` — add `{ label: 'Loop', href: '#loop' }`
  to `navLinks`
- `site/src/components/sections/HowItWorks.tsx` — add step 05 to `steps`
  array; update terminal mock if `initWizardOutput` is updated in commands.ts
- `site/src/components/layout/Footer.tsx` — remove architecton.ai link block
  (lines 38-46)

### No change required
- `site/src/data/templates.ts` — templates are accurate; "loop-ready" labelling
  is a P2 nicety, out of scope here
- `site/src/components/sections/Config.tsx` — sampleToml is representative;
  adding `[loop]` section is a P2 nicety
- `site/src/components/sections/Comparison.tsx` — component reads from data
  file; only data file changes

---

## Loop Section — Suggested Content

### Heading
"great loop — 13 agents, one command"

### Sub-heading / intro
"The great.sh Loop is a 13-role AI agent orchestration methodology, shipped
inside every great.sh install. One command configures Claude Code with a
full team: requirements curators, spec writers, builders, testers, security
auditors, UX reviewers, documenters, and an observer. Type `/loop [task]`
and the team goes to work."

### Execution model (visual or list)
Sequential: Nightingale → Lovelace → Socrates → Humboldt
Parallel team: Da Vinci + Turing + Kerckhoffs + Nielsen
Finish: Rams + Hopper + Knuth + Gutenberg + Deming (observer)

### Commands block (terminal)
```
$ great loop install --project

  great.sh Loop — Installing agent team

  [check] 13 agent personas -> ~/.claude/agents/
  [check] 4 commands -> ~/.claude/commands/
  [check] Agent Teams config -> ~/.claude/teams/loop/
  [check] Settings with Agent Teams enabled -> ~/.claude/settings.json
  [check] .tasks/ created, .gitignore updated

  great.sh Loop installed!

  14 roles: 4 teammates + 9 subagents + 1 team lead
  Usage: claude -> /loop [task description]
```

### Slash commands list
`/loop`, `/bugfix`, `/deploy`, `/discover`

### Setup CTA
Two commands:
```
great loop install           # global: adds agents to ~/.claude/
great loop install --project # also sets up .tasks/ kanban in your repo
```

---

## Dependencies

- No blocking CLI tasks — `great loop` is fully implemented and tested.
- Site build: `pnpm build:site` must pass with zero TypeScript errors.
- The new `Loop.tsx` component must follow existing patterns:
  - `AnimatedSection` wrapper with matching `id`
  - `Container` for max-width / padding
  - `TerminalWindow` for the terminal block
  - `motion` from `motion/react` for stagger animations
  - Dark terminal theme: bg `#0a0a0a`, accent green `#22c55e`
  - Fonts: Space Grotesk headings, JetBrains Mono for code

---

## Out of Scope for This Task

- Adding a `[loop]` section to `sampleToml` (P2)
- Marking templates as "loop-ready" (P2)
- A dedicated `/loop` docs page (P2 — docs task)
- Any backend or analytics changes
- Changing the install script itself
