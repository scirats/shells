use anyhow::{bail, Context, Result};
use dialoguer::Confirm;
use kernel::{ShellName, ShellService};
use std::io::IsTerminal;

use crate::cli::{NameArgs, RemoveArgs};

pub async fn start(service: &ShellService, args: NameArgs) -> Result<()> {
    service.start(&ShellName::parse(&args.name)?).await?;
    Ok(())
}

pub async fn stop(service: &ShellService, args: NameArgs) -> Result<()> {
    service.stop(&ShellName::parse(&args.name)?).await?;
    Ok(())
}

pub async fn remove(service: &ShellService, args: RemoveArgs) -> Result<()> {
    if !args.yes {
        if !std::io::stdin().is_terminal() {
            bail!("refusing to remove without confirmation; pass --yes");
        }
        if !confirm(&args.name)? {
            println!("aborted");
            return Ok(());
        }
    }

    service.remove(&ShellName::parse(&args.name)?).await?;
    println!("removed shell '{}'", args.name);
    Ok(())
}

fn confirm(name: &str) -> Result<bool> {
    Confirm::new()
        .with_prompt(format!("Remove shell '{name}' and delete its VM?"))
        .default(false)
        .interact()
        .context("failed to read confirmation")
}
