# Socrates Review: 0011 -- Fix Loop Documentation and Marketing Site Accuracy

**Spec:** `.tasks/ready/0011-loop-site-accuracy.md`
**Backlog:** `.tasks/backlog/0011-loop-site-accuracy.md`
**Reviewed:** 2026-02-21

**Verdict:** APPROVED WITH NOTES

---

## Issues Found

### 1. commands.ts line 84: "4 commands" should be "5 commands"

**Severity: WARNING**

The spec adds `/backlog` as a fifth slash command in the site UI and correctly
updates the slash commands heading from "Four" to "Five." However, the terminal
output simulation in `site/src/data/commands.ts` line 84 still reads:

```
[check] 4 commands -> ~/.claude/commands/
```

There are now 5 command files in `loop/commands/` (backlog, loop, bugfix,
deploy, discover). The spec explicitly addresses line 83 (persona count) and
line 33 (agent team string) but silently leaves line 84 unchanged, creating an
internal inconsistency: the site's slash commands section will show 5 commands
while the terminal output immediately adjacent says 4 were installed.

**Recommendation:** Add a Change 7 (or amend Change 6) to update line 84 from
`4 commands` to `5 commands`.

### 2. Spec's note JSX uses bare `+` that may confuse readers

**Severity: NOTE**

The note renderer JSX contains:

```tsx
<div className="text-text-tertiary text-xs font-mono mt-2 pl-1">
  + {phase.note}
</div>
```

The literal `+` before `{phase.note}` will render as a visible plus sign in the
UI, which seems intentional (to suggest "also running in parallel"). This is
fine as a design choice, but the spec does not explain the visual semantics. A
builder might wonder if this is a typo or intentional. A one-line comment in the
spec explaining why the `+` is there would help.

### 3. Phase 3 label simplification is undocumented rationale

**Severity: NOTE**

The canonical `loop.md` defines 5 phases (Phase 1: Sequential, Phase 2: Spawn
Team, Phase 3: Gate Check, Phase 4: Finish, Phase 5: Clean Up). The spec
collapses these into 3 marketing phases, merging Phases 3-5 into
"Phase 3 -- Gate + Finish." This is a reasonable simplification for a marketing
page, and the agent ordering within the merged phase is correct. However, the
spec does not document WHY three phases were chosen over five. A visitor who
reads `loop.md` would see five phases and might report the site as inaccurate.

**Recommendation:** Add a brief note in the "Out of Scope" or "Edge Cases"
section acknowledging that the marketing site intentionally consolidates 5
canonical phases into 3 visual phases for readability, and noting that the
agent ordering is preserved.

---

## Cross-Check Results

### Agent Roster Accuracy (16 roles)

| # | Agent | Spec Phase | loop.md Phase | Match? |
|---|-------|-----------|---------------|--------|
| 1 | Nightingale | Phase 1 | Phase 1 (#1) | YES |
| 2 | Lovelace | Phase 1 | Phase 1 (#2) | YES |
| 3 | Socrates | Phase 1 | Phase 1 (#3) | YES |
| 4 | Humboldt | Phase 1 | Phase 1 (#4) | YES |
| 5 | Da Vinci | Phase 2 | Phase 2 (teammate) | YES |
| 6 | Turing | Phase 2 | Phase 2 (teammate) | YES |
| 7 | Kerckhoffs | Phase 2 | Phase 2 (teammate) | YES |
| 8 | Nielsen | Phase 2 | Phase 2 (teammate) | YES |
| 9 | Wirth | Phase 2 (note) | Phase 2 (parallel subagent) | YES |
| 10 | Dijkstra | Phase 3 | Phase 3 (gate check) | YES |
| 11 | Rams | Phase 3 | Phase 4 (finish) | YES |
| 12 | Hopper | Phase 3 | Phase 4 (finish) | YES |
| 13 | Knuth | Phase 3 | Phase 4 (finish) | YES |
| 14 | Gutenberg | Phase 3 | Phase 4 (finish) | YES |
| 15 | Von Braun | /deploy only | /deploy only | YES |
| 16 | Deming | Phase 3 (closes loop) | Phase 5 (clean up) | YES |

**Result: All 16 roles correctly placed. Von Braun correctly excluded from main loop diagram.**

### Agent Persona File Count

15 files in `loop/agents/` (no Deming file -- he is team lead defined in
CLAUDE.md). Spec correctly states line 83 of commands.ts ("15 agent personas")
is accurate and should NOT change. Verified.

### Slash Commands Accuracy

| Command | Spec Description | Source File Description | Match? |
|---------|-----------------|----------------------|--------|
| /backlog | "Capture requirements into .tasks/backlog/ -- run this first" | "Create a task file in .tasks/backlog/" | YES (spec adds helpful "run this first" guidance) |
| /loop | "Full 16-agent development cycle" | "Execute the full 16-agent great.sh Loop" | YES |
| /bugfix | "Targeted fix: reproduce, patch, verify, commit" | "reproduce + map -> fix + verify -> commit" | YES (accurate summarization) |
| /deploy | "Build and verify release artifacts" | "build release + verify -> smoke test" | YES |
| /discover | "UX discovery sweep -- Nielsen maps journeys, Nightingale files issues" | "Nielsen (full exploratory sweep) -> Nightingale (triage to backlog)" | YES |

**Result: All 5 commands accurately described.**

### Methodology Strings vs Agent Files

Spot-checked all 16 methodology strings against agent persona files:

- Nightingale: "Transforms chaos into organized task files with statistical discipline" -- matches `nightingale.md` description closely ("Transforms chaos into organized task files"). The "statistical discipline" comes from the persona's historical context (Nightingale's statistical work). **Accurate.**
- Wirth: "Measures artifact size, flags regressions" -- matches `wirth.md` checks #1 and the agent description. **Accurate.**
- Dijkstra: "Structured programming principles -- reviews quality, complexity, structure" -- matches `dijkstra.md` opening and review criteria. **Accurate.**
- Kerckhoffs: "Audits credentials, permissions, input validation, supply chain" -- matches `kerckhoffs.md` audit checklist items 1-4. **Accurate.**
- All other methodology strings verified as faithful summaries of source agent files.

### CSS/Tailwind Classes

All CSS classes used in spec JSX are valid per `site/tailwind.config.ts`:
- `bg-bg-secondary` -> colors.bg.secondary (#111111)
- `text-text-primary` -> colors.text.primary (#e8e8e8)
- `text-text-tertiary` -> colors.text.tertiary (#555555)
- `border-border` -> colors.border (#222222)
- `text-accent` -> colors.accent.DEFAULT (#22c55e)
- `bg-accent-muted` -> colors.accent.muted (rgba)
- `font-display` -> fontFamily.display (Space Grotesk)
- `font-mono` -> fontFamily.mono (JetBrains Mono)
- `max-w-[220px]` -> Tailwind JIT arbitrary value (valid)

**Result: All CSS classes valid.**

### TypeScript Feasibility

The spec adds `methodology` and optional `note` fields to inline `const`
declarations. TypeScript will infer the types automatically from the const
literal. The JSX accesses `agent.methodology` and `phase.note` which will be
correctly typed. No explicit interface declaration is needed (the spec provides
one for documentation purposes but notes it can remain inline).

**Result: TypeScript will compile without errors.**

### Acceptance Criteria Coverage

| Backlog Criterion | Spec Change | Covered? |
|------------------|-------------|----------|
| Phase diagram correct roster and order | Change 1 | YES |
| Von Braun NOT in main loop | Change 1 | YES |
| /backlog added to slash commands | Change 2 | YES |
| /discover description corrected | Change 2 | YES |
| Methodology descriptions on agent cards | Changes 1 (data + JSX) | YES |
| "16 roles" consistently used | Changes 3, 4, 5, 6 | YES |
| HowItWorks step 5 two-step workflow | Change 4 | YES |

**Result: All acceptance criteria have corresponding spec changes.**

---

## Verdict Rationale

This is a well-crafted spec for what is fundamentally a content accuracy task.
Every change is traceable to a specific inaccuracy identified in the backlog
item, and every replacement string has been cross-checked against the canonical
source files. The agent roster, ordering, methodology descriptions, and slash
command descriptions are all accurate.

The single WARNING (line 84 "4 commands" -> "5 commands") is a real
inconsistency that should be addressed before or during implementation, but it
is small enough that a builder could fix it inline without a spec revision.

The two NOTEs are documentation quality improvements, not correctness issues.

**APPROVED WITH NOTES** -- the builder should update `commands.ts` line 84 from
"4 commands" to "5 commands" as part of this task, even though the spec does not
explicitly call for it.
