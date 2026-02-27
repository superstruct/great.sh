use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};

use crate::cli::output;
use crate::config;

#[derive(ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: TemplateCommand,
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,
    /// Apply a template to the current project
    Apply {
        /// Template name
        name: String,
    },
    /// Update templates from registry
    Update,
}

/// Built-in template definitions.
struct Template {
    name: &'static str,
    description: &'static str,
    content: &'static str,
}

/// Built-in templates.
///
/// Each template can declare its own `[tools.cli]` entries — `great apply`
/// installs them alongside any tools the user has added manually. This is
/// how domain-specific templates (e.g. SaaS requiring hasura-cli) pull in
/// the CLI tools they need without the user having to know upfront.
fn builtin_templates() -> Vec<Template> {
    vec![
        Template {
            name: "ai-fullstack-ts",
            description: "Full-stack TypeScript with AI agents and MCP servers",
            content: include_str!("../../templates/ai-fullstack-ts.toml"),
        },
        Template {
            name: "ai-fullstack-py",
            description: "Full-stack Python with AI agents and MCP servers",
            content: include_str!("../../templates/ai-fullstack-py.toml"),
        },
        Template {
            name: "ai-minimal",
            description: "Minimal AI setup with Claude only",
            content: include_str!("../../templates/ai-minimal.toml"),
        },
        Template {
            name: "saas-multi-tenant",
            description: "Multi-tenant SaaS with Hasura, AWS CDK, and AI agents",
            content: include_str!("../../templates/saas-multi-tenant.toml"),
        },
    ]
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        TemplateCommand::List => run_list(),
        TemplateCommand::Apply { name } => run_apply(&name),
        TemplateCommand::Update => run_update(),
    }
}

fn run_list() -> Result<()> {
    output::header("Available Templates");
    println!();

    // Show built-in templates
    output::info("Built-in:");
    for tmpl in builtin_templates() {
        output::info(&format!("  {} — {}", tmpl.name, tmpl.description));
    }

    // Show downloaded templates
    let downloaded = list_downloaded_templates();
    if !downloaded.is_empty() {
        println!();
        output::info("Downloaded:");
        for name in &downloaded {
            output::info(&format!("  {}", name));
        }
    }

    println!();
    output::info("Apply with: great template apply <name>");
    output::info("Update with: great template update");
    Ok(())
}

/// List template names from the downloaded templates directory.
fn list_downloaded_templates() -> Vec<String> {
    let dir = match template_download_dir() {
        Some(d) if d.exists() => d,
        _ => return Vec::new(),
    };

    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    names.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// Return the template download directory (~/.local/share/great/templates/).
fn template_download_dir() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|d| d.join("great").join("templates"))
}

fn run_apply(name: &str) -> Result<()> {
    // Try built-in templates first
    let templates = builtin_templates();
    let template_content = if let Some(tmpl) = templates.iter().find(|t| t.name == name) {
        tmpl.content.to_string()
    } else {
        // Try downloaded templates
        match load_downloaded_template(name) {
            Some(content) => content,
            None => {
                output::error(&format!("Unknown template: {}", name));
                output::info("Available templates:");
                for t in &templates {
                    output::info(&format!("  {}", t.name));
                }
                let downloaded = list_downloaded_templates();
                for d in &downloaded {
                    output::info(&format!("  {} (downloaded)", d));
                }
                return Ok(());
            }
        }
    };

    let config_path = std::path::Path::new("great.toml");

    if config_path.exists() {
        // Merge with existing config
        output::info("Existing great.toml found — merging template.");

        let existing = config::load(Some("great.toml"))?;
        let template_config: crate::config::schema::GreatConfig = toml::from_str(&template_content)
            .context(format!("failed to parse template '{}'", name))?;

        let merged = merge_configs(existing, template_config);

        let toml_string =
            toml::to_string_pretty(&merged).context("failed to serialize merged config")?;
        std::fs::write(config_path, toml_string).context("failed to write great.toml")?;

        output::success(&format!("Merged template '{}' into great.toml", name));
    } else {
        std::fs::write(config_path, &template_content).context("failed to write great.toml")?;
        output::success(&format!("Created great.toml from template '{}'", name));
    }

    output::info("Run `great apply` to provision your environment.");
    Ok(())
}

/// Try to load a template from the downloaded templates directory.
fn load_downloaded_template(name: &str) -> Option<String> {
    let dir = template_download_dir()?;
    let path = dir.join(format!("{}.toml", name));
    std::fs::read_to_string(path).ok()
}

/// Fetch latest templates from the GitHub repository.
fn run_update() -> Result<()> {
    output::header("Updating templates");
    println!();

    let rt = tokio::runtime::Runtime::new().context("failed to create async runtime")?;
    rt.block_on(fetch_templates_from_github())
}

/// Download template files from the superstruct/great.sh GitHub repository.
async fn fetch_templates_from_github() -> Result<()> {
    let api_url = "https://api.github.com/repos/superstruct/great.sh/contents/templates";

    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .header("User-Agent", "great-sh")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("failed to reach GitHub API")?;

    if !response.status().is_success() {
        output::warning("Could not fetch templates from GitHub.");
        output::info("Built-in templates are still available.");
        output::info("Update great itself for new built-in templates: great update");
        return Ok(());
    }

    let entries: Vec<serde_json::Value> = response
        .json()
        .await
        .context("failed to parse GitHub API response")?;

    let toml_files: Vec<_> = entries
        .iter()
        .filter(|e| {
            e["name"]
                .as_str()
                .map(|n| n.ends_with(".toml"))
                .unwrap_or(false)
        })
        .collect();

    if toml_files.is_empty() {
        output::info("No templates found in repository.");
        return Ok(());
    }

    // Ensure download directory exists
    let dir = template_download_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine template directory"))?;
    std::fs::create_dir_all(&dir).context("failed to create template directory")?;

    let mut updated = 0;
    for entry in &toml_files {
        let name = match entry["name"].as_str() {
            Some(n) => n,
            None => continue,
        };
        let download_url = match entry["download_url"].as_str() {
            Some(u) => u,
            None => continue,
        };

        let resp = client
            .get(download_url)
            .header("User-Agent", "great-sh")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(content) = r.text().await {
                    let dest = dir.join(name);
                    if std::fs::write(&dest, &content).is_ok() {
                        output::success(&format!("  {} — updated", name));
                        updated += 1;
                    }
                }
            }
            _ => {
                output::warning(&format!("  {} — failed to download", name));
            }
        }
    }

    println!();
    output::info(&format!(
        "Updated {} templates to {}",
        updated,
        dir.display()
    ));

    Ok(())
}

/// Merge two configs: existing values take precedence, template fills gaps.
fn merge_configs(
    existing: crate::config::schema::GreatConfig,
    template: crate::config::schema::GreatConfig,
) -> crate::config::schema::GreatConfig {
    crate::config::schema::GreatConfig {
        project: existing.project.or(template.project),
        tools: match (existing.tools, template.tools) {
            (Some(e), Some(t)) => {
                let mut runtimes = t.runtimes;
                runtimes.extend(e.runtimes); // existing wins
                let cli = match (e.cli, t.cli) {
                    (Some(ec), Some(tc)) => {
                        let mut merged = tc;
                        merged.extend(ec);
                        Some(merged)
                    }
                    (e_cli, t_cli) => e_cli.or(t_cli),
                };
                Some(crate::config::schema::ToolsConfig { runtimes, cli })
            }
            (e, t) => e.or(t),
        },
        agents: match (existing.agents, template.agents) {
            (Some(mut e), Some(t)) => {
                for (k, v) in t {
                    e.entry(k).or_insert(v);
                }
                Some(e)
            }
            (e, t) => e.or(t),
        },
        mcp: match (existing.mcp, template.mcp) {
            (Some(mut e), Some(t)) => {
                for (k, v) in t {
                    e.entry(k).or_insert(v);
                }
                Some(e)
            }
            (e, t) => e.or(t),
        },
        secrets: existing.secrets.or(template.secrets),
        platform: existing.platform.or(template.platform),
        mcp_bridge: existing.mcp_bridge.or(template.mcp_bridge),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;
    use std::collections::HashMap;

    #[test]
    fn test_merge_both_empty() {
        let result = merge_configs(GreatConfig::default(), GreatConfig::default());
        assert!(result.project.is_none());
        assert!(result.tools.is_none());
        assert!(result.agents.is_none());
        assert!(result.mcp.is_none());
        assert!(result.secrets.is_none());
        assert!(result.platform.is_none());
    }

    #[test]
    fn test_merge_existing_takes_precedence_project() {
        let existing = GreatConfig {
            project: Some(ProjectConfig {
                name: Some("existing".into()),
                description: Some("existing desc".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let template = GreatConfig {
            project: Some(ProjectConfig {
                name: Some("template".into()),
                description: Some("template desc".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let proj = result.project.unwrap();
        assert_eq!(proj.name.as_deref(), Some("existing"));
        assert_eq!(proj.description.as_deref(), Some("existing desc"));
    }

    #[test]
    fn test_merge_template_fills_gap_project() {
        let existing = GreatConfig::default();
        let template = GreatConfig {
            project: Some(ProjectConfig {
                name: Some("from-template".into()),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let proj = result.project.unwrap();
        assert_eq!(proj.name.as_deref(), Some("from-template"));
    }

    #[test]
    fn test_merge_tools_runtimes_merged() {
        let existing = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::from([("node".into(), "20".into())]),
                cli: None,
            }),
            ..Default::default()
        };
        let template = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::from([
                    ("node".into(), "22".into()),
                    ("python".into(), "3.12".into()),
                ]),
                cli: None,
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let tools = result.tools.unwrap();
        assert_eq!(tools.runtimes.get("node").unwrap(), "20");
        assert_eq!(tools.runtimes.get("python").unwrap(), "3.12");
    }

    #[test]
    fn test_merge_tools_cli_merged() {
        let existing = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::new(),
                cli: Some(HashMap::from([("ripgrep".into(), "14".into())])),
            }),
            ..Default::default()
        };
        let template = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::new(),
                cli: Some(HashMap::from([
                    ("ripgrep".into(), "latest".into()),
                    ("fd".into(), "latest".into()),
                ])),
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let cli = result.tools.unwrap().cli.unwrap();
        assert_eq!(cli.get("ripgrep").unwrap(), "14");
        assert_eq!(cli.get("fd").unwrap(), "latest");
    }

    #[test]
    fn test_merge_tools_only_existing() {
        let existing = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::from([("node".into(), "20".into())]),
                cli: None,
            }),
            ..Default::default()
        };
        let template = GreatConfig::default();
        let result = merge_configs(existing, template);
        let tools = result.tools.unwrap();
        assert_eq!(tools.runtimes.get("node").unwrap(), "20");
    }

    #[test]
    fn test_merge_tools_only_template() {
        let existing = GreatConfig::default();
        let template = GreatConfig {
            tools: Some(ToolsConfig {
                runtimes: HashMap::from([("python".into(), "3.12".into())]),
                cli: None,
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let tools = result.tools.unwrap();
        assert_eq!(tools.runtimes.get("python").unwrap(), "3.12");
    }

    #[test]
    fn test_merge_agents_existing_wins() {
        let existing = GreatConfig {
            agents: Some(HashMap::from([(
                "claude".into(),
                AgentConfig {
                    provider: Some("anthropic".into()),
                    model: Some("opus".into()),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let template = GreatConfig {
            agents: Some(HashMap::from([(
                "claude".into(),
                AgentConfig {
                    provider: Some("anthropic".into()),
                    model: Some("sonnet".into()),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let agents = result.agents.unwrap();
        assert_eq!(agents["claude"].model.as_deref(), Some("opus"));
    }

    #[test]
    fn test_merge_agents_template_fills_new_keys() {
        let existing = GreatConfig {
            agents: Some(HashMap::from([(
                "claude".into(),
                AgentConfig {
                    provider: Some("anthropic".into()),
                    model: None,
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let template = GreatConfig {
            agents: Some(HashMap::from([(
                "gpt".into(),
                AgentConfig {
                    provider: Some("openai".into()),
                    model: Some("gpt-4".into()),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let agents = result.agents.unwrap();
        assert!(agents.contains_key("claude"));
        assert!(agents.contains_key("gpt"));
        assert_eq!(agents["gpt"].model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_merge_mcp_existing_wins() {
        let existing = GreatConfig {
            mcp: Some(HashMap::from([(
                "fs".into(),
                McpConfig {
                    command: "existing-cmd".into(),
                    args: None,
                    env: None,
                    transport: None,
                    url: None,
                    enabled: None,
                },
            )])),
            ..Default::default()
        };
        let template = GreatConfig {
            mcp: Some(HashMap::from([
                (
                    "fs".into(),
                    McpConfig {
                        command: "template-cmd".into(),
                        args: None,
                        env: None,
                        transport: None,
                        url: None,
                        enabled: None,
                    },
                ),
                (
                    "db".into(),
                    McpConfig {
                        command: "db-cmd".into(),
                        args: None,
                        env: None,
                        transport: None,
                        url: None,
                        enabled: None,
                    },
                ),
            ])),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let mcp = result.mcp.unwrap();
        assert_eq!(mcp["fs"].command, "existing-cmd");
        assert_eq!(mcp["db"].command, "db-cmd");
    }

    #[test]
    fn test_merge_secrets_existing_wins() {
        let existing = GreatConfig {
            secrets: Some(SecretsConfig {
                provider: Some("1password".into()),
                required: Some(vec!["KEY_A".into()]),
            }),
            ..Default::default()
        };
        let template = GreatConfig {
            secrets: Some(SecretsConfig {
                provider: Some("env".into()),
                required: Some(vec!["KEY_B".into()]),
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let secrets = result.secrets.unwrap();
        assert_eq!(secrets.provider.as_deref(), Some("1password"));
    }

    #[test]
    fn test_merge_platform_existing_wins() {
        let existing = GreatConfig {
            platform: Some(PlatformConfig {
                macos: Some(PlatformOverride {
                    extra_tools: Some(vec!["coreutils".into()]),
                }),
                wsl2: None,
                linux: None,
            }),
            ..Default::default()
        };
        let template = GreatConfig {
            platform: Some(PlatformConfig {
                macos: Some(PlatformOverride {
                    extra_tools: Some(vec!["gnu-sed".into()]),
                }),
                wsl2: None,
                linux: None,
            }),
            ..Default::default()
        };
        let result = merge_configs(existing, template);
        let platform = result.platform.unwrap();
        let macos_tools = platform.macos.unwrap().extra_tools.unwrap();
        assert_eq!(macos_tools, vec!["coreutils"]);
    }
}
