# 0040 -- Humboldt Scout Report: `great status` exit code consistency

| Field | Value |
|---|---|
| Task ID | 0040 |
| Scout | Humboldt |
| Date | 2026-03-04 |
| Spec | `.tasks/ready/0040-status-exit-code-spec.md` |
| Socrates verdict | APPROVED (no blockers) |

## Summary

Small, precise refactor. One source file, one test file, four deletion sites plus
one comment replacement plus one doc comment update plus two test renames plus one
new test. All line numbers independently verified below.

---

## File 1: `src/cli/status.rs` (242 lines)

### Lines to delete (4 sites)

| Line | Content | Action |
|---|---|---|
| 129 | `let mut has_critical_issues = false;` | Delete |
| 182 | `has_critical_issues = true;` | Delete |
| 200 | `has_critical_issues = true;` | Delete |
| 264 | `has_critical_issues = true;` | Delete |

### Block to remove and replace (lines 274-279)

Current:
```rust
    // NOTE: Intentional use of process::exit — the status command must print
    // its full report before exiting non-zero. Using bail!() would abort
    // mid-report, which is wrong for a diagnostic command.
    if has_critical_issues {
        std::process::exit(1);
    }
```

Replace with:
```rust
    // Exit 0 regardless of issues found. The status command is informational:
    // missing tools/secrets are reported via colored output above, not via
    // exit code. This matches `great status --json` (which uses has_issues)
    // and the convention of git-status(1) and systemctl-status(1).
```

### Doc comment update (line 288)

Current:
```rust
/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
```

Replace with:
```rust
/// Serialize full status report as JSON to stdout.
///
/// Both human and JSON modes always exit 0. Issues are signalled via the
/// `has_issues` field and `issues` array in the JSON payload.
```

### Function signatures (unchanged, context only)

- `pub fn run(args: Args) -> Result<()>` — line 94
- `fn run_json(info: &platform::PlatformInfo, config_path: Option<&str>, config: Option<&config::GreatConfig>) -> Result<()>` — line 289

### No orphaned import

`std::process::exit` is called via fully qualified path at line 278. No `use`
statement exists for `std::process`. Nothing to clean up on the import side.

---

## File 2: `tests/cli_smoke.rs` (2000+ lines)

### Test `status_exit_code_nonzero_missing_tools` — line 1937

Rename to `status_exit_zero_even_with_missing_tools`.
Flip `.failure()` to `.success()`.
Keep `.stderr(predicate::str::contains("not installed"))`.

### Test `status_exit_code_nonzero_missing_secrets` — line 1960

Rename to `status_exit_zero_even_with_missing_secrets`.
Flip `.failure()` to `.success()`.
Keep `.stderr(predicate::str::contains("missing"))`.

### Test `status_json_always_exits_zero_even_with_issues` — line 1984

No changes. Already asserts `.success()`.

### New test — insert after line 2008 (after the JSON test closes)

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

### Existing status tests at lines 57-88 (unchanged)

- `status_shows_platform` (line 58) — asserts `.success()`, unaffected
- `status_warns_no_config` (line 69) — asserts `.success()`, unaffected
- `status_json_outputs_json` (line 80) — asserts `.success()`, unaffected
- `status_no_config_exits_zero` (line 2011) — asserts `.success()`, unaffected
- `status_verbose_with_config_shows_capabilities` (line 2021) — unaffected

---

## Dependency map

```
src/cli/status.rs
  ├── crate::cli::output          (unchanged)
  ├── crate::cli::util            (unchanged)
  ├── crate::config               (unchanged)
  └── crate::platform             (unchanged)
```

No new dependencies. No changes to public API. No callers of `run()` inspect
the return value beyond `?` propagation in `src/main.rs`.

---

## Out-of-scope process::exit calls (confirmed not touched)

| File | Line | Context |
|---|---|---|
| `src/cli/diff.rs` | 41 | "No great.toml found" error path |
| `src/cli/doctor.rs` | 267 | Diagnostic fail path, own contract |

Both are unrelated to task 0040.

---

## CI / docs scan

- `.github/workflows/*.yml` — no reference to `great status` exit codes
- `README.md` — mentions `great status` by name but no exit code contract
- No docs assert exit-code behavior for `great status`

No documentation changes needed beyond the inline code comment and the
`run_json` doc comment specified in the spec.

---

## Risks

None. This is a pure deletion with a comment replacement. The `Result<()>`
return type is unchanged; the function already returns `Ok(())` on all non-exit
paths. Clippy will enforce correctness: if any `has_critical_issues` assignment
is left behind, the compiler emits `unused_variable`; if the `if` block is left
but the variable removed, the compiler fails to build.

---

## Recommended build order

1. `src/cli/status.rs` — delete `has_critical_issues` declaration (line 129)
2. `src/cli/status.rs` — delete three assignment sites (lines 182, 200, 264)
3. `src/cli/status.rs` — replace process::exit block with comment (lines 274-279)
4. `src/cli/status.rs` — update `run_json` doc comment (line 288)
5. `tests/cli_smoke.rs` — rename + flip `status_exit_code_nonzero_missing_tools` (line 1937)
6. `tests/cli_smoke.rs` — rename + flip `status_exit_code_nonzero_missing_secrets` (line 1960)
7. `tests/cli_smoke.rs` — add new combined test after line 2008
8. `cargo clippy` — confirm zero warnings
9. `cargo test` — confirm all status tests green
