use std::fmt::Write as FmtWrite;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Session metadata piped by Claude Code on each statusline tick.
/// All fields are optional -- Claude Code may omit any of them, and
/// the entire blob may be absent or empty.
#[derive(Debug, Deserialize, Default)]
pub struct SessionInfo {
    #[allow(dead_code)] // Deserialized for forward-compatibility; not rendered.
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub context_tokens: Option<u64>,
    pub context_window: Option<u64>,
    #[allow(dead_code)] // Deserialized for forward-compatibility; not rendered.
    pub workspace: Option<String>,
    // session_id and transcript_path removed -- sensitive, unused by rendering.
    // serde_json silently drops unknown fields, so no deserialization breakage.
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
///
/// Note: `segments` and `agent_names` fields from the original spec draft
/// were removed because the rendering logic does not currently consume them.
/// They may be added in a future iteration when custom segment ordering and
/// agent label overrides are implemented.
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

    // 4. Read agent state
    let (state, had_parse_error) = read_state(&config.state_file, config.session_timeout_secs);

    // 5. Resolve terminal width
    let width = resolve_width(args.width);
    let use_unicode = !args.no_unicode;

    // 6. Render
    let line = render(&session, &state, &config, width, use_unicode, had_parse_error);

    // 7. Print exactly one line to stdout
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
/// Reads at most 64KB from stdin to avoid blocking on large inputs.
fn parse_stdin() -> SessionInfo {
    let mut buf = Vec::with_capacity(65536);
    let _ = std::io::stdin().lock().take(65536).read_to_end(&mut buf);

    if buf.is_empty() {
        return SessionInfo::default();
    }

    serde_json::from_slice(&buf).unwrap_or_default()
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
/// Appends a reset sequence if truncation happened mid-escape.
fn truncate_to_width(s: &str, max_visible: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut visible = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            out.push(c);
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
            out.push(c);
        } else {
            if visible >= max_visible {
                break;
            }
            out.push(c);
            visible += 1;
        }
    }
    out
}

/// Format a token count as a human-readable string (e.g. 45230 -> "45K").
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
// Segment renderers
// ---------------------------------------------------------------------------

/// Render cost segment (e.g. "$0.14"). Two decimal places for consistency.
fn render_cost(session: &SessionInfo) -> Option<String> {
    session.cost_usd.map(|c| format!("${:.2}", c))
}

/// Render context window segment with color threshold (e.g. "45K/200K").
fn render_context(session: &SessionInfo, _use_unicode: bool) -> Option<String> {
    let tokens = session.context_tokens?;
    let window = session.context_window?;

    // Avoid division by zero
    if window == 0 {
        return None;
    }

    let ratio = tokens as f64 / window as f64;
    let text = format!("{}/{}", format_tokens(tokens), format_tokens(window));

    let colored_text = if ratio >= 0.8 {
        text.bright_red().to_string()
    } else if ratio >= 0.5 {
        text.yellow().to_string()
    } else {
        text.green().to_string()
    };

    Some(colored_text)
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
        parts.push(
            format!("{}{}", counts.done, done_sym)
                .green()
                .to_string(),
        );
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
            AgentStatus::Running => "\u{25CF}",  // filled circle
            AgentStatus::Done => "\u{2713}",     // checkmark
            AgentStatus::Queued => "\u{25CC}",   // dotted circle
            AgentStatus::Error => "\u{2717}",    // ballot X
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

/// Render the agent indicators segment (wide mode: "1\u{25CF} 2\u{25CF} 3\u{25CC} 4\u{2717}").
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

/// Render the agent indicators segment (medium mode: "\u{25CF}\u{25CF}\u{25CC}\u{2717}\u{25CF}\u{25CF}\u{25CB}\u{25CF}").
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
// Main render function
// ---------------------------------------------------------------------------

/// Render a compact cost+context string for medium mode (e.g. "$0.14 45K/200K").
/// Space-separated, no separator bars. Returns None if both are absent.
fn render_compact_cost_context(session: &SessionInfo, use_unicode: bool) -> Option<String> {
    let cost = render_cost(session);
    let ctx = render_context(session, use_unicode);

    match (cost, ctx) {
        (Some(c), Some(x)) => Some(format!("{} {}", c, x)),
        (Some(c), None) => Some(c),
        (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

/// Main render function. Returns the final single-line string.
/// The output is truncated to `width` visible columns to prevent line wrapping.
fn render(
    session: &SessionInfo,
    state: &LoopState,
    _config: &StatuslineConfig,
    width: u16,
    use_unicode: bool,
    had_parse_error: bool,
) -> String {
    let mut out = String::with_capacity(256);
    let w = width as usize;

    let icon = if use_unicode {
        "\u{26A1}".bright_yellow().to_string()
    } else {
        ">".bright_yellow().to_string()
    };

    let sep = if use_unicode {
        format!(" {} ", "\u{2502}".dimmed())
    } else {
        format!(" {} ", "|".dimmed())
    };

    // Width mode selection
    if width > 120 {
        // Wide mode
        let _ = write!(out, "{} {}", icon, "loop".bold());

        if had_parse_error {
            let _ = write!(out, "{}{}", sep, "ERR:state".bright_red());
        } else if !state.agents.is_empty() {
            // F3: Try wide agent indicators first; if they would overflow,
            // fall back to medium (compact) indicators.
            let wide_agents = render_agents_wide(&state.agents, use_unicode);
            let summary = render_summary(&state.agents, use_unicode);

            // Estimate: prefix "X loop" ~ 7 + sep(3) + agents + sep(3) + summary
            let overhead = 7 + 3 + 3 + visible_len(&summary);
            let agents_budget = w.saturating_sub(overhead + 20); // reserve ~20 for cost/ctx/elapsed

            if visible_len(&wide_agents) <= agents_budget {
                let _ = write!(out, "{}{}", sep, wide_agents);
            } else {
                // Fall back to medium-mode compact indicators
                let _ = write!(out, "{}{}", sep, render_agents_medium(&state.agents, use_unicode));
            }
            let _ = write!(out, "{}{}", sep, summary);
        } else {
            let _ = write!(out, "{}{}", sep, "idle".dimmed());
        }

        if !had_parse_error {
            if let Some(cost) = render_cost(session) {
                let _ = write!(out, "{}{}", sep, cost);
            }
            if let Some(ctx) = render_context(session, use_unicode) {
                let _ = write!(out, "{}{}", sep, ctx);
            }
            if let Some(elapsed) = render_elapsed(state) {
                let _ = write!(out, "{}{}", sep, elapsed);
            }
        }
    } else if width >= 80 {
        // Medium mode
        let _ = write!(out, "{} {}", icon, "loop".bold());

        if had_parse_error {
            let _ = write!(out, "{}{}", sep, "ERR:state".bright_red());
        } else {
            if !state.agents.is_empty() {
                let _ = write!(out, "{}{}", sep, render_agents_medium(&state.agents, use_unicode));
                let _ = write!(out, "{}{}", sep, render_summary(&state.agents, use_unicode));
            } else {
                let _ = write!(out, "{}{}", sep, "idle".dimmed());
            }

            // F1: Add compact cost+context in medium mode
            if let Some(cc) = render_compact_cost_context(session, use_unicode) {
                let _ = write!(out, "{}{}", sep, cc);
            }

            if let Some(elapsed) = render_elapsed(state) {
                let _ = write!(out, "{}{}", sep, elapsed);
            }
        }
    } else {
        // Narrow mode
        if had_parse_error {
            let _ = write!(out, "{} {}", icon, "ERR:state".bright_red());
        } else if !state.agents.is_empty() {
            let summary = render_summary(&state.agents, use_unicode);
            let _ = write!(out, "{} {}", icon, summary);
            if let Some(elapsed) = render_elapsed(state) {
                let _ = write!(out, "{}{}", sep, elapsed);
            }
        } else {
            let _ = write!(out, "{} {}", icon, "idle".dimmed());
            if let Some(elapsed) = render_elapsed(state) {
                let _ = write!(out, "{}{}", sep, elapsed);
            }
        }
    }

    // F3: Final overflow guard -- truncate to terminal width
    if w > 0 && visible_len(&out) > w {
        out = truncate_to_width(&out, w);
    }

    out
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
        colored::control::set_override(false);
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Error, updated_at: 0 },
            AgentState { id: 4, name: "d".into(), status: AgentStatus::Queued, updated_at: 0 },
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
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Queued, updated_at: 0 },
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
            AgentState { id: 1, name: "nightingale".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "lovelace".into(), status: AgentStatus::Running, updated_at: 0 },
        ];
        let result = render_agents_wide(&agents, true);
        assert!(result.contains('1'));
        assert!(result.contains('2'));
    }

    #[test]
    fn test_render_agents_medium_ascii() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Done, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Error, updated_at: 0 },
        ];
        let result = render_agents_medium(&agents, false);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_wide_mode() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            context_tokens: Some(45000),
            context_window: Some(200000),
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
            agents: vec![AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            }],
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
        assert!(
            line.chars().all(|c| c.is_ascii()),
            "all characters must be ASCII: {}",
            line
        );
    }

    // --- Timeout ---

    #[test]
    fn test_apply_timeout_demotes_stale_agents() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Running, updated_at: now - 60 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Running, updated_at: now - 5 },
            AgentState { id: 3, name: "c".into(), status: AgentStatus::Done, updated_at: now - 60 },
        ];
        apply_timeout(&mut agents, 30);
        assert_eq!(agents[0].status, AgentStatus::Idle);
        assert_eq!(agents[1].status, AgentStatus::Running);
        assert_eq!(agents[2].status, AgentStatus::Done);
    }

    // --- Context color threshold ---

    #[test]
    fn test_context_below_50_percent() {
        colored::control::set_override(false);
        let session = SessionInfo {
            context_tokens: Some(40000),
            context_window: Some(200000),
            ..Default::default()
        };
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
        let result = render_context(&session, true);
        assert!(result.is_none());
    }

    // --- Width resolution ---

    #[test]
    fn test_resolve_width_from_args() {
        assert_eq!(resolve_width(Some(120)), 120);
    }

    // Note: resolve_width fallback test omitted because mutating env vars
    // (env::remove_var) is unsound in multi-threaded test execution.
    // The fallback logic is trivially correct and tested via integration tests.

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
        // Should contain agent 30 but not agent 31
        assert!(result.contains("30"));
        assert!(!result.contains("31"));
        // Should contain ellipsis
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
        // Should contain ellipsis character
        assert!(result.contains('\u{2026}'));
    }

    // --- Path traversal security ---

    #[test]
    fn test_read_state_rejects_path_traversal() {
        // A path with ".." should NOT be read; it falls back to the default
        // state file path. The key assertion: it must not read /etc/passwd.
        let (state, err) = read_state("/tmp/../etc/passwd", 30);
        // /etc/passwd is not valid JSON, so if it were read we would get
        // had_parse_error = true. The path traversal guard should prevent
        // reading it entirely by substituting the default path.
        // Since the default path may or may not exist, we only assert
        // the traversal path was NOT read (i.e. no parse error from passwd).
        let (_default_state, _) = read_state("/tmp/great-loop/state.json", 30);
        // If the traversal were allowed, /etc/passwd would cause a parse error.
        // The fallback default path either doesn't exist (err=false) or has valid data.
        // Either way, err should match what the default path produces.
        let (_, default_err) = read_state("/tmp/great-loop/state.json", 30);
        assert_eq!(
            err, default_err,
            "path traversal should fall back to default path behavior"
        );
        // Verify agents match default path behavior too
        assert_eq!(state.agents.len(), _default_state.agents.len());
    }

    // --- Idle rendering ---

    #[test]
    fn test_render_idle_when_no_agents() {
        colored::control::set_override(false);
        let session = SessionInfo::default();
        let state = LoopState::default();
        let config = StatuslineConfig::default();

        let wide = render(&session, &state, &config, 150, true, false);
        assert!(wide.contains("idle"));

        let medium = render(&session, &state, &config, 100, true, false);
        assert!(medium.contains("idle"));

        let narrow = render(&session, &state, &config, 60, true, false);
        assert!(narrow.contains("idle"));
    }

    // --- F1: Medium mode includes cost+context ---

    #[test]
    fn test_render_medium_mode_includes_cost_and_context() {
        colored::control::set_override(false);
        let session = SessionInfo {
            cost_usd: Some(0.14),
            context_tokens: Some(45000),
            context_window: Some(200000),
            ..Default::default()
        };
        let state = LoopState {
            agents: vec![AgentState {
                id: 1,
                name: "a".into(),
                status: AgentStatus::Done,
                updated_at: 0,
            }],
            ..Default::default()
        };
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 100, true, false);
        assert!(line.contains("$0.14"), "medium mode must show cost: {}", line);
        assert!(line.contains("45K/200K"), "medium mode must show context: {}", line);
    }

    // --- F2: Summary uses per-agent vocabulary ---

    #[test]
    fn test_render_summary_splits_running_and_queued() {
        colored::control::set_override(false);
        let agents = vec![
            AgentState { id: 1, name: "a".into(), status: AgentStatus::Running, updated_at: 0 },
            AgentState { id: 2, name: "b".into(), status: AgentStatus::Queued, updated_at: 0 },
        ];
        let summary = render_summary(&agents, false);
        // ASCII: running=*, queued=. -- must be separate
        assert!(summary.contains('*'), "should show running indicator: {}", summary);
        assert!(summary.contains('.'), "should show queued indicator: {}", summary);
    }

    // --- F3: Overflow guard ---

    #[test]
    fn test_visible_len_strips_ansi() {
        assert_eq!(visible_len("hello"), 5);
        assert_eq!(visible_len("\x1b[31mred\x1b[0m"), 3);
        assert_eq!(visible_len(""), 0);
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("hello world", 5), "hello");
        // ANSI codes should be preserved, visible chars truncated
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
            context_tokens: Some(45000),
            context_window: Some(200000),
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
        // Width 130 is wide mode but tight with 25 agents
        let line = render(&session, &state, &config, 130, false, false);
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
            context_tokens: Some(45000),
            context_window: Some(200000),
            ..Default::default()
        };
        let state = LoopState::default();
        let config = StatuslineConfig::default();
        let line = render(&session, &state, &config, 150, true, true);
        assert!(line.contains("ERR:state"));
        assert!(!line.contains("$0.14"), "cost must not leak after ERR:state: {}", line);
        assert!(!line.contains("45K"), "context must not leak after ERR:state: {}", line);
    }
}
