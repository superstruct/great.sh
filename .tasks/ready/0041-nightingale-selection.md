# Nightingale Selection — Task 0041

**Selected task:** 0041 — `great mcp test <name>` shows wrong error when no [mcp] section exists
**Rejected task:** 0040 — `great status` exit code inconsistency (design decision required)
**Selection date:** 2026-03-04
**Selected by:** Nightingale (Florence Nightingale, Requirements Curator)

---

## Selection Rationale

Both tasks are P3. The tiebreaker is actionability.

**0040 is not implementable this iteration.** The task file states explicitly: "No implementation is in scope for this task. A single exit-code contract must be documented before implementation begins." Selecting 0040 would produce a decision document — no code shipped, no user-facing improvement delivered. The three options (A, B, C) require a human design decision that has not been made.

**0041 is fully specified and immediately actionable.** The fix is a single conditional guard on an early-exit path in `src/cli/mcp.rs`. The root cause is verified against the source: `mcps.is_empty()` at line 183 fires before the per-name lookup at line 188, making the name-specific error at line 192 unreachable when the MCP map is empty. The repair is to scope the early-exit to the `name.is_none()` case only, so a named lookup always proceeds to the existing error path.

User-facing value: a user running `great mcp test myserver` against a config with no `[mcp]` section currently receives a generic warning that ignores their input. After the fix they receive `MCP server 'myserver' not found in great.toml`, which tells them exactly what went wrong and how to fix it.

---

## Task Summary

| Field | Value |
|---|---|
| Priority | P3 |
| Type | bugfix |
| Module | `src/cli/mcp.rs` — `run_test()` |
| Complexity | XS (~5 lines) |
| Dependencies | None |

---

## Verified Source State

File: `/home/isaac/src/sh.great/src/cli/mcp.rs`

```rust
// Line 183 — early-exit fires unconditionally when MCP map is empty
if mcps.is_empty() {
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}

// Line 188 — name-specific lookup; unreachable when map is empty
let servers_to_test: Vec<(&String, &crate::config::schema::McpConfig)> = match name {
    Some(n) => match mcps.get_key_value(n) {
        Some(pair) => vec![pair],
        None => {
            // Line 192 — correct error; currently unreachable when mcps is empty
            output::error(&format!("MCP server '{}' not found in great.toml", n));
            return Ok(());
        }
    },
    None => mcps.iter().collect(),
};
```

**Required change:** Add `&& name.is_none()` to the guard condition at line 183, or restructure so the name-specific branch is evaluated before the generic empty-map guard.

---

## Acceptance Criteria (from task file — verified complete)

1. `great mcp test nonexistent_xyz` (no `[mcp]` section) prints `MCP server 'nonexistent_xyz' not found in great.toml` and exits 0.
2. `great mcp test nonexistent_xyz` (some servers exist, but not this one) prints the same name-specific error (no regression).
3. `great mcp test` (no name, no `[mcp]` section) continues to print the generic "No MCP servers declared" warning (no regression).
4. `great mcp test` (no name, servers exist) continues to test all servers (no regression).

All four criteria are independently testable. Four distinct code paths, four test cases. No ambiguity.

---

## Notes for Lovelace (Spec)

The simplest correct fix is:

```rust
if mcps.is_empty() && name.is_none() {
    output::warning("No MCP servers declared in great.toml.");
    return Ok(());
}
```

This lets a named lookup fall through to the `match name` block, which handles the "not found" case correctly at line 192. No new logic is required.

An alternative is to restructure so the `match name` block comes first and the empty-map guard only applies to the `None` arm — but the single-condition guard above is simpler and achieves identical behaviour.

Do not change exit codes. Task 0040 (exit code design decision) is intentionally deferred and out of scope here.
