# 0027: Wire `--non-interactive` Global Flag Through to `apply` and `doctor`

**Priority:** P2
**Type:** bugfix / refactor
**Module:** `src/main.rs`, `src/cli/apply.rs`, `src/cli/doctor.rs`, `src/cli/sudo.rs`
**Status:** complete
**Completed:** 2026-02-27
**Iteration:** 024
**Commit:** `444b56c`
**Estimated Complexity:** S

---

## Context

The `--non-interactive` global flag is defined on `Cli` in `src/cli/mod.rs` (line 35) and parsed
correctly by clap. However, `src/main.rs` dispatches each subcommand by destructuring
`cli.command` and discarding `cli.non_interactive` — it is never read after parsing:

```rust
// src/main.rs — cli.non_interactive is silently dropped here
match cli.command {
    Command::Apply(args) => cli::apply::run(args),
    Command::Doctor(args) => cli::doctor::run(args),
    ...
}
```

`src/cli/sudo.rs` line 61 has an explicit TODO acknowledging this gap:

```rust
// TODO: Also accept a `non_interactive` parameter from the --non-interactive
// global CLI flag once it is wired through to apply::run() / doctor::run().
pub fn ensure_sudo_cached(is_root: bool) -> SudoCacheResult {
    ...
    if !std::io::stdin().is_terminal() {    // <-- only tty detection, flag ignored
        return SudoCacheResult::NonInteractive;
    }
```

**Impact:** A user or CI job that runs `great apply --non-interactive` or
`great doctor --fix --non-interactive` will still receive a `sudo -v` password prompt
on platforms that need sudo (Ubuntu/Debian Homebrew install, apt). The flag has no effect
on the most interactive operation in the codebase.

Similarly, `doctor::run()` calls `package_manager::available_managers(false)` at line 97
(hardcoded `false`) inside the `--fix` block — it never uses `non_interactive` even if
it were threaded in.

---

## Requirements

1. `main.rs` must extract `non_interactive` from `Cli` and forward it to both `apply::run()`
   and `doctor::run()` (and any other subcommand that calls `ensure_sudo_cached` or
   `available_managers`).

2. `apply::Args` must gain a `non_interactive` field (or `apply::run()` must accept it as a
   second parameter). The field should not be exposed as a CLI arg (it is global); it should
   be set by `main.rs` before calling `run()`.

3. `doctor::Args` must gain the same treatment.

4. `ensure_sudo_cached(is_root: bool)` signature must be extended to
   `ensure_sudo_cached(is_root: bool, non_interactive: bool)`. When `non_interactive` is
   `true`, return `SudoCacheResult::NonInteractive` immediately (before the `stdin.is_terminal()`
   check), and remove the TODO comment.

5. The `available_managers(false)` call in `doctor.rs` line 97 must pass the actual
   `non_interactive` value rather than a hardcoded `false`.

---

## Acceptance Criteria

- [x] `great apply --non-interactive` on a platform that would otherwise trigger `sudo -v`
      exits without any sudo prompt (verified by running with `stdin` redirected AND without
      redirection but with the flag present).
- [x] `great doctor --fix --non-interactive` similarly skips the sudo prompt.
- [x] `cargo clippy` produces zero warnings (no dead-code warning for the parsed-but-unused
      `non_interactive` field on `Cli`).
- [x] `cargo test` passes with no regressions; the existing `already_root_returns_immediately`
      and `keepalive_drop_signals_stop` unit tests in `sudo.rs` still pass.
- [x] The TODO comment at `src/cli/sudo.rs:61` is removed.

---

## Dependencies

None. All affected files are self-contained within the `src/cli/` tree.

---

## Notes

- The `non_interactive` field must **not** appear as a subcommand-level `--non-interactive`
  flag on `apply` or `doctor` — it is already declared `global = true` on `Cli`. Adding it
  again at the subcommand level would create a duplicate-flag parse error. The correct
  pattern is to set it on the `Args` struct **after** clap parsing in `main.rs`, or to pass
  it as an extra argument to `run()`.
- `src/platform/package_manager.rs` already correctly accepts and propagates `non_interactive`
  through `Apt::new(non_interactive)` and `available_managers(non_interactive)` — only the
  call site in `doctor.rs` hardcodes `false`.
- `src/cli/statusline.rs`, `sync.rs`, `vault.rs`, `mcp.rs`, `template.rs`, `update.rs`,
  `loop_cmd.rs` do not call `ensure_sudo_cached` and do not need changes.
