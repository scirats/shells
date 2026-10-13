mod listener;
mod opener;
mod tunnel;

use std::path::PathBuf;
use std::sync::Arc;

use host::HostLayout;
use kernel::{BrowserBridge, BrowserSession, ShellName};

use listener::Listener;
use tunnel::SshTunnel;

pub use opener::{Opener, SystemOpener};

pub struct LimaBrowserBridge {
    layout: HostLayout,
    opener: Arc<dyn Opener>,
    guest_port: u16,
}

impl LimaBrowserBridge {
    pub fn new(layout: HostLayout, opener: Arc<dyn Opener>, guest_port: u16) -> Self {
        Self {
            layout,
            opener,
            guest_port,
        }
    }

    fn ssh_config(&self, shell: &ShellName) -> PathBuf {
        self.layout
            .lima_home()
            .join(shell.as_str())
            .join("ssh.config")
    }
}

impl BrowserBridge for LimaBrowserBridge {
    fn start(&self, shell: &ShellName) -> Option<Box<dyn BrowserSession>> {
        let ssh_config = self.ssh_config(shell);
        if !ssh_config.is_file() {
            tracing::warn!(shell = %shell, "no SSH config; skipping the browser bridge");
            return None;
        }

        let listener = match Listener::bind(self.opener.clone()) {
            Ok(listener) => listener,
            Err(err) => {
                tracing::warn!(shell = %shell, "cannot listen for the browser bridge: {err}");
                return None;
            }
        };

        let host = format!("lima-{}", shell.as_str());
        let tunnel = match SshTunnel::open(&ssh_config, &host, self.guest_port, listener.port()) {
            Ok(tunnel) => tunnel,
            Err(err) => {
                tracing::warn!(shell = %shell, "cannot tunnel the browser bridge: {err}");
                return None;
            }
        };

        tracing::info!(shell = %shell, "browser bridge ready: guest URLs open on the host");
        Some(Box::new(Session {
            _listener: listener,
            _tunnel: tunnel,
        }))
    }
}

struct Session {
    _listener: Listener,
    _tunnel: SshTunnel,
}

impl BrowserSession for Session {}
