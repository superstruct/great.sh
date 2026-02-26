# 0028: Statusline Hooks and Non-Destructive Install -- Technical Specification

**Task:** 0028-statusline-stuck-and-install
**Author:** Lovelace (Spec Writer)
**Date:** 2026-02-27
**Status:** ready
**Estimated Complexity:** L

---

## Summary

The statusline permanently displays "idle" because no hook handler writes the
agent state file. This spec defines three coordinated changes:

1. A shell hook script (`loop/hooks/update-state.sh`) that writes
   session-scoped agent state files in response to six Claude Code lifecycle
   events: `SubagentStart`, `SubagentStop`, `TeammateIdle`, `TaskCompleted`,
   `Stop`, `SessionEnd`.
2. Rust changes to `run_install()` that embed and deploy the hook script, inject
   `hooks` and `env` keys into `~/.claude/settings.json` non-destructively.
3. Rust changes to the statusline reader that extracts `session_id` from stdin
   to read the correct session-scoped state file.

All six hook event names are verified against the official Claude Code hooks
documentation at https://code.claude.com/docs/en/hooks. The previous revision of
this spec incorrectly removed `SubagentStart`, `TeammateIdle`, and
`TaskCompleted` -- all three are valid events.

---

## Hook Events Reference

All six events receive the common fields on stdin:

```json
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/path/to/project",
  "permission_mode": "default",
  "hook_event_name": "EventName"
}
```

Event-specific input fields used by `update-state.sh`:

| Event           | Extra Fields                                                                 | Blocks? | async? |
|-----------------|-----------------------------------------------------------------------------|---------|--------|
| `SubagentStart` | `agent_id`, `agent_type`                                                    | No      | Yes    |
| `SubagentStop`  | `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message` | No      | Yes    |
| `TeammateIdle`  | `teammate_name`, `team_name`                                                | No      | Yes    |
| `TaskCompleted` | `task_id`, `task_subject`, `task_description?`, `teammate_name?`, `team_name?` | No   | Yes    |
| `Stop`          | `stop_hook_active`, `last_assistant_message`                                | No      | Yes    |
| `SessionEnd`    | `reason`                                                                    | No      | Yes    |

All six hooks use `"async": true` because state file writes are side-effects
that must not slow down the main agent loop. The hook script always exits 0 on
success.

---

## Part 1: Hook Script -- `loop/hooks/update-state.sh`

### 1.1 File Location

Source: `loop/hooks/update-state.sh` (new file, committed to repo).
Deployed to: `~/.claude/hooks/great-loop/update-state.sh` during `great loop install`.
Embedded via: `include_str!("../../loop/hooks/update-state.sh")` in `loop_cmd.rs`.

### 1.2 State File Schema

The state file at `/tmp/great-loop/{session_id}/state.json` must match the
existing `LoopState` struct in `src/cli/statusline.rs` (lines 30-38):

```json
{
  "loop_id": "session-abc123",
  "started_at": 1740700000,
  "agents": [
    {
      "id": 1,
      "name": "agent-abc123",
      "status": "running",
      "updated_at": 1740700042
    },
    {
      "id": 2,
      "name": "agent-def456",
      "status": "done",
      "updated_at": 1740700055
    }
  ]
}
```

Field mapping to Rust types:

| JSON Field            | Rust Type                    | Notes                                        |
|-----------------------|------------------------------|----------------------------------------------|
| `loop_id`             | `Option<String>`             | Set to `session_id` on first write           |
| `started_at`          | `Option<u64>`                | Unix epoch seconds, set on first write       |
| `agents`              | `Vec<AgentState>`            | Array of agent objects                       |
| `agents[].id`         | `u32`                        | Sequential integer, assigned on first seen   |
| `agents[].name`       | `String`                     | Lookup key: `agent_id` for subagents, `teammate_name` for teammates, `"main"` for Stop. This is NOT a display name -- it is the stable key used for upsert matching. |
| `agents[].status`     | `AgentStatus` (enum)         | One of: `"idle"`, `"queued"`, `"running"`, `"done"`, `"error"` |
| `agents[].updated_at` | `u64`                        | Unix epoch seconds, updated on every write   |

### 1.3 Script Implementation

```bash
#!/usr/bin/env bash
# update-state.sh -- Claude Code hook handler for great-loop state tracking
# Receives JSON on stdin from Claude Code. Writes session-scoped state file.
# Dependencies: jq (required)
set -euo pipefail

# --- Read stdin ---
INPUT="$(cat)"

# --- Extract common fields ---
SESSION_ID="$(echo "$INPUT" | jq -r '.session_id // empty')"
EVENT="$(echo "$INPUT" | jq -r '.hook_event_name // empty')"

# Bail silently if missing critical fields (do not block Claude Code)
if [[ -z "$SESSION_ID" || -z "$EVENT" ]]; then
  exit 0
fi

# --- Validate session_id (defense-in-depth against path traversal) ---
if [[ ! "$SESSION_ID" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  exit 0
fi

# --- Derive paths ---
STATE_DIR="/tmp/great-loop/${SESSION_ID}"
STATE_FILE="${STATE_DIR}/state.json"

# --- Handle SessionEnd: cleanup and exit ---
if [[ "$EVENT" == "SessionEnd" ]]; then
  rm -rf "$STATE_DIR"
  exit 0
fi

# --- Ensure state directory exists ---
mkdir -p "$STATE_DIR"

# --- Initialize state file if absent ---
NOW="$(date +%s)"
if [[ ! -f "$STATE_FILE" ]]; then
  echo "{\"loop_id\":\"${SESSION_ID}\",\"started_at\":${NOW},\"agents\":[]}" > "$STATE_FILE"
fi

# --- Determine agent identity and status ---
case "$EVENT" in
  SubagentStart)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.agent_id // empty')"
    NEW_STATUS="running"
    ;;
  SubagentStop)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.agent_id // empty')"
    NEW_STATUS="done"
    ;;
  TeammateIdle)
    AGENT_KEY="$(echo "$INPUT" | jq -r '.teammate_name // empty')"
    NEW_STATUS="idle"
    ;;
  TaskCompleted)
    # Use teammate_name if present (team context), else task_id
    AGENT_KEY="$(echo "$INPUT" | jq -r '.teammate_name // .task_id // empty')"
    NEW_STATUS="done"
    ;;
  Stop)
    AGENT_KEY="main"
    NEW_STATUS="done"
    ;;
  *)
    # Unknown event -- ignore silently
    exit 0
    ;;
esac

# Bail if we could not determine an agent key
if [[ -z "$AGENT_KEY" ]]; then
  exit 0
fi

# --- Atomic state update (serialized with flock on Linux) ---
# Read current state, upsert agent, write to temp, then mv.
TMPFILE="${STATE_DIR}/state.json.tmp.$$"

# flock is Linux-only (util-linux). On macOS it is absent unless installed
# via Homebrew; we degrade gracefully to the racy-but-mostly-correct path.
# -w 5 bounds the wait; || true prevents set -e from aborting on timeout.
if command -v flock >/dev/null 2>&1; then
  exec 9>"${STATE_DIR}/.lock"
  flock -w 5 9 || true
fi

jq --arg key "$AGENT_KEY" \
   --arg status "$NEW_STATUS" \
   --argjson now "$NOW" \
   '
   # Find existing agent index by matching .name == $key
   (.agents | map(.name) | index($key)) as $idx |
   if $idx != null then
     # Update existing agent
     .agents[$idx].status = $status |
     .agents[$idx].updated_at = $now
   else
     # Append new agent with next sequential id
     .agents += [{
       "id": ((.agents | map(.id) | max // 0) + 1),
       "name": $key,
       "status": $status,
       "updated_at": $now
     }]
   end
   ' "$STATE_FILE" > "$TMPFILE" && mv "$TMPFILE" "$STATE_FILE"

exit 0
```

### 1.4 Agent Identity Strategy

The script uses a single stable key per agent to enable upsert (update-or-insert).
The `.name` field in the state JSON IS the lookup key -- it is not a human-readable
display name. This keeps the implementation simple and avoids a second field for
lookup. If display names are desired in the future, a separate `display_name`
field can be added.

| Event           | `.name` (lookup key, stored in state) | Source Field                    |
|-----------------|---------------------------------------|---------------------------------|
| `SubagentStart` | e.g. `"agent-abc123"`                 | `agent_id`                      |
| `SubagentStop`  | e.g. `"agent-abc123"`                 | `agent_id`                      |
| `TeammateIdle`  | e.g. `"davinci"`                      | `teammate_name`                 |
| `TaskCompleted` | e.g. `"davinci"` or `"task-xyz"`      | `teammate_name` or `task_id`    |
| `Stop`          | `"main"`                              | literal constant                |

**Invariant:** For every event pair (SubagentStart/SubagentStop, TeammateIdle/TaskCompleted),
the `.name` value is derived from the same source field (`agent_id` or `teammate_name`),
ensuring that the upsert lookup in jq always matches the previously-inserted entry.

### 1.5 Atomic Write Pattern

All state mutations follow: read -> transform with jq -> write to
`state.json.tmp.$$` (PID-scoped temp file in same directory) -> `mv` to
`state.json`. The `mv` is atomic on the same filesystem (POSIX guarantee).
`/tmp/` is always a single filesystem.

### 1.6 jq Dependency

`jq` is a hard requirement. The `great doctor` command already checks for
common CLI tools. If `jq` is absent, `update-state.sh` will fail with a
non-zero exit code, which Claude Code treats as a non-blocking error (logged in
verbose mode, does not interrupt the session). The `great loop status` command
(R6) will also check for `jq` availability.

### 1.7 Edge Cases

- **Empty stdin:** `jq -r '.session_id // empty'` produces empty string; script
  exits 0 silently.
- **Concurrent writes from same session:** Two hooks firing simultaneously for
  the same session could race. The `mv` is atomic but the read-transform is
  not. Mitigation: on Linux the script uses `flock -w 5` on `${STATE_DIR}/.lock`
  to serialize concurrent writes within the same session directory, bounding
  the wait to 5 seconds with a `|| true` fallback so a stale lock never aborts
  the script. On macOS, where `flock` is absent from the default userland,
  the script degrades gracefully to the racy-but-mostly-correct path (the same
  behavior that existed before Advisory 1 was addressed). Simultaneous events
  within a single session are rare in practice.
- **State file corruption:** If `jq` fails to parse a corrupted state file, the
  temp file is not created, `mv` does not execute, and the corrupt file remains.
  The next `SessionEnd` or session expiry will clean it up.
- **`/tmp` not writable:** `mkdir -p` fails, `set -e` causes exit 1 (non-blocking
  error). Statusline falls back to "idle" display.
- **Very long session with many agents:** The `agents` array grows unboundedly
  within a session. With 30+ agents the statusline renderer already caps display
  at 30 (see `render_agents_wide` line 513). The state file itself is small
  (~100 bytes per agent entry).

---

## Part 2: Rust Changes -- `src/cli/loop_cmd.rs`

### 2.1 New Embed Constant

Add after the existing `OBSERVER_TEMPLATE` constant (line 136):

```rust
/// Hook handler script embedded at compile time.
const HOOK_UPDATE_STATE: &str = include_str!("../../loop/hooks/update-state.sh");
```

### 2.2 New Helper: `hooks_value()`

Returns the hooks configuration JSON to inject into settings.json:

```rust
/// Returns the `hooks` JSON object for Claude Code settings.
///
/// Each event maps to an array with one matcher object containing
/// one command hook. The hook runs async (state writes are side-effects
/// that must not block the agent loop).
fn hooks_value() -> serde_json::Value {
    let cmd = "~/.claude/hooks/great-loop/update-state.sh";
    let hook_entry = |_event: &str| -> serde_json::Value {
        serde_json::json!([{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": cmd,
                "async": true
            }]
        }])
    };
    serde_json::json!({
        "SubagentStart": hook_entry("SubagentStart"),
        "SubagentStop": hook_entry("SubagentStop"),
        "TeammateIdle": hook_entry("TeammateIdle"),
        "TaskCompleted": hook_entry("TaskCompleted"),
        "Stop": hook_entry("Stop"),
        "SessionEnd": hook_entry("SessionEnd")
    })
}
```

### 2.3 New Helper: `is_great_loop_hook()`

Used by the merge logic to identify great-loop hook entries for deduplication:

```rust
/// Check if a hook matcher array entry is a great-loop hook.
/// Identifies by checking if any hook command contains "great-loop/update-state.sh".
fn is_great_loop_hook(entry: &serde_json::Value) -> bool {
    entry.get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|hook| {
                hook.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("great-loop/update-state.sh"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
```

### 2.4 Changes to `run_install()`

The following changes are applied to the `run_install()` function in
`src/cli/loop_cmd.rs`:

#### 2.4.1 Deploy Hook Script (after teams config write, before settings.json)

Insert after line 288 (`output::success("Agent Teams config -> ...")`):

```rust
// Write hook handler script
let hooks_dir = claude_dir.join("hooks").join("great-loop");
std::fs::create_dir_all(&hooks_dir)
    .context("failed to create ~/.claude/hooks/great-loop/ directory")?;
let hook_script_path = hooks_dir.join("update-state.sh");
std::fs::write(&hook_script_path, HOOK_UPDATE_STATE)
    .context("failed to write hook script")?;

// Make executable (Unix only)
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&hook_script_path)
        .context("failed to read hook script metadata")?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook_script_path, perms)
        .context("failed to set hook script permissions")?;
}
output::success("Hook handler -> ~/.claude/hooks/great-loop/update-state.sh");
```

#### 2.4.2 Replace Settings.json Handling with Non-Destructive Merge

Replace lines 290-362 (the entire settings.json handling block -- env check at
290-299, fresh-file creation at 300-324, and statusLine injection at 326-362)
with a unified read-merge-write pass:

```rust
// Handle settings.json (non-destructive merge for all keys)
let settings_path = claude_dir.join("settings.json");
if settings_path.exists() {
    let contents = std::fs::read_to_string(&settings_path)
        .context("failed to read ~/.claude/settings.json")?;
    match serde_json::from_str::<serde_json::Value>(&contents) {
        Ok(mut val) => {
            if let Some(obj) = val.as_object_mut() {
                let mut modified = false;

                // --- Inject env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS ---
                let env_obj = obj
                    .entry("env")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(env_map) = env_obj.as_object_mut() {
                    if !env_map.contains_key("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS") {
                        env_map.insert(
                            "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS".to_string(),
                            serde_json::json!("1"),
                        );
                        modified = true;
                    }
                }

                // --- Inject or merge hooks ---
                let desired_hooks = hooks_value();
                let hooks_obj = obj
                    .entry("hooks")
                    .or_insert_with(|| serde_json::json!({}));
                if let Some(hooks_map) = hooks_obj.as_object_mut() {
                    // Snapshot for idempotency check
                    let hooks_before = serde_json::to_string(hooks_map).unwrap_or_default();

                    if let Some(desired_map) = desired_hooks.as_object() {
                        for (event_name, desired_matchers) in desired_map {
                            if let Some(existing_arr) = hooks_map
                                .get_mut(event_name)
                                .and_then(|v| v.as_array_mut())
                            {
                                // Remove any existing great-loop entries (dedup)
                                existing_arr.retain(|entry| !is_great_loop_hook(entry));
                                // Append the great-loop entries
                                if let Some(new_entries) = desired_matchers.as_array() {
                                    existing_arr.extend(new_entries.iter().cloned());
                                }
                            } else {
                                // Event key does not exist yet -- insert
                                hooks_map.insert(
                                    event_name.clone(),
                                    desired_matchers.clone(),
                                );
                            }
                        }
                    }

                    // Only mark modified if hooks actually changed
                    let hooks_after = serde_json::to_string(hooks_map).unwrap_or_default();
                    if hooks_before != hooks_after {
                        modified = true;
                    }
                }

                // --- Inject or repair statusLine ---
                let needs_statusline = if !obj.contains_key("statusLine") {
                    true
                } else if let Some(sl) = obj.get("statusLine").and_then(|v| v.as_object()) {
                    !sl.contains_key("type")
                } else {
                    false
                };
                if needs_statusline {
                    obj.insert("statusLine".to_string(), statusline_value());
                    modified = true;
                }

                // --- Write back if anything changed ---
                if modified {
                    let formatted = serde_json::to_string_pretty(&val)
                        .context("failed to serialize settings.json")?;
                    std::fs::write(&settings_path, formatted)
                        .context("failed to write ~/.claude/settings.json")?;
                    output::success("Settings updated (env, hooks, statusLine) in ~/.claude/settings.json");
                } else {
                    output::success("Settings already configured in ~/.claude/settings.json");
                }
            }
        }
        Err(_) => {
            output::warning(
                "settings.json is not valid JSON; skipping injection"
            );
        }
    }
} else {
    // No existing settings.json -- create with all managed keys
    let mut default_settings = serde_json::json!({
        "env": {
            "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
        },
        "permissions": {
            "allow": [
                "Bash(cargo *)",
                "Bash(pnpm *)",
                "Read",
                "Write",
                "Edit",
                "Glob",
                "Grep",
                "LS"
            ]
        },
        "statusLine": statusline_value()
    });
    // Merge hooks into the default settings
    default_settings.as_object_mut().unwrap().insert(
        "hooks".to_string(),
        hooks_value(),
    );
    let formatted = serde_json::to_string_pretty(&default_settings)
        .context("failed to serialize default settings")?;
    std::fs::write(&settings_path, formatted)
        .context("failed to write ~/.claude/settings.json")?;
    output::success("Settings with Agent Teams, hooks, and statusLine -> ~/.claude/settings.json");
}
```

The old code had three separate blocks (env check at 290-299, fresh-file
creation at 300-324, statusLine injection at 326-362). The new code above
unifies all three into a single read-merge-write pass.

#### 2.4.3 Hook Script in `collect_existing_paths()`

Add to `collect_existing_paths()` so the hook script is included in the
overwrite-confirmation flow:

```rust
let hook_path = claude_dir.join("hooks").join("great-loop").join("update-state.sh");
if hook_path.exists() {
    existing.push(hook_path);
}
```

#### 2.4.4 Hook Cleanup in `run_uninstall()`

Add to `run_uninstall()` before the final summary:

```rust
// Remove hook handler directory
let hooks_dir = claude_dir.join("hooks").join("great-loop");
if hooks_dir.exists() {
    std::fs::remove_dir_all(&hooks_dir)
        .context("failed to remove ~/.claude/hooks/great-loop/")?;
    output::success("Removed hooks/great-loop/ directory");
}
```

### 2.5 Changes to `run_status()`

Add two new checks after the existing `Agent Teams env` check (after line 468):

```rust
// Check for hook handler script
let hook_script = claude_dir.join("hooks").join("great-loop").join("update-state.sh");
if hook_script.exists() {
    output::success("Hook handler: installed");
} else {
    output::warning("Hook handler: not installed (statusline will show 'idle')");
}

// Check for hooks in settings.json
if settings_path.exists() {
    let contents = std::fs::read_to_string(&settings_path).unwrap_or_default();
    if contents.contains("great-loop/update-state.sh") {
        output::success("Hooks config: registered in settings.json");
    } else {
        output::warning("Hooks config: not found in settings.json");
    }
}

// Check for jq (required by hook script)
match std::process::Command::new("jq").arg("--version").output() {
    Ok(output_result) if output_result.status.success() => {
        output::success("jq: available");
    }
    _ => {
        output::warning("jq: not found (required for statusline hook handler)");
    }
}
```

---

## Part 3: Rust Changes -- `src/cli/statusline.rs`

### 3.1 Re-add `session_id` to `SessionInfo`

Change the `SessionInfo` struct (lines 17-28) to re-add `session_id`:

```rust
#[derive(Debug, Deserialize, Default)]
pub struct SessionInfo {
    #[allow(dead_code)]
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub context_tokens: Option<u64>,
    pub context_window: Option<u64>,
    #[allow(dead_code)]
    pub workspace: Option<String>,
    /// Session ID used for state file path derivation. Not rendered.
    pub session_id: Option<String>,
}
```

The comment at line 26 ("session_id and transcript_path removed") is deleted.
`transcript_path` remains excluded -- it is not needed.

### 3.2 Modify `run_inner()` for Session-Scoped State Path

Change `run_inner()` (lines 144-182) to derive the state file path from
`session_id` when present:

```rust
fn run_inner(args: Args) -> Result<()> {
    // 1. Handle color override.
    if args.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(true);
    }

    // 2. Load config (silent fallback to defaults)
    let config = load_config();

    // 3. Parse stdin
    let session = parse_stdin();

    // 4. Derive state file path: session-scoped if session_id present,
    //    else fall back to config default for backward compatibility.
    //
    //    Known limitation: when session_id is absent (e.g., running
    //    `great statusline` manually outside Claude Code), all concurrent
    //    invocations fall back to `config.state_file` -- a single shared path.
    //    Multiple concurrent Claude Code sessions without session_id injection
    //    would read/write the same file, causing cross-session contamination.
    //    This is a known limitation of the non-session-scoped fallback and only
    //    affects manual invocation; normal Claude Code usage always provides
    //    session_id.
    let state_file_path = match &session.session_id {
        Some(sid) if !sid.is_empty() => {
            format!("/tmp/great-loop/{}/state.json", sid)
        }
        _ => config.state_file.clone(),
    };

    // 5. Read agent state
    let (state, had_parse_error) = read_state(&state_file_path, config.session_timeout_secs);

    // 6. Clean up stale session directories (lightweight, best-effort)
    cleanup_stale_sessions();

    // 7. Resolve terminal width
    let width = resolve_width(args.width);
    let use_unicode = !args.no_unicode;

    // 8. Render
    let line = render(
        &session,
        &state,
        &config,
        width,
        use_unicode,
        had_parse_error,
    );

    // 9. Print exactly one line to stdout
    println!("{}", line);

    Ok(())
}
```

### 3.3 New Function: `cleanup_stale_sessions()`

Add after `apply_timeout()`:

```rust
/// Remove session directories under `/tmp/great-loop/` whose mtime is
/// older than 24 hours. Best-effort: errors are silently ignored because
/// this runs on every statusline tick (~300ms) and must never slow it down.
fn cleanup_stale_sessions() {
    let base = std::path::Path::new("/tmp/great-loop");
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };

    let cutoff = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(24 * 60 * 60));
    let Some(cutoff) = cutoff else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(meta) = path.metadata() else {
            continue;
        };
        let Ok(mtime) = meta.modified() else {
            continue;
        };
        if mtime < cutoff {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
```

### 3.4 Session ID Security Validation

The `session_id` field flows into a filesystem path in two places: the bash hook
script (`mkdir -p`, file writes) and the Rust statusline reader (`format!()` path
derivation). Defense-in-depth is applied at both layers:

- **Bash hook script:** Validates `SESSION_ID` against `^[a-zA-Z0-9_-]+$` before
  using it in any path operation. A session_id containing `/`, `..`, or null bytes
  is silently rejected (exit 0). This prevents path traversal attacks where
  `../../etc` could escape `/tmp/great-loop/`.
- **Rust statusline reader:** The `read_state()` function (line 234) already
  rejects paths containing `".."`. The `format!()` interpolation in `run_inner()`
  constrains the path to `/tmp/great-loop/{sid}/state.json`.
- **Claude Code behavior:** Session IDs are UUID-format strings generated by
  Claude Code. Malicious values require a compromised Claude Code installation
  or tampered stdin pipeline -- both of which imply existing code execution.

---

## Part 4: Files Summary

### New Files

| File | Purpose |
|------|---------|
| `loop/hooks/update-state.sh` | Hook handler script (Part 1.3). Receives Claude Code lifecycle events, writes session-scoped state files. |

### Modified Files

| File | Changes |
|------|---------|
| `src/cli/loop_cmd.rs` | Add `HOOK_UPDATE_STATE` constant (2.1). Add `hooks_value()` helper (2.2). Add `is_great_loop_hook()` helper (2.3). Rewrite settings.json handling in `run_install()` to unified merge (2.4.2). Deploy hook script with +x (2.4.1). Update `collect_existing_paths()` (2.4.3). Update `run_uninstall()` (2.4.4). Extend `run_status()` (2.5). |
| `src/cli/statusline.rs` | Re-add `session_id` to `SessionInfo` (3.1). Modify `run_inner()` for session-scoped path derivation (3.2). Add `cleanup_stale_sessions()` (3.3). |

### No Changes

| File | Reason |
|------|--------|
| `Cargo.toml` | No new dependencies. `serde_json`, `dirs`, `colored` already present. |
| `src/cli/statusline.rs` render functions | No changes to rendering logic. The renderer already handles populated state correctly. |
| `tests/cli_smoke.rs` | Existing smoke tests remain valid. New tests go in separate test functions. |

---

## Part 5: Build Order

The implementation must proceed in this order due to compile-time dependencies:

1. **Create `loop/hooks/update-state.sh`** -- the source file must exist before
   `include_str!` can reference it.
2. **Modify `src/cli/statusline.rs`** -- re-add `session_id`, add
   `cleanup_stale_sessions()`, modify `run_inner()`.
3. **Modify `src/cli/loop_cmd.rs`** -- add embed constant (requires step 1),
   add helpers, rewrite `run_install()`, extend `run_status()`, update
   `run_uninstall()`.
4. **Add integration tests** -- requires all source changes to compile.

---

## Part 6: Deduplication Strategy for Hooks Merge

When `great loop install` runs on a machine that already has hooks configured
in `settings.json`, the merge must be idempotent. The strategy:

1. For each event name in the desired hooks (`SubagentStart`, etc.):
   - If the event key exists in the current `hooks` object:
     a. Filter the existing matcher array to **remove** any entries where a hook
        command contains `"great-loop/update-state.sh"` (using `is_great_loop_hook()`).
     b. **Append** the great-loop matcher entries to the filtered array.
   - If the event key does not exist: insert it with the great-loop matchers.
2. Non-great-loop hook entries for the same event are preserved.

This ensures:
- Running `great loop install` twice produces identical output (idempotent).
- User-defined hooks for the same events are never removed.
- Stale great-loop entries (e.g., from a previous version with a different
  script path) are replaced.

The identifier is the command string containing `"great-loop/update-state.sh"`.
This is stable across installations and versions.

---

## Part 7: Hook Configuration Injected into settings.json

The exact JSON structure injected into the `hooks` key:

```json
{
  "hooks": {
    "SubagentStart": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }],
    "SubagentStop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }],
    "TeammateIdle": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }],
    "TaskCompleted": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }],
    "Stop": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }],
    "SessionEnd": [{
      "matcher": "",
      "hooks": [{
        "type": "command",
        "command": "~/.claude/hooks/great-loop/update-state.sh",
        "async": true
      }]
    }]
  }
}
```

The `matcher` field is set to `""` (empty string) for all events. Events that
do not support matchers (`TeammateIdle`, `TaskCompleted`, `Stop`) silently
ignore the matcher field. Events that do support matchers (`SubagentStart`,
`SubagentStop`, `SessionEnd`) match all instances when matcher is empty.

The `"async": true` field ensures state writes run in the background and do not
block the agent's tool execution or response generation.

Path expansion: Claude Code expands `~` in hook command paths at invocation
time. The Rust installer stores the literal tilde `~/.claude/hooks/...` in the
JSON. The installer does NOT expand the path.

---

## Part 8: Platform Considerations

### macOS (ARM64 / x86_64)
- `/tmp/` is a symlink to `/private/tmp/` on macOS. Both the hook script and
  the statusline reader use `/tmp/great-loop/` which resolves correctly.
- `date +%s` is available on macOS (BSD date supports `%s`).
- `jq` is not installed by default on macOS. Users should install via
  `brew install jq`. The `great loop status` command warns if jq is missing.
- `flock` is NOT part of the macOS default BSD userland. The hook script
  detects availability via `command -v flock` at runtime and skips locking
  when absent. This degrades concurrent-write protection to last-writer-wins
  semantics, which is acceptable because simultaneous hook events targeting the
  same session are rare. No user action is required; no `brew install util-linux`
  dependency is introduced.

### Ubuntu / WSL2
- `/tmp/` is typically tmpfs (RAM-backed). State files benefit from fast I/O.
- `jq` is available via `apt install jq`.
- WSL2 shares `/tmp/` between WSL distributions but not with the Windows host.
  Claude Code sessions within the same WSL distribution are correctly isolated
  by `session_id`.

### All Platforms
- The `#[cfg(unix)]` guard on `chmod 755` means the hook script is not made
  executable on Windows. This is acceptable because Claude Code hooks on Windows
  run through WSL2 (which is Unix).
- `mkdir -p`, `mv`, `rm -rf`, `date +%s` are POSIX and available on all target
  platforms.

---

## Part 9: Acceptance Criteria

Eight criteria retained from the task file. The L complexity and six
requirements justify the count (see Nightingale selection notes).

1. **Install completeness:** After `great loop install`, `~/.claude/settings.json`
   contains `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1"`, the `statusLine`
   block, AND `hooks` registrations for all six events (`SubagentStart`,
   `SubagentStop`, `TeammateIdle`, `TaskCompleted`, `Stop`, `SessionEnd`). All
   pre-existing keys are preserved.

2. **Hook script deployed:** After `great loop install`, the file
   `~/.claude/hooks/great-loop/update-state.sh` exists and is executable
   (`chmod 755`).

3. **Session isolation:** Running `update-state.sh` with two different
   `session_id` values creates two separate state directories under
   `/tmp/great-loop/`. Reading the statusline with session A's `session_id` on
   stdin returns only session A's agents.

4. **State file populated:** After a `SubagentStart` hook fires (simulated by
   piping `{"session_id":"test-abc","hook_event_name":"SubagentStart","agent_id":"agent-1","agent_type":"Explore"}` to
   `update-state.sh`), `/tmp/great-loop/test-abc/state.json` exists with the
   agent's status set to `"running"`, and the statusline no longer renders
   `"idle"`.

5. **Idempotent install:** Running `great loop install` twice on a machine with
   a pre-existing correct `settings.json` does not duplicate any keys and does
   not overwrite non-managed keys. Verified by comparing sha256 before and
   after the second install.

6. **Session cleanup:** The `SessionEnd` hook removes the session state
   directory. The statusline's `cleanup_stale_sessions()` removes directories
   older than 24 hours on each invocation.

7. **Session ID in statusline:** `SessionInfo` includes
   `session_id: Option<String>`. When present, the state file path is derived as
   `/tmp/great-loop/{session_id}/state.json`. When absent, falls back to the
   `StatuslineConfig.state_file` default.

8. **CI green:** `cargo test` passes with zero failures. `cargo clippy` produces
   zero new warnings.

---

## Part 10: Testing Strategy

### 10.1 Unit Tests in `src/cli/loop_cmd.rs`

Add to the existing `mod tests` block:

```rust
#[test]
fn test_hooks_value_has_all_events() {
    let hooks = super::hooks_value();
    let obj = hooks.as_object().expect("hooks_value must be an object");
    let expected = [
        "SubagentStart", "SubagentStop", "TeammateIdle",
        "TaskCompleted", "Stop", "SessionEnd",
    ];
    for event in &expected {
        assert!(obj.contains_key(*event), "missing event: {}", event);
        let arr = obj[*event].as_array().expect("event value must be array");
        assert!(!arr.is_empty(), "event {} must have entries", event);
    }
}

#[test]
fn test_hooks_value_entries_have_async() {
    let hooks = super::hooks_value();
    for (event, matchers) in hooks.as_object().unwrap() {
        for matcher in matchers.as_array().unwrap() {
            for hook in matcher["hooks"].as_array().unwrap() {
                assert_eq!(
                    hook["async"].as_bool(), Some(true),
                    "hook for {} must be async", event
                );
            }
        }
    }
}

#[test]
fn test_is_great_loop_hook_positive() {
    let entry = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh"}]
    });
    assert!(super::is_great_loop_hook(&entry));
}

#[test]
fn test_is_great_loop_hook_negative() {
    let entry = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "/usr/local/bin/my-hook.sh"}]
    });
    assert!(!super::is_great_loop_hook(&entry));
}

#[test]
fn test_settings_merge_idempotent() {
    // Simulate a settings.json that already has great-loop hooks
    let mut settings = serde_json::json!({
        "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" },
        "hooks": super::hooks_value(),
        "statusLine": super::statusline_value(),
        "alwaysThinkingEnabled": true
    });

    let before = serde_json::to_string_pretty(&settings).unwrap();

    // Simulate the merge logic
    let desired = super::hooks_value();
    if let Some(hooks_map) = settings["hooks"].as_object_mut() {
        if let Some(desired_map) = desired.as_object() {
            for (event, desired_matchers) in desired_map {
                if let Some(arr) = hooks_map.get_mut(event).and_then(|v| v.as_array_mut()) {
                    arr.retain(|e| !super::is_great_loop_hook(e));
                    if let Some(new) = desired_matchers.as_array() {
                        arr.extend(new.iter().cloned());
                    }
                }
            }
        }
    }

    let after = serde_json::to_string_pretty(&settings).unwrap();
    assert_eq!(before, after, "merge must be idempotent");
}

#[test]
fn test_settings_merge_preserves_user_hooks() {
    let mut settings = serde_json::json!({
        "hooks": {
            "SubagentStart": [
                {
                    "matcher": "",
                    "hooks": [{"type": "command", "command": "/usr/local/bin/user-hook.sh"}]
                }
            ]
        }
    });

    let desired = super::hooks_value();
    if let Some(hooks_map) = settings["hooks"].as_object_mut() {
        if let Some(desired_map) = desired.as_object() {
            for (event, desired_matchers) in desired_map {
                if let Some(arr) = hooks_map.get_mut(event).and_then(|v| v.as_array_mut()) {
                    arr.retain(|e| !super::is_great_loop_hook(e));
                    if let Some(new) = desired_matchers.as_array() {
                        arr.extend(new.iter().cloned());
                    }
                } else {
                    hooks_map.insert(event.clone(), desired_matchers.clone());
                }
            }
        }
    }

    // User hook must still be present
    let sa = settings["hooks"]["SubagentStart"].as_array().unwrap();
    assert_eq!(sa.len(), 2, "user hook + great-loop hook");
    assert!(sa[0]["hooks"][0]["command"].as_str().unwrap().contains("user-hook.sh"));
    assert!(sa[1]["hooks"][0]["command"].as_str().unwrap().contains("great-loop"));
}
```

### 10.2 Unit Tests in `src/cli/statusline.rs`

Add to the existing `mod tests` block:

```rust
#[test]
fn test_session_info_with_session_id() {
    let json = r#"{"session_id":"abc-123","cost_usd":0.5}"#;
    let info: SessionInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.session_id.as_deref(), Some("abc-123"));
    assert!((info.cost_usd.unwrap() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_session_info_without_session_id() {
    let json = r#"{"cost_usd":0.5}"#;
    let info: SessionInfo = serde_json::from_str(json).unwrap();
    assert!(info.session_id.is_none());
}

#[test]
fn test_session_id_path_derivation() {
    let sid = "test-session-uuid";
    let path = format!("/tmp/great-loop/{}/state.json", sid);
    assert_eq!(path, "/tmp/great-loop/test-session-uuid/state.json");
}

#[test]
fn test_cleanup_stale_sessions_no_crash_on_missing_dir() {
    // /tmp/great-loop/ may not exist -- cleanup must not panic
    cleanup_stale_sessions();
}
```

### 10.3 Integration Test for Hook Script

Add to `tests/cli_smoke.rs` or a new `tests/hook_state.rs`:

```rust
/// Integration test: simulate SubagentStart event via update-state.sh,
/// verify state file is written, then verify statusline reads it.
#[test]
fn test_hook_writes_state_and_statusline_reads_it() {
    use assert_cmd::Command;
    use std::io::Write;

    let session_id = format!("test-{}", std::process::id());
    let state_dir = format!("/tmp/great-loop/{}", session_id);

    // Clean up from any prior run
    let _ = std::fs::remove_dir_all(&state_dir);

    // Find the hook script (built from source)
    let hook_script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("loop/hooks/update-state.sh");

    if !hook_script.exists() {
        eprintln!("Skipping: hook script not found at {}", hook_script.display());
        return;
    }

    // Check jq availability -- jq is a required dependency, fail explicitly
    let jq_output = std::process::Command::new("jq").arg("--version").output()
        .expect("jq is required for this test and must be installed (apt install jq / brew install jq)");
    assert!(
        jq_output.status.success(),
        "jq --version exited with non-zero status"
    );

    // Simulate SubagentStart
    let input = serde_json::json!({
        "session_id": session_id,
        "hook_event_name": "SubagentStart",
        "agent_id": "agent-test-1",
        "agent_type": "Explore",
        "cwd": "/tmp",
        "permission_mode": "default",
        "transcript_path": "/tmp/fake.jsonl"
    });

    let mut child = std::process::Command::new("bash")
        .arg(&hook_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn hook script");

    child.stdin.take().unwrap()
        .write_all(input.to_string().as_bytes())
        .expect("failed to write to hook stdin");

    let status = child.wait().expect("failed to wait for hook");
    assert!(status.success(), "hook script should exit 0");

    // Verify state file
    let state_file = format!("{}/state.json", state_dir);
    let contents = std::fs::read_to_string(&state_file)
        .expect("state file should exist after SubagentStart");
    let state: serde_json::Value = serde_json::from_str(&contents)
        .expect("state file should be valid JSON");

    assert_eq!(state["loop_id"].as_str(), Some(&*session_id));
    assert_eq!(state["agents"][0]["status"].as_str(), Some("running"));
    assert_eq!(state["agents"][0]["name"].as_str(), Some("agent-test-1"));

    // Verify statusline reads it (not "idle")
    let statusline_input = serde_json::json!({ "session_id": session_id });
    let output = Command::cargo_bin("great")
        .expect("binary exists")
        .arg("statusline")
        .arg("--no-color")
        .write_stdin(statusline_input.to_string())
        .output()
        .expect("statusline should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("idle"),
        "statusline should not show 'idle' when agents are running, got: {}",
        stdout
    );

    // Simulate SessionEnd (cleanup)
    let end_input = serde_json::json!({
        "session_id": session_id,
        "hook_event_name": "SessionEnd",
        "reason": "other",
        "cwd": "/tmp",
        "permission_mode": "default",
        "transcript_path": "/tmp/fake.jsonl"
    });
    let mut child2 = std::process::Command::new("bash")
        .arg(&hook_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn hook script");

    child2.stdin.take().unwrap()
        .write_all(end_input.to_string().as_bytes())
        .expect("failed to write to hook stdin");
    child2.wait().expect("failed to wait for hook");

    // Verify cleanup
    assert!(
        !std::path::Path::new(&state_dir).exists(),
        "SessionEnd should remove the session directory"
    );
}
```

### 10.4 Manual Verification Checklist

For the implementer to verify before marking done:

- [ ] `cargo build` succeeds (hook script exists for `include_str!`)
- [ ] `cargo test` -- all existing + new tests pass
- [ ] `cargo clippy` -- zero new warnings
- [ ] `great loop install` on clean machine creates settings.json with hooks
- [ ] `great loop install` on machine with existing settings.json merges correctly
- [ ] `great loop install && great loop install` is idempotent (diff is empty)
- [ ] `great loop status` reports hook handler, hooks config, and jq status
- [ ] `great loop uninstall` removes `~/.claude/hooks/great-loop/`
- [ ] Pipe simulated SubagentStart to hook script, verify state file written
- [ ] Pipe simulated SessionEnd to hook script, verify directory removed
- [ ] `echo '{"session_id":"test"}' | great statusline` reads from `/tmp/great-loop/test/state.json`
- [ ] `echo '{}' | great statusline` falls back to default path (backward compat)

---

## Part 11: Error Handling

| Scenario | Behavior | User-Visible Message |
|----------|----------|---------------------|
| `jq` not installed | Hook script fails with exit 1 (non-blocking). Statusline shows "idle". | `great loop status` warns: "jq: not found (required for statusline hook handler)" |
| `/tmp` not writable | Hook script `mkdir -p` fails, `set -e` exits 1. Non-blocking. | Statusline shows "idle". No user-facing error (hooks fail silently in async mode). |
| `settings.json` is not valid JSON | Merge is skipped entirely. | `output::warning("settings.json is not valid JSON; skipping injection")` |
| `settings.json` is read-only | `std::fs::write` fails. | `anyhow` error: "failed to write ~/.claude/settings.json" with OS error. |
| State file corrupted | `read_state()` returns `(default, true)`. | Statusline renders `"ERR:state"` in bright red. |
| `session_id` contains path separators | Hook script rejects via regex validation (exit 0). Rust `read_state()` also rejects `..`. | Hook: no state written. Statusline: renders "idle" (benign fallback). |
| Hook script receives unknown event | `case` falls through to `*) exit 0`. | No effect. |
| Concurrent hook writes to same session | On Linux: serialized via `flock -w 5` on `${STATE_DIR}/.lock` (timeout 5 s, `\|\| true` fallback). On macOS: no locking (graceful degradation); simultaneous writes are rare. | Linux: sequential writes, no data loss. macOS: last-writer-wins; benign for the rare case. |

---

## Part 12: Security Considerations

1. **Path traversal:** The `session_id` is interpolated into a filesystem path
   in both the bash hook script and the Rust statusline reader. The bash script
   validates `SESSION_ID` against `^[a-zA-Z0-9_-]+$` before any path operations,
   rejecting values containing `/`, `..`, or other dangerous characters. The Rust
   `read_state()` additionally rejects paths containing `".."`. Claude Code
   generates UUID-format session IDs. These two layers of validation provide
   defense-in-depth against path traversal.

2. **Temp file race (symlink attack):** The hook script writes to
   `/tmp/great-loop/{session_id}/state.json.tmp.$$`. An attacker could create a
   symlink at this path before the script runs. Mitigation: the `mkdir -p`
   creates the session directory owned by the current user. On a properly
   configured system, `/tmp` has the sticky bit set, preventing other users from
   creating files in directories they don't own. The PID-scoped temp file name
   (`$$`) adds additional collision resistance.

3. **Hook script injection:** The hook script path in settings.json uses a
   literal tilde (`~/.claude/hooks/great-loop/update-state.sh`). Claude Code
   expands `~` to `$HOME` at invocation time. The script is deployed by
   `great loop install` with known content (embedded via `include_str!`). An
   attacker with write access to `~/.claude/hooks/` could replace the script,
   but this requires the same privilege level as modifying `~/.claude/settings.json`
   itself -- no privilege escalation.

4. **Settings.json permissions:** The installer writes `settings.json` with
   default umask permissions. On most systems this is `644`. Claude Code's own
   installer uses the same approach.
