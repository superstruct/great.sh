# Review 0008: Runtime Version Manager Integration (mise)

**Spec:** `.tasks/ready/0008-runtime-manager-spec.md`
**Backlog:** `.tasks/backlog/0008-runtime-manager.md`
**Reviewer:** Socrates
**Round:** 2 (re-review after revision)
**Date:** 2026-02-25

---

## VERDICT: APPROVED

All BLOCKING and HIGH concerns from round 1 have been addressed. Two ADVISORY issues remain (one carried forward, one new), neither of which prevents implementation.

---

## Round 1 Concern Resolution

### 1. BLOCKING -- File framing: "create" vs "modify"

**Status: RESOLVED.**

The revised spec (line 5) explicitly states: `"Type: bug fix + behavioral change + new tests (modification of existing file)"`. The "Files to Modify" table (line 24-27) correctly lists the single target file. The "Files NOT Modified" table (lines 29-33) explicitly calls out `mod.rs` and `apply.rs` as already correct. The "Existing Code Inventory" table (lines 39-56) catalogs all 15 items in the existing file with a disposition column (KEEP vs MODIFY). This is exactly what was requested.

### 2. HIGH -- version_matches bug not framed as bugfix

**Status: RESOLVED.**

The revised spec dedicates an entire section ("Change 1: BUG FIX -- `version_matches` prefix-boundary matching", lines 60-144) to this issue. It shows the existing buggy code, explains the bug with concrete examples (`"3.120.0".starts_with("3.12")` returns true), provides a truth table with 9 cases, and includes dedicated regression tests. The fix logic is correct -- I traced all 9 truth table cases mentally and they all produce the expected results. The dot-boundary approach (`rest.starts_with('.')`) is sound.

### 3. HIGH -- ensure_installed brew-first strategy undocumented

**Status: RESOLVED.**

The revised spec dedicates a full section ("Change 2: BEHAVIORAL CHANGE -- `ensure_installed` brew-first strategy", lines 147-254) to this change. It shows existing code, replacement code, rationale (4 bullet points), platform impact table covering 6 platform scenarios, and explicitly acknowledges the Linuxbrew latency trade-off. The spec also notes (line 238) that the backlog suggested `ensure_installed(pkg_manager: &dyn PackageManager)` but the actual consumer (`apply.rs` line 495) calls with zero arguments, so the spec correctly matches the consumer.

### 4. HIGH -- installed_version "No version" vs "Not installed"

**Status: RESOLVED.**

The revised spec ("Change 3: BUG FIX", lines 258-313) adds case-insensitive checking for both `"no version"` and `"not installed"` via `to_lowercase()`. The replacement code at lines 293-308 is correct: `let lower = version.to_lowercase();` followed by `lower.contains("no version") || lower.contains("not installed")`. This handles current and future mise versions.

### 5. ADVISORY -- std::process import clarity

**Status: RESOLVED.**

The revised spec (lines 330-333) explicitly states: "Similarly, `std::process::Command` and `std::process::Stdio` are used via fully-qualified paths throughout the file [...] This is the existing pattern and will not be changed."

### 6. ADVISORY -- test_provision_result_fields flakiness

**Status: RESOLVED.**

The revised spec removes `test_provision_result_fields` entirely (lines 478-485) with a clear explanation of why it was environment-dependent and replaced it with pure-logic alternatives (`test_provision_skips_cli_key` and `test_provision_empty_runtimes`).

### 7. ADVISORY -- diff.rs listed as consumer

**Status: RESOLVED.**

The revised spec (lines 20-21) explicitly states: "`src/cli/diff.rs` is NOT a consumer of `MiseManager`. It uses `command_exists(name)` and `util::get_command_version(name)` independently. Any future integration belongs in a separate task."

### 8. ADVISORY -- version_matches param naming

**Status: RESOLVED.**

The revised spec keeps the parameter name `installed` (matching existing code), not `actual`. Line 108 explicitly states: "The parameter name stays `installed` (not renamed to `actual`) to minimize diff."

---

## New Concerns

### 9. Existing test count discrepancy in spec text

```
{
  "gap": "The spec repeatedly states the existing file has '7 tests' (line 12: '7 tests', line 56: '7 tests'). However, the actual runtime.rs has exactly 6 tests: test_mise_is_available, test_version_matches_exact, test_version_matches_prefix, test_version_matches_latest, test_version_no_match, test_provision_action_eq. The #[test] attribute appears 6 times between lines 210-252. Despite this, the final test count table (lines 488-505) correctly enumerates all 15 individual tests (6 existing + 9 new), and the 'New Tests to ADD' heading says '8 tests' while the code block actually contains 9 test functions.",
  "question": "Can the builder derive the correct test count from the enumerated table and code blocks?",
  "severity": "ADVISORY",
  "recommendation": "The enumerated table at lines 488-505 and the actual test code blocks are internally consistent and contain all 15 tests. The headings ('7 existing', '8 new') are off-by-one in both cases but cancel out to produce the correct total of 15. A competent builder following the code and table will produce the correct result. No action required."
}
```

### 10. No test for installed_version handling of "Not installed" string

```
{
  "gap": "Change 3 adds case-insensitive detection of 'Not installed' in the installed_version function, but no new test exercises this path. The only test for installed_version is test_installed_version_nonexistent_runtime which calls mise with a fake runtime name -- if mise is absent, it returns None from the .ok()? path (Command fails to execute), never reaching the string-parsing logic. If mise IS present, the output for a nonexistent runtime is likely an error exit (non-success status), also bypassing the string check.",
  "question": "Is the 'Not installed' string parsing tested by any of the 15 tests?",
  "severity": "ADVISORY",
  "recommendation": "The fix is simple and obviously correct (adding one more .contains() check to an existing conditional), so the lack of a direct unit test is acceptable. A proper test would require either mocking Command output or having mise installed with a declared-but-not-installed runtime, both of which add complexity beyond this task's scope. The acceptance criteria at line 618 explicitly call out this behavior, so manual verification can cover it."
}
```

---

## Verification Summary

| Check | Result |
|-------|--------|
| version_matches dot-boundary logic correct? | YES -- all 9 truth table cases verified |
| brew-first ensure_installed breaks any platform? | NO -- falls through to curl when brew absent |
| 15 tests feasible without mise installed? | YES -- pure logic tests need no external tools; Command-based tests use .ok()? which returns None gracefully |
| ToolsConfig construction in tests matches schema.rs? | YES -- `runtimes: HashMap<String, String>`, `cli: Option<HashMap<String, String>>` |
| Public API unchanged (apply.rs compatibility)? | YES -- all 6 public function signatures preserved |
| No .unwrap() in production code? | YES -- existing .unwrap_or("") at line 45 is not .unwrap(); spec adds no new unwraps outside #[cfg(test)] |
| Parameter name `installed` preserved? | YES -- spec explicitly keeps it |

---

## Summary

The revised spec is exemplary in its treatment of an existing-file modification: it inventories every item in the current file, categorizes each change (BUG FIX / BEHAVIORAL CHANGE / NEW ADDITION), shows before-and-after code with truth tables, documents platform impact, and provides a comprehensive test suite. All BLOCKING and HIGH concerns from round 1 are fully addressed. The two remaining ADVISORY concerns (off-by-one in heading counts, no direct test for "Not installed" string parsing) do not impede implementation.
