# Nightingale Selection — Iteration 008

**Selected task:** 0005 — `great doctor` Environment Diagnostician
**Priority:** P1
**Date:** 2026-02-24

## Rationale

**Dependencies are fully satisfied.** Tasks 0001 (platform detection), 0002 (config schema), 0003 (CLI infrastructure), and 0004 (status command) are all landed. There is no blocker.

**Refactor alignment with iteration 007 advisory.** Dijkstra flagged duplicated tool-iteration logic in the iteration 007 observer report. Requirement 3 of this task — extract `get_command_version()` to a single shared location imported by both `doctor.rs` and `status.rs` — is a direct, evidence-based response to that finding. This is the most structurally important change in the task.

**Natural progression from status.** The status command (0004) shows current environment state. Doctor diagnoses problems and advises fixes. Completing them in sequence means both commands share the same underlying utility layer, which is the correct final shape for the codebase.

**Enables downstream tasks.** Once `get_command_version()` is in a shared util, task 0006 (diff) can import it without introducing a third copy. Completing 0005 now avoids a larger consolidation effort later.

**Scoped and bounded.** Five acceptance criteria, all testable. The `--fix` scope is deliberately conservative (directory creation + PATH suggestions only); package installation is deferred to task 0009. No criteria were deferred out of this task that would leave it incomplete.

## Acceptance Criteria (from task file)

1. `cargo build` succeeds and `cargo clippy` produces zero warnings for `src/cli/doctor.rs`.
2. Integration tests pass: doctor runs successfully, `--fix` is accepted, git/cargo checks appear in output, no panic without config.
3. `get_command_version()` is defined in exactly one location and imported by both `doctor.rs` and `status.rs`.
4. When `great.toml` declares MCP servers, `great doctor` reports whether each server's command is available on PATH.
5. `great doctor` returns exit code 1 when any check fails, exit code 0 otherwise.

## What is NOT in scope

- `--json` output mode (deferred, noted in task 0005)
- Package installation via `--fix` (that is task 0009's domain)
- Making `DiagnosticResult` public across modules (deferred until a second consumer exists)
- Task 0014 tombstone cleanup (that task's own scope)
