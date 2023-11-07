use anyhow::Result;
use clap::{Args, Subcommand};

use self::solve_semver::SolveVersionCommand;

pub mod solve_semver;

/// Extra commands like auto-completion or self-update.
#[derive(Debug, Args)]
pub struct CargoCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl CargoCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::SolveVersion(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    SolveVersion(SolveVersionCommand),
}
