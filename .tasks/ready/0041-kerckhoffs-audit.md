# Kerckhoffs Security Audit — Task 0041

**File**: `/home/isaac/src/sh.great/src/cli/mcp.rs`
**Change**: Line 183, `run_test()` function

## What Changed

```diff
-    if mcps.is_empty() {
+    if mcps.is_empty() && name.is_none() {
```

Control flow refinement: only warn "No MCP servers declared" if both conditions are true:
1. No MCP servers exist in config
2. User did not request a specific server by name

If user provides a name for a non-existent server, fall through to the error message at line 192.

## Security Assessment

**Data Sources**: `name` parameter comes from clap CLI argument parsing (line 29, optional String).

**Data Usage**:
- Line 183: used in condition check only (no I/O)
- Line 188-189: used as HashMap lookup key via `mcps.get_key_value(n)`
- Line 192: interpolated into user-facing error message via format macro

**Threat Analysis**:
- No credential handling
- No file path operations (HashMap key only, not path traversal)
- No command injection (name not passed to shell or exec)
- No privilege escalation or system calls
- No external network calls
- No memory safety violations (no `.unwrap()`, proper error handling)

**Verdict**: PASS — No security issues introduced. Change is safe control flow refinement with no new attack surface.

---

**Commit decision**: APPROVED. Safe to merge.
