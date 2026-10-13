mod domain;
mod error;
mod ports;
mod service;

pub use domain::{Profile, Shell, ShellName, VmPhase, GUEST_STAGE_DIR, REQUIRED_PACKAGES};
pub use error::{CoreError, DomainError, EnvironmentError, ProvisionError, StoreError, VmError};
pub use ports::{
    BrowserBridge, BrowserSession, ProfileStore, Provisioner, ShellEnvironment, VmProvider, VmShell,
};
pub use service::{CreateRequest, ProfileOverrides, ShellService, ShellSummary};

#[cfg(test)]
mod tests;
