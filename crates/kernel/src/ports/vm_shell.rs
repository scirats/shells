use std::path::Path;

use async_trait::async_trait;

use crate::domain::{ShellName, GUEST_STAGE_DIR};
use crate::error::VmError;

#[async_trait]
pub trait VmShell: Send + Sync {
    async fn run(&self, name: &ShellName, command: &[String]) -> Result<(), VmError>;
    async fn open(&self, name: &ShellName) -> Result<(), VmError>;
    async fn copy_in(&self, name: &ShellName, source: &Path, target: &str) -> Result<(), VmError>;

    async fn stage(&self, name: &ShellName, source: &Path) -> Result<(), VmError> {
        self.run(name, &args(&["rm", "-rf", GUEST_STAGE_DIR]))
            .await?;
        self.run(name, &args(&["mkdir", "-p", GUEST_STAGE_DIR]))
            .await?;
        self.copy_in(name, &source.join("."), &format!("{GUEST_STAGE_DIR}/"))
            .await
    }

    async fn clean_stage(&self, name: &ShellName) -> Result<(), VmError> {
        self.run(name, &args(&["rm", "-rf", GUEST_STAGE_DIR])).await
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| part.to_string()).collect()
}
