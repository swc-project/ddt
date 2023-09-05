use anyhow::Result;
use clap::{Args, Subcommand};

/// Extra commands like auto-completion or self-update.
#[derive(Debug, Args)]
pub struct ExtraCommand {
    #[clap(subcommand)]
    cmd: Inner,
}

impl ExtraCommand {
    pub async fn run(self) -> Result<()> {
        //

        Ok(())
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

/// Update to the latest version of the tool.
#[derive(Debug, Args)]
struct SelfUpdateCommand {}
