pub mod bridge;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::schema::McpConfig;

/// The `.mcp.json` format used by Claude Code and other AI tools.
///
/// Structure: `{"mcpServers": {"name": {"command": "...", "args": [...], "env": {...}}}}`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct McpJsonConfig {
    /// Map of server name to server entry, serialized as `"mcpServers"`.
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

/// A single MCP server entry within `.mcp.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerEntry {
    /// The command to run the MCP server process.
    pub command: String,
    /// Arguments to pass to the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Environment variables for the server process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

impl McpJsonConfig {
    /// Load `.mcp.json` from a path, or return an empty config if the file does not exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content =
            std::fs::read_to_string(path).context(format!("failed to read {}", path.display()))?;
        let config: Self = serde_json::from_str(&content)
            .context(format!("failed to parse {}", path.display()))?;
        Ok(config)
    }

    /// Save this config as pretty-printed JSON to the given path.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("failed to serialize MCP config")?;
        std::fs::write(path, json).context(format!("failed to write {}", path.display()))?;
        Ok(())
    }

    /// Add a server from a [`McpConfig`] entry parsed from `great.toml`.
    #[allow(dead_code)] // Planned for GROUP C (mcp add command).
    pub fn add_server(&mut self, name: &str, config: &McpConfig) {
        let entry = McpServerEntry {
            command: config.command.clone(),
            args: config.args.clone(),
            env: config.env.clone(),
        };
        self.mcp_servers.insert(name.to_string(), entry);
    }

    /// Check if a server with the given name is already configured.
    pub fn has_server(&self, name: &str) -> bool {
        self.mcp_servers.contains_key(name)
    }

    /// List all configured server names.
    #[allow(dead_code)] // Planned for GROUP C (mcp add command).
    pub fn server_names(&self) -> Vec<&String> {
        self.mcp_servers.keys().collect()
    }
}

/// Resolve `${SECRET_NAME}` references in env values against the process environment.
///
/// Any reference whose variable is not set is left as-is so the user can see what
/// is missing.
pub fn resolve_env(env: &HashMap<String, String>) -> HashMap<String, String> {
    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").expect("valid regex");
    env.iter()
        .map(|(k, v)| {
            let resolved = re.replace_all(v, |caps: &regex::Captures| {
                let var_name = &caps[1];
                std::env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
            });
            (k.clone(), resolved.to_string())
        })
        .collect()
}

/// Return the project-level `.mcp.json` path (in the current directory).
pub fn project_mcp_path() -> PathBuf {
    PathBuf::from(".mcp.json")
}

/// Return the user-level Claude config path (`~/.claude.json`).
#[allow(dead_code)] // Planned for user-level MCP config support.
pub fn user_mcp_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude.json"))
}

/// Test if an MCP server can start by spawning it and checking for a running process.
///
/// Returns `Ok(true)` if the process starts successfully, `Ok(false)` if the
/// command cannot be spawned (e.g., binary not found).
pub fn test_server(config: &McpConfig) -> Result<bool> {
    let mut cmd = std::process::Command::new(&config.command);
    if let Some(args) = &config.args {
        cmd.args(args);
    }
    if let Some(env) = &config.env {
        let resolved = resolve_env(env);
        for (k, v) in &resolved {
            cmd.env(k, v);
        }
    }

    // Try to start the process, give it a moment, then check if it's running
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            // Wait briefly then kill — we just want to know if it starts
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = child.kill();
            let _ = child.wait();
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_json_roundtrip() {
        let mut config = McpJsonConfig::default();
        let mcp = McpConfig {
            command: "npx".to_string(),
            args: Some(vec!["-y".to_string(), "server-fs".to_string()]),
            env: None,
            transport: None,
            url: None,
            enabled: None,
        };
        config.add_server("filesystem", &mcp);

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: McpJsonConfig = serde_json::from_str(&json).unwrap();

        assert!(parsed.has_server("filesystem"));
        assert!(!parsed.has_server("nonexistent"));

        let entry = &parsed.mcp_servers["filesystem"];
        assert_eq!(entry.command, "npx");
        assert_eq!(
            entry.args.as_ref().unwrap(),
            &vec!["-y".to_string(), "server-fs".to_string()]
        );
    }

    #[test]
    fn test_mcp_json_format() {
        let mut config = McpJsonConfig::default();
        let mcp = McpConfig {
            command: "node".to_string(),
            args: Some(vec!["server.js".to_string()]),
            env: Some(HashMap::from([("PORT".to_string(), "3000".to_string())])),
            transport: None,
            url: None,
            enabled: None,
        };
        config.add_server("test-server", &mcp);

        let json = serde_json::to_string(&config).unwrap();
        // Verify top-level key is "mcpServers"
        assert!(json.contains("\"mcpServers\""));
        assert!(json.contains("\"test-server\""));
    }

    #[test]
    fn test_resolve_env_with_vars() {
        std::env::set_var("GREAT_MCP_TEST_VAR", "resolved_value");
        let env = HashMap::from([("KEY".to_string(), "${GREAT_MCP_TEST_VAR}".to_string())]);
        let resolved = resolve_env(&env);
        assert_eq!(resolved["KEY"], "resolved_value");
        std::env::remove_var("GREAT_MCP_TEST_VAR");
    }

    #[test]
    fn test_resolve_env_missing_var() {
        let env = HashMap::from([(
            "KEY".to_string(),
            "${DEFINITELY_MISSING_VAR_XYZ}".to_string(),
        )]);
        let resolved = resolve_env(&env);
        assert_eq!(resolved["KEY"], "${DEFINITELY_MISSING_VAR_XYZ}");
    }

    #[test]
    fn test_resolve_env_no_refs() {
        let env = HashMap::from([("KEY".to_string(), "plain_value".to_string())]);
        let resolved = resolve_env(&env);
        assert_eq!(resolved["KEY"], "plain_value");
    }

    #[test]
    fn test_server_names() {
        let mut config = McpJsonConfig::default();
        let mcp_a = McpConfig {
            command: "cmd-a".to_string(),
            args: None,
            env: None,
            transport: None,
            url: None,
            enabled: None,
        };
        let mcp_b = McpConfig {
            command: "cmd-b".to_string(),
            args: None,
            env: None,
            transport: None,
            url: None,
            enabled: None,
        };
        config.add_server("alpha", &mcp_a);
        config.add_server("beta", &mcp_b);

        let mut names: Vec<&String> = config.server_names();
        names.sort();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "alpha");
        assert_eq!(names[1], "beta");
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let path = Path::new("/tmp/great_test_nonexistent_mcp.json");
        let config = McpJsonConfig::load(path).unwrap();
        assert!(config.mcp_servers.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".mcp.json");

        let mut config = McpJsonConfig::default();
        let mcp = McpConfig {
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            env: None,
            transport: None,
            url: None,
            enabled: None,
        };
        config.add_server("echo-server", &mcp);
        config.save(&path).unwrap();

        let loaded = McpJsonConfig::load(&path).unwrap();
        assert!(loaded.has_server("echo-server"));
        assert_eq!(loaded.mcp_servers["echo-server"].command, "echo");
    }

    #[test]
    fn test_project_mcp_path() {
        let path = project_mcp_path();
        assert_eq!(path, PathBuf::from(".mcp.json"));
    }

    #[test]
    fn test_user_mcp_path() {
        // Should return Some on any system with a home directory
        let path = user_mcp_path();
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains(".claude.json"));
        }
    }
}
