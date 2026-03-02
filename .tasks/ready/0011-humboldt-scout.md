# 0011: Humboldt Scout Report — Update Site for great loop

**Scout:** Humboldt (Codebase Scout)
**Date:** 2026-02-20
**Spec:** `.tasks/ready/0011-update-site-great-loop-spec.md`
**Socrates review:** `.tasks/ready/0011-socrates-review.md` — PASS, non-blocking

---

## 1. Build Command

```bash
# From repo root (preferred — uses workspace filter):
pnpm build:site

# Root package.json (line 8) delegates to:
pnpm --filter great-sh build

# Which runs inside site/ (site/package.json line 9):
tsc -b && vite build
```

Do NOT run `pnpm build` from inside `site/` directly — use the workspace command
from root, or `cd site && pnpm build`.

---

## 2. File-by-File Change Map

### 2.1 FILES TO MODIFY

#### `/home/isaac/src/sh.great/site/src/data/commands.ts`
**Lines:** 1–82. Pure TypeScript data file, no imports, no JSX.

Two changes:
- **Line 32** — append one line to `initWizardOutput` before the closing backtick:
  ```
    Run \`great loop install\` to add the 13-agent team.`
  ```
  Current line 32: `  Run \`claude\` to start Claude Code with all MCP servers.\``
  New lines 32–33:
  ```
    Run \`claude\` to start Claude Code with all MCP servers.
    Run \`great loop install\` to add the 13-agent team.`
  ```

- **After line 82** (after closing backtick of `sampleToml`) — add `loopInstallOutput` export.
  Full content verbatim from spec section 4.3.2.
  Note: `[check]` is correct — matches existing pattern on lines 25–28.

#### `/home/isaac/src/sh.great/site/src/data/features.ts`
**Lines:** 1–33. Interface + array, no imports.

- **Lines 15–18** — replace `description` of item at index 1 (title: 'AI Agent Orchestration').
  Current: `'Claude Code as orchestrator with Codex, Gemini...'`
  New: `'The great.sh Loop: 13 specialized AI agents installed into Claude Code with one command...'`
  The `icon: 'brain'` field stays. The `Feature` interface is unchanged.

#### `/home/isaac/src/sh.great/site/src/data/comparison.ts`
**Lines:** 1–93. Interface + array.

- Insert new row at index 3 (after the `'AI CLI tool installation'` row at lines 31–38,
  before `'MCP server management'` at lines 39–47).
  New row:
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
  All values are `boolean`, which is valid per the `ComparisonRow` interface
  (`boolean | string` on all fields). No interface change.

#### `/home/isaac/src/sh.great/site/src/components/sections/HowItWorks.tsx`
**Lines:** 1–87.

Two changes:
- **Line 39** — heading text: `"Zero to hero in four steps"` → `"Zero to hero in five steps"`
- **Lines 7–32** — append step 05 to the `steps` array after step 04:
  ```typescript
  {
    number: '05',
    title: 'Start the Loop',
    description: 'Install the 13-agent team into Claude Code. Type /loop and describe your task.',
    command: 'great loop install --project',
  },
  ```
  The step object shape matches exactly: `number`, `title`, `description`, `command`.

#### `/home/isaac/src/sh.great/site/src/components/sections/Hero.tsx`
**Lines:** 1–99.

One change:
- **Line 50** — sub-tagline text only:
  `"One command. Fully configured. Cloud-synced. Open source."`
  → `"One command. 13 AI agents. Fully configured. Open source."`
  The wrapping `<motion.p>` and all classNames are unchanged.

#### `/home/isaac/src/sh.great/site/src/components/layout/Nav.tsx`
**Lines:** 1–94.

One change:
- **Lines 5–11** — insert `{ label: 'Loop', href: '#loop' }` after `'How it Works'`
  and before `'Templates'`:
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
  Mobile menu (lines 67–90) reads the same `navLinks` array — no additional change.

#### `/home/isaac/src/sh.great/site/src/components/layout/Footer.tsx`
**Lines:** 1–52.

One change:
- **Lines 28–48** — replace the `architecton.ai` div block.
  Current (lines 28–48): `mt-6` div with Superstruct + architecton.ai links.
  New (lines 28–33): `mt-6` div with Superstruct link only.
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

#### `/home/isaac/src/sh.great/site/src/App.tsx`
**Lines:** 1–27. Last file to edit (step 9 in spec build order).

Two changes:
- **After line 6** — add import: `import { Loop } from '@/components/sections/Loop'`
- **After line 19** (`<HowItWorks />`) — add: `<Loop />`

### 2.2 FILE TO CREATE

#### `/home/isaac/src/sh.great/site/src/components/sections/Loop.tsx`
New file. Full implementation provided verbatim in spec section 5.1.
Create before editing App.tsx (step 4 in build order).

---

## 3. Exact Agent List from `loop_cmd.rs`

Source of truth: `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`, lines 41–94.
Unit test at line 469 asserts all 13 are present.

**13 agents (AGENTS array, in order):**
1. nightingale
2. lovelace
3. socrates
4. humboldt
5. davinci
6. vonbraun  ← PRESENT IN SOURCE, ABSENT FROM SPEC'S Loop.tsx phases
7. turing
8. kerckhoffs
9. rams
10. nielsen
11. knuth
12. gutenberg
13. hopper

**14th role:** Deming — team lead/observer, not in AGENTS array, not installed as an agent file.
Line 259 of loop_cmd.rs confirms: `"14 roles: 4 teammates + 9 subagents + 1 team lead"`

---

## 4. Exact Slash Commands from `loop_cmd.rs`

Source of truth: `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`, lines 97–114.
Unit test at line 401 asserts count == 4.

**4 commands (COMMANDS array, in order):**
1. `loop` — full development cycle
2. `bugfix` — diagnose and fix a bug
3. `deploy` — build, test, and ship
4. `discover` — explore and document a codebase

These match the `slashCommands` array in the spec's Loop.tsx exactly.

---

## 5. Von Braun Resolution (Socrates NB-2)

Socrates flagged that Von Braun (`vonbraun` in AGENTS) is missing from the
spec's Loop.tsx phase diagram. The spec's phases list 4+4+5 = 13 names total,
but one of those is Deming (team lead, not an installed agent). Von Braun
(deploy) is therefore missing, leaving only 12 of 13 installed agents shown.

**Recommended resolution for Da Vinci:** Add Von Braun to Phase 2 (Parallel Team)
between Da Vinci and Turing. This matches the CLAUDE.md loop description:
`"Da Vinci (build) → parallel: [Von Braun (deploy) → Turing (test)...]"`.
The phase 2 block would become 5 agents: Da Vinci, Von Braun, Turing, Kerckhoffs, Nielsen.

Revised Phase 2:
```typescript
{
  label: 'Phase 2 -- Parallel Team',
  agents: [
    { name: 'Da Vinci', role: 'Build' },
    { name: 'Von Braun', role: 'Deploy' },
    { name: 'Turing', role: 'Test' },
    { name: 'Kerckhoffs', role: 'Security' },
    { name: 'Nielsen', role: 'UX' },
  ],
  flow: 'parallel' as const,
},
```

Total with this fix: 4 (phase 1) + 5 (phase 2) + 5 (phase 3) = 14 names shown,
matching `"14 roles"` in `loopInstallOutput`. The "14 roles" claim becomes
accurate: 13 installed agents + 1 team lead (Deming).

This is the recommended approach. The spec's phase diagram should be updated
in Loop.tsx when the file is created.

---

## 6. Component Patterns to Follow

### Import pattern (all section components follow this exactly)
```tsx
import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
// optional shared:
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { CodeBlock } from '@/components/shared/CodeBlock'
// data:
import { someExport } from '@/data/someFile'
// motion:
import { motion } from 'motion/react'
// lucide icons if needed:
import { SomeIcon } from 'lucide-react'
```

### Export pattern
All section components use named exports: `export function ComponentName() { ... }`
No default exports anywhere in the codebase.

### Section wrapper pattern
```tsx
<AnimatedSection id="section-id">
  <Container>
    {/* content */}
  </Container>
</AnimatedSection>
```
`AnimatedSection` props: `id` (required string), `className` (optional).
`Container` has no required props — provides `max-w-site px-6 md:px-12 mx-auto`.

### Motion animation pattern
```tsx
<motion.div
  initial={{ opacity: 0, y: 20 }}
  whileInView={{ opacity: 1, y: 0 }}
  viewport={{ once: true, margin: '-50px' }}
  transition={{ duration: 0.4, delay: i * 0.1 }}
>
```
`whileInView` with `once: true` is the universal pattern. `animate` (not
`whileInView`) is used only in Hero.tsx because it is always visible on load.

### Card styling pattern (Features, agent cards)
```
bg-bg-secondary border border-border rounded-xl p-8 hover:border-accent/30 transition-colors
```
Agent cards in Loop.tsx use `rounded-lg px-4 py-2.5` (smaller) — spec is correct.

### Heading pattern
```tsx
<h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
```
Sub-heading h3: `font-display text-xl text-text-primary mb-6`

### TerminalWindow usage
```tsx
<TerminalWindow title="command string">
  <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
    {dataExport}
  </pre>
</TerminalWindow>
```
The `pre` inside TerminalWindow is the universal pattern (HowItWorks line 77–79).

### Code/accent badge pattern (Config.tsx)
```tsx
<code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
  {text}
</code>
```

---

## 7. Dependency Map

```
Loop.tsx
  ├── @/components/shared/AnimatedSection   (exists, no change)
  ├── @/components/layout/Container         (exists, no change)
  ├── @/components/shared/TerminalWindow    (exists, no change)
  ├── @/data/commands → loopInstallOutput   (NEW export, must exist before Loop.tsx is imported)
  └── motion/react                          (installed: motion ^12.12.1)

App.tsx
  └── @/components/sections/Loop            (NEW file, must exist before App.tsx edit)

HowItWorks.tsx
  └── @/data/commands → initWizardOutput    (existing export, content extended)

Features.tsx
  └── @/data/features → features            (existing export, description updated)

Comparison.tsx
  └── @/data/comparison → comparisonData    (existing export, one row added)

Nav.tsx      — self-contained, no data imports
Footer.tsx   — self-contained, no data imports
Hero.tsx     — @/data/commands → installCommand (unchanged)
```

---

## 8. TypeScript Constraints (tsconfig.json)

- `strict: true` — all types enforced
- `noUnusedLocals: true` — any imported but unused symbol is a TS error
- `noUnusedParameters: true` — same for function params
- `paths: { "@/*": ["./src/*"] }` — alias confirmed for all imports
- `verbatimModuleSyntax: true` — type-only imports must use `import type`

**Gotcha:** `noUnusedLocals` means every import in Loop.tsx must be used.
The spec's Loop.tsx imports `motion` and uses it — confirmed safe.

---

## 9. Risks and Gotchas

### R1 — Von Braun omission (moderate)
The spec's Loop.tsx phase diagram omits Von Braun from all phases. If built
exactly as spec'd, the "14 roles" text in `loopInstallOutput` and the visual
count will not reconcile (13 names shown, 14 claimed). See section 5 above for
the fix: add Von Braun to Phase 2 as `{ name: 'Von Braun', role: 'Deploy' }`.

### R2 — `loopInstallOutput` omits the `--project` branch detail (low)
The spec's terminal mock shows the happy-path `--project` output including
`.tasks/ created, .gitignore updated`. The actual CLI only prints that line
when `project == true` AND after a second `output::header` block. The mock
is an aspirational simplification — acceptable for marketing. No action needed.

### R3 — `pnpm build:site` must be run from repo root (low)
The build script `pnpm build:site` is defined in the ROOT `package.json`
(line 8), not in `site/package.json`. Running `pnpm build:site` from inside
`site/` will fail with "missing script". Run from `/home/isaac/src/sh.great/`.

### R4 — Comparison row insertion position
Spec says insert at index 3 (after 'AI CLI tool installation', before 'MCP
server management'). Current `comparison.ts` has 'AI CLI tool installation'
at lines 31–38 and 'MCP server management' at lines 39–47. The insertion point
is exact — insert after the closing `},` on line 38.

### R5 — Footer line reference
Spec references "lines 28–48" for the architecton block. Confirmed: line 28
begins `<div className="mt-6 text-center text-text-tertiary text-xs">` and
line 48 closes with `</div>`. The replacement is lines 28–48 inclusive.

---

## 10. Recommended Build Order

Per spec section 6, verified against dependency map:

1. `site/src/data/commands.ts` — add `loopInstallOutput`, extend `initWizardOutput`
2. `site/src/data/features.ts` — update card index 1 description
3. `site/src/data/comparison.ts` — insert row at index 3
4. `site/src/components/sections/Loop.tsx` — CREATE new file (with Von Braun fix)
5. `site/src/components/sections/HowItWorks.tsx` — add step 05, update heading
6. `site/src/components/sections/Hero.tsx` — update sub-tagline line 50
7. `site/src/components/layout/Nav.tsx` — add Loop link to navLinks
8. `site/src/components/layout/Footer.tsx` — remove architecton block
9. `site/src/App.tsx` — import Loop, insert `<Loop />` after `<HowItWorks />`
10. Run `pnpm build:site` from repo root — must exit 0

---

## 11. No Legacy Repo References

The `/home/isaac/src/great-sh` legacy repo was checked implicitly via the
CLAUDE.md instruction. No patterns from that repo are needed for this task —
all required patterns exist in the current codebase.
