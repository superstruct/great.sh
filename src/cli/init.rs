use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::config::schema::*;
use crate::platform;

/// Arguments for the `great init` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    /// Template to initialize from (ai-fullstack-ts, ai-fullstack-py, ai-minimal, saas-multi-tenant)
    #[arg(long)]
    pub template: Option<String>,

    /// Overwrite existing configuration
    #[arg(long)]
    pub force: bool,
}

/// Run the interactive first-run wizard to generate a `great.toml` file.
///
/// When `--template` is provided, the wizard is skipped and a pre-built
/// template is written directly. Otherwise the user is guided through
/// project, tools, agents, MCP servers, and secrets configuration via
/// interactive stdin prompts (sent to stderr so stdout stays clean for
/// piping).
pub fn run(args: Args) -> Result<()> {
    output::header("great init");
    eprintln!();

    let config_path = Path::new("great.toml");

    // Check for existing config
    if config_path.exists() && !args.force {
        output::error("great.toml already exists. Use --force to overwrite.");
        return Ok(());
    }

    // If template specified, use that
    if let Some(template) = &args.template {
        return init_from_template(template, config_path);
    }

    // Detect platform
    let info = platform::detect_platform_info();
    output::info(&format!(
        "Detected platform: {}",
        info.platform.display_detailed()
    ));
    eprintln!();

    // Build config interactively
    let mut config = GreatConfig::default();

    // Project section
    let project_name = prompt("Project name", &detect_project_name())?;
    config.project = Some(ProjectConfig {
        name: Some(project_name),
        ..Default::default()
    });

    // Tools section
    output::header("Tools");
    eprintln!();

    let mut runtimes = HashMap::new();
    let mut cli_tools = HashMap::new();

    if prompt_yes_no("Install Node.js?", true)? {
        let version = prompt("Node.js version", "22")?;
        runtimes.insert("node".to_string(), version);
    }

    if prompt_yes_no("Install Python?", true)? {
        let version = prompt("Python version", "3.12")?;
        runtimes.insert("python".to_string(), version);
    }

    if prompt_yes_no("Install Rust?", false)? {
        runtimes.insert("rust".to_string(), "stable".to_string());
    }

    if prompt_yes_no("Install Deno?", false)? {
        runtimes.insert("deno".to_string(), "latest".to_string());
    }

    // Common CLI tools
    if prompt_yes_no("Install common CLI tools (ripgrep, fd, bat, jq)?", true)? {
        cli_tools.insert("rg".to_string(), "latest".to_string());
        cli_tools.insert("fd".to_string(), "latest".to_string());
        cli_tools.insert("bat".to_string(), "latest".to_string());
        cli_tools.insert("jq".to_string(), "latest".to_string());
    }

    if prompt_yes_no("Install GitHub CLI (gh)?", true)? {
        cli_tools.insert("gh".to_string(), "latest".to_string());
    }

    // Package managers
    if runtimes.contains_key("node")
        && prompt_yes_no("Install pnpm (fast Node.js package manager)?", true)?
    {
        cli_tools.insert("pnpm".to_string(), "latest".to_string());
    }
    if runtimes.contains_key("python")
        && prompt_yes_no("Install uv (fast Python package manager)?", true)?
    {
        cli_tools.insert("uv".to_string(), "latest".to_string());
    }

    // Shell prompt
    if prompt_yes_no("Install Starship prompt?", false)? {
        cli_tools.insert("starship".to_string(), "latest".to_string());
        // Starship shell config is handled by `great apply` after install
        output::info("  Nerd Font will also be installed (required for Starship glyphs)");
    }

    // Cloud CLIs
    eprintln!();
    output::header("Cloud CLIs");
    eprintln!();

    if prompt_yes_no("Install AWS CLI + CDK?", false)? {
        cli_tools.insert("aws".to_string(), "latest".to_string());
        // CDK install is handled via tool_install_spec in apply (npm install -g aws-cdk)
        cli_tools.insert("cdk".to_string(), "latest".to_string());
    }
    if prompt_yes_no("Install Azure CLI?", false)? {
        cli_tools.insert("az".to_string(), "latest".to_string());
    }
    if prompt_yes_no("Install Google Cloud CLI?", false)? {
        cli_tools.insert("gcloud".to_string(), "latest".to_string());
    }

    if !runtimes.is_empty() || !cli_tools.is_empty() {
        config.tools = Some(ToolsConfig {
            runtimes,
            cli: if cli_tools.is_empty() {
                None
            } else {
                Some(cli_tools)
            },
        });
    }

    // Agents section
    eprintln!();
    output::header("AI Agents");
    eprintln!();

    let mut agents = HashMap::new();

    // Claude is the primary agent
    agents.insert(
        "claude".to_string(),
        AgentConfig {
            provider: Some("anthropic".to_string()),
            model: Some("claude-sonnet-4-20250514".to_string()),
            ..Default::default()
        },
    );
    output::success("Claude Code — included (primary agent)");

    if prompt_yes_no("Add OpenAI Codex?", false)? {
        agents.insert(
            "codex".to_string(),
            AgentConfig {
                provider: Some("openai".to_string()),
                model: Some("codex-mini".to_string()),
                ..Default::default()
            },
        );
    }

    if prompt_yes_no("Add Google Gemini?", false)? {
        agents.insert(
            "gemini".to_string(),
            AgentConfig {
                provider: Some("google".to_string()),
                model: Some("gemini-2.5-pro".to_string()),
                ..Default::default()
            },
        );
    }

    config.agents = Some(agents);

    // MCP Servers section
    eprintln!();
    output::header("MCP Servers");
    eprintln!();

    let mut mcps = HashMap::new();

    if prompt_yes_no("Add filesystem MCP server?", true)? {
        mcps.insert(
            "filesystem".to_string(),
            McpConfig {
                command: "npx".to_string(),
                args: Some(vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    ".".to_string(),
                ]),
                env: None,
                transport: None,
                url: None,
                enabled: None,
            },
        );
    }

    // NOTE: GitHub MCP server removed in favour of the `gh` CLI which is
    // faster, requires no token plumbing, and already installed above.

    if !mcps.is_empty() {
        config.mcp = Some(mcps);
    }

    // MCP Bridge section
    eprintln!();
    output::header("MCP Bridge");
    eprintln!();

    if prompt_yes_no(
        "Enable built-in MCP bridge (routes MCP servers to all AI agents)?",
        false,
    )? {
        // Preset heuristic is agent-count-based (wizard context).
        // Templates use complexity-based presets (fullstack projects get "agent"
        // even with a single agent). These semantics differ intentionally —
        // do not "fix" one to match the other.
        let preset = if config.agents.as_ref().map_or(0, |a| a.len()) > 1 {
            "agent"
        } else {
            "minimal"
        };
        config.mcp_bridge = Some(McpBridgeConfig {
            preset: Some(preset.to_string()),
            ..Default::default()
        });
        output::success(&format!("MCP bridge enabled with {} preset", preset));
        output::info("  Presets: minimal (1 tool) | agent (6 tools) | research (8 tools) | full (9 tools)");
    }

    // Secrets section
    eprintln!();
    output::header("Secrets");
    eprintln!();

    let mut required_secrets = vec!["ANTHROPIC_API_KEY".to_string()];

    if config
        .agents
        .as_ref()
        .is_some_and(|a| a.contains_key("codex"))
    {
        required_secrets.push("OPENAI_API_KEY".to_string());
    }
    if config
        .agents
        .as_ref()
        .is_some_and(|a| a.contains_key("gemini"))
    {
        required_secrets.push("GOOGLE_API_KEY".to_string());
    }

    config.secrets = Some(SecretsConfig {
        provider: Some("env".to_string()),
        required: Some(required_secrets),
    });

    // Platform overrides
    let platform_cfg = match &info.platform {
        platform::Platform::MacOS { .. } => Some(PlatformConfig {
            macos: Some(PlatformOverride {
                extra_tools: Some(vec!["coreutils".to_string()]),
            }),
            ..Default::default()
        }),
        platform::Platform::Wsl { .. } => Some(PlatformConfig {
            wsl2: Some(PlatformOverride {
                extra_tools: Some(vec!["wslu".to_string()]),
            }),
            ..Default::default()
        }),
        _ => None,
    };
    config.platform = platform_cfg;

    // Serialize and write
    let toml_string =
        toml::to_string_pretty(&config).context("failed to serialize configuration")?;

    std::fs::write(config_path, &toml_string).context("failed to write great.toml")?;

    eprintln!();
    output::success(&format!("Created {}", config_path.display()));
    output::info("Next steps:");
    output::info("  1. Review great.toml and customize as needed");
    output::info("  2. Set your API keys: export ANTHROPIC_API_KEY=<key>");
    output::info("  3. Run: great apply");

    Ok(())
}

/// Initialize from a named built-in template, writing it directly to `config_path`.
fn init_from_template(template: &str, config_path: &Path) -> Result<()> {
    let toml_content = match template {
        "ai-fullstack-ts" => include_str!("../../templates/ai-fullstack-ts.toml"),
        "ai-fullstack-py" => include_str!("../../templates/ai-fullstack-py.toml"),
        "ai-minimal" => include_str!("../../templates/ai-minimal.toml"),
        "saas-multi-tenant" => include_str!("../../templates/saas-multi-tenant.toml"),
        _ => {
            output::error(&format!(
                "Unknown template: {}. Available: ai-fullstack-ts, ai-fullstack-py, ai-minimal, saas-multi-tenant",
                template
            ));
            return Ok(());
        }
    };

    std::fs::write(config_path, toml_content).context("failed to write great.toml")?;

    output::success(&format!(
        "Created {} from template '{}'",
        config_path.display(),
        template
    ));
    output::info("Run `great apply` to provision your environment.");
    Ok(())
}

/// Prompt the user for input with a default value.
///
/// The prompt is written to stderr so that stdout remains available for
/// structured/data output. When stdin is empty (e.g., piped `/dev/null`),
/// the default value is returned.
fn prompt(question: &str, default: &str) -> Result<String> {
    eprint!("  {} [{}]: ", question, default);
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

/// Prompt the user for a yes/no answer with a default.
///
/// Accepts "y", "yes" (case-insensitive) as affirmative; anything else
/// starting with "n" as negative. Empty input returns `default_yes`.
fn prompt_yes_no(question: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    eprint!("  {} [{}]: ", question, hint);
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        Ok(default_yes)
    } else {
        Ok(input.starts_with('y'))
    }
}

/// Try to detect the project name from the current directory name.
fn detect_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_project_name_returns_string() {
        let name = detect_project_name();
        assert!(!name.is_empty(), "project name should not be empty");
    }

    #[test]
    fn test_init_from_template_unknown() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let config_path = dir.path().join("great.toml");
        // Unknown template should not create a file, and should return Ok
        let result = init_from_template("nonexistent-template", &config_path);
        assert!(result.is_ok());
        assert!(
            !config_path.exists(),
            "file should not be created for unknown template"
        );
    }

    #[test]
    fn test_init_from_template_minimal() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let config_path = dir.path().join("great.toml");
        let result = init_from_template("ai-minimal", &config_path);
        assert!(result.is_ok());
        assert!(config_path.exists(), "great.toml should be created");

        let content = std::fs::read_to_string(&config_path).expect("failed to read");
        assert!(content.contains("[project]"));
        assert!(content.contains("[agents.claude]"));
    }

    #[test]
    fn test_init_from_template_fullstack_ts() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let config_path = dir.path().join("great.toml");
        let result = init_from_template("ai-fullstack-ts", &config_path);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&config_path).expect("failed to read");
        assert!(content.contains("node"));
        assert!(content.contains("typescript"));
        assert!(content.contains("Full-stack TypeScript"));
    }

    #[test]
    fn test_init_from_template_fullstack_py() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let config_path = dir.path().join("great.toml");
        let result = init_from_template("ai-fullstack-py", &config_path);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(&config_path).expect("failed to read");
        assert!(content.contains("python"));
        assert!(content.contains("Full-stack Python"));
    }

    #[test]
    fn test_templates_parse_as_valid_config() {
        // All embedded templates must deserialize into a valid GreatConfig
        let templates = [
            (
                "ai-fullstack-ts",
                include_str!("../../templates/ai-fullstack-ts.toml"),
            ),
            (
                "ai-fullstack-py",
                include_str!("../../templates/ai-fullstack-py.toml"),
            ),
            (
                "ai-minimal",
                include_str!("../../templates/ai-minimal.toml"),
            ),
            (
                "saas-multi-tenant",
                include_str!("../../templates/saas-multi-tenant.toml"),
            ),
        ];
        for (name, content) in &templates {
            let config: GreatConfig = toml::from_str(content)
                .unwrap_or_else(|e| panic!("template '{}' failed to parse: {}", name, e));
            assert!(
                config.project.is_some(),
                "template '{}' should have a [project] section",
                name
            );
            assert!(
                config.agents.is_some(),
                "template '{}' should have agents",
                name
            );
        }
    }

    #[test]
    fn test_templates_have_mcp_bridge() {
        let templates: &[(&str, &str, &str)] = &[
            (
                "ai-minimal",
                include_str!("../../templates/ai-minimal.toml"),
                "minimal",
            ),
            (
                "ai-fullstack-ts",
                include_str!("../../templates/ai-fullstack-ts.toml"),
                "agent",
            ),
            (
                "ai-fullstack-py",
                include_str!("../../templates/ai-fullstack-py.toml"),
                "agent",
            ),
            (
                "saas-multi-tenant",
                include_str!("../../templates/saas-multi-tenant.toml"),
                "full",
            ),
        ];
        for (name, content, expected_preset) in templates {
            let config: GreatConfig = toml::from_str(content)
                .unwrap_or_else(|e| panic!("template '{}' failed to parse: {}", name, e));
            let bridge = config.mcp_bridge.unwrap_or_else(|| {
                panic!("template '{}' should have a [mcp-bridge] section", name)
            });
            assert_eq!(
                bridge.preset.as_deref(),
                Some(*expected_preset),
                "template '{}' should have preset '{}'",
                name,
                expected_preset
            );
        }
    }

    #[test]
    fn test_default_config_has_no_mcp_bridge() {
        let config = GreatConfig::default();
        assert!(
            config.mcp_bridge.is_none(),
            "default config should not have mcp_bridge (opt-in only)"
        );
    }

    #[test]
    fn test_default_config_serializes() {
        // Ensure a default GreatConfig can be serialized to TOML without error
        let config = GreatConfig::default();
        let result = toml::to_string_pretty(&config);
        assert!(result.is_ok(), "default config should serialize");
    }
}
