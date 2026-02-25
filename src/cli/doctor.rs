use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::{bootstrap, output, tuning, util};
use crate::config;
use crate::platform::package_manager;
use crate::platform::{self, command_exists, Platform, PlatformInfo};

/// Arguments for the `great doctor` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}

#[derive(Default)]
struct DiagnosticResult {
    checks_passed: usize,
    checks_warned: usize,
    checks_failed: usize,
    fixable: Vec<FixableIssue>,
}

/// An issue that can potentially be auto-fixed.
struct FixableIssue {
    description: String,
    action: FixAction,
}

/// Actions the doctor can take to fix an issue.
enum FixAction {
    /// Install a tool via package managers. `binary` is the command to check on PATH.
    InstallTool { binary: String, brew_name: String },
    /// Install Homebrew itself.
    InstallHomebrew,
    /// Create the ~/.claude/ directory.
    CreateClaudeDir,
    /// Add ~/.local/bin to PATH in shell profile.
    AddLocalBinToPath,
    /// Install a system prerequisite (curl, git, build-essential, unzip).
    InstallSystemPrerequisite { name: String },
    /// Install Docker CE via official apt repository.
    InstallDocker,
    /// Install Claude Code via the official installer.
    InstallClaudeCode,
    /// Fix inotify max_user_watches below threshold.
    FixInotifyWatches,
}

/// Run the `great doctor` diagnostic command.
pub fn run(args: Args) -> Result<()> {
    if args.fix {
        output::info("Auto-fix mode enabled.");
        println!();
    }

    output::header("great doctor");
    println!();

    let mut result = DiagnosticResult::default();
    let info = platform::detect_platform_info();

    // 1. Platform check
    check_platform(&mut result);

    // 2. System prerequisites check
    check_system_prerequisites(&mut result, &info);

    // 3. Essential tools check
    check_essential_tools(&mut result);

    // 4. Docker check
    check_docker(&mut result, &info);

    // 5. AI agents check
    check_ai_agents(&mut result);

    // 6. Config check — load config here so it can be shared with MCP check
    let loaded_config = check_config(&mut result);

    // 7. MCP server checks (only if config was loaded successfully)
    if let Some(ref cfg) = loaded_config {
        check_mcp_servers(&mut result, cfg);
    }

    // 8. Shell check
    check_shell(&mut result);

    // 9. System tuning check (Linux/WSL only)
    check_system_tuning(&mut result, &info);

    // Attempt auto-fixes if --fix was passed
    if args.fix && !result.fixable.is_empty() {
        println!();
        output::header("Auto-fix");
        let managers = package_manager::available_managers(false);
        let mut fixed = 0;

        for issue in &result.fixable {
            output::info(&format!("Fixing: {}", issue.description));
            match &issue.action {
                FixAction::InstallTool { binary, brew_name } => {
                    let mut ok = false;
                    for mgr in &managers {
                        if mgr.install(brew_name, None).is_ok() && command_exists(binary) {
                            output::success(&format!(
                                "  {} — installed via {}",
                                binary,
                                mgr.name()
                            ));
                            ok = true;
                            fixed += 1;
                            break;
                        }
                    }
                    if !ok {
                        output::error(&format!("  {} — could not install", binary));
                    }
                }
                FixAction::InstallHomebrew => {
                    let status = std::process::Command::new("bash")
                        .args(["-c", "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            output::success("  Homebrew — installed");
                            fixed += 1;
                        }
                        _ => output::error("  Homebrew — install failed"),
                    }
                }
                FixAction::CreateClaudeDir => {
                    if let Some(home) = dirs::home_dir() {
                        let claude_dir = home.join(".claude");
                        match std::fs::create_dir_all(&claude_dir) {
                            Ok(()) => {
                                output::success("  ~/.claude/ — created");
                                fixed += 1;
                            }
                            Err(e) => output::error(&format!("  ~/.claude/ — failed: {}", e)),
                        }
                    }
                }
                FixAction::AddLocalBinToPath => {
                    if let Some(home) = dirs::home_dir() {
                        let shell = std::env::var("SHELL").unwrap_or_default();
                        let profile = if shell.contains("zsh") {
                            home.join(".zshrc")
                        } else {
                            home.join(".bashrc")
                        };
                        let line = "\n# Added by great doctor --fix\nexport PATH=\"$HOME/.local/bin:$PATH\"\n";
                        match std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&profile)
                        {
                            Ok(mut f) => {
                                use std::io::Write;
                                if f.write_all(line.as_bytes()).is_ok() {
                                    output::success(&format!(
                                        "  Added ~/.local/bin to PATH in {}",
                                        profile.display()
                                    ));
                                    fixed += 1;
                                }
                            }
                            Err(e) => output::error(&format!("  Failed: {}", e)),
                        }
                    }
                }
                FixAction::InstallSystemPrerequisite { name } => {
                    match name.as_str() {
                        "curl" => bootstrap::ensure_curl(false, &info.platform),
                        "git" => bootstrap::ensure_git(false, &info.platform),
                        "build-essential" => {
                            bootstrap::ensure_build_essential(false, &info.platform)
                        }
                        "unzip" => bootstrap::ensure_unzip(false, &info.platform),
                        _ => output::error(&format!("  Unknown prerequisite: {}", name)),
                    }
                    fixed += 1;
                }
                FixAction::InstallDocker => {
                    bootstrap::ensure_docker(false, &info);
                    fixed += 1;
                }
                FixAction::InstallClaudeCode => {
                    bootstrap::ensure_claude_code(false);
                    fixed += 1;
                }
                FixAction::FixInotifyWatches => {
                    tuning::apply_system_tuning(false, &info);
                    fixed += 1;
                }
            }
        }

        println!();
        output::info(&format!(
            "Fixed {} of {} issues.",
            fixed,
            result.fixable.len()
        ));
        output::info("Re-run `great doctor` to verify fixes.");
    }

    // Summary
    println!();
    output::header("Summary");
    output::info(&format!(
        "  {} passed, {} warnings, {} errors",
        result.checks_passed, result.checks_warned, result.checks_failed
    ));

    if result.checks_failed > 0 && !args.fix {
        println!();
        output::warning("Run `great doctor --fix` to attempt automatic fixes.");
    } else if result.checks_failed > 0 {
        println!();
        output::warning("Some issues remain — re-run `great doctor` to verify.");
    } else if result.checks_warned > 0 {
        println!();
        output::success("No critical issues found.");
    } else {
        println!();
        output::success("Environment is healthy!");
    }

    // NOTE: Intentional use of process::exit — the doctor command must print
    // its full report before exiting non-zero. Using bail!() would abort
    // mid-report, which is wrong for a diagnostic command.
    if result.checks_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn pass(result: &mut DiagnosticResult, msg: &str) {
    result.checks_passed += 1;
    output::success(msg);
}

fn warn(result: &mut DiagnosticResult, msg: &str) {
    result.checks_warned += 1;
    output::warning(msg);
}

fn fail(result: &mut DiagnosticResult, msg: &str) {
    result.checks_failed += 1;
    output::error(msg);
}

fn check_platform(result: &mut DiagnosticResult) {
    output::header("Platform");
    let info = platform::detect_platform_info();

    pass(
        result,
        &format!("Platform detected: {}", info.platform.display_detailed()),
    );

    // Check architecture
    match info.platform.arch() {
        platform::Architecture::X86_64 | platform::Architecture::Aarch64 => {
            pass(result, &format!("Architecture: {}", info.platform.arch()));
        }
        _ => {
            warn(
                result,
                "Unknown architecture — some tools may not be available",
            );
        }
    }

    // Check if running as root (not recommended)
    if info.is_root {
        warn(result, "Running as root — not recommended for development");
    } else {
        pass(result, "Running as regular user");
    }

    // Package manager availability — Homebrew is the primary package manager
    // on macOS, Ubuntu, and WSL Ubuntu. Apt is a fallback for system packages only.
    let caps = &info.capabilities;
    match &info.platform {
        Platform::MacOS { .. } => {
            if caps.has_homebrew {
                pass(result, "Homebrew: installed (primary package manager)");
            } else {
                fail(
                    result,
                    "Homebrew: not installed — run: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"",
                );
                result.fixable.push(FixableIssue {
                    description: "Install Homebrew".to_string(),
                    action: FixAction::InstallHomebrew,
                });
            }
        }
        Platform::Linux { distro, .. } | Platform::Wsl { distro, .. } => {
            let is_ubuntu_debian = matches!(
                distro,
                platform::LinuxDistro::Ubuntu | platform::LinuxDistro::Debian
            );
            if caps.has_homebrew {
                pass(
                    result,
                    "Homebrew (Linuxbrew): installed (primary package manager)",
                );
            } else if is_ubuntu_debian {
                fail(
                    result,
                    "Homebrew (Linuxbrew): not installed — required as primary package manager.",
                );
                result.fixable.push(FixableIssue {
                    description: "Install Homebrew (Linuxbrew)".to_string(),
                    action: FixAction::InstallHomebrew,
                });
            } else if caps.has_dnf {
                pass(result, "dnf: available");
            } else {
                warn(
                    result,
                    "No supported package manager found (homebrew recommended)",
                );
            }

            // Apt reported as available fallback, not as primary
            if caps.has_apt {
                pass(result, "apt: available (fallback for system packages)");
            }
        }
        _ => {}
    }
    println!();
}

fn check_essential_tools(result: &mut DiagnosticResult) {
    output::header("Essential Tools");

    // (binary, display name, brew formula, install hint)
    let essential: &[(&str, &str, &str, &str)] = &[
        (
            "git",
            "Git version control",
            "git",
            "https://git-scm.com/downloads",
        ),
        ("curl", "curl HTTP client", "curl", "brew install curl"),
        ("gh", "GitHub CLI", "gh", "brew install gh"),
        (
            "node",
            "Node.js runtime",
            "node",
            "https://nodejs.org or use mise",
        ),
        ("npm", "npm package manager", "npm", "Comes with Node.js"),
        ("pnpm", "pnpm package manager", "pnpm", "brew install pnpm"),
        ("cargo", "Rust toolchain", "rustup", "https://rustup.rs"),
    ];

    for (cmd, name, brew_name, install_hint) in essential {
        if command_exists(cmd) {
            let version = util::get_command_version(cmd);
            let ver_str = version.as_deref().unwrap_or("version unknown");
            pass(result, &format!("{}: {} ({})", name, cmd, ver_str));
        } else {
            fail(
                result,
                &format!("{}: not found — install: {}", name, install_hint),
            );
            result.fixable.push(FixableIssue {
                description: format!("Install {}", name),
                action: FixAction::InstallTool {
                    binary: cmd.to_string(),
                    brew_name: brew_name.to_string(),
                },
            });
        }
    }

    // Check for mise (runtime version manager)
    if command_exists("mise") {
        pass(result, "mise: installed (runtime version manager)");
    } else {
        warn(
            result,
            "mise: not installed — recommended for managing tool versions. Install: https://mise.jdx.dev",
        );
    }

    // Recommended extras
    let recommended = [
        ("bat", "bat (cat with syntax highlighting)", "bat"),
        ("uv", "uv (fast Python package manager)", "uv"),
        ("deno", "Deno runtime", "deno"),
        ("starship", "Starship prompt", "starship"),
    ];
    for (cmd, name, brew_name) in &recommended {
        if command_exists(cmd) {
            pass(result, &format!("{}: installed", name));
        } else {
            warn(result, &format!("{}: not found (optional)", name));
            result.fixable.push(FixableIssue {
                description: format!("Install {}", name),
                action: FixAction::InstallTool {
                    binary: cmd.to_string(),
                    brew_name: brew_name.to_string(),
                },
            });
        }
    }

    println!();
}

fn check_ai_agents(result: &mut DiagnosticResult) {
    output::header("AI Agents");

    // Check Claude Code
    if command_exists("claude") {
        pass(result, "Claude Code: installed");
    } else {
        fail(
            result,
            "Claude Code: not found — install: curl -fsSL https://claude.ai/install.sh | bash",
        );
        result.fixable.push(FixableIssue {
            description: "Install Claude Code".to_string(),
            action: FixAction::InstallClaudeCode,
        });
    }

    // Check OpenAI Codex (optional)
    if command_exists("codex") {
        pass(result, "OpenAI Codex CLI: installed");
    } else {
        warn(result, "OpenAI Codex CLI: not found (optional)");
    }

    // Check common API keys
    let api_keys = [
        ("ANTHROPIC_API_KEY", "Anthropic API key"),
        ("OPENAI_API_KEY", "OpenAI API key"),
    ];

    for (key, name) in &api_keys {
        if std::env::var(key).is_ok() {
            pass(result, &format!("{}: set", name));
        } else {
            warn(result, &format!("{}: not set ({})", name, key));
        }
    }

    println!();
}

fn check_config(result: &mut DiagnosticResult) -> Option<config::GreatConfig> {
    output::header("Configuration");

    let loaded_config = match config::discover_config() {
        Ok(path) => {
            pass(result, &format!("great.toml: found at {}", path.display()));
            let path_str = match path.to_str() {
                Some(s) => s,
                None => {
                    fail(
                        result,
                        &format!(
                            "great.toml: path contains non-UTF-8 characters: {}",
                            path.display()
                        ),
                    );
                    println!();
                    return None;
                }
            };
            match config::load(Some(path_str)) {
                Ok(cfg) => {
                    pass(result, "great.toml: valid syntax");
                    // Note: config::load() already validates and bails on errors,
                    // so the Error arm below is defensive only (warnings can appear).
                    let messages = cfg.validate();
                    for msg in &messages {
                        match msg {
                            config::schema::ConfigMessage::Warning(w) => {
                                warn(result, &format!("Config: {}", w));
                            }
                            config::schema::ConfigMessage::Error(e) => {
                                fail(result, &format!("Config: {}", e));
                            }
                        }
                    }
                    // Check secret references
                    let refs = cfg.find_secret_refs();
                    for secret_ref in &refs {
                        if std::env::var(secret_ref).is_ok() {
                            pass(result, &format!("Secret ${{{}}}: resolved", secret_ref));
                        } else {
                            fail(
                                result,
                                &format!("Secret ${{{}}}: not set in environment", secret_ref),
                            );
                        }
                    }
                    Some(cfg)
                }
                Err(e) => {
                    fail(result, &format!("great.toml: parse error — {}", e));
                    None
                }
            }
        }
        Err(_) => {
            warn(
                result,
                "great.toml: not found — run `great init` to create one",
            );
            None
        }
    };

    // Check Claude config directories
    let home = dirs::home_dir();
    if let Some(home) = &home {
        let claude_dir = home.join(".claude");
        if claude_dir.exists() {
            pass(result, "~/.claude/ directory: exists");
        } else {
            warn(result, "~/.claude/ directory: not found");
            result.fixable.push(FixableIssue {
                description: "Create ~/.claude/ directory".to_string(),
                action: FixAction::CreateClaudeDir,
            });
        }
    }

    println!();
    loaded_config
}

/// Check that MCP server commands declared in great.toml are available on PATH.
fn check_mcp_servers(result: &mut DiagnosticResult, cfg: &config::GreatConfig) {
    let mcps = match &cfg.mcp {
        Some(m) if !m.is_empty() => m,
        _ => return,
    };

    output::header("MCP Servers");

    for (name, mcp) in mcps {
        // Skip disabled servers
        if mcp.enabled == Some(false) {
            pass(result, &format!("{}: disabled (skipped)", name));
            continue;
        }

        if command_exists(&mcp.command) {
            let transport = mcp.transport.as_deref().unwrap_or("stdio");
            pass(
                result,
                &format!("{}: {} found [{}]", name, mcp.command, transport),
            );
        } else {
            fail(
                result,
                &format!("{}: command '{}' not found on PATH", name, mcp.command),
            );
        }
    }

    println!();
}

fn check_shell(result: &mut DiagnosticResult) {
    output::header("Shell");

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    pass(result, &format!("Shell: {}", shell));

    // Check if ~/.local/bin is in PATH
    if let Ok(path) = std::env::var("PATH") {
        if path.contains(".local/bin") {
            pass(result, "~/.local/bin in PATH");
        } else {
            warn(
                result,
                "~/.local/bin not in PATH — some tools may not be found",
            );
            result.fixable.push(FixableIssue {
                description: "Add ~/.local/bin to PATH".to_string(),
                action: FixAction::AddLocalBinToPath,
            });
        }
    }

    println!();
}

fn check_system_prerequisites(result: &mut DiagnosticResult, info: &PlatformInfo) {
    output::header("System Prerequisites");

    // curl
    if command_exists("curl") {
        pass(result, "curl: installed");
    } else {
        fail(result, "curl: not found");
        result.fixable.push(FixableIssue {
            description: "Install curl".to_string(),
            action: FixAction::InstallSystemPrerequisite {
                name: "curl".to_string(),
            },
        });
    }

    // git
    if command_exists("git") {
        pass(result, "git: installed");
    } else {
        fail(result, "git: not found");
        result.fixable.push(FixableIssue {
            description: "Install git".to_string(),
            action: FixAction::InstallSystemPrerequisite {
                name: "git".to_string(),
            },
        });
    }

    // build-essential / Xcode CLI tools
    if matches!(info.platform, Platform::MacOS { .. }) {
        let has_xcode = std::process::Command::new("xcode-select")
            .arg("-p")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if has_xcode {
            pass(result, "Xcode CLI tools: installed");
        } else {
            fail(
                result,
                "Xcode CLI tools: not found — run: xcode-select --install",
            );
            result.fixable.push(FixableIssue {
                description: "Install Xcode CLI tools".to_string(),
                action: FixAction::InstallSystemPrerequisite {
                    name: "build-essential".to_string(),
                },
            });
        }
    } else if bootstrap::is_apt_distro(&info.platform) {
        let has_be = std::process::Command::new("dpkg")
            .args(["-s", "build-essential"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if has_be {
            pass(result, "build-essential: installed");
        } else {
            fail(
                result,
                "build-essential: not found — run: sudo apt-get install -y build-essential",
            );
            result.fixable.push(FixableIssue {
                description: "Install build-essential".to_string(),
                action: FixAction::InstallSystemPrerequisite {
                    name: "build-essential".to_string(),
                },
            });
        }
    }

    // unzip
    if command_exists("unzip") {
        pass(result, "unzip: installed");
    } else {
        fail(result, "unzip: not found");
        result.fixable.push(FixableIssue {
            description: "Install unzip".to_string(),
            action: FixAction::InstallSystemPrerequisite {
                name: "unzip".to_string(),
            },
        });
    }

    println!();
}

fn check_docker(result: &mut DiagnosticResult, info: &PlatformInfo) {
    output::header("Docker");

    if command_exists("docker") {
        let daemon_ok = std::process::Command::new("docker")
            .arg("info")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if daemon_ok {
            pass(result, "Docker: installed and daemon running");
        } else {
            warn(result, "Docker: installed but daemon is not running");
        }
    } else {
        let msg = match &info.platform {
            Platform::MacOS { .. } => {
                "Docker: not found — install Docker Desktop (https://www.docker.com/products/docker-desktop/) or OrbStack (https://orbstack.dev/)"
            }
            Platform::Wsl { .. } => {
                "Docker: not found — install Docker Desktop for Windows with WSL2 backend (https://docs.docker.com/desktop/wsl/)"
            }
            _ => {
                "Docker: not found"
            }
        };

        fail(result, msg);

        // Only offer auto-fix on apt-based Linux (not WSL or macOS — those need GUI apps)
        if bootstrap::is_apt_distro(&info.platform)
            && !matches!(info.platform, Platform::Wsl { .. })
        {
            result.fixable.push(FixableIssue {
                description: "Install Docker CE".to_string(),
                action: FixAction::InstallDocker,
            });
        }
    }

    println!();
}

fn check_system_tuning(result: &mut DiagnosticResult, info: &PlatformInfo) {
    if !bootstrap::is_linux_like(&info.platform) {
        return;
    }

    output::header("System Tuning");

    let (current, sufficient) = tuning::check_inotify_watches();
    if let Some(current) = current {
        if sufficient {
            pass(
                result,
                &format!("inotify max_user_watches: {} (sufficient)", current),
            );
        } else {
            fail(
                result,
                &format!(
                    "inotify max_user_watches: {} (below 524288 recommended)",
                    current
                ),
            );
            result.fixable.push(FixableIssue {
                description: "Increase inotify max_user_watches".to_string(),
                action: FixAction::FixInotifyWatches,
            });
        }
    }

    println!();
}
