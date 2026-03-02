# 0006: `great diff` -- Declared vs Actual State

**Priority:** P1 (this iteration)
**Type:** feature
**Module:** `src/cli/diff.rs`
**Status:** in-progress

## Context

The `great diff` command has a substantial implementation in place (not a stub). It already loads
config, iterates declared tools (runtimes and CLI), MCP servers, and secrets, and prints colored
diff lines with `+` (green) and `~` (yellow) markers. A summary line is printed at the end.

Two gaps remain before this task is done:

1. **Version comparison is absent.** The current code only checks `command_exists()` for tools
   and shows them as either missing (`+`) or silently omitted if installed. It does not call
   `util::get_command_version()` (in `src/cli/util.rs`) to detect version mismatches and display
   installed-but-wrong-version tools with `~` (yellow). That gap means the `~` marker is only
   used for MCP `.mcp.json` state, not for tools.

2. **No-config path exits with code 0 instead of code 1.** When `discover_config()` fails and no
   `--config` flag is given, the current code calls `output::error()` then `return Ok(())`, which
   exits 0. The requirement and acceptance criteria specify exit code 1 in this case.

The shared version utility is `crate::cli::util::get_command_version(cmd: &str) -> Option<String>`,
which was extracted during task 0004/0005 work and already lives in `src/cli/util.rs`. The `diff`
module does not currently import it.

All other building blocks are confirmed present:
- `config::discover_config()` and `config::load()` -- `src/config/mod.rs`
- `platform::command_exists()` -- `src/platform/detection.rs`
- `config::GreatConfig::find_secret_refs()` -- `src/config/schema.rs`
- `output::success/warning/error/info/header` -- `src/cli/output.rs`

## Requirements

1. **Compare tools against system state with version detection**: For each tool declared in
   `[tools]` (both runtimes and `[tools.cli]`), check whether it is installed via
   `command_exists()`. If not installed, show with green `+` prefix. If installed, call
   `util::get_command_version()` and compare against the declared version string; if the detected
   version matches the declared version (substring match is acceptable), show with a dim/grey
   marker (no action needed). If installed but the version does not match, show with yellow `~`
   prefix. Use the `colored` crate.

2. **Compare MCP servers against system state**: For each MCP server declared in `[mcp.*]`, check
   whether the declared command exists on PATH. Also check whether a `.mcp.json` file exists in
   the current directory. Show servers with missing commands as `+` (green, needs installation),
   and servers whose command is available but that are absent from `.mcp.json` as `~` (yellow,
   needs configuration). Servers that are disabled (`enabled = false`) should be skipped, matching
   the behaviour already in `doctor.rs`.

3. **Compare secrets against environment**: For each secret in `secrets.required` and each
   `${SECRET_NAME}` reference found by `find_secret_refs()`, check whether the environment
   variable is set. Display unresolved secrets with a red `-` indicator showing they need to be
   provided before `great apply` can run.

4. **Exit code 1 when no config is available**: If `--config <path>` is provided but the file
   does not exist, return an error. If neither the flag nor auto-discovery finds a config, print a
   clear error message and exit with code 1 (not 0 as the current code does). Use
   `std::process::exit(1)` after the error print, consistent with the pattern in `status.rs` and
   `doctor.rs`.

5. **Numeric summary line**: At the end of the diff output, print: "N items to install, M items
   to configure, K secrets to resolve" (or "Environment matches configuration -- nothing to do" if
   fully resolved). Count each category separately from the diff walk.

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/diff.rs`.
- [ ] Running `great diff` with a sample `great.toml` that declares tools not present on the
  system shows those tools with `+` prefix in green.
- [ ] Running `great diff` with all declared tools installed and version matching shows "nothing
  to do" in the summary and exits 0.
- [ ] Running `great diff` without a `great.toml` (and without `--config`) prints an error and
  exits with code 1.
- [ ] Integration tests pass: diff with missing tools shows additions, diff with satisfied config
  shows clean, diff without config errors gracefully with exit code 1.

## Dependencies

- Task 0001 (platform detection) -- landed; `command_exists()` available in
  `src/platform/detection.rs`.
- Task 0002 (config schema) -- landed; `GreatConfig`, `find_secret_refs()` available in
  `src/config/schema.rs`.
- Task 0003 (CLI infrastructure) -- landed; `output` helpers available in `src/cli/output.rs`.
- Task 0004 (status command) -- landed; `util::get_command_version()` extracted to
  `src/cli/util.rs` and ready to use.
- Task 0005 (doctor command) -- landed; `enabled` field on `McpConfig` confirmed present,
  skip-disabled pattern confirmed in `src/cli/doctor.rs`.

## Notes

- The diff output format is visually distinct from `status`. Status answers "what is the state?",
  diff answers "what needs to change?" Prefix semantics: `+` (green) = needs adding, `~` (yellow)
  = partial/needs reconfiguring, `-` (red) = blocked (e.g., missing secret prevents MCP config).
- The `.mcp.json` check uses a path relative to cwd, which is correct: this file lives in the
  project root alongside `great.toml`.
- This command is the logical prerequisite for `great apply` (task 0009). Consider defining a
  `DiffResult` struct so that `apply` can call the same diff logic programmatically rather than
  re-parsing output. This is an architectural suggestion, not a hard requirement for this task.
- The `tools.runtimes` map in the config schema includes a `"cli"` key due to TOML flattening;
  the existing code already guards against this with `if name == "cli" { continue; }`. Preserve
  this guard.
