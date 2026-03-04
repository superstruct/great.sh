# 0042: `great status` — Doctor Hint on Issues

**Priority:** P2
**Type:** enhancement
**Module:** `src/cli/status.rs`
**Status:** backlog
**Estimated Complexity:** XS

## Context

In iteration 038, `great status` was changed to always exit 0, matching the `git status` convention. The decision was sound, but it removed a signal that some users relied on for CI exit-code gating. Nielsen (iteration 038 UX review) noted that nothing in the human-readable output guides those users toward `great doctor`, which *does* return a non-zero exit code when issues are found and is the correct tool for CI health checks.

The fix is minimal: append a single hint line to the human-readable report whenever at least one issue is detected (missing tool, missing secret, or unavailable MCP command). The hint must not appear in JSON output and must not appear when the environment is clean.

The natural insertion point is just before the final `println!()` at the end of `run()` in `src/cli/status.rs`. The function already traverses tools, MCP servers, and secrets; a boolean accumulator tracking whether any issue was found is sufficient.

## Acceptance Criteria

1. When any tool or secret is missing (or any declared MCP command is unavailable), `great status` prints a hint line after the report: `  Tip: use 'great doctor' for exit-code health checks in CI.`
2. The hint does not appear when all tools, secrets, and MCP commands are in a healthy state.
3. `great status --json` is unaffected — the hint line is never emitted in JSON mode (JSON mode already exits before reaching the human output path).
4. An integration test asserts hint presence when a declared tool is absent and hint absence when the environment is clean (or no `great.toml` is present with no issues).
5. `cargo clippy` produces zero new warnings; no existing tests regress.

## Files That Need to Change

- `src/cli/status.rs` — add `has_issues` accumulator to `run()` and print hint when true
- `tests/` — extend or add an integration test covering hint presence/absence

## Dependencies

- Task 0040 (status exit-code change) — already landed (iteration 038)

## Out of Scope

- Changing the exit code of `great status` (that is intentionally 0; task 0040 settled this)
- Modifying `great doctor` itself
- Adding hint text to `--json` output (the `issues` array already serves that purpose)
