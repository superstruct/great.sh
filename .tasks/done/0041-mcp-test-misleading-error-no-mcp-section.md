# 0041 — `great mcp test <name>` shows wrong error when no [mcp] section exists

- **Priority:** P3
- **Type:** bugfix
- **Module:** `src/cli/mcp.rs` — `run_test()`
- **Status:** selected (iter 037)
- **Estimated Complexity:** XS (single conditional, ~5 lines)

## Context

`run_test()` has an early-exit guard at the top of the function:

```rust
if mcps.is_empty() {
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}
```

This fires before the per-name lookup, so when the user runs `great mcp test myserver` against a config with no `[mcp]` section, the specific name is silently ignored and they see:

```
⚠ No MCP servers declared in great.toml.
```

The correct message — "MCP server 'myserver' not found in great.toml" — is already produced at line ~192 but is unreachable when the map is empty. The empty-map guard must be skipped (or made name-aware) when a specific name is supplied.

Verified in: Ubuntu 24.04 Docker container using `great init --template ai-minimal` (template has no MCP servers).

## Acceptance Criteria

1. `great mcp test nonexistent_xyz` (no `[mcp]` section in config) prints an error referencing the server name, e.g. `MCP server 'nonexistent_xyz' not found in great.toml`, and exits 0.
2. `great mcp test nonexistent_xyz` (some MCP servers exist, but not this one) continues to print the same name-specific error (no regression).
3. `great mcp test` (no name, no `[mcp]` section) continues to print the generic "No MCP servers declared" warning (no regression).
4. `great mcp test` (no name, servers exist) continues to test all servers (no regression).

## Files That Need to Change

- `/home/isaac/src/sh.great/src/cli/mcp.rs` — `run_test()`: move or guard the `mcps.is_empty()` early-exit so it only fires when `name` is `None`.

## Dependencies

None.

## Out of Scope

- Changing exit codes (tracked separately in task 0040).
- Modifying `run_list()` or `run_add()`.
