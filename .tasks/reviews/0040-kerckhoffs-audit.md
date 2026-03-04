# 0040 -- Security Audit: `great status` exit code consistency

| Field | Value |
|---|---|
| Task ID | 0040 |
| Auditor | Kerckhoffs |
| Date | 2026-03-04 |
| Verdict | **PASS** -- No CRITICAL or HIGH findings |
| Files reviewed | `src/cli/status.rs`, `tests/cli_smoke.rs`, `.tasks/ready/0040-status-exit-code-spec.md` |

## Scope

Pure deletion of `std::process::exit(1)` from the human-readable output path of `great status`, plus removal of the dead `has_critical_issues` variable and its three assignment sites. Test updates to flip `.failure()` to `.success()`, one new combined test.

## Checklist

### 1. Credential leakage

**PASS.** Secret values are never captured or printed.

- Human mode (line 257): `std::env::var(key).is_ok()` -- checks existence only, discards value.
- JSON mode (line 376): Same pattern -- `is_ok()` on `env::var`, value never serialized.
- `SecretStatus` struct contains only `name: String` and `is_set: bool` -- no value field.
- The change does not alter any output paths; it only removes the exit code branching.

### 2. Input validation / attack surface

**PASS.** No new attack surface.

- No new CLI arguments, flags, or user-controlled inputs are introduced.
- No new `Command::new()` calls or shell invocations.
- The change is strictly a deletion of control flow; it does not add code paths.
- Tool names from `great.toml` still go through `command_exists()` (the `which` crate) which does safe PATH lookup without shell spawning.

### 3. Error propagation

**PASS.** No sensitive data in error messages.

- The only `?` propagation in `run()` is the non-UTF-8 config path check (line 100-104), which uses `path.display()` -- this shows the filesystem path, not credentials.
- Config parse errors (line 111) print the parse error message, which could include TOML fragment text but not secret values (secrets are env vars, not TOML values).
- `run_json()` uses `serde_json::to_string_pretty(&report)?` -- the `StatusReport` struct contains no secret values (see credential leakage section above).

### 4. Supply chain

**PASS.** No dependency changes.

- `Cargo.toml` and `Cargo.lock` are unmodified (verified via `git diff`).
- No new crates introduced.

### 5. `process::exit` removal safety

**PASS.** Removing `process::exit(1)` is safe.

- `process::exit(1)` bypasses Drop cleanup, but this was at the end of `run()` after all output was flushed. Replacing it with `Ok(())` is strictly safer (allows normal stack unwinding).
- The `has_critical_issues` variable and its three assignment sites were cleanly removed -- no dead code or clippy warnings expected.
- JSON mode already returned `Ok(())` unconditionally; human mode now matches.

## Findings

### L1 (LOW): Pre-existing -- TOML parse errors may echo config content fragments

Line 111: `output::error(&format!("Failed to parse config: {}", e))` may include TOML content snippets in error messages. This is pre-existing and not introduced by this change. Tracked in prior audit 0004-L1.

## Conclusion

The change is a pure deletion with no new code paths, no new dependencies, and no credential exposure. The audit finds no CRITICAL or HIGH issues. The commit is not blocked.
