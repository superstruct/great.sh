use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    InSync,
    LocalAhead,
    RemoteAhead,
    Diverged,
    Unknown,
}

pub trait SyncProvider {
    fn push(&self) -> anyhow::Result<()>;
    fn pull(&self) -> anyhow::Result<()>;
    fn status(&self) -> anyhow::Result<SyncStatus>;
}
