use crate::clean::CleanCommand;
use anyhow::Result;
use clap::{Parser, Subcommand};

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
    let args = Args::parse();

    println!("Hello, world!");

    Ok(())
}
