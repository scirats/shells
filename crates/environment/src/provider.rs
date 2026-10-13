use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use host::{ensure_dir, HostLayout};
use kernel::{EnvironmentError, Shell, ShellEnvironment, ShellName, VmShell, GUEST_STAGE_DIR};
use text::Template;

use crate::render::{apply_script, env_script};

pub struct HostGuestEnvironment {
    layout: HostLayout,
    shell: Arc<dyn VmShell>,
    browser_port: u16,
}

impl HostGuestEnvironment {
    pub fn new(layout: HostLayout, shell: Arc<dyn VmShell>, browser_port: u16) -> Self {
        Self {
            layout,
            shell,
            browser_port,
        }
    }

    fn template(&self, name: &str) -> Result<Template, EnvironmentError> {
        Template::load(&self.layout.templates_dir().join(name)).map_err(internal)
    }

    fn write_script(&self, path: PathBuf, content: String) -> Result<(), EnvironmentError> {
        std::fs::write(path, content).map_err(internal)
    }
}

#[async_trait]
impl ShellEnvironment for HostGuestEnvironment {
    fn shell_dir(&self, name: &ShellName) -> String {
        self.layout.shell_dir(name.as_str()).display().to_string()
    }

    fn dotfiles_dir(&self, name: &ShellName) -> String {
        self.layout
            .dotfiles_dir(name.as_str())
            .display()
            .to_string()
    }

    async fn prepare(&self, shell: &Shell) -> Result<(), EnvironmentError> {
        let name = shell.name.as_str();
        let dir = self.layout.shell_dir(name);
        ensure_dir(&dir).map_err(internal)?;
        ensure_dir(&self.layout.dotfiles_dir(name)).map_err(internal)?;

        let env = self.template("env.sh")?;
        self.write_script(self.layout.env_script(name), env_script(&env, shell))?;

        let apply = self.template("apply.sh")?;
        self.write_script(
            self.layout.apply_script(name),
            apply_script(&apply, self.browser_port),
        )?;
        Ok(())
    }

    async fn apply(&self, shell: &Shell) -> Result<(), EnvironmentError> {
        self.prepare(shell).await?;
        let dir = self.layout.shell_dir(shell.name.as_str());
        self.shell
            .stage(&shell.name, &dir)
            .await
            .map_err(internal)?;
        let script = format!("{GUEST_STAGE_DIR}/apply.sh");
        let result = self
            .shell
            .run(&shell.name, &["bash".to_string(), script])
            .await
            .map_err(internal);
        let _ = self.shell.clean_stage(&shell.name).await;
        result
    }

    fn remove(&self, name: &ShellName) -> Result<(), EnvironmentError> {
        let dir = self.layout.shell_dir(name.as_str());
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(internal)?;
        }
        Ok(())
    }
}

fn internal(error: impl std::fmt::Display) -> EnvironmentError {
    EnvironmentError::Internal(error.to_string())
}
