use anyhow::Result;
use clap::{Parser, Subcommand};

use self::{extra::ExtraCommand, git::GitCommand, solve_version::SolveVersionsCommand};

mod extra;
mod git;
mod solve_version;

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    cmd: InnerCmd,
}

impl CliArgs {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            InnerCmd::Git(cmd) => {
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
    Git(GitCommand),
    SolveVersions(SolveVersionsCommand),
    X(ExtraCommand),
}
