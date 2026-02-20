use std::io::BufRead;

use anyhow::{Context, Result};
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
    /// Verify keychain access is working
    Login,
    /// Show vault provider status
    Unlock,
    /// Set a credential
    Set {
        /// Secret key name
        key: String,
        /// Secret value (prompted if omitted)
        value: Option<String>,
    },
    /// Import credentials from a .env file or provider
    Import {
        /// Path to .env file or provider name (env, keychain)
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

/// Verify that the keychain provider is accessible by writing and reading a test value.
fn run_login() -> Result<()> {
    output::header("Vault login");
    println!();

    let providers = vault::available_providers();
    let writable: Vec<_> = providers
        .iter()
        .filter(|p| p.name() != "env")
        .collect();

    if writable.is_empty() {
        output::error("No writable secret provider available.");
        output::info("Install a keychain tool:");
        output::info("  macOS: built-in (security command)");
        output::info("  Linux: sudo apt install libsecret-tools");
        return Ok(());
    }

    // Test the first writable provider
    let provider = &writable[0];
    output::info(&format!("Testing {} provider...", provider.name()));

    let test_key = "__great_sh_login_test";
    let test_value = "ok";

    match provider.set(test_key, test_value) {
        Ok(()) => {
            // Verify we can read it back
            match provider.get(test_key) {
                Ok(Some(v)) if v == test_value => {
                    output::success(&format!(
                        "Vault access verified via {} — secrets can be stored and retrieved.",
                        provider.name()
                    ));
                }
                Ok(_) => {
                    output::warning("Write succeeded but read returned unexpected value.");
                }
                Err(e) => {
                    output::warning(&format!("Write succeeded but read failed: {}", e));
                }
            }
        }
        Err(e) => {
            output::error(&format!("Failed to write to {}: {}", provider.name(), e));
            output::info("Check that your keychain/keyring is unlocked.");
        }
    }

    // Show all available providers
    println!();
    output::info("Available providers:");
    for p in &providers {
        let status = if p.is_available() { "available" } else { "not available" };
        output::info(&format!("  {} — {}", p.name(), status));
    }

    Ok(())
}

/// Show vault provider status — keychain is always available, no unlock needed.
fn run_unlock() -> Result<()> {
    output::header("Vault status");
    println!();

    let providers = vault::available_providers();
    output::info("Secrets are stored via system keychain (no separate unlock needed).");
    println!();

    for provider in &providers {
        let status = if provider.is_available() {
            "ready"
        } else {
            "not available"
        };
        if provider.is_available() {
            output::success(&format!("  {} — {}", provider.name(), status));
        } else {
            output::warning(&format!("  {} — {}", provider.name(), status));
        }
    }

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

/// Import secrets from a .env file or a named provider.
fn run_import(path: &str) -> Result<()> {
    // Check if path is a provider name that supports listing
    if path == "env" {
        return import_from_env_provider();
    }

    if let Some(provider) = vault::get_provider(path) {
        output::header(&format!("Importing from {}", provider.name()));
        if !provider.is_available() {
            output::error(&format!(
                "{} is not available on this system.",
                provider.name()
            ));
            return Ok(());
        }
        // 1Password and Bitwarden require interactive auth for listing.
        // Suggest using .env file export instead.
        output::warning(&format!(
            "{} does not support non-interactive listing.",
            provider.name()
        ));
        output::info("Export secrets to a .env file first, then import from the file:");
        output::info("  great vault import path/to/secrets.env");
        return Ok(());
    }

    // Otherwise treat as a .env file path
    import_from_dotenv(path)
}

/// Import secrets from the environment — find likely API keys and store them.
fn import_from_env_provider() -> Result<()> {
    output::header("Importing from environment");
    println!();

    let env_provider = vault::get_provider("env").expect("env provider always exists");
    let keys = env_provider.list(None)?;

    if keys.is_empty() {
        output::warning("No secret-like environment variables found.");
        output::info("Looking for variables containing KEY, TOKEN, SECRET, or PASSWORD.");
        return Ok(());
    }

    output::info(&format!("Found {} secret-like variables:", keys.len()));

    let providers = vault::available_providers();
    let target = providers
        .iter()
        .find(|p| p.name() != "env" && p.is_available());

    let target = match target {
        Some(t) => t,
        None => {
            for key in &keys {
                output::info(&format!("  {}", key));
            }
            output::warning("No writable provider available to import into.");
            return Ok(());
        }
    };

    let mut imported = 0;
    for key in &keys {
        if let Ok(Some(value)) = env_provider.get(key) {
            match target.set(key, &value) {
                Ok(()) => {
                    output::success(&format!("  {} — imported to {}", key, target.name()));
                    imported += 1;
                }
                Err(_) => {
                    output::error(&format!("  {} — failed to import", key));
                }
            }
        }
    }

    println!();
    output::info(&format!(
        "Imported {} of {} secrets to {}.",
        imported,
        keys.len(),
        target.name()
    ));

    Ok(())
}

/// Parse a .env file and store each key-value pair via the vault.
fn import_from_dotenv(path: &str) -> Result<()> {
    output::header(&format!("Importing from {}", path));
    println!();

    let file = std::fs::File::open(path)
        .context(format!("failed to open {}", path))?;
    let reader = std::io::BufReader::new(file);

    let providers = vault::available_providers();
    let target = providers
        .iter()
        .find(|p| p.name() != "env" && p.is_available());

    let target = match target {
        Some(t) => t,
        None => {
            output::error("No writable provider available to import into.");
            output::info("Install a keychain tool first, or use environment variables directly.");
            return Ok(());
        }
    };

    let mut imported = 0;
    let mut skipped = 0;

    for line in reader.lines() {
        let line = line.context("failed to read line")?;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip optional 'export ' prefix
        let trimmed = trimmed.strip_prefix("export ").unwrap_or(trimmed);

        // Split on first '='
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Strip quotes from value
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(value);

            if key.is_empty() || value.is_empty() {
                skipped += 1;
                continue;
            }

            match target.set(key, value) {
                Ok(()) => {
                    output::success(&format!("  {} — imported", key));
                    imported += 1;
                }
                Err(e) => {
                    output::error(&format!("  {} — failed: {}", key, e));
                    skipped += 1;
                }
            }
        } else {
            skipped += 1;
        }
    }

    println!();
    output::info(&format!(
        "Imported {} secrets to {} ({} skipped).",
        imported,
        target.name(),
        skipped
    ));

    Ok(())
}
