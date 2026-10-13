use anyhow::Result;
use kernel::{ShellName, ShellService};

use crate::cli::EnterArgs;

pub async fn enter(service: &ShellService, args: EnterArgs) -> Result<()> {
    let name = ShellName::parse(&args.name)?;
    service.enter(&name).await?;
    Ok(())
}
