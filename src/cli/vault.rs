use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: VaultCommand,
}

#[derive(Subcommand)]
pub enum VaultCommand {
    /// Authenticate with the vault service
    Login,

    /// Unlock the local vault
    Unlock,

    /// Set a credential
    Set {
        /// Credential key
        key: String,
        /// Credential value
        value: String,
    },

    /// Import credentials from a file
    Import {
        /// Path to credentials file
        path: String,
    },
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        VaultCommand::Login => output::warning("great vault login: not yet implemented"),
        VaultCommand::Unlock => output::warning("great vault unlock: not yet implemented"),
        VaultCommand::Set { .. } => output::warning("great vault set: not yet implemented"),
        VaultCommand::Import { .. } => {
            output::warning("great vault import: not yet implemented");
        }
    }
    Ok(())
}
