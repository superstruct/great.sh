# Scout Report 0012 — Restore architecton.ai Footer Link + Fix Templates Section

**Scout:** Alexander von Humboldt
**Date:** 2026-02-20
**Spec:** `.tasks/ready/0012-restore-architecton-fix-templates-spec.md`
**Socrates verdict:** PASS (no blocking issues)

---

## File Map

### 1. `/home/isaac/src/sh.great/site/src/data/templates.ts`

Total lines: 57

**Interface block (lines 1-8)** — `source` field must be added as last property:
```
Line 7:   tools: string[]
Line 8: }
```
Insert `source: 'architecton.ai' | 'builtin'` between line 7 and 8.

**Template objects — `source` field insertion points:**
- Entry 1 (`ai-fullstack-ts`): after line 18 (`tools: [...]`), before `},` on line 19
- Entry 2 (`ai-fullstack-py`): after line 27 (`tools: [...]`), before `},` on line 28
- Entry 3 (`ai-data-science`): after line 36 (`tools: [...]`), before `},` on line 37
- Entry 4 (`ai-devops`): after line 45 (`tools: [...]`), before `},` on line 46
- Entry 5 (`ai-minimal`): after line 54 (`tools: [...]`), before `},` on line 55

All 5 entries get `source: 'architecton.ai'`. No entries added or removed.

No other file imports `Template` interface directly. Only consumer is `Templates.tsx`.

---

### 2. `/home/isaac/src/sh.great/site/src/components/sections/Templates.tsx`

Total lines: 63

**Imports (lines 1-4):** No changes. All four imports (`AnimatedSection`, `Container`, `templates`, `motion`) remain.

**Change 3a — Heading (line 11):**
```
Current:  Start with a template
New:      Template marketplace
```
Line 10-12 is the full `<h2>` block. Only the text node on line 11 changes.

**Change 3b — Subheading (lines 13-15):**
```
Current line 13: <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
Current line 14:   Curated environment configs encoding best-practice AI dev setups. Use as-is or customize.
Current line 15: </p>
```
Replace entire `<p>` block with multi-line JSX containing architecton.ai link and `great template apply` code span. `className` on `<p>` is unchanged.

**Change 3c — Source badge in card (lines 27-31):**
```
Current:
  27:       <div className="mb-4">
  28:         <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
  29:           {template.id}
  30:         </code>
  31:       </div>
```
Replace with `flex items-center gap-2` div containing the existing `<code>` plus conditional `<span>` badge. The `mb-4` class moves to the new outer div.

**Change 3d — CTA link after grid (line 59-60):**
```
Current line 59:       </div>
Current line 60:     </Container>
```
Insert CTA `<div>` block between closing grid `</div>` (line 59) and closing `</Container>` (line 60). No existing code moves or wraps.

---

### 3. `/home/isaac/src/sh.great/site/src/components/layout/Footer.tsx`

Total lines: 42

**Git history confirms removal:**
- `a2e97a4` — commit that removed the architecton.ai link (current state)
- `80bafbf` — original commit that included the link

**Current attribution block (lines 28-38):**
```
28:         <div className="mt-6 text-center text-text-tertiary text-xs">
29:           Built by{' '}
30:           <a
31:             href="https://superstruct.nz"
32:             target="_blank"
33:             rel="noopener noreferrer"
34:             className="hover:text-text-secondary transition-colors"
35:           >
36:             Superstruct
37:           </a>
38:         </div>
```

**Exact change:** After the closing `</a>` on line 37, before `</div>` on line 38, insert the architecton.ai block. The `<div>` open tag (line 28) and close tag (line 38) are unchanged. Only the inner content grows.

**Styling pattern to match:** Restored link uses `className="hover:text-text-secondary transition-colors"` — identical to the Superstruct link on line 34. Already verified consistent with site theme.

---

## Dependency Map

```
templates.ts (interface + data)
    └── Templates.tsx (only consumer of Template type and templates array)

Footer.tsx
    └── standalone, no data dependency
```

No circular dependencies. No cascading type changes beyond the `source` field addition.

---

## Patterns to Follow

- External links: always `target="_blank" rel="noopener noreferrer"` — confirmed across Footer.tsx and spec
- Accent color links in body text: `className="text-accent hover:underline"` — matches existing site pattern
- Footer attribution links: `className="hover:text-text-secondary transition-colors"` — matches Superstruct link
- Badge/pill styling: `text-xs text-text-tertiary border border-border px-2 py-0.5 rounded` — consistent with tertiary text treatment in the codebase
- Tailwind `flex items-center gap-2` for inline icon+label rows — used throughout site components

---

## Risks

1. **Low: Verbatim copy-paste of full file listing.** Spec section "Full Templates.tsx after changes" (lines 239-328) provides the complete file. Socrates flagged this: builder must use targeted edits (sections 3a-3d), not overwrite the whole file. The full listing is reference only.

2. **Low: `tools` field dead data.** The `tools` array is defined in the interface (line 6), populated in all 5 data entries, but never rendered in `Templates.tsx`. This is pre-existing technical debt, out of scope for this task.

3. **None: Type safety.** TypeScript enforces the required `source` field at build time. If any entry is missing it, `pnpm build:site` will fail with a clear error.

---

## Recommended Build Order

Per spec section "Build Order" — confirmed correct:

1. **`templates.ts`** — add `source` to interface and all 5 data entries. Safe first because `Templates.tsx` does not reference `source` yet; TypeScript will not error on the data file in isolation.
2. **`Templates.tsx`** — update heading, subheading, card badge, and CTA. At this point `source` is available on the type, so the conditional render compiles cleanly.
3. **`Footer.tsx`** — restore architecton.ai block. Fully independent of the other two files; order here is arbitrary, but last keeps the diff clean.

**Build gate:** `pnpm build:site` from `/home/isaac/src/sh.great/site/` must exit 0 with zero TypeScript errors.

---

## Summary for Builder

Three targeted edits, no new files, no new dependencies.

- `templates.ts`: 1 interface line + 5 data property insertions
- `Templates.tsx`: 4 localised changes (heading text, subheading block, card div, post-grid CTA)
- `Footer.tsx`: 1 block insertion inside an existing `<div>` (lines 28-38), after the Superstruct `</a>`

All `old_string` targets are verified unique in their files. Spec is authoritative over the task description on footer placement.
