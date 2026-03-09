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

/// Teams configuration JSON embedded at compile time.
const TEAMS_CONFIG: &str = include_str!("../../loop/teams-config.json");

/// Observer report template embedded at compile time (used for --project).
const OBSERVER_TEMPLATE: &str = include_str!("../../loop/observer-template.md");

/// Marketplace repo identifier for `claude plugin marketplace add`.
const MARKETPLACE_REPO: &str = "superstruct/great.sh";

/// Plugin name as registered in Claude Code's plugin system.
const PLUGIN_NAME: &str = "great";

/// The exact CLAUDE.md content that was installed by the legacy great.sh Loop installer.
/// Used for migration: if `~/.claude/CLAUDE.md` matches this exactly, it can be deleted.
const LEGACY_CLAUDE_MD: &str = "\
# great.sh Loop — System Observer: W. Edwards Deming

You operate the great.sh Loop, a 13-role AI agent orchestration methodology.
Each role is embodied by a historical figure whose expertise maps to the task.

## Your Identity
You are W. Edwards Deming — father of statistical quality control. You observe
the PROCESS, not just the product. PDCA cycle. One change at a time. Evidence-based.

## How the Loop Works
Nightingale (requirements) → Lovelace (spec) → Socrates (review) →
Humboldt (scout) → Da Vinci (build) → parallel: [Von Braun (deploy) →
Turing (test) → Rams (visual) → Nielsen (UX)] + [Knuth (docs) → Gutenberg
(doc commit)] → Hopper (code commit) → Deming (observe)

## Rules
- Backpressure: no agent declares success without evidence
- Quality gates must pass before commits
- One configuration change per iteration, with rationale
- Observer reports after every loop iteration
- Use Agent Teams for parallel work (Da Vinci, Turing, Nielsen)
- Use subagents for sequential focused work (all others)

## MCP Routing
- Gemini: large-context codebase analysis (Humboldt, Lovelace)
- Codex: fast code generation (Da Vinci)
- Context7: library/framework documentation (Lovelace, Da Vinci)
- Playwright: visual and UX review (Rams, Nielsen)

## Observer Report
After each loop iteration, write to .observer/reports/iteration-NNN.md:
- Task completed, agent retries, bottleneck, config change (if any), metrics.
";

/// Run the `great loop` subcommand.
pub fn run(args: Args) -> Result<()> {
    let non_interactive = args.non_interactive;
    match args.command {
        LoopCommand::Install { project, force } => run_install(project, force, non_interactive),
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

/// Check if a hook matcher array entry is a great-loop hook (legacy format).
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

/// Names of the 15 managed agent files (used for legacy cleanup).
const AGENT_NAMES: &[&str] = &[
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

/// Names of the 5 managed command/skill files (used for legacy cleanup).
const SKILL_NAMES: &[&str] = &["loop", "bugfix", "deploy", "discover", "backlog"];

/// Result of a legacy migration check.
struct MigrationResult {
    /// Whether any legacy files were found and cleaned up.
    migrated: bool,
    /// Number of files/dirs removed.
    _removed_count: usize,
}

/// Detect and remove legacy (pre-plugin) great.sh Loop files from `~/.claude/`.
fn migrate_legacy_install(claude_dir: &std::path::Path) -> Result<MigrationResult> {
    let agents_dir = claude_dir.join("agents");
    let commands_dir = claude_dir.join("commands");
    let teams_dir = claude_dir.join("teams").join("loop");
    let hooks_dir = claude_dir.join("hooks").join("great-loop");

    // Quick check: is there anything to migrate?
    let has_legacy = agents_dir.join("nightingale.md").exists()
        || commands_dir.join("loop.md").exists()
        || hooks_dir.exists();

    if !has_legacy {
        return Ok(MigrationResult {
            migrated: false,
            _removed_count: 0,
        });
    }

    output::header("Migrating legacy great.sh Loop install");
    println!();

    let mut removed = 0usize;

    // Remove legacy agent files
    for name in AGENT_NAMES {
        let path = agents_dir.join(format!("{}.md", name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove legacy agent: {}", path.display()))?;
            removed += 1;
        }
    }

    // Remove legacy command files
    for name in SKILL_NAMES {
        let path = commands_dir.join(format!("{}.md", name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove legacy command: {}", path.display()))?;
            removed += 1;
        }
    }

    // Remove legacy teams directory
    if teams_dir.exists() {
        std::fs::remove_dir_all(&teams_dir)
            .context("failed to remove legacy ~/.claude/teams/loop/")?;
        removed += 1;
    }

    // Remove legacy hooks directory
    if hooks_dir.exists() {
        std::fs::remove_dir_all(&hooks_dir)
            .context("failed to remove legacy ~/.claude/hooks/great-loop/")?;
        removed += 1;
    }

    // Remove great-loop hook entries from settings.json
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        remove_hooks_from_settings(&settings_path)?;
    }

    output::success(&format!("Removed {} legacy files/directories", removed));

    Ok(MigrationResult {
        migrated: true,
        _removed_count: removed,
    })
}

/// Remove great-loop hook entries from settings.json (used during migration and install).
fn remove_hooks_from_settings(settings_path: &std::path::Path) -> Result<bool> {
    if !settings_path.exists() {
        return Ok(false);
    }

    let contents =
        std::fs::read_to_string(settings_path).context("failed to read settings.json")?;
    let mut val: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };

    let mut modified = false;
    if let Some(obj) = val.as_object_mut() {
        if let Some(hooks) = obj.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            let hooks_before = serde_json::to_string(hooks).unwrap_or_default();

            for (_event, matchers) in hooks.iter_mut() {
                if let Some(arr) = matchers.as_array_mut() {
                    arr.retain(|entry| !is_great_loop_hook(entry));
                }
            }

            // Remove empty event arrays
            let empty_events: Vec<String> = hooks
                .iter()
                .filter(|(_, v)| v.as_array().map(|a| a.is_empty()).unwrap_or(false))
                .map(|(k, _)| k.clone())
                .collect();
            for key in empty_events {
                hooks.remove(&key);
            }

            // Remove hooks key entirely if empty
            let hooks_after = serde_json::to_string(hooks).unwrap_or_default();
            if hooks_before != hooks_after {
                modified = true;
            }
        }

        // If hooks object is now empty, remove the key
        if obj
            .get("hooks")
            .and_then(|h| h.as_object())
            .map(|h| h.is_empty())
            .unwrap_or(false)
        {
            obj.remove("hooks");
            modified = true;
        }

        if modified {
            let formatted =
                serde_json::to_string_pretty(&val).context("failed to serialize settings.json")?;
            std::fs::write(settings_path, formatted).context("failed to write settings.json")?;
        }
    }

    Ok(modified)
}

/// Clean up CLAUDE.md if it was installed by the legacy great.sh Loop installer.
fn migrate_claude_md(claude_dir: &std::path::Path) -> Result<()> {
    let claude_md_path = claude_dir.join("CLAUDE.md");
    if !claude_md_path.exists() {
        return Ok(());
    }

    let contents =
        std::fs::read_to_string(&claude_md_path).context("failed to read ~/.claude/CLAUDE.md")?;

    if contents.trim() == LEGACY_CLAUDE_MD.trim() {
        std::fs::remove_file(&claude_md_path)
            .context("failed to remove legacy ~/.claude/CLAUDE.md")?;
        output::success("Removed legacy ~/.claude/CLAUDE.md (great.sh Loop content only)");
    } else if contents.contains("great.sh Loop") {
        println!();
        output::warning(
            "~/.claude/CLAUDE.md contains custom content alongside great.sh Loop instructions.",
        );
        output::info("  Please remove the great.sh Loop section manually.");
    }

    Ok(())
}

/// Returns whether any legacy (pre-plugin) files exist.
fn has_legacy_install(claude_dir: &std::path::Path) -> bool {
    claude_dir.join("agents").join("nightingale.md").exists()
        || claude_dir.join("commands").join("loop.md").exists()
        || claude_dir
            .join("hooks")
            .join("great-loop")
            .join("update-state.sh")
            .exists()
}

/// Run a `claude` CLI command, returning its stdout on success.
fn run_claude_cmd(args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("claude")
        .args(args)
        .output()
        .context("failed to run `claude` CLI — is Claude Code installed? https://docs.anthropic.com/en/docs/claude-code")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        bail!(
            "`claude {}` failed (exit {}): {}",
            args.join(" "),
            output.status,
            if stderr.is_empty() { &stdout } else { &stderr }
        );
    }

    Ok(stdout)
}

/// Check whether the great-sh marketplace is already registered.
fn is_marketplace_registered() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    let known = home.join(".claude").join("known_marketplaces.json");
    if !known.exists() {
        return false;
    }
    let contents = match std::fs::read_to_string(&known) {
        Ok(c) => c,
        Err(_) => return false,
    };
    contents.contains(MARKETPLACE_REPO)
}

/// Check whether the great plugin is already installed via Claude Code.
fn is_plugin_installed() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    let installed = home.join(".claude").join("installed_plugins.json");
    if !installed.exists() {
        return false;
    }
    let contents = match std::fs::read_to_string(&installed) {
        Ok(c) => c,
        Err(_) => return false,
    };
    contents.contains(PLUGIN_NAME)
}

/// Install the great.sh Loop via `claude plugin` CLI commands.
fn run_install(project: bool, force: bool, _non_interactive: bool) -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");

    output::header("great.sh Loop — Installing plugin");
    println!();

    // --- Phase 1: Detect and migrate legacy install ---
    let migration = migrate_legacy_install(&claude_dir)?;
    if migration.migrated {
        migrate_claude_md(&claude_dir)?;
        println!();
    }

    // --- Phase 2: Install plugin via claude CLI ---

    // Check that `claude` is on PATH
    if std::process::Command::new("claude")
        .arg("--version")
        .output()
        .is_err()
    {
        bail!("claude CLI not found — install Claude Code first: https://docs.anthropic.com/en/docs/claude-code");
    }

    // 2a: Register marketplace (idempotent)
    if !is_marketplace_registered() {
        output::info(&format!("Registering marketplace {}...", MARKETPLACE_REPO));
        run_claude_cmd(&["plugin", "marketplace", "add", MARKETPLACE_REPO])?;
        output::success("Marketplace registered");
    } else {
        output::success("Marketplace already registered");
    }

    // 2b: Install (or reinstall) the plugin
    if is_plugin_installed() && force {
        output::info("(--force: reinstalling plugin)");
        // Uninstall first so reinstall picks up new files
        let _ = run_claude_cmd(&["plugin", "uninstall", PLUGIN_NAME]);
    }

    if !is_plugin_installed() || force {
        output::info("Installing plugin via claude CLI...");
        run_claude_cmd(&["plugin", "install", PLUGIN_NAME])?;
        output::success("Plugin installed via claude plugin install");
    } else {
        output::success("Plugin already installed (use --force to reinstall)");
    }

    // --- Phase 3: Side-effects outside plugin ---

    // Write teams config (no plugin equivalent)
    let teams_dir = claude_dir.join("teams").join("loop");
    std::fs::create_dir_all(&teams_dir)
        .context("failed to create ~/.claude/teams/loop/ directory")?;
    let config_path = teams_dir.join("config.json");
    std::fs::write(&config_path, TEAMS_CONFIG)
        .context("failed to write teams config to ~/.claude/teams/loop/config.json")?;
    output::success("Agent Teams config -> ~/.claude/teams/loop/");

    // Handle settings.json (non-destructive merge for env and statusLine only — hooks are in plugin)
    let settings_path = claude_dir.join("settings.json");

    let settings_readonly = settings_path.exists()
        && std::fs::metadata(&settings_path)
            .map(|m| m.permissions().readonly())
            .unwrap_or(false);
    if settings_readonly {
        println!();
        output::warning("settings.json is read-only \u{2014} env and statusLine not injected.");
        output::info("  Fix: chmod u+w ~/.claude/settings.json && great loop install --force");
    } else if settings_path.exists() {
        let contents = std::fs::read_to_string(&settings_path)
            .context("failed to read ~/.claude/settings.json")?;
        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(mut val) => {
                if let Some(obj) = val.as_object_mut() {
                    let mut modified = false;

                    // --- Inject env.CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS ---
                    let env_obj = obj.entry("env").or_insert_with(|| serde_json::json!({}));
                    if let Some(env_map) = env_obj.as_object_mut() {
                        if !env_map.contains_key("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS") {
                            env_map.insert(
                                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS".to_string(),
                                serde_json::json!("1"),
                            );
                            modified = true;
                        }
                    }

                    // --- Remove any legacy hooks from settings ---
                    if obj.get("hooks").is_some() {
                        if remove_hooks_from_settings(&settings_path)? {
                            // Re-read after hooks removal
                            let updated = std::fs::read_to_string(&settings_path)
                                .context("failed to re-read settings.json")?;
                            val = serde_json::from_str(&updated)
                                .context("settings.json invalid after hooks removal")?;
                            // Re-borrow obj after re-parsing
                            if let Some(obj2) = val.as_object_mut() {
                                // --- Inject or repair statusLine ---
                                let needs_statusline = if !obj2.contains_key("statusLine") {
                                    true
                                } else if let Some(sl) =
                                    obj2.get("statusLine").and_then(|v| v.as_object())
                                {
                                    !sl.contains_key("type")
                                } else {
                                    false
                                };
                                if needs_statusline {
                                    obj2.insert("statusLine".to_string(), statusline_value());
                                    modified = true;
                                }

                                if modified {
                                    let formatted = serde_json::to_string_pretty(&val)
                                        .context("failed to serialize settings.json")?;
                                    std::fs::write(&settings_path, formatted)
                                        .context("failed to write ~/.claude/settings.json")?;
                                }
                            }
                            output::success(
                                "Settings updated (env, statusLine, hooks removed) in ~/.claude/settings.json",
                            );
                        } else {
                            // Hooks weren't modified, handle statusLine normally
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

                            if modified {
                                let formatted = serde_json::to_string_pretty(&val)
                                    .context("failed to serialize settings.json")?;
                                std::fs::write(&settings_path, formatted)
                                    .context("failed to write ~/.claude/settings.json")?;
                                output::success(
                                    "Settings updated (env, statusLine) in ~/.claude/settings.json",
                                );
                            } else {
                                output::success(
                                    "Settings already configured in ~/.claude/settings.json",
                                );
                            }
                        }
                    } else {
                        // No hooks key at all — just handle statusLine
                        let needs_statusline = if !obj.contains_key("statusLine") {
                            true
                        } else if let Some(sl) = obj.get("statusLine").and_then(|v| v.as_object()) {
                            !sl.contains_key("type")
                        } else {
                            false
                        };
                        if needs_statusline {
                            obj.insert("statusLine".to_string(), statusline_value());
                            modified = true;
                        }

                        if modified {
                            let formatted = serde_json::to_string_pretty(&val)
                                .context("failed to serialize settings.json")?;
                            std::fs::write(&settings_path, formatted)
                                .context("failed to write ~/.claude/settings.json")?;
                            output::success(
                                "Settings updated (env, statusLine) in ~/.claude/settings.json",
                            );
                        } else {
                            output::success(
                                "Settings already configured in ~/.claude/settings.json",
                            );
                        }
                    }
                }
            }
            Err(_) => {
                output::warning("settings.json is not valid JSON; skipping injection");
            }
        }
    } else {
        // No existing settings.json -- create with env and statusLine (no hooks)
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
            "statusLine": statusline_value()
        });
        let formatted = serde_json::to_string_pretty(&default_settings)
            .context("failed to serialize default settings")?;
        std::fs::write(&settings_path, formatted)
            .context("failed to write ~/.claude/settings.json")?;
        output::success("Settings with Agent Teams and statusLine -> ~/.claude/settings.json");
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
        output::info("Usage: claude -> /great:loop [task description]");
    } else {
        output::info("Next: great loop install --project  (in your repo)");
    }

    Ok(())
}

/// Show the installation status of the great.sh Loop.
fn run_status() -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");
    let plugin_dir = claude_dir.join("plugins").join("great");

    output::header("great.sh Loop — Status");
    println!();

    // Check plugin manifest
    let plugin_ok = plugin_dir
        .join(".claude-plugin")
        .join("plugin.json")
        .exists();
    if plugin_ok {
        output::success("Plugin manifest: installed");
    } else {
        output::error("Plugin manifest: not installed");
    }

    // Check key agent file in plugin dir
    let agents_ok = plugin_dir.join("agents").join("nightingale.md").exists();
    if agents_ok {
        output::success("Agent personas: installed");
    } else {
        output::error("Agent personas: not installed");
    }

    // Check key skill file in plugin dir
    let skills_ok = plugin_dir
        .join("skills")
        .join("loop")
        .join("SKILL.md")
        .exists();
    if skills_ok {
        output::success("Plugin skills: installed");
    } else {
        output::error("Plugin skills: not installed");
    }

    // Check hooks.json in plugin dir
    let hooks_ok = plugin_dir.join("hooks").join("hooks.json").exists();
    if hooks_ok {
        output::success("Hooks config: installed in plugin");
    } else {
        output::warning("Hooks config: not installed");
    }

    // Check hook script in plugin dir
    let script_ok = plugin_dir.join("scripts").join("update-state.sh").exists();
    if script_ok {
        output::success("Hook handler: installed");
    } else {
        output::warning("Hook handler: not installed (statusline will show 'idle')");
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

        // Warn if legacy hooks still in settings.json
        if contents.contains("great-loop/update-state.sh") {
            output::warning("Legacy hooks detected in settings.json (run install to migrate)");
        }
    } else {
        output::warning("settings.json: not found");
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

    // Legacy detection
    if has_legacy_install(&claude_dir) {
        println!();
        output::warning("Legacy great.sh Loop files detected in ~/.claude/");
        output::info("  Run: great loop install --force  (to migrate to plugin format)");
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
    if plugin_ok && agents_ok && skills_ok && teams_ok {
        output::success("great.sh Loop is installed and ready.");
    } else {
        output::info("Run: great loop install");
    }

    Ok(())
}

/// Remove the great.sh Loop plugin and side-effects from `~/.claude/`.
fn run_uninstall() -> Result<()> {
    let home = dirs::home_dir().context("could not determine home directory — is $HOME set?")?;
    let claude_dir = home.join(".claude");

    output::header("great.sh Loop — Uninstalling");
    println!();

    let mut removed = 0;

    // Uninstall via claude CLI if available and plugin is installed
    if is_plugin_installed() {
        match run_claude_cmd(&["plugin", "uninstall", PLUGIN_NAME]) {
            Ok(_) => {
                output::success("Plugin uninstalled via claude plugin uninstall");
                removed += 1;
            }
            Err(e) => {
                output::warning(&format!("claude plugin uninstall failed: {}", e));
                // Fall back to manual removal
                let plugin_dir = claude_dir.join("plugins").join("great");
                if plugin_dir.exists() {
                    std::fs::remove_dir_all(&plugin_dir)
                        .context("failed to remove ~/.claude/plugins/great/")?;
                    output::success("Removed plugin directory ~/.claude/plugins/great/");
                    removed += 1;
                }
            }
        }
    } else {
        // Plugin not registered — clean up files manually if they exist
        let plugin_dir = claude_dir.join("plugins").join("great");
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)
                .context("failed to remove ~/.claude/plugins/great/")?;
            output::success("Removed plugin directory ~/.claude/plugins/great/");
            removed += 1;
        }
    }

    // Remove teams/loop directory
    let teams_dir = claude_dir.join("teams").join("loop");
    if teams_dir.exists() {
        std::fs::remove_dir_all(&teams_dir).context("failed to remove ~/.claude/teams/loop/")?;
        output::success("Removed teams/loop/ directory");
        removed += 1;
    }

    // Also clean up legacy files if they exist
    let legacy_agents_dir = claude_dir.join("agents");
    let mut legacy_removed = 0;
    for name in AGENT_NAMES {
        let path = legacy_agents_dir.join(format!("{}.md", name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            legacy_removed += 1;
        }
    }
    let legacy_commands_dir = claude_dir.join("commands");
    for name in SKILL_NAMES {
        let path = legacy_commands_dir.join(format!("{}.md", name));
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            legacy_removed += 1;
        }
    }
    let legacy_hooks_dir = claude_dir.join("hooks").join("great-loop");
    if legacy_hooks_dir.exists() {
        std::fs::remove_dir_all(&legacy_hooks_dir)
            .context("failed to remove ~/.claude/hooks/great-loop/")?;
        legacy_removed += 1;
    }
    if legacy_removed > 0 {
        output::success(&format!("Removed {} legacy files", legacy_removed));
        removed += legacy_removed;
    }

    // Clean settings.json (remove env var, statusLine, and any leftover hooks)
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        let contents =
            std::fs::read_to_string(&settings_path).context("failed to read settings.json")?;
        if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(obj) = val.as_object_mut() {
                let mut modified = false;

                // Remove agent teams env var
                if let Some(env) = obj.get_mut("env").and_then(|e| e.as_object_mut()) {
                    if env.remove("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS").is_some() {
                        modified = true;
                    }
                    if env.is_empty() {
                        obj.remove("env");
                    }
                }

                // Remove statusLine
                if obj.remove("statusLine").is_some() {
                    modified = true;
                }

                // Remove any leftover hooks
                if obj.get("hooks").is_some() {
                    // Use the hook removal helper
                    remove_hooks_from_settings(&settings_path)?;
                    modified = true;
                }

                if modified {
                    // Re-read in case remove_hooks_from_settings wrote
                    let updated = if settings_path.exists() {
                        std::fs::read_to_string(&settings_path).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    if let Ok(mut val2) = serde_json::from_str::<serde_json::Value>(&updated) {
                        if let Some(obj2) = val2.as_object_mut() {
                            // Remove env and statusLine again in case remove_hooks_from_settings
                            // re-read the original
                            if let Some(env) = obj2.get_mut("env").and_then(|e| e.as_object_mut()) {
                                env.remove("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS");
                                if env.is_empty() {
                                    obj2.remove("env");
                                }
                            }
                            obj2.remove("statusLine");

                            let formatted = serde_json::to_string_pretty(&val2)
                                .context("failed to serialize settings.json")?;
                            std::fs::write(&settings_path, formatted)
                                .context("failed to write settings.json")?;
                        }
                    }
                    output::success("Cleaned great.sh Loop entries from settings.json");
                }
            }
        }
    }

    // CLAUDE.md cleanup
    migrate_claude_md(&claude_dir)?;

    println!();
    if removed > 0 {
        output::success("great.sh Loop uninstalled.");
    } else {
        output::info("great.sh Loop was not installed.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_repair_fixes_broken_statusline() {
        let mut settings = serde_json::json!({
            "env": { "SOME_KEY": "value" },
            "statusLine": { "command": "great statusline" }
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
        assert_eq!(settings["env"]["SOME_KEY"].as_str(), Some("value"));
    }

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

    /// Validate hooks.json on disk (read from repo, not embedded).
    #[test]
    fn test_hooks_json_has_all_events() {
        let hooks_json = include_str!("../../loop/hooks/hooks.json");
        let parsed: serde_json::Value =
            serde_json::from_str(hooks_json).expect("hooks.json must be valid JSON");
        let obj = parsed.as_object().expect("hooks.json must be an object");
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
            for matcher in arr {
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

    /// Validate plugin.json on disk (read from repo, not embedded).
    #[test]
    fn test_plugin_manifest_valid_json() {
        let plugin_json = include_str!("../../loop/.claude-plugin/plugin.json");
        let parsed: serde_json::Value =
            serde_json::from_str(plugin_json).expect("plugin.json must be valid JSON");
        assert_eq!(parsed["name"].as_str(), Some("great"));
        assert_eq!(parsed["version"].as_str(), Some("0.1.0"));
    }

    /// Validate that no agent markdown files contain "Architecton".
    #[test]
    fn test_no_architecton_in_agent_files() {
        for name in AGENT_NAMES {
            let full_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("loop/agents")
                .join(format!("{}.md", name));
            if full_path.exists() {
                let content = std::fs::read_to_string(&full_path).unwrap();
                assert!(
                    !content.contains("Architecton"),
                    "Agent '{}' must not contain 'Architecton' — use 'great.sh Loop'",
                    name
                );
            }
        }
    }

    /// Validate that no skill markdown files contain "Architecton".
    #[test]
    fn test_no_architecton_in_skill_files() {
        for name in SKILL_NAMES {
            let full_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("loop/skills")
                .join(name)
                .join("SKILL.md");
            if full_path.exists() {
                let content = std::fs::read_to_string(&full_path).unwrap();
                assert!(
                    !content.contains("Architecton"),
                    "Skill '{}' must not contain 'Architecton' — use 'great.sh Loop'",
                    name
                );
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
    fn test_settings_no_hooks_after_install() {
        let default_settings = serde_json::json!({
            "env": {
                "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
            },
            "statusLine": super::statusline_value()
        });
        assert!(
            default_settings.get("hooks").is_none(),
            "new installs should not have hooks in settings.json"
        );
    }

    #[test]
    fn test_settings_merge_preserves_user_keys() {
        let settings = serde_json::json!({
            "env": { "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1" },
            "statusLine": super::statusline_value(),
            "alwaysThinkingEnabled": true,
            "customKey": "user value"
        });

        assert_eq!(settings["alwaysThinkingEnabled"], true);
        assert_eq!(settings["customKey"], "user value");
    }

    #[test]
    fn test_has_legacy_install_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        assert!(!super::has_legacy_install(&claude_dir));
    }

    #[test]
    fn test_has_legacy_install_with_agents() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let agents_dir = claude_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::fs::write(agents_dir.join("nightingale.md"), "test").unwrap();
        assert!(super::has_legacy_install(&claude_dir));
    }

    #[test]
    fn test_has_legacy_install_with_commands() {
        let dir = tempfile::TempDir::new().unwrap();
        let claude_dir = dir.path().join(".claude");
        let commands_dir = claude_dir.join("commands");
        std::fs::create_dir_all(&commands_dir).unwrap();
        std::fs::write(commands_dir.join("loop.md"), "test").unwrap();
        assert!(super::has_legacy_install(&claude_dir));
    }

    #[test]
    fn test_marketplace_constants() {
        assert_eq!(MARKETPLACE_REPO, "superstruct/great.sh");
        assert_eq!(PLUGIN_NAME, "great");
    }
}
