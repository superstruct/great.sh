# 0031 — Smoke Tests for `loop` and `mcp-bridge` Subcommands

| Field      | Value                    |
|------------|--------------------------|
| Priority   | P2                       |
| Type       | feature (test coverage)  |
| Module     | tests/cli_smoke.rs       |
| Status     | backlog                  |
| Complexity | M                        |

## Context

`tests/cli_smoke.rs` has 80+ tests covering `status`, `doctor`, `diff`, `apply`,
`template`, `mcp`, `vault`, `sync`, `update`, and `statusline`.

Two subcommands shipped in iterations 021 and 029 have **zero integration test
coverage**:

- `great loop` — install / status / uninstall (3 sub-paths)
- `great mcp-bridge` — starts an async stdio JSON-RPC 2.0 server

The `loop` paths are straightforward: `loop status` exits cleanly when not
installed, and `loop install --force` into a tempdir writes files that can be
asserted. The `mcp-bridge` server blocks on stdin so live-server tests are out of
scope; the testable surface is `--help`, the invalid-preset error path, and the
"no backends on PATH" exit code (exit 1, not panic).

If a contributor breaks the `loop install` write path or the `mcp-bridge` argument
parser, CI gives no signal today.

## Requirements

1. Add `loop` section to `tests/cli_smoke.rs` with tests for `loop --help`,
   `loop status` (not installed), `loop install --force` into a `TempDir`, and
   `loop uninstall` (no-op when not installed).

2. Add `mcp-bridge` section to `tests/cli_smoke.rs` with tests for
   `mcp-bridge --help` and `mcp-bridge --preset invalid_preset` (exit nonzero,
   stderr contains "invalid preset").

3. `loop install --force` test must assert that at least one agent file is
   written under the test home dir (use `HOME` env override) — proving the
   write path executed, not just that the command exited zero.

4. `loop status` with no prior install must exit zero and print output that
   includes the word "not installed" (case-insensitive).

5. No test may require network access, a real keychain, or an AI CLI binary on
   PATH; all `loop` tests must use a `TempDir` as `HOME` via the `HOME` env var.

## Acceptance Criteria

- [ ] `cargo test` passes on ubuntu-latest and macos-latest with no new `#[ignore]`
      attributes added.
- [ ] `great loop --help` test asserts stdout contains "Install" and "Status".
- [ ] `great loop status` (fresh HOME tempdir) test asserts exit 0 and stderr/stdout
      contains "not installed" (case-insensitive).
- [ ] `great loop install --force` (fresh HOME tempdir) test asserts exit 0 and that
      at least one file exists under `$HOME/.claude/agents/`.
- [ ] `great mcp-bridge --preset invalid_preset_xyz` test asserts exit nonzero and
      stderr contains "invalid preset".

## Dependencies

- None. All targeted code paths are already fully implemented.
- `loop_cmd.rs` install logic writes to `~/.claude/agents/` — HOME override
  makes this hermetic.

## Notes

- `mcp-bridge --help` will exit 0 immediately with no blocking; it is safe to
  test without stdin tricks.
- `mcp-bridge` with a valid preset but no AI CLI on PATH currently calls
  `anyhow::bail!` and exits nonzero — this is also testable but is covered by
  the invalid-preset criterion above.
- `loop uninstall` when not installed should be a graceful no-op (exit 0); if
  it panics or returns exit 1 that is a bug to fix alongside the test.
- Do not add `mcp-bridge` live-server tests (they require a real backend and
  blocking stdin); those belong in a separate task if needed.
