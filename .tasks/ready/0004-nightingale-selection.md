# Nightingale Selection — Task 0004

**Selected task:** 0004 — `great status` Environment State Reporter
**Date:** 2026-02-24
**Selected by:** Florence Nightingale (Requirements Curator)

---

## Selection Rationale

### Backlog state at selection time

| Task | Priority | Status | Blocked? |
|------|----------|--------|----------|
| 0004 | P1 | in-progress | No — 0001, 0002, 0003 all landed |
| 0005 | P1 | in-progress | No — but shares util-extraction concern with 0004 |
| 0006 | P1 | in-progress | No — but benefits from shared util landing in 0004/0005 first |
| 0007 | P1 | pending | No |
| 0008 | P1 | pending | Yes — needs 0007 |
| 0009 | P0 | pending | Yes — needs 0006, 0007, 0008 |
| 0010 | P0 | pending | Partially — GROUP A and GROUP J unblocked |
| 0014 | P3 | pending | No |
| 0015 | P2 | pending | No |

### Why 0004

1. All three declared dependencies (0001, 0002, 0003) are confirmed landed. No external blocks.
2. The implementation is already substantial — the gap is well-defined: expand JSON output, expand verbose mode, add integration tests, harden error paths, add exit-code semantics. This is completion work, not greenfield.
3. The `get_tool_version()` / `get_command_version()` duplication noted in 0004 and 0005 is best addressed by finishing 0004 first; 0005 can then extract the shared utility cleanly with a concrete implementation to work from rather than two diverging stubs.
4. Integration tests produced by 0004 establish the `assert_cmd` harness pattern that 0005 and 0006 will reuse.
5. `great status` is the primary read-only diagnostic surface — strengthening it unblocks CI usage (exit-code semantics) and gives all future apply/doctor work a reliable baseline to test against.

### Why not 0009 or 0010

0009 (apply) remains blocked by 0006, 0007, and 0008. The P0 GROUP A and GROUP J items in 0010 are unblocked, but the umbrella structure of 0010 makes partial execution risky without the underlying commands (0004–0008) first reaching a stable state.

---

## Scope Summary for Lovelace

Task 0004 requires the following concrete changes to `src/cli/status.rs` and the test suite:

1. **`run_json()` expansion** — serialize full status (tools, MCP, agents, secrets) using `serde_json`. Add `serde_json` to `Cargo.toml` if not already present.
2. **Verbose mode expansion** — in verbose mode, show full version strings for tools and full command paths for MCP servers.
3. **Integration tests** — four `assert_cmd` cases: no config, valid config, `--json` (parse output as JSON), `--verbose` (no error).
4. **Path UTF-8 error handling** — replace `.unwrap_or_default()` on path-to-str with proper propagation or a clear warning.
5. **Exit code semantics** — non-zero exit when critical issues detected (missing required secrets, declared tools absent); `--json` always exits 0.

Acceptance criteria are already defined in `.tasks/backlog/0004-status-command.md`. No criteria changes are needed — all five are testable and within scope.

---

## Admin note

Task 0003 (CLI infrastructure) has been confirmed landed and moved to `.tasks/done/0003-cli-infrastructure.md`. The backlog entry is tombstoned.
