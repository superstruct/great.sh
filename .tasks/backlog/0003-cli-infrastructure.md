# 0003: CLI Framework -- Shared Infrastructure

**Priority:** P0 (foundation)
**Type:** feature
**Module:** `src/cli/` + `src/main.rs`
**Status:** pending

## Context

The current CLI framework in `src/cli/mod.rs` defines a `Cli` struct with a `Command` enum dispatched in `src/main.rs`. All 10 subcommands are stubbed with `println!("not yet implemented")`. There are no shared output helpers, no global flags for verbosity or interactivity control, and no config auto-discovery wired into the dispatch pipeline.

The `colored` crate (3.0) and `indicatif` crate (0.17) are already in `Cargo.toml` dependencies but unused. The `config::discover_config()` function exists but is not called from `main.rs`.

## Requirements

1. **Colored output helper module** (`src/cli/output.rs`): Implement functions `success(msg: &str)`, `warning(msg: &str)`, `error(msg: &str)`, `info(msg: &str)` that print to stderr with consistent formatting. Success: green with check mark prefix. Warning: yellow with warning prefix. Error: red with X prefix. Info: blue with info prefix. Each function respects a global verbosity level -- `quiet` suppresses info and success, `verbose` is the default pass-through. Use the `colored` crate for ANSI color output.

2. **Progress spinner**: Add a `spinner(message: &str) -> ProgressBar` convenience function in `output.rs` that creates an `indicatif::ProgressBar` with a spinner style and the given message. Callers use `.finish_with_message()` when done. This gives all subcommands a consistent loading indicator.

3. **Global flags on the `Cli` struct**: Add `--verbose` (`-v`), `--quiet` (`-q`), and `--non-interactive` flags as clap args on the top-level `Cli` struct (not on each subcommand). These must be parsed before subcommand dispatch and made available to subcommand `run()` functions. Consider passing them via a shared `Context` struct or by storing them in a thread-local / static after parse.

4. **Config auto-discovery in dispatch**: In `main.rs`, after parsing CLI args but before dispatching to a subcommand, attempt `config::discover_config()`. If found, load it with `config::load()` and pass the `GreatConfig` (or `None` if not found) to subcommand handlers. The `init` subcommand should work without an existing config (it creates one). Other subcommands should warn if no config is found but not hard-fail (doctor, status can report "no config found").

5. **Wire output helpers into existing stubs**: Replace the `println!("... not yet implemented")` in each subcommand stub with `output::warning("... not yet implemented")` so all output immediately uses the new formatting system. Update each subcommand's `run()` signature if needed to accept the shared context.

## Acceptance Criteria

- [ ] `output::success("done")` prints a green-colored line with check mark prefix to stderr, and `output::error("failed")` prints red with X prefix.
- [ ] `great --verbose status` and `great --quiet status` are both accepted by the CLI parser without errors (integration test via `assert_cmd`).
- [ ] Running `great status` from a directory without `great.toml` prints a warning (not a hard error/panic) and exits cleanly.
- [ ] Running `great init` from a directory without `great.toml` does not error on missing config (init creates config).
- [ ] All 10 existing subcommand stubs use `output::warning()` instead of raw `println!()`.

## Dependencies

- Task 0002 (config schema) should ideally land first or concurrently, since the config struct loaded in dispatch comes from that schema. However, the current minimal `GreatConfig` is sufficient to wire up the plumbing -- the schema can be enriched independently.

## Notes

- The `Cli` struct currently has no fields other than `command`. Adding global args means changing `Cli` to:
  ```rust
  pub struct Cli {
      #[arg(short, long, global = true)]
      pub verbose: bool,
      #[arg(short, long, global = true)]
      pub quiet: bool,
      #[arg(long, global = true)]
      pub non_interactive: bool,
      #[command(subcommand)]
      pub command: Command,
  }
  ```
  The `global = true` attribute makes these flags available before or after the subcommand name.
- For the shared context pattern, a simple struct works:
  ```rust
  pub struct Context {
      pub config: Option<GreatConfig>,
      pub verbose: bool,
      pub quiet: bool,
      pub non_interactive: bool,
  }
  ```
  Each subcommand's `run()` changes from `run(args: Args) -> Result<()>` to `run(args: Args, ctx: &Context) -> Result<()>`.
- The `colored` crate respects `NO_COLOR` env var automatically, which is good for CI environments.
- Output goes to stderr so that stdout remains clean for machine-parseable output in future commands (e.g., `great status --json`).
