use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::util::{cargo_build::CargoBuildTarget, ensure_cargo_subcommand};

/// Comamnds to reduce the size of the binary.
#[derive(Debug, Args)]
pub(super) struct BinSizeCommand {
    #[clap(subcommand)]
    cmd: Cmd,
}

impl BinSizeCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Cmd::SelectPerCrate(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    SelectPerCrate(SelectPerCrateCommand),
}

/// Select the optimization level for each crate.
#[derive(Debug, Args)]
struct SelectPerCrateCommand {
    #[clap(flatten)]
    build_target: CargoBuildTarget,
}

impl SelectPerCrateCommand {
    pub async fn run(self) -> Result<()> {
        ensure_cargo_subcommand("bloat")
            .await
            .context("You can install bloat by `cargo install cargo-bloat`")?;

        Ok(())
    }
}
