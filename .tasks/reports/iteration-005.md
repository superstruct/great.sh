# Observer Report — Iteration 005

**Date:** 2026-02-24
**Observer:** W. Edwards Deming
**Task:** 0002 — TOML Config Parser — great.toml Schema

## Summary

Foundation layer: enriched the great.toml config schema with missing fields (`version`, `api_key`, `enabled`), added `Default` derives to 4 structs, expanded `validate()` with 5 new checks, expanded `find_secret_refs()` to scan agent API keys, and added 19 new unit tests. Updated 17 downstream construction sites across 3 files.

## Changes Committed

**Commit:** `a55c3c1` — `feat(config): enrich great.toml schema with version, api_key, enabled fields`

| File | Change |
|------|--------|
| `src/config/schema.rs` | New fields, Default derives, expanded validate(), expanded find_secret_refs(), 19 new tests |
| `src/cli/init.rs` | 4 construction site updates (ProjectConfig, AgentConfig x3, McpConfig) |
| `src/cli/template.rs` | 10 construction site updates (ProjectConfig x3, AgentConfig x4, McpConfig x3) |
| `src/mcp/mod.rs` | 5 McpConfig construction site updates |

## Agent Performance

| Agent | Role | Retries | Result |
|-------|------|---------|--------|
| Nightingale | Requirements | 0 | PASS — selected 0002 (P0 foundation, 0001 dep done) |
| Lovelace | Spec | 0 | PASS — 998-line spec covering all enrichments |
| Socrates | Review gate | 0 | APPROVED — 8 advisory, 0 blocking |
| Humboldt | Scout | 0 | PASS — mapped 16 construction sites, identified tech debt |
| Da Vinci (Deming) | Build | 1 | R1 missed 3 ProjectConfig + 1 init.rs ProjectConfig sites; R2 fixed |
| Turing | Test | 0 | FAIL (compilation) → identified 3 ProjectConfig sites → PASS after fix |
| Kerckhoffs | Security | 0 | BLOCK (same compilation issue) + 3 advisory → PASS after fix |
| Wirth | Performance | 0 | PASS — 9.97 MiB (-0.11%), no new deps |
| Dijkstra | Code review | 0 | APPROVED — 4 WARN advisory (regex duplication, double iteration, transport/url, error message) |
| Nielsen | UX | N/A | Skipped — no UI change |
| Rams | Visual | N/A | Skipped — no visual component |
| Hopper | Commit | 0 | Committed a55c3c1 |

## Build Fix Cycle

- Turing and Kerckhoffs both independently identified the same CRITICAL issue: 3 `ProjectConfig` construction sites in `template.rs` and 1 in `init.rs` missing the new `version` field
- Root cause: Da Vinci searched for `AgentConfig`, `McpConfig` construction sites (found and fixed all) but missed `ProjectConfig` construction sites because the spec's downstream compatibility matrix did not list `ProjectConfig` as needing updates (it was omitted)
- **Spec gap**: Lovelace's spec listed downstream impacts for `AgentConfig` and `McpConfig` but not `ProjectConfig` — the `init.rs` ProjectConfig at line 61 was not in the spec's "Files to Modify" table
- Fix was trivial: add `..Default::default()` to 4 ProjectConfig literals

## Bottleneck

**Spec coverage gap for ProjectConfig construction sites.** The Lovelace spec correctly identified all `AgentConfig` (7 sites) and `McpConfig` (9 sites) construction sites but missed `ProjectConfig` (4 sites). This caused a build failure caught by Turing/Kerckhoffs, requiring 1 fix cycle.

**Root cause**: The spec's downstream compatibility matrix focused on the new `api_key`/`enabled` fields (which only affect AgentConfig/McpConfig) but did not account for the `version` field added to ProjectConfig, which also needs construction site updates.

## Metrics

- **Files changed:** 4
- **Lines added:** 405
- **Lines removed:** 5
- **Tests added:** 19 new unit tests
- **Tests total:** 239 (182 unit + 57 integration)
- **Agent retries:** 1 (build fix for missed construction sites)
- **Blocking issues found in review:** 1 (compilation failure from missed ProjectConfig sites)
- **Non-blocking issues:** 7 (3 Kerckhoffs advisory, 4 Dijkstra WARN)
- **Build status:** GREEN
- **Binary size:** 9.97 MiB (-0.11% from 9.98 MiB baseline)

## Advisory Issues for Backlog

### Kerckhoffs
- MEDIUM (P2): `AgentConfig` derives `Debug` — literal api_key values would leak via `{:?}`. Consider custom Debug impl that redacts api_key.
- LOW (P3): `find_secret_refs()` doesn't scan `McpConfig.url` or `McpConfig.args` for secret patterns.
- LOW (P3): `enabled = false` has no production consumer — `apply.rs` doesn't check it before writing MCP entries.

### Dijkstra
- WARN: Regex pattern `\$\{([A-Z_][A-Z0-9_]*)\}` compiled fresh in 3 separate locations — should use `LazyLock`.
- WARN: Agent validation iterates the agents map twice (provider check + whitelist check).
- WARN: SSE transport doesn't require URL but HTTP does — intent undocumented.
- WARN: Secret name validation error message doesn't explain leading-character constraint.

### Wirth (tech debt)
- NOTE: Regex compiled per call in `find_secret_refs()` — should be `OnceLock`.

## Config Change

**None.** The build fix cycle was caused by a spec coverage gap (missed ProjectConfig sites), not a pipeline configuration issue. The Turing/Kerckhoffs parallel review correctly caught it before commit. The pipeline is functioning as designed — parallel review catches build issues efficiently.

**Observation for next iteration:** If the next task also adds fields to existing structs, Lovelace should be explicitly prompted to audit ALL struct construction sites, not just the ones receiving the most new fields. A `grep -n 'StructName {'` across the codebase should be standard practice in the spec's downstream compatibility section.
