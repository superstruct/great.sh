# 0040 -- Specification: `great status` exit code consistency (Option A)

| Field | Value |
|---|---|
| Task ID | 0040 |
| Title | `great status` exit code inconsistency between human and JSON modes |
| Type | refactor |
| Decision | Option A -- always exit 0 from both modes |
| Status | READY (spec complete) |
| Spec author | Lovelace |
| Date | 2026-03-04 |
| Estimated effort | 30 minutes |

## Summary

Remove the `std::process::exit(1)` call from the human-readable output path of
`great status`. After this change, both `great status` and `great status --json`
always exit 0 regardless of whether tools or secrets are missing. Issues are
communicated via output content (colored stderr lines in human mode; `has_issues`
and `issues` array in JSON mode), never via exit code.

This matches the convention of `git status` and `systemctl status`: exit 0 means
the command itself ran successfully; the output carries the diagnostic payload.

## Out of scope

- No changes to JSON schema (`StatusReport`, `has_issues`, `issues` array).
- No new CLI flags (no `--check`).
- No changes to what constitutes a "critical issue."
- No changes to `great doctor` or any other subcommand.
- No changes to `run_json` logic (it already exits 0).

## File changes

### 1. `src/cli/status.rs` -- remove `process::exit(1)` block

**Location:** Lines 274-279 in the current file (end of the `run` function).

**Remove this block entirely:**

```rust
    // NOTE: Intentional use of process::exit — the status command must print
    // its full report before exiting non-zero. Using bail!() would abort
    // mid-report, which is wrong for a diagnostic command.
    if has_critical_issues {
        std::process::exit(1);
    }
```

**Replace with this comment:**

```rust
    // Exit 0 regardless of issues found. The status command is informational:
    // missing tools/secrets are reported via colored output above, not via
    // exit code. This matches `great status --json` (which uses has_issues)
    // and the convention of git-status(1) and systemctl-status(1).
```

The `has_critical_issues` local variable is still set by the tool/secret checks
above (lines 182, 200, 264). It remains in the code -- it drives no logic after
this change, but removing it is a separate cleanup concern. The builder MAY
remove it and the assignments if they prefer a cleaner diff, but it is not
required. Either way is acceptable; both pass clippy (unused variable would
produce a warning, so if the builder removes the exit block, the `let mut
has_critical_issues` and its three assignment sites should also be removed to
avoid a clippy `unused_variable` warning).

**Recommended approach (cleaner):** Remove the `let mut has_critical_issues =
false;` declaration at line 129, remove the three `has_critical_issues = true;`
assignments at lines 182, 200, and 264, and remove the entire `if
has_critical_issues` block at lines 274-279. Replace lines 274-279 with the
comment above. This avoids any dead-code warnings.

### 2. `src/cli/status.rs` -- update doc comment on `run_json`

**Current (line 288):**

```rust
/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
```

**Replace with:**

```rust
/// Serialize full status report as JSON to stdout.
///
/// Both human and JSON modes always exit 0. Issues are signalled via the
/// `has_issues` field and `issues` array in the JSON payload.
```

### 3. `tests/cli_smoke.rs` -- update two existing tests, keep one unchanged

Three existing tests are affected by this change:

**Test `status_exit_code_nonzero_missing_tools` (line 1937):**

This test currently asserts `.failure()`. It must be updated to assert
`.success()` instead. The stderr assertion for "not installed" should be kept --
the output still reports missing tools.

**Current:**

```rust
#[test]
fn status_exit_code_nonzero_missing_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}
```

**Replace with:**

```rust
#[test]
fn status_exit_zero_even_with_missing_tools() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"
"#,
    )
    .unwrap();

    // Status is informational: exit 0 even when tools are missing.
    // Missing tools are reported via stderr output, not exit code.
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("not installed"));
}
```

**Test `status_exit_code_nonzero_missing_secrets` (line 1960):**

Same treatment -- flip `.failure()` to `.success()` and rename.

**Current:**

```rust
#[test]
fn status_exit_code_nonzero_missing_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing"));
}
```

**Replace with:**

```rust
#[test]
fn status_exit_zero_even_with_missing_secrets() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    // Status is informational: exit 0 even when secrets are missing.
    // Missing secrets are reported via stderr output, not exit code.
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stderr(predicate::str::contains("missing"));
}
```

**Test `status_json_always_exits_zero_even_with_issues` (line 1984):**

No changes needed. This test already asserts `.success()` and remains valid.

### 4. `tests/cli_smoke.rs` -- add a combined test

Add a new test that verifies both modes return exit 0 for the same environment
state. Place it after the existing `status_json_always_exits_zero_even_with_issues`
test.

```rust
#[test]
fn status_human_and_json_exit_codes_match() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("great.toml"),
        r#"
[project]
name = "test"

[tools.cli]
nonexistent-tool-xyz-9999 = "latest"

[secrets]
provider = "env"
required = ["GREAT_STATUS_TEST_NONEXISTENT_SECRET"]
"#,
    )
    .unwrap();

    // Human mode: exit 0 despite issues
    great()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success();

    // JSON mode: exit 0 despite issues (unchanged behaviour)
    great()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success();
}
```

## Build order

1. Edit `src/cli/status.rs` -- remove exit(1) block and dead variable, update comments.
2. Edit `tests/cli_smoke.rs` -- update two tests, add one new test.
3. Run `cargo clippy` -- confirm no warnings (especially no `unused_variable`).
4. Run `cargo test` -- confirm all tests pass, including the renamed/updated ones.

## Edge cases

| Scenario | Expected exit code | Notes |
|---|---|---|
| No `great.toml` found | 0 | Already works (line 165 prints warning, returns Ok) |
| Config parse error | 0 | Already works (line 111 prints error, continues with None) |
| All tools installed, all secrets set | 0 | No change from current behaviour |
| Missing tools, human mode | 0 | **Changed from 1 to 0** |
| Missing secrets, human mode | 0 | **Changed from 1 to 0** |
| Missing tools + secrets, JSON mode | 0 | No change from current behaviour |
| Non-UTF-8 config path | Error propagated via `?` | No change; anyhow error, exit non-zero |
| `--verbose` flag with missing tools | 0 | Verbose only adds detail, same exit logic |

## Error handling

The only exit-code change is removing the intentional `process::exit(1)`. All
other error propagation (anyhow `?` for config path issues, serde_json errors)
remains unchanged. Those represent command failures (could not run), not
diagnostic findings (ran successfully, found issues).

## Security considerations

None. This change affects only exit codes, not output content. No secrets are
exposed, no permissions change.

## Platform coverage

The change is pure Rust logic with no platform-specific code paths. The existing
tests run identically on macOS ARM64/x86_64, Ubuntu, and WSL2. No
platform-conditional behaviour is introduced or removed.

## Verification checklist

- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (all status tests green)
- [ ] `great status` in a directory with no `great.toml` exits 0
- [ ] `great status` in a directory with a config referencing missing tools exits 0
- [ ] `great status --json` behaviour is unchanged
- [ ] No `std::process::exit` calls remain in `status.rs`
