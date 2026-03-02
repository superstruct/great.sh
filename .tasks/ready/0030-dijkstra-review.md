# 0030: MCP Bridge Hardening — Dijkstra Code Review

**Reviewer:** Edsger Dijkstra (Code Reviewer)
**Date:** 2026-02-28
**Files reviewed:**
- `src/mcp/bridge/server.rs`
- `src/mcp/bridge/backends.rs`
- `src/mcp/bridge/registry.rs`
- `src/cli/mcp_bridge.rs`
- `src/cli/doctor.rs`
- `src/config/schema.rs`
- `src/main.rs`
- `Cargo.toml`

---

```
VERDICT: APPROVED

Issues:
- [WARN] src/mcp/bridge/backends.rs:7-8,16-17 — dead_code annotation
  format deviates from project convention.
  Convention (from prior reviews): "#[allow(dead_code)] // Planned for GROUP X (description)."
  Actual: "#[allow(dead_code)] // Set for API completeness; doctor reads from BackendSpec via all_backend_specs()."
  The `display_name` field IS actually read by doctor.rs via `all_backend_specs()`,
  which means the annotation is factually misleading — this field is not dead
  code at all. The compiler annotation is unnecessary for both fields because
  `display_name` and `api_key_env` are propagated through `discover_backends()`
  into `BackendConfig` and consumed by `all_backend_specs()`. If clippy reports
  them as dead, the annotation is warranted; if not, it is noise.

- [WARN] src/mcp/bridge/server.rs:467-486 — `truncate_output` length check
  uses `text.len()` (byte count) as a proxy for the char count limit
  `MAX_RESPONSE_CHARS`. The comment calls the constant "characters" and the
  truncation message says "80,000 chars", but the guard `text.len() >
  MAX_RESPONSE_CHARS` is true only when byte count exceeds 80,000. For ASCII
  input this is equivalent; for UTF-8 multi-byte input the byte count exceeds
  the char count, so the guard triggers slightly early. The char boundary walk
  via `char_indices` then correctly limits to 80,000 *chars worth of byte
  offset*, not 80,000 chars. The result: the guard is conservative but the
  truncated string can contain fewer than 80,000 chars. This is safe but the
  semantics are misleading — the constant is documented as "characters" but
  the guard measures bytes. A minor inconsistency, not a correctness defect.

- [WARN] src/cli/mcp_bridge.rs:108-111 — `auto_approve` is resolved from
  config only; there is no `--no-auto-approve` CLI flag to override it. The
  spec (Item B) specifies this as a config-only knob, so this is consistent
  with the spec. However, the pattern breaks the established convention that
  CLI args win over config (used for every other field in the same function).
  If a user wants to suppress auto-approve for a single invocation without
  editing their config file, there is no way to do so. Advisory only —
  consistent with the spec as written.

Summary: All five items (A through E) are correctly implemented per spec;
error propagation, naming, and abstraction boundaries are sound; one
dead_code annotation is factually suspect, one constant naming inconsistency
is minor, and one missing CLI override flag is a spec gap rather than a
code defect.
```

---

## Detailed Analysis

### Item A: Path Traversal Prevention

`validate_path()` (`server.rs:406-432`) is correctly structured. The early
return on `allowed_dirs: None` is the right guard. Canonicalization happens
at the call site before prefix checking. The `start_bridge` canonicalization
of allowed_dirs at startup (`server.rs:499-521`) is correct — resolve once,
check many times, with actionable tracing warnings on failure.

The empty-list warning (`server.rs:515-519`) addresses the edge case from the
spec. The error message format includes both the raw and canonical paths, which
is exactly what a user needs to debug a rejection.

### Item B: auto_approve Propagation

The propagation chain is clean: `McpBridgeConfig.auto_approve` → `mcp_bridge::run()`
→ `TaskRegistry::new(timeout_secs, auto_approve)` → `GreatBridge::new(... auto_approve)`
→ `build_command_args(... self.auto_approve)`. Each link passes the value
rather than re-reading config. The `pub auto_approve: bool` on `TaskRegistry`
(`registry.rs:68`) is the only public field alongside `default_timeout`; both
are read directly by `server.rs` (`run_sync` reads `registry.default_timeout`).
This is a minor abstraction boundary concern — `default_timeout` and
`auto_approve` are logically registry construction parameters, not fields
callers should read post-construction. Advisory only; the pattern is consistent
with the existing `default_timeout` exposure.

### Item C: Doctor Refactor

`check_mcp_bridge()` (`doctor.rs:612-691`) is now decoupled from hardcoded
backend names. The `all_backend_specs()` return type — a `Vec` of unnamed
tuples `(&'static str, &'static str, Option<&'static str>)` — is workable
for a private bridge function but would benefit from named fields if extended.
For five backends this is not a complexity concern.

The condition at `doctor.rs:677` (`if bridge_config.is_some()`) correctly
gates the `.mcp.json` registration check. The auto-approve warning correctly
defaults to `true` when config is absent, matching the spec's stated intent
that the warning fires even without `[mcp-bridge]`.

### Item D: Verbose/Quiet Forwarding

The log-level precedence chain (`mcp_bridge.rs:47-55`) is a simple three-way
if/else — correct, readable, follows the spec exactly. The `match log_level.as_str()`
block (`mcp_bridge.rs:57-71`) with a fallback `eprintln!` is the right pattern
for an unknown-level guard without `bail!`.

The `#[arg(skip)]` pattern for `verbose` and `quiet` fields (`mcp_bridge.rs:37-41`)
is consistent with the established pattern in `doctor.rs` and `loop_cmd.rs`.

### Item E: Binary Size

`[profile.release]` section in `Cargo.toml` with `lto = true`, `strip = true`,
`codegen-units = 1` is correct. These are the standard three knobs. No issues.

---

## Dead Code Annotation Finding (WARN detail)

`BackendConfig.display_name` and `BackendConfig.api_key_env` carry
`#[allow(dead_code)]` annotations (`backends.rs:7,16`). The justification
comment reads "Set for API completeness; doctor reads from BackendSpec via
all_backend_specs()." This is ambiguous: `all_backend_specs()` returns data
from `BackendSpec` (the private struct), not from `BackendConfig`. The
`BackendConfig` fields `display_name` and `api_key_env` are populated in
`discover_backends()` but never read from a `BackendConfig` instance
(the doctor uses `all_backend_specs()` which bypasses `BackendConfig`
entirely). So the annotations are technically correct — these fields ARE
dead on `BackendConfig`. But the annotation comment is misleading: it
implies the fields are used, when they are actually populated-but-unread
carrier fields. The annotation format also deviates from the project
convention of naming the future GROUP that will consume the item. The
correct annotation under project convention would be:

```rust
#[allow(dead_code)] // Planned for GROUP J (BackendConfig consumer API).
pub display_name: &'static str,
```

This is advisory — the code is correct, the annotation comment is imprecise.
