# 0040 — `great status` exit code inconsistency between human and JSON modes

| Field | Value |
|---|---|
| Priority | P3 |
| Type | refactor |
| Module | `src/cli/status.rs` |
| Status | backlog |
| Estimated Complexity | Small |

## Context

`great status` exits 1 when any tool or secret is missing. `great status --json`
always exits 0, signalling issues via the `has_issues` boolean field in the JSON
payload. This split was introduced deliberately: the human-readable path uses
`std::process::exit(1)` with an explanatory comment; the JSON path documents
"Always returns Ok (exit 0)."

The two modes therefore behave differently for identical underlying state. A fresh
environment where tools are not yet installed will cause `great status` to exit 1,
but `great status --json` to exit 0. This violates the principle of least surprise
and makes the command unreliable in automation.

Verified in Ubuntu 24.04 Docker container. The inconsistency is not a crash — it
is a design decision that was never fully resolved.

### Observed behaviour

```
great status          # exits 1 if any tool or secret missing
great status --json   # always exits 0 regardless of has_issues value
```

### Why this matters

- `great status && echo "OK"` fails in any fresh environment, so the command
  cannot be used as a readiness check in scripts or CI pipelines without the
  `--json` workaround.
- Convention among status-style commands (`git status`, `systemctl status`) is
  exit 0 = command ran successfully; exit non-zero = command itself failed.
  Reporting "things are missing" is not a command failure.
- The JSON mode implicitly sets the correct convention (exit 0, data carries the
  signal) but leaves human mode doing the opposite.

### Current code reference

`src/cli/status.rs` lines 274–279 (human path):

```rust
// NOTE: Intentional use of process::exit — the status command must print
// its full report before exiting non-zero. Using bail!() would abort
// mid-report, which is wrong for a diagnostic command.
if has_critical_issues {
    std::process::exit(1);
}
```

`src/cli/status.rs` line 288 (JSON path):

```rust
/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
```

## Decision required

Three options are on the table. The team must pick one before implementation
begins. No implementation is in scope for this task.

| Option | Human mode | JSON mode | Notes |
|---|---|---|---|
| A | exit 0 always | exit 0 always | `status` = informational; issues visible via output only |
| B | exit 1 on issues | exit 1 when `has_issues: true` | consistent but breaks scripting convention |
| C | exit 0 always; add `--check` flag that exits 1 on issues | exit 0 always | explicit opt-in for scripting; no surprises |

Option A or C align with how `git status` and `systemctl status` behave. Option B
achieves consistency but entrenches a pattern that is already the source of the
complaint.

## Acceptance Criteria

1. A single exit-code contract is documented and applies identically to both
   `great status` and `great status --json` for the same environment state.
2. `great status` exits 0 in a fresh environment where tools are not yet
   installed (regardless of which option is chosen, the current exit-1-on-missing
   behaviour in human mode is the thing under review).
3. If Option C is chosen, `great status --check` exits 1 when any critical issue
   is present and exits 0 when none are.
4. The code comment at `status.rs:274` and the doc comment at `status.rs:288` are
   updated to reflect the chosen contract.
5. At least one integration test in `tests/` covers the exit code of `great status`
   in a simulated environment with a missing tool.

## Files That Need to Change

- `src/cli/status.rs` — exit logic in `run_human` and `run_json`; option C adds
  a `--check` flag to the `Args` struct
- `tests/` — new or updated integration test for exit-code behaviour

## Dependencies

None. Standalone decision and implementation.

## Out of Scope

- Changes to the JSON schema itself (`has_issues`, `issues` array)
- Any change to what constitutes a "critical issue"
- The `doctor` subcommand (separate command, separate exit-code contract)
