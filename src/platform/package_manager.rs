use anyhow::{bail, Context, Result};

use super::detection::command_exists;

/// Trait for package manager operations. Object-safe.
pub trait PackageManager {
    /// Human-readable name of this package manager.
    fn name(&self) -> &str;

    /// Check if this package manager is available on the system.
    fn is_available(&self) -> bool;

    /// Check if a package is installed.
    fn is_installed(&self, package: &str) -> bool;

    /// Get the installed version of a package, if any.
    #[allow(dead_code)]
    fn installed_version(&self, package: &str) -> Option<String>;

    /// Install a package, optionally at a specific version.
    fn install(&self, package: &str, version: Option<&str>) -> Result<()>;

    /// Update a package to the latest version.
    #[allow(dead_code)]
    fn update(&self, package: &str) -> Result<()>;
}

// -------------------------------------------------------------------
// Homebrew
// -------------------------------------------------------------------

/// Homebrew package manager — primary for macOS, Ubuntu, and WSL Ubuntu.
///
/// Homebrew (Linuxbrew on Linux) is the preferred package manager for all
/// supported platforms because it provides up-to-date tool versions without
/// requiring sudo. `great apply` step 2b auto-installs Homebrew if missing
/// on macOS, Ubuntu bare metal, or Ubuntu under WSL2.
///
/// Apt is kept only as a fallback for system-level packages that must come
/// from OS repos (e.g. docker, chrome, build-essential).
pub struct Homebrew;

impl PackageManager for Homebrew {
    fn name(&self) -> &str {
        "homebrew"
    }

    fn is_available(&self) -> bool {
        command_exists("brew")
    }

    fn is_installed(&self, package: &str) -> bool {
        std::process::Command::new("brew")
            .args(["list", "--formula", package])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn installed_version(&self, package: &str) -> Option<String> {
        let output = std::process::Command::new("brew")
            .args(["list", "--versions", package])
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            // Format: "package 1.2.3" — take the version part
            text.split_whitespace().nth(1).map(|v| v.to_string())
        } else {
            None
        }
    }

    fn install(&self, package: &str, version: Option<&str>) -> Result<()> {
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let mut cmd = std::process::Command::new("brew");
        cmd.arg("install");
        if let Some(ver) = version {
            // Homebrew uses formula@version for versioned installs
            if ver != "latest" {
                cmd.arg(format!("{}@{}", package, ver));
            } else {
                cmd.arg(package);
            }
        } else {
            cmd.arg(package);
        }
        let status = cmd
            .status()
            .context(format!("failed to run brew install {}", package))?;
        if !status.success() {
            bail!(
                "brew install {} failed with exit code {:?}",
                package,
                status.code()
            );
        }
        Ok(())
    }

    fn update(&self, package: &str) -> Result<()> {
        let status = std::process::Command::new("brew")
            .args(["upgrade", package])
            .status()
            .context(format!("failed to run brew upgrade {}", package))?;
        if !status.success() {
            bail!("brew upgrade {} failed", package);
        }
        Ok(())
    }
}

// -------------------------------------------------------------------
// Apt
// -------------------------------------------------------------------

/// Apt package manager (Debian / Ubuntu) — fallback only.
///
/// Used as a last resort for system-level packages that aren't in Homebrew
/// or need OS-repo versions (e.g. docker-ce from Docker's apt repo, google-chrome
/// from Google's repo, build-essential). For regular CLI tools, Homebrew is
/// preferred because it doesn't require sudo and ships newer versions.
pub struct Apt;

impl PackageManager for Apt {
    fn name(&self) -> &str {
        "apt"
    }

    fn is_available(&self) -> bool {
        command_exists("apt-get")
    }

    fn is_installed(&self, package: &str) -> bool {
        std::process::Command::new("dpkg")
            .args(["-s", package])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn installed_version(&self, package: &str) -> Option<String> {
        let output = std::process::Command::new("dpkg")
            .args(["-s", package])
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(version) = line.strip_prefix("Version: ") {
                    return Some(version.trim().to_string());
                }
            }
        }
        None
    }

    fn install(&self, package: &str, _version: Option<&str>) -> Result<()> {
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let status = std::process::Command::new("sudo")
            .args(["apt-get", "install", "-y", package])
            .status()
            .context(format!("failed to run apt-get install {}", package))?;
        if !status.success() {
            bail!("apt-get install {} failed (may need sudo)", package);
        }
        Ok(())
    }

    fn update(&self, package: &str) -> Result<()> {
        // apt-get upgrade only works for all packages; for a single package, reinstall
        let status = std::process::Command::new("sudo")
            .args(["apt-get", "install", "--only-upgrade", "-y", package])
            .status()
            .context(format!("failed to update {}", package))?;
        if !status.success() {
            bail!("apt-get upgrade {} failed", package);
        }
        Ok(())
    }
}

// -------------------------------------------------------------------
// Cargo
// -------------------------------------------------------------------

/// Cargo package manager for Rust crates installed via `cargo install`.
pub struct CargoInstaller;

impl PackageManager for CargoInstaller {
    fn name(&self) -> &str {
        "cargo"
    }

    fn is_available(&self) -> bool {
        command_exists("cargo")
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check if the binary produced by the crate is on PATH
        command_exists(package)
    }

    fn installed_version(&self, package: &str) -> Option<String> {
        let output = std::process::Command::new(package)
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let first_line = text.lines().next().unwrap_or("").trim();
            if first_line.is_empty() {
                None
            } else {
                Some(first_line.to_string())
            }
        } else {
            None
        }
    }

    fn install(&self, package: &str, version: Option<&str>) -> Result<()> {
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(["install", package]);
        if let Some(ver) = version {
            if ver != "latest" {
                cmd.args(["--version", ver]);
            }
        }
        let status = cmd
            .status()
            .context(format!("failed to run cargo install {}", package))?;
        if !status.success() {
            bail!("cargo install {} failed", package);
        }
        Ok(())
    }

    fn update(&self, package: &str) -> Result<()> {
        // cargo install --force will reinstall/update
        let status = std::process::Command::new("cargo")
            .args(["install", package, "--force"])
            .status()
            .context(format!("failed to update {}", package))?;
        if !status.success() {
            bail!("cargo install --force {} failed", package);
        }
        Ok(())
    }
}

// -------------------------------------------------------------------
// Npm
// -------------------------------------------------------------------

/// npm global package manager for Node.js tools.
pub struct NpmInstaller;

impl PackageManager for NpmInstaller {
    fn name(&self) -> &str {
        "npm"
    }

    fn is_available(&self) -> bool {
        command_exists("npm")
    }

    fn is_installed(&self, package: &str) -> bool {
        // Check if the npm global package provides a binary on PATH
        command_exists(package)
    }

    fn installed_version(&self, package: &str) -> Option<String> {
        // Try running the binary --version first (more reliable)
        let output = std::process::Command::new(package)
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let first_line = text.lines().next().unwrap_or("").trim();
            if first_line.is_empty() {
                None
            } else {
                Some(first_line.to_string())
            }
        } else {
            None
        }
    }

    fn install(&self, package: &str, version: Option<&str>) -> Result<()> {
        if self.is_installed(package) {
            return Ok(()); // Idempotent
        }
        let pkg_spec = match version {
            Some(ver) if ver != "latest" => format!("{}@{}", package, ver),
            _ => package.to_string(),
        };
        let status = std::process::Command::new("npm")
            .args(["install", "-g", &pkg_spec])
            .status()
            .context(format!("failed to run npm install -g {}", pkg_spec))?;
        if !status.success() {
            bail!("npm install -g {} failed", pkg_spec);
        }
        Ok(())
    }

    fn update(&self, package: &str) -> Result<()> {
        let status = std::process::Command::new("npm")
            .args(["update", "-g", package])
            .status()
            .context(format!("failed to update {}", package))?;
        if !status.success() {
            bail!("npm update -g {} failed", package);
        }
        Ok(())
    }
}

// -------------------------------------------------------------------
// Factory
// -------------------------------------------------------------------

/// Get all available package managers for the current platform, ordered by preference.
///
/// Order: Homebrew (preferred) → Cargo → npm → Apt (fallback).
/// Homebrew is first because it provides up-to-date versions without sudo on all
/// supported platforms (macOS, Ubuntu, WSL Ubuntu). Apt is last because it requires
/// sudo and often ships older versions — it's kept as a fallback for system-level
/// packages only.
pub fn available_managers() -> Vec<Box<dyn PackageManager>> {
    let mut managers: Vec<Box<dyn PackageManager>> = Vec::new();

    // Homebrew first — primary package manager on macOS, Ubuntu, and WSL Ubuntu
    let brew = Homebrew;
    if brew.is_available() {
        managers.push(Box::new(brew));
    }

    let cargo = CargoInstaller;
    if cargo.is_available() {
        managers.push(Box::new(cargo));
    }

    let npm = NpmInstaller;
    if npm.is_available() {
        managers.push(Box::new(npm));
    }

    // Apt last — fallback for system-level packages (docker, chrome, build-essential)
    let apt = Apt;
    if apt.is_available() {
        managers.push(Box::new(apt));
    }

    managers
}

// -------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homebrew_is_available() {
        // Just verify it doesn't panic
        let brew = Homebrew;
        let _ = brew.is_available();
    }

    #[test]
    fn test_apt_is_available() {
        let apt = Apt;
        let _ = apt.is_available();
    }

    #[test]
    fn test_cargo_is_available() {
        let cargo = CargoInstaller;
        // cargo should be available since we're building with it
        assert!(cargo.is_available());
    }

    #[test]
    fn test_cargo_is_installed_for_existing_binary() {
        let cargo = CargoInstaller;
        // 'cargo' binary should exist
        assert!(cargo.is_installed("cargo"));
    }

    #[test]
    fn test_cargo_not_installed_for_fake_package() {
        let cargo = CargoInstaller;
        assert!(!cargo.is_installed("nonexistent_tool_xyz_12345"));
    }

    #[test]
    fn test_available_managers_returns_non_empty() {
        let managers = available_managers();
        // At minimum, cargo should be available since we're running cargo test
        assert!(!managers.is_empty());
    }

    #[test]
    fn test_trait_is_object_safe() {
        // This test verifies the trait can be used as a trait object
        let _: Box<dyn PackageManager> = Box::new(CargoInstaller);
    }
}
