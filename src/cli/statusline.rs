use std::fmt::Write as FmtWrite;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Session metadata piped by Claude Code on each statusline tick.
/// All fields are optional -- Claude Code may omit any of them, and
/// the entire blob may be absent or empty.
///
/// Parsed exclusively via `parse_session_info_bytes` which handles
/// both the official nested schema and legacy flat fields.
#[derive(Debug, Default)]
pub struct SessionInfo {
    /// Model display name (e.g. "Opus 4.6")
    pub model_name: Option<String>,
    /// Model identifier (e.g. "claude-opus-4-6")
    #[allow(dead_code)] // Parsed for forward-compatibility; not rendered yet.
    pub model_id: Option<String>,
    /// Total accumulated cost in USD
    pub cost_usd: Option<f64>,
    /// Total duration in milliseconds
    #[allow(dead_code)] // Parsed for forward-compatibility; not rendered yet.
    pub total_duration_ms: Option<u64>,
    /// Total lines added in the session
    pub lines_added: Option<u64>,
    /// Total lines removed in the session
    pub lines_removed: Option<u64>,
    /// Total context tokens used (input + output)
    pub context_tokens: Option<u64>,
    /// Context window size in tokens
    pub context_window: Option<u64>,
    /// Pre-calculated context usage percentage from upstream
    pub used_percentage: Option<f64>,
    /// Whether the session exceeds the 200k token threshold
    #[allow(dead_code)] // Parsed for forward-compatibility; not rendered yet.
    pub exceeds_200k: Option<bool>,
    /// Session ID used for state file path derivation. Not rendered.
    pub session_id: Option<String>,
    /// Claude Code version string
    #[allow(dead_code)] // Parsed for forward-compatibility; not rendered yet.
    pub version: Option<String>,
}

/// The full state file written by hook handlers.
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)] // loop_id is deserialized but not consumed by rendering.
pub struct LoopState {
    pub loop_id: Option<String>,
    pub started_at: Option<u64>,
    #[serde(default)]
    pub agents: Vec<AgentState>,
}

/// Status of a single agent in the loop.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)] // name is deserialized for forward-compatibility but not rendered yet.
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
}

impl Default for StatuslineConfig {
    fn default() -> Self {
        Self {
            state_file: "/tmp/great-loop/state.json".to_string(),
            session_timeout_secs: 30,
        }
    }
}

/// Aggregated counts of agents by status.
#[derive(Debug, Default)]
struct StatusCounts {
    done: u32,
    running: u32,
    queued: u32,
    error: u32,
    idle: u32,
}

// ---------------------------------------------------------------------------
// Clap Args
// ---------------------------------------------------------------------------

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

    /// Use powerline glyphs (requires Nerd Fonts)
    #[arg(long)]
    pub powerline: bool,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Entry point. Wraps `run_inner` in `catch_unwind` so that panics
/// are swallowed and the process always exits 0.
pub fn run(args: Args) -> Result<()> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_inner(args)));

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(_)) => {
            println!();
            Ok(())
        }
        Err(_) => {
            println!();
            Ok(())
        }
    }
}

/// Actual implementation of the statusline subcommand.
fn run_inner(args: Args) -> Result<()> {
    // 1. Handle color override.
    //    Because Claude Code pipes stdout (not a TTY), colored would normally
    //    disable colors. Force them on unless --no-color or NO_COLOR is set.
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
    let state_file_path = match &session.session_id {
        Some(sid)
            if !sid.is_empty()
                && sid.len() <= 200
                && sid
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') =>
        {
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
        args.powerline,
        had_parse_error,
    );

    // 9. Print exactly one line to stdout
    println!("{}", line);

    Ok(())
}

// ---------------------------------------------------------------------------
// Config loading
// ---------------------------------------------------------------------------

/// Load the statusline TOML config.
/// Checks `GREAT_STATUSLINE_CONFIG` env var first (for testing), then
/// falls back to `~/.config/great/statusline.toml` (platform-appropriate).
/// Returns default config if file is missing or unparseable.
fn load_config() -> StatuslineConfig {
    let config_path = std::env::var("GREAT_STATUSLINE_CONFIG")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::config_dir().map(|d| d.join("great").join("statusline.toml")));

    match config_path {
        Some(path) if path.exists() => match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => StatuslineConfig::default(),
        },
        _ => StatuslineConfig::default(),
    }
}

// ---------------------------------------------------------------------------
// Stdin parsing
// ---------------------------------------------------------------------------

/// Parse session JSON from stdin. Returns default on empty/malformed input.
/// Reads at most 1 MiB from stdin as a runaway-input guard; Claude Code
/// payloads are far smaller, but a truncated JSON blob fails to parse and
/// blanks the whole statusline, so the cap must comfortably exceed real input.
fn parse_stdin() -> SessionInfo {
    const MAX_STDIN: u64 = 1024 * 1024;
    let mut buf = Vec::with_capacity(65536);
    let _ = std::io::stdin()
        .lock()
        .take(MAX_STDIN)
        .read_to_end(&mut buf);

    if buf.is_empty() {
        return SessionInfo::default();
    }

    parse_session_info_bytes(&buf)
}

/// Parse session JSON in a schema-tolerant way.
///
/// Supports both the official Claude Code nested schema and legacy flat fields.
/// Tries nested paths first, then falls back to flat fields for backward
/// compatibility. Each field is parsed independently so one incompatible
/// field type does not wipe out the whole payload.
fn parse_session_info_bytes(input: &[u8]) -> SessionInfo {
    let root: Value = match serde_json::from_slice(input) {
        Ok(v) => v,
        Err(_) => return SessionInfo::default(),
    };

    // model_name: /model/display_name -> /model as string
    let model_name = root
        .pointer("/model/display_name")
        .and_then(json_string)
        .or_else(|| root.get("model").and_then(json_string));

    // model_id: /model/id
    let model_id = root.pointer("/model/id").and_then(json_string);

    // cost_usd: /cost/total_cost_usd -> /cost_usd
    let cost_usd = root
        .pointer("/cost/total_cost_usd")
        .and_then(json_f64)
        .or_else(|| root.get("cost_usd").and_then(json_f64));

    // total_duration_ms: /cost/total_duration_ms
    let total_duration_ms = root.pointer("/cost/total_duration_ms").and_then(json_u64);

    // lines_added: /cost/total_lines_added
    let lines_added = root.pointer("/cost/total_lines_added").and_then(json_u64);

    // lines_removed: /cost/total_lines_removed
    let lines_removed = root.pointer("/cost/total_lines_removed").and_then(json_u64);

    // context_tokens: sum /context_window/total_input_tokens + total_output_tokens
    //   -> /context_tokens (flat)
    //   -> legacy /context_window/used_tokens, /context_window/used
    let context_tokens = {
        let input_tokens = root
            .pointer("/context_window/total_input_tokens")
            .and_then(json_u64);
        let output_tokens = root
            .pointer("/context_window/total_output_tokens")
            .and_then(json_u64);
        match (input_tokens, output_tokens) {
            (Some(i), Some(o)) => Some(i + o),
            (Some(i), None) => Some(i),
            (None, Some(o)) => Some(o),
            (None, None) => root
                .get("context_tokens")
                .and_then(json_u64)
                .or_else(|| {
                    root.pointer("/context_window/used_tokens")
                        .and_then(json_u64)
                })
                .or_else(|| root.pointer("/context_window/used").and_then(json_u64)),
        }
    };

    // context_window: /context_window/context_window_size
    //   -> /context_window as u64 (flat scalar)
    //   -> /context_window/max_tokens
    //   -> /context_window/max
    let context_window = root
        .pointer("/context_window/context_window_size")
        .and_then(json_u64)
        .or_else(|| root.get("context_window").and_then(json_u64))
        .or_else(|| {
            root.pointer("/context_window/max_tokens")
                .and_then(json_u64)
        })
        .or_else(|| root.pointer("/context_window/max").and_then(json_u64));

    // used_percentage: /context_window/used_percentage
    let used_percentage = root
        .pointer("/context_window/used_percentage")
        .and_then(json_f64);

    // exceeds_200k: /exceeds_200k_tokens
    let exceeds_200k = root.get("exceeds_200k_tokens").and_then(|v| v.as_bool());

    // session_id: /session_id
    let session_id = root.get("session_id").and_then(json_string);

    // version: /version
    let version = root.get("version").and_then(json_string);

    SessionInfo {
        model_name,
        model_id,
        cost_usd,
        total_duration_ms,
        lines_added,
        lines_removed,
        context_tokens,
        context_window,
        used_percentage,
        exceeds_200k,
        session_id,
        version,
    }
}

/// Parse a JSON value into a UTF-8 string.
fn json_string(v: &Value) -> Option<String> {
    v.as_str().map(ToString::to_string)
}

/// Parse a JSON value into a non-negative integer token count.
fn json_u64(v: &Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    if let Some(n) = v.as_i64() {
        return (n >= 0).then_some(n as u64);
    }
    if let Some(n) = v.as_f64() {
        return (n.is_finite() && n >= 0.0).then_some(n as u64);
    }
    v.as_str()?.parse::<u64>().ok()
}

/// Parse a JSON value into a floating-point number.
fn json_f64(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    v.as_str()?.parse::<f64>().ok()
}

// ---------------------------------------------------------------------------
// State file reading
// ---------------------------------------------------------------------------

/// Read and parse the agent state file.
/// Returns `(state, had_parse_error)`.
/// - Missing file: `(default, false)` -- not an error, just no agents.
/// - Malformed file: `(default, true)` -- signals renderer to show ERR:state.
fn read_state(path: &str, timeout_secs: u64) -> (LoopState, bool) {
    // Security: reject paths containing ".." to prevent traversal.
    if path.contains("..") {
        return read_state("/tmp/great-loop/state.json", timeout_secs);
    }

    match std::fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<LoopState>(&contents) {
            Ok(mut state) => {
                state.agents.sort_by_key(|a| a.id);
                apply_timeout(&mut state.agents, timeout_secs);
                (state, false)
            }
            Err(_) => (LoopState::default(), true),
        },
        Err(_) => (LoopState::default(), false),
    }
}

/// Demote Running/Queued agents to Idle if their `updated_at` is older than
/// `now - timeout_secs`.
fn apply_timeout(agents: &mut [AgentState], timeout_secs: u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    for agent in agents.iter_mut() {
        if (agent.status == AgentStatus::Running || agent.status == AgentStatus::Queued)
            && now.saturating_sub(agent.updated_at) > timeout_secs
        {
            agent.status = AgentStatus::Idle;
        }
    }
}

/// Remove session directories under `/tmp/great-loop/` whose mtime is
/// older than 24 hours. Best-effort: errors are silently ignored because
/// this runs on every statusline tick (~300ms) and must never slow it down.
fn cleanup_stale_sessions() {
    let base = std::path::Path::new("/tmp/great-loop");
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };

    let cutoff = SystemTime::now().checked_sub(std::time::Duration::from_secs(24 * 60 * 60));
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

// ---------------------------------------------------------------------------
// Width resolution
// ---------------------------------------------------------------------------

/// Determine rendering width from args, $COLUMNS, or fallback of 80.
fn resolve_width(args_width: Option<u16>) -> u16 {
    if let Some(w) = args_width {
        return w;
    }

    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(w) = cols.parse::<u16>() {
            if w > 0 {
                return w;
            }
        }
    }

    80
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Compute the visible length of a string by stripping ANSI escape sequences.
/// Counts Unicode characters (not bytes) for correct width of multi-byte chars.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Truncate a string to at most `max_visible` visible columns.
/// Preserves ANSI escape sequences but cuts visible characters.
/// Appends a reset sequence when truncating colored output so an
/// unclosed color/style cannot bleed past the statusline.
fn truncate_to_width(s: &str, max_visible: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut visible = 0;
    let mut in_escape = false;
    let mut truncated = false;
    let mut saw_escape = false;
    for c in s.chars() {
        if in_escape {
            out.push(c);
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
            saw_escape = true;
            out.push(c);
        } else {
            if visible >= max_visible {
                truncated = true;
                break;
            }
            out.push(c);
            visible += 1;
        }
    }
    if truncated && saw_escape {
        out.push_str("\x1b[0m");
    }
    out
}

/// Format a token count as a human-readable string (e.g. 45230 -> "45K").
/// Kept for potential future use even though context rendering now uses percentages.
#[allow(dead_code)]
fn format_tokens(tokens: u64) -> String {
    if tokens < 1000 {
        return tokens.to_string();
    }
    if tokens < 1_000_000 {
        let k = tokens / 1000;
        return format!("{}K", k);
    }
    let m = tokens as f64 / 1_000_000.0;
    format!("{:.1}M", m)
}

/// Format a duration in seconds as "Xm Ys" or "Xh Ym" for longer durations.
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    if seconds < 3600 {
        let m = seconds / 60;
        let s = seconds % 60;
        return format!("{}m{}s", m, s);
    }
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    format!("{}h{}m", h, m)
}

/// Count agents by status, returning aggregated `StatusCounts`.
fn count_statuses(agents: &[AgentState]) -> StatusCounts {
    let mut counts = StatusCounts::default();
    for agent in agents {
        match agent.status {
            AgentStatus::Done => counts.done += 1,
            AgentStatus::Running => counts.running += 1,
            AgentStatus::Queued => counts.queued += 1,
            AgentStatus::Error => counts.error += 1,
            AgentStatus::Idle | AgentStatus::Unknown => counts.idle += 1,
        }
    }
    counts
}

// ---------------------------------------------------------------------------
// Separator
// ---------------------------------------------------------------------------

/// Create the separator string based on mode flags.
fn make_separator(use_unicode: bool, powerline: bool) -> String {
    if powerline {
        " \u{E0B1} ".to_string()
    } else if use_unicode {
        format!(" {} ", "\u{2502}".dimmed())
    } else {
        format!(" {} ", "|".dimmed())
    }
}

// ---------------------------------------------------------------------------
// Segment renderers
// ---------------------------------------------------------------------------

/// Render cost segment (e.g. "$0.14"). Two decimal places for consistency.
fn render_cost(session: &SessionInfo) -> Option<String> {
    session.cost_usd.map(|c| format!("${:.2}", c))
}

/// Render context bar with percentage.
/// Wide (>120): [####------] 42%
/// Medium/narrow: 42%
fn render_context_bar(session: &SessionInfo, width: u16, use_unicode: bool) -> Option<String> {
    // Get percentage: prefer used_percentage, fallback to calculation
    let pct = session.used_percentage.or_else(|| {
        let tokens = session.context_tokens? as f64;
        let window = session.context_window? as f64;
        if window == 0.0 {
            return None;
        }
        Some((tokens / window) * 100.0)
    })?;

    let pct_clamped = pct.clamp(0.0, 100.0);

    // Color based on percentage
    let apply_color = |s: String| -> String {
        if pct_clamped >= 80.0 {
            s.bright_red().to_string()
        } else if pct_clamped >= 50.0 {
            s.yellow().to_string()
        } else {
            s.green().to_string()
        }
    };

    if width > 120 {
        // Wide: [####------] 42%
        let bar_width = 10;
        let filled = ((pct_clamped / 100.0) * bar_width as f64).round() as usize;
        let empty = bar_width - filled;
        let (fill_char, empty_char) = if use_unicode {
            ("\u{2588}", "\u{2591}")
        } else {
            ("#", "-")
        };
        let bar = format!(
            "[{}{}] {}%",
            fill_char.repeat(filled),
            empty_char.repeat(empty),
            pct_clamped as u32
        );
        Some(apply_color(bar))
    } else {
        // Medium/narrow: just "42%"
        Some(apply_color(format!("{}%", pct_clamped as u32)))
    }
}

/// Render the model display name, dimmed.
fn render_model(session: &SessionInfo) -> Option<String> {
    session.model_name.as_ref().map(|m| m.dimmed().to_string())
}

/// Render lines changed segment (e.g. "+12 -3").
fn render_lines_changed(session: &SessionInfo) -> Option<String> {
    let added = session.lines_added.unwrap_or(0);
    let removed = session.lines_removed.unwrap_or(0);
    if added == 0 && removed == 0 {
        return None;
    }
    let mut parts = Vec::new();
    if added > 0 {
        parts.push(format!("+{}", added).green().to_string());
    }
    if removed > 0 {
        parts.push(format!("-{}", removed).red().to_string());
    }
    Some(parts.join(" "))
}

/// Render elapsed time since loop start (e.g. "3m42s").
fn render_elapsed(state: &LoopState) -> Option<String> {
    let started = state.started_at?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let elapsed = now.saturating_sub(started);
    Some(format_duration(elapsed))
}

/// Render the summary counters using the same symbol vocabulary as per-agent
/// indicators for consistency. Shows done, running, queued, and error counts
/// separately. Only non-zero counts are shown.
fn render_summary(agents: &[AgentState], use_unicode: bool) -> String {
    let counts = count_statuses(agents);
    let mut parts: Vec<String> = Vec::new();

    let (done_sym, running_sym, queued_sym, error_sym) = if use_unicode {
        ("\u{2713}", "\u{25CF}", "\u{25CC}", "\u{2717}")
    } else {
        ("v", "*", ".", "X")
    };

    if counts.done > 0 {
        parts.push(format!("{}{}", counts.done, done_sym).green().to_string());
    }

    if counts.running > 0 {
        parts.push(
            format!("{}{}", counts.running, running_sym)
                .bright_green()
                .to_string(),
        );
    }

    if counts.queued > 0 {
        parts.push(
            format!("{}{}", counts.queued, queued_sym)
                .yellow()
                .to_string(),
        );
    }

    if counts.error > 0 {
        parts.push(
            format!("{}{}", counts.error, error_sym)
                .bright_red()
                .to_string(),
        );
    }

    if parts.is_empty() {
        return "idle".dimmed().to_string();
    }

    parts.join(" ")
}

// ---------------------------------------------------------------------------
// Agent indicator renderers
// ---------------------------------------------------------------------------

/// Status symbol for an agent.
fn status_symbol(status: AgentStatus, use_unicode: bool) -> &'static str {
    if use_unicode {
        match status {
            AgentStatus::Running => "\u{25CF}", // filled circle
            AgentStatus::Done => "\u{2713}",    // checkmark
            AgentStatus::Queued => "\u{25CC}",  // dotted circle
            AgentStatus::Error => "\u{2717}",   // ballot X
            AgentStatus::Idle | AgentStatus::Unknown => "\u{25CB}", // open circle
        }
    } else {
        match status {
            AgentStatus::Running => "*",
            AgentStatus::Done => "v",
            AgentStatus::Queued => ".",
            AgentStatus::Error => "X",
            AgentStatus::Idle | AgentStatus::Unknown => "-",
        }
    }
}

/// Apply color to a status symbol string.
fn colorize_status(text: &str, status: AgentStatus) -> String {
    match status {
        AgentStatus::Running => text.bright_green().to_string(),
        AgentStatus::Done => text.green().to_string(),
        AgentStatus::Queued => text.yellow().to_string(),
        AgentStatus::Error => text.bright_red().to_string(),
        AgentStatus::Idle | AgentStatus::Unknown => text.dimmed().to_string(),
    }
}

/// Render the agent indicators segment (wide mode: "1X 2X 3X 4X").
fn render_agents_wide(agents: &[AgentState], use_unicode: bool) -> String {
    let max_agents = 30;
    let mut out = String::with_capacity(agents.len() * 5);
    let display_count = agents.len().min(max_agents);

    for (i, agent) in agents.iter().take(display_count).enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let sym = status_symbol(agent.status, use_unicode);
        let indicator = format!("{}{}", agent.id, sym);
        let _ = write!(out, "{}", colorize_status(&indicator, agent.status));
    }

    if agents.len() > max_agents {
        let ellipsis = if use_unicode { "\u{2026}" } else { "..." };
        let _ = write!(out, " {}", ellipsis.dimmed());
    }

    out
}

/// Render the agent indicators segment (medium mode: compact symbols).
fn render_agents_medium(agents: &[AgentState], use_unicode: bool) -> String {
    let max_agents = 30;
    let mut out = String::with_capacity(agents.len() * 4);
    let display_count = agents.len().min(max_agents);

    for agent in agents.iter().take(display_count) {
        let sym = status_symbol(agent.status, use_unicode);
        let _ = write!(out, "{}", colorize_status(sym, agent.status));
    }

    if agents.len() > max_agents {
        let ellipsis = if use_unicode { "\u{2026}" } else { "..." };
        let _ = write!(out, "{}", ellipsis.dimmed());
    }

    out
}

// ---------------------------------------------------------------------------
// Three-state loop detection
// ---------------------------------------------------------------------------

/// Returns true if a loop is present (has agents or a started_at timestamp).
fn has_loop(state: &LoopState) -> bool {
    !state.agents.is_empty() || state.started_at.is_some()
}

/// Returns true if any agent is in an active state (Running, Queued, or Error).
fn is_loop_active(agents: &[AgentState]) -> bool {
    agents.iter().any(|a| {
        matches!(
            a.status,
            AgentStatus::Running | AgentStatus::Queued | AgentStatus::Error
        )
    })
}

// ---------------------------------------------------------------------------
// Main render function
// ---------------------------------------------------------------------------

/// Main render function. Returns the final single-line string.
/// The output is truncated to `width` visible columns to prevent line wrapping.
///
/// Three display states:
/// - State A (no loop): session stats only
/// - State B (loop idle): collapsed agent summary + session stats
/// - State C (loop active): full dashboard with agent details
fn render(
    session: &SessionInfo,
    state: &LoopState,
    _config: &StatuslineConfig,
    width: u16,
    use_unicode: bool,
    powerline: bool,
    had_parse_error: bool,
) -> String {
    let mut out = String::with_capacity(256);
    let w = width as usize;

    let icon = if use_unicode {
        "\u{26A1}".bright_yellow().to_string()
    } else {
        ">".bright_yellow().to_string()
    };

    let sep = make_separator(use_unicode, powerline);

    let loop_present = has_loop(state);
    let loop_active = loop_present && is_loop_active(&state.agents);

    if had_parse_error {
        // ERR:state rendering -- same for all states
        if loop_present {
            let _ = write!(out, "{} {}", icon, "loop".bold());
        } else {
            let _ = write!(out, "{}", icon);
        }
        let _ = write!(out, "{}{}", sep, "ERR:state".bright_red());
    } else if !loop_present {
        // State A: No loop -- session stats only, no icon, no "loop" label
        render_state_a(&mut out, session, &sep, width, use_unicode);
    } else if !loop_active {
        // State B: Loop idle -- collapsed summary
        render_state_b(&mut out, session, state, &icon, &sep, width, use_unicode);
    } else {
        // State C: Loop active -- full dashboard
        render_state_c(&mut out, session, state, &icon, &sep, width, use_unicode);
    }

    // Final overflow guard -- truncate to terminal width
    if w > 0 && visible_len(&out) > w {
        out = truncate_to_width(&out, w);
    }

    out
}

/// State A: No loop present. Show session stats only.
fn render_state_a(
    out: &mut String,
    session: &SessionInfo,
    sep: &str,
    width: u16,
    use_unicode: bool,
) {
    let mut segments: Vec<String> = Vec::new();

    if let Some(ctx) = render_context_bar(session, width, use_unicode) {
        segments.push(ctx);
    }
    if let Some(cost) = render_cost(session) {
        segments.push(cost);
    }

    if width > 120 {
        // Wide: context bar | cost | lines changed | model
        if let Some(lines) = render_lines_changed(session) {
            segments.push(lines);
        }
        if let Some(model) = render_model(session) {
            segments.push(model);
        }
    } else if width >= 80 {
        // Medium: context % | cost | lines changed
        if let Some(lines) = render_lines_changed(session) {
            segments.push(lines);
        }
    }
    // Narrow (<80): context % | cost

    let _ = write!(out, "{}", segments.join(sep));
}

/// State B: Loop idle (all agents done). Collapsed display.
fn render_state_b(
    out: &mut String,
    session: &SessionInfo,
    state: &LoopState,
    icon: &str,
    sep: &str,
    width: u16,
    use_unicode: bool,
) {
    // Icon + collapsed summary
    let summary = render_summary(&state.agents, use_unicode);
    let _ = write!(out, "{} {}", icon, summary);

    if let Some(ctx) = render_context_bar(session, width, use_unicode) {
        let _ = write!(out, "{}{}", sep, ctx);
    }
    if let Some(cost) = render_cost(session) {
        let _ = write!(out, "{}{}", sep, cost);
    }

    if width > 120 {
        // Wide: also show lines changed + model
        if let Some(lines) = render_lines_changed(session) {
            let _ = write!(out, "{}{}", sep, lines);
        }
        if let Some(model) = render_model(session) {
            let _ = write!(out, "{}{}", sep, model);
        }
    }
}

/// State C: Loop active. Full dashboard with agent details.
fn render_state_c(
    out: &mut String,
    session: &SessionInfo,
    state: &LoopState,
    icon: &str,
    sep: &str,
    width: u16,
    use_unicode: bool,
) {
    let _ = write!(out, "{} {}", icon, "loop".bold());

    if width > 120 {
        // Wide: icon loop | agents_wide | summary | context bar | cost | elapsed
        let wide_agents = render_agents_wide(&state.agents, use_unicode);
        let summary = render_summary(&state.agents, use_unicode);

        // Estimate overhead for budget calculation
        let overhead = 7 + 3 + 3 + visible_len(&summary);
        let agents_budget = (width as usize).saturating_sub(overhead + 20);

        if visible_len(&wide_agents) <= agents_budget {
            let _ = write!(out, "{}{}", sep, wide_agents);
        } else {
            let _ = write!(
                out,
                "{}{}",
                sep,
                render_agents_medium(&state.agents, use_unicode)
            );
        }
        let _ = write!(out, "{}{}", sep, summary);
    } else if width >= 80 {
        // Medium: icon loop | medium_agents summary | context % | cost | elapsed
        let _ = write!(
            out,
            "{}{}",
            sep,
            render_agents_medium(&state.agents, use_unicode)
        );
        let _ = write!(out, " {}", render_summary(&state.agents, use_unicode));
    } else {
        // Narrow: icon summary | context % | cost
        let _ = write!(out, "{}{}", sep, render_summary(&state.agents, use_unicode));
    }

    if let Some(ctx) = render_context_bar(session, width, use_unicode) {
        let _ = write!(out, "{}{}", sep, ctx);
    }
    if let Some(cost) = render_cost(session) {
        let _ = write!(out, "{}{}", sep, cost);
    }
    if let Some(elapsed) = render_elapsed(state) {
        let _ = write!(out, "{}{}", sep, elapsed);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Data parsing ---

    #[test]
    fn test_parse_session_info_full() {
        let json = r#"{"model":{"id":"claude-opus-4-6","display_name":"Opus 4.6"},"cost":{"total_cost_usd":0.142},"context_window":{"context_window_size":200000,"total_input_tokens":40000,"total_output_tokens":5230}}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.model_name.as_deref(), Some("Opus 4.6"));
        assert!((info.cost_usd.unwrap() - 0.142).abs() < f64::EPSILON);
        assert_eq!(info.context_tokens, Some(45230));
        assert_eq!(info.context_window, Some(200000));
    }

    #[test]
    fn test_parse_session_info_empty_json() {
        let info = parse_session_info_bytes(b"{}");
        assert!(info.model_name.is_none());
        assert!(info.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_info_invalid_json() {
        let info = parse_session_info_bytes(b"not json");
        assert!(info.model_name.is_none());
        assert!(info.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_info_context_window_object() {
        let json = r#"{
            "session_id": "sess-123",
            "cost":{"total_cost_usd":0.12},
            "context_window": {
                "total_input_tokens": 40000,
                "total_output_tokens": 5230,
                "context_window_size": 200000,
                "used_percentage": 22.6
            }
        }"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.session_id.as_deref(), Some("sess-123"));
        assert!((info.cost_usd.unwrap() - 0.12).abs() < f64::EPSILON);
        assert_eq!(info.context_tokens, Some(45230));
        assert_eq!(info.context_window, Some(200000));
    }

    #[test]
    fn test_parse_session_info_keeps_session_id_on_partial_schema_mismatch() {
        let json = r#"{
            "session_id": "sess-456",
            "context_window": {"used_percentage": 50}
        }"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.session_id.as_deref(), Some("sess-456"));
        assert_eq!(info.context_tokens, None);
        assert_eq!(info.context_window, None);
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
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
state_file = "/custom/path.json"
session_timeout_secs = 60
"#;
        let config: StatuslineConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.state_file, "/custom/path.json");
        assert_eq!(config.session_timeout_secs, 60);
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

    // --- Cost formatting ---

    #[test]
    fn test_render_cost_two_decimals() {
        let session = SessionInfo {
            cost_usd: Some(0.142857),
            ..Default::default()
        };
        assert_eq!(render_cost(&session).unwrap(), "$0.14");
    }

    #[test]
    fn test_render_cost_zero() {
        let session = SessionInfo {
            cost_usd: Some(0.0),
            ..Default::default()
        };
        assert_eq!(render_cost(&session).unwrap(), "$0.00");
    }

    #[test]
    fn test_render_cost_whole_number() {
        let session = SessionInfo {
            cost_usd: Some(1.5),
            ..Default::default()
        };
        assert_eq!(render_cost(&session).unwrap(), "$1.50");
    }

    #[test]
    fn test_render_cost_absent() {
        let session = SessionInfo::default();
        assert!(render_cost(&session).is_none());
    }

    // --- Status counting ---

    #[test]
    fn test_count_statuses() {
        let agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 3,
                name: "c".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
            AgentState {
                id: 4,
                name: "d".into(),
                status: AgentStatus::Error,
                updated_at: 0,
            },
            AgentState {
                id: 5,
                name: "e".into(),
                status: AgentStatus::Idle,
                updated_at: 0,
            },
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
        colored::control::set_override(false);
        let agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
            AgentState {
                id: 3,
                name: "c".into(),
                status: AgentStatus::Error,
                updated_at: 0,
            },
            AgentState {
                id: 4,
                name: "d".into(),
                status: AgentStatus::Queued,
                updated_at: 0,
            },
        ];
        let summary = render_summary(&agents, true);
        assert!(summary.contains('\u{2713}')); // checkmark (done)
        assert!(summary.contains('\u{25CF}')); // filled circle (running)
        assert!(summary.contains('\u{25CC}')); // dotted circle (queued)
        assert!(summary.contains('\u{2717}')); // ballot X (error)
    }

    #[test]
    fn test_render_summary_ascii() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
            AgentState {
                id: 3,
                name: "c".into(),
                status: AgentStatus::Queued,
                updated_at: 0,
            },
        ];
        let summary = render_summary(&agents, false);
        assert!(summary.contains('v')); // done
        assert!(summary.contains('*')); // running
        assert!(summary.contains('.')); // queued
    }

    #[test]
    fn test_render_agents_wide_unicode() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState {
                id: 1,
                name: "nightingale".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "lovelace".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
        ];
        let result = render_agents_wide(&agents, true);
        assert!(result.contains('1'));
        assert!(result.contains('2'));
    }

    #[test]
    fn test_render_agents_medium_ascii() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
            AgentState {
                id: 3,
                name: "c".into(),
                status: AgentStatus::Error,
                updated_at: 0,
            },
        ];
        let result = render_agents_medium(&agents, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_wide_mode() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(22.5),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - 222,
            ),
            agents: vec![
                AgentState {
                    id: 1,
                    name: "a".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
                AgentState {
                    id: 2,
                    name: "b".into(),
                    status: AgentStatus::Running,
                    updated_at: 0,
                },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false, false);
        assert!(
            line.contains("loop"),
            "active loop should show 'loop': {}",
            line
        );
        assert!(line.contains("$0.14"), "should show cost: {}", line);
        assert!(line.contains("22%"), "should show context %: {}", line);
    }

    #[test]
    fn test_render_narrow_mode() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 60, true, false, false);
        // State A with no data -- may be empty or minimal
        assert!(visible_len(&line) <= 60);
    }

    #[test]
    fn test_render_with_parse_error() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 100, true, false, true);
        assert!(line.contains("ERR:state"));
    }

    #[test]
    fn test_no_unicode_output_is_ascii_only() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(30.0),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(0),
            agents: vec![
                AgentState {
                    id: 1,
                    name: "a".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
                AgentState {
                    id: 2,
                    name: "b".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, false, false, false);
        assert!(line.is_ascii(), "all characters must be ASCII: {}", line);
    }

    // --- Timeout ---

    #[test]
    fn test_apply_timeout_demotes_stale_agents() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Running,
                updated_at: now - 60,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Running,
                updated_at: now - 5,
            },
            AgentState {
                id: 3,
                name: "c".into(),
                status: AgentStatus::Done,
                updated_at: now - 60,
            },
        ];
        apply_timeout(&mut agents, 30);
        assert_eq!(agents[0].status, AgentStatus::Idle);
        assert_eq!(agents[1].status, AgentStatus::Running);
        assert_eq!(agents[2].status, AgentStatus::Done);
    }

    // --- Width resolution ---

    #[test]
    fn test_resolve_width_from_args() {
        assert_eq!(resolve_width(Some(120)), 120);
    }

    // --- Truncation at >30 agents ---

    #[test]
    fn test_render_agents_wide_truncation() {
        colored::control::set_override(false);
        let agents: Vec<AgentState> = (1..=35)
            .map(|i| AgentState {
                id: i,
                name: format!("agent{}", i),
                status: AgentStatus::Running,
                updated_at: 0,
            })
            .collect();
        let result = render_agents_wide(&agents, false);
        assert!(result.contains("30"));
        assert!(!result.contains("31"));
        assert!(result.contains("..."));
    }

    #[test]
    fn test_render_agents_medium_truncation() {
        colored::control::set_override(false);
        let agents: Vec<AgentState> = (1..=35)
            .map(|i| AgentState {
                id: i,
                name: format!("agent{}", i),
                status: AgentStatus::Done,
                updated_at: 0,
            })
            .collect();
        let result = render_agents_medium(&agents, true);
        assert!(result.contains('\u{2026}'));
    }

    // --- Path traversal security ---

    #[test]
    fn test_read_state_rejects_path_traversal() {
        let (state, err) = read_state("/tmp/../etc/passwd", 30);
        let (_default_state, _) = read_state("/tmp/great-loop/state.json", 30);
        let (_, default_err) = read_state("/tmp/great-loop/state.json", 30);
        assert_eq!(
            err, default_err,
            "path traversal should fall back to default path behavior"
        );
        assert_eq!(state.agents.len(), _default_state.agents.len());
    }

    // --- Three-state idle rendering ---

    #[test]
    fn test_render_idle_when_no_agents() {
        // State A: no loop present. Should NOT show "idle" or "loop" -- just session stats.
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();

        let wide = render(&session, &state, &config, 150, true, false, false);
        assert!(
            !wide.contains("idle"),
            "State A should not contain 'idle': {}",
            wide
        );
        assert!(
            !wide.contains("loop"),
            "State A should not contain 'loop': {}",
            wide
        );
    }

    // --- Medium mode includes cost+context ---

    #[test]
    fn test_render_medium_mode_includes_cost_and_context() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(22.5),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(0),
            agents: vec![AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            }],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 100, true, false, false);
        assert!(
            line.contains("$0.14"),
            "medium mode must show cost: {}",
            line
        );
        assert!(
            line.contains("22%"),
            "medium mode must show context %: {}",
            line
        );
    }

    // --- Summary uses per-agent vocabulary ---

    #[test]
    fn test_render_summary_splits_running_and_queued() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Running,
                updated_at: 0,
            },
            AgentState {
                id: 2,
                name: "b".into(),
                status: AgentStatus::Queued,
                updated_at: 0,
            },
        ];
        let summary = render_summary(&agents, false);
        assert!(
            summary.contains('*'),
            "should show running indicator: {}",
            summary
        );
        assert!(
            summary.contains('.'),
            "should show queued indicator: {}",
            summary
        );
    }

    // --- Overflow guard ---

    #[test]
    fn test_visible_len_strips_ansi() {
        assert_eq!(visible_len("hello"), 5);
        assert_eq!(visible_len("\x1b[31mred\x1b[0m"), 3);
        assert_eq!(visible_len(""), 0);
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("hello world", 5), "hello");
        let colored = "\x1b[31mred text\x1b[0m";
        let truncated = truncate_to_width(colored, 3);
        assert!(truncated.contains("\x1b[31m"));
        assert_eq!(visible_len(&truncated), 3);
    }

    #[test]
    fn test_render_wide_many_agents_does_not_exceed_width() {
        colored::control::set_override(false);
        let agents: Vec<AgentState> = (1..=25)
            .map(|i| AgentState {
                id: i,
                name: format!("agent{}", i),
                status: AgentStatus::Running,
                updated_at: 0,
            })
            .collect();
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(22.5),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - 60,
            ),
            agents,
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 130, false, false, false);
        assert!(
            visible_len(&line) <= 130,
            "output must not exceed width 130, got {} visible chars: {}",
            visible_len(&line),
            line
        );
    }

    #[test]
    fn test_render_wide_err_state_no_leaked_segments() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(22.5),
            ..Default::default()
        };
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false, true);
        assert!(line.contains("ERR:state"));
        assert!(
            !line.contains("$0.14"),
            "cost must not leak after ERR:state: {}",
            line
        );
        assert!(
            !line.contains("22%"),
            "context must not leak after ERR:state: {}",
            line
        );
    }

    #[test]
    fn test_session_info_with_session_id() {
        let json = r#"{"session_id":"abc-123","cost_usd":0.5}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.session_id.as_deref(), Some("abc-123"));
        assert!((info.cost_usd.unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_session_info_without_session_id() {
        let json = r#"{"cost_usd":0.5}"#;
        let info = parse_session_info_bytes(json.as_bytes());
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
        cleanup_stale_sessions();
    }

    // --- New tests for official schema support ---

    #[test]
    fn test_parse_official_schema() {
        let json = r#"{"model":{"id":"claude-opus-4-6","display_name":"Opus 4.6"},"cost":{"total_cost_usd":0.14,"total_duration_ms":5000,"total_lines_added":12,"total_lines_removed":3},"context_window":{"context_window_size":200000,"total_input_tokens":80000,"total_output_tokens":4000,"used_percentage":42.0},"session_id":"sess-1","version":"1.0.0"}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.model_name.as_deref(), Some("Opus 4.6"));
        assert_eq!(info.model_id.as_deref(), Some("claude-opus-4-6"));
        assert!((info.cost_usd.unwrap() - 0.14).abs() < f64::EPSILON);
        assert_eq!(info.total_duration_ms, Some(5000));
        assert_eq!(info.lines_added, Some(12));
        assert_eq!(info.lines_removed, Some(3));
        assert_eq!(info.context_tokens, Some(84000)); // 80000 + 4000
        assert_eq!(info.context_window, Some(200000));
        assert!((info.used_percentage.unwrap() - 42.0).abs() < f64::EPSILON);
        assert_eq!(info.session_id.as_deref(), Some("sess-1"));
        assert_eq!(info.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_parse_backward_compat_flat() {
        let json = r#"{"model":"claude-opus-4-6","cost_usd":0.14,"context_tokens":45000,"context_window":200000}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.model_name.as_deref(), Some("claude-opus-4-6"));
        assert!((info.cost_usd.unwrap() - 0.14).abs() < f64::EPSILON);
        assert_eq!(info.context_tokens, Some(45000));
        assert_eq!(info.context_window, Some(200000));
    }

    #[test]
    fn test_parse_model_nested_object() {
        let json = r#"{"model":{"id":"claude-opus-4-6","display_name":"Opus 4.6"}}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert_eq!(info.model_name.as_deref(), Some("Opus 4.6"));
        assert_eq!(info.model_id.as_deref(), Some("claude-opus-4-6"));
    }

    #[test]
    fn test_parse_cost_nested() {
        let json = r#"{"cost":{"total_cost_usd":1.23}}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert!((info.cost_usd.unwrap() - 1.23).abs() < f64::EPSILON);
    }

    #[test]
    fn test_render_context_bar_green() {
        colored::control::set_override(false);
        let session = SessionInfo {
            used_percentage: Some(30.0),
            ..Default::default()
        };
        let result = render_context_bar(&session, 150, true);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.contains("30%"), "should contain 30%: {}", r);
        assert!(r.contains('\u{2588}'), "wide mode should have bar: {}", r);
    }

    #[test]
    fn test_render_context_bar_yellow() {
        colored::control::set_override(false);
        let session = SessionInfo {
            used_percentage: Some(65.0),
            ..Default::default()
        };
        let result = render_context_bar(&session, 150, true);
        assert!(result.unwrap().contains("65%"));
    }

    #[test]
    fn test_render_context_bar_red() {
        colored::control::set_override(false);
        let session = SessionInfo {
            used_percentage: Some(85.0),
            ..Default::default()
        };
        let result = render_context_bar(&session, 150, true);
        assert!(result.unwrap().contains("85%"));
    }

    #[test]
    fn test_render_context_bar_ascii() {
        colored::control::set_override(false);
        let session = SessionInfo {
            used_percentage: Some(50.0),
            ..Default::default()
        };
        let result = render_context_bar(&session, 150, false);
        let r = result.unwrap();
        assert!(r.contains('#'), "ASCII mode should use # for filled: {}", r);
        assert!(r.contains('-'), "ASCII mode should use - for empty: {}", r);
    }

    #[test]
    fn test_render_context_bar_medium_no_bar() {
        colored::control::set_override(false);
        let session = SessionInfo {
            used_percentage: Some(42.0),
            ..Default::default()
        };
        let result = render_context_bar(&session, 100, true);
        let r = result.unwrap();
        assert!(r.contains("42%"));
        assert!(
            !r.contains('\u{2588}'),
            "medium mode should not have bar: {}",
            r
        );
    }

    #[test]
    fn test_render_lines_changed() {
        colored::control::set_override(false);
        let session = SessionInfo {
            lines_added: Some(12),
            lines_removed: Some(3),
            ..Default::default()
        };
        let result = render_lines_changed(&session);
        let r = result.unwrap();
        assert!(r.contains("+12"), "should contain +12: {}", r);
        assert!(r.contains("-3"), "should contain -3: {}", r);
    }

    #[test]
    fn test_render_lines_changed_zero() {
        let session = SessionInfo::default();
        assert!(render_lines_changed(&session).is_none());
    }

    #[test]
    fn test_render_model_display() {
        colored::control::set_override(false);
        let session = SessionInfo {
            model_name: Some("Opus 4.6".to_string()),
            ..Default::default()
        };
        let result = render_model(&session);
        assert!(result.unwrap().contains("Opus 4.6"));
    }

    #[test]
    fn test_render_no_loop_state() {
        // State A: no agents, no started_at -> no icon, no "loop" label
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(42.0),
            ..Default::default()
        };
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false, false);
        assert!(
            !line.contains("loop"),
            "State A should not contain 'loop': {}",
            line
        );
        assert!(line.contains("$0.14"), "State A should show cost: {}", line);
        assert!(
            line.contains("42%"),
            "State A should show context %: {}",
            line
        );
    }

    #[test]
    fn test_render_loop_idle_collapsed() {
        // State B: all agents done -> collapsed display
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(42.0),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(0),
            agents: vec![
                AgentState {
                    id: 1,
                    name: "a".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
                AgentState {
                    id: 2,
                    name: "b".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
                AgentState {
                    id: 3,
                    name: "c".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false, false);
        assert!(
            line.contains('3'),
            "State B should show done count: {}",
            line
        );
        assert!(
            line.contains('\u{2713}'),
            "State B should show done symbol: {}",
            line
        );
        assert!(
            !line.contains("loop"),
            "State B should not contain 'loop': {}",
            line
        );
    }

    #[test]
    fn test_render_loop_active_expanded() {
        // State C: running agents -> full dashboard
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(42.0),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - 222,
            ),
            agents: vec![
                AgentState {
                    id: 1,
                    name: "a".into(),
                    status: AgentStatus::Done,
                    updated_at: 0,
                },
                AgentState {
                    id: 2,
                    name: "b".into(),
                    status: AgentStatus::Running,
                    updated_at: 0,
                },
                AgentState {
                    id: 3,
                    name: "c".into(),
                    status: AgentStatus::Queued,
                    updated_at: 0,
                },
            ],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, false, false);
        assert!(
            line.contains("loop"),
            "State C should contain 'loop': {}",
            line
        );
        assert!(line.contains("$0.14"), "State C should show cost: {}", line);
    }

    #[test]
    fn test_used_percentage_direct() {
        let json = r#"{"context_window":{"used_percentage":42.5,"context_window_size":200000}}"#;
        let info = parse_session_info_bytes(json.as_bytes());
        assert!((info.used_percentage.unwrap() - 42.5).abs() < f64::EPSILON);
        assert_eq!(info.context_window, Some(200000));
    }

    #[test]
    fn test_powerline_separator() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            used_percentage: Some(42.0),
            ..Default::default()
        };
        let state = LoopState {
            started_at: Some(0),
            agents: vec![AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            }],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, true, false);
        assert!(
            line.contains('\u{E0B1}'),
            "powerline mode should use chevron: {}",
            line
        );
    }
}
