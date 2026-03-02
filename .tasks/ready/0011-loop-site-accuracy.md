# Spec 0011: Fix Loop Documentation and Marketing Site Accuracy

**Task:** `.tasks/backlog/0011-loop-site-accuracy.md`
**Status:** ready
**Priority:** P1
**Type:** docs (site copy corrections, no backend changes)
**Files to modify:** 4 files, 0 files to create

---

## Summary

The marketing site misrepresents the great.sh Loop in six ways: wrong agent
phase ordering, missing agents (Wirth, Dijkstra), a missing slash command
(`/backlog`), thin agent descriptions, an incorrect agent count (15 vs 16),
and misleading "one command" workflow copy. This spec provides exact
replacement content for each affected file so a builder can make all changes
without consulting the loop source-of-truth files.

---

## Files to Modify

| # | File | Changes |
|---|------|---------|
| 1 | `site/src/components/sections/Loop.tsx` | Phase data, slash commands data, heading text, intro paragraph |
| 2 | `site/src/components/sections/HowItWorks.tsx` | Step 5 description, step count in heading |
| 3 | `site/src/data/features.ts` | Agent count in description string |
| 4 | `site/src/data/commands.ts` | Two "15" references in terminal output strings |

---

## Change 1: Loop.tsx -- Phase Agent Data (lines 7-40)

### TypeScript Interface

The existing inline type `{ name: string; role: string }` must be extended to
include a `methodology` field. This is the new shape for each agent entry:

```typescript
interface PhaseAgent {
  name: string       // Display name (e.g., "Nightingale")
  role: string       // Short role label (e.g., "Requirements")
  methodology: string // One-line methodology description
}

interface Phase {
  label: string
  agents: PhaseAgent[]
  flow: 'sequential' | 'parallel'
  note?: string      // Optional footnote shown below the phase (used for Wirth)
}
```

These types do not need to be exported or placed in a separate file. They can
remain as inline `const` declarations. The important thing is that every agent
object now has three fields: `name`, `role`, `methodology`.

### Exact Replacement Data

Replace the entire `const phases` array (lines 7-40) with:

```typescript
const phases = [
  {
    label: 'Phase 1 -- Sequential',
    agents: [
      { name: 'Nightingale', role: 'Requirements', methodology: 'Transforms chaos into organized task files with statistical discipline' },
      { name: 'Lovelace', role: 'Spec', methodology: 'Produces self-contained specs so precise a builder needs nothing else' },
      { name: 'Socrates', role: 'Review', methodology: 'Adversarial plan approval gate using structured elenchus' },
      { name: 'Humboldt', role: 'Scout', methodology: 'Maps codebase connections before anyone touches code' },
    ],
    flow: 'sequential' as const,
  },
  {
    label: 'Phase 2 -- Parallel Team',
    agents: [
      { name: 'Da Vinci', role: 'Build', methodology: 'Turns specs into working code, runs all quality gates' },
      { name: 'Turing', role: 'Test', methodology: 'Adversarial tester -- proves the build is broken' },
      { name: 'Kerckhoffs', role: 'Security', methodology: 'Audits credentials, permissions, input validation, supply chain' },
      { name: 'Nielsen', role: 'UX', methodology: '10 Usability Heuristics applied to every user journey' },
    ],
    flow: 'parallel' as const,
    note: 'Wirth (Performance Sentinel) runs in parallel -- measures artifact size, flags regressions',
  },
  {
    label: 'Phase 3 -- Gate + Finish',
    agents: [
      { name: 'Dijkstra', role: 'Code Review', methodology: 'Structured programming principles -- reviews quality, complexity, structure' },
      { name: 'Rams', role: 'Visual Review', methodology: '10 Principles of Good Design applied to output aesthetics' },
      { name: 'Hopper', role: 'Commit', methodology: 'Never commits a broken build -- all gates must pass' },
      { name: 'Knuth', role: 'Docs', methodology: 'Every code example must work -- docs and release notes' },
      { name: 'Gutenberg', role: 'Doc Commit', methodology: 'Commits documentation independently of code' },
      { name: 'Deming', role: 'Observe', methodology: 'PDCA cycle -- observer report, one config change if needed' },
    ],
    flow: 'sequential' as const,
  },
]
```

**Key differences from current:**
- Phase 2: Von Braun REMOVED (he belongs to `/deploy` only). Wirth added as
  a `note` on Phase 2 (he is a parallel subagent, not a teammate).
- Phase 3: Dijkstra ADDED before Rams. Hopper moved AFTER Dijkstra and Rams
  (was incorrectly second). Label changed from "Finish" to "Gate + Finish".
- Every agent now has a `methodology` string.

### Rendering the `methodology` field

In the JSX agent card (currently lines 86-93), add the methodology text as a
third line. Replace:

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

With:

```tsx
<div className="bg-bg-secondary border border-border rounded-lg px-4 py-2.5 hover:border-accent/30 transition-colors">
  <div className="font-display text-sm text-text-primary">
    {agent.name}
  </div>
  <div className="text-text-tertiary text-xs">
    {agent.role}
  </div>
  <div className="text-text-tertiary text-xs mt-0.5 max-w-[220px]">
    {agent.methodology}
  </div>
</div>
```

The `max-w-[220px]` prevents cards from becoming excessively wide on desktop
while allowing the text to wrap naturally. At 375px mobile, the cards already
stack vertically via `flex-wrap`, so no additional mobile breakpoint is needed.

### Rendering the `note` field on Phase 2

After the agent cards `div` (the `<div className="flex flex-wrap ...">` block),
add a conditional note renderer. Insert immediately after the closing `</div>`
of the flex-wrap container (after line 100 in current code) and before the
closing `</motion.div>`:

```tsx
{phase.note && (
  <div className="text-text-tertiary text-xs font-mono mt-2 pl-1">
    + {phase.note}
  </div>
)}
```

---

## Change 2: Loop.tsx -- Slash Commands Data (lines 42-47)

### Exact Replacement

Replace the entire `const slashCommands` array with:

```typescript
const slashCommands = [
  { cmd: '/backlog', desc: 'Capture requirements into .tasks/backlog/ -- run this first' },
  { cmd: '/loop', desc: 'Full 16-agent development cycle' },
  { cmd: '/bugfix', desc: 'Targeted fix: reproduce, patch, verify, commit' },
  { cmd: '/deploy', desc: 'Build and verify release artifacts' },
  { cmd: '/discover', desc: 'UX discovery sweep -- Nielsen maps journeys, Nightingale files issues' },
]
```

**Key differences from current:**
- `/backlog` ADDED as first entry with "run this first" guidance.
- `/loop` description updated to include "16-agent".
- `/bugfix` description made more specific (reproduce, patch, verify, commit).
- `/deploy` description corrected from "Build, test, and ship" to match actual
  command: "Build and verify release artifacts" (Von Braun builds + Turing
  smoke tests, no "ship" step).
- `/discover` description corrected from "Explore and document a codebase" to
  the actual purpose: UX discovery sweep by Nielsen, findings triaged by
  Nightingale.

### Heading Update

Replace the heading text (line 128-129):

**Before:**
```tsx
<h3 className="font-display text-xl text-text-primary mb-6">
  Four slash commands
</h3>
```

**After:**
```tsx
<h3 className="font-display text-xl text-text-primary mb-6">
  Five slash commands
</h3>
```

---

## Change 3: Loop.tsx -- Heading and Intro Paragraph (lines 54-68)

### Heading (line 54-57)

**Before:**
```tsx
<h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
  great loop{' '}
  <span className="text-text-secondary">— 15 agents, one command</span>
</h2>
```

**After:**
```tsx
<h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
  great loop{' '}
  <span className="text-text-secondary">— 16 roles, two steps</span>
</h2>
```

### Intro Paragraph (lines 58-68)

**Before:**
```tsx
<p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto leading-relaxed">
  The great.sh Loop is a 15-role AI agent orchestration methodology that
  ships inside every great.sh install. One command configures Claude Code
  with a full team: requirements analysts, spec writers, builders, testers,
  security auditors, UX reviewers, documenters, and an observer.
  Type{' '}
  <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
    /loop [task]
  </code>{' '}
  and the team goes to work.
</p>
```

**After:**
```tsx
<p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto leading-relaxed">
  The great.sh Loop is a 16-role AI agent orchestration methodology that
  ships inside every great.sh install. One command configures Claude Code
  with a full team: requirements analysts, spec writers, builders, testers,
  security auditors, performance sentinels, code reviewers, UX inspectors,
  visual reviewers, documenters, and an observer. Type{' '}
  <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
    /backlog
  </code>{' '}
  to capture requirements, then{' '}
  <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
    /loop
  </code>{' '}
  to execute.
</p>
```

**Key differences:**
- "15-role" changed to "16-role".
- Role list expanded to include "performance sentinels, code reviewers" (Wirth,
  Dijkstra) and "visual reviewers" (Rams), "UX inspectors" (Nielsen distinction).
- "Type `/loop [task]`" replaced with two-step workflow:
  "`/backlog` to capture requirements, then `/loop` to execute."

---

## Change 4: HowItWorks.tsx -- Step 5 (lines 32-37)

### Option: Revise Step 5 description (keep 5 steps)

The heading says "Zero to hero in five steps." To avoid renumbering and
restructuring the layout, revise step 5 in place to communicate the two-step
workflow.

**Before:**
```typescript
{
  number: '05',
  title: 'Start the Loop',
  description: 'Install the 15-agent team into Claude Code. Type /loop and describe your task.',
  command: 'great loop install --project',
},
```

**After:**
```typescript
{
  number: '05',
  title: 'Start the Loop',
  description: 'Install the 16-role agent team into Claude Code. Type /backlog to capture requirements, then /loop to build.',
  command: 'great loop install --project',
},
```

**Key differences:**
- "15-agent" changed to "16-role".
- Workflow changed from "Type /loop and describe your task" to the two-step
  workflow: "/backlog to capture requirements, then /loop to build."

---

## Change 5: features.ts -- Agent Count (line 17)

### Exact Replacement

**Before (line 17):**
```typescript
'The great.sh Loop: 15 specialized AI agents installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, UX reviews, docs, and deploys — orchestrated as a team. Run great loop install to set it up.',
```

**After:**
```typescript
'The great.sh Loop: 16 specialized AI roles installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, performance checks, code reviews, UX inspections, visual reviews, docs, and deploys — orchestrated as a team. Run great loop install to set it up.',
```

**Key differences:**
- "15 specialized AI agents" changed to "16 specialized AI roles".
- List expanded to include "performance checks" (Wirth), "code reviews"
  (Dijkstra), "UX inspections" (Nielsen), and "visual reviews" (Rams) for
  completeness.

---

## Change 6: commands.ts -- Terminal Output Strings (lines 33, 83)

These are terminal output simulation strings shown in the marketing site.
They must be consistent with the 16-role count.

### Line 83

**Before:**
```typescript
  [check] 15 agent personas -> ~/.claude/agents/
```

**After:**
```typescript
  [check] 15 agent personas -> ~/.claude/agents/
```

**No change.** This line is technically correct: there ARE 15 agent persona
files in `loop/agents/` (Deming is the team lead, not an agent persona file).
The distinction between "15 persona files" and "16 roles" is accurate.

### Line 84

**Before:**
```typescript
  [check] 4 commands -> ~/.claude/commands/
```

**After:**
```typescript
  [check] 5 commands -> ~/.claude/commands/
```

There are now 5 command files (backlog, loop, bugfix, deploy, discover).

### Line 33

**Before:**
```typescript
  Run \`great loop install\` to add the 15-agent team.`
```

**After:**
```typescript
  Run \`great loop install\` to add the 16-role agent team.`
```

This aligns the `great init` wizard output with the canonical "16 roles"
terminology used on line 91 of the same file.

---

## Implementation Build Order

All changes are independent data/copy edits. No change depends on another.
A builder can make them in any order. Recommended order for efficient review:

1. **`site/src/data/features.ts`** -- Single string replacement (smallest change)
2. **`site/src/data/commands.ts`** -- Single string replacement
3. **`site/src/components/sections/HowItWorks.tsx`** -- Single object replacement
4. **`site/src/components/sections/Loop.tsx`** -- Largest change: data arrays,
   JSX template, heading, and paragraph

---

## Complete Agent Roster Reference

For the builder's reference, here is the canonical 16-role roster with the
exact methodology strings to use, derived from each agent's source file in
`loop/agents/`:

| # | Name | Role Label | Methodology String | Loop Phase | Type |
|---|------|-----------|-------------------|------------|------|
| 1 | Nightingale | Requirements | Transforms chaos into organized task files with statistical discipline | Phase 1 | Subagent |
| 2 | Lovelace | Spec | Produces self-contained specs so precise a builder needs nothing else | Phase 1 | Subagent |
| 3 | Socrates | Review | Adversarial plan approval gate using structured elenchus | Phase 1 | Subagent |
| 4 | Humboldt | Scout | Maps codebase connections before anyone touches code | Phase 1 | Subagent |
| 5 | Da Vinci | Build | Turns specs into working code, runs all quality gates | Phase 2 | Teammate |
| 6 | Turing | Test | Adversarial tester -- proves the build is broken | Phase 2 | Teammate |
| 7 | Kerckhoffs | Security | Audits credentials, permissions, input validation, supply chain | Phase 2 | Teammate |
| 8 | Nielsen | UX | 10 Usability Heuristics applied to every user journey | Phase 2 | Teammate |
| 9 | Wirth | Performance | Measures artifact size, flags regressions | Phase 2 (parallel subagent) | Subagent |
| 10 | Dijkstra | Code Review | Structured programming principles -- reviews quality, complexity, structure | Phase 3 | Subagent |
| 11 | Rams | Visual Review | 10 Principles of Good Design applied to output aesthetics | Phase 3 | Subagent |
| 12 | Hopper | Commit | Never commits a broken build -- all gates must pass | Phase 3 | Subagent |
| 13 | Knuth | Docs | Every code example must work -- docs and release notes | Phase 3 | Subagent |
| 14 | Gutenberg | Doc Commit | Commits documentation independently of code | Phase 3 | Subagent |
| 15 | Von Braun | Deploy | Builds and verifies release artifacts with abort procedures | /deploy only | Subagent |
| 16 | Deming | Observe | PDCA cycle -- observer report, one config change if needed | Phase 3 (closes loop) | Team Lead |

Note: Von Braun (#15) does NOT appear in the `/loop` phase diagram. He appears
only in the `/deploy` slash command. He is still one of the 16 roles.

---

## Complete Slash Commands Reference

| Command | Description (exact string for site) | Source File |
|---------|-------------------------------------|-------------|
| `/backlog` | Capture requirements into .tasks/backlog/ -- run this first | `loop/commands/backlog.md` |
| `/loop` | Full 16-agent development cycle | `loop/commands/loop.md` |
| `/bugfix` | Targeted fix: reproduce, patch, verify, commit | `loop/commands/bugfix.md` |
| `/deploy` | Build and verify release artifacts | `loop/commands/deploy.md` |
| `/discover` | UX discovery sweep -- Nielsen maps journeys, Nightingale files issues | `loop/commands/discover.md` |

---

## Edge Cases

### Mobile Layout (375px)

The agent cards use `flex-wrap` and will stack vertically on narrow screens.
The new `methodology` text (constrained to `max-w-[220px]`) will wrap to
2-3 lines per card. At 375px viewport:

- Phase 1 (4 agents): cards stack to ~2 columns, then wrap. Total height
  increases by approximately 60-80px due to methodology text.
- Phase 2 (4 agents + note): same stacking. The Wirth note line adds ~20px.
- Phase 3 (6 agents): was 5, now 6 agents. Cards stack similarly.

The builder should verify that no horizontal overflow occurs at 375px after
changes. The existing `flex-wrap` and `gap-2` spacing should handle this
without additional CSS.

### Card Width on Desktop

The `max-w-[220px]` on the methodology `div` prevents cards from growing
beyond ~252px total width (220px + 32px padding). With 6 agents in Phase 3,
the total row width is approximately 6 * 252px + 5 * 24px (gaps) = ~1632px.
On a standard `max-w-7xl` (1280px) container, Phase 3 will wrap to two rows.
This is acceptable and matches the existing behavior for 5 agents.

### Text Content

All strings use ASCII double-hyphens (`--`) rather than em dashes to match
the existing terminal-aesthetic style used throughout the site (e.g.,
`Phase 1 -- Sequential`). No curly quotes or Unicode dashes.

---

## Security Considerations

No security implications. All changes are static string content in the
marketing site. No user input, no API calls, no credential handling.

---

## Testing Strategy

### Manual Visual Testing

1. Run `pnpm dev` from the `site/` directory.
2. Verify the Loop section at desktop (1440px), tablet (768px), and mobile (375px):
   - Phase 1 shows 4 agents in correct order with methodology text.
   - Phase 2 shows 4 agents (no Von Braun) with Wirth note below.
   - Phase 3 shows 6 agents in correct order: Dijkstra, Rams, Hopper, Knuth,
     Gutenberg, Deming.
   - All agent cards display three lines: name, role, methodology.
   - No horizontal overflow at any breakpoint.
3. Verify slash commands section shows 5 commands with `/backlog` first.
4. Verify heading reads "16 roles, two steps".
5. Verify intro paragraph mentions `/backlog` then `/loop`.
6. Verify HowItWorks step 5 reads "16-role" and mentions `/backlog`.
7. Verify Features section reads "16 specialized AI roles".

### Build Verification

```bash
cd site && pnpm build:site
```

Must complete with zero TypeScript errors. The only structural change is
adding a `methodology` field and an optional `note` field to inline objects,
which TypeScript will infer automatically from the `const` declaration.

### Content Accuracy Checklist

- [ ] Agent count: "16" appears in Loop.tsx heading, intro, HowItWorks step 5, features.ts
- [ ] Agent count: "15" does NOT appear anywhere in the four modified files (except commands.ts line 83 which correctly refers to persona files)
- [ ] Phase 2: Von Braun is NOT present
- [ ] Phase 2: Wirth note is present
- [ ] Phase 3: Dijkstra is present and appears BEFORE Rams
- [ ] Phase 3: Hopper appears AFTER Dijkstra and Rams
- [ ] Slash commands: `/backlog` is listed first
- [ ] Slash commands: `/discover` description mentions "UX discovery sweep"
- [ ] Intro paragraph: mentions `/backlog` before `/loop`

---

## Out of Scope

- No changes to the actual loop agent files or command files.
- No changes to the Loop.tsx JSX structure beyond the agent card template and
  the note renderer -- layout, animation, and styling remain unchanged.
- The `commands.ts` line 83 (`15 agent personas`) is technically accurate
  (15 persona files exist; Deming is team lead, not a persona file) and is
  NOT changed.
- Mobile-specific CSS breakpoints are not added; the existing `flex-wrap`
  handles responsive layout.

---

## Diff Summary

| File | Lines Changed | Nature |
|------|--------------|--------|
| `site/src/components/sections/Loop.tsx` | ~60 lines | Data arrays rewritten, JSX card template extended, heading + paragraph rewritten |
| `site/src/components/sections/HowItWorks.tsx` | 2 lines | Step 5 description string |
| `site/src/data/features.ts` | 1 line | Description string in second feature |
| `site/src/data/commands.ts` | 2 lines | "15-agent" to "16-role agent" in init output, "4 commands" to "5 commands" in loop install output |
