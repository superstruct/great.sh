use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GreatConfig {
    pub tools: Option<Vec<ToolConfig>>,
    pub agents: Option<Vec<AgentConfig>>,
    pub mcp: Option<Vec<McpConfig>>,
    pub vault: Option<VaultConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub version: Option<String>,
    pub install: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfig {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub provider: Option<String>,
    pub path: Option<String>,
}
