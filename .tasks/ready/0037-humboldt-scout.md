# 0037 Humboldt Scout Report — MCP Bridge Degraded Mode

**Scout:** Alexander von Humboldt
**Date:** 2026-03-03
**Spec:** `.tasks/ready/0037-mcp-bridge-spec.md`
**Complexity:** S — 3 files, ~25 lines changed, 0 new dependencies

---

## 1. File Map — Exact Change Sites

### File 1: `src/cli/mcp_bridge.rs`

| Lines | Change | Description |
|---|---|---|
| 126–131 | **Remove** | Delete the `if backends.is_empty() { anyhow::bail!(...) }` guard entirely |
| 133–140 | **Wrap** | Wrap the `tracing::info!` block in `if !backends.is_empty() { ... }` |

Insert (replaces lines 126–131):
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

Wrap (lines 133–140):
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

No import changes needed. `tracing::warn!` and `tracing::info!` are already used on
lines 133 and 587 of server.rs; the `tracing` crate is in `Cargo.toml` via tokio.

### File 2: `src/mcp/bridge/server.rs`

| Lines | Change | Description |
|---|---|---|
| 21 (after) | **Insert constant** | Add `NO_BACKENDS_MSG` after the existing `MAX_FILE_BYTES` constant at line 21 |
| 367–383 | **Add early return** | Insert a `self.backends.is_empty()` guard at the top of `list_tools` |
| 388–395 | **Add early return** | Insert a `self.backends.is_empty()` guard at the top of `call_tool` |

**Constant insert site** — after line 21 (`const MAX_FILE_BYTES: usize = 100 * 1024;`):
```rust
/// Human-readable error returned by tool calls when no backends are available.
const NO_BACKENDS_MSG: &str =
    "No AI CLI backends found. Install at least one of: gemini, codex, claude, grok, ollama. \
     Run `great doctor` to check backend availability.";
```

**`list_tools` guard** — insert before line 372 (`let allowed = self.preset.tool_names();`):
```rust
        if self.backends.is_empty() {
            return std::future::ready(Ok(ListToolsResult {
                meta: None,
                tools: vec![],
                next_cursor: None,
            }));
        }
```

**`call_tool` guard** — insert before line 393 (`let context = ToolCallContext::new(...)`):
```rust
        if self.backends.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(NO_BACKENDS_MSG)]));
        }
```

No new imports needed. `Content`, `CallToolResult`, `ListToolsResult` are all
imported via the existing `use rmcp::model::*;` wildcard at line 7.

### File 3: `tests/cli_smoke.rs`

| Lines | Change | Description |
|---|---|---|
| 1971 (append) | **Add 2 tests** | Append after the final `}` at line 1970 |

Two tests to append:
- `mcp_bridge_starts_without_backends` — sends `initialize` + `tools/list`, asserts exit 0
  and `"tools":[]` in stdout
- `mcp_bridge_no_backends_emits_warning` — sends `initialize`, asserts stderr contains
  "degraded mode" at `--log-level warn`

Both tests use `.env("PATH", "/nonexistent")` to guarantee zero backends. Both call
`.output()` (not `.assert()`) so they can inspect stdout AND stderr independently.

---

## 2. Pattern Analysis

### How `list_tools` works

`list_tools` is an override in `ServerHandler for GreatBridge` (lines 367–383).
The `#[tool_router]` macro on the `impl GreatBridge` block auto-generates `Self::tool_router()`
which populates `self.tool_router: ToolRouter<Self>` with every `#[tool]`-annotated method.

`list_tools` calls `self.tool_router.list_all()` to get all registered tools, then
filters by `self.preset.tool_names()` (a `HashSet<&str>`). The return type is
`impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_`,
so the early return must be `std::future::ready(Ok(...))` — NOT an `async` block —
because the trait impl uses the `impl Trait` return-position form, not `async fn`.

### How `call_tool` works

`call_tool` is an `async fn` override in `ServerHandler` (lines 388–395).
It creates a `ToolCallContext` and delegates to `self.tool_router.call(context)`.
The early return for the no-backends guard is a normal `return Ok(...)` inside
the async body — the `async fn` signature is already correct.

### `resolve_backend` relationship

`resolve_backend` (lines 406–432) is called by every tool handler that needs a
backend (`prompt`, `run`, `research`, `analyze_code`, `clink`). When backends is empty
and `name` is `None`, it returns `Err(McpError::invalid_params("no backends available", None))`.
This would propagate as a JSON-RPC protocol error (code -32602), not a tool-level error.
The `call_tool` guard intercepts BEFORE this path is ever reached, so `resolve_backend`
needs no changes.

### `CallToolResult::error` signature

Already used in 12 places in server.rs. Pattern:
```rust
CallToolResult::error(vec![Content::text(some_string)])
```
Both `CallToolResult::error` and `Content::text` come from `rmcp::model::*` (line 7).

---

## 3. Dependency Graph

```
tests/cli_smoke.rs
    └── uses assert_cmd::Command (via `great()` helper, line 7)
        └── `.env("PATH", "/nonexistent")` — assert_cmd 2.0 supports this
        └── `.write_stdin(...)` — confirmed used at lines 768, 863, etc.
        └── `.timeout(Duration::from_secs(10))` — confirmed at line 755
        └── `.output()` — confirmed at lines 782–784, 799–801, etc.

src/cli/mcp_bridge.rs
    └── calls discover_backends() -> Vec<BackendConfig>
    └── calls start_bridge(backends, ...) in rt.block_on(...)
    └── tracing::warn! — available (tracing is transitive via tokio, no direct dep needed)

src/mcp/bridge/server.rs
    └── ServerHandler::list_tools — returns impl Future, NOT async fn
        └── early return uses std::future::ready(Ok(...)) -- SAME as existing code
    └── ServerHandler::call_tool — async fn
        └── early return uses Ok(CallToolResult::error(...)) -- SAME as 12 existing callsites
    └── NO_BACKENDS_MSG — new &str constant, no external deps
```

---

## 4. Test Infrastructure

### Existing mcp_bridge tests (lines 1942–1970)

```rust
// Pattern A: .assert() chain (for tests that don't need stdout/stderr inspection)
great()
    .args(["mcp-bridge", "--help"])
    .assert()
    .success()
    .stdout(predicate::str::contains("..."));

// Pattern B: .assert().failure() for exit code check
great()
    .args(["mcp-bridge", "--preset", "invalid"])
    .assert()
    .failure();
```

Note: the existing `mcp_bridge_unknown_preset_fails` and `mcp_bridge_unknown_preset_shows_error_message`
tests do NOT use `.env("PATH", "/nonexistent")` — they fail at the `Preset::from_str` stage
before reaching `discover_backends`, so the bail! guard is never hit. These tests are
unaffected by the change.

### New test pattern (append after line 1970)

The new tests use **Pattern C** — `.output()` with manual stdout/stderr inspection:
```rust
let output = great()
    .args([...])
    .env("PATH", "/nonexistent")
    .write_stdin(concat!(..., "\n", ..., "\n"))
    .timeout(std::time::Duration::from_secs(10))
    .output()
    .expect("failed to execute mcp-bridge");
let stdout = String::from_utf8_lossy(&output.stdout);
assert!(output.status.success(), "...");
assert!(stdout.contains("protocolVersion"), "...");
```

This pattern is confirmed working at lines 779–793 and 795–809 (statusline tests).

### Why `.env("PATH", "/nonexistent")` works

`discover_backends` uses the `which` crate to locate binaries. The `which` crate
reads `PATH` from the subprocess environment. Setting `PATH=/nonexistent` prevents
all binary lookups from succeeding, guaranteeing `Vec::new()` is returned from
`discover_backends`. The GREAT_*_CLI environment variables are not set in the test
subprocess, so the `std::env::var(spec.env_override).ok()` fallback also returns `None`.

---

## 5. Risk Assessment

### Low Risk

1. **`list_tools` return type** — The method signature uses `impl Future` (not `async fn`).
   The early return `std::future::ready(Ok(...))` must match exactly. Verified: existing
   code at lines 378–382 already uses this exact form. The guard mirrors that pattern.

2. **`call_tool` is `async fn`** — The guard is a simple early `return Ok(...)`. No
   `.await` required. No lifetime complications. Zero risk.

3. **`tracing::warn!` in mcp_bridge.rs** — The `tracing` crate is already a transitive
   dep (confirmed in MEMORY.md). `tracing::info!` is already used in the same function
   at line 133. No `use` statement needed.

### Medium Risk

4. **Test may hang if bridge does not close on stdin-EOF** — The bridge server loop
   is driven by `rmcp::transport::io::stdio()`. When stdin closes (after `write_stdin`
   delivers bytes and test harness closes the pipe), the server should exit cleanly.
   The existing `mcp_bridge_protocol.sh` script at line 7 notes "bridge exits when stdin
   closes." The 10-second timeout on the test guards against any unexpected hang.
   Da Vinci should verify this by running the test manually before committing.

5. **JSON output format for tools/list** — The test asserts `stdout.contains(r#""tools":[]"#)`
   with a fallback `|| stdout.contains(r#""tools": []"#)`. The rmcp library controls JSON
   serialization; serde_json typically omits spaces after `:` when using `to_string()`.
   Check rmcp version to confirm: likely `"tools":[]` without space. The OR condition
   in the test spec handles both cases safely.

### Informational

6. **`resolve_backend` still returns JSON-RPC error if somehow reached** — The `call_tool`
   guard returns before routing when backends is empty, so `resolve_backend` is never called
   in the zero-backends case. No change to `resolve_backend` needed and no regression risk.

7. **`start_bridge` logs backends at INFO level (line 587–596)** — This log line is in
   `server.rs::start_bridge`, not `mcp_bridge.rs`. It runs AFTER the bail! guard is removed.
   When backends is empty it will log `"great-mcp-bridge starting (preset=..., backends=)"`.
   The trailing empty string is cosmetically odd but harmless. The spec does NOT ask for a
   fix here; only the info log in `mcp_bridge.rs` is wrapped.

---

## 6. Import Additions Needed

### `src/cli/mcp_bridge.rs`
**None.** `tracing::warn!` is called as a path expression, no `use` statement required.
`tracing` is already a transitive dep available to all crates.

### `src/mcp/bridge/server.rs`
**None.** `Content`, `CallToolResult`, `ListToolsResult` are all covered by
`use rmcp::model::*;` (line 7).

### `tests/cli_smoke.rs`
**None.** The file already imports `assert_cmd::Command`, `predicates::prelude::*`, and
`tempfile::TempDir`. The new tests use `std::time::Duration` which is always in scope via
`std::`. The `.env()`, `.write_stdin()`, `.timeout()`, and `.output()` methods are all part
of `assert_cmd 2.0` which is already in `[dev-dependencies]`.

---

## 7. Recommended Build Order

Per spec section "Build Order" — all three files have no compile-time interdependencies.
Da Vinci can modify them in any order, but this sequence minimizes wasted compile cycles:

1. **`src/cli/mcp_bridge.rs`** — Remove the bail! guard (two edits). `cargo check` passes
   immediately since the type signature of `run()` is unchanged.

2. **`src/mcp/bridge/server.rs`** — Add constant + two guards (three edits). `cargo check`
   verifies the constant type and the return type compatibility of the `list_tools` early return.

3. **`tests/cli_smoke.rs`** — Append two test functions. `cargo test mcp_bridge` runs all
   five mcp_bridge tests.

4. **`cargo clippy -- -D warnings`** — Verify no new warnings.

5. **Manual smoke test** (optional, recommended):
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}
   {"jsonrpc":"2.0","method":"notifications/initialized"}
   {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
     | PATH=/nonexistent cargo run -- mcp-bridge --preset minimal --log-level warn
   ```
   Expected: two JSON lines on stdout (`initialize` response + `tools/list` response),
   `"tools":[]` in the second line, "degraded mode" on stderr.

---

## Summary for Da Vinci

Three files. Three kinds of edits:

| File | Edit | Lines |
|---|---|---|
| `src/cli/mcp_bridge.rs` | Replace bail! with tracing::warn! | 126–131 |
| `src/cli/mcp_bridge.rs` | Wrap info log in `if !backends.is_empty()` | 133–140 |
| `src/mcp/bridge/server.rs` | Add `NO_BACKENDS_MSG` constant after line 21 | insert |
| `src/mcp/bridge/server.rs` | Add empty-backends guard in `list_tools` before line 372 | insert |
| `src/mcp/bridge/server.rs` | Add empty-backends guard in `call_tool` before line 393 | insert |
| `tests/cli_smoke.rs` | Append 2 tests after line 1970 | append |

Zero new dependencies. Zero new public types. Zero signature changes. All types used in
the new code (`Content`, `CallToolResult`, `ListToolsResult`, `tracing::warn!`) are
already imported at their respective change sites.
