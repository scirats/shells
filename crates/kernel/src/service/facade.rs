use std::sync::Arc;

use crate::domain::{Shell, ShellName};
use crate::error::CoreError;
use crate::ports::{
    BrowserBridge, ProfileStore, Provisioner, ShellEnvironment, VmProvider, VmShell,
};

#[derive(Clone)]
pub struct ShellService {
    pub(crate) store: Arc<dyn ProfileStore>,
    pub(crate) vm: Arc<dyn VmProvider>,
    pub(crate) shell: Arc<dyn VmShell>,
    pub(crate) environment: Arc<dyn ShellEnvironment>,
    pub(crate) provisioner: Arc<dyn Provisioner>,
    pub(crate) browser: Arc<dyn BrowserBridge>,
}

impl ShellService {
    pub fn new(
        store: Arc<dyn ProfileStore>,
        vm: Arc<dyn VmProvider>,
        shell: Arc<dyn VmShell>,
        environment: Arc<dyn ShellEnvironment>,
        provisioner: Arc<dyn Provisioner>,
        browser: Arc<dyn BrowserBridge>,
    ) -> Self {
        Self {
            store,
            vm,
            shell,
            environment,
            provisioner,
            browser,
        }
    }

    pub(crate) async fn get(&self, name: &ShellName) -> Result<Shell, CoreError> {
        let profile = self.store.get(name).await?.ok_or_else(|| {
            CoreError::NotFound(format!(
                "shell '{name}' does not exist (try `shell create {name}`)"
            ))
        })?;
        Ok(Shell {
            name: name.clone(),
            profile,
        })
    }

    pub(crate) async fn ensure_running(&self, shell: &Shell) -> Result<(), CoreError> {
        if !self.vm.phase(&shell.name).await?.is_running() {
            self.vm.start(&shell.name).await?;
        }
        Ok(())
    }
}
