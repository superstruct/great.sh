use anyhow::{bail, Context, Result};

use super::detection::command_exists;

/// Result of provisioning a single runtime.
#[derive(Debug, Clone)]
pub struct ProvisionResult {
    pub name: String,
    pub declared_version: String,
    pub action: ProvisionAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvisionAction {
    /// Already installed at a compatible version.
    AlreadyCorrect,
    /// Newly installed.
    Installed,
    /// Updated from a different version.
    Updated,
    /// Installation failed.
    Failed(String),
}

/// Manages runtimes via the `mise` tool version manager.
pub struct MiseManager;

impl MiseManager {
    /// Check if mise is installed.
    pub fn is_available() -> bool {
        command_exists("mise")
    }

    /// Get the mise version string.
    pub fn version() -> Option<String> {
        let output = std::process::Command::new("mise")
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

    /// Install mise itself. Tries curl installer since mise isn't in most package managers.
    pub fn ensure_installed() -> Result<()> {
        if Self::is_available() {
            return Ok(());
        }

        // Use the official installer
        let status = std::process::Command::new("sh")
            .args(["-c", "curl -fsSL https://mise.jdx.dev/install.sh | sh"])
            .status()
            .context("failed to run mise installer")?;

        if !status.success() {
            bail!("mise installation failed — install manually: https://mise.jdx.dev");
        }

        // Verify it worked
        if !Self::is_available() {
            bail!("mise was installed but not found on PATH — you may need to restart your shell");
        }

        Ok(())
    }

    /// Install a runtime at a specific version.
    pub fn install_runtime(name: &str, version: &str) -> Result<()> {
        if !Self::is_available() {
            bail!("mise is not installed — run `great doctor` for installation instructions");
        }

        let spec = format!("{}@{}", name, version);

        // Install the runtime
        let status = std::process::Command::new("mise")
            .args(["install", &spec])
            .status()
            .context(format!("failed to run mise install {}", spec))?;

        if !status.success() {
            bail!("mise install {} failed", spec);
        }

        // Activate globally
        let status = std::process::Command::new("mise")
            .args(["use", "--global", &spec])
            .status()
            .context(format!("failed to run mise use --global {}", spec))?;

        if !status.success() {
            bail!("mise use --global {} failed", spec);
        }

        Ok(())
    }

    /// Get the currently active version of a runtime.
    pub fn installed_version(name: &str) -> Option<String> {
        let output = std::process::Command::new("mise")
            .args(["current", name])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .ok()?;

        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let version = text.trim();
            if version.is_empty() || version.contains("No version") {
                None
            } else {
                Some(version.to_string())
            }
        } else {
            None
        }
    }

    /// Check if an installed version matches the declared version.
    ///
    /// Uses prefix matching: declared "22" matches installed "22.11.0",
    /// declared "3.12" matches "3.12.5", declared "stable" always matches.
    pub fn version_matches(declared: &str, installed: &str) -> bool {
        if declared == "latest" || declared == "stable" {
            return true;
        }
        // Prefix match: "22" matches "22.x.y", "3.12" matches "3.12.z"
        installed.starts_with(declared)
    }

    /// Provision all runtimes from a ToolsConfig.
    /// Skips the "cli" key which is reserved for CLI tools.
    pub fn provision_from_config(
        tools: &crate::config::schema::ToolsConfig,
    ) -> Vec<ProvisionResult> {
        let mut results = Vec::new();

        for (name, declared_version) in &tools.runtimes {
            // Skip the "cli" key — it's a sub-table, not a runtime
            if name == "cli" {
                continue;
            }

            let result = Self::provision_single(name, declared_version);
            results.push(result);
        }

        results
    }

    fn provision_single(name: &str, declared_version: &str) -> ProvisionResult {
        // Check if already installed at the right version
        if let Some(current) = Self::installed_version(name) {
            if Self::version_matches(declared_version, &current) {
                return ProvisionResult {
                    name: name.to_string(),
                    declared_version: declared_version.to_string(),
                    action: ProvisionAction::AlreadyCorrect,
                };
            }
            // Wrong version — update
            match Self::install_runtime(name, declared_version) {
                Ok(()) => ProvisionResult {
                    name: name.to_string(),
                    declared_version: declared_version.to_string(),
                    action: ProvisionAction::Updated,
                },
                Err(e) => ProvisionResult {
                    name: name.to_string(),
                    declared_version: declared_version.to_string(),
                    action: ProvisionAction::Failed(e.to_string()),
                },
            }
        } else {
            // Not installed — install
            match Self::install_runtime(name, declared_version) {
                Ok(()) => ProvisionResult {
                    name: name.to_string(),
                    declared_version: declared_version.to_string(),
                    action: ProvisionAction::Installed,
                },
                Err(e) => ProvisionResult {
                    name: name.to_string(),
                    declared_version: declared_version.to_string(),
                    action: ProvisionAction::Failed(e.to_string()),
                },
            }
        }
    }
}

// -------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mise_is_available() {
        // Just verify it doesn't panic — mise may or may not be installed
        let _ = MiseManager::is_available();
    }

    #[test]
    fn test_version_matches_exact() {
        assert!(MiseManager::version_matches("22.11.0", "22.11.0"));
    }

    #[test]
    fn test_version_matches_prefix() {
        assert!(MiseManager::version_matches("22", "22.11.0"));
        assert!(MiseManager::version_matches("3.12", "3.12.5"));
    }

    #[test]
    fn test_version_matches_latest() {
        assert!(MiseManager::version_matches("latest", "22.11.0"));
        assert!(MiseManager::version_matches("stable", "1.75.0"));
    }

    #[test]
    fn test_version_no_match() {
        assert!(!MiseManager::version_matches("22", "20.11.0"));
        assert!(!MiseManager::version_matches("3.12", "3.11.5"));
    }

    #[test]
    fn test_provision_action_eq() {
        assert_eq!(
            ProvisionAction::AlreadyCorrect,
            ProvisionAction::AlreadyCorrect
        );
        assert_eq!(ProvisionAction::Installed, ProvisionAction::Installed);
        assert_ne!(ProvisionAction::Installed, ProvisionAction::Updated);
    }
}
