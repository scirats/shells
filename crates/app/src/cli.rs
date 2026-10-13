use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "shell",
    version,
    about = "Shell manager: isolated Lima environments over your host files"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Create a new shell (profile + VM).")]
    Create(CreateArgs),
    #[command(about = "Enter a shell (open an interactive session).")]
    Enter(EnterArgs),
    #[command(about = "Remove a shell and its VM.")]
    Remove(RemoveArgs),
    #[command(about = "List shells and their state (or show details for one).")]
    List(ListArgs),
    #[command(about = "Start a shell's VM.")]
    Start(NameArgs),
    #[command(about = "Stop a shell's VM.")]
    Stop(NameArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(help = "Shell name (lowercase letters, digits and '-').")]
    pub name: String,
    #[arg(long, help = "Lima template (e.g. ubuntu, debian).")]
    pub image: Option<String>,
    #[arg(long, help = "Number of vCPUs.")]
    pub cpus: Option<u32>,
    #[arg(long, help = "Memory, e.g. 4GiB.")]
    pub memory: Option<String>,
    #[arg(long, help = "Disk size, e.g. 40GiB.")]
    pub disk: Option<String>,
    #[arg(
        long = "workspace",
        help = "Host directory to expose, writable. It appears inside the guest at ~/Workspace. Required."
    )]
    pub workspace: Option<String>,
}

#[derive(Debug, Args)]
pub struct EnterArgs {
    #[arg(help = "Shell name.")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    #[arg(help = "Shell name.")]
    pub name: String,
    #[arg(long, short = 'y', help = "Do not ask for confirmation.")]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(help = "Show full details for a single shell instead of the summary table.")]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct NameArgs {
    #[arg(help = "Shell name.")]
    pub name: String,
}
