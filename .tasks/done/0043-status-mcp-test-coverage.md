# 0043 — Status MCP Test Coverage

| Field | Value |
|---|---|
| Priority | P3 |
| Type | test |
| Module | `tests/cli_smoke.rs` |
| Status | backlog |
| Estimated Complexity | XS |

## Context

Task 0042 (iteration 039) added a `great doctor` hint to `great status` when
`has_issues` is true. The `has_issues` flag is set in three places inside
`run()` in `src/cli/status.rs`:

- Line 191 / 210 — missing CLI tool
- Line 254 — MCP server command not found on PATH
- Line 269 — required secret not set

The existing integration tests cover the tools path (via `nonexistent_tool_*`)
and implicitly the secrets path, but no test exercises the MCP server branch.
The gap is twofold:

1. Human-readable mode: no test configures a `[mcp]` server with a
   nonexistent command and verifies the "not found" line or the doctor hint.
2. JSON mode: `run_json()` builds `McpStatus` entries with a
   `command_available: false` field but never pushes to `issues`, so
   `has_issues` stays `false` even when an MCP command is missing. No test
   catches this bug.

## Acceptance Criteria

1. A new test `status_mcp_missing_command_shows_not_found` writes a
   `great.toml` declaring one MCP server whose command does not exist, runs
   `great status`, and asserts stderr contains `"not found"`.

2. The same test (or a sibling assertion) confirms the `great doctor` hint
   appears in stderr (`"great doctor"`).

3. Both assertions above pass with exit code 0.

4. A new test `status_json_mcp_missing_sets_has_issues` writes an equivalent
   `great.toml`, runs `great status --json`, and asserts stdout contains
   `"has_issues": true`.

5. Criterion 4 currently fails (documenting the bug in `run_json()`); the fix
   is to push to `issues` inside the MCP closure in `run_json()` when
   `command_available` is false — mirroring the tools path at lines 322–323.

## Files That Need to Change

- `tests/cli_smoke.rs` — add two test functions under the `// Status` section
- `src/cli/status.rs` — fix `run_json()` MCP closure to push to `issues` when
  `command_available` is false (prerequisite for criterion 4 to pass)

## Dependencies

- Task 0042 (done, iteration 039) — doctor hint must already be present in
  `run()` for criterion 2 to be meaningful

## Out of Scope

- Changing exit codes for `great status`
- Testing MCP servers whose command exists on PATH
- Testing the `--verbose` MCP output format
