use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;

/// Clean unused, old project files.
///
/// 1. This runs `git fetch --all --prune` on all projects. (does not support
/// dry run) 2. This removes
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
        let git_projects = find_git_projects(&self.dir)
            .await
            .with_context(|| format!("failed to find git projects from {}", self.dir.display()))?;

        Ok(())
    }
}

async fn find_git_projects(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(vec![])
}

async fn run_git_fetch_all_prune(dir: PathBuf) -> Result<()> {}
