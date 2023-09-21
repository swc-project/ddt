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
            Inner::Lockfile(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    /// Resolves merge conflicts in the lockfile.
    Lockfile(LockfileCommand),
}

///
#[derive(Debug, Args)]
struct LockfileCommand {}
