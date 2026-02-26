# Nightingale Selection — Task 0028
# Statusline Always Shows "idle" + Non-Destructive settings.json Install

**Selected:** 2026-02-27
**Priority:** P1
**Type:** bugfix + feature
**Module:** `loop/hooks/` (new) + `src/cli/loop_cmd.rs` + `src/cli/statusline.rs`
**Estimated Complexity:** L

---

## Priority Justification

P1 is correct. The statusline is the primary live feedback mechanism for the
great-loop. It has never worked in any real session — not because the renderer
is broken, but because the state file writer was never built. This is a
foundational gap: any user who installs great-loop and watches the statusline
sees a permanently frozen "idle" indicator regardless of agent activity.

The concurrency bug (Problem 2) compounds this: even if a user hand-crafted a
state file, two concurrent sessions would corrupt each other's state, making
session isolation a correctness requirement, not an enhancement.

Problem 3 (env var not injected for existing settings.json) means that on the
common case — any machine with a pre-existing `~/.claude/settings.json` — agent
teams are silently disabled after `great loop install`. This is a P1 install
regression.

No higher-priority open tasks exist. 0028 is the only genuinely open backlog
item.

---

## Scope Summary

**Six requirements covering three distinct problems:**

**R1 — Session-scoped state file architecture**
Replace `/tmp/great-loop/state.json` (single global) with
`/tmp/great-loop/{session_id}/state.json` (per-session). Both hook scripts and
the statusline reader derive their path from the `session_id` field in their
respective stdin JSON payloads. Stale session directories (mtime > 24h) are
cleaned up on each statusline invocation or by the `SessionEnd` hook.

**R2 — Hook handler scripts (state file writers)**
Create `loop/hooks/update-state.sh`: a single shell script that receives
Claude Code lifecycle events on stdin (`SubagentStart`, `SubagentStop`,
`TeammateIdle`, `TaskCompleted`, `Stop`, `SessionEnd`), extracts `session_id`
and `hook_event_name`, and atomically writes the agent's status into the
session-scoped state file. Atomic write via temp-file-then-mv.

**R3 — Inject hooks into settings.json during install**
Extend `run_install()` in `src/cli/loop_cmd.rs` to merge a `hooks` block into
`~/.claude/settings.json` non-destructively (same `serde_json::Value` pattern
already used for `statusLine` at lines 326-362). If a `hooks` key already
exists, merge great-loop matchers in without replacing other entries.

**R4 — Non-destructive env var injection for existing settings.json**
Replace the manual warning at lines 292-299 of `src/cli/loop_cmd.rs` with
automatic `serde_json::Value` merge injection of
`env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"`. Preserves all other env
keys and all other top-level settings.json keys.

**R5 — Statusline session_id extraction**
Re-add `session_id: Option<String>` to `SessionInfo` in `statusline.rs`
(currently stripped at line 26). When present, derive state file path as
`/tmp/great-loop/{session_id}/state.json`. When absent, fall back to
`StatuslineConfig.state_file` default for manual invocation and backward
compatibility.

**R6 — great loop status reports hooks and env**
Extend `run_status()` to check and report whether the `hooks` key and
`env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` are present in `settings.json`.

---

## Standards Assessment

The task file is thorough and well-structured. Observations:

**Meets standards:**
- Priority, Type, Module, Status, Date, Estimated Complexity headers: all present.
- Context section explains root cause with specific file locations and line numbers.
- Requirements are numbered (R1-R6) and actionable.
- Files-that-need-to-change table is explicit and complete.
- Dependencies section names completed tasks and external references.
- Backward compatibility is addressed explicitly.

**Flag: Acceptance criteria count exceeds the max-5 rule (8 criteria listed).**

The 8 checkboxes are each individually testable and necessary — they do not
overlap. However, for Nightingale's standard of max 5, the following
consolidation is recommended if the Lovelace spec needs trimming:

  - Criteria 1 + 2 can be merged: "After install, settings.json contains
    all three injected blocks AND update-state.sh is installed executable."
  - Criteria 6 + 7 can be merged: "SessionEnd/mtime cleanup removes stale
    dirs AND statusline derives session-scoped path from session_id on stdin."
  - Criteria 3 + 4 cover session isolation (writer side) and statusline
    rendering (reader side) — these are the core correctness check and should
    be kept separate.
  - Criterion 5 (idempotent install) and criterion 8 (cargo test/clippy) are
    each standalone gates.

Consolidation would yield 5 criteria. However, since the task is L complexity
and the 8 criteria map cleanly to the 6 requirements, the recommendation is
to **retain all 8 and not split into a second task** — the criteria are
already at the right granularity for a single implementation pass.

**Flag: Hook event contract should be verified by Humboldt before Da Vinci codes.**

The Notes section states "verified" for the hook invocation contract, but cites
no source document and no test. The events listed (`SubagentStart`,
`SubagentStop`, `TeammateIdle`, `TaskCompleted`, `Stop`, `SessionEnd`) should
be cross-checked against the current Claude Code documentation via Context7
MCP before the hook script is written. If any event name is incorrect, the
hook will be silently ignored.

**Flag: `hooks` key merge semantics need clarification for Lovelace.**

R3 says "merge the great-loop matchers into it rather than replacing the whole
key" if a `hooks` key already exists. The hooks schema is an object where each
key is an event name mapping to an array of matcher objects. Merging at the
event-name level (not the full-replace level) is safe only if the array append
does not produce duplicates on repeated installs. Lovelace should specify a
deduplication strategy: either check for an existing great-loop command entry
before appending, or replace the great-loop entry by marker comment/field.

---

## Notes for Lovelace (spec writer)

1. **Read `src/cli/loop_cmd.rs` lines 280-370** in full before speccing the
   install path changes. The existing statusLine merge pattern (lines 326-362)
   is the template for both the env var injection (R4) and the hooks injection
   (R3). The spec should show both as structural parallels.

2. **Read `src/cli/statusline.rs` lines 17-30** to confirm the exact struct
   field that was removed and what surrounding fields exist, so the re-addition
   in R5 is placed correctly.

3. **The hook script path expansion** — the hooks config stores
   `~/.claude/hooks/great-loop/update-state.sh` with a literal tilde. Claude
   Code must expand `~` at invocation time. Confirm this is Claude Code's
   behavior (not the Rust installer's responsibility) before finalising the
   stored path format.

4. **Atomic write pattern for `update-state.sh`:** the recommended approach
   is `jq` for JSON mutation + write to `${STATE_DIR}/state.json.tmp` + `mv`.
   If `jq` is unavailable, the script must degrade gracefully (not crash).
   Lovelace should specify the `jq` availability check and fallback behavior.

5. **`include_str!` embed location:** `loop/hooks/update-state.sh` will be
   embedded in `loop_cmd.rs`. The existing embed constants (lines 44-130)
   use `include_str!` at the top of the file. Confirm the pattern and add
   the new constant alongside the existing ones.

6. **Test strategy:** R2's acceptance criterion 4 (pipe simulated hook event,
   check file written, check statusline output) can be an integration test in
   `tests/` using `assert_cmd` + `tempfile`. Lovelace should include this in
   the spec's test plan.
