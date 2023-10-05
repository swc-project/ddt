use anyhow::Result;
use clap::{Args, Subcommand};

use self::{flamegraph::FlamegraphCommand, instruments::InstrumentsCommand};

mod flamegraph;
mod instruments;
mod util;

/// Profiles performance
#[derive(Debug, Args)]
pub struct ProfileCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl ProfileCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::FlamegraphCommand(cmd) => cmd.run().await,
            Inner::Instruments(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    FlamegraphCommand(FlamegraphCommand),
    Instruments(InstrumentsCommand),
}
