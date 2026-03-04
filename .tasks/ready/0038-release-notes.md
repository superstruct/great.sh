# Release Notes: Task 0038 — Restore Default SIGPIPE Handling

**Date:** 2026-03-04
**Scope:** `src/main.rs`

---

## What Changed

`great` commands previously exited with code 101 and printed a broken-pipe
error to stderr when the read end of a pipe closed before output was fully
consumed. Any pipeline using `| head`, `| grep -q`, `| jq -e`, or similar
tools could trigger this.

`src/main.rs` now calls `libc::signal(SIGPIPE, SIG_DFL)` at startup (Unix
only, via `#[cfg(unix)]`). This restores the POSIX default: when the pipe
breaks, the kernel terminates the process cleanly. The shell reports the exit
status as 141 (128 + SIGPIPE signal number), which downstream tooling treats
as a normal pipe termination.

**Before:**

```sh
$ great status --json | head -0; echo "exit: $?"
Error: write /dev/stdout: broken pipe
exit: 101
```

**After:**

```sh
$ great status --json | head -0; echo "exit: $?"
exit: 141
```

No output to stderr. No spurious error message.

---

## Why

CI pipelines running `great status --json | jq '.has_issues'` or
`great status --json | head -1` panicked when `jq` or `head` exited early.
The exit-101 panic message was alarming and misleading — users filed bug
reports believing the CLI was broken.

---

## Migration Notes

No changes to `great.toml`, CLI flags, or any subcommand's documented exit
codes. Normal operation is unaffected on all platforms (Windows has no SIGPIPE;
the `#[cfg(unix)]` guard makes the call a no-op there).

The only observable change is on Unix: broken-pipe exits now produce exit 141
instead of exit 101, and no error message on stderr.
