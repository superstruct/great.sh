# Socrates Review: Spec 0042 -- `great status` Doctor Hint on Issues

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-03-04
**Spec:** `.tasks/ready/0042-status-doctor-hint-spec.md`
**Task:** `.tasks/backlog/0042-status-doctor-hint.md`

---

VERDICT: APPROVED

---

## Concerns

### 1. MCP unavailable triggers `has_issues` in human mode but NOT in JSON mode

```
{
  "gap": "The JSON path (`run_json`) does not push to `issues` when an MCP command is unavailable (lines 344-356 of status.rs). The human path will now flag MCP-unavailable via `has_issues`. This creates a behavioral asymmetry: `great status` shows the doctor hint for a missing MCP command, but `great status --json` reports `has_issues: false` for the same state.",
  "question": "Should the spec acknowledge this pre-existing asymmetry explicitly, or is it intentionally out of scope?",
  "severity": "ADVISORY",
  "recommendation": "Add a note to the Edge Cases table acknowledging that the JSON `has_issues` field does not currently track MCP command availability and that aligning it is a separate task. This prevents the builder from 'fixing' it as a drive-by change."
}
```

### 2. `has_issues` variable name collides with the JSON struct field

```
{
  "gap": "The `StatusReport` struct already has a field named `has_issues` (line 29 of status.rs). The new local variable in `run()` is also named `has_issues`. While they live in different scopes and there is no compilation issue, a future refactor that moves the human path into a helper function returning a struct could create confusion.",
  "question": "Is the naming collision between the local `has_issues` boolean and the `StatusReport.has_issues` field intentional or incidental?",
  "severity": "ADVISORY",
  "recommendation": "This is fine as-is given the XS scope. A comment like `// Tracks whether to show the doctor hint (distinct from StatusReport.has_issues)` would help, but is not required."
}
```

### 3. No test for MCP-unavailable triggering the hint

```
{
  "gap": "The spec tests missing tools and missing secrets, but does not test that an unavailable MCP command triggers the hint. The spec identifies three trigger sites (tools, MCP, secrets) but only tests two of them.",
  "question": "Is the missing MCP test case an intentional omission (e.g., hard to set up in integration tests) or an oversight?",
  "severity": "ADVISORY",
  "recommendation": "If MCP command availability is easy to test (declare an MCP server with a nonexistent command like `nonexistent-mcp-xyz-9999`), add a test. If it is hard to set up, document why it is omitted."
}
```

### 4. Hint text uses single quotes around `great doctor` -- consistency with existing output

```
{
  "gap": "The hint text is: `Tip: use 'great doctor' for exit-code health checks in CI.` The existing no-config warning at line 163 uses backticks: `Run \`great init\` to create one.` This is a minor style inconsistency.",
  "question": "Should the hint use backticks for command references to match the existing convention, or are single quotes acceptable?",
  "severity": "ADVISORY",
  "recommendation": "Use backticks for consistency: `Tip: use \`great doctor\` for exit-code health checks in CI.` -- but either is fine for XS scope."
}
```

### 5. Re-introduction of tracking variable after task 0040 removed one

```
{
  "gap": "Task 0040 removed `has_critical_issues` (and its three assignment sites) because it drove no logic after the exit(1) removal. Task 0042 re-introduces a nearly identical pattern (`has_issues` with three assignment sites). This is a legitimate new use (hint printing vs exit code), but the pattern echo deserves acknowledgment.",
  "question": "Is there a cleaner alternative that avoids re-threading a mutable boolean through the same three sites -- for example, counting issues or checking at the end whether any error lines were printed?",
  "severity": "ADVISORY",
  "recommendation": "The mutable boolean is the simplest correct approach for XS scope. No change needed, but the spec could add a one-line note acknowledging the deliberate reuse of the pattern that 0040 removed, to prevent a reviewer from flagging it as a revert."
}
```

### 6. `output::info` prefix may be visually confusing for a hint/tip

```
{
  "gap": "The `output::info()` function prints with a blue info icon prefix (line 19 of output.rs). The same function is used throughout `run()` for factual status lines like 'Platform: ...' and 'Config: ...'. Using it for a suggestion/tip blurs the distinction between status data and actionable advice.",
  "question": "Would `output::warning()` (yellow) be more appropriate for an actionable tip, or is the blue info prefix intentional to keep the tone non-alarming?",
  "severity": "ADVISORY",
  "recommendation": "Either is defensible. If blue is chosen (as spec states), the 'Tip:' prefix in the text itself provides sufficient differentiation. No change required."
}
```

### 7. Hint placement relative to the trailing blank line

```
{
  "gap": "The spec says to insert the hint 'after the final println!() (current line 267) and before Ok(()) (current line 274)'. The final `println!()` at line 267 prints a blank line. This means the hint will appear after the blank line, visually separated from the report body. If the intent is for the hint to appear as part of the report (before the trailing blank line), the insertion point should be before line 267.",
  "question": "Should the doctor hint appear before or after the trailing blank line? The current spec places it after, which means it appears visually detached from the report.",
  "severity": "ADVISORY",
  "recommendation": "Consider placing the hint before `println!()` so it reads as the report's closing line, then the blank line provides the final visual separator. The builder can judge visually, but the spec should clarify intent."
}
```

---

## Verification of Spec Claims

| Claim | Verified |
|---|---|
| `output::info()` writes to stderr | Yes -- `eprintln!` at line 20 of `src/cli/output.rs` |
| JSON mode returns before human path | Yes -- line 121-123 of `status.rs` |
| Line numbers for insertion points | Yes -- all match current `status.rs` |
| Test function names match current code | Yes -- verified at lines 1937, 1962, 2048 of `tests/cli_smoke.rs` |
| `predicate::str::contains(...).not()` is available | Yes -- standard `predicates` crate API |
| Three issue sites (tools runtime, tools cli, MCP, secrets) | Yes -- four insertion points across three categories, spec correctly identifies all |

## Summary

A clean, well-scoped XS spec with accurate line references and correct understanding of the output module. Seven ADVISORY concerns (MCP test gap, naming collision, hint placement, style consistency) but nothing that blocks implementation. The builder should pay attention to concern 3 (MCP test coverage) and concern 7 (hint placement relative to the trailing blank line).
