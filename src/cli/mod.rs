use anyhow::Result;
use clap::{Parser, Subcommand};

use self::{clean::CleanCommand, extra::ExtraCommand, solve_version::SolveVersionsCommand};

mod clean;
mod extra;
mod solve_version;

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: InnerCmd,
}

impl CliArgs {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            InnerCmd::Clean(cmd) => {
                cmd.run().await?;
            }
            InnerCmd::SolveVersions(cmd) => {
                cmd.run().await?;
            }
            InnerCmd::X(cmd) => {
                cmd.run().await?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
enum InnerCmd {
    X(ExtraCommand),
    Clean(CleanCommand),
    SolveVersions(SolveVersionsCommand),
}
