# Iteration 026 — Observer Report

**Date:** 2026-02-27
**Task:** 0029 — Inbuilt MCP Bridge Server
**Observer:** W. Edwards Deming

---

## Task Completed

New `great mcp-bridge` subcommand: a stdio MCP server (JSON-RPC 2.0) bridging 5 AI CLI backends (gemini, codex, claude, grok, ollama) into a unified tool surface. Eliminates the Node.js/npm dependency previously required for MCP bridge servers. Uses the `rmcp` crate (official Rust MCP SDK) for protocol handling.

9 tools across 4 presets (minimal/agent/research/full), async process registry with timeout and cleanup, `great apply` auto-registration in `.mcp.json`, `great doctor` backend health checks.

## Files Changed

- 7 new files: `src/mcp/bridge/{mod,backends,registry,tools,server}.rs`, `src/cli/mcp_bridge.rs`, `tests/mcp_bridge_protocol.sh`
- 8 modified files: `Cargo.toml`, `src/config/schema.rs`, `src/mcp/mod.rs`, `src/cli/{mod,apply,doctor,template}.rs`, `tests/cli_smoke.rs`
- 5 new dependencies: rmcp 0.16, uuid 1.x, tracing 0.1, tracing-subscriber 0.3, schemars 1.0, libc 0.2 (unix)

## Agent Performance

| Agent | Role | Retries | Notes |
|-------|------|---------|-------|
| Nightingale | Task selection | 0 | Only 1 open task in backlog (0029) |
| Lovelace | Spec writer | 0 | 1,932-line spec produced |
| Socrates | Spec reviewer | 0 | APPROVED WITH CONDITIONS (0 blocking, 10 warnings) |
| Humboldt | Codebase scout | 0 | Mapped 15 files, exact insertion points |
| Da Vinci | Builder | 1 | Initial build complete; 1 retry cycle for UTF-8 truncation fix + security fixes + UX fixes |
| Turing | Tester | 0 | PASS — confirmed all gates green after fix |
| Kerckhoffs | Security | 0 | 1 HIGH found (UTF-8 panic), cleared after fix |
| Nielsen | UX | 0 | 3 P2 cosmetic fixes, no blockers |
| Wirth | Performance | 0 | Baseline: 10.9MB, projected +29-53% (rmcp is primary driver) |
| Dijkstra | Code quality | 0 | 3 BLOCKs + 3 WARNs, all fixed by Deming |
| Rams | Visual | 0 | 3 issues: 2 already fixed, 1 (global flags) deferred |
| Knuth | Docs | 0 | Release notes written |
| Hopper | Commit | 0 | Pending |

## Bottleneck

**Da Vinci ↔ reviewers handoff.** Da Vinci completed the build, then 3 reviewers worked in parallel. The single commit-blocking bug (UTF-8 truncation in `truncate_output`) was found independently by both Turing and Kerckhoffs — good convergent validation. Da Vinci had proactively fixed it before my relay message, suggesting the Kerckhoffs→Da Vinci direct messaging path was faster than the lead-mediated path.

## Metrics

- Quality gates: build 0 warnings, clippy 0 warnings, 319 tests (227 unit + 91 integration + 1 hook)
- Security: 1 HIGH fixed, 3 MEDIUM deferred (P2), 2 LOW deferred (P3)
- UX: 3 P2 cosmetic fixes applied
- Code quality: 3 blocks + 3 warnings fixed

## Follow-up Items (Non-blocking)

- P2: File path validation in research/analyze_code tools (`--allowed-dirs` flag)
- P2: `--dangerously-skip-permissions` doctor warning and config opt-out for Claude backend
- P2: Refactor `check_mcp_bridge()` to use `discover_backends()` instead of hardcoded list
- P3: Process group isolation for `run_sync()` (orphan grandchildren on timeout)
- P3: Registry test coverage (spawn/kill/wait/concurrent tests)
- P3: `killpg(0)` guard for `GREAT_*_CLI` env overrides
- MEASURE: Binary size post-build (Wirth projected +29-53%, needs actual measurement)
- DEFERRED: Global flags (`--verbose`, `--quiet`) wiring for mcp-bridge subcommand

## Config Change

None this iteration. No process bottleneck warranting a config change — the team parallelization worked well.
