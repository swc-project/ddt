use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

use crate::cli::{CleanCommand, ExtraCommand, SolveVersionsCommand};

mod cli;
mod semver;
mod util;

#[derive(Debug, Parser)]
struct CliArgs {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    X(ExtraCommand),
    Clean(CleanCommand),
    SolveVersions(SolveVersionsCommand),
}

#[tokio::main]

async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .pretty()
        .init();

    let args = CliArgs::parse();

    info!("Start");

    match args.cmd {
        Command::Clean(cmd) => {
            cmd.run().await?;
        }
        Command::SolveVersions(cmd) => {
            cmd.run().await?;
        }
        Command::X(cmd) => {
            cmd.run().await?;
        }
    }

    Ok(())
}
