# 0033 -- Bridge Section Component Spec

**Task:** Wire `mcpBridgeOutput` into a new Bridge section on the great.sh marketing site.
**Priority:** P2 | **Complexity:** S | **Type:** feature (site)
**Author:** Lovelace | **Date:** 2026-02-28

## Summary

Create a dedicated `Bridge` section component for the marketing site that showcases the
`great mcp-bridge` feature. The section uses the same two-column layout pattern as the
existing `Config` section (heading + bullets on the left, visual element on the right),
with a `TerminalWindow` rendering the already-exported `mcpBridgeOutput` string. Wire
the section into `App.tsx` between `<Loop />` and `<Templates />`, and add a
corresponding nav link.

This resolves the Dijkstra WARN from iteration 029 about the unused `mcpBridgeOutput`
export in `commands.ts`.

## Files to Create

### 1. `site/src/components/sections/Bridge.tsx` (new file)

```tsx
import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { mcpBridgeOutput } from '@/data/commands'
import { motion } from 'motion/react'

const bridgeFeatures = [
  {
    label: '5 backends',
    desc: 'Gemini, Codex, Claude, Grok, and Ollama -- any mix of cloud and local models.',
  },
  {
    label: '4 presets',
    desc: 'minimal (1 tool), agent (6), research (8), full (9) -- pick the surface area you need.',
  },
  {
    label: 'Zero Node.js',
    desc: 'Pure Rust, compiled into the great binary. No npx, no node_modules, no npm audit.',
  },
  {
    label: 'Auto-registered',
    desc: 'great apply writes the bridge entry into .mcp.json so Claude Code discovers it automatically.',
  },
]

export function Bridge() {
  return (
    <AnimatedSection id="bridge">
      <Container>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-start">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="flex flex-col justify-center"
          >
            <h2 className="font-display text-3xl md:text-4xl text-text-primary mb-4">
              great mcp-bridge{' '}
              <span className="text-text-secondary">
                — five backends, zero Node.js
              </span>
            </h2>
            <p className="text-text-secondary mb-6 leading-relaxed">
              A single MCP server that multiplexes five AI CLI backends over
              JSON-RPC 2.0 stdio. No sidecar processes, no JavaScript runtime
              — just one Rust binary speaking the Model Context Protocol.
            </p>
            <ul className="space-y-3 text-text-secondary text-sm">
              {bridgeFeatures.map((f) => (
                <li key={f.label} className="flex items-start gap-2">
                  <span className="text-accent mt-0.5">&#10003;</span>
                  <span>
                    <strong className="text-text-primary">{f.label}</strong>
                    {' -- '}
                    {f.desc}
                  </span>
                </li>
              ))}
            </ul>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <TerminalWindow title="great mcp-bridge --preset agent">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {mcpBridgeOutput}
              </pre>
            </TerminalWindow>
          </motion.div>
        </div>
      </Container>
    </AnimatedSection>
  )
}
```

## Files to Modify

### 2. `site/src/App.tsx`

**Change 1 -- Add import (after the Loop import, line 7):**

```diff
 import { Loop } from '@/components/sections/Loop'
+import { Bridge } from '@/components/sections/Bridge'
 import { Templates } from '@/components/sections/Templates'
```

**Change 2 -- Add section element (between `<Loop />` and `<Templates />`, line 21-22):**

```diff
         <Loop />
+        <Bridge />
         <Templates />
```

The resulting section order becomes:
`Hero -> Features -> Config -> HowItWorks -> Loop -> Bridge -> Templates -> Comparison -> OpenSource`

### 3. `site/src/components/layout/Nav.tsx`

**Change -- Add nav link (between Loop and Templates entries, line 9-10):**

```diff
   { label: 'Loop', href: '#loop' },
+  { label: 'Bridge', href: '#bridge' },
   { label: 'Templates', href: '#templates' },
```

This brings the total nav link count from 6 to 7. The mobile hamburger menu (collapsing
at the `md:` breakpoint) handles overflow gracefully, and 7 text links fit within the
desktop nav without horizontal scroll on standard viewport widths (1280px+).

## Interfaces

All interfaces used already exist. No new types are introduced.

### AnimatedSection (consumed, not modified)
```ts
interface AnimatedSectionProps {
  id: string
  children: ReactNode
  className?: string
}
```

### Container (consumed, not modified)
```ts
interface ContainerProps {
  children: ReactNode
  className?: string
}
```

### TerminalWindow (consumed, not modified)
```ts
interface TerminalWindowProps {
  title?: string       // renders in the title bar; defaults to "Terminal"
  children: ReactNode  // content rendered inside the terminal body
  className?: string
}
```

### mcpBridgeOutput (consumed, not modified)
```ts
// Exported from @/data/commands as a plain string constant.
export const mcpBridgeOutput: string
```

## Implementation Approach

### Build Order

1. Create `site/src/components/sections/Bridge.tsx` -- this can be done first since it
   only imports from existing modules (`AnimatedSection`, `Container`, `TerminalWindow`,
   `mcpBridgeOutput`, `motion`). No circular dependencies.

2. Edit `site/src/App.tsx` -- add the import and JSX element. This depends on step 1
   (the file must exist to import from).

3. Edit `site/src/components/layout/Nav.tsx` -- add the nav link entry. This is
   independent of steps 1-2 (no import needed, just a data array change).

Steps 2 and 3 can be done in parallel once step 1 is complete.

## Design Decisions

### Pattern: Config.tsx over Loop.tsx

The backlog task says "follow the exact structural pattern of Loop.tsx," but the Bridge
section is structurally much closer to `Config.tsx`:

- **Config.tsx**: Two-column grid (`lg:grid-cols-2`), left = heading + paragraph + bullet
  list, right = code/terminal visual. No phase data, no slash commands, no complex
  sub-layouts.
- **Loop.tsx**: Three-phase agent roster with flow arrows, *then* a two-column terminal +
  slash commands section below. Much more complex.

The Bridge spec uses the Config two-column pattern with Loop-style `motion.div` entrance
animations (staggered delays of 0.1s and 0.2s). This gives visual consistency with both
sections.

### Bullet list structure

Bullets use the same `<ul className="space-y-3">` + checkmark pattern as Config.tsx
(lines 21-42 of Config.tsx), but with the label bolded for scannability. The four
bullet points are derived from verified Rust source:

| Claim | Source |
|-------|--------|
| 5 backends: Gemini, Codex, Claude, Grok, Ollama | `src/mcp/bridge/backends.rs` lines 31-77: `BACKEND_SPECS` array with 5 entries |
| 4 presets: minimal, agent, research, full | `src/mcp/bridge/tools.rs` lines 131-141: `enum Preset` with 4 variants |
| Tool counts per preset (1, 6, 8, 9) | `src/mcp/bridge/tools.rs` lines 197-200: unit tests assert exact counts |
| Zero Node.js / pure Rust | The bridge is compiled into the `great` binary; no JS runtime dependency |
| Auto-registered by `great apply` | `.mcp.json` entry written during apply |

### Heading tone

The heading "great mcp-bridge -- five backends, zero Node.js" mirrors the Loop heading
pattern: `great loop -- 16 roles, two steps`. Both use the command name followed by a
dash and a short value proposition.

## Edge Cases

1. **mcpBridgeOutput is empty string**: Not possible -- it is a `const` string literal
   in `commands.ts` (lines 98-110). The `TerminalWindow` renders children inside a
   `<div>` so an empty pre tag would simply show nothing. No crash.

2. **Nav overflow on narrow desktop (768-1024px)**: At `md:` breakpoint (768px), the nav
   switches from inline links to hamburger menu. At 1024px+ with 7 links, each link is
   ~60-80px wide + 32px gap = ~700px total, well within the `max-w-site` container.
   No overflow.

3. **Mobile hamburger menu**: The existing `navLinks.map()` in the mobile dropdown
   (Nav.tsx lines 74-83) automatically includes any entries added to the `navLinks`
   array. Adding "Bridge" requires no additional mobile-specific code.

4. **Scroll-to-anchor**: The `id="bridge"` on the `AnimatedSection` enables
   `#bridge` anchor links. The `AnimatedSection` uses `motion.section` which renders
   a standard `<section id="bridge">` in the DOM.

5. **SSR / Vite SSG**: The site is a pure client-side SPA (Vite, no SSR). The `motion`
   library's `whileInView` uses IntersectionObserver, which is available in all target
   browsers. No SSR hydration mismatch concern.

## Error Handling

This is a static marketing page with no runtime errors to handle. The only failure mode
is a TypeScript compilation error, caught by:
- `pnpm --filter great-sh build` (runs `tsc --noEmit && vite build`)
- The CI workflow `build:site` step

If the import path `@/data/commands` or `@/components/sections/Bridge` is wrong,
TypeScript will emit a module-not-found error at build time. There are no runtime API
calls, no user input, and no network requests in this component.

## Security Considerations

None. This task adds a static React component with hardcoded display text. No user input
is processed. No API keys, secrets, or credentials are referenced. The `mcpBridgeOutput`
string contains only example terminal output with placeholder markers like
`(GEMINI_API_KEY set)`, not actual key values.

## Testing Strategy

### Build verification (required)
```bash
pnpm --filter great-sh build
```
Must exit 0 with zero TypeScript errors. This is the primary acceptance gate.

### Visual verification (manual)
```bash
pnpm --filter great-sh dev
```
Then navigate to `http://localhost:5173/#bridge` and verify:
- Section appears between Loop and Templates
- Two-column layout on desktop (lg breakpoint, 1024px+)
- Single-column stack on mobile (<1024px)
- Terminal window shows the `mcpBridgeOutput` content with correct formatting
- Four bullet points render with green checkmarks
- Nav link "Bridge" appears between "Loop" and "Templates"
- Clicking "Bridge" in nav scrolls to the section
- Mobile hamburger menu includes "Bridge" entry

### Automated tests
No new test files are needed. The site has no component-level test suite (no vitest,
no React Testing Library configured). The existing CI pipeline runs `pnpm build:site`
which catches type errors and broken imports.

### Rust tests
No Rust files are modified. `cargo test` count remains unchanged.

## Acceptance Criteria

- [ ] `site/src/components/sections/Bridge.tsx` exists and exports a `Bridge` function
      component
- [ ] `Bridge` imports `mcpBridgeOutput` from `@/data/commands` and renders it inside
      `<TerminalWindow title="great mcp-bridge --preset agent">`
- [ ] `site/src/App.tsx` imports `Bridge` from `@/components/sections/Bridge` and renders
      `<Bridge />` between `<Loop />` and `<Templates />`
- [ ] `site/src/components/layout/Nav.tsx` includes
      `{ label: 'Bridge', href: '#bridge' }` between the Loop and Templates entries
- [ ] `pnpm --filter great-sh build` exits 0
- [ ] The four feature bullets (5 backends, 4 presets, zero Node.js, auto-registered)
      are present and factually match `src/mcp/bridge/backends.rs` and
      `src/mcp/bridge/tools.rs`
- [ ] No new npm/pnpm dependencies added
- [ ] No Rust source files modified
- [ ] No changes to `commands.ts` or `features.ts`

## Appendix: Complete Diff Summary

Three files touched. Net additions: ~65 lines new code, ~4 lines modified.

| File | Action | Lines changed |
|------|--------|---------------|
| `site/src/components/sections/Bridge.tsx` | Create | ~65 lines (new file) |
| `site/src/App.tsx` | Modify | +2 lines (1 import, 1 JSX element) |
| `site/src/components/layout/Nav.tsx` | Modify | +1 line (1 nav link entry) |
