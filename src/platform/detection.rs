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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub has_homebrew: bool,
    pub has_apt: bool,
    pub has_dnf: bool,
    pub has_pacman: bool,
    pub has_snap: bool,
    pub has_systemd: bool,
    pub is_wsl2: bool,
    pub has_docker: bool,
}

/// Complete platform detection result: OS, capabilities, user context, and shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// Uses the `which` crate for pure-Rust PATH resolution. No shell spawning.
/// Returns `false` if the command is not found or the lookup itself fails.
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Detect CPU architecture from `std::env::consts::ARCH`.
pub fn detect_architecture() -> Architecture {
    match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Aarch64,
        _ => Architecture::Unknown,
    }
}

/// Detect available package managers, services, and WSL2 status.
pub fn detect_capabilities(_platform: &Platform) -> PlatformCapabilities {
    PlatformCapabilities {
        has_homebrew: command_exists("brew"),
        has_apt: command_exists("apt"),
        has_dnf: command_exists("dnf"),
        has_pacman: command_exists("pacman"),
        has_snap: command_exists("snap"),
        has_systemd: std::path::Path::new("/run/systemd/system").exists(),
        is_wsl2: is_wsl2(),
        has_docker: command_exists("docker"),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Returns `true` when running inside Windows Subsystem for Linux (WSL1 or WSL2).
///
/// Three-tier detection: environment variable, WSLInterop file, /proc/version.
fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return true;
    }

    if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
        return true;
    }

    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

/// Returns `true` only when running inside WSL2 (not WSL1).
///
/// Detection: `/proc/sys/fs/binfmt_misc/WSLInterop` file exists in WSL2 but
/// is absent in WSL1.
fn is_wsl2() -> bool {
    std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
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
#[cfg(unix)]
fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|uid| uid.trim() == "0")
        .unwrap_or(false)
}

/// Check if the current user is root (always false on non-Unix).
#[cfg(not(unix))]
fn is_root() -> bool {
    false
}

/// Return the current user's shell.
fn detect_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "unknown".into())
    }
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "unknown".into())
    }
    #[cfg(not(any(unix, windows)))]
    {
        "unknown".into()
    }
}

// ---------------------------------------------------------------------------
// Test support: OsProbe trait and _with_probe variants
// ---------------------------------------------------------------------------

#[cfg(test)]
trait OsProbe {
    fn read_file(&self, path: &str) -> Option<String>;
    fn env_var(&self, name: &str) -> Option<String>;
    fn path_exists(&self, path: &str) -> bool;
    fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String>;
}

#[cfg(test)]
fn is_wsl_with_probe(probe: &dyn OsProbe) -> bool {
    if probe.env_var("WSL_DISTRO_NAME").is_some() {
        return true;
    }

    if probe.path_exists("/proc/sys/fs/binfmt_misc/WSLInterop") {
        return true;
    }

    probe
        .read_file("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

#[cfg(test)]
fn is_wsl2_with_probe(probe: &dyn OsProbe) -> bool {
    probe.path_exists("/proc/sys/fs/binfmt_misc/WSLInterop")
}

#[cfg(test)]
fn detect_linux_distro_with_probe(probe: &dyn OsProbe) -> LinuxDistro {
    let content = match probe.read_file("/etc/os-release") {
        Some(c) => c,
        None => return LinuxDistro::Other("unknown".into()),
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

#[cfg(test)]
fn is_root_with_probe(probe: &dyn OsProbe) -> bool {
    probe
        .command_output("id", &["-u"])
        .map(|uid| uid.trim() == "0")
        .unwrap_or(false)
}

#[cfg(test)]
fn detect_shell_with_probe(probe: &dyn OsProbe) -> String {
    probe.env_var("SHELL").unwrap_or_else(|| "unknown".into())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockProbe {
        files: HashMap<String, String>,
        env_vars: HashMap<String, String>,
        paths: HashMap<String, bool>,
        commands: HashMap<String, String>,
    }

    impl MockProbe {
        fn new() -> Self {
            Self {
                files: HashMap::new(),
                env_vars: HashMap::new(),
                paths: HashMap::new(),
                commands: HashMap::new(),
            }
        }
    }

    impl OsProbe for MockProbe {
        fn read_file(&self, path: &str) -> Option<String> {
            self.files.get(path).cloned()
        }

        fn env_var(&self, name: &str) -> Option<String> {
            self.env_vars.get(name).cloned()
        }

        fn path_exists(&self, path: &str) -> bool {
            self.paths.get(path).copied().unwrap_or(false)
        }

        fn command_output(&self, cmd: &str, args: &[&str]) -> Option<String> {
            let key = format!("{} {}", cmd, args.join(" "));
            self.commands.get(&key).cloned()
        }
    }

    // -----------------------------------------------------------------------
    // Machine-dependent tests (tests 1-10)
    // -----------------------------------------------------------------------

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
    fn test_command_exists_empty_string() {
        assert!(!command_exists(""));
    }

    #[test]
    fn test_platform_display() {
        let p = detect_platform();
        let s = format!("{}", p);
        assert!(["macos", "linux", "wsl", "windows", "unknown"].contains(&s.as_str()));
    }

    #[test]
    fn test_platform_display_detailed() {
        let p = detect_platform();
        let detail = p.display_detailed();
        assert!(!detail.is_empty());
        // display_detailed always includes architecture info
        let arch_strs = ["X86_64", "Aarch64", "Unknown"];
        assert!(
            arch_strs.iter().any(|a| detail.contains(a)),
            "display_detailed() should contain architecture: {}",
            detail
        );
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
        // On WSL2, both Platform::Wsl and is_wsl2 should be true.
        // On non-WSL, is_wsl2 should be false.
        if !matches!(platform, Platform::Wsl { .. }) {
            assert!(!caps.is_wsl2);
        }
    }

    #[test]
    fn test_platform_serialize_roundtrip() {
        let platform = detect_platform();
        let json = serde_json::to_string(&platform).expect("serialize");
        let roundtripped: Platform = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(platform, roundtripped);
    }

    // -----------------------------------------------------------------------
    // Mock-based tests (tests 11-24)
    // -----------------------------------------------------------------------

    #[test]
    fn test_wsl_detected_from_env_var() {
        let mut probe = MockProbe::new();
        probe
            .env_vars
            .insert("WSL_DISTRO_NAME".into(), "Ubuntu".into());
        assert!(is_wsl_with_probe(&probe));
    }

    #[test]
    fn test_wsl_detected_from_interop_file() {
        let mut probe = MockProbe::new();
        probe
            .paths
            .insert("/proc/sys/fs/binfmt_misc/WSLInterop".into(), true);
        assert!(is_wsl_with_probe(&probe));
    }

    #[test]
    fn test_wsl_detected_from_proc_version() {
        let mut probe = MockProbe::new();
        probe.files.insert(
            "/proc/version".into(),
            "Linux version 5.15.0-1-Microsoft".into(),
        );
        assert!(is_wsl_with_probe(&probe));
    }

    #[test]
    fn test_not_wsl_when_all_checks_fail() {
        let probe = MockProbe::new();
        assert!(!is_wsl_with_probe(&probe));
    }

    #[test]
    fn test_wsl2_true_when_interop_exists() {
        let mut probe = MockProbe::new();
        probe
            .paths
            .insert("/proc/sys/fs/binfmt_misc/WSLInterop".into(), true);
        assert!(is_wsl2_with_probe(&probe));
    }

    #[test]
    fn test_wsl2_false_when_interop_missing() {
        let probe = MockProbe::new();
        assert!(!is_wsl2_with_probe(&probe));
    }

    #[test]
    fn test_distro_ubuntu() {
        let mut probe = MockProbe::new();
        probe.files.insert(
            "/etc/os-release".into(),
            "NAME=\"Ubuntu\"\nID=ubuntu\nVERSION_ID=\"24.04\"".into(),
        );
        assert_eq!(detect_linux_distro_with_probe(&probe), LinuxDistro::Ubuntu);
    }

    #[test]
    fn test_distro_unknown_on_missing_file() {
        let probe = MockProbe::new();
        assert_eq!(
            detect_linux_distro_with_probe(&probe),
            LinuxDistro::Other("unknown".into())
        );
    }

    #[test]
    fn test_distro_quoted_id() {
        let mut probe = MockProbe::new();
        probe
            .files
            .insert("/etc/os-release".into(), "ID=\"debian\"".into());
        assert_eq!(detect_linux_distro_with_probe(&probe), LinuxDistro::Debian);
    }

    #[test]
    fn test_distro_arch() {
        let mut probe = MockProbe::new();
        probe
            .files
            .insert("/etc/os-release".into(), "ID=arch".into());
        assert_eq!(detect_linux_distro_with_probe(&probe), LinuxDistro::Arch);
    }

    #[test]
    fn test_is_root_true() {
        let mut probe = MockProbe::new();
        probe.commands.insert("id -u".into(), "0\n".into());
        assert!(is_root_with_probe(&probe));
    }

    #[test]
    fn test_is_root_false() {
        let mut probe = MockProbe::new();
        probe.commands.insert("id -u".into(), "1000\n".into());
        assert!(!is_root_with_probe(&probe));
    }

    #[test]
    fn test_detect_shell_unix() {
        let mut probe = MockProbe::new();
        probe.env_vars.insert("SHELL".into(), "/bin/zsh".into());
        assert_eq!(detect_shell_with_probe(&probe), "/bin/zsh");
    }

    #[test]
    fn test_detect_shell_unset() {
        let probe = MockProbe::new();
        assert_eq!(detect_shell_with_probe(&probe), "unknown");
    }
}
