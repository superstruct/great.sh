# Socrates Review: Spec 0013 -- Fix `statusLine` Schema

**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0013-statusline-fix-spec.md`
**Source:** `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24

---

## VERDICT: APPROVED

---

## Line Reference Verification

I cross-referenced every line number claim in the spec against the source file at `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`:

| Spec Claim | Actual | Status |
|------------|--------|--------|
| Line 141: closing `}` of `pub fn run` | Line 141 is `}` closing `pub fn run` | CORRECT |
| Line 143: doc comment for `run_install` | Line 143 is `/// Install the great.sh Loop agent team...` | CORRECT |
| Lines 217-219: broken `statusLine` JSON literal in new-file path | Lines 217-219 contain `"statusLine": { "command": "great statusline" }` | CORRECT |
| Line 238: broken `statusLine` JSON literal in existing-file injection | Line 238 contains `serde_json::json!({"command": "great statusline"})` | CORRECT |
| Lines 228-254: full injection block | Lines 228-254 match the quoted code exactly | CORRECT |
| Lines 201-220: full `default_settings` block | Lines 201-220 match the quoted code exactly | CORRECT |

All line references are accurate.

---

## Completeness Analysis

The spec correctly identifies all code paths that write `statusLine`. I confirmed via grep that only `/home/isaac/src/sh.great/src/cli/loop_cmd.rs` contains `statusLine` references -- no other files in the codebase write this key.

The three changes (helper extraction, new-file fix, existing-file repair) cover:
1. Fresh install (no `settings.json` exists) -- Change 2
2. Existing file without `statusLine` key -- Change 3 (inject branch)
3. Existing file with broken `statusLine` (missing `type`) -- Change 3 (repair branch)
4. Existing file with correct `statusLine` -- Change 3 (no-op branch)

---

## Concerns

```
{
  "gap": "Tests 3 and 4 (test_broken_statusline_detected, test_correct_statusline_not_detected_as_broken) only assert JSON structure properties -- they do not exercise the actual repair logic in run_install. They verify that a broken shape lacks 'type' and a correct shape has 'type', but not that the repair code path actually detects and fixes the broken shape.",
  "question": "Are these tests sufficient to prevent regression, or should at least one integration-style test exercise the repair logic itself (e.g., write a broken settings.json to a tempdir, call the repair logic, and assert the file is corrected)?",
  "severity": "ADVISORY",
  "recommendation": "Consider adding a test that exercises the actual repair branch by creating a tempfile with the broken shape and verifying the output. However, since run_install has filesystem side effects tied to the real home directory, this would require refactoring to accept a path parameter. The current tests are acceptable for an S-sized bugfix -- they guard the helper function and the detection condition. The manual verification steps in the spec cover the integration path."
}
```

```
{
  "gap": "The run_status function (lines 314-382) does not check whether statusLine is correctly shaped. After this fix, great loop status will report 'installed and ready' even if the user has a broken statusLine from a previous install.",
  "question": "Should the spec include a statusLine health check in run_status to warn users who have not yet re-run install?",
  "severity": "ADVISORY",
  "recommendation": "This is a separate enhancement, not a regression from this fix. The repair logic in run_install handles the case when the user re-runs install. A status check could be a follow-up task. Not blocking for this bugfix."
}
```

```
{
  "gap": "The spec says 'Insert after line 141' for Change 1, but Change 2 says 'Replace lines 217-219'. After inserting the helper function (approximately 8 lines including blank lines), the original line 217 will shift to approximately line 225. The spec's line references for Changes 2 and 3 refer to the ORIGINAL line numbers, not post-insertion numbers.",
  "question": "Is the implementer expected to understand that line references in Changes 2 and 3 refer to the original source, not the file after Change 1 is applied?",
  "severity": "ADVISORY",
  "recommendation": "The spec's build order section says 'apply in this order', and the 'Exact Code Changes' section uses original line numbers consistently. The implementer (Da Vinci) should apply changes bottom-up or use content matching rather than line numbers. The spec provides full quoted code blocks for context matching, which is sufficient. No change needed -- this is standard practice for multi-edit specs."
}
```

```
{
  "gap": "The repair logic only detects a missing 'type' field. It does not check whether 'type' has the correct value 'command'. If a user manually sets 'type' to some other value (e.g., 'text'), the repair logic will not fix it.",
  "question": "Is this intentional? The spec explicitly states 'we only fix missing type, not wrong type' -- is that the correct design choice?",
  "severity": "ADVISORY",
  "recommendation": "This is a deliberate design choice documented in the spec's 'Repair Logic Pseudocode' section: 'It already has a type field (even if the value is something unexpected -- we only fix missing type, not wrong type)'. This is the conservative approach -- if a user deliberately set a different type value, we should not silently overwrite it. The only known broken state is the missing type field produced by the current great.sh code. APPROVED as-is."
}
```

---

## Correctness Assessment

The proposed fix is correct. The `statusline_value()` helper produces the exact JSON shape Claude Code requires:

```json
{"type": "command", "command": "great statusline"}
```

Using a single helper function for both code paths eliminates the root cause (duplicated literals diverging). The repair logic correctly handles the three possible states of existing files: missing key, broken key, correct key.

The `serde_json::json!` macro accepts function calls in value position, so `"statusLine": statusline_value()` is valid Rust syntax.

## Safety Assessment

- The repair only modifies the `statusLine` key. All other keys in `settings.json` are preserved by serde_json's `Value` round-trip.
- The `else` fallback (non-object `statusLine`, or `statusLine` with existing `type` field) is a safe no-op.
- The concurrent access analysis is correct: both writers produce the same correct shape, so last-writer-wins is fine.
- No new dependencies, no new file paths, no user input in the JSON output.

## Test Coverage Assessment

The four proposed tests cover:
1. Helper function produces correct shape (guards against future regression of the exact bug being fixed)
2. Full `default_settings` block includes correct shape (guards the new-file path)
3. Broken shape is detectable (validates the detection condition)
4. Correct shape is not falsely detected as broken (guards against unnecessary rewrites)

These are adequate unit tests for an S-sized bugfix. The manual verification steps provide integration coverage.

---

## Summary

Spec is thorough, line references are verified accurate, the fix is correct, edge cases are exhaustive, and the repair logic is conservatively safe. All advisory concerns are documented design choices or follow-up work outside the scope of this bugfix. Approved for implementation.
