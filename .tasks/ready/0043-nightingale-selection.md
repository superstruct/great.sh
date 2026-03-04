# Nightingale Selection — Task 0043

**Selected task:** 0043 — Status MCP Test Coverage
**Date:** 2026-03-04
**Selected by:** Florence Nightingale (Requirements Curator)

## Selection Rationale

The backlog contains one task. Task 0043 is unblocked: its only dependency
(task 0042) shipped in iteration 039. Scope is XS — two test functions and a
one-line fix in `run_json()`.

The task also surfaces a latent bug: the MCP closure in `run_json()` populates
`command_available: false` but never appends to `issues`, leaving `has_issues`
false when an MCP command is missing. This contradicts the contract documented
in the status command's header comment and would mislead any CI script
consuming `great status --json`. The fix is a single `issues.push(...)` call
mirroring the pattern already used for missing tools (lines 322–323 of
`src/cli/status.rs`).

## Scope Summary for Lovelace

Concrete changes required:

1. **`src/cli/status.rs`, `run_json()`, lines 357–369** — inside the MCP
   closure, after computing `command_available`, add:
   ```rust
   if !command_available {
       issues.push(format!("MCP server '{}' command '{}' not found", name, m.command));
   }
   ```
   Note: the closure currently borrows `issues` immutably via `and_then` /
   `map`. To mutate `issues` inside it, the MCP block will need to be
   restructured to use explicit `if-let` (matching the tools block pattern
   above it).

2. **`tests/cli_smoke.rs`, Status section** — add two tests:
   - `status_mcp_missing_command_shows_not_found`: write a `great.toml` with
     `[mcp.fake-mcp] command = "nonexistent_mcp_status_xyz_11111"`, run
     `great status`, assert exit 0, stderr contains `"not found"`, stderr
     contains `"great doctor"`.
   - `status_json_mcp_missing_sets_has_issues`: same config, run
     `great status --json`, assert exit 0, stdout contains
     `"has_issues": true`.

All five acceptance criteria in `.tasks/backlog/0043-status-mcp-test-coverage.md`
are testable with no environment assumptions beyond a command that does not
exist on PATH.
