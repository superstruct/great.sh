# 0011: Fix Loop Documentation and Marketing Site Accuracy

**Priority:** P1
**Created:** 2026-02-21
**Type:** docs
**Module:** `site/` + `loop/`
**Status:** backlog

## Problem

The marketing site (`site/src/components/sections/Loop.tsx` and
`site/src/components/sections/HowItWorks.tsx`) misrepresents the loop's
structure, agent order, agent descriptions, and workflow in several concrete
ways. The `features.ts` copy also contains a minor count error. These
inaccuracies erode trust with technically sophisticated visitors who will verify
what they read against the actual loop files.

### Specific Inaccuracies

#### 1. Agent phases in Loop.tsx are structurally wrong

`site/src/components/sections/Loop.tsx` lines 7-40 define three phases:

**Current Phase 2 (parallel):**
```
Da Vinci + Von Braun + Turing + Kerckhoffs + Nielsen
```

**Actual structure per `loop/commands/loop.md`:**
- Phase 2 spawns four teammates: Da Vinci, Turing, Kerckhoffs, Nielsen (parallel)
- Von Braun is NOT a Phase 2 teammate in `/loop` — Von Braun only appears in the
  separate `/deploy` command
- Wirth runs in parallel with the teammate team as an independent subagent
  (performance sentinel, not shown at all)

**Current Phase 3 (finish):**
```
Rams -> Hopper -> Knuth -> Gutenberg -> Deming
```

**Actual order per `loop/commands/loop.md` Phase 3/4/5:**
- Dijkstra (code quality gate) runs after teammate phase, before commit
- Rams (visual review) runs after Dijkstra
- Hopper (code commit) follows — ALL gates must pass
- Knuth + Gutenberg (docs) run after Hopper
- Deming (observer) closes the loop

Hopper is currently shown second but should be after Dijkstra and Rams.
Dijkstra is missing from the phase diagram entirely.

#### 2. /backlog command is missing from the site's slash commands list

`site/src/components/sections/Loop.tsx` lines 42-47 lists four slash commands:
`/loop`, `/bugfix`, `/deploy`, `/discover`.

The `/backlog` command (`loop/commands/backlog.md`) is the correct entry point
for requirements capture. The user's feedback explicitly calls out that
`/backlog` should be run BEFORE `/loop` (separate steps). This is the most
common first interaction with the loop and must be surfaced prominently.

The `/discover` description is also inaccurate: "Explore and document a
codebase" — the actual command (`loop/commands/discover.md`) runs Nielsen for a
UX discovery sweep followed by Nightingale triaging findings to backlog. It is
about UX discovery, not general codebase exploration.

#### 3. Agent role descriptions are thin and interchangeable

The phase diagram shows bare role labels: "Requirements", "Spec", "Build",
"Test", etc. These give no sense of the historical figure, their methodology,
or why that specific person was chosen. The agent files contain rich descriptions
that are not surfaced at all.

Current labels vs richer descriptions available in agent files:
- Nightingale: "Requirements" — actually transforms chaos into organized task
  files using statistical discipline
- Socrates: "Review" — adversarial plan approval gate using structured elenchus
- Humboldt: "Scout" — maps codebase connections before anyone touches code
- Kerckhoffs: "Security" — applies Kerckhoffs' 6 cryptographic principles to
  audit credential leakage, permissions, supply chain
- Wirth: absent — performance sentinel fighting binary/bundle size regressions
- Dijkstra: absent — code quality gate, structured programming principles
- Rams: "Visual QA" — applies 10 Principles of Good Design to output aesthetics
- Nielsen: "UX" — 10 Usability Heuristics, last gate before commit
- Hopper: "Commit" — never commits a broken build, pre-commit gate runs all CI

#### 4. features.ts agent count is wrong

`site/src/data/features.ts` line 17:
```
'The great.sh Loop: 15 specialized AI agents...'
```

The actual roster is 16 roles: 4 teammates (Da Vinci, Turing, Kerckhoffs,
Nielsen) + 11 subagents (Nightingale, Lovelace, Socrates, Humboldt, Wirth,
Dijkstra, Rams, Knuth, Gutenberg, Hopper, Von Braun) + 1 team lead (Deming).
This is confirmed by `loop/teams-config.json` description field and the install
terminal output in `commands.ts` line 91: "16 roles: 4 teammates + 11 subagents
+ 1 team lead". The features.ts copy says 15 while the install output says 16 —
these must be reconciled to 16.

#### 5. HowItWorks.tsx step 5 description is incomplete

`site/src/components/sections/HowItWorks.tsx` lines 34-38, step 5:
```
title: 'Start the Loop',
description: 'Install the 15-agent team into Claude Code. Type /loop and describe your task.',
command: 'great loop install --project',
```

This conflates installation with usage and still says "15-agent". The correct
two-step workflow is: first run `/backlog` to capture requirements, then run
`/loop` to execute. The description should also reflect the 16-role count.

#### 6. Loop.tsx intro copy says "one command" which is misleading

`site/src/components/sections/Loop.tsx` lines 64-67:
```
Type `/loop [task]` and the team goes to work.
```

The recommended workflow is `/backlog [raw idea]` first (requirements capture by
Nightingale into `.tasks/backlog/`), then `/loop [task-id]` to execute. Telling
users to jump straight to `/loop` skips the requirements step and may produce
poor results.

---

## Acceptance Criteria

- [ ] `Loop.tsx` phase diagram shows the correct agent roster and order:
  Phase 1 (sequential): Nightingale, Lovelace, Socrates, Humboldt;
  Phase 2 (parallel): Da Vinci, Turing, Kerckhoffs, Nielsen (+ Wirth as
  independent parallel subagent);
  Phase 3 (sequential): Dijkstra, Rams, Hopper, Knuth+Gutenberg, Deming.
  Von Braun does NOT appear in the main loop diagram.
- [ ] `Loop.tsx` slash commands list includes `/backlog` (shown first or
  prominently labelled "start here") with description matching its actual
  purpose: requirements capture before running `/loop`. `/discover` description
  corrected to "UX discovery sweep — Nielsen maps journeys, Nightingale files issues".
- [ ] Each agent card in the phase diagram includes a one-line methodology
  description drawn from the agent files (historical figure + what they
  contribute), not just a bare role word.
- [ ] `features.ts` and all copy in `Loop.tsx`, `HowItWorks.tsx` consistently
  use "16 roles" / "16 agents" (not 15).
- [ ] `HowItWorks.tsx` step 5 splits into two steps or revises its description
  to communicate the two-step workflow: `great loop install --project` to set up,
  then `/backlog` before `/loop`.

## Notes

- Source of truth for loop order: `/home/isaac/src/sh.great/loop/commands/loop.md`
- Source of truth for agent descriptions: `/home/isaac/src/sh.great/loop/agents/*.md`
- Source of truth for slash commands: `/home/isaac/src/sh.great/loop/commands/*.md`
- The install terminal output in `site/src/data/commands.ts` line 91 already
  correctly says "16 roles" — use this as the canonical count.
- Von Braun belongs in the `/deploy` command description, not the main loop
  phase diagram. He could be noted in a "other commands" context.
- Wirth (Performance Sentinel) and Dijkstra (Code Reviewer) are currently
  invisible in all marketing copy — they are real, gated agents that block
  commits and must be represented.
- Agent card width may need to grow to accommodate one-line descriptions;
  check mobile layout at 375px after changes.
- No change to the actual loop agent files or commands is in scope for this
  task — site and docs accuracy only.
