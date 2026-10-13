use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HostLayout {
    home: PathBuf,
}

impl HostLayout {
    pub fn discover() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self::with_home(home)
    }

    fn with_home(home: PathBuf) -> Self {
        Self { home }
    }

    fn root(&self) -> PathBuf {
        self.home.join(".local").join("share").join("shell")
    }

    pub fn database(&self) -> PathBuf {
        self.root().join("shell.db")
    }

    pub fn lima_home(&self) -> PathBuf {
        self.root().join("lima")
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.root().join("templates")
    }

    pub fn machines_dir(&self) -> PathBuf {
        self.root().join("machines")
    }

    pub fn machine_yaml(&self, name: &str) -> PathBuf {
        self.machines_dir().join(format!("{name}.yaml"))
    }

    fn shells_dir(&self) -> PathBuf {
        self.root().join("shells")
    }

    pub fn shell_dir(&self, name: &str) -> PathBuf {
        self.shells_dir().join(name)
    }

    pub fn dotfiles_dir(&self, name: &str) -> PathBuf {
        self.shell_dir(name).join("home")
    }

    pub fn apply_script(&self, name: &str) -> PathBuf {
        self.shell_dir(name).join("apply.sh")
    }

    pub fn env_script(&self, name: &str) -> PathBuf {
        self.shell_dir(name).join("env.sh")
    }

    pub fn terminfo_source(&self, name: &str) -> PathBuf {
        self.shell_dir(name).join("terminfo")
    }

    pub fn bootstrap_script(&self, name: &str) -> PathBuf {
        self.shell_dir(name).join("bootstrap.sh")
    }

    pub fn expand(&self, input: &str) -> PathBuf {
        if input == "~" {
            self.home.clone()
        } else if let Some(rest) = input.strip_prefix("~/") {
            self.home.join(rest)
        } else {
            PathBuf::from(input)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> HostLayout {
        HostLayout::with_home(PathBuf::from("/home/u"))
    }

    #[test]
    fn expands_tilde_against_the_home() {
        assert_eq!(layout().expand("~"), PathBuf::from("/home/u"));
        assert_eq!(layout().expand("~/a/b"), PathBuf::from("/home/u/a/b"));
        assert_eq!(layout().expand("/abs"), PathBuf::from("/abs"));
    }

    #[test]
    fn keeps_everything_under_one_root() {
        let layout = layout();
        let root = PathBuf::from("/home/u/.local/share/shell");
        assert_eq!(layout.root(), root);
        assert_eq!(layout.database(), root.join("shell.db"));
        assert_eq!(layout.templates_dir(), root.join("templates"));
        assert_eq!(
            layout.machine_yaml("alpha"),
            root.join("machines/alpha.yaml")
        );
        assert_eq!(layout.shell_dir("alpha"), root.join("shells/alpha"));
    }
}
