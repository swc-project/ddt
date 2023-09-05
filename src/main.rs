use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{clean::CleanCommand, cli::solve_version::SolveVersionsCommand};

mod clean;
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
    }

    Ok(())
}
