# 0011: Update Marketing Site -- Promote great loop

**Status:** ready
**Priority:** P0
**Estimated Complexity:** M
**Author:** Lovelace (spec)
**Date:** 2026-02-20

---

## 1. Overview

The marketing site at `site/` predates the `great loop` feature. This spec details
exact changes to position `great loop` as the primary differentiator of great.sh.
The loop is a 13-agent AI orchestration methodology installed into Claude Code with
a single command. It is the feature no competitor has.

**What changes:**
- New dedicated Loop section component between HowItWorks and Templates
- Updated Features card copy for "AI Agent Orchestration"
- New step 05 in HowItWorks
- New comparison row for agent orchestration
- Updated `initWizardOutput` terminal mock
- New `loopInstallOutput` data export for the Loop section terminal block
- Nav gains "Loop" anchor link
- Footer removes `architecton.ai` reference
- Hero sub-tagline updated to reference the loop

**What does not change:** Templates data, Config section, sampleToml, Comparison component
logic, OpenSource section, Tailwind config, globals.css, shared components
(AnimatedSection, TerminalWindow, CodeBlock, Container).

---

## 2. Files to Modify

| File | Change type |
|------|-------------|
| `site/src/data/features.ts` | Edit card #2 copy |
| `site/src/data/commands.ts` | Append line to `initWizardOutput`; add `loopInstallOutput` export |
| `site/src/data/comparison.ts` | Add one row |
| `site/src/components/sections/HowItWorks.tsx` | Add step 05; update heading |
| `site/src/components/sections/Hero.tsx` | Update sub-tagline |
| `site/src/components/layout/Nav.tsx` | Add "Loop" link |
| `site/src/components/layout/Footer.tsx` | Remove architecton.ai block |
| `site/src/App.tsx` | Import and insert `<Loop />` |

## 3. Files to Create

| File | Purpose |
|------|---------|
| `site/src/components/sections/Loop.tsx` | New dedicated Loop section |

---

## 4. Exact Content Changes

### 4.1 Hero Sub-tagline (`Hero.tsx`)

**Current** (line 50):
```
One command. Fully configured. Cloud-synced. Open source.
```

**New:**
```
One command. 13 AI agents. Fully configured. Open source.
```

This is the only change to Hero.tsx. The primary tagline ("The managed AI dev environment.") stays.

---

### 4.2 Features Card Update (`features.ts`)

Replace the second item in the `features` array (index 1, "AI Agent Orchestration"):

```typescript
{
  title: 'AI Agent Orchestration',
  description:
    'The great.sh Loop: 13 specialized AI agents installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, UX reviews, docs, and deploys — orchestrated as a team. Run great loop install to set it up.',
  icon: 'brain',
},
```

No changes to the other three feature cards. No changes to the `Feature` interface.

---

### 4.3 Commands Data (`commands.ts`)

**4.3.1 Update `initWizardOutput`**

Replace the final line of the template literal. Current ending:

```
  Run \`claude\` to start Claude Code with all MCP servers.`
```

New ending:

```
  Run \`claude\` to start Claude Code with all MCP servers.
  Run \`great loop install\` to add the 13-agent team.`
```

**4.3.2 Add `loopInstallOutput` export**

Add this new export after `sampleToml`:

```typescript
export const loopInstallOutput = `$ great loop install --project

  great.sh Loop -- Installing agent team

  [check] 13 agent personas -> ~/.claude/agents/
  [check] 4 commands -> ~/.claude/commands/
  [check] Agent Teams config -> ~/.claude/teams/loop/
  [check] Settings with Agent Teams enabled -> ~/.claude/settings.json
  [check] .tasks/ created, .gitignore updated

  great.sh Loop installed!

  14 roles: 4 teammates + 9 subagents + 1 team lead
  Usage: claude -> /loop [task description]`
```

Note: Use `[check]` as a plain text stand-in for the checkmark character, matching
the pattern already used in `initWizardOutput`. The terminal CSS renders this in
`text-text-secondary` which gives the right muted look.

---

### 4.4 Comparison Data (`comparison.ts`)

Add one new row to the `comparisonData` array. Insert it at index 3 (after
"AI CLI tool installation", before "MCP server management") to group AI features
together:

```typescript
{
  feature: 'AI agent orchestration loop',
  great: true,
  chezmoi: false,
  mise: false,
  nix: false,
  mcpm: false,
  manual: false,
},
```

No changes to the `ComparisonRow` interface. No changes to `Comparison.tsx`.

---

### 4.5 HowItWorks Update (`HowItWorks.tsx`)

**4.5.1 Update the heading**

Current:
```
Zero to hero in four steps
```

New:
```
Zero to hero in five steps
```

**4.5.2 Add step 05 to the `steps` array**

Append after the existing step 04:

```typescript
{
  number: '05',
  title: 'Start the Loop',
  description: 'Install the 13-agent team into Claude Code. Type /loop and describe your task.',
  command: 'great loop install --project',
},
```

No other changes to HowItWorks.tsx. The terminal mock on the right already pulls
from `initWizardOutput` which will show the updated text from section 4.3.1.

---

### 4.6 Nav Update (`Nav.tsx`)

Add one entry to the `navLinks` array. Insert it after "How it Works" and before
"Templates":

```typescript
const navLinks = [
  { label: 'Features', href: '#features' },
  { label: 'Config', href: '#config' },
  { label: 'How it Works', href: '#how-it-works' },
  { label: 'Loop', href: '#loop' },
  { label: 'Templates', href: '#templates' },
  { label: 'Compare', href: '#compare' },
]
```

No other changes to Nav.tsx. The mobile menu reads from the same array.

---

### 4.7 Footer Update (`Footer.tsx`)

Remove the `architecton.ai` reference block. Replace lines 28-48 (the `<div className="mt-6 ...">` block):

**Current:**
```tsx
<div className="mt-6 text-center text-text-tertiary text-xs">
  Built by{' '}
  <a
    href="https://superstruct.nz"
    target="_blank"
    rel="noopener noreferrer"
    className="hover:text-text-secondary transition-colors"
  >
    Superstruct
  </a>
  {' '}&middot; Part of the{' '}
  <a
    href="https://architecton.ai"
    target="_blank"
    rel="noopener noreferrer"
    className="hover:text-text-secondary transition-colors"
  >
    architecton.ai
  </a>
  {' '}ecosystem
</div>
```

**New:**
```tsx
<div className="mt-6 text-center text-text-tertiary text-xs">
  Built by{' '}
  <a
    href="https://superstruct.nz"
    target="_blank"
    rel="noopener noreferrer"
    className="hover:text-text-secondary transition-colors"
  >
    Superstruct
  </a>
</div>
```

---

### 4.8 App.tsx Update

Add import and insert `<Loop />` between `<HowItWorks />` and `<Templates />`:

```tsx
import { Nav } from '@/components/layout/Nav'
import { Footer } from '@/components/layout/Footer'
import { Hero } from '@/components/sections/Hero'
import { Features } from '@/components/sections/Features'
import { Config } from '@/components/sections/Config'
import { HowItWorks } from '@/components/sections/HowItWorks'
import { Loop } from '@/components/sections/Loop'
import { Templates } from '@/components/sections/Templates'
import { Comparison } from '@/components/sections/Comparison'
import { OpenSource } from '@/components/sections/OpenSource'

export function App() {
  return (
    <div className="min-h-screen bg-bg-primary">
      <Nav />
      <main>
        <Hero />
        <Features />
        <Config />
        <HowItWorks />
        <Loop />
        <Templates />
        <Comparison />
        <OpenSource />
      </main>
      <Footer />
    </div>
  )
}
```

---

## 5. New Component: Loop.tsx

**File:** `site/src/components/sections/Loop.tsx`

This section is the centrepiece. It has four visual zones stacked vertically:
1. Heading and introductory paragraph
2. Agent flow diagram (the sequential-then-parallel execution model)
3. Terminal block showing `great loop install --project` output
4. Slash commands strip

### 5.1 Full Component Implementation

```tsx
import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { loopInstallOutput } from '@/data/commands'
import { motion } from 'motion/react'

const phases = [
  {
    label: 'Phase 1 -- Sequential',
    agents: [
      { name: 'Nightingale', role: 'Requirements' },
      { name: 'Lovelace', role: 'Spec' },
      { name: 'Socrates', role: 'Review' },
      { name: 'Humboldt', role: 'Scout' },
    ],
    flow: 'sequential' as const,
  },
  {
    label: 'Phase 2 -- Parallel Team',
    agents: [
      { name: 'Da Vinci', role: 'Build' },
      { name: 'Turing', role: 'Test' },
      { name: 'Kerckhoffs', role: 'Security' },
      { name: 'Nielsen', role: 'UX' },
    ],
    flow: 'parallel' as const,
  },
  {
    label: 'Phase 3 -- Finish',
    agents: [
      { name: 'Rams', role: 'Visual QA' },
      { name: 'Hopper', role: 'Commit' },
      { name: 'Knuth', role: 'Docs' },
      { name: 'Gutenberg', role: 'Doc Commit' },
      { name: 'Deming', role: 'Observe' },
    ],
    flow: 'sequential' as const,
  },
]

const slashCommands = [
  { cmd: '/loop', desc: 'Full development cycle' },
  { cmd: '/bugfix', desc: 'Diagnose and fix a bug' },
  { cmd: '/deploy', desc: 'Build, test, and ship' },
  { cmd: '/discover', desc: 'Explore and document a codebase' },
]

export function Loop() {
  return (
    <AnimatedSection id="loop">
      <Container>
        {/* Heading */}
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          great loop{' '}
          <span className="text-text-secondary">— 13 agents, one command</span>
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto leading-relaxed">
          The great.sh Loop is a 13-role AI agent orchestration methodology that
          ships inside every great.sh install. One command configures Claude Code
          with a full team: requirements analysts, spec writers, builders, testers,
          security auditors, UX reviewers, documenters, and an observer.
          Type{' '}
          <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
            /loop [task]
          </code>{' '}
          and the team goes to work.
        </p>

        {/* Agent flow */}
        <div className="space-y-8 mb-16">
          {phases.map((phase, phaseIdx) => (
            <motion.div
              key={phase.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.4, delay: phaseIdx * 0.15 }}
            >
              <div className="text-text-tertiary text-xs font-mono uppercase tracking-wider mb-3">
                {phase.label}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {phase.agents.map((agent, i) => (
                  <div key={agent.name} className="flex items-center gap-2">
                    <div className="bg-bg-secondary border border-border rounded-lg px-4 py-2.5 hover:border-accent/30 transition-colors">
                      <div className="font-display text-sm text-text-primary">
                        {agent.name}
                      </div>
                      <div className="text-text-tertiary text-xs">
                        {agent.role}
                      </div>
                    </div>
                    {i < phase.agents.length - 1 && (
                      <span className="text-text-tertiary text-sm font-mono">
                        {phase.flow === 'parallel' ? '+' : '\u2192'}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </motion.div>
          ))}
        </div>

        {/* Two-column: terminal + slash commands */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <TerminalWindow title="great loop install --project">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {loopInstallOutput}
              </pre>
            </TerminalWindow>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="flex flex-col justify-center"
          >
            <h3 className="font-display text-xl text-text-primary mb-6">
              Four slash commands
            </h3>
            <div className="space-y-4">
              {slashCommands.map((sc) => (
                <div key={sc.cmd} className="flex items-baseline gap-3">
                  <code className="text-accent text-sm bg-accent-muted px-2 py-1 rounded font-mono flex-shrink-0">
                    {sc.cmd}
                  </code>
                  <span className="text-text-secondary text-sm">{sc.desc}</span>
                </div>
              ))}
            </div>

            <div className="mt-8 space-y-2">
              <code className="block text-text-secondary text-xs font-mono">
                <span className="text-accent">$</span> great loop install{' '}
                <span className="text-text-tertiary"># global: adds agents to ~/.claude/</span>
              </code>
              <code className="block text-text-secondary text-xs font-mono">
                <span className="text-accent">$</span> great loop install --project{' '}
                <span className="text-text-tertiary"># also sets up .tasks/ kanban</span>
              </code>
            </div>
          </motion.div>
        </div>
      </Container>
    </AnimatedSection>
  )
}
```

### 5.2 Component Design Notes

- Uses `AnimatedSection` with `id="loop"` matching the nav anchor
- Uses `Container` for max-width consistency
- Uses `TerminalWindow` for the install output block, matching HowItWorks pattern
- Uses `motion` from `motion/react` for stagger animations, matching existing components
- Agent cards use `bg-bg-secondary border border-border rounded-lg` matching Features cards
- Phase labels use `text-text-tertiary text-xs font-mono uppercase tracking-wider`
  matching the category labels in Templates
- Sequential agents separated by `->` arrows, parallel agents by `+` signs
- All 13 agents plus Deming (observer, phase 3) are shown -- 14 names total,
  matching the "14 roles" in the install output
- Slash commands listed with accent-colored code badges matching the Config section
  code style
- Two-line command reference at the bottom provides both install variants

### 5.3 Responsive Behavior

- Agent flow cards wrap with `flex-wrap` on narrow viewports
- Two-column grid (`lg:grid-cols-2`) collapses to single-column on mobile
- Terminal block goes full-width on mobile, slash commands stack below
- All text remains readable at 320px viewport width (tested by existing `text-sm`
  and `text-xs` patterns)

---

## 6. Build Order

The builder should implement changes in this order to maintain a buildable state
at each step:

1. **`site/src/data/commands.ts`** -- Add `loopInstallOutput` export and update
   `initWizardOutput`. No component references this yet, so build stays green.

2. **`site/src/data/features.ts`** -- Update card #2 copy. Pure data change,
   existing Features component picks it up automatically.

3. **`site/src/data/comparison.ts`** -- Add new row. Existing Comparison component
   picks it up automatically.

4. **`site/src/components/sections/Loop.tsx`** -- Create the new component file.
   Not yet imported, so build stays green.

5. **`site/src/components/sections/HowItWorks.tsx`** -- Add step 05 and update heading.

6. **`site/src/components/sections/Hero.tsx`** -- Update sub-tagline.

7. **`site/src/components/layout/Nav.tsx`** -- Add "Loop" link.

8. **`site/src/components/layout/Footer.tsx`** -- Remove architecton.ai block.

9. **`site/src/App.tsx`** -- Import `Loop` and insert into layout. This is last
   because it wires the new component into the page.

10. **Verify:** `pnpm build:site` must pass with zero TypeScript errors.

---

## 7. Edge Cases

### 7.1 Mobile viewport (< 768px)
- Agent flow cards must wrap gracefully. The `flex-wrap` class handles this.
- Arrow/plus separators between agents remain inline with the cards.
- Nav "Loop" link appears in the mobile slide-out menu (already handled by
  reading from the shared `navLinks` array).

### 7.2 Many nav links (6 total after adding Loop)
- Desktop nav has 6 text links plus the GitHub icon. At 1200px max-width with
  `gap-8`, this fits comfortably (each link ~80px average = ~480px + gaps ~240px
  = ~720px, well within 1200px).
- Mobile menu is a vertical list, no overflow concern.

### 7.3 Comparison table horizontal scroll
- The table already has `overflow-x-auto`. Adding one row does not change
  horizontal dimensions.

### 7.4 HowItWorks step count mismatch
- The heading must say "five steps" (not "four"). Spec explicitly changes this.
- Terminal mock height may increase slightly with the extra line in
  `initWizardOutput`. The `whitespace-pre-wrap` and flexible height handle this.

### 7.5 prefers-reduced-motion
- All `motion` animations are already disabled by the global CSS rule in
  `globals.css` lines 61-71. The new Loop section uses `motion` the same way
  as existing sections, so no additional work needed.

---

## 8. Error Handling

This is a static marketing site with no runtime errors to handle. The only
failure mode is a TypeScript build error. The acceptance gate is:

```bash
cd site && pnpm build:site
```

Must exit 0 with no type errors. The builder should run this after step 9.

---

## 9. Security Considerations

- No user input is processed. All content is static.
- No API calls. No secrets. No authentication.
- External links (`superstruct.nz`, `github.com/great-sh/great`) use
  `rel="noopener noreferrer"` per existing pattern.
- The `architecton.ai` link removal eliminates a domain that may no longer be
  under project control.

---

## 10. Testing Strategy

### 10.1 Build verification
```bash
cd site && pnpm build:site
```
Must exit 0. This validates all TypeScript types, import paths, and JSX.

### 10.2 Visual verification (manual or Playwright)
- Load the site at `http://localhost:5173` (`pnpm dev` in `site/`)
- Verify the Loop section appears between HowItWorks and Templates
- Verify all 13 agents + Deming are visible in the flow diagram
- Verify the terminal block shows the `great loop install --project` output
- Verify the 4 slash commands are listed
- Verify the nav "Loop" link scrolls to the section
- Verify the footer no longer mentions architecton.ai
- Verify the hero sub-tagline reads "13 AI agents"
- Verify the comparison table has the new "AI agent orchestration loop" row
- Verify mobile layout (375px width): nav menu shows Loop, agent cards wrap

### 10.3 Accessibility
- Heading hierarchy: h2 for section title, h3 for "Four slash commands" sub-heading
- Code elements use `<code>` tags with monospace font
- All interactive elements (nav links) are standard `<a>` tags
- Color contrast: accent green (#22c55e) on dark backgrounds meets WCAG AA for
  large text. Body text (#888888 on #0a0a0a) follows existing site patterns.

---

## 11. Out of Scope

- Adding a `[loop]` section to `sampleToml` in commands.ts (P2)
- Marking templates as "loop-ready" in templates.ts (P2)
- A dedicated /loop documentation page (P2 -- docs task)
- Any backend, analytics, or infrastructure changes
- Changes to the Rust CLI or loop agent markdown files
- Changes to Tailwind config, globals.css, or shared components
- Changing the install script
