use thiserror::Error;

#[derive(Error, Debug)]
pub enum GreatError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
