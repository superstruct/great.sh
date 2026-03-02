# Security Audit: 0013 — Fix statusLine Schema

**Auditor:** Kerckhoffs
**Date:** 2026-02-24
**File:** `/home/isaac/src/sh.great/src/cli/loop_cmd.rs`
**Verdict:** PASS (no CRITICAL or HIGH findings)

---

## Scope

The diff introduces:

1. A `statusline_value()` helper returning a compile-time JSON literal (lines 147-152)
2. `statusLine` field added to the new-file creation path (line 228)
3. Repair logic: read-modify-write on existing `~/.claude/settings.json` (lines 238-275)
4. Four unit tests (lines 560-627)
5. Test count fix: `COMMANDS.len()` assertion updated from 4 to 5 (line 465)

---

## Checklist

### 1. Credential Leakage

**PASS.** No secrets, tokens, API keys, or credentials are involved. The JSON written contains only the static strings `"command"`, `"great statusline"`, and `"type"`. The file being modified (`~/.claude/settings.json`) may contain user permissions but no credentials are read, logged, or printed.

### 2. File Path Injection

**PASS.** All file paths are constructed from `dirs::home_dir()` (which reads `$HOME` or platform equivalent) joined with hardcoded path segments: `.claude`, `settings.json`, `agents`, `commands`, `teams/loop`. No user-supplied CLI arguments flow into any file path in the changed code. The `project: bool` flag only controls whether `.tasks/` is created in the current working directory.

Path traversal is not possible because:
- `home_dir()` returns an absolute path
- All `.join()` calls use static string literals
- No user-controlled strings are concatenated into paths

### 3. JSON Injection

**PASS.** No user input is incorporated into any JSON value. The `statusline_value()` function (line 147) returns a compile-time literal via `serde_json::json!`. Both code paths that write JSON use either this helper or other static literals. The repair logic reads the existing JSON, deserializes it to `serde_json::Value`, modifies only the `statusLine` key with the static value, and re-serializes. No string interpolation or format! macros are used to construct JSON.

### 4. File Permission Preservation

**LOW (informational).** `std::fs::write()` on an existing file preserves the file's existing permissions on both Linux and macOS. However, when creating a new `settings.json` (line 232), the file inherits the process umask (typically 0022, resulting in mode 644). This is acceptable behavior for a non-secret configuration file. The file contains permission allowlists and environment variables, not credentials.

Note: `serde_json::to_string_pretty` followed by `std::fs::write` is a truncate-and-rewrite operation, not an atomic rename. This is consistent with the pre-existing code and is the standard pattern in this codebase.

### 5. TOCTOU Race Conditions

**LOW (informational).** The read-modify-write cycle at lines 238-275 has a theoretical TOCTOU window: another process could modify `settings.json` between the `read_to_string` (line 239) and the `write` (line 263). However, as the spec correctly notes:

- Both competing writers would produce the correct `statusLine` shape
- The last writer wins, but the result is correct regardless of ordering
- This is a user-initiated CLI tool, not a daemon; concurrent invocations are unlikely
- File locking would add complexity disproportionate to the risk

This is not a security vulnerability in this context.

### 6. Command Injection / Shell Spawning

**PASS.** No `Command::new()`, `std::process`, or shell invocations are present in the changed code. The `statusline_value()` contains the string `"great statusline"` which is a command that Claude Code will execute, but this is a static string, not user-controlled.

### 7. Error Handling

**PASS.** All `std::fs` operations use `.context()` for actionable error messages. Invalid JSON is handled gracefully (line 269-273: warning printed, operation skipped). Non-object JSON values are handled by the `if let Some(obj)` guard (line 243). No `.unwrap()` in production code.

### 8. Supply Chain

**PASS.** No new dependencies introduced. The change uses only `serde_json` (already a dependency) and `std::fs` (standard library). No new crate imports.

### 9. Logic Correctness (Security-Relevant)

**PASS.** The repair logic correctly handles the following edge cases without data loss:

- Missing `statusLine` key: injects correct value, preserves all other keys
- Broken `statusLine` (missing `type`): replaces only `statusLine`, preserves all other keys
- Correct `statusLine`: no-op, file not rewritten
- `statusLine` is a non-object value (null, string, number): no-op (falls through to `else { false }`)
- Invalid JSON: warning printed, no write attempted
- File just created in the `else` branch above (lines 211-235): repair block enters but finds correct shape, `needs_write` is false -- no double-write

### 10. New-file creation path includes statusLine (line 228)

**PASS.** The new-file path now includes `"statusLine": statusline_value()` which produces the correct shape. Previously this path did not include a `statusLine` key at all (the injection block below was supposed to add it, but it used the broken shape).

Wait -- re-reading the diff more carefully. The diff from `main` shows the old code had:
```rust
"statusLine": {
    "command": "great statusline"
}
```
This was already in the new-file path but with the broken shape. The fix replaces it with `statusline_value()`. Confirmed correct.

---

## Findings Summary

| ID | Severity | Finding | Action |
|----|----------|---------|--------|
| -- | -- | No CRITICAL findings | -- |
| -- | -- | No HIGH findings | -- |
| L1 | LOW | Non-atomic write (truncate + rewrite) on settings.json | Informational; consistent with existing codebase pattern |
| L2 | LOW | TOCTOU window in read-modify-write | Informational; concurrent invocation produces correct result regardless |

---

## Verification

- `cargo test -- loop_cmd`: 15/15 passed
- `cargo clippy -- -D warnings`: clean (0 warnings in this crate)
- No `.unwrap()` in production code paths
- No shell spawning in changed code
- No user input flows into file paths or JSON values

---

## Verdict

**PASS.** No CRITICAL or HIGH findings. The changes are safe to commit. The code correctly fixes the schema bug using a single-source-of-truth helper function, handles all edge cases without data loss, and introduces no new attack surface.
