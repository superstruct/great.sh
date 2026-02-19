use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: TemplateCommand,
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,

    /// Apply a template
    Apply {
        /// Template name
        name: String,
    },

    /// Update cached templates
    Update,
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        TemplateCommand::List => output::warning("great template list: not yet implemented"),
        TemplateCommand::Apply { .. } => {
            output::warning("great template apply: not yet implemented");
        }
        TemplateCommand::Update => output::warning("great template update: not yet implemented"),
    }
    Ok(())
}
