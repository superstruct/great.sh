use anyhow::{bail, Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;

/// Arguments for the `loop` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: LoopCommand,

    /// Not a CLI argument -- hidden from clap.
    #[arg(skip)]
    pub non_interactive: bool,
}

/// Subcommands for managing the great.sh Loop agent team.
#[derive(Subcommand)]
pub enum LoopCommand {
    /// Install the great.sh Loop agent team to ~/.claude/
    Install {
        /// Also set up .tasks/ working state in current directory
        #[arg(long)]
        project: bool,

        /// Overwrite existing files without prompting
        #[arg(long)]
        force: bool,
    },
    /// Show loop installation status
    Status,
    /// Remove loop agent files from ~/.claude/
    Uninstall,
}

/// An agent persona markdown file embedded at compile time.
struct AgentFile {
    name: &'static str,
    content: &'static str,
}

/// A slash-command markdown file embedded at compile time.
struct CommandFile {
    name: &'static str,
    content: &'static str,
}

/// All 15 agent persona files shipped with the great.sh Loop.
const AGENTS: &[AgentFile] = &[
    AgentFile {
        name: "nightingale",
        content: include_str!("../../loop/agents/nightingale.md"),
    },
    AgentFile {
        name: "lovelace",
        content: include_str!("../../loop/agents/lovelace.md"),
    },
    AgentFile {
        name: "socrates",
        content: include_str!("../../loop/agents/socrates.md"),
    },
    AgentFile {
        name: "humboldt",
        content: include_str!("../../loop/agents/humboldt.md"),
    },
    AgentFile {
        name: "davinci",
        content: include_str!("../../loop/agents/davinci.md"),
    },
    AgentFile {
        name: "vonbraun",
        content: include_str!("../../loop/agents/vonbraun.md"),
    },
    AgentFile {
        name: "turing",
        content: include_str!("../../loop/agents/turing.md"),
    },
    AgentFile {
        name: "kerckhoffs",
        content: include_str!("../../loop/agents/kerckhoffs.md"),
    },
    AgentFile {
        name: "rams",
        content: include_str!("../../loop/agents/rams.md"),
    },
    AgentFile {
        name: "nielsen",
        content: include_str!("../../loop/agents/nielsen.md"),
    },
    AgentFile {
        name: "knuth",
        content: include_str!("../../loop/agents/knuth.md"),
    },
    AgentFile {
        name: "gutenberg",
        content: include_str!("../../loop/agents/gutenberg.md"),
    },
    AgentFile {
        name: "hopper",
        content: include_str!("../../loop/agents/hopper.md"),
    },
    AgentFile {
        name: "dijkstra",
        content: include_str!("../../loop/agents/dijkstra.md"),
    },
    AgentFile {
        name: "wirth",
        content: include_str!("../../loop/agents/wirth.md"),
    },
];

/// All 5 slash-command files shipped with the great.sh Loop.
const COMMANDS: &[CommandFile] = &[
    CommandFile {
        name: "loop",
        content: include_str!("../../loop/commands/loop.md"),
    },
    CommandFile {
        name: "bugfix",
        content: include_str!("../../loop/commands/bugfix.md"),
    },
    CommandFile {
        name: "deploy",
        content: include_str!("../../loop/commands/deploy.md"),
    },
    CommandFile {
        name: "discover",
        content: include_str!("../../loop/commands/discover.md"),
    },
    CommandFile {
        name: "backlog",
        content: include_str!("../../loop/commands/backlog.md"),
    },
];

/// Teams configuration JSON embedded at compile time.
const TEAMS_CONFIG: &str = include_str!("../../loop/teams-config.json");

/// Observer report template embedded at compile time.
const OBSERVER_TEMPLATE: &str = include_str!("../../loop/observer-template.md");

/// Hook handler script embedded at compile time.
const HOOK_UPDATE_STATE: &str = include_str!("../../loop/hooks/update-state.sh");

/// Run the `great loop` subcommand.
pub fn run(args: Args) -> Result<()> {
    let non_interactive = args.non_interactive;
    match args.command {
        LoopCommand::Install { project, force } => {
            run_install(project, force, non_interactive)
        }
        LoopCommand::Status => run_status(),
        LoopCommand::Uninstall => run_uninstall(),
    }
}

/// Returns the correct `statusLine` JSON value for Claude Code settings.
///
/// Claude Code requires `"type": "command"` as a discriminator field.
/// Without it, the entire settings.json is rejected by the validator.
fn statusline_value() -> serde_json::Value {
    serde_json::json!({
        "type": "command",
        "command": "great statusline"
    })
}

/// Returns the `hooks` JSON object for Claude Code settings.
///
/// Each event maps to an array with one matcher object containing
/// one command hook. The hook runs async (state writes are side-effects
/// that must not block the agent loop).
fn hooks_value() -> serde_json::Value {
    let cmd = "~/.claude/hooks/great-loop/update-state.sh";
    let hook_entry = |_event: &str| -> serde_json::Value {
        serde_json::json!([{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": cmd,
                "async": true
            }]
        }])
    };
    serde_json::json!({
        "SubagentStart": hook_entry("SubagentStart"),
        "SubagentStop": hook_entry("SubagentStop"),
        "TeammateIdle": hook_entry("TeammateIdle"),
        "TaskCompleted": hook_entry("TaskCompleted"),
        "Stop": hook_entry("Stop"),
        "SessionEnd": hook_entry("SessionEnd")
    })
}

/// Check if a hook matcher array entry is a great-loop hook.
/// Identifies by checking if any hook command contains "great-loop/update-state.sh".
fn is_great_loop_hook(entry: &serde_json::Value) -> bool {
    entry
        .get("hooks")
        .and_then(|h| h.as_array())
        .map(|hooks| {
            hooks.iter().any(|hook| {
                hook.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("great-loop/update-state.sh"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Returns the subset of the 21 managed file paths that already exist on disk.
///
/// Each returned path is the full absolute path. The caller uses this list to
/// decide whether to prompt the user for confirmation before overwriting.
fn collect_existing_paths(claude_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let agents_dir = claude_dir.join("agents");
    let commands_dir = claude_dir.join("commands");
    let teams_dir = claude_dir.join("teams").join("loop");

    let mut existing = Vec::new();

    for agent in AGENTS {
        let path = agents_dir.join(format!("{}.md", agent.name));
        if path.exists() {
            existing.push(path);
        }
    }

    for cmd in COMMANDS {
        let path = commands_dir.join(format!("{}.md", cmd.name));
        if path.exists() {
            existing.push(path);
        }
    }

    let config_path = teams_dir.join("config.json");
    if config_path.exists() {
        existing.push(config_path);
    }

    let hook_path = claude_dir
        .join("hooks")
        .join("great-loop")
        .join("update-state.sh");
    if hook_path.exists() {
        existing.push(hook_path);
    }

    existing
}

/// Prompts the user to confirm overwriting existing files, or aborts in non-TTY contexts.
///
/// Returns `Ok(true)` if the user confirms, `Ok(false)` if they decline or stdin is not a TTY.
fn confirm_overwrite(
    existing: &[std::path::PathBuf],
    non_interactive: bool,
) -> Result<bool> {
    use std::io::IsTerminal;

    eprintln!();
    output::warning("The following files already exist and will be overwritten:");
    for path in existing {
        let display = if let Some(home) = dirs::home_dir() {
            if let Ok(relative) = path.strip_prefix(&home) {
                format!("~/{}", relative.display())
            } else {
                path.display().to_string()
            }
        } else {
            path.display().to_string()
        };
        eprintln!("  {}", display);
    }

    if non_interactive || !std::io::stdin().is_terminal() {
        eprintln!();
        output::error("stdin is not a terminal -- cannot prompt for confirmation.");
        output::info("Re-run with --force to overwrite: great loop install --force");
        return Ok(false);
    }

    eprint!("Overwrite? [y/N] ");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("failed to read from stdin")?;

    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

/// Install the great.sh Loop agent team to `~/.claude/`.
fn run_install(project: bool, force: bool, non_interactive: bool) -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");

    output::header("great.sh Loop — Installing agent team");
    println!();

    // Create directories
    let agents_dir = claude_dir.join("agents");
    let commands_dir = claude_dir.join("commands");
    let teams_dir = claude_dir.join("teams").join("loop");

    std::fs::create_dir_all(&agents_dir).context("failed to create ~/.claude/agents/ directory")?;
    std::fs::create_dir_all(&commands_dir)
        .context("failed to create ~/.claude/commands/ directory")?;
    std::fs::create_dir_all(&teams_dir)
        .context("failed to create ~/.claude/teams/loop/ directory")?;

    // Check for existing files before writing
    let existing = collect_existing_paths(&claude_dir);
    if !existing.is_empty() && !force {
        let confirmed = confirm_overwrite(&existing, non_interactive)?;
        if !confirmed {
            bail!("aborted: no files were modified");
        }
    }

    if force && !existing.is_empty() {
        output::info("(--force: overwriting existing files)");
    }

    // Write agent files
    for agent in AGENTS {
        let path = agents_dir.join(format!("{}.md", agent.name));
        std::fs::write(&path, agent.content)
            .with_context(|| format!("failed to write agent file: {}", path.display()))?;
    }
    output::success(&format!(
        "{} agent personas -> ~/.claude/agents/",
        AGENTS.len()
    ));

    // Write command files
    for cmd in COMMANDS {
        let path = commands_dir.join(format!("{}.md", cmd.name));
        std::fs::write(&path, cmd.content)
            .with_context(|| format!("failed to write command file: {}", path.display()))?;
    }
    output::success(&format!(
        "{} commands -> ~/.claude/commands/",
        COMMANDS.len()
    ));

    // Write teams config
    let config_path = teams_dir.join("config.json");
    std::fs::write(&config_path, TEAMS_CONFIG)
        .context("failed to write teams config to ~/.claude/teams/loop/config.json")?;
    output::success("Agent Teams config -> ~/.claude/teams/loop/");

    // Write hook handler script
    let hooks_dir = claude_dir.join("hooks").join("great-loop");
    std::fs::create_dir_all(&hooks_dir)
        .context("failed to create ~/.claude/hooks/great-loop/ directory")?;
    let hook_script_path = hooks_dir.join("update-state.sh");
    std::fs::write(&hook_script_path, HOOK_UPDATE_STATE)
        .context("failed to write hook script")?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_script_path)
            .context("failed to read hook script metadata")?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_script_path, perms)
            .context("failed to set hook script permissions")?;
    }
    output::success("Hook handler -> ~/.claude/hooks/great-loop/update-state.sh");

    // Handle settings.json (non-destructive merge for all keys)
    let settings_path = claude_dir.join("settings.json");

    // Guard: skip settings.json injection if the file is read-only
    let settings_readonly = settings_path.exists()
        && std::fs::metadata(&settings_path)
            .map(|m| m.permissions().readonly())
            .unwrap_or(false);
    if settings_readonly {
        println!();
        output::warning(
            "settings.json is read-only \u{2014} hooks and statusLine not injected.",
        );
        output::info(
            "  Fix: chmod u+w ~/.claude/settings.json && great loop install --force",
        );
    } else if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json")?;
        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(mut val) => {
                if let Some(obj) = val.as_object_mut() {
                    let mut modified = false;

                    // --- Inject env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS ---
                    let env_obj = obj
                        .entry("env")
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(env_map) = env_obj.as_object_mut() {
                        if !env_map.contains_key("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS") {
                            env_map.insert(
                                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS".to_string(),
                                serde_json::json!("1"),
                            );
                            modified = true;
                        }
                    }

                    // --- Inject or merge hooks ---
                    let desired_hooks = hooks_value();
                    let hooks_obj = obj
                        .entry("hooks")
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(hooks_map) = hooks_obj.as_object_mut() {
                        // Snapshot for idempotency check
                        let hooks_before =
                            serde_json::to_string(hooks_map).unwrap_or_default();

                        if let Some(desired_map) = desired_hooks.as_object() {
                            for (event_name, desired_matchers) in desired_map {
                                if let Some(existing_arr) = hooks_map
                                    .get_mut(event_name)
                                    .and_then(|v| v.as_array_mut())
                                {
                                    // Remove any existing great-loop entries (dedup)
                                    existing_arr
                                        .retain(|entry| !is_great_loop_hook(entry));
                                    // Append the great-loop entries
                                    if let Some(new_entries) = desired_matchers.as_array()
                                    {
                                        existing_arr
                                            .extend(new_entries.iter().cloned());
                                    }
                                } else {
                                    // Event key does not exist yet -- insert
                                    hooks_map.insert(
                                        event_name.clone(),
                                        desired_matchers.clone(),
                                    );
                                }
                            }
                        }

                        // Only mark modified if hooks actually changed
                        let hooks_after =
                            serde_json::to_string(hooks_map).unwrap_or_default();
                        if hooks_before != hooks_after {
                            modified = true;
                        }
                    }

                    // --- Inject or repair statusLine ---
                    let needs_statusline = if !obj.contains_key("statusLine") {
                        true
                    } else if let Some(sl) =
                        obj.get("statusLine").and_then(|v| v.as_object())
                    {
                        !sl.contains_key("type")
                    } else {
                        false
                    };
                    if needs_statusline {
                        obj.insert("statusLine".to_string(), statusline_value());
                        modified = true;
                    }

                    // --- Write back if anything changed ---
                    if modified {
                        let formatted = serde_json::to_string_pretty(&val)
                            .context("failed to serialize settings.json")?;
                        std::fs::write(&settings_path, formatted)
                            .context("failed to write ~/.claude/settings.json")?;
                        output::success(
                            "Settings updated (env, hooks, statusLine) in ~/.claude/settings.json",
                        );
                    } else {
                        output::success(
                            "Settings already configured in ~/.claude/settings.json",
                        );
                    }
                }
            }
            Err(_) => {
                output::warning(
                    "settings.json is not valid JSON; skipping injection",
                );
            }
        }
    } else {
        // No existing settings.json -- create with all managed keys
        let mut default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "permissions": {
                "allow": [
                    "Bash(cargo *)",
                    "Bash(pnpm *)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep",
                    "LS"
                ]
            },
            "statusLine": statusline_value()
        });
        // Merge hooks into the default settings
        if let Some(obj) = default_settings.as_object_mut() {
            obj.insert("hooks".to_string(), hooks_value());
        }
        let formatted = serde_json::to_string_pretty(&default_settings)
            .context("failed to serialize default settings")?;
        std::fs::write(&settings_path, formatted)
            .context("failed to write ~/.claude/settings.json")?;
        output::success(
            "Settings with Agent Teams, hooks, and statusLine -> ~/.claude/settings.json",
        );
    }

    // Project working state
    if project {
        println!();
        output::header("Setting up project working state");

        let task_dirs = ["backlog", "ready", "in-progress", "done", "reports"];
        for dir in &task_dirs {
            let path = std::path::Path::new(".tasks").join(dir);
            std::fs::create_dir_all(&path)
                .with_context(|| format!("failed to create .tasks/{}/", dir))?;
        }

        // Write observer template
        let template_path = std::path::Path::new(".tasks/reports/.template.md");
        std::fs::write(template_path, OBSERVER_TEMPLATE)
            .context("failed to write .tasks/reports/.template.md")?;

        // Append to .gitignore if needed
        let gitignore_path = std::path::Path::new(".gitignore");
        let needs_entry = if gitignore_path.exists() {
            let contents =
                std::fs::read_to_string(gitignore_path).context("failed to read .gitignore")?;
            !contents.contains(".tasks/")
        } else {
            true
        };

        if needs_entry {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(gitignore_path)
                .context("failed to open .gitignore for appending")?;
            use std::io::Write;
            writeln!(file)?;
            writeln!(file, "# great.sh Loop working state")?;
            writeln!(file, ".tasks/")?;
        }

        output::success(".tasks/ created, .gitignore updated");
    }

    // Summary
    println!();
    output::header("great.sh Loop installed!");
    println!();
    output::info("16 roles: 4 teammates + 11 subagents + 1 team lead");
    output::info("All Claude: Opus + Sonnet + Haiku");
    println!();
    if project {
        output::info("Usage: claude -> /loop [task description]");
    } else {
        output::info("Next: great loop install --project  (in your repo)");
    }

    Ok(())
}

/// Show the installation status of the great.sh Loop.
fn run_status() -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");

    output::header("great.sh Loop — Status");
    println!();

    // Check key agent file
    let agents_ok = claude_dir.join("agents").join("nightingale.md").exists();
    if agents_ok {
        output::success("Agent personas: installed");
    } else {
        output::error("Agent personas: not installed");
    }

    // Check key command file
    let commands_ok = claude_dir.join("commands").join("loop.md").exists();
    if commands_ok {
        output::success("Loop commands: installed");
    } else {
        output::error("Loop commands: not installed");
    }

    // Check teams config
    let teams_ok = claude_dir
        .join("teams")
        .join("loop")
        .join("config.json")
        .exists();
    if teams_ok {
        output::success("Agent Teams config: installed");
    } else {
        output::error("Agent Teams config: not installed");
    }

    // Check settings.json for agent teams env
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path).unwrap_or_default();
        if contents.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS") {
            output::success("Agent Teams env: enabled in settings.json");
        } else {
            output::warning("Agent Teams env: not found in settings.json");
        }
    } else {
        output::warning("settings.json: not found");
    }

    // Check for hook handler script
    let hook_script = claude_dir
        .join("hooks")
        .join("great-loop")
        .join("update-state.sh");
    if hook_script.exists() {
        output::success("Hook handler: installed");
    } else {
        output::warning("Hook handler: not installed (statusline will show 'idle')");
    }

    // Check for hooks in settings.json
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path).unwrap_or_default();
        if contents.contains("great-loop/update-state.sh") {
            output::success("Hooks config: registered in settings.json");
        } else {
            output::warning("Hooks config: not found in settings.json");
        }
    }

    // Check for jq (required by hook script)
    match std::process::Command::new("jq").arg("--version").output() {
        Ok(output_result) if output_result.status.success() => {
            output::success("jq: available");
        }
        _ => {
            output::warning("jq: not found (required for statusline hook handler)");
        }
    }

    // Check project state
    println!();
    let tasks_exists = std::path::Path::new(".tasks").exists();
    if tasks_exists {
        output::success("Project state: .tasks/ found in current directory");
    } else {
        output::info("Project state: no .tasks/ in current directory");
        output::info("  Run: great loop install --project");
    }

    // Overall verdict
    println!();
    if agents_ok && commands_ok && teams_ok {
        output::success("great.sh Loop is installed and ready.");
    } else {
        output::info("Run: great loop install");
    }

    Ok(())
}

/// Remove the great.sh Loop agent files from `~/.claude/`.
fn run_uninstall() -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");

    output::header("great.sh Loop — Uninstalling");
    println!();

    let mut removed = 0;

    // Remove agent files
    let agents_dir = claude_dir.join("agents");
    for agent in AGENTS {
        let path = agents_dir.join(format!("{}.md", agent.name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            removed += 1;
        }
    }
    output::success(&format!("Removed {} agent files", removed));

    // Remove command files
    let commands_dir = claude_dir.join("commands");
    let mut cmd_removed = 0;
    for cmd in COMMANDS {
        let path = commands_dir.join(format!("{}.md", cmd.name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            cmd_removed += 1;
        }
    }
    output::success(&format!("Removed {} command files", cmd_removed));

    // Remove teams/loop directory
    let teams_dir = claude_dir.join("teams").join("loop");
    if teams_dir.exists() {
        std::fs::remove_dir_all(&teams_dir).context("failed to remove ~/.claude/teams/loop/")?;
        output::success("Removed teams/loop/ directory");
    }

    // Remove hook handler directory
    let hooks_dir = claude_dir.join("hooks").join("great-loop");
    if hooks_dir.exists() {
        std::fs::remove_dir_all(&hooks_dir)
            .context("failed to remove ~/.claude/hooks/great-loop/")?;
        output::success("Removed hooks/great-loop/ directory");
    }

    println!();
    output::info("settings.json was NOT modified (may contain other config).");
    output::success("great.sh Loop uninstalled.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_count() {
        assert_eq!(AGENTS.len(), 15);
    }

    #[test]
    fn test_commands_count() {
        assert_eq!(COMMANDS.len(), 5);
    }

    #[test]
    fn test_agent_names_unique() {
        let mut names: Vec<&str> = AGENTS.iter().map(|a| a.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), AGENTS.len());
    }

    #[test]
    fn test_command_names_unique() {
        let mut names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), COMMANDS.len());
    }

    #[test]
    fn test_teams_config_valid_json() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(TEAMS_CONFIG);
        assert!(parsed.is_ok(), "teams-config.json must be valid JSON");
    }

    #[test]
    fn test_teams_config_has_loop_name() {
        let parsed: serde_json::Value = serde_json::from_str(TEAMS_CONFIG).expect("valid JSON");
        assert_eq!(parsed["name"].as_str(), Some("loop"));
    }

    #[test]
    fn test_no_architecton_in_agents() {
        for agent in AGENTS {
            assert!(
                !agent.content.contains("Architecton"),
                "Agent '{}' must not contain 'Architecton' — use 'great.sh Loop'",
                agent.name
            );
        }
    }

    #[test]
    fn test_no_architecton_in_commands() {
        for cmd in COMMANDS {
            assert!(
                !cmd.content.contains("Architecton"),
                "Command '{}' must not contain 'Architecton' — use 'great.sh Loop'",
                cmd.name
            );
        }
    }

    #[test]
    fn test_no_architecton_in_teams_config() {
        assert!(
            !TEAMS_CONFIG.contains("Architecton"),
            "teams-config.json must not contain 'Architecton' — use 'great.sh Loop'"
        );
    }

    #[test]
    fn test_observer_template_not_empty() {
        assert!(!OBSERVER_TEMPLATE.is_empty());
        assert!(OBSERVER_TEMPLATE.contains("Observer Report"));
    }

    #[test]
    fn test_all_expected_agents_present() {
        let expected = [
            "nightingale",
            "lovelace",
            "socrates",
            "humboldt",
            "davinci",
            "vonbraun",
            "turing",
            "kerckhoffs",
            "rams",
            "nielsen",
            "knuth",
            "gutenberg",
            "hopper",
            "dijkstra",
            "wirth",
        ];
        for name in &expected {
            assert!(
                AGENTS.iter().any(|a| a.name == *name),
                "Missing agent: {}",
                name
            );
        }
    }

    #[test]
    fn test_statusline_value_has_type_command() {
        let val = super::statusline_value();
        let obj = val.as_object().expect("statusline_value must be an object");
        assert_eq!(
            obj.get("type").and_then(|v| v.as_str()),
            Some("command"),
            "statusLine must contain \"type\": \"command\""
        );
        assert_eq!(
            obj.get("command").and_then(|v| v.as_str()),
            Some("great statusline"),
            "statusLine must contain \"command\": \"great statusline\""
        );
    }

    #[test]
    fn test_default_settings_statusline_has_type() {
        let default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "permissions": {
                "allow": [
                    "Bash(cargo *)",
                    "Bash(pnpm *)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep",
                    "LS"
                ]
            },
            "statusLine": super::statusline_value()
        });
        let sl = &default_settings["statusLine"];
        assert_eq!(sl["type"].as_str(), Some("command"));
        assert_eq!(sl["command"].as_str(), Some("great statusline"));
    }

    /// Simulate the repair branch: given a settings object with a broken
    /// statusLine (missing "type"), the repair logic should replace it with
    /// the correct shape from statusline_value().
    #[test]
    fn test_repair_fixes_broken_statusline() {
        let mut settings = serde_json::json!({
            "env": { "SOME_KEY": "value" },
            "statusLine": { "command": "great statusline" }
        });

        // Run the same decision logic as run_install's repair branch
        let obj = settings.as_object_mut().unwrap();
        let needs_write = if !obj.contains_key("statusLine") {
            obj.insert("statusLine".to_string(), super::statusline_value());
            true
        } else if let Some(sl) = obj.get("statusLine").and_then(|v| v.as_object()) {
            if !sl.contains_key("type") {
                obj.insert("statusLine".to_string(), super::statusline_value());
                true
            } else {
                false
            }
        } else {
            false
        };

        assert!(needs_write, "broken statusLine should trigger a write");
        let sl = obj.get("statusLine").unwrap();
        assert_eq!(
            sl["type"].as_str(),
            Some("command"),
            "repair must add type field"
        );
        assert_eq!(
            sl["command"].as_str(),
            Some("great statusline"),
            "repair must preserve command"
        );
        // Other keys must survive the round-trip
        assert_eq!(settings["env"]["SOME_KEY"].as_str(), Some("value"));
    }

    /// Simulate the repair branch: given a settings object with a correct
    /// statusLine (has "type": "command"), the repair logic should NOT
    /// trigger a write.
    #[test]
    fn test_correct_statusline_skips_repair() {
        let mut settings = serde_json::json!({
            "statusLine": { "type": "command", "command": "great statusline" }
        });

        let obj = settings.as_object_mut().unwrap();
        let needs_write = if !obj.contains_key("statusLine") {
            obj.insert("statusLine".to_string(), super::statusline_value());
            true
        } else if let Some(sl) = obj.get("statusLine").and_then(|v| v.as_object()) {
            if !sl.contains_key("type") {
                obj.insert("statusLine".to_string(), super::statusline_value());
                true
            } else {
                false
            }
        } else {
            false
        };

        assert!(
            !needs_write,
            "correct statusLine should NOT trigger a write"
        );
    }

    #[test]
    fn test_collect_existing_paths_empty_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let existing = super::collect_existing_paths(&claude_dir);
        assert!(
            existing.is_empty(),
            "fresh directory should have no existing managed files"
        );
    }

    #[test]
    fn test_collect_existing_paths_partial_install() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(agents_dir.join("davinci.md"), "test").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert_eq!(existing.len(), 1);
        assert!(existing[0].ends_with("davinci.md"));
    }

    #[test]
    fn test_collect_existing_paths_full_install() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        let commands_dir = claude_dir.join("commands");
        let teams_dir = claude_dir.join("teams").join("loop");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::create_dir_all(&commands_dir).unwrap();
        std::fs::create_dir_all(&teams_dir).unwrap();

        for agent in super::AGENTS {
            std::fs::write(agents_dir.join(format!("{}.md", agent.name)), "test").unwrap();
        }
        for cmd in super::COMMANDS {
            std::fs::write(commands_dir.join(format!("{}.md", cmd.name)), "test").unwrap();
        }
        std::fs::write(teams_dir.join("config.json"), "{}").unwrap();

        // Create hook script path so collect_existing_paths detects it
        let hooks_dir = claude_dir.join("hooks").join("great-loop");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        std::fs::write(hooks_dir.join("update-state.sh"), "test").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert_eq!(
            existing.len(),
            22,
            "full install should detect all 22 managed files, got {}",
            existing.len()
        );
    }

    #[test]
    fn test_collect_existing_paths_only_commands() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let commands_dir = claude_dir.join("commands");
        std::fs::create_dir_all(&commands_dir).unwrap();

        std::fs::write(commands_dir.join("loop.md"), "test").unwrap();
        std::fs::write(commands_dir.join("bugfix.md"), "test").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert_eq!(existing.len(), 2);
    }

    #[test]
    fn test_collect_existing_paths_ignores_unknown_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(agents_dir.join("custom-agent.md"), "user content").unwrap();

        let existing = super::collect_existing_paths(&claude_dir);
        assert!(
            existing.is_empty(),
            "non-managed files should not be detected"
        );
    }

    #[test]
    fn test_install_variant_has_force_flag() {
        use clap::Parser;

        #[derive(clap::Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: super::LoopCommand,
        }

        let cli = TestCli::parse_from(["test", "install", "--force"]);
        match cli.cmd {
            super::LoopCommand::Install { force, project } => {
                assert!(force, "--force should be true");
                assert!(!project, "--project should default to false");
            }
            _ => panic!("expected Install variant"),
        }

        let cli2 = TestCli::parse_from(["test", "install"]);
        match cli2.cmd {
            super::LoopCommand::Install { force, .. } => {
                assert!(!force, "--force should default to false");
            }
            _ => panic!("expected Install variant"),
        }
    }

    #[test]
    fn test_hooks_value_has_all_events() {
        let hooks = super::hooks_value();
        let obj = hooks.as_object().expect("hooks_value must be an object");
        let expected = [
            "SubagentStart",
            "SubagentStop",
            "TeammateIdle",
            "TaskCompleted",
            "Stop",
            "SessionEnd",
        ];
        for event in &expected {
            assert!(obj.contains_key(*event), "missing event: {}", event);
            let arr = obj[*event].as_array().expect("event value must be array");
            assert!(!arr.is_empty(), "event {} must have entries", event);
        }
    }

    #[test]
    fn test_hooks_value_entries_have_async() {
        let hooks = super::hooks_value();
        for (event, matchers) in hooks.as_object().unwrap() {
            for matcher in matchers.as_array().unwrap() {
                for hook in matcher["hooks"].as_array().unwrap() {
                    assert_eq!(
                        hook["async"].as_bool(),
                        Some(true),
                        "hook for {} must be async",
                        event
                    );
                }
            }
        }
    }

    #[test]
    fn test_is_great_loop_hook_positive() {
        let entry = serde_json::json!({
            "matcher": "",
            "hooks": [{"type": "command", "command": "~/.claude/hooks/great-loop/update-state.sh"}]
        });
        assert!(super::is_great_loop_hook(&entry));
    }

    #[test]
    fn test_is_great_loop_hook_negative() {
        let entry = serde_json::json!({
            "matcher": "",
            "hooks": [{"type": "command", "command": "/usr/local/bin/my-hook.sh"}]
        });
        assert!(!super::is_great_loop_hook(&entry));
    }

    #[test]
    fn test_settings_merge_idempotent() {
        // Simulate a settings.json that already has great-loop hooks
        let mut settings = serde_json::json!({
            "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" },
            "hooks": super::hooks_value(),
            "statusLine": super::statusline_value(),
            "alwaysThinkingEnabled": true
        });

        let before = serde_json::to_string_pretty(&settings).unwrap();

        // Simulate the merge logic
        let desired = super::hooks_value();
        if let Some(hooks_map) = settings["hooks"].as_object_mut() {
            if let Some(desired_map) = desired.as_object() {
                for (event, desired_matchers) in desired_map {
                    if let Some(arr) =
                        hooks_map.get_mut(event).and_then(|v| v.as_array_mut())
                    {
                        arr.retain(|e| !super::is_great_loop_hook(e));
                        if let Some(new) = desired_matchers.as_array() {
                            arr.extend(new.iter().cloned());
                        }
                    }
                }
            }
        }

        let after = serde_json::to_string_pretty(&settings).unwrap();
        assert_eq!(before, after, "merge must be idempotent");
    }

    #[test]
    fn test_settings_merge_preserves_user_hooks() {
        let mut settings = serde_json::json!({
            "hooks": {
                "SubagentStart": [
                    {
                        "matcher": "",
                        "hooks": [{"type": "command", "command": "/usr/local/bin/user-hook.sh"}]
                    }
                ]
            }
        });

        let desired = super::hooks_value();
        if let Some(hooks_map) = settings["hooks"].as_object_mut() {
            if let Some(desired_map) = desired.as_object() {
                for (event, desired_matchers) in desired_map {
                    if let Some(arr) =
                        hooks_map.get_mut(event).and_then(|v| v.as_array_mut())
                    {
                        arr.retain(|e| !super::is_great_loop_hook(e));
                        if let Some(new) = desired_matchers.as_array() {
                            arr.extend(new.iter().cloned());
                        }
                    } else {
                        hooks_map.insert(event.clone(), desired_matchers.clone());
                    }
                }
            }
        }

        // User hook must still be present
        let sa = settings["hooks"]["SubagentStart"]
            .as_array()
            .unwrap();
        assert_eq!(sa.len(), 2, "user hook + great-loop hook");
        assert!(sa[0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("user-hook.sh"));
        assert!(sa[1]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("great-loop"));
    }
}
