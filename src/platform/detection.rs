use serde::{Deserialize, Serialize};

/// CPU architecture detected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}

/// Known Linux distribution families.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    Arch,
    Other(String),
}

/// Detected operating system and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    MacOS {
        version: Option<String>,
        arch: Architecture,
    },
    Linux {
        distro: LinuxDistro,
        version: Option<String>,
        arch: Architecture,
    },
    Wsl {
        distro: LinuxDistro,
        version: Option<String>,
        arch: Architecture,
    },
    Windows {
        arch: Architecture,
    },
    Unknown,
}

/// Available system package managers and services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCapabilities {
    pub has_homebrew: bool,
    pub has_apt: bool,
    pub has_dnf: bool,
    pub has_snap: bool,
    pub has_systemd: bool,
    pub is_wsl: bool,
    pub has_docker: bool,
}

/// Complete platform detection result: OS, capabilities, user context, and shell.
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub capabilities: PlatformCapabilities,
    pub is_root: bool,
    pub shell: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect the current platform, including WSL detection, distro, version, and
/// architecture.
pub fn detect_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOS {
            version: detect_macos_version(),
            arch: detect_architecture(),
        }
    } else if cfg!(target_os = "linux") {
        let arch = detect_architecture();
        let distro = detect_linux_distro();
        let version = detect_linux_version();

        if is_wsl() {
            Platform::Wsl {
                distro,
                version,
                arch,
            }
        } else {
            Platform::Linux {
                distro,
                version,
                arch,
            }
        }
    } else if cfg!(target_os = "windows") {
        Platform::Windows {
            arch: detect_architecture(),
        }
    } else {
        Platform::Unknown
    }
}

/// Collect full platform info: OS details, capabilities, root status, and shell.
pub fn detect_platform_info() -> PlatformInfo {
    let platform = detect_platform();
    let capabilities = detect_capabilities(&platform);
    let is_root = is_root();
    let shell = detect_shell();

    PlatformInfo {
        platform,
        capabilities,
        is_root,
        shell,
    }
}

/// Check whether a command is available on `$PATH`.
///
/// Uses `which` on Unix and `where` on Windows. Returns `false` if the
/// command is not found or the lookup itself fails.
pub fn command_exists(cmd: &str) -> bool {
    let lookup = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    std::process::Command::new(lookup)
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Detect CPU architecture from `std::env::consts::ARCH`.
pub fn detect_architecture() -> Architecture {
    match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Aarch64,
        _ => Architecture::Unknown,
    }
}

/// Detect available package managers, services, and WSL status.
pub fn detect_capabilities(platform: &Platform) -> PlatformCapabilities {
    let is_wsl = matches!(platform, Platform::Wsl { .. });

    PlatformCapabilities {
        has_homebrew: command_exists("brew"),
        has_apt: command_exists("apt"),
        has_dnf: command_exists("dnf"),
        has_snap: command_exists("snap"),
        has_systemd: std::path::Path::new("/run/systemd/system").exists(),
        is_wsl,
        has_docker: command_exists("docker"),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns `true` when running inside Windows Subsystem for Linux.
///
/// Checks the `WSL_DISTRO_NAME` environment variable first (fast path), then
/// falls back to inspecting `/proc/version` for the "microsoft" marker.
fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return true;
    }

    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

/// Parse the `ID=` field from `/etc/os-release` into a `LinuxDistro`.
fn detect_linux_distro() -> LinuxDistro {
    let content = match std::fs::read_to_string("/etc/os-release") {
        Ok(c) => c,
        Err(_) => return LinuxDistro::Other("unknown".into()),
    };

    for line in content.lines() {
        if let Some(id) = line.strip_prefix("ID=") {
            let id = id.trim().trim_matches('"');
            return match id {
                "ubuntu" => LinuxDistro::Ubuntu,
                "debian" => LinuxDistro::Debian,
                "fedora" => LinuxDistro::Fedora,
                "arch" => LinuxDistro::Arch,
                other => LinuxDistro::Other(other.to_string()),
            };
        }
    }

    LinuxDistro::Other("unknown".into())
}

/// Parse the `VERSION_ID=` field from `/etc/os-release`.
fn detect_linux_version() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;

    for line in content.lines() {
        if let Some(ver) = line.strip_prefix("VERSION_ID=") {
            let ver = ver.trim().trim_matches('"');
            if ver.is_empty() {
                return None;
            }
            return Some(ver.to_string());
        }
    }

    None
}

/// Run `sw_vers -productVersion` to obtain the macOS version string.
fn detect_macos_version() -> Option<String> {
    let output = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;

    if output.status.success() {
        let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if ver.is_empty() {
            None
        } else {
            Some(ver)
        }
    } else {
        None
    }
}

/// Check if the current user is root.
fn is_root() -> bool {
    std::env::var("USER").is_ok_and(|u| u == "root")
}

/// Return the current user's login shell from `$SHELL`.
fn detect_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "unknown".into())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_architecture() {
        let arch = detect_architecture();
        // On any real machine this should not be Unknown
        assert_ne!(arch, Architecture::Unknown);
    }

    #[test]
    fn test_detect_platform_not_unknown() {
        let platform = detect_platform();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_command_exists_positive() {
        // `ls` exists on all Unix systems
        assert!(command_exists("ls"));
    }

    #[test]
    fn test_command_exists_negative() {
        assert!(!command_exists("nonexistent_command_xyz_12345"));
    }

    #[test]
    fn test_platform_display() {
        let p = detect_platform();
        let s = format!("{}", p);
        assert!(["macos", "linux", "wsl", "windows", "unknown"].contains(&s.as_str()));
    }

    #[test]
    fn test_detect_platform_info() {
        let info = detect_platform_info();
        // Shell should be non-empty
        assert!(!info.shell.is_empty());
        // Platform display should work
        assert!(!info.platform.display_detailed().is_empty());
    }

    #[test]
    fn test_detect_capabilities() {
        let platform = detect_platform();
        let caps = detect_capabilities(&platform);
        // is_wsl should match the platform variant
        match &platform {
            Platform::Wsl { .. } => assert!(caps.is_wsl),
            _ => assert!(!caps.is_wsl),
        }
    }
}
