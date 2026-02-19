pub mod schema;

use std::path::PathBuf;

use anyhow::Result;

use schema::GreatConfig;

/// Load configuration from the default or specified path.
pub fn load(path: Option<&str>) -> Result<GreatConfig> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => discover_config()?,
    };

    let contents = std::fs::read_to_string(&config_path)?;
    let config: GreatConfig = toml::from_str(&contents)?;
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
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join("great"))
        .ok_or_else(|| anyhow::anyhow!("could not determine data directory"))
}

/// Return the platform-specific config directory (~/.config/great on Linux).
pub fn config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join("great"))
        .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))
}
