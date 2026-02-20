use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists};

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

/// Run the `great status` command.
///
/// Detects the current platform, attempts to load `great.toml`, and prints a
/// color-coded report covering declared tools, MCP servers, agents, and
/// required secrets. When no config file is found the command still succeeds,
/// showing platform-only information with a helpful hint.
pub fn run(args: Args) -> Result<()> {
    let info = platform::detect_platform_info();

    if args.json {
        return run_json(&info);
    }

    output::header("great status");
    println!();

    // -- Platform section ---------------------------------------------------
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

    // -- Config section -----------------------------------------------------
    let config = match config::discover_config() {
        Ok(path) => {
            output::info(&format!("Config: {}", path.display()));
            match config::load(Some(path.to_str().unwrap_or_default())) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    output::error(&format!("Failed to parse config: {}", e));
                    None
                }
            }
        }
        Err(_) => {
            output::warning("No great.toml found. Run `great init` to create one.");
            None
        }
    };

    if let Some(cfg) = &config {
        // -- Tools section --------------------------------------------------
        if let Some(tools) = &cfg.tools {
            println!();
            output::header("Tools");

            // Runtime tools (flattened keys excluding "cli")
            for (name, version) in &tools.runtimes {
                if name == "cli" {
                    continue;
                }
                let installed = command_exists(name);
                let actual_version = if installed {
                    get_tool_version(name)
                } else {
                    None
                };
                print_tool_status(name, version, installed, actual_version.as_deref());
            }

            // CLI tools from [tools.cli]
            if let Some(cli_tools) = &tools.cli {
                for (name, version) in cli_tools {
                    let installed = command_exists(name);
                    let actual_version = if installed {
                        get_tool_version(name)
                    } else {
                        None
                    };
                    print_tool_status(name, version, installed, actual_version.as_deref());
                }
            }
        }

        // -- Agents section -------------------------------------------------
        if let Some(agents) = &cfg.agents {
            println!();
            output::header("Agents");
            for (name, agent) in agents {
                let provider = agent.provider.as_deref().unwrap_or("unknown");
                let model = agent.model.as_deref().unwrap_or("default");
                output::info(&format!("  {} ({}/{})", name, provider, model));
            }
        }

        // -- MCP Servers section --------------------------------------------
        if let Some(mcps) = &cfg.mcp {
            println!();
            output::header("MCP Servers");
            for (name, mcp) in mcps {
                let cmd_available = command_exists(&mcp.command);
                if cmd_available {
                    output::success(&format!("  {} ({})", name, mcp.command));
                } else {
                    output::error(&format!("  {} ({} — not found)", name, mcp.command));
                }
            }
        }

        // -- Secrets section ------------------------------------------------
        if let Some(secrets) = &cfg.secrets {
            if let Some(required) = &secrets.required {
                println!();
                output::header("Secrets");
                for key in required {
                    if std::env::var(key).is_ok() {
                        output::success(&format!("  {} — set", key));
                    } else {
                        output::error(&format!("  {} — missing", key));
                    }
                }
            }
        }
    }

    println!();
    Ok(())
}

/// Output basic platform information as JSON to stdout (for piping / scripting).
fn run_json(info: &platform::PlatformInfo) -> Result<()> {
    println!(
        r#"{{"platform": "{}", "arch": "{:?}", "shell": "{}"}}"#,
        info.platform,
        info.platform.arch(),
        info.shell
    );
    Ok(())
}

/// Try to get a tool's version by running `<tool> --version`.
///
/// Returns the first non-empty line of stdout, trimmed. Returns `None` if the
/// command fails or produces no output.
fn get_tool_version(tool: &str) -> Option<String> {
    let output = std::process::Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("").trim();
        if first_line.is_empty() {
            None
        } else {
            Some(first_line.to_string())
        }
    } else {
        None
    }
}

/// Print a single tool's status line with color coding.
///
/// Shows a green checkmark when the tool is installed (with the detected
/// version), or a red cross when it is missing.
fn print_tool_status(
    name: &str,
    declared_version: &str,
    installed: bool,
    actual_version: Option<&str>,
) {
    if installed {
        let ver_info = actual_version.unwrap_or("installed");
        output::success(&format!("  {} {} ({})", name, declared_version, ver_info));
    } else {
        output::error(&format!("  {} {} — not installed", name, declared_version));
    }
}
