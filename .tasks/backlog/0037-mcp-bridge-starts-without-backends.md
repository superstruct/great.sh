# 0037 — mcp-bridge: Start and Respond to initialize When No Backends Are Found

**Priority:** P1
**Type:** bugfix
**Module:** `src/cli/mcp_bridge.rs`, `src/mcp/bridge/server.rs`
**Status:** Backlog
**Complexity:** S
**Created:** 2026-03-02

## Context

`great mcp-bridge` currently calls `anyhow::bail!` if `discover_backends`
returns an empty list (lines 127-131 of `src/cli/mcp_bridge.rs`). This causes
the process to exit 1 before reading a single byte from stdin, so any MCP
client that opens the bridge immediately loses its connection with no valid
JSON-RPC response.

The problem is most acute during onboarding: a brand-new install of great.sh
on a machine that has no AI CLIs yet (a common Docker / CI scenario) makes the
bridge completely unusable. MCP clients (Claude Desktop, Cursor, etc.) see a
dead process rather than a bridge in degraded mode.

The correct behavior is: the bridge is an MCP server. MCP servers MUST always
respond to `initialize` and `tools/list`. When no backends are present the
response to `tools/list` is an empty array; tool calls return a JSON-RPC error
(`-32601 Method not found` or a structured tool error) explaining that no
backends are installed.

**Reproduction** (Ubuntu 24.04, no AI CLIs on PATH):

```
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  | great mcp-bridge
```

Actual output:
```
Error: no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama
exit: 1
```

Expected output: a valid JSON-RPC 2.0 `initialize` response on stdout, then
the server reading further messages from stdin (returning empty `tools/list`,
or a descriptive error on tool calls).

**Code location of the hard guard:**
`src/cli/mcp_bridge.rs` lines 127-131:
```rust
let backends = discover_backends(&backend_filter);
if backends.is_empty() {
    anyhow::bail!(
        "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama"
    );
}
```

## Acceptance Criteria

- [ ] Piping an MCP `initialize` request to `great mcp-bridge` on a machine
  with no AI CLIs returns a well-formed JSON-RPC 2.0 response on stdout (i.e.
  the process does NOT exit 1 before responding). Verified with:
  `echo '{"jsonrpc":"2.0","id":1,"method":"initialize",...}' | great mcp-bridge`
  producing valid JSON on stdout.

- [ ] `tools/list` returns an empty `tools` array (not an error) when no
  backends are discovered, allowing MCP clients to connect without crashing.

- [ ] Any tool call made while no backends are available returns a JSON-RPC
  error response with a human-readable message such as
  `"No AI CLI backends found. Install at least one of: gemini, codex, claude, grok, ollama."`
  rather than silently crashing the bridge process.

- [ ] A `WARN`-level log line is emitted to stderr when the bridge starts with
  zero backends (e.g. `warn: no AI CLI backends found; bridge running in
  degraded mode`), so operators can diagnose the situation without reading
  client-side errors.

- [ ] `cargo test` continues to pass; existing unit tests in
  `src/mcp/bridge/backends.rs` are unaffected, and at least one new
  integration test (or `#[test]`) asserts that `run()` does not return an
  `Err` when no backends are present and an `initialize` message is sent.

## Files That Need to Change

- `src/cli/mcp_bridge.rs` — remove the hard `bail!` guard; pass an empty
  `Vec<BackendConfig>` to `start_bridge` with a warn-level log instead.
- `src/mcp/bridge/server.rs` — ensure `GreatBridge::new` and the tool
  dispatch path handle an empty backend list gracefully (tool calls return
  a structured error rather than panicking or unwrapping).

## Dependencies

None. All changes are within the MCP bridge module.

## Out of Scope

- Auto-installing any backend CLI (belongs in `great doctor --fix` / task 0005).
- Surfacing backend availability in `tools/list` metadata (future enhancement).
- Changing the behavior when a non-empty `--backends` filter is specified but
  none of the named binaries are found (same fix applies, but keep as a
  follow-on to keep this change small).
