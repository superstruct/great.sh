# 0041 — Code Review: Dijkstra (Edsger)

**Task:** 0041 — `great mcp test <name>` shows wrong error when no `[mcp]` section exists

**Reviewer:** Dijkstra (Code Structure, Correctness, Complexity)

**Date:** 2026-03-04

---

## VERDICT: APPROVED

---

## Issues

None.

---

## Detailed Assessment

### Production Code Change: `/home/isaac/src/sh.great/src/cli/mcp.rs` Line 183

**Before:**
```rust
if mcps.is_empty() {
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}
```

**After:**
```rust
if mcps.is_empty() && name.is_none() {
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}
```

**Correctness:** ✓ The guard now correctly distinguishes two cases:
- `mcps.is_empty() && name.is_none()`: User ran `great mcp test` (no name) — show generic warning.
- `mcps.is_empty() && name.is_some()`: User ran `great mcp test xyz` — fall through to line 188 where `mcps.get_key_value(n)` returns `None`, triggering the name-specific error at line 192.

The control flow is sound. The early-exit guard only fires when *both* conditions hold, allowing the name-aware lookup to execute when a name is provided.

**Complexity:** ✓ Minimal. Single boolean operator added. Cyclomatic complexity unchanged. Decision tree is straightforward.

**Abstraction Boundary:** ✓ Function responsibility preserved: "test one or all MCP servers." The function still accepts `Option<&str>` and dispatches accordingly.

### Integration Tests: `/home/isaac/src/sh.great/tests/cli_smoke.rs` Lines 497–590

Four new tests added, each corresponding to one acceptance criterion:

| Test | AC# | Scenario | Setup | Assertion |
|------|-----|----------|-------|-----------|
| `mcp_test_named_no_mcp_section_shows_name_error()` | 1 | Named server, no `[mcp]` section | Config: `[project]` only | stderr contains "MCP server 'nonexistent_xyz' not found" |
| `mcp_test_named_not_found_shows_name_error()` | 2 | Named server, exists elsewhere | Config: has `[mcp.filesystem]` | stderr contains "MCP server 'nonexistent_xyz' not found" |
| `mcp_test_no_name_no_mcp_section_shows_generic_warning()` | 3 | No name, no `[mcp]` section | Config: `[project]` only | stderr contains "No MCP servers declared" |
| `mcp_test_no_name_with_servers_tests_all()` | 4 | No name, servers exist | Config: has `[mcp.test-echo]` | stdout contains "Testing MCP Servers" |

**Test Quality:**
- ✓ Each test is independent: uses fresh `TempDir`, writes isolated `great.toml`.
- ✓ Assertions are precise: check for specific substrings in stderr/stdout, verify success exit code.
- ✓ No unwrap() in test paths: all unwrap calls are on infallible operations (TempDir creation, file I/O in test setup).
- ✓ Naming is clear: test names follow the "AC#" pattern and describe the scenario directly.
- ✓ Pattern consistency: uses the project's `assert_cmd` + `predicates` + `tempfile` pattern (seen in surrounding tests like `mcp_add_formats_toml_correctly()`).

**Coverage:**
- AC1 and AC2 cover the two variants of the named-server case (with and without existing servers).
- AC3 and AC4 cover the two variants of the unnamed case (with and without servers).
- All four acceptance criteria from the spec are addressed.

### Error Handling

✓ No change to error propagation. All error paths use `output::error()` or `output::warning()` and return `Ok(())` (exit code 0). Behavior is unchanged from the spec.

### Naming & Conventions

✓ No new public APIs or identifiers. Test function names follow the project pattern: `mcp_<context>_<scenario>_<expectation>()`.

---

## Summary

A minimal, correct fix to a control-flow bug in `run_test()`. The single-line guard condition addition properly gates the early exit to cases where no servers exist *and* no specific server was requested. Comprehensive integration tests cover all acceptance criteria without introducing complexity or unsafe patterns. Approved for merge.
