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

    match cli.command {
        Command::Init(args) => cli::init::run(args),
        Command::Apply(args) => cli::apply::run(args),
        Command::Status(args) => cli::status::run(args),
        Command::Sync(args) => cli::sync::run(args),
        Command::Vault(args) => cli::vault::run(args),
        Command::Mcp(args) => cli::mcp::run(args),
        Command::Doctor(args) => cli::doctor::run(args),
        Command::Update(args) => cli::update::run(args),
        Command::Diff(args) => cli::diff::run(args),
        Command::Template(args) => cli::template::run(args),
        Command::Loop(args) => cli::loop_cmd::run(args),
        Command::Statusline(args) => cli::statusline::run(args),
    }
}
