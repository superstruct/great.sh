pub mod apply;
pub mod diff;
pub mod doctor;
pub mod init;
pub mod mcp;
pub mod status;
pub mod sync;
pub mod template;
pub mod update;
pub mod vault;

use clap::{Parser, Subcommand};

/// The managed AI dev environment
#[derive(Parser)]
#[command(name = "great", version, about, long_about = None)]
pub struct Cli {
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
}
