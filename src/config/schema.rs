use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level great.toml configuration.
///
/// Represents the full schema for a project's `great.toml` file, including
/// project metadata, tool versions, AI agent definitions, MCP server configs,
/// secret management, and platform-specific overrides.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreatConfig {
    /// Project metadata (name, description).
    pub project: Option<ProjectConfig>,
    /// Tool versions and CLI tool declarations.
    pub tools: Option<ToolsConfig>,
    /// Named AI agent configurations.
    pub agents: Option<HashMap<String, AgentConfig>>,
    /// Named MCP (Model Context Protocol) server configurations.
    pub mcp: Option<HashMap<String, McpConfig>>,
    /// Secret/credential management configuration.
    pub secrets: Option<SecretsConfig>,
    /// Platform-specific overrides (macOS, Linux, WSL2).
    pub platform: Option<PlatformConfig>,
}

/// Project metadata section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// The project name.
    pub name: Option<String>,
    /// A brief description of the project.
    pub description: Option<String>,
}

/// Tools section — top-level keys are runtime versions, `[tools.cli]` holds CLI tools.
///
/// Templates can declare their own `[tools.cli]` entries (e.g. `hasura-cli`,
/// `aws`, `cdk`) which get merged into the user's config when applied via
/// `great template apply`. This lets domain-specific templates pull in the
/// CLI tools they need.
///
/// Example:
/// ```toml
/// [tools]
/// node = "22"
/// python = "3.12"
/// deno = "latest"
///
/// [tools.cli]
/// ripgrep = "latest"
/// gh = "latest"
/// bat = "latest"
/// pnpm = "latest"
/// uv = "latest"
/// starship = "latest"
/// # Cloud CLIs
/// aws = "latest"
/// cdk = "latest"          # installed via npm (npm i -g aws-cdk)
/// az = "latest"           # installed as brew install azure-cli
/// gcloud = "latest"       # installed as brew install google-cloud-sdk
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    /// Runtime tools with their version strings (e.g., `node = "22"`).
    /// These are collected via `#[serde(flatten)]` from any top-level key
    /// in the `[tools]` table that is not `cli`.
    #[serde(flatten)]
    pub runtimes: HashMap<String, String>,
    /// CLI tools under `[tools.cli]`, each with a version string.
    pub cli: Option<HashMap<String, String>>,
}

/// Configuration for a named AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// The provider for this agent (e.g., "anthropic", "openai").
    pub provider: Option<String>,
    /// The model identifier (e.g., "claude-sonnet-4-20250514").
    pub model: Option<String>,
}

/// Configuration for a named MCP (Model Context Protocol) server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// The command to run the MCP server.
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Option<Vec<String>>,
    /// Environment variables for the server process.
    /// Values may contain `${SECRET_NAME}` references.
    pub env: Option<HashMap<String, String>>,
    /// Transport type: `"stdio"` (default) or `"http"`.
    pub transport: Option<String>,
    /// URL for HTTP transport.
    pub url: Option<String>,
}

/// Secret and credential management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// The secret provider: `"env"`, `"1password"`, `"bitwarden"`, `"keychain"`.
    pub provider: Option<String>,
    /// Secret keys that must be present for the project to function.
    pub required: Option<Vec<String>>,
}

/// Platform-specific override container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfig {
    /// Overrides applied on macOS.
    pub macos: Option<PlatformOverride>,
    /// Overrides applied on WSL2.
    pub wsl2: Option<PlatformOverride>,
    /// Overrides applied on native Linux.
    pub linux: Option<PlatformOverride>,
}

/// Platform-specific overrides that augment the base configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverride {
    /// Additional tools to install on this platform.
    pub extra_tools: Option<Vec<String>>,
}

/// A validation message produced by [`GreatConfig::validate`].
#[derive(Debug, Clone)]
pub enum ConfigMessage {
    /// A non-fatal issue that should be reported to the user.
    Warning(String),
    /// A fatal issue that prevents the config from being used.
    Error(String),
}

impl GreatConfig {
    /// Validate the configuration, returning a list of warnings and errors.
    ///
    /// Checks include:
    /// - Agents should have at least a provider or model specified.
    /// - Secret names in `secrets.required` must be valid environment variable names
    ///   (ASCII alphanumeric and underscores only).
    pub fn validate(&self) -> Vec<ConfigMessage> {
        let mut messages = Vec::new();

        // Check: if agents are declared, at least one should have a provider
        if let Some(agents) = &self.agents {
            for (name, agent) in agents {
                if agent.provider.is_none() && agent.model.is_none() {
                    messages.push(ConfigMessage::Warning(format!(
                        "agent '{}' has no provider or model specified",
                        name
                    )));
                }
            }
        }

        // Check: if secrets.required is set, validate they look like env var names
        if let Some(secrets) = &self.secrets {
            if let Some(required) = &secrets.required {
                for key in required {
                    if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        messages.push(ConfigMessage::Error(format!(
                            "invalid secret name '{}': must be alphanumeric with underscores",
                            key
                        )));
                    }
                }
            }
        }

        messages
    }

    /// Find all `${SECRET_NAME}` references in string values throughout the config.
    ///
    /// Scans MCP server environment variables for patterns like `${POSTGRES_URL}`.
    /// Returns a sorted, deduplicated list of referenced secret names.
    pub fn find_secret_refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").expect("valid regex");

        // Scan MCP env values for secret references
        if let Some(mcps) = &self.mcp {
            for mcp in mcps.values() {
                if let Some(env) = &mcp.env {
                    for value in env.values() {
                        for cap in re.captures_iter(value) {
                            refs.push(cap[1].to_string());
                        }
                    }
                }
            }
        }

        refs.sort();
        refs.dedup();
        refs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[project]
name = "test"
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.unwrap().name.unwrap(), "test");
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[project]
name = "my-project"
description = "Test project"

[tools]
node = "22"
python = "3.12"

[tools.cli]
ripgrep = "latest"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[mcp.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]

[platform.macos]
extra_tools = ["coreutils"]
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        assert!(config.project.is_some());
        assert!(config.tools.is_some());
        let tools = config.tools.unwrap();
        assert_eq!(tools.runtimes.get("node").unwrap(), "22");
        assert_eq!(tools.runtimes.get("python").unwrap(), "3.12");
        assert_eq!(tools.cli.unwrap().get("ripgrep").unwrap(), "latest");
        assert!(config.agents.unwrap().contains_key("claude"));
        assert!(config.mcp.unwrap().contains_key("filesystem"));
    }

    #[test]
    fn test_parse_empty_config() {
        let config: GreatConfig = toml::from_str("").unwrap();
        assert!(config.project.is_none());
        assert!(config.tools.is_none());
    }

    #[test]
    fn test_find_secret_refs() {
        let toml_str = r#"
[mcp.postgres]
command = "npx"
env = { DATABASE_URL = "${POSTGRES_URL}", API_KEY = "${MY_API_KEY}" }

[mcp.other]
command = "test"
env = { PLAIN = "no-secrets-here" }
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let refs = config.find_secret_refs();
        assert_eq!(refs, vec!["MY_API_KEY", "POSTGRES_URL"]);
    }

    #[test]
    fn test_validate_warns_on_empty_agent() {
        let toml_str = r#"
[agents.empty]
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let messages = config.validate();
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_validate_invalid_secret_name() {
        let toml_str = r#"
[secrets]
provider = "env"
required = ["VALID_KEY", "invalid-key-with-dashes"]
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let messages = config.validate();
        let has_error = messages
            .iter()
            .any(|m| matches!(m, ConfigMessage::Error(_)));
        assert!(has_error, "expected an error for invalid secret name");
    }

    #[test]
    fn test_validate_valid_config_no_messages() {
        let toml_str = r#"
[project]
name = "valid"

[agents.claude]
provider = "anthropic"
model = "claude-sonnet-4-20250514"

[secrets]
provider = "env"
required = ["ANTHROPIC_API_KEY"]
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let messages = config.validate();
        assert!(
            messages.is_empty(),
            "expected no warnings or errors for valid config"
        );
    }

    #[test]
    fn test_roundtrip_serialize() {
        let toml_str = r#"
[project]
name = "test"

[tools]
node = "22"

[agents.claude]
provider = "anthropic"
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let serialized = toml::to_string(&config).unwrap();
        let config2: GreatConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(
            config.project.as_ref().unwrap().name,
            config2.project.as_ref().unwrap().name
        );
    }

    #[test]
    fn test_tools_cli_only() {
        let toml_str = r#"
[tools.cli]
ripgrep = "latest"
fd-find = "latest"
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let tools = config.tools.unwrap();
        assert!(tools.runtimes.is_empty());
        let cli = tools.cli.unwrap();
        assert_eq!(cli.get("ripgrep").unwrap(), "latest");
        assert_eq!(cli.get("fd-find").unwrap(), "latest");
    }

    #[test]
    fn test_mcp_with_transport() {
        let toml_str = r#"
[mcp.remote]
command = "great-mcp"
transport = "http"
url = "http://localhost:8080"
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let mcps = config.mcp.unwrap();
        let remote = mcps.get("remote").unwrap();
        assert_eq!(remote.transport.as_deref(), Some("http"));
        assert_eq!(remote.url.as_deref(), Some("http://localhost:8080"));
    }

    #[test]
    fn test_platform_overrides() {
        let toml_str = r#"
[platform.macos]
extra_tools = ["coreutils", "gnu-sed"]

[platform.wsl2]
extra_tools = ["wslu"]

[platform.linux]
extra_tools = ["build-essential"]
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let platform = config.platform.unwrap();
        assert_eq!(
            platform.macos.unwrap().extra_tools.unwrap(),
            vec!["coreutils", "gnu-sed"]
        );
        assert_eq!(platform.wsl2.unwrap().extra_tools.unwrap(), vec!["wslu"]);
        assert_eq!(
            platform.linux.unwrap().extra_tools.unwrap(),
            vec!["build-essential"]
        );
    }

    #[test]
    fn test_find_secret_refs_no_mcps() {
        let config = GreatConfig::default();
        let refs = config.find_secret_refs();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_find_secret_refs_deduplicates() {
        let toml_str = r#"
[mcp.a]
command = "cmd"
env = { X = "${SHARED}", Y = "${SHARED}" }

[mcp.b]
command = "cmd"
env = { Z = "${SHARED}" }
"#;
        let config: GreatConfig = toml::from_str(toml_str).unwrap();
        let refs = config.find_secret_refs();
        assert_eq!(refs, vec!["SHARED"]);
    }
}
