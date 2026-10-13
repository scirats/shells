use crate::domain::{Profile, ShellName, VmPhase};
use crate::error::CoreError;

use super::ShellService;

#[derive(Debug, Clone)]
pub struct ShellSummary {
    pub name: ShellName,
    pub profile: Profile,
    pub phase: VmPhase,
}

impl ShellService {
    pub async fn inspect(&self, name: &ShellName) -> Result<ShellSummary, CoreError> {
        let shell = self.get(name).await?;
        let phase = self.vm.phase(name).await?;
        Ok(ShellSummary {
            name: shell.name,
            profile: shell.profile,
            phase,
        })
    }

    pub async fn list(&self) -> Result<Vec<ShellSummary>, CoreError> {
        let entries = self.store.list().await?;
        let names: Vec<ShellName> = entries.iter().map(|(name, _)| name.clone()).collect();
        let phases = self.vm.phases(&names).await?;
        Ok(entries
            .into_iter()
            .zip(phases)
            .map(|((name, profile), phase)| ShellSummary {
                name,
                profile,
                phase,
            })
            .collect())
    }

    pub fn shell_dir(&self, name: &ShellName) -> String {
        self.environment.shell_dir(name)
    }

    pub fn dotfiles_dir(&self, name: &ShellName) -> String {
        self.environment.dotfiles_dir(name)
    }
}
