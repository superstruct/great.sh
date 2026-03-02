# 0014 — Nightingale Selection: Backlog Pruning

**Selected task:** 0014-backlog-pruning
**Priority:** P3
**Type:** chore
**Iteration:** 027
**Date:** 2026-02-27
**Agent:** Florence Nightingale (Requirements Curator)

---

## Task Summary

This is a no-code housekeeping pass. No Rust source files, site files, or
configuration files change. The only output is a cleaner `.tasks/` directory
where every remaining backlog item reflects real, outstanding work.

---

## Current Backlog Inventory (pre-prune)

14 files in `.tasks/backlog/` at the time of selection. Status assessed against
`.tasks/done/` contents and MEMORY.md records of source verification.

### Category 1: Already in done/ — delete from backlog/

These files exist in `.tasks/done/` and their backlog copies serve no purpose.
Some are stubs (3 lines pointing to done). Some are full task bodies that were
never removed from backlog/ when done/ was populated.

| File | Backlog content | Done/ entry | Action |
|---|---|---|---|
| 0001-platform-detection.md | Full original task body | done/0001 exists | Delete from backlog/ |
| 0002-config-schema.md | Full original task body | done/0002 exists | Delete from backlog/ |
| 0003-cli-infrastructure.md | 3-line stub "moved to done" | done/0003 exists | Delete from backlog/ |
| 0004-status-command.md | Stub pointing to done | done/0004 exists | Delete from backlog/ |
| 0005-doctor-command.md | Stub pointing to done | done/0005 exists | Delete from backlog/ |
| 0007-package-manager.md | Stub pointing to done | done/0007 exists | Delete from backlog/ |
| 0015-fix-macos-cross-dockerfile-base-image.md | Stub pointing to done | done/0015 exists | Delete from backlog/ |
| 0017-bump-rust-in-windows-cross-dockerfile.md | Stub pointing to done | done/0017 exists | Delete from backlog/ |
| 0019-macos-cross-time-crate-msrv.md | One-liner pointing to done | done/0019 exists | Delete from backlog/ |
| 0023-replace-beta-with-alpha.md | Full body, Status: DONE in header | done/0023 exists | Delete from backlog/ |
| 0024-fix-file-command-missing-in-cross-containers.md | Full body, Status: DONE in header | done/0024 exists | Delete from backlog/ |

Note: git status already shows 0006, 0008, 0009 as deleted (D) — those three
were removed in a prior working-tree change and are not in the backlog listing.
They do not need further action.

### Category 2: Genuinely open — retain in backlog/

| File | Status | Notes |
|---|---|---|
| 0010-complete-all-stubs.md | Partially complete — see annotation below | Umbrella, 11 groups |
| 0014-backlog-pruning.md | In progress (this iteration) | Self-referential |
| 0020-docker-cross-compile-ux-issues.md | Open, P2 | 4 UX issues from Nielsen review |

---

## Special Case: 0010 Umbrella Task

0010-complete-all-stubs.md tracks 11 groups (A through K). Source code has been
read and verified across multiple prior iterations. The current completion status
per group is:

| Group | Description | Status | Evidence |
|---|---|---|---|
| A | Tool Install Mapping Table | DONE | apply.rs tool_install_spec verified in source |
| B | Starship Configuration | DONE | configure_starship verified in apply.rs |
| C | MCP Add Command | DONE | mcp.rs uses toml_edit for format-preserving writes |
| D | Doctor --fix | DONE | doctor.rs has FixAction enum; --fix path implemented |
| E | Update Command | DONE | update.rs 241 lines; GitHub API + binary swap |
| F | Vault Completion | DONE | vault.rs all 4 commands implemented |
| G | Sync Pull --apply | DONE | sync.rs pull --apply with backup implemented |
| H | Template Update from Registry | DONE | template.rs update fetches from GitHub API |
| I | Dead Code and Safety Cleanup | DONE | Iteration 016; clippy clean confirmed |
| J | Integration Test Coverage | DONE | 80+ tests in cli_smoke.rs verified |
| K | Docker Test Rigs | UNVERIFIED | Dockerfiles exist; test.sh rigging not confirmed |

Action for 0010: annotate the file with a per-group completion table (the table
above) and move it to `.tasks/done/0010-complete-all-stubs.md` once GROUP K is
confirmed. If GROUP K remains unverified, update the Status header to reflect
that K is the only outstanding item.

The immediate pruning pass annotation should mark Groups A through J as DONE
and flag K as "unverified — Dockerfiles present, test rig unknown."

---

## Pruning Actions (Ordered)

1. Delete from backlog/: 0001, 0002, 0003, 0004, 0005, 0007, 0015, 0017, 0019,
   0023, 0024. (11 files)

2. Annotate 0010: Add a per-group status table in an "Audit Log" section at the
   bottom of the file. Groups A-J marked DONE with iteration citations. Group K
   marked "unverified". Update Status header from "pending" to "partially complete
   — GROUP K unverified".

3. Update 0014: Append an "Audit Log" section recording the date of this pruning
   pass and a one-line summary of what changed. (Required by 0014 acceptance
   criteria.)

4. Retain 0020: File is open with 4 unresolved issues. No changes needed to its
   content during this pass.

---

## Post-Prune Backlog State

After the pruning actions above, `.tasks/backlog/` will contain exactly 3 files:

| File | Priority | Status |
|---|---|---|
| 0010-complete-all-stubs.md | P0 umbrella | Partially complete — GROUP K unverified |
| 0014-backlog-pruning.md | P3 | In progress (complete when Audit Log appended) |
| 0020-docker-cross-compile-ux-issues.md | P2 | Open — 4 UX issues |

Next available task number: 0027 (unchanged — no new tasks created in this pass).

---

## Acceptance Criteria Verification Map

Criteria from 0014-backlog-pruning.md, matched to actions above:

- [x] Every file already fully implemented or superseded removed from backlog/
  -- Action 1 above covers 11 files; git-deleted 0006/0008/0009 already gone.
- [ ] Task 0010 annotated with per-group completion status
  -- Action 2 above; to be executed by the Da Vinci agent.
- [x] All remaining backlog files conform to standard header format
  -- 0020 already has standard format. 0014 has it. 0010 has it (will gain Audit Log).
- [x] No task in backlog/ references a code path that no longer exists
  -- After pruning, only 0010 (with group annotation) and 0020 remain; both reference current code.
- [ ] Audit Log section appended to 0014
  -- Action 3 above; to be executed after the pass completes.

---

## Notes for Executing Agent (Da Vinci)

This is purely a filesystem + text task. The steps are:

1. In `.tasks/backlog/`, delete the 11 files listed in Category 1 above using
   standard file removal. Do not copy them to done/ — done/ copies already exist.

2. Open `.tasks/backlog/0010-complete-all-stubs.md` and:
   a. Change "Status: pending" header to "Status: partially complete — GROUP K unverified"
   b. Append an "## Audit Log" section at the bottom with the group status table
      from this document.

3. Open `.tasks/backlog/0014-backlog-pruning.md` and append an "## Audit Log"
   section recording: date 2026-02-27, 11 stale files removed, 0010 annotated,
   GROUP K flagged for follow-up verification.

4. Run no tests. There is no code to compile. The output is a clean `.tasks/`
   directory.

No new task files need to be created. No done/ writes are needed. No source files
are touched.
