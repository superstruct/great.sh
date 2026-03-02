# Review: 0012 Loop Section Visual Polish

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `.tasks/ready/0012-loop-section-visual-polish.md`
**Backlog:** `.tasks/backlog/0012-loop-section-visual-polish.md`
**Date:** 2026-02-21
**Round:** 1

---

## VERDICT: REJECTED

---

## Concerns

### 1. Breakpoint Math Is Self-Contradictory -- Connectors Will Be Visible During Wrapping

```
{
  "gap": "The spec's own calculations prove that Phase 1 and 2 cards (4 agents) do NOT fit on a single row at the lg breakpoint (1024px), yet the fix shows connectors at lg and above.",
  "question": "The spec states on line 167: 'At the lg breakpoint (1024px), the container usable width is min(1200, 1024) - 96px padding = 928px, which fits 3 cards.' Then on line 169 it claims: 'For Phases 1 and 2 (4 agents each), that requires roughly 4 * 254 + 3 * 28 = 1100px, which fits inside lg (1104px usable).' The 1104px figure comes from the max container width (1200 - 96 = 1104), NOT the lg breakpoint (1024 - 96 = 928). At viewports between 1024px and ~1196px, Phase 1 and 2 cards will wrap to multiple rows while connectors are visible via `lg:inline` -- reproducing exactly the visual artifact Issue 2 claims to fix. Why does the spec choose `lg` as the breakpoint when its own arithmetic shows the layout does not fit until approximately 1200px?",
  "severity": "BLOCKING",
  "recommendation": "Either (a) change the connector visibility to `xl:inline` (1280px) so connectors only appear when all 4 cards reliably fit on one row, or (b) add explicit `min-w` / `max-w` constraints to ensure cards are narrow enough to fit 4-across at 928px (each card would need to be under ~220px including padding, not 254px), or (c) acknowledge the wrapping-with-connectors behavior at 1024-1196px is acceptable and explain WHY. The spec cannot simultaneously claim 928px usable width at lg AND claim 1100px of content fits at lg."
}
```

### 2. Implementation Note Contradicts Its Own Code Blocks

```
{
  "gap": "The implementation note on line 134 says 'Use a literal em dash character in the source, not the \\u2014 Unicode escape' but then every code block in the Exact Changes section shows `\\u2014` as the replacement string (e.g., Change 1a: `label: 'Phase 1 \\u2014 Sequential'`). The Build Order section (line 289) also uses what appears to be either three hyphens or a rendered em dash inside backticks. A builder reading the code blocks will use `\\u2014`; a builder reading the implementation note will use a literal character. These produce different source code.",
  "question": "Should the TSX source contain the literal Unicode character (---) or the JavaScript escape sequence (\\u2014)? Both render identically at runtime, but the spec must be unambiguous about which appears in the source file. Which is it?",
  "severity": "ADVISORY",
  "recommendation": "Pick one and make ALL code blocks and prose consistent. If literal em dash: show the literal character in code blocks and remove the \\u2014 notation. If escape sequence: remove the 'Use a literal em dash character' instruction. The 'Shortcut' paragraph should match the decision."
}
```

### 3. Phase 3 Grid Cards May Be Unacceptably Narrow at 320px

```
{
  "gap": "At 320px viewport with `grid-cols-2`, the usable width is 320 - 48px (px-6 padding) = 272px. With gap-2 (8px), each grid cell gets (272 - 8) / 2 = 132px. The agent cards contain three lines of text: name (font-display text-sm), role (text-xs), and methodology (text-xs, max-w-[220px]). At 132px, even short methodology strings like 'Audits credentials, permissions, input validation, supply chain' will wrap to 4-5 lines, making each card very tall and the 2-column grid visually dense.",
  "question": "Has anyone verified that 132px-wide Phase 3 cards are visually acceptable at 320px? The spec's edge case section (line 297) asserts 'Acceptable' without showing the visual result. Phases 1 and 2 at 320px would show single-column stacked cards (since they use flex-wrap, not grid), making Phase 3 the only phase with a 2-column layout at this width -- a visual inconsistency. Should Phase 3 also be single-column at the smallest viewports (e.g., `grid-cols-1 sm:grid-cols-2 lg:grid-cols-3`)?",
  "severity": "ADVISORY",
  "recommendation": "Consider `grid-cols-1 sm:grid-cols-2 lg:grid-cols-3` for Phase 3 so that at the narrowest viewports it matches the single-column stacking behavior of Phases 1 and 2. At minimum, the spec should explain why 2-column at 320px is preferable to 1-column, since the other phases stack to 1 column at that width."
}
```

### 4. Testing Matrix Does Not Cover the Broken Range (1024-1196px)

```
{
  "gap": "The testing table (lines 330-335) tests at 375px, 768px, 1024px, and 1200px. The expected behavior at 1024px is '4 cards in a row' for Phase 1-2, but as established in Concern 1, cards will actually wrap at 1024px. The testing matrix will produce a false-pass result if the tester does not notice wrapping, or will reveal the bug but the spec provides no guidance on what the correct behavior should be.",
  "question": "What is the expected visual result for Phase 1 and Phase 2 at exactly 1024px and at 1100px? The testing matrix claims '4 cards in a row' at 1024px, but the math shows wrapping. Is this a testing error or is the spec's layout math wrong?",
  "severity": "BLOCKING",
  "recommendation": "After resolving Concern 1, update the testing matrix to reflect the actual expected behavior at each breakpoint. If connectors move to xl:inline, add a 1280px row to the matrix."
}
```

### 5. Phase Label Rendering Context Not Verified

```
{
  "gap": "The spec states on line 305 that phase labels are rendered in 'text-xs font-mono uppercase tracking-wider' and that em dashes render correctly in monospace fonts. However, `uppercase` transforms all text to uppercase -- including the em dash character. While CSS text-transform: uppercase does not affect punctuation, the spec does not verify that the em dash plus surrounding spaces renders well in the specific font (JetBrains Mono) at text-xs with tracking-wider applied. In some monospace fonts, em dashes can appear as narrow as hyphens or can misalign with the tracking.",
  "question": "Has the em dash been visually verified in JetBrains Mono at text-xs with tracking-wider? The testing checklist (line 339) says 'All phase labels display em dash, not double-hyphen' but does not ask the tester to verify the em dash is visually distinguishable from a long hyphen in this specific rendering context.",
  "severity": "ADVISORY",
  "recommendation": "Add a note to the testing checklist asking the tester to confirm the em dash is visually distinct and aesthetically acceptable in the phase label rendering context (JetBrains Mono, text-xs, uppercase, tracking-wider)."
}
```

---

## Acceptance Criteria Coverage

| Backlog Criterion | Spec Coverage | Assessment |
|---|---|---|
| Phase labels use em dash consistently | Issue 1: 11 string replacements covering all `--` occurrences | COMPLETE (all 11 occurrences in source verified) |
| Mobile layout hides or repositions flow connectors | Issue 2: `hidden lg:inline` | FLAWED -- connectors visible during wrapping at 1024-1196px (see Concern 1) |
| Phase 3 cards display fully at all desktop widths | Issue 3: grid layout | COMPLETE, though 320px behavior is questionable (see Concern 3) |

---

## What the Spec Gets Right

- Line number references are accurate against the current source (`/home/isaac/src/sh.great/site/src/components/sections/Loop.tsx`). All 11 `--` occurrences verified.
- The `'note' in phase` conditional (line 111 of source) is correctly outside the scope of Change 3 (lines 89-109) and will be unaffected by the JSX restructure.
- The `phase.agents.length > 5` threshold correctly isolates Phase 3 (6 agents) from Phases 1 and 2 (4 agents each).
- Change 3's combined AFTER block correctly integrates Issue 2's `hidden lg:inline` into the connector span, avoiding duplication.
- The Tailwind classes used (`hidden`, `lg:inline`, `grid`, `grid-cols-2`, `lg:grid-cols-3`) are all valid default Tailwind utilities. No custom config required.
- Security, error handling, and scope sections are appropriately marked as not applicable.

---

## Summary

The spec contains a self-contradictory breakpoint calculation: it acknowledges 928px usable width at lg but then claims 1100px of content fits at lg, leading to a connector visibility fix that does not actually work at the lg breakpoint (1024-1196px range). This must be resolved before implementation.
