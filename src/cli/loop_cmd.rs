use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;

/// Arguments for the `loop` subcommand.
#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: LoopCommand,
}

/// Subcommands for managing the great.sh Loop agent team.
#[derive(Subcommand)]
pub enum LoopCommand {
    /// Install the great.sh Loop agent team to ~/.claude/
    Install {
        /// Also set up .tasks/ working state in current directory
        #[arg(long)]
        project: bool,
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

/// All 4 slash-command files shipped with the great.sh Loop.
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

/// Run the `great loop` subcommand.
pub fn run(args: Args) -> Result<()> {
    match args.command {
        LoopCommand::Install { project } => run_install(project),
        LoopCommand::Status => run_status(),
        LoopCommand::Uninstall => run_uninstall(),
    }
}

/// Install the great.sh Loop agent team to `~/.claude/`.
fn run_install(project: bool) -> Result<()> {
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

    // Write 4 command files
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

    // Handle settings.json (non-destructive)
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json")?;
        if !contents.contains("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS") {
            println!();
            output::warning("Add to your ~/.claude/settings.json:");
            output::info("  \"env\": { \"CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS\": \"1\" }");
        }
    } else {
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
            "statusLine": {
                "command": "great statusline"
            }
        });
        let formatted = serde_json::to_string_pretty(&default_settings)
            .context("failed to serialize default settings")?;
        std::fs::write(&settings_path, formatted)
            .context("failed to write ~/.claude/settings.json")?;
        output::success("Settings with Agent Teams enabled -> ~/.claude/settings.json");
    }

    // Inject statusLine key into existing settings.json if not already present
    if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json for statusLine injection")?;
        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(mut val) => {
                if let Some(obj) = val.as_object_mut() {
                    if !obj.contains_key("statusLine") {
                        obj.insert(
                            "statusLine".to_string(),
                            serde_json::json!({"command": "great statusline"}),
                        );
                        let formatted = serde_json::to_string_pretty(&val)
                            .context("failed to serialize settings.json")?;
                        std::fs::write(&settings_path, formatted)
                            .context("failed to write ~/.claude/settings.json")?;
                        output::success("Statusline registered in ~/.claude/settings.json");
                    }
                }
            }
            Err(_) => {
                output::warning(
                    "settings.json is not valid JSON; skipping statusLine injection",
                );
            }
        }
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
}
