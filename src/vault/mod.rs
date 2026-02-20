use anyhow::{bail, Context, Result};

/// Trait for secret providers. Object-safe.
pub trait SecretProvider {
    /// Name of this provider.
    fn name(&self) -> &str;

    /// Check if this provider is available on the system.
    fn is_available(&self) -> bool;

    /// Get a secret by key.
    fn get(&self, key: &str) -> Result<Option<String>>;

    /// Set a secret.
    fn set(&self, key: &str, value: &str) -> Result<()>;

    /// List all keys (or keys with a given prefix).
    fn list(&self, prefix: Option<&str>) -> Result<Vec<String>>;
}

// -------------------------------------------------------------------
// Environment Variable Provider (simplest)
// -------------------------------------------------------------------

/// Reads secrets from environment variables. Always available.
pub struct EnvProvider;

impl SecretProvider for EnvProvider {
    fn name(&self) -> &str {
        "env"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        match std::env::var(key) {
            Ok(val) => Ok(Some(val)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set(&self, _key: &str, _value: &str) -> Result<()> {
        bail!("Cannot persist secrets to environment variables. Set them in your shell profile or use a different provider.");
    }

    fn list(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let vars: Vec<String> = std::env::vars()
            .filter_map(|(k, _)| {
                if let Some(p) = prefix {
                    if k.starts_with(p) {
                        Some(k)
                    } else {
                        None
                    }
                } else {
                    // Return only likely secret keys (contain KEY, TOKEN, SECRET, PASSWORD)
                    if k.contains("KEY")
                        || k.contains("TOKEN")
                        || k.contains("SECRET")
                        || k.contains("PASSWORD")
                    {
                        Some(k)
                    } else {
                        None
                    }
                }
            })
            .collect();
        Ok(vars)
    }
}

// -------------------------------------------------------------------
// 1Password CLI Provider
// -------------------------------------------------------------------

/// Reads secrets from 1Password via the `op` CLI.
pub struct OnePasswordProvider;

impl SecretProvider for OnePasswordProvider {
    fn name(&self) -> &str {
        "1password"
    }

    fn is_available(&self) -> bool {
        crate::platform::command_exists("op")
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        let output = std::process::Command::new("op")
            .args(["read", key])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .context("failed to run `op read`")?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn set(&self, _key: &str, _value: &str) -> Result<()> {
        bail!("Writing to 1Password is not yet supported. Use the 1Password app.");
    }

    fn list(&self, _prefix: Option<&str>) -> Result<Vec<String>> {
        bail!("Listing 1Password items requires interactive authentication");
    }
}

// -------------------------------------------------------------------
// Bitwarden CLI Provider
// -------------------------------------------------------------------

/// Reads secrets from Bitwarden via the `bw` CLI.
pub struct BitwardenProvider;

impl SecretProvider for BitwardenProvider {
    fn name(&self) -> &str {
        "bitwarden"
    }

    fn is_available(&self) -> bool {
        crate::platform::command_exists("bw")
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        let output = std::process::Command::new("bw")
            .args(["get", "password", key])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .context("failed to run `bw get`")?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn set(&self, _key: &str, _value: &str) -> Result<()> {
        bail!("Writing to Bitwarden is not yet supported. Use the Bitwarden CLI.");
    }

    fn list(&self, _prefix: Option<&str>) -> Result<Vec<String>> {
        bail!("Listing Bitwarden items requires interactive authentication");
    }
}

// -------------------------------------------------------------------
// System Keychain Provider (macOS Keychain / Linux secret-tool)
// -------------------------------------------------------------------

/// Stores and retrieves secrets via the OS keychain (macOS `security` or
/// Linux `secret-tool`).
pub struct KeychainProvider;

impl SecretProvider for KeychainProvider {
    fn name(&self) -> &str {
        "keychain"
    }

    fn is_available(&self) -> bool {
        crate::platform::command_exists("security")
            || crate::platform::command_exists("secret-tool")
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        if cfg!(target_os = "macos") {
            let output = std::process::Command::new("security")
                .args(["find-generic-password", "-s", "great-sh", "-a", key, "-w"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .context("failed to read from macOS Keychain")?;

            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(Some(value))
            } else {
                Ok(None)
            }
        } else {
            // Linux: secret-tool
            let output = std::process::Command::new("secret-tool")
                .args(["lookup", "application", "great-sh", "key", key])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .context("failed to read from system keyring")?;

            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(Some(value))
            } else {
                Ok(None)
            }
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        if cfg!(target_os = "macos") {
            let status = std::process::Command::new("security")
                .args([
                    "add-generic-password",
                    "-s",
                    "great-sh",
                    "-a",
                    key,
                    "-w",
                    value,
                    "-U",
                ])
                .status()
                .context("failed to write to macOS Keychain")?;
            if !status.success() {
                bail!("Failed to store secret in macOS Keychain");
            }
        } else {
            // Linux: secret-tool
            let mut child = std::process::Command::new("secret-tool")
                .args([
                    "store",
                    "--label",
                    &format!("great-sh: {}", key),
                    "application",
                    "great-sh",
                    "key",
                    key,
                ])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .context("failed to run secret-tool")?;

            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin
                    .write_all(value.as_bytes())
                    .context("failed to write secret to secret-tool")?;
            }

            let status = child.wait().context("secret-tool failed")?;
            if !status.success() {
                bail!("Failed to store secret in system keyring");
            }
        }
        Ok(())
    }

    fn list(&self, _prefix: Option<&str>) -> Result<Vec<String>> {
        // Keychain listing is complex -- return empty for now
        Ok(Vec::new())
    }
}

// -------------------------------------------------------------------
// Provider Discovery
// -------------------------------------------------------------------

/// Get all available secret providers, ordered by preference.
///
/// The order is: system keychain, 1Password, Bitwarden, environment variables.
/// Only providers that report themselves as available are included, except for
/// `EnvProvider` which is always appended as the lowest-priority fallback.
pub fn available_providers() -> Vec<Box<dyn SecretProvider>> {
    let mut providers: Vec<Box<dyn SecretProvider>> = Vec::new();

    let keychain = KeychainProvider;
    if keychain.is_available() {
        providers.push(Box::new(keychain));
    }

    let op = OnePasswordProvider;
    if op.is_available() {
        providers.push(Box::new(op));
    }

    let bw = BitwardenProvider;
    if bw.is_available() {
        providers.push(Box::new(bw));
    }

    // Env is always available, but lowest priority
    providers.push(Box::new(EnvProvider));

    providers
}

/// Get a specific provider by name.
///
/// Valid names: `"env"`, `"1password"`, `"bitwarden"`, `"keychain"`.
/// Returns `None` for unrecognised names.
pub fn get_provider(name: &str) -> Option<Box<dyn SecretProvider>> {
    match name {
        "env" => Some(Box::new(EnvProvider)),
        "1password" => Some(Box::new(OnePasswordProvider)),
        "bitwarden" => Some(Box::new(BitwardenProvider)),
        "keychain" => Some(Box::new(KeychainProvider)),
        _ => None,
    }
}

// -------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_provider_get_existing_var() {
        // HOME is set on every Unix system
        let provider = EnvProvider;
        let result = provider.get("HOME").expect("get should not fail");
        assert!(result.is_some(), "HOME should be set");
        assert!(!result.unwrap().is_empty(), "HOME should be non-empty");
    }

    #[test]
    fn env_provider_get_missing_var() {
        let provider = EnvProvider;
        let result = provider
            .get("GREAT_SH_TOTALLY_NONEXISTENT_VAR_12345")
            .expect("get should not fail");
        assert!(result.is_none(), "missing var should return None");
    }

    #[test]
    fn env_provider_set_returns_error() {
        let provider = EnvProvider;
        let result = provider.set("FOO", "bar");
        assert!(result.is_err(), "env provider set should return an error");
    }

    #[test]
    fn env_provider_list_with_prefix() {
        // Set a known env var so we can find it
        std::env::set_var("GREAT_TEST_PREFIX_ABC", "1");
        let provider = EnvProvider;
        let keys = provider
            .list(Some("GREAT_TEST_PREFIX_"))
            .expect("list should not fail");
        assert!(
            keys.contains(&"GREAT_TEST_PREFIX_ABC".to_string()),
            "should find the test var"
        );
        std::env::remove_var("GREAT_TEST_PREFIX_ABC");
    }

    #[test]
    fn trait_object_safety() {
        // Verify that SecretProvider can be used as a trait object
        let provider: Box<dyn SecretProvider> = Box::new(EnvProvider);
        assert_eq!(provider.name(), "env");
        assert!(provider.is_available());
    }

    #[test]
    fn available_providers_always_includes_env() {
        let providers = available_providers();
        assert!(!providers.is_empty(), "should have at least one provider");
        // The last provider should always be env
        let last = providers.last().expect("non-empty");
        assert_eq!(last.name(), "env");
    }

    #[test]
    fn get_provider_known_names() {
        assert!(get_provider("env").is_some());
        assert!(get_provider("1password").is_some());
        assert!(get_provider("bitwarden").is_some());
        assert!(get_provider("keychain").is_some());
    }

    #[test]
    fn get_provider_unknown_name_returns_none() {
        assert!(get_provider("nonexistent").is_none());
        assert!(get_provider("").is_none());
    }
}
