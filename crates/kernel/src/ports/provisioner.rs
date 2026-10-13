use async_trait::async_trait;

use crate::domain::Shell;
use crate::error::ProvisionError;

#[async_trait]
pub trait Provisioner: Send + Sync {
    async fn provision(&self, shell: &Shell) -> Result<(), ProvisionError>;
}
