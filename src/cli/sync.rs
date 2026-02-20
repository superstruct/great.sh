use anyhow::{Context, Result};
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
    Pull {
        /// Apply the pulled config to great.toml (backs up existing)
        #[arg(long)]
        apply: bool,
    },
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        SyncCommand::Push => run_push(),
        SyncCommand::Pull { apply } => run_pull(apply),
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

    // Save locally (cloud sync is a future feature)
    output::warning("Cloud sync is not yet available. Saving locally.");

    let save_path = sync::save_local(&data)?;
    output::success(&format!("Saved to {}", save_path.display()));

    output::info("When cloud sync is available, this will upload encrypted config to great.sh.");
    Ok(())
}

fn run_pull(apply: bool) -> Result<()> {
    output::header("great sync pull");
    println!();

    // Load from local storage (cloud sync is a future feature)
    output::warning("Cloud sync is not yet available. Loading from local storage.");

    match sync::load_local()? {
        Some(data) => {
            output::info(&format!("Found sync blob: {} bytes", data.len()));

            // Verify the blob is valid TOML before applying
            let content =
                String::from_utf8(data.clone()).context("sync blob is not valid UTF-8")?;

            if let Err(e) = toml::from_str::<crate::config::schema::GreatConfig>(&content) {
                output::error(&format!("Sync blob contains invalid config: {}", e));
                output::info(
                    "The stored config may be corrupted. Run `great sync push` to re-sync.",
                );
                return Ok(());
            }

            if apply {
                // Find or default the config path
                let config_path = config::discover_config()
                    .unwrap_or_else(|_| std::path::PathBuf::from("great.toml"));

                // Backup existing config if it exists
                if config_path.exists() {
                    let backup = config_path.with_extension("toml.bak");
                    std::fs::copy(&config_path, &backup)
                        .context("failed to backup existing great.toml")?;
                    output::info(&format!(
                        "Backed up existing config to {}",
                        backup.display()
                    ));
                }

                // Write the pulled data
                std::fs::write(&config_path, &content).context("failed to write great.toml")?;

                output::success(&format!("Applied sync data to {}", config_path.display()));
                output::info("Run `great apply` to provision the restored environment.");
            } else {
                // Preview mode: show the config content
                output::info("Sync blob content:");
                println!();
                // Show first 40 lines as preview
                for (i, line) in content.lines().enumerate() {
                    if i >= 40 {
                        output::info(&format!(
                            "  ... ({} more lines)",
                            content.lines().count() - 40
                        ));
                        break;
                    }
                    println!("  {}", line);
                }
                println!();
                output::info("Use `great sync pull --apply` to overwrite current config.");
            }
        }
        None => {
            output::warning("No sync data found. Run `great sync push` first.");
        }
    }

    Ok(())
}
