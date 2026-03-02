# Scout Report: 0011 -- Loop Site Accuracy

**Scout:** Alexander von Humboldt
**Date:** 2026-02-21
**Spec:** `.tasks/ready/0011-loop-site-accuracy.md`

---

## 1. File Map

### 1a. `site/src/components/sections/Loop.tsx` (158 lines)

**Imports (lines 1-5)**
```
1: import { AnimatedSection }    from '@/components/shared/AnimatedSection'
2: import { Container }          from '@/components/layout/Container'
3: import { TerminalWindow }     from '@/components/shared/TerminalWindow'
4: import { loopInstallOutput }  from '@/data/commands'   <-- data dependency
5: import { motion }             from 'motion/react'
```

**`const phases` array (lines 7-40)** -- needs full replacement
- Line 7: array opens
- Lines 8-17: Phase 1 (4 agents, sequential) -- agents have `{name, role}` only
- Lines 18-28: Phase 2 (5 agents: Da Vinci, Von Braun, Turing, Kerckhoffs, Nielsen)
  - Von Braun at line 22 must be REMOVED
  - 5 agents drops to 4
  - `note` field must be ADDED to the Phase 2 object
- Lines 29-39: Phase 3 label "Finish" at line 30 -- needs change to "Gate + Finish"
  - Agents: Rams, Hopper, Knuth, Gutenberg, Deming (5 agents, no Dijkstra)
  - Dijkstra must be INSERTED before Rams
  - Hopper stays after Dijkstra and Rams (correct position in new spec)
- Line 40: array closes

**`const slashCommands` array (lines 42-47)** -- needs full replacement
- Line 42: array opens
- Line 43: `/loop` (desc: 'Full development cycle')
- Line 44: `/bugfix` (desc: 'Diagnose and fix a bug')
- Line 45: `/deploy` (desc: 'Build, test, and ship')
- Line 46: `/discover` (desc: 'Explore and document a codebase')
- Line 47: array closes
- `/backlog` is entirely absent and must be added as the FIRST entry

**`export function Loop()` body:**
- Line 54-57: Heading `<h2>` -- contains "15 agents, one command" at line 56
- Lines 58-68: Intro `<p>` -- "15-role" at line 59, "/loop [task]" at line 65
- Lines 70-104: Agent flow render loop (no changes to outer structure)
- Lines 83-100: Per-agent inner `<div>` card template -- needs `methodology` line added
  - Exact current card (lines 86-93):
    ```tsx
    <div className="bg-bg-secondary border border-border rounded-lg px-4 py-2.5 hover:border-accent/30 transition-colors">
      <div className="font-display text-sm text-text-primary">
        {agent.name}
      </div>
      <div className="text-text-tertiary text-xs">
        {agent.role}
      </div>
    </div>
    ```
  - Add `methodology` div after `role` div, before closing `</div>`
- Line 101: `</div>` closes the `flex flex-wrap` container
- Line 102: `</motion.div>` closes the phase block
  - The `{phase.note && ...}` block must be inserted BETWEEN lines 101 and 102
- Lines 128-130: Slash commands `<h3>` heading -- "Four slash commands" at line 129
- Lines 105-157: Remainder of component (no other changes required)

---

### 1b. `site/src/components/sections/HowItWorks.tsx` (93 lines)

**`const steps` array (lines 7-38)**
- Step 5 object at **lines 32-37**:
  ```typescript
  {
    number: '05',
    title: 'Start the Loop',
    description: 'Install the 15-agent team into Claude Code. Type /loop and describe your task.',
    command: 'great loop install --project',
  },
  ```
- Only the `description` field changes. Exact target string:
  `'Install the 15-agent team into Claude Code. Type /loop and describe your task.'`

**Heading (line 44):** "Zero to hero in five steps" -- NO CHANGE (spec confirms keep as-is)

**Import at line 4:** `import { initWizardOutput } from '@/data/commands'`
- This is a different named export (`initWizardOutput`) from the same `commands.ts` file.
  No change needed here.

---

### 1c. `site/src/data/features.ts` (33 lines)

**Line 17** (single long string, the `description` of the second feature):
```typescript
'The great.sh Loop: 15 specialized AI agents installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, UX reviews, docs, and deploys — orchestrated as a team. Run great loop install to set it up.',
```
- Target: `15 specialized AI agents` --> `16 specialized AI roles`
- Also expand the list to include "performance checks, code reviews, UX inspections, visual reviews"
- Note: the em dash in the string (`—`) is a real Unicode em dash (U+2014), not `--`

---

### 1d. `site/src/data/commands.ts` (93 lines)

Three locations in this file; one is a NO-CHANGE:

**Line 33** -- inside `initWizardOutput` template literal:
```
  Run \`great loop install\` to add the 15-agent team.`
```
Change: `15-agent team` --> `16-role agent team`

**Line 83** -- inside `loopInstallOutput` template literal:
```
  [check] 15 agent personas -> ~/.claude/agents/
```
NO CHANGE. The spec explicitly states this line is technically accurate (15 persona files; Deming is team lead, not a persona file). Confirmed: `loop/agents/` contains exactly 15 `.md` files.

**Line 84** -- inside `loopInstallOutput` template literal:
```
  [check] 4 commands -> ~/.claude/commands/
```
Change: `4 commands` --> `5 commands`
Confirmed: `loop/commands/` contains exactly 5 files: `backlog.md`, `loop.md`, `bugfix.md`, `deploy.md`, `discover.md`.

**Line 91** (already correct, no change):
```
  16 roles: 4 teammates + 11 subagents + 1 team lead
```

---

## 2. Dependency Graph

```
App.tsx
  imports Loop         from site/src/components/sections/Loop.tsx
  imports HowItWorks   from site/src/components/sections/HowItWorks.tsx
  imports Features     from site/src/components/sections/Features.tsx

Loop.tsx
  imports loopInstallOutput from site/src/data/commands.ts  [SHARED with no other component]

HowItWorks.tsx
  imports initWizardOutput  from site/src/data/commands.ts  [SHARED -- Config.tsx also imports sampleToml]

Features.tsx
  imports features           from site/src/data/features.ts [sole consumer]

site/src/data/commands.ts  -- named exports:
  installCommand    -> Hero.tsx
  initWizardOutput  -> HowItWorks.tsx
  sampleToml        -> Config.tsx
  loopInstallOutput -> Loop.tsx
```

No circular dependencies. All four files under change have exactly one consumer each for the relevant export (except `commands.ts` which has four named exports consumed by four different components; changes touch only `initWizardOutput` and `loopInstallOutput`).

---

## 3. CSS / Tailwind Audit

**Tailwind version:** `^3.4.17` (JIT mode, arbitrary values supported via `max-w-[220px]` syntax)

**New class introduced by spec:** `max-w-[220px]`
- This is a Tailwind JIT arbitrary value. It does NOT appear anywhere in the codebase today (confirmed by grep).
- Tailwind 3 JIT generates it on-demand from source scanning. The `tailwind.config.ts` content glob is `./src/**/*.{ts,tsx}` which covers `Loop.tsx`. Safe to use.

**Existing classes used in the spec's JSX additions -- all confirmed present in codebase:**
- `text-text-tertiary` -- present in Loop.tsx lines 80, 90
- `text-xs` -- present throughout
- `font-mono` -- present in Loop.tsx line 80
- `mt-0.5` -- standard Tailwind spacing unit
- `mt-2` -- present in Loop.tsx line 142
- `pl-1` -- standard Tailwind spacing unit

**No new Tailwind plugins or theme extensions are required.**

---

## 4. Ground Truth Verification

Verified against actual source files in `loop/`:

| Claim | Source | Status |
|-------|--------|--------|
| 15 agent persona files in `loop/agents/` | Glob confirms 15 `.md` files | Correct -- line 83 stays unchanged |
| 5 command files in `loop/commands/` | Glob confirms 5 `.md` files: backlog, loop, bugfix, deploy, discover | Correct -- line 84 changes to "5 commands" |
| Deming is NOT an agent persona file | `loop/agents/` has no `deming.md` | Confirmed |
| `/backlog` command exists | `loop/commands/backlog.md` present | Confirmed |

---

## 5. Additional Files Mentioning "15 agents" / "16-agent" -- Beyond the 4 Target Files

A broad grep across all site and loop files found:

| File | Line | Content | Action |
|------|------|---------|--------|
| `site/src/data/features.ts` | 17 | "15 specialized AI agents" | Change (in scope) |
| `site/src/data/commands.ts` | 33 | "15-agent team" | Change (in scope) |
| `site/src/data/commands.ts` | 83 | "15 agent personas" | NO CHANGE (accurate) |
| `site/src/components/sections/Loop.tsx` | 56 | "15 agents, one command" | Change (in scope) |
| `site/src/components/sections/Loop.tsx` | 59 | "15-role AI agent" | Change (in scope) |
| `site/src/components/sections/HowItWorks.tsx` | 35 | "15-agent team" | Change (in scope) |
| `CLAUDE.md` (project) | 77 | "15 agents" in loop/agents/ comment | Out of scope (accurate; spec says so) |
| `loop/commands/loop.md` | 5 | "full 16-agent great.sh Loop" | Already correct, no change needed |

**No additional site files found with stale "15" counts.** The four files in the spec are the complete set.

---

## 6. Risks and Technical Debt

**Risk 1: TypeScript type inference for `methodology`**
The spec does not add a TypeScript interface declaration -- it relies on `const` inference.
This is safe for Tailwind/TSX rendering but means the type of `phase.agents[n].methodology`
is inferred as `string` from the literal. If `methodology` is accidentally omitted from any
agent object, TypeScript will NOT error (the inferred type becomes `{name: string, role: string}`
with no `methodology`). The builder must ensure every agent object in the replacement
`phases` array includes all three fields.

**Risk 2: `{phase.note && ...}` placement**
The `note` renderer must be inserted between line 101 (`</div>` closing `flex flex-wrap`) and
line 102 (`</motion.div>`). If inserted inside the `flex flex-wrap` div instead, it will be
rendered as a flex child (inline) rather than a block below the agent row. Exact placement matters.

**Risk 3: Phase 3 agent count on desktop**
Adding Dijkstra brings Phase 3 from 5 to 6 agents. The spec notes this will wrap to two rows
on `max-w-7xl` containers. This is expected behavior -- no CSS fix needed -- but the builder
should visually verify at 1440px that the wrapping is clean.

**Risk 4: Arrow separators between agents**
The current JSX renders `->` arrows between agents (line 94-98). With Phase 3 gaining Dijkstra,
the arrow rendering is purely count-driven (`i < phase.agents.length - 1`), so no JSX
adjustment is needed -- it will render correctly automatically.

**No existing technical debt** in the four target files blocks this change.

---

## 7. Recommended Build Order

All changes are independent. Recommended order per spec, smallest to largest:

1. `site/src/data/features.ts` -- line 17, one string
2. `site/src/data/commands.ts` -- lines 33 and 84, two strings
3. `site/src/components/sections/HowItWorks.tsx` -- line 35, one string
4. `site/src/components/sections/Loop.tsx` -- phases array, slashCommands array, heading, paragraph, card JSX, note renderer (largest)

After all changes: `cd site && pnpm build:site` must produce zero TypeScript errors.
