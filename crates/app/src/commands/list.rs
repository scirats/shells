use anyhow::Result;
use kernel::{ShellName, ShellService};

use crate::cli::ListArgs;
use crate::render;

pub async fn list(service: &ShellService, args: ListArgs) -> Result<()> {
    match args.name {
        Some(name) => {
            let summary = service.inspect(&ShellName::parse(&name)?).await?;
            println!("{}", render::detail(&summary, service));
        }
        None => {
            let summaries = service.list().await?;
            if summaries.is_empty() {
                println!("no shells yet. Create one with `shell create <name>`.");
                return Ok(());
            }
            println!("{}", render::table(&summaries));
        }
    }
    Ok(())
}
