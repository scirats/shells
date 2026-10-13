use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid shell name '{0}': use lowercase letters, digits and '-', starting with a letter or digit")]
    InvalidName(String),
    #[error("a workspace is required, e.g. `--workspace ~/Projects`")]
    NoWorkspace,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("storage failure: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("virtual machine failure: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum ProvisionError {
    #[error("provision failure: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum EnvironmentError {
    #[error("environment failure: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Vm(#[from] VmError),
    #[error(transparent)]
    Provision(#[from] ProvisionError),
    #[error(transparent)]
    Environment(#[from] EnvironmentError),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    NotFound(String),
}
