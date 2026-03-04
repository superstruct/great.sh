# Release Notes — Task 0043

## Fixed

- `great status --json` now correctly sets `has_issues: true` and populates
  the `issues` array when MCP server commands are not found on PATH.

  Previously, missing MCP commands were reflected only in the per-server
  `mcp` array (`command_available: false`). The top-level `has_issues` flag
  remained `false`, causing CI scripts that gate on that field to silently
  miss MCP configuration problems.

## Added

- Integration tests covering the MCP-unavailable path in both human-readable
  and JSON output modes.
