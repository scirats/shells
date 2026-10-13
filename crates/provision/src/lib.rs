use std::sync::Arc;

use async_trait::async_trait;
use host::{ensure_dir, HostLayout};
use kernel::{ProvisionError, Provisioner, Shell, VmShell, GUEST_STAGE_DIR, REQUIRED_PACKAGES};
use text::Template;

pub struct SystemProvisioner {
    layout: HostLayout,
    shell: Arc<dyn VmShell>,
}

impl SystemProvisioner {
    pub fn new(layout: HostLayout, shell: Arc<dyn VmShell>) -> Self {
        Self { layout, shell }
    }

    fn write_bootstrap(&self, shell: &Shell) -> Result<(), ProvisionError> {
        ensure_dir(&self.layout.shell_dir(shell.name.as_str())).map_err(internal)?;
        let template =
            Template::load(&self.layout.templates_dir().join("bootstrap.sh")).map_err(internal)?;
        std::fs::write(
            self.layout.bootstrap_script(shell.name.as_str()),
            render(&template, shell),
        )
        .map_err(internal)
    }
}

#[async_trait]
impl Provisioner for SystemProvisioner {
    async fn provision(&self, shell: &Shell) -> Result<(), ProvisionError> {
        self.write_bootstrap(shell)?;
        let dir = self.layout.shell_dir(shell.name.as_str());
        self.shell
            .stage(&shell.name, &dir)
            .await
            .map_err(internal)?;
        let script = format!("{GUEST_STAGE_DIR}/bootstrap.sh");
        let result = self
            .shell
            .run(
                &shell.name,
                &["sudo".to_string(), "bash".to_string(), script],
            )
            .await
            .map_err(internal);
        let _ = self.shell.clean_stage(&shell.name).await;
        result
    }
}

fn render(template: &Template, shell: &Shell) -> String {
    let packages = REQUIRED_PACKAGES.join(" ");
    template.render(&[("name", shell.name.as_str()), ("packages", &packages)])
}

fn internal(error: impl std::fmt::Display) -> ProvisionError {
    ProvisionError::Internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{Profile, ShellName};

    const BOOTSTRAP: &str = include_str!("../../../templates/bootstrap.sh");

    fn shell() -> Shell {
        Shell {
            name: ShellName::parse("alpha").unwrap(),
            profile: Profile::default(),
        }
    }

    #[test]
    fn renders_hostname_and_packages() {
        let out = render(&Template::new(BOOTSTRAP), &shell());
        assert!(out.contains("hostnamectl set-hostname alpha"));
        assert!(out.contains("--no-install-recommends curl ca-certificates git rsync zsh"));
        assert!(!out.contains("{{"));
    }
}
