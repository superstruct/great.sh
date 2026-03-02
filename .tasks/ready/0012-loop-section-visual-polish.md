# Spec 0012: Loop Section Visual Polish

**Task:** `.tasks/backlog/0012-loop-section-visual-polish.md`
**Status:** ready
**Priority:** P3
**Type:** UI (site)
**File to modify:** `site/src/components/sections/Loop.tsx` (single file, all changes)

---

## Summary

Three pre-existing visual issues in the Loop section of the marketing site need correction. All changes are confined to `site/src/components/sections/Loop.tsx`. No new files, no dependency changes, no config changes.

---

## Issue 1: Double-Hyphen Phase Labels

### Problem

Phase labels in the `phases` array use ASCII double-hyphen `--` while the section heading uses Unicode em dash (U+2014). This is a typographic inconsistency.

Affected strings -- phase labels (lines 9, 19, 30 of `Loop.tsx`):
- `'Phase 1 -- Sequential'`
- `'Phase 2 -- Parallel Team'`
- `'Phase 3 -- Gate + Finish'`

Agent methodology strings and the phase note also contain `--`:
- Line 22: `'Adversarial tester -- proves the build is broken'`
- Line 27: `'Wirth (Performance Sentinel) runs in parallel -- measures artifact size, flags regressions'`
- Line 32: `'Structured programming principles -- reviews quality, complexity, structure'`
- Line 34: `'Never commits a broken build -- all gates must pass'`
- Line 35: `'Every code example must work -- docs and release notes'`
- Line 37: `'PDCA cycle -- observer report, one config change if needed'`

The `slashCommands` array also contains `--`:
- Line 44: `'Capture requirements into .tasks/backlog/ -- run this first'`
- Line 48: `'UX discovery sweep -- Nielsen maps journeys, Nightingale files issues'`

### Fix

Replace every occurrence of ` -- ` with ` --- ` (space-emdash-space) in the `phases` data array and the `slashCommands` array. This includes all label strings, methodology/note strings, and slash command descriptions that use double-hyphen.

### Exact Changes

**Change 1a** -- Line 9:
```
// BEFORE
    label: 'Phase 1 -- Sequential',
// AFTER
    label: 'Phase 1 \u2014 Sequential',
```

**Change 1b** -- Line 19:
```
// BEFORE
    label: 'Phase 2 -- Parallel Team',
// AFTER
    label: 'Phase 2 \u2014 Parallel Team',
```

**Change 1c** -- Line 22:
```
// BEFORE
      { name: 'Turing', role: 'Test', methodology: 'Adversarial tester -- proves the build is broken' },
// AFTER
      { name: 'Turing', role: 'Test', methodology: 'Adversarial tester \u2014 proves the build is broken' },
```

**Change 1d** -- Line 27:
```
// BEFORE
    note: 'Wirth (Performance Sentinel) runs in parallel -- measures artifact size, flags regressions',
// AFTER
    note: 'Wirth (Performance Sentinel) runs in parallel \u2014 measures artifact size, flags regressions',
```

**Change 1e** -- Line 30:
```
// BEFORE
    label: 'Phase 3 -- Gate + Finish',
// AFTER
    label: 'Phase 3 \u2014 Gate + Finish',
```

**Change 1f** -- Line 32:
```
// BEFORE
      { name: 'Dijkstra', role: 'Code Review', methodology: 'Structured programming principles -- reviews quality, complexity, structure' },
// AFTER
      { name: 'Dijkstra', role: 'Code Review', methodology: 'Structured programming principles \u2014 reviews quality, complexity, structure' },
```

**Change 1g** -- Line 34:
```
// BEFORE
      { name: 'Hopper', role: 'Commit', methodology: 'Never commits a broken build -- all gates must pass' },
// AFTER
      { name: 'Hopper', role: 'Commit', methodology: 'Never commits a broken build \u2014 all gates must pass' },
```

**Change 1h** -- Line 35:
```
// BEFORE
      { name: 'Knuth', role: 'Docs', methodology: 'Every code example must work -- docs and release notes' },
// AFTER
      { name: 'Knuth', role: 'Docs', methodology: 'Every code example must work \u2014 docs and release notes' },
```

**Change 1i** -- Line 37:
```
// BEFORE
      { name: 'Deming', role: 'Observe', methodology: 'PDCA cycle -- observer report, one config change if needed' },
// AFTER
      { name: 'Deming', role: 'Observe', methodology: 'PDCA cycle \u2014 observer report, one config change if needed' },
```

**Change 1j** -- Line 44:
```
// BEFORE
  { cmd: '/backlog', desc: 'Capture requirements into .tasks/backlog/ -- run this first' },
// AFTER
  { cmd: '/backlog', desc: 'Capture requirements into .tasks/backlog/ \u2014 run this first' },
```

**Change 1k** -- Line 48:
```
// BEFORE
  { cmd: '/discover', desc: 'UX discovery sweep -- Nielsen maps journeys, Nightingale files issues' },
// AFTER
  { cmd: '/discover', desc: 'UX discovery sweep \u2014 Nielsen maps journeys, Nightingale files issues' },
```

**Implementation note:** Use a literal em dash character in the source, not the `\u2014` Unicode escape. The existing heading on line 58 already uses the literal character. All eleven replacements follow the same pattern: ` -- ` becomes ` \u2014 `.

**Shortcut:** The builder may use a global find-and-replace of ` -- ` with ` \u2014 ` across the entire file, since every occurrence of ` -- ` in `Loop.tsx` should become an em dash. Verify the heading on line 58 already uses the literal em dash and is unaffected.

---

## Issue 2: Mobile Flow Connectors

### Problem

At narrow viewports (375px and below ~1024px), agent cards stack vertically due to `flex-wrap` on the container (line 89). However, the connector symbols (`\u2192` for sequential, `+` for parallel) are rendered inside the same flex child as the preceding card (lines 103-107). When cards wrap, each connector appears to the right of its card rather than between rows, creating a visual artifact that obscures the sequential vs. parallel flow distinction.

### Analysis

The current DOM structure per agent is:

```tsx
<div key={agent.name} className="flex items-center gap-2">   {/* wrapper */}
  <div className="bg-bg-secondary ...">                       {/* card */}
    ...
  </div>
  {i < phase.agents.length - 1 && (
    <span className="text-text-tertiary text-sm font-mono">  {/* connector */}
      {phase.flow === 'parallel' ? '+' : '\u2192'}
    </span>
  )}
</div>
```

The wrapper div is a flex child of the outer `flex flex-wrap` container. When wrapping occurs, the connector stays attached to the right of its card rather than appearing between cards.

### Breakpoint Selection

The Tailwind config uses default breakpoints. Cards are approximately 254px wide (220px max-w methodology + 32px horizontal padding + 2px border). At the `lg` breakpoint (1024px), the container usable width is `min(1200, 1024) - 96px padding = 928px`, which fits 3 cards. Below `lg`, cards begin wrapping to fewer per row. At `md` (768px), usable width is `768 - 96 = 672px`, fitting only 2 cards. At `sm` and below, only 1 card fits per row.

The connectors only make visual sense when all cards in a phase sit on a single row. For Phases 1 and 2 (4 agents each), that requires roughly `4 * 254 + 3 * 28 = 1100px`, which fits inside `lg` (1104px usable). For Phase 3 (6 agents), even `lg` is insufficient -- but that is addressed in Issue 3.

**Decision:** Hide connectors below `lg` (1024px). Above `lg`, Phases 1 and 2 fit on a single row with connectors visible. Phase 3 connector behavior at desktop is addressed in Issue 3.

### Fix

Add Tailwind responsive utilities to the connector `<span>` to hide it below `lg`.

### Exact Changes

**Change 2** -- Lines 103-107, replace the connector span's className:

```tsx
// BEFORE (line 104)
                      <span className="text-text-tertiary text-sm font-mono">
// AFTER
                      <span className="hidden lg:inline text-text-tertiary text-sm font-mono">
```

This hides the connector on screens below 1024px (`hidden`) and shows it inline at `lg` and above (`lg:inline`).

---

## Issue 3: Phase 3 Card Layout at Desktop

### Problem

Phase 3 has 6 agents. At the max container width of 1200px (1104px usable after padding), 6 cards at ~254px each require ~1608px. This forces a wrap to two rows: approximately 4 cards on the first row and 2 on the second. The second row looks incomplete and unbalanced, especially with dangling connectors.

### Analysis and Approach

Three options were considered:

1. **Reduce card width for Phase 3 only.** At `max-w-[160px]` for methodology text, each card is ~194px. Six cards + 5 connectors: `6*194 + 5*28 = 1304px`. Still too wide for 1104px usable.

2. **Use a CSS grid with fixed columns.** A `grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6` would force all 6 cards into a single row at `lg` if cards are narrow enough. But with the methodology text, cards need at least ~200px, and `1104/6 = 184px` -- too tight.

3. **Use a 3-column grid at all desktop sizes.** A `grid grid-cols-2 lg:grid-cols-3` layout gives Phase 3 two balanced rows of 3 cards each at desktop, and a 2-column layout on tablets. This is visually clean, symmetric, and works at all breakpoints. Connectors are omitted inside the grid since spatial adjacency (left-to-right, top-to-bottom) conveys flow.

**Decision:** Option 3. Apply a grid layout to Phase 3 specifically, and keep flex layout for Phases 1 and 2. The connectors are already hidden below `lg` (Issue 2). For Phase 3, connectors are hidden entirely because even at desktop the grid layout replaces the linear flow metaphor with spatial adjacency.

### Fix

Restructure the agent rendering to conditionally use a grid for Phase 3 (6 agents) and flex for other phases. The cleanest approach is to branch on the agent count or phase index in the JSX.

### Exact Changes

**Change 3** -- Replace lines 89-109 (the agent rendering block) with the following:

```tsx
// BEFORE (lines 89-109)
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
                      <div className="text-text-tertiary text-xs mt-0.5 max-w-[220px]">
                        {agent.methodology}
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

// AFTER
              <div className={
                phase.agents.length > 5
                  ? 'grid grid-cols-2 lg:grid-cols-3 gap-2'
                  : 'flex flex-wrap items-center gap-2'
              }>
                {phase.agents.map((agent, i) => (
                  <div key={agent.name} className={
                    phase.agents.length > 5
                      ? ''
                      : 'flex items-center gap-2'
                  }>
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
                    {phase.agents.length <= 5 && i < phase.agents.length - 1 && (
                      <span className="hidden lg:inline text-text-tertiary text-sm font-mono">
                        {phase.flow === 'parallel' ? '+' : '\u2192'}
                      </span>
                    )}
                  </div>
                ))}
              </div>
```

Key differences from the original:

1. The outer `<div>` uses a ternary: if the phase has more than 5 agents, use `grid grid-cols-2 lg:grid-cols-3 gap-2`; otherwise keep the existing `flex flex-wrap items-center gap-2`.
2. The inner wrapper `<div>` only gets `flex items-center gap-2` for non-grid phases (it would interfere with grid cell sizing).
3. The connector `<span>` is only rendered for phases with 5 or fewer agents, and uses `hidden lg:inline` per Issue 2.
4. Phase 3 (6 agents) gets no connectors -- the 3-column grid provides visual flow via reading order.

---

## Build Order

All three issues are in a single file and can be implemented in one pass. Recommended order:

1. **Issue 1** -- String replacements in the `phases` data array (lines 9, 19, 22, 27, 30, 32, 34, 35, 37) and `slashCommands` array (lines 44, 48). Purely data changes, no logic. A global find-and-replace of ` -- ` with ` --- ` across the file handles all 11 occurrences.
2. **Issue 2 + Issue 3** -- Restructure the agent rendering JSX block (lines 89-109). Issues 2 and 3 are intertwined in the same code block, so implement together using the combined "AFTER" block from Issue 3.

---

## Edge Cases

### Viewport widths
- **320px (iPhone SE):** Cards stack in a single column. Grid phases use `grid-cols-2`, so each card spans half width (~142px). The `max-w-[220px]` on methodology text will not constrain at this width since the cell is narrower -- text wraps naturally. Acceptable.
- **375px (iPhone 12/13):** Same as 320px but slightly wider cells. Works correctly.
- **768px (iPad portrait):** Flex phases show cards wrapping 2-per-row without connectors. Grid phase shows 2 columns. Clean.
- **1024px (lg breakpoint):** Flex phases show 4 cards in a row with connectors visible. Grid phase shows 3 columns. All balanced.
- **1200px+ (max-w-site cap):** Same as 1024px but with more breathing room. No change in layout logic since container maxes at 1200px.

### Text length
- Em dash with surrounding spaces ` \u2014 ` is 3 characters vs ` -- ` (4 characters). No risk of text overflow changing.
- Phase labels are rendered in `text-xs font-mono uppercase tracking-wider` -- em dash renders correctly in monospace fonts.

### RTL / i18n
- Not applicable. Site is English-only, LTR.

---

## Error Handling

Not applicable -- this is a static JSX component with no runtime errors, network calls, or user input. All changes are to static data and conditional CSS classes.

---

## Security Considerations

None. No user input is processed. No dynamic content injection. All strings are compile-time constants embedded in the source.

---

## Testing Strategy

### Manual Visual Testing (Required)

Run `pnpm dev` from the `site/` directory and verify at these viewport widths using browser DevTools responsive mode:

| Viewport | Phase 1-2 | Phase 3 | Connectors |
|----------|-----------|---------|------------|
| 375px    | Cards stack, 1 per row | 2-column grid | Hidden |
| 768px    | Cards wrap, 2 per row | 2-column grid | Hidden |
| 1024px   | 4 cards in a row | 3-column grid | Visible (Phase 1-2 only) |
| 1200px   | 4 cards in a row | 3-column grid | Visible (Phase 1-2 only) |

### Checklist

- [ ] All phase labels display em dash, not double-hyphen (3 labels)
- [ ] All methodology strings with double-hyphen now show em dash (5 strings)
- [ ] Phase note string shows em dash (1 string)
- [ ] Slash command descriptions show em dash, not double-hyphen (2 strings: /backlog, /discover)
- [ ] At 375px: no connector arrows or plus signs visible
- [ ] At 375px: Phase 3 cards in 2-column grid
- [ ] At 1024px: Phase 1 and 2 show arrow/plus connectors between cards
- [ ] At 1024px: Phase 3 shows 3-column grid with no connectors
- [ ] Heading em dash on line 58 unchanged (already correct)
- [ ] Hover effect (`hover:border-accent/30`) still works on all cards
- [ ] No horizontal scrollbar at any tested viewport width

### Automated Testing

No automated tests are required for this change. The existing smoke tests in `tests/cli_smoke.rs` test the Rust CLI, not the site. The site has no test infrastructure and adding visual regression testing is out of scope for a P3 polish task.

### Build Verification

Run `pnpm build:site` from the project root to confirm TypeScript compilation succeeds and no type errors are introduced.

---

## Files Modified

| File | Change Type |
|------|-------------|
| `site/src/components/sections/Loop.tsx` | Edit: string replacements + JSX restructure |

No files created. No files deleted. No dependency changes.

---

## Acceptance Criteria Mapping

| Backlog Criterion | Addressed By |
|-------------------|--------------|
| Phase labels use em dash consistently | Issue 1: all 11 string replacements (3 labels, 6 methodologies/notes, 2 slash command descriptions) |
| Mobile layout hides or repositions flow connectors | Issue 2: `hidden lg:inline` on connector spans |
| Phase 3 cards display fully at all desktop widths | Issue 3: `grid grid-cols-2 lg:grid-cols-3` layout |
