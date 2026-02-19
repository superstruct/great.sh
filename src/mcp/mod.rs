use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: McpStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum McpStatus {
    Running,
    Stopped,
    Error(String),
    Unknown,
}

pub trait McpManager {
    fn list(&self) -> anyhow::Result<Vec<McpServer>>;
    fn add(&self, name: &str, command: &str, args: &[String]) -> anyhow::Result<()>;
    fn test(&self, name: &str) -> anyhow::Result<McpStatus>;
}
