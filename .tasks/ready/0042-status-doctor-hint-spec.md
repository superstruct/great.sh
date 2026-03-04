# Spec 0042: `great status` Doctor Hint on Issues

**Task:** `.tasks/backlog/0042-status-doctor-hint.md`
**Author:** Ada Lovelace (Spec Writer)
**Date:** 2026-03-04
**Complexity:** XS

## Summary

When `great status` (human mode) detects missing tools, unavailable MCP commands, or missing secrets, it should print a single hint line directing users to `great doctor` for CI exit-code gating. The hint must not appear in JSON mode or when the environment is clean.

## Files to Modify

| File | Change |
|---|---|
| `src/cli/status.rs` | Add `has_issues` accumulator; print hint before `Ok(())` |
| `tests/cli_smoke.rs` | Update 2 tests, add 1 new test |

No new files are created. No dependencies are added.

## Interface Changes

None. No public API, struct, or type signature changes. The only observable change is a new stderr line in human-readable output.

## Implementation Approach

### 1. `src/cli/status.rs` -- `run()` function

**Step 1: Declare the accumulator.**

Insert immediately after the `// -- Human-readable mode` comment (line 125), before `output::header("great status")`:

```rust
let mut has_issues = false;
```

**Step 2: Set `has_issues = true` at each issue site.**

There are exactly three locations in the human-readable path where issues are detected. Each requires a single `has_issues = true;` statement:

(a) **Missing tool** -- inside `print_tool_status()` when `installed` is `false`. Rather than threading a mutable reference through the helper, set `has_issues` at the call sites in `run()`. Both tool loops (runtimes and cli) already call `command_exists(name)` and bind the result to `installed`. After each call to `print_tool_status(...)`, add:

```rust
if !installed {
    has_issues = true;
}
```

Concretely, this means two insertion points inside the `if let Some(tools) = &cfg.tools` block:
- After the `print_tool_status(...)` call in the runtimes loop (around current line 188).
- After the `print_tool_status(...)` call in the cli tools loop (around current line 205).

(b) **Unavailable MCP command** -- in the MCP section, the `else` branch at line 246 calls `output::error(...)` when `!cmd_available`. Add `has_issues = true;` inside that branch:

```rust
} else {
    output::error(&format!("  {} ({} -- not found)", name, mcp.command));
    has_issues = true;
}
```

(c) **Missing secret** -- in the secrets section, the `else` branch at line 260 calls `output::error(...)` when the env var is not set. Add `has_issues = true;` inside that branch:

```rust
} else {
    output::error(&format!("  {} -- missing", key));
    has_issues = true;
}
```

**Step 3: Print the hint.**

After the final `println!()` (current line 267) and before `Ok(())` (current line 274), insert:

```rust
if has_issues {
    output::info("Tip: use 'great doctor' for exit-code health checks in CI.");
}
```

This uses `output::info()` which writes to stderr with a blue info prefix. The hint is a single line, easy to spot but not alarming.

### 2. `run_json()` -- no changes

JSON mode returns at line 122 before the human-readable path. The `has_issues` accumulator lives entirely within the human branch. No changes to `run_json()`.

### 3. `tests/cli_smoke.rs`

**Build order: implement `status.rs` changes first, then update tests.**

**(a) Update `status_exit_zero_even_with_missing_tools` (line 1937).**

Add a second stderr assertion after the existing one:

```rust
great()
    .current_dir(dir.path())
    .arg("status")
    .assert()
    .success()
    .stderr(predicate::str::contains("not installed"))
    .stderr(predicate::str::contains("great doctor"));
```

**(b) Update `status_exit_zero_even_with_missing_secrets` (line 1962).**

Add a second stderr assertion after the existing one:

```rust
great()
    .current_dir(dir.path())
    .arg("status")
    .assert()
    .success()
    .stderr(predicate::str::contains("missing"))
    .stderr(predicate::str::contains("great doctor"));
```

**(c) Add new test `status_no_doctor_hint_when_clean`.**

Insert after `status_no_config_exits_zero` (after line 2056). This test uses a config with only a `[project]` section (no tools, no secrets, no MCP) so nothing triggers `has_issues`:

```rust
#[test]
fn status_no_doctor_hint_when_clean() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();

    // Clean config: no tools, secrets, or MCP declared -- no hint expected.
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("great doctor").not());
}
```

Note: `predicate::str::contains(...).not()` is provided by the `predicates` crate already in the test dependencies.

## Edge Cases

| Scenario | Expected Behavior |
|---|---|
| No `great.toml` found | No hint. The warning "No great.toml found" is informational, not an issue in the tools/secrets/MCP sense. `has_issues` stays `false`. |
| Config parse error | No hint. The config is `None`, so the tools/secrets/MCP sections are skipped entirely. `has_issues` stays `false`. |
| `--json` flag | No hint. JSON path returns before the human-readable block where `has_issues` lives. |
| All tools installed, all secrets set | No hint. `has_issues` stays `false`. |
| Mixed: some tools OK, one missing | Hint appears. Any single `true` assignment triggers it. |
| MCP command missing but tools and secrets OK | Hint appears. |
| `--verbose` flag | No effect on hint logic. Hint appears or not based solely on `has_issues`. |

## Error Handling

No new error paths. The hint is printed via `output::info()` which writes to stderr. If stderr is closed (broken pipe), the existing SIGPIPE handler (task 0039) handles it gracefully.

## Security Considerations

None. The hint is a static string containing no user data, secrets, or file paths.

## Testing Strategy

| Test | Assertion | Coverage |
|---|---|---|
| `status_exit_zero_even_with_missing_tools` | stderr contains "great doctor" | Missing tool triggers hint |
| `status_exit_zero_even_with_missing_secrets` | stderr contains "great doctor" | Missing secret triggers hint |
| `status_no_doctor_hint_when_clean` | stderr does NOT contain "great doctor" | Clean config suppresses hint |
| `status_json_always_exits_zero_even_with_issues` | (existing, unchanged) | JSON mode unaffected |
| `status_no_config_exits_zero` | (existing, unchanged) | No config = no hint (implicit) |

All tests run on all platforms (macOS ARM64/x86_64, Ubuntu, WSL2) via `cargo test`. No platform-specific behavior in this change.

## Verification Commands

```bash
cargo clippy -- -D warnings    # Zero new warnings
cargo test                     # All tests pass including the three modified/added ones
cargo run -- status            # Visual check: hint appears when issues exist
```
