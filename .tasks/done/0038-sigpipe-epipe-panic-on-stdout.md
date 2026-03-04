# 0038 — CLI: Handle SIGPIPE / EPIPE Gracefully Instead of Panicking

**Priority:** P2
**Type:** bugfix
**Module:** `src/main.rs`, all subcommands that write to stdout
**Status:** Backlog
**Complexity:** S
**Created:** 2026-03-02

## Context

Any `great` subcommand that writes to stdout panics (exit code 101) when the
read end of a pipe closes before the write is complete. This is a POSIX SIGPIPE
/ EPIPE condition. Rust's standard library does not install a SIGPIPE handler
by default on Linux; instead, `println!` / `write!` propagate an `io::Error`
which — when it reaches `main() -> Result<()>` — causes anyhow to print a
backtrace and exit 101 (Rust's panic exit code).

The failure is most reproducible via `great status --json` because it emits a
large single `println!` call, but the same root cause affects every subcommand
(`great doctor`, `great diff`, `great template list`, etc.).

**Reproduction** (Ubuntu 24.04, Docker container, or any Linux shell):

```bash
# head reads 0 lines then closes the pipe — great exits 101
great status --json | head -0

# process-substitution variant — stderr shows broken pipe message
great status --json > >(head -0)
```

**Actual behavior:**

- Exit code: 101 (Rust panic/anyhow error propagation)
- stderr: `Error: write /dev/stdout: broken pipe` (or similar anyhow error)

**Expected behavior:**

- Exit code: 0 (or the process is terminated by SIGPIPE, which shells report
  as exit 141 — both are acceptable to downstream tooling)
- No output to stderr

**Root cause (code):**

`src/cli/status.rs` line 415:

```rust
println!("{}", serde_json::to_string_pretty(&report)?);
```

The `?` propagates the `BrokenPipe` `io::Error` to `main()`, which calls the
anyhow error handler and exits 101. The same pattern exists wherever `println!`
or `print!` is used across all subcommands.

**Why this matters:**

- CI pipelines doing `great status --json | jq '.has_issues'` panic if jq
  exits early (e.g. after finding the first match with `jq -e`).
- `great status --json | head -1` — a common "is the tool healthy?" one-liner
  — reliably panics.
- Any pipeline using `| grep -q`, `| head`, or `| wc -l` may trigger this.
- The panic message on stderr is alarming and confusing; users file bug reports
  thinking the CLI is broken, not that the pipe closed.
- `great status --json` is documented to always exit 0 in JSON mode (by
  design); exiting 101 on EPIPE is an undocumented and inconsistent exception.

## Acceptance Criteria

- [ ] `great status --json | head -0` exits 0 (or 141) and produces no output
  on stderr. Verified by: `great status --json | head -0; echo $?` returning
  0 or 141, with stderr empty.

- [ ] `great status --json | head -1` exits 0 (or 141) with no stderr output,
  confirming the fix works when some output is consumed before the pipe closes.

- [ ] At least two additional subcommands that write to stdout (`great doctor`,
  `great diff`, or `great template list`) are verified to not panic on
  `| head -0` — confirming the fix is applied at the binary entry point rather
  than per-call-site.

- [ ] Normal operation is unaffected: `great status --json` with stdout open
  exits 0 and produces valid JSON; `great status` (human mode) with issues
  still exits 1 as before.

- [ ] `cargo test` passes with no regressions.

## Suggested Fix Approaches

Three options exist; exactly one should be chosen and documented in the commit:

**Option A — `unix_sigpipe` attribute (Rust 1.73+, simplest):**
Add `#[unix_sigpipe = "sig_dfl"]` to `fn main()` in `src/main.rs`. This
restores the default POSIX SIGPIPE disposition so the kernel kills the process
cleanly (exit 141) rather than delivering an `io::Error`.

**Option B — Ignore SIGPIPE at startup, swallow BrokenPipe errors:**
Call `unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) }` early in `main`,
then wrap the top-level `Result` handler to treat `io::ErrorKind::BrokenPipe`
as a clean exit 0.

**Option C — Wrap stdout writes per-call-site:**
Replace `println!` calls with explicit `writeln!(std::io::stdout(), ...)` and
match on `io::ErrorKind::BrokenPipe` to return `Ok(())` instead of
propagating.

Option A is preferred: it is the least invasive, requires no unsafe code or
extra dependencies, and matches the behavior users expect from a POSIX CLI tool.

## Files That Need to Change

- `src/main.rs` — primary fix location (Option A: one-line attribute)
- Potentially `src/cli/status.rs`, `src/cli/doctor.rs`, `src/cli/diff.rs`,
  `src/cli/template.rs` if a per-call-site approach (Option C) is chosen

## Dependencies

None. This is a self-contained fix to the binary entry point (or individual
write sites).

## Out of Scope

- Changing exit codes for non-EPIPE error paths (separate concern).
- Handling SIGPIPE in the MCP bridge server (that process is long-lived and
  EPIPE there has different semantics — track separately if needed).
- Windows behavior (Windows does not have SIGPIPE; the fix is no-op or
  conditional on `#[cfg(unix)]`).
