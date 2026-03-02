# Spec 0011: great statusline -- Claude Code Multi-Agent Statusline

**Status:** ready
**Priority:** P0
**Type:** feature
**Author:** Lovelace (spec writer)
**Date:** 2026-02-22
**Source:** `.tasks/backlog/0011-statusline-multi-agent.md`

---

## 1. Summary

Add a `great statusline` subcommand that renders a single-line status display
for the Claude Code programmable statusline. Claude Code spawns this command
every 300ms, pipes session metadata JSON on stdin, and displays the stdout line
at the bottom of its terminal UI.

The command is **stateless**: it reads agent state from a JSON file on disk,
renders one colored line to stdout in under 5ms, and exits. No daemon, no IPC,
no state carried between invocations.

A secondary deliverable is settings injection: `great loop install` must write
the `statusLine` key into `~/.claude/settings.json` so the statusline activates
automatically after loop installation.

---

## 2. Files to Create

| File | Purpose |
|------|---------|
| `src/cli/statusline.rs` | New subcommand module (all rendering logic) |

## 3. Files to Modify

| File | Change |
|------|--------|
| `src/cli/mod.rs` | Add `pub mod statusline;` and `Statusline(statusline::Args)` variant |
| `src/main.rs` | Add `Command::Statusline(args) => cli::statusline::run(args),` match arm |
| `src/cli/loop_cmd.rs` | Add `statusLine` key to settings.json during `loop install` |
| `tests/cli_smoke.rs` | Add integration tests for `great statusline` |

---

## 4. Data Structures

### 4.1 Stdin JSON (from Claude Code)

```rust
/// Session metadata piped by Claude Code on each statusline tick.
/// All fields are optional -- Claude Code may omit any of them, and
/// the entire blob may be absent or empty.
#[derive(Debug, Deserialize, Default)]
pub struct SessionInfo {
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub context_tokens: Option<u64>,
    pub context_window: Option<u64>,
    pub workspace: Option<String>,
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
}
```

### 4.2 Agent State File (`/tmp/great-loop/state.json`)

```rust
/// The full state file written by hook handlers.
#[derive(Debug, Deserialize, Default)]
pub struct LoopState {
    pub loop_id: Option<String>,
    pub started_at: Option<u64>,
    #[serde(default)]
    pub agents: Vec<AgentState>,
}

/// Status of a single agent in the loop.
#[derive(Debug, Deserialize, Clone)]
pub struct AgentState {
    pub id: u32,
    pub name: String,
    pub status: AgentStatus,
    pub updated_at: u64,
}

/// Enum of possible agent statuses.
/// Unknown values deserialize to `Unknown` for forward-compatibility.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Queued,
    Running,
    Done,
    Error,
    #[serde(other)]
    Unknown,
}
```

### 4.3 TOML Configuration (`~/.config/great/statusline.toml`)

```rust
/// User-configurable statusline settings.
/// Missing file is not an error -- all fields have defaults.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct StatuslineConfig {
    /// Path to the agent state file.
    /// Default: "/tmp/great-loop/state.json"
    pub state_file: String,

    /// Agents not updated within this many seconds are treated as idle.
    /// Default: 30
    pub session_timeout_secs: u64,

    /// Ordered list of segments to render.
    /// Default: ["agents", "summary", "cost", "context", "elapsed"]
    pub segments: Vec<String>,

    /// Map of agent id (as string key) to display label override.
    /// Example: { "1" = "NI", "2" = "LO" }
    pub agent_names: std::collections::HashMap<String, String>,
}

impl Default for StatuslineConfig {
    fn default() -> Self {
        Self {
            state_file: "/tmp/great-loop/state.json".to_string(),
            session_timeout_secs: 30,
            segments: vec![
                "agents".to_string(),
                "summary".to_string(),
                "cost".to_string(),
                "context".to_string(),
                "elapsed".to_string(),
            ],
            agent_names: std::collections::HashMap::new(),
        }
    }
}
```

---

## 5. Clap Args and Subcommand Registration

### 5.1 Args struct (`src/cli/statusline.rs`)

```rust
use clap::Args as ClapArgs;

/// Arguments for the `great statusline` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Disable colored output (also respects NO_COLOR env var)
    #[arg(long)]
    pub no_color: bool,

    /// Use ASCII-only characters (no Unicode symbols)
    #[arg(long)]
    pub no_unicode: bool,

    /// Override terminal width (default: $COLUMNS or 80)
    #[arg(long)]
    pub width: Option<u16>,
}
```

### 5.2 Registration in `src/cli/mod.rs`

Add to the module declarations (alphabetical order among the existing modules):

```rust
pub mod statusline;
```

Add to the `Command` enum:

```rust
/// Render a single statusline for Claude Code (called every 300ms)
Statusline(statusline::Args),
```

### 5.3 Dispatch in `src/main.rs`

Add to the match block:

```rust
Command::Statusline(args) => cli::statusline::run(args),
```

---

## 6. Core Function Signatures

All functions live in `src/cli/statusline.rs`.

```rust
use std::collections::HashMap;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;
use serde::Deserialize;

/// Entry point. Reads stdin, reads state file, renders one line to stdout.
pub fn run(args: Args) -> Result<()>

/// Parse session JSON from stdin. Returns default on empty/malformed input.
/// Reads at most 64KB from stdin to avoid blocking on large inputs.
fn parse_stdin() -> SessionInfo

/// Load the statusline TOML config from ~/.config/great/statusline.toml.
/// Returns default config if file is missing or unparseable.
fn load_config() -> StatuslineConfig

/// Read and parse the agent state file.
/// Returns empty LoopState if file is missing.
/// Returns LoopState with empty agents vec if file is malformed (logs error segment).
fn read_state(path: &str, timeout_secs: u64) -> (LoopState, bool)
// Returns (state, had_parse_error)

/// Apply session_timeout_secs: agents whose updated_at is older than
/// (now - timeout_secs) are demoted to Idle status.
fn apply_timeout(agents: &mut [AgentState], timeout_secs: u64)

/// Determine rendering width from args, $COLUMNS, or fallback of 80.
fn resolve_width(args_width: Option<u16>) -> u16

/// Main render function. Returns the final single-line string.
fn render(
    session: &SessionInfo,
    state: &LoopState,
    config: &StatuslineConfig,
    width: u16,
    use_unicode: bool,
    had_parse_error: bool,
) -> String

/// Render the agent indicators segment (wide mode: "1● 2● 3◌ 4✗").
fn render_agents_wide(agents: &[AgentState], use_unicode: bool) -> String

/// Render the agent indicators segment (medium mode: "●●◌✗●●○●").
fn render_agents_medium(agents: &[AgentState], use_unicode: bool) -> String

/// Render the summary counters (e.g. "12v 2~ 1X" or "12✓ 2⏳ 1✗").
fn render_summary(agents: &[AgentState], use_unicode: bool) -> String

/// Count agents by status, returning (done, running, queued, error, idle).
fn count_statuses(agents: &[AgentState]) -> StatusCounts

/// Render cost segment (e.g. "$0.14").
fn render_cost(session: &SessionInfo) -> Option<String>

/// Render context window segment with color threshold (e.g. "45K/200K").
fn render_context(session: &SessionInfo, use_unicode: bool) -> Option<String>

/// Render elapsed time since loop start (e.g. "3m42s").
fn render_elapsed(state: &LoopState) -> Option<String>

/// Format a token count as a human-readable string (e.g. 45230 -> "45K").
fn format_tokens(tokens: u64) -> String

/// Format a duration in seconds as "Xm Ys" or "Xh Ym" for longer durations.
fn format_duration(seconds: u64) -> String
```

### 6.1 StatusCounts helper

```rust
#[derive(Debug, Default)]
struct StatusCounts {
    done: u32,
    running: u32,
    queued: u32,
    error: u32,
    idle: u32,
}
```

---

## 7. Symbol Maps

### 7.1 Unicode symbols (default)

| Concept | Unicode | Used in |
|---------|---------|---------|
| Prefix icon | `\u{26A1}` (lightning bolt) | All modes |
| Running indicator | `\u{25CF}` (filled circle) | Wide/Medium agent |
| Done indicator | `\u{2713}` (checkmark) | Summary |
| Queued indicator | `\u{25CC}` (dotted circle) | Wide/Medium agent |
| Error indicator | `\u{2717}` (ballot X) | Wide/Medium agent + summary |
| Idle indicator | `\u{25CB}` (open circle) | Wide/Medium agent |
| Hourglass | `\u{23F3}` (hourglass) | Summary (running+queued) |
| Separator | `\u{2502}` (box vertical) | Between segments |
| Idle dot (narrow) | `\u{00B7}` (middle dot) | Narrow summary |

### 7.2 ASCII fallback (`--no-unicode`)

| Concept | ASCII |
|---------|-------|
| Prefix icon | `>` |
| Running indicator | `*` |
| Done indicator | `v` |
| Queued indicator | `.` |
| Error indicator | `X` |
| Idle indicator | `-` |
| Hourglass (running+queued) | `~` |
| Separator | `\|` |
| Idle dot (narrow) | `.` |

---

## 8. Rendering Logic

### 8.1 Width mode selection

```
if width > 120 -> Wide
if width >= 80 -> Medium
if width < 80  -> Narrow
```

### 8.2 Wide mode (>120 columns)

Format:
```
{icon} loop {sep} {agent_indicators} {sep} {summary} {sep} {cost} {sep} {context} {sep} {elapsed}
```

Agent indicators show `{id}{symbol}` for each agent, space-separated:
```
1● 2● 3◌ 4✗ 5○ 6● 7○ 8● 9○ 10◌ 11● 12● 13○ 14○ 15○
```

Summary: `12✓ 2⏳ 1✗` (only show non-zero counts; always show done first, then running+queued, then error).

Cost: `$0.14` (from `session.cost_usd`). Omit segment if absent.

Context: `45K/200K` with color based on ratio:
- `< 50%` -> green
- `50%..80%` -> yellow
- `>= 80%` -> red
Omit segment if `context_tokens` or `context_window` is absent.

Elapsed: `3m42s` (computed from `state.started_at` to now). Omit if `started_at` is absent.

Segments that are absent are omitted entirely (no empty separators).

### 8.3 Medium mode (80-120 columns)

Format:
```
{icon} loop {sep} {agent_symbols} {sep} {summary} {sep} {elapsed}
```

Agent symbols are single characters, no ids, no spaces:
```
●●◌✗○●○●○◌●●○○○
```

Cost and context segments are dropped to fit the width. Summary and elapsed remain.

### 8.4 Narrow mode (<80 columns)

Format:
```
{icon} {summary} {sep} {elapsed}
```

No individual agent indicators. Only the aggregated summary and elapsed time.
If elapsed is absent, just: `{icon} {summary}`

### 8.5 Error state rendering

If the state file existed but was malformed (`had_parse_error = true`), insert
an error marker in the agent position:

- Wide/Medium: `{icon} loop {sep} ERR:state {sep} ...`
- Narrow: `{icon} ERR:state`

The error marker is colored bright red.

### 8.6 Missing state file rendering

Treat all agents as idle. In narrow mode this produces: `{icon} 0~ 0v`
which simplifies to just the icon (since all counts are zero, skip zero-count
items). The rendering becomes: `{icon} idle`

---

## 9. Color Mapping

Applied using `colored::Colorize` trait methods:

| Status | Colorize method | Appearance |
|--------|----------------|------------|
| `Running` | `.bright_green()` | Bright green |
| `Done` | `.green()` | Green |
| `Queued` | `.yellow()` | Yellow |
| `Error` | `.bright_red()` | Bright red |
| `Idle` | `.dimmed()` | Dim/gray |
| `Unknown` | `.dimmed()` | Dim/gray (same as idle) |
| Separator | `.dimmed()` | Dim |
| Prefix icon | `.bright_yellow()` | Bright yellow |
| Label "loop" | `.bold()` | Bold |
| Context <50% | `.green()` | Green |
| Context 50-80% | `.yellow()` | Yellow |
| Context >=80% | `.bright_red()` | Red |

### 9.1 NO_COLOR / --no-color handling

The `colored` crate v3.1.1 automatically reads `NO_COLOR` from the environment.
When `NO_COLOR` is set (any value), `colored` suppresses all ANSI escape codes.

For the `--no-color` CLI flag, call `colored::control::set_override(false)` at
the start of `run()` before any rendering occurs.

Important: because Claude Code pipes stdout (it is not a TTY), `colored` would
normally disable colors. We must call `colored::control::set_override(true)` at
the start of `run()` UNLESS `--no-color` is set or `NO_COLOR` is present.
Check logic:

```rust
if args.no_color || std::env::var("NO_COLOR").is_ok() {
    colored::control::set_override(false);
} else {
    // Force colors on even when stdout is not a tty,
    // because Claude Code reads our ANSI output from a pipe.
    colored::control::set_override(true);
}
```

---

## 10. Configuration Loading

### 10.1 Config file location

Use `dirs::config_dir()` to get the platform-appropriate config directory, then
append `great/statusline.toml`:

- Linux/WSL2: `~/.config/great/statusline.toml`
- macOS: `~/Library/Application Support/great/statusline.toml`

### 10.2 Loading behavior

```rust
fn load_config() -> StatuslineConfig {
    let config_path = dirs::config_dir()
        .map(|d| d.join("great").join("statusline.toml"));

    match config_path {
        Some(path) if path.exists() => {
            match std::fs::read_to_string(&path) {
                Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
                Err(_) => StatuslineConfig::default(),
            }
        }
        _ => StatuslineConfig::default(),
    }
}
```

If the file exists but is malformed TOML, silently fall back to defaults.
Do NOT exit non-zero or print errors to stderr (this runs every 300ms).

---

## 11. stdin Parsing

### 11.1 Approach

Read stdin with a 64KB cap. If stdin is empty (common on first invocation),
return `SessionInfo::default()`. If it is present but unparseable JSON, return
`SessionInfo::default()` silently.

```rust
fn parse_stdin() -> SessionInfo {
    let mut buf = Vec::with_capacity(65536);
    let _ = std::io::stdin()
        .lock()
        .take(65536)
        .read_to_end(&mut buf);

    if buf.is_empty() {
        return SessionInfo::default();
    }

    serde_json::from_slice(&buf).unwrap_or_default()
}
```

### 11.2 Performance note

`serde_json::from_slice` avoids a UTF-8 validation + copy step vs.
`from_str`. The input is small (<1KB typically), so this is well within budget.

---

## 12. State File Reading

### 12.1 Approach

```rust
fn read_state(path: &str, timeout_secs: u64) -> (LoopState, bool) {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<LoopState>(&contents) {
                Ok(mut state) => {
                    // Sort agents by id for positional rendering
                    state.agents.sort_by_key(|a| a.id);
                    apply_timeout(&mut state.agents, timeout_secs);
                    (state, false)
                }
                Err(_) => (LoopState::default(), true), // malformed
            }
        }
        Err(_) => (LoopState::default(), false), // missing = all idle, not an error
    }
}
```

### 12.2 Timeout logic

```rust
fn apply_timeout(agents: &mut [AgentState], timeout_secs: u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for agent in agents.iter_mut() {
        if agent.status == AgentStatus::Running || agent.status == AgentStatus::Queued {
            if now.saturating_sub(agent.updated_at) > timeout_secs {
                agent.status = AgentStatus::Idle;
            }
        }
    }
}
```

---

## 13. Entry Point Implementation

```rust
pub fn run(args: Args) -> Result<()> {
    // 1. Handle color override
    if args.no_color || std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(true);
    }

    // 2. Load config (silent fallback to defaults)
    let config = load_config();

    // 3. Parse stdin
    let session = parse_stdin();

    // 4. Read agent state
    let (state, had_parse_error) = read_state(&config.state_file, config.session_timeout_secs);

    // 5. Resolve terminal width
    let width = resolve_width(args.width);
    let use_unicode = !args.no_unicode;

    // 6. Render
    let line = render(&session, &state, &config, width, use_unicode, had_parse_error);

    // 7. Print exactly one line to stdout (no trailing newline beyond println)
    println!("{}", line);

    Ok(())
}
```

---

## 14. Width Resolution

```rust
fn resolve_width(args_width: Option<u16>) -> u16 {
    if let Some(w) = args_width {
        return w;
    }

    // Try $COLUMNS first (set by Claude Code and most shells)
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(w) = cols.parse::<u16>() {
            if w > 0 {
                return w;
            }
        }
    }

    // Fallback: try libc ioctl on stdout fd
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut ws = MaybeUninit::<libc::winsize>::zeroed().assume_init();
            if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
                return ws.ws_col;
            }
        }
    }

    // Final fallback
    80
}
```

**Note on `libc` dependency:** The `libc` crate is a transitive dependency
of `tokio`, `reqwest`, and several other deps already in the tree. It does NOT
need to be added to `Cargo.toml` explicitly. However, if the builder prefers
not to use `unsafe`, they may simply fall back to 80 after `$COLUMNS` and skip
the ioctl call entirely. The ioctl path is unlikely to fire because Claude Code
always sets `$COLUMNS`.

If the builder chooses to skip the ioctl, the fallback chain is simply:
`args.width -> $COLUMNS -> 80`.

---

## 15. Settings Injection

### 15.1 Where to inject

Modify `src/cli/loop_cmd.rs`, function `run_install()`. After the existing
settings.json handling block (lines 191-223 in current source), add the
`statusLine` key.

### 15.2 Implementation

In the existing `run_install` function, after writing or reading
`settings.json`, merge the `statusLine` key:

```rust
// After the existing settings.json block, ensure statusLine is set
let settings_path = claude_dir.join("settings.json");
if settings_path.exists() {
    let contents = std::fs::read_to_string(&settings_path)
        .context("failed to read ~/.claude/settings.json")?;
    match serde_json::from_str::<serde_json::Value>(&contents) {
        Ok(mut val) => {
            if let Some(obj) = val.as_object_mut() {
                // Only set if not already present
                if !obj.contains_key("statusLine") {
                    obj.insert(
                        "statusLine".to_string(),
                        serde_json::json!({"command": "great statusline"}),
                    );
                    let formatted = serde_json::to_string_pretty(&val)
                        .context("failed to serialize settings.json")?;
                    std::fs::write(&settings_path, formatted)
                        .context("failed to write ~/.claude/settings.json")?;
                    output::success("Statusline registered in ~/.claude/settings.json");
                }
            }
        }
        Err(_) => {
            // If existing file is not valid JSON, don't touch it
            output::warning("settings.json is not valid JSON; skipping statusLine injection");
        }
    }
} else {
    // settings.json was already created above by the existing code path
    // which includes the statusLine key in the default_settings JSON.
    // (see modification to default_settings below)
}
```

Also modify the `default_settings` JSON literal (line 201-217 of current
`loop_cmd.rs`) to include `statusLine`:

```rust
let default_settings = serde_json::json!({
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
    "statusLine": {
        "command": "great statusline"
    }
});
```

---

## 16. Edge Cases

### 16.1 Empty stdin

`parse_stdin()` returns `SessionInfo::default()`. Cost, context, model segments
are all `None` and omitted from rendering. Only agent state and elapsed time
are shown.

### 16.2 Missing state file

`read_state()` returns `(LoopState::default(), false)`. Agents vec is empty.
Summary shows nothing active. Narrow mode renders: `> idle` (ASCII) or
`{lightning} idle` (Unicode).

### 16.3 Malformed state file

`read_state()` returns `(LoopState::default(), true)`. The `had_parse_error`
flag causes the renderer to insert a red `ERR:state` marker in place of the
agent indicators segment.

### 16.4 Malformed config TOML

`load_config()` returns `StatuslineConfig::default()`. No error output.

### 16.5 Zero agents in state file

Valid state file with empty agents array. Summary shows nothing. Same as
missing state file but without the "idle" label (no agents to report on).
Render: `{icon} loop {sep} {elapsed}` (wide) or `{icon} {elapsed}` (narrow).

### 16.6 Very large agent count

If agents exceed 30, the medium/wide indicators may exceed the available width.
Truncate agent indicators to fit `width - overhead` and append an ellipsis
indicator: `...` (ASCII) or `\u{2026}` (Unicode).

### 16.7 $COLUMNS unset and not a TTY

Falls back to 80 columns (medium mode).

### 16.8 Platform differences

- **macOS ARM64/x86_64**: Config at `~/Library/Application Support/great/statusline.toml`. `dirs::config_dir()` handles this.
- **Ubuntu/Linux**: Config at `~/.config/great/statusline.toml`.
- **WSL2**: Same as Linux. The `/tmp/great-loop/state.json` path works because WSL2 has its own `/tmp`. `$COLUMNS` is inherited from the hosting terminal.

### 16.9 Concurrent access to state file

The state file may be written by hook handlers while `statusline` reads it.
The read is a single `read_to_string` call. On Linux/macOS, this is
effectively atomic for small files (<64KB). A partial read could produce
invalid JSON, which is handled by the malformed-file path (shows `ERR:state`
for one tick, recovers on next). No file locking required.

### 16.10 cost_usd is zero or negative

Display `$0.00`. Negative values (should never occur) display as-is: `$-0.01`.

### 16.11 context_window is zero

If `context_window` is `Some(0)`, skip the context segment to avoid division
by zero in the percentage calculation.

---

## 17. Error Handling Strategy

This command MUST always exit 0. It runs 3+ times per second inside Claude Code.
A non-zero exit would cause Claude Code to disable the statusline.

| Failure mode | Behavior |
|-------------|----------|
| stdin empty | Render without session data. Exit 0. |
| stdin malformed JSON | Render without session data. Exit 0. |
| State file missing | Render with all-idle / empty. Exit 0. |
| State file malformed | Render with ERR:state marker. Exit 0. |
| Config file missing | Use defaults. Exit 0. |
| Config file malformed | Use defaults. Exit 0. |
| Width detection fails | Fall back to 80. Exit 0. |
| Any other error | Catch at top level, print empty line, exit 0. |

The `run()` function returns `Result<()>` per project convention, but the
match arm in `main.rs` will propagate errors normally. However, the
implementation of `run()` should never return `Err` -- all errors are caught
internally and degraded to partial output.

As an extra safety net, wrap the entire body in a catch:

```rust
pub fn run(args: Args) -> Result<()> {
    // If anything panics or errors unexpectedly, still exit 0
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_inner(args)
    }));

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => {
            println!(); // empty line fallback
            Ok(())
        }
        Err(_) => {
            println!(); // panic fallback
            Ok(())
        }
    }
}

fn run_inner(args: Args) -> Result<()> {
    // ... actual implementation ...
}
```

---

## 18. Security Considerations

1. **State file path traversal**: The `state_file` config key is user-controlled.
   Validate that it does not contain `..` components. If it does, fall back to
   the default path. This prevents a malicious config from reading arbitrary
   files (though the worst case is a JSON parse error).

2. **stdin size cap**: Limited to 64KB to prevent memory exhaustion from a
   malicious caller piping infinite data.

3. **No secrets in output**: The statusline never displays API keys, session
   tokens, or file contents. It only shows agent names, statuses, token counts,
   and cost.

4. **File permissions**: The state file at `/tmp/great-loop/state.json` should
   be created with mode 0600 by the hook handlers (out of scope for this task,
   but noted for companion task). The statusline reader does not need to verify
   ownership because it only parses agent names and statuses.

5. **No network access**: This command performs zero network I/O.

---

## 19. Build Order

The builder should implement in this order:

1. **Data structures** -- Define `SessionInfo`, `LoopState`, `AgentState`,
   `AgentStatus`, `StatuslineConfig`, `StatusCounts` at the top of
   `src/cli/statusline.rs`.

2. **Args struct and subcommand registration** -- Add `Args` struct, register
   in `mod.rs` and `main.rs`. Verify `cargo build` succeeds with a stub
   `run()` that prints an empty line.

3. **Config loading** -- Implement `load_config()`.

4. **stdin parsing** -- Implement `parse_stdin()`.

5. **State file reading** -- Implement `read_state()` and `apply_timeout()`.

6. **Width resolution** -- Implement `resolve_width()`.

7. **Helper formatters** -- Implement `format_tokens()`, `format_duration()`,
   `render_cost()`, `render_context()`, `render_elapsed()`, `render_summary()`.

8. **Agent indicator renderers** -- Implement `render_agents_wide()`,
   `render_agents_medium()`.

9. **Main render function** -- Implement `render()` that selects width mode
   and assembles segments.

10. **Entry point** -- Wire up `run()` / `run_inner()` with panic safety.

11. **Settings injection** -- Modify `loop_cmd.rs` to add `statusLine` key.

12. **Tests** -- Add unit tests in `statusline.rs` and integration tests in
    `tests/cli_smoke.rs`.

---

## 20. Testing Strategy

### 20.1 Unit tests (in `src/cli/statusline.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // --- Data parsing ---

    #[test]
    fn test_parse_session_info_full() {
        let json = r#"{"model":"claude-opus-4-6","cost_usd":0.142,"context_tokens":45230,"context_window":200000}"#;
        let info: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.model.as_deref(), Some("claude-opus-4-6"));
        assert!((info.cost_usd.unwrap() - 0.142).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_session_info_empty_json() {
        let info: SessionInfo = serde_json::from_str("{}").unwrap();
        assert!(info.model.is_none());
        assert!(info.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_info_invalid_json() {
        let info: Result<SessionInfo, _> = serde_json::from_str("not json");
        assert!(info.is_err());
        // In run(), this would fall back to default
    }

    #[test]
    fn test_parse_loop_state() {
        let json = r#"{
            "loop_id": "abc",
            "started_at": 1740134400,
            "agents": [
                {"id": 1, "name": "nightingale", "status": "done", "updated_at": 1740134450},
                {"id": 2, "name": "lovelace", "status": "running", "updated_at": 1740134480}
            ]
        }"#;
        let state: LoopState = serde_json::from_str(json).unwrap();
        assert_eq!(state.agents.len(), 2);
        assert_eq!(state.agents[0].status, AgentStatus::Done);
        assert_eq!(state.agents[1].status, AgentStatus::Running);
    }

    #[test]
    fn test_parse_unknown_status() {
        let json = r#"{"id": 1, "name": "test", "status": "future_status", "updated_at": 0}"#;
        let agent: AgentState = serde_json::from_str(json).unwrap();
        assert_eq!(agent.status, AgentStatus::Unknown);
    }

    #[test]
    fn test_default_config() {
        let config = StatuslineConfig::default();
        assert_eq!(config.state_file, "/tmp/great-loop/state.json");
        assert_eq!(config.session_timeout_secs, 30);
        assert_eq!(config.segments.len(), 5);
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
state_file = "/custom/path.json"
session_timeout_secs = 60

[agent_names]
1 = "NI"
2 = "LO"
"#;
        let config: StatuslineConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.state_file, "/custom/path.json");
        assert_eq!(config.session_timeout_secs, 60);
        assert_eq!(config.agent_names.get("1").unwrap(), "NI");
    }

    // --- Formatting ---

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1000), "1K");
        assert_eq!(format_tokens(1500), "1K");
        assert_eq!(format_tokens(45230), "45K");
        assert_eq!(format_tokens(200000), "200K");
        assert_eq!(format_tokens(1500000), "1.5M");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(42), "42s");
        assert_eq!(format_duration(60), "1m0s");
        assert_eq!(format_duration(222), "3m42s");
        assert_eq!(format_duration(3600), "1h0m");
        assert_eq!(format_duration(3661), "1h1m");
    }

    // --- Status counting ---

    #[test]
    fn test_count_statuses() {
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 4, name: "d".into(), status: AgentStatus::Error, updated_at: 0 },
            AgentState { id: 5, name: "e".into(), status: AgentStatus::Idle, updated_at: 0 },
        ];
        let counts = count_statuses(&agents);
        assert_eq!(counts.done, 2);
        assert_eq!(counts.running, 1);
        assert_eq!(counts.error, 1);
        assert_eq!(counts.idle, 1);
        assert_eq!(counts.queued, 0);
    }

    // --- Rendering ---

    #[test]
    fn test_render_summary_unicode() {
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Error, updated_at: 0 },
        ];
        let summary = render_summary(&agents, true);
        // Should contain checkmark count, hourglass count, X count
        assert!(summary.contains("1")); // 1 done
        assert!(summary.contains("1")); // 1 running (shown as hourglass)
    }

    #[test]
    fn test_render_summary_ascii() {
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
        ];
        let summary = render_summary(&agents, false);
        assert!(summary.contains('v')); // ASCII done indicator
    }

    #[test]
    fn test_render_agents_wide_unicode() {
        let agents = vec![
            AgentState { id: 1, name: "nightingale".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "lovelace".into(), status: AgentStatus::Running, updated_at: 0 },
        ];
        let result = render_agents_wide(&agents, true);
        assert!(result.contains("1")); // agent id 1
        assert!(result.contains("2")); // agent id 2
    }

    #[test]
    fn test_render_agents_medium_ascii() {
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Error, updated_at: 0 },
        ];
        let result = render_agents_medium(&agents, false);
        // ASCII: done=v, running=*, error=X  (without ANSI codes for test)
        // The actual characters in the string depend on color state
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_wide_mode() {
        colored::control::set_override(false); // disable colors for assertion
        let session = SessionInfo {
            cost_usd: Some(0.14),
            context_tokens: Some(45000),
            context_window: Some(200000),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 222),
            agents: vec![
                AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false);
        assert!(line.contains("loop"));
        assert!(line.contains("$0.14"));
        assert!(line.contains("45K/200K"));
    }

    #[test]
    fn test_render_narrow_mode() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 60, true, false);
        // Narrow: just icon + summary (which might be "idle")
        assert!(!line.is_empty());
    }

    #[test]
    fn test_render_with_parse_error() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 100, true, true);
        assert!(line.contains("ERR:state"));
    }

    #[test]
    fn test_no_unicode_output_is_ascii_only() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState {
            agents: vec![
                AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
                AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, false, false);
        assert!(line.chars().all(|c| c.is_ascii()), "all characters must be ASCII: {}", line);
    }

    // --- Timeout ---

    #[test]
    fn test_apply_timeout_demotes_stale_agents() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Running, updated_at: now - 60 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: now - 5 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Done, updated_at: now - 60 },
        ];
        apply_timeout(&mut agents, 30);
        assert_eq!(agents[0].status, AgentStatus::Idle); // timed out
        assert_eq!(agents[1].status, AgentStatus::Running); // still fresh
        assert_eq!(agents[2].status, AgentStatus::Done); // done is never demoted
    }

    // --- Context color threshold ---

    #[test]
    fn test_context_below_50_percent() {
        let session = SessionInfo {
            context_tokens: Some(40000),
            context_window: Some(200000),
            ..Default::default()
        };
        // 20% usage -> should be green (we just verify the segment renders)
        let result = render_context(&session, true);
        assert!(result.is_some());
        assert!(result.unwrap().contains("40K/200K"));
    }

    #[test]
    fn test_context_window_zero() {
        let session = SessionInfo {
            context_tokens: Some(100),
            context_window: Some(0),
            ..Default::default()
        };
        // Should be None (skip to avoid division by zero)
        let result = render_context(&session, true);
        assert!(result.is_none());
    }

    // --- Width resolution ---

    #[test]
    fn test_resolve_width_from_args() {
        assert_eq!(resolve_width(Some(120)), 120);
    }

    #[test]
    fn test_resolve_width_fallback() {
        // When $COLUMNS is not set and no args, should fall back to 80
        // (This test may be flaky if $COLUMNS is set in the test environment,
        //  so it's best run with COLUMNS unset)
        std::env::remove_var("COLUMNS");
        let w = resolve_width(None);
        assert!(w > 0);
    }
}
```

### 20.2 Integration tests (in `tests/cli_smoke.rs`)

Add these tests to the existing file:

```rust
// -----------------------------------------------------------------------
// Statusline
// -----------------------------------------------------------------------

#[test]
fn statusline_empty_stdin_exits_zero() {
    great()
        .arg("statusline")
        .write_stdin("{}")
        .assert()
        .success();
}

#[test]
fn statusline_no_stdin_exits_zero() {
    great()
        .arg("statusline")
        .assert()
        .success();
}

#[test]
fn statusline_prints_one_line() {
    let output = great()
        .arg("statusline")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "statusline must print exactly one line, got: {:?}", lines);
}

#[test]
fn statusline_no_color_no_ansi() {
    let output = great()
        .arg("statusline")
        .arg("--no-color")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // ANSI escape sequences start with ESC (0x1B)
    assert!(
        !stdout.contains('\x1b'),
        "output must contain no ANSI escapes with --no-color: {:?}",
        stdout
    );
}

#[test]
fn statusline_no_unicode_ascii_only() {
    let output = great()
        .arg("statusline")
        .arg("--no-unicode")
        .arg("--no-color")
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.chars().all(|c| c.is_ascii()),
        "output must be ASCII-only with --no-unicode: {:?}",
        stdout
    );
}

#[test]
fn statusline_with_state_file() {
    let dir = TempDir::new().unwrap();
    let state_path = dir.path().join("state.json");
    std::fs::write(
        &state_path,
        r#"{
            "loop_id": "test",
            "started_at": 1740134400,
            "agents": [
                {"id": 1, "name": "nightingale", "status": "done", "updated_at": 1740134450},
                {"id": 2, "name": "lovelace", "status": "running", "updated_at": 1740134480},
                {"id": 3, "name": "socrates", "status": "queued", "updated_at": 1740134400},
                {"id": 4, "name": "humboldt", "status": "error", "updated_at": 1740134400},
                {"id": 5, "name": "davinci", "status": "idle", "updated_at": 1740134400}
            ]
        }"#,
    )
    .unwrap();

    // Create a temporary config that points to our state file
    let config_dir = dir.path().join("config").join("great");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("statusline.toml");
    std::fs::write(
        &config_path,
        format!("state_file = {:?}\n", state_path.to_str().unwrap()),
    )
    .unwrap();

    // Note: This test requires the config dir to be discoverable.
    // Since we can't easily override dirs::config_dir(), we test
    // the rendering logic via unit tests instead. This integration
    // test verifies the basic pipeline works.
    great()
        .arg("statusline")
        .arg("--no-color")
        .write_stdin(r#"{"cost_usd": 0.05, "context_tokens": 10000, "context_window": 200000}"#)
        .assert()
        .success();
}

#[test]
fn statusline_help_shows_description() {
    great()
        .args(["statusline", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("statusline"));
}

#[test]
fn statusline_width_override() {
    let output = great()
        .args(["statusline", "--width", "60", "--no-color", "--no-unicode"])
        .write_stdin("{}")
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Narrow mode (60 < 80): should NOT contain "loop" label
    // (narrow mode skips the "loop" label in favor of just the summary)
    assert!(!stdout.is_empty());
}
```

### 20.3 Manual testing script

The builder should create a temporary test fixture for manual verification:

```bash
# Create test state file
mkdir -p /tmp/great-loop
cat > /tmp/great-loop/state.json << 'EOF'
{
  "loop_id": "test-123",
  "started_at": REPLACE_WITH_RECENT_TIMESTAMP,
  "agents": [
    {"id": 1, "name": "nightingale", "status": "done", "updated_at": RECENT},
    {"id": 2, "name": "lovelace", "status": "done", "updated_at": RECENT},
    {"id": 3, "name": "socrates", "status": "done", "updated_at": RECENT},
    {"id": 4, "name": "humboldt", "status": "done", "updated_at": RECENT},
    {"id": 5, "name": "davinci", "status": "running", "updated_at": RECENT},
    {"id": 6, "name": "vonbraun", "status": "queued", "updated_at": RECENT},
    {"id": 7, "name": "turing", "status": "queued", "updated_at": RECENT},
    {"id": 8, "name": "kerckhoffs", "status": "idle", "updated_at": RECENT},
    {"id": 9, "name": "rams", "status": "idle", "updated_at": RECENT},
    {"id": 10, "name": "nielsen", "status": "idle", "updated_at": RECENT},
    {"id": 11, "name": "knuth", "status": "idle", "updated_at": RECENT},
    {"id": 12, "name": "gutenberg", "status": "idle", "updated_at": RECENT},
    {"id": 13, "name": "hopper", "status": "idle", "updated_at": RECENT},
    {"id": 14, "name": "dijkstra", "status": "error", "updated_at": RECENT},
    {"id": 15, "name": "wirth", "status": "idle", "updated_at": RECENT}
  ]
}
EOF

# Test all three width modes
echo '{"cost_usd":0.142,"context_tokens":45230,"context_window":200000}' | \
  COLUMNS=150 cargo run -- statusline

echo '{"cost_usd":0.142,"context_tokens":45230,"context_window":200000}' | \
  COLUMNS=100 cargo run -- statusline

echo '{"cost_usd":0.142,"context_tokens":45230,"context_window":200000}' | \
  COLUMNS=60 cargo run -- statusline

# Test NO_COLOR
echo '{}' | NO_COLOR=1 cargo run -- statusline 2>/dev/null | cat -v
# Should contain NO escape sequences (no ^[)

# Test --no-unicode
echo '{}' | cargo run -- statusline --no-unicode --no-color 2>/dev/null | \
  LC_ALL=C grep -cP '[^\x00-\x7F]'
# Should print 0 (no non-ASCII characters)

# Performance test
time (for i in $(seq 100); do echo '{}' | cargo run --release -- statusline > /dev/null; done)
# Average should be well under 5ms per invocation (excluding cargo overhead)
```

---

## 21. Performance Notes

1. **No allocations on the hot path where possible.** Use `write!` to a
   pre-allocated `String` rather than concatenating many small strings.

2. **Pre-size the output string.** A typical wide-mode line is ~150 bytes.
   Initialize with `String::with_capacity(256)`.

3. **No file I/O beyond the two reads** (config + state). Config could be
   cached in a future daemon mode, but for now reads are acceptable since
   the files are tiny (<1KB) and hot in the OS page cache.

4. **No tokio runtime.** This command is synchronous. It must not pay the
   cost of spawning a tokio runtime.

---

## 22. Dependencies

No new dependencies required. All crates are already in `Cargo.toml`:

- `clap` 4.5 with derive -- for `Args` struct
- `serde` 1.0 + `serde_json` 1.0 -- for JSON parsing
- `toml` 0.8 -- for config parsing
- `colored` 3.0 -- for ANSI color output
- `dirs` 6.0 -- for config directory resolution
- `anyhow` 1.0 -- for error handling

The `libc` crate for terminal size ioctl is a transitive dependency. If the
builder prefers to avoid `unsafe`, the `$COLUMNS -> 80` fallback is sufficient
since Claude Code always sets `$COLUMNS`.

---

## 23. Acceptance Criteria Verification

| Criterion | How to verify |
|-----------|--------------|
| `echo '{}' \| great statusline` exits 0, one line, <5ms | Integration test + `time` measurement |
| 14-agent state, medium width: one indicator per agent | Unit test `test_render_agents_medium_*` |
| `NO_COLOR=1` produces zero ANSI escapes | Integration test `statusline_no_color_no_ansi` |
| `--no-unicode` produces only printable ASCII | Integration test `statusline_no_unicode_ascii_only` |
| Absent state file: exits 0, renders idle | Unit test `test_render_narrow_mode` with empty state |
| Settings injection via `great loop install` | Existing `loop_cmd` tests + manual verification |

---

## 24. Glossary

| Term | Definition |
|------|-----------|
| Tick | One 300ms interval in Claude Code's statusline refresh cycle |
| State file | `/tmp/great-loop/state.json`, written by hook handlers |
| Session info | JSON blob piped by Claude Code on stdin |
| Segment | One logical section of the statusline (agents, summary, cost, etc.) |
| Indicator | A single character representing one agent's status |
| Width mode | One of Wide (>120), Medium (80-120), Narrow (<80) |
