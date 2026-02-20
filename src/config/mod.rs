pub mod schema;

use std::path::PathBuf;

use anyhow::Result;

// Re-exported for downstream consumption by CLI subcommands.
#[allow(unused_imports)]
pub use schema::{
    AgentConfig, ConfigMessage, GreatConfig, McpConfig, PlatformConfig, PlatformOverride,
    ProjectConfig, SecretsConfig, ToolsConfig,
};

/// Load configuration from the specified path (or discover it), parse, and validate.
///
/// Returns the parsed [`GreatConfig`] on success. Validation warnings are printed
/// to stderr; validation errors cause the load to fail with a descriptive message.
pub fn load(path: Option<&str>) -> Result<GreatConfig> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => discover_config()?,
    };

    let contents = std::fs::read_to_string(&config_path)?;
    let config: GreatConfig = toml::from_str(&contents)?;

    // Run validation and report issues
    let messages = config.validate();
    for msg in &messages {
        match msg {
            ConfigMessage::Warning(w) => {
                eprintln!("config warning: {}", w);
            }
            ConfigMessage::Error(e) => {
                anyhow::bail!("config error in {}: {}", config_path.display(), e);
            }
        }
    }

    Ok(config)
}

/// Search for great.toml in current directory and parents.
pub fn discover_config() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.as_path();

    loop {
        let candidate = dir.join("great.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!("no great.toml found in current directory or parents"),
        }
    }
}

/// Return the platform-specific data directory (~/.local/share/great on Linux).
#[allow(dead_code)]
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join("great"))
        .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))
}

/// Return the platform-specific config directory (~/.config/great on Linux).
#[allow(dead_code)]
pub fn config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join("great"))
        .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("great.toml");
        std::fs::write(&config_path, "[project]\nname = \"test-project\"\n").unwrap();
        let config = load(Some(config_path.to_str().unwrap())).unwrap();
        assert_eq!(config.project.unwrap().name.unwrap(), "test-project");
    }

    #[test]
    fn test_load_missing_file_errors() {
        let result = load(Some("/tmp/definitely_not_a_real_great_toml_xyz.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_toml_errors() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("great.toml");
        std::fs::write(&config_path, "this is not valid [[[ toml").unwrap();
        let result = load(Some(config_path.to_str().unwrap()));
        assert!(result.is_err());
    }

    #[test]
    fn test_data_dir_returns_path() {
        let result = data_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("great"));
    }

    #[test]
    fn test_config_dir_returns_path() {
        let result = config_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("great"));
    }
}
