use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use futures::future::try_join_all;
use tokio::process::Command;

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
    ///
    /// If this is a child of a git repository, this command will run `git fetch
    /// --all --prune` on it and clean only subdirectories.
    dir: PathBuf,
}

impl CleanCommand {
    pub async fn run(&self) -> Result<()> {
        let git_projects = find_git_projects(&self.dir)
            .await
            .with_context(|| format!("failed to find git projects from {}", self.dir.display()))?;

        try_join_all(git_projects.iter().map(|dir| run_git_fetch_all_prune(dir)))
            .await
            .context("failed to run git fetch step")?;

        Ok(())
    }
}

async fn find_git_projects(dir: &Path) -> Result<Vec<PathBuf>> {
    /// Find recursively git projects from the given directory.
    async fn find(dir: &Path) -> Result<Vec<PathBuf>> {
        Ok(vec![dir.to_path_buf()])
    }

    // TODO: Check if `dir` is in a git repository.

    find(dir).await
}

/// - `dir`: The root directory of git repository.
async fn run_git_fetch_all_prune(dir: &Path) -> Result<()> {
    let mut c = Command::new("git");
    c.arg("fetch").arg("--all").arg("--prune");
    c.kill_on_drop(true);

    // TODO: Log status code
    let _status = c.status().await.with_context(|| {
        format!(
            "failed to get status of `git fetch --all --prune` for {}",
            dir.display()
        )
    })?;

    Ok(())
}
