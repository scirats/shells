use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::{Profile, Shell, ShellName, VmPhase};
use crate::error::{CoreError, DomainError, EnvironmentError, ProvisionError, StoreError, VmError};
use crate::ports::{
    BrowserBridge, BrowserSession, ProfileStore, Provisioner, ShellEnvironment, VmProvider, VmShell,
};
use crate::{CreateRequest, ProfileOverrides, ShellService};

#[derive(Default)]
struct FakeStore {
    profiles: Mutex<HashMap<String, Profile>>,
}

#[async_trait]
impl ProfileStore for FakeStore {
    async fn get(&self, name: &ShellName) -> Result<Option<Profile>, StoreError> {
        Ok(self.profiles.lock().unwrap().get(name.as_str()).cloned())
    }

    async fn put(&self, name: &ShellName, profile: &Profile) -> Result<(), StoreError> {
        self.profiles
            .lock()
            .unwrap()
            .insert(name.as_str().to_string(), profile.clone());
        Ok(())
    }

    async fn delete(&self, name: &ShellName) -> Result<(), StoreError> {
        self.profiles.lock().unwrap().remove(name.as_str());
        Ok(())
    }

    async fn list(&self) -> Result<Vec<(ShellName, Profile)>, StoreError> {
        Ok(self
            .profiles
            .lock()
            .unwrap()
            .iter()
            .map(|(name, profile)| (ShellName::parse(name).unwrap(), profile.clone()))
            .collect())
    }
}

#[derive(Default)]
struct FakeVm {
    phases: Mutex<HashMap<String, VmPhase>>,
    created: Mutex<Vec<String>>,
}

#[async_trait]
impl VmProvider for FakeVm {
    async fn phase(&self, name: &ShellName) -> Result<VmPhase, VmError> {
        Ok(self.phase_of(name))
    }

    async fn phases(&self, names: &[ShellName]) -> Result<Vec<VmPhase>, VmError> {
        Ok(names.iter().map(|name| self.phase_of(name)).collect())
    }

    async fn create(&self, shell: &Shell) -> Result<(), VmError> {
        self.created
            .lock()
            .unwrap()
            .push(shell.name.as_str().to_string());
        Ok(())
    }

    async fn start(&self, name: &ShellName) -> Result<(), VmError> {
        self.set(name, VmPhase::Running);
        Ok(())
    }

    async fn stop(&self, name: &ShellName) -> Result<(), VmError> {
        self.set(name, VmPhase::Stopped);
        Ok(())
    }

    async fn force_stop(&self, name: &ShellName) -> Result<(), VmError> {
        self.stop(name).await
    }

    async fn remove(&self, name: &ShellName) -> Result<(), VmError> {
        self.set(name, VmPhase::NotCreated);
        Ok(())
    }
}

impl FakeVm {
    fn phase_of(&self, name: &ShellName) -> VmPhase {
        self.phases
            .lock()
            .unwrap()
            .get(name.as_str())
            .cloned()
            .unwrap_or(VmPhase::NotCreated)
    }

    fn set(&self, name: &ShellName, phase: VmPhase) {
        self.phases
            .lock()
            .unwrap()
            .insert(name.as_str().to_string(), phase);
    }
}

#[derive(Default)]
struct FakeShell;

#[async_trait]
impl VmShell for FakeShell {
    async fn run(&self, _name: &ShellName, _command: &[String]) -> Result<(), VmError> {
        Ok(())
    }

    async fn open(&self, _name: &ShellName) -> Result<(), VmError> {
        Ok(())
    }

    async fn copy_in(
        &self,
        _name: &ShellName,
        _source: &std::path::Path,
        _target: &str,
    ) -> Result<(), VmError> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeEnvironment;

#[async_trait]
impl ShellEnvironment for FakeEnvironment {
    fn shell_dir(&self, name: &ShellName) -> String {
        format!("/tmp/{name}")
    }

    fn dotfiles_dir(&self, name: &ShellName) -> String {
        format!("/tmp/{name}/home")
    }

    async fn prepare(&self, _shell: &Shell) -> Result<(), EnvironmentError> {
        Ok(())
    }

    async fn apply(&self, _shell: &Shell) -> Result<(), EnvironmentError> {
        Ok(())
    }

    fn remove(&self, _name: &ShellName) -> Result<(), EnvironmentError> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeProvisioner;

#[async_trait]
impl Provisioner for FakeProvisioner {
    async fn provision(&self, _shell: &Shell) -> Result<(), ProvisionError> {
        Ok(())
    }
}

#[derive(Default)]
struct FakeBrowser;

impl BrowserBridge for FakeBrowser {
    fn start(&self, _name: &ShellName) -> Option<Box<dyn BrowserSession>> {
        None
    }
}

fn service() -> ShellService {
    ShellService::new(
        Arc::new(FakeStore::default()),
        Arc::new(FakeVm::default()),
        Arc::new(FakeShell),
        Arc::new(FakeEnvironment),
        Arc::new(FakeProvisioner),
        Arc::new(FakeBrowser),
    )
}

fn request(name: &str) -> CreateRequest {
    CreateRequest {
        name: ShellName::parse(name).unwrap(),
        overrides: ProfileOverrides::default(),
    }
}

#[tokio::test]
async fn create_requires_explicit_mounts() {
    let error = service().create(request("alpha")).await.unwrap_err();
    assert!(matches!(error, CoreError::Domain(DomainError::NoWorkspace)));
}

#[tokio::test]
async fn create_persists_the_profile() {
    let service = service();
    let mut req = request("alpha");
    req.overrides.workspace = Some("~/Projects".into());

    service.create(req).await.unwrap();

    let shell = service
        .get(&ShellName::parse("alpha").unwrap())
        .await
        .unwrap();
    assert_eq!(shell.profile.workspace.as_deref(), Some("~/Projects"));
}

#[tokio::test]
async fn create_conflicts_on_existing_shell() {
    let service = service();
    let mut first = request("alpha");
    first.overrides.workspace = Some("~/Projects".into());
    service.create(first).await.unwrap();

    let mut second = request("alpha");
    second.overrides.workspace = Some("~/Projects".into());
    assert!(matches!(
        service.create(second).await.unwrap_err(),
        CoreError::Conflict(_)
    ));
}

#[tokio::test]
async fn list_reports_every_stored_shell() {
    let service = service();
    for name in ["alpha", "beta"] {
        let mut req = request(name);
        req.overrides.workspace = Some("~/Projects".into());
        service.create(req).await.unwrap();
    }

    let mut summaries = service.list().await.unwrap();
    summaries.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].name.as_str(), "alpha");
    assert_eq!(summaries[1].name.as_str(), "beta");
    assert!(summaries.iter().all(|s| s.phase == VmPhase::Stopped));
}
