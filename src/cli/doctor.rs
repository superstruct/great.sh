use anyhow::Result;
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::config;
use crate::platform::{self, command_exists, Platform};

/// Arguments for the `great doctor` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Attempt to fix issues automatically
    #[arg(long)]
    pub fix: bool,
}

struct DiagnosticResult {
    checks_passed: usize,
    checks_warned: usize,
    checks_failed: usize,
}

/// Run the `great doctor` diagnostic command.
///
/// Checks platform requirements, essential tools, AI agent availability,
/// configuration validity, and shell environment. Prints a summary of
/// passed, warned, and failed checks.
pub fn run(args: Args) -> Result<()> {
    if args.fix {
        output::info("Auto-fix mode enabled (not yet implemented — reporting only).");
        println!();
    }

    output::header("great doctor");
    println!();

    let mut result = DiagnosticResult {
        checks_passed: 0,
        checks_warned: 0,
        checks_failed: 0,
    };

    // 1. Platform check
    check_platform(&mut result);

    // 2. Essential tools check
    check_essential_tools(&mut result);

    // 3. AI agents check
    check_ai_agents(&mut result);

    // 4. Config check
    check_config(&mut result);

    // 5. Shell check
    check_shell(&mut result);

    // Summary
    println!();
    output::header("Summary");
    output::info(&format!(
        "  {} passed, {} warnings, {} errors",
        result.checks_passed, result.checks_warned, result.checks_failed
    ));

    if result.checks_failed > 0 {
        println!();
        output::warning("Run `great doctor --fix` to attempt automatic fixes.");
    } else if result.checks_warned > 0 {
        println!();
        output::success("No critical issues found.");
    } else {
        println!();
        output::success("Environment is healthy!");
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
            pass(result, &format!("Architecture: {:?}", info.platform.arch()));
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
            }
        }
        Platform::Linux { distro, .. } | Platform::Wsl { distro, .. } => {
            let is_ubuntu_debian = matches!(
                distro,
                platform::LinuxDistro::Ubuntu | platform::LinuxDistro::Debian
            );
            if caps.has_homebrew {
                pass(result, "Homebrew (Linuxbrew): installed (primary package manager)");
            } else if is_ubuntu_debian {
                fail(
                    result,
                    "Homebrew (Linuxbrew): not installed — required as primary package manager. Run: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"",
                );
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

    let essential = [
        (
            "git",
            "Git version control",
            "https://git-scm.com/downloads",
        ),
        (
            "curl",
            "curl HTTP client",
            "apt install curl / brew install curl",
        ),
        (
            "gh",
            "GitHub CLI",
            "https://cli.github.com or brew install gh",
        ),
        (
            "node",
            "Node.js runtime",
            "https://nodejs.org or use mise",
        ),
        ("npm", "npm package manager", "Comes with Node.js"),
        (
            "pnpm",
            "pnpm package manager",
            "npm install -g pnpm or brew install pnpm",
        ),
        ("cargo", "Rust toolchain", "https://rustup.rs"),
    ];

    for (cmd, name, install_hint) in &essential {
        if command_exists(cmd) {
            let version = get_command_version(cmd);
            let ver_str = version.as_deref().unwrap_or("version unknown");
            pass(result, &format!("{}: {} ({})", name, cmd, ver_str));
        } else {
            fail(
                result,
                &format!("{}: not found — install: {}", name, install_hint),
            );
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
        ("bat", "bat (cat with syntax highlighting)", "brew install bat"),
        ("uv", "uv (fast Python package manager)", "brew install uv / pip install uv"),
        ("deno", "Deno runtime", "brew install deno / mise install deno"),
        ("starship", "Starship prompt", "brew install starship"),
    ];
    for (cmd, name, install_hint) in &recommended {
        if command_exists(cmd) {
            pass(result, &format!("{}: installed", name));
        } else {
            warn(result, &format!("{}: not found (optional) — {}", name, install_hint));
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
            "Claude Code: not found — install: npm install -g @anthropic-ai/claude-code",
        );
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

fn check_config(result: &mut DiagnosticResult) {
    output::header("Configuration");

    match config::discover_config() {
        Ok(path) => {
            pass(
                result,
                &format!("great.toml: found at {}", path.display()),
            );
            match config::load(Some(path.to_str().unwrap_or_default())) {
                Ok(cfg) => {
                    pass(result, "great.toml: valid syntax");
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
                }
                Err(e) => {
                    fail(result, &format!("great.toml: parse error — {}", e));
                }
            }
        }
        Err(_) => {
            warn(
                result,
                "great.toml: not found — run `great init` to create one",
            );
        }
    }

    // Check Claude config directories
    let home = dirs::home_dir();
    if let Some(home) = &home {
        let claude_dir = home.join(".claude");
        if claude_dir.exists() {
            pass(result, "~/.claude/ directory: exists");
        } else {
            warn(result, "~/.claude/ directory: not found");
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
        }
    }

    println!();
}

/// Try to get a command's version string.
///
/// Runs `<cmd> --version` and returns the first line of stdout, or `None`
/// if the command fails or produces no output.
fn get_command_version(cmd: &str) -> Option<String> {
    let output = std::process::Command::new(cmd)
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
