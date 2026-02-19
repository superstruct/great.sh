use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

pub fn run(_args: Args) -> Result<()> {
    output::warning("great update: not yet implemented");
    Ok(())
}
