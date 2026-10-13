use std::process::Command;

pub(crate) struct Terminfo {
    pub(crate) name: String,
    pub(crate) source: String,
}

pub(crate) fn current() -> Option<Terminfo> {
    let name = std::env::var("TERM").ok().filter(|name| !name.is_empty())?;
    let output = Command::new("infocmp").arg("-x").arg(&name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let source = String::from_utf8(output.stdout).ok()?;
    Some(Terminfo { name, source })
}
