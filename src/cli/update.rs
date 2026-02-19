use anyhow::Result;
use clap::Args as ClapArgs;

#[derive(ClapArgs)]
pub struct Args {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

pub fn run(_args: Args) -> Result<()> {
    println!("great update: not yet implemented");
    Ok(())
}
