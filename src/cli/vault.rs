use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

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
        VaultCommand::Login => println!("great vault login: not yet implemented"),
        VaultCommand::Unlock => println!("great vault unlock: not yet implemented"),
        VaultCommand::Set { .. } => println!("great vault set: not yet implemented"),
        VaultCommand::Import { .. } => println!("great vault import: not yet implemented"),
    }
    Ok(())
}
