# 0030: MCP Bridge Hardening — Humboldt Scout Report

**Scout:** Alexander von Humboldt
**Spec:** `.tasks/ready/0030-mcp-bridge-hardening-spec.md`
**Review:** `.tasks/ready/0030-socrates-review.md`
**Date:** 2026-02-28

---

## Verified State

- `cargo check` passes (16 dead_code warnings, expected).
- Release binary confirmed: `target/release/great` = **14,269,080 bytes (13.6 MiB)**. Matches spec.
- No `[profile.release]` section exists in `Cargo.toml` — it must be added from scratch.
- `McpBridgeConfig` at `src/config/schema.rs:145` has `#[serde(rename_all = "kebab-case")]` — CONFIRMED.
  - TOML keys will be `auto-approve` and `allowed-dirs` (with hyphens).
  - Rust field names remain `auto_approve` and `allowed_dirs` (snake_case).
  - Socrates BLOCKING concern is real: all doctor warning strings must use `auto-approve = false`.

---

## File Map by Item

### Item C — Doctor Refactor (build first)

**`src/mcp/bridge/backends.rs`**

- Lines 1-15: `BackendConfig` struct — add `display_name: &'static str` field after `name` (line 5).
- Lines 18-25: `BackendSpec` struct — add `display_name: &'static str` field after `name` (line 19).
- Lines 27-68: `BACKEND_SPECS` array — add `display_name` to each of the 5 entries.
  - Line 28: `gemini` → `display_name: "Gemini CLI"`
  - Line 36: `codex` → `display_name: "Codex CLI"`
  - Line 44: `claude` → `display_name: "Claude CLI"`
  - Line 52: `grok` → `display_name: "Grok CLI"`
  - Line 60: `ollama` → `display_name: "Ollama"`
- Line 96-102: `discover_backends()` return value `BackendConfig { name, binary, model, ... }` — add `display_name: spec.display_name`.
- Line 13: Remove `#[allow(dead_code)]` annotation on `api_key_env`.
- After line 105 (after `discover_backends`): Insert new public function `all_backend_specs()` returning `Vec<(&'static str, &'static str, Option<&'static str>)>`.
- Lines 176-238 (test module): All `BackendConfig` struct literals need `display_name` added. 5 tests affected:
  - `test_build_command_args_ollama` line 177
  - `test_build_command_args_claude` line 191
  - `test_build_command_args_claude_with_system_prompt` line 204
  - `test_build_command_args_gemini_with_system_prompt` line 219

**`src/cli/doctor.rs`**

- Line 1-7: Add import `use crate::mcp::bridge::backends::all_backend_specs;`.
- Line 92-99: Change gate from `mcp_bridge.is_some()` to unconditional (or PATH-based); pass `bridge_cfg: Option<&McpBridgeConfig>` to `check_mcp_bridge`.
  - Also add `use crate::config::schema::McpBridgeConfig;` if not already present.
- Line 615: Change signature from `fn check_mcp_bridge(result: &mut DiagnosticResult)` to `fn check_mcp_bridge(result: &mut DiagnosticResult, bridge_config: Option<&McpBridgeConfig>)`.
- Lines 619-625: Replace hardcoded `backends` array with `all_backend_specs()` call.
- Lines 628-648: Replace manual loop body with the spec's new loop using `(binary, display_name, api_key_env)` tuple from `all_backend_specs()`.
  - NOTE: Current code tests `api_key_env.is_empty()` (empty string sentinel). New code tests `Option<&'static str>` — `None` vs `Some(env_var)`. The struct field is already `Option`, so drop the empty-string sentinel pattern.
- Lines 657-667: Wrap `.mcp.json` registration check in `if bridge_config.is_some() { ... }`.
- After line 655 (`any_found` check), insert the auto-approve warning block (from Item B — implement Item B here after Item C).

---

### Item D — Global Flags (build second)

**`src/cli/mcp_bridge.rs`**

- Line 29 (after `log_level` field): Add two `#[arg(skip)]` fields:
  ```rust
  #[arg(skip)]
  pub verbose: bool,
  #[arg(skip)]
  pub quiet: bool,
  ```
- Also add `--allowed-dirs` arg here (Item A shares this location):
  ```rust
  #[arg(long, value_delimiter = ',')]
  pub allowed_dirs: Option<Vec<String>>,
  ```
- Lines 34-48: Replace log-level resolution block with precedence chain: `--log-level` > `verbose` > `quiet` > default `"warn"`. Also add `"trace"` arm to the match (currently missing — current code has `other` fallback).

**`src/main.rs`**

- Line 39: Change `Command::McpBridge(args) => cli::mcp_bridge::run(args)` to forward `verbose` and `quiet`:
  ```rust
  Command::McpBridge(mut args) => {
      args.verbose = cli.verbose;
      args.quiet = cli.quiet;
      cli::mcp_bridge::run(args)
  }
  ```
  Pattern confirmed from lines 19-21 (Apply), 27-30 (Doctor), 34-36 (Loop).

---

### Item B — Auto-approve Opt-out (build third)

**`src/mcp/bridge/backends.rs`**

- Line 110-115: Add `auto_approve: bool` parameter to `build_command_args` signature (5th param).
- Lines 128-131: Wrap `args.push(flag.to_string())` in `if auto_approve { ... }`.
- Tests at lines 184, 198, 212, 226: Add `true` as 5th argument to all `build_command_args` calls.
- After existing tests: Add `test_build_command_args_claude_no_auto_approve` and `test_all_backend_specs_returns_all`.

**`src/mcp/bridge/registry.rs`**

- Lines 65-68: `TaskRegistry` struct — add `pub auto_approve: bool` field after `default_timeout`.
- Line 71-76: `TaskRegistry::new()` — add `auto_approve: bool` param; store in struct.
- Line 94: `build_command_args(backend, prompt, model_override, system_prompt)` — add `self.auto_approve` as 5th arg.
- Lines 337-338: Test `TaskRegistry::new(300)` — add `true` second arg.

**`src/mcp/bridge/server.rs`**

- Lines 27-33: `GreatBridge` struct — add `auto_approve: bool` field before `tool_router`.
- Lines 37-50: `GreatBridge::new()` — add `auto_approve: bool` param; store in struct.
- Line 60-65: `build_command_args(backend, &params.0.prompt, params.0.model.as_deref(), None)` — add `self.auto_approve`.
- Line 192-197: Same pattern in `research` tool handler.
- Line 236-240: Same pattern in `analyze_code` tool handler.
- Lines 441-446: `start_bridge()` signature — add `auto_approve: bool` param.
- Line 447: `GreatBridge::new(...)` call — add `auto_approve` arg.

**`src/config/schema.rs`**

- Line 163 (after `preset` field, before closing brace of `McpBridgeConfig`): Add:
  ```rust
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auto_approve: Option<bool>,
  ```
  TOML key: `auto-approve` (kebab-case).

**`src/cli/mcp_bridge.rs`**

- After `timeout_secs` resolution (line 73): Add `auto_approve` resolution (default `true`):
  ```rust
  let auto_approve = bridge_config
      .as_ref()
      .and_then(|c| c.auto_approve)
      .unwrap_or(true);
  ```
- Line 103: `TaskRegistry::new(timeout_secs)` — add `auto_approve` second arg.
- Line 107: `start_bridge(backends, default_backend, registry, preset)` — add `auto_approve` arg.

**`src/cli/doctor.rs`**

- After the `any_found` check block (after line 655), insert auto-approve warning for Claude.
  - Check `bridge_config.and_then(|b| b.auto_approve).unwrap_or(true)`.
  - Use `"Set auto-approve = false in [mcp-bridge]"` (kebab-case — BLOCKING fix from Socrates).
  - Gate on `command_exists("claude")`.

---

### Item A — Path Traversal (build fourth)

**`src/mcp/bridge/server.rs`**

- Line 1: Add `use std::path::PathBuf;` to imports.
- Lines 27-33: `GreatBridge` struct — add `allowed_dirs: Option<Vec<PathBuf>>` field before `auto_approve`.
- Lines 37-50: `GreatBridge::new()` — add `allowed_dirs: Option<Vec<PathBuf>>` param; store in struct.
- After `resolve_backend` helper (currently at line 347): Insert private `validate_path(&self, raw_path: &str) -> Result<(), String>` method.
- Lines 167-169 (research file loop, `for path in files {`): Insert `validate_path` guard before `std::fs::read(path)`.
- Lines 220-221 (analyze_code, `if std::path::Path::new(...).exists() {`): Insert `validate_path` guard before `std::fs::read_to_string(...)`.
- Lines 441-446: `start_bridge()` signature — add `allowed_dirs: Option<Vec<PathBuf>>` param.
- Line 447: `GreatBridge::new(...)` call — add `allowed_dirs` arg.
- After `GreatBridge::new(...)` in `start_bridge()`: Canonicalize `allowed_dirs` and emit startup warning if resolved list is empty.

**`src/config/schema.rs`**

- After `auto_approve` field (line ~164): Add:
  ```rust
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allowed_dirs: Option<Vec<String>>,
  ```
  TOML key: `allowed-dirs` (kebab-case).

**`src/cli/mcp_bridge.rs`**

- `Args` struct: `allowed_dirs` field already specified in Item D above.
- After `auto_approve` resolution: Add `allowed_dirs` merge (CLI wins over config):
  ```rust
  let allowed_dirs_raw = args.allowed_dirs
      .or_else(|| bridge_config.as_ref().and_then(|c| c.allowed_dirs.clone()));
  let allowed_dirs = allowed_dirs_raw.map(|dirs| {
      dirs.into_iter().map(std::path::PathBuf::from).collect::<Vec<_>>()
  });
  ```
- `start_bridge(...)` call — add `allowed_dirs` as final arg.

---

### Item E — Binary Size (build last)

**`Cargo.toml`**

- No `[profile.release]` section currently exists. Add after `[dev-dependencies]`:
  ```toml
  [profile.release]
  lto = true
  codegen-units = 1
  strip = true
  ```
- Current binary: 14,269,080 bytes. Target: under 12,582,912 bytes (12.5 MiB).
- `tempfile = "3.0"` is already in `[dev-dependencies]` — no new deps needed for Item A tests.

---

## Dependency Map (Build Order)

```
Item C (backends.rs display_name + all_backend_specs)
  └─ enables doctor.rs refactor (uses all_backend_specs)

Item D (mcp_bridge.rs Args skip fields + main.rs forwarding)
  └─ independent of C, can be done in parallel with C

Item B (auto_approve param ripple)
  ├─ requires: backends.rs build_command_args signature change (C done first for display_name)
  ├─ requires: registry.rs TaskRegistry::new signature change
  ├─ requires: server.rs GreatBridge struct + start_bridge signature change
  ├─ requires: mcp_bridge.rs TaskRegistry::new call site update
  └─ requires: doctor.rs auto-approve warning (builds on C's refactored check_mcp_bridge)

Item A (path traversal)
  ├─ requires: server.rs GreatBridge struct (B adds auto_approve first)
  ├─ requires: start_bridge() already has new params from B
  └─ requires: mcp_bridge.rs run() already has allowed_dirs arg slot

Item E (Cargo.toml profile)
  └─ independent; do after all code changes to avoid slow builds during development
```

---

## Exact Call-Site Count for `build_command_args`

The spec's note "4 call sites, all in server.rs" is wrong (Socrates advisory #5).
Actual count, verified:

| File | Line | Context |
|------|------|---------|
| `src/mcp/bridge/server.rs` | 60 | `prompt` tool handler |
| `src/mcp/bridge/server.rs` | 192 | `research` tool handler |
| `src/mcp/bridge/server.rs` | 236 | `analyze_code` tool handler |
| `src/mcp/bridge/registry.rs` | 94 | `spawn_task` (async task path) |

Total: 3 in server.rs, 1 in registry.rs = **4 total**. All 4 must add `auto_approve` param.

---

## Exact Call-Site Count for `TaskRegistry::new`

| File | Line | Context |
|------|------|---------|
| `src/cli/mcp_bridge.rs` | 103 | `TaskRegistry::new(timeout_secs)` |
| `src/mcp/bridge/registry.rs` | 337 | test: `TaskRegistry::new(300)` |

Both must add `auto_approve: bool` second arg.

---

## BLOCKING Concern — Socrates kebab-case

Confirmed at `src/config/schema.rs:145`:
```rust
#[serde(rename_all = "kebab-case")]
pub struct McpBridgeConfig {
```

Impact on builder:
- Rust field: `auto_approve` → TOML key: `auto-approve`
- Rust field: `allowed_dirs` → TOML key: `allowed-dirs`
- Doctor warning string at spec line 486 reads `"Set auto-approve = false in [mcp-bridge]"` — this is correct (uses hyphens). The spec's acceptance criteria elsewhere use underscores — builder must use hyphens in all user-facing strings.

---

## Risks and Technical Debt

1. **Doctor gate change (Item B5/C):** Changing `check_mcp_bridge` from gated-by-config to gated-by-PATH means users with Claude Code installed (very common) will see a new "MCP Bridge" section in `great doctor` output. This is intentional per spec but is a visible UX change. Test that it doesn't appear when no backends are on PATH.

2. **`check_mcp_bridge` signature ripple:** The function is private (lowercase) and only called in one place (line 98). The refactor from `fn check_mcp_bridge(result)` to `fn check_mcp_bridge(result, bridge_config)` is low risk.

3. **`start_bridge()` parameter growth:** After Items A and B, the signature will have 6 parameters. This is within acceptable range but should be watched. The spec does not propose a config struct to bundle them — this is fine for M complexity.

4. **Empty allowed_dirs list:** Socrates concern #4 — the spec says to add a startup `tracing::warn!` when the resolved list is empty (after canonicalization). The A6 code block in the spec omits this. The builder must add it explicitly after the filter_map in `start_bridge()`.

5. **Ollama in doctor auto-approve check:** `claude` is the only backend checked for the auto-approve warning (spec B5). Ollama has `auto_approve_flag: None` so it's correctly excluded. Other backends (gemini `-y`, grok `-y`, codex `--full-auto`) are not separately warned about — only Claude is called out. This matches the spec.

6. **`strip = true` vs `strip = "debuginfo"`:** Socrates advisory #10 suggests `strip = "debuginfo"` to preserve symbol names for backtraces. The spec uses `strip = true`. Builder should implement `strip = true` per spec but note this tradeoff in the observer report.

7. **`backend_config` variable name collision:** In `mcp_bridge.rs` `run()`, the current variable is `bridge_config` (line 55-58, `Option<McpBridgeConfig>`). The spec also uses `bridge_config`. Consistent — no rename needed.

---

## Files Modified (Summary)

| File | Lines Changed | Items |
|------|--------------|-------|
| `src/mcp/bridge/backends.rs` | ~30 lines + tests | B, C |
| `src/mcp/bridge/registry.rs` | ~8 lines + test | B |
| `src/mcp/bridge/server.rs` | ~25 lines | A, B |
| `src/cli/mcp_bridge.rs` | ~25 lines | A, B, D |
| `src/cli/doctor.rs` | ~35 lines | B, C |
| `src/config/schema.rs` | ~8 lines | A, B |
| `src/main.rs` | ~4 lines | D |
| `Cargo.toml` | ~4 lines | E |

No new files created. No new crate dependencies.
