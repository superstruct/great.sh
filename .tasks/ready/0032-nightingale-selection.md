# Nightingale Selection: Task 0032

**Date:** 2026-02-28
**Selected Task:** 0032 — Marketing Site: Surface mcp-bridge as a Feature
**Task File:** `.tasks/done/0032-site-mcp-bridge-feature.md`
**Priority:** P2
**Type:** feature (content/site)
**Complexity:** S

---

## Backlog Survey

| # | Title | Priority | Status | Blocked? |
|---|-------|----------|--------|----------|
| 0010 | Complete All Stubs (umbrella) | P0 | Stale duplicate — all 11 groups DONE in source; done/ copy exists and is complete | N/A |
| 0014 | Prune and Reconcile the Task Backlog | P3 | Stale duplicate — audit completed iter 022; done/ copy with Audit Log exists | N/A |
| 0030 | MCP Bridge Hardening | P2 | DONE — shipped in commit 8f04bef; release-notes exist at `.tasks/ready/0030-release-notes.md` | N/A |
| 0031 | Loop + MCP Bridge Smoke Tests | P2 | DONE — release-notes exist at `.tasks/ready/0031-release-notes.md` confirming 6 tests added | N/A |
| 0032 | Marketing Site: Surface mcp-bridge | P2 | OPEN — spec exists but was REJECTED by Socrates (2 BLOCKING concerns) | No |

**Effective backlog depth:** 1 open task (0032).

The two stale backlog files (0010, 0014) have done/ counterparts marked complete. They are leftover artefacts that were not cleaned up when the pruning pass ran in iter 022. These should be deleted by the builder as a housekeeping step alongside 0032, but they are not substantive open work.

---

## Selection: 0032 — Marketing Site: Surface mcp-bridge as a Feature

### Rationale

0032 is the only substantively open task in the backlog. It is unblocked — all Rust code it describes (`great mcp-bridge`, path traversal prevention, auto-approve config) shipped in iterations 026 and 028. The spec was written and reviewed; Socrates rejected it with two BLOCKING concerns (incorrect TOML key formats). The spec author (Lovelace) must revise the spec before Da Vinci can build.

This is a site-only task: data files in `site/src/data/` plus one component icon-map update. No Rust changes. Build verification is `pnpm --filter great-sh build` exits 0.

### Blocking Socrates Concerns (spec must be revised before build)

**Concern 1 — TOML section header** (BLOCKING)
The spec and backlog both write `[mcp_bridge]` (underscore), but `src/config/schema.rs` has `#[serde(rename = "mcp-bridge")]`. The correct TOML header is `[mcp-bridge]` (hyphen). Marketing copy showing an invalid config key that is silently ignored by the parser is a user-facing correctness issue.

**Concern 2 — TOML inner field names** (BLOCKING)
`McpBridgeConfig` in `src/config/schema.rs` uses `#[serde(rename_all = "kebab-case")]`. The sample TOML must therefore use `default-backend` (not `default_backend`). Same applies to any future references to `timeout-secs`, `auto-approve`, `allowed-dirs`.

**Concern 3 — Version in demo output** (ADVISORY, not blocking)
`mcpBridgeOutput` in the spec includes `great-mcp-bridge v0.3.0` but the crate is at `0.1.0`. Recommend removing the version string entirely to match the style of `loopInstallOutput` (no versions in any other demo string).

**Concern 4 — `Unplug` icon semantics** (ADVISORY, not blocking)
`Unplug` represents a disconnected plug, semantically inverted for a "bridge" feature. Builder should evaluate `Cable`, `Network`, or `Link2` as alternatives. No correctness issue — any choice renders without error.

### Revised Acceptance Criteria

The backlog acceptance criteria are correct in intent but must reference the corrected TOML key format:

- [ ] `site/src/data/features.ts` contains exactly 5 `Feature` entries; the new entry has `icon: "bridge"` (or alternative icon key confirmed in `iconMap`) and title `"Built-in AI Bridge"`
- [ ] `site/src/data/comparison.ts` contains a row with `feature: "Built-in multi-AI bridge (no Node.js)"` where `great: true` and all other columns are `false`
- [ ] `site/src/data/commands.ts` exports `mcpBridgeOutput` showing `$ great mcp-bridge --preset agent` startup output with at least 3 detected backends listed; version string omitted
- [ ] `site/src/data/commands.ts` `sampleToml` includes a `[mcp-bridge]` stanza (hyphen, not underscore) with `preset = "agent"` and `default-backend = "gemini"` (kebab-case, not snake_case)
- [ ] `pnpm --filter great-sh build` exits 0 with no TypeScript errors after all changes (icon mapping must be wired if a new icon key is introduced)

### Housekeeping to perform alongside this task

The following stale backlog files should be deleted (both have complete done/ counterparts):
- `/home/isaac/src/sh.great/.tasks/backlog/0010-complete-all-stubs.md` — delete; done/ copy is authoritative
- `/home/isaac/src/sh.great/.tasks/backlog/0014-backlog-pruning.md` — delete; done/ copy with Audit Log is authoritative

---

## Instructions for Lovelace

Revise `.tasks/ready/0032-site-mcp-bridge-spec.md` to fix the two BLOCKING concerns:

1. Change every occurrence of `[mcp_bridge]` to `[mcp-bridge]` in the `sampleToml` sections (lines 313-323, 329-376 of current spec).
2. Change `default_backend = "gemini"` to `default-backend = "gemini"` (and any other inner fields from snake_case to kebab-case).
3. Remove `v0.3.0` from the `mcpBridgeOutput` demo string (advisory; recommended fix).
4. Evaluate icon alternatives for `bridge`: `Cable`, `Network`, or `Link2` (advisory; builder's judgment call).

After revision, resubmit to Socrates for round 2 review, then proceed to Da Vinci.
