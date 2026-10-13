use std::io;
use std::process::{Command, Stdio};

pub trait Opener: Send + Sync {
    fn open(&self, url: &str) -> io::Result<()>;
}

pub struct SystemOpener;

impl Opener for SystemOpener {
    fn open(&self, url: &str) -> io::Result<()> {
        let program = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let status = Command::new(program)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other("the host opener exited with a failure"))
        }
    }
}
