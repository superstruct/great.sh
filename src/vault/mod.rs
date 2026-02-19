pub trait VaultProvider {
    fn login(&self) -> anyhow::Result<()>;
    fn unlock(&self) -> anyhow::Result<()>;
    fn get(&self, key: &str) -> anyhow::Result<String>;
    fn set(&self, key: &str, value: &str) -> anyhow::Result<()>;
    fn import(&self, path: &str) -> anyhow::Result<()>;
}
