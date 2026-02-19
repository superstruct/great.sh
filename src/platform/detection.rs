#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Linux,
    Wsl,
    Windows,
    Unknown,
}

/// Detect the current platform, including WSL detection.
pub fn detect_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOS
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        if is_wsl() {
            Platform::Wsl
        } else {
            Platform::Linux
        }
    } else {
        Platform::Unknown
    }
}

fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}
