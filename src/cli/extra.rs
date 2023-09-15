use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};

use crate::util::wrap;

/// Extra commands like auto-completion or self-update.
#[derive(Debug, Args)]
pub struct ExtraCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl ExtraCommand {
    pub async fn run(self) -> Result<()> {
        match self.cmd {
            Inner::Completion(cmd) => cmd.run().await,
            Inner::SelfUpdate(cmd) => cmd.run().await,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Inner {
    Completion(CompletionCommand),
    SelfUpdate(SelfUpdateCommand),
}

/// Generate auto-completion scripts for your shell.
#[derive(Debug, Args)]
struct CompletionCommand {}

impl CompletionCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // TODO

            bail!("not implemented")
        })
        .await
        .context("failed to install auto-completion")
    }
}

/// Update to the latest version of the tool.
#[derive(Debug, Args)]
struct SelfUpdateCommand {}

impl SelfUpdateCommand {
    pub async fn run(self) -> Result<()> {
        wrap(async move {
            // TODO

            bail!("not implemented")
        })
        .await
        .context("failed to self-update")
    }
}
