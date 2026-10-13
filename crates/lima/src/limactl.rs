use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use host::ensure_dir;
use kernel::VmError;

pub(crate) struct Limactl {
    binary: PathBuf,
    lima_home: PathBuf,
}

impl Limactl {
    pub(crate) fn new(lima_home: PathBuf) -> Self {
        Self {
            binary: resolve_binary(),
            lima_home,
        }
    }

    pub(crate) fn capture(&self, args: &[String]) -> Result<Output, VmError> {
        self.ensure_home()?;
        let output = self
            .command()
            .args(args)
            .output()
            .map_err(|err| VmError::Internal(format!("cannot run limactl: {err}")))?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(self.failure(args, &output))
        }
    }

    pub(crate) fn interactive(&self, args: &[String]) -> Result<(), VmError> {
        self.ensure_home()?;
        let status = self
            .command()
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|err| VmError::Internal(format!("cannot run limactl: {err}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(VmError::Internal(format!(
                "`limactl {}` exited with {status}",
                args.join(" ")
            )))
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command.env("LIMA_HOME", &self.lima_home);
        command
    }

    fn ensure_home(&self) -> Result<(), VmError> {
        ensure_dir(&self.lima_home).map_err(|err| VmError::Internal(err.to_string()))
    }

    fn failure(&self, args: &[String], output: &Output) -> VmError {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut message = format!("`limactl {}` failed:\n{}", args.join(" "), stderr.trim());
        if !stdout.trim().is_empty() {
            message.push_str(&format!("\n{}", stdout.trim()));
        }
        VmError::Internal(message)
    }
}

fn resolve_binary() -> PathBuf {
    if let Some(explicit) = std::env::var_os("SHELL_LIMACTL") {
        return PathBuf::from(explicit);
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("limactl");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    for dir in ["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"] {
        let candidate = PathBuf::from(dir).join("limactl");
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("limactl")
}
