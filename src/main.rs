mod cli;
mod config;
mod mcp;
mod platform;
mod sync;
mod vault;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let non_interactive = cli.non_interactive;

    match cli.command {
        Command::Init(args) => cli::init::run(args),
        Command::Apply(mut args) => {
            args.non_interactive = non_interactive;
            cli::apply::run(args)
        }
        Command::Status(args) => cli::status::run(args),
        Command::Sync(args) => cli::sync::run(args),
        Command::Vault(args) => cli::vault::run(args),
        Command::Mcp(args) => cli::mcp::run(args),
        Command::Doctor(mut args) => {
            args.non_interactive = non_interactive;
            cli::doctor::run(args)
        }
        Command::Update(args) => cli::update::run(args),
        Command::Diff(args) => cli::diff::run(args),
        Command::Template(args) => cli::template::run(args),
        Command::Loop(mut args) => {
            args.non_interactive = non_interactive;
            cli::loop_cmd::run(args)
        }
        Command::Statusline(args) => cli::statusline::run(args),
    }
}
