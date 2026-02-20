use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Status of sync between local and remote.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SyncStatus {
    InSync,
    LocalNewer,
    RemoteNewer,
    Conflict,
    NeverSynced,
}

/// Encrypted blob for sync transport.
#[derive(Debug)]
#[allow(dead_code)]
pub struct SyncBlob {
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// Get the local sync directory (~/.local/share/great/sync/).
pub fn sync_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine local data directory"))?;
    let sync_dir = data_dir.join("great").join("sync");
    Ok(sync_dir)
}

/// Export current config as bytes for sync.
pub fn export_config(config_path: &Path) -> Result<Vec<u8>> {
    let content = std::fs::read(config_path)
        .context(format!("failed to read {}", config_path.display()))?;
    Ok(content)
}

/// Import config from bytes.
#[allow(dead_code)]
pub fn import_config(data: &[u8], config_path: &Path) -> Result<()> {
    std::fs::write(config_path, data)
        .context(format!("failed to write {}", config_path.display()))?;
    Ok(())
}

/// Save a sync blob to local storage.
pub fn save_local(data: &[u8]) -> Result<PathBuf> {
    let dir = sync_dir()?;
    std::fs::create_dir_all(&dir)
        .context("failed to create sync directory")?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = format!("sync-{}.bin", timestamp);
    let path = dir.join(&filename);

    std::fs::write(&path, data)
        .context("failed to write sync blob")?;

    // Also update the "latest" symlink/copy
    let latest = dir.join("latest.bin");
    std::fs::write(&latest, data)
        .context("failed to write latest sync blob")?;

    Ok(path)
}

/// Load the latest sync blob from local storage.
pub fn load_local() -> Result<Option<Vec<u8>>> {
    let dir = sync_dir()?;
    let latest = dir.join("latest.bin");

    if !latest.exists() {
        return Ok(None);
    }

    let data = std::fs::read(&latest)
        .context("failed to read latest sync blob")?;
    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_dir_returns_path() {
        let result = sync_dir();
        assert!(result.is_ok(), "sync_dir() should return Ok");
        let path = result.unwrap();
        assert!(
            path.ends_with("great/sync"),
            "sync_dir() path should end with 'great/sync', got: {}",
            path.display()
        );
    }

    #[test]
    fn test_export_config_reads_file() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let file_path = tmp.path().join("test-config.toml");
        let content = b"[tool]\nname = \"great\"\n";
        std::fs::write(&file_path, content).expect("failed to write test file");

        let result = export_config(&file_path);
        assert!(result.is_ok(), "export_config should succeed for existing file");
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_export_config_missing_file_errors() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let missing = tmp.path().join("nonexistent.toml");

        let result = export_config(&missing);
        assert!(
            result.is_err(),
            "export_config should return Err for a nonexistent file"
        );
    }

    #[test]
    fn test_import_export_roundtrip() {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let original = tmp.path().join("original.toml");
        let restored = tmp.path().join("restored.toml");

        let content = b"[settings]\ntheme = \"dark\"\n";
        std::fs::write(&original, content).expect("failed to write original file");

        // Export from the original file
        let exported = export_config(&original).expect("export_config failed");

        // Import into a new location
        import_config(&exported, &restored).expect("import_config failed");

        // Verify the restored file matches
        let restored_content = std::fs::read(&restored).expect("failed to read restored file");
        assert_eq!(
            restored_content, content,
            "imported config should match the original content"
        );
    }

    #[test]
    fn test_load_local_no_data_returns_ok() {
        // load_local reads from the real sync_dir(). If latest.bin does not
        // exist there, it should return Ok(None). If it does exist (from a
        // previous run), it should return Ok(Some(_)). Either way, it must
        // not panic.
        let result = load_local();
        assert!(
            result.is_ok(),
            "load_local() should return Ok even when no data has been saved"
        );
    }
}
