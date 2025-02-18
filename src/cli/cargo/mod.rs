mod bin_size;

use self::bin_size::BinSizeCommand;
use anyhow::Result;
use clap::{Args, Subcommand};

/// Some misc comamnds for cargo.
#[derive(Debug, Args)]
pub struct CargoCommand {
    #[clap(subcommand)]
    cmd: Cmd,
}

impl CargoCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Cmd::BinSize(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    BinSize(BinSizeCommand),
}
