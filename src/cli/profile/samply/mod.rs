use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use self::{cargo::CargoCommand, run::RunCommand};
use crate::util::wrap;

mod cargo;
mod run;

/// Invokes `instruments` from xcode. Works only on macOS.
#[derive(Debug, Args)]
pub(super) struct SamplyCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl SamplyCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            match self.cmd {
                Inner::Run(cmd) => cmd.run(Default::default()).await,
                Inner::Cargo(cmd) => cmd.run().await,
            }
        })
        .await
        .context("failed to run instruments")
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
    Cargo(CargoCommand),
}
