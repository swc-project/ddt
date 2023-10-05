use anyhow::Result;
use clap::{Args, Subcommand};

use self::{flamegraph::FlamegraphCommand, instruments::InstrumentsCommand};

mod cpu_per_fn;
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
            Inner::Flamegraph(cmd) => cmd.run().await,
            Inner::Instruments(cmd) => cmd.run().await,
            Inner::CpuPerFn(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Flamegraph(FlamegraphCommand),
    Instruments(InstrumentsCommand),
    CpuPerFn(CpuPerFnCommand),
}
