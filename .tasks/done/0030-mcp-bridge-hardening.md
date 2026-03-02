# 0030: MCP Bridge Hardening — Security, Code Quality, and UX Follow-ups

**Priority:** P2
**Type:** hardening
**Module:** `src/mcp/bridge/`, `src/cli/{mcp_bridge,doctor}.rs`, `src/config/schema.rs`
**Status:** open
**Estimated Complexity:** M

## Context

Task 0029 shipped the inbuilt MCP bridge in iteration 026. Five follow-up items
were deferred from the reviewer cycle (Socrates concern #9, Socrates concern #10,
Dijkstra code quality note, Rams UX note, Wirth size measurement). None were
blocking for the initial commit, but all are P2 and must be resolved before the
bridge is considered production-hardened.

### Item A — Path traversal prevention in `research` and `analyze_code`

`server.rs` reads arbitrary file paths passed by the MCP client without
validation. The `research` tool iterates `params.0.files` and calls
`std::fs::read(path)` directly (line 169). The `analyze_code` tool calls
`std::fs::read_to_string(&params.0.code_or_path)` after a bare `Path::exists()`
check (line 221). No path canonicalization or directory allowlist is applied.

Socrates (concern #10) noted that the "AI assistant already has filesystem
access" argument only holds when the bridge is used exclusively by Claude Code
running as the same user. Any other MCP client (or a prompt injection that
constructs a crafted file path) can read `/etc/shadow`, `~/.ssh/id_rsa`, or
`~/.aws/credentials` through these two tools.

The mitigation is an optional `--allowed-dirs` CLI flag (and matching
`allowed_dirs` config key in `[mcp-bridge]`) that, when set, canonicalizes each
requested path and rejects any path whose canonical form does not start with one
of the allowed directory prefixes. When unset, behavior is unchanged (the single-
user, same-machine threat model is documented in the error message).

### Item B — `--dangerously-skip-permissions` opt-out and doctor warning

`backends.rs` hardcodes `auto_approve_flag: Some("--dangerously-skip-permissions")`
for the Claude backend in `BACKEND_SPECS` (line 48). This flag is passed
unconditionally whenever the Claude backend handles a tool call. A user who
installed Claude Code for interactive use and later adds `[mcp-bridge]` to
`great.toml` may not realize they are granting full auto-approval to every
bridged call.

Socrates (concern #9) recommended: (a) make the flag configurable per-backend
in `McpBridgeConfig`, with `auto_approve = true` as the default to preserve
existing behavior, and (b) add a `great doctor` warning that names the flag
explicitly when the Claude backend is available.

The config opt-out path: `[mcp-bridge] auto_approve = false` suppresses the
`--dangerously-skip-permissions` flag from the Claude backend (and the `-y`/
`--full-auto` flags from gemini/codex/grok). The doctor warning appears
regardless of the config setting, but is styled as `warn` when auto-approve is
on (default) and `pass` when it has been explicitly disabled.

### Item C — Refactor `check_mcp_bridge()` to use `discover_backends()`

`doctor.rs` line 619 contains a hardcoded backend list:

```rust
let backends = [
    ("gemini", "Gemini CLI", "GEMINI_API_KEY"),
    ("codex", "Codex CLI", "OPENAI_API_KEY"),
    ("claude", "Claude CLI", ""),
    ("grok", "Grok CLI", "XAI_API_KEY"),
    ("ollama", "Ollama", ""),
];
```

This list duplicates `BACKEND_SPECS` in `backends.rs`. When a new backend is
added (or a name/env key changes), `doctor.rs` will silently drift out of sync.
The `api_key_env` field on `BackendConfig` is already annotated with
`#[allow(dead_code)] // Planned for doctor integration` (backends.rs line 13),
confirming this refactor was always intended.

Replace the hardcoded slice with a call to `discover_backends(&[])` (no filter)
and drive the output from the returned `BackendConfig` structs.

### Item D — Wire global flags into `mcp-bridge` subcommand

`src/cli/mcp_bridge.rs` accepts its own `--log-level` flag for tracing control
but ignores the global `--verbose` and `--quiet` flags forwarded from `main.rs`.
The `Args` struct has no `non_interactive`, `verbose`, or `quiet` fields (unlike
`doctor.rs` which uses `non_interactive`). Rams noted this gap as a UX issue:
users who run `great --verbose mcp-bridge` expect the same verbosity behavior
they get from other subcommands.

The mapping: `--verbose` promotes the tracing filter from `warn` to `debug`;
`--quiet` demotes it from `warn` to `error`. The existing `--log-level` flag
wins if explicitly passed (CLI arg takes precedence over derived global).
`non_interactive` is not applicable to this subcommand (it is a server, not a
prompt-driven command) and need not be forwarded.

### Item E — Measure and mitigate binary size growth

Wirth projected a +29–53% binary size increase from the five new dependencies
(rmcp 0.16, uuid 1.x, tracing 0.1, tracing-subscriber 0.3, schemars 1.0,
libc 0.2). The actual impact was not measured before the iteration-026 commit
(the build ran but no `ls -lh target/release/great` before/after comparison
was recorded). The current baseline is 10.9 MB (iteration-026 observer report).

This item requires: (1) build a release binary on the main branch as of this
task and record its size, (2) if growth exceeds 15% from the 10.9 MB baseline
(threshold: ~12.5 MB), investigate and apply mitigations (e.g., replace
`tracing-subscriber` with direct `eprintln!` macros, strip debug symbols, or
enable LTO). Record the measured size and any mitigations in the iteration
report.

## Requirements

1. Add path canonicalization + optional `allowed_dirs` guard to the `research`
   and `analyze_code` tool handlers in `server.rs`; expose via `--allowed-dirs`
   CLI flag and `allowed_dirs` config key in `McpBridgeConfig`.
2. Add `auto_approve: Option<bool>` to `McpBridgeConfig`; when `false`, suppress
   all auto-approval flags from every backend's command args. Add a `great doctor`
   warning that names `--dangerously-skip-permissions` explicitly when the Claude
   backend is found and auto-approve is enabled (default).
3. Replace the hardcoded backend list in `check_mcp_bridge()` in `doctor.rs`
   with `discover_backends(&[])` driven by the `BackendConfig.api_key_env` field.
4. Forward global `--verbose` / `--quiet` flags from `main.rs` into the
   `mcp-bridge` subcommand's tracing initialization, with `--log-level` taking
   precedence when explicitly provided.
5. Measure the release binary size post-0029; document in the iteration report;
   apply mitigations if size exceeds 12.5 MB.

## Acceptance Criteria

- [ ] `great mcp-bridge --allowed-dirs /home/user/projects` rejects a `research`
  or `analyze_code` call targeting `/etc/shadow` with a clear error message
  (`"path not in allowed directories"`); a path inside `/home/user/projects`
  succeeds normally.
- [ ] `great doctor` with Claude CLI on PATH and no `[mcp-bridge]` config shows
  a warning line that contains the text `--dangerously-skip-permissions`; adding
  `auto_approve = false` to `[mcp-bridge]` in `great.toml` changes that line
  from `warn` to `pass` (or `info`).
- [ ] Adding a sixth backend to `BACKEND_SPECS` in `backends.rs` automatically
  appears in `great doctor`'s MCP Bridge section without any change to
  `doctor.rs`; confirmed by unit test or code inspection showing no hardcoded
  backend name strings in `check_mcp_bridge()`.
- [ ] `great --verbose mcp-bridge` starts the server with tracing at `debug`
  level; `great --quiet mcp-bridge` starts it at `error` level; `great
  mcp-bridge --log-level info` overrides both and uses `info` regardless of
  global flags.
- [ ] A release binary size measurement is recorded in the iteration report; if
  size exceeds 12.5 MB, at least one mitigation is applied and the post-
  mitigation size is also recorded.

## Files That Need to Change

- `src/mcp/bridge/server.rs` — add `allowed_dirs: Option<Vec<PathBuf>>` to
  `GreatBridge`; validate paths in `research` and `analyze_code` handlers.
- `src/mcp/bridge/backends.rs` — make `build_command_args()` respect an
  `auto_approve_enabled: bool` parameter; remove `#[allow(dead_code)]` on
  `api_key_env`.
- `src/cli/mcp_bridge.rs` — add `--allowed-dirs` arg; accept forwarded
  `verbose`/`quiet` booleans from `main.rs`; derive tracing filter from global
  flags when `--log-level` is not explicitly set.
- `src/cli/doctor.rs` — replace hardcoded `backends` slice in
  `check_mcp_bridge()` with `discover_backends(&[])`; add auto-approve warning
  for Claude backend.
- `src/config/schema.rs` — add `auto_approve: Option<bool>` and
  `allowed_dirs: Option<Vec<String>>` fields to `McpBridgeConfig`.
- `src/main.rs` — forward `verbose`/`quiet` from `Cli` into `mcp-bridge` `run()`
  (same pattern as `non_interactive` forwarding to `apply`/`doctor`).

## Dependencies

- Requires the 0029 implementation to be on main (confirmed: iteration-026 commit
  is on main as of 2026-02-28).
- No new crate dependencies anticipated. `std::path::Path::canonicalize()` is
  sufficient for path validation.

## Out of Scope

- P3 items from iteration-026: process group isolation for `run_sync()`, registry
  unit test coverage, `killpg(0)` guard for env overrides. Those remain in the
  observer follow-up list and will be filed separately if prioritized.
- Network-exposed bridge hardening (TLS, authentication) — the bridge is
  stdin/stdout only; network exposure is explicitly out of scope for v1.
- Per-backend `allowed_dirs` granularity — a single global list per bridge
  instance is sufficient for v1.
