use anyhow::Result;
use clap::Args as ClapArgs;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file to diff against
    #[arg(long)]
    pub config: Option<String>,
}

pub fn run(_args: Args) -> Result<()> {
    println!("great diff: not yet implemented");
    Ok(())
}
