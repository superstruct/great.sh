# Release Notes — Task 0028: Statusline Session Hooks

## What Was Broken

`great statusline` always displayed `idle` regardless of how many agents were
actually running. The renderer in `src/cli/statusline.rs` correctly falls back
to `"idle"` when the agent list is empty — but the agent list was always empty
because the state file `/tmp/great-loop/state.json` was never written. No hook
handlers existed to populate it.

A secondary problem: the single global state file meant concurrent Claude Code
sessions (e.g., one per project directory) would overwrite each other's state,
causing each session's statusline to show the wrong agents or snap to `"idle"`
when the other session ended.

A third problem: `great loop install` on machines with a pre-existing
`~/.claude/settings.json` would only print a manual warning asking the user to
add `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` by hand. The hook configuration was
never written at all. In practice, agent teams and statusline hooks were
non-functional on most reinstalls.

## What Was Fixed

### Session-scoped state files

State files are now stored at `/tmp/great-loop/{session_id}/state.json`
(one directory per Claude Code session). The `SessionInfo` struct in
`src/cli/statusline.rs` was extended with `session_id: Option<String>`;
when present, the statusline derives the per-session path automatically.
When absent (manual invocation), it falls back to the configured default
path for backward compatibility.

### Hook handler script

`loop/hooks/update-state.sh` is a new bash script that receives Claude Code
lifecycle events on stdin and writes atomic state updates to the correct
session directory. It handles:

- `SubagentStart` — marks an agent as `running`
- `SubagentStop` — marks an agent as `done`
- `TeammateIdle` — marks an agent as `idle`
- `TaskCompleted` — marks the task agent as `done`
- `Stop` — marks the main session as `done`
- `SessionEnd` — removes the session directory entirely

Writes are atomic: the script writes to a temp file then `mv`s it into place.
On Linux, state updates are serialized with `flock`; macOS degrades gracefully
without it. Path traversal is rejected by allowlist regex on `session_id`.

The script is embedded via `include_str!` in `loop_cmd.rs` and written to
`~/.claude/hooks/great-loop/update-state.sh` (mode 0755) during install.

### Non-destructive settings.json merge

`great loop install` now non-destructively merges all managed keys into any
pre-existing `~/.claude/settings.json`, preserving all user-defined keys:

- `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"` — injected if absent
- `hooks` — six event handlers (listed above) are merged per-event; existing
  non-great-loop matchers are preserved; great-loop entries are deduplicated
  on repeat installs
- `statusLine` — unchanged from prior behavior

The previous code path that printed a manual warning for existing `settings.json`
files is replaced with this automatic merge.

### Stale session cleanup

On each `great statusline` tick, session directories under `/tmp/great-loop/`
with mtime older than 24 hours are removed. This is best-effort (errors are
silently ignored) and runs in approximately one `readdir` call per tick.
`SessionEnd` hook events also immediately remove the session directory.

### `great loop status` checks hooks

`great loop status` now reports:

- Whether `~/.claude/hooks/great-loop/update-state.sh` is installed
- Whether `hooks` keys are registered in `settings.json`
- Whether `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` is set in `settings.json`
- Whether `jq` is available (required by the hook script at runtime)

### `--non-interactive` wired to `loop` subcommand

`great --non-interactive loop install` now propagates the flag correctly through
`main.rs` to `run_install()`. In non-interactive mode (or when stdin is not a
TTY), install aborts without prompting when existing files would be overwritten.
Use `--force` to bypass. Previously the flag had no effect on the loop subcommand.

## Breaking Changes

None. The state file path change is transparent: when `session_id` is present
in stdin (normal Claude Code operation), the new session-scoped path is used
automatically. When absent, the old default path is used unchanged.

## Migration

Run the following command to update an existing installation:

```
great loop install --force
```

This overwrites the hook script and merges the new keys into `settings.json`.
After running, verify with:

```
great loop status
```

All six checks (agent personas, loop commands, Agent Teams config, env var,
hook handler, hooks config) should report success. The `jq` warning, if shown,
requires installing `jq` separately (`apt install jq` / `brew install jq`).

## Files Changed

| File | Change |
|---|---|
| `loop/hooks/update-state.sh` | New: Claude Code hook handler script |
| `src/cli/statusline.rs` | Added `session_id` to `SessionInfo`; session-scoped path derivation in `run_inner()`; `cleanup_stale_sessions()` |
| `src/cli/loop_cmd.rs` | Hook script embed + install; non-destructive env/hooks/statusLine merge; `run_status()` extended with hooks/env/jq checks |
| `src/main.rs` | `--non-interactive` propagated to `loop` subcommand |
| `tests/hook_state.rs` | New: end-to-end integration test (hook writes state, statusline reads it, SessionEnd cleans up) |
