use async_trait::async_trait;

use crate::domain::{Shell, ShellName};
use crate::error::EnvironmentError;

#[async_trait]
pub trait ShellEnvironment: Send + Sync {
    fn shell_dir(&self, name: &ShellName) -> String;
    fn dotfiles_dir(&self, name: &ShellName) -> String;
    async fn prepare(&self, shell: &Shell) -> Result<(), EnvironmentError>;
    async fn apply(&self, shell: &Shell) -> Result<(), EnvironmentError>;
    fn remove(&self, name: &ShellName) -> Result<(), EnvironmentError>;
}
