# 0051 — `great mcp add` hardcodes npx @modelcontextprotocol/server-<name>

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bug |
| Module | `src/cli/mcp.rs` |
| Status | backlog |
| Estimated Complexity | M |

## Problem

`great mcp add <name>` writes `command = "npx"` with args
`["-y", "@modelcontextprotocol/server-<name>"]` for every server name. Most
real MCP servers are not published under that scope, so the generated config
usually requires hand-editing.

## Proposed Fix

Maintain a small registry of well-known servers (name → command/args/transport)
and fall back to writing a commented placeholder plus a warning pointing at
the config file when the name is unknown. Do not silently guess a package.

## Acceptance Criteria

- Known names (e.g. filesystem, memory, playwright, context7) produce working configs
- Unknown names produce an explicit warning and a placeholder that fails validation until edited
- Test coverage for both paths
