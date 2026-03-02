# Nightingale Selection — Iteration 026

**Date:** 2026-02-27
**Selected Task:** 0026 — `great diff` Output Channel Redesign (renumbered from backlog/0021)
**Priority:** P2
**Type:** enhancement

---

## Backlog Review Summary

### 0009 — apply command (VERIFIED DONE, move to done/)

`src/cli/apply.rs` is 1,003 lines of complete, working implementation. All five
acceptance criteria from the backlog file are satisfied in source:

- Config loading with auto-discovery and --config override: lines 380–387
- Dry-run mode with full preview and exit 0: lines 394–397, 843–844
- --yes flag wired to skip prompt (args.yes field, used throughout): line 365
- Missing config exits with code 1: lines 381–384 via Context bail
- Idempotency: every install step calls command_exists() before installing
- Integration tests: resolve_secret_refs tests in mod tests (lines 963–1001)

The backlog file `/home/isaac/src/sh.great/.tasks/backlog/0009-apply-command.md`
is stale. It should be moved to `.tasks/done/`.

### 0010 — complete all stubs (VERIFIED DONE per prior iteration memory)

All groups A–J verified done in source per Nightingale memory notes. Backlog file
is stale and should be moved to `.tasks/done/`.

### 0014 — backlog pruning (P3 chore)

Still valid but lowest priority. Stale files confirmed: 0001, 0002, 0003, 0004,
0005, 0007, 0009, 0010 remain in backlog/ despite appearing in done/. This is
housekeeping and should not block real work.

### 0020 — Docker cross-compile UX (P2, NOT READY)

Four items bundled together with acceptance criteria of "each issue resolved or
explicitly deferred." This is too vague to hand to Lovelace for a spec. The
criteria need measurable, testable statements per issue before this task is
selectable. Deferred.

### 0021 backlog — diff output channel redesign (NUMBER COLLISION, renumbered to 0026)

The done/ directory already contains `0021-fix-loop-dir-missing-from-cross-build-context.md`
(a different fix completed in a prior iteration). The backlog file
`0021-diff-output-channel-redesign.md` has the same number, creating a collision.

This task is genuine pending work — confirmed by reading `src/cli/output.rs`:
all output helpers (`success`, `warning`, `error`, `info`, `header`) write to
`eprintln!` (stderr). `great diff` therefore sends section headers to stderr and
diff lines to stdout, making the output incoherent when either channel is
redirected. The task is renumbered to **0026**.

---

## Selected Task: 0026 — `great diff` Output Channel Redesign

**Priority:** P2
**Type:** enhancement
**Module:** `src/cli/diff.rs`, `src/cli/output.rs`
**Estimated Complexity:** S–M (bounded to diff.rs; output.rs changes are additive)
**Dependencies:** None

### Selection Rationale

1. No blocking dependencies.
2. Clear, bounded scope: one command (`diff`), two files.
3. Real user impact: `great diff | grep "+"` silently drops all section headers
   because they go to stderr. Any CI pipeline that captures diff output gets
   incoherent results.
4. Acceptance criteria are already testable (shell redirections).
5. No other higher-priority unblocked work exists in the backlog.

The 0021 backlog file number conflict is a housekeeping hazard (future tasks
could collide again). This selection file uses 0026 and the task file must be
created at `.tasks/backlog/0026-diff-output-channel-redesign.md`.

### Implementation Approach

The narrowest correct fix: add `println!`-based variants of `header`, `info`,
`success` to `output.rs` (e.g., `header_stdout`, `info_stdout`, `success_stdout`)
and use them exclusively in `diff.rs`. This keeps `status`, `doctor`, and
`apply` on stderr (interactive commands benefit from stderr not polluting
piped stdout) while making `diff` fully stdout-routed for pipeline use.

A global output.rs redesign is explicitly out of scope — that would require
touching apply, status, doctor, sync, vault, mcp, template, and loop_cmd
simultaneously, creating a large blast radius for a P2 item.

### Testable Acceptance Criteria

1. `great diff 2>/dev/null` produces coherent, complete output on stdout:
   section headers, diff lines, and summary are all present.
2. `great diff 1>/dev/null` produces no content on stdout; only fatal errors
   (e.g., missing great.toml) appear on stderr.
3. `cargo clippy` passes with zero new warnings after the change.
4. Existing integration tests that assert on diff output pass (update channel
   assertions from stderr to stdout where appropriate).

### Housekeeping Actions Required (before or alongside this iteration)

- Move `/home/isaac/src/sh.great/.tasks/backlog/0009-apply-command.md` to
  `/home/isaac/src/sh.great/.tasks/done/0009-apply-command.md`
- Create new task file at
  `/home/isaac/src/sh.great/.tasks/backlog/0026-diff-output-channel-redesign.md`
  (renumbered from the stale 0021 backlog file)
- Delete or archive
  `/home/isaac/src/sh.great/.tasks/backlog/0021-diff-output-channel-redesign.md`
  to resolve the number collision
