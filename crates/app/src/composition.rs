use std::sync::Arc;

use anyhow::{Context, Result};

use environment::HostGuestEnvironment;
use host::HostLayout;
use kernel::{BrowserBridge, ShellService, VmProvider, VmShell};
use lima::{LimaBrowserBridge, LimaVmProvider, SystemOpener};
use provision::SystemProvisioner;
use store::SqliteProfileStore;

const BROWSER_BRIDGE_PORT: u16 = 2718;

pub async fn build() -> Result<Arc<ShellService>> {
    let layout = HostLayout::discover();
    crate::templates::sync(&layout)?;

    let provider = Arc::new(LimaVmProvider::new(layout.clone()));
    let vm: Arc<dyn VmProvider> = provider.clone();
    let shell: Arc<dyn VmShell> = provider.clone();
    let browser: Arc<dyn BrowserBridge> = Arc::new(LimaBrowserBridge::new(
        layout.clone(),
        Arc::new(SystemOpener),
        BROWSER_BRIDGE_PORT,
    ));

    let store = Arc::new(
        SqliteProfileStore::open(layout.database())
            .await
            .context("cannot open the shell database")?,
    );
    let environment = Arc::new(HostGuestEnvironment::new(
        layout.clone(),
        shell.clone(),
        BROWSER_BRIDGE_PORT,
    ));
    let provisioner = Arc::new(SystemProvisioner::new(layout, shell.clone()));
    Ok(Arc::new(ShellService::new(
        store,
        vm,
        shell,
        environment,
        provisioner,
        browser,
    )))
}
