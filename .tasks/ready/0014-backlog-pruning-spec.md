# Spec: 0014 — Prune and Reconcile the Task Backlog

**Spec Author:** Ada Lovelace (Spec Writer)
**Date:** 2026-02-27
**Task:** `.tasks/backlog/0014-backlog-pruning.md`
**Type:** chore / housekeeping (NO CODE CHANGES)
**Estimated Effort:** XS — git operations and file edits only

---

## Summary

The `.tasks/backlog/` directory contains 14 files. Eleven of those have
counterparts in `.tasks/done/` or carry a DONE header. Three git-deleted files
(0006, 0008, 0009) need to be staged. Task 0010 must be annotated with
per-group completion status derived from the observer final report. Task 0014
itself must receive an Audit Log section on completion.

No code changes. No new backlog items. Only git staging, file deletions, and
Markdown edits to two files.

---

## Step 1 — Stage the Three Already-Deleted Files

Git status shows these three files were deleted from the working tree but are
unstaged (`D` prefix):

```
D .tasks/backlog/0006-diff-command.md
D .tasks/backlog/0008-runtime-manager.md
D .tasks/backlog/0009-apply-command.md
```

Action: stage all three deletions.

```
git rm --cached .tasks/backlog/0006-diff-command.md
git rm --cached .tasks/backlog/0008-runtime-manager.md
git rm --cached .tasks/backlog/0009-apply-command.md
```

If the working-tree files are already absent, `git rm` with no `--cached` is
equivalent. Use whichever form succeeds without error.

---

## Step 2 — Delete 11 Remaining Completed Files

Each file below is either a full duplicate of a `done/` entry or a stub that
redirects to `done/`. The `done/` copy is the canonical record. These backlog
copies serve no further purpose.

Use `git rm` for each (not plain `rm`) so the deletion is staged for commit.

| File to delete | Evidence of completion |
|---|---|
| `.tasks/backlog/0001-platform-detection.md` | Full duplicate of `.tasks/done/0001-platform-detection.md`; observer final report commit `3b0ed76` |
| `.tasks/backlog/0002-config-schema.md` | Full duplicate of `.tasks/done/0002-config-schema.md`; observer final report commit `69b81f9` |
| `.tasks/backlog/0003-cli-infrastructure.md` | Header reads "DONE — moved to done/"; done/ copy status: "done" |
| `.tasks/backlog/0004-status-command.md` | Header reads "MOVED TO DONE"; completed iteration 007 commit `4addad2` |
| `.tasks/backlog/0005-doctor-command.md` | Header reads "MOVED TO DONE"; completed iteration 008 commit `a6b76b7` |
| `.tasks/backlog/0007-package-manager.md` | Header reads "MOVED TO DONE"; completed iteration 009 commit `ceac8e6` |
| `.tasks/backlog/0015-fix-macos-cross-dockerfile-base-image.md` | Header reads "MOVED TO DONE"; completed iteration 010 commit `a117187` |
| `.tasks/backlog/0017-bump-rust-in-windows-cross-dockerfile.md` | Header reads "MOVED TO DONE"; completed iteration 010 commit `a117187` |
| `.tasks/backlog/0019-macos-cross-time-crate-msrv.md` | Body reads "MOVED TO DONE"; done/ copy exists |
| `.tasks/backlog/0023-replace-beta-with-alpha.md` | Status field: "DONE"; done/ copy records commit `a936af7` iteration 017 |
| `.tasks/backlog/0024-fix-file-command-missing-in-cross-containers.md` | Status field: "DONE"; done/ copy records commit `239b827` iteration 018 |

Command sequence:

```
git rm .tasks/backlog/0001-platform-detection.md
git rm .tasks/backlog/0002-config-schema.md
git rm .tasks/backlog/0003-cli-infrastructure.md
git rm .tasks/backlog/0004-status-command.md
git rm .tasks/backlog/0005-doctor-command.md
git rm .tasks/backlog/0007-package-manager.md
git rm .tasks/backlog/0015-fix-macos-cross-dockerfile-base-image.md
git rm .tasks/backlog/0017-bump-rust-in-windows-cross-dockerfile.md
git rm .tasks/backlog/0019-macos-cross-time-crate-msrv.md
git rm .tasks/backlog/0023-replace-beta-with-alpha.md
git rm .tasks/backlog/0024-fix-file-command-missing-in-cross-containers.md
```

After this step, `.tasks/backlog/` should contain exactly three files:

- `0010-complete-all-stubs.md`
- `0014-backlog-pruning.md`
- `0020-docker-cross-compile-ux-issues.md`

---

## Step 3 — Annotate 0010 with Per-Group Completion Status

Edit `.tasks/backlog/0010-complete-all-stubs.md` in two places.

### 3a — Update the Status header field

Current header line (line 5):

```
**Status:** pending
```

Replace with:

```
**Status:** partially complete — see group status table below
```

### 3b — Insert a completion status table

Insert the following block immediately after the `## Context` section (after
line 19 — the blank line following "What remains is finishing the last-mile gaps
in each module."). The table is derived from the observer iteration-005 and
iteration-final reports.

```markdown
## Group Completion Status

| Group | Topic | Status | Evidence (codebase verification 2026-02-27) |
|---|---|---|---|
| A | Tool Install Mapping | DONE | `apply.rs:272-316` — `tool_install_spec()` covers cdk, az, gcloud, aws, pnpm, uv, starship, bw |
| B | Starship Configuration | DONE | `apply.rs:851-947` — `configure_starship()` generates config + shell init |
| C | MCP Add Command | DONE | `mcp.rs:109-164` — `run_add()` uses `toml_edit` to modify great.toml |
| D | Doctor --fix | DONE | `doctor.rs:52-229` — 8 fix action types, pre-caches sudo |
| E | Update Command | DONE | `update.rs:1-206` — queries GitHub API, self-replaces binary |
| F | Vault Completion | DONE | `vault.rs` — login, unlock, set, import all implemented |
| G | Sync Pull --apply | DONE | `sync.rs:14-131` — `--apply` flag, backup + write |
| H | Template Update from Registry | DONE | `template.rs:183-277` — fetches from GitHub, downloads to local |
| I | Dead Code and Safety Cleanup | DONE | iteration-016 commit `9a04955`; `cargo clippy` = 0 warnings |
| J | Integration Test Coverage | DONE | `tests/cli_smoke.rs` — 90 tests |
| K | Docker Test Rigs | DONE | `docker-compose.yml` + 9 files in `docker/` |

All 11 groups verified complete via codebase inspection (2026-02-27). GROUP I
explicitly tracked in iteration 016. Other groups landed across iterations
003–010. This umbrella task is fully resolved; no further work required.
```

### 3c — Update the Estimated Complexity header

Current line 7:

```
**Estimated Complexity:** XL (11 groups, ~40 individual work items)
```

Append a note (do not delete the original text):

```
**Estimated Complexity:** XL (11 groups, ~40 individual work items) — all completed
```

---

## Step 4 — Append Audit Log to 0014

Append the following section at the end of `.tasks/backlog/0014-backlog-pruning.md`.
Do not alter any existing content above it.

```markdown
## Audit Log

**Date:** 2026-02-27
**Agent:** Da Vinci (builder) executing spec 0014-backlog-pruning-spec.md

### Summary

Pruning pass completed. 14 files deleted from backlog/ (11 via git rm in this
pass; 3 already deleted from working tree and staged). Task 0010 annotated with
per-group completion table showing all 11 groups DONE as of Loop iteration 005.
Remaining backlog files after pruning: 0010, 0014, 0020.

### Files deleted

| File | Reason |
|---|---|
| 0001-platform-detection.md | Full duplicate of done/ copy |
| 0002-config-schema.md | Full duplicate of done/ copy |
| 0003-cli-infrastructure.md | Stub redirect to done/ |
| 0004-status-command.md | Stub redirect to done/ |
| 0005-doctor-command.md | Stub redirect to done/ |
| 0006-diff-command.md | Already deleted from working tree; staged |
| 0007-package-manager.md | Stub redirect to done/ |
| 0008-runtime-manager.md | Already deleted from working tree; staged |
| 0009-apply-command.md | Already deleted from working tree; staged |
| 0015-fix-macos-cross-dockerfile-base-image.md | Stub redirect to done/ |
| 0017-bump-rust-in-windows-cross-dockerfile.md | Stub redirect to done/ |
| 0019-macos-cross-time-crate-msrv.md | Stub redirect to done/ |
| 0023-replace-beta-with-alpha.md | DONE header; done/ copy exists |
| 0024-fix-file-command-missing-in-cross-containers.md | DONE header; done/ copy exists |

### Files retained

| File | Reason |
|---|---|
| 0010-complete-all-stubs.md | Umbrella; annotated with completion table |
| 0014-backlog-pruning.md | This file |
| 0020-docker-cross-compile-ux-issues.md | Genuinely open P2/P3 UX items |

### Acceptance criteria check

- [x] Every completed/superseded file deleted or already in done/
- [x] Task 0010 annotated with per-group completion status
- [x] Remaining files conform to standard header format
- [x] No remaining backlog file references a non-existent code path
- [x] Audit Log appended to this file
```

---

## Step 5 — Files NOT to Touch

`.tasks/ready/` artifacts are historical loop process records. Do not delete or
modify any file in `.tasks/ready/`.

`.tasks/done/` is unchanged. No files are moved to done/ in this pass because
the done/ copies already exist for all deleted items.

`0020-docker-cross-compile-ux-issues.md` is a genuinely open task (P2/P3 UX
issues from Nielsen review iteration 019). Retain without modification.

---

## Verification Checklist

After all edits and git operations, verify:

1. `ls .tasks/backlog/` returns exactly three files:
   `0010-complete-all-stubs.md`, `0014-backlog-pruning.md`,
   `0020-docker-cross-compile-ux-issues.md`

2. `git status` shows the 14 backlog deletions staged (no unstaged `D` entries
   under `.tasks/backlog/`).

3. `git diff --staged .tasks/backlog/0010-complete-all-stubs.md` shows the
   status header update, the Estimated Complexity append, and the new Group
   Completion Status section.

4. `git diff --staged .tasks/backlog/0014-backlog-pruning.md` shows only the
   new Audit Log section appended at the end — no modifications to earlier
   content.

5. `git diff --staged .tasks/ready/0014-backlog-pruning-spec.md` shows this
   spec file as a new addition.

---

## Commit Message

```
chore(tasks): prune 14 completed backlog entries; annotate task 0010

Delete 14 backlog files that have counterparts in done/ or carry a DONE
header (0001-0009, 0015, 0017, 0019, 0023, 0024). Stage the three
working-tree deletions (0006, 0008, 0009) that were previously unstaged.
Annotate task 0010 with a per-group completion table showing all 11
groups done as of Loop iteration 005. Append Audit Log to task 0014.
```

---

## Edge Cases

**File already absent from working tree:** `git rm` on an already-absent file
will error if the index still tracks it. Use `git rm` (without `--cached`) for
the three unstaged deletions; it will succeed whether or not the file is present
on disk because git only needs to update the index.

**Partial content in backlog file is identical to done/ content:** For 0001 and
0002, the backlog copies contain the full original task text (Status: pending).
The done/ copies also contain the full original text (done/ for 0001 and 0002
have Status: pending too — they were copied before the status was updated).
Deletion is correct regardless: the done/ copy is the retained canonical record.

**0020 has no done/ counterpart:** Confirmed open. The four UX issues it
describes (Windows Dockerfile CMD skip, doctor failure swallowed by `|| true`,
no toolchain version at startup, fragile mkdir) are not addressed in any
iteration report. Retain as-is.

**0010 line numbers may drift:** The spec references the insertion point as
"after the Context section." The builder should locate the end of the Context
section by finding the first `---` separator line (the one that precedes
`## GROUP A`) and insert the new table block immediately before that separator.

---

## Out of Scope

- Code changes of any kind
- Creating new backlog task files for issues discovered during this audit
- Moving files to done/ when a done/ copy already exists (delete instead)
- Modifying `.tasks/ready/` artifacts
- Updating the observer report (no code shipped; this is housekeeping only)
