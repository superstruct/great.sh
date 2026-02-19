use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;
use crate::vault;

/// Arguments for the `great vault` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: VaultCommand,
}

/// Available vault subcommands.
#[derive(Subcommand)]
pub enum VaultCommand {
    /// Authenticate with the great.sh cloud vault
    Login,
    /// Unlock the local encrypted vault
    Unlock,
    /// Set a credential
    Set {
        /// Secret key name
        key: String,
        /// Secret value (prompted if omitted)
        value: Option<String>,
    },
    /// Import credentials from a file or provider
    Import {
        /// Path to import file or provider name
        path: String,
    },
}

/// Run the vault subcommand specified in `args`.
pub fn run(args: Args) -> Result<()> {
    match args.command {
        VaultCommand::Login => run_login(),
        VaultCommand::Unlock => run_unlock(),
        VaultCommand::Set { key, value } => run_set(&key, value.as_deref()),
        VaultCommand::Import { path } => run_import(&path),
    }
}

fn run_login() -> Result<()> {
    output::warning("Cloud vault login is not yet available.");
    output::info("For now, use `great vault set <KEY>` to store secrets locally.");
    Ok(())
}

fn run_unlock() -> Result<()> {
    output::warning("Local encrypted vault is not yet available.");
    output::info("For now, secrets are stored via system keychain or environment variables.");
    Ok(())
}

fn run_set(key: &str, value: Option<&str>) -> Result<()> {
    output::header(&format!("Setting secret: {}", key));

    let secret_value = match value {
        Some(v) => v.to_string(),
        None => {
            // Read from stdin (for piped input)
            output::info("Enter secret value (or pipe it in):");
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if secret_value.is_empty() {
        output::error("Secret value cannot be empty.");
        return Ok(());
    }

    // Try providers in order of preference
    let providers = vault::available_providers();

    for provider in &providers {
        // Skip env provider for set operations
        if provider.name() == "env" {
            continue;
        }

        match provider.set(key, &secret_value) {
            Ok(()) => {
                output::success(&format!("Secret '{}' stored via {}", key, provider.name()));
                return Ok(());
            }
            Err(_) => continue, // Try next provider
        }
    }

    // If no provider could store it, suggest alternatives
    output::error("Could not store secret -- no writable provider available.");
    output::info(&format!(
        "Set it as an environment variable instead: export {}=<value>",
        key
    ));
    Ok(())
}

fn run_import(path: &str) -> Result<()> {
    // Check if path is a provider name
    if let Some(provider) = vault::get_provider(path) {
        output::header(&format!("Importing from {}", provider.name()));
        if !provider.is_available() {
            output::error(&format!(
                "{} is not available on this system",
                provider.name()
            ));
            return Ok(());
        }

        match provider.list(None) {
            Ok(keys) => {
                if keys.is_empty() {
                    output::warning("No secrets found to import.");
                } else {
                    output::info(&format!("Found {} secrets:", keys.len()));
                    for key in &keys {
                        output::info(&format!("  {}", key));
                    }
                    output::info("Import functionality is not yet fully implemented.");
                }
            }
            Err(e) => {
                output::error(&format!("Failed to list secrets: {}", e));
            }
        }
        return Ok(());
    }

    // Otherwise treat as a file path
    output::warning(&format!(
        "File import from '{}' is not yet implemented.",
        path
    ));
    output::info("Supported import sources: env, 1password, bitwarden, keychain");
    Ok(())
}
