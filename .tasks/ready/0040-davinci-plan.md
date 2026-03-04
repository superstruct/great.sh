# 0040 -- Da Vinci Implementation Plan

## Summary

Remove `process::exit(1)` from `great status` human mode so both human and JSON modes always exit 0. Issues are communicated via output content, not exit code.

## Edits

### 1. `src/cli/status.rs` -- remove dead variable and exit block

- **Delete** `let mut has_critical_issues = false;` (line 129)
- **Delete** `has_critical_issues = true;` at line 182 (runtime tools not installed)
- **Delete** `has_critical_issues = true;` at line 200 (CLI tools not installed)
- **Delete** `has_critical_issues = true;` at line 264 (secrets missing)
- **Replace** lines 274-279 (the `if has_critical_issues { std::process::exit(1); }` block and its comment) with:

```rust
    // Exit 0 regardless of issues found. The status command is informational:
    // missing tools/secrets are reported via colored output above, not via
    // exit code. This matches `great status --json` (which uses has_issues)
    // and the convention of git-status(1) and systemctl-status(1).
```

### 2. `src/cli/status.rs` -- update `run_json` doc comment (line 288)

Replace:
```rust
/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
```

With:
```rust
/// Serialize full status report as JSON to stdout.
///
/// Both human and JSON modes always exit 0. Issues are signalled via the
/// `has_issues` field and `issues` array in the JSON payload.
```

### 3. `tests/cli_smoke.rs` -- rename and flip two tests

- `status_exit_code_nonzero_missing_tools` (line 1937) -> `status_exit_zero_even_with_missing_tools`, change `.failure()` to `.success()`, add explanatory comment
- `status_exit_code_nonzero_missing_secrets` (line 1960) -> `status_exit_zero_even_with_missing_secrets`, change `.failure()` to `.success()`, add explanatory comment

### 4. `tests/cli_smoke.rs` -- add new combined test

Insert `status_human_and_json_exit_codes_match` after the existing `status_json_always_exits_zero_even_with_issues` test. This test creates a config with both missing tools and missing secrets, then asserts both human mode and `--json` mode exit 0.

## Quality gates

- `cargo clippy` -- zero warnings
- `cargo test` -- all tests pass

## Notes

- No new dependencies
- No changes to JSON schema or `run_json` logic
- No platform-specific code affected
- `std::process::exit` was called via fully qualified path; no import cleanup needed
