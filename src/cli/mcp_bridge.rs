use anyhow::{Context, Result};
use clap::Args as ClapArgs;

use crate::config;
use crate::mcp::bridge::backends::discover_backends;
use crate::mcp::bridge::registry::TaskRegistry;
use crate::mcp::bridge::server::start_bridge;
use crate::mcp::bridge::tools::Preset;

/// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) — no Node.js required.
#[derive(ClapArgs)]
pub struct Args {
    /// Tool preset: minimal, agent, research, full.
    /// Controls which tool groups are exposed via tools/list.
    #[arg(long)]
    pub preset: Option<String>,

    /// Comma-separated list of enabled backends: gemini, codex, claude, grok, ollama.
    /// Omit to auto-detect all installed.
    #[arg(long, value_delimiter = ',')]
    pub backends: Option<Vec<String>>,

    /// Per-task timeout in seconds.
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Logging verbosity for stderr: off, error, warn, info, debug.
    #[arg(long)]
    pub log_level: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    // Initialize tracing to stderr (Socrates #6: use try_init to avoid panic)
    let log_level = args.log_level.unwrap_or_else(|| "warn".to_string());
    let filter = match log_level.as_str() {
        "off" => "off",
        "error" => "error",
        "warn" => "warn",
        "info" => "info",
        "debug" => "debug",
        other => {
            eprintln!(
                "warning: unknown log level '{}', defaulting to 'warn'",
                other
            );
            "warn"
        }
    };
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();

    // Load config if available (non-fatal if missing)
    let bridge_config = config::discover_config()
        .ok()
        .and_then(|path| config::load(path.to_str()).ok())
        .and_then(|cfg| cfg.mcp_bridge);

    // Merge config with CLI args (CLI args win via Option — no sentinel values)
    let backend_filter: Vec<String> = args
        .backends
        .or_else(|| bridge_config.as_ref().and_then(|c| c.backends.clone()))
        .unwrap_or_default();

    let default_backend = bridge_config
        .as_ref()
        .and_then(|c| c.default_backend.clone());

    let timeout_secs = args
        .timeout
        .or_else(|| bridge_config.as_ref().and_then(|c| c.timeout_secs))
        .unwrap_or(300);

    let preset_str = args
        .preset
        .or_else(|| bridge_config.as_ref().and_then(|c| c.preset.clone()))
        .unwrap_or_else(|| "agent".to_string());

    let preset = Preset::from_str(&preset_str).context(format!(
        "invalid preset '{}' — use: minimal, agent, research, full",
        preset_str
    ))?;

    // Discover backends
    let backends = discover_backends(&backend_filter);
    if backends.is_empty() {
        anyhow::bail!(
            "no AI CLI backends found on PATH. Install at least one of: gemini, codex, claude, grok, ollama"
        );
    }

    tracing::info!(
        "Discovered backends: {}",
        backends
            .iter()
            .map(|b| b.name)
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Create registry and start the server
    let registry = TaskRegistry::new(timeout_secs);

    // Build and run the tokio runtime (third-site pattern, same as update.rs)
    let rt = tokio::runtime::Runtime::new().context("failed to create tokio runtime")?;
    rt.block_on(start_bridge(backends, default_backend, registry, preset))
}
