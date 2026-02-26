use std::collections::BTreeSet;

use anyhow::Result;
use clap::Args as ClapArgs;
use colored::Colorize;

use crate::cli::output;
use crate::cli::util;
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
/// - `-` (red) — blocked, requires manual resolution (e.g., missing secret)
pub fn run(args: Args) -> Result<()> {
    // Load config
    let config_path = match &args.config {
        Some(p) => std::path::PathBuf::from(p),
        None => match config::discover_config() {
            Ok(p) => p,
            Err(_) => {
                output::error("No great.toml found. Run `great init` to create one.");
                std::process::exit(1);
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
    let mut install_count: usize = 0;
    let mut configure_count: usize = 0;

    // Tools diff
    if let Some(tools) = &cfg.tools {
        let mut tool_diffs = Vec::new();

        // Runtime tools
        for (name, declared_version) in &tools.runtimes {
            if name == "cli" {
                continue;
            }
            let installed = command_exists(name);
            if !installed {
                install_count += 1;
                tool_diffs.push(format!(
                    "  {} {} {}",
                    "+".green(),
                    name.bold(),
                    format!("(need {})", declared_version).dimmed()
                ));
            } else if declared_version != "latest" && declared_version != "stable" {
                if let Some(actual) = util::get_command_version(name) {
                    if !actual.contains(declared_version.as_str()) {
                        configure_count += 1;
                        tool_diffs.push(format!(
                            "  {} {} {}",
                            "~".yellow(),
                            name.bold(),
                            format!("(want {}, have {})", declared_version, actual).dimmed()
                        ));
                    }
                }
            }
        }

        // CLI tools
        if let Some(cli_tools) = &tools.cli {
            for (name, declared_version) in cli_tools {
                let installed = command_exists(name);
                if !installed {
                    install_count += 1;
                    tool_diffs.push(format!(
                        "  {} {} {}",
                        "+".green(),
                        name.bold(),
                        format!("(need {})", declared_version).dimmed()
                    ));
                } else if declared_version != "latest" && declared_version != "stable" {
                    if let Some(actual) = util::get_command_version(name) {
                        if !actual.contains(declared_version.as_str()) {
                            configure_count += 1;
                            tool_diffs.push(format!(
                                "  {} {} {}",
                                "~".yellow(),
                                name.bold(),
                                format!("(want {}, have {})", declared_version, actual).dimmed()
                            ));
                        }
                    }
                }
            }
        }

        if !tool_diffs.is_empty() {
            has_diff = true;
            output::header("Tools");
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
            // Skip disabled servers
            if mcp.enabled == Some(false) {
                continue;
            }

            // Check if the command for this MCP server exists
            let cmd_available = command_exists(&mcp.command);
            if !cmd_available {
                install_count += 1;
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
                    configure_count += 1;
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
            output::header("MCP Servers");
            for diff in &mcp_diffs {
                println!("{}", diff);
            }
            println!();
        }
    }

    // Secrets diff (unified with deduplication)
    let mut all_missing_secrets: BTreeSet<String> = BTreeSet::new();
    let mut secret_diffs: Vec<String> = Vec::new();

    // Phase 1: secrets.required
    if let Some(secrets) = &cfg.secrets {
        if let Some(required) = &secrets.required {
            for key in required {
                if std::env::var(key).is_err() {
                    all_missing_secrets.insert(key.clone());
                    secret_diffs.push(format!(
                        "  {} {} {}",
                        "-".red(),
                        key.bold(),
                        "(not set in environment)".dimmed()
                    ));
                }
            }
        }
    }

    // Phase 2: find_secret_refs (MCP env + agent api_key)
    let secret_refs = cfg.find_secret_refs();
    for ref_name in &secret_refs {
        if std::env::var(ref_name).is_err() && !all_missing_secrets.contains(ref_name) {
            all_missing_secrets.insert(ref_name.clone());
            secret_diffs.push(format!(
                "  {} {} {}",
                "-".red(),
                ref_name.bold(),
                "(referenced in config, not set)".dimmed()
            ));
        }
    }

    let secrets_count = all_missing_secrets.len();

    if !secret_diffs.is_empty() {
        has_diff = true;
        output::header("Secrets");
        for diff in &secret_diffs {
            println!("{}", diff);
        }
        println!();
    }

    if !has_diff {
        output::success("Environment matches configuration — nothing to do.");
    } else {
        let mut parts = Vec::new();
        if install_count > 0 {
            parts.push(format!("{} to install", install_count));
        }
        if configure_count > 0 {
            parts.push(format!("{} to configure", configure_count));
        }
        if secrets_count > 0 {
            parts.push(format!("{} secrets to resolve", secrets_count));
        }
        let summary = parts.join(", ");
        output::info(&format!("{} — run `great apply` to reconcile.", summary));
    }

    Ok(())
}
