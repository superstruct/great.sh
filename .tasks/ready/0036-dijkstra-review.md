# 0036 -- Dijkstra Code Review: Fix Site Sync Misinformation

**Reviewer:** Edsger Dijkstra
**Task:** `.tasks/backlog/0036-fix-site-sync-misinformation.md`
**Spec:** `.tasks/ready/0036-fix-site-sync-spec.md`
**Date:** 2026-02-28

---

## VERDICT: APPROVED

---

## Issues

None.

---

## Verification Record

All 6 changes from the spec were verified against current file state by direct
file read. Every change matches the spec's "After" block exactly.

### Change 1 -- `site/src/data/features.ts` lines 26-31

- Title: `'Credential Vault'` (was `'Cloud-Synced Credentials'`). Correct.
- Description: `'Store API keys in your system keychain, import from .env
  files, and snapshot config locally. BYO credentials \u2014 cloud sync
  coming soon.'` Correct.
- Icon `'shield'` retained. `iconMap` in `Features.tsx` maps `shield` to
  `ShieldCheck`; no import change needed. Feature count stays at 5; the
  odd-count centering logic at `Features.tsx` line 38 is unaffected.

### Change 2 -- `site/src/components/sections/Features.tsx` line 24

- Subtitle now reads "config sync" (was "cross-machine sync"). Correct.
- No JSX structural change. No import change.

### Change 3 -- `site/src/data/comparison.ts` line 77

- `great: 'Local only'` (was `great: true`). Correct.
- Type safety confirmed: `ComparisonRow.great` is typed `boolean | string`
  (line 3 of `comparison.ts`). The string literal is within the declared
  union. No interface change required.
- Render path confirmed: `CellValue` in `Comparison.tsx` lines 6-10 handles
  three cases -- `true` (green check), `false` (gray X), string (span text).
  The string branch is already exercised by existing rows such as
  `great: 'Via mise'` at line 86 and `great: 'Local only'` at line 77.
- The `row[tool.key]` access at `Comparison.tsx` line 60 is type-safe:
  `tool.key` is a union of the six non-`feature` keys of `ComparisonRow`,
  each typed `boolean | string`, which matches `CellValue`'s prop type
  exactly.

### Change 4 -- `site/src/components/sections/HowItWorks.tsx` lines 27-30

- `title: 'Snapshot'` (was `'Sync'`). Correct.
- `description: 'Save a local config snapshot. Restore it anytime, or on a
  fresh install.'` Correct.
- `command: 'great sync push'` unchanged. Step count remains 5; animation
  delay computation `i * 0.1` is unaffected.

### Change 5 -- `site/src/components/sections/OpenSource.tsx` lines 14-21

- First paragraph: removes false paid-tier and cloud-sync claims. Replacement
  "Every feature works without an account. No paywalls, no telemetry" is
  accurate.
- Second paragraph: "Secrets stay in your system keychain." replaces "All
  encryption happens client-side." which was misleading (the CLI implements no
  encryption; it delegates to the OS keychain). The replacement is accurate.
- "BYO credentials. We never see your API keys." retained verbatim. Correct.
- Both `<p>` `className` attributes unchanged.

### Change 6 -- `site/src/data/commands.ts` line 22

- `(stored in system keychain)` (was `(stored in encrypted vault)`). Correct.
- Character counts are equal (27 chars including parens); the `<pre>` terminal
  alignment in `HowItWorks.tsx` line 83 is preserved.
- `provider = "great-vault"` in `sampleToml` (line 76) correctly left
  untouched; it is a schema identifier, not a false claim.

---

## Post-edit grep sweep (run by reviewer)

Pattern: `cloud.sync|cross.machine.sync|encrypted vault|paid tier|encryption happens`

Two matches found -- both are correct:

1. `comparison.ts:76` -- the row **label** `'Cross-machine sync'`. This is the
   feature category name shown in the table header column, not a claim about
   great.sh's capability. The great.sh **value** in that row is `'Local only'`.
   Correct disposition per spec.

2. `features.ts:29` -- the phrase `cloud sync coming soon`. This is intentional
   roadmap language placed by Change 1. It is accurate (cloud sync is not yet
   available) and forward-looking (it is on the roadmap). Correct.

Zero residual false claims.

---

## Structural observations

- No new imports introduced in any file.
- No existing imports made unused by any change.
- No TypeScript interface changes.
- No component structural changes.
- All 6 edits are pure string replacements within existing data structures.
- Abstraction boundaries are unchanged: data files (`features.ts`,
  `comparison.ts`, `commands.ts`) remain pure data; presentation components
  (`Features.tsx`, `HowItWorks.tsx`, `OpenSource.tsx`) remain presentation-only.

---

## Summary

Six correct, minimal, type-safe string replacements that eliminate seven false
or misleading marketing claims. Every change matches the Socrates-approved
spec exactly. No structural, type, or abstraction issues.
