# 0011: great statusline — Claude Code Multi-Agent Statusline

**Priority:** P0
**Type:** feature
**Module:** `src/cli/statusline.rs` (new) + `src/cli/mod.rs` (registration)
**Status:** pending
**Date:** 2026-02-21
**Estimated Complexity:** L

## Context

Claude Code exposes a fully programmable statusline at the bottom of its
terminal UI. It is rendered every 300ms, accepts ANSI color codes, and has no
formal timeout. For the great.sh Loop (14–16 agent orchestrator), this surface
is the ideal ambient display: which agents are running, which have completed,
which have errored — all without interrupting the active agent session.

`great statusline` is a stateless Rust subcommand. Claude Code spawns it on
each 300ms tick, passing a JSON blob on stdin (session metadata). The command
reads agent state from a well-known filesystem path, renders one line to stdout
in under 5ms, and exits. No daemon. No IPC. No state kept between calls.

The companion feature `great hooks install` (tracked separately) writes the
TeammateIdle and TaskCompleted hook handlers that update the agent state files
this command reads.

---

## Requirements

1. **Subcommand registration** — Register `great statusline` in `src/cli/mod.rs`
   following the existing `Args` struct + `pub fn run(args) -> Result<()>` pattern.

2. **Stdin JSON parsing** — Parse the Claude Code session JSON from stdin with a
   hard cap of 10ms; if stdin is empty or unparseable, render with session data
   absent (agent state still displays). Expected fields: `model`, `cost_usd`,
   `context_tokens`, `context_window`, `workspace`, `session_id`,
   `transcript_path`.

3. **Agent state file** — Read `/tmp/great-loop/state.json` (path configurable
   via `~/.config/great/statusline.toml`). Schema: array of agent objects with
   fields `id` (1–16), `name` (string), `status` (one of: `idle`, `queued`,
   `running`, `done`, `error`), `updated_at` (unix timestamp). Missing file
   treated as all-idle; malformed file prints an error segment but does not exit
   non-zero.

4. **Adaptive width rendering** — Terminal width read from `$COLUMNS` env var or
   `termsize` crate fallback:
   - Wide (>120 cols): `⚡ loop │ 1● 2● 3◌ 4✗ … │ 12✓ 2⏳ 1✗ │ cost │ ctx │ 3m42s`
   - Medium (80–120 cols): `⚡ loop │ ●●◌✗●●○● … │ 12✓ 2⏳ 1✗ │ 3m42s`
   - Narrow (<80 cols): `⚡ 12✓ 2⏳ 1✗ 1· │ 3m42s`

5. **Semantic color mapping** — Apply ANSI colors using the `colored` crate
   (already in `Cargo.toml`):
   - `running` → bright green
   - `done` → green
   - `queued` → yellow
   - `error` → bright red
   - `idle` → dim (gray)
   Respect `NO_COLOR` env var and `--no-color` flag: strip all ANSI codes.
   Respect `--no-unicode` flag: replace `●◌✗○⚡│✓⏳` with ASCII equivalents
   (`*`, `.`, `X`, `-`, `>`, `|`, `v`, `~`).

6. **TOML configuration** — Read `~/.config/great/statusline.toml` if present.
   Supported keys: `state_file` (path), `session_timeout_secs` (integer,
   default 30 — agents not updated within this window show as idle), `segments`
   (ordered list: `agents`, `summary`, `cost`, `context`, `elapsed`),
   `agent_names` (map of id → display label override). Missing config file is
   not an error; all values have defaults.

7. **Settings injection** — `great init` and a new `great configure` subcommand
   (or addition to existing `loop install`) must write the `statusLine` key to
   `~/.claude/settings.json` (global) or `.claude/settings.json` (project).
   Value: `{"command": "great statusline"}`. This is a distinct deliverable but
   must land in the same PR to avoid a chicken-and-egg issue.

---

## Acceptance Criteria

- [ ] `echo '{}' | great statusline` exits 0 and prints exactly one line to stdout
      within 5ms on a cold binary (measured with `time`).
- [ ] With a valid `/tmp/great-loop/state.json` containing 14 agents in mixed
      states, the medium-width output contains one colored indicator per agent in
      positional order (verified by stripping ANSI and counting characters).
- [ ] With `NO_COLOR=1` set, the output contains zero ANSI escape sequences
      (verified by piping through `cat -v`).
- [ ] With `--no-unicode`, all output characters are printable ASCII (verified
      by `LC_ALL=C grep -P '[^\x00-\x7F]'` returning non-zero).
- [ ] When `/tmp/great-loop/state.json` is absent, the command exits 0 and
      renders the narrow summary with all agents shown as idle.

---

## Out of Scope

- The hook handlers that write `/tmp/great-loop/state.json` — tracked as a
  companion task (`great hooks install`).
- A TUI or interactive agent dashboard.
- Site / marketing changes.
- Windows native support (WSL2 terminal inherits COLUMNS correctly; native
  cmd.exe is not a target for this command).

---

## Dependencies

- **`colored` crate** (3.0) — already in `Cargo.toml`, handles `NO_COLOR`.
- **`serde` + `serde_json`** — already present; used for stdin and state file
  parsing.
- **Terminal width** — use `$COLUMNS` env var first; add `termsize` crate
  (0.1.x, ~200 lines) only if the env var is absent and a `libc` call is
  needed. Check `Cargo.toml` before adding.
- **`toml` / `serde`** — already present for config parsing.
- **Task 0003** (CLI infrastructure) — landed. Subcommand pattern established.
- **Companion: `great hooks install`** — the state file writer. Can be stubbed
  with a hand-crafted JSON file for testing this task in isolation.

---

## Technical Notes

### State file schema (reference)

```json
{
  "loop_id": "abc123",
  "started_at": 1740134400,
  "agents": [
    { "id": 1, "name": "nightingale", "status": "done",    "updated_at": 1740134450 },
    { "id": 2, "name": "lovelace",   "status": "running",  "updated_at": 1740134480 },
    { "id": 3, "name": "socrates",   "status": "queued",   "updated_at": 1740134400 }
  ]
}
```

### Stdin JSON schema (from Claude Code)

```json
{
  "model":            "claude-opus-4-6",
  "cost_usd":         0.142,
  "context_tokens":   45230,
  "context_window":   200000,
  "workspace":        "/home/user/project",
  "session_id":       "sess_abc123",
  "transcript_path":  "/tmp/claude-transcript-abc123.jsonl"
}
```

### Context window color thresholds (community pattern)

Apply to the context segment: green (<50%), yellow (50–80%), red (>80%).

### Performance budget

| Phase              | Budget |
|--------------------|--------|
| Stdin read + parse | <3ms   |
| State file read    | <1ms   |
| Render to stdout   | <1ms   |
| Total wall clock   | <5ms   |

Use `simd-json` (or `sonic-rs`) only if profiling shows `serde_json` is the
bottleneck. Prefer the existing `serde_json` dep first.

### Community implementations to reference

- Context window color thresholds and cost display: adapt from open-source
  Claude Code statusline implementations on GitHub that use similar ANSI
  segment patterns (search: `claudecode statusline rust`).
- Positional agent encoding with colored single-char indicators is the primary
  novel contribution of this task over existing implementations.
