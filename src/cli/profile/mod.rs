mod instruments;

use anyhow::Result;
use clap::{Args, Subcommand};

use self::instruments::InstrumentsCommand;

/// Profiles performance
#[derive(Debug, Args)]
pub struct ProfileCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl ProfileCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::Instruments(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Instruments(InstrumentsCommand),
}
