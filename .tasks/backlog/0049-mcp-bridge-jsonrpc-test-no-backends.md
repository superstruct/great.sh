# 0049 — mcp-bridge JSON-RPC tests fail without backend CLIs

| Field | Value |
|---|---|
| Priority | P3 |
| Type | test |
| Module | `test-in-docker.sh`, `src/mcp/bridge/` |
| Status | backlog |
| Estimated Complexity | S |

## Problem

The comprehensive test suite sends JSON-RPC `initialize` and `tools/list` requests to `great mcp-bridge --preset <preset>`, but the bridge can't respond because no backend CLIs (claude, gemini, codex) are installed in the Docker container. The bridge exits with code 1 and produces no output.

## Failing Tests (5)

- `mcp-bridge --preset minimal gave no valid JSON-RPC response`
- `mcp-bridge --preset agent gave no valid JSON-RPC response`
- `mcp-bridge --preset research gave no valid JSON-RPC response`
- `mcp-bridge --preset full gave no valid JSON-RPC response`
- `mcp-bridge tools/list gave no valid response`

Plus 4 warnings for tool count checks.

## Proposed Fix

Either:
1. **Mock backend** — Add a `--dry-run` or `--mock` flag to mcp-bridge that responds with canned JSON-RPC responses without spawning real backends
2. **Update test expectations** — Change these tests to expect graceful failure (non-zero exit, no crash) when backends are unavailable, rather than expecting valid JSON-RPC responses
3. **Install a mock server** — Add a simple echo server script to Docker that mimics a backend CLI

Option 2 is the simplest; option 1 is the most useful long-term.

## Evidence

Docker test run 2026-03-04 on Ubuntu 24.04. All 5 mcp-bridge JSON-RPC tests fail because no backend CLIs are available.
