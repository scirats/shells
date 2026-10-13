use async_trait::async_trait;

use crate::domain::{Profile, ShellName};
use crate::error::StoreError;

#[async_trait]
pub trait ProfileStore: Send + Sync {
    async fn get(&self, name: &ShellName) -> Result<Option<Profile>, StoreError>;
    async fn put(&self, name: &ShellName, profile: &Profile) -> Result<(), StoreError>;
    async fn delete(&self, name: &ShellName) -> Result<(), StoreError>;
    async fn list(&self) -> Result<Vec<(ShellName, Profile)>, StoreError>;
}
