#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmPhase {
    Running,
    Stopped,
    NotCreated,
    Unknown(String),
}

impl VmPhase {
    pub fn is_running(&self) -> bool {
        matches!(self, VmPhase::Running)
    }

    pub fn label(&self) -> String {
        match self {
            VmPhase::Running => "Running".to_string(),
            VmPhase::Stopped => "Stopped".to_string(),
            VmPhase::NotCreated => "not-created".to_string(),
            VmPhase::Unknown(raw) => raw.clone(),
        }
    }
}
