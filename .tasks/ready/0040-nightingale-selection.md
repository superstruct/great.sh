# Nightingale Selection — Task 0040

| Field | Value |
|---|---|
| Task ID | 0040 |
| Title | `great status` exit code inconsistency between human and JSON modes |
| Priority | P3 |
| Type | refactor |
| Status | READY |
| Date selected | 2026-03-04 |

## Requirements Summary

`great status` exits 1 when tools or secrets are missing; `great status --json`
always exits 0. The same underlying environment state produces different exit
codes depending on the output format. The fix must choose a single, documented
exit-code contract and apply it consistently to both modes.

Files in scope: `src/cli/status.rs`, `tests/` (new integration test).
Nothing else changes.

## Decision Recommendation: Option A

**Always exit 0 from `great status`, regardless of issues found.**

Rationale:

1. Matches established convention. `git status` exits 0 even when the working
   tree is dirty. `systemctl status` exits non-zero only when a unit is actively
   failed or not found — not merely degraded. A status command reports state; it
   does not fail because that state is imperfect.

2. Simplest implementation. The existing `std::process::exit(1)` block at
   `status.rs:274–279` is removed. Both code paths then share the same contract
   already held by JSON mode. Net diff is a deletion plus two comment updates.

3. No new surface area. Option C (add `--check`) is also convention-aligned but
   introduces a new flag, new argument parsing, and a separate acceptance
   criterion. That work is out of proportion for a P3 cosmetic inconsistency.
   If `--check` is later needed for CI use-cases it can be its own task.

4. Resolves the immediate scripting hazard. `great status && echo "OK"` will
   stop failing on fresh environments, which is the reported pain.

Option B (exit 1 in both modes) is explicitly rejected: it entrenches the
non-conventional behaviour and breaks any script that relies on the current
JSON-mode contract.

## Acceptance Criteria (Option A)

1. `great status` exits 0 in any environment, including one where tools are
   not yet installed.
2. `great status --json` exits 0 (unchanged — this is already the behaviour).
3. Both modes exit 0 for the identical environment state.
4. Code comment at `status.rs:274` and doc comment at `status.rs:288` are
   updated to document the exit-0 contract explicitly.
5. At least one integration test in `tests/` asserts that `great status` exits
   0 when a critical issue (simulated missing tool) is present.

## Risk Assessment

Low. The change is a one-line deletion plus comment updates. The only risk is
a script somewhere that relies on `great status` exiting 1 as a failure signal;
that pattern contradicts Unix status-command convention and is already broken
relative to `--json` mode, so no regression is introduced.

## Dependencies

None. Standalone refactor.
