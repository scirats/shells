use crate::domain::{ShellName, VmPhase};
use crate::error::CoreError;

use super::ShellService;

impl ShellService {
    pub async fn enter(&self, name: &ShellName) -> Result<(), CoreError> {
        let shell = self.get(name).await?;
        self.ensure_running(&shell).await?;
        self.environment.apply(&shell).await?;
        let _session = self.browser.start(name);
        self.shell.open(name).await?;
        Ok(())
    }

    pub async fn exec(&self, name: &ShellName, command: &[String]) -> Result<(), CoreError> {
        let shell = self.get(name).await?;
        self.ensure_running(&shell).await?;
        self.shell.run(name, command).await?;
        Ok(())
    }

    pub async fn start(&self, name: &ShellName) -> Result<(), CoreError> {
        self.get(name).await?;
        self.vm.start(name).await?;
        Ok(())
    }

    pub async fn stop(&self, name: &ShellName) -> Result<(), CoreError> {
        self.get(name).await?;
        self.vm.stop(name).await?;
        Ok(())
    }

    pub async fn remove(&self, name: &ShellName) -> Result<(), CoreError> {
        let has_profile = self.store.get(name).await?.is_some();
        let phase = self.vm.phase(name).await?;
        if !has_profile && phase == VmPhase::NotCreated {
            return Err(CoreError::NotFound(format!(
                "shell '{name}' does not exist"
            )));
        }
        if phase != VmPhase::NotCreated {
            self.vm.force_stop(name).await?;
            self.vm.remove(name).await?;
        }
        self.store.delete(name).await?;
        self.environment.remove(name)?;
        Ok(())
    }
}
