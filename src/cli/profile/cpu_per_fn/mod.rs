use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use self::{cargo::CargoCommand, run::RunCommand};
use crate::util::wrap;

mod cargo;
mod run;

#[derive(Debug, Args)]
pub(super) struct CpuPerFnCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl CpuPerFnCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            match self.cmd {
                Inner::Run(cmd) => cmd.run(Default::default()).await,
                Inner::Cargo(cmd) => cmd.run().await,
            }
        })
        .await
        .context("failed to create flamegraph")
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
    Cargo(CargoCommand),
}
