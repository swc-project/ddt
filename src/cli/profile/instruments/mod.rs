use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};

use crate::util::wrap;

mod run;

/// Invokes `instruments` from xcode. Works only on macOS.
#[derive(Debug, Args)]
pub(super) struct InstrumentsCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl InstrumentsCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::Run(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Run(RunCommand),
}

/// Invoke a binary file under the `instruments` tool.
#[derive(Debug, Args)]
struct RunCommand {}

impl RunCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // TODO

            bail!("not implemented")
        })
        .await
        .context("failed to run instruments with a specified binary")
    }
}
