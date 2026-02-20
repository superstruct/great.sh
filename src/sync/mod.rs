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
