use serde::{Deserialize, Serialize};

pub const REQUIRED_PACKAGES: &[&str] =
    &["curl", "ca-certificates", "git", "rsync", "zsh", "neovim"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub image: String,
    pub cpus: u32,
    pub memory: String,
    pub disk: String,
    pub workspace: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            image: "ubuntu".to_string(),
            cpus: 4,
            memory: "4GiB".to_string(),
            disk: "40GiB".to_string(),
            workspace: None,
        }
    }
}
