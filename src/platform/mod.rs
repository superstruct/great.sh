pub mod detection;

pub use detection::{detect_platform, Platform};

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macos"),
            Platform::Linux => write!(f, "linux"),
            Platform::Wsl => write!(f, "wsl"),
            Platform::Windows => write!(f, "windows"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}
