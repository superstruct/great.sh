use anyhow::Result;
use clap::Args as ClapArgs;

#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}

pub fn run(_args: Args) -> Result<()> {
    println!("great doctor: not yet implemented");
    Ok(())
}
