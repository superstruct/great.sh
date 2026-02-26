# 0028: Statusline Hooks and Non-Destructive Install -- Socrates Review

**Spec:** `.tasks/ready/0028-statusline-hooks-spec.md`
**Task:** `.tasks/in-progress/0028-statusline-stuck-and-install.md`
**Reviewer:** Socrates (Adversarial Spec Reviewer)
**Date:** 2026-02-27
**Round:** 1

---

## VERDICT: REJECTED

Two BLOCKING concerns must be resolved before this spec is implementable. Both
are correctness bugs in the hook script's jq logic that would produce wrong
state file content.

---

## Part 1: Hook Script (`loop/hooks/update-state.sh`)

### BLOCK 1 -- `$name` variable dead in jq upsert; `name` field stores agent_id instead of display name

**Gap:** The script extracts two variables per event: `AGENT_KEY` (the lookup
key, e.g. `agent_id`) and `AGENT_NAME` (the display name, e.g. `agent_type`).
Both are passed to jq:

```bash
jq --arg key "$AGENT_KEY" \
   --arg name "$AGENT_NAME" \
   ...
```

But the jq body never references `$name`. When creating a new agent entry, it
writes:

```jq
"name": $key,
```

This stores the `agent_id` (e.g. `"agent-abc123"`) as the agent's `name` field,
not the `agent_type` (e.g. `"Explore"`). The `AgentState.name` field is
described in spec section 1.4 as the "display" name, and the `LoopState` schema
at section 1.2 shows `"name": "davinci"` -- clearly a display name, not an
opaque ID.

**Question:** Should the `name` field store the display name (`$name`) or the
lookup key (`$key`)? If `$name`, then the jq lookup must change to use a
different field (or `$key` must be stored in a separate field). If `$key`, then
the table in section 1.4 is wrong and the schema example at 1.2 is misleading.

**Severity:** BLOCKING

**Recommendation:** Either:
(a) Add a separate `key` field to AgentState for lookup and store `$name` in
    `name`, updating the Rust struct to include `key: Option<String>`, OR
(b) Use `$name` as both lookup key and display name (accepting that teammate
    names are unique within a team, and using `agent_type` as key for subagents
    -- noting this means two subagents of the same type cannot coexist), OR
(c) Keep `$key` as the name and update the spec's table 1.4 and schema example
    1.2 to be consistent with this choice. The statusline renderer uses
    `agent.name` only in `render_agents_wide` (line 522 prefix is `agent.id`
    not `agent.name`), so the display impact is minimal currently.

Option (c) is the simplest and introduces no Rust struct changes.

---

### BLOCK 2 -- jq upsert uses `.name` for lookup but `$key` differs from `$name`

**Gap:** This is the dual of BLOCK 1 but affects the UPDATE path (not just
INSERT). When an agent already exists, the jq lookup is:

```jq
(.agents | map(.name) | index($key)) as $idx
```

If the first `SubagentStart` stored `$key` (agent_id) as `.name`, then
`SubagentStop` with the same `agent_id` would correctly find and update it. But
if a future revision changes to store `$name` (agent_type) as `.name`, then
the lookup by `$key` (agent_id) would fail and create a duplicate.

The spec must explicitly state which value is the lookup key and ensure the
stored `.name` field matches the lookup field across ALL events. Currently the
SubagentStart path sets `AGENT_KEY=agent_id` and `AGENT_NAME=agent_type` -- if
`$key` is stored as name, the start/stop pair is consistent (both use
agent_id), but the display is wrong. If `$name` is stored as name, the lookup
is broken.

**Question:** Can you confirm the invariant: for every event pair
(SubagentStart/SubagentStop, etc.), `AGENT_KEY` is the same value, and
`.name` in the state file always equals `AGENT_KEY`?

**Severity:** BLOCKING (this is the same root cause as BLOCK 1 but affects
correctness of the update path)

**Recommendation:** Resolve together with BLOCK 1. The simplest fix: replace
`"name": $key` with `"name": $key` explicitly and document that `.name` IS the
lookup key, not a display name. If display names are desired later, add a
separate `display_name` field in a future task.

---

### CONCERN 1 -- Concurrent hook writes are not truly serialized

**Gap:** Section 1.7 claims "Claude Code serializes event delivery within a
session" but this is asserted without citation. The hooks are registered with
`"async": true`, meaning Claude Code fires them asynchronously. If two events
for the same session fire in quick succession (e.g., two SubagentStart events
for different agents), both hook script instances may read the same
`state.json`, compute independent transforms, and the last `mv` wins -- losing
one agent entry.

**Question:** Does the Claude Code hooks documentation guarantee that async
hooks for the same session are serialized (queued), or can they run in parallel?

**Severity:** ADVISORY (worst case is a transient missing agent entry that
self-corrects on the next event, as the spec notes)

**Recommendation:** Add a flock-based advisory lock if parallelism is possible:
```bash
exec 9>"${STATE_DIR}/.lock"
flock 9
# ... jq upsert ...
```
Or accept the race and document it as a known limitation.

---

### PASS -- set -euo pipefail and silent exit strategy

The script uses `set -euo pipefail` and exits 0 on missing fields. The
`// empty` jq pattern correctly produces empty strings for missing fields. The
`-z` test on AGENT_KEY catches the fallthrough. Sound.

### PASS -- Atomic write pattern

The `mv` from `state.json.tmp.$$` is POSIX-atomic on the same filesystem.
`/tmp` is always a single filesystem. The PID-scoped temp file prevents
collisions between different sessions' hook invocations. Sound.

### PASS -- SessionEnd cleanup

`rm -rf "$STATE_DIR"` correctly removes the session directory. The guard
`[[ "$EVENT" == "SessionEnd" ]]` is checked before `mkdir -p`, so cleanup
doesn't recreate the directory. Sound.

---

## Part 2: Rust Changes -- `src/cli/loop_cmd.rs`

### PASS -- Embed constant placement

The spec places `HOOK_UPDATE_STATE` after line 136 (`OBSERVER_TEMPLATE`). This
is correct -- the constant is at module scope alongside the other embedded
content constants. The `include_str!` path `"../../loop/hooks/update-state.sh"`
is correct relative to `src/cli/loop_cmd.rs`.

### PASS -- `hooks_value()` function

All 6 events are listed. The `"async": true` key compiles in `serde_json::json!`
because the key is a string literal (`"async"`), not the Rust keyword. The
closure `hook_entry` has an unused parameter `_event` but this is harmless
(prefixed with underscore). Sound.

### CONCERN 2 -- Idempotency: `modified = true` set unconditionally in hooks merge loop

**Gap:** In section 2.4.2, the hooks merge logic sets `modified = true` on
every iteration of the event loop, even when the hooks array is unchanged
(retain removes 1 entry, extend adds 1 identical entry). This causes an
unnecessary file rewrite on every `great loop install` after the first.

While `serde_json::to_string_pretty` IS deterministic for the same `Value` tree
(preserving insertion order), so the SHA256 of the file should match (satisfying
acceptance criterion #5), the unnecessary I/O is wasteful and the `modified`
flag is semantically wrong.

**Question:** Should `modified` be set conditionally (only when retain actually
removed something or a new event was inserted)?

**Severity:** ADVISORY (the acceptance test passes because the output is
byte-identical, but the logic is misleading)

**Recommendation:** Track whether `retain` actually removed any entries (compare
array length before/after) and whether the else branch inserted a new key.
Only set `modified = true` when actual changes occur.

### PASS -- `is_great_loop_hook()` deduplication

The identifier `"great-loop/update-state.sh"` is stable and specific. The
function correctly traverses the nested JSON structure. The `contains()` check
is appropriately broad -- it would match even if the path prefix changed from
`~/.claude/hooks/` to an absolute path.

### PASS -- Hook script deployment with chmod 755

The `#[cfg(unix)]` guard is appropriate. The `set_mode(0o755)` ensures
executable permission. The script is embedded via `include_str!` so the content
is always the version compiled into the binary. Sound.

### PASS -- `collect_existing_paths()` and `run_uninstall()` updates

The hook script path is added to the overwrite-confirmation flow and the
uninstall flow. Both use `claude_dir.join("hooks").join("great-loop")` which is
consistent with the deployment path. Sound.

### PASS -- `run_status()` extensions

Three new checks (hook script file, hooks in settings.json, jq availability)
are added after the existing checks. The `jq --version` check uses
`Command::new("jq")` which is the standard pattern in this codebase
(`command_exists` uses `which::which` but `run_status` directly invokes, which
is acceptable for a version check).

### CONCERN 3 -- settings.json handling replaces lines 290-362 as a unit

**Gap:** The spec says "This replaces the existing lines 290-362" but does not
show the unified replacement as a single diff. The current code has three
separate blocks: (1) env check at 290-299, (2) fresh file at 300-324, (3)
statusLine injection at 326-362. The new code unifies all three into one
read-merge-write pass. This is correct but the instruction to "replace lines
292-299" in section 2.4.2 title conflicts with the later note that says
"replaces lines 290-362."

**Question:** Is the implementer expected to replace lines 290-362 (the entire
settings.json handling block) or just lines 292-299 (the manual warning)?

**Severity:** ADVISORY (the section 2.4.2 code block is self-contained and
clearly shows the full replacement; the heading is just misleading)

**Recommendation:** Clarify the heading of 2.4.2 to say "Replace lines 290-362"
instead of "Replace lines 292-299."

---

## Part 3: Rust Changes -- `src/cli/statusline.rs`

### PASS -- `session_id` re-added to `SessionInfo`

The field is `pub session_id: Option<String>` with no `#[allow(dead_code)]`
annotation (because it IS used for path derivation, not just forward-compat).
The existing comment at line 26 is deleted. The `#[derive(Default)]` means
`session_id` defaults to `None`. Sound.

### PASS -- State path derivation in `run_inner()`

```rust
let state_file_path = match &session.session_id {
    Some(sid) if !sid.is_empty() => {
        format!("/tmp/great-loop/{}/state.json", sid)
    }
    _ => config.state_file.clone(),
};
```

The empty-string guard (`!sid.is_empty()`) handles the edge case of
`"session_id": ""`. The fallback to `config.state_file` preserves backward
compatibility. Sound.

### PASS -- `cleanup_stale_sessions()`

Uses `SystemTime::now().checked_sub()` to compute the 24-hour cutoff (avoids
underflow). Uses `let _ = std::fs::remove_dir_all(&path)` for best-effort
cleanup. Iterates only directories. Sound.

### CONCERN 4 -- `cleanup_stale_sessions()` runs on every statusline tick (~300ms)

**Gap:** The cleanup function calls `read_dir` + `metadata` on every invocation.
On a system with many session directories (unlikely but possible during
development/testing), this adds latency to every statusline render.

**Question:** Should cleanup be rate-limited (e.g., only run once per minute,
tracked via a timestamp file or in-memory state)?

**Severity:** ADVISORY (the function is best-effort and short-circuits on
missing directory; the overhead is typically one `readdir` syscall on an empty
or near-empty directory)

**Recommendation:** Accept for now. If performance becomes an issue, add a
timestamp check (e.g., only clean up if `/tmp/great-loop/.last-cleanup` mtime
is older than 5 minutes).

### CONCERN 5 -- Path traversal: session_id with `/` creates nested directories

**Gap:** The spec's security section (3.4) claims that a session_id containing
`/` would only create nested dirs under `/tmp/great-loop/` and `read_state()`
would fail to find a file. But the HOOK SCRIPT also uses the session_id to
`mkdir -p "/tmp/great-loop/${SESSION_ID}"`. A session_id of `../../etc` would
cause `mkdir -p /tmp/great-loop/../../etc` which resolves to `/etc` (the `..`
traversal escapes the base directory).

The Rust `read_state()` function checks for `..` in the path, but the BASH
hook script does NOT validate `SESSION_ID` before using it in `mkdir -p` and
file paths.

**Question:** Should the hook script validate `SESSION_ID` to contain only
alphanumeric characters, hyphens, and underscores before using it in paths?

**Severity:** ADVISORY (Claude Code generates UUID-format session IDs, so this
requires a malicious Claude Code installation or a compromised hooks stdin
pipeline -- both of which imply the attacker already has code execution)

**Recommendation:** Add a validation guard in the hook script after extracting
SESSION_ID:
```bash
if [[ ! "$SESSION_ID" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  exit 0
fi
```
This is defense-in-depth and costs one regex match.

---

## Part 4: Requirement Coverage

| Requirement | Spec Coverage | Verdict |
|-------------|---------------|---------|
| R1 -- Session-scoped state files | Part 1 (hook script) + Part 3 (statusline). Path `/tmp/great-loop/{session_id}/state.json`. | PASS |
| R2 -- Hook handler scripts | Part 1. All 6 events handled. | PASS (modulo BLOCK 1/2 on name vs key) |
| R3 -- Inject hooks into settings.json | Part 2 sections 2.2-2.4. All 6 events in hooks_value(). | PASS |
| R4 -- Non-destructive env var injection | Part 2 section 2.4.2 (env merge block). | PASS |
| R5 -- Statusline session_id extraction | Part 3 sections 3.1-3.2. | PASS |
| R6 -- great loop status checks | Part 2 section 2.5. Three new checks (hook file, hooks config, jq). | PASS |

All 6 requirements from the backlog are addressed in the spec.

---

## Part 5: Line Number Accuracy

| Spec Claim | Actual Code | Verdict |
|------------|-------------|---------|
| `read_state()` at line 238 (backlog) | Line 232 in current code | Minor drift, ADVISORY |
| `render()` at lines 596-684 (backlog) | Lines 573-692 in current code | Close enough, ADVISORY |
| `render_summary()` at line 468 (spec) | Line 430 in current code | Off by 38 lines. The spec says "line 468" but the function is at line 430. | ADVISORY |
| `SessionInfo` at lines 17-28 (spec) | Lines 17-28 in current code | Exact match | PASS |
| `run_inner()` at lines 144-182 (spec) | Lines 145-182 in current code | Off by 1 at start | PASS |
| `run_install()` settings at 290-362 (spec) | Lines 290-362 in current code | Exact match | PASS |
| `run_status()` at 422-490 (spec) | Lines 422-490 in current code | Exact match | PASS |
| `run_agents_wide` line 513 (spec 1.7) | Line 513 in current code | Exact match | PASS |

---

## Part 6: Hook Event Field Verification

The spec's Hook Events Reference table claims these extra fields:

| Event | Spec Claims | Assessment |
|-------|-------------|------------|
| `SubagentStart` | `agent_id`, `agent_type` | Plausible. No contradicting evidence. |
| `SubagentStop` | `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message` | Spec table omits `stop_hook_active` but backlog includes it. Minor inconsistency -- the script does not use `stop_hook_active` so no functional impact. |
| `TeammateIdle` | `teammate_name`, `team_name` | Plausible. |
| `TaskCompleted` | `task_id`, `task_subject`, `task_description?`, `teammate_name?`, `team_name?` | Plausible. |
| `Stop` | `stop_hook_active`, `last_assistant_message` | Consistent with docs. |
| `SessionEnd` | `reason` | Consistent with docs. |

No blocking issues. The field names used by the script (`agent_id`, `agent_type`,
`teammate_name`, `task_id`) match the table.

---

## Part 7: Testing Strategy

### PASS -- Unit tests cover key invariants

The spec provides tests for:
- `hooks_value()` has all 6 events
- Async flag is set on all hooks
- `is_great_loop_hook()` positive and negative cases
- Idempotent merge (sha256 comparison)
- User hooks preserved during merge
- `SessionInfo` with and without session_id

### PASS -- Integration test covers end-to-end flow

The `test_hook_writes_state_and_statusline_reads_it` test simulates
SubagentStart, verifies state file, verifies statusline output, simulates
SessionEnd, and verifies cleanup. The jq availability check and hook script
existence check allow graceful skip.

### CONCERN 6 -- Integration test assumes jq is installed

**Gap:** The integration test skips with `eprintln!("Skipping: jq not
available")` if jq is missing. This means CI environments without jq would
silently skip the most important test.

**Question:** Is jq guaranteed to be available in your CI environment?

**Severity:** ADVISORY (the skip is visible in test output)

**Recommendation:** Ensure CI Docker images have jq installed, or add a
`#[cfg(feature = "integration")]` gate instead of a runtime skip so CI failures
are visible.

---

## Part 8: Backlog vs Spec Discrepancies

| Item | Backlog | Spec | Assessment |
|------|---------|------|------------|
| R3 hooks example | 5 events (no TaskCompleted) | 6 events (includes TaskCompleted) | Spec is MORE complete; correct |
| SubagentStop extra fields | Includes `stop_hook_active` | Omits `stop_hook_active` from table | ADVISORY -- script doesn't use it |
| `.unwrap()` in run_status() | N/A | Line 461: `unwrap_or_default()` (existing) | PASS -- `unwrap_or_default` is safe |
| Cleanup strategy | "Alternatively, hook scripts can register SessionEnd" | Both implemented (SessionEnd + mtime cleanup) | PASS -- belt and suspenders |

---

## Summary

The spec is comprehensive, well-structured, and covers all 6 backlog
requirements with appropriate depth. The build order is correct, the
deduplication strategy is sound, and the testing strategy is thorough. However,
the hook script's jq upsert logic has a bug where the `$name` variable is
extracted but never used, causing the `name` field in the state file to store
the agent lookup key instead of the display name. This must be resolved (even
if the resolution is to document that `.name` IS the lookup key) before the
spec can be approved.

---

## Required Changes for Approval

1. **BLOCK 1+2:** Resolve the `$name` vs `$key` ambiguity in the jq upsert.
   Either:
   - (a) Use `$name` in the jq body: `"name": $name` and change the lookup to
     use a separate stored field, OR
   - (c) Keep `"name": $key` and update spec section 1.4 table's "Agent Name"
     column to reflect that `.name` stores the lookup key (agent_id, teammate_name,
     etc.), not a human-readable display name. Also update the schema example in
     section 1.2 to show `"name": "agent-abc123"` instead of `"name": "davinci"`.

2. **ADVISORY items** (recommended but not required for approval):
   - Add session_id validation in the hook script (CONCERN 5)
   - Fix `modified = true` to be conditional (CONCERN 2)
   - Clarify the 2.4.2 heading about which lines are replaced (CONCERN 3)
