# 0033 -- Socrates Review: Bridge Section Component

**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0033-bridge-section-spec.md`
**Backlog:** `/home/isaac/src/sh.great/.tasks/backlog/0033-site-mcp-bridge-section-component.md`
**Reviewer:** Socrates | **Date:** 2026-02-28 | **Round:** 1

---

## VERDICT: APPROVED

---

## Verification Summary

### 1. TerminalWindow API
**Claim:** `title?: string` prop with default `"Terminal"`, `children: ReactNode`, `className?: string`.
**Actual:** `/home/isaac/src/sh.great/site/src/components/shared/TerminalWindow.tsx` lines 4-8 define exactly this interface. Line 10: `title = 'Terminal'` default. CONFIRMED.

### 2. Config.tsx pattern match
**Claim:** Two-column grid (`lg:grid-cols-2`), left = heading + paragraph + bullet list, right = visual.
**Actual:** `/home/isaac/src/sh.great/site/src/components/sections/Config.tsx` line 10: `grid grid-cols-1 lg:grid-cols-2 gap-12 items-start`. Left column has heading (line 12), paragraph (line 15), bullet list with `space-y-3` and checkmark pattern (lines 21-42). Right column is `CodeBlock`. CONFIRMED.

### 3. Loop.tsx heading consistency
**Claim:** Heading uses pattern `great mcp-bridge` + secondary span with em dash.
**Actual:** `/home/isaac/src/sh.great/site/src/components/sections/Loop.tsx` lines 56-58: `great loop` + `<span className="text-text-secondary">-- 16 roles, two steps</span>` using U+2014 em dash. Spec line 61-65 mirrors this pattern with `-- five backends, zero Node.js` also using U+2014. CONFIRMED.

### 4. motion import and animation props
**Claim:** `import { motion } from 'motion/react'` with `initial/whileInView/viewport/transition` props.
**Actual:** Loop.tsx line 5: `import { motion } from 'motion/react'`. Lines 81-84 and 129-132 use identical animation prop shapes. The spec's animation values (`opacity: 0, y: 20`, staggered delays 0.1/0.2) match Loop.tsx's two-column section (lines 128-145). CONFIRMED.

### 5. Nav link count (6 -> 7)
**Claim:** Current 6 links, adding "Bridge" brings to 7, fits at 1280px+.
**Actual:** `/home/isaac/src/sh.great/site/src/components/layout/Nav.tsx` lines 5-12: 6 entries (Features, Config, How it Works, Loop, Templates, Compare). Plus GitHub icon (line 52-59). Desktop nav at `md:` breakpoint (768px+) with `gap-8`. CONFIRMED -- see advisory concern below.

### 6. App.tsx insertion point
**Claim:** Insert `<Bridge />` between `<Loop />` (line 21) and `<Templates />` (line 22).
**Actual:** `/home/isaac/src/sh.great/site/src/App.tsx` line 21: `<Loop />`, line 22: `<Templates />`. Import insertion after line 7 (`Loop` import), before line 8 (`Templates` import). CONFIRMED.

### 7. Rust source claims
**5 backends:** `/home/isaac/src/sh.great/src/mcp/bridge/backends.rs` lines 31-77: `BACKEND_SPECS` array with gemini, codex, claude, grok, ollama. CONFIRMED.
**4 presets:** `/home/isaac/src/sh.great/src/mcp/bridge/tools.rs` lines 131-141: `enum Preset` with Minimal, Agent, Research, Full. CONFIRMED.
**Tool counts:** tools.rs lines 197-200: Minimal=1, Agent=6, Research=8, Full=9. CONFIRMED.
**Agent tools:** tools.rs lines 158-165: prompt, run, wait, list_tasks, get_result, kill_task. Matches `mcpBridgeOutput` at commands.ts line 108. CONFIRMED.

### 8. mcpBridgeOutput export
**Claim:** Exported string constant at commands.ts lines 98-110.
**Actual:** `/home/isaac/src/sh.great/site/src/data/commands.ts` lines 98-110: `export const mcpBridgeOutput` with exact terminal demo content. CONFIRMED.

### 9. Backlog conformance
The backlog (requirement 1) says "Follow the exact structural pattern of `Loop.tsx`." The spec explicitly deviates to follow Config.tsx pattern instead, with a justified rationale in "Design Decisions" (spec lines 196-209): Loop.tsx has three-phase agent roster + flow arrows, making it structurally inappropriate for a simple two-column section. The spec's hybrid approach (Config layout + Loop-style motion animations) is the correct call.

---

## Concerns

### Concern 1
```
{
  "gap": "Nav overflow at md breakpoint (768-1023px) with 7 links + GitHub icon",
  "question": "At 768px viewport, 7 text links (~60-80px each) + gap-8 (32px x 6 gaps) + GitHub icon + logo = ~780-820px content vs ~744px available (768 - 24px padding). Has anyone tested that 7 links do not wrap or overflow at exactly md breakpoint?",
  "severity": "ADVISORY",
  "recommendation": "The spec claims 'standard viewport widths (1280px+)' which is fine, but the actual breakpoint where inline nav appears is md (768px). At 768-900px this may be tight. This is a visual QA item for the manual verification checklist, not a blocking issue since wrapping is cosmetic and the hamburger menu handles < md. Consider noting '768-1024px' in the visual verification step."
}
```

### Concern 2
```
{
  "gap": "Double entrance animation (AnimatedSection + inner motion.div)",
  "question": "AnimatedSection applies motion.section with initial={{ opacity: 0, y: 30 }} and the inner motion.div elements add their own initial={{ opacity: 0, y: 20 }}. Does this create a compounding 50px vertical offset on first render?",
  "severity": "ADVISORY",
  "recommendation": "This is an existing pattern in Loop.tsx (same double-animation nesting) so it is proven to work. However, Config.tsx does NOT use inner motion.div -- it relies solely on AnimatedSection. The spec's hybrid approach is intentional but differs from Config. No action needed; just noting the conscious deviation."
}
```

---

## What the spec gets right

1. **All interface claims verified against source.** TerminalWindow, AnimatedSection, Container props all match actual TypeScript interfaces character-for-character.

2. **All Rust claims verified against source.** 5 backends, 4 presets, tool counts (1/6/8/9), and exact tool names all match `backends.rs` and `tools.rs`.

3. **Insertion points are exact.** App.tsx line numbers, Nav.tsx array position, import ordering -- all match the actual files.

4. **Design deviation is justified.** The spec correctly identifies that Config.tsx is a better structural template than Loop.tsx and documents the reasoning. The backlog's "exact structural pattern of Loop.tsx" instruction would have produced an over-engineered component.

5. **Edge cases are thorough.** Empty string, nav overflow, mobile menu, scroll-to-anchor, SSR -- all addressed.

6. **No new dependencies.** All imports are from existing modules. No package.json changes.

7. **Build order is correct.** Step 1 (create file) before step 2 (import it), step 3 independent.

---

## Summary

A clean, well-verified spec for a small-complexity site task. Every factual claim about component interfaces, Rust source, file contents, and line numbers has been verified against the actual codebase. The two advisory concerns are cosmetic edge cases that do not impede implementation.
