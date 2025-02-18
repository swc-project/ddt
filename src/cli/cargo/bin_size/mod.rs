use anyhow::Result;
use clap::{Args, Subcommand};

/// Comamnds to reduce the size of the binary.
#[derive(Debug, Args)]
pub(super) struct BinSizeCommand {
    #[clap(subcommand)]
    cmd: Cmd,
}

impl BinSizeCommand {
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    SelectPerCrate(SelectPerCrateCommand),
}

/// Select the optimization level for each crate.
#[derive(Debug, Args)]
struct SelectPerCrateCommand {}
