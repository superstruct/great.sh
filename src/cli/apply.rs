use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists};
use crate::platform::package_manager;
use crate::platform::runtime::{MiseManager, ProvisionAction};

/// Apply the configuration from great.toml — install tools, configure MCP servers,
/// and resolve secrets.
#[derive(ClapArgs)]
pub struct Args {
    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(long, short)]
    pub yes: bool,
}

/// Execute the `great apply` command.
///
/// Reads `great.toml`, detects the platform, then walks through each
/// configuration section — runtimes (via mise), CLI tools (via package
/// managers), MCP servers (`.mcp.json`), required secrets, and
/// platform-specific overrides — applying or previewing changes.
pub fn run(args: Args) -> Result<()> {
    output::header("great apply");
    println!();

    // 1. Load config
    let config_path = match &args.config {
        Some(p) => std::path::PathBuf::from(p),
        None => config::discover_config()
            .context("no great.toml found — run `great init` to create one")?,
    };

    output::info(&format!("Config: {}", config_path.display()));
    let cfg = config::load(config_path.to_str())?;

    // 2. Detect platform
    let info = platform::detect_platform_info();
    output::info(&format!("Platform: {}", info.platform.display_detailed()));
    println!();

    if args.dry_run {
        output::warning("Dry run mode — no changes will be made");
        println!();
    }

    // 2b. Ensure Homebrew is available (primary package manager for macOS, Ubuntu, and WSL Ubuntu).
    // Homebrew (Linuxbrew) is preferred over apt for CLI tools because it provides
    // up-to-date versions without needing sudo. Apt is kept only as a fallback for
    // system-level packages (e.g. docker, chrome from official repos).
    let needs_homebrew = match &info.platform {
        platform::Platform::MacOS { .. } => true,
        platform::Platform::Linux { distro, .. } | platform::Platform::Wsl { distro, .. } => {
            matches!(
                distro,
                platform::LinuxDistro::Ubuntu | platform::LinuxDistro::Debian
            )
        }
        _ => false,
    };

    if needs_homebrew && !info.capabilities.has_homebrew {
        let platform_label = match &info.platform {
            platform::Platform::MacOS { .. } => "macOS",
            platform::Platform::Wsl { .. } => "WSL Ubuntu",
            _ => "Ubuntu/Debian",
        };
        if args.dry_run {
            output::info(&format!(
                "Homebrew not found — would install (primary package manager for {})",
                platform_label
            ));
        } else {
            output::warning(&format!(
                "Homebrew not found — installing (primary package manager for {})...",
                platform_label
            ));
            let status = std::process::Command::new("bash")
                .args([
                    "-c",
                    "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"",
                ])
                .status();
            match status {
                Ok(s) if s.success() => {
                    output::success("Homebrew installed successfully");
                    // On Linux, brew is installed to /home/linuxbrew/.linuxbrew or ~/.linuxbrew.
                    // The user's shell profile needs `eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"`
                    // but that only takes effect in new shells. For this session, try to add it to PATH.
                    if !matches!(info.platform, platform::Platform::MacOS { .. }) {
                        output::info("Note: You may need to run `eval \"$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)\"` or restart your shell.");
                    }
                }
                _ => {
                    output::error("Failed to install Homebrew — some tools may not install");
                    output::info(
                        "Install manually: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"",
                    );
                }
            }
        }
        println!();
    }

    // 3. Install runtimes via mise
    if let Some(tools) = &cfg.tools {
        // Check if there are any runtimes to install (exclude "cli" key)
        let has_runtimes = tools.runtimes.keys().any(|k| k != "cli");
        if has_runtimes {
            output::header("Runtimes (via mise)");

            if args.dry_run {
                for (name, version) in &tools.runtimes {
                    if name == "cli" {
                        continue;
                    }
                    let current = MiseManager::installed_version(name);
                    match current {
                        Some(cur) if MiseManager::version_matches(version, &cur) => {
                            output::success(&format!(
                                "  {} {} — already at {}",
                                name, version, cur
                            ));
                        }
                        Some(cur) => {
                            output::warning(&format!(
                                "  {} {} — currently {} (would update)",
                                name, version, cur
                            ));
                        }
                        None => {
                            output::info(&format!("  {} {} — would install", name, version));
                        }
                    }
                }
            } else {
                // Ensure mise is available
                if !MiseManager::is_available() {
                    output::warning("mise not found — installing...");
                    if let Err(e) = MiseManager::ensure_installed() {
                        output::error(&format!("Failed to install mise: {}", e));
                        output::warning(
                            "Skipping runtime installation. Install mise manually: https://mise.jdx.dev",
                        );
                    }
                }

                if MiseManager::is_available() {
                    let results = MiseManager::provision_from_config(tools);
                    for result in &results {
                        match &result.action {
                            ProvisionAction::AlreadyCorrect => {
                                output::success(&format!(
                                    "  {} {} — up to date",
                                    result.name, result.declared_version
                                ));
                            }
                            ProvisionAction::Installed => {
                                output::success(&format!(
                                    "  {} {} — installed",
                                    result.name, result.declared_version
                                ));
                            }
                            ProvisionAction::Updated => {
                                output::success(&format!(
                                    "  {} {} — updated",
                                    result.name, result.declared_version
                                ));
                            }
                            ProvisionAction::Failed(err) => {
                                output::error(&format!(
                                    "  {} {} — failed: {}",
                                    result.name, result.declared_version, err
                                ));
                            }
                        }
                    }
                }
            }
            println!();
        }

        // 4. Install CLI tools via package managers
        //
        // TODO: Some CLI tools need special install paths that differ from
        // a simple `brew install <name>` / `apt install <name>`:
        //   - cdk:    `npm install -g aws-cdk`
        //   - az:     `brew install azure-cli` (name differs) or curl installer on Linux
        //   - gcloud: `brew install google-cloud-sdk` or snap/curl on Linux
        //   - aws:    `brew install awscli` (name differs) or curl installer on Linux
        //   - pnpm:   `npm install -g pnpm` or `brew install pnpm`
        //   - uv:     `brew install uv` or `pip install uv` or curl installer
        //   - starship: `brew install starship` (then needs shell init config)
        //   - bitwarden-cli: `npm install -g @bitwarden/cli`
        //
        // Consider adding a tool-name-to-install-command mapping table, or
        // an `install_hint` field in the schema for tools that need it.
        if let Some(cli_tools) = &tools.cli {
            if !cli_tools.is_empty() {
                output::header("CLI Tools");
                let managers = package_manager::available_managers();

                for (name, version) in cli_tools {
                    if command_exists(name) {
                        output::success(&format!("  {} — already installed", name));
                        continue;
                    }

                    if args.dry_run {
                        output::info(&format!("  {} {} — would install", name, version));
                        continue;
                    }

                    // Try to install via available package managers
                    let version_opt = if version == "latest" {
                        None
                    } else {
                        Some(version.as_str())
                    };
                    let mut installed = false;

                    for mgr in &managers {
                        match mgr.install(name, version_opt) {
                            Ok(()) => {
                                output::success(&format!(
                                    "  {} — installed via {}",
                                    name,
                                    mgr.name()
                                ));
                                installed = true;
                                break;
                            }
                            Err(_) => continue, // Try next manager
                        }
                    }

                    if !installed {
                        output::error(&format!(
                            "  {} — could not install (no package manager succeeded)",
                            name
                        ));
                    }
                }
                println!();
            }
        }
    }

    // 5. Configure MCP servers
    if let Some(mcps) = &cfg.mcp {
        if !mcps.is_empty() {
            output::header("MCP Servers");

            let mcp_json_path = Path::new(".mcp.json");
            let mut mcp_config: HashMap<String, serde_json::Value> = if mcp_json_path.exists() {
                let content = std::fs::read_to_string(mcp_json_path)
                    .context("failed to read .mcp.json")?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                HashMap::new()
            };

            // Get the mcpServers object, or create it
            let servers = mcp_config
                .entry("mcpServers".to_string())
                .or_insert_with(|| serde_json::json!({}));

            let servers_obj = match servers.as_object_mut() {
                Some(obj) => obj,
                None => {
                    output::error("  .mcp.json mcpServers is not an object");
                    return Ok(());
                }
            };

            let mut changed = false;

            for (name, mcp) in mcps {
                // Check if already configured
                if servers_obj.contains_key(name) {
                    output::success(&format!("  {} — already configured", name));
                    continue;
                }

                if args.dry_run {
                    output::info(&format!(
                        "  {} — would configure ({})",
                        name, mcp.command
                    ));
                    continue;
                }

                // Build the MCP server config entry
                let mut server_entry = serde_json::json!({
                    "command": mcp.command,
                });

                if let Some(args_list) = &mcp.args {
                    server_entry["args"] = serde_json::json!(args_list);
                }

                // Resolve env vars — replace ${SECRET_NAME} with actual values
                if let Some(env) = &mcp.env {
                    let mut resolved_env = serde_json::Map::new();
                    for (key, value) in env {
                        let resolved = resolve_secret_refs(value);
                        resolved_env.insert(key.clone(), serde_json::Value::String(resolved));
                    }
                    server_entry["env"] = serde_json::Value::Object(resolved_env);
                }

                servers_obj.insert(name.clone(), server_entry);
                output::success(&format!("  {} — configured ({})", name, mcp.command));
                changed = true;
            }

            // Write .mcp.json if changed
            if changed && !args.dry_run {
                let json = serde_json::to_string_pretty(&mcp_config)
                    .context("failed to serialize .mcp.json")?;
                std::fs::write(mcp_json_path, json).context("failed to write .mcp.json")?;
                output::info("  Updated .mcp.json");
            }

            println!();
        }
    }

    // 5b. Install bitwarden-cli if secrets provider is bitwarden and bw is missing
    if let Some(secrets) = &cfg.secrets {
        if secrets.provider.as_deref() == Some("bitwarden") && !command_exists("bw") {
            if args.dry_run {
                output::info("bitwarden-cli (bw) — would install (secrets provider is bitwarden)");
            } else {
                output::header("Bitwarden CLI");
                output::info("Secrets provider is bitwarden — installing bw CLI...");
                // TODO: Try npm install -g @bitwarden/cli first, then brew/apt fallback
                let managers = package_manager::available_managers();
                let mut installed = false;
                for mgr in &managers {
                    if mgr.install("@bitwarden/cli", None).is_ok() {
                        output::success(&format!("  bw — installed via {}", mgr.name()));
                        installed = true;
                        break;
                    }
                }
                if !installed {
                    output::error("  bw — could not install. Install manually: npm install -g @bitwarden/cli");
                }
                println!();
            }
        }
    }

    // 5c. Configure Starship prompt if starship is in CLI tools
    // TODO: After installing starship, we should also:
    //   - Generate ~/.config/starship.toml with a great.sh preset
    //   - Detect the user's shell and add the init line to their profile:
    //     bash:  eval "$(starship init bash)"  -> ~/.bashrc
    //     zsh:   eval "$(starship init zsh)"   -> ~/.zshrc
    //     fish:  starship init fish | source   -> ~/.config/fish/config.fish
    //   - This requires a new `configure_starship()` helper in platform/

    // 6. Check secrets
    if let Some(secrets) = &cfg.secrets {
        if let Some(required) = &secrets.required {
            let missing: Vec<&String> =
                required.iter().filter(|k| std::env::var(k).is_err()).collect();
            if !missing.is_empty() {
                output::header("Secrets");
                for key in &missing {
                    output::warning(&format!(
                        "  {} — not set (set via environment or `great vault set {}`)",
                        key, key
                    ));
                }
                println!();
            }
        }
    }

    // 7. Apply platform-specific overrides
    if let Some(platform_cfg) = &cfg.platform {
        let override_tools = match &info.platform {
            platform::Platform::MacOS { .. } => {
                platform_cfg.macos.as_ref().and_then(|o| o.extra_tools.as_ref())
            }
            platform::Platform::Wsl { .. } => {
                platform_cfg.wsl2.as_ref().and_then(|o| o.extra_tools.as_ref())
            }
            platform::Platform::Linux { .. } => {
                platform_cfg.linux.as_ref().and_then(|o| o.extra_tools.as_ref())
            }
            _ => None,
        };

        if let Some(extra_tools) = override_tools {
            if !extra_tools.is_empty() {
                output::header("Platform-specific tools");
                let managers = package_manager::available_managers();
                for tool in extra_tools {
                    if command_exists(tool) {
                        output::success(&format!("  {} — already installed", tool));
                        continue;
                    }
                    if args.dry_run {
                        output::info(&format!("  {} — would install", tool));
                        continue;
                    }
                    let mut installed = false;
                    for mgr in &managers {
                        if mgr.install(tool, None).is_ok() {
                            output::success(&format!(
                                "  {} — installed via {}",
                                tool,
                                mgr.name()
                            ));
                            installed = true;
                            break;
                        }
                    }
                    if !installed {
                        output::error(&format!("  {} — could not install", tool));
                    }
                }
                println!();
            }
        }
    }

    // Summary
    if args.dry_run {
        output::info("Dry run complete. Run `great apply` without --dry-run to apply changes.");
    } else {
        output::success("Apply complete.");
    }

    Ok(())
}

/// Replace `${SECRET_NAME}` references in a string with environment variable values.
///
/// Scans for patterns like `${POSTGRES_URL}` and substitutes the value of the
/// corresponding environment variable. If the variable is not set, the reference
/// is left as-is so the user can see what is missing.
fn resolve_secret_refs(value: &str) -> String {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").expect("valid regex");
    re.replace_all(value, |caps: &regex::Captures| {
        let var_name = &caps[1];
        std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_secret_refs_with_env() {
        std::env::set_var("GREAT_TEST_SECRET", "hunter2");
        let result = resolve_secret_refs("postgres://${GREAT_TEST_SECRET}@localhost");
        assert_eq!(result, "postgres://hunter2@localhost");
        std::env::remove_var("GREAT_TEST_SECRET");
    }

    #[test]
    fn test_resolve_secret_refs_missing_env() {
        let result = resolve_secret_refs("key=${DEFINITELY_NOT_SET_XYZ_12345}");
        assert_eq!(result, "key=${DEFINITELY_NOT_SET_XYZ_12345}");
    }

    #[test]
    fn test_resolve_secret_refs_no_refs() {
        let result = resolve_secret_refs("plain string with no references");
        assert_eq!(result, "plain string with no references");
    }

    #[test]
    fn test_resolve_secret_refs_multiple() {
        std::env::set_var("GREAT_TEST_A", "alpha");
        std::env::set_var("GREAT_TEST_B", "beta");
        let result = resolve_secret_refs("${GREAT_TEST_A} and ${GREAT_TEST_B}");
        assert_eq!(result, "alpha and beta");
        std::env::remove_var("GREAT_TEST_A");
        std::env::remove_var("GREAT_TEST_B");
    }

    #[test]
    fn test_resolve_secret_refs_empty_string() {
        let result = resolve_secret_refs("");
        assert_eq!(result, "");
    }
}
