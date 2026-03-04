# 0043 Socrates Review -- Status MCP Test Coverage and JSON Bug Fix

**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-03-04
**Spec:** `.tasks/ready/0043-status-mcp-test-spec.md`
**Task:** `.tasks/backlog/0043-status-mcp-test-coverage.md`

---

## VERDICT: APPROVED

---

## Bug Diagnosis Verification

The spec correctly identifies the bug. In `/home/isaac/src/sh.great/src/cli/status.rs` lines 357-369, the `run_json()` MCP block builds `McpStatus` entries with `command_available: command_exists(&m.command)` but never calls `issues.push()` when `command_available` is false. This means `has_issues` (derived from `!issues.is_empty()` at line 416) will be false even when an MCP server command is missing from PATH.

By contrast, the tools block (lines 311-355) and secrets block (lines 383-408) both push to `issues` when they detect problems.

The human-readable `run()` function handles this correctly at line 254: `has_issues = true` is set when `!cmd_available`. The asymmetry between `run()` and `run_json()` confirms this is a real bug.

## Config Parsing Verification

The test config `[mcp.fake-server]\ncommand = "nonexistent_mcp_status_xyz_9999"` will parse correctly. The `McpConfig` struct in `/home/isaac/src/sh.great/src/config/schema.rs` lines 96-111 requires only `command: String`; all other fields (`args`, `env`, `transport`, `url`, `enabled`) are `Option`. TOML table keys with hyphens are valid. Confirmed by existing test `test_mcp_with_transport` at schema.rs line 514 which uses `[mcp.remote]` with only `command` and two optional fields.

## JSON Output Structure Verification

The test assertions match the `StatusReport` struct (lines 15-31):
- `parsed.get("mcp")` -- matches field `mcp: Option<Vec<McpStatus>>`
- `s["name"]`, `s["command_available"]` -- matches `McpStatus` fields (lines 43-51)
- `parsed["has_issues"]` -- matches field `has_issues: bool`
- `parsed["issues"]` -- matches field `issues: Vec<String>`

No new fields are added. No JSON schema change.

---

## Concerns

### 1.

```
{
  "gap": "The spec claims the closure chain prevents mutable borrow of `issues`, but the secrets block (lines 386-398) successfully uses `.map()` with `issues.push()` inside nested closures.",
  "question": "Is the refactor to if-let actually necessary, or could the fix be a 3-line addition of `if !command_exists(...) { issues.push(...) }` inside the existing `.map()` closure -- matching the secrets block pattern?",
  "severity": "ADVISORY",
  "recommendation": "The spec's proposed refactor is valid and improves consistency with the tools block. However, the stated justification (borrow-checker prevents it) appears incorrect. The builder should note that a simpler in-place fix inside the existing closure would also compile. The refactor is preferable for consistency, not necessity."
}
```

### 2.

```
{
  "gap": "The test TOML config omits [project] name field requirement ambiguity.",
  "question": "The test uses `[project]\nname = \"test\"` but the config schema shows `ProjectConfig.name` is `Option<String>`. Is the `[project]` section even needed for these tests, or is it just defensive?",
  "severity": "ADVISORY",
  "recommendation": "This is fine as-is -- it matches existing test patterns (e.g., `status_human_and_json_exit_codes_match` at cli_smoke.rs line 2021). No change needed, but the spec could note it is optional."
}
```

### 3.

```
{
  "gap": "Test 1 asserts stderr contains 'not found' but does not assert it contains the server name 'fake-server'.",
  "question": "Should Test 1 also assert that stderr contains 'fake-server' to confirm the correct MCP server triggered the message, not some other tool?",
  "severity": "ADVISORY",
  "recommendation": "Add `.stderr(predicate::str::contains(\"fake-server\"))` to Test 1 for specificity. The current test could pass if any unrelated 'not found' message appears on stderr."
}
```

### 4.

```
{
  "gap": "The spec does not mention the `enabled` field on McpConfig. The current MCP block in both `run()` and `run_json()` does not check `enabled`.",
  "question": "Should MCP servers with `enabled = false` be skipped entirely, or is that out of scope for this task?",
  "severity": "ADVISORY",
  "recommendation": "Out of scope -- neither the current code nor the task references `enabled`. But the builder should not inadvertently break future `enabled` support by making assumptions about iteration order."
}
```

### 5.

```
{
  "gap": "HashMap iteration order is non-deterministic. The spec's test asserts `mcp.iter().any(|s| s[\"name\"] == \"fake-server\")` which handles this correctly with `.any()`, but the issue message format is not pinned.",
  "question": "If multiple MCP servers were declared, would the issues array ordering matter for any downstream consumer?",
  "severity": "ADVISORY",
  "recommendation": "No action needed -- the tests use `.any()` correctly and only declare one MCP server. This is noted for completeness."
}
```

### 6.

```
{
  "gap": "The spec's proposed issue message format is 'MCP server '<name>' command '<command>' not found' but the human-readable mode at line 253 uses a different format: '  <name> (<command> -- not found)'.",
  "question": "Is the inconsistency between JSON issue messages and human-readable output intentional? The tools block also uses different formats between modes.",
  "severity": "ADVISORY",
  "recommendation": "Acceptable -- JSON issues are structured error descriptions while human output is display-formatted. This matches the tools pattern where JSON says 'tool X is not installed' while human shows 'X version -- not installed'. No change needed."
}
```

### 7.

```
{
  "gap": "The spec says to insert tests after line 2048 but the next test (`status_no_config_exits_zero`) starts at line 2050.",
  "question": "Is the insertion point between `status_human_and_json_exit_codes_match` and `status_no_config_exits_zero` correct? These are both status tests so the grouping makes sense.",
  "severity": "ADVISORY",
  "recommendation": "Correct insertion point. The builder should add the `#[test]` attribute and blank line separator matching existing conventions."
}
```

---

## Summary

The bug diagnosis is correct, the fix is sound, the test configs will parse, the JSON assertions match the actual output structure, and there are no borrow-checker issues with the proposed refactor -- though the stated reason for the refactor (borrow-checker friction) is likely inaccurate since the simpler in-place fix inside the existing closure would also compile, as demonstrated by the secrets block using the same pattern.
