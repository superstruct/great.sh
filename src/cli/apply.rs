use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,
}

pub fn run(_args: Args) -> Result<()> {
    output::warning("great apply: not yet implemented");
    Ok(())
}
