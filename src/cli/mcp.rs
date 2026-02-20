use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;
use crate::config;
use crate::mcp::{self, McpJsonConfig};
use crate::platform::command_exists;

/// Arguments for the `great mcp` command group.
#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: McpCommand,
}

/// Subcommands for MCP server management.
#[derive(Subcommand)]
pub enum McpCommand {
    /// List all configured MCP servers
    List,
    /// Add an MCP server to configuration
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

/// Dispatch the `great mcp <subcommand>` invocation.
pub fn run(args: Args) -> Result<()> {
    match args.command {
        McpCommand::List => run_list(),
        McpCommand::Add { name } => run_add(&name),
        McpCommand::Test { name } => run_test(name.as_deref()),
    }
}

/// List all MCP servers declared in `great.toml` and/or present in `.mcp.json`.
fn run_list() -> Result<()> {
    output::header("MCP Servers");
    println!();

    // Load from great.toml
    let declared = match config::discover_config() {
        Ok(path) => match config::load(Some(path.to_str().unwrap_or_default())) {
            Ok(cfg) => cfg.mcp.unwrap_or_default(),
            Err(_) => std::collections::HashMap::new(),
        },
        Err(_) => std::collections::HashMap::new(),
    };

    // Load from .mcp.json
    let mcp_path = mcp::project_mcp_path();
    let mcp_json = McpJsonConfig::load(&mcp_path).unwrap_or_default();

    if declared.is_empty() && mcp_json.mcp_servers.is_empty() {
        output::warning("No MCP servers configured.");
        output::info("Add one with: great mcp add <name>");
        return Ok(());
    }

    // Show declared servers from great.toml
    if !declared.is_empty() {
        output::info("From great.toml:");
        for (name, mcp_cfg) in &declared {
            let cmd_available = command_exists(&mcp_cfg.command);
            let in_mcp_json = mcp_json.has_server(name);

            let status = match (cmd_available, in_mcp_json) {
                (true, true) => "ready",
                (true, false) => "command found, not in .mcp.json",
                (false, true) => "in .mcp.json, command missing",
                (false, false) => "not configured",
            };

            if cmd_available && in_mcp_json {
                output::success(&format!("  {} ({}) — {}", name, mcp_cfg.command, status));
            } else if cmd_available {
                output::warning(&format!("  {} ({}) — {}", name, mcp_cfg.command, status));
            } else {
                output::error(&format!("  {} ({}) — {}", name, mcp_cfg.command, status));
            }
        }
    }

    // Show any servers in .mcp.json not in great.toml
    let extra: Vec<_> = mcp_json
        .mcp_servers
        .keys()
        .filter(|k| !declared.contains_key(*k))
        .collect();

    if !extra.is_empty() {
        println!();
        output::info("In .mcp.json only (not in great.toml):");
        for name in extra {
            let entry = &mcp_json.mcp_servers[name];
            output::info(&format!("  {} ({})", name, entry.command));
        }
    }

    Ok(())
}

/// Add an MCP server entry to `great.toml` using format-preserving editing.
fn run_add(name: &str) -> Result<()> {
    output::header(&format!("Adding MCP server: {}", name));

    // Check if great.toml exists
    let config_path = match config::discover_config() {
        Ok(p) => p,
        Err(_) => {
            output::error("No great.toml found. Run `great init` first.");
            return Ok(());
        }
    };

    // Read current config to check for duplicates
    let cfg = config::load(Some(config_path.to_str().unwrap_or_default()))?;
    if let Some(mcps) = &cfg.mcp {
        if mcps.contains_key(name) {
            output::warning(&format!("MCP server '{}' already in great.toml", name));
            return Ok(());
        }
    }

    // Use toml_edit for format-preserving modification
    let content = std::fs::read_to_string(&config_path).context("failed to read great.toml")?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .context("failed to parse great.toml for editing")?;

    // Ensure [mcp] table exists
    if doc.get("mcp").is_none() {
        doc["mcp"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // Build the server entry as an inline table
    let mut server_table = toml_edit::Table::new();
    server_table.insert("command", toml_edit::value("npx"));
    let mut args_array = toml_edit::Array::new();
    args_array.push("-y");
    args_array.push(format!("@modelcontextprotocol/server-{}", name));
    server_table.insert("args", toml_edit::value(args_array));

    // Insert into the mcp table
    if let Some(mcp_item) = doc.get_mut("mcp") {
        if let Some(mcp_table) = mcp_item.as_table_mut() {
            mcp_table.insert(name, toml_edit::Item::Table(server_table));
        }
    }

    // Write back
    std::fs::write(&config_path, doc.to_string()).context("failed to write great.toml")?;

    output::success(&format!("Added MCP server '{}' to great.toml", name));
    output::info("Run `great apply` to configure it in .mcp.json.");

    Ok(())
}

/// Test one or all MCP servers declared in `great.toml` by attempting to spawn them.
fn run_test(name: Option<&str>) -> Result<()> {
    output::header("Testing MCP Servers");
    println!();

    let cfg = match config::discover_config()
        .and_then(|p| config::load(Some(p.to_str().unwrap_or_default())))
    {
        Ok(cfg) => cfg,
        Err(_) => {
            output::error("No valid great.toml found.");
            return Ok(());
        }
    };

    let mcps = cfg.mcp.unwrap_or_default();

    if mcps.is_empty() {
        output::warning("No MCP servers declared in great.toml.");
        return Ok(());
    }

    let servers_to_test: Vec<(&String, &crate::config::schema::McpConfig)> = match name {
        Some(n) => match mcps.get_key_value(n) {
            Some(pair) => vec![pair],
            None => {
                output::error(&format!("MCP server '{}' not found in great.toml", n));
                return Ok(());
            }
        },
        None => mcps.iter().collect(),
    };

    for (server_name, server_config) in servers_to_test {
        // First check if command exists on PATH
        if !command_exists(&server_config.command) {
            output::error(&format!(
                "  {} — command '{}' not found",
                server_name, server_config.command
            ));
            continue;
        }

        let spinner = output::spinner(&format!("Testing {}...", server_name));
        match mcp::test_server(server_config) {
            Ok(true) => {
                spinner.finish_and_clear();
                output::success(&format!("  {} — server starts successfully", server_name));
            }
            Ok(false) => {
                spinner.finish_and_clear();
                output::error(&format!("  {} — server failed to start", server_name));
            }
            Err(e) => {
                spinner.finish_and_clear();
                output::error(&format!("  {} — error: {}", server_name, e));
            }
        }
    }

    Ok(())
}
