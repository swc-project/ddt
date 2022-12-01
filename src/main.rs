use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::clean::CleanCommand;

mod clean;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Clean(CleanCommand),
}

#[tokio::main]

async fn main() -> Result<()> {
    let args: Args = Args::parse();

    match args.cmd {
        Command::Clean(cmd) => {
            cmd.run().await?;
        }
    }

    Ok(())
}
