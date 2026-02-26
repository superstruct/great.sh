# 0028 Humboldt Scout Report
**Task:** 0028-statusline-hooks-spec
**Scout:** Humboldt
**Date:** 2026-02-27
**Status:** Ready for Da Vinci

---

## 1. New File to Create

### `loop/hooks/update-state.sh`
- Does NOT exist yet. `loop/hooks/` directory does NOT exist.
- Must be created before `cargo build` (compile-time `include_str!` dependency).
- Full implementation is given verbatim in spec §1.3 (lines 111-221).
- Key behaviors: reads stdin JSON, extracts `session_id` + `hook_event_name`, validates session_id against `^[a-zA-Z0-9_-]+$`, writes to `/tmp/great-loop/{session_id}/state.json`, uses `flock -w 5` on Linux (degrades gracefully on macOS), `mv`-atomic write via PID-scoped tmpfile, `SessionEnd` removes the directory.
- Dependencies: `jq` (hard required), `bash`, `date +%s`, `flock` (optional).

---

## 2. Modified File: `src/cli/statusline.rs` (1367 lines total)

### 2.1 `SessionInfo` struct — lines 17-28
**Current state:** 6 fields, `session_id` deliberately excluded with comment "removed -- sensitive, unused by rendering".
**Change:** Re-add `session_id: Option<String>` with `#[allow(dead_code)]` and doc comment "Session ID used for state file path derivation. Not rendered." Delete the comment at line 26-27 that explains its removal.

```
Line 17: #[derive(Debug, Deserialize, Default)]
Line 18: pub struct SessionInfo {
Line 19:   model: Option<String>
Line 20:   cost_usd: Option<f64>
Line 21:   context_tokens: Option<u64>
Line 22:   context_window: Option<u64>
Line 24:   workspace: Option<String>
Line 26-27: // comment to delete
Line 28: }
```

### 2.2 `LoopState` struct — lines 31-38
No changes. Already correct schema.

### 2.3 `AgentState` struct — lines 41-48
No changes.

### 2.4 `AgentStatus` enum — lines 52-62
No changes.

### 2.5 `StatuslineConfig` struct — lines 71-90
No changes. `state_file` default remains `"/tmp/great-loop/state.json"` as the backward-compat fallback.

### 2.6 `run_inner()` — lines 144-182
**Change:** Derive state file path from `session.session_id` if present. Insert path derivation between step 3 (parse stdin) and step 4 (read agent state). Also add call to `cleanup_stale_sessions()` after reading state.

New step numbering (per spec §3.2):
- Step 4 becomes: derive `state_file_path` from `session.session_id` or fallback to `config.state_file`
- Step 5: `read_state(&state_file_path, ...)` (was `&config.state_file`)
- Step 6: call `cleanup_stale_sessions()` (new)
- Steps 7-9: unchanged (width, render, print)

### 2.7 `parse_stdin()` — lines 213-222
No changes. Already reads up to 64KB from stdin with `serde_json::from_slice`. The new `session_id` field will deserialize automatically because `SessionInfo` derives `Deserialize`.

### 2.8 `read_state()` — lines 232-248
No changes. Already has `".."` traversal guard. Will be called with the new session-scoped path.

### 2.9 `apply_timeout()` — lines 251-266
No changes.

### 2.10 New function `cleanup_stale_sessions()` — insert after `apply_timeout()`, before `resolve_width()`
Insert at approximately line 267. Full implementation in spec §3.3. Reads `/tmp/great-loop/` directory, removes subdirs with mtime > 24 hours. Best-effort: all errors silently ignored.

### 2.11 `render()` — lines 571-692
No changes. Already handles populated state correctly.

### 2.12 Existing tests that construct `SessionInfo` — multiple in `mod tests` (lines 698-1366)
The tests use `..Default::default()` to fill missing fields, so adding `session_id: Option<String>` to `SessionInfo` with `Default` does not break any existing test. Verify the struct literal at line 793 uses `..Default::default()` — it does.

### 2.13 New unit tests to add to `mod tests` block (after line 1366)
Four tests per spec §10.2:
- `test_session_info_with_session_id`
- `test_session_info_without_session_id`
- `test_session_id_path_derivation`
- `test_cleanup_stale_sessions_no_crash_on_missing_dir`

---

## 3. Modified File: `src/cli/loop_cmd.rs` (870 lines total)

### 3.1 `include_str!` pattern — lines 44-130
Current pattern: one `const` per file, `include_str!("../../loop/agents/nightingale.md")` etc.

**New constant to add after `OBSERVER_TEMPLATE` (line 136):**
```rust
const HOOK_UPDATE_STATE: &str = include_str!("../../loop/hooks/update-state.sh");
```
Path relative to `src/cli/loop_cmd.rs` is `../../loop/hooks/update-state.sh`. This matches the existing pattern for all other embedded files.

### 3.2 `statusline_value()` — lines 151-156
No changes. Used by both old and new settings merge logic.

### 3.3 New helper `hooks_value()` — insert after `statusline_value()`
Full implementation in spec §2.2. Returns `serde_json::Value` object with 6 event keys, each mapping to an array with one matcher entry containing `"type": "command"`, `"command": "~/.claude/hooks/great-loop/update-state.sh"`, `"async": true`. Literal tilde `~` is NOT expanded by the installer.

### 3.4 New helper `is_great_loop_hook()` — insert after `hooks_value()`
Full implementation in spec §2.3. Checks if a hook matcher array entry's `hooks[].command` contains `"great-loop/update-state.sh"`.

### 3.5 `collect_existing_paths()` — lines 162-189
**Change:** Add hook script path check before the `existing` return. Append after line 186:
```rust
let hook_path = claude_dir.join("hooks").join("great-loop").join("update-state.sh");
if hook_path.exists() {
    existing.push(hook_path);
}
```

### 3.6 `run_install()` — lines 231-419
This is the largest change. Current structure:

| Lines | Block |
|-------|-------|
| 232-247 | Home dir + create directories (agents, commands, teams) |
| 249-256 | Check existing files, confirm overwrite |
| 258-260 | Force message |
| 262-288 | Write agent files, command files, teams config |
| 290-299 | Check env var in existing settings.json (manual warning) |
| 300-323 | Create fresh settings.json if not exists |
| 326-362 | Inject statusLine into existing settings.json |
| 364-403 | Project working state (--project flag) |
| 406-419 | Summary |

**Change A — Deploy hook script (insert after line 288):**
Create `~/.claude/hooks/great-loop/` directory, write `HOOK_UPDATE_STATE` content to `update-state.sh`, set `chmod 755` via `#[cfg(unix)]` block using `std::os::unix::fs::PermissionsExt` with `set_mode(0o755)`. Print success message.

**Change B — Replace lines 290-362 with unified merge block:**
The three separate blocks (env warning 290-299, fresh-file creation 300-323, statusLine injection 326-362) are replaced with a single read-merge-write pass. Full Rust code in spec §2.4.2.

Structure of new block:
- `if settings_path.exists()` branch: read JSON, merge `env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`, merge `hooks` (using `is_great_loop_hook` for dedup), inject/repair `statusLine`, write back only if `modified = true`.
- `else` branch: create fresh settings.json with all four managed keys (`env`, `permissions`, `statusLine`, `hooks`).

**Risk:** The `or_insert_with` pattern on `serde_json::Map` requires the `obj` to be `as_object_mut()`. This follows existing patterns in the codebase (lines 332, 338 in current code).

### 3.7 `run_status()` — lines 422-490
**Change:** Add three new checks after the existing agent teams env check (currently ends around line 468). Add after the settings.json env check block:
- Hook script file existence check (`~/.claude/hooks/great-loop/update-state.sh`)
- Hooks config presence in settings.json (string search for `"great-loop/update-state.sh"`)
- `jq` availability via `std::process::Command::new("jq").arg("--version")`

### 3.8 `run_uninstall()` — lines 492-539
**Change:** Add hook directory removal before the final summary (before line 534). Remove `~/.claude/hooks/great-loop/` with `std::fs::remove_dir_all`.

### 3.9 `mod tests` block — lines 541-870
**New unit tests to add** (spec §10.1):
- `test_hooks_value_has_all_events`
- `test_hooks_value_entries_have_async`
- `test_is_great_loop_hook_positive`
- `test_is_great_loop_hook_negative`
- `test_settings_merge_idempotent`
- `test_settings_merge_preserves_user_hooks`

Existing test at line 547 (`test_agents_count` asserting `AGENTS.len() == 15`) is unaffected. No agent count changes.

---

## 4. New Integration Test File: `tests/hook_state.rs`

New file. The spec places the integration test in `tests/cli_smoke.rs` or a new `tests/hook_state.rs`. Given the size of `cli_smoke.rs` (1871 lines), a new file is preferred.

Full test implementation in spec §10.3:
- `test_hook_writes_state_and_statusline_reads_it`
- Checks for `jq` availability (asserts if missing — fails explicitly rather than skipping)
- Simulates `SubagentStart` by piping JSON to `bash loop/hooks/update-state.sh`
- Asserts state file exists with correct `loop_id`, `agents[0].status == "running"`
- Runs `great statusline --no-color` with `session_id` on stdin, asserts output does not contain `"idle"`
- Simulates `SessionEnd`, asserts session directory removed

**Important:** `hook_script` path is derived via `env!("CARGO_MANIFEST_DIR")` — this is the established Rust test pattern for referencing source files.

---

## 5. Dependency Map

### Compile-time
```
loop/hooks/update-state.sh  <-- must exist before cargo build
  └── included by: src/cli/loop_cmd.rs (include_str! HOOK_UPDATE_STATE)
```

### Runtime (hook script)
```
update-state.sh
  ├── jq (hard dep, must be in PATH)
  ├── bash (shebang)
  ├── date +%s (POSIX)
  ├── flock (soft dep, Linux only -- absent on macOS)
  └── writes to: /tmp/great-loop/{session_id}/state.json
```

### Runtime (statusline)
```
great statusline
  ├── reads stdin: session_id field (new)
  ├── derives path: /tmp/great-loop/{session_id}/state.json
  ├── fallback: config.state_file = "/tmp/great-loop/state.json"
  └── calls: cleanup_stale_sessions() (best-effort /tmp/great-loop/ GC)
```

### Settings.json merge
```
run_install()
  ├── hooks_value() -> serde_json::json!({...})  [new]
  ├── is_great_loop_hook() -> bool               [new]
  ├── statusline_value()                         [existing, unchanged]
  └── writes: ~/.claude/settings.json
             ~/.claude/hooks/great-loop/update-state.sh  [new]
```

---

## 6. `serde_json::Value` Manipulation Patterns in Codebase

Existing patterns to follow (from current `run_install()` lines 326-362):

```rust
// Pattern 1: parse to Value
serde_json::from_str::<serde_json::Value>(&contents)

// Pattern 2: get mutable map
val.as_object_mut()

// Pattern 3: insert key
obj.insert("key".to_string(), serde_json::json!({...}))

// Pattern 4: check key presence
obj.contains_key("statusLine")

// Pattern 5: nested optional access
obj.get("statusLine").and_then(|v| v.as_object())

// Pattern 6: pretty serialize
serde_json::to_string_pretty(&val)

// Pattern 7: serde_json::json! macro
serde_json::json!({"type": "command", "command": "..."})
```

New patterns needed (spec §2.4.2):

```rust
// Pattern 8: or_insert_with (entry API)
obj.entry("env").or_insert_with(|| serde_json::json!({}))

// Pattern 9: as_object_mut on nested value
env_obj.as_object_mut()

// Pattern 10: retain on array
existing_arr.retain(|entry| !is_great_loop_hook(entry))

// Pattern 11: extend array
existing_arr.extend(new_entries.iter().cloned())
```

All patterns are within `serde_json` 1.0, already in `Cargo.toml`.

---

## 7. PathBuf Patterns in Codebase

Consistent pattern throughout `run_install()`:
```rust
let hooks_dir = claude_dir.join("hooks").join("great-loop");
let hook_script_path = hooks_dir.join("update-state.sh");
```
Uses `.join()` chaining on `std::path::Path` / `PathBuf`. All directory creation uses `std::fs::create_dir_all`. Error context uses `.context("failed to create ...")`.

---

## 8. `#[cfg(unix)]` Pattern

The chmod block in spec §2.4.1 uses:
```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&hook_script_path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook_script_path, perms)?;
}
```
This is the correct Rust pattern for Unix-only permission setting. `PermissionsExt` is in `std::os::unix::fs`, which is available on Linux and macOS but not Windows. The `#[cfg(unix)]` guard is correct because all supported Claude Code platforms are Unix.

---

## 9. Risks and Technical Debt

### Risk 1: `flock` not on macOS
**Severity:** Low. The spec explicitly addresses this with `command -v flock >/dev/null 2>&1` runtime check. Degrades to racy-but-mostly-correct last-writer-wins. Not a build or compile risk.

### Risk 2: `jq` not installed on user machine
**Severity:** Medium for UX, Low for code correctness. Hook script exits 1, Claude Code treats async hook failure as non-blocking. `great loop status` warns. No change needed in Rust code.

### Risk 3: `loop/hooks/` directory absent causes `include_str!` compile failure
**Severity:** High. The file MUST be created first. Build order is:
1. `loop/hooks/update-state.sh` (new shell script)
2. `src/cli/statusline.rs` changes
3. `src/cli/loop_cmd.rs` changes
4. New tests

### Risk 4: `serde_json` Map entry API ordering
`serde_json::Map` uses `IndexMap` internally which preserves insertion order. The `or_insert_with` entry API will insert at the end if the key is absent. This is cosmetically fine for settings.json.

### Risk 5: Existing test `test_default_config` at line 749
Asserts `state_file == "/tmp/great-loop/state.json"`. This is the fallback path, unchanged. No breakage.

### Risk 6: `SessionInfo` struct literal tests
Tests at lines 791-822 use `..Default::default()` for `SessionInfo` construction. Adding `session_id: Option<String>` with `Default` (which `Option<T>` implements as `None`) will not break these tests.

### Risk 7: `cleanup_stale_sessions()` runs every statusline tick
The spec notes this runs "~every 300ms". The implementation uses early returns for all error cases (`let Ok(...) else { return; }`). The `let-else` syntax requires Rust 1.65+. The project uses Rust 2021 edition which targets stable Rust; verify MSRV in CI if any concern. Current `Cargo.toml` has `edition = "2021"` but no explicit `rust-version` field.

### Technical Debt Noted
- `StatuslineConfig.state_file` default `/tmp/great-loop/state.json` (flat path) becomes a legacy fallback once session-scoping is active. Could be deprecated in a future iteration.
- The existing `run_install()` env warning block (lines 290-299) is being replaced with the merge logic. The manual warning ("Add to your ~/.claude/settings.json") disappears -- the new code injects automatically. This is an improvement, not debt.

---

## 10. Recommended Build Order

1. **Create `loop/hooks/update-state.sh`** — exact content from spec §1.3. Make `loop/hooks/` directory first.
2. **Modify `src/cli/statusline.rs`:**
   - Re-add `session_id` to `SessionInfo` (lines 17-28)
   - Add `cleanup_stale_sessions()` after `apply_timeout()` (~line 267)
   - Rewrite `run_inner()` (lines 144-182) for session-scoped path + cleanup call
   - Add 4 unit tests to `mod tests`
3. **Modify `src/cli/loop_cmd.rs`:**
   - Add `HOOK_UPDATE_STATE` constant after line 136
   - Add `hooks_value()` after `statusline_value()` (~line 156)
   - Add `is_great_loop_hook()` after `hooks_value()`
   - Update `collect_existing_paths()` (add hook path check, ~line 186)
   - Rewrite `run_install()` settings.json block (replace lines 290-362 with unified merge; add hook deploy block after line 288)
   - Extend `run_status()` (add 3 checks after ~line 468)
   - Update `run_uninstall()` (add hook dir removal before ~line 534)
   - Add 6 unit tests to `mod tests`
4. **Create `tests/hook_state.rs`** — full integration test from spec §10.3
5. **`cargo check`** — must pass with zero new errors
6. **`cargo clippy`** — must pass with zero new warnings
7. **`cargo test`** — all tests must pass

---

## 11. No-Change Files

Per spec §4 "No Changes":
- `Cargo.toml` — no new dependencies needed (`serde_json`, `dirs`, `colored` already present)
- `src/cli/statusline.rs` render functions — no changes to `render()`, `render_summary()`, `render_agents_wide()`, `render_agents_medium()`
- `loop/` agents, commands, teams-config, observer-template — no changes
- `tests/cli_smoke.rs` — existing tests remain valid; new tests go in `tests/hook_state.rs`
