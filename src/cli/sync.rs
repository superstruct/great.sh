use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

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
        SyncCommand::Push => println!("great sync push: not yet implemented"),
        SyncCommand::Pull => println!("great sync pull: not yet implemented"),
    }
    Ok(())
}
