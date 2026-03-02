# Iteration 031 — Observer Report

**Date:** 2026-02-28
**Task:** 0034 — Wire mcp-bridge into init wizard and built-in templates
**Observer:** W. Edwards Deming

---

## Task Completed

The `great init` wizard now includes an opt-in MCP Bridge section that asks "Enable built-in MCP bridge (routes MCP servers to all AI agents)?" (default: no). When enabled, it writes `[mcp-bridge]` with `preset = "minimal"` to the generated `great.toml`. All four built-in templates now ship with `[mcp-bridge]` stanzas: `minimal` for ai-minimal, `agent` for ai-fullstack-ts and ai-fullstack-py, `full` for saas-multi-tenant.

This closes the last onboarding gap for the MCP bridge feature shipped in iterations 026–030.

## Commit

- `f7177b0` — feat(init): wire mcp-bridge into init wizard and templates

## Files Changed

| File | Action | Lines |
|------|--------|-------|
| `src/cli/init.rs` | Modify | +66 (wizard section + 2 tests) |
| `templates/ai-minimal.toml` | Modify | +3 |
| `templates/ai-fullstack-ts.toml` | Modify | +3 |
| `templates/ai-fullstack-py.toml` | Modify | +3 |
| `templates/saas-multi-tenant.toml` | Modify | +3 |

## Agent Performance

| Agent | Role | Retries | Notes |
|-------|------|---------|-------|
| Nightingale | Task creation | 0 | Backlog empty; surveyed 33 done tasks, created 0034 from onboarding gap analysis |
| Lovelace | Spec writer | 0 | Clean spec with copy-paste code, all line numbers verified |
| Socrates | Spec reviewer | 0 | APPROVED, 3 advisory (non-interactive gap, intent test, no integration test) |
| Humboldt | Codebase scout | 0 | Full map with exact line numbers, all insertion points correct |
| Da Vinci | Builder | 1 | Build complete first pass; 1 fix cycle for Nielsen prompt wording blocker |
| Turing | Tester | 0 | ALL PASS, zero failures across 329 tests + clippy |
| Kerckhoffs | Security | 0 | CLEAN, no findings at any severity |
| Nielsen | UX | 0 | 1 blocker (prompt wording), 2 non-blocking (P2: preset descriptions, P3: --help missing saas-multi-tenant) |
| Wirth | Performance | 0 | PASS, baseline: 8.556 MiB binary, 330 tests in 1.8s |
| Dijkstra | Code quality | 0 | APPROVED, 2 WARNs (preset info ordering, wizard preset vs multi-agent context) |
| Rams | Visual | 0 | APPROVED — consistent rhythm, correct defaults |
| Hopper | Committer | 0 | Clean commit, 5 files staged |
| Knuth | Docs | 0 | Release notes written |

## Metrics

- Build: `cargo test` exits 0 — 231 unit + 97 integration + 1 hook = 329 total (+2 new)
- Clippy: clean (0 warnings)
- Binary size: 8.556 MiB (unchanged — S-complexity text addition)
- Files changed: 5 (0 new + 5 modified)
- Lines added: ~78

## Bottleneck

**Nielsen blocker was the only re-work cycle.** Da Vinci's initial prompt wording ("Enable the built-in MCP bridge (great mcp-bridge)?") was not self-describing for new users. Nielsen caught this correctly — every other wizard prompt explains what the feature does. Fix was a one-line string change. Total delay: ~1 minute.

**Nightingale backlog survey remained the longest sequential step** (~2 min) — same pattern as iteration 030. With 33 completed tasks to survey, this is expected.

## Follow-up Items (Non-blocking)

- P2 (Nielsen): Preset names in wizard info message lack descriptions — user can't evaluate their choice
- P3 (Nielsen): `--help` doc comment for `--template` flag missing `saas-multi-tenant`
- INFO (Dijkstra): Wizard hardcodes `preset = "minimal"` even if user configured multiple agents earlier in the same wizard run; semantically `"agent"` would be better in that case
- INFO (Rams): Single-quote inconsistency in success message vs info message

## Config Change

None. The full team composition was used for this iteration. For future S-complexity CLI-only tasks, the lightweight composition (skip Wirth for non-perf-sensitive changes) could save ~2 min, but the baseline measurement from Wirth was valuable as a first post-bridge iteration.
