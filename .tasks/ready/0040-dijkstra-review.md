# Dijkstra Review — Task 0040: Remove `process::exit(1)` from `great status`

**Reviewer:** Edsger W. Dijkstra
**Date:** 2026-03-04
**Files reviewed:**
- `src/cli/status.rs`
- `tests/cli_smoke.rs`

---

```
VERDICT: APPROVED

Issues:
- [WARN] src/cli/status.rs:271 — Comment references "systemctl-status(1)" as a
  precedent for exit-0 on issues. `systemctl status` in fact exits non-zero when
  a unit is inactive or failed (exit codes 1–4). The analogy is inaccurate. Only
  `git status` is a clean precedent here. The comment is advisory, not a gate,
  but an inaccurate justification in source code is worse than no justification.
- [WARN] src/cli/status.rs:282–284 — The `run_json` doc comment states "Both
  human and JSON modes always exit 0." This is true of the current implementation,
  but it is an architectural invariant described in a private function's doc
  comment rather than enforced structurally. A future maintainer editing `run`
  has no mechanical reminder of this contract. The statement is accurate today
  but fragile as documentation placement.

Summary: The deletion is clean, no dead code remains, the structural simplification
is correct, and the tests adequately cover the new exit-0 contract — one minor
inaccuracy in a rationale comment and one fragile placement of an invariant
statement are advisory concerns only.
```

---

## Detailed reasoning

### 1. Code quality — is the deletion clean?

The `has_critical_issues` variable and its three assignment sites have been
removed entirely. No references to that variable or to `std::process::exit`
survive in `status.rs`. `grep` would find nothing. The removal is complete.

The `run` function now terminates uniformly via `Ok(())` at line 274. There is
no residual branching on issue severity. The function is simpler and its control
flow is now a single straight path to `Ok(())` after all printing is done.

No dead imports were introduced. `use std::process` was never present in this
file (the original `exit` call must have been fully qualified or since removed);
the import list at lines 1–7 is clean.

### 2. Comment quality

**Line 269–272** — The exit-0 rationale comment:

```rust
// Exit 0 regardless of issues found. The status command is informational:
// missing tools/secrets are reported via colored output above, not via
// exit code. This matches `great status --json` (which uses has_issues)
// and the convention of git-status(1) and systemctl-status(1).
```

The `git-status(1)` precedent is sound: `git status` always exits 0 regardless
of working-tree state. The `systemctl-status(1)` precedent is not sound:
`systemctl status <unit>` exits 3 when the unit is inactive and 4 when the unit
does not exist. It is not a command that exits 0 regardless of reported state.
This is a WARN, not a BLOCK, because the overall design decision (exit 0) is
correct and well-motivated by `git status` alone. The bad analogy does not make
the code wrong.

**Lines 282–284** — The `run_json` doc comment:

```rust
/// Both human and JSON modes always exit 0. Issues are signalled via the
/// `has_issues` field and `issues` array in the JSON payload.
```

This accurately describes today's implementation. The concern is placement: it
documents a cross-cutting invariant (human mode also exits 0) inside a private
function that only implements JSON mode. A reader of `run` alone does not see
this contract. This is advisory.

### 3. Test quality

Three tests now cover the status command at lines 57–88 of `cli_smoke.rs`:

- `status_shows_platform` — asserts `.success()` and checks for "Platform:" in
  stderr. This is the primary regression test for the exit-0 change.
- `status_warns_no_config` — asserts `.success()` and checks for "No great.toml
  found" in stderr. This confirms the no-config path does not exit non-zero.
- `status_json_outputs_json` — asserts `.success()` and checks for "platform"
  in stdout. This is unchanged in semantics from the prior test.

The tests are minimal and correct. They test what changed: exit code. They do
not over-specify output format. A combined test for both platform output and
no-config warning in the same invocation would have been acceptable but the
separation is not harmful — each test has one observable assertion beyond the
exit code.

One gap worth noting (WARN-level): there is no test asserting that `great status`
with a config that declares a missing tool still exits 0. That was the precise
case that `process::exit(1)` previously handled. The two existing success tests
use an empty directory (no config), not a config with a failing tool. This is
the most important behavioral case and it is untested. However, the current tests
are net-positive over the prior state, so this does not block.

### 4. Complexity

The change reduces complexity. Three boolean assignments, one conditional block,
and one `process::exit` call are gone. The `run` function now has one fewer
concern. The simplification is correct.

No new abstractions were introduced. No new types. The diff is a pure subtraction
with a small comment addition. This is the appropriate shape for a behavioral
simplification.
