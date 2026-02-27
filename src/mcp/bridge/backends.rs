/// Static configuration for a supported AI CLI backend.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Backend identifier: "gemini", "codex", "claude", "grok", "ollama".
    pub name: &'static str,
    /// Human-readable display name for doctor/diagnostic output.
    #[allow(dead_code)]
    // Set for API completeness; doctor reads from BackendSpec via all_backend_specs().
    pub display_name: &'static str,
    /// Resolved absolute path to the CLI binary.
    pub binary: String,
    /// Model override (set via config or tool call parameter).
    pub model: Option<String>,
    /// CLI flag to bypass interactive approval prompts.
    pub auto_approve_flag: Option<&'static str>,
    /// Environment variable name for the API key (None = uses login/no key).
    #[allow(dead_code)]
    // Set for API completeness; doctor reads from BackendSpec via all_backend_specs().
    pub api_key_env: Option<&'static str>,
}

/// Per-backend static defaults.
struct BackendSpec {
    name: &'static str,
    display_name: &'static str,
    default_binary: &'static str,
    env_override: &'static str,
    auto_approve_flag: Option<&'static str>,
    api_key_env: Option<&'static str>,
    default_model: Option<&'static str>,
}

const BACKEND_SPECS: &[BackendSpec] = &[
    BackendSpec {
        name: "gemini",
        display_name: "Gemini CLI",
        default_binary: "gemini",
        env_override: "GREAT_GEMINI_CLI",
        auto_approve_flag: Some("-y"),
        api_key_env: Some("GEMINI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "codex",
        display_name: "Codex CLI",
        default_binary: "codex",
        env_override: "GREAT_CODEX_CLI",
        auto_approve_flag: Some("--full-auto"),
        api_key_env: Some("OPENAI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "claude",
        display_name: "Claude CLI",
        default_binary: "claude",
        env_override: "GREAT_CLAUDE_CLI",
        auto_approve_flag: Some("--dangerously-skip-permissions"),
        api_key_env: None,
        default_model: None,
    },
    BackendSpec {
        name: "grok",
        display_name: "Grok CLI",
        default_binary: "grok",
        env_override: "GREAT_GROK_CLI",
        auto_approve_flag: Some("-y"),
        api_key_env: Some("XAI_API_KEY"),
        default_model: None,
    },
    BackendSpec {
        name: "ollama",
        display_name: "Ollama",
        default_binary: "ollama",
        env_override: "GREAT_OLLAMA_CLI",
        auto_approve_flag: None,
        api_key_env: None,
        default_model: Some("llama3.2"),
    },
];

/// Discover available backends by checking PATH (via `which` crate) and
/// environment variable overrides.
///
/// If `filter` is non-empty, only backends whose name appears in `filter`
/// are considered. Otherwise all backends with a discoverable binary are
/// returned.
pub fn discover_backends(filter: &[String]) -> Vec<BackendConfig> {
    BACKEND_SPECS
        .iter()
        .filter(|spec| filter.is_empty() || filter.iter().any(|f| f == spec.name))
        .filter_map(|spec| {
            let binary = std::env::var(spec.env_override).ok().or_else(|| {
                which::which(spec.default_binary)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
            })?;

            // For ollama, check GREAT_OLLAMA_MODEL env for default model
            let model = if spec.name == "ollama" {
                std::env::var("GREAT_OLLAMA_MODEL")
                    .ok()
                    .or_else(|| spec.default_model.map(|s| s.to_string()))
            } else {
                None
            };

            Some(BackendConfig {
                name: spec.name,
                display_name: spec.display_name,
                binary,
                model,
                auto_approve_flag: spec.auto_approve_flag,
                api_key_env: spec.api_key_env,
            })
        })
        .collect()
}

/// Return static metadata for all known backends, regardless of whether
/// they are installed. Used by `great doctor` to report availability.
pub fn all_backend_specs() -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    BACKEND_SPECS
        .iter()
        .map(|s| (s.name, s.display_name, s.api_key_env))
        .collect()
}

/// Build the command arguments for invoking a backend with a prompt.
///
/// Returns `(binary, args_vec)` ready for `tokio::process::Command`.
/// When `auto_approve` is `false`, the backend's auto-approval flag
/// (e.g. `--dangerously-skip-permissions`) is suppressed. Note that
/// backends invoked without auto-approval may prompt on stdin; since
/// the bridge does not pipe stdin, they will typically detect a
/// non-TTY and either error or auto-decline.
pub fn build_command_args(
    backend: &BackendConfig,
    prompt: &str,
    model_override: Option<&str>,
    system_prompt: Option<&str>,
    auto_approve: bool,
) -> (String, Vec<String>) {
    let mut args = Vec::new();

    if backend.name == "ollama" {
        // ollama run <model> <prompt>
        args.push("run".to_string());
        let model = model_override
            .map(|s| s.to_string())
            .or_else(|| backend.model.clone())
            .unwrap_or_else(|| "llama3.2".to_string());
        args.push(model);
        args.push(prompt.to_string());
    } else {
        // Standard pattern: [binary] [auto_approve_flag] [-p prompt]
        if auto_approve {
            if let Some(flag) = backend.auto_approve_flag {
                args.push(flag.to_string());
            }
        }
        // System prompt injection (claude supports --system-prompt)
        if let Some(sp) = system_prompt {
            if backend.name == "claude" {
                args.push("--system-prompt".to_string());
                args.push(sp.to_string());
            }
            // For other backends, system prompt is prepended to the main prompt below.
        }
        if let Some(m) = model_override.or(backend.model.as_deref()) {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
        args.push("-p".to_string());
        // For backends without native system prompt support, prepend it
        if let Some(sp) = system_prompt {
            if backend.name != "claude" {
                args.push(format!("SYSTEM: {}\n\nTASK: {}", sp, prompt));
            } else {
                args.push(prompt.to_string());
            }
        } else {
            args.push(prompt.to_string());
        }
    }

    (backend.binary.clone(), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_specs_complete() {
        assert_eq!(BACKEND_SPECS.len(), 5);
        let names: Vec<&str> = BACKEND_SPECS.iter().map(|s| s.name).collect();
        assert!(names.contains(&"gemini"));
        assert!(names.contains(&"codex"));
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"grok"));
        assert!(names.contains(&"ollama"));
    }

    #[test]
    fn test_build_command_args_ollama() {
        let backend = BackendConfig {
            name: "ollama",
            display_name: "Ollama",
            binary: "/usr/bin/ollama".to_string(),
            model: Some("llama3.2".to_string()),
            auto_approve_flag: None,
            api_key_env: None,
        };
        let (bin, args) = build_command_args(&backend, "hello world", None, None, true);
        assert_eq!(bin, "/usr/bin/ollama");
        assert_eq!(args, vec!["run", "llama3.2", "hello world"]);
    }

    #[test]
    fn test_build_command_args_claude() {
        let backend = BackendConfig {
            name: "claude",
            display_name: "Claude CLI",
            binary: "/usr/bin/claude".to_string(),
            model: None,
            auto_approve_flag: Some("--dangerously-skip-permissions"),
            api_key_env: None,
        };
        let (bin, args) = build_command_args(&backend, "hello", None, None, true);
        assert_eq!(bin, "/usr/bin/claude");
        assert_eq!(args, vec!["--dangerously-skip-permissions", "-p", "hello"]);
    }

    #[test]
    fn test_build_command_args_claude_with_system_prompt() {
        let backend = BackendConfig {
            name: "claude",
            display_name: "Claude CLI",
            binary: "/usr/bin/claude".to_string(),
            model: None,
            auto_approve_flag: Some("--dangerously-skip-permissions"),
            api_key_env: None,
        };
        let (_, args) = build_command_args(&backend, "hello", None, Some("You are helpful"), true);
        assert!(args.contains(&"--system-prompt".to_string()));
        assert!(args.contains(&"You are helpful".to_string()));
    }

    #[test]
    fn test_build_command_args_gemini_with_system_prompt() {
        let backend = BackendConfig {
            name: "gemini",
            display_name: "Gemini CLI",
            binary: "/usr/bin/gemini".to_string(),
            model: None,
            auto_approve_flag: Some("-y"),
            api_key_env: Some("GEMINI_API_KEY"),
        };
        let (_, args) = build_command_args(&backend, "hello", None, Some("You are helpful"), true);
        // System prompt should be prepended to the prompt text
        assert!(args
            .last()
            .map_or(false, |a| a.contains("SYSTEM: You are helpful")));
        assert!(args.last().map_or(false, |a| a.contains("TASK: hello")));
    }

    #[test]
    fn test_build_command_args_claude_no_auto_approve() {
        let backend = BackendConfig {
            name: "claude",
            display_name: "Claude CLI",
            binary: "/usr/bin/claude".to_string(),
            model: None,
            auto_approve_flag: Some("--dangerously-skip-permissions"),
            api_key_env: None,
        };
        let (_, args) = build_command_args(&backend, "hello", None, None, false);
        assert!(!args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(args.contains(&"-p".to_string()));
        assert!(args.contains(&"hello".to_string()));
    }

    #[test]
    fn test_all_backend_specs_returns_all() {
        let specs = all_backend_specs();
        assert_eq!(specs.len(), BACKEND_SPECS.len());
    }

    #[test]
    fn test_discover_backends_empty_filter_returns_all_available() {
        let backends = discover_backends(&[]);
        for b in &backends {
            assert!(!b.binary.is_empty());
        }
    }
}
