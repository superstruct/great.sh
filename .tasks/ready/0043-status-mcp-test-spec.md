# 0043 -- Status MCP Test Coverage and JSON Bug Fix

**Author:** Ada Lovelace (Spec Writer)
**Date:** 2026-03-04
**Task:** `.tasks/backlog/0043-status-mcp-test-coverage.md`
**Complexity:** XS (one bug fix, two test functions)

## Summary

Add integration tests for the MCP-unavailable path in `great status` (both
human-readable and JSON modes), and fix a latent bug where `run_json()` never
pushes to the `issues` vector when an MCP server command is missing from PATH.

## Files to Modify

| File | Change |
|---|---|
| `src/cli/status.rs` | Fix `run_json()` MCP block: restructure closure into explicit `if-let` and push to `issues` when `!command_available` |
| `tests/cli_smoke.rs` | Add two test functions: `status_mcp_missing_command_shows_not_found`, `status_json_mcp_missing_sets_has_issues` |

No new files created. No dependencies added.

## Part 1: Bug Fix in `src/cli/status.rs`

### Current Code (lines 357--369)

```rust
let mcp = config.and_then(|cfg| {
    cfg.mcp.as_ref().map(|mcps| {
        mcps.iter()
            .map(|(name, m)| McpStatus {
                name: name.clone(),
                command: m.command.clone(),
                args: m.args.clone(),
                command_available: command_exists(&m.command),
                transport: m.transport.clone(),
            })
            .collect()
    })
});
```

### Problem

The MCP block uses `and_then` / `map` / `iter().map()` -- a chain of
immutable closures. This means `issues` (a `&mut Vec<String>`) cannot be
borrowed inside the closure. The `McpStatus` structs are built with
`command_available: false` but the `issues` vector is never appended to,
so `has_issues` stays `false` in the final JSON even when an MCP server
command is missing.

By contrast, the tools block (lines 311--355) and the secrets block
(lines 383--408) both use explicit `if-let` patterns that allow mutable
access to `issues`.

### Required Change

Replace the closure chain with an explicit `if-let` block, matching the
pattern already used for tools. The replacement code:

```rust
let mcp = if let Some(cfg) = config {
    if let Some(mcps) = cfg.mcp.as_ref() {
        let mut result = Vec::new();
        for (name, m) in mcps {
            let available = command_exists(&m.command);
            if !available {
                issues.push(format!(
                    "MCP server '{}' command '{}' not found",
                    name, m.command
                ));
            }
            result.push(McpStatus {
                name: name.clone(),
                command: m.command.clone(),
                args: m.args.clone(),
                command_available: available,
                transport: m.transport.clone(),
            });
        }
        Some(result)
    } else {
        None
    }
} else {
    None
};
```

### Exact edit location

Replace lines 357--369 of `src/cli/status.rs` (the entire `let mcp = config.and_then(...)` block) with the code above. The surrounding code (the `let tools = ...` block ending at line 355, and the `let agents = ...` block starting at line 371) is unchanged.

### Behavioral contract after fix

- When an MCP server's command is not on PATH, `issues` receives a string
  of the form `"MCP server '<name>' command '<command>' not found"`.
- `has_issues` (derived from `!issues.is_empty()` at line 416) becomes
  `true`.
- The `mcp` array in JSON output still contains the `McpStatus` entry with
  `command_available: false` -- no change to the MCP array shape.

## Part 2: Integration Tests in `tests/cli_smoke.rs`

Both tests go in the `// Status` section of the file. Insert them after
the existing `status_human_and_json_exit_codes_match` test (currently the
last status test, ending at line 2048).

### Test 1: `status_mcp_missing_command_shows_not_found`

```rust
#[test]
fn status_mcp_missing_command_shows_not_found() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.fake-server]
command = "nonexistent_mcp_status_xyz_9999"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("not found"))
        .stderr(predicate::str::contains("great doctor"));
}
```

**What it verifies:**
1. Exit code is 0 (`.success()`).
2. stderr contains "not found" -- the MCP unavailable message from line 253
   of `run()`: `output::error(&format!("  {} ({} -- not found)", ...))`.
3. stderr contains "great doctor" -- the hint from line 279 of `run()`,
   printed when `has_issues` is true.

**Why the command name:** `nonexistent_mcp_status_xyz_9999` is deliberately
absurd to guarantee it does not exist on any CI runner or developer machine.

### Test 2: `status_json_mcp_missing_sets_has_issues`

```rust
#[test]
fn status_json_mcp_missing_sets_has_issues() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[mcp.fake-server]
command = "nonexistent_mcp_status_xyz_9999"
"#,
    )
    .unwrap();

    let output = great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");

    // MCP array should exist and contain our server
    let mcp = parsed.get("mcp").unwrap().as_array().unwrap();
    assert!(mcp.iter().any(|s| s["name"] == "fake-server"));
    assert!(mcp.iter().any(|s| s["command_available"] == false));

    // has_issues must be true (this is the bug that Part 1 fixes)
    assert_eq!(parsed["has_issues"], true);

    // issues array must mention the server
    let issues = parsed["issues"].as_array().unwrap();
    assert!(issues.iter().any(|i| {
        let s = i.as_str().unwrap_or("");
        s.contains("fake-server") && s.contains("not found")
    }));
}
```

**What it verifies:**
1. Exit code is 0.
2. The `mcp` array contains the declared server with `command_available: false`.
3. `has_issues` is `true` (this assertion fails without the Part 1 fix).
4. The `issues` array contains a string mentioning both the server name and
   "not found".

## Build Order

1. Apply the bug fix in `src/cli/status.rs` (Part 1).
2. Add the two tests to `tests/cli_smoke.rs` (Part 2).
3. Run `cargo test status_mcp_missing_command_shows_not_found status_json_mcp_missing_sets_has_issues` to confirm both pass.
4. Run `cargo test` to confirm no regressions.
5. Run `cargo clippy` to confirm no warnings.

## Edge Cases

| Scenario | Expected behavior |
|---|---|
| No `[mcp]` section in config | MCP block returns `None`; no issues pushed; tests do not exercise this path (already covered by `status_json_no_config_still_valid`) |
| MCP command exists on PATH | `command_available: true`; no issue pushed; out of scope for these tests |
| Multiple MCP servers, some missing | Each missing server gets its own `issues.push()`; `has_issues` is `true` |
| Empty MCP command string `""` | `command_exists("")` returns `false`; treated as missing; issue pushed |

## Error Handling

No new error types or error paths. The fix adds an `issues.push()` call
inside an existing code path. The command always exits 0.

## Security Considerations

None. The tests use intentionally nonexistent command names. No secrets,
network calls, or filesystem writes beyond the temp directory.

## Platform Compatibility

All three target platforms (macOS ARM64/x86_64, Ubuntu, WSL2) are covered.
The `command_exists()` function already handles platform differences
internally. The nonexistent command name `nonexistent_mcp_status_xyz_9999`
will not resolve on any platform.

## Testing Strategy

| Test | Mode | Asserts |
|---|---|---|
| `status_mcp_missing_command_shows_not_found` | Human-readable | exit 0, stderr "not found", stderr "great doctor" |
| `status_json_mcp_missing_sets_has_issues` | JSON | exit 0, mcp array present, `command_available: false`, `has_issues: true`, issues array mentions server |

Both tests are deterministic, require no network, no env vars, and no
installed tools beyond the `great` binary itself.
