# 0005: `great doctor` -- Environment Diagnostician

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/cli/doctor.rs`
**Status:** in-progress

## Context

The `great doctor` command is substantially implemented with five diagnostic sections: platform checks (architecture, root status, package manager availability), essential tools (git, curl, node, npm, cargo with version detection, plus mise recommendation), AI agents (claude, codex commands and API key environment variables), config validation (parse, validate with `ConfigMessage`, secret reference resolution), and shell checks (PATH includes `~/.local/bin`). A summary with pass/warn/fail counts is displayed at the end.

Current implementation gaps: the `--fix` flag is declared but auto-fix logic is not implemented (it prints a note and falls through to report-only mode), there is a duplicated `get_command_version()` helper that also exists in `status.rs`, and there are no integration tests. The diagnostic checks are also not extensible -- adding a new check category requires modifying the `run()` function directly.

## Requirements

1. **Add integration tests**: Write tests using `assert_cmd` that verify: (a) `great doctor` exits successfully and includes the "Summary" section in output, (b) `great doctor --fix` is accepted without error, (c) output includes checks for git and cargo (which are known to be installed on the dev machine), (d) running without `great.toml` produces a warning in the config section but does not panic.

2. **Implement basic auto-fix for `--fix` mode**: For the initial iteration, support at least these fixes: (a) create `~/.local/bin` directory if it does not exist, (b) suggest shell profile edits for PATH issues (print the `export` line to add). Full package installation fixes are deferred to the `great apply` command (task 0009). Each fix attempt should report success or failure.

3. **Extract `get_command_version()` to shared utility**: The function duplicated between `doctor.rs` and `status.rs` should be moved to a shared location (e.g., `src/cli/util.rs` or added to `src/platform/detection.rs` alongside `command_exists()`). Both modules should import from the shared location.

4. **Add MCP server checks**: The doctor does not currently check MCP server requirements. If `great.toml` declares MCP servers, doctor should verify that the declared commands exist on PATH, similar to what `status.rs` already does. This fills a diagnostic gap between status (which shows state) and doctor (which should diagnose and advise).

5. **Validate exit code semantics**: `great doctor` should return exit code 0 when no failures are found (warnings are acceptable), and exit code 1 when any check fails. This enables CI pipelines to gate on `great doctor` passing.

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/doctor.rs`.
- [ ] Integration tests pass: doctor runs successfully, `--fix` is accepted, git/cargo checks appear in output, no panic without config.
- [ ] `get_command_version()` is defined in exactly one location and imported by both `doctor.rs` and `status.rs`.
- [ ] When `great.toml` declares MCP servers, `great doctor` reports whether each server's command is available on PATH.
- [ ] `great doctor` returns exit code 1 when any check fails, exit code 0 otherwise.

## Dependencies

- Task 0001 (platform detection) -- already landed; `detect_platform_info()`, `command_exists()`, `PlatformCapabilities` are available.
- Task 0002 (config schema) -- already landed; `GreatConfig::validate()`, `ConfigMessage`, `find_secret_refs()` are available.
- Task 0003 (CLI infrastructure) -- already landed; `output` helpers and global flags are wired.
- Task 0004 (status command) -- concurrent; the shared `get_command_version()` extraction affects both.

## Notes

- The `DiagnosticResult` struct is currently private to `doctor.rs`. If other commands need structured diagnostic output in the future, consider making it public in a shared module.
- The `--fix` mode should be conservative: only perform fixes that are safe and reversible. Creating directories and suggesting profile edits qualify. Installing packages does not -- that is `great apply`'s domain.
- The `check_config` function currently uses `.unwrap_or_default()` on `path.to_str()` -- same issue noted in task 0004. Fix here as well.
- Consider adding a `--json` flag (not in this task) for CI-friendly output in a future iteration, mirroring the status command's JSON mode.
