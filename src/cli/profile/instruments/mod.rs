use anyhow::Result;
use clap::{Args, Subcommand};

use self::run::RunCommand;

mod run;
mod util;

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
