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
#[derive(Debug, Args)]
struct ResolveLockfileConflictCommand {}

impl ResolveLockfileConflictCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // TODO

            bail!("not implemented")
        })
        .await
        .context("failed to install auto-completion")
    }
}
