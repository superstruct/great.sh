# 0036 -- Humboldt Scout Report: Fix Site Sync Misinformation

**Task:** `.tasks/backlog/0036-fix-site-sync-misinformation.md`
**Spec:** `.tasks/ready/0036-fix-site-sync-spec.md`
**Socrates verdict:** APPROVED (Round 2)
**Scout:** Humboldt
**Date:** 2026-02-28

---

## Summary

6 content edits across 5 TypeScript/TSX files. Zero Rust changes. Zero type
changes. All edits are pure string replacements. No new imports, no new
components, no structural changes to any file.

---

## File Map: Every Edit Point

### 1. `site/src/data/features.ts`

Lines 26-31 (the fourth entry in the `features` array):

```
26    {
27      title: 'Cloud-Synced Credentials',        <-- CHANGE title
28      description:
29        'Zero-knowledge encrypted vault syncs API keys and config across machines. BYO credentials — we never see your keys.',   <-- CHANGE description
30      icon: 'shield',
31    },
```

**Before:** title `'Cloud-Synced Credentials'`, description as above
**After:** title `'Credential Vault'`, description `'Store API keys in your system keychain, import from .env files, and snapshot config locally. BYO credentials — cloud sync coming soon.'`
**Gotcha:** Icon stays `'shield'` -- maps to `ShieldCheck` in `Features.tsx` line 11. No icon map change needed. Feature count stays at 5, so the CSS grid odd-count centering at `Features.tsx` line 38 is unchanged.

---

### 2. `site/src/components/sections/Features.tsx`

Line 24 (inside the subtitle `<p>`):

```
23        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto">
24          great.sh is the only tool that touches all five layers: template provisioning,
25          AI agent orchestration, MCP configuration, credential management, and cross-machine sync.
26        </p>
```

**Before:** last two words of line 25 are `cross-machine sync`
**After:** last two words of line 25 become `config sync`
**Gotcha:** Line 24 itself is not touched. The edit is the trailing phrase on line 25 only. No JSX structural change.

---

### 3. `site/src/data/comparison.ts`

Lines 76-83 (the "Cross-machine sync" row):

```
76    {
77      feature: 'Cross-machine sync',
78      great: true,           <-- CHANGE this value only
79      chezmoi: 'Git-based',
80      mise: false,
81      nix: 'Git-based',
82      mcpm: false,
83      manual: false,
84    },
```

**Before:** `great: true`
**After:** `great: 'Local only'`
**Type check:** `ComparisonRow` interface (lines 1-9) types every tool column as `boolean | string`. The `great` field is `boolean | string` at line 3. Changing `true` to `'Local only'` is fully type-safe.
**Render check:** `CellValue` in `Comparison.tsx` lines 6-10 handles the three cases: `true` renders green check, `false` renders gray X, any string renders `<span className="text-text-secondary text-xs">{value}</span>`. No component change needed.
**Precedent:** `comparison.ts` line 86 already has `great: 'Via mise'` for the "Runtime version management" row -- same pattern.

---

### 4. `site/src/components/sections/HowItWorks.tsx`

Lines 26-31 (step 04 object in the `steps` array):

```
26    {
27      number: '04',
28      title: 'Sync',                                                            <-- CHANGE title
29      description: 'Push your encrypted config to the cloud. Pull it on any new machine.',  <-- CHANGE description
30      command: 'great sync push',
31    },
```

**Before:** title `'Sync'`, description as above
**After:** title `'Snapshot'`, description `'Save a local config snapshot. Restore it anytime, or on a fresh install.'`
**Gotcha:** `command: 'great sync push'` stays unchanged -- that is the correct CLI command. Step count remains 5; array length unchanged; animation delays (`i * 0.1`) are unaffected.

---

### 5. `site/src/components/sections/OpenSource.tsx`

Lines 14-21 (two adjacent paragraphs):

```
14          <p className="text-text-secondary mb-6 leading-relaxed">
15            The CLI is free and open source under the Apache 2.0 license. All local features work without
16            an account. Cloud sync and team features are optional paid tiers &mdash; the core
17            tool is yours to keep, forever.
18          </p>
19          <p className="text-text-tertiary text-sm mb-10">
20            BYO credentials. We never see your API keys. All encryption happens client-side.
21          </p>
```

**Before:** lines 15-17 contain paid-tier claim; line 20 contains encryption claim
**After (first paragraph):** `The CLI is free and open source under the Apache 2.0 license. Every feature works without an account. No paywalls, no telemetry &mdash; the tool is yours to keep, forever.`
**After (second paragraph):** `BYO credentials. We never see your API keys. Secrets stay in your system keychain.`
**Gotcha:** Two `&mdash;` HTML entities are present -- the first paragraph keeps one (`&mdash;`), the second paragraph loses its sentence (the "All encryption..." sentence is removed entirely, not just changed). Keep the `className` attributes exactly as-is on both `<p>` tags.

---

### 6. `site/src/data/commands.ts`

Line 21 (inside the `initWizardOutput` template literal):

```
21    ? Enter your API keys (stored in encrypted vault):
```

**Before:** `(stored in encrypted vault)`
**After:** `(stored in system keychain)`
**Gotcha:** This is inside a template literal (backtick string starting at line 3). The parenthetical is plain text -- no escaping, no interpolation. Character counts match: both are 27 characters including parentheses, so the `<pre>` terminal alignment in `HowItWorks.tsx` line 83 is preserved exactly.
**Not touched:** `commands.ts` line 76 has `provider = "great-vault"` inside `sampleToml` -- this is the actual schema provider name, not a false claim. Do not touch it.

---

## Files NOT Modified

| File | Reason |
|------|--------|
| `site/src/components/sections/Comparison.tsx` | `CellValue` already handles strings at lines 6-10. Zero changes needed. |
| `site/src/components/sections/OpenSource.tsx` lines 31-68 | GitHub links and alpha notice are accurate. |
| `site/src/data/features.ts` line 23 | "cross-client config sync" in MCP card is accurate -- refers to `great apply` writing MCP config to multiple client config files on the same machine. |
| `site/src/components/sections/Config.tsx` | "Apply it on any machine" is accurate -- `great.toml` is portable via version control. |
| `site/src/components/sections/Bridge.tsx` | "any mix of cloud and local models" is accurate -- refers to cloud AI APIs vs local Ollama, not cloud sync. |
| All Rust source files | Zero Rust changes in this task. |

---

## Dependency Map

These 6 edits have zero cross-dependencies. They can be applied in any order or
in parallel. No import changes, no new exports, no interface changes.

```
features.ts (Change 1)       -- standalone data edit
Features.tsx (Change 2)      -- standalone string edit in JSX
comparison.ts (Change 3)     -- standalone value edit; CellValue handles it already
HowItWorks.tsx (Change 4)    -- standalone object edit in const array
OpenSource.tsx (Change 5)    -- standalone paragraph text edits
commands.ts (Change 6)       -- standalone template literal edit
```

---

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| TypeScript type error on `great: 'Local only'` | None | `ComparisonRow.great` is `boolean \| string` at line 3 |
| `CellValue` not rendering string | None | Lines 6-10 of Comparison.tsx already render string values |
| Feature card grid layout break (5 cards, odd count) | None | Features.tsx line 38 already centers the last card on odd count; feature count stays 5 |
| Terminal alignment break in `<pre>` block | None | Both parentheticals are 27 chars; no change to character count |
| Missing `&mdash;` entity in OpenSource.tsx | Low | Copy the entity verbatim from the before text; both paragraphs use standard HTML entities |

---

## Recommended Build Order

1. `site/src/data/features.ts` -- title + description of entry at lines 26-31
2. `site/src/data/comparison.ts` -- single value at line 78: `true` to `'Local only'`
3. `site/src/data/commands.ts` -- single word at line 21: `encrypted vault` to `system keychain`
4. `site/src/components/sections/Features.tsx` -- two words at line 25: `cross-machine sync` to `config sync`
5. `site/src/components/sections/HowItWorks.tsx` -- title + description of step 04 at lines 28-29
6. `site/src/components/sections/OpenSource.tsx` -- two paragraphs at lines 14-21

Data files first (1-3), then components (4-6). Pure data edits carry no JSX
parse risk and will surface type errors immediately on save.

---

## Verification Commands

```bash
# TypeScript type check + production build (must exit 0)
cd /home/isaac/src/sh.great && pnpm build:site

# Post-edit grep: must return zero matches
cd /home/isaac/src/sh.great && grep -rn -i \
  'cloud.sync\|cross.machine.sync\|encrypted vault\|paid tier\|encryption happens' \
  site/src/
```

---

## Technical Debt Observed

None introduced by this task. One pre-existing note: the `initWizardOutput`
template literal in `commands.ts` (lines 3-33) is a hardcoded fake terminal
session with no connection to actual CLI behavior. If the init wizard
interactive flow changes in the future, this string must be manually kept in
sync. Not in scope for this task -- flagging for awareness.
