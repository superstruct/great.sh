pub mod detection;
pub mod package_manager;
pub mod runtime;

#[allow(unused_imports)]
pub use detection::{
    command_exists, detect_architecture, detect_platform, detect_platform_info, Architecture,
    LinuxDistro, Platform, PlatformCapabilities, PlatformInfo,
};

#[allow(unused_imports)]
pub use runtime::{MiseManager, ProvisionAction, ProvisionResult};

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacOS { .. } => write!(f, "macos"),
            Platform::Linux { .. } => write!(f, "linux"),
            Platform::Wsl { .. } => write!(f, "wsl"),
            Platform::Windows { .. } => write!(f, "windows"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

impl Platform {
    /// Human-readable detailed description, e.g. "macOS 15.3.1 (aarch64)".
    pub fn display_detailed(&self) -> String {
        match self {
            Platform::MacOS { version, arch } => {
                let ver = version.as_deref().unwrap_or("unknown");
                format!("macOS {} ({})", ver, arch)
            }
            Platform::Linux {
                distro,
                version,
                arch,
            } => {
                let ver = version.as_deref().unwrap_or("unknown");
                format!("Linux {:?} {} ({})", distro, ver, arch)
            }
            Platform::Wsl {
                distro,
                version,
                arch,
            } => {
                let ver = version.as_deref().unwrap_or("unknown");
                format!("WSL {:?} {} ({})", distro, ver, arch)
            }
            Platform::Windows { arch } => format!("Windows ({})", arch),
            Platform::Unknown => "Unknown".to_string(),
        }
    }

    /// Return the architecture for this platform.
    pub fn arch(&self) -> Architecture {
        match self {
            Platform::MacOS { arch, .. } => *arch,
            Platform::Linux { arch, .. } => *arch,
            Platform::Wsl { arch, .. } => *arch,
            Platform::Windows { arch } => *arch,
            Platform::Unknown => Architecture::Unknown,
        }
    }
}
