use std::process::Command;

use crate::cli::output;
use crate::platform::{command_exists, LinuxDistro, Platform, PlatformInfo};

/// Returns true if the platform is an apt-based distro (Ubuntu or Debian on Linux or WSL).
pub fn is_apt_distro(platform: &Platform) -> bool {
    match platform {
        Platform::Linux { distro, .. } | Platform::Wsl { distro, .. } => {
            matches!(distro, LinuxDistro::Ubuntu | LinuxDistro::Debian)
        }
        _ => false,
    }
}

/// Returns true if the platform is Linux or WSL (i.e. can use Linux system commands).
pub fn is_linux_like(platform: &Platform) -> bool {
    matches!(platform, Platform::Linux { .. } | Platform::Wsl { .. })
}

/// Returns true if the distro is Ubuntu specifically (not Debian).
fn is_ubuntu(platform: &Platform) -> bool {
    match platform {
        Platform::Linux { distro, .. } | Platform::Wsl { distro, .. } => {
            matches!(distro, LinuxDistro::Ubuntu)
        }
        _ => false,
    }
}

// ── Individual prerequisite functions ───────────────────────────────────

/// Ensure curl is installed.
pub fn ensure_curl(dry_run: bool, platform: &Platform) {
    if command_exists("curl") {
        output::success("  curl — already installed");
        return;
    }

    if dry_run {
        output::info("  curl — would install");
        return;
    }

    if is_apt_distro(platform) {
        run_sudo_apt_install(&["curl"], "curl");
    } else if matches!(platform, Platform::MacOS { .. }) {
        // curl ships with macOS, but if somehow missing:
        output::info("  curl — should be available on macOS. Install Xcode CLI tools: xcode-select --install");
    } else {
        output::warning("  curl — not found; install manually for your platform");
    }
}

/// Ensure git is installed.
pub fn ensure_git(dry_run: bool, platform: &Platform) {
    if command_exists("git") {
        output::success("  git — already installed");
        return;
    }

    if dry_run {
        output::info("  git — would install");
        return;
    }

    if is_ubuntu(platform) {
        // Ubuntu: add the git-core PPA for latest version
        let steps: &[(&str, &[&str])] = &[
            (
                "sudo",
                &["apt-get", "install", "-y", "software-properties-common"],
            ),
            ("sudo", &["add-apt-repository", "-y", "ppa:git-core/ppa"]),
            ("sudo", &["apt-get", "update"]),
            ("sudo", &["apt-get", "install", "-y", "git"]),
        ];
        for (cmd, args) in steps {
            let status = Command::new(cmd).args(*args).status();
            if !matches!(status, Ok(s) if s.success()) {
                output::error(&format!("  git — failed at: {} {}", cmd, args.join(" ")));
                return;
            }
        }
        output::success("  git — installed via PPA (Ubuntu)");
    } else if is_apt_distro(platform) {
        // Debian: plain apt install
        run_sudo_apt_install(&["git"], "git");
    } else if matches!(platform, Platform::MacOS { .. }) {
        run_xcode_select_install("git");
    } else {
        output::warning("  git — not found; install manually for your platform");
    }
}

/// Ensure build-essential (or Xcode CLI tools on macOS) is installed.
pub fn ensure_build_essential(dry_run: bool, platform: &Platform) {
    if matches!(platform, Platform::MacOS { .. }) {
        // Check if Xcode CLI tools are installed
        let has_xcode = Command::new("xcode-select")
            .arg("-p")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if has_xcode {
            output::success("  build tools (Xcode CLI) — already installed");
            return;
        }

        if dry_run {
            output::info("  build tools (Xcode CLI) — would install");
            return;
        }

        run_xcode_select_install("build tools (Xcode CLI)");
    } else if is_apt_distro(platform) {
        // Check if build-essential is already available via dpkg
        let has_be = Command::new("dpkg")
            .args(["-s", "build-essential"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if has_be {
            output::success("  build-essential — already installed");
            return;
        }

        if dry_run {
            output::info("  build-essential — would install");
            return;
        }

        run_sudo_apt_install(&["build-essential", "procps", "file"], "build-essential");
    } else {
        // Other Linux distros — just note it
        if dry_run {
            output::info("  build tools — check manually for your distro");
        }
    }
}

/// Ensure unzip is installed.
pub fn ensure_unzip(dry_run: bool, platform: &Platform) {
    if command_exists("unzip") {
        output::success("  unzip — already installed");
        return;
    }

    if dry_run {
        output::info("  unzip — would install");
        return;
    }

    if is_apt_distro(platform) {
        run_sudo_apt_install(&["unzip"], "unzip");
    } else if matches!(platform, Platform::MacOS { .. }) {
        let status = Command::new("brew").args(["install", "unzip"]).status();
        match status {
            Ok(s) if s.success() => output::success("  unzip — installed via Homebrew"),
            _ => output::error("  unzip — failed to install. Run: brew install unzip"),
        }
    } else {
        output::warning("  unzip — not found; install manually for your platform");
    }
}

/// Install all system prerequisites under a single header.
pub fn ensure_prerequisites(dry_run: bool, info: &PlatformInfo) {
    output::header("System Prerequisites");

    ensure_curl(dry_run, &info.platform);
    ensure_git(dry_run, &info.platform);
    ensure_build_essential(dry_run, &info.platform);
    ensure_unzip(dry_run, &info.platform);

    println!();
}

// ── Docker ──────────────────────────────────────────────────────────────

/// Ensure Docker is installed and the daemon is running.
pub fn ensure_docker(dry_run: bool, info: &PlatformInfo) {
    output::header("Docker");

    if command_exists("docker") {
        // Check if the daemon is running
        let daemon_ok = Command::new("docker")
            .arg("info")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if daemon_ok {
            output::success("  Docker — installed and daemon running");
        } else {
            output::warning("  Docker — installed but daemon is not running");
            output::info(
                "  Start it with: sudo systemctl start docker (Linux) or launch Docker Desktop",
            );
        }
        println!();
        return;
    }

    if dry_run {
        output::info("  Docker — would install");
        println!();
        return;
    }

    match &info.platform {
        Platform::Linux { distro, .. } => {
            if matches!(distro, LinuxDistro::Ubuntu | LinuxDistro::Debian) {
                install_docker_apt(distro);
            } else {
                output::info("  Docker — install via your distro's package manager or https://docs.docker.com/engine/install/");
            }
        }
        Platform::MacOS { .. } => {
            output::info("  Docker — not installed. Options:");
            output::info("    Docker Desktop: https://www.docker.com/products/docker-desktop/");
            output::info("    OrbStack (lighter): https://orbstack.dev/");
        }
        Platform::Wsl { .. } => {
            output::info("  Docker — not installed. Recommended:");
            output::info("    Docker Desktop for Windows with WSL2 backend: https://docs.docker.com/desktop/wsl/");
        }
        _ => {
            output::info("  Docker — not installed. Visit https://docs.docker.com/get-docker/");
        }
    }

    println!();
}

/// Install Docker CE on an apt-based distro (Ubuntu or Debian) using the official repo.
fn install_docker_apt(distro: &LinuxDistro) {
    let distro_name = match distro {
        LinuxDistro::Ubuntu => "ubuntu",
        LinuxDistro::Debian => "debian",
        _ => {
            output::error("  Docker auto-install only supports Ubuntu and Debian");
            return;
        }
    };

    output::info("  Installing Docker CE via official apt repository...");

    // Install prerequisites for the Docker repo
    let prereqs = Command::new("sudo")
        .args([
            "apt-get",
            "install",
            "-y",
            "ca-certificates",
            "curl",
            "gnupg",
        ])
        .status();
    if !matches!(prereqs, Ok(s) if s.success()) {
        output::error("  Docker — failed to install repo prerequisites");
        return;
    }

    // Add Docker's official GPG key
    let keyring_dir = "/etc/apt/keyrings";
    let _ = Command::new("sudo")
        .args(["install", "-m", "0755", "-d", keyring_dir])
        .status();

    let gpg_url = format!("https://download.docker.com/linux/{}/gpg", distro_name);
    let gpg_status = Command::new("bash")
        .args([
            "-c",
            &format!(
                "curl -fsSL {} | sudo gpg --dearmor -o {}/docker.gpg && sudo chmod a+r {}/docker.gpg",
                gpg_url, keyring_dir, keyring_dir
            ),
        ])
        .status();
    if !matches!(gpg_status, Ok(s) if s.success()) {
        output::error("  Docker — failed to add GPG key");
        return;
    }

    // Add the Docker apt repository
    let repo_cmd = format!(
        r#"echo "deb [arch=$(dpkg --print-architecture) signed-by={keyring}/docker.gpg] https://download.docker.com/linux/{distro} $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null"#,
        keyring = keyring_dir,
        distro = distro_name,
    );
    let repo_status = Command::new("bash").args(["-c", &repo_cmd]).status();
    if !matches!(repo_status, Ok(s) if s.success()) {
        output::error("  Docker — failed to add apt repository");
        return;
    }

    // Install Docker packages
    let update = Command::new("sudo").args(["apt-get", "update"]).status();
    if !matches!(update, Ok(s) if s.success()) {
        output::error("  Docker — apt-get update failed");
        return;
    }

    let install = Command::new("sudo")
        .args([
            "apt-get",
            "install",
            "-y",
            "docker-ce",
            "docker-ce-cli",
            "containerd.io",
            "docker-buildx-plugin",
            "docker-compose-plugin",
        ])
        .status();
    if !matches!(install, Ok(s) if s.success()) {
        output::error("  Docker — package install failed");
        return;
    }

    // Add current user to the docker group
    if let Ok(user) = std::env::var("USER") {
        let _ = Command::new("sudo")
            .args(["usermod", "-aG", "docker", &user])
            .status();
        output::info(&format!(
            "  Added {} to the docker group (log out and back in to take effect)",
            user
        ));
    }

    output::success("  Docker CE — installed");
}

// ── Claude Code ─────────────────────────────────────────────────────────

/// Ensure Claude Code is installed.
pub fn ensure_claude_code(dry_run: bool) {
    if command_exists("claude") {
        output::success("  Claude Code — already installed");
        return;
    }

    if dry_run {
        output::info("  Claude Code — would install");
        return;
    }

    output::info("  Installing Claude Code...");
    let status = Command::new("bash")
        .args(["-c", "curl -fsSL https://claude.ai/install.sh | bash"])
        .status();
    match status {
        Ok(s) if s.success() => output::success("  Claude Code — installed"),
        _ => {
            output::error("  Claude Code — install failed");
            output::info("  Install manually: curl -fsSL https://claude.ai/install.sh | bash");
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Run `sudo apt-get install -y <packages>` and report success/failure.
fn run_sudo_apt_install(packages: &[&str], display_name: &str) {
    let mut args = vec!["apt-get", "install", "-y"];
    args.extend_from_slice(packages);
    let status = Command::new("sudo").args(&args).status();
    match status {
        Ok(s) if s.success() => output::success(&format!("  {} — installed via apt", display_name)),
        _ => output::error(&format!(
            "  {} — failed to install. Run: sudo apt-get install -y {}",
            display_name,
            packages.join(" ")
        )),
    }
}

/// Run `xcode-select --install` for macOS and report.
fn run_xcode_select_install(display_name: &str) {
    let status = Command::new("xcode-select").arg("--install").status();
    match status {
        Ok(s) if s.success() => {
            output::success(&format!(
                "  {} — Xcode CLI tools install triggered",
                display_name
            ));
            output::info("  Follow the system dialog to complete installation");
        }
        _ => {
            output::error(&format!(
                "  {} — failed. Run manually: xcode-select --install",
                display_name
            ));
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Architecture;

    #[test]
    fn is_apt_distro_ubuntu_linux() {
        let p = Platform::Linux {
            distro: LinuxDistro::Ubuntu,
            version: Some("24.04".into()),
            arch: Architecture::X86_64,
        };
        assert!(is_apt_distro(&p));
    }

    #[test]
    fn is_apt_distro_debian_wsl() {
        let p = Platform::Wsl {
            distro: LinuxDistro::Debian,
            version: Some("12".into()),
            arch: Architecture::X86_64,
        };
        assert!(is_apt_distro(&p));
    }

    #[test]
    fn is_apt_distro_fedora() {
        let p = Platform::Linux {
            distro: LinuxDistro::Fedora,
            version: Some("39".into()),
            arch: Architecture::X86_64,
        };
        assert!(!is_apt_distro(&p));
    }

    #[test]
    fn is_apt_distro_macos() {
        let p = Platform::MacOS {
            version: Some("15.0".into()),
            arch: Architecture::Aarch64,
        };
        assert!(!is_apt_distro(&p));
    }

    #[test]
    fn is_linux_like_linux() {
        let p = Platform::Linux {
            distro: LinuxDistro::Ubuntu,
            version: None,
            arch: Architecture::X86_64,
        };
        assert!(is_linux_like(&p));
    }

    #[test]
    fn is_linux_like_wsl() {
        let p = Platform::Wsl {
            distro: LinuxDistro::Ubuntu,
            version: None,
            arch: Architecture::X86_64,
        };
        assert!(is_linux_like(&p));
    }

    #[test]
    fn is_linux_like_macos() {
        let p = Platform::MacOS {
            version: None,
            arch: Architecture::Aarch64,
        };
        assert!(!is_linux_like(&p));
    }

    #[test]
    fn is_linux_like_windows() {
        let p = Platform::Windows {
            arch: Architecture::X86_64,
        };
        assert!(!is_linux_like(&p));
    }
}
