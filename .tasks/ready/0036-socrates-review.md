# 0036 -- Socrates Review: Fix Site Sync Misinformation

**Spec:** `.tasks/ready/0036-fix-site-sync-spec.md`
**Backlog:** `.tasks/backlog/0036-fix-site-sync-misinformation.md`
**Reviewer:** Socrates
**Date:** 2026-02-28
**Round:** 2

---

## VERDICT: APPROVED

---

## Round 1 Blocking Concerns -- Resolution Verification

### Blocking Concern 1 (two additional false claims in HowItWorks.tsx and OpenSource.tsx): RESOLVED

Round 1 identified two untouched false claims:
- HowItWorks.tsx line 29: "Push your encrypted config to the cloud. Pull it on any new machine."
- OpenSource.tsx lines 15-17: "Cloud sync and team features are optional paid tiers"

Round 2 spec adds Change 4 (HowItWorks.tsx) and Change 5 (OpenSource.tsx) with full before/after diffs. Verified against actual source:

- `/home/isaac/src/sh.great/site/src/components/sections/HowItWorks.tsx` lines 26-31: spec "Before" block matches exactly. Replacement title "Snapshot" and description "Save a local config snapshot. Restore it anytime, or on a fresh install." are accurate per `sync.rs` behavior.

- `/home/isaac/src/sh.great/site/src/components/sections/OpenSource.tsx` lines 14-21: spec "Before" block matches exactly. Replacement addresses all three sub-claims: removes "Cloud sync and team features are optional paid tiers" (false -- nothing is paid), removes "All encryption happens client-side" (misleading -- CLI implements no encryption), and adds accurate copy about no-account/no-paywall model and system keychain.

### Blocking Concern 2 (commands.ts "encrypted vault"): RESOLVED

Round 1 flagged this as ADVISORY; Round 2 spec promoted it to Change 6 with explicit fix. Verified `/home/isaac/src/sh.great/site/src/data/commands.ts` line 21: spec "Before" matches exactly. "stored in encrypted vault" -> "stored in system keychain" is accurate.

---

## Round 1 Advisory Concerns -- Resolution Verification

### Advisory 3 (backlog deviation on comparison value): ADDRESSED

Spec documents the deliberate deviation from backlog's `'Local (cloud coming soon)'` to `'Local only'` with clear rationale in Change 3. Rationale is sound: comparison table cells use terse labels (e.g., "Git-based", "Via mise"), and "coming soon" implies committed timeline. Roadmap signal is preserved in Change 1 (feature card) and Change 4 (HowItWorks step). Accepted.

### Advisory 4 ("cross-client config sync" in MCP feature card): VERIFIED ACCURATE

Spec documents verification that `features.ts` line 23 "cross-client config sync" refers to `great apply` writing MCP server configs to multiple AI client config files from `great.toml`, not cross-machine sync. Confirmed in the exhaustive inventory table with "ACCURATE -- no fix needed" verdict. Accepted.

### Advisory 5 (description length causing layout issues): ADDRESSED

Round 2 description is 139 characters (down from Round 1's 197 characters). Spec notes this is shorter than both the MCP Server Management card (182 chars) and the AI Agent Orchestration card (269 chars). Visual verification via `pnpm --filter great-sh preview` remains in the testing strategy. Accepted.

---

## New Content Verification (Round 2 additions)

### Change 4: HowItWorks.tsx

- Before block (lines 26-31): MATCH with actual file
- After: title "Snapshot", description "Save a local config snapshot. Restore it anytime, or on a fresh install.", command stays `great sync push`
- Step count remains 5, no layout impact
- The command `great sync push` is retained -- accurate, this is the actual CLI command

### Change 5: OpenSource.tsx

- Before block (lines 14-21): MATCH with actual file
- After: first paragraph removes false paid-tier claim, second paragraph replaces "All encryption happens client-side" with "Secrets stay in your system keychain"
- "BYO credentials. We never see your API keys." retained -- accurate
- Paragraph lengths decrease slightly (spec says 155 vs 157 and 69 vs 74) -- no layout risk

### Change 6: commands.ts

- Before (line 21): MATCH with actual file
- Parenthetical character count: "(stored in encrypted vault)" and "(stored in system keychain)" are both 27 characters including parens. Terminal alignment preserved.
- The `sampleToml` at line 76 (`provider = "great-vault"`) is NOT touched -- correctly excluded, as "great-vault" is the actual schema provider name, not a false claim.

---

## Exhaustive Grep Sweep

Ran `grep -i` for `cloud`, `cross.machine`, `encrypt`, `sync`, `paid|tier|account`, and `vault` across `/home/isaac/src/sh.great/site/src/`. Every hit accounted for:

| File:Line | Content | Disposition |
|-----------|---------|-------------|
| `features.ts:27` | "Cloud-Synced Credentials" | Change 1 |
| `features.ts:29` | "encrypted vault syncs API keys and config across machines" | Change 1 |
| `features.ts:23` | "cross-client config sync" | Verified accurate (MCP cross-client, not cross-machine) |
| `Features.tsx:24` | "cross-machine sync" | Change 2 |
| `comparison.ts:76-77` | "Cross-machine sync" + `great: true` | Change 3 |
| `HowItWorks.tsx:28-29` | "Sync" + "Push your encrypted config to the cloud" | Change 4 |
| `HowItWorks.tsx:30` | "great sync push" | Accurate CLI command, no fix needed |
| `OpenSource.tsx:16` | "Cloud sync and team features are optional paid tiers" | Change 5 |
| `OpenSource.tsx:20` | "All encryption happens client-side" | Change 5 |
| `commands.ts:21` | "stored in encrypted vault" | Change 6 |
| `commands.ts:76` | `provider = "great-vault"` | Accurate schema name, no fix needed |
| `Bridge.tsx:10` | "any mix of cloud and local models" | Accurate (cloud AI APIs, not cloud sync) |
| `Config.tsx:19` | "Apply it on any machine" | Accurate (great.toml is portable via VCS) |

**Zero false claims remain unaddressed after Changes 1-6.**

---

## Remaining Advisory (non-blocking)

### Advisory: Character count claim precision

```
{
  "gap": "Spec claims both parentheticals are '26 vs 26 chars' but actual count including parens is 27 each.",
  "question": "Is this an off-by-one in the spec's counting, or did the spec exclude one boundary character?",
  "severity": "ADVISORY",
  "recommendation": "Trivial. The counts match each other regardless of the absolute number. No action needed."
}
```

---

## Summary

The Round 2 spec is comprehensive, accurate, and complete. It identifies 7 false or misleading claims across 5 files (plus 1 additional file for Change 6), provides exact before/after diffs that match the actual source code, addresses all Round 1 blocking and advisory concerns with documented rationale, and includes a post-edit grep verification step to confirm no false claims survive. The exhaustive grep sweep confirms zero missed hits. Approved for implementation.
