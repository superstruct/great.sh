# 0028: Statusline Always Shows "idle" + Non-Destructive settings.json Install

**Priority:** P1
**Type:** bugfix + feature
**Module:** `loop/hooks/` (new) + `src/cli/loop_cmd.rs` + `src/cli/statusline.rs`
**Status:** in-progress
**Date:** 2026-02-27
**Estimated Complexity:** L

---

## Context

Two distinct problems share a root cause and a single fix surface.

### Problem 1: Statusline is permanently stuck at "idle"

The `great statusline` command (implemented in `src/cli/statusline.rs`) reads
agent state from `/tmp/great-loop/state.json` and renders the loop's activity
display. When that file is absent, `read_state()` at line 238 returns
`LoopState::default()` whose `agents` vec is empty. All three render branches
(wide, medium, narrow) in the `render()` function (lines 596-684) check
`state.agents.is_empty()` and fall through to displaying `"idle"` when no
agents are present.

The state file is **never written**. Task 0011 (done) implemented the renderer.
Task 0011 noted in "Out of Scope" that the companion feature — "hook handlers
that write `/tmp/great-loop/state.json`" — was deferred as a separate task
called `great hooks install`. That companion task was never created and never
implemented. There are zero hook handlers in the codebase that update agent
state. As a result the statusline will always display:

```
⚡ loop │ idle
```

regardless of how many agents are actually running. Confirmed by inspecting the
`loop/` directory: no hooks files exist. Confirmed by grepping for
`great-loop/state.json` writes: only `read_state()` in `statusline.rs` touches
that path.

**Fix:** Implement Claude Code hook handlers that write session-scoped state
files on agent lifecycle events, wired through `great loop install` so hooks
are registered in `~/.claude/settings.json` alongside the statusLine config.

### Problem 2: Single global state file cannot handle concurrent sessions

The current state file path is hardcoded to `/tmp/great-loop/state.json` — a
single global file. This is a race condition for any user who runs multiple
Claude Code sessions (common: one terminal per project).

**Scenario:** User runs `claude` in `~/src/project-a` and `~/src/project-b`
simultaneously. Both sessions' hooks write to the same file. The statusline
in project-a shows agents from project-b. When project-b finishes, project-a's
statusline shows "idle" even though its agents are still running.

**How Claude Code solves this elsewhere:**
- `~/.claude/projects/{mangled-path}/{session-uuid}/` — per-session conversation
- `~/.claude/session-env/{session-uuid}` — per-session environment
- `~/.claude/todos/{session-uuid}-agent-{agent-uuid}.json` — session+agent scoped

The pattern is clear: **`session_id` is the primary isolation key.**

**Critical discovery:** Claude Code passes `session_id` in **both** the hooks
stdin JSON and the statusline stdin JSON:

```json
// Hooks stdin (every event):
{ "session_id": "abc123...", "hook_event_name": "SubagentStart", ... }

// Statusline stdin (every tick):
{ "session_id": "abc123...", "workspace": { "project_dir": "/path" }, ... }
```

The `session_id` field was **deliberately removed** from `SessionInfo` in
`statusline.rs` (line 26: "session_id and transcript_path removed — sensitive,
unused by rendering"). It needs to be re-added — not for rendering, but for
state file path derivation.

**Fix:** Session-scoped state files at `/tmp/great-loop/{session_id}/state.json`.
Hook scripts extract `session_id` from stdin JSON via `jq -r '.session_id'`,
write to the session-specific path. Statusline extracts `session_id` from stdin
JSON, reads from the matching path. Stale session dirs are cleaned up via
mtime-based expiry (e.g., >24h).

### Problem 3: `great loop install` does not non-destructively inject settings

The install path in `src/cli/loop_cmd.rs` at lines 290-324 has two branches:

- **No existing settings.json (line 300-324):** Creates a new file with the
  full default object including `statusLine` and `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`.
  Correct.
- **Existing settings.json (line 292-299):** Reads the file, checks for the
  agent teams env var, and if missing prints a **manual warning** asking the
  user to add it by hand. Does NOT inject it automatically. This means on any
  machine where the user already has a `~/.claude/settings.json` (the common
  case), the critical `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` env var is not
  written, and agent teams will not function.

The statusLine injection block (lines 326-362) does perform a non-destructive
JSON merge into existing files using `serde_json::Value` round-trip, preserving
all other keys. That part is correct. The gap is the env var.

Additionally, `great loop install` has no mechanism to write Claude Code
`hooks` into `settings.json`. Claude Code hooks are configured under a `hooks`
key in `settings.json` (or `.claude/settings.json`). Without injecting hook
registrations during `great loop install`, hook handlers can never be
automatically activated.

**Fix:** Replace the manual-hint branch for existing `settings.json` with
non-destructive JSON-merge injection of the `env` key (same pattern already
used for `statusLine` at lines 326-362), and add a parallel injection block
for the `hooks` key.

---

## Requirements

### R1 — Session-scoped state file architecture

Replace the single global `/tmp/great-loop/state.json` with per-session state
files at `/tmp/great-loop/{session_id}/state.json`.

**State file path derivation:**
1. Hook scripts: extract `session_id` from stdin JSON via `jq -r '.session_id'`.
   Derive path as `/tmp/great-loop/${SESSION_ID}/state.json`. Create the
   session directory if absent (`mkdir -p`).
2. Statusline command: re-add `session_id: Option<String>` to the `SessionInfo`
   struct in `statusline.rs` (currently stripped at line 26). When `session_id`
   is present in the stdin JSON, derive the state file path as
   `/tmp/great-loop/{session_id}/state.json`, **overriding** the
   `StatuslineConfig.state_file` default. When absent (e.g., manual invocation),
   fall back to the configured default path for backward compatibility.

**Stale session cleanup:**
- On each `great statusline` invocation, scan `/tmp/great-loop/` for session
  directories with mtime older than 24 hours and remove them. This is
  lightweight (readdir + stat) and prevents `/tmp` accumulation.
- Alternatively, hook scripts can register a `SessionEnd` or `Stop` handler
  that removes the session directory on exit.

### R2 — Hook handler scripts (state file writers)

Create hook handler scripts in `loop/hooks/` that write agent state changes
to the session-scoped state file:

- `loop/hooks/update-state.sh` — single script handling all events. Receives
  JSON on stdin from Claude Code with `session_id`, `hook_event_name`, and
  event-specific fields. Extracts session_id, derives state path, and updates
  the agent entry.

Claude Code hook events to register:
- `SubagentStart` — set agent status to `"running"` with current timestamp.
- `SubagentStop` — set agent status to `"done"`.
- `TeammateIdle` — set agent status to `"idle"`.
- `TaskCompleted` — update agent status to `"done"`.
- `Stop` — set agent status to `"done"`.
- `SessionEnd` — clean up the session state directory.

Each write must be atomic: write to a temp file in the same directory, then
`mv` to `state.json` to avoid races with the statusline reader.

The script is embedded via `include_str!` in `src/cli/loop_cmd.rs` (same
pattern as agent `.md` files, lines 44-130) and written to
`~/.claude/hooks/great-loop/` during `great loop install`.

### R3 — Inject hooks into settings.json during install

Extend the `run_install()` function in `src/cli/loop_cmd.rs` to write a `hooks`
block into `~/.claude/settings.json` (non-destructively — preserve all other
keys). The hooks block registers the agent lifecycle hook handlers for the
events listed in R2. Use the same `serde_json::Value` merge pattern as the
existing statusLine injection (lines 326-362). If a `hooks` key already exists,
merge the great-loop matchers into it rather than replacing the whole key.

Example hooks configuration to inject:
```json
{
  "hooks": {
    "SubagentStart": [{ "matcher": "", "hooks": [{ "type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh" }] }],
    "SubagentStop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh" }] }],
    "TeammateIdle": [{ "matcher": "", "hooks": [{ "type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh" }] }],
    "Stop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh" }] }],
    "SessionEnd": [{ "matcher": "", "hooks": [{ "type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh" }] }]
  }
}
```

### R4 — Non-destructive env var injection for existing settings.json

Replace the manual warning at lines 292-299 of `src/cli/loop_cmd.rs` with
automatic non-destructive injection of `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`
into the existing `settings.json`. Use `serde_json::Value` merge: read the
file, parse as object, add `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"` if
absent (preserving all other env vars and all other top-level keys), write back.

### R5 — Statusline session_id extraction and path derivation

Re-add `session_id: Option<String>` to the `SessionInfo` struct in
`src/cli/statusline.rs` (line 17-28). When present, use it to derive the
state file path as `/tmp/great-loop/{session_id}/state.json`, overriding the
`StatuslineConfig.state_file` default. When absent, fall back to the
configured default for backward compatibility with manual invocation.

This is the **reader-side** counterpart to R2's writer-side session scoping.
Without this, the statusline would still read from the global default path
even though hooks are writing to session-scoped paths.

### R6 — great loop status checks hooks and env

Extend `run_status()` in `src/cli/loop_cmd.rs` (lines 422-490) to check:
- Whether the `hooks` key is present in `settings.json` with the great-loop
  matchers.
- Whether `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` is present in
  `settings.json`.
Report each as success/warning in the same style as existing checks.

---

## Acceptance Criteria

- [ ] After `great loop install`, `~/.claude/settings.json` contains
      `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"`, the `statusLine`
      block, AND hooks registrations for `SubagentStart`, `SubagentStop`,
      `TeammateIdle`, `Stop`, and `SessionEnd`. All pre-existing keys are
      preserved (verified by diffing before/after with a fixture containing
      `alwaysThinkingEnabled` and custom `permissions`).

- [ ] After `great loop install`, the hook handler script exists at
      `~/.claude/hooks/great-loop/update-state.sh` and is executable.

- [ ] **Session isolation:** Simulating two concurrent sessions by running
      `update-state.sh` with two different `session_id` values creates two
      separate state directories under `/tmp/great-loop/`. Reading the
      statusline with session A's `session_id` on stdin returns only
      session A's agents — not session B's.

- [ ] After a `SubagentStart` hook fires (simulated by piping
      `{"session_id":"test-abc","hook_event_name":"SubagentStart",...}` to
      `update-state.sh`), `/tmp/great-loop/test-abc/state.json` is created
      with the agent's status set to `"running"`, and
      `echo '{"session_id":"test-abc"}' | great statusline` no longer
      renders `"idle"` for that agent.

- [ ] Running `great loop install` twice on a machine with a pre-existing
      correct `settings.json` does not duplicate any keys and does not
      overwrite any keys not managed by great.sh (idempotent; verified by
      comparing sha256 of the file before and after the second install).

- [ ] `SessionEnd` hook (or mtime-based cleanup in statusline) removes stale
      session directories older than 24 hours from `/tmp/great-loop/`.

- [ ] `SessionInfo` in `statusline.rs` includes `session_id: Option<String>`.
      When present, the state file path is derived as
      `/tmp/great-loop/{session_id}/state.json`. When absent (manual
      invocation), falls back to the `StatuslineConfig.state_file` default.

- [ ] `cargo test` passes with zero failures; `cargo clippy` produces zero
      new warnings.

---

## Files That Need to Change

| File | Change |
|------|--------|
| `loop/hooks/update-state.sh` | New: single hook handler script. Reads stdin JSON, extracts `session_id` + `hook_event_name`, writes session-scoped state file atomically. Handles SubagentStart/Stop, TeammateIdle, Stop, SessionEnd. |
| `src/cli/loop_cmd.rs` | Add hook file embed constant; add hook dir creation + file write in `run_install()`; replace manual env warning (lines 292-299) with non-destructive JSON merge injection; add hooks key injection block; extend `run_status()` with hooks/env checks |
| `src/cli/statusline.rs` | Re-add `session_id: Option<String>` to `SessionInfo` (line 17-28); modify `run_inner()` to derive session-scoped state file path when session_id is present; add stale session cleanup on each invocation |

---

## Dependencies

- Task 0011 (done) — `great statusline` renderer is already implemented and
  will work correctly once the state file is populated.
- Task 0013 (done) — `statusline_value()` helper and correct JSON shape are
  already in place; this task builds on that.
- Claude Code hooks documentation — hook invocation contract (env vars passed
  to hook scripts, stdin JSON shape) must be verified against the Claude Code
  docs before writing hook scripts.
- `serde_json` — already in `Cargo.toml`; used for all JSON merge operations.

---

## Notes

### Session isolation design rationale

Claude Code's own data model uses `session_id` (UUID) as the primary isolation
key across its entire `~/.claude/` directory tree:

```
~/.claude/projects/{mangled-path}/{session-uuid}/       # conversation data
~/.claude/session-env/{session-uuid}                    # per-session env
~/.claude/todos/{session-uuid}-agent-{agent-uuid}.json  # per-session+agent
```

The great-loop state file follows this pattern:
```
/tmp/great-loop/{session-id}/state.json
```

Why `/tmp/` and not `~/.claude/`:
- State files are ephemeral (valid only for the duration of a session)
- `/tmp/` is auto-cleaned by the OS on reboot
- Avoids polluting `~/.claude/` with session-scoped artifacts
- The statusline command runs every ~300ms; `/tmp/` is typically tmpfs (RAM)

### Claude Code hook invocation contract (verified)

Hooks receive JSON on stdin with:
```json
{
  "session_id": "uuid",
  "cwd": "/path",
  "hook_event_name": "SubagentStart",
  "tool_name": "...",        // event-specific
  "tool_input": { ... }      // event-specific
}
```

Available events relevant to agent tracking: `SubagentStart`, `SubagentStop`,
`TeammateIdle`, `TaskCompleted`, `Stop`, `SessionEnd`. The env var
`CLAUDE_PROJECT_DIR` is set for all hooks.

### Backward compatibility

The `StatuslineConfig.state_file` default remains `/tmp/great-loop/state.json`.
When `session_id` is absent from stdin (manual invocation, testing), the
statusline falls back to this path. This preserves the existing test suite
and allows `echo '{}' | great statusline` to work without session context.

### Rendering

The `render_summary()` function at `src/cli/statusline.rs` line 468 returns
`"idle".dimmed()` when the agent list is empty. Once the state file is
populated, this idle fallback becomes the correct "no active loop" state rather
than a stuck display.

### Non-destructive merge

The settings.json merge approach (R4) follows the pattern already established
at `src/cli/loop_cmd.rs` lines 326-362. The fix is mechanical: lift lines
292-299 out of the text-search branch and into a `serde_json::Value` merge
block matching the existing injection pattern.
