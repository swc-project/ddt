use anyhow::Result;
use clap::{Parser, Subcommand};

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
    let args = CliArgs::parse();

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
