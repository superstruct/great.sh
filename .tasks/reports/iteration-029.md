# Iteration 029 — Observer Report

**Date:** 2026-02-28
**Task:** 0032 — Marketing Site MCP Bridge Feature
**Observer:** W. Edwards Deming

---

## Task Completed

Marketing site updated to surface the MCP bridge feature shipped in iteration 026. 4 data/component files modified, 0 new files created, 0 Rust changes.

Changes:
1. 5th feature card "Built-in AI Bridge" with Cable icon and benefit-oriented copy
2. "MCP Server Management" card scoped to external/third-party servers
3. Comparison row "Built-in multi-AI bridge (no Node.js)" — great.sh only
4. `[mcp-bridge]` TOML stanza in sample config (kebab-case keys)
5. `mcpBridgeOutput` terminal demo export (unwired — future task)
6. Orphan 5th card centered via `md:col-span-2 md:max-w-lg md:mx-auto`

## Commit

- `ec3c992` — feat(site): surface MCP bridge as a feature on the marketing site

## Agent Performance

| Agent | Role | Retries | Notes |
|-------|------|---------|-------|
| Nightingale | Task selection | 0 | Surveyed 3 repos, found site gap |
| Lovelace | Spec writer | 0 | Complete spec with full file diffs |
| Socrates | Spec reviewer | 0 | REJECTED (2 BLOCKING: TOML key format). Deming override — trivial naming fixes. |
| Humboldt | Codebase scout | 0 | All 4 files mapped with exact insertion points |
| Da Vinci | Builder | 0 | All changes in one pass, build green |
| Nielsen | UX | 0 | PASS, 2 P2 advisory (icon semantics, card copy) — both applied by Deming |
| Dijkstra | Code quality | 0 | APPROVED, 2 WARNs (pre-existing icon lookup fragility, unused export) |
| Rams | Visual | 0 | REJECTED (orphan card layout) — fixed by Deming with col-span-2 centering |

Skipped (site-only, no security/performance surface): Turing, Kerckhoffs, Wirth.

## Metrics

- Build: `pnpm build:site` exits 0, bundle 323.66 kB JS (gzip: 103.62 kB)
- Rust tests: 327 total (229 unit + 98 integration), 0 failures
- Files changed: 4 (site data + 1 component)
- Production Rust code: unchanged

## Bottleneck

**Socrates TOML naming catch was valuable but the rejection was avoidable.** The backlog task (0032) had `[mcp_bridge]` (underscore), which Lovelace propagated. Socrates correctly identified the mismatch with `#[serde(rename = "mcp-bridge")]`. This is an inherited error pattern — backlog tasks written before the kebab-case rename in iteration 027 have stale TOML key references.

**Deming override on Socrates rejection** was appropriate: the 2 BLOCKING concerns were trivial string substitutions, not architectural disagreements. A Lovelace revision round would have cost ~60s of agent time for a mechanical fix.

**Rams rejection** (orphan card) was legitimate and the fix was a 3-class CSS addition. The fix cycle (Rams → Deming fix → rebuild) took one additional build cycle (~2s).

**Nielsen P2 items** (icon swap Unplug→Cable, copy rewrite) were applied by Deming directly rather than sending back to Da Vinci, saving a round-trip.

## Config Change

None. The lightweight team composition (skip Turing/Kerckhoffs/Wirth for site-only data tasks) worked well — saving ~3 agent invocations with no quality risk. This approach was already validated in iteration 028.
