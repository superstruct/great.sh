# 0037 Socrates Review -- MCP Bridge Degraded Mode (No Backends)

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Spec:** `.tasks/ready/0037-mcp-bridge-spec.md`
**Backlog:** `.tasks/backlog/0037-mcp-bridge-starts-without-backends.md`
**Date:** 2026-03-03
**Round:** 1 of 1

---

## VERDICT: APPROVED

---

## Elenchus

### Concern 1: Backlog says JSON-RPC error, spec says tool-level error -- intentional divergence?

```json
{
  "gap": "The backlog acceptance criterion 3 says tool calls should return 'a JSON-RPC error response with a human-readable message' (and mentions error code -32601). The spec explicitly returns a CallToolResult with isError: true instead, which is a successful JSON-RPC response with a tool-level error flag. The spec addresses this divergence in Section 2 (Q: What JSON-RPC error code?) but does not call out that it is intentionally overriding the backlog's stated format.",
  "question": "Is the spec's decision to use tool-level errors (isError: true) rather than JSON-RPC protocol errors (-32601) consistent with how other bridge failures are reported, and does the backlog author agree with this divergence?",
  "severity": "ADVISORY",
  "recommendation": "The spec's reasoning is sound -- CallToolResult::error is the established pattern in server.rs (12 existing call sites use it), and the MCP spec distinguishes between protocol errors and tool execution errors. The backlog's mention of -32601 appears to be an imprecise suggestion rather than a hard requirement. The spec should note this divergence more prominently, but it is not blocking."
}
```

**Verification:** I confirmed 12 `CallToolResult::error(vec![Content::text(...)])` call sites in `src/mcp/bridge/server.rs` (lines 89, 105, 128, 167, 181, 216, 252, 270, 275, 301, 316, 339). The spec is following the established codebase pattern.

### Concern 2: call_tool guard blocks backend-independent tools

```json
{
  "gap": "The call_tool guard (Change E) returns NO_BACKENDS_MSG for ALL tool calls when backends is empty, including wait, list_tasks, get_result, kill_task, and cleanup_tasks -- tools that do not call resolve_backend and could theoretically function without any backends.",
  "question": "Since tools/list returns an empty array in degraded mode, no well-behaved MCP client will call any tool. But is the blanket guard at call_tool level the right place, or should the guard be per-tool (only on tools that call resolve_backend)?",
  "severity": "ADVISORY",
  "recommendation": "The blanket guard is pragmatically correct: tools/list returns empty, so no client should call any tool. The alternative (per-tool guards) would be more precise but adds complexity for zero practical benefit since no tools are advertised. The spec's approach is simpler and defensible. No change needed."
}
```

### Concern 3: Test uses PATH=/nonexistent -- cargo binary resolution

```json
{
  "gap": "The test sets env('PATH', '/nonexistent') which removes ALL standard tools from PATH. If the great binary or its runtime (tokio, tracing_subscriber) attempt any PATH-dependent operation during startup, the test could fail for reasons unrelated to the bug fix.",
  "question": "Does the great binary's startup path (config discovery, tracing init, tokio runtime creation) depend on any PATH-resolved external command?",
  "severity": "ADVISORY",
  "recommendation": "Verified: config::discover_config() does NOT use which:: or Command::new. tracing_subscriber::fmt() does not depend on PATH. tokio::runtime::Runtime::new() does not depend on PATH. assert_cmd::Command::cargo_bin() resolves the binary via CARGO_BIN_EXE_great (absolute path). The test approach is safe."
}
```

### Concern 4: Spec line number for test insertion

```json
{
  "gap": "Spec says 'Append after the existing mcp_bridge_unknown_preset_shows_error_message test (line 1970)'. The actual file ends at line 1970 with a closing brace, and line 1971 is EOF.",
  "question": "Is the insertion point correct?",
  "severity": "ADVISORY",
  "recommendation": "Verified: mcp_bridge_unknown_preset_shows_error_message spans lines 1963-1970. Line 1970 is the closing brace. The spec's instruction to append after this test is correct -- Da Vinci should append after line 1970."
}
```

### Concern 5: protocolVersion string in test input

```json
{
  "gap": "The test sends protocolVersion '2025-03-26' in the initialize request. The server responds with ProtocolVersion::LATEST (line 349 of server.rs). If the rmcp crate's protocol negotiation rejects mismatched versions, the test could fail.",
  "question": "Does rmcp's initialize handler reject or downgrade connections when the client's protocolVersion differs from ProtocolVersion::LATEST?",
  "severity": "ADVISORY",
  "recommendation": "MCP protocol spec says the server SHOULD respond with its own protocolVersion regardless of what the client sends. rmcp's ServerHandler::get_info() returns the server's version unconditionally (there is no version-negotiation rejection in the handler). The test assertion checks for 'protocolVersion' in the output (any value), not a specific version string. This is safe."
}
```

### Concern 6: Acceptance criteria coverage

Checking each backlog acceptance criterion against the spec:

| Backlog Criterion | Spec Coverage | Status |
|---|---|---|
| 1. initialize returns well-formed JSON-RPC response, not exit 1 | Change A (remove bail), Test mcp_bridge_starts_without_backends asserts exit 0 + protocolVersion | COVERED |
| 2. tools/list returns empty tools array | Change D (guard list_tools), Test asserts `"tools":[]` | COVERED |
| 3. Tool call returns error with human-readable message | Change E (guard call_tool with NO_BACKENDS_MSG), manual test plan | COVERED |
| 4. WARN-level log to stderr | Change A (tracing::warn!), Test mcp_bridge_no_backends_emits_warning | COVERED |
| 5. cargo test passes, new test asserts run() does not Err | Change F adds two tests; existing tests unaffected | COVERED |

All five acceptance criteria are covered.

### Concern 7: Out-of-scope items -- is anything hidden that should be done now?

```json
{
  "gap": "The backlog marks '--backends filter with nothing matching' as out of scope. The spec acknowledges the fix applies naturally but defers test coverage.",
  "question": "Since the code change (removing bail!) applies equally to the --backends filter case, should at least one test for --backends nonexistent be included now?",
  "severity": "ADVISORY",
  "recommendation": "The fix does apply naturally -- discover_backends with a filter for a nonexistent backend returns empty vec, which now triggers degraded mode instead of bail!. Adding a test is low-cost but the backlog explicitly defers it. The spec is consistent with the backlog's scoping decision. Da Vinci MAY add a test if time permits but is not required to."
}
```

### Concern 8: resolve_backend still returns McpError -- dead path in degraded mode?

```json
{
  "gap": "The spec does NOT modify resolve_backend(). In degraded mode, the call_tool guard catches all tool calls before they reach resolve_backend. But the resolve_backend method still contains McpError::invalid_params('no backends available') at line 429. This is now dead code in the degraded case.",
  "question": "Is the dead code in resolve_backend a problem?",
  "severity": "ADVISORY",
  "recommendation": "Not a problem. The resolve_backend error path is still reachable in the non-degraded case when a specific backend name is requested but not found (line 408-420). The 'no backends available' fallback at line 429 is theoretically dead when backends is empty (since call_tool intercepts first), but it serves as a defense-in-depth safety net. The spec is correct not to modify it."
}
```

---

## Summary of Verification

| Check | Result |
|---|---|
| Line numbers match actual code | All verified correct (mcp_bridge.rs:125-140, server.rs:17-21, 367-383, 388-395) |
| Code snippets match actual code | All "current code" blocks are exact matches |
| No new dependencies needed | Correct -- tracing already in Cargo.toml, CallToolResult::error pattern established |
| No .unwrap() in production code | No new .unwrap() introduced |
| Edge cases documented | 7 edge cases in table, all reasonable |
| Security considerations | Addressed -- no new attack surface, no info leak, DoS harmless |
| Test infrastructure | assert_cmd::Command has .output(), .write_stdin(), .timeout(), .env() -- all verified in existing tests |
| Backlog acceptance criteria | All 5 covered |

---

## Verdict

**APPROVED.** This is a clean, well-scoped bugfix spec. The changes are minimal (remove a bail!, add two guards, add a constant, add two tests), the code locations are verified, and all acceptance criteria are covered. The design decision to use tool-level errors (isError: true) rather than JSON-RPC protocol errors is the correct choice and follows existing codebase conventions. The only advisory notes are: (1) the divergence from backlog's "-32601" suggestion should be noted for the backlog author, and (2) Da Vinci may optionally add a `--backends nonexistent` test case.

Summary: Solid S-complexity bugfix spec with exact code locations, verified line numbers, complete acceptance criteria coverage, and a well-reasoned design decision on error response format.
