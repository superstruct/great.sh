# 0040 -- Socrates Review: `great status` exit code consistency

| Field | Value |
|---|---|
| Spec | `.tasks/ready/0040-status-exit-code-spec.md` |
| Task | `.tasks/backlog/0040-status-exit-code-inconsistency.md` |
| Reviewer | Socrates |
| Date | 2026-03-04 |
| Round | 1 of 3 |

## VERDICT: APPROVED

No BLOCKING concerns. The spec is precise, the line references are accurate, the
test plan covers the behavioral change, and the edge case table is thorough. This
is a well-scoped refactor with a clear rationale.

## Verification performed

1. **Line number accuracy.** All spec references verified against current source:
   - `src/cli/status.rs` lines 129, 182, 200, 264, 274-279, 288 -- all match.
   - `tests/cli_smoke.rs` lines 1937, 1960, 1984 -- all match.

2. **Acceptance criteria coverage.** The task (`0040-status-exit-code-inconsistency.md`)
   lists five acceptance criteria. The spec satisfies all five for Option A:
   - AC1 (single exit-code contract): yes, both modes exit 0.
   - AC2 (exit 0 in fresh environment): yes, the `process::exit(1)` is removed.
   - AC3 (Option C `--check` flag): N/A, Option A chosen; spec explicitly scopes this out.
   - AC4 (comments updated): yes, both the code comment and `run_json` doc comment are updated.
   - AC5 (integration test for missing tool): yes, existing test is updated plus a new combined test added.

3. **Other `process::exit` call sites.** Grepped for `process::exit` across `src/`.
   Found calls in `diff.rs:41` and `doctor.rs:263,267`. The spec correctly limits
   its scope to `status.rs` only -- the other sites are separate commands with their
   own contracts.

4. **Clippy correctness.** The spec correctly identifies that removing the `if
   has_critical_issues` block without removing the variable and its three assignment
   sites would trigger `unused_variable`. The recommended approach (remove all four
   sites) is sound.

## Concerns

1. {
     "gap": "The spec says removing has_critical_issues and its assignments avoids dead-code warnings, but frames this as optional ('the builder MAY remove it'). The recommended approach section then says to do it. This is slightly contradictory.",
     "question": "Should the spec pick one approach and mandate it, rather than offering two paths?",
     "severity": "ADVISORY",
     "recommendation": "Strike the 'MAY' language and keep only the recommended approach. Two valid implementation paths create unnecessary ambiguity for the builder. The 'leave has_critical_issues in' path produces a clippy warning, making it not truly acceptable."
   }

2. {
     "gap": "The new combined test `status_human_and_json_exit_codes_match` tests exit codes but does not assert on output content. It duplicates exit-code assertions already covered by the two updated tests.",
     "question": "What does the combined test prove that the individual updated tests do not already cover?",
     "severity": "ADVISORY",
     "recommendation": "The combined test is fine as a belt-and-suspenders check and documents the 'both modes agree' invariant. No change needed, but the builder should understand it is a redundancy test, not a coverage test."
   }

3. {
     "gap": "The spec does not mention whether the `use` statement for `std::process` needs to be removed. Currently it is called inline (`std::process::exit(1)`) so there is no dedicated import to clean up.",
     "question": "Is there a `use std::process` import that would become dead after this change?",
     "severity": "ADVISORY",
     "recommendation": "No action needed -- verified that `process::exit` is called via its fully qualified path (`std::process::exit(1)`) at line 278, so there is no orphaned import."
   }

4. {
     "gap": "The edge case table shows 'Config parse error' exits 0, but the actual code at line 109-113 prints an error and continues with config=None. If the JSON serialization later fails (serde_json error at line 415), that would propagate as a non-zero exit. The spec's error handling section mentions this but the edge case table does not distinguish these two failure modes.",
     "question": "Should the edge case table have a row for 'serde_json serialization failure' to be complete?",
     "severity": "ADVISORY",
     "recommendation": "Minor table completeness issue. The error handling section already addresses this correctly -- serde_json errors are command failures, not diagnostic findings. No change strictly required."
   }

5. {
     "gap": "No mention of backward compatibility or migration. Users or CI scripts that rely on `great status` returning exit 1 for missing tools will silently stop detecting issues.",
     "question": "Is there any documentation, README, or CHANGELOG entry needed to communicate this behavioral change to users?",
     "severity": "ADVISORY",
     "recommendation": "The project is pre-1.0 (version 0.1.0), so breaking changes are expected. A CHANGELOG entry or commit message noting the behavioral change would be good practice but is not blocking for a pre-release project."
   }

## Summary

Clean, well-referenced spec for a small refactor with no BLOCKING issues. The line numbers are accurate, the clippy implications are correctly identified, the test plan covers the change, and the acceptance criteria from the original task are fully satisfied. Approved for implementation.
