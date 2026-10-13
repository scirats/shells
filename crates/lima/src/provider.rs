use std::path::{Path, PathBuf};

use async_trait::async_trait;
use host::{ensure_dir, HostLayout};
use kernel::{Shell, ShellName, VmError, VmPhase, VmProvider, VmShell};
use text::Template;

use crate::limactl::Limactl;
use crate::parse::{parse_instances, phase_of, Instance};
use crate::render::{lima_yaml, mounts_block, MountSpec, YamlInput};

const WORKSPACE_MOUNT_POINT: &str = "{{.Home}}/Workspace";

pub struct LimaVmProvider {
    layout: HostLayout,
    limactl: Limactl,
}

impl LimaVmProvider {
    pub fn new(layout: HostLayout) -> Self {
        let limactl = Limactl::new(layout.lima_home());
        Self { layout, limactl }
    }

    fn instances(&self) -> Result<Vec<Instance>, VmError> {
        let output = self
            .limactl
            .capture(&["list".into(), "--format".into(), "json".into()])?;
        parse_instances(String::from_utf8_lossy(&output.stdout).trim())
    }

    fn mounts(&self, shell: &Shell) -> Vec<MountSpec> {
        let mut mounts = Vec::new();
        if let Some(raw) = &shell.profile.workspace {
            let path = self.layout.expand(raw);
            if !path.exists() {
                tracing::warn!(
                    mount = %path.display(),
                    "mount does not exist; Lima will create it on the host as an empty directory"
                );
            }
            mounts.push(MountSpec {
                location: path.display().to_string(),
                mount_point: Some(WORKSPACE_MOUNT_POINT.to_string()),
                writable: true,
            });
        }
        mounts
    }

    fn suppress_default_mounts(&self) -> Result<(), VmError> {
        let path = self
            .layout
            .lima_home()
            .join("_templates")
            .join("_default")
            .join("mounts.yaml");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| VmError::Internal(err.to_string()))?;
        }
        std::fs::write(&path, "mounts: []\n").map_err(|err| VmError::Internal(err.to_string()))
    }

    fn write_yaml(&self, shell: &Shell) -> Result<PathBuf, VmError> {
        ensure_dir(&self.layout.machines_dir())
            .map_err(|err| VmError::Internal(err.to_string()))?;
        let path = self.layout.machine_yaml(shell.name.as_str());
        std::fs::write(&path, self.render(shell)?)
            .map_err(|err| VmError::Internal(err.to_string()))?;
        Ok(path)
    }

    fn render(&self, shell: &Shell) -> Result<String, VmError> {
        let template = Template::load(&self.layout.templates_dir().join("lima.yaml"))
            .map_err(|err| VmError::Internal(err.to_string()))?;
        let profile = &shell.profile;
        let mounts = mounts_block(&self.mounts(shell));
        Ok(lima_yaml(
            &template,
            &YamlInput {
                image: &profile.image,
                cpus: profile.cpus,
                memory: &profile.memory,
                disk: &profile.disk,
                mounts: &mounts,
            },
        ))
    }

    fn login_shell(&self, name: &ShellName) -> Option<String> {
        let script = r#"grep "^$(id -un):" /etc/passwd | cut -d: -f7"#;
        let args: Vec<String> = vec![
            "shell".into(),
            name.as_str().into(),
            "--".into(),
            "sh".into(),
            "-c".into(),
            script.into(),
        ];
        let output = self.limactl.capture(&args).ok()?;
        let shell = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!shell.is_empty()).then_some(shell)
    }
}

#[async_trait]
impl VmProvider for LimaVmProvider {
    async fn phase(&self, name: &ShellName) -> Result<VmPhase, VmError> {
        Ok(phase_of(&self.instances()?, name.as_str()))
    }

    async fn phases(&self, names: &[ShellName]) -> Result<Vec<VmPhase>, VmError> {
        let instances = self.instances()?;
        Ok(names
            .iter()
            .map(|name| phase_of(&instances, name.as_str()))
            .collect())
    }

    async fn create(&self, shell: &Shell) -> Result<(), VmError> {
        self.suppress_default_mounts()?;
        if self.phase(&shell.name).await? != VmPhase::NotCreated {
            self.force_stop(&shell.name).await?;
            self.remove(&shell.name).await?;
        }

        tracing::info!(
            shell = %shell.name,
            "creating Lima instance (the first create downloads the base image)"
        );
        let yaml = self.write_yaml(shell)?;
        let args: Vec<String> = vec![
            "create".into(),
            "--tty=false".into(),
            "--name".into(),
            shell.name.as_str().to_string(),
            yaml.display().to_string(),
        ];
        self.limactl.interactive(&args)
    }

    async fn start(&self, name: &ShellName) -> Result<(), VmError> {
        self.limactl
            .interactive(&["start".into(), "--progress".into(), name.as_str().into()])
    }

    async fn stop(&self, name: &ShellName) -> Result<(), VmError> {
        self.limactl
            .interactive(&["stop".into(), name.as_str().into()])
    }

    async fn force_stop(&self, name: &ShellName) -> Result<(), VmError> {
        let _ = self
            .limactl
            .capture(&["stop".into(), "--force".into(), name.as_str().into()]);
        Ok(())
    }

    async fn remove(&self, name: &ShellName) -> Result<(), VmError> {
        self.limactl
            .capture(&["delete".into(), "--force".into(), name.as_str().into()])?;
        Ok(())
    }
}

#[async_trait]
impl VmShell for LimaVmProvider {
    async fn run(&self, name: &ShellName, command: &[String]) -> Result<(), VmError> {
        let mut args: Vec<String> = vec!["shell".into(), name.as_str().into()];
        if !command.is_empty() {
            args.push("--".into());
            args.extend(command.iter().cloned());
        }
        self.limactl.interactive(&args)
    }

    async fn open(&self, name: &ShellName) -> Result<(), VmError> {
        let mut args: Vec<String> = vec!["shell".into()];
        if let Some(shell) = self.login_shell(name) {
            args.push("--shell".into());
            args.push(shell);
        }
        args.push(name.as_str().into());
        self.limactl.interactive(&args)
    }

    async fn copy_in(&self, name: &ShellName, source: &Path, target: &str) -> Result<(), VmError> {
        let destination = format!("{}:{}", name.as_str(), target);
        self.limactl
            .capture(&[
                "copy".into(),
                "-r".into(),
                source.display().to_string(),
                destination,
            ])
            .map(|_| ())
    }
}
