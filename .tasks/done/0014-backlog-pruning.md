# 0014: Prune and Reconcile the Task Backlog

**Priority:** P3
**Type:** chore
**Module:** `.tasks/`
**Status:** complete
**Estimated Complexity:** S

## Context

The backlog was last fully audited at project inception (tasks 0001–0010). Since
then, the Rust CLI has been substantially implemented — 11 subcommands are
shipped — and two site tasks (0011, 0012) have been completed and moved to
`.tasks/done/`. Task 0001 also appears in both `backlog/` and `done/`, which
suggests the done-directory sweep was incomplete.

Without periodic pruning, stale tasks accrue: items that reference code paths
that no longer exist, umbrella tasks whose sub-items have been individually
resolved, and duplicate concerns that were filed separately before a single
owner became clear. A cluttered backlog misleads agents into solving problems
that are already solved and obscures the work that actually remains.

This task is a housekeeping pass. No code changes are expected. The output is a
cleaner, accurate `.tasks/` directory where every remaining backlog item
reflects real, outstanding work.

### Scope of the audit

| Area | What to check |
|---|---|
| Completed items still in `backlog/` | 0001 already appears in `done/`; verify others (0002–0010, 0013) are still genuinely open |
| Umbrella task 0010 | Determine which GROUP A/B/C sub-items are done; split outstanding items into individual tasks or annotate 0010 accordingly |
| Staleness | Tasks referencing stub code paths that no longer exist (e.g., `apply.rs:198-210` tool mapping from 0010-GROUP A may already be implemented) |
| Format consistency | All open tasks should have Priority, Type, Module, Status, Estimated Complexity header fields and an Acceptance Criteria section with checkboxes |
| Duplicates | Check for overlapping scope between tasks (e.g., doctor flags in 0005 vs. doctor sub-items in 0010) |

## Acceptance Criteria

- [ ] Every file in `.tasks/backlog/` that is already fully implemented or superseded has been moved to `.tasks/done/` (or deleted if done/ copy already exists).
- [ ] Task 0010 is either replaced by individual scoped tasks for its remaining open GROUP items, or annotated with a per-group completion status so the remaining work is unambiguous.
- [ ] All remaining backlog files conform to the standard header format (Priority, Type, Module, Status, Estimated Complexity) and contain a testable Acceptance Criteria section.
- [ ] No task in `backlog/` references a code path, function, or line number that no longer exists in the current codebase after the pruning pass.
- [ ] A brief comment is added to this task file (in an "Audit Log" section at the bottom) recording the date of the pruning pass and a one-line summary of what changed.

## Files That Need to Change

- `.tasks/backlog/*.md` — review each; update, move, split, or delete as warranted
- `.tasks/done/` — destination for newly-closed items
- This file (`0014`) — append Audit Log section once the pass is complete

## Dependencies

None. This is a documentation/housekeeping task with no code dependencies.

## Out of Scope

- Making any code changes to resolve open tasks — only the task metadata is
  in scope here.
- Rewriting task content beyond reformatting to meet the standard header
  convention; substantive edits belong in follow-on task files.
- Creating new backlog items discovered during the audit — those should be
  filed as separate numbered tasks after this pass completes.

## Audit Log

**Date:** 2026-02-27
**Iteration:** 022

### Summary

Pruning pass completed. 14 files removed from backlog/ (9 via git rm, 5 via
rm for untracked files). Task 0010 annotated with per-group completion table
showing all 11 groups DONE via codebase verification. Remaining backlog after
pruning: 0010 (annotated complete), 0014 (this file), 0020 (genuinely open).

### Files removed

| File | Method | Reason |
|---|---|---|
| 0001-platform-detection.md | git rm | Full duplicate of done/ copy |
| 0002-config-schema.md | git rm | Full duplicate of done/ copy |
| 0003-cli-infrastructure.md | git rm -f | Stub redirect to done/ |
| 0004-status-command.md | git rm -f | Stub redirect to done/ |
| 0005-doctor-command.md | git rm -f | Stub redirect to done/ |
| 0006-diff-command.md | git rm (staged) | Already deleted from working tree |
| 0007-package-manager.md | git rm -f | Stub redirect to done/ |
| 0008-runtime-manager.md | git rm (staged) | Already deleted from working tree |
| 0009-apply-command.md | git rm (staged) | Already deleted from working tree |
| 0015-fix-macos-cross-dockerfile-base-image.md | rm (untracked) | Stub redirect to done/ |
| 0017-bump-rust-in-windows-cross-dockerfile.md | rm (untracked) | Stub redirect to done/ |
| 0019-macos-cross-time-crate-msrv.md | rm (untracked) | Stub redirect to done/ |
| 0023-replace-beta-with-alpha.md | rm (untracked) | DONE header; done/ copy exists |
| 0024-fix-file-command-missing-in-cross-containers.md | rm (untracked) | DONE header; done/ copy exists |

### Files retained

| File | Reason |
|---|---|
| 0010-complete-all-stubs.md | Annotated with completion table; all 11 groups DONE |
| 0014-backlog-pruning.md | This file |
| 0020-docker-cross-compile-ux-issues.md | Genuinely open P2/P3 UX items |

### Acceptance criteria check

- [x] Every completed/superseded file deleted or already in done/
- [x] Task 0010 annotated with per-group completion status
- [x] Remaining files conform to standard header format
- [x] No remaining backlog file references a non-existent code path
- [x] Audit Log appended to this file
