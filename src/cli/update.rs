use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;

/// Version of this binary, set at compile time from Cargo.toml.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ClapArgs)]
pub struct Args {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

/// Check for or perform a self-update of the `great` CLI.
pub fn run(args: Args) -> Result<()> {
    output::header("great update");
    println!();
    output::info(&format!("Current version: {}", CURRENT_VERSION));

    if args.check {
        output::info("Checking for updates...");
        // In production, this would check GitHub releases API
        output::warning("Update checking is not yet available.");
        output::info("Check https://github.com/great-sh/great/releases for the latest version.");
        return Ok(());
    }

    output::warning("Self-update is not yet available.");
    output::info("To update, re-run the install script:");
    output::info("  curl -sSL https://great.sh/install.sh | bash");
    println!();
    output::info("Or use your package manager:");
    output::info("  brew upgrade great  (macOS)");
    output::info("  cargo install great-sh  (via Cargo)");

    Ok(())
}
