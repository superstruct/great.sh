# Scout Report: 0014 — Prune and Reconcile the Task Backlog

**Scout:** Alexander von Humboldt
**Date:** 2026-02-27
**Task:** `.tasks/backlog/0014-backlog-pruning.md`
**Iteration:** 027

---

## 1. Exact File Inventory — `.tasks/backlog/`

14 files currently on disk (confirmed via `ls`):

| # | File | Content Type | Git State |
|---|---|---|---|
| 1 | `0001-platform-detection.md` | Full original task body | tracked, unmodified |
| 2 | `0002-config-schema.md` | Full original task body | tracked, unmodified |
| 3 | `0003-cli-infrastructure.md` | 3-line "DONE — moved to done/" stub | tracked, modified (M) |
| 4 | `0004-status-command.md` | 5-line "MOVED TO DONE" stub | tracked, modified (M) |
| 5 | `0005-doctor-command.md` | 4-line "MOVED TO DONE" stub | tracked, modified (M) |
| 6 | `0006-diff-command.md` | ABSENT from working tree | tracked, deleted (D) unstaged |
| 7 | `0007-package-manager.md` | 4-line "MOVED TO DONE" stub | tracked, modified (M) |
| 8 | `0008-runtime-manager.md` | ABSENT from working tree | tracked, deleted (D) unstaged |
| 9 | `0009-apply-command.md` | ABSENT from working tree | tracked, deleted (D) unstaged |
| 10 | `0010-complete-all-stubs.md` | Full umbrella task, 390 lines | tracked, unmodified |
| 11 | `0014-backlog-pruning.md` | Full task body, 63 lines | tracked, unmodified |
| 12 | `0015-fix-macos-cross-dockerfile-base-image.md` | 4-line "MOVED TO DONE" stub | tracked, unmodified |
| 13 | `0017-bump-rust-in-windows-cross-dockerfile.md` | 4-line "MOVED TO DONE" stub | tracked, unmodified |
| 14 | `0019-macos-cross-time-crate-msrv.md` | 1-line "MOVED TO DONE" stub | tracked, unmodified |
| 15 | `0020-docker-cross-compile-ux-issues.md` | Full open task, 25 lines | tracked, unmodified |
| 16 | `0023-replace-beta-with-alpha.md` | Full body, Status: DONE header | tracked, unmodified |
| 17 | `0024-fix-file-command-missing-in-cross-containers.md` | Full body, Status: DONE header | tracked, unmodified |

NOTE: `ls` shows 14 files because 0006, 0008, and 0009 are absent from disk.
The index still tracks them — git status reports them as unstaged deletions.
Total files tracked by git: 17. Files on disk: 14.

---

## 2. Exact File Inventory — `.tasks/done/`

26 files confirmed on disk:

```
0001-platform-detection.md
0002-config-schema.md
0003-cli-infrastructure.md
0004-status-command.md
0005-doctor-command.md
0006-diff-command.md
0007-package-manager.md
0008-runtime-manager.md
0009-apply-command.md
0011-loop-site-accuracy.md
0011-statusline-multi-agent.md
0011-update-site-great-loop.md
0012-loop-section-visual-polish.md
0012-restore-architecton-fix-templates.md
0013-statusline-settings-schema-mismatch.md
0015-fix-macos-cross-dockerfile-base-image.md
0016-loop-install-overwrite-safety.md
0017-bump-rust-in-windows-cross-dockerfile.md
0018-bump-rust-in-aarch64-cross-dockerfile.md
0019-macos-cross-time-crate-msrv.md
0021-fix-loop-dir-missing-from-cross-build-context.md
0022-diff-counter-marker-consistency.md
0023-replace-beta-with-alpha.md
0024-fix-file-command-missing-in-cross-containers.md
0025-homebrew-non-admin-failure-handling.md
0026-diff-output-channel-redesign.md
```

---

## 3. Cross-Reference: Backlog vs. Done

### Category A — Delete from backlog/ (done/ counterpart confirmed)

All 11 `done/` counterparts verified on disk via filesystem check.

| Backlog File | Done/ Counterpart | Confirmed |
|---|---|---|
| `0001-platform-detection.md` | `done/0001-platform-detection.md` | YES |
| `0002-config-schema.md` | `done/0002-config-schema.md` | YES |
| `0003-cli-infrastructure.md` | `done/0003-cli-infrastructure.md` | YES |
| `0004-status-command.md` | `done/0004-status-command.md` | YES |
| `0005-doctor-command.md` | `done/0005-doctor-command.md` | YES |
| `0007-package-manager.md` | `done/0007-package-manager.md` | YES |
| `0015-fix-macos-cross-dockerfile-base-image.md` | `done/0015-fix-macos-cross-dockerfile-base-image.md` | YES |
| `0017-bump-rust-in-windows-cross-dockerfile.md` | `done/0017-bump-rust-in-windows-cross-dockerfile.md` | YES |
| `0019-macos-cross-time-crate-msrv.md` | `done/0019-macos-cross-time-crate-msrv.md` | YES |
| `0023-replace-beta-with-alpha.md` | `done/0023-replace-beta-with-alpha.md` | YES |
| `0024-fix-file-command-missing-in-cross-containers.md` | `done/0024-fix-file-command-missing-in-cross-containers.md` | YES |

### Category B — Already deleted from working tree (done/ counterpart confirmed)

| Backlog File | Done/ Counterpart | Git State |
|---|---|---|
| `0006-diff-command.md` | `done/0006-diff-command.md` | D (unstaged deletion) |
| `0008-runtime-manager.md` | `done/0008-runtime-manager.md` | D (unstaged deletion) |
| `0009-apply-command.md` | `done/0009-apply-command.md` | D (unstaged deletion) |

These three need only `git rm` to stage the index deletion. Files are already gone from disk.

### Category C — Genuinely open, retain in backlog/

| Backlog File | Done/ Counterpart | Status |
|---|---|---|
| `0010-complete-all-stubs.md` | None | Open; needs per-group annotation |
| `0014-backlog-pruning.md` | None | In progress; needs Audit Log appended |
| `0020-docker-cross-compile-ux-issues.md` | None | Open P2/P3; 4 UX issues unresolved |

### Category D — In done/ but NOT in backlog/ (no action needed)

These done/ entries have no corresponding backlog file (were filed and closed without leaving backlog stubs):

```
0011-loop-site-accuracy.md, 0011-statusline-multi-agent.md,
0011-update-site-great-loop.md, 0012-loop-section-visual-polish.md,
0012-restore-architecton-fix-templates.md,
0013-statusline-settings-schema-mismatch.md,
0016-loop-install-overwrite-safety.md,
0018-bump-rust-in-aarch64-cross-dockerfile.md,
0021-fix-loop-dir-missing-from-cross-build-context.md,
0022-diff-counter-marker-consistency.md,
0025-homebrew-non-admin-failure-handling.md,
0026-diff-output-channel-redesign.md
```

No action required for these.

---

## 4. Git Status — `.tasks/backlog/`

From `git status .tasks/backlog/` (unstaged, not staged):

```
modified:   .tasks/backlog/0003-cli-infrastructure.md
modified:   .tasks/backlog/0004-status-command.md
modified:   .tasks/backlog/0005-doctor-command.md
deleted:    .tasks/backlog/0006-diff-command.md
modified:   .tasks/backlog/0007-package-manager.md
deleted:    .tasks/backlog/0008-runtime-manager.md
deleted:    .tasks/backlog/0009-apply-command.md
```

The "modified" files contain stub redirects — they were rewritten from full task bodies to
short "MOVED TO DONE" stubs at some point in a prior iteration but never staged/committed.
This means git still tracks them as modified relative to HEAD. `git rm` will handle them
correctly (removes from index and disk; disk content is already a stub so no information lost).

The three deleted files (0006, 0008, 0009) are absent on disk. Their index entries must be
removed with `git rm`. Running `git rm <path>` on an absent file updates the index only.

---

## 5. `test-files/` Directory

Untracked per git status. Contents (confirmed via `ls`):

```
great-aarch64-apple-darwin
great-aarch64-unknown-linux-gnu
great-x86_64-apple-darwin
```

These are compiled binary artifacts for three targets. They are entirely unrelated to
the `.tasks/` housekeeping work. No action required for this task.

Technical debt note: these binaries should be in `.gitignore`. They are currently
untracked and will remain so unless explicitly added. The `test-files/` path appears
to be referenced in `0020-docker-cross-compile-ux-issues.md` issue #4 (fragile mkdir
against read-only workspace mount). This connection is informational only — issue #4
is out of scope for task 0014.

---

## 6. Dependency Map

This task has no code dependencies. All operations are:

1. `git rm` — 14 files across two mechanisms (11 present on disk, 3 already absent)
2. Edit — `0010-complete-all-stubs.md` (status header + group table insert)
3. Edit — `0014-backlog-pruning.md` (Audit Log append)

No Rust source files, site files, or configuration files are touched.

No compilation or testing required.

---

## 7. Risks

**Risk 1 — `git rm` on modified files:** Files 0003, 0004, 0005, 0007 are marked
`modified` in git status (stub content diverges from HEAD). Running `git rm` will
fail with "error: ... has local modifications" unless `--force` is used. Use `git rm -f`
for these four files or run `git checkout -- <file>` first, then `git rm`.
Alternatively: `git rm --force` is safe here because the done/ canonical copy is retained.

**Risk 2 — 0010 insertion point:** The spec instructs inserting the group table "before the
first `---` separator line that precedes `## GROUP A`." That separator is at line 22 of
`0010-complete-all-stubs.md` (confirmed by reading the file). The builder must not insert
at the wrong `---` marker. There is only one `---` at line 22; all other section breaks
are rendered via `---` separators between groups at lines 58, 86, 112, etc. Insert at line 22.

**Risk 3 — 0010 status update ambiguity:** The spec says change `**Status:** pending` to
`**Status:** partially complete — see group status table below`. The Nightingale selection
(and the Lovelace spec) diverge slightly here: Nightingale says "partially complete — GROUP K
unverified" while Lovelace says all 11 groups including K are DONE (citing docker-compose.yml
153 lines + 9 files in docker/). The Lovelace spec (most recent artifact) supersedes; use
"all groups complete — see group status table below" or match the exact text in the spec.
Da Vinci should follow the Lovelace spec verbatim.

---

## 8. Recommended Build Order

```
Step 1: Stage the 3 unstaged deletions
  git rm .tasks/backlog/0006-diff-command.md
  git rm .tasks/backlog/0008-runtime-manager.md
  git rm .tasks/backlog/0009-apply-command.md

Step 2: Remove 4 modified stub files (require --force)
  git rm -f .tasks/backlog/0003-cli-infrastructure.md
  git rm -f .tasks/backlog/0004-status-command.md
  git rm -f .tasks/backlog/0005-doctor-command.md
  git rm -f .tasks/backlog/0007-package-manager.md

Step 3: Remove 7 unmodified stub/duplicate files
  git rm .tasks/backlog/0001-platform-detection.md
  git rm .tasks/backlog/0002-config-schema.md
  git rm .tasks/backlog/0015-fix-macos-cross-dockerfile-base-image.md
  git rm .tasks/backlog/0017-bump-rust-in-windows-cross-dockerfile.md
  git rm .tasks/backlog/0019-macos-cross-time-crate-msrv.md
  git rm .tasks/backlog/0023-replace-beta-with-alpha.md
  git rm .tasks/backlog/0024-fix-file-command-missing-in-cross-containers.md

Step 4: Edit .tasks/backlog/0010-complete-all-stubs.md
  a. Line 6: change "**Status:** pending" per spec
  b. Line 7: append "— all completed" to Estimated Complexity
  c. Insert Group Completion Status table block before line 22 (the "---" before GROUP A)

Step 5: Edit .tasks/backlog/0014-backlog-pruning.md
  a. Append Audit Log section at end of file

Step 6: Verify
  ls .tasks/backlog/ → must show exactly 3 files
  git status .tasks/backlog/ → no unstaged entries
```

---

## 9. Post-Prune Expected State

`.tasks/backlog/` will contain exactly:

```
0010-complete-all-stubs.md   (annotated umbrella; all 11 groups DONE)
0014-backlog-pruning.md      (this task; Audit Log appended)
0020-docker-cross-compile-ux-issues.md  (open P2/P3; 4 UX issues)
```

Next available task number: **0027** (no new tasks created in this pass).
