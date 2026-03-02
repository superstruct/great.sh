# Task 0030 — Design Review: MCP Bridge Hardening CLI output

**Reviewer:** Dieter Rams
**Date:** 2026-02-28
**Files reviewed:**
- `/home/isaac/src/sh.great/src/cli/mcp_bridge.rs`
- `/home/isaac/src/sh.great/src/cli/doctor.rs` (lines 613–691, `check_mcp_bridge`)
- `/home/isaac/src/sh.great/src/cli/mod.rs` (line 80)
- `/home/isaac/src/sh.great/src/cli/output.rs`

**Commands run:**
- `cargo run -- mcp-bridge --help`
- `cargo run -- doctor`
- `cargo run -- diff --help` (reference baseline)
- `cargo run -- apply --help` (reference baseline)

---

## Verdict: REJECTED

Two issues require correction. One is a structural display defect (Principle 8 —
Thorough). One is a redundant control surface that creates user confusion
(Principle 2 — Useful / Principle 10 — As little as possible). Both are
correctable without architectural change.

---

## Issues

### Issue 1 — Principle 8 (Thorough): global flags interleaved through option list

`great mcp-bridge --help` shows:

```
Options:
      --preset <PRESET>              Tool preset: ...
  -v, --verbose                      Increase output verbosity
      --backends <BACKENDS>          Comma-separated list of ...
  -q, --quiet                        Suppress all output except errors
      --non-interactive              Disable interactive prompts ...
      --timeout <TIMEOUT>            Per-task timeout in seconds
      --log-level <LOG_LEVEL>        Logging verbosity for stderr: ...
      --allowed-dirs <ALLOWED_DIRS>  Restrict file-reading tools ...
  -h, --help                         Print help
```

The global flags (`-v`, `-q`, `--non-interactive`) are scattered through the
list at alphabetical positions v, q, and n respectively. They appear between
`--preset` and `--backends`, and between `--backends` and `--timeout`. The
user cannot distinguish at a glance which flags are subcommand-specific and
which are global.

Compare `great diff --help`:

```
Options:
      --config <CONFIG>  Path to configuration file to diff against
  -v, --verbose          Increase output verbosity
  -q, --quiet            Suppress all output except errors
      --non-interactive  Disable interactive prompts (for CI/automation)
  -h, --help             Print help
```

In `diff`, `diff apply`, and all other subcommands, the globals happen to cluster
at the end because the subcommand-specific args begin with letters earlier than
v/q/n. `mcp-bridge` has five subcommand-specific args spanning a-l-p-t, which
causes the globals to be alphabetically interleaved throughout the list. The
result exposes the absence of explicit section grouping that all other subcommands
accidentally benefit from.

Clap supports `#[command(next_help_heading = "...")]` to group args into named
sections. Adding a "Bridge Options" heading for the five subcommand-specific args
would separate them from the inherited globals cleanly:

```
Bridge Options:
      --preset <PRESET>
      --backends <BACKENDS>
      --timeout <TIMEOUT>
      --log-level <LOG_LEVEL>
      --allowed-dirs <ALLOWED_DIRS>

Options:
  -v, --verbose
  -q, --quiet
      --non-interactive
  -h, --help
```

This would make `mcp-bridge` the most clearly structured subcommand in the
CLI, appropriate for the most complex subcommand in the binary.

**Required correction:** Add `#[command(next_help_heading = "Bridge Options")]`
before the first subcommand-specific arg in `mcp_bridge::Args` (before `preset`),
so globals separate into their own section. The five bridge-specific args:
`--preset`, `--backends`, `--timeout`, `--log-level`, `--allowed-dirs`.

---

### Issue 2 — Principle 2 (Useful) / Principle 10 (As little as possible): dual verbosity controls

`great mcp-bridge --help` now exposes two overlapping verbosity controls:

```
  -v, --verbose          Increase output verbosity
  -q, --quiet            Suppress all output except errors
      --log-level <LOG_LEVEL>   Logging verbosity for stderr: off, error, warn, info, debug, trace
```

Item D of this task wired `--verbose` and `--quiet` into mcp-bridge (correctly
addressing the 0029 REJECTED issue). The implementation is: `--verbose` maps
to `"debug"`, `--quiet` maps to `"error"`, `--log-level` overrides both.
Three flags, one setting.

This creates a genuine usability question: a user reading the help cannot
determine which to use. The `--log-level` flag is more expressive; `--verbose`
and `--quiet` are more ergonomic for quick invocations. Together, they suggest
uncertainty about which control surface is canonical.

The `doctor.rs` auto-approve warning illustrates the conflict in practice: the
bridge is a stdio server process. It runs until killed. After startup it emits
no interactive output. The global `--verbose`/`--quiet` convention is designed
for commands that produce user-facing output (doctor, diff, apply). For a server
process, log level is the correct model — it governs tracing, not user messaging.

There are two acceptable resolutions:

**Option A (preferred):** Retain both surfaces but clarify the relationship in
the `--log-level` help text:

```rust
/// Logging verbosity for stderr: off, error, warn, info, debug, trace.
/// Overrides --verbose and --quiet when both are specified.
```

This is honest (Principle 6) — it states the precedence rule — and minimal
(Principle 10) — it requires no code change beyond one line of doc comment.

**Option B:** Remove `--log-level` and map the five-level tracing filter to
`--verbose`/`--quiet` with a middle ground, accepting that `trace` and `off`
become unreachable via CLI. This reduces the surface to zero custom flags for
verbosity, but loses `trace` and `off` levels needed for debugging the bridge.

Option A is the correct choice.

**Required correction:** Amend the `--log-level` doc comment in
`/home/isaac/src/sh.great/src/cli/mcp_bridge.rs` (line 27–29) to document
its precedence over `--verbose`/`--quiet`. Single-line change.

---

## What is working well

**All three 0029 REJECTED issues are resolved.** The em-dash is correct in
`mod.rs` line 80 (`— no Node.js required`), in `doctor.rs` line 685
(`— run \`great apply\` to register`), and `--verbose`/`--quiet` are now wired
through to the log-level resolver. Clean execution.

**The `check_mcp_bridge` refactor (Item C) is visually correct.** The function
now calls `all_backend_specs()` and produces output identical in structure to
the 0029 version. No regression. The section header, pass/warn/fail cadence,
and trailing `println!()` are all consistent with every other doctor section.

**The auto-approve warning is appropriately surfaced.** The message appears in
the "MCP Bridge" section of `great doctor` output, which is the correct location
(grouped with the bridge's other checks). It correctly uses `warn` (yellow ⚠),
not `fail` (red ✗), because this is a configuration advisory, not a broken state.
The counterpart pass message when `auto-approve = false` is set uses the correct
`pass` (green ✓). Good use of the signal hierarchy.

**The `--allowed-dirs` help text is precise.** "Restrict file-reading tools
(research, analyze_code) to paths under these directories. Comma-separated.
Omit to allow all paths." This names the exact tools affected, states the scope,
and documents the omit-means-allow default. Principle 6 (Honest) satisfied.

**The auto-approve warning text is honest about the flag name.** Quoting the
literal flag `--dangerously-skip-permissions` in the warning message, and
providing the exact TOML key `auto-approve = false` (correctly kebab-case after
the Socrates BLOCKING correction), makes the message immediately actionable.
Principle 4 (Understandable) satisfied.

**The "no backends found" consolidated warning is appropriate.** When no
backends are on PATH, a single warning is emitted rather than five individual
"not found" lines. Principle 10 applied correctly.

---

## Summary

| # | Principle | File | Description | Severity |
|---|-----------|------|-------------|----------|
| 1 | 8 — Thorough | `src/cli/mcp_bridge.rs` | Global flags interleaved through option list — add `next_help_heading` | Blocking |
| 2 | 2/10 — Useful/As little as possible | `src/cli/mcp_bridge.rs` line 27 | `--log-level` precedence undocumented — add one doc comment line | Blocking |

Issue 1 requires one attribute annotation. Issue 2 requires one doc comment
addition. No structural changes.

---

*Less, but better.*
