use std::path::Path;
use std::process::{Child, Command, Stdio};

const GUARD: &str = "\
ssh \"$@\" </dev/null >/dev/null 2>&1 &
pid=$!
read -r _guard
kill \"$pid\" 2>/dev/null
wait \"$pid\" 2>/dev/null
";

pub struct SshTunnel {
    child: Child,
}

impl SshTunnel {
    pub fn open(
        ssh_config: &Path,
        ssh_host: &str,
        guest_port: u16,
        host_port: u16,
    ) -> std::io::Result<Self> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(GUARD)
            .arg("guard")
            .arg("-F")
            .arg(ssh_config)
            .args(["-o", "ControlMaster=no", "-o", "ControlPath=none"])
            .args(["-o", "BatchMode=yes", "-o", "ExitOnForwardFailure=yes"])
            .arg("-N")
            .arg("-R")
            .arg(format!("127.0.0.1:{guest_port}:127.0.0.1:{host_port}"))
            .arg(ssh_host)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self { child })
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}
