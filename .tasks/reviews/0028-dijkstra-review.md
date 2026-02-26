# Code Review — Task 0028: Session-Scoped Hook Handlers for Statusline

**Reviewer:** Edsger Dijkstra (Code Reviewer role)
**Date:** 2026-02-27
**Branch:** main

---

## VERDICT: APPROVED

---

## Per-File Assessment

### 1. `loop/hooks/update-state.sh` — New

**Structure:** Clean linear flow. Validation, dispatch, update — each stage
distinct and separately readable. The function is approximately 125 lines but
logical phases are visibly delimited by comments.

**Path traversal defense (lines 20–25):** The regex allowlist
`^[a-zA-Z0-9._-]+$` combined with the 200-character length cap is correct
defense-in-depth. The shell reads `SESSION_ID` from untrusted JSON input;
this guard is necessary and the implementation is sound.

**Atomic write (lines 84–123):** Temp-file-then-`mv` is the correct pattern
for atomic replacement on Linux. The `flock` path on Linux and silent
degradation on macOS (where `flock` may be absent) is documented and
appropriate.

**`jq` re-invocations (lines 51–64):** Each event arm re-invokes `jq` on
`$INPUT` separately to extract its field. This is three separate subprocess
calls for events with both `agent_id` and `status` derivation. The input has
already been read into `$INPUT` so correctness is unaffected, but it is
mildly redundant. This is not a BLOCK; the script runs transiently, latency
is invisible to the user, and a single compound `jq` expression extracting
all fields at once would reduce readability for marginal gain.

**SessionEnd before `mkdir -p` (lines 32–35):** Cleanup exits before
creating the directory when the directory does not yet exist. This is
correct: there is nothing to clean if the directory was never created.

**`trap` placement (line 87):** The trap is set after `flock`, meaning if
`flock` acquisition blocks for 5 seconds and the script is interrupted, the
temp file from a prior interrupted run might linger. However, `TMPFILE` is
not created until after the `trap` is set (line 84 declares the variable,
line 123 creates the file), so there is no window where a file exists without
a trap. Correct.

**`max // 0` in jq (line 117):** When `agents` is empty, `map(.id) | max`
returns `null`; the `// 0` alternative makes the first `id` equal to 1.
Correct.

**Verdict:** No structural defects.

---

### 2. `src/cli/statusline.rs` — Modified

**`run` wraps `run_inner` in `catch_unwind` (lines 128–142):** This is the
correct approach for a statusline: the process must exit 0 even if the
rendering logic panics, to avoid breaking the shell prompt. The design is
explicit and documented.

**Session ID validation in Rust (lines 172–183):** The character allowlist
mirrors the shell script. Good: both sides of the trust boundary enforce
the same invariant independently.

**`read_state` path traversal fallback (lines 261–263):** A path containing
`".."` falls back to `/tmp/great-loop/state.json` rather than failing with
an error. This is correct for a statusline that must never crash, but the
behavior could surprise a developer who configures a legitimate path
containing `..` in their config file. The comment makes the security intent
clear, which is sufficient.

**`cleanup_stale_sessions` called on every tick (line 189):** The function
runs on each statusline render (~300ms interval). It does a `read_dir` and
`metadata` call per entry. On a machine with many old sessions this could
add latency. The comment acknowledges "best-effort" and errors are silently
ignored. Acceptable given the constraints of a statusline hook, but worth
noting that mtime-based cleanup operates at the directory level (not the
state file), which is correct.

**`render` function length (lines 632–751):** The `render` function is 120
lines and contains three width-mode branches (wide/medium/narrow). Each
branch is self-contained. Cyclomatic complexity is bounded; the branches do
not share mutable state. This is at the upper boundary of comfortable
comprehension but is acceptable given the rendering domain.

**`visible_len` does not handle CSI sequences beyond `m` (lines 354–369):**
The ANSI parser accepts escape sequences of the form `\x1b[...m` only.
Other CSI sequences (cursor movement, etc.) would be counted as visible
characters. This is a known limitation and the code only emits `colored`
output which produces standard `m`-terminated sequences. No defect in
practice.

**`.unwrap()` usage in `#[cfg(test)]` (lines 1055–1058, 1130–1132, etc.):**
All `.unwrap()` calls in this file appear exclusively inside `#[cfg(test)]`
blocks. This conforms to the project convention.

**Test coverage:** Exemplary. 30+ unit tests covering data parsing, format
helpers, rendering across all three width modes, path traversal, timeout
demotion, ANSI truncation, and the new session_id field. The test at line
1443 `test_session_id_path_derivation` is trivially correct (just string
formatting) but its presence as documentation is fine.

**Verdict:** No structural defects.

---

### 3. `src/cli/loop_cmd.rs` — Modified

**`hook_entry` closure (lines 175–184):** The closure accepts an `_event`
parameter that it does not use. The event name is embedded in the outer
`serde_json::json!` call rather than in the hook entry itself (which is
correct — Claude Code keys the event from the map key). The `_event`
parameter with the underscore prefix correctly signals intentional disuse.
This is not a defect, but the parameter could be removed entirely to reduce
confusion. Advisory only.

**`run_install` settings merge (lines 376–525):** This is the most complex
function in the changeset at ~150 lines. It handles six cases: readonly
file, existing parseable JSON, existing JSON with agent teams env already
set, hooks injection, statusLine injection/repair, and fresh file creation.
The branching is dense but each branch is labeled with a comment and targets
a distinct semantic condition. No branch shares mutable state with another
in a way that creates ordering hazards.

**`unwrap_or_default()` on settings read (line 624):** At line 624,
`std::fs::read_to_string(&settings_path).unwrap_or_default()` silently
swallows an IO error on a file that `settings_path.exists()` already
confirmed exists. A race condition (file deleted between the `exists()` check
and the `read_to_string`) would produce an empty string, which
`contains(...)` would then correctly return `false` for — degrading to a
"warning" state display. This is acceptable for a status display but differs
from the install path which uses `.context(...)`. The behavior is safe but
inconsistent.

- [WARN] `src/cli/loop_cmd.rs:624` — `read_to_string(...).unwrap_or_default()` in `run_status` silently swallows IO errors. Pattern used at line 647 as well. The install path uses `.context(...)`. Consider consistent error propagation; the status subcommand could print a warning to the user rather than silently defaulting to "not found."

**Duplicate `read_to_string` calls (lines 624 and 647):** `settings_path` is
read twice in `run_status` — once to check for `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`
and again to check for `great-loop/update-state.sh`. This is a minor
redundancy (two filesystem reads of the same file). For a status-display
function this is acceptable.

- [WARN] `src/cli/loop_cmd.rs:624,647` — settings.json is read twice in `run_status`. Extract to a single `read_to_string` call with the result reused for both string-contains checks.

**`collect_existing_paths` does not count observer template (lines 216–251):**
The function's doc comment says "21 managed file paths" but the comment at
line 1012 in the test asserts 22. The discrepancy is in the doc comment only;
the assertion tests actual behavior (15 agents + 5 commands + 1 config + 1
hook = 22). The doc comment at line 212 is stale.

- [WARN] `src/cli/loop_cmd.rs:212` — Doc comment says "21 managed file paths" but behavior produces 22 (15 agents + 5 commands + 1 teams config + 1 hook script). Stale count in the comment.

**Tests:** Thorough. Settings merge idempotency, user-hook preservation, hook
`async` flag assertion, statusLine repair, and `collect_existing_paths`
boundary cases are all covered. The tests duplicate the production logic
literally (e.g. `test_repair_fixes_broken_statusline` contains an
inline copy of the repair decision tree). This makes the test fragile to
refactoring but is a common pattern in Rust for testing private decision
logic without extracting it. Advisory only.

**Verdict:** No blocking defects. Three advisory warnings.

---

### 4. `src/main.rs` — Modified

**Non-interactive wiring (lines 34–37):** Follows the exact same pattern
already established for `Apply` (line 19–22) and `Doctor` (line 27–30).
Consistent. No issues.

**Verdict:** Clean.

---

### 5. `tests/hook_state.rs` — New integration test

**Structure:** Linear sequence: invoke hook via subprocess, assert state file
content, invoke statusline binary, assert output, simulate SessionEnd, assert
cleanup. Each phase is clearly delimited by comments.

**`.unwrap()` on `stdin.take()` (lines 60, 115):** The `.take()` on a
`Stdio::piped()` child's stdin returns `Option<ChildStdin>`. After spawning
with `Stdio::piped()`, `stdin` is always `Some`. The `.unwrap()` is safe in
this context; in a test, panicking on `None` is the correct behavior. This
conforms to project conventions for `#[cfg(test)]` contexts.

**`jq` availability check (lines 27–36):** The test explicitly requires `jq`
and fails with an actionable message if absent. Correct.

**Soft skip pattern (lines 18–24):** When the hook script is not found, the
test prints to stderr and returns (passes). This is the right behavior for a
CI environment that may not have the script in a known location. The path is
derived from `CARGO_MANIFEST_DIR` which is set by cargo at test time, so
in normal use the script will always be found. The skip is a conservative
safety net.

**Negative assertion on statusline output (lines 89–93):** The test asserts
`!stdout.contains("idle")` rather than asserting a specific positive value
(e.g., `"running"`). This is intentionally loose — the exact rendered string
depends on terminal width and color state — but it is sufficient to confirm
the state was read. Acceptable.

**No test for `SubagentStop` or `SessionEnd` state transitions:** The test
covers SubagentStart and SessionEnd. The intermediate stop/done transition
is untested at the integration level (it is covered by the bash script's
unit-level behavior, tested implicitly). This is a minor coverage gap but
not a BLOCK for this changeset.

**Verdict:** No structural defects.

---

## Summary of Issues

| Severity | File | Line | Description |
|----------|------|------|-------------|
| WARN | `src/cli/loop_cmd.rs` | 624, 647 | `settings.json` read twice in `run_status`; single read with result reused is cleaner |
| WARN | `src/cli/loop_cmd.rs` | 624, 647 | `unwrap_or_default()` on file read swallows IO errors silently; inconsistent with install path which uses `.context()` |
| WARN | `src/cli/loop_cmd.rs` | 212 | Doc comment says "21 managed file paths"; actual count (and test assertion) is 22 |

---

## Overall Assessment

The changeset is structurally sound. Abstraction boundaries are clean: the
bash hook owns state writing; the Rust statusline owns state reading and
rendering; the loop installer owns settings injection. Responsibilities do
not bleed across module boundaries. Error handling in the hot path (the
statusline render) correctly degrades to empty output rather than crashing.
The session-scoping design — using session_id as a subdirectory under
`/tmp/great-loop/` — is simple and correct. Path traversal defenses are
present on both the bash and Rust sides, independently. Test coverage is
extensive across all three files that contain new logic.

The three advisory warnings are minor. None affects correctness. The
duplicate-read issue in `run_status` is a maintenance debt item, not a
defect. This change may proceed to commit.
