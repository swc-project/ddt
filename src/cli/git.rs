use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};

use crate::util::wrap;

/// Extra commands like auto-completion or self-update.
#[derive(Debug, Args)]
pub struct GitCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl GitCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::ResolveLockfileConflict(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    ResolveLockfileConflict(ResolveLockfileConflictCommand),
}

/// Resolve merge conflicts in the lockfile.
///
/// Note that this command do **not** understand the lockfile.
/// This command simply ignores the conflict and runs some command which can
/// generate the lockfile.
#[derive(Debug, Args)]
struct ResolveLockfileConflictCommand {
    args: Vec<String>,
}

impl ResolveLockfileConflictCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            if self.args.len() != 5 {
                bail!(
                    "The ddt-lockfile merge driver expects 5 arguments. Please ensure that you \
                     configured git driver properly. It should be


                    driver = ddt git resolve-lockfile-conflict %O %A %B %L %P
                     "
                )
            }

            // TODO

            dbg!(&self.args);

            bail!("not implemented")
        })
        .await
        .context("failed to resolve lockfile conflict")
    }
}
