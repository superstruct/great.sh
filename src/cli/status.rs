use anyhow::Result;
use clap::Args as ClapArgs;

use crate::platform;

#[derive(ClapArgs)]
pub struct Args {
    /// Show detailed status information
    #[arg(long, short)]
    pub verbose: bool,

    /// Output status as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let platform = platform::detect_platform();

    if args.json {
        println!(r#"{{"platform": "{}", "status": "not yet implemented"}}"#, platform);
    } else {
        println!("great status: not yet implemented");
        if args.verbose {
            println!("  platform: {}", platform);
        }
    }
    Ok(())
}
