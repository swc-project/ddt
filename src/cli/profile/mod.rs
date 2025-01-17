use anyhow::Result;
use clap::{Args, Subcommand};
use samply::SamplyCommand;

use self::{flamegraph::FlamegraphCommand, instruments::InstrumentsCommand};

mod flamegraph;
mod instruments;
mod samply;
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
            Inner::Flamegraph(cmd) => cmd.run().await,
            Inner::Instruments(cmd) => cmd.run().await,
            Inner::Samply(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Flamegraph(FlamegraphCommand),
    Instruments(InstrumentsCommand),
    Samply(SamplyCommand),
}
