use anyhow::Result;
use kernel::{CreateRequest, ProfileOverrides, ShellName, ShellService};

use crate::cli::CreateArgs;

pub async fn create(service: &ShellService, args: CreateArgs) -> Result<()> {
    let request = CreateRequest {
        name: ShellName::parse(&args.name)?,
        overrides: overrides(&args),
    };

    service.create(request).await?;
    println!(
        "created and provisioned shell '{}' (powered off). Start it with `shell start {}`.",
        args.name, args.name
    );
    Ok(())
}

fn overrides(args: &CreateArgs) -> ProfileOverrides {
    ProfileOverrides {
        image: args.image.clone(),
        cpus: args.cpus,
        memory: args.memory.clone(),
        disk: args.disk.clone(),
        workspace: args.workspace.clone(),
    }
}
