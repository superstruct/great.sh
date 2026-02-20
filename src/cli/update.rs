use anyhow::{bail, Context, Result};
use clap::Args as ClapArgs;

use crate::cli::output;
use crate::platform;

/// Version of this binary, set at compile time from Cargo.toml.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub repository for release checking.
const GITHUB_REPO: &str = "great-sh/great";

#[derive(ClapArgs)]
pub struct Args {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

/// Check for or perform a self-update of the `great` CLI.
pub fn run(args: Args) -> Result<()> {
    output::header("great update");
    println!();
    output::info(&format!("Current version: {}", CURRENT_VERSION));

    let rt = tokio::runtime::Runtime::new().context("failed to create async runtime")?;

    if args.check {
        return rt.block_on(check_for_update());
    }

    rt.block_on(self_update())
}

/// Check the latest release on GitHub and report whether an update is available.
async fn check_for_update() -> Result<()> {
    output::info("Checking for updates...");

    match fetch_latest_version().await {
        Ok(latest) => {
            let current = semver::Version::parse(CURRENT_VERSION)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0));
            let remote =
                semver::Version::parse(&latest).unwrap_or_else(|_| semver::Version::new(0, 0, 0));

            if remote > current {
                output::warning(&format!(
                    "Update available: {} → {}",
                    CURRENT_VERSION, latest
                ));
                output::info("Run `great update` to install the latest version.");
            } else {
                output::success(&format!("Already up to date ({})", CURRENT_VERSION));
            }
        }
        Err(e) => {
            output::error(&format!("Failed to check for updates: {}", e));
            output::info("Check https://github.com/great-sh/great/releases manually.");
        }
    }

    Ok(())
}

/// Download the latest release and replace the current binary.
async fn self_update() -> Result<()> {
    output::info("Checking for updates...");

    let latest = match fetch_latest_version().await {
        Ok(v) => v,
        Err(e) => {
            output::error(&format!("Failed to check for updates: {}", e));
            output::info("To update manually, re-run the install script:");
            output::info("  curl -sSL https://great.sh/install.sh | bash");
            return Ok(());
        }
    };

    let current =
        semver::Version::parse(CURRENT_VERSION).unwrap_or_else(|_| semver::Version::new(0, 0, 0));
    let remote = semver::Version::parse(&latest).unwrap_or_else(|_| semver::Version::new(0, 0, 0));

    if remote <= current {
        output::success(&format!("Already up to date ({})", CURRENT_VERSION));
        return Ok(());
    }

    output::info(&format!("Updating {} → {}...", CURRENT_VERSION, latest));

    // Determine the asset name for this platform
    let asset_name = release_asset_name();
    let download_url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        GITHUB_REPO, latest, asset_name
    );

    let spinner = output::spinner("Downloading...");

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", format!("great-sh/{}", CURRENT_VERSION))
        .send()
        .await
        .context("failed to download release")?;

    if !response.status().is_success() {
        spinner.finish_and_clear();
        output::error(&format!(
            "Download failed (HTTP {}). Asset '{}' may not exist for this platform.",
            response.status(),
            asset_name
        ));
        output::info("To update manually:");
        output::info("  curl -sSL https://great.sh/install.sh | bash");
        return Ok(());
    }

    let bytes = response.bytes().await.context("failed to read download")?;

    spinner.finish_and_clear();

    // Write to a temp file then atomically replace
    let current_exe = std::env::current_exe().context("failed to locate current binary")?;
    let temp_path = current_exe.with_extension("new");

    std::fs::write(&temp_path, &bytes).context("failed to write downloaded binary")?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))
            .context("failed to set permissions")?;
    }

    // Atomic rename
    let backup_path = current_exe.with_extension("bak");
    std::fs::rename(&current_exe, &backup_path).context("failed to backup current binary")?;

    match std::fs::rename(&temp_path, &current_exe) {
        Ok(()) => {
            let _ = std::fs::remove_file(&backup_path);
            output::success(&format!("Updated to v{}", latest));
        }
        Err(e) => {
            // Restore backup
            let _ = std::fs::rename(&backup_path, &current_exe);
            bail!("failed to install new binary: {}", e);
        }
    }

    Ok(())
}

/// Fetch the latest version tag from GitHub Releases API.
async fn fetch_latest_version() -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", format!("great-sh/{}", CURRENT_VERSION))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .context("failed to reach GitHub API")?;

    if !response.status().is_success() {
        bail!(
            "GitHub API returned HTTP {} — releases may not exist yet",
            response.status()
        );
    }

    let json: serde_json::Value = response
        .json()
        .await
        .context("failed to parse GitHub response")?;

    let tag = json["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no tag_name in release response"))?;

    // Strip leading 'v' if present
    let version = tag.strip_prefix('v').unwrap_or(tag);
    Ok(version.to_string())
}

/// Return the expected release asset name for the current platform.
fn release_asset_name() -> String {
    let info = platform::detect_platform_info();
    let (os, arch_str) = match (&info.platform, info.platform.arch()) {
        (platform::Platform::MacOS { .. }, platform::Architecture::Aarch64) => ("macos", "aarch64"),
        (platform::Platform::MacOS { .. }, _) => ("macos", "x86_64"),
        (
            platform::Platform::Linux { .. } | platform::Platform::Wsl { .. },
            platform::Architecture::Aarch64,
        ) => ("linux", "aarch64"),
        _ => ("linux", "x86_64"),
    };
    format!("great-{}-{}", os, arch_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_asset_name_format() {
        let name = release_asset_name();
        assert!(
            name.starts_with("great-"),
            "should start with 'great-': {}",
            name
        );
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3, "should have 3 parts: {}", name);
        assert!(
            ["linux", "macos"].contains(&parts[1]),
            "OS should be linux or macos: {}",
            name
        );
        assert!(
            ["x86_64", "aarch64"].contains(&parts[2]),
            "arch should be x86_64 or aarch64: {}",
            name
        );
    }

    #[test]
    fn test_release_asset_name_stable() {
        // Calling twice should return the same result
        let a = release_asset_name();
        let b = release_asset_name();
        assert_eq!(a, b);
    }
}
