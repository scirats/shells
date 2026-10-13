use super::{Profile, ShellName};

#[derive(Debug, Clone)]
pub struct Shell {
    pub name: ShellName,
    pub profile: Profile,
}
