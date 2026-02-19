use anyhow::Result;
use clap::{Args as ClapArgs, Subcommand};

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Subcommand)]
pub enum McpCommand {
    /// List configured MCP servers
    List,

    /// Add an MCP server
    Add {
        /// Server name
        name: String,
    },

    /// Test MCP server connectivity
    Test {
        /// Server name (tests all if omitted)
        name: Option<String>,
    },
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        McpCommand::List => println!("great mcp list: not yet implemented"),
        McpCommand::Add { .. } => println!("great mcp add: not yet implemented"),
        McpCommand::Test { .. } => println!("great mcp test: not yet implemented"),
    }
    Ok(())
}
