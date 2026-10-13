use async_trait::async_trait;

use crate::domain::{Shell, ShellName, VmPhase};
use crate::error::VmError;

#[async_trait]
pub trait VmProvider: Send + Sync {
    async fn phase(&self, name: &ShellName) -> Result<VmPhase, VmError>;
    async fn phases(&self, names: &[ShellName]) -> Result<Vec<VmPhase>, VmError>;
    async fn create(&self, shell: &Shell) -> Result<(), VmError>;
    async fn start(&self, name: &ShellName) -> Result<(), VmError>;
    async fn stop(&self, name: &ShellName) -> Result<(), VmError>;
    async fn force_stop(&self, name: &ShellName) -> Result<(), VmError>;
    async fn remove(&self, name: &ShellName) -> Result<(), VmError>;
}
