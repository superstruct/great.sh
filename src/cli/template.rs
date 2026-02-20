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

    for tmpl in builtin_templates() {
        output::info(&format!("  {} — {}", tmpl.name, tmpl.description));
    }

    println!();
    output::info("Apply with: great template apply <name>");
    Ok(())
}

fn run_apply(name: &str) -> Result<()> {
    let templates = builtin_templates();
    let tmpl = match templates.iter().find(|t| t.name == name) {
        Some(t) => t,
        None => {
            output::error(&format!("Unknown template: {}", name));
            output::info("Available templates:");
            for t in &templates {
                output::info(&format!("  {}", t.name));
            }
            return Ok(());
        }
    };

    let config_path = std::path::Path::new("great.toml");

    if config_path.exists() {
        // Merge with existing config
        output::info("Existing great.toml found — merging template.");

        let existing = config::load(Some("great.toml"))?;
        let template_config: crate::config::schema::GreatConfig = toml::from_str(tmpl.content)
            .context(format!("failed to parse template '{}'", name))?;

        // Merge: template values fill in gaps, existing values take precedence
        let merged = merge_configs(existing, template_config);

        let toml_string = toml::to_string_pretty(&merged)
            .context("failed to serialize merged config")?;
        std::fs::write(config_path, toml_string)
            .context("failed to write great.toml")?;

        output::success(&format!("Merged template '{}' into great.toml", name));
    } else {
        // Write template directly
        std::fs::write(config_path, tmpl.content)
            .context("failed to write great.toml")?;

        output::success(&format!("Created great.toml from template '{}'", name));
    }

    output::info("Run `great apply` to provision your environment.");
    Ok(())
}

fn run_update() -> Result<()> {
    output::warning("Template registry is not yet available.");
    output::info("Built-in templates are bundled with the great binary.");
    output::info("Update great itself to get new templates: great update");
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
    }
}
