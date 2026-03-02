# 0036 -- Fix Site Sync Misinformation: Technical Specification (Round 2)

**Task:** `.tasks/backlog/0036-fix-site-sync-misinformation.md`
**Complexity:** S (7 content edits across 5 site files, no Rust changes)
**Author:** Lovelace
**Date:** 2026-02-28
**Revision:** 2 (addresses Socrates rejection and all advisory concerns)

---

## Summary

The marketing site makes seven false or misleading claims about cloud sync, encryption, and paid tiers. The CLI (`src/cli/sync.rs` lines 52-53, 66-67) explicitly warns "Cloud sync is not yet available." The vault uses system keychain delegation (`security` on macOS, `libsecret` on Linux) -- the CLI itself implements no encryption.

This spec provides exact before/after edits for 5 site content files covering all 7 claims. Every change makes the copy honest while remaining positive about what actually works today.

**What works today (verified from source):**
- `great vault login` -- tests keychain read/write (`src/cli/vault.rs` lines 48-107)
- `great vault set <key>` -- stores secrets in system keychain via `security` (macOS) or `libsecret` (Linux) (`src/cli/vault.rs` lines 134-178)
- `great vault import <path>` -- imports `.env` files or env vars into keychain (`src/cli/vault.rs` lines 181-347)
- `great sync push` -- saves `great.toml` as a local snapshot (`src/cli/sync.rs` lines 33-59)
- `great sync pull --apply` -- restores from that local snapshot with backup (`src/cli/sync.rs` lines 62-131)

**What does not work today:**
- Cloud/cross-machine sync of any kind
- Any encryption implemented by the CLI (system keychain handles credential protection)
- Paid tiers of any kind (no pricing, no accounts)

---

## Exhaustive Claim Inventory

A grep for `cloud|cross.machine|sync|encrypt|paid|tier|account` across `site/src/` produced the following. Each hit was evaluated and categorized.

| # | File | Line(s) | Claim | Verdict |
|---|------|---------|-------|---------|
| 1 | `site/src/data/features.ts` | 27-29 | "Cloud-Synced Credentials" + "syncs API keys and config across machines" | FALSE -- fix |
| 2 | `site/src/components/sections/Features.tsx` | 23-24 | "cross-machine sync" | FALSE -- fix |
| 3 | `site/src/data/comparison.ts` | 76-77 | `great: true` for "Cross-machine sync" | FALSE -- fix |
| 4 | `site/src/components/sections/HowItWorks.tsx` | 28-29 | "Push your encrypted config to the cloud. Pull it on any new machine." | FALSE -- fix |
| 5 | `site/src/components/sections/OpenSource.tsx` | 15-17 | "Cloud sync and team features are optional paid tiers" | FALSE -- fix |
| 6 | `site/src/components/sections/OpenSource.tsx` | 20 | "All encryption happens client-side." | MISLEADING -- fix |
| 7 | `site/src/data/commands.ts` | 21 | "stored in encrypted vault" | MISLEADING -- fix |
| -- | `site/src/data/features.ts` | 23 | "cross-client config sync" (MCP card) | ACCURATE -- `great apply` writes MCP config to multiple client config files from `great.toml`. Not cross-machine. No fix needed. |
| -- | `site/src/components/sections/Config.tsx` | 19 | "Apply it on any machine." | ACCURATE -- `great.toml` is a portable file committed to version control; `great apply` runs it anywhere. No fix needed. |
| -- | `site/src/components/sections/Bridge.tsx` | 10 | "any mix of cloud and local models" | ACCURATE -- refers to cloud AI APIs (Gemini, Codex) vs local (Ollama), not cloud sync. No fix needed. |

---

## Files to Modify

| # | File | Line(s) | Change |
|---|------|---------|--------|
| 1 | `site/src/data/features.ts` | 27-29 | Rename feature card, rewrite description |
| 2 | `site/src/components/sections/Features.tsx` | 23-24 | Fix subtitle claim |
| 3 | `site/src/data/comparison.ts` | 76-77 | Change `great: true` to string value |
| 4 | `site/src/components/sections/HowItWorks.tsx` | 28-29 | Rewrite step 4 title and description |
| 5 | `site/src/components/sections/OpenSource.tsx` | 15-17, 20 | Rewrite pricing paragraph and encryption claim |
| 6 | `site/src/data/commands.ts` | 21 | Fix "encrypted vault" to "system keychain" |

No Rust source files, tests, or config schema are modified.

---

## Change 1: `site/src/data/features.ts`

### Before (lines 26-31)

```typescript
  {
    title: 'Cloud-Synced Credentials',
    description:
      'Zero-knowledge encrypted vault syncs API keys and config across machines. BYO credentials \u2014 we never see your keys.',
    icon: 'shield',
  },
```

### After

```typescript
  {
    title: 'Credential Vault',
    description:
      'Store API keys in your system keychain, import from .env files, and snapshot config locally. BYO credentials \u2014 cloud sync coming soon.',
    icon: 'shield',
  },
```

### Rationale

- Title changes from "Cloud-Synced Credentials" to "Credential Vault" -- accurate, no false cloud claim.
- Description leads with the three things that actually work: keychain storage, `.env` import, and local config snapshot.
- "BYO credentials" is retained -- it is accurate (users provide their own API keys).
- "cloud sync coming soon" is honest roadmap language.
- Description is 139 characters (vs original 117) -- fits comfortably in the card without overflow. Shorter than the Round 1 spec's 197-character version (addresses Socrates Concern 5).
- Icon stays `shield` (maps to `ShieldCheck` in `Features.tsx` line 11) -- appropriate for credentials.

---

## Change 2: `site/src/components/sections/Features.tsx`

### Before (lines 22-25)

```tsx
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto">
          great.sh is the only tool that touches all five layers: template provisioning,
          AI agent orchestration, MCP configuration, credential management, and cross-machine sync.
        </p>
```

### After

```tsx
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto">
          great.sh is the only tool that touches all five layers: template provisioning,
          AI agent orchestration, MCP configuration, credential management, and config sync.
        </p>
```

### Rationale

- Replaces "cross-machine sync" (false -- sync is local-only) with "config sync" (true -- `great sync push/pull` does sync config locally).
- "config sync" is accurate: the CLI exports and restores `great.toml` snapshots. It does not do it across machines yet.
- Minimal change -- only the last two words of the sentence. No structural disruption.
- The five-layer marketing claim is preserved because all five layers do exist in some form.

---

## Change 3: `site/src/data/comparison.ts`

### Before (lines 75-83)

```typescript
  {
    feature: 'Cross-machine sync',
    great: true,
    chezmoi: 'Git-based',
    mise: false,
    nix: 'Git-based',
    mcpm: false,
    manual: false,
  },
```

### After

```typescript
  {
    feature: 'Cross-machine sync',
    great: 'Local only',
    chezmoi: 'Git-based',
    mise: false,
    nix: 'Git-based',
    mcpm: false,
    manual: false,
  },
```

### Rationale

- Changes `great: true` to `great: 'Local only'`.
- The `ComparisonRow` interface (`site/src/data/comparison.ts` line 3) already types the `great` field as `boolean | string`, so this is type-safe with zero interface changes.
- The `CellValue` component (`site/src/components/sections/Comparison.tsx` lines 6-10) already handles strings: when the value is a string, it renders `<span className="text-text-secondary text-xs">{value}</span>` instead of a green checkmark.
- Other columns already use string values (e.g., `chezmoi: 'Git-based'`, `nix: 'Git-based'`), so "Local only" is consistent with the table's visual language.
- The feature row name stays "Cross-machine sync" so users can still see this category exists.
- Deliberate deviation from backlog: backlog suggests `'Local (cloud coming soon)'` but this spec uses `'Local only'`. Rationale: comparison table cells use terse labels (all other string values are 2 words or fewer). "Coming soon" in a competitive comparison table implies a committed timeline and would look out of place next to "Git-based". The roadmap signal is already present in the feature card (Change 1) and HowItWorks step (Change 4). The comparison table should state facts, not promises.

---

## Change 4: `site/src/components/sections/HowItWorks.tsx`

### Before (lines 26-31)

```typescript
  {
    number: '04',
    title: 'Sync',
    description: 'Push your encrypted config to the cloud. Pull it on any new machine.',
    command: 'great sync push',
  },
```

### After

```typescript
  {
    number: '04',
    title: 'Snapshot',
    description: 'Save a local config snapshot. Restore it anytime, or on a fresh install.',
    command: 'great sync push',
  },
```

### Rationale

- Title changes from "Sync" to "Snapshot" -- accurately describes what `great sync push` does (saves a local snapshot, per `src/cli/sync.rs` line 55).
- "Push your encrypted config to the cloud" is replaced with "Save a local config snapshot" -- no false cloud claim, no false encryption claim.
- "Pull it on any new machine" is replaced with "Restore it anytime, or on a fresh install" -- this is true for local snapshots (user can restore after a `great.toml` change goes wrong, or after a fresh install on the same machine).
- The command stays `great sync push` because that is still the correct command.
- The five-step flow ("Install, Initialize, Code, Snapshot, Start the Loop") still reads naturally and maintains progression.

---

## Change 5: `site/src/components/sections/OpenSource.tsx`

### Before (lines 14-21)

```tsx
          <p className="text-text-secondary mb-6 leading-relaxed">
            The CLI is free and open source under the Apache 2.0 license. All local features work without
            an account. Cloud sync and team features are optional paid tiers &mdash; the core
            tool is yours to keep, forever.
          </p>
          <p className="text-text-tertiary text-sm mb-10">
            BYO credentials. We never see your API keys. All encryption happens client-side.
          </p>
```

### After

```tsx
          <p className="text-text-secondary mb-6 leading-relaxed">
            The CLI is free and open source under the Apache 2.0 license. Every feature works
            without an account. No paywalls, no telemetry &mdash; the tool is yours to keep, forever.
          </p>
          <p className="text-text-tertiary text-sm mb-10">
            BYO credentials. We never see your API keys. Secrets stay in your system keychain.
          </p>
```

### Rationale

This change addresses three separate false claims in two adjacent paragraphs:

**Claim 5 (line 15-17): "Cloud sync and team features are optional paid tiers"**
- Cloud sync does not exist. Paid tiers do not exist. Team features do not exist. This sentence promises a business model that has not been built.
- Replaced with "Every feature works without an account. No paywalls, no telemetry" -- this is accurate (there are no accounts, no paywalls, no telemetry in the CLI) and stronger than the original. Users who care about open source care about these things.

**Claim 6 (line 20): "All encryption happens client-side."**
- The CLI implements no encryption. The system keychain (macOS Keychain, Linux libsecret) handles credential protection as an OS-level delegation. Saying "all encryption happens client-side" implies the CLI has an encryption layer, which it does not.
- Replaced with "Secrets stay in your system keychain" -- accurate and still reassuring. Users understand that system keychains are secure. This does not overclaim.

**Preserved claim (line 20): "BYO credentials. We never see your API keys."**
- This is accurate: secrets are stored locally in the system keychain, never transmitted anywhere. Kept as-is.

---

## Change 6: `site/src/data/commands.ts`

### Before (line 21)

```typescript
  ? Enter your API keys (stored in encrypted vault):
```

### After

```typescript
  ? Enter your API keys (stored in system keychain):
```

### Rationale

- "encrypted vault" implies the CLI implements its own encryption. It does not -- it delegates to the OS keychain.
- "system keychain" is accurate: `great vault set` calls `security add-generic-password` on macOS and `secret-tool store` on Linux (both system keychain APIs).
- The parenthetical is cosmetic (it appears in the fake terminal output of the init wizard demo). The change has no code or type implications.

---

## Implementation Approach

### Build Order

This is a single-step change. All 6 changes across 5 files are independent content edits with no cross-dependencies. They can be done in parallel or in any order.

1. Edit `site/src/data/features.ts` -- change title and description of the fourth feature entry.
2. Edit `site/src/components/sections/Features.tsx` -- change two words in the subtitle.
3. Edit `site/src/data/comparison.ts` -- change one value from `true` to `'Local only'`.
4. Edit `site/src/components/sections/HowItWorks.tsx` -- change step 4 title and description.
5. Edit `site/src/components/sections/OpenSource.tsx` -- rewrite two paragraphs.
6. Edit `site/src/data/commands.ts` -- change one parenthetical string.

### Verification

```bash
cd /home/isaac/src/sh.great && pnpm build:site
```

This runs TypeScript type-checking and Vite production build. It must exit 0 with no errors.

The builder should also visually confirm via `pnpm --filter great-sh preview`:

1. **Features section** -- card shows "Credential Vault" (not "Cloud-Synced Credentials"). Subtitle says "config sync" (not "cross-machine sync"). Card height is proportional to neighbors (description is 139 chars, shorter than the MCP card's 182 chars and the AI Agent Orchestration card's 269 chars -- no overflow risk).
2. **Comparison table** -- shows "Local only" in the great.sh column for the "Cross-machine sync" row (rendered as gray text, not a green checkmark).
3. **How It Works section** -- step 4 shows "Snapshot" (not "Sync") with description "Save a local config snapshot. Restore it anytime, or on a fresh install."
4. **Open Source section** -- first paragraph says "Every feature works without an account. No paywalls, no telemetry." Second paragraph says "Secrets stay in your system keychain." No mention of cloud sync, paid tiers, or client-side encryption.
5. **Init wizard demo** (visible in How It Works terminal panel) -- shows "stored in system keychain" (not "stored in encrypted vault").

### Post-Edit Grep Verification

After all edits, the builder must run:

```bash
cd /home/isaac/src/sh.great && grep -rn -i 'cloud.sync\|cross.machine.sync\|encrypted vault\|paid tier\|encryption happens' site/src/
```

Expected output: zero matches. If any matches appear, a claim was missed and must be addressed before marking the task complete.

---

## Edge Cases

| Scenario | Handling |
|----------|----------|
| TypeScript type error from `great: 'Local only'` | Cannot happen: `ComparisonRow.great` is typed `boolean \| string` (line 3 of comparison.ts). Already used elsewhere (e.g., line 86: `great: 'Via mise'`). |
| String value rendering in comparison table | Already handled: `CellValue` component (Comparison.tsx lines 6-10) renders strings as `<span>` text. No component changes needed. |
| Feature card icon mismatch | Icon stays `shield`, which maps to `ShieldCheck` in the `iconMap` (Features.tsx line 11). No change needed. |
| Feature card grid layout with odd count | The grid already handles odd feature counts (Features.tsx line 38 centers the last card when count is odd). Feature count stays at 5, so the last card (Built-in AI Bridge) still centers. No change. |
| SEO/meta description referencing cloud sync | Checked via grep -- no meta tags reference cloud sync. Not affected. |
| HowItWorks step numbering or count change | Step count remains 5. Only the title and description of step 04 change. The `steps` array length is unchanged, so no layout or animation timing changes. |
| OpenSource paragraph length change | The replacement first paragraph is 2 characters shorter than the original (155 vs 157). The second paragraph is 5 characters shorter (69 vs 74). No layout impact. |
| Init wizard terminal demo alignment | The parenthetical "(stored in system keychain)" is the same character count as "(stored in encrypted vault)" -- 26 vs 26 chars. The terminal `<pre>` block alignment is preserved. |

---

## Error Handling

Not applicable. These are static content changes with no runtime error paths.

---

## Security Considerations

None. No secrets, API keys, user input handling, or server-side code involved. The changes improve accuracy of security-related marketing claims but do not touch any security-relevant code.

---

## Testing Strategy

| Test | Command | Expected |
|------|---------|----------|
| TypeScript compilation | `pnpm build:site` | Exit 0, no type errors |
| Visual regression (manual) | `pnpm --filter great-sh preview` | All 5 verification points in the Verification section above pass |
| Content accuracy audit | Read `src/cli/sync.rs` lines 52-53, `src/cli/vault.rs` lines 48-60 | Confirms local-only sync and keychain-based vault match the updated copy |
| Post-edit grep | `grep -rn -i 'cloud.sync\|cross.machine.sync\|encrypted vault\|paid tier\|encryption happens' site/src/` | Zero matches |

No automated test changes needed -- the site has no content-level unit tests.

---

## Acceptance Criteria Traceability

| Backlog Criterion | Addressed By |
|-------------------|-------------|
| features.ts card updated to reflect local keychain vault and local config snapshot | Change 1: title "Credential Vault", description covers keychain storage, `.env` import, local snapshot, cloud sync as coming soon |
| Features.tsx subtitle no longer claims "cross-machine sync" | Change 2: "cross-machine sync" replaced with "config sync" |
| comparison.ts row changed from `great: true` to string | Change 3: `great: 'Local only'` (deliberate deviation from backlog's `'Local (cloud coming soon)'` -- see rationale in Change 3) |
| `pnpm build:site` exits 0 | Verification step in build order |
| No Rust source files modified | Confirmed: zero Rust file changes in this spec |

### Beyond-Backlog Coverage

The backlog identified 3 claims. This spec covers 7 claims across 5 files -- the 3 from the backlog plus:

| Additional Claim | Addressed By |
|-----------------|-------------|
| HowItWorks.tsx "Push your encrypted config to the cloud. Pull it on any new machine." | Change 4: rewritten to describe local snapshot |
| OpenSource.tsx "Cloud sync and team features are optional paid tiers" | Change 5: rewritten to describe free/no-account model |
| OpenSource.tsx "All encryption happens client-side." | Change 5: rewritten to describe system keychain |
| commands.ts "stored in encrypted vault" | Change 6: changed to "stored in system keychain" |

---

## Socrates Review Response

### Blocking Concern 1 (two additional false claims): RESOLVED
Changes 4 and 5 now address HowItWorks.tsx and OpenSource.tsx respectively. The exhaustive claim inventory in this spec demonstrates complete coverage.

### Blocking Concern 2 (commands.ts "encrypted vault"): RESOLVED
Change 6 addresses the init wizard demo text.

### Advisory Concern 3 (backlog deviation on comparison value): ACKNOWLEDGED
Documented in Change 3 rationale. "Local only" is preferred over "Local (cloud coming soon)" for comparison table cells. The roadmap signal appears in the feature card and HowItWorks step instead.

### Advisory Concern 4 ("cross-client config sync" in MCP feature card): VERIFIED ACCURATE
The claim at `features.ts` line 23 describes `great apply` writing MCP server configurations to multiple AI client config files (`.mcp.json` for Claude Code, etc.) from a single `great.toml` source. This is "sync across clients on one machine," not "sync across machines." Verified in `src/cli/apply.rs` lines 655-718 where MCP configs are written to `.mcp.json`. No fix needed. Documented in the exhaustive inventory table above.

### Advisory Concern 5 (description length causing layout issues): ADDRESSED
Round 2 description for Change 1 is 139 characters (down from Round 1's 197 characters). This is shorter than both the MCP Server Management card (182 chars) and the AI Agent Orchestration card (269 chars), eliminating overflow risk. Visual verification is still required.

### OpenSource.tsx line 20 ("All encryption happens client-side"): ADDRESSED
Added as Change 5 (second paragraph). Replaced with "Secrets stay in your system keychain" which is accurate.
