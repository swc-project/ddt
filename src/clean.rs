use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub(crate) struct CleanCommand {
    #[clap(short, long)]
    dry_run: bool,

    /// The directory to clean.
    dir: PathBuf,
}

impl CleanCommand {
    pub async fn run(&self) -> Result<()> {
        Ok(())
    }
}
