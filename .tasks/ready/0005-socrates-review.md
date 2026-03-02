# 0005: `great doctor` Command -- Socrates Review

**Spec:** `/home/isaac/src/sh.great/.tasks/ready/0005-doctor-command-spec.md`
**Task:** `/home/isaac/src/sh.great/.tasks/backlog/0005-doctor-command.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-24

---

## VERDICT: APPROVED

---

## BLOCKING Concerns

None.

---

## ADVISORY Concerns

### 1. Existing tests `doctor_checks_system_prerequisites` and `doctor_checks_docker` assert `.success()` but are not updated

```
{
  "gap": "The spec only updates `doctor_runs_diagnostics` to remove `.success()`, but two other existing tests -- `doctor_checks_system_prerequisites` (line 305) and `doctor_checks_docker` (line 316) -- also assert `.success()`. After `process::exit(1)` is added, these tests will fail on any machine where ANY doctor check fails (e.g., missing Homebrew, missing Docker), not just the check they are testing.",
  "question": "Should the builder also remove `.success()` from `doctor_checks_system_prerequisites` and `doctor_checks_docker`, or is there a guarantee that these tests always run in environments where all checks pass?",
  "severity": "ADVISORY",
  "recommendation": "Add these two tests to the 'Update existing tests' section with the same treatment: remove `.success()` and assert only stderr content. The builder will encounter compile-time success but runtime test failures on CI if this is not addressed."
}
```

**Evidence:** `/home/isaac/src/sh.great/tests/cli_smoke.rs` lines 298-318 -- both tests assert `.success()`. The spec (Section 4e) only addresses the `doctor_runs_diagnostics` test at line 95.

### 2. `doctor_fix_runs_without_crash` also asserts `.success()` and will break under `--fix`

```
{
  "gap": "The `doctor_fix_runs_without_crash` test (line 109, currently `#[ignore]`) asserts `.success()`. The spec notes in Section 6 that `--fix` still exits 1 if failures were detected (counter reflects pre-fix state). If this test is ever un-ignored, it will fail.",
  "question": "Should the spec explicitly note that this ignored test's `.success()` assertion must also be updated if/when it is un-ignored?",
  "severity": "ADVISORY",
  "recommendation": "Add a note in the spec about this test. Since it is `#[ignore]`, it will not cause CI failures now, but a future maintainer removing the ignore will be surprised."
}
```

### 3. Double validation: `config::load` already calls `validate()` and bails on errors

```
{
  "gap": "The spec's `check_config` calls `config::load(Some(path_str))` which internally calls `cfg.validate()` and bails on any `ConfigMessage::Error`. Then `check_config` calls `cfg.validate()` again. Since errors already caused `config::load` to bail, the `ConfigMessage::Error` branch in `check_config` is dead code -- it can never be reached.",
  "question": "Is the double validation intentional (defensive programming) or an oversight? Should `check_config` use a lower-level parse function that does not validate, allowing doctor to report validation errors in its own format?",
  "severity": "ADVISORY",
  "recommendation": "This is an existing pattern (the current `check_config` already double-validates). Consider adding a comment noting the redundancy, or refactoring `config::load` to accept a `validate: bool` parameter. Not blocking because the behavior is correct, just redundant."
}
```

**Evidence:** `/home/isaac/src/sh.great/src/config/mod.rs` lines 28-38 -- `load()` calls `validate()` and bails on errors before returning `Ok(config)`.

### 4. Test count claim: "All 8 integration tests pass" includes 1 `#[ignore]` test

```
{
  "gap": "The spec claims '4 existing + 4 new = 8' tests. But `doctor_fix_runs_without_crash` has `#[ignore]` and does not run under `cargo test`. Only 7 tests actually execute.",
  "question": "Should the acceptance criterion say '7 tests pass (3 active existing + 4 new)' or is the intent to count all defined tests regardless of ignore status?",
  "severity": "ADVISORY",
  "recommendation": "Clarify the count to avoid confusion during verification. The builder might waste time looking for a missing 8th test."
}
```

### 5. Minor line number discrepancy in status.rs call sites

```
{
  "gap": "The spec claims `get_tool_version(name)` appears at lines 182, 199, 320, 335 in status.rs. Actual positions are lines 182, 200, 320, 336 -- two are off by one.",
  "question": "Will the builder be confused by the line number discrepancy?",
  "severity": "ADVISORY",
  "recommendation": "Not a functional issue since the spec correctly instructs using `replace_all` on the string pattern `get_tool_version(name)`. The line numbers are context only. No action needed unless the builder uses line-based editing."
}
```

### 6. Task requirement 2 (auto-fix) is already implemented but spec should acknowledge this explicitly

```
{
  "gap": "Task requirement 2 says 'Implement basic auto-fix for --fix mode'. The spec summary correctly states 'The --fix auto-fix logic is already fully implemented and must not be rewritten.' However, the task description says '--fix flag is declared but auto-fix logic is not implemented'. This contradiction between task description and current code state could confuse reviewers.",
  "question": "Was the auto-fix logic implemented in a prior iteration that postdates the task description? Should the task file be updated to reflect current state?",
  "severity": "ADVISORY",
  "recommendation": "The spec handles this correctly by preserving the existing auto-fix code verbatim. The task description appears stale. No action needed in the spec."
}
```

**Evidence:** `/home/isaac/src/sh.great/src/cli/doctor.rs` lines 99-212 -- full auto-fix implementation is already present with 8 FixAction variants.

### 7. `process::exit(1)` bypasses destructors and cleanup

```
{
  "gap": "The spec uses `std::process::exit(1)` which is an abrupt termination that skips Drop impls and buffer flushes. While this pattern exists in `status.rs` and the spec includes a justifying comment, there is an alternative: returning a custom error type that main.rs maps to exit code 1.",
  "question": "Has the alternative of returning a typed error (e.g., `GreatError::DiagnosticFailure`) from `run()` and letting `main.rs` map it to exit code 1 been considered?",
  "severity": "ADVISORY",
  "recommendation": "This follows the established `status.rs` precedent and the comment explains the rationale. The risk is minimal since the command has already flushed all output. No action needed."
}
```

**Evidence:** `/home/isaac/src/sh.great/src/cli/status.rs` lines 283-288 -- identical pattern with identical comment.

---

## Requirement Coverage Verification

| Task Requirement | Spec Section | Covered? |
|---|---|---|
| 1. Add integration tests | Section 4 (4 new tests + 1 modified) | Yes |
| 2. Implement basic auto-fix | Preserved from existing code (acknowledged in summary) | Yes (pre-existing) |
| 3. Extract `get_command_version()` to shared utility | Section 1 (util.rs), Section 2e, Section 3 | Yes |
| 4. Add MCP server checks | Section 2d (`check_mcp_servers`) | Yes |
| 5. Validate exit code semantics | Section 2b (`process::exit(1)` block) | Yes |
| Bonus: Fix `unwrap_or_default()` path safety | Section 2c (match on `path.to_str()`) | Yes |

---

## Code Correctness Observations

1. **Will compile:** The imports are correct. `util` is added to the `crate::cli` import group. `command_exists` is already in scope via `use crate::platform::{self, command_exists, ...}`. The `check_mcp_servers` function correctly accesses `mcp.command`, `mcp.enabled`, and `mcp.transport` which all exist on `McpConfig` (verified in `/home/isaac/src/sh.great/src/config/schema.rs` lines 93-108).

2. **No `.unwrap()` in production:** The spec replaces `unwrap_or_default()` with an explicit match. The `unwrap_or("")` in `get_command_version` (on `.lines().next()`) is safe since it operates on a known-good string, and returns `""` which is then caught by the `is_empty()` check.

3. **Serde compatibility:** Not applicable -- no serialization changes.

4. **Downstream callers of `check_config`:** The function is private (`fn check_config`, not `pub fn`), called only from `run()` in the same file. The return type change from `()` to `Option<config::GreatConfig>` only affects this one call site, which the spec updates.

5. **Build order is sound:** Each step is independently compilable. Step 1 (create util.rs) may trigger a dead_code warning but not an error. Steps 2-3 eliminate the warning by importing the function.

---

## Summary

A well-structured spec that addresses all 5 task requirements with accurate line references, correct Rust code, and a sensible build order. The primary risk is 2 additional existing tests (`doctor_checks_system_prerequisites`, `doctor_checks_docker`) that assert `.success()` and will break on environments where doctor checks fail after exit-code semantics are added -- the builder should address these alongside the `doctor_runs_diagnostics` fix documented in section 4e.
