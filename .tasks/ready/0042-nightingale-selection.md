# Nightingale Selection — Task 0042

**Selected task:** 0042 — `great status` Doctor Hint on Issues
**Date:** 2026-03-04
**Selected by:** Florence Nightingale (Requirements Curator)

## Selection Rationale

The backlog is otherwise empty. Task 0042 is the only candidate and is unblocked: its sole dependency (task 0040, status exit-code change) landed in iteration 038.

The task is XS in scope — a boolean accumulator and a single `println!` in `run()`, plus one integration test case. Risk of regression is low because JSON mode is entirely unaffected (it returns before reaching the human output path). The change closes a usability gap identified by Nielsen without reopening the exit-code debate settled in iteration 038.

## Scope Summary for Lovelace

Concrete changes required:

1. **`run()` in `src/cli/status.rs`** — introduce a `let mut has_issues = false;` accumulator. Set it to `true` inside each branch that currently calls `output::error()` for a missing tool, missing secret, or unavailable MCP command. After the final section, before the trailing `println!()`, print the hint when `has_issues` is true:
   ```
     Tip: use 'great doctor' for exit-code health checks in CI.
   ```
   No changes to `run_json()` — it is already isolated from the human output path.

2. **Integration test** — add two cases (or extend an existing test file):
   - With a `great.toml` that declares a tool known not to exist on the test host: assert stdout contains `Tip:`.
   - With no `great.toml` (or a clean config): assert stdout does not contain `Tip:`.

Acceptance criteria are defined in `.tasks/backlog/0042-status-doctor-hint.md`. All five are testable without environment assumptions beyond what the existing test suite already provides.
