# 0037: MCP Bridge Degraded Mode (No Backends) -- Specification

**Author:** Ada Lovelace (Spec Writer)
**Task:** `.tasks/backlog/0037-mcp-bridge-starts-without-backends.md`
**Date:** 2026-03-03
**Complexity:** S (two files changed, one test added, no new dependencies)

---

## 1. Problem Statement

`great mcp-bridge` calls `anyhow::bail!` at `src/cli/mcp_bridge.rs` line 127-131
when `discover_backends` returns an empty list. This terminates the process with
exit code 1 before reading stdin, so MCP clients (Claude Desktop, Cursor, etc.)
receive no JSON-RPC response and see a dead connection. The bug is most acute
during onboarding: a fresh great.sh install in Docker or CI with no AI CLIs on
PATH makes the bridge completely unusable. Per the MCP specification, an MCP
server MUST always respond to `initialize` and `tools/list`. The correct behavior
is: start the bridge, respond to `initialize` normally, return an empty `tools`
array from `tools/list`, and return a structured tool-level error (not a JSON-RPC
protocol error) on any tool call, explaining that no backends are installed.

---

## 2. Solution Design

### Design Decisions

**Q: Should `tools/list` return an empty array or list tools that return errors?**
A: Empty array. The `#[tool_router]` macro on `GreatBridge` statically registers
all tools (prompt, run, wait, etc.) via `Self::tool_router()`, but the existing
`list_tools` override in `ServerHandler for GreatBridge` already filters tools
through `self.preset.tool_names()`. When backends are empty, we add one additional
filter: if `self.backends.is_empty()`, return an empty tools vec. This is the
cleanest approach because: (a) MCP clients will not attempt to call tools they
cannot see, (b) it requires no new tool metadata, and (c) it matches the MCP
spec guidance that `tools/list` reflects available capabilities.

**Q: What error should tool calls return when no backends are available?**
A: A `CallToolResult::error(...)` with `is_error: true` and a human-readable
Content::text message. This is a *tool-level* error (HTTP 200, JSON-RPC success,
but `isError: true` in the result), NOT a JSON-RPC protocol error. Rationale:
the tool was found (the router has it registered), but it cannot execute because
no backends are available. Using `CallToolResult::error` is consistent with how
all other tool-level failures in the bridge are reported (e.g., backend spawn
failures, timeout errors). The existing `resolve_backend` method already returns
`Err(McpError::invalid_params("no backends available", None))` which propagates
as a JSON-RPC error -- we will change this to return a `CallToolResult::error`
instead, keeping the tool dispatch within the success path.

**Q: What JSON-RPC error code?**
A: Not applicable. We do NOT return a JSON-RPC error. Tool calls return a
successful JSON-RPC response with `isError: true` in the `CallToolResult` body.
If somehow a tool call reaches the router for a tool name that is not registered
(e.g., because `tools/list` was empty and the client called a tool by name
anyway), the `ToolRouter::call` method already returns
`ErrorData::invalid_params("tool not found", None)` which maps to JSON-RPC
error code `-32602`. No change needed for that path.

### Approach Summary

1. **Remove the bail guard** in `mcp_bridge.rs` -- replace with `tracing::warn!`.
2. **Adjust log message** for discovered backends to handle the empty case.
3. **Guard `list_tools`** in `server.rs` to return empty tools when backends is empty.
4. **Guard `call_tool`** in `server.rs` to return a `CallToolResult::error` before
   routing when backends is empty.
5. **Add an integration test** in `tests/cli_smoke.rs` that sends an `initialize`
   request to `great mcp-bridge` on a machine with no backends and asserts the
   process exits 0 with valid JSON on stdout.

---

## 3. Exact Changes Per File

### File 1: `src/cli/mcp_bridge.rs`

**Change A: Remove bail guard (lines 126-131)**

Current code:
```rust
    // Discover backends
    let backends = discover_backends(&backend_filter);
    if backends.is_empty() {
        anyhow::bail!(
            "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama"
        );
    }
```

Replace with:
```rust
    // Discover backends
    let backends = discover_backends(&backend_filter);
    if backends.is_empty() {
        tracing::warn!(
            "no AI CLI backends found on PATH; bridge starting in degraded mode. \
             Install at least one of: gemini, codex, claude, grok, ollama"
        );
    }
```

**Change B: Adjust info log (lines 133-140)**

Current code:
```rust
    tracing::info!(
        "Discovered backends: {}",
        backends
            .iter()
            .map(|b| b.name)
            .collect::<Vec<_>>()
            .join(", ")
    );
```

Replace with:
```rust
    if !backends.is_empty() {
        tracing::info!(
            "Discovered backends: {}",
            backends
                .iter()
                .map(|b| b.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
```

This avoids emitting `Discovered backends: ` (empty string) which would be
confusing alongside the warn message above.

### File 2: `src/mcp/bridge/server.rs`

**Change C: Constant for the degraded-mode error message**

Add after the existing constants at lines 17-21:

```rust
/// Human-readable error message returned by tool calls when no backends are available.
const NO_BACKENDS_MSG: &str =
    "No AI CLI backends found. Install at least one of: gemini, codex, claude, grok, ollama. \
     Run `great doctor` to check backend availability.";
```

**Change D: Guard `list_tools` override (lines 367-383)**

Current code:
```rust
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let allowed = self.preset.tool_names();
        let all_tools = self.tool_router.list_all();
        let filtered: Vec<Tool> = all_tools
            .into_iter()
            .filter(|t| allowed.contains(&t.name.as_ref()))
            .collect();
        std::future::ready(Ok(ListToolsResult {
            meta: None,
            tools: filtered,
            next_cursor: None,
        }))
    }
```

Replace with:
```rust
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        // When no backends are available, return an empty tools list.
        // MCP clients will see zero capabilities but maintain a valid connection.
        if self.backends.is_empty() {
            return std::future::ready(Ok(ListToolsResult {
                meta: None,
                tools: vec![],
                next_cursor: None,
            }));
        }

        let allowed = self.preset.tool_names();
        let all_tools = self.tool_router.list_all();
        let filtered: Vec<Tool> = all_tools
            .into_iter()
            .filter(|t| allowed.contains(&t.name.as_ref()))
            .collect();
        std::future::ready(Ok(ListToolsResult {
            meta: None,
            tools: filtered,
            next_cursor: None,
        }))
    }
```

**Change E: Guard `call_tool` override (lines 388-395)**

Current code:
```rust
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let context = ToolCallContext::new(self, request, context);
        self.tool_router.call(context).await
    }
```

Replace with:
```rust
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        // When no backends are available, return a tool-level error rather than
        // routing to a handler that will fail on resolve_backend().
        if self.backends.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(NO_BACKENDS_MSG)]));
        }

        let context = ToolCallContext::new(self, request, context);
        self.tool_router.call(context).await
    }
```

Note: This returns a `CallToolResult` with `is_error: Some(true)`, which is a
successful JSON-RPC response (the tool was invoked but reported an error). This
is distinct from returning `Err(McpError::...)` which would produce a JSON-RPC
error response. The tool-level error approach is correct here because:
- The `call_tool` method was successfully dispatched (the server is running)
- The tool itself cannot operate (no backends)
- MCP clients handle `isError: true` gracefully and display the message to the user

### File 3: `tests/cli_smoke.rs`

**Change F: Add integration test for degraded-mode startup**

Append after the existing `mcp_bridge_unknown_preset_shows_error_message` test
(line 1970):

```rust
/// Verify that `great mcp-bridge` starts and responds to `initialize` even when
/// no AI CLI backends are on PATH. The bridge must NOT bail! with exit code 1.
///
/// We send three newline-delimited JSON-RPC messages:
///   1. initialize request
///   2. notifications/initialized notification
///   3. tools/list request
///
/// Then close stdin (via write_stdin). The bridge should produce valid JSON-RPC
/// responses for (1) and (3) on stdout and exit 0.
#[test]
fn mcp_bridge_starts_without_backends() {
    // Use a PATH with no AI CLI binaries to guarantee zero backends.
    // Setting PATH to a minimal value ensures discover_backends returns empty.
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        "\n",
    );

    let output = great()
        .args(["mcp-bridge", "--preset", "minimal", "--log-level", "off"])
        .env("PATH", "/nonexistent")
        .write_stdin(input)
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("failed to execute mcp-bridge");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must exit successfully (not bail!)
    assert!(
        output.status.success(),
        "mcp-bridge should exit 0 in degraded mode, got: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    // stdout must contain a valid initialize response with protocolVersion
    assert!(
        stdout.contains("protocolVersion"),
        "initialize response missing protocolVersion. stdout: {}",
        stdout,
    );

    // stdout must contain a tools/list response with an empty tools array
    assert!(
        stdout.contains(r#""tools":[]"#) || stdout.contains(r#""tools": []"#),
        "tools/list should return empty tools array. stdout: {}",
        stdout,
    );
}

/// Verify that stderr contains a degraded-mode warning when no backends are found.
#[test]
fn mcp_bridge_no_backends_emits_warning() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
        "\n",
    );

    let output = great()
        .args(["mcp-bridge", "--preset", "minimal", "--log-level", "warn"])
        .env("PATH", "/nonexistent")
        .write_stdin(input)
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("failed to execute mcp-bridge");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("degraded mode") || stderr.contains("no AI CLI backends"),
        "stderr should contain degraded mode warning. stderr: {}",
        stderr,
    );
}
```

### File 4: `tests/mcp_bridge_protocol.sh` (informational, no change required)

The existing shell script at line 7 has the comment "Requires at least one AI CLI
backend on PATH." This remains valid for the *positive* test path. The new Rust
integration tests in `cli_smoke.rs` cover the *negative* (no-backends) path. No
changes needed to this file, but the builder may optionally add a second test case
to this script.

---

## 4. Error Response Format

When a tool call arrives with no backends available, the JSON-RPC response on
stdout will be:

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

This is a **successful** JSON-RPC response (no `"error"` key at the top level).
The `isError: true` flag in the `result` object signals to the MCP client that
the tool call failed. MCP clients (Claude Desktop, Cursor) display the text
content as an error message to the user.

This is distinct from a JSON-RPC protocol error, which would look like:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "no backends available"
  }
}
```

We explicitly do NOT use the protocol-error form because:
1. The tool exists (it is registered in the router)
2. The server is functioning correctly
3. The failure is an operational condition (missing backends), not a protocol violation
4. Tool-level errors are the idiomatic MCP pattern for "tool ran but failed"

---

## 5. Test Plan

### Integration Tests (added to `tests/cli_smoke.rs`)

| Test Name | What It Verifies |
|---|---|
| `mcp_bridge_starts_without_backends` | Process exits 0, stdout contains valid `initialize` response with `protocolVersion`, and `tools/list` returns empty `tools` array |
| `mcp_bridge_no_backends_emits_warning` | stderr contains the degraded-mode warning at WARN level |

### Manual Protocol Test

Run the existing `tests/mcp_bridge_protocol.sh` on a machine WITH backends to
confirm no regression. Then run the following manually with no backends on PATH:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"prompt","arguments":{"prompt":"hello"}}}' \
  | PATH=/nonexistent cargo run -- mcp-bridge --preset full --log-level warn 2>/tmp/bridge-stderr.log
```

Expected:
- Three JSON-RPC responses on stdout (initialize, tools/list, tools/call)
- tools/list has `"tools": []`
- tools/call has `"isError": true` and the NO_BACKENDS_MSG text
- /tmp/bridge-stderr.log contains "degraded mode"

### Existing Tests (regression)

- `cargo test` -- all existing tests in `tests/cli_smoke.rs` must pass unchanged
- `cargo clippy` -- no new warnings

### Platform Coverage

The integration tests use `env("PATH", "/nonexistent")` which works identically
on all target platforms:
- **Linux (Ubuntu, WSL2):** PATH override prevents `which` from finding any binary
- **macOS (ARM64, x86_64):** Same behavior; `which` crate uses PATH env var

No platform-specific code paths are affected by this change.

---

## 6. Acceptance Criteria Checklist

- [ ] Piping an MCP `initialize` request to `great mcp-bridge` on a machine with
      no AI CLIs returns a well-formed JSON-RPC 2.0 response on stdout (process
      does NOT exit 1 before responding).

- [ ] `tools/list` returns an empty `tools` array (not an error) when no backends
      are discovered.

- [ ] Any tool call made while no backends are available returns a JSON-RPC success
      response with `isError: true` and a human-readable message including
      "No AI CLI backends found" and mentioning `great doctor`.

- [ ] A WARN-level log line is emitted to stderr when the bridge starts with zero
      backends, containing "degraded mode".

- [ ] `cargo test` passes, including the two new integration tests:
      `mcp_bridge_starts_without_backends` and `mcp_bridge_no_backends_emits_warning`.

- [ ] `cargo clippy` produces no new warnings.

- [ ] Existing `mcp_bridge_help_shows_description`, `mcp_bridge_unknown_preset_fails`,
      and `mcp_bridge_unknown_preset_shows_error_message` tests continue to pass.

---

## 7. Out of Scope

- **Auto-installing backend CLIs** -- belongs in `great doctor --fix` (task 0005).
- **Surfacing backend availability in `tools/list` metadata** -- future enhancement;
  tools are simply omitted for now.
- **Changing behavior when a non-empty `--backends` filter matches nothing** -- the
  same fix applies naturally (discover_backends returns empty, bridge starts in
  degraded mode), but explicit test coverage for this case is deferred.
- **Dynamic backend hot-reload** -- detecting newly-installed backends at runtime
  without restarting the bridge is a separate feature.
- **Changes to `src/mcp/bridge/backends.rs`** -- no modifications needed; the
  `discover_backends` function correctly returns an empty `Vec` when no backends
  are found, which is the desired behavior.

---

## Build Order

This is a single-step change with no internal dependencies:

1. **Modify `src/cli/mcp_bridge.rs`** (Changes A, B) -- remove bail, add warning
2. **Modify `src/mcp/bridge/server.rs`** (Changes C, D, E) -- add constant, guard list_tools and call_tool
3. **Modify `tests/cli_smoke.rs`** (Change F) -- add two integration tests
4. Run `cargo test` and `cargo clippy` to verify

All three files can be modified in parallel since they have no compile-time
interdependencies (the test binary depends on the library, but the changes are
additive and do not alter any existing public interface).

---

## Interfaces (Full Type Signatures)

No new public types, traits, or functions are introduced. The changes are to
existing method bodies only.

### Modified Functions

```rust
// src/cli/mcp_bridge.rs
pub fn run(args: Args) -> Result<()>
// Signature unchanged. Body removes bail! guard.
```

```rust
// src/mcp/bridge/server.rs -- ServerHandler impl
fn list_tools(
    &self,
    _request: Option<PaginatedRequestParams>,
    _context: RequestContext<RoleServer>,
) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_
// Signature unchanged. Body adds early return for empty backends.
```

```rust
// src/mcp/bridge/server.rs -- ServerHandler impl
async fn call_tool(
    &self,
    request: CallToolRequestParams,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError>
// Signature unchanged. Body adds early return for empty backends.
```

### New Constants

```rust
// src/mcp/bridge/server.rs
const NO_BACKENDS_MSG: &str =
    "No AI CLI backends found. Install at least one of: gemini, codex, claude, grok, ollama. \
     Run `great doctor` to check backend availability.";
```

---

## Edge Cases

| Scenario | Expected Behavior |
|---|---|
| Zero backends, `initialize` request | Normal `initialize` response with full `ServerInfo` |
| Zero backends, `tools/list` request | `{"tools": []}` (empty array, not an error) |
| Zero backends, `tools/call` for any tool name | `CallToolResult` with `isError: true` and `NO_BACKENDS_MSG` |
| Zero backends, `ping` request | Normal pong response (unaffected) |
| `--backends codex` but codex not on PATH | Same as zero backends (discover returns empty) |
| Backends become available after startup | Not handled (bridge must be restarted; out of scope) |
| Multiple JSON-RPC messages in a single stdin stream | All handled normally; degraded mode is per-request |
| Client sends `tools/call` for a tool NOT in the router | `ToolRouter::call` returns `ErrorData::invalid_params("tool not found")` -- unchanged, but this path is only reachable if the call_tool guard is bypassed; since the guard checks `self.backends.is_empty()` and returns before reaching the router, it will return `NO_BACKENDS_MSG` instead |

---

## Security Considerations

- **No new attack surface.** The change removes a crash path and adds graceful
  error handling. The bridge in degraded mode cannot execute any backend commands
  because no backends are available.
- **No information leakage.** The error message lists the supported backend names
  (gemini, codex, claude, grok, ollama), which are already public and documented.
- **Denial of service.** A client could send many tool calls to a degraded bridge.
  This is harmless because the error response is computed synchronously with no
  subprocess spawning, making it cheaper than a normal tool call.
- **PATH manipulation in tests.** The test uses `env("PATH", "/nonexistent")` which
  is safe in the assert_cmd subprocess and does not affect the test runner.
