# 0036 — Fix Site: Accurate Copy for Vault and Sync Features

**Priority:** P1
**Type:** bugfix (content)
**Module:** `site/src/data/`
**Status:** Backlog
**Complexity:** S
**Created:** 2026-02-28

## Context

The marketing site makes two concrete claims about cloud sync that contradict
what the CLI actually does:

**Claim 1 — features.ts (the feature card):**
```
title: 'Cloud-Synced Credentials',
description: 'Zero-knowledge encrypted vault syncs API keys and config
across machines. BYO credentials — we never see your keys.',
```

**What the CLI does** — `src/cli/sync.rs` lines 52-54:
```rust
// Save locally (cloud sync is a future feature)
output::warning("Cloud sync is not yet available. Saving locally.");
```

A user who installs great.sh after reading this card, then runs
`great sync push`, is greeted with a warning that the feature they came for
does not exist. This is a trust-breaking conversion funnel failure.

**Claim 2 — Features.tsx (section subtitle):**
```
great.sh is the only tool that touches all five layers: template provisioning,
AI agent orchestration, MCP configuration, credential management,
and cross-machine sync.
```
"cross-machine sync" is stated as a current capability, not a roadmap item.

**Claim 3 — comparison.ts (comparison table row):**
```
{ feature: 'Cross-machine sync', great: true, ... }
```
`true` renders as a checkmark. The CLI outputs an explicit "not yet available"
warning for this operation.

**What actually works today:**
- `great vault set/import` — stores secrets in the local system keychain (real)
- `great sync push` — saves a local snapshot of `great.toml` (real, useful)
- `great sync pull --apply` — restores from that local snapshot (real, useful)
- Cloud/cross-machine sync — not implemented (roadmap)

The fix is to update these three content files so they accurately describe the
working subset. Cloud sync should be described as "coming soon" or "local sync"
with an honest roadmap note. This does not require any Rust changes.

## Acceptance Criteria

- [ ] `site/src/data/features.ts` — the "Cloud-Synced Credentials" feature card
  title and description are updated to reflect what exists today: local keychain
  vault via `great vault`, local config snapshot via `great sync`, with cloud
  sync explicitly marked as coming soon. Example rewrite:
  `title: 'Credential Vault'`, description covering `great vault set/import`
  (system keychain), `great sync push/pull` (local config snapshot),
  and one line noting cloud sync is on the roadmap.

- [ ] `site/src/components/sections/Features.tsx` — the section subtitle
  (line 24) no longer claims "cross-machine sync" as a current capability;
  replace with an accurate fifth-layer description (e.g., "credential and
  config management") or remove the explicit list and use a more general phrase.

- [ ] `site/src/data/comparison.ts` — the "Cross-machine sync" row for great.sh
  is changed from `great: true` to `great: 'Local (cloud coming soon)'`
  so the comparison table does not show a false green checkmark.

- [ ] `pnpm --filter great-sh build` (or `pnpm build:site`) exits 0 with zero
  TypeScript errors after all changes.

- [ ] No Rust source files, tests, or config schema are modified by this task.

## Dependencies

None. All changes are in site content files.

## Notes

- Exact file locations and line references:
  - `site/src/data/features.ts` line 28-30 (the fourth feature entry)
  - `site/src/components/sections/Features.tsx` line 24 (subtitle paragraph)
  - `site/src/data/comparison.ts` line 76-83 (the "Cross-machine sync" row)

- The icon for the vault feature card is `shield` (ShieldCheck from lucide-react,
  mapped in Features.tsx line 11). The icon is appropriate for vault/credentials;
  keep it regardless of copy changes.

- The rewrite should be honest but not pessimistic. "Local config snapshot sync
  — cloud sync coming soon" is accurate and still conveys forward momentum.
  Do not remove the feature card; just make it factual.

- Vault reality check: `great vault login/unlock/set/import` are all
  implemented and functional (src/cli/vault.rs). The vault feature card
  description should lead with what works: system keychain storage via
  `security` (macOS) or `libsecret` (Linux), `.env` file import, and the
  `great sync push/pull` config snapshot workflow. These are genuine
  differentiators worth marketing honestly.

- Dijkstra advisory (iter-032): `map_or(0, ...)` / `map_or(false, ...)` idiom
  inconsistency in init.rs is P3; defer to a future minor cleanup task.
  Do not bundle it here.
