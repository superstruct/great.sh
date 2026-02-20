use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;

use crate::cli::output;
use crate::config;
use crate::platform::command_exists;

/// Arguments for the `great diff` subcommand.
///
/// Compares declared configuration in `great.toml` against the actual system
/// state, showing what tools, MCP servers, and secrets need to be installed,
/// configured, or set.
#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file to diff against
    #[arg(long)]
    pub config: Option<String>,
}

/// Run the `great diff` subcommand.
///
/// Loads the project's `great.toml` (or the file given by `--config`) and
/// compares each declared item against what is actually present on the system.
/// Differences are printed using colored markers:
///
/// - `+` (green) — needs to be added / installed
/// - `~` (yellow) — partially configured, needs attention
pub fn run(args: Args) -> Result<()> {
    // Load config
    let config_path = match &args.config {
        Some(p) => std::path::PathBuf::from(p),
        None => match config::discover_config() {
            Ok(p) => p,
            Err(_) => {
                output::error("No great.toml found. Run `great init` to create one.");
                return Ok(());
            }
        },
    };

    let config_path_str = config_path.to_str().unwrap_or_default();
    let cfg = config::load(Some(config_path_str))?;

    output::header("great diff");
    output::info(&format!(
        "Comparing {} against system state",
        config_path.display()
    ));
    println!();

    let mut has_diff = false;

    // Tools diff
    if let Some(tools) = &cfg.tools {
        let mut tool_diffs = Vec::new();

        // Runtime tools
        for (name, declared_version) in &tools.runtimes {
            // The flattened map includes the "cli" key — skip it since
            // CLI tools are handled separately below.
            if name == "cli" {
                continue;
            }
            let installed = command_exists(name);
            if !installed {
                tool_diffs.push(format!(
                    "  {} {} {}",
                    "+".green(),
                    name.bold(),
                    format!("(need {})", declared_version).dimmed()
                ));
            }
        }

        // CLI tools
        if let Some(cli_tools) = &tools.cli {
            for (name, declared_version) in cli_tools {
                let installed = command_exists(name);
                if !installed {
                    tool_diffs.push(format!(
                        "  {} {} {}",
                        "+".green(),
                        name.bold(),
                        format!("(need {})", declared_version).dimmed()
                    ));
                }
            }
        }

        if !tool_diffs.is_empty() {
            has_diff = true;
            output::header("Tools — need to install:");
            for diff in &tool_diffs {
                println!("{}", diff);
            }
            println!();
        }
    }

    // MCP Servers diff
    if let Some(mcps) = &cfg.mcp {
        let mut mcp_diffs = Vec::new();

        for (name, mcp) in mcps {
            // Check if the command for this MCP server exists
            let cmd_available = command_exists(&mcp.command);
            if !cmd_available {
                mcp_diffs.push(format!(
                    "  {} {} {}",
                    "+".green(),
                    name.bold(),
                    format!("({} — not found)", mcp.command).dimmed()
                ));
            }

            // Check if MCP config exists in project .mcp.json
            let mcp_json_path = std::path::Path::new(".mcp.json");
            if !mcp_json_path.exists() {
                // .mcp.json doesn't exist at all — all declared servers need configuring
                if cmd_available {
                    mcp_diffs.push(format!(
                        "  {} {} {}",
                        "~".yellow(),
                        name.bold(),
                        "(command available, needs .mcp.json config)".dimmed()
                    ));
                }
            }
        }

        if !mcp_diffs.is_empty() {
            has_diff = true;
            output::header("MCP Servers — need configuration:");
            for diff in &mcp_diffs {
                println!("{}", diff);
            }
            println!();
        }
    }

    // Secrets diff
    if let Some(secrets) = &cfg.secrets {
        if let Some(required) = &secrets.required {
            let mut secret_diffs = Vec::new();

            for key in required {
                if std::env::var(key).is_err() {
                    secret_diffs.push(format!(
                        "  {} {} {}",
                        "+".green(),
                        key.bold(),
                        "(not set in environment)".dimmed()
                    ));
                }
            }

            if !secret_diffs.is_empty() {
                has_diff = true;
                output::header("Secrets — need to set:");
                for diff in &secret_diffs {
                    println!("{}", diff);
                }
                println!();
            }
        }
    }

    // Also check for secret references in MCP env
    let secret_refs = cfg.find_secret_refs();
    let mut unresolved_refs = Vec::new();
    for ref_name in &secret_refs {
        if std::env::var(ref_name).is_err() {
            unresolved_refs.push(ref_name.clone());
        }
    }
    if !unresolved_refs.is_empty() {
        has_diff = true;
        output::header("Secret References — unresolved:");
        for name in &unresolved_refs {
            println!(
                "  {} {} {}",
                "+".green(),
                name.bold(),
                "(referenced in MCP env, not set)".dimmed()
            );
        }
        println!();
    }

    if !has_diff {
        output::success("System matches declared configuration. No changes needed.");
    } else {
        output::info("Run `great apply` to reconcile these differences.");
    }

    Ok(())
}
