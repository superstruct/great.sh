pub mod apply;
pub mod bootstrap;
pub mod diff;
pub mod doctor;
pub mod init;
pub mod loop_cmd;
pub mod mcp;
pub mod mcp_bridge;
pub mod output;
pub mod status;
pub mod statusline;
pub mod sudo;
pub mod sync;
pub mod template;
pub mod tuning;
pub mod update;
pub mod util;
pub mod vault;

use clap::{Parser, Subcommand};

/// The managed AI dev environment
#[derive(Parser)]
#[command(name = "great", version, about, long_about = None)]
pub struct Cli {
    /// Increase output verbosity
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Disable interactive prompts (for CI/automation)
    #[arg(long, global = true)]
    pub non_interactive: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new great.sh environment
    Init(init::Args),

    /// Apply configuration to the current environment
    Apply(apply::Args),

    /// Show environment status
    Status(status::Args),

    /// Sync configuration with the cloud
    Sync(sync::Args),

    /// Manage encrypted credentials
    Vault(vault::Args),

    /// Manage MCP servers
    Mcp(mcp::Args),

    /// Diagnose environment issues
    Doctor(doctor::Args),

    /// Update great.sh to the latest version
    Update(update::Args),

    /// Show configuration diff
    Diff(diff::Args),

    /// Manage configuration templates
    Template(template::Args),

    /// Install and manage the great.sh Loop agent team
    Loop(loop_cmd::Args),

    /// Render a single statusline for Claude Code (called every 300ms)
    Statusline(statusline::Args),

    /// Start an inbuilt MCP bridge server (stdio JSON-RPC 2.0) — no Node.js required
    #[command(name = "mcp-bridge")]
    McpBridge(mcp_bridge::Args),
}
