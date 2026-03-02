# 0009: `great apply` -- The Core Provisioning Engine

**Priority:** P0 (blocks everything -- this is the primary value-delivery command)
**Type:** feature
**Module:** `src/cli/apply.rs`
**Status:** pending

## Context

The `great apply` command is the central command of the entire CLI. It reads `great.toml`, computes the diff between declared state and actual system state, and provisions everything needed to bring the environment into compliance. It is currently a stub that prints "not yet implemented". The `Args` struct already defines `--config` (alternative config path), `--dry-run` (preview mode), and `--yes` (skip confirmation prompts).

This command orchestrates all the infrastructure built in prior tasks: platform detection (0001) for OS-aware decisions, config schema (0002) for reading declarations, CLI infrastructure (0003) for output and progress, diff logic (0006) for computing what needs to change, package managers (0007) for installing tools, and the runtime manager (0008) for provisioning runtimes via mise.

## Requirements

1. **Read and validate configuration**: Load `great.toml` (via `--config` path or auto-discovery). Run `GreatConfig::validate()` and abort on errors (warnings are displayed but do not block). If no config is found, print an error directing the user to `great init` and exit with code 1.

2. **Compute diff and plan**: Reuse or invoke the diff logic from task 0006 to determine what needs to change. Build an ordered execution plan: (a) install package managers if needed (e.g., Homebrew on macOS), (b) install runtimes via mise (task 0008), (c) install CLI tools via appropriate package manager (task 0007), (d) configure MCP servers, (e) inject secrets into MCP configs. Display the plan to the user before executing.

3. **Support `--dry-run` mode**: When `--dry-run` is passed, compute and display the full execution plan but do not execute any steps. The output should clearly indicate it is a preview. Exit with code 0 after displaying the plan.

4. **Support `--yes` for non-interactive mode**: By default, display the execution plan and prompt "Proceed? [y/N]" before executing. When `--yes` is passed, skip the prompt and execute immediately. This is essential for CI/automation usage.

5. **Execute provisioning steps with progress**: For each step in the plan, display progress using `output::spinner()`. On success, show `output::success()` with what was installed/configured. On failure, show `output::error()` with the error message and continue to the next step (do not abort the entire run on a single failure). At the end, display a summary: "X installed, Y configured, Z failed".

## Acceptance Criteria

- [ ] `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/apply.rs`.
- [ ] `great apply --dry-run` with a sample `great.toml` displays the execution plan without modifying the system, and exits with code 0.
- [ ] `great apply --yes` with a `great.toml` declaring an already-installed tool completes idempotently (reports "already installed", does not reinstall).
- [ ] Running `great apply` without a `great.toml` prints an error and exits with code 1.
- [ ] Integration tests verify: dry-run output, missing config error, and idempotent re-run behavior.

## Dependencies

- Task 0002 (config schema) -- already landed; `GreatConfig`, `validate()`, `find_secret_refs()` are available.
- Task 0003 (CLI infrastructure) -- already landed; `output` helpers, `spinner()`, global flags are available.
- Task 0006 (diff command) -- should land first or concurrently; diff logic is reused for plan computation.
- Task 0007 (package manager) -- must land first; `PackageManager` trait and implementations are required for tool installation.
- Task 0008 (runtime manager) -- must land first; `MiseManager` is required for runtime provisioning.

## Notes

- **Idempotency is the cardinal rule**: `great apply` must be safe to run multiple times. Every install step checks `is_installed()` first. Every config write checks if the file already has the correct content. The user should be able to run `great apply` after every `git pull` without fear.
- **MCP server configuration**: The apply command writes `.mcp.json` in the project root (or the location specified by the agent tool -- e.g., `~/.claude/` for Claude Code). The format is a JSON object with server names as keys and `{command, args, env}` as values. Secret references (`${SECRET_NAME}`) in env values are resolved from environment variables at write time. If a secret is not set, the apply command should warn but still write the config with the unresolved reference (the MCP server will fail at runtime, but the config file is still valid).
- **Execution order matters**: Package managers must be installed before runtimes (mise needs Homebrew or curl), runtimes before CLI tools (some CLI tools may depend on Node.js or Python), CLI tools before MCP config (MCP commands must exist on PATH).
- **Error recovery**: Individual step failures should not abort the entire run. The summary at the end tells the user what failed. They can fix the issue and re-run `great apply` (idempotency ensures already-completed steps are skipped).
- This task has the highest dependency count of any task so far. It should be the last to be implemented in Phase 3 but is P0 because it is the command that delivers the product's core value proposition.
