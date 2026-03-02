# Dijkstra Code Review — Task 0029: MCP Bridge

**Reviewer:** Edsger W. Dijkstra (Code Reviewer)
**Date:** 2026-02-27
**Files reviewed:**
- `src/mcp/bridge/mod.rs`
- `src/mcp/bridge/backends.rs`
- `src/mcp/bridge/registry.rs`
- `src/mcp/bridge/tools.rs`
- `src/mcp/bridge/server.rs`
- `src/cli/mcp_bridge.rs`
- `src/config/schema.rs` (McpBridgeConfig section)
- `src/cli/apply.rs` (MCP bridge registration block)

---

```
VERDICT: REJECTED

Issues:
- [BLOCK] src/config/schema.rs:148 — Malformed doc comment: `/ Valid values:
  "gemini", "codex", "claude", "grok", "ollama".` is missing the second slash.
  This is not a `///` doc comment — it is a bare `/` followed by text, which
  compiles as a divide-operator parse error or silently becomes a dead comment
  depending on context. It will not appear in rustdoc and may fail `cargo doc`.
  Fix: `/// Valid values: "gemini", "codex", "claude", "grok", "ollama".`

- [BLOCK] src/mcp/bridge/backends.rs:13 — Dead code annotation violates
  established project convention. Convention (confirmed in MEMORY.md and
  every other annotation in this codebase): the comment must name the GROUP
  that will consume the item, e.g., `// Planned for GROUP X (description).`
  This annotation reads: `// Reserved for programmatic API key checks.` — no
  GROUP named. The reader cannot determine when this field becomes live.
  Same violation at src/mcp/bridge/registry.rs:42: `// Available for
  duration calculations by callers.` — no GROUP named.

- [WARN] src/cli/mcp_bridge.rs:69-85 — CLI default-value detection is
  fragile. The timeout and preset merge logic uses sentinel comparison
  (`args.timeout != 300`, `args.preset != "agent"`) to distinguish
  "user set this" from "clap applied the default". If the user explicitly
  passes `--timeout 300` or `--preset agent`, the config value is silently
  preferred over the explicit CLI argument. This is surprising behaviour.
  Consider using `Option<u64>` / `Option<String>` for these fields with no
  `default_value` on the Clap arg, and apply defaults at the end of the
  merge chain.

- [WARN] src/cli/apply.rs:727,735 — Double-guard on `cfg.mcp_bridge`. The
  outer `if cfg.mcp_bridge.is_some()` at line 727 enters the block, then
  line 735 immediately does `if let Some(ref bridge_cfg) = cfg.mcp_bridge`
  again. The outer guard ensures `is_some()`, so the inner `if let` always
  matches and the `None` arm is unreachable. Use `if let Some(ref bridge_cfg)
  = cfg.mcp_bridge` as a single guard from the start.

- [WARN] src/mcp/bridge/tools.rs:144 — `Preset::from_str` is an inherent
  method returning `Option<Self>`. Rust provides `std::str::FromStr` for
  exactly this purpose, which integrates with clap's value parsing and yields
  better error messages. This is a minor deviation from idiomatic Rust; not
  a blocker given that `from_str` is only called from one site, but worth
  noting for future extension.

- [WARN] src/mcp/bridge/registry.rs:217 — `pid_copy` is introduced solely
  to move a `u32` into the async block. `u32` is `Copy`; the variable can
  simply be named `pid` and captured by copy without introducing a new binding.
  The alias adds noise without adding clarity.

Summary: Two BLOCK issues must be resolved before merge — a malformed doc
comment in schema.rs that will corrupt rustdoc output, and two dead-code
annotations that violate the established GROUP-naming convention enforced
across the entire codebase.
```

---

## Detailed notes by file

### `src/mcp/bridge/backends.rs`

The `BACKEND_SPECS` constant and `discover_backends` function are cleanly
separated. The `build_command_args` function is slightly complex — it
branches on `backend.name` twice (lines 118 and 134/147) — but both branches
are distinct, the logic is shallow, and the function is well-commented. The
tests cover the important cases. Acceptable.

The `#[allow(dead_code)]` annotation on `api_key_env` (line 13) breaks the
project convention that requires naming the GROUP that will activate the field.
Every other annotation in this codebase follows the pattern
`// Planned for GROUP X (description).` This one does not. **BLOCK.**

### `src/mcp/bridge/registry.rs`

The `TaskRegistry` design is sound: a `HashMap` behind an `Arc<Mutex<>>`,
background tasks update state, `snapshot()` decouples the observable report
from internal state. The `wait_for_tasks` polling loop (line 264–280) holds
the lock while checking, drops it, sleeps 100ms, then re-acquires. This is
correct and the `drop(tasks)` comment makes intent explicit.

The `pid_copy` alias at line 217 is unnecessary noise — `u32` is `Copy` and
`pid` is already a bare binding. **WARN.**

The `#[allow(dead_code)]` on `TaskSnapshot::started_at` (line 42) lacks a
GROUP name. **BLOCK.**

### `src/mcp/bridge/tools.rs`

The `Preset` and parameter structs are clean data definitions. The
`Preset::tool_names()` method duplicates tool names across match arms
(cumulative set maintained manually). A future regression could introduce
a tool into `Agent` but not `Research`. The `test_presets_are_cumulative`
test catches this; the risk is bounded.

### `src/mcp/bridge/server.rs`

`GreatBridge` is the integration point. At approximately 475 lines it is
at the upper limit of comfortable review. The `research` function at lines
159–206 mixes file I/O logic with prompt assembly and then synchronous
invocation. This is three concerns in one function. It works, but if the
file-reading logic grows (permissions, symlinks, binary detection), it
will exceed a single reason to change. **WARN** withheld at this cycle
since the complexity is bounded by the current scope; flag for future
extraction.

`resolve_backend` (lines 349–383) has one nested match expression with a
dangling `.ok_or_else()` call (lines 370–382) that reads as if chained to
nothing. The intent is clear on careful reading, but the formatting could
be improved. Not a blocker.

`run_sync` returns `Result<String, String>` — error as `String`, not
`anyhow::Error`. This is intentional because callers convert errors to
`CallToolResult::error`, but it deviates from the codebase convention of
`anyhow::Result`. **WARN** withheld — the deviation is local to a private
function and does not cross module boundaries.

`truncate_output` correctly walks `char_indices` to avoid splitting
multi-byte characters. Well done.

### `src/cli/mcp_bridge.rs`

The sentinel-value merge for `timeout` and `preset` (lines 69–85) is
fragile: if a user explicitly passes `--timeout 300`, the config file value
is used instead. This inverts the expected precedence of explicit CLI flags.
**WARN.**

### `src/config/schema.rs`

Line 148: `/ Valid values: ...` — the leading slash is not doubled. This is
not a doc comment. It will fail `cargo doc` or silently produce a field with
no documentation for the second sentence. **BLOCK.**

### `src/cli/apply.rs`

The bridge registration block (lines 726–780) is correct. The double-guard
pattern (`is_some()` outer, `if let Some` inner) is redundant but not
harmful. **WARN.**

---

## Items to fix before merge

1. `src/config/schema.rs:148` — add the missing `/` to make `/// Valid values:`
2. `src/mcp/bridge/backends.rs:13` — add GROUP name to dead_code comment
3. `src/mcp/bridge/registry.rs:42` — add GROUP name to dead_code comment
