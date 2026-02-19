use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;
use crate::config;
use crate::sync;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: SyncCommand,
}

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Push local configuration to sync storage
    Push,
    /// Pull configuration from sync storage
    Pull,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        SyncCommand::Push => run_push(),
        SyncCommand::Pull => run_pull(),
    }
}

fn run_push() -> Result<()> {
    output::header("great sync push");
    println!();

    // Find and read current config
    let config_path = match config::discover_config() {
        Ok(p) => p,
        Err(_) => {
            output::error("No great.toml found. Nothing to sync.");
            return Ok(());
        }
    };

    output::info(&format!("Config: {}", config_path.display()));

    // Export config
    let data = sync::export_config(&config_path)?;
    output::info(&format!("Config size: {} bytes", data.len()));

    // Note: In production, this would encrypt the data with AES-256-GCM
    // and upload to the great.sh cloud. For now, save locally.
    output::warning("Cloud sync is not yet available. Saving locally.");

    let save_path = sync::save_local(&data)?;
    output::success(&format!("Saved to {}", save_path.display()));

    output::info("When cloud sync is available, this will upload encrypted config to great.sh.");
    Ok(())
}

fn run_pull() -> Result<()> {
    output::header("great sync pull");
    println!();

    // Note: In production, this would download from great.sh cloud
    output::warning("Cloud sync is not yet available. Loading from local storage.");

    match sync::load_local()? {
        Some(data) => {
            output::info(&format!("Found sync blob: {} bytes", data.len()));

            // For now, just show what we'd do
            output::info("Would restore great.toml from sync blob.");
            output::info("Use `great sync pull --apply` to overwrite current config (not yet implemented).");
        }
        None => {
            output::warning("No sync data found. Run `great sync push` first.");
        }
    }

    Ok(())
}
