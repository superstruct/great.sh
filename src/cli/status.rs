use anyhow::Result;
use clap::Args as ClapArgs;
use serde::Serialize;

use crate::cli::{output, util};
use crate::config;
use crate::platform::{self, command_exists};

// ---------------------------------------------------------------------------
// JSON serialization structs
// ---------------------------------------------------------------------------

/// Top-level JSON output for `great status --json`.
#[derive(Serialize)]
struct StatusReport {
    platform: String,
    arch: String,
    shell: String,
    is_root: bool,
    config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp: Option<Vec<McpStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agents: Option<Vec<AgentStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Vec<SecretStatus>>,
    has_issues: bool,
    issues: Vec<String>,
}

#[derive(Serialize)]
struct ToolStatus {
    name: String,
    declared_version: String,
    installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_version: Option<String>,
}

#[derive(Serialize)]
struct McpStatus {
    name: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    command_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    transport: Option<String>,
}

#[derive(Serialize)]
struct AgentStatus {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Serialize)]
struct SecretStatus {
    name: String,
    is_set: bool,
}

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

/// Arguments for the `great status` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Show detailed status information
    #[arg(long, short)]
    pub verbose: bool,

    /// Output status as JSON
    #[arg(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run the `great status` command.
///
/// Detects the current platform, attempts to load `great.toml`, and prints a
/// color-coded report covering declared tools, MCP servers, agents, and
/// required secrets. When no config file is found the command still succeeds,
/// showing platform-only information with a helpful hint.
pub fn run(args: Args) -> Result<()> {
    let info = platform::detect_platform_info();

    // -- Discover and load config (shared by both output modes) ---------
    let (config_path_str, config) = match config::discover_config() {
        Ok(path) => {
            let path_str = path
                .to_str()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "config path contains non-UTF-8 characters: {}",
                        path.display()
                    )
                })?;
            let path_owned = path_str.to_string();
            match config::load(Some(&path_owned)) {
                Ok(cfg) => (Some(path_owned), Some(cfg)),
                Err(e) => {
                    if !args.json {
                        output::error(&format!("Failed to parse config: {}", e));
                    }
                    (Some(path_owned), None)
                }
            }
        }
        Err(_) => (None, None),
    };

    // -- JSON mode: serialize and exit (always exit 0) ------------------
    if args.json {
        return run_json(&info, config_path_str.as_deref(), config.as_ref());
    }

    // -- Human-readable mode --------------------------------------------
    output::header("great status");
    println!();

    let mut has_critical_issues = false;

    // Platform section
    output::info(&format!("Platform: {}", info.platform.display_detailed()));
    if args.verbose {
        let caps = &info.capabilities;
        let mut cap_list = Vec::new();
        if caps.has_homebrew {
            cap_list.push("homebrew");
        }
        if caps.has_apt {
            cap_list.push("apt");
        }
        if caps.has_dnf {
            cap_list.push("dnf");
        }
        if caps.has_snap {
            cap_list.push("snap");
        }
        if caps.has_systemd {
            cap_list.push("systemd");
        }
        if caps.has_docker {
            cap_list.push("docker");
        }
        if !cap_list.is_empty() {
            output::info(&format!("Capabilities: {}", cap_list.join(", ")));
        }
        output::info(&format!("Shell: {}", info.shell));
        output::info(&format!("Root: {}", info.is_root));
    }

    // Config section
    if let Some(ref path) = config_path_str {
        output::info(&format!("Config: {}", path));
    } else {
        output::warning("No great.toml found. Run `great init` to create one.");
    }

    if let Some(cfg) = &config {
        // Tools section
        if let Some(tools) = &cfg.tools {
            println!();
            output::header("Tools");

            for (name, version) in &tools.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = command_exists(name);
                let actual_version = if installed {
                    util::get_command_version(name)
                } else {
                    has_critical_issues = true;
                    None
                };
                print_tool_status(
                    name,
                    version,
                    installed,
                    actual_version.as_deref(),
                    args.verbose,
                );
            }

            if let Some(cli_tools) = &tools.cli {
                for (name, version) in cli_tools {
                    let installed = command_exists(name);
                    let actual_version = if installed {
                        util::get_command_version(name)
                    } else {
                        has_critical_issues = true;
                        None
                    };
                    print_tool_status(
                        name,
                        version,
                        installed,
                        actual_version.as_deref(),
                        args.verbose,
                    );
                }
            }
        }

        // Agents section
        if let Some(agents) = &cfg.agents {
            println!();
            output::header("Agents");
            for (name, agent) in agents {
                let provider = agent.provider.as_deref().unwrap_or("unknown");
                let model = agent.model.as_deref().unwrap_or("default");
                output::info(&format!("  {} ({}/{})", name, provider, model));
            }
        }

        // MCP Servers section
        if let Some(mcps) = &cfg.mcp {
            println!();
            output::header("MCP Servers");
            for (name, mcp) in mcps {
                let cmd_available = command_exists(&mcp.command);
                if cmd_available {
                    if args.verbose {
                        let args_str = mcp
                            .args
                            .as_ref()
                            .map(|a| a.join(" "))
                            .unwrap_or_default();
                        let transport = mcp.transport.as_deref().unwrap_or("stdio");
                        if args_str.is_empty() {
                            output::success(&format!(
                                "  {} ({} [{}])",
                                name, mcp.command, transport
                            ));
                        } else {
                            output::success(&format!(
                                "  {} ({} {} [{}])",
                                name, mcp.command, args_str, transport
                            ));
                        }
                    } else {
                        output::success(&format!("  {} ({})", name, mcp.command));
                    }
                } else {
                    output::error(&format!(
                        "  {} ({} -- not found)",
                        name, mcp.command
                    ));
                }
            }
        }

        // Secrets section
        if let Some(secrets) = &cfg.secrets {
            if let Some(required) = &secrets.required {
                println!();
                output::header("Secrets");
                for key in required {
                    if std::env::var(key).is_ok() {
                        output::success(&format!("  {} -- set", key));
                    } else {
                        has_critical_issues = true;
                        output::error(&format!("  {} -- missing", key));
                    }
                }
            }
        }
    }

    println!();

    // NOTE: Intentional use of process::exit — the status command must print
    // its full report before exiting non-zero. Using bail!() would abort
    // mid-report, which is wrong for a diagnostic command.
    if has_critical_issues {
        std::process::exit(1);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

/// Serialize full status report as JSON to stdout. Always returns Ok (exit 0).
fn run_json(
    info: &platform::PlatformInfo,
    config_path: Option<&str>,
    config: Option<&config::GreatConfig>,
) -> Result<()> {
    let mut issues: Vec<String> = Vec::new();

    if config.is_none() {
        issues.push("no great.toml found — run `great init` to create one".to_string());
    }

    // Build tools list using explicit if-let to avoid borrow-checker friction
    // with mutable `issues` inside chained closures.
    let tools = if let Some(cfg) = config {
        if let Some(t) = cfg.tools.as_ref() {
            let mut result = Vec::new();
            for (name, version) in &t.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = command_exists(name);
                let actual_version = if installed {
                    util::get_command_version(name)
                } else {
                    issues.push(format!("tool '{}' is not installed", name));
                    None
                };
                result.push(ToolStatus {
                    name: name.clone(),
                    declared_version: version.clone(),
                    installed,
                    actual_version,
                });
            }
            if let Some(cli_tools) = &t.cli {
                for (name, version) in cli_tools {
                    let installed = command_exists(name);
                    let actual_version = if installed {
                        util::get_command_version(name)
                    } else {
                        issues.push(format!("tool '{}' is not installed", name));
                        None
                    };
                    result.push(ToolStatus {
                        name: name.clone(),
                        declared_version: version.clone(),
                        installed,
                        actual_version,
                    });
                }
            }
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    let mcp = config.and_then(|cfg| {
        cfg.mcp.as_ref().map(|mcps| {
            mcps.iter()
                .map(|(name, m)| McpStatus {
                    name: name.clone(),
                    command: m.command.clone(),
                    args: m.args.clone(),
                    command_available: command_exists(&m.command),
                    transport: m.transport.clone(),
                })
                .collect()
        })
    });

    let agents = config.and_then(|cfg| {
        cfg.agents.as_ref().map(|a| {
            a.iter()
                .map(|(name, agent)| AgentStatus {
                    name: name.clone(),
                    provider: agent.provider.clone(),
                    model: agent.model.clone(),
                })
                .collect()
        })
    });

    let secrets = if let Some(cfg) = config {
        if let Some(s) = cfg.secrets.as_ref() {
            if let Some(required) = s.required.as_ref() {
                let result: Vec<SecretStatus> = required
                    .iter()
                    .map(|key| {
                        let is_set = std::env::var(key).is_ok();
                        if !is_set {
                            issues.push(format!("required secret '{}' is missing", key));
                        }
                        SecretStatus {
                            name: key.clone(),
                            is_set,
                        }
                    })
                    .collect();
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let report = StatusReport {
        platform: info.platform.to_string(),
        arch: info.platform.arch().to_string(),
        shell: info.shell.clone(),
        is_root: info.is_root,
        config_path: config_path.map(|s| s.to_string()),
        has_issues: !issues.is_empty(),
        issues,
        tools,
        mcp,
        agents,
        secrets,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Print a single tool's status line with color coding.
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
    verbose: bool,
) {
    if installed {
        let ver_info = match (actual_version, verbose) {
            (Some(full), true) => full,
            (Some(full), false) => full.split_whitespace().last().unwrap_or(full),
            (None, _) => "installed",
        };
        output::success(&format!(
            "  {} {} ({})",
            name, declared_version, ver_info
        ));
    } else {
        output::error(&format!(
            "  {} {} -- not installed",
            name, declared_version
        ));
    }
}
