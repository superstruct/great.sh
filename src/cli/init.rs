use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    /// Template to initialize from
    #[arg(long)]
    pub template: Option<String>,

    /// Overwrite existing configuration
    #[arg(long)]
    pub force: bool,
}

pub fn run(_args: Args) -> Result<()> {
    output::warning("great init: not yet implemented");
    Ok(())
}
