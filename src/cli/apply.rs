use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::cli::{bootstrap, tuning};
use crate::config;
use crate::platform::package_manager::{self, PackageManager};
use crate::platform::runtime::{MiseManager, ProvisionAction};
use crate::platform::{self, command_exists, Platform, PlatformInfo};

// ── Nerd Font support ────────────────────────────────────────────────

const NERD_FONT_VERSION: &str = "v3.4.0";
const NERD_FONT_BASE_URL: &str = "https://github.com/ryanoasis/nerd-fonts/releases/download";

struct NerdFontSpec {
    display_name: &'static str,
    zip_name: &'static str,
    brew_cask: &'static str,
    file_prefix: &'static str,
}

fn nerd_font_for_platform(platform: &Platform) -> NerdFontSpec {
    match platform {
        Platform::MacOS { .. } => NerdFontSpec {
            display_name: "MesloLG Nerd Font",
            zip_name: "Meslo",
            brew_cask: "font-meslo-lg-nerd-font",
            file_prefix: "MesloLGS",
        },
        _ => NerdFontSpec {
            display_name: "UbuntuSans Nerd Font",
            zip_name: "UbuntuSans",
            brew_cask: "font-ubuntu-sans-nerd-font",
            file_prefix: "UbuntuSansNerdFont",
        },
    }
}

/// Check whether any file in `dir` starts with `file_prefix`.
fn has_nerd_font(dir: &Path, file_prefix: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().starts_with(file_prefix))
}

/// Return true if the platform-appropriate Nerd Font is already installed.
fn nerd_font_installed(platform: &Platform, spec: &NerdFontSpec) -> bool {
    match platform {
        Platform::MacOS { .. } => {
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return false,
            };
            has_nerd_font(&home.join("Library/Fonts"), spec.file_prefix)
                || has_nerd_font(Path::new("/Library/Fonts"), spec.file_prefix)
        }
        _ => {
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return false,
            };
            has_nerd_font(&home.join(".local/share/fonts"), spec.file_prefix)
        }
    }
}

/// Download a Nerd Font zip and extract `.ttf` files to `~/.local/share/fonts`.
fn download_and_install_nerd_font(home: &Path, spec: &NerdFontSpec) -> Result<()> {
    let url = format!(
        "{}/{}/{}.zip",
        NERD_FONT_BASE_URL, NERD_FONT_VERSION, spec.zip_name
    );

    let sp = output::spinner(&format!("Downloading {} ...", spec.display_name));

    let response =
        reqwest::blocking::get(&url).with_context(|| format!("failed to download {}", url))?;
    let bytes = response.bytes().context("failed to read font zip bytes")?;

    sp.set_message(format!("Extracting {} ...", spec.display_name));

    let font_dir = home.join(".local/share/fonts");
    std::fs::create_dir_all(&font_dir).context("failed to create ~/.local/share/fonts")?;

    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).context("failed to open font zip")?;

    let mut count = 0usize;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.ends_with(".ttf") {
            // Use only the filename, not any directory prefix inside the zip
            let file_name = Path::new(&name)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(name.clone());

            let out_path = font_dir.join(&file_name);
            let mut out_file = std::fs::File::create(&out_path)
                .with_context(|| format!("failed to create {}", out_path.display()))?;
            std::io::copy(&mut file, &mut out_file)?;
            count += 1;
        }
    }

    sp.finish_and_clear();

    if count == 0 {
        anyhow::bail!("no .ttf files found in {}.zip", spec.zip_name);
    }

    // Rebuild font cache
    let _ = std::process::Command::new("fc-cache")
        .args(["-fv"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    Ok(())
}

/// Copy Nerd Font files from the Linux font dir to the Windows per-user font dir (WSL2).
fn copy_fonts_to_windows(home: &Path, file_prefix: &str) -> Result<()> {
    // Get Windows username
    let output = std::process::Command::new("cmd.exe")
        .args(["/c", "echo", "%USERNAME%"])
        .output()
        .context("failed to run cmd.exe to get Windows username")?;
    let win_user = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if win_user.is_empty() || win_user.contains('%') {
        anyhow::bail!("could not determine Windows username");
    }

    let win_font_dir = Path::new("/mnt/c/Users")
        .join(&win_user)
        .join("AppData/Local/Microsoft/Windows/Fonts");

    if !win_font_dir.exists() {
        std::fs::create_dir_all(&win_font_dir)
            .context("failed to create Windows per-user font directory")?;
    }

    let linux_font_dir = home.join(".local/share/fonts");
    let entries =
        std::fs::read_dir(&linux_font_dir).context("failed to read Linux font directory")?;

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(file_prefix) && name.ends_with(".ttf") {
            let dest = win_font_dir.join(&name);
            if let Err(e) = std::fs::copy(entry.path(), &dest) {
                output::warning(&format!(
                    "  Could not copy {} to Windows fonts: {}",
                    name, e
                ));
            }
        }
    }

    Ok(())
}

/// Install a Nerd Font appropriate for the current platform.
/// Errors are reported but never block the rest of `great apply`.
fn install_nerd_font(dry_run: bool, platform_info: &PlatformInfo) {
    let spec = nerd_font_for_platform(&platform_info.platform);

    if nerd_font_installed(&platform_info.platform, &spec) {
        output::success(&format!("  {} — already installed", spec.display_name));
        return;
    }

    if dry_run {
        output::info(&format!("  {} — would install", spec.display_name));
        return;
    }

    match &platform_info.platform {
        Platform::MacOS { .. } => {
            let status = std::process::Command::new("brew")
                .args(["install", "--cask", spec.brew_cask])
                .status();
            match status {
                Ok(s) if s.success() => {
                    output::success(&format!("  {} — installed via Homebrew", spec.display_name));
                }
                _ => {
                    output::error(&format!(
                        "  {} — failed to install via brew. Run: brew install --cask {}",
                        spec.display_name, spec.brew_cask
                    ));
                }
            }
        }
        Platform::Wsl { .. } => {
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => {
                    output::error("  Could not determine home directory for font install");
                    return;
                }
            };
            match download_and_install_nerd_font(&home, &spec) {
                Ok(()) => {
                    output::success(&format!("  {} — installed", spec.display_name));
                    // Also copy to Windows side so the terminal can use them
                    if let Err(e) = copy_fonts_to_windows(&home, spec.file_prefix) {
                        output::warning(&format!(
                            "  Could not copy fonts to Windows (install manually in Windows Terminal settings): {}",
                            e
                        ));
                    } else {
                        output::success("  Nerd Font files copied to Windows user fonts");
                        output::info(
                            "  Note: You may need to select the font in your terminal settings",
                        );
                    }
                }
                Err(e) => {
                    output::error(&format!(
                        "  {} — failed to install: {}",
                        spec.display_name, e
                    ));
                }
            }
        }
        _ => {
            // Generic Linux
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => {
                    output::error("  Could not determine home directory for font install");
                    return;
                }
            };
            match download_and_install_nerd_font(&home, &spec) {
                Ok(()) => {
                    output::success(&format!("  {} — installed", spec.display_name));
                }
                Err(e) => {
                    output::error(&format!(
                        "  {} — failed to install: {}",
                        spec.display_name, e
                    ));
                }
            }
        }
    }
}

/// Special install instructions for CLI tools that can't use a simple
/// `brew install <name>` or `apt install <name>`.
struct ToolInstallSpec {
    /// Homebrew formula name (if different from the tool name in great.toml).
    brew_name: Option<&'static str>,
    /// npm package name for `npm install -g`.
    npm_package: Option<&'static str>,
    /// The binary name to check on PATH after install.
    binary_name: &'static str,
}

/// Look up special install instructions for a CLI tool.
fn tool_install_spec(name: &str) -> Option<ToolInstallSpec> {
    match name {
        "cdk" => Some(ToolInstallSpec {
            brew_name: None,
            npm_package: Some("aws-cdk"),
            binary_name: "cdk",
        }),
        "aws" => Some(ToolInstallSpec {
            brew_name: Some("awscli"),
            npm_package: None,
            binary_name: "aws",
        }),
        "az" => Some(ToolInstallSpec {
            brew_name: Some("azure-cli"),
            npm_package: None,
            binary_name: "az",
        }),
        "gcloud" => Some(ToolInstallSpec {
            brew_name: Some("google-cloud-sdk"),
            npm_package: None,
            binary_name: "gcloud",
        }),
        "pnpm" => Some(ToolInstallSpec {
            brew_name: Some("pnpm"),
            npm_package: Some("pnpm"),
            binary_name: "pnpm",
        }),
        "uv" => Some(ToolInstallSpec {
            brew_name: Some("uv"),
            npm_package: None,
            binary_name: "uv",
        }),
        "starship" => Some(ToolInstallSpec {
            brew_name: Some("starship"),
            npm_package: None,
            binary_name: "starship",
        }),
        "bw" | "bitwarden-cli" => Some(ToolInstallSpec {
            brew_name: None,
            npm_package: Some("@bitwarden/cli"),
            binary_name: "bw",
        }),
        _ => None,
    }
}

/// Try to install a tool using its special install spec.
/// Returns Ok(Some(method)) on success, Ok(None) if no method worked.
fn install_with_spec(
    spec: &ToolInstallSpec,
    managers: &[Box<dyn PackageManager>],
    version_opt: Option<&str>,
) -> Result<Option<String>> {
    // Try npm first if npm_package is specified
    if let Some(npm_pkg) = spec.npm_package {
        for mgr in managers {
            if mgr.name() == "npm"
                && mgr.install(npm_pkg, version_opt).is_ok()
                && command_exists(spec.binary_name)
            {
                return Ok(Some("npm".to_string()));
            }
        }
    }

    // Try brew with special formula name
    if let Some(brew_name) = spec.brew_name {
        for mgr in managers {
            if mgr.name() == "homebrew"
                && mgr.install(brew_name, version_opt).is_ok()
                && command_exists(spec.binary_name)
            {
                return Ok(Some("homebrew".to_string()));
            }
        }
    }

    Ok(None)
}

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

    /// Set by main.rs from the global --non-interactive flag.
    /// Not a CLI argument -- hidden from clap.
    #[arg(skip)]
    pub non_interactive: bool,
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

    // 2a. Pre-cache sudo credentials before any installs that need root.
    // Note: The `needs_sudo` platform check intentionally duplicates the `needs_homebrew`
    // match at line ~410 because sudo must be cached *before* `ensure_prerequisites()`
    // (which runs `sudo apt-get`), and `needs_homebrew` is computed after that call.
    let needs_sudo = !args.dry_run && {
        let needs_homebrew = match &info.platform {
            platform::Platform::MacOS { .. } => true,
            platform::Platform::Linux { distro, .. }
            | platform::Platform::Wsl { distro, .. } => {
                matches!(
                    distro,
                    platform::LinuxDistro::Ubuntu | platform::LinuxDistro::Debian
                )
            }
            _ => false,
        };
        (needs_homebrew && !info.capabilities.has_homebrew)
            || bootstrap::is_apt_distro(&info.platform)
    };

    let _sudo_keepalive = if needs_sudo {
        use crate::cli::sudo::{ensure_sudo_cached, SudoCacheResult};
        match ensure_sudo_cached(info.is_root, args.non_interactive) {
            SudoCacheResult::Cached(keepalive) => Some(keepalive),
            _ => None,
        }
    } else {
        None
    };

    // 2b. System prerequisites — before Homebrew since Homebrew needs curl/git/build tools.
    bootstrap::ensure_prerequisites(args.dry_run, &info);

    // 2c. Ensure Homebrew is available (primary package manager for macOS, Ubuntu, and WSL Ubuntu).
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

        // 4. Install CLI tools via package managers (with special-case handling)
        if let Some(cli_tools) = &tools.cli {
            if !cli_tools.is_empty() {
                output::header("CLI Tools");
                let managers = package_manager::available_managers(args.non_interactive);

                for (name, version) in cli_tools {
                    // Check binary name — some tools have different binary vs config names
                    let check_name = tool_install_spec(name)
                        .map(|s| s.binary_name)
                        .unwrap_or(name.as_str());

                    if command_exists(check_name) {
                        output::success(&format!("  {} — already installed", name));
                        continue;
                    }

                    if args.dry_run {
                        output::info(&format!("  {} {} — would install", name, version));
                        continue;
                    }

                    let version_opt = if version == "latest" {
                        None
                    } else {
                        Some(version.as_str())
                    };

                    // Try special install spec first
                    let mut installed = false;
                    if let Some(spec) = tool_install_spec(name) {
                        match install_with_spec(&spec, &managers, version_opt) {
                            Ok(Some(method)) => {
                                output::success(&format!(
                                    "  {} — installed via {} (special)",
                                    name, method
                                ));
                                installed = true;
                            }
                            Ok(None) => {} // Fall through to generic install
                            Err(e) => {
                                output::error(&format!("  {} — install error: {}", name, e));
                                continue;
                            }
                        }
                    }

                    // Fall back to generic package manager install
                    if !installed {
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
                                Err(_) => continue,
                            }
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
                let content =
                    std::fs::read_to_string(mcp_json_path).context("failed to read .mcp.json")?;
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
                    output::info(&format!("  {} — would configure ({})", name, mcp.command));
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
                let managers = package_manager::available_managers(args.non_interactive);
                let spec = tool_install_spec("bw").expect("bw has install spec");
                match install_with_spec(&spec, &managers, None) {
                    Ok(Some(method)) => {
                        output::success(&format!("  bw — installed via {}", method));
                    }
                    _ => {
                        output::error(
                            "  bw — could not install. Install manually: npm install -g @bitwarden/cli",
                        );
                    }
                }
                println!();
            }
        }
    }

    // 5c. Configure Starship prompt and Nerd Font
    let has_starship_in_config = cfg
        .tools
        .as_ref()
        .and_then(|t| t.cli.as_ref())
        .map(|cli| cli.contains_key("starship"))
        .unwrap_or(false);

    if has_starship_in_config {
        if command_exists("starship") {
            configure_starship(args.dry_run);
        }
        install_nerd_font(args.dry_run, &info);
    }

    // 6. Check secrets
    if let Some(secrets) = &cfg.secrets {
        if let Some(required) = &secrets.required {
            let missing: Vec<&String> = required
                .iter()
                .filter(|k| std::env::var(k).is_err())
                .collect();
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
            platform::Platform::MacOS { .. } => platform_cfg
                .macos
                .as_ref()
                .and_then(|o| o.extra_tools.as_ref()),
            platform::Platform::Wsl { .. } => platform_cfg
                .wsl2
                .as_ref()
                .and_then(|o| o.extra_tools.as_ref()),
            platform::Platform::Linux { .. } => platform_cfg
                .linux
                .as_ref()
                .and_then(|o| o.extra_tools.as_ref()),
            _ => None,
        };

        if let Some(extra_tools) = override_tools {
            if !extra_tools.is_empty() {
                output::header("Platform-specific tools");
                let managers = package_manager::available_managers(args.non_interactive);
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
                            output::success(&format!("  {} — installed via {}", tool, mgr.name()));
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

    // 8. Docker
    bootstrap::ensure_docker(args.dry_run, &info);

    // 9. Claude Code
    output::header("Claude Code");
    bootstrap::ensure_claude_code(args.dry_run);
    println!();

    // 10. System tuning (Linux/WSL only)
    tuning::apply_system_tuning(args.dry_run, &info);

    // Summary
    if args.dry_run {
        output::info("Dry run complete. Run `great apply` without --dry-run to apply changes.");
    } else {
        output::success("Apply complete.");
    }

    Ok(())
}

/// Configure Starship prompt: generate config file and add shell init lines.
fn configure_starship(dry_run: bool) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };

    // 1. Generate ~/.config/starship.toml if it doesn't exist
    let starship_config = home.join(".config").join("starship.toml");
    if !starship_config.exists() {
        if dry_run {
            output::info("  starship — would create ~/.config/starship.toml");
        } else {
            let preset = r#"# great.sh starship preset
format = "$all"

[character]
success_symbol = "[➜](bold green)"
error_symbol = "[✗](bold red)"

[directory]
truncation_length = 3

[git_branch]
symbol = " "

[nodejs]
symbol = " "

[python]
symbol = " "

[rust]
symbol = " "
"#;
            if let Some(parent) = starship_config.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&starship_config, preset) {
                Ok(()) => output::success("  starship — created ~/.config/starship.toml"),
                Err(e) => output::error(&format!("  starship — failed to write config: {}", e)),
            }
        }
    }

    // 2. Add shell init line to the user's shell profile
    let shell = std::env::var("SHELL").unwrap_or_default();
    let (profile_path, init_line) = if shell.contains("zsh") {
        (home.join(".zshrc"), "eval \"$(starship init zsh)\"")
    } else if shell.contains("fish") {
        (
            home.join(".config").join("fish").join("config.fish"),
            "starship init fish | source",
        )
    } else {
        // Default to bash
        (home.join(".bashrc"), "eval \"$(starship init bash)\"")
    };

    // Check if init line already exists
    let already_configured = profile_path
        .exists()
        .then(|| std::fs::read_to_string(&profile_path).unwrap_or_default())
        .map(|content| content.contains("starship init"))
        .unwrap_or(false);

    if already_configured {
        output::success("  starship — shell init already configured");
    } else if dry_run {
        output::info(&format!(
            "  starship — would add init to {}",
            profile_path.display()
        ));
    } else {
        let line = format!("\n# Added by great.sh\n{}\n", init_line);
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&profile_path)
        {
            Ok(mut f) => {
                use std::io::Write;
                if f.write_all(line.as_bytes()).is_ok() {
                    output::success(&format!(
                        "  starship — added init to {}",
                        profile_path.display()
                    ));
                }
            }
            Err(e) => output::error(&format!(
                "  starship — failed to update {}: {}",
                profile_path.display(),
                e
            )),
        }
    }
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
