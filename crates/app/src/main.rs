mod cli;
mod commands;
mod composition;
mod logging;
mod render;
mod templates;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    logging::init();

    let cli = Cli::parse();
    if let Err(err) = run(cli).await {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let service = composition::build().await?;
    match cli.command {
        Command::Create(args) => commands::create(&service, args).await,
        Command::Enter(args) => commands::enter(&service, args).await,
        Command::Remove(args) => commands::remove(&service, args).await,
        Command::List(args) => commands::list(&service, args).await,
        Command::Start(args) => commands::start(&service, args).await,
        Command::Stop(args) => commands::stop(&service, args).await,
    }
}
