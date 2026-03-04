# Release Notes — Task 0037: MCP Bridge Degraded Mode (No Backends)

**Date:** 2026-03-04
**Scope:** `src/cli/mcp_bridge.rs`, `src/mcp/bridge/server.rs`, `tests/cli_smoke.rs`
**Cargo version:** 0.1.0 (no version bump; bugfix iteration)

---

## Summary

`great mcp-bridge` no longer exits with an error when no AI CLI backends are
installed. The bridge now starts in degraded mode, responds to all MCP protocol
messages, and returns actionable error text when a tool is called without
backends present.

---

## User-facing changes

### Bridge starts without crashing when no backends are installed

Previously, running `great mcp-bridge` on a machine with no AI CLI tools
(gemini, codex, claude, grok, ollama) on PATH produced:

```
Error: no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama
```

and the process exited 1 before reading a single byte from stdin. Any MCP
client (Claude Desktop, Cursor, VS Code extensions) that opened the bridge
immediately lost its connection with no JSON-RPC response.

After this fix, the bridge starts normally and emits a warning to stderr:

```
WARN great: No AI CLI backends found on PATH; bridge starting in degraded mode.
     Install at least one of: gemini, codex, claude, grok, ollama
```

The bridge then reads MCP messages from stdin and responds correctly.

This is most important during onboarding: a fresh `great init` on a machine
that has not yet installed any AI CLIs, or a Docker/CI image with only the
`great` binary present, will no longer produce a dead MCP connection.

### `tools/list` returns an empty array in degraded mode

When no backends are available, the `tools/list` response is:

```json
{"jsonrpc":"2.0","id":2,"result":{"tools":[]}}
```

MCP clients will display "no tools available" rather than an error. The
connection remains open and valid.

### Tool calls return an actionable error message

If a client calls a tool by name while the bridge is in degraded mode (for
example, because the client cached a previous `tools/list`), the response is
a tool-level error with a human-readable explanation:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "No AI CLI backends found. Install at least one of: gemini, codex, claude, grok, ollama. Run `great doctor` to check backend availability."
      }
    ],
    "isError": true
  }
}
```

This is a successful JSON-RPC response (no top-level `"error"` key). The
`isError: true` flag signals to the MCP client that the tool could not run.
The message directs the user to `great doctor`, which shows which backends are
missing and how to install them.

---

## No migration needed

No configuration changes are required. `great.toml` files and `.mcp.json`
registrations are unaffected. The behavior change is automatic on upgrade.

Users who previously received the startup error must have had no backends
installed. After upgrading, the bridge will start and they can follow the
`great doctor` output to install a backend when ready.

---

## Reproduction (before fix)

```bash
# Ubuntu 24.04, no AI CLIs on PATH
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  | great mcp-bridge
# Output: Error: no AI CLI backends found on PATH ...
# Exit: 1
```

## Verification (after fix)

```bash
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | PATH=/nonexistent great mcp-bridge --preset minimal --log-level warn
```

Expected stdout: two JSON-RPC responses (initialize + tools/list with empty
`"tools":[]` array). Expected stderr: one WARN line containing "degraded mode".
Expected exit: 0.
