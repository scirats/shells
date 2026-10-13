use crate::domain::{Profile, Shell, ShellName};
use crate::error::{CoreError, DomainError};

use super::overrides::apply;
use super::{ProfileOverrides, ShellService};

#[derive(Debug, Clone)]
pub struct CreateRequest {
    pub name: ShellName,
    pub overrides: ProfileOverrides,
}

impl ShellService {
    pub async fn create(&self, request: CreateRequest) -> Result<(), CoreError> {
        self.ensure_absent(&request.name).await?;
        Self::ensure_workspace(&request)?;

        let shell = Shell {
            name: request.name.clone(),
            profile: apply(Profile::default(), &request.overrides),
        };

        self.store.put(&shell.name, &shell.profile).await?;
        self.environment.prepare(&shell).await?;
        self.vm.create(&shell).await?;
        self.vm.start(&shell.name).await?;
        self.provisioner.provision(&shell).await?;
        self.vm.stop(&shell.name).await?;
        Ok(())
    }

    async fn ensure_absent(&self, name: &ShellName) -> Result<(), CoreError> {
        if self.store.get(name).await?.is_some() {
            return Err(CoreError::Conflict(format!(
                "shell '{name}' already exists"
            )));
        }
        Ok(())
    }

    fn ensure_workspace(request: &CreateRequest) -> Result<(), CoreError> {
        if request.overrides.workspace.is_none() {
            return Err(DomainError::NoWorkspace.into());
        }
        Ok(())
    }
}
