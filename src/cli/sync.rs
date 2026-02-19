use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: SyncCommand,
}

#[derive(Subcommand)]
pub enum SyncCommand {
    /// Push local configuration to the cloud
    Push,

    /// Pull configuration from the cloud
    Pull,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        SyncCommand::Push => output::warning("great sync push: not yet implemented"),
        SyncCommand::Pull => output::warning("great sync pull: not yet implemented"),
    }
    Ok(())
}
