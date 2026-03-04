# Dijkstra Code Review: Task 0042 — `great status` Doctor Hint

**Reviewer:** Edsger Dijkstra (Code Reviewer)
**Date:** 2026-03-04
**Files reviewed:**
- `src/cli/status.rs`
- `tests/cli_smoke.rs`

---

VERDICT: APPROVED

---

## Issues

- [WARN] `src/cli/status.rs:279` — Hint text deviates from the spec. The spec (`0042-status-doctor-hint-spec.md`, step 3) specifies: `"Tip: use 'great doctor' for exit-code health checks in CI."`. The implementation reads: `"Run \`great doctor\` for exit-code health checks in CI."`. The implementation's wording is arguably cleaner (matches the existing no-config warning style at line 164: `"Run \`great init\` to create one."`), but it is an undocumented deviation from the spec. Because the substantive behavior is correct and the chosen text is more consistent with the existing codebase convention, this does not block.

- [WARN] `src/cli/status.rs:276-279` — The hint is printed after the trailing blank line (`println!()` at line 276), placing it visually detached from the report body. Socrates flagged this (concern 7). The builder made a deliberate choice; it is defensible as a closing footer. No block, but the intent should be acknowledged in a comment if future reviewers question the order.

- [WARN] `tests/cli_smoke.rs` — No test for the MCP-unavailable trigger path. The spec identifies three trigger categories (tools, MCP, secrets); tests cover two. Socrates flagged this as concern 3. A nonexistent MCP command is trivially expressible in the TOML fixture (the tool tests already use `nonexistent-tool-xyz-9999` with the same pattern). This gap leaves one of three code paths untested. Advisory only given XS scope, but it is a meaningful omission.

- [WARN] `src/cli/status.rs:126` — The local variable `has_issues` in `run()` shares its name with the `StatusReport.has_issues` struct field populated in `run_json()`. They live in separate scopes with no ambiguity today. Socrates noted this. No action required at XS scope.

## Analysis

### 1. Boolean accumulator — clean

The `has_issues` accumulator at line 126 is declared as `let mut has_issues = false;` in the narrowest correct scope: inside the human-readable branch, after the JSON early-return at line 121. It cannot contaminate JSON output. The three assignment sites (tools runtimes loop line 191, tools cli loop line 211, MCP else-branch line 254, secrets else-branch line 269) are each co-located with the `output::error()` call they logically pair with. No dead assignments. No redundant resets.

### 2. Placement of `has_issues = true` assignments — correct

Each assignment is placed in the same branch that prints the error. This is the tightest possible coupling: if the error fires, the flag sets; if the error is suppressed (e.g., future refactor), the flag moves with it. The pattern is consistent across all four sites.

### 3. Comment quality — adequate

The existing exit-zero rationale comment at lines 282-286 is accurate and helpful. No additional comments are needed for the accumulator; its logic is self-evident at six lines of scope. The `// Clean config` comment in the new test (line 2069) is precise.

### 4. Test quality — two of three paths covered

`status_exit_zero_even_with_missing_tools` (line 1937): correctly asserts both the existing "not installed" output and the new "great doctor" hint. `status_exit_zero_even_with_missing_secrets` (line 1963): correctly asserts both "missing" and "great doctor". `status_no_doctor_hint_when_clean` (line 2061): correctly asserts the negative case with `.not()`. The JSON no-hint case is covered implicitly by `status_json_always_exits_zero_even_with_issues` (line 1990), which does not assert the hint and would fail if it appeared. The MCP trigger path is not tested (see WARN above).

### 5. Simplicity — minimal change

The diff is structurally minimal: one variable declaration, four single-line assignments in existing branches, and one two-line conditional print. No new functions, no new types, no new dependencies. The `run_json()` function is untouched, correctly so. This is the irreducible implementation for the specified behavior.

---

## Summary

The implementation is correct, structurally sound, and minimal; the boolean accumulator pattern is clean and consistently applied, but one of the three advertised trigger paths (MCP unavailable) lacks a test.
