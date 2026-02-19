# 0006: `great diff` -- Declared vs Actual State

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/cli/diff.rs`
**Status:** in-progress

## Context

The `great diff` command is currently a stub that prints "not yet implemented". The `Args` struct defines a `--config` flag for specifying an alternative config file path. The purpose of `diff` is to compare the declarations in `great.toml` against the actual system state and produce a clear, color-coded report of what needs to change -- serving as the "preview" step before `great apply`.

The building blocks already exist: `config::discover_config()` and `config::load()` for reading config, `platform::command_exists()` for checking tool installation, `get_tool_version()` (in `status.rs`) for detecting installed versions, and `output::success/warning/error` for color-coded messaging. The `status` command already performs these checks but presents them as a status report, not as a diff. The `diff` command should present the same data in a diff-oriented format: what is missing, what is present, what is partial.

## Requirements

1. **Compare tools against system state**: For each tool declared in `[tools]` (both runtimes and `[tools.cli]`), check whether it is installed via `command_exists()` and what version is detected via `get_tool_version()`. Display missing tools with a green `+` prefix (needs to be added), installed tools with a dim/grey indicator (no action needed), and version mismatches with a yellow `~` prefix (partial match). Use the `colored` crate for output formatting.

2. **Compare MCP servers against system state**: For each MCP server declared in `[mcp.*]`, check whether the declared command exists on PATH. Also check whether a `.mcp.json` file exists in the current directory (indicating MCP config has been written). Show servers with missing commands as needing installation, and servers not yet in `.mcp.json` as needing configuration.

3. **Compare secrets against environment**: For each secret in `secrets.required` and each `${SECRET_NAME}` reference found by `find_secret_refs()`, check whether the environment variable is set. Display unresolved secrets with a red indicator showing they need to be provided before `great apply` can inject them.

4. **Support `--config` flag**: If `--config <path>` is provided, load that file instead of running `config::discover_config()`. If neither the flag nor auto-discovery finds a config, print a clear error message and exit with code 1. The diff command requires a config to diff against -- unlike `status` and `doctor`, it cannot produce useful output without one.

5. **Summary line**: At the end of the diff output, print a summary: "N items to install, M items to configure, K secrets to resolve" (or "Environment matches configuration -- nothing to do" if fully resolved). This gives a quick answer to "do I need to run `great apply`?"

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/diff.rs`.
- [ ] Running `great diff` with a sample `great.toml` that declares tools not present on the system shows those tools with `+` prefix in green.
- [ ] Running `great diff` with all declared tools installed shows "nothing to do" or equivalent in the summary.
- [ ] Running `great diff` without a `great.toml` (and without `--config`) prints an error and exits with code 1.
- [ ] Integration tests pass: diff with missing tools shows additions, diff with satisfied config shows clean, diff without config errors gracefully.

## Dependencies

- Task 0001 (platform detection) -- already landed; `command_exists()` is available.
- Task 0002 (config schema) -- already landed; `GreatConfig`, `find_secret_refs()` are available.
- Task 0003 (CLI infrastructure) -- already landed; `output` helpers are available.
- Task 0004 (status command) -- concurrent; `get_tool_version()` should be extracted to a shared location (see task 0005 requirement 3) and used here.

## Notes

- The diff output format should be visually distinct from `status`. Status answers "what is the state?", diff answers "what needs to change?" The prefixes are key: `+` (green) = needs adding, `~` (yellow) = needs updating, `-` (red) = declared but blocked (e.g., missing secret prevents MCP config).
- The `.mcp.json` file format follows the convention used by Claude Code and other AI tools: a JSON object with server names as keys and `{command, args, env}` as values. The diff command only checks for its existence and whether declared servers are present in it -- it does not write or modify it.
- This command is the logical prerequisite for `great apply` (task 0009). The apply command will likely reuse the diff computation internally to determine what actions to take.
- Consider defining a `DiffResult` struct that captures the comparison outcome programmatically, so that `apply` can call the same logic without re-parsing output. This is an architectural suggestion, not a hard requirement for this task.
