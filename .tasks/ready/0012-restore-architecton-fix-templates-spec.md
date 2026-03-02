# Spec 0012: Restore architecton.ai Footer Link + Fix Templates Section

**Task:** `.tasks/backlog/0012-restore-architecton-fix-templates.md`
**Status:** ready
**Estimated Complexity:** Small (3 files, copy/link changes only)

---

## Summary

Two corrections from iteration 001:

1. The architecton.ai ecosystem link was removed from the Footer in commit `a2e97a4`. It must be restored to its original form.
2. The Templates section presents 5 paid marketplace templates as if they ship free with the CLI. The section must clearly attribute them to architecton.ai and add a CTA link. The data file needs a `source` field so the component can display provenance.

The CLI bundles 4 built-in templates (`ai-fullstack-ts`, `ai-fullstack-py`, `ai-minimal`, `saas-multi-tenant`) via `include_str!` in `/home/isaac/src/sh.great/src/cli/template.rs`. The site data lists 5 templates that do not match the CLI builtins exactly (site has `ai-data-science` and `ai-devops` which are not in the repo; site omits `saas-multi-tenant` which is in the repo). This mismatch reinforces the need to frame the site templates as marketplace examples, not bundled offerings.

---

## Files to Modify

| File | Change |
|------|--------|
| `/home/isaac/src/sh.great/site/src/components/layout/Footer.tsx` | Restore the architecton.ai ecosystem link removed in `a2e97a4` |
| `/home/isaac/src/sh.great/site/src/data/templates.ts` | Add `source` field to `Template` interface and each entry |
| `/home/isaac/src/sh.great/site/src/components/sections/Templates.tsx` | Update heading/subheading copy, add source badge per card, add CTA link to architecton.ai |

No new files are created.

---

## Build Order

1. `templates.ts` -- add interface field and data first (no component depends on `source` yet, so safe)
2. `Templates.tsx` -- update component to consume new field and reframe copy
3. `Footer.tsx` -- restore the original architecton.ai link

---

## 1. Footer.tsx -- Restore architecton.ai Link

### What changed in a2e97a4 (the erroneous removal)

The following block was deleted from the `<div className="mt-6 text-center ...">` element, after the Superstruct link:

```tsx
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
```

### Exact change

Restore that block. The full `<div className="mt-6 ...">` element must read:

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

This is a verbatim restoration of the original code from commit `80bafbf`.

---

## 2. templates.ts -- Add Source Field

### Current interface

```ts
export interface Template {
  name: string
  id: string
  description: string
  agents: string[]
  mcpServers: string[]
  tools: string[]
}
```

### New interface

```ts
export interface Template {
  name: string
  id: string
  description: string
  agents: string[]
  mcpServers: string[]
  tools: string[]
  source: 'architecton.ai' | 'builtin'
}
```

The `source` field is a string literal union. All 5 existing templates get `source: 'architecton.ai'` because they represent marketplace offerings, not CLI builtins.

### Exact data changes

Add `source: 'architecton.ai'` as the last property in each of the 5 template objects. Example for the first entry:

```ts
  {
    name: 'AI Full Stack (TypeScript)',
    id: 'ai-fullstack-ts',
    description:
      'TypeScript full-stack development with Claude Code + Codex + Gemini, GitHub/Filesystem/Memory/Playwright MCP servers.',
    agents: ['Claude Code', 'Codex CLI', 'Gemini CLI'],
    mcpServers: ['GitHub', 'Filesystem', 'Memory', 'Playwright', 'Brave Search'],
    tools: ['Node 22', 'TypeScript', 'gh', 'Docker', 'Starship', 'fzf', 'ripgrep'],
    source: 'architecton.ai',
  },
```

Repeat for all 5 entries. No entries are added or removed.

---

## 3. Templates.tsx -- Reframe Section Copy and Add CTA

### 3a. Heading change

**Current:**
```tsx
<h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
  Start with a template
</h2>
```

**New:**
```tsx
<h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
  Template marketplace
</h2>
```

### 3b. Subheading change

**Current:**
```tsx
<p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
  Curated environment configs encoding best-practice AI dev setups. Use as-is or customize.
</p>
```

**New:**
```tsx
<p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
  Production-ready environment configs available via{' '}
  <a
    href="https://architecton.ai"
    target="_blank"
    rel="noopener noreferrer"
    className="text-accent hover:underline"
  >
    architecton.ai
  </a>
  . Install with <code className="text-accent font-mono text-sm">great template apply</code>.
</p>
```

### 3c. Source badge on each template card

Inside each `motion.div` card, immediately after the existing `<code>` tag that shows `template.id`, add a source badge. Replace the current `<div className="mb-4">` block:

**Current:**
```tsx
<div className="mb-4">
  <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
    {template.id}
  </code>
</div>
```

**New:**
```tsx
<div className="mb-4 flex items-center gap-2">
  <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
    {template.id}
  </code>
  {template.source === 'architecton.ai' && (
    <span className="text-xs text-text-tertiary border border-border px-2 py-0.5 rounded">
      via architecton.ai
    </span>
  )}
</div>
```

This renders a subtle bordered pill badge "via architecton.ai" next to each template ID. The conditional check against `template.source` means any future `builtin` templates would not show the badge.

### 3d. CTA link after the grid

After the closing `</div>` of the template grid (`className="grid grid-cols-1 ..."`) and before the closing `</Container>`, add a centered CTA:

```tsx
        <div className="mt-12 text-center">
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 text-accent hover:underline font-medium"
          >
            Browse all templates on architecton.ai
            <span aria-hidden="true">&rarr;</span>
          </a>
        </div>
```

This sits below the grid, providing a clear external CTA.

### Full Templates.tsx after changes

For absolute clarity, here is the complete file the builder should produce:

```tsx
import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { templates } from '@/data/templates'
import { motion } from 'motion/react'

export function Templates() {
  return (
    <AnimatedSection id="templates">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          Template marketplace
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
          Production-ready environment configs available via{' '}
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent hover:underline"
          >
            architecton.ai
          </a>
          . Install with <code className="text-accent font-mono text-sm">great template apply</code>.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {templates.map((template, i) => (
            <motion.div
              key={template.id}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="bg-bg-secondary border border-border rounded-xl p-6 hover:border-accent/30 transition-colors flex flex-col"
            >
              <div className="mb-4 flex items-center gap-2">
                <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
                  {template.id}
                </code>
                {template.source === 'architecton.ai' && (
                  <span className="text-xs text-text-tertiary border border-border px-2 py-0.5 rounded">
                    via architecton.ai
                  </span>
                )}
              </div>
              <h3 className="font-display text-lg text-text-primary mb-2">{template.name}</h3>
              <p className="text-text-secondary text-sm mb-4 flex-1">{template.description}</p>

              <div className="space-y-3 pt-4 border-t border-border">
                <div>
                  <span className="text-text-tertiary text-xs uppercase tracking-wider">Agents</span>
                  <div className="flex flex-wrap gap-1.5 mt-1">
                    {template.agents.map((a) => (
                      <span key={a} className="text-xs bg-bg-tertiary text-text-secondary px-2 py-0.5 rounded">
                        {a}
                      </span>
                    ))}
                  </div>
                </div>
                <div>
                  <span className="text-text-tertiary text-xs uppercase tracking-wider">MCP Servers</span>
                  <div className="flex flex-wrap gap-1.5 mt-1">
                    {template.mcpServers.map((s) => (
                      <span key={s} className="text-xs bg-bg-tertiary text-text-secondary px-2 py-0.5 rounded">
                        {s}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <div className="mt-12 text-center">
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 text-accent hover:underline font-medium"
          >
            Browse all templates on architecton.ai
            <span aria-hidden="true">&rarr;</span>
          </a>
        </div>
      </Container>
    </AnimatedSection>
  )
}
```

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| `source` field missing on a template object | TypeScript will catch this at build time since the field is required in the interface |
| Future `builtin` templates added to data | The `{template.source === 'architecton.ai' && ...}` conditional means builtin templates show no badge |
| architecton.ai domain is unreachable | Links are standard `<a>` tags -- the browser handles dead links. No runtime impact on the site |
| Template grid has 5 items (not divisible by 3) | Already the case today. The `lg:grid-cols-3` grid handles this correctly with 2 items in the last row |

---

## Error Handling

No runtime error handling changes. All changes are static copy/markup. TypeScript enforces the `source` field at compile time via the `Template` interface.

---

## Security Considerations

- All external links use `rel="noopener noreferrer"` and `target="_blank"` (consistent with existing pattern)
- No user input is rendered; all content is static
- No new dependencies introduced

---

## Platform Considerations

No platform-specific behavior. These are React component changes rendered identically on all browsers/platforms. The Tailwind responsive breakpoints (`md:`, `lg:`) handle mobile vs desktop layout as before.

---

## Testing Strategy

### Build gate
```bash
cd /home/isaac/src/sh.great/site && pnpm build:site
```
Must complete with zero TypeScript errors and zero warnings.

### Manual verification checklist

1. **Footer link present:** Scroll to footer. Confirm "Part of the architecton.ai ecosystem" text appears after "Built by Superstruct", with a clickable link to `https://architecton.ai`.

2. **Templates heading:** The section heading reads "Template marketplace" (not "Start with a template").

3. **Templates subheading:** Contains "available via architecton.ai" with a clickable link, and mentions `great template apply`.

4. **Source badges:** Each of the 5 template cards shows a "via architecton.ai" badge next to the template ID.

5. **CTA link:** Below the template grid, "Browse all templates on architecton.ai" with an arrow is visible and clickable.

6. **No free/included language:** No text on the page says "free", "included", "bundled", or "ships with" in relation to the 5 marketplace templates.

7. **Dark theme legibility:** The "via architecton.ai" badge border is visible against `bg-bg-secondary`. The CTA link uses `text-accent` (#22c55e) and is legible against `bg-bg-primary` (#0a0a0a).

8. **Mobile layout:** On viewport < 768px, the footer attribution text wraps cleanly. Template cards stack in a single column. The CTA link remains centered.

---

## Verification Gate

The builder declares this task complete when:

- [ ] `pnpm build:site` exits 0
- [ ] `git diff` shows changes only in the 3 specified files
- [ ] Footer.tsx matches the original `80bafbf` version of the architecton.ai block exactly
- [ ] All 5 template entries in `templates.ts` have `source: 'architecton.ai'`
- [ ] Templates.tsx contains no language implying templates are free or included
- [ ] All external links use `rel="noopener noreferrer"` and `target="_blank"`
