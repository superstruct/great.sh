# 0004: `great status` -- Environment State Reporter

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/cli/status.rs`
**Status:** in-progress

## Context

The `great status` command is substantially implemented. It already reads `great.toml` via `config::discover_config()` and `config::load()`, gracefully handles missing config (shows a warning and platform-only info), checks declared tools against installed state using `command_exists()` and `get_tool_version()`, checks MCP server command availability, checks required secrets against environment variables, and produces color-coded output via the `output` helpers.

Current implementation gaps: the `--json` output is minimal (only platform/arch/shell -- does not include tools, agents, MCP, or secrets), the `--verbose` flag only affects the platform section, and there are no integration tests.

## Requirements

1. **Expand JSON output mode**: The `run_json()` function currently emits only `platform`, `arch`, and `shell`. It must include the full status report: tools (with installed/missing state and detected versions), MCP servers (with command availability), agents, and secrets (set/missing). Use `serde_json` for proper serialization instead of manual format strings.

2. **Expand verbose mode**: The `--verbose` flag currently only affects the platform capabilities section. In verbose mode, tool checks should also show the full version string from `--version` output, and MCP servers should show the full command path and args.

3. **Add integration tests**: Write tests using `assert_cmd` that verify: (a) `great status` exits successfully with no `great.toml` present, (b) `great status` exits successfully with a valid `great.toml` in a temp directory, (c) `great status --json` outputs valid JSON to stdout, (d) `great status --verbose` is accepted without error.

4. **Handle config parse errors gracefully**: The current code calls `.unwrap_or_default()` on the config path's `to_str()` conversion. Replace this with proper error propagation or a clear warning message if the path contains non-UTF-8 characters.

5. **Add exit code semantics**: Return a non-zero exit code (via `std::process::exit(1)` or by returning an error) when critical issues are detected (e.g., required secrets missing, declared tools not installed) to support CI usage. The `--json` mode should always exit 0 and encode the status in the JSON payload.

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/status.rs`.
- [ ] `great status --json` emits valid JSON that includes tool, MCP, and secret status fields.
- [ ] Integration tests pass: status with no config, status with valid config, status with `--json`, status with `--verbose`.
- [ ] Running `great status` in a directory without `great.toml` prints a warning and exits cleanly (exit code 0).
- [ ] No `.unwrap()` calls remain in `src/cli/status.rs` production paths (the existing `.unwrap_or_default()` on path conversion is replaced).

## Dependencies

- Task 0001 (platform detection) -- already landed; `detect_platform_info()`, `command_exists()` are available.
- Task 0002 (config schema) -- already landed; `GreatConfig`, `ToolsConfig`, `McpConfig`, `SecretsConfig` are defined.
- Task 0003 (CLI infrastructure) -- already landed; `output` helpers, global flags, and config auto-discovery are wired.

## Notes

- The `get_tool_version()` helper duplicates the same function in `doctor.rs` (`get_command_version()`). Consider extracting to a shared utility in `src/cli/output.rs` or a new `src/cli/util.rs` to avoid divergence. This is not blocking but should be noted for a future refactor task.
- The `serde_json` crate is not currently in `Cargo.toml` dependencies -- it will need to be added for proper JSON serialization. Alternatively, use `serde_json` re-exported from another dependency if available.
- The status command is the primary "read-only" diagnostic tool and should never modify system state.
