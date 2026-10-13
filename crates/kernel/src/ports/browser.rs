use crate::domain::ShellName;

pub trait BrowserBridge: Send + Sync {
    fn start(&self, shell: &ShellName) -> Option<Box<dyn BrowserSession>>;
}

pub trait BrowserSession: Send {}
