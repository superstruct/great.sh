# Socrates Review — 0014 Backlog Pruning Spec

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-27
**Spec:** `.tasks/ready/0014-backlog-pruning-spec.md`
**Backlog task:** `.tasks/backlog/0014-backlog-pruning.md`

---

## VERDICT: APPROVED

This is a housekeeping spec. All file targets are correct, all retained files are
genuinely open, and the 0010 annotation is substantively accurate. Three
ADVISORY concerns are noted below; none block execution.

---

## Concerns

```json
{
  "gap": "GROUP I evidence references a non-existent observer report",
  "question": "The GROUP I table row cites 'iteration-016 commit 9a04955' but .observer/reports/ contains only iteration-001 through iteration-005 and iteration-final. No iteration-016.md exists. Should the evidence read iteration-005 (the one that actually records 'Dead code warnings: 11 → 0, Clippy warnings: 5 → 0') plus the codebase-inspection note, or should a commit hash be verified before inclusion?",
  "severity": "ADVISORY",
  "recommendation": "Replace 'iteration-016 commit 9a04955' with 'iteration-005 commits 7721706, e285e5f; cargo clippy = 0 warnings confirmed by codebase inspection 2026-02-27'. The iteration-005 report is the canonical record for GROUP I completion. If commit 9a04955 is real (from a later loop not yet in .observer/reports/), add the corresponding iteration report first."
}
```

```json
{
  "gap": "Three line-count claims in the GROUP J, K, and F evidence rows do not match current codebase state",
  "question": "GROUP J says '1871 lines' but tests/cli_smoke.rs is currently 1654 lines. GROUP K says 'docker-compose.yml (153 lines)' but the file is 140 lines. GROUP F says 'vault.rs:1-348' but vault.rs is 297 lines. Are these line counts from an earlier snapshot, or were they fabricated? Since line counts drift, does the spec need exact counts, or would 'see file' suffice?",
  "severity": "ADVISORY",
  "recommendation": "Correct the three values: tests/cli_smoke.rs (1654 lines, 90 tests), docker-compose.yml (140 lines), vault.rs:1-297. All three groups are demonstrably DONE regardless — the inaccurate counts affect only the annotation quality, not correctness. Alternatively, drop the line counts and keep only function names and verified line ranges as evidence, which age better."
}
```

```json
{
  "gap": "GROUP D annotation says '7 fix action types' but doctor.rs defines 8 FixAction variants",
  "question": "doctor.rs:32-49 defines InstallTool, InstallHomebrew, CreateClaudeDir, AddLocalBinToPath, InstallSystemPrerequisite, InstallDocker, InstallClaudeCode, and FixInotifyWatches — that is 8 variants, not 7. Should the annotation be corrected to '8 fix action types'?",
  "severity": "ADVISORY",
  "recommendation": "Update GROUP D evidence from '7 fix action types' to '8 fix action types' before the annotation is committed. This is a minor factual error in a documentation annotation; it does not affect the correctness of the DONE verdict."
}
```

---

## What Was Verified

The following claims were checked against the live codebase and are correct:

**File inventory (Steps 1 and 2):**
- `.tasks/backlog/` currently has 14 files; `.tasks/done/` counterparts exist for all
  14 targets (including the 3 git-deleted ones at 0006, 0008, 0009 whose done/ copies
  are present at `.tasks/done/0006-diff-command.md`, `0008-runtime-manager.md`,
  `0009-apply-command.md`).
- Stubs verified: 0003 ("DONE — moved to done/"), 0004 ("MOVED TO DONE"), 0019
  ("MOVED TO DONE"), 0023 (Status: DONE), 0024 (Status: DONE).
- The 0001 backlog file is full-content (not a stub), but deletion is correct because
  the done/ canonical copy exists.
- After deletion, exactly 3 files remain: 0010, 0014, 0020. Count confirmed.

**0020 retention:**
- The file carries `Status: backlog` (no DONE marker), all 4 UX issues have open
  checkboxes, no done/ counterpart exists. Correctly retained.

**GROUP A evidence — apply.rs:272-316:**
- `tool_install_spec()` starts at line 272, closing brace at line 316. Accurate.
- All 8 tools (cdk, az, gcloud, aws, pnpm, uv, starship, bw/bitwarden-cli) are
  present in the match arms.

**GROUP B evidence — apply.rs:851-947:**
- `configure_starship()` starts at line 852 (doc comment line 851), closing brace
  at line 947. Accurate.

**GROUP C evidence — mcp.rs:109-164:**
- `run_add()` doc comment at 109, function start at 110, closing brace at 164.
  Uses `toml_edit` as claimed. Accurate.

**GROUP D evidence — doctor.rs:52-229:**
- `pub fn run()` at line 52, fix-loop summary block ends at line 229. Accurate.
- Sudo pre-caching confirmed at lines 99-114. Accurate.

**GROUP E evidence — update.rs:1-206:**
- File is 202 lines (upper bound 206 is off by 4 — acceptable approximation for
  an annotation). Accurate in substance.

**GROUP G evidence — sync.rs:14-131:**
- `--apply` flag defined at line 22, `run_pull(apply)` ends at line 131.
  sync.rs is 110 lines but the function ends at 131. Accurate.

**GROUP H evidence — template.rs:183-277:**
- `run_update()` at line 183, `fetch_templates_from_github()` at 192,
  function closes at line 277. Accurate.

**GROUP I (substantive completeness):**
- `cargo clippy` = 0 warnings confirmed by iteration-005 report metrics table.
  The iteration number cited (016) is wrong; the substantive claim (0 warnings)
  is correct.

**GROUP J evidence — 90 tests:**
- `#[test]` count in tests/cli_smoke.rs: 90. Accurate.
  Line count (1654 vs. claimed 1871) is inaccurate — see ADVISORY above.

**GROUP K evidence — 9 files in docker/:**
- docker/ contains 5 Dockerfiles + 4 shell scripts = 9 files. Accurate.
  docker-compose.yml line count (140 vs. claimed 153) is inaccurate — see
  ADVISORY above.

**Step 3 insertion point:**
- 0010 line 19 is the blank line ending the Context section; line 21 is the first
  `---` separator. Step 3b text and the Edge Case note are consistent: insert
  between line 19 and line 21.

**Status header update (Step 3a):**
- Current line 5 reads `**Status:** pending`. Replacement is correct.

**Estimated Complexity append (Step 3c):**
- Current line 7 reads `**Estimated Complexity:** XL (11 groups, ~40 individual
  work items)`. The append-not-delete instruction is correct — the original text
  is preserved.

**Audit Log format (Step 4):**
- The backlog task's Acceptance Criteria (line 43) requires "a brief comment ...
  in an 'Audit Log' section at the bottom recording the date of the pruning pass
  and a one-line summary of what changed." The spec's Audit Log provides date,
  agent, summary paragraph, and two tables. Exceeds the minimum requirement.

**Summary text inconsistency (Step 4):**
- The Audit Log summary says "all 11 groups DONE as of Loop iteration 005."
  This is consistent with the observer reports that exist. The GROUP I row in the
  table says "iteration-016" — minor internal inconsistency, covered by the
  first ADVISORY above.

**Step 5 (files not to touch):**
- `.tasks/ready/` preservation is correctly scoped. No done/ moves are needed
  (all done/ copies pre-exist). 0020 retention is justified.

**Verification checklist (Step 6):**
- All 5 verification commands are meaningful and testable. Item 5 (`git diff
  --staged .tasks/ready/0014-backlog-pruning-spec.md`) correctly assumes the
  spec file itself will be a new addition in the same commit.

**Commit message:**
- Enumerates all 14 deletions by ID range. Count is correct (9 from 0001-0009
  + 5 from 0015/0017/0019/0023/0024 = 14). Accurate.

---

## Summary

The spec correctly identifies all 14 files to delete, correctly retains the three
open files (0010, 0014, 0020), and produces an annotation table whose DONE verdicts
are confirmed by the live codebase; three minor factual errors in the evidence
details (a non-existent iteration reference, two wrong line counts, and an off-by-one
fix-action count) are ADVISORY and should be corrected but do not block execution.
