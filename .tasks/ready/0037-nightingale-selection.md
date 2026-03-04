# Nightingale Selection Report — Iteration 034

**Date:** 2026-03-03
**Selected Task:** 0037 — mcp-bridge: Start and Respond to `initialize` When No Backends Are Found
**Selected By:** Florence Nightingale (Requirements Curator)

---

## 1. Selection Rationale

### Priority comparison across all five backlog tasks

| Task | Priority | Type | Complexity | Notes |
|------|----------|------|------------|-------|
| 0037 | P1 | bugfix | S | Hard crash before first byte; breaks MCP clients entirely |
| 0038 | P2 | bugfix | S | SIGPIPE panic on pipe close; bad UX but not a dead process |
| 0039 | P2 | bugfix | S | WSL false-positive inside Docker; wrong platform detection |
| 0040 | P3 | refactor | S | Exit code design disagreement; requires team decision first |
| 0041 | P3 | bugfix | XS | Wrong error message in mcp test; cosmetic, not functional |

**0037 is the clear selection.** It is the only P1 task in the batch. The failure mode is a complete loss of MCP protocol compliance: the bridge process exits with code 1 before reading a single byte from stdin, so any MCP client (Claude Desktop, Cursor) that attempts to connect receives no `initialize` response and treats the bridge as broken. This is not a degraded experience — it is a hard failure on any machine without AI CLIs pre-installed, which includes every Docker container, every CI runner, and every brand-new developer workstation.

The remaining tasks (0038, 0039) are P2 and unblocked, but they are polish relative to 0037's breakage. 0040 is blocked on a team design decision before implementation can begin. 0041 is cosmetic.

---

## 2. Task Requirements Summary

### Problem statement

`src/cli/mcp_bridge.rs` lines 127-131 contain a hard guard:

```rust
let backends = discover_backends(&backend_filter);
if backends.is_empty() {
    anyhow::bail!(
        "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama"
    );
}
```

This guard fires before the MCP server ever reads from stdin. The bridge is supposed to be an MCP server, and MCP servers are required to respond to `initialize` and `tools/list` regardless of internal state. The correct behavior when no backends are present is a degraded-mode server: it starts, responds to protocol negotiation, returns an empty `tools/list`, and returns a structured JSON-RPC error on tool calls explaining that no backends are installed.

### Two files require changes

**`src/cli/mcp_bridge.rs`**
- Remove the `bail!` guard.
- Replace with a `warn!`-level log: `"no AI CLI backends found; bridge running in degraded mode"`.
- Pass the empty `Vec<BackendConfig>` into `start_bridge` unchanged.

**`src/mcp/bridge/server.rs`**
- Verify that `GreatBridge::new` accepts an empty backend list without panicking (structurally it does — `backends` is stored as `Arc<Vec<BackendConfig>>`; no unwrap on first element).
- Verify that tool dispatch returns a proper JSON-RPC error (not a panic or unwrap) when the backends vec is empty.
- The `research()` method (line 159) and `analyze_code()` method (line 213) are the two dispatch paths; both need a checked guard before selecting a backend.

### Acceptance criteria (verbatim from task file)

1. Piping an MCP `initialize` request to `great mcp-bridge` on a machine with no AI CLIs returns a well-formed JSON-RPC 2.0 response on stdout. The process does NOT exit 1 before responding.
2. `tools/list` returns an empty `tools` array (not an error) when no backends are discovered.
3. Any tool call made while no backends are available returns a JSON-RPC error response with a human-readable message naming the missing backends, rather than crashing the bridge process.
4. A `WARN`-level log line is emitted to stderr when the bridge starts with zero backends.
5. `cargo test` passes; at least one new test asserts that `run()` does not return `Err` when no backends are present and an `initialize` message is sent.

---

## 3. Dependencies and Risks

**Dependencies:** None. All changes are confined to the MCP bridge module. No other subcommands, config loading, or platform detection code is involved.

**Risks:**

- **rmcp macro behavior with empty tool lists.** The `#[tool_router]` and `#[tool]` macros from the `rmcp` crate generate `tools/list` responses based on registered tools, not on backend availability. Registered tools are compile-time; backends are runtime. The empty `tools` array requirement (criterion 2) may conflict with how rmcp generates `tools/list` — it may always return the full set of registered tool names regardless. Lovelace should inspect the rmcp `ServerHandler` trait's `list_tools` method to understand whether it can be overridden to return an empty set, or whether criterion 2 needs to be interpreted as "tools/list returns the registered tools but each call returns an error." The task file is ambiguous on this point and Lovelace must resolve it before coding begins.

- **Test infrastructure for stdin MCP messages.** The new integration test must send a valid JSON-RPC `initialize` message on stdin and assert on stdout. This requires either a subprocess test with piped I/O (via `assert_cmd` + `write_stdin`) or an in-process unit test that calls `run()` with a fake stdin. The `assert_cmd` approach is preferred for integration confidence but requires the binary to be built before the test runs. Lovelace should confirm which approach is used and document it in the spec.

- **`--backends` filter edge case.** The task marks this out of scope: when a non-empty `--backends` filter is specified but no named binaries are found, the same fix should apply but is deferred. Lovelace should add a note that the spec does NOT cover this case so Da Vinci does not accidentally handle it.

---

## 4. Confirmation: Ready for Lovelace

This task is ready for spec writing.

- Source code location of the bug is confirmed: `/home/isaac/src/sh.great/src/cli/mcp_bridge.rs` lines 127-131.
- Structural analysis of `GreatBridge::new` confirms it accepts an empty `Vec<BackendConfig>` without panicking at construction time.
- The two dispatch methods (`research`, `analyze_code`) in `/home/isaac/src/sh.great/src/mcp/bridge/server.rs` are the primary code paths requiring a backend-availability guard.
- All 5 acceptance criteria are testable and within the size budget (max 5 rule satisfied).
- No other tasks block this one.

**Lovelace's primary open question:** Can rmcp's `tools/list` response be overridden at runtime to return an empty array when no backends are available, or does the spec need to be adjusted to allow returning the full registered tool list with each tool call returning a structured error? This must be resolved by reading the rmcp crate documentation (via Context7 MCP) before the implementation spec is finalised.
