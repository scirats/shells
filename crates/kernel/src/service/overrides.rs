use crate::domain::Profile;

#[derive(Debug, Default, Clone)]
pub struct ProfileOverrides {
    pub image: Option<String>,
    pub cpus: Option<u32>,
    pub memory: Option<String>,
    pub disk: Option<String>,
    pub workspace: Option<String>,
}

pub(crate) fn apply(mut base: Profile, overrides: &ProfileOverrides) -> Profile {
    if let Some(value) = &overrides.image {
        base.image = value.clone();
    }
    if let Some(value) = overrides.cpus {
        base.cpus = value;
    }
    if let Some(value) = &overrides.memory {
        base.memory = value.clone();
    }
    if let Some(value) = &overrides.disk {
        base.disk = value.clone();
    }
    if let Some(value) = &overrides.workspace {
        base.workspace = Some(value.clone());
    }
    base
}
