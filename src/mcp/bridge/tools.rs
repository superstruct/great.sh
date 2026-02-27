use schemars::JsonSchema;
use serde::Deserialize;

// -- Tool parameter structs -----------------------------------------------

/// Parameters for the `prompt` tool (synchronous single round-trip).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PromptParams {
    /// The prompt text to send.
    pub prompt: String,
    /// Backend name: gemini, codex, claude, grok, ollama. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Parameters for the `run` tool (async task spawn).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunParams {
    /// Prompt to send asynchronously.
    pub prompt: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Override per-task timeout in seconds.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parameters for the `wait` tool (block until tasks complete).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitParams {
    /// Task IDs to wait for.
    pub task_ids: Vec<String>,
    /// Timeout in seconds for the wait operation.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parameters for the `get_result` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetResultParams {
    /// Task ID to retrieve results for.
    pub task_id: String,
}

/// Parameters for the `kill_task` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct KillTaskParams {
    /// Task ID to kill.
    pub task_id: String,
}

/// Parameters for the `research` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResearchParams {
    /// Research query.
    pub query: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Absolute file paths to include as context.
    #[serde(default)]
    pub files: Option<Vec<String>>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Parameters for the `analyze_code` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeCodeParams {
    /// Code snippet or absolute file path.
    pub code_or_path: String,
    /// Type of analysis: review, explain, optimize, security, test.
    pub analysis_type: AnalysisType,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

/// Analysis types for the `analyze_code` tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisType {
    Review,
    Explain,
    Optimize,
    Security,
    Test,
}

impl AnalysisType {
    /// Return the instruction prefix for this analysis type.
    pub fn instruction_prefix(&self) -> &'static str {
        match self {
            Self::Review => "Review this code for correctness, design, and maintainability:\n\n",
            Self::Explain => "Explain what this code does, step by step:\n\n",
            Self::Optimize => "Suggest performance and readability improvements for this code:\n\n",
            Self::Security => "Audit this code for security vulnerabilities:\n\n",
            Self::Test => "Write comprehensive tests for this code:\n\n",
        }
    }
}

/// Parameters for the `clink` tool (isolated subagent spawn).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClinkParams {
    /// Custom system prompt for the subagent.
    pub system_prompt: String,
    /// Task prompt for the subagent.
    pub prompt: String,
    /// Backend name. Omit for default.
    #[serde(default)]
    pub backend: Option<String>,
    /// Optional model override.
    #[serde(default)]
    pub model: Option<String>,
}

// -- Preset system --------------------------------------------------------

/// Tool presets control which tool groups are exposed via `tools/list`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    /// Chat only: prompt
    Minimal,
    /// Chat + agent: prompt, run, wait, list_tasks, get_result, kill_task
    Agent,
    /// Chat + agent + research + analysis
    Research,
    /// All groups including subagent (clink)
    Full,
}

impl Preset {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "minimal" => Some(Self::Minimal),
            "agent" => Some(Self::Agent),
            "research" => Some(Self::Research),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    /// Returns the tool names included in this preset.
    pub fn tool_names(&self) -> Vec<&'static str> {
        match self {
            Self::Minimal => vec!["prompt"],
            Self::Agent => vec![
                "prompt",
                "run",
                "wait",
                "list_tasks",
                "get_result",
                "kill_task",
            ],
            Self::Research => vec![
                "prompt",
                "run",
                "wait",
                "list_tasks",
                "get_result",
                "kill_task",
                "research",
                "analyze_code",
            ],
            Self::Full => vec![
                "prompt",
                "run",
                "wait",
                "list_tasks",
                "get_result",
                "kill_task",
                "research",
                "analyze_code",
                "clink",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_tool_counts() {
        assert_eq!(Preset::Minimal.tool_names().len(), 1);
        assert_eq!(Preset::Agent.tool_names().len(), 6);
        assert_eq!(Preset::Research.tool_names().len(), 8);
        assert_eq!(Preset::Full.tool_names().len(), 9);
    }

    #[test]
    fn test_preset_from_str() {
        assert_eq!(Preset::from_str("minimal"), Some(Preset::Minimal));
        assert_eq!(Preset::from_str("agent"), Some(Preset::Agent));
        assert_eq!(Preset::from_str("research"), Some(Preset::Research));
        assert_eq!(Preset::from_str("full"), Some(Preset::Full));
        assert_eq!(Preset::from_str("invalid"), None);
    }

    #[test]
    fn test_analysis_type_prefix() {
        assert!(AnalysisType::Review.instruction_prefix().contains("Review"));
        assert!(AnalysisType::Explain
            .instruction_prefix()
            .contains("Explain"));
        assert!(AnalysisType::Optimize
            .instruction_prefix()
            .contains("improvements"));
        assert!(AnalysisType::Security
            .instruction_prefix()
            .contains("security"));
        assert!(AnalysisType::Test.instruction_prefix().contains("tests"));
    }

    #[test]
    fn test_presets_are_cumulative() {
        let minimal = Preset::Minimal.tool_names();
        let agent = Preset::Agent.tool_names();
        let research = Preset::Research.tool_names();
        let full = Preset::Full.tool_names();

        // Each preset is a superset of the previous
        for tool in &minimal {
            assert!(agent.contains(tool));
        }
        for tool in &agent {
            assert!(research.contains(tool));
        }
        for tool in &research {
            assert!(full.contains(tool));
        }
    }
}
