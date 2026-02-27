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
    /// CLI flags to enable structured/JSON output from this backend.
    pub output_format_flags: &'static [&'static str],
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
    /// CLI flags to enable structured/JSON output from this backend.
    output_format_flags: &'static [&'static str],
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
        output_format_flags: &["--output-format", "json"],
    },
    BackendSpec {
        name: "codex",
        display_name: "Codex CLI",
        default_binary: "codex",
        env_override: "GREAT_CODEX_CLI",
        auto_approve_flag: Some("--full-auto"),
        api_key_env: Some("OPENAI_API_KEY"),
        default_model: None,
        output_format_flags: &["--json"],
    },
    BackendSpec {
        name: "claude",
        display_name: "Claude CLI",
        default_binary: "claude",
        env_override: "GREAT_CLAUDE_CLI",
        auto_approve_flag: Some("--dangerously-skip-permissions"),
        api_key_env: None,
        default_model: None,
        output_format_flags: &["--output-format", "stream-json", "--verbose"],
    },
    BackendSpec {
        name: "grok",
        display_name: "Grok CLI",
        default_binary: "grok",
        env_override: "GREAT_GROK_CLI",
        auto_approve_flag: Some("-y"),
        api_key_env: Some("XAI_API_KEY"),
        default_model: None,
        output_format_flags: &[],
    },
    BackendSpec {
        name: "ollama",
        display_name: "Ollama",
        default_binary: "ollama",
        env_override: "GREAT_OLLAMA_CLI",
        auto_approve_flag: None,
        api_key_env: None,
        default_model: Some("llama3.2"),
        output_format_flags: &[],
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
                output_format_flags: spec.output_format_flags,
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

/// The result of `build_command_args`: binary, CLI args, and an optional
/// prompt to pipe via stdin (for prompts exceeding `ARG_MAX`).
pub struct CommandSpec {
    pub binary: String,
    pub args: Vec<String>,
    /// When set, the prompt is delivered via stdin instead of as a CLI arg.
    /// The caller should use `Stdio::piped()` and write these bytes, then
    /// close stdin.
    pub stdin_prompt: Option<String>,
}

/// Byte threshold above which prompts are delivered via stdin instead of CLI
/// arg to avoid OS `ARG_MAX` limits. ~100K chars ≈ 100 KB.
const STDIN_PROMPT_THRESHOLD: usize = 100_000;

/// Build the command arguments for invoking a backend with a prompt.
///
/// Returns a `CommandSpec` ready for `tokio::process::Command`.
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
    session_id: Option<&str>,
) -> CommandSpec {
    let mut args = Vec::new();
    let mut stdin_prompt: Option<String> = None;

    if backend.name == "ollama" {
        // ollama run <model> <prompt>
        args.push("run".to_string());
        let model = model_override
            .map(|s| s.to_string())
            .or_else(|| backend.model.clone())
            .unwrap_or_else(|| "llama3.2".to_string());
        args.push(model);
        args.push(prompt.to_string());
    } else if backend.name == "codex" {
        // codex exec [OPTIONS] [PROMPT] — non-interactive mode with positional prompt
        // Session resume: codex exec resume <thread_id> [OPTIONS] [PROMPT]
        if let Some(sid) = session_id {
            args.push("exec".to_string());
            args.push("resume".to_string());
            args.push(sid.to_string());
        } else {
            args.push("exec".to_string());
        }
        if auto_approve {
            if let Some(flag) = backend.auto_approve_flag {
                args.push(flag.to_string());
            }
        }
        // Output format flags (e.g. --json)
        for flag in backend.output_format_flags {
            args.push((*flag).to_string());
        }
        if let Some(m) = model_override.or(backend.model.as_deref()) {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
        // System prompt prepended (codex has no native --system-prompt)
        let final_prompt = if let Some(sp) = system_prompt {
            format!("SYSTEM: {}\n\nTASK: {}", sp, prompt)
        } else {
            prompt.to_string()
        };
        if final_prompt.len() > STDIN_PROMPT_THRESHOLD {
            stdin_prompt = Some(final_prompt);
        } else {
            args.push(final_prompt);
        }
    } else {
        // Standard pattern: [binary] [auto_approve_flag] [output_flags] -p <prompt>
        // Used by gemini, claude, grok.
        if auto_approve {
            if let Some(flag) = backend.auto_approve_flag {
                args.push(flag.to_string());
            }
        }
        // Output format flags (e.g. --output-format stream-json --verbose)
        for flag in backend.output_format_flags {
            args.push((*flag).to_string());
        }
        // Session resume for claude: -r <session_id> (before -p)
        if backend.name == "claude" {
            if let Some(sid) = session_id {
                args.push("-r".to_string());
                args.push(sid.to_string());
            }
        } else if session_id.is_some() {
            // gemini's -r is index-based (not ID-based), grok has no session support
            tracing::warn!(
                "session_id not supported for backend '{}', ignoring",
                backend.name
            );
        }
        // System prompt injection (claude supports --append-system-prompt)
        if let Some(sp) = system_prompt {
            if backend.name == "claude" {
                args.push("--append-system-prompt".to_string());
                args.push(sp.to_string());
            }
            // For other backends, system prompt is prepended to the main prompt below.
        }
        if let Some(m) = model_override.or(backend.model.as_deref()) {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
        args.push("-p".to_string());
        // For backends without native system prompt support, prepend it.
        // Prefix with a space if the prompt starts with '-' to prevent
        // yargs-based CLIs (gemini) from misinterpreting it as a flag.
        let final_prompt = if let Some(sp) = system_prompt {
            if backend.name != "claude" {
                format!("SYSTEM: {}\n\nTASK: {}", sp, prompt)
            } else {
                prompt.to_string()
            }
        } else {
            prompt.to_string()
        };
        if final_prompt.len() > STDIN_PROMPT_THRESHOLD {
            // Placeholder arg for -p; actual prompt comes via stdin
            args.push("(prompt via stdin)".to_string());
            stdin_prompt = Some(final_prompt);
        } else if final_prompt.starts_with('-') {
            args.push(format!(" {}", final_prompt));
        } else {
            args.push(final_prompt);
        }
    }

    CommandSpec {
        binary: backend.binary.clone(),
        args,
        stdin_prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_backend(name: &'static str) -> BackendConfig {
        let spec = BACKEND_SPECS.iter().find(|s| s.name == name).unwrap();
        BackendConfig {
            name: spec.name,
            display_name: spec.display_name,
            binary: format!("/usr/bin/{}", spec.default_binary),
            model: spec.default_model.map(|s| s.to_string()),
            auto_approve_flag: spec.auto_approve_flag,
            api_key_env: spec.api_key_env,
            output_format_flags: spec.output_format_flags,
        }
    }

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
        let backend = make_backend("ollama");
        let spec = build_command_args(&backend, "hello world", None, None, true, None);
        assert_eq!(spec.binary, "/usr/bin/ollama");
        assert_eq!(spec.args, vec!["run", "llama3.2", "hello world"]);
        assert!(spec.stdin_prompt.is_none());
    }

    #[test]
    fn test_build_command_args_claude() {
        let backend = make_backend("claude");
        let spec = build_command_args(&backend, "hello", None, None, true, None);
        assert_eq!(spec.binary, "/usr/bin/claude");
        assert_eq!(
            spec.args,
            vec![
                "--dangerously-skip-permissions",
                "--output-format",
                "stream-json",
                "--verbose",
                "-p",
                "hello"
            ]
        );
    }

    #[test]
    fn test_build_command_args_claude_with_system_prompt() {
        let backend = make_backend("claude");
        let spec = build_command_args(&backend, "hello", None, Some("You are helpful"), true, None);
        assert!(spec.args.contains(&"--append-system-prompt".to_string()));
        assert!(spec.args.contains(&"You are helpful".to_string()));
    }

    #[test]
    fn test_build_command_args_gemini_with_system_prompt() {
        let backend = make_backend("gemini");
        let spec = build_command_args(&backend, "hello", None, Some("You are helpful"), true, None);
        // System prompt should be prepended to the prompt text
        assert!(spec
            .args
            .last()
            .map_or(false, |a| a.contains("SYSTEM: You are helpful")));
        assert!(spec
            .args
            .last()
            .map_or(false, |a| a.contains("TASK: hello")));
    }

    #[test]
    fn test_build_command_args_claude_no_auto_approve() {
        let backend = make_backend("claude");
        let spec = build_command_args(&backend, "hello", None, None, false, None);
        assert!(!spec
            .args
            .contains(&"--dangerously-skip-permissions".to_string()));
        assert!(spec.args.contains(&"-p".to_string()));
        assert!(spec.args.contains(&"hello".to_string()));
    }

    #[test]
    fn test_build_command_args_codex() {
        let backend = make_backend("codex");
        let spec = build_command_args(&backend, "hello", None, None, true, None);
        assert_eq!(spec.binary, "/usr/bin/codex");
        assert_eq!(spec.args, vec!["exec", "--full-auto", "--json", "hello"]);
    }

    #[test]
    fn test_build_command_args_codex_with_system_prompt() {
        let backend = make_backend("codex");
        let spec = build_command_args(&backend, "task", None, Some("You are helpful"), true, None);
        assert_eq!(spec.args[0], "exec");
        assert!(spec
            .args
            .last()
            .unwrap()
            .contains("SYSTEM: You are helpful"));
        assert!(spec.args.last().unwrap().contains("TASK: task"));
    }

    #[test]
    fn test_build_command_args_dash_prefix_escaped() {
        let backend = make_backend("gemini");
        let spec = build_command_args(
            &backend,
            "--- FILE: test ---\ncontent",
            None,
            None,
            true,
            None,
        );
        // Prompt should be space-prefixed to avoid yargs misinterpreting leading dashes
        let prompt_arg = spec.args.last().unwrap();
        assert!(prompt_arg.starts_with(' '));
        assert!(prompt_arg.contains("--- FILE: test ---"));
    }

    #[test]
    fn test_build_command_args_gemini_output_format() {
        let backend = make_backend("gemini");
        let spec = build_command_args(&backend, "hello", None, None, true, None);
        assert!(spec.args.contains(&"--output-format".to_string()));
        assert!(spec.args.contains(&"json".to_string()));
    }

    #[test]
    fn test_build_command_args_codex_json_flag() {
        let backend = make_backend("codex");
        let spec = build_command_args(&backend, "hello", None, None, true, None);
        assert!(spec.args.contains(&"--json".to_string()));
    }

    #[test]
    fn test_build_command_args_claude_session_resume() {
        let backend = make_backend("claude");
        let spec = build_command_args(&backend, "hello", None, None, true, Some("session-abc-123"));
        assert!(spec.args.contains(&"-r".to_string()));
        assert!(spec.args.contains(&"session-abc-123".to_string()));
    }

    #[test]
    fn test_build_command_args_codex_session_resume() {
        let backend = make_backend("codex");
        let spec = build_command_args(&backend, "hello", None, None, true, Some("thread-xyz-789"));
        // codex exec resume <thread_id> ...
        assert_eq!(spec.args[0], "exec");
        assert_eq!(spec.args[1], "resume");
        assert_eq!(spec.args[2], "thread-xyz-789");
    }

    #[test]
    fn test_build_command_args_stdin_for_long_prompt() {
        let backend = make_backend("claude");
        let long_prompt = "x".repeat(STDIN_PROMPT_THRESHOLD + 1);
        let spec = build_command_args(&backend, &long_prompt, None, None, true, None);
        assert!(spec.stdin_prompt.is_some());
        assert_eq!(spec.stdin_prompt.as_ref().unwrap().len(), long_prompt.len());
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
