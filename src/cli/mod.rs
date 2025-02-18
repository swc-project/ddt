use self::cargo::CargoCommand;
use anyhow::Result;
use clap::{Parser, Subcommand};

use self::{extra::ExtraCommand, git::GitCommand, profile::ProfileCommand};

mod cargo;
mod extra;
mod git;
mod profile;
mod util;

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: InnerCmd,
}

impl CliArgs {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            InnerCmd::Profile(cmd) => {
                cmd.run().await?;
            }
            InnerCmd::Git(cmd) => {
                cmd.run().await?;
            }
            InnerCmd::X(cmd) => {
                cmd.run().await?;
            }
            InnerCmd::Cargo(cmd) => {
                cmd.run().await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
enum InnerCmd {
    Cargo(CargoCommand),
    Profile(ProfileCommand),
    Git(GitCommand),
    X(ExtraCommand),
}
