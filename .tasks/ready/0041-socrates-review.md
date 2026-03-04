# 0041 -- Socrates Review: `great mcp test <name>` shows wrong error when no `[mcp]` section exists

- **Reviewer:** Socrates (Adversarial Spec Reviewer)
- **Date:** 2026-03-04
- **Round:** 1

---

## VERDICT: APPROVED

---

## Key Findings

### 1. Line numbers are accurate

Verified against `/home/isaac/src/sh.great/src/cli/mcp.rs`:
- `run_test()` starts at line 167 (spec says 167) -- correct
- `let mcps = cfg.mcp.unwrap_or_default()` at line 181 (spec says 181) -- correct
- `if mcps.is_empty()` guard at line 183 (spec says 183) -- correct
- `get_key_value(n)` at line 189 (spec says 189) -- correct
- `output::error(...)` "not found" message at line 192 (spec says 192) -- correct

### 2. Root cause analysis is correct

The buggy guard at line 183 unconditionally returns when `mcps` is empty, preventing execution from reaching the name-specific error at line 192. The fix (`&& name.is_none()`) is the minimal correct change. I traced the control flow for each of the 5 behavioral matrix scenarios and all produce the expected output after the fix.

### 3. Behavioral matrix is exhaustive

The 5 scenarios cover the full cross-product of {named, unnamed} x {empty, non-empty, matching} states. No scenarios are missing.

### 4. Edge cases are well-considered

The spec correctly identifies and handles:
- No `great.toml` at all (line 171-178 `Err` branch, unaffected)
- Invalid TOML (same `Err` path)
- Empty `[mcp]` table (parses to `Some(HashMap::new())`, same as missing section)
- Empty string name (acceptable behavior: "MCP server '' not found")
- Platform independence (pure in-memory control flow)

### 5. Tests are correct and sufficient

All four tests verified against the actual codebase:
- `great()` helper exists at line 6 of `tests/cli_smoke.rs`
- `TempDir`, `predicate`, `Command` imports already present
- `output::header()` uses `eprintln!` (line 25 of `output.rs`), so `.stderr()` assertions are correct
- `output::warning()` and `output::error()` also use `eprintln!`, so all stderr assertions are correct
- No naming conflicts with existing tests (`mcp_list_no_config`, `mcp_add_no_config`, `mcp_add_creates_entry` exist but no `mcp_test_*` pattern)

### 6. No regressions possible

The change adds a conjunction (`&& name.is_none()`) to an existing guard. When `name` is `None` (the pre-fix common path), behavior is identical. The only change is when `name` is `Some(...)` AND `mcps` is empty -- a case that was previously short-circuited with a wrong message.

---

## Concerns

### ADVISORY

```
{
  "gap": "AC4 test spawns `echo hello` via mcp::test_server(), which sleeps 500ms",
  "question": "Does adding a 500ms sleep to the test suite concern the team?",
  "severity": "ADVISORY",
  "recommendation": "No action required. The 500ms cost is trivial in the context of a full cargo test run. Note for future: if many such tests accumulate, consider a mock."
}
```

```
{
  "gap": "Spec section 2 comment says 'Line 181' for unwrap_or_default but the code block shows it as // Line 181 while the 'before' block in section 3 starts at line 183 -- no line 181 shown in the before/after diff",
  "question": "Is the diff context sufficient for the builder to locate the change?",
  "severity": "ADVISORY",
  "recommendation": "The before/after blocks in section 3 are unambiguous (only one `if mcps.is_empty()` in the file). Builder will have no difficulty locating the change."
}
```

---

## Summary

This is a textbook XS bugfix spec: accurate line numbers, correct root cause, minimal one-line fix, exhaustive behavioral matrix, and four well-targeted regression tests covering every acceptance criterion from the backlog. No blocking concerns.
