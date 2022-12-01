use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

/// Clean unused, old project files.
///
/// 1. This runs `git fetch --all --prune` on all projects.
/// 2. This removes
///
///  - the unused files in `target` directory.
///  - local git brach which has a remote branch but it's removed.
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
